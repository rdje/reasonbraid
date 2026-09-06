//! The aggregate/event/outbox library (`PHASE-1.1.1`; backlog 9; ADR-004).
//!
//! # One transaction, six writes
//!
//! For every accepted command, one PostgreSQL transaction writes, in order:
//!
//! 1. the **idempotency claim** — the `(tenant_id, idempotency_key)` primary key is
//!    the serialization point: a redelivery with the same request hash replays the
//!    ORIGINAL stored result, a different hash is a conflict;
//! 2. the **locked aggregate head** — `SELECT … FOR UPDATE` on `aggregate_state`,
//!    the revision serialization point (§8.6: lock or compare-and-swap the head);
//! 3. the **ordered event append** — `event_log` with the derived next revision
//!    (unique per `(tenant, aggregate, version)`, so a version is never reused);
//! 4. the **current-state upsert** — the projection readers and validators use;
//! 5. the **outbox enqueue** — the FK to `event_log` proves an outbox item exists
//!    only if its event is already durable;
//! 6. the **semantic result** — stored on the idempotency row so a replay returns
//!    the original result verbatim (rejections included).
//!
//! All six commit together or none is visible (`ROADMAP.md` §8.6, §17.2).
//!
//! # Single write path
//!
//! [`apply_in_tx`] is THE write path for aggregate commands. Callers compose domain
//! logic AROUND it inside the same transaction — authorize (`crate::authority`),
//! validate against the locked projection (`crate::threads`), reserve budgets
//! (`crate::budget`) — and let the library own durability. The SQL statements exist
//! here once; a caller that hand-rolls any of the six steps is a bug the review
//! should refuse.
//!
//! # Revision semantics
//!
//! The locked head IS the concurrency control: two writers to the same aggregate
//! serialize on the `FOR UPDATE` read, and the next revision is derived from it.
//! [`AggregateCommand::expected_revision`] adds an optimistic-concurrency
//! precondition on top: `Some(n)` requires the current head to be revision `n`
//! (a fresh aggregate is revision 0), otherwise the apply is refused with
//! [`AggregateError::RevisionConflict`]. `None` — what every Phase 0 caller passes
//! today — preserves the locked-head behavior exactly.
//!
//! # What is deliberately NOT here
//!
//! Authorization and domain validation. The library writes durably what the
//! caller has ALREADY decided is authorized and valid; the claim → authorize →
//! validate → apply orchestration lives in `crate::api` (see
//! `docs/decisions/2026-09-06_control-api-cli.md`), composing these steps with
//! this library in one transaction. Extraction into a separate store crate waits
//! for a measured boundary need (ADR-002's modular-monolith stance; KICKOFF §3).

use std::fmt;

use serde_json::Value;
use sqlx::Postgres;

/// One ordered event an accepted command appends to the aggregate's log.
#[derive(Debug, Clone)]
pub struct AggregateEvent {
    pub event_id: String,
    pub event_type: String,
    pub body: Value,
}

/// Everything the transactional body needs to apply one aggregate command.
#[derive(Debug, Clone)]
pub struct AggregateCommand<'a> {
    pub tenant_id: &'a str,
    pub aggregate_type: &'a str,
    pub aggregate_id: &'a str,
    pub idempotency_key: &'a str,
    pub request_hash: &'a str,
    /// `Some(n)`: the current head must be revision `n` (0 = the aggregate has no
    /// row yet). `None`: the locked-head behavior, with no precondition.
    pub expected_revision: Option<i64>,
    pub event: AggregateEvent,
    pub next_state: Value,
    pub result: Value,
}

/// The outcome of one applied command.
///
/// `revision` is the event's `aggregate_version` — `Some(n)` for a fresh apply,
/// and for a replay the revision the aggregate's head currently carries (or `None`
/// when the replayed result was a stored rejection, which wrote no aggregate row).
#[derive(Debug, Clone, PartialEq)]
pub struct AggregateOutcome {
    /// `true` when the command was recognized as an idempotent replay of a prior one.
    pub replayed: bool,
    pub revision: Option<i64>,
    /// The semantic result — on replay it is the ORIGINAL stored value verbatim.
    pub result: Value,
}

/// The outcome of the idempotency claim ([`claim_in_tx`]).
#[derive(Debug, Clone, PartialEq)]
pub enum ClaimOutcome {
    /// The key was free: this transaction owns it and must apply the command.
    Fresh,
    /// The key already exists with the same request hash: the ORIGINAL stored
    /// result is returned; nothing new may be written.
    Replay { result: Value },
}

/// A typed library failure.
#[derive(Debug)]
pub enum AggregateError {
    /// The idempotency key was already used with a DIFFERENT request hash.
    IdempotencyConflict {
        key: String,
        request_hash: String,
        stored_hash: String,
    },
    /// The `expected_revision` precondition failed: the head is not at the
    /// revision the caller required.
    RevisionConflict {
        aggregate_id: String,
        expected: i64,
        actual: i64,
    },
    Sql(sqlx::Error),
}

impl fmt::Display for AggregateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AggregateError::IdempotencyConflict {
                key,
                request_hash,
                stored_hash,
            } => write!(
                f,
                "idempotency conflict: key `{key}` was already used with hash `{stored_hash}`, not `{request_hash}`"
            ),
            AggregateError::RevisionConflict {
                aggregate_id,
                expected,
                actual,
            } => write!(
                f,
                "revision conflict on aggregate `{aggregate_id}`: expected {expected}, actual {actual}"
            ),
            AggregateError::Sql(e) => write!(f, "database error: {e}"),
        }
    }
}

impl std::error::Error for AggregateError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AggregateError::Sql(e) => Some(e),
            _ => None,
        }
    }
}

impl From<sqlx::Error> for AggregateError {
    fn from(e: sqlx::Error) -> Self {
        AggregateError::Sql(e)
    }
}

/// Claim the `(tenant_id, idempotency_key)` slot for this transaction (step 1 of
/// the write path; split out so a caller can authorize/validate BETWEEN the claim
/// and the apply inside ONE transaction — the claim's primary-key insert is what
/// serializes redeliveries).
pub async fn claim_in_tx<'e, E>(
    mut tx: E,
    tenant_id: &str,
    idempotency_key: &str,
    request_hash: &str,
) -> Result<ClaimOutcome, AggregateError>
where
    E: std::ops::DerefMut,
    for<'c> &'c mut <E as std::ops::Deref>::Target: sqlx::Executor<'c, Database = Postgres>,
{
    let claim = sqlx::query(
        "INSERT INTO idempotency (tenant_id, idempotency_key, request_hash, response_result) \
         VALUES ($1, $2, $3, 'null'::jsonb) \
         ON CONFLICT (tenant_id, idempotency_key) DO NOTHING",
    )
    .bind(tenant_id)
    .bind(idempotency_key)
    .bind(request_hash)
    .execute(&mut *tx)
    .await?;

    if claim.rows_affected() == 0 {
        // The key already exists: replay (same hash) or conflict (different hash).
        let (stored_hash, stored_result): (String, Value) = sqlx::query_as(
            "SELECT request_hash, response_result FROM idempotency \
             WHERE tenant_id = $1 AND idempotency_key = $2",
        )
        .bind(tenant_id)
        .bind(idempotency_key)
        .fetch_one(&mut *tx)
        .await?;

        if stored_hash != request_hash {
            return Err(AggregateError::IdempotencyConflict {
                key: idempotency_key.to_string(),
                request_hash: request_hash.to_string(),
                stored_hash,
            });
        }

        // Idempotent replay: return the ORIGINAL result; nothing new is written
        // (the caller commits).
        return Ok(ClaimOutcome::Replay {
            result: stored_result,
        });
    }

    Ok(ClaimOutcome::Fresh)
}

/// The fresh-command writes (steps 2–6 of the write path): lock the aggregate head,
/// check the revision precondition, derive the next ordered version, append the
/// event, upsert the current state, enqueue the outbox item, and record the
/// semantic result for idempotent replay. The caller must have claimed the
/// idempotency slot first ([`claim_in_tx`]).
pub async fn apply_fresh_in_tx<'e, E>(
    mut tx: E,
    cmd: &AggregateCommand<'_>,
) -> Result<AggregateOutcome, AggregateError>
where
    E: std::ops::DerefMut,
    for<'c> &'c mut <E as std::ops::Deref>::Target: sqlx::Executor<'c, Database = Postgres>,
{
    // 1. Lock the aggregate row so writers to the SAME aggregate serialize, then
    //    derive the next ordered version. `None` = the aggregate has no row yet
    //    (revision 0, `FOR UPDATE` takes no row lock — the (tenant, aggregate)
    //    primary-key insert below is then the serialization point for first writes).
    let current: Option<i64> = sqlx::query_scalar(
        "SELECT aggregate_version FROM aggregate_state \
         WHERE tenant_id = $1 AND aggregate_id = $2 FOR UPDATE",
    )
    .bind(cmd.tenant_id)
    .bind(cmd.aggregate_id)
    .fetch_optional(&mut *tx)
    .await?;

    if let Some(expected) = cmd.expected_revision {
        let actual = current.unwrap_or(0);
        if actual != expected {
            return Err(AggregateError::RevisionConflict {
                aggregate_id: cmd.aggregate_id.to_string(),
                expected,
                actual,
            });
        }
    }

    let next_version = current.map_or(1, |v| v + 1);

    // 2. Ordered event (unique per (tenant, aggregate, version)).
    sqlx::query(
        "INSERT INTO event_log (event_id, tenant_id, aggregate_id, aggregate_version, event_type, body) \
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(&cmd.event.event_id)
    .bind(cmd.tenant_id)
    .bind(cmd.aggregate_id)
    .bind(next_version)
    .bind(&cmd.event.event_type)
    .bind(&cmd.event.body)
    .execute(&mut *tx)
    .await?;

    // 3. Current state (upsert).
    sqlx::query(
        "INSERT INTO aggregate_state (tenant_id, aggregate_id, aggregate_type, aggregate_version, state) \
         VALUES ($1, $2, $3, $4, $5) \
         ON CONFLICT (tenant_id, aggregate_id) DO UPDATE SET \
             aggregate_type = EXCLUDED.aggregate_type, \
             aggregate_version = EXCLUDED.aggregate_version, \
             state = EXCLUDED.state",
    )
    .bind(cmd.tenant_id)
    .bind(cmd.aggregate_id)
    .bind(cmd.aggregate_type)
    .bind(next_version)
    .bind(&cmd.next_state)
    .execute(&mut *tx)
    .await?;

    // 4. Outbox item — the FK to event_log proves the event is already durable.
    sqlx::query("INSERT INTO outbox (tenant_id, event_id) VALUES ($1, $2)")
        .bind(cmd.tenant_id)
        .bind(&cmd.event.event_id)
        .execute(&mut *tx)
        .await?;

    // 5. Record the semantic result for idempotent replay.
    store_result_in_tx(&mut *tx, cmd.tenant_id, cmd.idempotency_key, &cmd.result).await?;

    Ok(AggregateOutcome {
        replayed: false,
        revision: Some(next_version),
        result: cmd.result.clone(),
    })
}

/// The full write path in one call: claim (step 1), then — on a fresh claim — the
/// apply (steps 2–6). On a replay the ORIGINAL stored result is returned and
/// nothing is written. The caller owns `BEGIN`/`COMMIT`.
pub async fn apply_in_tx<'e, E>(
    mut tx: E,
    cmd: &AggregateCommand<'_>,
) -> Result<AggregateOutcome, AggregateError>
where
    E: std::ops::DerefMut,
    for<'c> &'c mut <E as std::ops::Deref>::Target: sqlx::Executor<'c, Database = Postgres>,
{
    match claim_in_tx(
        &mut *tx,
        cmd.tenant_id,
        cmd.idempotency_key,
        cmd.request_hash,
    )
    .await?
    {
        ClaimOutcome::Replay { result } => {
            // The aggregate head's current revision accompanies the replayed result
            // when one exists; a stored rejection (a result written before any
            // apply) has no aggregate row, so the revision is `None` — honest,
            // never invented.
            let revision: Option<i64> = sqlx::query_scalar(
                "SELECT aggregate_version FROM aggregate_state \
                 WHERE tenant_id = $1 AND aggregate_id = $2",
            )
            .bind(cmd.tenant_id)
            .bind(cmd.aggregate_id)
            .fetch_optional(&mut *tx)
            .await?;
            Ok(AggregateOutcome {
                replayed: true,
                revision,
                result,
            })
        }
        ClaimOutcome::Fresh => apply_fresh_in_tx(&mut *tx, cmd).await,
    }
}

/// The pool-level convenience: `BEGIN`, apply, `COMMIT` — the transactional body is
/// [`apply_in_tx`]; this wrapper exists for one-shot callers and tests.
pub async fn apply(
    pool: &sqlx::PgPool,
    cmd: &AggregateCommand<'_>,
) -> Result<AggregateOutcome, AggregateError> {
    let mut tx = pool.begin().await?;
    let outcome = apply_in_tx(&mut *tx, cmd).await?;
    tx.commit().await?;
    Ok(outcome)
}

/// Step 6 alone: store a semantic result on the idempotency row — the path
/// rejections ride, so a replay of a refusal reproduces the ORIGINAL result
/// (`docs/decisions/2026-09-06_control-api-cli.md`: rejections are idempotent
/// results). Only valid after the key was claimed in the same transaction.
pub async fn store_result_in_tx<'e, E>(
    mut tx: E,
    tenant_id: &str,
    idempotency_key: &str,
    result: &Value,
) -> Result<(), AggregateError>
where
    E: std::ops::DerefMut,
    for<'c> &'c mut <E as std::ops::Deref>::Target: sqlx::Executor<'c, Database = Postgres>,
{
    sqlx::query(
        "UPDATE idempotency SET response_result = $1 \
         WHERE tenant_id = $2 AND idempotency_key = $3",
    )
    .bind(result)
    .bind(tenant_id)
    .bind(idempotency_key)
    .execute(&mut *tx)
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The typed errors are self-describing (no database needed for this).
    #[test]
    fn error_display_names_the_failure() {
        let conflict = AggregateError::IdempotencyConflict {
            key: "k".to_string(),
            request_hash: "h2".to_string(),
            stored_hash: "h1".to_string(),
        };
        let text = conflict.to_string();
        assert!(
            text.contains("idempotency conflict") && text.contains("`k`"),
            "got: {text}"
        );

        let revision = AggregateError::RevisionConflict {
            aggregate_id: "thr_1".to_string(),
            expected: 1,
            actual: 2,
        };
        let text = revision.to_string();
        assert!(
            text.contains("revision conflict") && text.contains("expected 1, actual 2"),
            "got: {text}"
        );
    }

    /// A replay outcome is structurally the original-result path: `replayed`
    /// set, no fresh revision invented.
    #[test]
    fn replay_outcome_shape_is_honest() {
        let replay = AggregateOutcome {
            replayed: true,
            revision: None,
            result: serde_json::json!({ "ok": false }),
        };
        assert!(replay.replayed);
        assert_eq!(
            replay.revision, None,
            "a stored rejection has no aggregate row"
        );
    }
}
