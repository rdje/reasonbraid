//! The WP2 leased outbox worker (`PHASE-0.2.2`): claim work with a lease and a fencing
//! value, deliver, acknowledge — with each phase its own commit point.
//!
//! # Why each phase is its own transaction
//!
//! `ROADMAP.md` §17.3: "Workers claim durable jobs with lease owner, lease expiry, attempt
//! number, and deterministic operation ID. Completion and resulting state are committed
//! transactionally. Lease expiry permits recovery; fencing tokens prevent a stale worker
//! from committing after a newer lease." KICKOFF WP2's kill points (3–5) are exactly the
//! seams *between* these phases, so each phase here commits alone:
//!
//! 1. [`claim_ready`] — atomically lease up to `limit` ready items (a new per-claim fencing
//!    token, an expiry, an incremented attempt). Committed ⇒ the claim is durable.
//! 2. [`deliver`] — write the delivery effect into the deduplicated `outbox_delivery` sink.
//!    Committed ⇒ the effect exists.
//! 3. [`complete`] — acknowledge: mark `dispatched` and clear the lease. Committed ⇒ the
//!    item is terminal and will never be claimed again.
//!
//! A worker that "crashes" between phases leaves the item recoverable: the lease expires and
//! another worker reclaims it. A stale worker's [`complete`] is refused because the fencing
//! token it holds no longer matches (a newer claim replaced it) or its lease has expired.
//!
//! # Deterministic clock
//!
//! Every function takes the caller's `now`; nothing consults the database clock. A test can
//! advance past a lease expiry without sleeping — and the code never depends on application
//! and database clocks agreeing.

use chrono::{DateTime, Duration, Utc};
use sqlx::PgPool;

/// One outbox item a worker holds under lease, as returned by [`claim_ready`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimedOutboxItem {
    pub outbox_id: i64,
    pub tenant_id: String,
    pub event_id: String,
    /// The claim count on this item (1 for the first claim, 2 after one re-claim, …).
    pub attempt: i64,
    /// The fencing handle for this claim; must be presented to [`complete`].
    pub lease_token: String,
}

/// The outcome of [`deliver`] with respect to the deduplicated sink.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeliverOutcome {
    /// The sink row was inserted — this delivery produced the (first) domain effect.
    Delivered,
    /// The event was already delivered — a redelivery produced no second effect.
    Duplicate,
}

/// The outcome of [`complete`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompleteOutcome {
    /// The item was acknowledged: `dispatched = true`, lease cleared.
    Completed,
    /// The item was NOT acknowledged: the presented lease token is not the current one
    /// (a newer claim superseded this worker) or the lease has expired.
    LeaseLost,
}

/// Atomically claim up to `limit` ready outbox items for `worker_id`.
///
/// A ready item is one that is not yet dispatched and either never leased or whose lease has
/// expired at `now`. The claim is one `UPDATE … FROM (SELECT … FOR UPDATE SKIP LOCKED)`
/// statement: concurrent workers never double-claim (each row is locked by exactly one), and
/// every claimed row gets a fresh unguessable fencing token (`gen_random_uuid()`), a new
/// `lease_until`, and an incremented `attempt`.
pub async fn claim_ready(
    pool: &PgPool,
    worker_id: &str,
    now: DateTime<Utc>,
    lease_for: Duration,
    limit: i64,
) -> Result<Vec<ClaimedOutboxItem>, sqlx::Error> {
    let lease_until = now + lease_for;
    // $1 worker_id · $2 lease_until · $3 now (eligibility) · $4 limit
    let rows = sqlx::query_as::<_, (i64, String, String, i64, String)>(
        r#"
        WITH candidate AS (
            SELECT outbox_id
            FROM outbox
            WHERE NOT dispatched
              AND (lease_until IS NULL OR lease_until <= $3)
            ORDER BY outbox_id
            LIMIT $4
            FOR UPDATE SKIP LOCKED
        )
        UPDATE outbox o
        SET lease_owner = $1,
            lease_token = gen_random_uuid()::text,
            lease_until = $2,
            attempt     = o.attempt + 1
        FROM candidate c
        WHERE o.outbox_id = c.outbox_id
        RETURNING o.outbox_id, o.tenant_id, o.event_id, o.attempt, o.lease_token
        "#,
    )
    .bind(worker_id)
    .bind(lease_until)
    .bind(now)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(
            |(outbox_id, tenant_id, event_id, attempt, lease_token)| ClaimedOutboxItem {
                outbox_id,
                tenant_id,
                event_id,
                attempt,
                lease_token,
            },
        )
        .collect())
}

/// Deliver one event into the deduplicated sink (its own commit point).
///
/// `outbox_delivery.event_id` is the primary key, so a redelivery of the same event inserts
/// nothing: [`DeliverOutcome::Duplicate`] is how "transport redelivery produces one domain
/// effect" holds at the worker stage (kill point 4).
pub async fn deliver(
    pool: &PgPool,
    event_id: &str,
    outbox_id: i64,
) -> Result<DeliverOutcome, sqlx::Error> {
    let res = sqlx::query(
        "INSERT INTO outbox_delivery (event_id, outbox_id) VALUES ($1, $2) \
         ON CONFLICT (event_id) DO NOTHING",
    )
    .bind(event_id)
    .bind(outbox_id)
    .execute(pool)
    .await?;

    Ok(if res.rows_affected() == 1 {
        DeliverOutcome::Delivered
    } else {
        DeliverOutcome::Duplicate
    })
}

/// Acknowledge a claimed item (its own commit point).
///
/// The acknowledgement is refused — [`CompleteOutcome::LeaseLost`] — unless the caller still
/// holds the *current* fencing token AND its lease is still live at `now`. A stale worker
/// whose claim was superseded by a newer one fails the token check; an expired lease fails
/// the liveness check; an already-completed item fails both (its lease was cleared). Refusal
/// writes nothing: the item stays claimable, so recovery is always possible.
pub async fn complete(
    pool: &PgPool,
    outbox_id: i64,
    lease_token: &str,
    now: DateTime<Utc>,
) -> Result<CompleteOutcome, sqlx::Error> {
    let res = sqlx::query(
        "UPDATE outbox \
         SET dispatched  = true, \
             lease_owner = NULL, \
             lease_token = NULL, \
             lease_until = NULL \
         WHERE outbox_id = $1 AND lease_token = $2 AND lease_until > $3",
    )
    .bind(outbox_id)
    .bind(lease_token)
    .bind(now)
    .execute(pool)
    .await?;

    Ok(if res.rows_affected() == 1 {
        CompleteOutcome::Completed
    } else {
        CompleteOutcome::LeaseLost
    })
}
