//! The development budget engine, server side (`PHASE-0.5.2`; `ROADMAP.md` §14.3,
//! §14.6): multi-dimensional ceilings, atomic reservations against the CURRENTLY HELD
//! amount, settlement with real usage, release, and recorded denials.
//!
//! # The invariant this enforces
//!
//! **A reservation is only issued when the ceiling covers (held + requested)** —
//! held being every active, unexpired reservation plus everything settled. The check
//! and the insert are ONE transaction (the dev profile's serialization point is the
//! single-writer transaction; a concurrency-hardened version is Phase 2's job, like
//! the `.2.1` aggregate gap-locking note). Denials are rows too — the audit trail
//! covers what was NOT reserved.
//!
//! # Settlement semantics
//!
//! Settling with usage LOWER than the reservation frees the difference implicitly
//! (held = active + settled). Usage HIGHER than the reservation is recorded as an
//! overrun (the §14.3 "estimate error" leg) — never clamped, never dropped.

use chrono::{DateTime, Duration, Utc};
use reasonbraid_core::{BudgetDimensions, BudgetError, ReservationReference};
use sqlx::PgPool;

/// The reservation the engine issued (the node verifies it before dispatching).
#[derive(Debug, Clone, PartialEq)]
pub struct Reservation {
    pub reference: ReservationReference,
    pub ceiling_id: String,
    pub tenant_id: String,
    pub thread_id: String,
    pub expires_at: DateTime<Utc>,
}

/// The outcome of settling a reservation: actual usage vs the hold.
#[derive(Debug, Clone, PartialEq)]
pub struct Settlement {
    /// True when actual usage exceeded the reserved dimensions (recorded, never clamped).
    pub overrun: bool,
    /// The usage dims the reservation did not cover (the overrun amount).
    pub overrun_dimensions: BudgetDimensions,
}

/// Create a ceiling for one thread (the dev profile's scoping unit).
pub async fn create_ceiling(
    pool: &PgPool,
    ceiling_id: &str,
    tenant_id: &str,
    thread_id: &str,
    dimensions: &BudgetDimensions,
) -> Result<(), sqlx::Error> {
    let mut conn = pool.acquire().await?;
    create_ceiling_in_tx(&mut *conn, ceiling_id, tenant_id, thread_id, dimensions).await
}

/// The transactional body of [`create_ceiling`] — shared with the `.6.1` thread-create
/// handler so the ceiling row commits in the SAME transaction as the thread's event
/// (`BUDGET-003`: a thread exists with its ceiling, not without one).
pub(crate) async fn create_ceiling_in_tx<'e, E>(
    mut tx: E,
    ceiling_id: &str,
    tenant_id: &str,
    thread_id: &str,
    dimensions: &BudgetDimensions,
) -> Result<(), sqlx::Error>
where
    E: std::ops::DerefMut,
    for<'c> &'c mut <E as std::ops::Deref>::Target: sqlx::Executor<'c, Database = sqlx::Postgres>,
{
    sqlx::query(
        "INSERT INTO budget_ceilings (ceiling_id, tenant_id, thread_id, dimensions, policy_version) \
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(ceiling_id)
    .bind(tenant_id)
    .bind(thread_id)
    .bind(serde_json::to_value(dimensions).expect("dimensions serialize"))
    .bind("dev-budget-1")
    .execute(&mut *tx)
    .await?;
    Ok(())
}

/// Reserve `requested` against a ceiling, atomically, for `held_for`. The result is a
/// typed [`BudgetError::Unavailable`] when the ceiling does not cover it — and the
/// denial row is committed either way.
pub async fn create_reservation(
    pool: &PgPool,
    ceiling_id: &str,
    tenant_id: &str,
    thread_id: &str,
    requested: &BudgetDimensions,
    held_for: Duration,
    at: DateTime<Utc>,
) -> Result<Reservation, BudgetError> {
    let mut tx = pool.begin().await.map_err(|e| BudgetError::Unavailable {
        detail: e.to_string(),
    })?;
    let result = create_reservation_in_tx(
        &mut *tx, ceiling_id, tenant_id, thread_id, requested, held_for, at,
    )
    .await;
    // The denial row (when the ceiling refused) must COMMIT even though the call
    // returns an error — denials are audit records too.
    tx.commit().await.map_err(|e| BudgetError::Unavailable {
        detail: e.to_string(),
    })?;
    result
}

/// The transactional body of [`create_reservation`] — shared with the `.6.2`
/// invite/challenge dispatch so the reservation (or its denial row) commits in the
/// SAME transaction as the thread event and the inbox enqueue. The caller owns
/// `BEGIN`/`COMMIT`; on `Unavailable` a `denied` row has been written and the
/// caller's transaction remains usable.
pub(crate) async fn create_reservation_in_tx<'e, E>(
    mut tx: E,
    ceiling_id: &str,
    tenant_id: &str,
    thread_id: &str,
    requested: &BudgetDimensions,
    held_for: Duration,
    at: DateTime<Utc>,
) -> Result<Reservation, BudgetError>
where
    E: std::ops::DerefMut,
    for<'c> &'c mut <E as std::ops::Deref>::Target: sqlx::Executor<'c, Database = sqlx::Postgres>,
{
    let ceiling_row: Option<(String, String, serde_json::Value)> = sqlx::query_as(
        "SELECT ceiling_id, tenant_id, dimensions FROM budget_ceilings WHERE ceiling_id = $1",
    )
    .bind(ceiling_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| BudgetError::Unavailable {
        detail: e.to_string(),
    })?;
    let Some((_, _, dimensions)) = ceiling_row else {
        return Err(BudgetError::Unavailable {
            detail: format!("ceiling `{ceiling_id}` does not exist"),
        });
    };
    let ceiling: BudgetDimensions = serde_json::from_value(dimensions).expect("stored dims parse");

    // THE spend circuit breaker (`.3.2`, backlog 23): the per-TENANT latch
    // checks before any ceiling math. While tripped, every NEW reservation
    // is refused with the typed reason; an armed-but-untripped breaker trips
    // (in THIS transaction) the moment the tenant's recorded spend — settled
    // usage + active holds across all its ceilings — crosses the threshold.
    check_spend_breaker_in_tx(&mut *tx, tenant_id, requested, at).await?;

    // held = active (unexpired) reservations + settled usage, summed in Rust over
    // the stored rows (readable, testable — the single-writer dev profile needs no
    // locking aggregate).
    let held_rows: Vec<serde_json::Value> = sqlx::query_scalar(
        "SELECT COALESCE( \
             CASE WHEN r.status = 'active' AND r.expires_at > $2 \
                  THEN r.dimensions ELSE r.usage END, \
             '{}'::jsonb) \
         FROM budget_reservations r \
         WHERE r.ceiling_id = $1 AND r.status IN ('active', 'settled')",
    )
    .bind(ceiling_id)
    .bind(at)
    .fetch_all(&mut *tx)
    .await
    .map_err(|e| BudgetError::Unavailable {
        detail: e.to_string(),
    })?;
    let mut held = BudgetDimensions::default();
    for row in held_rows {
        let dims: BudgetDimensions =
            serde_json::from_value(row).map_err(|e| BudgetError::Unavailable {
                detail: format!("stored held amount is malformed: {e}"),
            })?;
        held = held.add(&dims);
    }

    let remaining = ceiling
        .subtract(&held)
        .map_err(|e| BudgetError::Unavailable {
            detail: format!("the ceiling's ledger is inconsistent: {e}"),
        })?;

    if !remaining.covers(requested) {
        // The denial is itself the audit record (the id is server-generated).
        sqlx::query(
            "INSERT INTO budget_reservations \
             (reservation_id, ceiling_id, tenant_id, thread_id, dimensions, status, reason, created_at) \
             VALUES (gen_random_uuid()::text, $1, $2, $3, $4, 'denied', $5, $6)",
        )
        .bind(ceiling_id)
        .bind(tenant_id)
        .bind(thread_id)
        .bind(serde_json::to_value(requested).expect("dims serialize"))
        .bind(format!(
            "the ceiling does not cover the requested dimensions (held: {held:?})"
        ))
        .bind(at)
        .execute(&mut *tx)
        .await
        .map_err(|e| BudgetError::Unavailable {
            detail: e.to_string(),
        })?;
        return Err(BudgetError::Unavailable {
            detail: "the ceiling does not cover the requested dimensions".to_string(),
        });
    }

    let expires_at = at + held_for;
    let reservation_id: String = sqlx::query_scalar(
        "INSERT INTO budget_reservations \
         (reservation_id, ceiling_id, tenant_id, thread_id, dimensions, status, expires_at, created_at) \
         VALUES (gen_random_uuid()::text, $1, $2, $3, $4, 'active', $5, $6) \
         RETURNING reservation_id",
    )
    .bind(ceiling_id)
    .bind(tenant_id)
    .bind(thread_id)
    .bind(serde_json::to_value(requested).expect("dims serialize"))
    .bind(expires_at)
    .bind(at)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| BudgetError::Unavailable {
        detail: e.to_string(),
    })?;

    Ok(Reservation {
        reference: ReservationReference {
            reservation_id: reservation_id.clone(),
            dimensions: *requested,
            issued_at: at,
        },
        ceiling_id: ceiling_id.to_string(),
        tenant_id: tenant_id.to_string(),
        thread_id: thread_id.to_string(),
        expires_at,
    })
}

/// Settle a reservation with ACTUAL usage: the row records the usage (which may
/// overrun the hold — reported, never clamped) and stops holding anything.
pub async fn settle_reservation(
    pool: &PgPool,
    reservation_id: &str,
    usage: &BudgetDimensions,
    at: DateTime<Utc>,
) -> Result<Option<Settlement>, sqlx::Error> {
    let mut conn = pool.acquire().await?;
    settle_reservation_in_tx(&mut *conn, reservation_id, usage, at).await
}

/// The transactional body of [`settle_reservation`] — shared with the `.6.2`
/// work-result path so a node's reported usage settles the reservation in the SAME
/// transaction as the contribution it produced. Idempotent: a settled/released/
/// denied row is terminal, so a duplicate event or an idempotency replay settles
/// nothing twice.
pub(crate) async fn settle_reservation_in_tx<'e, E>(
    mut tx: E,
    reservation_id: &str,
    usage: &BudgetDimensions,
    at: DateTime<Utc>,
) -> Result<Option<Settlement>, sqlx::Error>
where
    E: std::ops::DerefMut,
    for<'c> &'c mut <E as std::ops::Deref>::Target: sqlx::Executor<'c, Database = sqlx::Postgres>,
{
    let row: Option<(String, serde_json::Value, String)> = sqlx::query_as(
        "SELECT ceiling_id, dimensions, status FROM budget_reservations WHERE reservation_id = $1",
    )
    .bind(reservation_id)
    .fetch_optional(&mut *tx)
    .await?;
    let Some((_, reserved_json, status)) = row else {
        return Ok(None);
    };
    if status != "active" {
        return Ok(None); // settled/released/denied rows are terminal — idempotent no-op
    }
    let reserved: BudgetDimensions =
        serde_json::from_value(reserved_json).expect("stored dims parse");

    // The overrun = usage minus reserved, per dimension, floored at 0.
    fn overrun_of(held: Option<u64>, used: Option<u64>) -> Option<u64> {
        let held = held.unwrap_or(0);
        let used = used.unwrap_or(0);
        (used > held).then(|| used - held)
    }
    let overrun_dimensions = BudgetDimensions {
        calls: overrun_of(reserved.calls, usage.calls),
        input_tokens: overrun_of(reserved.input_tokens, usage.input_tokens),
        output_tokens: overrun_of(reserved.output_tokens, usage.output_tokens),
        wall_clock_seconds: overrun_of(reserved.wall_clock_seconds, usage.wall_clock_seconds),
    };
    let overrun = overrun_dimensions != BudgetDimensions::default();

    sqlx::query(
        "UPDATE budget_reservations SET status = 'settled', usage = $2, settled_at = $3 \
         WHERE reservation_id = $1 AND status = 'active'",
    )
    .bind(reservation_id)
    .bind(serde_json::to_value(usage).expect("dims serialize"))
    .bind(at)
    .execute(&mut *tx)
    .await?;

    Ok(Some(Settlement {
        overrun,
        overrun_dimensions,
    }))
}

/// Release a reservation: it stops holding; the unused remainder returns to the pool.
pub async fn release_reservation(
    pool: &PgPool,
    reservation_id: &str,
    at: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE budget_reservations SET status = 'released', settled_at = $2 \
         WHERE reservation_id = $1 AND status = 'active'",
    )
    .bind(reservation_id)
    .bind(at)
    .execute(pool)
    .await?;
    Ok(())
}

// ── Spend circuit breakers (`.3.2`; backlog 23) ─────────────────────────────

/// The per-tenant spend latch check, IN the caller's reservation transaction.
///
/// A tripped breaker refuses every new reservation with the typed reason; an
/// armed breaker trips the moment the tenant's recorded spend (settled usage
/// plus active holds across ALL its ceilings) plus the REQUESTED reservation
/// crosses the declared threshold. The trip and the refusal commit with the
/// caller's transaction — the latch never lags the ledger it guards.
pub(crate) async fn check_spend_breaker_in_tx<'e, E>(
    mut tx: E,
    tenant_id: &str,
    requested: &BudgetDimensions,
    at: DateTime<Utc>,
) -> Result<(), BudgetError>
where
    E: std::ops::DerefMut,
    for<'c> &'c mut <E as std::ops::Deref>::Target: sqlx::Executor<'c, Database = sqlx::Postgres>,
{
    let breaker: Option<(serde_json::Value, Option<DateTime<Utc>>, Option<String>)> =
        sqlx::query_as(
            "SELECT threshold, tripped_at, tripped_reason FROM spend_breakers \
             WHERE tenant_id = $1",
        )
        .bind(tenant_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| BudgetError::Unavailable {
            detail: e.to_string(),
        })?;
    let Some((threshold, tripped_at, tripped_reason)) = breaker else {
        return Ok(()); // no breaker armed
    };
    if tripped_at.is_some() {
        return Err(BudgetError::Unavailable {
            detail: format!(
                "the tenant's spend circuit breaker is tripped{} — reset it before new work",
                tripped_reason
                    .as_deref()
                    .map(|r| format!(" ({r})"))
                    .unwrap_or_default()
            ),
        });
    }
    let threshold: BudgetDimensions =
        serde_json::from_value(threshold).map_err(|e| BudgetError::Unavailable {
            detail: format!("the stored breaker threshold is malformed: {e}"),
        })?;

    // The tenant's recorded spend: settled usage + active holds, across every
    // ceiling (the breaker is tenant-scoped, unlike the per-thread ceilings).
    let spend_rows: Vec<serde_json::Value> = sqlx::query_scalar(
        "SELECT COALESCE( \
             CASE WHEN r.status = 'active' AND r.expires_at > $2 \
                  THEN r.dimensions ELSE r.usage END, \
             '{}'::jsonb) \
         FROM budget_reservations r \
         WHERE r.tenant_id = $1 AND r.status IN ('active', 'settled')",
    )
    .bind(tenant_id)
    .bind(at)
    .fetch_all(&mut *tx)
    .await
    .map_err(|e| BudgetError::Unavailable {
        detail: e.to_string(),
    })?;
    let mut spend = BudgetDimensions::default();
    for row in spend_rows {
        let dims: BudgetDimensions =
            serde_json::from_value(row).map_err(|e| BudgetError::Unavailable {
                detail: format!("stored spend is malformed: {e}"),
            })?;
        spend = spend.add(&dims);
    }
    let projected = spend.add(requested);

    // A dimension the threshold does not meter is NOT constrained by it (the
    // breaker is an addition to the ceiling, not a second ceiling).
    if threshold.covers(&projected) {
        return Ok(());
    }

    // THE trip: the latch flips in the caller's transaction, with the reason.
    sqlx::query(
        "UPDATE spend_breakers SET tripped_at = $2, tripped_reason = $3 \
         WHERE tenant_id = $1",
    )
    .bind(tenant_id)
    .bind(at)
    .bind(format!(
        "the recorded spend {spend:?} plus the requested {requested:?} crossed the threshold {threshold:?}"
    ))
    .execute(&mut *tx)
    .await
    .map_err(|e| BudgetError::Unavailable {
        detail: e.to_string(),
    })?;
    Err(BudgetError::Unavailable {
        detail: format!(
            "the tenant's spend circuit breaker tripped: {spend:?} + {requested:?} crossed {threshold:?}"
        ),
    })
}
