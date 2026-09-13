//! WP6 node-wiring integration tests (`PHASE-0.6.2`): the two-host demo's contract
//! over a live PostgreSQL — an accepted invite dispatches a work item (with a
//! reservation against the thread ceiling) into the invited role's inbox; a
//! node-emitted `work_result` folds into the thread through the SAME
//! claim → authorize → validate → apply flow a CLI command rides; duplicate
//! transport produces one domain effect; a challenge dispatches revise work; a
//! budget denial enqueues the work item WITHOUT a reservation (the node's budget
//! gate refuses the dispatch); and a result arriving after close is stored as the
//! work command's idempotent rejection.
//!
//! All channel traffic uses the PUBLIC surface (`handshake` reads the inbox —
//! inspection never needs database surgery); direct SQL is assertion-only.
//!
//! Run with `scripts/run_pg_tests.sh` locally or the `pg-tests` CI job. Without
//! `DATABASE_URL` these skip, so `make check` stays green offline.

#[path = "support/mod.rs"]
mod pg_test_support;

#[path = "support/cleanup.rs"]
mod pg_cleanup;

use std::net::SocketAddr;
use std::sync::{Arc, OnceLock};

use reasonbraid_core::{CommandEnvelope, RequestId, PROTOCOL_VERSION};
use reasonbraid_server::{
    api_router, ca::ensure_server_ca, node_router, CHANNEL_VERSION, PRINCIPAL_HEADER,
};
use serde_json::{json, Value};
use sqlx::PgPool;

/// The dev signing secret every node in this suite enrolls with (the dev
/// trust-store stance; the `.1.2.2` handshake proves possession of it).
const DEV_SECRET: &str = "dev-secret";

/// The wiring tests own the thread/channel/authority/budget tables: tests never
/// run concurrently against the same PG database.
static WIRING_LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();

async fn guard() -> tokio::sync::MutexGuard<'static, ()> {
    WIRING_LOCK
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
    // Purge in FK order: this suite exclusively owns these tables for its duration.
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
        let addr = listener.local_addr().unwrap();
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

async fn enroll(client: &reqwest::Client, base: &str, body: Value) -> (u16, Value) {
    let response = client
        .post(format!("{base}/v1/enrollments"))
        .json(&body)
        .send()
        .await
        .expect("enroll request");
    let status = response.status().as_u16();
    (status, response.json().await.expect("enroll json"))
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

async fn get(client: &reqwest::Client, base: &str, path: &str, principal: &str) -> (u16, Value) {
    let response = client
        .get(format!("{base}{path}"))
        .header(PRINCIPAL_HEADER, principal)
        .send()
        .await
        .expect("get request");
    let status = response.status().as_u16();
    (status, response.json().await.expect("get json"))
}

/// The admin revocation verb (`.1.3.2`; the `.1.5.2` test uses it to bump the
/// tenant epoch).
async fn admin_revoke(
    client: &reqwest::Client,
    base: &str,
    path: &str,
    principal: &str,
    tenant: &str,
) -> (u16, Value) {
    let response = client
        .post(format!("{base}{path}"))
        .header(PRINCIPAL_HEADER, principal)
        .json(&json!({ "tenant_id": tenant, "reason": "test revocation" }))
        .send()
        .await
        .expect("revoke request");
    let status = response.status().as_u16();
    (status, response.json().await.expect("revoke json"))
}

/// Enroll a node through the PUBLIC surface (`.1.2.1`): an authorized human
/// issues a one-time token and the node consumes it with its dev secret. The
/// dev wiring collapses node==role: the node id IS the role wire id.
async fn enroll_node(
    client: &reqwest::Client,
    base: &str,
    human: &str,
    tenant: &str,
    node_id: &str,
    secret: &str,
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
            "key_secret": secret,
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

/// The authenticated public channel path (`.1.2.2`): the node reports holding
/// nothing, proves its key over the reported fields, and receives its inbox
/// replay + a fresh lease. Returns the response, the fencing token, and the
/// lease epoch.
async fn handshake(
    client: &reqwest::Client,
    base: &str,
    node_id: &str,
    cert_hex: &str,
    key_hex: &str,
) -> (Value, String, i64) {
    let key_der = from_hex(key_hex).expect("key hex");
    let der = rustls_pki_types::PrivateKeyDer::try_from(key_der).expect("key DER");
    let key = rcgen::KeyPair::from_der_and_sign_algo(&der, &rcgen::PKCS_ECDSA_P256_SHA256)
        .expect("key parses");
    let proof = reasonbraid_node::compute_cert_proof(&key, CHANNEL_VERSION, node_id, 0, &[], &[]);
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
        }))
        .send()
        .await
        .expect("handshake request");
    assert_eq!(response.status().as_u16(), 200, "handshake succeeds");
    let body: Value = response.json().await.expect("handshake json");
    let token = body["fencing_token"]
        .as_str()
        .expect("the handshake returns a fencing token")
        .to_string();
    let epoch = body["lease_epoch"]
        .as_i64()
        .expect("the handshake returns the lease epoch");
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
    (status, response.json().await.expect("event json"))
}

/// The explicit accept (`.1.3.1`): the invited ROLE accepts through the control
/// API — the transaction that dispatches the work item (an accepted invitation
/// exists iff its work does).
async fn accept_invitation(
    client: &reqwest::Client,
    base: &str,
    role: &str,
    thread: &str,
    tenant: &str,
    key: &str,
) {
    let (status, accepted) = command(
        client,
        base,
        &format!("/v1/threads/{thread}/commands"),
        role,
        &envelope(
            "thread.accept_invitation",
            key,
            json!({ "tenant_id": tenant }),
        ),
    )
    .await;
    assert_eq!(status, 200, "the invited role accepts: {accepted}");
    assert_eq!(accepted["event_type"], json!("thread.invitation_accepted"));
}

/// The events of one thread, in order (the API inspection surface).
async fn thread_events(
    client: &reqwest::Client,
    base: &str,
    thread_id: &str,
    tenant: &str,
    principal: &str,
) -> Vec<Value> {
    let (status, body) = get(
        client,
        base,
        &format!("/v1/threads/{thread_id}/events?tenant_id={tenant}"),
        principal,
    )
    .await;
    assert_eq!(status, 200, "events view succeeds");
    body["events"].as_array().expect("events array").clone()
}

/// A full bootstrap: human (tenant owner) + one role, and a created thread.
/// Returns (tenant, human, role, thread).
async fn bootstrap(
    client: &reqwest::Client,
    base: &str,
) -> (String, String, String, String, String, String) {
    let (status, human) = enroll(
        client,
        base,
        json!({ "kind": "human", "name": "organizer" }),
    )
    .await;
    assert_eq!(status, 200, "human enrolls");
    let tenant = human["tenant_id"].as_str().unwrap().to_string();
    let human_id = human["principal_id"].as_str().unwrap().to_string();

    let (status, role) = enroll(
        client,
        base,
        json!({ "kind": "role", "name": "agent-a", "tenant_id": tenant }),
    )
    .await;
    assert_eq!(status, 200, "role enrolls");
    let role_id = role["principal_id"].as_str().unwrap().to_string();

    // The dev wiring's node IS the role: enroll it (`.1.2.1`) so the `.1.2.2`
    // authenticated handshake has a key to verify.
    let (cert_hex, key_hex) =
        enroll_node(client, base, &human_id, &tenant, &role_id, DEV_SECRET).await;

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
                "subject": "two-host demo",
                "objective": "prove delivery survives crashes",
            }),
        ),
    )
    .await;
    assert_eq!(status, 200, "thread creates: {created:?}");
    let thread = created["thread_id"].as_str().unwrap().to_string();
    (tenant, human_id, role_id, thread, cert_hex, key_hex)
}

/// A `work_result` payload as the node emits it (`.6.2` contract).
fn work_result(
    command_id: &str,
    work_kind: &str,
    content: &str,
    reservation_id: &str,
    attempt_id: &str,
) -> Value {
    json!({
        "kind": "work_result",
        "work_kind": work_kind,
        "command_id": command_id,
        "reservation_id": reservation_id,
        "attempt_id": attempt_id,
        "content": content,
        "usage": { "input_tokens": 41, "output_tokens": 17 },
    })
}

/// THE `.6.2` contract: an accepted invite dispatches work into the invited
/// role's inbox in the SAME transaction as the invite event, and the work item
/// carries a reservation against the thread ceiling.
#[tokio::test]
async fn invite_dispatches_work_with_a_reservation() {
    let _g = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let client = reqwest::Client::new();
    let (tenant, human, role, thread, cert_hex, key_hex) = bootstrap(&client, &server.base()).await;

    let (status, _) = command(
        &client,
        &server.base(),
        &format!("/v1/threads/{thread}/commands"),
        &human,
        &envelope(
            "thread.invite",
            "key-invite",
            json!({ "tenant_id": tenant, "agent_role": role }),
        ),
    )
    .await;
    assert_eq!(status, 200, "invite succeeds");

    // THE `.1.3.1` contract: the invite recorded a PENDING invitation and
    // enqueued NOTHING — the work item exists only after the explicit accept.
    let (pre_handshake, _token, _ep1) =
        handshake(&client, &server.base(), &role, &cert_hex, &key_hex).await;
    assert_eq!(
        pre_handshake["replay"].as_array().unwrap().len(),
        0,
        "a pending invitation enqueues no work"
    );
    accept_invitation(
        &client,
        &server.base(),
        &role,
        &thread,
        &tenant,
        "key-accept-invite",
    )
    .await;

    // The node reads its inbox through the PUBLIC channel surface.
    let (handshake, _token, _ep6) =
        handshake(&client, &server.base(), &role, &cert_hex, &key_hex).await;
    let replay = handshake["replay"].as_array().expect("replay array");
    assert_eq!(
        replay.len(),
        1,
        "the invitation produced exactly one work item"
    );
    let work = &replay[0];
    assert_eq!(work["tenant_id"].as_str().unwrap(), tenant);
    assert_eq!(work["thread_id"].as_str().unwrap(), thread);
    assert!(
        work["command_id"]
            .as_str()
            .unwrap()
            .starts_with("work_evt_"),
        "the work command id correlates with its trigger event"
    );
    let payload = &work["payload"];
    assert_eq!(payload["kind"].as_str().unwrap(), "contribute");
    assert_eq!(payload["agent_role"].as_str().unwrap(), role);
    assert!(
        payload["reservation"]["reservation_id"].as_str().is_some(),
        "the work item carries a reservation"
    );
    assert!(
        payload["reservation"]["dimensions"]["calls"]
            .as_u64()
            .unwrap()
            >= 1,
        "the reservation covers a dispatch"
    );

    let active: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM budget_reservations WHERE status = 'active'")
            .fetch_one(&pool)
            .await
            .expect("count active reservations");
    assert_eq!(active, 1, "exactly one active reservation was issued");
}

/// A node-emitted work result folds into the thread as a contribution — and
/// duplicate transport produces exactly ONE domain effect at both layers (the
/// receipt dedupe and the idempotency claim).
#[tokio::test]
async fn node_result_becomes_one_contribution_despite_duplicates() {
    let _g = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let client = reqwest::Client::new();
    let (tenant, human, role, thread, cert_hex, key_hex) = bootstrap(&client, &server.base()).await;

    let (status, _) = command(
        &client,
        &server.base(),
        &format!("/v1/threads/{thread}/commands"),
        &human,
        &envelope(
            "thread.invite",
            "key-invite",
            json!({ "tenant_id": tenant, "agent_role": role }),
        ),
    )
    .await;
    assert_eq!(status, 200, "invite succeeds");
    accept_invitation(
        &client,
        &server.base(),
        &role,
        &thread,
        &tenant,
        "key-accept-result",
    )
    .await;

    let (handshake, token, ep7) =
        handshake(&client, &server.base(), &role, &cert_hex, &key_hex).await;
    let work = handshake["replay"][0].clone();
    let command_id = work["command_id"].as_str().unwrap().to_string();
    let reservation_id = work["payload"]["reservation"]["reservation_id"]
        .as_str()
        .unwrap()
        .to_string();
    let result = work_result(
        &command_id,
        "contribute",
        "the agent's independent answer",
        &reservation_id,
        "att_00000000-0000-7000-8000-000000000001",
    );

    // First delivery: accepted, and the thread gains exactly one contribution.
    let (status, receipt) = submit_event(
        &client,
        &server.base(),
        &role,
        &token,
        ep7,
        "evt_00000000-0000-7000-8000-000000000001",
        "op_00000000-0000-7000-8000-000000000001",
        &result,
    )
    .await;
    assert_eq!(status, 200, "event accepted");
    assert_eq!(receipt["accepted"], json!(true));

    let events = thread_events(&client, &server.base(), &thread, &tenant, &human).await;
    let contributions: Vec<&Value> = events
        .iter()
        .filter(|e| e["event_type"] == "thread.contribution_submitted")
        .collect();
    assert_eq!(contributions.len(), 1, "exactly one contribution exists");
    assert_eq!(
        contributions[0]["body"]["author"].as_str().unwrap(),
        role,
        "the contribution's author is the node's role"
    );
    assert_eq!(
        contributions[0]["body"]["content"].as_str().unwrap(),
        "the agent's independent answer",
        "the adapter content landed verbatim"
    );

    // The reservation settled with the reported usage.
    let settled: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM budget_reservations WHERE reservation_id = $1 AND status = 'settled'",
    )
    .bind(&reservation_id)
    .fetch_one(&pool)
    .await
    .expect("count settled");
    assert_eq!(settled, 1, "the reservation settled with the result");

    // THE `.1.6.2` run writer: the result's attempt id linked the run to the
    // role's CURRENT incarnation, in the same transaction as the fold.
    let run: (String, String, String, String, String) = sqlx::query_as(
        "SELECT r.run_id, r.attempt_id, r.incarnation_id, i.role_id, r.tenant_id \
         FROM runs r JOIN incarnations i ON i.incarnation_id = r.incarnation_id",
    )
    .fetch_one(&pool)
    .await
    .expect("the run row");
    assert!(
        run.0.starts_with("run_"),
        "the run id is branded: {}",
        run.0
    );
    assert_eq!(
        run.1, "att_00000000-0000-7000-8000-000000000001",
        "the run links the result's attempt"
    );
    assert_eq!(run.3, role, "the run links the role's incarnation");
    assert_eq!(run.4, tenant, "the run belongs to the tenant");

    // The inspection surface shows the chain (no database surgery).
    let response = client
        .get(format!(
            "{}/v1/admin/runs?tenant_id={tenant}",
            server.base()
        ))
        .header(PRINCIPAL_HEADER, &human)
        .send()
        .await
        .expect("runs list");
    assert_eq!(response.status().as_u16(), 200);
    let runs: Value = response.json().await.expect("runs json");
    let listed = runs["runs"].as_array().expect("runs array");
    assert_eq!(listed.len(), 1, "exactly one run: {runs}");
    assert_eq!(
        listed[0]["attempt_id"],
        json!("att_00000000-0000-7000-8000-000000000001")
    );
    assert_eq!(listed[0]["role_id"], json!(role));

    // Duplicate transport #1: the SAME event id → the receipt dedupe says no.
    let (status, receipt) = submit_event(
        &client,
        &server.base(),
        &role,
        &token,
        ep7,
        "evt_00000000-0000-7000-8000-000000000001",
        "op_00000000-0000-7000-8000-000000000001",
        &result,
    )
    .await;
    assert_eq!(status, 200, "duplicate event answered");
    assert_eq!(
        receipt["accepted"],
        json!(false),
        "the duplicate was refused"
    );

    // Duplicate transport #2: a NEW event id re-delivering the SAME work result →
    // the idempotency claim replays the original application.
    let (status, receipt) = submit_event(
        &client,
        &server.base(),
        &role,
        &token,
        ep7,
        "evt_00000000-0000-7000-8000-000000000002",
        "op_00000000-0000-7000-8000-000000000002",
        &result,
    )
    .await;
    assert_eq!(status, 200, "redelivered work result answered");
    assert_eq!(
        receipt["accepted"],
        json!(true),
        "the new receipt is recorded"
    );

    let events = thread_events(&client, &server.base(), &thread, &tenant, &human).await;
    let contributions: Vec<&Value> = events
        .iter()
        .filter(|e| e["event_type"] == "thread.contribution_submitted")
        .collect();
    assert_eq!(
        contributions.len(),
        1,
        "the redelivered result produced NO second domain effect"
    );

    // Neither duplicate produced a second run: one result = one run, ever.
    let run_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM runs")
        .fetch_one(&pool)
        .await
        .expect("count runs");
    assert_eq!(run_count, 1, "the duplicates wrote no second run");

    // The audit trail records the role's authorized application.
    let (status, audit) = get(
        &client,
        &server.base(),
        &format!("/v1/threads/{thread}/audit?tenant_id={tenant}"),
        &human,
    )
    .await;
    assert_eq!(status, 200, "audit view succeeds");
    let applied: Vec<&Value> = audit["records"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| r["action"] == "thread_contribute" && r["decision"] == "allowed")
        .collect();
    assert!(!applied.is_empty(), "the node's application is audited");
}

/// A challenge of a role's contribution dispatches revise work to that role's
/// node; the node's revision folds in and answers the challenge.
#[tokio::test]
async fn challenge_dispatches_revise_work_and_the_revision_lands() {
    let _g = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let client = reqwest::Client::new();
    let (tenant, human, role, thread, cert_hex, key_hex) = bootstrap(&client, &server.base()).await;

    let (status, _) = command(
        &client,
        &server.base(),
        &format!("/v1/threads/{thread}/commands"),
        &human,
        &envelope(
            "thread.invite",
            "key-invite",
            json!({ "tenant_id": tenant, "agent_role": role }),
        ),
    )
    .await;
    assert_eq!(status, 200, "invite succeeds");
    accept_invitation(
        &client,
        &server.base(),
        &role,
        &thread,
        &tenant,
        "key-accept-challenge",
    )
    .await;

    let (first_view, first_token, ep4) =
        handshake(&client, &server.base(), &role, &cert_hex, &key_hex).await;
    let command_id = first_view["replay"][0]["command_id"]
        .as_str()
        .unwrap()
        .to_string();
    let reservation_id = first_view["replay"][0]["payload"]["reservation"]["reservation_id"]
        .as_str()
        .unwrap()
        .to_string();
    let (status, _) = submit_event(
        &client,
        &server.base(),
        &role,
        &first_token,
        ep4,
        "evt_00000000-0000-7000-8000-000000000011",
        "op_00000000-0000-7000-8000-000000000011",
        &work_result(
            &command_id,
            "contribute",
            "answer one",
            &reservation_id,
            "att_00000000-0000-7000-8000-000000000011",
        ),
    )
    .await;
    assert_eq!(status, 200, "contribution lands");

    let events = thread_events(&client, &server.base(), &thread, &tenant, &human).await;
    let contribution_id = events
        .iter()
        .find(|e| e["event_type"] == "thread.contribution_submitted")
        .expect("the contribution exists")["event_id"]
        .as_str()
        .unwrap()
        .to_string();

    let (status, _) = command(
        &client,
        &server.base(),
        &format!("/v1/threads/{thread}/commands"),
        &human,
        &envelope(
            "thread.challenge",
            "key-challenge",
            json!({
                "tenant_id": tenant,
                "target_event_id": contribution_id,
                "content": "is this justified?",
            }),
        ),
    )
    .await;
    assert_eq!(status, 200, "challenge succeeds");

    // The challenge dispatched revise work to the contribution's author (the role).
    let (replay_view, replay_token, ep5) =
        handshake(&client, &server.base(), &role, &cert_hex, &key_hex).await;
    let replay = replay_view["replay"].as_array().expect("replay array");
    assert_eq!(replay.len(), 2, "invite work + revise work");
    let revise = replay
        .iter()
        .find(|c| c["payload"]["kind"] == "revise")
        .expect("the revise work item exists");
    let challenge_id = thread_events(&client, &server.base(), &thread, &tenant, &human)
        .await
        .iter()
        .find(|e| e["event_type"] == "thread.challenge_posted")
        .expect("the challenge event exists")["event_id"]
        .as_str()
        .unwrap()
        .to_string();
    assert_eq!(
        revise["payload"]["target_event_id"].as_str().unwrap(),
        challenge_id,
        "the revise work targets the challenge (the domain's revise target)"
    );

    let (status, _) = submit_event(
        &client,
        &server.base(),
        &role,
        &replay_token,
        ep5,
        "evt_00000000-0000-7000-8000-000000000012",
        "op_00000000-0000-7000-8000-000000000012",
        &work_result(
            revise["command_id"].as_str().unwrap(),
            "revise",
            "the revision",
            revise["payload"]["reservation"]["reservation_id"]
                .as_str()
                .unwrap(),
            "att_00000000-0000-7000-8000-000000000012",
        ),
    )
    .await;
    assert_eq!(status, 200, "revision lands");

    let events = thread_events(&client, &server.base(), &thread, &tenant, &human).await;
    assert!(
        events
            .iter()
            .any(|e| e["event_type"] == "thread.revision_submitted"),
        "the revision is in the timeline"
    );
    let (status, body) = get(
        &client,
        &server.base(),
        &format!("/v1/threads/{thread}?tenant_id={tenant}"),
        &human,
    )
    .await;
    assert_eq!(status, 200, "state view succeeds");
    assert_eq!(
        body["state"]["open_challenges"].as_u64().unwrap(),
        0,
        "the revision answered the challenge"
    );
    assert_eq!(body["state"]["revisions"].as_u64().unwrap(), 1);
}

/// A ceiling that cannot cover the work reservation denies the dispatch AT THE
/// INVITE — the work item is still enqueued (without a reservation), so the
/// node's budget gate refuses `failed_before_dispatch` instead of calling a
/// provider.
#[tokio::test]
async fn budget_denial_enqueues_work_without_a_reservation() {
    let _g = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let client = reqwest::Client::new();
    let (tenant, human, role, _thread, cert_hex, key_hex) =
        bootstrap(&client, &server.base()).await;

    let (status, created) = command(
        &client,
        &server.base(),
        "/v1/threads",
        &human,
        &envelope(
            "thread.create",
            "key-create-tight",
            json!({
                "tenant_id": tenant,
                "subject": "tight budget",
                "objective": "deny the dispatch",
                "budget": { "calls": 0 },
            }),
        ),
    )
    .await;
    assert_eq!(status, 200, "the tight thread creates");
    let thread = created["thread_id"].as_str().unwrap().to_string();

    let (status, _) = command(
        &client,
        &server.base(),
        &format!("/v1/threads/{thread}/commands"),
        &human,
        &envelope(
            "thread.invite",
            "key-invite-tight",
            json!({ "tenant_id": tenant, "agent_role": role }),
        ),
    )
    .await;
    assert_eq!(
        status, 200,
        "the invite itself succeeds (dispatch is denied, not the invite)"
    );
    accept_invitation(
        &client,
        &server.base(),
        &role,
        &thread,
        &tenant,
        "key-accept-tight",
    )
    .await;

    let (handshake, _token, _ep6) =
        handshake(&client, &server.base(), &role, &cert_hex, &key_hex).await;
    let work = &handshake["replay"][0];
    assert_eq!(work["payload"]["kind"].as_str().unwrap(), "contribute");
    assert!(
        work["payload"]["reservation"].is_null(),
        "no reservation: the ceiling refused"
    );
    assert!(
        work["payload"]["reservation_reason"]
            .as_str()
            .unwrap()
            .contains("budget"),
        "the denial reason rides with the work item"
    );

    let denied: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM budget_reservations WHERE status = 'denied'")
            .fetch_one(&pool)
            .await
            .expect("count denials");
    assert_eq!(denied, 1, "the denial is itself a recorded row");
}

/// A work result arriving after the thread closed is stored as the work
/// command's idempotent REJECTION — the receipt still stands (the node did emit
/// it), and a re-delivery reproduces the rejection without a domain effect.
#[tokio::test]
async fn result_after_close_is_stored_as_a_rejection() {
    let _g = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let client = reqwest::Client::new();
    let (tenant, human, role, thread, cert_hex, key_hex) = bootstrap(&client, &server.base()).await;

    let (status, _) = command(
        &client,
        &server.base(),
        &format!("/v1/threads/{thread}/commands"),
        &human,
        &envelope(
            "thread.invite",
            "key-invite",
            json!({ "tenant_id": tenant, "agent_role": role }),
        ),
    )
    .await;
    assert_eq!(status, 200, "invite succeeds");
    accept_invitation(
        &client,
        &server.base(),
        &role,
        &thread,
        &tenant,
        "key-accept-close",
    )
    .await;
    let (handshake, token, ep7) =
        handshake(&client, &server.base(), &role, &cert_hex, &key_hex).await;
    let command_id = handshake["replay"][0]["command_id"]
        .as_str()
        .unwrap()
        .to_string();

    let (status, _) = command(
        &client,
        &server.base(),
        &format!("/v1/threads/{thread}/commands"),
        &human,
        &envelope(
            "thread.close",
            "key-close",
            json!({ "tenant_id": tenant, "reason": "time" }),
        ),
    )
    .await;
    assert_eq!(status, 200, "thread closes");

    let result = work_result(
        &command_id,
        "contribute",
        "too late",
        "",
        "att_00000000-0000-7000-8000-000000000021",
    );
    let (status, receipt) = submit_event(
        &client,
        &server.base(),
        &role,
        &token,
        ep7,
        "evt_00000000-0000-7000-8000-000000000021",
        "op_00000000-0000-7000-8000-000000000021",
        &result,
    )
    .await;
    assert_eq!(status, 200, "the receipt stands");
    assert_eq!(receipt["accepted"], json!(true));

    // The rejection is stored: a re-delivery (new event id, same work) replays it.
    let (status, receipt) = submit_event(
        &client,
        &server.base(),
        &role,
        &token,
        ep7,
        "evt_00000000-0000-7000-8000-000000000022",
        "op_00000000-0000-7000-8000-000000000022",
        &result,
    )
    .await;
    assert_eq!(status, 200, "redelivery answered");
    assert_eq!(receipt["accepted"], json!(true));

    let stored: (Value,) = sqlx::query_as(
        "SELECT response_result FROM idempotency WHERE tenant_id = $1 AND idempotency_key = $2",
    )
    .bind(&tenant)
    .bind(&command_id)
    .fetch_one(&pool)
    .await
    .expect("the stored result");
    assert_eq!(stored.0["ok"], json!(false), "the rejection was stored");
    assert_eq!(
        stored.0["error"]["code"].as_str().unwrap(),
        "invalid_transition",
        "the stored code names the transition refusal"
    );

    let events = thread_events(&client, &server.base(), &thread, &tenant, &human).await;
    assert!(
        !events
            .iter()
            .any(|e| e["event_type"] == "thread.contribution_submitted"),
        "no contribution entered the closed thread"
    );
    // Closure preserved the timeline as-is: the close event is last.
    assert_eq!(
        events.last().unwrap()["event_type"].as_str().unwrap(),
        "thread.closed"
    );
}

/// Ordinary channel events (WP3 traffic without a work payload) stay receipts:
/// no domain effect, no idempotency claim.
#[tokio::test]
async fn ordinary_channel_events_stay_receipts_only() {
    let _g = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let client = reqwest::Client::new();
    let (tenant, human, role, thread, cert_hex, key_hex) = bootstrap(&client, &server.base()).await;

    // The `.1.2.2` contract: authenticate (handshake → fencing token) before any
    // channel traffic, even a receipt-only event.
    let (_handshake, token, ep8) =
        handshake(&client, &server.base(), &role, &cert_hex, &key_hex).await;
    let (status, receipt) = submit_event(
        &client,
        &server.base(),
        &role,
        &token,
        ep8,
        "evt_00000000-0000-7000-8000-000000000031",
        "op_00000000-0000-7000-8000-000000000031",
        &json!({ "kind": "some_other_signal", "value": 1 }),
    )
    .await;
    assert_eq!(status, 200, "the event is accepted");
    assert_eq!(receipt["accepted"], json!(true));

    let claims: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM idempotency WHERE tenant_id = $1")
        .bind(&tenant)
        .fetch_one(&pool)
        .await
        .expect("count claims");
    assert_eq!(claims, 1, "only the thread-create command claimed a key");
    let events = thread_events(&client, &server.base(), &thread, &tenant, &human).await;
    assert_eq!(events.len(), 1, "the thread timeline is untouched");
}

/// A temporary journal path under the repo's build dir (same-volume locality, §13).
fn journal_path(name: &str) -> std::path::PathBuf {
    let base = std::env::var_os("CARGO_TARGET_TMPDIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target")
        });
    let unique = uuid::Uuid::now_v7();
    let dir = base
        .join("cached-decision-live")
        .join(format!("{name}-{unique}"));
    std::fs::create_dir_all(dir.parent().expect("the fixture parent")).unwrap();
    // Exclusive: an existing directory belongs to another fixture or an
    // earlier run, and must never be adopted.
    std::fs::DirBuilder::new()
        .create(&dir)
        .expect("the fixture directory is new");
    dir.join("node.db")
}

/// THE `.1.5.2` acceptance leg (ADR-008), measured end-to-end: the delivery
/// carries the admission decision; a FRESH, epoch-current cached allow drives
/// the REAL node worker to completion (the contribution lands); a revocation
/// bumps the tenant epoch; and the NEXT dispatch — of work delivered BEFORE the
/// revocation — is refused at the dispatch boundary WITHOUT a re-ask (the
/// adapter never runs, the refusal is journaled, no contribution lands).
#[tokio::test]
async fn a_revocation_invalidates_the_cached_decision_at_the_next_dispatch() {
    let _g = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let client = reqwest::Client::new();
    let (tenant, human, role, thread, cert_hex, key_hex) = bootstrap(&client, &server.base()).await;

    // The invitation rides the accept; the work item carries the admission decision.
    let (status, _) = command(
        &client,
        &server.base(),
        &format!("/v1/threads/{thread}/commands"),
        &human,
        &envelope(
            "thread.invite",
            "key-cd-inv",
            json!({ "tenant_id": tenant, "agent_role": role }),
        ),
    )
    .await;
    assert_eq!(status, 200, "invite succeeds");
    accept_invitation(
        &client,
        &server.base(),
        &role,
        &thread,
        &tenant,
        "key-cd-acc",
    )
    .await;

    // THE delivery carries the decision: the handshake replay names the admitting
    // record + digest + decision time + the epoch at decision time (0 — no
    // revocation yet), and the response carries the current epoch.
    let (view, _token, _ep9) = handshake(&client, &server.base(), &role, &cert_hex, &key_hex).await;
    assert_eq!(view["revocation_epoch"], json!(0), "fresh tenant epoch");
    let work = &view["replay"][0];
    assert!(
        work["authz_ref"]
            .as_str()
            .is_some_and(|s| s.starts_with("authz_")),
        "the delivery names the admitting record: {work}"
    );
    assert!(
        work["policy_digest"]
            .as_str()
            .is_some_and(|s| !s.is_empty()),
        "the delivery carries the policy digest"
    );
    assert!(
        work["decided_at"].as_str().is_some(),
        "the delivery carries the decision time"
    );
    assert_eq!(work["revocation_epoch"], json!(0), "decided under epoch 0");

    // Drive the REAL node worker against the live server: the fresh, epoch-
    // current cached allow dispatches and the contribution lands.
    let key_der = from_hex(&key_hex).expect("key hex");
    let der = rustls_pki_types::PrivateKeyDer::try_from(key_der).expect("key DER");
    let key = rcgen::KeyPair::from_der_and_sign_algo(&der, &rcgen::PKCS_ECDSA_P256_SHA256)
        .expect("key parses");
    let cert_der = from_hex(&cert_hex).expect("cert hex");
    let node = reasonbraid_node::Node::open(
        journal_path("revoked-cache"),
        server.base(),
        role.clone(),
        cert_der,
        key,
    )
    .await
    .expect("open node");
    node.reconcile()
        .await
        .expect("reconcile journals the delivery");
    let worker = reasonbraid_node::Worker::new(
        node.clone(),
        reasonbraid_adapter::FakeAdapter::new(
            vec![reasonbraid_adapter::ScriptStep::Complete { usage: None }],
            reasonbraid_adapter::StatusLookupSpec::Unsupported,
            reasonbraid_adapter::AdapterCapabilities {
                streaming: false,
                cancellation: reasonbraid_adapter::CancellationStrength::BestEffort,
                provider_idempotency: false,
                status_lookup: false,
                tool_support: false,
                policy_injection: reasonbraid_adapter::PolicyInjectionMode::None,
            },
        ),
        reasonbraid_node::LocalBudget::new(reasonbraid_core::BudgetDimensions {
            calls: Some(100),
            input_tokens: Some(100_000),
            output_tokens: Some(100_000),
            wall_clock_seconds: Some(10_000),
        }),
        std::time::Duration::from_millis(50),
    );
    worker.tick().await.expect("the fresh allow dispatches");

    let events = thread_events(&client, &server.base(), &thread, &tenant, &human).await;
    let contribution_id = events
        .iter()
        .find(|e| e["event_type"] == "thread.contribution_submitted")
        .expect("the contribution exists")["event_id"]
        .as_str()
        .unwrap()
        .to_string();

    // Second work item, delivered BEFORE the revocation (the epoch is still 0):
    // the human challenges the contribution, the revise work rides the
    // challenge transaction.
    let (status, _) = command(
        &client,
        &server.base(),
        &format!("/v1/threads/{thread}/commands"),
        &human,
        &envelope(
            "thread.challenge",
            "key-cd-chl",
            json!({
                "tenant_id": tenant,
                "target_event_id": contribution_id,
                "content": "justify",
            }),
        ),
    )
    .await;
    assert_eq!(status, 200, "challenge succeeds");
    let epoch_after_delivery: i64 =
        sqlx::query_scalar("SELECT revocation_epoch FROM tenants WHERE tenant_id = $1")
            .bind(&tenant)
            .fetch_one(&pool)
            .await
            .expect("epoch");
    assert_eq!(epoch_after_delivery, 0, "still no revocation yet");

    // Revoke the role's grant — the epoch bump is the cache-invalidation signal.
    let role_grant: String = sqlx::query_scalar(
        "SELECT grant_id FROM authority_grants WHERE tenant_id = $1 AND status = 'active' \
         AND subject_kind = 'role'",
    )
    .bind(&tenant)
    .fetch_one(&pool)
    .await
    .expect("the role's active grant");
    let (status, revoked) = admin_revoke(
        &client,
        &server.base(),
        &format!("/v1/admin/grants/{role_grant}/revoke"),
        &human,
        &tenant,
    )
    .await;
    assert_eq!(status, 200, "the revocation succeeds: {revoked}");
    let epoch_after_revocation: i64 =
        sqlx::query_scalar("SELECT revocation_epoch FROM tenants WHERE tenant_id = $1")
            .bind(&tenant)
            .fetch_one(&pool)
            .await
            .expect("epoch");
    assert_eq!(
        epoch_after_revocation, 1,
        "the revocation bumped the tenant epoch (measured)"
    );

    // THE next dispatch refuses WITHOUT a re-ask: the revise work was decided
    // under epoch 0, the node now knows epoch 1 — the cached decision is stale,
    // so the dispatch boundary refuses it and the adapter never runs.
    worker
        .tick()
        .await
        .expect("the tick journals the refusal, no error");
    let refused = node
        .journal()
        .work_items()
        .await
        .expect("work items")
        .into_iter()
        .find(|w| w.latest_attempt_status.as_deref() == Some("failed_before_dispatch"))
        .expect("the revise dispatch was refused");
    let attempts = node
        .journal()
        .attempts_for_operation(refused.operation_id.as_deref().expect("operation"))
        .await
        .expect("attempt list");
    assert_eq!(attempts.len(), 1, "exactly one attempt — the refusal");
    let evidence = attempts[0].evidence.clone().unwrap_or_default();
    assert!(
        evidence.contains("stale"),
        "the refusal names the staleness (recorded epoch 0 vs current 1): {evidence}"
    );
    assert_eq!(attempts[0].status, "failed_before_dispatch");

    let events = thread_events(&client, &server.base(), &thread, &tenant, &human).await;
    assert_eq!(
        events
            .iter()
            .filter(|e| e["event_type"] == "thread.contribution_submitted")
            .count(),
        1,
        "the refused revise dispatched nothing — exactly one contribution landed"
    );
}

/// Lowercase-hex decode (the enroll response ships DER as hex).
fn from_hex(s: &str) -> Result<Vec<u8>, String> {
    if !s.len().is_multiple_of(2) {
        return Err("odd-length hex".to_string());
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(|e| e.to_string()))
        .collect()
}

/// THE `.2.4` acceptance leg, measured end-to-end: a node whose adapter keeps
/// refusing re-dispatches bounded, then reports the DEAD LETTER — the server
/// auto-quarantines the inbox row with the reason. The operator REPLAYS the
/// command (the quarantine clears, the admission decision refreshes, the row
/// re-sequences), and the node's NEXT tick re-delivers it under the fresh
/// decision — the re-dispatch completes and the contribution lands.
#[tokio::test]
async fn a_dead_lettered_command_replays_and_redispatches() {
    let _g = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let client = reqwest::Client::new();
    let (tenant, human, role, thread, cert_hex, key_hex) = bootstrap(&client, &server.base()).await;

    let (status, _) = command(
        &client,
        &server.base(),
        &format!("/v1/threads/{thread}/commands"),
        &human,
        &envelope(
            "thread.invite",
            "key-dl-inv",
            json!({ "tenant_id": tenant, "agent_role": role }),
        ),
    )
    .await;
    assert_eq!(status, 200, "invite succeeds");
    accept_invitation(
        &client,
        &server.base(),
        &role,
        &thread,
        &tenant,
        "key-dl-acc",
    )
    .await;

    let cert_der = from_hex(&cert_hex).expect("cert hex");
    let journal = journal_path("dead-letter-live");

    // Phase 1: a worker whose adapter ALWAYS refuses before dispatch. Three
    // bounded attempts (`.2.3`), then the retry gate's terminal refusal
    // reports the dead letter — the server auto-quarantines the inbox row.
    let command_id = {
        let key_der = from_hex(&key_hex).expect("key hex");
        let der = rustls_pki_types::PrivateKeyDer::try_from(key_der).expect("key DER");
        let key = rcgen::KeyPair::from_der_and_sign_algo(&der, &rcgen::PKCS_ECDSA_P256_SHA256)
            .expect("key parses");
        let node = reasonbraid_node::Node::open(
            journal.clone(),
            server.base(),
            role.clone(),
            cert_der.clone(),
            key,
        )
        .await
        .expect("open node");
        node.reconcile().await.expect("reconcile");
        let work = node.journal().work_items().await.expect("work items");
        let command_id = work[0].command_id.clone();
        let worker = reasonbraid_node::Worker::new(
            node.clone(),
            reasonbraid_adapter::FakeAdapter::new(
                vec![reasonbraid_adapter::ScriptStep::FailBeforeDispatch {
                    reason: "provider unavailable".to_string(),
                }],
                reasonbraid_adapter::StatusLookupSpec::Unsupported,
                reasonbraid_adapter::AdapterCapabilities {
                    streaming: false,
                    cancellation: reasonbraid_adapter::CancellationStrength::BestEffort,
                    provider_idempotency: false,
                    status_lookup: false,
                    tool_support: false,
                    policy_injection: reasonbraid_adapter::PolicyInjectionMode::None,
                },
            ),
            reasonbraid_node::LocalBudget::new(reasonbraid_core::BudgetDimensions {
                calls: Some(100),
                input_tokens: Some(100_000),
                output_tokens: Some(100_000),
                wall_clock_seconds: Some(10_000),
            }),
            std::time::Duration::from_millis(50),
        );
        // Four ticks: 3 bounded attempts + the terminal refusal (the report).
        for _ in 0..4 {
            worker.tick().await.expect("tick");
        }
        command_id
    };

    // The server auto-quarantined the row with the refusal reason.
    let quarantine: (Option<chrono::DateTime<chrono::Utc>>, Option<String>) = sqlx::query_as(
        "SELECT quarantined_at, quarantine_reason FROM node_inbox \
             WHERE node_id = $1 AND command_id = $2",
    )
    .bind(&role)
    .bind(&command_id)
    .fetch_one(&pool)
    .await
    .expect("the row");
    assert!(
        quarantine.0.is_some(),
        "the dead letter auto-quarantined the inbox row"
    );
    assert!(
        quarantine
            .1
            .as_deref()
            .is_some_and(|r| r.contains("bounded retry budget is exhausted")),
        "the quarantine carries the terminal reason (why re-dispatch stopped): {:?}",
        quarantine.1
    );

    // THE operator replay: the quarantine clears, the admission decision
    // refreshes, the row re-sequences to the delivery tail.
    let response = client
        .post(format!("{}/v1/nodes/replay", server.base()))
        .header(PRINCIPAL_HEADER, &human)
        .json(&json!({
            "tenant_id": tenant,
            "node_id": role,
            "command_id": command_id,
        }))
        .send()
        .await
        .expect("replay request");
    assert_eq!(response.status().as_u16(), 200, "the replay succeeds");

    // Phase 2: a worker over the SAME journal with a COMPLETING adapter —
    // the re-delivered command refreshes its cached decision, the retry gate
    // re-arms (the old refusals precede the fresh decision), and the
    // re-dispatch completes: the contribution lands.
    {
        let key_der = from_hex(&key_hex).expect("key hex");
        let der = rustls_pki_types::PrivateKeyDer::try_from(key_der).expect("key DER");
        let key = rcgen::KeyPair::from_der_and_sign_algo(&der, &rcgen::PKCS_ECDSA_P256_SHA256)
            .expect("key parses");
        let node = reasonbraid_node::Node::open(
            journal,
            server.base(),
            role.clone(),
            cert_der.clone(),
            key,
        )
        .await
        .expect("open node");
        node.reconcile().await.expect("reconcile");
        let worker = reasonbraid_node::Worker::new(
            node.clone(),
            reasonbraid_adapter::FakeAdapter::new(
                vec![reasonbraid_adapter::ScriptStep::Complete { usage: None }],
                reasonbraid_adapter::StatusLookupSpec::Unsupported,
                reasonbraid_adapter::AdapterCapabilities {
                    streaming: false,
                    cancellation: reasonbraid_adapter::CancellationStrength::BestEffort,
                    provider_idempotency: false,
                    status_lookup: false,
                    tool_support: false,
                    policy_injection: reasonbraid_adapter::PolicyInjectionMode::None,
                },
            ),
            reasonbraid_node::LocalBudget::new(reasonbraid_core::BudgetDimensions {
                calls: Some(100),
                input_tokens: Some(100_000),
                output_tokens: Some(100_000),
                wall_clock_seconds: Some(10_000),
            }),
            std::time::Duration::from_millis(50),
        );
        worker.tick().await.expect("the replayed dispatch");
    }

    let events = thread_events(&client, &server.base(), &thread, &tenant, &human).await;
    assert_eq!(
        events
            .iter()
            .filter(|e| e["event_type"] == "thread.contribution_submitted")
            .count(),
        1,
        "the replayed command dispatched exactly once — one contribution landed"
    );
}
