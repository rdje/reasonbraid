//! Integration tests for the WP2 leased outbox worker (`PHASE-0.2.2`).
//!
//! Like the `.2.1` atomic-transaction tests, these need a live PostgreSQL. Run them with
//! `scripts/run_pg_tests.sh` locally, or the `pg-tests` CI job. Without `DATABASE_URL` they
//! skip, so `make check` / `cargo test --all` stay green offline.
//!
//! The tests exercise the acceptance: **stale leased workers cannot commit after a newer
//! fencing value**, plus the KICKOFF WP2 kill points 3–5 (after claim / after delivery /
//! after acknowledgement). Every "crash" is simulated by stopping at a phase boundary — each
//! phase commits alone, so what remains in the database is exactly what survived the crash.
//! The lease clock is caller-supplied, so expiry is advanced deterministically with no sleeps.

use chrono::{Duration, Utc};
use reasonbraid_server::{
    apply_command, claim_ready, complete, deliver, Command, CompleteOutcome, DeliverOutcome,
};
use sqlx::PgPool;
use std::sync::OnceLock;

/// The outbox is ONE shared queue, so these tests must not run concurrently and must own the
/// queue while they run: a parallel test — or leftover rows from the `atomic_transaction`
/// binary, which shares the same database and seeds undispatched items — would be claimed by
/// a worker test that expects only its own item (the claim query scans the whole queue,
/// oldest item first). A module-level async mutex serializes the tests deterministically,
/// whatever `--test-threads` the harness passes, and [`pool`] purges the queue first.
static QUEUE_LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();

async fn queue_guard() -> tokio::sync::MutexGuard<'static, ()> {
    QUEUE_LOCK
        .get_or_init(|| tokio::sync::Mutex::new(()))
        .lock()
        .await
}

/// A fixed lease length for every claim in these tests.
const LEASE: Duration = Duration::seconds(60);

async fn pool() -> Option<PgPool> {
    let url = match std::env::var("DATABASE_URL") {
        Ok(u) => u,
        Err(_) => {
            eprintln!(
                "SKIP: DATABASE_URL is unset — run scripts/run_pg_tests.sh for the real PostgreSQL proof"
            );
            return None;
        }
    };
    let pool = PgPool::connect(&url)
        .await
        .expect("connect to DATABASE_URL");
    sqlx::migrate!("../../migrations")
        .run(&pool)
        .await
        .expect("apply migrations");
    // The queue is exclusively owned by these tests for their duration: drop every existing
    // delivery and outbox row (the delivery sink first — it references outbox) so each test
    // sees exactly the one item it seeds.
    sqlx::query("DELETE FROM outbox_delivery")
        .execute(&pool)
        .await
        .expect("purge delivery rows");
    sqlx::query("DELETE FROM outbox")
        .execute(&pool)
        .await
        .expect("purge outbox rows");
    Some(pool)
}

/// Seed one committed thread-create command so the outbox holds exactly one ready item.
/// Returns the event_id and outbox_id of that item.
async fn seed_one_item(pool: &PgPool, tag: &str) -> (String, i64) {
    let c = Command {
        tenant_id: format!("ten_ow_{tag}"),
        aggregate_type: "thread".to_string(),
        aggregate_id: format!("thr_ow_{tag}"),
        idempotency_key: format!("key_ow_{tag}"),
        request_hash: "h".to_string(),
        event_id: format!("evt_ow_{tag}"),
        event_type: "thread.created".to_string(),
        body: serde_json::json!({ "tag": tag }),
        next_state: serde_json::json!({ "status": "open", "tag": tag }),
        result: serde_json::json!({ "tag": tag }),
    };
    let outcome = apply_command(pool, &c).await.expect("seed applies");
    assert!(!outcome.replayed);

    let (outbox_id,): (i64,) =
        sqlx::query_as("SELECT outbox_id FROM outbox WHERE tenant_id = $1 AND event_id = $2")
            .bind(&c.tenant_id)
            .bind(&c.event_id)
            .fetch_one(pool)
            .await
            .expect("read outbox id");
    (c.event_id, outbox_id)
}

/// Remove a test's own item (delivery sink row, then outbox row) so it can never be claimed
/// by a LATER test: the claim query scans the whole shared queue oldest-item-first, so any
/// leftover leased-but-incomplete row would leak into the next test once its lease lapses.
async fn cleanup_item(pool: &PgPool, outbox_id: i64) {
    sqlx::query("DELETE FROM outbox_delivery WHERE outbox_id = $1")
        .bind(outbox_id)
        .execute(pool)
        .await
        .expect("cleanup delivery row");
    sqlx::query("DELETE FROM outbox WHERE outbox_id = $1")
        .bind(outbox_id)
        .execute(pool)
        .await
        .expect("cleanup outbox row");
}

async fn delivery_count(pool: &PgPool, event_id: &str) -> i64 {
    let (n,): (i64,) = sqlx::query_as("SELECT count(*) FROM outbox_delivery WHERE event_id = $1")
        .bind(event_id)
        .fetch_one(pool)
        .await
        .expect("count deliveries");
    n
}

async fn outbox_state(
    pool: &PgPool,
    outbox_id: i64,
) -> (bool, Option<String>, Option<String>, i64) {
    sqlx::query_as(
        "SELECT dispatched, lease_owner, lease_token, attempt FROM outbox WHERE outbox_id = $1",
    )
    .bind(outbox_id)
    .fetch_one(pool)
    .await
    .expect("read outbox row")
}

/// The claim is exclusive: two workers racing for one ready item — only one wins it.
#[tokio::test]
async fn claim_is_exclusive_between_concurrent_workers() {
    let _guard = queue_guard().await;
    let Some(pool) = pool().await else { return };
    let (_, outbox_id) = seed_one_item(&pool, "excl").await;
    let now = Utc::now();

    let a = claim_ready(&pool, "worker-a", now, LEASE, 1);
    let b = claim_ready(&pool, "worker-b", now, LEASE, 1);
    let (a, b) = tokio::join!(a, b);
    let (a, b) = (a.expect("claim a"), b.expect("claim b"));

    let winner = match (a.len(), b.len()) {
        (1, 0) => ("worker-a", &a[0]),
        (0, 1) => ("worker-b", &b[0]),
        other => panic!("expected exactly one winner, got {other:?}"),
    };
    assert_eq!(winner.1.outbox_id, outbox_id);
    assert_eq!(winner.1.attempt, 1, "first claim is attempt 1");
    assert!(
        !winner.1.lease_token.is_empty(),
        "a fencing token was issued"
    );

    let (dispatched, owner, _, attempt) = outbox_state(&pool, outbox_id).await;
    assert!(!dispatched);
    assert_eq!(
        owner.as_deref(),
        Some(winner.0),
        "the lease belongs to the winning worker"
    );
    assert_eq!(attempt, 1);
    cleanup_item(&pool, outbox_id).await;
}

/// A claim is only possible when the item is unclaimed or its lease has expired; a re-claim
/// after expiry issues a NEW fencing token and increments the attempt.
#[tokio::test]
async fn reclaim_after_lease_expiry_issues_new_token_and_increments_attempt() {
    let _guard = queue_guard().await;
    let Some(pool) = pool().await else { return };
    let (_, outbox_id) = seed_one_item(&pool, "reclaim").await;
    let now = Utc::now();

    let first = claim_ready(&pool, "worker-a", now, LEASE, 1)
        .await
        .expect("first claim");
    assert_eq!(first.len(), 1);
    let a = &first[0];

    // While worker-a's lease is live, worker-b cannot claim the item.
    let during = claim_ready(&pool, "worker-b", now + Duration::seconds(30), LEASE, 1)
        .await
        .expect("claim during lease");
    assert!(during.is_empty(), "a live lease must block re-claim");

    // After the lease expires, worker-b can claim; it gets a NEW token and attempt 2.
    let after = now + Duration::seconds(61);
    let second = claim_ready(&pool, "worker-b", after, LEASE, 1)
        .await
        .expect("claim after expiry");
    assert_eq!(second.len(), 1);
    let b = &second[0];
    assert_eq!(b.outbox_id, outbox_id);
    assert_eq!(b.attempt, 2);
    assert_ne!(
        b.lease_token, a.lease_token,
        "a new claim has a new fencing token"
    );

    let (dispatched, owner, _, attempt) = outbox_state(&pool, outbox_id).await;
    assert!(!dispatched);
    assert_eq!(owner.as_deref(), Some("worker-b"));
    assert_eq!(attempt, 2);
    cleanup_item(&pool, outbox_id).await;
}

/// THE fencing acceptance: worker-a's lease expires, worker-b claims with a newer fencing
/// value, and worker-a can then NEVER complete — its token is stale. Worker-b completes.
/// A second complete by worker-b is also refused (the item is terminal).
#[tokio::test]
async fn stale_worker_cannot_complete_after_a_newer_fencing_value() {
    let _guard = queue_guard().await;
    let Some(pool) = pool().await else { return };
    let (_, outbox_id) = seed_one_item(&pool, "fence").await;
    let now = Utc::now();

    let first = claim_ready(&pool, "worker-a", now, LEASE, 1)
        .await
        .expect("worker-a claim");
    let a = &first[0];

    // Lease expires; worker-b claims (a newer fencing value supersedes worker-a's).
    let later = now + Duration::seconds(61);
    let second = claim_ready(&pool, "worker-b", later, LEASE, 1)
        .await
        .expect("worker-b claim");
    assert_eq!(second.len(), 1);
    let b = &second[0];

    // Worker-a tries to acknowledge with its stale token — REFUSED, nothing changes.
    let stale = complete(&pool, a.outbox_id, &a.lease_token, later)
        .await
        .expect("stale complete runs");
    assert_eq!(
        stale,
        CompleteOutcome::LeaseLost,
        "stale worker must not commit after a newer fencing value"
    );
    let (dispatched, owner, token, attempt) = outbox_state(&pool, outbox_id).await;
    assert!(!dispatched, "stale completion wrote nothing");
    assert_eq!(owner.as_deref(), Some("worker-b"));
    assert_eq!(token.as_deref(), Some(b.lease_token.as_str()));
    assert_eq!(attempt, 2);

    // Worker-b (current token, live lease) completes; the item is terminal.
    let fresh = complete(&pool, b.outbox_id, &b.lease_token, later)
        .await
        .expect("fresh complete runs");
    assert_eq!(fresh, CompleteOutcome::Completed);
    let (dispatched, owner, token, _) = outbox_state(&pool, outbox_id).await;
    assert!(dispatched);
    assert_eq!(owner, None);
    assert_eq!(token, None);

    // A second complete — even with the same token — is refused: the lease was cleared.
    let again = complete(&pool, b.outbox_id, &b.lease_token, later)
        .await
        .expect("second complete runs");
    assert_eq!(again, CompleteOutcome::LeaseLost);
    cleanup_item(&pool, outbox_id).await;
}

/// A lease that expired cannot complete even WITH its matching token: the item must be
/// re-claimed first (the completion liveness check is separate from the token check).
#[tokio::test]
async fn expired_lease_cannot_complete_even_with_matching_token() {
    let _guard = queue_guard().await;
    let Some(pool) = pool().await else { return };
    let (_, outbox_id) = seed_one_item(&pool, "expiry").await;
    let now = Utc::now();

    let claimed = claim_ready(&pool, "worker-a", now, LEASE, 1)
        .await
        .expect("claim");
    let a = &claimed[0];

    let after = now + Duration::seconds(61);
    let outcome = complete(&pool, a.outbox_id, &a.lease_token, after)
        .await
        .expect("complete after own expiry");
    assert_eq!(
        outcome,
        CompleteOutcome::LeaseLost,
        "an expired lease must not acknowledge"
    );

    let (dispatched, owner, token, _) = outbox_state(&pool, outbox_id).await;
    assert!(!dispatched);
    assert_eq!(owner.as_deref(), Some("worker-a"));
    assert_eq!(token.as_deref(), Some(a.lease_token.as_str()));
    cleanup_item(&pool, outbox_id).await;
}

/// Kill point 3 — crash AFTER the outbox claim, BEFORE delivery. The claim is durable but no
/// delivery effect exists; once the lease expires, another worker reclaims and the effect is
/// produced exactly once.
#[tokio::test]
async fn kill_point_3_after_claim_before_delivery_recovers_via_reclaim() {
    let _guard = queue_guard().await;
    let Some(pool) = pool().await else { return };
    let (event_id, outbox_id) = seed_one_item(&pool, "kp3").await;
    let now = Utc::now();

    let claimed = claim_ready(&pool, "worker-a", now, LEASE, 1)
        .await
        .expect("claim");
    assert_eq!(claimed.len(), 1);

    // Worker-a "crashes" here: the claim committed, the delivery never ran.
    assert_eq!(
        delivery_count(&pool, &event_id).await,
        0,
        "kill point 3: no delivery before the crash point"
    );
    let (dispatched, owner, _, attempt) = outbox_state(&pool, outbox_id).await;
    assert!(!dispatched);
    assert_eq!(owner.as_deref(), Some("worker-a"));
    assert_eq!(attempt, 1);

    // After the lease expires, worker-b reclaims and completes the delivery.
    let after = now + Duration::seconds(61);
    let reclaimed = claim_ready(&pool, "worker-b", after, LEASE, 1)
        .await
        .expect("reclaim");
    assert_eq!(reclaimed.len(), 1);
    let b = &reclaimed[0];
    assert_eq!(b.attempt, 2);

    assert_eq!(
        deliver(&pool, &event_id, b.outbox_id)
            .await
            .expect("deliver"),
        DeliverOutcome::Delivered
    );
    assert_eq!(
        complete(&pool, b.outbox_id, &b.lease_token, after)
            .await
            .expect("complete"),
        CompleteOutcome::Completed
    );

    assert_eq!(
        delivery_count(&pool, &event_id).await,
        1,
        "exactly one domain effect despite the crash and reclaim"
    );
    cleanup_item(&pool, outbox_id).await;
}

/// Kill point 4 — crash AFTER delivery, BEFORE acknowledgement. The delivery effect is
/// durable but the outbox item is still leased and undispatched; after expiry a redelivery
/// deduplicates to the SAME single effect, then the acknowledgement commits.
#[tokio::test]
async fn kill_point_4_after_delivery_before_acknowledgement_deduplicates() {
    let _guard = queue_guard().await;
    let Some(pool) = pool().await else { return };
    let (event_id, outbox_id) = seed_one_item(&pool, "kp4").await;
    let now = Utc::now();

    let claimed = claim_ready(&pool, "worker-a", now, LEASE, 1)
        .await
        .expect("claim");
    let a = &claimed[0];

    // Delivery commits (the effect exists)…
    assert_eq!(
        deliver(&pool, &event_id, a.outbox_id)
            .await
            .expect("deliver"),
        DeliverOutcome::Delivered
    );
    // …then worker-a "crashes" before the acknowledgement.
    let (dispatched, _, _, _) = outbox_state(&pool, outbox_id).await;
    assert!(
        !dispatched,
        "kill point 4: no acknowledgement before the crash point"
    );
    assert_eq!(delivery_count(&pool, &event_id).await, 1);

    // After expiry, worker-b reclaims and redelivers: the sink PK dedupes — one effect only.
    let after = now + Duration::seconds(61);
    let reclaimed = claim_ready(&pool, "worker-b", after, LEASE, 1)
        .await
        .expect("reclaim");
    assert_eq!(reclaimed.len(), 1);
    let b = &reclaimed[0];
    assert_eq!(b.attempt, 2);

    assert_eq!(
        deliver(&pool, &event_id, b.outbox_id)
            .await
            .expect("redeliver"),
        DeliverOutcome::Duplicate,
        "redelivery produced no second domain effect"
    );
    assert_eq!(delivery_count(&pool, &event_id).await, 1);

    assert_eq!(
        complete(&pool, b.outbox_id, &b.lease_token, after)
            .await
            .expect("complete"),
        CompleteOutcome::Completed
    );
    let (dispatched, owner, token, _) = outbox_state(&pool, outbox_id).await;
    assert!(dispatched);
    assert_eq!(owner, None);
    assert_eq!(token, None);
    cleanup_item(&pool, outbox_id).await;
}

/// Kill point 5 — crash AFTER the acknowledgement, BEFORE worker completion. The item is
/// dispatched and its lease cleared; nothing further ever claims it, so the acknowledged
/// event is never redelivered.
#[tokio::test]
async fn kill_point_5_after_acknowledgement_is_terminal() {
    let _guard = queue_guard().await;
    let Some(pool) = pool().await else { return };
    let (event_id, outbox_id) = seed_one_item(&pool, "kp5").await;
    let now = Utc::now();

    let claimed = claim_ready(&pool, "worker-a", now, LEASE, 1)
        .await
        .expect("claim");
    let a = &claimed[0];
    assert_eq!(
        deliver(&pool, &event_id, a.outbox_id)
            .await
            .expect("deliver"),
        DeliverOutcome::Delivered
    );
    assert_eq!(
        complete(&pool, a.outbox_id, &a.lease_token, now)
            .await
            .expect("complete"),
        CompleteOutcome::Completed
    );

    // Worker-a "crashes" after the acknowledgement, before returning.
    let (dispatched, owner, token, _) = outbox_state(&pool, outbox_id).await;
    assert!(dispatched, "acknowledgement committed");
    assert_eq!(owner, None);
    assert_eq!(token, None);

    // Even long after any lease would have expired, no claim returns the item.
    let later = now + Duration::hours(1);
    let again = claim_ready(&pool, "worker-b", later, LEASE, 10)
        .await
        .expect("claim after ack");
    assert!(
        again.is_empty(),
        "an acknowledged item must never be claimed or redelivered again"
    );
    assert_eq!(delivery_count(&pool, &event_id).await, 1);
    cleanup_item(&pool, outbox_id).await;
}
