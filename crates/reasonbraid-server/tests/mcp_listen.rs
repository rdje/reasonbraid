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

/// `SIGNOFF-REPAIR.6.2.2` — a late delivery is APPLIED and the cursor does not
/// go backwards.
///
/// 🔴 `SET last_cursor = $1` was unconditional, so recording cursor 100 and then
/// cursor 5 left the stored cursor at **5**. `resume_plan` reads that value as
/// *the OWN cursor*, so one out-of-order delivery rewound the resume point and
/// every delivery above it was re-offered on the next reconnect.
///
/// ⛔ **The decision this control asserts is a CLAMP, not a refusal**, and the
/// two differ observably. A late delivery is a real delivery: the dedup window
/// has already said it is new, so refusing it would DROP it — a worse outcome
/// than the cursor problem it would fix. The delivery is applied and recorded;
/// only the high-water mark is protected.
#[tokio::test]
async fn a_late_delivery_is_applied_and_the_cursor_does_not_rewind() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    sqlx::query("INSERT INTO tenants (tenant_id) VALUES ($1)")
        .bind("ten_aaaaaaaa-bbbb-4ccc-8ddd-eeeeeeeeeeee")
        .execute(&pool)
        .await
        .expect("seed the tenant");
    let tenant = "ten_aaaaaaaa-bbbb-4ccc-8ddd-eeeeeeeeeeee";

    let record = |id: String, cursor: i64| {
        let pool = pool.clone();
        async move {
            let mut tx = pool.begin().await.expect("begin");
            let accepted = reasonbraid_server::mcp_listen_internal::record(
                &mut *tx, tenant, "sub-m", &id, cursor,
            )
            .await
            .expect("record");
            tx.commit().await.expect("commit");
            accepted
        }
    };
    let state = || {
        let pool = pool.clone();
        async move {
            reasonbraid_server::mcp_listen_internal::state(&pool, tenant, "sub-m")
                .await
                .expect("read")
                .expect("the state exists")
        }
    };

    assert!(
        record("d-hi".into(), 100).await,
        "the first delivery accepts"
    );
    assert_eq!(state().await, (100, Some("d-hi".to_string())));

    // ── THE LATE DELIVERY ────────────────────────────────────────────────────
    assert!(
        record("d-late".into(), 5).await,
        "a late delivery is still a REAL delivery and must be applied — refusing \
         it would drop it, which is worse than the cursor problem it would fix"
    );
    let (cursor, last) = state().await;
    assert_eq!(
        cursor, 100,
        "the cursor is a HIGH-WATER MARK and does not rewind; `resume_plan` reads \
         it as the resume point, so a rewind re-offers everything above it"
    );
    assert_eq!(
        last.as_deref(),
        Some("d-hi"),
        "`last_delivery` is the delivery that SET the cursor, so the pair stays one \
         fact rather than two independent latest-writes"
    );

    // ⭐ And the late delivery really was recorded, not silently dropped: its id
    // is in the dedup window, so replaying it is the replay SKIP. Without this,
    // a repair that simply ignored low cursors would pass every assertion above.
    assert!(
        !record("d-late".into(), 5).await,
        "the late delivery was RECORDED — replaying it is the skip"
    );

    // ── AND FORWARD PROGRESS IS UNAFFECTED ───────────────────────────────────
    assert!(
        record("d-next".into(), 101).await,
        "a newer delivery accepts"
    );
    assert_eq!(
        state().await,
        (101, Some("d-next".to_string())),
        "a delivery above the mark advances BOTH halves"
    );
}

/// `SIGNOFF-REPAIR.6.2.3` — a malformed dedup window REFUSES the delivery and
/// keeps the evidence.
///
/// 🔴 `serde_json::from_value(window).unwrap_or_default()` turned any window that
/// is not an array of strings into an EMPTY one, so the subscription silently
/// stopped deduplicating and a known delivery replayed and was ACCEPTED. ⛔ And
/// it was self-erasing: the UPDATE that followed overwrote the malformed value
/// with a fresh one-element array, destroying the only evidence that anything
/// had been wrong.
///
/// **The decision is FAIL-CLOSED**, and the control asserts both halves of it:
/// the delivery is refused with a typed error naming the subscription, and the
/// malformed value is still in the column afterwards.
#[tokio::test]
async fn a_malformed_dedup_window_refuses_the_delivery_and_keeps_the_evidence() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    sqlx::query("INSERT INTO tenants (tenant_id) VALUES ($1)")
        .bind("ten_aaaaaaaa-bbbb-4ccc-8ddd-eeeeeeeeeeee")
        .execute(&pool)
        .await
        .expect("seed the tenant");
    let tenant = "ten_aaaaaaaa-bbbb-4ccc-8ddd-eeeeeeeeeeee";

    let record = |sub: &'static str, id: &'static str, cursor: i64| {
        let pool = pool.clone();
        async move {
            let mut tx = pool.begin().await.expect("begin");
            let out =
                reasonbraid_server::mcp_listen_internal::record(&mut *tx, tenant, sub, id, cursor)
                    .await;
            match out {
                Ok(accepted) => {
                    tx.commit().await.expect("commit");
                    Ok(accepted)
                }
                Err(e) => Err(e.to_string()),
            }
        }
    };

    // A healthy subscription, and a second one that stays healthy throughout —
    // the positive arm, so a repair that refused everything would fail here.
    assert!(
        record("sub-bad", "v-1", 1).await.unwrap(),
        "the first accepts"
    );
    assert!(
        record("sub-ok", "w-1", 1).await.unwrap(),
        "the control accepts"
    );

    let corrupt = serde_json::json!({ "not": "an array" });
    sqlx::query(
        "UPDATE mcp_listen_state SET dedup_window = $1 \
         WHERE tenant_id = $2 AND subscription_id = $3",
    )
    .bind(&corrupt)
    .bind(tenant)
    .bind("sub-bad")
    .execute(&pool)
    .await
    .expect("corrupt the window");

    // ── FAIL CLOSED ──────────────────────────────────────────────────────────
    let refusal = record("sub-bad", "v-2", 2)
        .await
        .expect_err("a malformed window REFUSES rather than deduplicating nothing");
    assert!(
        refusal.contains("sub-bad"),
        "the refusal names the subscription an operator has to go and look at: {refusal}"
    );

    // ⭐ THE EVIDENCE SURVIVES. The shipped code's UPDATE overwrote the malformed
    // value on its way past, so by the time anyone noticed the duplicates the
    // reason was gone. A refusal that still rewrote the row would pass the
    // assertion above and destroy the same evidence.
    let after: serde_json::Value = sqlx::query_scalar(
        "SELECT dedup_window FROM mcp_listen_state \
         WHERE tenant_id = $1 AND subscription_id = $2",
    )
    .bind(tenant)
    .bind("sub-bad")
    .fetch_one(&pool)
    .await
    .expect("the window after the refusal");
    assert_eq!(
        after, corrupt,
        "the malformed value is UNTOUCHED — the refusal did not erase what an \
         operator needs to diagnose it"
    );

    // ⛔ And the replay that the shipped code accepted is still refused, which is
    // the defect's actual consequence and not merely its mechanism.
    let replay = record("sub-bad", "v-1", 1)
        .await
        .expect_err("the known delivery is not silently re-accepted either");
    assert!(replay.contains("sub-bad"), "{replay}");

    // ── AND THE HEALTHY SUBSCRIPTION IS UNAFFECTED ───────────────────────────
    assert!(
        record("sub-ok", "w-2", 2).await.unwrap(),
        "a well-formed window still accepts a new delivery — the refusal is \
         scoped to the row that is broken, not to the surface"
    );
    assert!(
        !record("sub-ok", "w-1", 1).await.unwrap(),
        "and still deduplicates"
    );
}
