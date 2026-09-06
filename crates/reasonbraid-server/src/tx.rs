//! The WP2 atomic transaction: current state, ordered event, idempotency result, and
//! outbox item are written in one PostgreSQL transaction — or none is.

use serde_json::Value;
use sqlx::PgPool;

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

    // 1. Claim the idempotency slot. The PK is what makes a redelivery idempotent: exactly
    //    one transaction can own the key.
    let claim = sqlx::query(
        "INSERT INTO idempotency (tenant_id, idempotency_key, request_hash, response_result) \
         VALUES ($1, $2, $3, 'null'::jsonb) \
         ON CONFLICT (tenant_id, idempotency_key) DO NOTHING",
    )
    .bind(&cmd.tenant_id)
    .bind(&cmd.idempotency_key)
    .bind(&cmd.request_hash)
    .execute(&mut *tx)
    .await?;

    if claim.rows_affected() == 0 {
        // The key already exists: replay (same hash) or conflict (different hash).
        let (stored_hash, stored_result): (String, Value) = sqlx::query_as(
            "SELECT request_hash, response_result FROM idempotency \
             WHERE tenant_id = $1 AND idempotency_key = $2",
        )
        .bind(&cmd.tenant_id)
        .bind(&cmd.idempotency_key)
        .fetch_one(&mut *tx)
        .await?;

        if stored_hash != cmd.request_hash {
            return Err(ApplyError::IdempotencyConflict {
                key: cmd.idempotency_key.clone(),
                request_hash: cmd.request_hash.clone(),
                stored_hash,
            });
        }

        // Idempotent replay: return the ORIGINAL result; nothing new is written.
        tx.commit().await?;
        return Ok(CommandOutcome {
            replayed: true,
            result: stored_result,
        });
    }

    // 2. Fresh command. Lock the aggregate row so writers to the SAME aggregate serialize,
    //    then derive the next ordered version.
    let current: Option<i64> = sqlx::query_scalar(
        "SELECT aggregate_version FROM aggregate_state \
         WHERE tenant_id = $1 AND aggregate_id = $2 FOR UPDATE",
    )
    .bind(&cmd.tenant_id)
    .bind(&cmd.aggregate_id)
    .fetch_optional(&mut *tx)
    .await?;

    let next_version = current.map_or(1, |v| v + 1);

    // 3. Ordered event (unique per (tenant, aggregate, version)).
    sqlx::query(
        "INSERT INTO event_log (event_id, tenant_id, aggregate_id, aggregate_version, event_type, body) \
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(&cmd.event_id)
    .bind(&cmd.tenant_id)
    .bind(&cmd.aggregate_id)
    .bind(next_version)
    .bind(&cmd.event_type)
    .bind(&cmd.body)
    .execute(&mut *tx)
    .await?;

    // 4. Current state (upsert).
    sqlx::query(
        "INSERT INTO aggregate_state (tenant_id, aggregate_id, aggregate_type, aggregate_version, state) \
         VALUES ($1, $2, $3, $4, $5) \
         ON CONFLICT (tenant_id, aggregate_id) DO UPDATE SET \
             aggregate_type = EXCLUDED.aggregate_type, \
             aggregate_version = EXCLUDED.aggregate_version, \
             state = EXCLUDED.state",
    )
    .bind(&cmd.tenant_id)
    .bind(&cmd.aggregate_id)
    .bind(&cmd.aggregate_type)
    .bind(next_version)
    .bind(&cmd.next_state)
    .execute(&mut *tx)
    .await?;

    // 5. Outbox item — the FK to event_log proves the event is already durable.
    sqlx::query("INSERT INTO outbox (tenant_id, event_id) VALUES ($1, $2)")
        .bind(&cmd.tenant_id)
        .bind(&cmd.event_id)
        .execute(&mut *tx)
        .await?;

    // 6. Record the semantic result for idempotent replay.
    sqlx::query(
        "UPDATE idempotency SET response_result = $1 \
         WHERE tenant_id = $2 AND idempotency_key = $3",
    )
    .bind(&cmd.result)
    .bind(&cmd.tenant_id)
    .bind(&cmd.idempotency_key)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(CommandOutcome {
        replayed: false,
        result: cmd.result.clone(),
    })
}
