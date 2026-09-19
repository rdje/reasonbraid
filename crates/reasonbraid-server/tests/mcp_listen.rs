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
/// cursor 5 left the stored cursor at **5**. `reconnect` reads that value as
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
        "the cursor is a HIGH-WATER MARK and does not rewind; `reconnect` reads \
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

// ─────────────────────────────────────────────────────────────────────────────
// `SIGNOFF-REPAIR.6.2.4` — the reconnect ritual, over a REAL socket.
//
// The profile is quoted in the leaf from its only two sources, `ROADMAP.md`
// §9.6 and ADR-024, and both name the same five steps: reauthorize → recreate
// the listen request → reconcile any source-specific gap → resume from the OWN
// cursor → surface the possible-gap when the upstream offers no replay.
//
// ⛔ **What this control does NOT qualify.** The upstream below is an ordinary
// HTTP server and the client below is `reqwest`; neither is the rmcp
// Streamable-HTTP transport, which is not a dependency of this workspace
// (`SIGNOFF-REPAIR.6.6` owns that decision and the supply-chain pass it needs).
// What is real here is the SOCKET and the TRUNCATION: the stream is cut
// mid-body by the server, the client observes the transport error, and the
// resume that follows is driven from the durable state rather than from
// anything the dead connection held. That is the claim, and nothing wider.
// ─────────────────────────────────────────────────────────────────────────────

use std::sync::{Arc, Mutex};

use axum::response::IntoResponse as _;
use futures_util::StreamExt as _;
use reasonbraid_server::mcp_listen_internal as listen;

/// What the test upstream observed and what it will answer.
struct UpstreamInner {
    /// Every step the upstream saw, in order — the ritual's own record, written
    /// by the side that cannot be fooled by the client's intentions.
    calls: Vec<String>,
    /// The credential the LAST reauthorization issued. A listen request that
    /// presents anything else is refused, which is what makes step 1
    /// load-bearing rather than decorative.
    token: Option<String>,
    /// `None` → this upstream offers no replay mechanism; `Some(c)` → it
    /// replays from `c` onward and no earlier.
    replay: Option<i64>,
    /// The deliveries the next listen stream emits before it is cut.
    frames: Vec<(String, i64)>,
    /// Whether the next listen stream is truncated mid-body.
    truncate: bool,
    /// The signal that performs the cut.
    ///
    /// ⛔ **The disconnect is triggered by the CONTROL, not by the scheduler,
    /// and the first version of this harness got it wrong.** Emitting the error
    /// straight after the frames let hyper abort the connection before the
    /// client had parsed the response head, so `send()` itself failed with
    /// `connection closed before message completed` and the control never
    /// reached the resume it exists to measure. The cut is still a real
    /// transport abort — the chunked body ends with no terminating chunk — it
    /// simply happens at a point the control names.
    cut: Arc<tokio::sync::Notify>,
}

#[derive(Clone)]
struct Upstream(Arc<Mutex<UpstreamInner>>);

impl Upstream {
    fn new(replay: Option<i64>, frames: Vec<(String, i64)>) -> Self {
        Self(Arc::new(Mutex::new(UpstreamInner {
            calls: Vec::new(),
            token: None,
            replay,
            frames,
            truncate: true,
            cut: Arc::new(tokio::sync::Notify::new()),
        })))
    }

    /// Cut the listen stream that is currently open. `notify_one` stores a
    /// permit, so the order of this call against the server's own progress does
    /// not matter.
    fn cut(&self) {
        let signal = Arc::clone(&self.0.lock().expect("upstream lock").cut);
        signal.notify_one();
    }

    fn calls(&self) -> Vec<String> {
        self.0.lock().expect("upstream lock").calls.clone()
    }

    fn set_replay(&self, replay: Option<i64>) {
        self.0.lock().expect("upstream lock").replay = replay;
    }

    fn set_frames(&self, frames: Vec<(String, i64)>) {
        self.0.lock().expect("upstream lock").frames = frames;
    }

    fn set_truncate(&self, truncate: bool) {
        self.0.lock().expect("upstream lock").truncate = truncate;
    }
}

/// `POST /authorize` — step 1. Every call mints a FRESH credential and
/// invalidates the previous one.
async fn upstream_authorize(
    axum::extract::State(state): axum::extract::State<Upstream>,
) -> axum::Json<serde_json::Value> {
    let token = format!("tok-{}", uuid::Uuid::now_v7());
    {
        let mut inner = state.0.lock().expect("upstream lock");
        inner.calls.push("reauthorize".to_string());
        inner.token = Some(token.clone());
    }
    axum::Json(serde_json::json!({ "token": token }))
}

/// `GET /listen` — step 2. Refuses a credential that is not the current one,
/// records the `resume_from` the client presented, declares what it can replay,
/// and streams the scripted frames before cutting the body.
async fn upstream_listen(
    axum::extract::State(state): axum::extract::State<Upstream>,
    axum::extract::Query(query): axum::extract::Query<std::collections::HashMap<String, String>>,
    headers: axum::http::HeaderMap,
) -> axum::response::Response {
    let presented = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .unwrap_or_default()
        .to_string();
    let (frames, truncate, replay_header, cut) = {
        let mut inner = state.0.lock().expect("upstream lock");
        if inner.token.as_deref() != Some(presented.as_str()) {
            inner.calls.push("listen:refused".to_string());
            return (
                axum::http::StatusCode::UNAUTHORIZED,
                "the listen request did not present the current credential",
            )
                .into_response();
        }
        let subscription = query.get("subscription").cloned().unwrap_or_default();
        let resume_from = query
            .get("resume_from")
            .cloned()
            .unwrap_or_else(|| "none".to_string());
        inner.calls.push(format!(
            "listen sub={subscription} resume_from={resume_from}"
        ));
        let replay_header = match inner.replay {
            None => "none".to_string(),
            Some(cursor) => format!("from:{cursor}"),
        };
        (
            inner.frames.clone(),
            inner.truncate,
            replay_header,
            Arc::clone(&inner.cut),
        )
    };

    // ⛔ The disconnect is REAL and it is produced HERE: the body stream yields
    // its frames and then an ERROR, so hyper abandons the chunked body without
    // its terminating chunk and the client sees a transport failure — not a
    // clean end of stream it could mistake for "the upstream had nothing more".
    let delivered = futures_util::stream::iter(frames.into_iter().map(|(delivery_id, cursor)| {
        Ok(format!("{{\"delivery_id\":\"{delivery_id}\",\"cursor\":{cursor}}}\n").into_bytes())
    }));
    let tail: futures_util::stream::BoxStream<'static, Result<Vec<u8>, std::io::Error>> =
        if truncate {
            Box::pin(futures_util::stream::once(async move {
                cut.notified().await;
                Err(std::io::Error::other(
                    "the upstream cut the listen stream mid-body",
                ))
            }))
        } else {
            Box::pin(futures_util::stream::empty())
        };

    axum::response::Response::builder()
        .status(axum::http::StatusCode::OK)
        .header("x-upstream-replay", replay_header)
        .body(axum::body::Body::from_stream(delivered.chain(tail)))
        .expect("the listen response builds")
}

/// Render a transport failure with its whole cause chain.
///
/// ⛔ `reqwest::Error`'s own `Display` is the top frame only — "error sending
/// request for url (…)" — which names the request and not the fault. A control
/// whose failures are unreadable is a control that gets guessed at.
fn describe(error: &dyn std::error::Error) -> String {
    let mut rendered = error.to_string();
    let mut source = error.source();
    while let Some(cause) = source {
        rendered.push_str(" <- ");
        rendered.push_str(&cause.to_string());
        source = cause.source();
    }
    rendered
}

/// The gateway's side of the seam: a real HTTP client against the upstream
/// above. It lives in the test, not in the shipped crate, because a
/// hand-written request is not the MCP transport profile and this repository
/// does not ship prose that outruns what it has built (`.6.6`).
struct HttpUpstream {
    base: String,
    client: reqwest::Client,
}

impl listen::ListenUpstream for HttpUpstream {
    type Deliveries = std::pin::Pin<
        Box<
            dyn futures_util::Stream<Item = Result<listen::Delivery, listen::UpstreamError>> + Send,
        >,
    >;

    fn reauthorize(
        &self,
    ) -> impl std::future::Future<
        Output = Result<listen::UpstreamAuthorization, listen::UpstreamError>,
    > + Send {
        let url = format!("{}/authorize", self.base);
        let client = self.client.clone();
        async move {
            let response = client
                .post(&url)
                .send()
                .await
                .map_err(|e| listen::UpstreamError::Reauthorization(describe(&e)))?;
            let status = response.status();
            if !status.is_success() {
                return Err(listen::UpstreamError::Reauthorization(format!(
                    "the upstream answered {status}"
                )));
            }
            let body: serde_json::Value = response
                .json()
                .await
                .map_err(|e| listen::UpstreamError::Reauthorization(e.to_string()))?;
            let token = body["token"].as_str().ok_or_else(|| {
                listen::UpstreamError::Reauthorization(
                    "the upstream returned no credential".to_string(),
                )
            })?;
            Ok(listen::UpstreamAuthorization::new(token.to_string()))
        }
    }

    fn recreate_listen(
        &self,
        authorization: &listen::UpstreamAuthorization,
        subscription_id: &str,
        resume_from: Option<i64>,
    ) -> impl std::future::Future<
        Output = Result<listen::UpstreamListen<Self::Deliveries>, listen::UpstreamError>,
    > + Send {
        let url = format!("{}/listen", self.base);
        let client = self.client.clone();
        let credential = authorization.credential().to_string();
        let subscription_id = subscription_id.to_string();
        async move {
            let mut request = client
                .get(&url)
                .query(&[("subscription", subscription_id.as_str())])
                .bearer_auth(credential);
            if let Some(cursor) = resume_from {
                request = request.query(&[("resume_from", cursor.to_string())]);
            }
            let response = request
                .send()
                .await
                .map_err(|e| listen::UpstreamError::Listen(describe(&e)))?;
            let status = response.status();
            if !status.is_success() {
                return Err(listen::UpstreamError::Listen(format!(
                    "the upstream answered {status}"
                )));
            }
            let replay = match response
                .headers()
                .get("x-upstream-replay")
                .and_then(|v| v.to_str().ok())
            {
                Some("none") | None => listen::UpstreamReplay::None,
                Some(declared) => {
                    let cursor = declared.strip_prefix("from:").and_then(|c| c.parse().ok());
                    match cursor {
                        Some(cursor) => listen::UpstreamReplay::From(cursor),
                        // ⛔ An undecodable declaration is NOT read as a replay.
                        // The rule both sources state is that the continuation
                        // is never advertised as stronger than the upstream can
                        // prove, and an answer we cannot parse proves nothing.
                        None => listen::UpstreamReplay::None,
                    }
                }
            };
            // ⛔ Framed on NEWLINES over a buffer, not one delivery per chunk.
            // A chunk boundary is a transport artefact: two frames can arrive
            // coalesced and one frame can arrive split, and a decoder that
            // assumed otherwise would pass this control on this machine and
            // drop deliveries on a slower one.
            let bytes = Box::pin(response.bytes_stream());
            let deliveries = futures_util::stream::unfold(
                (bytes, Vec::<u8>::new(), false),
                |(mut bytes, mut buffered, finished)| async move {
                    if finished {
                        return None;
                    }
                    loop {
                        if let Some(end) = buffered.iter().position(|byte| *byte == b'\n') {
                            let line: Vec<u8> = buffered.drain(..=end).collect();
                            let line = &line[..line.len() - 1];
                            if line.is_empty() {
                                continue;
                            }
                            let decoded = serde_json::from_slice::<serde_json::Value>(line)
                                .map_err(|e| listen::UpstreamError::Listen(e.to_string()))
                                .map(|frame| listen::Delivery {
                                    delivery_id: frame["delivery_id"]
                                        .as_str()
                                        .unwrap_or_default()
                                        .to_string(),
                                    cursor: frame["cursor"].as_i64().unwrap_or_default(),
                                });
                            return Some((decoded, (bytes, buffered, false)));
                        }
                        match bytes.next().await {
                            Some(Ok(chunk)) => buffered.extend_from_slice(chunk.as_ref()),
                            // The truncation: the transport failed mid-body, and
                            // it is reported as a delivery error rather than as
                            // a clean end of stream.
                            Some(Err(e)) => {
                                let refusal = listen::UpstreamError::Listen(e.to_string());
                                return Some((Err(refusal), (bytes, buffered, true)));
                            }
                            None => return None,
                        }
                    }
                },
            );
            Ok(listen::UpstreamListen {
                replay,
                deliveries: Box::pin(deliveries) as Self::Deliveries,
            })
        }
    }
}

/// Bind the test upstream on an ephemeral loopback port and serve it.
async fn serve_upstream(state: Upstream) -> String {
    let router = axum::Router::new()
        .route("/authorize", axum::routing::post(upstream_authorize))
        .route("/listen", axum::routing::get(upstream_listen))
        .with_state(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind the upstream");
    let address = listener.local_addr().expect("the upstream address");
    tokio::spawn(async move {
        axum::serve(listener, router).await.expect("serve upstream");
    });
    format!("http://{address}")
}

/// Drain a resumed stream into the durable state, recording each delivery the
/// way a caller must: one transaction per delivery, so the state commits WITH
/// whatever effects that caller applies. Returns the accepted ids and the
/// transport failure that ended the stream, if there was one.
async fn drain(
    pool: &sqlx::PgPool,
    tenant: &str,
    subscription: &str,
    mut deliveries: std::pin::Pin<
        Box<
            dyn futures_util::Stream<Item = Result<listen::Delivery, listen::UpstreamError>> + Send,
        >,
    >,
) -> (Vec<String>, Option<String>) {
    let mut accepted = Vec::new();
    let mut failure = None;
    while let Some(next) = deliveries.next().await {
        match next {
            Ok(delivery) => {
                let mut tx = pool.begin().await.expect("begin");
                let is_new = listen::record(
                    &mut *tx,
                    tenant,
                    subscription,
                    &delivery.delivery_id,
                    delivery.cursor,
                )
                .await
                .expect("record the delivery");
                tx.commit().await.expect("commit");
                if is_new {
                    accepted.push(delivery.delivery_id);
                }
            }
            Err(e) => {
                failure = Some(e.to_string());
                break;
            }
        }
    }
    (accepted, failure)
}

/// `SIGNOFF-REPAIR.6.2.4` — the five-step reconnect ritual, driven across a
/// REAL disconnect, with the possible-gap flag exercised in BOTH directions.
///
/// ⚠️ **A build, not a repair, so there is no RED to show.** What replaces it
/// is that every clause of the leaf's acceptance is asserted against something
/// the gateway cannot fake: the upstream writes its own record of which steps
/// it saw and in what order, the credential it demands changes on every
/// reauthorization, and the resume point it receives is compared against the
/// durable row rather than against anything the client remembered.
#[tokio::test]
async fn the_reconnect_ritual_resumes_from_the_durable_cursor_across_a_real_disconnect() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let tenant = "ten_aaaaaaaa-bbbb-4ccc-8ddd-eeeeeeeeeeee";
    sqlx::query("INSERT INTO tenants (tenant_id) VALUES ($1)")
        .bind(tenant)
        .execute(&pool)
        .await
        .expect("seed the tenant");
    let subscription = "sub-reconnect";

    // An upstream with NO replay mechanism — the profile's literal possible-gap
    // condition — offering two deliveries before it cuts the stream.
    let upstream_state = Upstream::new(None, vec![("d-10".into(), 10), ("d-11".into(), 11)]);
    let base = serve_upstream(upstream_state.clone()).await;
    let gateway = HttpUpstream {
        base,
        client: reqwest::Client::new(),
    };

    // ── THE FIRST CONNECT, and the disconnect that ends it ───────────────────
    let first = listen::reconnect(&pool, &gateway, tenant, subscription)
        .await
        .expect("the first connect completes the ritual");
    assert_eq!(
        first.plan.resume_from, None,
        "nothing has been accepted yet, so there is no own cursor to resume from"
    );
    assert!(
        first.plan.possible_gap,
        "a subscription that has accepted nothing claims no continuity"
    );
    upstream_state.cut();
    let (accepted, failure) = drain(&pool, tenant, subscription, first.deliveries).await;
    assert_eq!(accepted, vec!["d-10".to_string(), "d-11".to_string()]);
    let failure = failure.expect(
        "the stream ended in a TRANSPORT FAILURE, not a clean end — if this is None the \
         upstream finished its body and the control is not exercising a disconnect at all",
    );
    assert!(
        !failure.is_empty(),
        "the truncation surfaced as a delivery error: {failure}"
    );

    // The durable state survived the connection that produced it.
    let (cursor, last) = listen::state(&pool, tenant, subscription)
        .await
        .expect("read the durable state")
        .expect("the row exists");
    assert_eq!(cursor, 11, "the cursor is the last ACCEPTED delivery's");
    assert_eq!(last.as_deref(), Some("d-11"));

    // ── ARM A — the reconnect, with an upstream that offers no replay ────────
    let arm_a = listen::reconnect(&pool, &gateway, tenant, subscription)
        .await
        .expect("the reconnect completes the ritual");
    assert_eq!(
        arm_a.plan.resume_from,
        Some(11),
        "step 4: the resume point is the OWN cursor, read from the durable row"
    );
    assert_eq!(arm_a.plan.upstream_replay, listen::UpstreamReplay::None);
    assert!(
        arm_a.plan.possible_gap,
        "step 5: no replay mechanism upstream → the possible-gap SURFACES"
    );
    drop(arm_a.deliveries);

    // ⭐ THE UPSTREAM'S OWN RECORD OF THE RITUAL — the order, and the fact that
    // the recreated request carried the DURABLE cursor rather than restarting.
    let calls = upstream_state.calls();
    assert_eq!(
        calls,
        vec![
            "reauthorize".to_string(),
            format!("listen sub={subscription} resume_from=none"),
            "reauthorize".to_string(),
            format!("listen sub={subscription} resume_from=11"),
        ],
        "the ritual's steps, in the profile's order, as the upstream saw them"
    );

    // ── ARM B — a replay that REACHES our cursor closes the gap ──────────────
    // ⛔ Without this arm a `possible_gap` hard-coded to `true` would pass every
    // assertion above, and the flag would carry no information at all.
    upstream_state.set_replay(Some(0));
    let arm_b = listen::reconnect(&pool, &gateway, tenant, subscription)
        .await
        .expect("the reconnect completes");
    assert_eq!(arm_b.plan.resume_from, Some(11));
    assert_eq!(arm_b.plan.upstream_replay, listen::UpstreamReplay::From(0));
    assert!(
        !arm_b.plan.possible_gap,
        "a replay reaching back below our cursor leaves nothing unproven"
    );
    drop(arm_b.deliveries);

    // ── ARM C — a replay whose FLOOR is above our cursor still gaps ──────────
    // 🔴 The arm the old `resume_plan(own_cursor, upstream_replay: bool)` could
    // not express: this upstream answers *yes, I replay*, and cannot produce
    // deliveries 12 through 49. A boolean read that as continuity.
    upstream_state.set_replay(Some(50));
    let arm_c = listen::reconnect(&pool, &gateway, tenant, subscription)
        .await
        .expect("the reconnect completes");
    assert_eq!(arm_c.plan.resume_from, Some(11));
    assert_eq!(arm_c.plan.upstream_replay, listen::UpstreamReplay::From(50));
    assert!(
        arm_c.plan.possible_gap,
        "the upstream offers a replay it cannot reach back far enough to honour, \
         so the continuation is NOT advertised as stronger than it can prove"
    );
    drop(arm_c.deliveries);

    // ── THE RESUMED STREAM ACTUALLY DELIVERS, and the dedup still holds ──────
    // A ritual that returns a plan over a dead stream has reconnected to
    // nothing. This arm lets the upstream finish its body and replays `d-11`
    // ahead of a new delivery, so the resumed stream is proved to carry traffic
    // AND the durable dedup is proved to survive the reconnect.
    upstream_state.set_truncate(false);
    upstream_state.set_frames(vec![("d-11".into(), 11), ("d-12".into(), 12)]);
    let resumed = listen::reconnect(&pool, &gateway, tenant, subscription)
        .await
        .expect("the reconnect completes");
    let (accepted, failure) = drain(&pool, tenant, subscription, resumed.deliveries).await;
    assert_eq!(
        failure, None,
        "this stream ends cleanly — the truncation is the upstream's choice, not the client's"
    );
    assert_eq!(
        accepted,
        vec!["d-12".to_string()],
        "the replayed `d-11` is the dedup SKIP and only the new delivery is accepted"
    );
    let (cursor, last) = listen::state(&pool, tenant, subscription)
        .await
        .expect("read")
        .expect("the row exists");
    assert_eq!(cursor, 12);
    assert_eq!(last.as_deref(), Some("d-12"));
}

/// `SIGNOFF-REPAIR.6.2.4` — step 1 is LOAD-BEARING, and the control proves it
/// by showing what the upstream does to a listen request that skipped it.
///
/// ⛔ Without this the ritual above would pass identically against an upstream
/// that ignored the credential entirely, and `reauthorize` would be a step the
/// control watched happen rather than a step anything depended on
/// (`docs/knowledge/a-control-that-passes-for-an-unrelated-reason.md`).
#[tokio::test]
async fn a_listen_request_that_skips_the_reauthorization_is_refused_upstream() {
    let upstream_state = Upstream::new(None, vec![("d-1".into(), 1)]);
    let base = serve_upstream(upstream_state.clone()).await;
    let gateway = HttpUpstream {
        base,
        client: reqwest::Client::new(),
    };

    // A credential from a reauthorization that a LATER one has superseded — the
    // shape a gateway would present if it cached the token and reconnected
    // without step 1.
    let stale = listen::ListenUpstream::reauthorize(&gateway)
        .await
        .expect("the first reauthorization issues a credential");
    let fresh = listen::ListenUpstream::reauthorize(&gateway)
        .await
        .expect("the second reauthorization issues a different credential");
    assert_ne!(
        stale.credential(),
        fresh.credential(),
        "each reauthorization mints a NEW credential, or the arm below proves nothing"
    );

    let refused = listen::ListenUpstream::recreate_listen(&gateway, &stale, "sub-stale", Some(4))
        .await
        .err()
        .expect("the upstream refuses a listen request carrying a superseded credential");
    assert!(
        matches!(refused, listen::UpstreamError::Listen(ref what) if what.contains("401")),
        "the refusal is the upstream's, and it names the status: {refused}"
    );
    assert_eq!(
        upstream_state.calls().last().map(String::as_str),
        Some("listen:refused"),
        "the upstream recorded the refusal, so the step is enforced there and not \
         merely asserted here"
    );

    // And the SAME request with the current credential is admitted, so the
    // refusal above is about the credential and not about the request's shape.
    listen::ListenUpstream::recreate_listen(&gateway, &fresh, "sub-stale", Some(4))
        .await
        .expect("the current credential is admitted");
}
