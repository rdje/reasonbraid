//! Integration tests for the aggregate/event/outbox library (`PHASE-1.1.1`;
//! backlog 9, ADR-004).
//!
//! The library is the single write path the WP2 machinery was extracted into: the
//! SAME six writes (claim → locked head → event → state → outbox → result) as the
//! proven `tx` path, re-proven here through the library's own surface — fresh apply
//! vs replay, request-hash conflicts, the optional revision precondition, and the
//! outbox→event integrity fact. Like the other PostgreSQL suites, these tests skip
//! without `DATABASE_URL` (run them via `scripts/run_pg_tests.sh` or the `pg-tests`
//! CI job).

use reasonbraid_server::agg::{apply, AggregateCommand, AggregateError, AggregateEvent};
use sqlx::PgPool;

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
    Some(pool)
}

/// A library-shaped command for a throwaway `counter` aggregate (the test
/// aggregate the acceptance names — not the thread domain).
fn counter_cmd<'a>(
    tenant: &'a str,
    aggregate: &'a str,
    key: &'a str,
    hash: &'a str,
    event_id: &'a str,
    expected: Option<i64>,
) -> AggregateCommand<'a> {
    AggregateCommand {
        tenant_id: tenant,
        aggregate_type: "counter",
        aggregate_id: aggregate,
        idempotency_key: key,
        request_hash: hash,
        expected_revision: expected,
        event: AggregateEvent {
            event_id: event_id.to_string(),
            event_type: "counter.incremented".to_string(),
            body: serde_json::json!({ "event_id": event_id }),
        },
        next_state: serde_json::json!({ "count": 1 }),
        result: serde_json::json!({ "ok": true, "event_id": event_id }),
    }
}

/// The acceptance's helper proof: a FRESH apply writes all four durable records
/// and returns the derived revision; a REPLAY of the same key + hash returns the
/// ORIGINAL result, writes nothing new, and reports the head's current revision.
#[tokio::test]
async fn fresh_apply_then_replay_returns_the_original_result() {
    let Some(pool) = pool().await else { return };
    let tenant = "ten_agg1";
    let aggregate = "ctr_agg1";

    let cmd = counter_cmd(tenant, aggregate, "key_agg1", "hash_a", "evt_agg1_1", None);
    let fresh = apply(&pool, &cmd).await.expect("fresh apply succeeds");
    assert!(!fresh.replayed, "the first submission is not a replay");
    assert_eq!(fresh.revision, Some(1), "the first event is revision 1");
    assert_eq!(
        fresh.result, cmd.result,
        "a fresh apply returns the command's semantic result"
    );

    // Durable-state proof, from a SEPARATE connection: event, state, outbox, and the
    // stored idempotency result all exist after the commit.
    let (n_events,): (i64,) =
        sqlx::query_as("SELECT count(*) FROM event_log WHERE tenant_id = $1 AND aggregate_id = $2")
            .bind(tenant)
            .bind(aggregate)
            .fetch_one(&pool)
            .await
            .expect("count events");
    assert_eq!(n_events, 1);
    let (revision,): (i64,) = sqlx::query_as(
        "SELECT aggregate_version FROM aggregate_state WHERE tenant_id = $1 AND aggregate_id = $2",
    )
    .bind(tenant)
    .bind(aggregate)
    .fetch_one(&pool)
    .await
    .expect("read state revision");
    assert_eq!(revision, 1);
    let (n_outbox,): (i64,) = sqlx::query_as("SELECT count(*) FROM outbox WHERE event_id = $1")
        .bind(&cmd.event.event_id)
        .fetch_one(&pool)
        .await
        .expect("count outbox");
    assert_eq!(n_outbox, 1, "the outbox item implies the event is durable");

    // The replay: same key, same hash — the ORIGINAL result, nothing new written.
    let replay = apply(&pool, &cmd).await.expect("replay succeeds");
    assert!(replay.replayed, "the second submission is a replay");
    assert_eq!(replay.result, cmd.result, "the ORIGINAL result, verbatim");
    assert_eq!(
        replay.revision,
        Some(1),
        "the replay reports the head's revision"
    );

    let (n_events,): (i64,) =
        sqlx::query_as("SELECT count(*) FROM event_log WHERE tenant_id = $1 AND aggregate_id = $2")
            .bind(tenant)
            .bind(aggregate)
            .fetch_one(&pool)
            .await
            .expect("count events after replay");
    assert_eq!(n_events, 1, "a replay appends no second event");
}

/// The conflict arm: the same key with a DIFFERENT request hash is refused — a
/// typed error, and no second event exists.
#[tokio::test]
async fn a_different_request_hash_under_the_same_key_is_a_conflict() {
    let Some(pool) = pool().await else { return };
    let tenant = "ten_agg2";
    let aggregate = "ctr_agg2";

    let first = counter_cmd(tenant, aggregate, "key_agg2", "hash_a", "evt_agg2_1", None);
    apply(&pool, &first).await.expect("first apply");

    let conflicting = counter_cmd(tenant, aggregate, "key_agg2", "hash_b", "evt_agg2_2", None);
    let err = apply(&pool, &conflicting)
        .await
        .expect_err("a different hash under the same key must conflict");
    assert!(
        matches!(
            err,
            AggregateError::IdempotencyConflict {
                ref key,
                ref stored_hash,
                ..
            } if key == "key_agg2" && stored_hash == "hash_a"
        ),
        "got: {err}"
    );

    let (n_events,): (i64,) =
        sqlx::query_as("SELECT count(*) FROM event_log WHERE tenant_id = $1 AND aggregate_id = $2")
            .bind(tenant)
            .bind(aggregate)
            .fetch_one(&pool)
            .await
            .expect("count events after conflict");
    assert_eq!(n_events, 1, "the conflict wrote no event");
}

/// The optional optimistic-concurrency precondition: `Some(n)` requires the head
/// to be at revision `n` — 0 for an aggregate with no row — and a stale
/// expectation is a typed refusal that writes nothing.
#[tokio::test]
async fn the_expected_revision_precondition_holds_and_refuses() {
    let Some(pool) = pool().await else { return };
    let tenant = "ten_agg3";
    let aggregate = "ctr_agg3";

    // A fresh aggregate is revision 0: `Some(0)` accepts the first event.
    let first = counter_cmd(
        tenant,
        aggregate,
        "key_agg3a",
        "hash_a",
        "evt_agg3_1",
        Some(0),
    );
    let outcome = apply(&pool, &first)
        .await
        .expect("revision 0 precondition holds");
    assert_eq!(outcome.revision, Some(1));

    // A stale expectation is refused with the actual revision named.
    let stale = counter_cmd(
        tenant,
        aggregate,
        "key_agg3b",
        "hash_b",
        "evt_agg3_2",
        Some(0),
    );
    let err = apply(&pool, &stale)
        .await
        .expect_err("a stale expectation must refuse");
    assert!(
        matches!(
            err,
            AggregateError::RevisionConflict {
                expected: 0,
                actual: 1,
                ..
            }
        ),
        "got: {err}"
    );

    // The correct expectation accepts, and the revision advances.
    let right = counter_cmd(
        tenant,
        aggregate,
        "key_agg3c",
        "hash_c",
        "evt_agg3_3",
        Some(1),
    );
    let outcome = apply(&pool, &right)
        .await
        .expect("revision 1 precondition holds");
    assert_eq!(outcome.revision, Some(2));

    let (n_events,): (i64,) =
        sqlx::query_as("SELECT count(*) FROM event_log WHERE tenant_id = $1 AND aggregate_id = $2")
            .bind(tenant)
            .bind(aggregate)
            .fetch_one(&pool)
            .await
            .expect("count events");
    assert_eq!(n_events, 2, "the refused command wrote no event");
}

/// The outbox→event integrity fact through the library's surface: every outbox
/// row names an event that is already durable (the FK chain the reconciliation
/// design rests on).
#[tokio::test]
async fn an_outbox_item_implies_its_event_is_durable() {
    let Some(pool) = pool().await else { return };
    let cmd = counter_cmd(
        "ten_agg4",
        "ctr_agg4",
        "key_agg4",
        "hash_a",
        "evt_agg4_1",
        None,
    );
    apply(&pool, &cmd).await.expect("apply succeeds");

    let (joined,): (i64,) = sqlx::query_as(
        "SELECT count(*) FROM outbox o JOIN event_log e ON e.event_id = o.event_id \
         WHERE o.event_id = $1 AND e.tenant_id = $2 AND e.aggregate_id = $3",
    )
    .bind(&cmd.event.event_id)
    .bind(cmd.tenant_id)
    .bind(cmd.aggregate_id)
    .fetch_one(&pool)
    .await
    .expect("join outbox to event");
    assert_eq!(joined, 1, "the outbox row joins to its durable event");
}
