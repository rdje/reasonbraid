//! The MCP listen-stream durable-state proof (`PHASE-8.3.4`, ADR-024,
//! §9.6, migration 0050): the listen stream is the EPHEMERAL transport
//! state; the durable state — the subscription, the cursor, the delivery
//! ids, the deduplication — stays in REASONBRAID. Measured:
//!   - the first delivery registers the subscription (the cursor starts
//!     there);
//!   - a DUPLICATE delivery id is the replay SKIP (the cursor unchanged);
//!   - the next delivery advances the cursor + the dedup window;
//!   - the resume plan is the OWN cursor + the honest possible-gap flag
//!     (the pure machine — the unit test).
//!
//! Run with `scripts/run_pg_tests.sh` locally or the `pg-tests` CI job.
//! Without `DATABASE_URL` these skip, so `make check` stays green offline.

#[path = "support/mod.rs"]
mod pg_test_support;

#[path = "support/cleanup.rs"]
mod pg_cleanup;

use std::sync::OnceLock;

use sqlx::PgPool;

static LISTEN_LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();

async fn guard() -> tokio::sync::MutexGuard<'static, ()> {
    LISTEN_LOCK
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
    // Preserve the original scope plus its explicit FK dependencies; retain the CA.
    pg_cleanup::delete_tables(
        &pool,
        &[
            "mcp_listen_state",
            "cross_domain_receipts",
            "federation_agreements",
            "quota_events",
            "usage_quotas",
            "outbox_delivery",
            "outbox",
            "node_events",
            "node_inbox",
            "budget_reservations",
            "budget_ceilings",
            "spend_breakers",
            "administrative_effects",
            "authorization_records",
            "authority_grants",
            "enrollments",
            "enrollment_boundaries",
            "profile_versions",
            "agent_profiles",
            "runs",
            "incarnations",
            "recruitment_offers",
            "agent_roles",
            "human_principals",
            "idempotency",
            "event_log",
            "aggregate_state",
            "tenant_bootstrap_requests",
            "node_certificates",
            "node_keys",
            "node_leases",
            "nodes",
            "hosts",
            "node_enrollment_tokens",
            "recruitment_panels",
            "recruitment_responses",
            "recruitment_calls",
            "tenants",
        ],
    )
    .await
    .expect("purge checked fixture plan");
    Some(pool)
}

#[tokio::test]
async fn the_durable_state_dedups_and_resumes_from_the_own_cursor() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    // The seed tenant (the listen state references it).
    sqlx::query("INSERT INTO tenants (tenant_id) VALUES ($1)")
        .bind("ten_aaaaaaaa-bbbb-4ccc-8ddd-eeeeeeeeeeee")
        .execute(&pool)
        .await
        .expect("seed the tenant");
    let tenant = "ten_aaaaaaaa-bbbb-4ccc-8ddd-eeeeeeeeeeee";

    // 1. The first delivery registers the state.
    let mut tx = pool.begin().await.expect("begin");
    let accepted = reasonbraid_server::mcp_listen_internal::record(
        &mut *tx,
        tenant,
        "sub-1",
        "delivery-1",
        10,
    )
    .await
    .expect("record");
    assert!(accepted, "the first delivery accepts");
    tx.commit().await.expect("commit");

    // 2. The duplicate delivery id is the replay SKIP.
    let mut tx = pool.begin().await.expect("begin");
    let accepted = reasonbraid_server::mcp_listen_internal::record(
        &mut *tx,
        tenant,
        "sub-1",
        "delivery-1",
        10,
    )
    .await
    .expect("record");
    assert!(!accepted, "the duplicate delivery skips");
    tx.commit().await.expect("commit");

    // 3. The next delivery advances the cursor.
    let mut tx = pool.begin().await.expect("begin");
    let accepted = reasonbraid_server::mcp_listen_internal::record(
        &mut *tx,
        tenant,
        "sub-1",
        "delivery-2",
        11,
    )
    .await
    .expect("record");
    assert!(accepted, "the next delivery accepts");
    tx.commit().await.expect("commit");

    let state: (i64, Option<String>) =
        reasonbraid_server::mcp_listen_internal::state(&pool, tenant, "sub-1")
            .await
            .expect("read")
            .expect("the state exists");
    assert_eq!(state.0, 11, "the cursor advanced to the accepted delivery");
    assert_eq!(
        state.1.as_deref(),
        Some("delivery-2"),
        "the last delivery records"
    );
}
