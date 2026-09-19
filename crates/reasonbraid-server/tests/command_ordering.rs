//! Control-API admissions and commands against authority changes
//! (`SIGNOFF-REPAIR.3.3.4.4`, `.3.3.4.4.1`, `.3.3.4.6`).
//!
//! `.3.3.4.2` built the tenant guard and `.3.3.4.3.x` integrated it into the
//! standalone authority writers, so an issuance and a revocation now serialize
//! against each other. The THREAD COMMAND path was left out of that
//! integration: `run_thread_command` opens a plain transaction, claims the
//! idempotency key, and authorizes on `Utc::now()` — the process clock — with no
//! guard anywhere. `apply_authorized_command` does the same.
//!
//! The consequence is not a race that has to be caught in the act. It is an
//! ORDERING that simply does not exist, and a held guard makes that visible
//! deterministically: an exclusive tenant guard is exactly what a revocation
//! holds, so a command that runs to completion while one is held is a command
//! that cannot be ordered against revocation at all.
//!
//! These controls therefore hold the guard from the test and observe the
//! command, rather than spawning two racers and hoping for an interleaving.
#![cfg(any(target_os = "linux", target_os = "macos"))]

#[path = "support/mod.rs"]
mod pg_test_support;

#[path = "support/cleanup.rs"]
mod pg_cleanup;

// The PRODUCTION guard module, compiled into this test so the control takes the
// same lock the server takes rather than a copy of its SQL. This control uses a
// subset of it, and unused items in this compilation unit are not dead code in
// the crate — `acquire_in_tx` and `database_now_in_tx` are both called from
// `api.rs` and `authority.rs`.
#[allow(dead_code)]
#[path = "../src/authority/transaction.rs"]
mod tenant_transaction;

use std::net::SocketAddr;
use std::time::Duration;

use reasonbraid_core::TenantId;
use reasonbraid_core::PROTOCOL_VERSION;
use reasonbraid_server::{api_router, PRINCIPAL_HEADER};
use serde_json::{json, Value};
use sqlx::PgPool;
use tenant_transaction::{transact, GuardError, GuardMode};
use tokio::sync::oneshot;
use tokio::time::timeout;

async fn pool() -> Option<PgPool> {
    let pool = pg_test_support::pool().await?;
    sqlx::migrate!("../../migrations")
        .run(&pool)
        .await
        .expect("apply migrations");
    pg_cleanup::delete_tables(
        &pool,
        &[
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
            "node_enroll_audit",
            "node_keys",
            "node_certificates",
            "server_ca",
            "node_leases",
            "runs",
            "incarnations",
            "node_proof_nonces",
            "nodes",
            "hosts",
            "profile_versions",
            "agent_profiles",
            "recruitment_panels",
            "recruitment_responses",
            "recruitment_offers",
            "recruitment_calls",
            "agent_roles",
            "human_principals",
            "evidence_citations",
            "claim_assessments",
            "derivations",
            "evidence_snapshots",
            "reference_registrations",
            "resource_references",
            "quota_events",
            "usage_quotas",
            "federation_agreements",
            "cross_domain_receipts",
            "mcp_listen_state",
            "tenant_bootstrap_requests",
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
            "idempotency",
            "event_log",
            "aggregate_state",
        ],
    )
    .await
    .expect("purge checked fixture plan");
    Some(pool)
}

async fn serve(pool: &PgPool) -> SocketAddr {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind the control API");
    let addr = listener.local_addr().expect("the listener address");
    let router = api_router(pool.clone());
    tokio::spawn(async move {
        axum::serve(listener, router).await.expect("serve");
    });
    addr
}

/// Hold one tenant guard until released, reporting when it is actually held so
/// the control never races its own fixture.
struct Holder {
    release: oneshot::Sender<()>,
    job: tokio::task::JoinHandle<()>,
}

async fn hold(pool: &PgPool, tenant: TenantId, mode: GuardMode) -> Holder {
    let pool = pool.clone();
    let (entered_tx, entered) = oneshot::channel();
    let (release, release_rx) = oneshot::channel();
    let job = tokio::spawn(async move {
        transact(&pool, &[(tenant, mode)], move |_| {
            Box::pin(async move {
                let _ = entered_tx.send(());
                let _ = release_rx.await;
                Ok::<(), GuardError>(())
            })
        })
        .await
        .expect("the holder's guarded transaction completes");
    });
    timeout(Duration::from_secs(5), entered)
        .await
        .expect("the holder acquires its guard")
        .expect("the holder reports entry");
    Holder { release, job }
}

impl Holder {
    async fn release(self) {
        let _ = self.release.send(());
        timeout(Duration::from_secs(5), self.job)
            .await
            .expect("the holder finishes")
            .expect("the holder's task joins");
    }
}

/// The command envelope the control API actually accepts. A control that posts a
/// bare body is refused at deserialization and "completes" without reaching any
/// lock at all — which looks exactly like the defect while proving nothing. The
/// event count is what exposed that, and it is asserted below for the same reason.
fn envelope(operation: &str, key: &str, body: Value) -> Value {
    json!({
        "protocol_version": PROTOCOL_VERSION,
        "operation": operation,
        "request_id": format!("req_{}", uuid::Uuid::now_v7()),
        "idempotency_key": key,
        "expected_aggregate_version": null,
        "body": body,
        "authority_context": null,
        "client_context": {},
    })
}

async fn post(client: &reqwest::Client, url: String, principal: &str, body: Value) -> (u16, Value) {
    let response = client
        .post(url)
        .header(PRINCIPAL_HEADER, principal)
        .json(&body)
        .send()
        .await
        .expect("request");
    let status = response.status().as_u16();
    let text = response.text().await.expect("body");
    (
        status,
        serde_json::from_str(&text).unwrap_or_else(|_| json!({ "raw": text })),
    )
}

/// A thread command must not complete while an exclusive tenant guard is held.
///
/// An exclusive guard is what a revocation takes. A command that completes
/// anyway has no ordering against revocation — not a narrow race window, but no
/// ordering at all — which is what `.3.3.4.4` exists to establish.
///
/// Against the unrepaired command path this control FAILS by succeeding: the
/// thread is created, with its event and audit record, while the guard is held.
#[tokio::test(flavor = "multi_thread")]
async fn a_thread_command_waits_for_an_exclusive_tenant_guard() {
    let Some(pool) = pool().await else { return };
    let addr = serve(&pool).await;
    let client = reqwest::Client::new();
    let base = format!("http://{addr}");

    let (status, human) = post(
        &client,
        format!("{base}/v1/enrollments"),
        "unused",
        json!({ "kind": "human", "name": "orderer" }),
    )
    .await;
    assert_eq!(status, 200, "the human enrolls: {human}");
    let principal = human["principal_id"].as_str().unwrap().to_string();
    let tenant_text = human["tenant_id"].as_str().unwrap().to_string();
    let tenant: TenantId = tenant_text.parse().expect("the tenant id parses");

    let before: i64 = sqlx::query_scalar("SELECT count(*) FROM event_log WHERE tenant_id = $1")
        .bind(&tenant_text)
        .fetch_one(&pool)
        .await
        .expect("count events before");

    let holder = hold(&pool, tenant, GuardMode::Exclusive).await;

    let command = {
        let client = client.clone();
        let base = base.clone();
        let principal = principal.clone();
        let tenant_text = tenant_text.clone();
        tokio::spawn(async move {
            post(
                &client,
                format!("{base}/v1/threads"),
                &principal,
                envelope(
                    "thread.create",
                    "order-control-1",
                    json!({
                        "tenant_id": tenant_text,
                        "subject": "ordered against authority",
                        "objective": "prove the command waits",
                    }),
                ),
            )
            .await
        })
    };

    // The command must still be in flight. If it has finished, it never took the
    // guard — and the count below says what it did while unordered.
    let settled = timeout(Duration::from_secs(3), async {
        loop {
            if command.is_finished() {
                return true;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    })
    .await
    .unwrap_or(false);

    let during: i64 = sqlx::query_scalar("SELECT count(*) FROM event_log WHERE tenant_id = $1")
        .bind(&tenant_text)
        .fetch_one(&pool)
        .await
        .expect("count events during");

    assert!(
        !settled,
        "the thread command completed while an exclusive tenant guard was held: \
         it takes no guard, so it has no ordering against revocation at all \
         (event rows {before} -> {during})"
    );
    assert_eq!(
        before,
        during,
        "the command wrote {} event row(s) while the guard was held",
        during - before
    );

    holder.release().await;

    let (status, body) = timeout(Duration::from_secs(10), command)
        .await
        .expect("the command completes once the guard is released")
        .expect("the command task joins");
    assert_eq!(
        status, 200,
        "the command succeeds normally after the wait: {body}"
    );
    let after: i64 = sqlx::query_scalar("SELECT count(*) FROM event_log WHERE tenant_id = $1")
        .bind(&tenant_text)
        .fetch_one(&pool)
        .await
        .expect("count events after");
    assert!(
        after > during,
        "the released command produced its effect: {during} -> {after}"
    );
}

/// A SHARED guard must not block a command, and an unrelated tenant must never
/// be affected — the bound that keeps the ordering from becoming a global lock.
#[tokio::test(flavor = "multi_thread")]
async fn a_shared_guard_and_an_unrelated_tenant_do_not_block_a_command() {
    let Some(pool) = pool().await else { return };
    let addr = serve(&pool).await;
    let client = reqwest::Client::new();
    let base = format!("http://{addr}");

    let (_, human) = post(
        &client,
        format!("{base}/v1/enrollments"),
        "unused",
        json!({ "kind": "human", "name": "sharer" }),
    )
    .await;
    let principal = human["principal_id"].as_str().unwrap().to_string();
    let tenant_text = human["tenant_id"].as_str().unwrap().to_string();
    let tenant: TenantId = tenant_text.parse().expect("the tenant id parses");

    // Another tenant's guard, held exclusively, is irrelevant to this command.
    let stranger = hold(&pool, TenantId::new(), GuardMode::Exclusive).await;
    // This tenant's guard, held SHARED, is compatible with a command's own read.
    let shared = hold(&pool, tenant, GuardMode::Shared).await;

    let (status, body) = timeout(
        Duration::from_secs(10),
        post(
            &client,
            format!("{base}/v1/threads"),
            &principal,
            envelope(
                "thread.create",
                "order-control-2",
                json!({
                    "tenant_id": tenant_text,
                    "subject": "compatible",
                    "objective": "a shared guard is not an obstacle",
                }),
            ),
        ),
    )
    .await
    .expect("a shared guard and a stranger's guard do not block the command");
    assert_eq!(status, 200, "the command proceeds: {body}");

    shared.release().await;
    stranger.release().await;
}

/// Authority that ends WHILE a command waits must be evaluated after the wait,
/// not as it stood when the request arrived (`SIGNOFF-REPAIR.3.3.4.4.1`).
///
/// The authority chapter publishes this as a contract row. It follows from the
/// decision time being sampled after the guard and the idempotency claim have
/// both waited — and until this control existed it was argued rather than
/// measured, which is the gap `.3.3.4.5` found while building the equivalent
/// control for the node-result path.
///
/// Expiry is time passing rather than an operation, so the fixture makes it
/// pass: the grant's window is ended underneath the blocked command. What the
/// control separates is the WAIT — remove `acquire_in_tx` from
/// `run_thread_command` and the command completes before the authority changes
/// at all, which is exactly how this behaved before `.3.3.4.4`. It does not
/// separate `clock_timestamp()` from a process clock read at the same point;
/// both are after the wait, and the honest claim is the one this asserts.
#[tokio::test(flavor = "multi_thread")]
async fn a_grant_that_ends_while_a_command_waits_is_evaluated_after_the_wait() {
    let Some(pool) = pool().await else { return };
    let addr = serve(&pool).await;
    let client = reqwest::Client::new();
    let base = format!("http://{addr}");

    let (status, human) = post(
        &client,
        format!("{base}/v1/enrollments"),
        "unused",
        json!({ "kind": "human", "name": "expirer" }),
    )
    .await;
    assert_eq!(status, 200, "the human enrolls: {human}");
    let principal = human["principal_id"].as_str().unwrap().to_string();
    let tenant_text = human["tenant_id"].as_str().unwrap().to_string();
    let tenant: TenantId = tenant_text.parse().expect("the tenant id parses");

    let before: i64 = sqlx::query_scalar("SELECT count(*) FROM event_log WHERE tenant_id = $1")
        .bind(&tenant_text)
        .fetch_one(&pool)
        .await
        .expect("count events before");

    let holder = hold(&pool, tenant, GuardMode::Exclusive).await;

    let command = {
        let client = client.clone();
        let base = base.clone();
        let principal = principal.clone();
        let tenant_text = tenant_text.clone();
        tokio::spawn(async move {
            post(
                &client,
                format!("{base}/v1/threads"),
                &principal,
                envelope(
                    "thread.create",
                    "order-control-3",
                    json!({
                        "tenant_id": tenant_text,
                        "subject": "authority ends underneath",
                        "objective": "prove the post-wait evaluation",
                    }),
                ),
            )
            .await
        })
    };

    // The command is waiting for the guard; end the caller's authority under it.
    tokio::time::sleep(Duration::from_millis(500)).await;
    let ended = sqlx::query(
        "UPDATE authority_grants SET expires_at = now() - interval '1 second' \
         WHERE tenant_id = $1 AND subject_id = $2",
    )
    .bind(&tenant_text)
    .bind(&principal)
    .execute(&pool)
    .await
    .expect("end the human's grant")
    .rows_affected();
    assert_eq!(ended, 1, "exactly one grant was ended");

    holder.release().await;

    let (status, body) = timeout(Duration::from_secs(10), command)
        .await
        .expect("the command completes once the guard is released")
        .expect("the command task joins");
    assert_eq!(
        status, 403,
        "authority that ended while the command waited must refuse it: {body}"
    );
    let after: i64 = sqlx::query_scalar("SELECT count(*) FROM event_log WHERE tenant_id = $1")
        .bind(&tenant_text)
        .fetch_one(&pool)
        .await
        .expect("count events after");
    assert_eq!(
        before,
        after,
        "the refused command wrote {} event row(s)",
        after - before
    );
}

async fn get(client: &reqwest::Client, url: String, principal: &str) -> (u16, Value) {
    let response = client
        .get(url)
        .header(PRINCIPAL_HEADER, principal)
        .send()
        .await
        .expect("request");
    let status = response.status().as_u16();
    let text = response.text().await.expect("body");
    (
        status,
        serde_json::from_str(&text).unwrap_or_else(|_| json!({ "raw": text })),
    )
}

/// One enrolled human and its tenant — the fixture every admission control below
/// shares.
async fn enrolled_human(
    client: &reqwest::Client,
    base: &str,
    name: &str,
) -> (String, String, TenantId) {
    let (status, human) = post(
        client,
        format!("{base}/v1/enrollments"),
        "unused",
        json!({ "kind": "human", "name": name }),
    )
    .await;
    assert_eq!(status, 200, "the human enrolls: {human}");
    let principal = human["principal_id"].as_str().unwrap().to_string();
    let tenant_text = human["tenant_id"].as_str().unwrap().to_string();
    let tenant: TenantId = tenant_text.parse().expect("the tenant id parses");
    (principal, tenant_text, tenant)
}

/// Wait, bounded, for a spawned request to finish; `false` means it is still in
/// flight, which is what an ordering against a held guard looks like.
async fn settles_within(job: &tokio::task::JoinHandle<(u16, Value)>, budget: Duration) -> bool {
    timeout(budget, async {
        loop {
            if job.is_finished() {
                return true;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    })
    .await
    .unwrap_or(false)
}

/// A named inspection admission must not complete while an EXCLUSIVE tenant
/// guard is held (`SIGNOFF-REPAIR.3.3.4.6`).
///
/// The eight frozen-tenant reads COMMIT an admission record before returning
/// protected data. Committing it while a revocation holds the tenant's exclusive
/// guard means the evidence in that record — the selected parent's status, the
/// grant's scope — was read with no ordering against the writer changing them.
///
/// Against the unrepaired path this control FAILS by succeeding.
#[tokio::test(flavor = "multi_thread")]
async fn an_inspection_admission_waits_for_an_exclusive_tenant_guard() {
    let Some(pool) = pool().await else { return };
    let addr = serve(&pool).await;
    let client = reqwest::Client::new();
    let base = format!("http://{addr}");
    let (principal, tenant_text, tenant) = enrolled_human(&client, &base, "inspector").await;

    let before: i64 =
        sqlx::query_scalar("SELECT count(*) FROM authorization_records WHERE tenant_id = $1")
            .bind(&tenant_text)
            .fetch_one(&pool)
            .await
            .expect("count admissions before");

    let holder = hold(&pool, tenant, GuardMode::Exclusive).await;

    let read = {
        let client = client.clone();
        let base = base.clone();
        let principal = principal.clone();
        let tenant_text = tenant_text.clone();
        tokio::spawn(async move {
            get(
                &client,
                format!("{base}/v1/admin/grants?tenant_id={tenant_text}"),
                &principal,
            )
            .await
        })
    };

    let settled = settles_within(&read, Duration::from_secs(3)).await;
    let during: i64 =
        sqlx::query_scalar("SELECT count(*) FROM authorization_records WHERE tenant_id = $1")
            .bind(&tenant_text)
            .fetch_one(&pool)
            .await
            .expect("count admissions during");
    assert!(
        !settled,
        "the inspection admission committed while an exclusive tenant guard was \
         held: its authority evidence has no ordering against the writer that \
         changes it (admission rows {before} -> {during})"
    );
    assert_eq!(
        before, during,
        "the read committed an admission record while the guard was held"
    );

    holder.release().await;

    let (status, body) = timeout(Duration::from_secs(10), read)
        .await
        .expect("the read completes once the guard is released")
        .expect("the read task joins");
    assert_eq!(status, 200, "the read succeeds after the wait: {body}");
}

/// Authority that ends while an inspection waits must be evaluated after the
/// wait — the same post-wait rule the command path holds.
#[tokio::test(flavor = "multi_thread")]
async fn authority_ended_while_an_inspection_waits_is_evaluated_after_the_wait() {
    let Some(pool) = pool().await else { return };
    let addr = serve(&pool).await;
    let client = reqwest::Client::new();
    let base = format!("http://{addr}");
    let (principal, tenant_text, tenant) =
        enrolled_human(&client, &base, "expiring-inspector").await;

    let holder = hold(&pool, tenant, GuardMode::Exclusive).await;

    let read = {
        let client = client.clone();
        let base = base.clone();
        let principal = principal.clone();
        let tenant_text = tenant_text.clone();
        tokio::spawn(async move {
            get(
                &client,
                format!("{base}/v1/admin/boundaries?tenant_id={tenant_text}"),
                &principal,
            )
            .await
        })
    };

    tokio::time::sleep(Duration::from_millis(500)).await;
    let ended = sqlx::query(
        "UPDATE authority_grants SET expires_at = now() - interval '1 second' \
         WHERE tenant_id = $1 AND subject_id = $2",
    )
    .bind(&tenant_text)
    .bind(&principal)
    .execute(&pool)
    .await
    .expect("end the administrator's grant")
    .rows_affected();
    assert_eq!(ended, 1, "exactly one grant was ended");

    holder.release().await;

    let (status, body) = timeout(Duration::from_secs(10), read)
        .await
        .expect("the read completes once the guard is released")
        .expect("the read task joins");
    assert_eq!(
        status, 403,
        "authority that ended while the read waited must refuse it: {body}"
    );
}

/// The frozen-boundary exception must SURVIVE the ordering: a boundary revoked
/// while the read waits still admits its otherwise-eligible administrator,
/// because the exception ignores boundary status by design.
///
/// This is the bound that keeps the guard from turning an approved carve-out
/// into a refusal, and it is the reason the mode is Shared rather than a fence
/// on every read.
#[tokio::test(flavor = "multi_thread")]
async fn a_boundary_frozen_while_an_inspection_waits_still_admits_it() {
    let Some(pool) = pool().await else { return };
    let addr = serve(&pool).await;
    let client = reqwest::Client::new();
    let base = format!("http://{addr}");
    let (principal, tenant_text, tenant) = enrolled_human(&client, &base, "frozen-inspector").await;

    let holder = hold(&pool, tenant, GuardMode::Exclusive).await;

    let read = {
        let client = client.clone();
        let base = base.clone();
        let principal = principal.clone();
        let tenant_text = tenant_text.clone();
        tokio::spawn(async move {
            get(
                &client,
                format!("{base}/v1/admin/grants?tenant_id={tenant_text}"),
                &principal,
            )
            .await
        })
    };

    tokio::time::sleep(Duration::from_millis(500)).await;
    let frozen =
        sqlx::query("UPDATE enrollment_boundaries SET status = 'revoked' WHERE tenant_id = $1")
            .bind(&tenant_text)
            .execute(&pool)
            .await
            .expect("freeze the boundary")
            .rows_affected();
    assert_eq!(frozen, 1, "exactly one boundary was frozen");

    holder.release().await;

    let (status, body) = timeout(Duration::from_secs(10), read)
        .await
        .expect("the read completes once the guard is released")
        .expect("the read task joins");
    assert_eq!(
        status, 200,
        "the frozen-boundary exception must still admit its administrator: {body}"
    );
}

/// The ordinary (non-exception) standalone read admission takes the guard too:
/// the thread inspection path authorizes through `authorize`, not through the
/// frozen-tenant evaluator, and its audit record is evidence in the same way.
#[tokio::test(flavor = "multi_thread")]
async fn a_thread_inspection_admission_waits_for_an_exclusive_tenant_guard() {
    let Some(pool) = pool().await else { return };
    let addr = serve(&pool).await;
    let client = reqwest::Client::new();
    let base = format!("http://{addr}");
    let (principal, tenant_text, tenant) = enrolled_human(&client, &base, "thread-inspector").await;

    let holder = hold(&pool, tenant, GuardMode::Exclusive).await;

    let read = {
        let client = client.clone();
        let base = base.clone();
        let principal = principal.clone();
        let tenant_text = tenant_text.clone();
        tokio::spawn(async move {
            get(
                &client,
                format!("{base}/v1/threads?tenant_id={tenant_text}"),
                &principal,
            )
            .await
        })
    };

    let settled = settles_within(&read, Duration::from_secs(3)).await;
    assert!(
        !settled,
        "the thread inspection admission committed while an exclusive tenant \
         guard was held: it authorizes through the unguarded standalone entry \
         point, so its audit evidence has no ordering against an authority writer"
    );

    holder.release().await;

    let (status, body) = timeout(Duration::from_secs(10), read)
        .await
        .expect("the read completes once the guard is released")
        .expect("the read task joins");
    assert_eq!(status, 200, "the read succeeds after the wait: {body}");
}
