//! The quarantine-preserving-evidence proof (`PHASE-7.1.3.3`, §16.11,
//! `docs/decisions/2026-09-08_quarantine-preserves-evidence.md`): the
//! retention NEVER deletes a quarantined row — the preservation SURVIVES
//! the disposition. A dead-lettered row is delivered (acknowledged) by
//! definition, so the prune's quarantine exclusion is the enforceable line.
//! Measured:
//!   - an old acknowledged QUARANTINED row survives the prune (the row +
//!     the reason stay);
//!   - an old acknowledged non-quarantined row is pruned (the sweep still
//!     works);
//!   - a recent row survives (the age gate still works);
//!   - the prune's measured receipt (before/deleted/after in one
//!     transaction) matches the rows exactly.
//!
//! Run with `scripts/run_pg_tests.sh` locally or the `pg-tests` CI job.
//! Without `DATABASE_URL` these skip, so `make check` stays green offline.

#[path = "support/mod.rs"]
mod pg_test_support;

use std::net::SocketAddr;
use std::sync::OnceLock;

use reasonbraid_server::{api_router, PRINCIPAL_HEADER};
use serde_json::{json, Value};
use sqlx::PgPool;

static QUARANTINE_LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();

async fn guard() -> tokio::sync::MutexGuard<'static, ()> {
    QUARANTINE_LOCK
        .get_or_init(|| tokio::sync::Mutex::new(()))
        .lock()
        .await
}

async fn pool() -> Option<PgPool> {
    let pool = pg_test_support::pool().await?;
    sqlx::migrate!("../../migrations")
        .run(&pool)
        .await
        .expect("apply migrations");
    for table in [
        "quota_events",
        "usage_quotas",
        "outbox_delivery",
        "outbox",
        "node_events",
        "node_inbox",
        "budget_reservations",
        "budget_ceilings",
        "authorization_records",
        "authority_grants",
        "enrollments",
        "enrollment_boundaries",
        "profile_versions",
        "agent_profiles",
        "agent_roles",
        "human_principals",
        "idempotency",
        "event_log",
        "aggregate_state",
        "cross_domain_receipts",
        "tenants",
    ] {
        sqlx::query(&format!("DELETE FROM {table}"))
            .execute(&pool)
            .await
            .expect("purge table");
    }
    Some(pool)
}

struct TestServer {
    addr: SocketAddr,
    _handle: tokio::task::JoinHandle<()>,
}

impl TestServer {
    async fn start(pool: &PgPool) -> Self {
        let router = api_router(pool.clone());
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind ephemeral loopback port");
        let addr = listener.local_addr().unwrap();
        let handle = tokio::spawn(async move {
            axum::serve(listener, router).await.expect("serve");
        });
        Self {
            addr,
            _handle: handle,
        }
    }

    fn base(&self) -> String {
        format!("http://{}", self.addr)
    }
}

async fn post(
    client: &reqwest::Client,
    base: &str,
    path: &str,
    principal: &str,
    body: &Value,
) -> (u16, Value) {
    let response = client
        .post(format!("{base}{path}"))
        .header(PRINCIPAL_HEADER, principal)
        .json(body)
        .send()
        .await
        .expect("post request");
    let status = response.status().as_u16();
    (status, response.json().await.expect("post json"))
}

#[tokio::test]
async fn the_retention_never_deletes_a_quarantined_row() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    // 1. The bootstrap human (the prune requires the tenant_admin grant).
    let (status, human) = post(
        &client,
        &base,
        "/v1/enrollments",
        "",
        &json!({ "kind": "human", "name": "qua-human" }),
    )
    .await;
    assert_eq!(status, 200, "the human enrolls: {human}");
    let human_id = human["principal_id"].as_str().unwrap().to_string();
    let tenant = human["tenant_id"].as_str().unwrap().to_string();

    // 2. The seeded inbox for one node — three rows:
    //    A: delivered (acknowledged) 2h ago, dead-lettered 1h ago → the
    //       EVIDENCE the preservation rule protects;
    //    B: delivered 2h ago, never quarantined → the sweep's target;
    //    C: delivered 60s ago → inside the window.
    let node_id = "nod_seeded_evidence";
    let old_ack = chrono::Utc::now() - chrono::Duration::hours(2);
    let quarantine_at = chrono::Utc::now() - chrono::Duration::hours(1);
    let recent_ack = chrono::Utc::now() - chrono::Duration::seconds(60);
    sqlx::query(
        "INSERT INTO node_inbox \
         (node_id, cursor, command_id, tenant_id, thread_id, payload, acknowledged_at, \
          quarantined_at, quarantine_reason) \
         VALUES ($1, 1, 'cmd-a', $2, 'thr_a', '{}'::jsonb, $3, $4, $5)",
    )
    .bind(node_id)
    .bind(&tenant)
    .bind(old_ack)
    .bind(quarantine_at)
    .bind("dead-lettered: budget denial")
    .execute(&pool)
    .await
    .expect("seed the quarantined row");
    sqlx::query(
        "INSERT INTO node_inbox \
         (node_id, cursor, command_id, tenant_id, thread_id, payload, acknowledged_at) \
         VALUES ($1, 2, 'cmd-b', $2, 'thr_b', '{}'::jsonb, $3)",
    )
    .bind(node_id)
    .bind(&tenant)
    .bind(old_ack)
    .execute(&pool)
    .await
    .expect("seed the prunable row");
    sqlx::query(
        "INSERT INTO node_inbox \
         (node_id, cursor, command_id, tenant_id, thread_id, payload, acknowledged_at) \
         VALUES ($1, 3, 'cmd-c', $2, 'thr_c', '{}'::jsonb, $3)",
    )
    .bind(node_id)
    .bind(&tenant)
    .bind(recent_ack)
    .execute(&pool)
    .await
    .expect("seed the recent row");

    // 3. The prune (min_age 3600s): the measured receipt — before 3,
    //    deleted 1 (ONLY the non-quarantined old row), after 2.
    let (status, pruned) = post(
        &client,
        &base,
        "/v1/nodes/inbox/prune",
        &human_id,
        &json!({ "tenant_id": tenant, "node_id": node_id, "min_age_seconds": 3600 }),
    )
    .await;
    assert_eq!(status, 200, "the prune answers: {pruned}");
    assert_eq!(pruned["before"], json!(3), "the measured before");
    assert_eq!(pruned["deleted"], json!(1), "only the sweep's target left");
    assert_eq!(pruned["after"], json!(2), "the measured after");

    // 4. The evidence survived the disposition: the quarantined row and its
    //    REASON are intact; the prunable row is gone; the recent row stays.
    let quarantined: Option<(Option<chrono::DateTime<chrono::Utc>>, Option<String>)> =
        sqlx::query_as(
            "SELECT quarantined_at, quarantine_reason FROM node_inbox \
             WHERE node_id = $1 AND command_id = 'cmd-a'",
        )
        .bind(node_id)
        .fetch_optional(&pool)
        .await
        .expect("read the quarantined row");
    let Some((at, reason)) = quarantined else {
        panic!("the quarantined row must SURVIVE the retention");
    };
    assert!(
        at.is_some(),
        "the quarantine fact is untouched by the disposition"
    );
    assert_eq!(
        reason.as_deref(),
        Some("dead-lettered: budget denial"),
        "the quarantine reason survives"
    );

    let prunable: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM node_inbox WHERE node_id = $1 AND command_id = 'cmd-b'",
    )
    .bind(node_id)
    .fetch_one(&pool)
    .await
    .expect("count the prunable row");
    assert_eq!(prunable, 0, "the sweep removed its target");

    let recent: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM node_inbox WHERE node_id = $1 AND command_id = 'cmd-c'",
    )
    .bind(node_id)
    .fetch_one(&pool)
    .await
    .expect("count the recent row");
    assert_eq!(recent, 1, "the age gate kept the recent row");
}
