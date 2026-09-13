//! Node results against authority changes (`SIGNOFF-REPAIR.3.3.4.5`).
//!
//! `.3.3.4.4` ordered the THREAD COMMAND path against a tenant authority change:
//! `run_thread_command` takes the tenant guard before its idempotency claim, and
//! a revocation's exclusive guard therefore fences every command that has not
//! passed that point. The NODE RESULT path folds a node-emitted `work_result`
//! into its thread through the same claim → authorize → validate → apply flow,
//! and was left out of that integration: `node_channel::events` opens a plain
//! transaction, locks the node's LEASE row, writes the receipt, and applies the
//! domain effect with no tenant guard anywhere.
//!
//! The consequence is not a race that has to be caught in the act. It is an
//! ORDERING that does not exist, and a held guard makes that visible
//! deterministically: an exclusive tenant guard is exactly what a revocation
//! holds, so a node result that runs to completion while one is held is a result
//! that cannot be ordered against revocation at all.
//!
//! These controls therefore hold a lock from the test and observe the handler,
//! rather than spawning racers and hoping for an interleaving.
//!
//! One control is about the same transaction for a different reason: the handler
//! discards the application's error and commits regardless. A SQL failure inside
//! the application aborts the transaction, so that COMMIT is executed as a
//! ROLLBACK — and the handler still answers `accepted: true` for a receipt that
//! was never written. That is the exact failure the guard runner's own
//! `SELECT 1` probe exists to prevent, in a transaction that has no such probe.
#![cfg(any(target_os = "linux", target_os = "macos"))]

#[path = "support/mod.rs"]
mod pg_test_support;

#[path = "support/cleanup.rs"]
mod pg_cleanup;

// The PRODUCTION guard module, compiled into this test so the controls take the
// same lock the server takes rather than a copy of its SQL. This control uses a
// subset of it, and unused items in this compilation unit are not dead code in
// the crate — `acquire_in_tx` and `database_now_in_tx` are both called from
// `api.rs` and `authority.rs`.
#[allow(dead_code)]
#[path = "../src/authority/transaction.rs"]
mod tenant_transaction;

use std::net::SocketAddr;
use std::sync::{Arc, OnceLock};
use std::time::Duration;

use reasonbraid_core::TenantId;
use reasonbraid_core::{CommandEnvelope, RequestId, PROTOCOL_VERSION};
use reasonbraid_server::{
    api_router, ca::ensure_server_ca, node_router, CHANNEL_VERSION, PRINCIPAL_HEADER,
};
use serde_json::{json, Value};
use sqlx::PgPool;
use tenant_transaction::{transact, GuardError, GuardMode};
use tokio::sync::oneshot;
use tokio::time::timeout;

/// The dev signing secret this suite's nodes enroll with (the dev trust-store
/// stance; the `.1.2.2` handshake proves possession of the issued leaf's key).
const DEV_SECRET: &str = "dev-secret";

/// This suite owns the thread/channel/authority/budget tables for its duration.
static SUITE_LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();

async fn suite_guard() -> tokio::sync::MutexGuard<'static, ()> {
    SUITE_LOCK
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
    pg_cleanup::delete_tables(
        &pool,
        &[
            "outbox_delivery",
            "outbox",
            "node_events",
            "node_inbox",
            "node_leases",
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
            "claim_assessments",
            "derivations",
            "evidence_snapshots",
            "resource_references",
            "quota_events",
            "usage_quotas",
            "federation_agreements",
            "cross_domain_receipts",
            "mcp_listen_state",
            "tenant_bootstrap_requests",
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

/// The running server half: control API and node channel on one listener.
struct TestServer {
    addr: SocketAddr,
    _handle: tokio::task::JoinHandle<()>,
}

impl TestServer {
    async fn start(pool: &PgPool) -> Self {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind ephemeral loopback port");
        let addr = listener.local_addr().expect("the listener address");
        let ca = Arc::new(ensure_server_ca(pool).await.expect("server CA"));
        let router = api_router(pool.clone()).merge(node_router(pool.clone(), ca));
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

fn envelope(operation: &str, key: &str, body: Value) -> CommandEnvelope {
    CommandEnvelope {
        protocol_version: PROTOCOL_VERSION.to_string(),
        operation: operation.to_string(),
        request_id: RequestId::new(),
        idempotency_key: key.to_string(),
        expected_aggregate_version: None,
        body,
        authority_context: None,
        client_context: Default::default(),
    }
}

async fn enroll(client: &reqwest::Client, base: &str, body: Value) -> Value {
    let response = client
        .post(format!("{base}/v1/enrollments"))
        .json(&body)
        .send()
        .await
        .expect("enroll request");
    assert_eq!(response.status().as_u16(), 200, "the enrollment succeeds");
    response.json().await.expect("enroll json")
}

async fn command(
    client: &reqwest::Client,
    base: &str,
    path: &str,
    principal: &str,
    env: &CommandEnvelope,
) -> (u16, Value) {
    let response = client
        .post(format!("{base}{path}"))
        .header(PRINCIPAL_HEADER, principal)
        .json(env)
        .send()
        .await
        .expect("command request");
    let status = response.status().as_u16();
    (status, response.json().await.expect("command json"))
}

fn from_hex(s: &str) -> Vec<u8> {
    assert!(s.len().is_multiple_of(2), "hex is even-length");
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).expect("hex digit"))
        .collect()
}

/// Enroll a node through the PUBLIC surface (`.1.2.1`): this fixture's human
/// administrator issues a one-time token and the node consumes it with its dev
/// secret. ⚠️ A human is what THIS fixture uses, not what the route requires —
/// issuance gates on the `TenantAdmin` grant, which an agent role may also hold
/// (`SIGNOFF-REPAIR.4.1.4`). The dev
/// wiring collapses node==role: the node id IS the role wire id.
async fn enroll_node(
    client: &reqwest::Client,
    base: &str,
    human: &str,
    tenant: &str,
    node_id: &str,
) -> (String, String) {
    let response = client
        .post(format!("{base}/v1/nodes/enroll-tokens"))
        .header(PRINCIPAL_HEADER, human)
        .json(&json!({
            "tenant_id": tenant,
            "node_id": node_id,
            "host_claim": "seed-host",
        }))
        .send()
        .await
        .expect("issue-token request");
    assert_eq!(response.status().as_u16(), 200, "the token issues");
    let issued: Value = response.json().await.expect("issue-token json");

    let response = client
        .post(format!("{base}/v1/nodes/enroll"))
        .json(&json!({
            "token_id": issued["token_id"],
            "node_id": node_id,
            "host_claim": "seed-host",
            "nonce": issued["nonce"],
            "key_secret": DEV_SECRET,
        }))
        .send()
        .await
        .expect("enroll request");
    assert_eq!(response.status().as_u16(), 200, "the node enrolls");
    let body: Value = response.json().await.expect("enroll json");
    (
        body["cert_der"].as_str().unwrap().to_string(),
        body["key_der"].as_str().unwrap().to_string(),
    )
}

/// The authenticated channel path (`.1.2.2`): the node proves its leaf's key over
/// the reported fields and receives its inbox replay plus a fresh lease.
async fn handshake(
    client: &reqwest::Client,
    base: &str,
    node_id: &str,
    cert_hex: &str,
    key_hex: &str,
) -> (Value, String, i64) {
    let der = rustls_pki_types::PrivateKeyDer::try_from(from_hex(key_hex)).expect("key DER");
    let key = rcgen::KeyPair::from_der_and_sign_algo(&der, &rcgen::PKCS_ECDSA_P256_SHA256)
        .expect("key parses");
    let nonce = reasonbraid_node::fresh_proof_nonce();
    let proof =
        reasonbraid_node::compute_cert_proof(&key, CHANNEL_VERSION, node_id, 0, &[], &[], &nonce);
    let response = client
        .post(format!("{base}/v1/nodes/handshake"))
        .json(&json!({
            "channel_version": CHANNEL_VERSION,
            "node_id": node_id,
            "last_acked_cursor": 0,
            "pending_operations": [],
            "ambiguous_attempts": [],
            "cert_der": cert_hex,
            "proof_signature": proof,
            "nonce": nonce,
        }))
        .send()
        .await
        .expect("handshake request");
    assert_eq!(response.status().as_u16(), 200, "handshake succeeds");
    let body: Value = response.json().await.expect("handshake json");
    let token = body["fencing_token"].as_str().expect("token").to_string();
    let epoch = body["lease_epoch"].as_i64().expect("epoch");
    (body, token, epoch)
}

#[allow(clippy::too_many_arguments)]
async fn submit_event(
    client: &reqwest::Client,
    base: &str,
    node_id: &str,
    fencing_token: &str,
    lease_epoch: i64,
    event_id: &str,
    operation_id: &str,
    payload: &Value,
) -> (u16, Value) {
    let response = client
        .post(format!("{base}/v1/nodes/events"))
        .json(&json!({
            "channel_version": CHANNEL_VERSION,
            "node_id": node_id,
            "event_id": event_id,
            "operation_id": operation_id,
            "payload": payload,
            "fencing_token": fencing_token,
            "lease_epoch": lease_epoch,
        }))
        .send()
        .await
        .expect("event request");
    let status = response.status().as_u16();
    let text = response.text().await.expect("event body");
    (
        status,
        serde_json::from_str(&text).unwrap_or_else(|_| json!({ "raw": text })),
    )
}

/// A `work_result` payload as the node emits it (`.6.2` contract).
fn work_result(command_id: &str, content: &str, reservation_id: &str, attempt_id: &str) -> Value {
    json!({
        "kind": "work_result",
        "work_kind": "contribute",
        "command_id": command_id,
        "reservation_id": reservation_id,
        "attempt_id": attempt_id,
        "content": content,
        "usage": { "input_tokens": 41, "output_tokens": 17 },
    })
}

/// One tenant, one role/node, one thread with an ACCEPTED invitation — so the
/// role's inbox holds exactly one dispatched work item with its reservation.
struct Fixture {
    tenant: String,
    tenant_id: TenantId,
    human: String,
    role: String,
    cert_hex: String,
    key_hex: String,
}

async fn fixture(client: &reqwest::Client, base: &str) -> Fixture {
    let human = enroll(
        client,
        base,
        json!({ "kind": "human", "name": "organizer" }),
    )
    .await;
    let tenant = human["tenant_id"].as_str().unwrap().to_string();
    let human_id = human["principal_id"].as_str().unwrap().to_string();

    let role = enroll(
        client,
        base,
        json!({ "kind": "role", "name": "agent-a", "tenant_id": tenant }),
    )
    .await;
    let role_id = role["principal_id"].as_str().unwrap().to_string();

    let (cert_hex, key_hex) = enroll_node(client, base, &human_id, &tenant, &role_id).await;

    let (status, created) = command(
        client,
        base,
        "/v1/threads",
        &human_id,
        &envelope(
            "thread.create",
            "key-create",
            json!({
                "tenant_id": tenant,
                "subject": "ordered node results",
                "objective": "prove the fold waits for authority",
            }),
        ),
    )
    .await;
    assert_eq!(status, 200, "thread creates: {created}");
    let thread = created["thread_id"].as_str().unwrap().to_string();

    let (status, invited) = command(
        client,
        base,
        &format!("/v1/threads/{thread}/commands"),
        &human_id,
        &envelope(
            "thread.invite",
            "key-invite",
            json!({ "tenant_id": tenant, "agent_role": role_id }),
        ),
    )
    .await;
    assert_eq!(status, 200, "invite succeeds: {invited}");

    let (status, accepted) = command(
        client,
        base,
        &format!("/v1/threads/{thread}/commands"),
        &role_id,
        &envelope(
            "thread.accept_invitation",
            "key-accept",
            json!({ "tenant_id": tenant }),
        ),
    )
    .await;
    assert_eq!(
        status, 200,
        "the role accepts, dispatching work: {accepted}"
    );

    Fixture {
        tenant_id: tenant.parse().expect("the tenant id parses"),
        tenant,
        human: human_id,
        role: role_id,
        cert_hex,
        key_hex,
    }
}

/// The dispatched work item's command id and reservation, read through the
/// PUBLIC channel surface (the node's own inbox view), plus a live lease.
async fn work_item(
    client: &reqwest::Client,
    base: &str,
    f: &Fixture,
) -> (String, String, String, i64) {
    let (body, token, epoch) = handshake(client, base, &f.role, &f.cert_hex, &f.key_hex).await;
    let replay = body["replay"].as_array().expect("replay array");
    assert_eq!(replay.len(), 1, "exactly one work item was dispatched");
    let command_id = replay[0]["command_id"].as_str().unwrap().to_string();
    let reservation = replay[0]["payload"]["reservation"]["reservation_id"]
        .as_str()
        .unwrap()
        .to_string();
    (command_id, reservation, token, epoch)
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

async fn contribution_count(pool: &PgPool, tenant: &str) -> i64 {
    sqlx::query_scalar(
        "SELECT count(*) FROM event_log \
         WHERE tenant_id = $1 AND event_type = 'thread.contribution_submitted'",
    )
    .bind(tenant)
    .fetch_one(pool)
    .await
    .expect("count contributions")
}

async fn receipt_count(pool: &PgPool, node_id: &str) -> i64 {
    sqlx::query_scalar("SELECT count(*) FROM node_events WHERE node_id = $1")
        .bind(node_id)
        .fetch_one(pool)
        .await
        .expect("count receipts")
}

/// Wait, bounded, until PostgreSQL itself reports a backend BLOCKED on a lock
/// whose query text matches — an observed wait, never an elapsed-sleep guess.
async fn wait_for_blocked(pool: &PgPool, fragment: &str) {
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    loop {
        let blocked: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM pg_stat_activity \
             WHERE wait_event_type = 'Lock' AND query LIKE $1",
        )
        .bind(format!("%{fragment}%"))
        .fetch_one(pool)
        .await
        .expect("read pg_stat_activity");
        if blocked > 0 {
            return;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "no backend is blocked on a lock for a query matching `{fragment}`"
        );
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

/// A node result must not complete while an EXCLUSIVE tenant guard is held.
///
/// An exclusive guard is what a revocation takes. A node result that completes
/// anyway has no ordering against revocation — not a narrow race window, but no
/// ordering at all.
///
/// Against the unrepaired path this control FAILS by succeeding: the receipt and
/// the contribution are both written while the guard is held.
#[tokio::test(flavor = "multi_thread")]
async fn a_node_result_waits_for_an_exclusive_tenant_guard() {
    let _g = suite_guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let client = reqwest::Client::new();
    let base = server.base();
    let f = fixture(&client, &base).await;
    let (command_id, reservation, token, epoch) = work_item(&client, &base, &f).await;

    let before_receipts = receipt_count(&pool, &f.role).await;
    let before_contributions = contribution_count(&pool, &f.tenant).await;

    let holder = hold(&pool, f.tenant_id, GuardMode::Exclusive).await;

    let result = work_result(
        &command_id,
        "the agent's independent answer",
        &reservation,
        "att_00000000-0000-7000-8000-000000000001",
    );
    let submission = {
        let client = client.clone();
        let base = base.clone();
        let role = f.role.clone();
        let token = token.clone();
        tokio::spawn(async move {
            submit_event(
                &client,
                &base,
                &role,
                &token,
                epoch,
                "evt_00000000-0000-7000-8000-000000000001",
                "op_00000000-0000-7000-8000-000000000001",
                &result,
            )
            .await
        })
    };

    // The submission must still be in flight. If it has finished, it never took
    // the guard — and the counts below say what it did while unordered.
    let settled = timeout(Duration::from_secs(3), async {
        loop {
            if submission.is_finished() {
                return true;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    })
    .await
    .unwrap_or(false);

    let during_receipts = receipt_count(&pool, &f.role).await;
    let during_contributions = contribution_count(&pool, &f.tenant).await;

    assert!(
        !settled,
        "the node result completed while an exclusive tenant guard was held: it \
         takes no guard, so it has no ordering against revocation at all \
         (receipts {before_receipts} -> {during_receipts}, contributions \
         {before_contributions} -> {during_contributions})"
    );
    assert_eq!(
        before_receipts, during_receipts,
        "the handler wrote a receipt while the guard was held"
    );
    assert_eq!(
        before_contributions, during_contributions,
        "the handler folded a contribution while the guard was held"
    );

    holder.release().await;

    let (status, receipt) = timeout(Duration::from_secs(10), submission)
        .await
        .expect("the submission completes once the guard is released")
        .expect("the submission task joins");
    assert_eq!(
        status, 200,
        "the result is accepted after the wait: {receipt}"
    );
    assert_eq!(receipt["accepted"], json!(true), "{receipt}");
    assert_eq!(
        contribution_count(&pool, &f.tenant).await,
        before_contributions + 1,
        "the released result produced exactly one contribution"
    );
}

/// The tenant guard is taken BEFORE the node's lease row — the selected order,
/// observed rather than argued.
///
/// The control holds the LEASE row, so the handler is blocked at the lease
/// whatever the order is. What separates the two orders is whether the handler
/// is holding the tenant guard while it waits: with the guard first, a
/// concurrent EXCLUSIVE acquisition of the same tenant must block behind it;
/// with the lease first, that acquisition finds the guard free and completes.
///
/// Against the unrepaired path this control FAILS: the guard is free.
#[tokio::test(flavor = "multi_thread")]
async fn the_tenant_guard_is_taken_before_the_lease_lock() {
    let _g = suite_guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let client = reqwest::Client::new();
    let base = server.base();
    let f = fixture(&client, &base).await;
    let (command_id, reservation, token, epoch) = work_item(&client, &base, &f).await;

    // Hold the node's lease row, so the handler blocks at `verify_fencing_in_tx`.
    let mut lease_tx = pool.begin().await.expect("begin the lease holder");
    let held: String =
        sqlx::query_scalar("SELECT node_id FROM node_leases WHERE node_id = $1 FOR UPDATE")
            .bind(&f.role)
            .fetch_one(&mut *lease_tx)
            .await
            .expect("hold the node's lease row");
    assert_eq!(held, f.role);

    let result = work_result(
        &command_id,
        "the answer that must wait",
        &reservation,
        "att_00000000-0000-7000-8000-000000000002",
    );
    let submission = {
        let client = client.clone();
        let base = base.clone();
        let role = f.role.clone();
        let token = token.clone();
        tokio::spawn(async move {
            submit_event(
                &client,
                &base,
                &role,
                &token,
                epoch,
                "evt_00000000-0000-7000-8000-000000000002",
                "op_00000000-0000-7000-8000-000000000002",
                &result,
            )
            .await
        })
    };

    // The handler is genuinely waiting on the lease row, observed in the server's
    // own lock view rather than assumed after a sleep.
    wait_for_blocked(&pool, "FROM node_leases").await;

    // Now: does it hold the tenant guard while it waits there?
    let probe = {
        let pool = pool.clone();
        let tenant = f.tenant_id;
        tokio::spawn(async move {
            transact(&pool, &[(tenant, GuardMode::Exclusive)], |_| {
                Box::pin(async { Ok::<(), GuardError>(()) })
            })
            .await
        })
    };
    let acquired = timeout(Duration::from_secs(2), async {
        loop {
            if probe.is_finished() {
                return true;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    })
    .await
    .unwrap_or(false);
    assert!(
        !acquired,
        "an exclusive tenant guard was acquired while the node-result handler \
         waited on the node's lease row: the handler reached the lease WITHOUT \
         the tenant guard, which is the inversion this leaf exists to remove"
    );

    // Release the lease: the handler finishes, then the probe gets the guard.
    lease_tx.rollback().await.expect("release the lease row");
    let (status, receipt) = timeout(Duration::from_secs(10), submission)
        .await
        .expect("the submission completes once the lease is released")
        .expect("the submission task joins");
    assert_eq!(status, 200, "the result is accepted: {receipt}");
    timeout(Duration::from_secs(10), probe)
        .await
        .expect("the probe acquires once the handler commits")
        .expect("the probe task joins")
        .expect("the probe's guarded transaction completes");
}

/// A SHARED guard and an unrelated tenant's exclusive guard must not block a
/// node result — the bound that keeps the ordering from becoming a global lock.
#[tokio::test(flavor = "multi_thread")]
async fn a_shared_guard_and_an_unrelated_tenant_do_not_block_a_node_result() {
    let _g = suite_guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let client = reqwest::Client::new();
    let base = server.base();
    let f = fixture(&client, &base).await;
    let (command_id, reservation, token, epoch) = work_item(&client, &base, &f).await;

    let stranger = hold(&pool, TenantId::new(), GuardMode::Exclusive).await;
    let shared = hold(&pool, f.tenant_id, GuardMode::Shared).await;

    let result = work_result(
        &command_id,
        "compatible with a shared reader",
        &reservation,
        "att_00000000-0000-7000-8000-000000000003",
    );
    let (status, receipt) = timeout(
        Duration::from_secs(10),
        submit_event(
            &client,
            &base,
            &f.role,
            &token,
            epoch,
            "evt_00000000-0000-7000-8000-000000000003",
            "op_00000000-0000-7000-8000-000000000003",
            &result,
        ),
    )
    .await
    .expect("a shared guard and a stranger's guard do not block the node result");
    assert_eq!(status, 200, "the result is accepted: {receipt}");
    assert_eq!(
        contribution_count(&pool, &f.tenant).await,
        1,
        "the result folded normally"
    );

    shared.release().await;
    stranger.release().await;
}

/// Authority that is revoked WHILE the result waits must be evaluated as it
/// stands after the wait, not as it stood when the request arrived.
///
/// The grant is expired under the held guard — expiry is time passing, not an
/// operation, so the fixture makes it pass — and the result then has to see it.
/// Against the unrepaired path this control FAILS: the result never waits, so it
/// commits its contribution before the authority changes at all.
#[tokio::test(flavor = "multi_thread")]
async fn authority_changed_while_the_result_waits_is_evaluated_after_the_wait() {
    let _g = suite_guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let client = reqwest::Client::new();
    let base = server.base();
    let f = fixture(&client, &base).await;
    let (command_id, reservation, token, epoch) = work_item(&client, &base, &f).await;

    let holder = hold(&pool, f.tenant_id, GuardMode::Exclusive).await;

    let result = work_result(
        &command_id,
        "an answer whose authority expires first",
        &reservation,
        "att_00000000-0000-7000-8000-000000000004",
    );
    let submission = {
        let client = client.clone();
        let base = base.clone();
        let role = f.role.clone();
        let token = token.clone();
        tokio::spawn(async move {
            submit_event(
                &client,
                &base,
                &role,
                &token,
                epoch,
                "evt_00000000-0000-7000-8000-000000000004",
                "op_00000000-0000-7000-8000-000000000004",
                &result,
            )
            .await
        })
    };

    // The result is waiting for the guard; end the role's authority underneath it.
    tokio::time::sleep(Duration::from_millis(500)).await;
    let ended = sqlx::query(
        "UPDATE authority_grants SET expires_at = now() - interval '1 second' \
         WHERE tenant_id = $1 AND subject_id = $2",
    )
    .bind(&f.tenant)
    .bind(&f.role)
    .execute(&pool)
    .await
    .expect("expire the role's grant")
    .rows_affected();
    assert_eq!(ended, 1, "exactly one role grant was expired");

    holder.release().await;

    let (status, receipt) = timeout(Duration::from_secs(10), submission)
        .await
        .expect("the submission completes once the guard is released")
        .expect("the submission task joins");
    assert_eq!(status, 200, "the receipt still commits: {receipt}");
    assert_eq!(
        receipt["accepted"],
        json!(true),
        "the node DID emit this event: {receipt}"
    );
    assert_eq!(
        contribution_count(&pool, &f.tenant).await,
        0,
        "authority that ended while the result waited must refuse the fold"
    );
}

/// Authority revoked BEFORE the result arrives cannot authorize the fold — the
/// preserved half of the ordering, kept as a regression guard.
#[tokio::test(flavor = "multi_thread")]
async fn a_revoked_grant_refuses_a_later_node_result() {
    let _g = suite_guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let client = reqwest::Client::new();
    let base = server.base();
    let f = fixture(&client, &base).await;
    let (command_id, reservation, token, epoch) = work_item(&client, &base, &f).await;

    let grant: String =
        sqlx::query_scalar("SELECT grant_id FROM authority_grants WHERE subject_id = $1")
            .bind(&f.role)
            .fetch_one(&pool)
            .await
            .expect("the role's grant");
    let response = client
        .post(format!("{base}/v1/admin/grants/{grant}/revoke"))
        .header(PRINCIPAL_HEADER, &f.human)
        .json(&json!({ "tenant_id": f.tenant, "reason": "role retired" }))
        .send()
        .await
        .expect("revoke request");
    assert_eq!(
        response.status().as_u16(),
        200,
        "the administrator revokes the role's grant"
    );

    let result = work_result(
        &command_id,
        "an answer from revoked authority",
        &reservation,
        "att_00000000-0000-7000-8000-000000000005",
    );
    let (status, receipt) = submit_event(
        &client,
        &base,
        &f.role,
        &token,
        epoch,
        "evt_00000000-0000-7000-8000-000000000005",
        "op_00000000-0000-7000-8000-000000000005",
        &result,
    )
    .await;
    assert_eq!(status, 200, "the receipt still commits: {receipt}");
    assert_eq!(receipt["accepted"], json!(true), "{receipt}");
    assert_eq!(
        contribution_count(&pool, &f.tenant).await,
        0,
        "revoked authority cannot authorize a later thread effect"
    );
    assert_eq!(
        receipt_count(&pool, &f.role).await,
        1,
        "the node DID emit the event, so its receipt is durable"
    );
}

/// A storage failure inside the fold must not be reported as an accepted receipt.
///
/// The handler discards the application's error and commits anyway. A SQL failure
/// aborts the transaction, so that COMMIT runs as a ROLLBACK: nothing is written,
/// and the node is told `accepted: true` for a receipt that does not exist. It
/// will therefore never re-emit, and the result is lost silently.
///
/// The fault is injected as a trigger on `event_log`, reverted at the end, so
/// production bytes are unchanged while a real SQL error occurs at a real write.
#[tokio::test(flavor = "multi_thread")]
async fn a_storage_failure_in_the_fold_is_not_reported_as_an_accepted_receipt() {
    let _g = suite_guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let client = reqwest::Client::new();
    let base = server.base();
    let f = fixture(&client, &base).await;
    let (command_id, reservation, token, epoch) = work_item(&client, &base, &f).await;

    sqlx::query(
        "CREATE OR REPLACE FUNCTION rb_test_reject_marker() RETURNS trigger AS $$ \
         BEGIN \
           IF NEW.body::text LIKE '%RB-INJECTED-STORAGE-FAULT%' THEN \
             RAISE EXCEPTION 'injected storage fault'; \
           END IF; \
           RETURN NEW; \
         END $$ LANGUAGE plpgsql",
    )
    .execute(&pool)
    .await
    .expect("create the fault function");
    sqlx::query(
        "CREATE TRIGGER rb_test_reject_marker BEFORE INSERT ON event_log \
         FOR EACH ROW EXECUTE FUNCTION rb_test_reject_marker()",
    )
    .execute(&pool)
    .await
    .expect("install the fault trigger");

    let result = work_result(
        &command_id,
        "RB-INJECTED-STORAGE-FAULT",
        &reservation,
        "att_00000000-0000-7000-8000-000000000006",
    );
    let (status, receipt) = submit_event(
        &client,
        &base,
        &f.role,
        &token,
        epoch,
        "evt_00000000-0000-7000-8000-000000000006",
        "op_00000000-0000-7000-8000-000000000006",
        &result,
    )
    .await;

    let receipts = receipt_count(&pool, &f.role).await;
    let contributions = contribution_count(&pool, &f.tenant).await;

    // Revert the injection before asserting, so a failure cannot leave the fault
    // installed for the next control in this suite.
    sqlx::query("DROP TRIGGER rb_test_reject_marker ON event_log")
        .execute(&pool)
        .await
        .expect("remove the fault trigger");
    sqlx::query("DROP FUNCTION rb_test_reject_marker()")
        .execute(&pool)
        .await
        .expect("remove the fault function");

    assert_eq!(
        contributions, 0,
        "the injected fault stopped the fold, as intended"
    );
    assert_eq!(
        receipts, 0,
        "the aborted transaction's COMMIT ran as a ROLLBACK, so no receipt exists"
    );
    assert_ne!(
        status, 200,
        "the handler answered success for a receipt that was rolled back \
         (status {status}, body {receipt}): the node will never re-emit it"
    );
}
