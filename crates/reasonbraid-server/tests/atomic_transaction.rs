//! Integration tests for the WP2 atomic transaction (`.2.1`).
//!
//! These are the only place the acceptance is proven: they need a live PostgreSQL. Run
//! them with `scripts/run_pg_tests.sh` locally, or the `pg-tests` CI job. Without
//! `DATABASE_URL` they skip, so `make check` / `cargo test --all` stay green offline.

use reasonbraid_server::{apply_command, ApplyError, Command};
use serde_json::Value;
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

fn cmd(tag: &str, hash: &str, event_id: &str) -> Command {
    Command {
        tenant_id: format!("ten_{tag}"),
        aggregate_type: "thread".to_string(),
        aggregate_id: format!("thr_{tag}"),
        idempotency_key: format!("key_{tag}"),
        request_hash: hash.to_string(),
        event_id: event_id.to_string(),
        event_type: "thread.created".to_string(),
        body: serde_json::json!({ "tag": tag }),
        next_state: serde_json::json!({ "status": "open", "tag": tag }),
        result: serde_json::json!({ "tag": tag, "event_id": event_id }),
    }
}

/// Acceptance 1: a successful response corresponds to committed durable state. Read back
/// all four tables from a *separate* connection after the commit.
#[tokio::test]
async fn successful_response_implies_committed_durable_state() {
    let Some(pool) = pool().await else { return };
    let c = cmd("ac1", "h1", "evt_ac1_1");
    let outcome = apply_command(&pool, &c).await.expect("apply succeeds");
    assert!(!outcome.replayed);

    let mut conn = pool.acquire().await.expect("second connection");

    let (n_events,): (i64,) =
        sqlx::query_as("SELECT count(*) FROM event_log WHERE tenant_id = $1 AND aggregate_id = $2")
            .bind(&c.tenant_id)
            .bind(&c.aggregate_id)
            .fetch_one(&mut *conn)
            .await
            .expect("count events");
    assert_eq!(n_events, 1, "exactly one event committed");

    let (version,): (i64,) = sqlx::query_as(
        "SELECT aggregate_version FROM aggregate_state WHERE tenant_id = $1 AND aggregate_id = $2",
    )
    .bind(&c.tenant_id)
    .bind(&c.aggregate_id)
    .fetch_one(&mut *conn)
    .await
    .expect("read state");
    assert_eq!(version, 1);

    let (result,): (Value,) = sqlx::query_as(
        "SELECT response_result FROM idempotency WHERE tenant_id = $1 AND idempotency_key = $2",
    )
    .bind(&c.tenant_id)
    .bind(&c.idempotency_key)
    .fetch_one(&mut *conn)
    .await
    .expect("read idempotency");
    assert_eq!(result, c.result);

    let (n_outbox,): (i64,) =
        sqlx::query_as("SELECT count(*) FROM outbox WHERE tenant_id = $1 AND event_id = $2")
            .bind(&c.tenant_id)
            .bind(&c.event_id)
            .fetch_one(&mut *conn)
            .await
            .expect("count outbox");
    assert_eq!(n_outbox, 1, "exactly one outbox item committed");
}

/// Acceptance 2: resubmission with the same key and hash returns the ORIGINAL result and
/// writes nothing new.
#[tokio::test]
async fn same_key_and_hash_returns_original_result() {
    let Some(pool) = pool().await else { return };
    let c = cmd("ac2", "h", "evt_ac2_1");
    let first = apply_command(&pool, &c).await.expect("first apply");
    assert!(!first.replayed);

    let replay = apply_command(&pool, &c).await.expect("replay");
    assert!(replay.replayed);
    assert_eq!(
        replay.result, first.result,
        "original result returned verbatim"
    );

    let (n,): (i64,) = sqlx::query_as("SELECT count(*) FROM event_log WHERE tenant_id = $1")
        .bind(&c.tenant_id)
        .fetch_one(&pool)
        .await
        .expect("count");
    assert_eq!(n, 1, "no second event");
}

/// Acceptance 3: resubmission with the same key but a DIFFERENT hash is a conflict.
#[tokio::test]
async fn different_hash_is_conflict() {
    let Some(pool) = pool().await else { return };
    let first = apply_command(&pool, &cmd("ac3", "h1", "evt_ac3_1"))
        .await
        .expect("first");
    assert!(!first.replayed);

    let conflict = apply_command(&pool, &cmd("ac3", "h2", "evt_ac3_2")).await;
    assert!(
        matches!(conflict, Err(ApplyError::IdempotencyConflict { .. })),
        "expected IdempotencyConflict, got {conflict:?}"
    );

    let (n,): (i64,) = sqlx::query_as("SELECT count(*) FROM event_log WHERE tenant_id = 'ten_ac3'")
        .fetch_one(&pool)
        .await
        .expect("count");
    assert_eq!(n, 1, "conflict wrote no event");
}

/// Acceptance 4: transport redelivery produces exactly one domain effect. The first
/// response is "lost" (commit happened, outcome dropped); the redelivered command replays.
#[tokio::test]
async fn transport_redelivery_produces_one_domain_effect() {
    let Some(pool) = pool().await else { return };
    let c = cmd("ac4", "h", "evt_ac4_1");
    apply_command(&pool, &c)
        .await
        .expect("first (response 'lost')");

    let replay = apply_command(&pool, &c).await.expect("redelivery");
    assert!(replay.replayed);

    let (n_events,): (i64,) = sqlx::query_as("SELECT count(*) FROM event_log WHERE tenant_id = $1")
        .bind(&c.tenant_id)
        .fetch_one(&pool)
        .await
        .expect("count events");
    let (n_outbox,): (i64,) = sqlx::query_as("SELECT count(*) FROM outbox WHERE tenant_id = $1")
        .bind(&c.tenant_id)
        .fetch_one(&pool)
        .await
        .expect("count outbox");
    assert_eq!(n_events, 1, "redelivery produced exactly one event");
    assert_eq!(n_outbox, 1, "redelivery produced exactly one outbox item");
}

/// Kill point "before commit": a failure mid-transaction rolls back ALL four writes —
/// including the idempotency claim made in step 1.
#[tokio::test]
async fn failure_before_commit_rolls_back_all_four_writes() {
    let Some(pool) = pool().await else { return };
    apply_command(&pool, &cmd("ac5", "h1", "evt_X"))
        .await
        .expect("first");

    // A second command uses a fresh idempotency key but reuses event_id "evt_X", so the
    // event PK violates mid-transaction — after the idempotency claim was already inserted.
    let second = Command {
        tenant_id: "ten_ac5".to_string(),
        aggregate_type: "thread".to_string(),
        aggregate_id: "thr_ac5".to_string(),
        idempotency_key: "key_ac5_second".to_string(),
        request_hash: "h2".to_string(),
        event_id: "evt_X".to_string(), // reuse → PK violation
        event_type: "thread.created".to_string(),
        body: serde_json::json!({ "tag": "ac5_second" }),
        next_state: serde_json::json!({ "status": "open" }),
        result: serde_json::json!({ "tag": "ac5_second" }),
    };
    let r = apply_command(&pool, &second).await;
    assert!(
        matches!(r, Err(ApplyError::Sql(_))),
        "expected a DB error, got {r:?}"
    );

    // The idempotency claim for the second key must have rolled back too.
    let (n_idem,): (i64,) = sqlx::query_as(
        "SELECT count(*) FROM idempotency \
         WHERE tenant_id = 'ten_ac5' AND idempotency_key = 'key_ac5_second'",
    )
    .fetch_one(&pool)
    .await
    .expect("count idempotency");
    assert_eq!(
        n_idem, 0,
        "idempotency claim rolled back with the transaction"
    );

    // State still version 1, one event, one outbox — no partial effect.
    let (version,): (i64,) = sqlx::query_as(
        "SELECT aggregate_version FROM aggregate_state \
         WHERE tenant_id = 'ten_ac5' AND aggregate_id = 'thr_ac5'",
    )
    .fetch_one(&pool)
    .await
    .expect("read state");
    assert_eq!(version, 1);

    let (n_events,): (i64,) =
        sqlx::query_as("SELECT count(*) FROM event_log WHERE tenant_id = 'ten_ac5'")
            .fetch_one(&pool)
            .await
            .expect("count events");
    assert_eq!(n_events, 1);

    let (n_outbox,): (i64,) =
        sqlx::query_as("SELECT count(*) FROM outbox WHERE tenant_id = 'ten_ac5'")
            .fetch_one(&pool)
            .await
            .expect("count outbox");
    assert_eq!(n_outbox, 1);
}
