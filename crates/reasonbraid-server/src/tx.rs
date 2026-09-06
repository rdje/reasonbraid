//! The typed command surface over the aggregate library (`agg`).
//!
//! `PHASE-1.1.1` extracts the WP2 transaction machinery into [`crate::agg`] — the
//! single write path. This module keeps the Phase 0 public shape (`Command`,
//! [`CommandOutcome`], [`ApplyError`], [`apply_command`]) as a thin, typed shim:
//! every function delegates to the library, converting between the two shapes.
//! `tx::Command` carries the wire/typed fields callers assemble; `agg`'s
//! `AggregateCommand` carries the same data borrowed, plus the optional
//! `expected_revision` precondition — which this shim always leaves `None`, so
//! behavior is byte-for-byte the Phase 0 one.
//!
//! Callers that need the new capability (an optimistic-concurrency precondition)
//! use [`crate::agg`] directly.

use serde_json::Value;
use sqlx::PgPool;

use crate::agg::{self, AggregateCommand, AggregateError, AggregateEvent};

/// A command the control plane has already validated and is ready to record durably.
///
/// Identifiers are plain strings here: the server records wire forms, and wiring
/// `reasonbraid-core`'s branded `Id<K>` types into the command handlers is a later leaf
/// (WP5 identity / WP6 vertical slice). This type proves the *transaction*, not the
/// domain-type richness.
#[derive(Debug, Clone)]
pub struct Command {
    pub tenant_id: String,
    pub aggregate_type: String,
    pub aggregate_id: String,
    pub idempotency_key: String,
    pub request_hash: String,
    pub event_id: String,
    pub event_type: String,
    pub body: Value,
    pub next_state: Value,
    /// The semantic result returned to the client and persisted for idempotent replay.
    pub result: Value,
}

/// The outcome of a durable apply. `result` is authoritative on both the fresh and the
/// replayed path: on replay it is the *original* stored result, never a recomputed one.
#[derive(Debug, Clone, PartialEq)]
pub struct CommandOutcome {
    /// `true` when the command was recognized as an idempotent replay of a prior one.
    pub replayed: bool,
    pub result: Value,
}

/// An application-level failure. `Sql` wraps driver/constraint errors; the other arm is a
/// deliberate, typed outcome of the transaction's own logic.
#[derive(Debug)]
pub enum ApplyError {
    /// The idempotency key was already used with a *different* request hash.
    IdempotencyConflict {
        key: String,
        request_hash: String,
        stored_hash: String,
    },
    Sql(sqlx::Error),
}

impl std::fmt::Display for ApplyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApplyError::IdempotencyConflict {
                key,
                request_hash,
                stored_hash,
            } => write!(
                f,
                "idempotency conflict: key `{key}` was already used with hash `{stored_hash}`, not `{request_hash}`"
            ),
            ApplyError::Sql(e) => write!(f, "database error: {e}"),
        }
    }
}

impl std::error::Error for ApplyError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ApplyError::Sql(e) => Some(e),
            _ => None,
        }
    }
}

impl From<sqlx::Error> for ApplyError {
    fn from(e: sqlx::Error) -> Self {
        ApplyError::Sql(e)
    }
}

impl From<AggregateError> for ApplyError {
    fn from(e: AggregateError) -> Self {
        match e {
            AggregateError::IdempotencyConflict {
                key,
                request_hash,
                stored_hash,
            } => ApplyError::IdempotencyConflict {
                key,
                request_hash,
                stored_hash,
            },
            // Documented internal invariant: this shim always passes
            // `expected_revision = None`, so the library can never produce a
            // revision conflict on this path.
            AggregateError::RevisionConflict { .. } => {
                unreachable!("the tx shim never sets expected_revision")
            }
            AggregateError::Sql(e) => ApplyError::Sql(e),
        }
    }
}

/// The outcome of the idempotency claim (mirrors [`agg::ClaimOutcome`]).
pub(crate) enum ClaimOutcome {
    Fresh,
    Replay { result: Value },
}

/// Apply a command in one transaction: idempotency claim, ordered event, current state,
/// and outbox item commit together, or none of them is ever visible.
///
/// # Atomicity
///
/// Every write runs on one `BEGIN … COMMIT`. A failure at any step returns [`ApplyError`]
/// and drops the transaction, which rolls back the idempotency claim too — no partial
/// effect is visible. The outbox row carries a foreign key to the event, so an outbox item
/// *implies* its event is durable.
///
/// # Idempotency
///
/// The `(tenant_id, idempotency_key)` primary key is claimed first with
/// `INSERT … ON CONFLICT DO NOTHING`. A second submission of the same key blocks on the
/// unique index until the first commits, then takes the replay/conflict branch.
pub async fn apply_command(pool: &PgPool, cmd: &Command) -> Result<CommandOutcome, ApplyError> {
    let mut tx = pool.begin().await?;
    let outcome = apply_command_in_tx(&mut *tx, cmd).await?;
    tx.commit().await?;
    Ok(outcome)
}

/// The transactional body of [`apply_command`], shared with the authorized path
/// (`PHASE-0.5.1`): the authorization decision record and the four durability writes
/// commit in ONE transaction. Generic over the executor so a caller can pass either a
/// transaction (this crate) or the executor of one; the caller owns `BEGIN`/`COMMIT`.
pub(crate) async fn apply_command_in_tx<'e, E>(
    mut tx: E,
    cmd: &Command,
) -> Result<CommandOutcome, ApplyError>
where
    E: std::ops::DerefMut,
    for<'c> &'c mut <E as std::ops::Deref>::Target: sqlx::Executor<'c, Database = sqlx::Postgres>,
{
    let outcome = agg::apply_in_tx(&mut *tx, &as_aggregate_command(cmd)).await?;
    Ok(CommandOutcome {
        replayed: outcome.replayed,
        result: outcome.result,
    })
}

/// Claim the `(tenant_id, idempotency_key)` slot for this transaction.
///
/// The primary key is the serialization point that makes redelivery idempotent:
/// exactly one transaction can own the key. The `.6.1` thread handlers call this
/// BEFORE their domain validation so a replay returns the original result without
/// re-validating against state the original command may have since changed.
pub(crate) async fn claim_idempotency_in_tx<'e, E>(
    mut tx: E,
    tenant_id: &str,
    idempotency_key: &str,
    request_hash: &str,
) -> Result<ClaimOutcome, ApplyError>
where
    E: std::ops::DerefMut,
    for<'c> &'c mut <E as std::ops::Deref>::Target: sqlx::Executor<'c, Database = sqlx::Postgres>,
{
    match agg::claim_in_tx(&mut *tx, tenant_id, idempotency_key, request_hash).await? {
        agg::ClaimOutcome::Fresh => Ok(ClaimOutcome::Fresh),
        agg::ClaimOutcome::Replay { result } => Ok(ClaimOutcome::Replay { result }),
    }
}

/// The fresh-command writes (steps 2–6 of the aggregate library's write path): lock
/// the aggregate row, derive the next ordered version, write event + current state +
/// outbox item, and record the semantic result for idempotent replay. The caller must
/// have claimed the idempotency slot first ([`claim_idempotency_in_tx`]).
pub(crate) async fn apply_fresh_in_tx<'e, E>(
    mut tx: E,
    cmd: &Command,
) -> Result<CommandOutcome, ApplyError>
where
    E: std::ops::DerefMut,
    for<'c> &'c mut <E as std::ops::Deref>::Target: sqlx::Executor<'c, Database = sqlx::Postgres>,
{
    let outcome = agg::apply_fresh_in_tx(&mut *tx, &as_aggregate_command(cmd)).await?;
    Ok(CommandOutcome {
        replayed: outcome.replayed,
        result: outcome.result,
    })
}

/// The borrowed, library-shaped view of a [`Command`]. The shim's documented
/// invariant: `expected_revision` is always `None` here — optimistic-concurrency
/// preconditions are the caller's explicit choice via [`crate::agg`] directly.
fn as_aggregate_command(cmd: &Command) -> AggregateCommand<'_> {
    AggregateCommand {
        tenant_id: &cmd.tenant_id,
        aggregate_type: &cmd.aggregate_type,
        aggregate_id: &cmd.aggregate_id,
        idempotency_key: &cmd.idempotency_key,
        request_hash: &cmd.request_hash,
        expected_revision: None,
        event: AggregateEvent {
            event_id: cmd.event_id.clone(),
            event_type: cmd.event_type.clone(),
            body: cmd.body.clone(),
        },
        next_state: cmd.next_state.clone(),
        result: cmd.result.clone(),
    }
}
