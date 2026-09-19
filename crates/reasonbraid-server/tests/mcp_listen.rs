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
            "node_enrollment_tokens",
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
            "node_proof_nonces",
            "nodes",
            "hosts",
            "recruitment_panels",
            "recruitment_responses",
            "recruitment_calls",
            "evidence_citations",
            "reference_registrations",
            "routing_recommendations",
            "routing_resolutions",
            "policy_reviews",
            "policy_outcomes",
            "policy_corrections",
            "policy_drift",
            "policy_publications",
            "policy_projections",
            "policy_approvals",
            "policy_decisions",
            "policy_proposals",
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

/// `SIGNOFF-REPAIR.6.2.1` — the dedup window retains the MOST RECENT ids.
///
/// 🔴 It retained the OLDEST: `next.push(id)` appends to the end and
/// `next.truncate(DEDUP_WINDOW)` keeps the FRONT, so once the window was full
/// every new id was appended at index `DEDUP_WINDOW` and discarded on the same
/// line. Past 64 deliveries the window froze and nothing recent deduplicated.
///
/// ⛔ **Both arms are necessary and the second is the one that is easy to omit.**
/// A window that simply grew without bound would refuse every replay and pass
/// the first assertion — so the control also proves an id that has FALLEN OUT is
/// accepted, which is the bound still holding. One arm alone cannot distinguish
/// *the newest are kept* from *everything is kept*.
#[tokio::test]
async fn the_dedup_window_retains_the_most_recent_deliveries() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    sqlx::query("INSERT INTO tenants (tenant_id) VALUES ($1)")
        .bind("ten_aaaaaaaa-bbbb-4ccc-8ddd-eeeeeeeeeeee")
        .execute(&pool)
        .await
        .expect("seed the tenant");
    let tenant = "ten_aaaaaaaa-bbbb-4ccc-8ddd-eeeeeeeeeeee";
    let window = reasonbraid_server::mcp_listen_internal::DEDUP_WINDOW;

    let record = |id: String, cursor: i64| {
        let pool = pool.clone();
        async move {
            let mut tx = pool.begin().await.expect("begin");
            let accepted = reasonbraid_server::mcp_listen_internal::record(
                &mut *tx, tenant, "sub-w", &id, cursor,
            )
            .await
            .expect("record");
            tx.commit().await.expect("commit");
            accepted
        }
    };

    // Drive PAST the boundary — the defect is invisible below it, which is why
    // the suite's two-delivery control never saw it.
    let total = window + 6;
    for i in 0..total {
        assert!(
            record(format!("d-{i:03}"), i as i64).await,
            "delivery {i} is new and must be accepted"
        );
    }

    let stored: serde_json::Value = sqlx::query_scalar(
        "SELECT dedup_window FROM mcp_listen_state \
         WHERE tenant_id = $1 AND subscription_id = $2",
    )
    .bind(tenant)
    .bind("sub-w")
    .fetch_one(&pool)
    .await
    .expect("the stored window");
    let ids: Vec<String> = serde_json::from_value(stored).expect("the window is an array");
    assert_eq!(ids.len(), window, "the window stays BOUNDED at its size");

    // ARM 1 — the MOST RECENT delivery is deduplicated.
    let newest = format!("d-{:03}", total - 1);
    assert!(
        ids.contains(&newest),
        "the newest id is IN the window: first={:?} last={:?}",
        ids.first(),
        ids.last()
    );
    assert!(
        !record(newest.clone(), 9_000).await,
        "replaying the most recent delivery `{newest}` is the replay SKIP — accepting \
         it is a DOUBLE DELIVERY, because the caller commits the effects on `true`"
    );

    // ARM 2 — an id that has FALLEN OUT is accepted, so the bound still holds.
    // ⛔ Without this, an unbounded window passes arm 1 and the repair would be a
    // memory leak wearing a fix's clothes.
    let evicted = "d-000".to_string();
    assert!(
        !ids.contains(&evicted),
        "the oldest id has left the bounded window: {ids:?}"
    );
    assert!(
        record(evicted, 9_001).await,
        "an id outside the window is accepted — the window is bounded, not infinite"
    );

    // ⭐ And the cursor is untouched by a replay, which the suite's original
    // control asserts at two deliveries and is re-asserted here past the bound.
    let state = reasonbraid_server::mcp_listen_internal::state(&pool, tenant, "sub-w")
        .await
        .expect("read")
        .expect("the state exists");
    assert_eq!(
        state.0, 9_001,
        "the accepted out-of-window delivery advanced the cursor; the refused \
         replay before it did not"
    );
}
