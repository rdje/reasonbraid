//! WP6 control-API integration tests (`PHASE-0.6.1`): the CLI's HTTP surface over a
//! live PostgreSQL — enroll bootstrap, the full thread flow, denials with audit rows,
//! idempotent replay/conflict, invalid transitions, forged-field rejection, and
//! inspection through the API only (the acceptance: **no database surgery required
//! to inspect state**).
//!
//! Run with `scripts/run_pg_tests.sh` locally or the `pg-tests` CI job. Without
//! `DATABASE_URL` these skip, so `make check` stays green offline.

#[path = "support/mod.rs"]
mod pg_test_support;

#[path = "support/cleanup.rs"]
mod pg_cleanup;

use std::net::SocketAddr;
use std::sync::OnceLock;

use reasonbraid_core::{
    AuthorityContext, ClientContext, CommandEnvelope, RequestId, TargetSelector, PROTOCOL_VERSION,
};
use reasonbraid_server::{api_router, PRINCIPAL_HEADER};
use serde_json::{json, Value};
use sqlx::PgPool;

/// The API tests own the thread/authority/budget tables (like the channel tests own
/// the inbox): tests never run concurrently against the same PG database.
static API_LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();

async fn api_guard() -> tokio::sync::MutexGuard<'static, ()> {
    API_LOCK
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
            "budget_reservations",
            "budget_ceilings",
            "spend_breakers",
            "authorization_records",
            "authority_grants",
            "enrollments",
            "enrollment_boundaries",
            "node_enroll_audit",
            "node_keys",
            "node_certificates",
            "server_ca",
            "node_leases",
            "node_enrollment_tokens",
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

/// The running server half: an axum listener on an ephemeral loopback port.
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
        let router = api_router(pool.clone());
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

/// Build a command envelope (the CLI's client-side shape).
fn envelope(operation: &str, key: &str, body: Value) -> CommandEnvelope {
    CommandEnvelope {
        protocol_version: PROTOCOL_VERSION.to_string(),
        operation: operation.to_string(),
        request_id: RequestId::new(),
        idempotency_key: key.to_string(),
        expected_aggregate_version: None,
        body,
        authority_context: None,
        client_context: ClientContext::default(),
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

/// The happy path end to end — and the `.6.1` acceptance: every inspection happens
/// through the API, never through the database.
#[tokio::test]
async fn full_flow_inspects_state_through_the_api_only() {
    let _guard = api_guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    // 1. Bootstrap: a human creates the tenant (boundary + admin grant), a role
    //    enrolls into it.
    let (status, alice) = enroll(&client, &base, json!({ "kind": "human", "name": "alice" })).await;
    assert_eq!(status, 200, "enroll alice: {alice}");
    assert_eq!(alice["replayed"], json!(false));
    let tenant = alice["tenant_id"].as_str().unwrap().to_string();
    let alice_id = alice["principal_id"].as_str().unwrap().to_string();
    assert!(alice["boundary_id"].is_string(), "bootstrap boundary");
    assert!(alice["grant_id"].is_string(), "bootstrap grant");

    let (status, reviewer) = enroll(
        &client,
        &base,
        json!({ "kind": "role", "name": "reviewer", "tenant_id": tenant }),
    )
    .await;
    assert_eq!(status, 200, "enroll reviewer: {reviewer}");
    let reviewer_id = reviewer["principal_id"].as_str().unwrap().to_string();
    assert!(reviewer_id.starts_with("rol_"));

    // The same enroll is an idempotent replay of the ORIGINAL principal id.
    let (status, again) = enroll(
        &client,
        &base,
        json!({ "kind": "role", "name": "reviewer", "tenant_id": tenant }),
    )
    .await;
    assert_eq!(status, 200);
    assert_eq!(again["replayed"], json!(true));
    assert_eq!(again["principal_id"], reviewer["principal_id"]);

    // 2. Create the thread (as alice), with an explicit budget.
    let (status, created) = command(
        &client,
        &base,
        "/v1/threads",
        &alice_id,
        &envelope(
            "thread.create",
            "k-create",
            json!({
                "tenant_id": tenant,
                "subject": "should we ship?",
                "objective": "decide with evidence",
                "budget": { "calls": 3 },
            }),
        ),
    )
    .await;
    assert_eq!(status, 200, "create: {created}");
    let thread_id = created["thread_id"].as_str().unwrap().to_string();
    assert_eq!(created["thread_state"], json!("open"));

    // 3. Invite the reviewer (PENDING); the reviewer ACCEPTS (explicit
    //    participants, `.1.3.1`); contribute; challenge; revise; close.
    let (status, invited) = command(
        &client,
        &base,
        &format!("/v1/threads/{thread_id}/commands"),
        &alice_id,
        &envelope(
            "thread.invite",
            "k-invite",
            json!({ "tenant_id": tenant, "agent_role": reviewer_id }),
        ),
    )
    .await;
    assert_eq!(status, 200, "invite: {invited}");

    let (status, accepted) = command(
        &client,
        &base,
        &format!("/v1/threads/{thread_id}/commands"),
        &reviewer_id,
        &envelope(
            "thread.accept_invitation",
            "k-accept",
            json!({ "tenant_id": tenant }),
        ),
    )
    .await;
    assert_eq!(status, 200, "accept: {accepted}");
    assert_eq!(accepted["event_type"], json!("thread.invitation_accepted"));

    let (status, contributed) = command(
        &client,
        &base,
        &format!("/v1/threads/{thread_id}/commands"),
        &reviewer_id,
        &envelope(
            "thread.contribute",
            "k-contribute",
            json!({
                "tenant_id": tenant,
                "content": "Ship it: the kill-risk experiments are green.",
                "kind": "claim",
                "evidence_refs": [
                    { "uri": "https://example.org/kill-risk-report", "digest": "sha256:abc123", "note": "the WP2 sweep" },
                    { "uri": "https://example.org/journal-sweep" },
                ],
            }),
        ),
    )
    .await;
    assert_eq!(status, 200, "contribute: {contributed}");
    let contribution_event = contributed["event_id"].as_str().unwrap().to_string();
    assert_eq!(
        contributed["event_type"],
        json!("thread.contribution_submitted")
    );

    let (status, challenged) = command(
        &client,
        &base,
        &format!("/v1/threads/{thread_id}/commands"),
        &alice_id,
        &envelope(
            "thread.challenge",
            "k-challenge",
            json!({
                "tenant_id": tenant,
                "target_event_id": contribution_event,
                "content": "Which experiments, exactly?",
            }),
        ),
    )
    .await;
    assert_eq!(status, 200, "challenge: {challenged}");
    let challenge_event = challenged["event_id"].as_str().unwrap().to_string();

    let (status, revised) = command(
        &client,
        &base,
        &format!("/v1/threads/{thread_id}/commands"),
        &reviewer_id,
        &envelope(
            "thread.revise",
            "k-revise",
            json!({
                "tenant_id": tenant,
                "target_event_id": challenge_event,
                "content": "The SQLite kill-point sweep and the Codex qualification.",
            }),
        ),
    )
    .await;
    assert_eq!(status, 200, "revise: {revised}");

    let (status, closed) = command(
        &client,
        &base,
        &format!("/v1/threads/{thread_id}/commands"),
        &alice_id,
        &envelope(
            "thread.close",
            "k-close",
            json!({ "tenant_id": tenant, "reason": "decision reached" }),
        ),
    )
    .await;
    assert_eq!(status, 200, "close: {closed}");
    assert_eq!(closed["thread_state"], json!("closed"));

    // 4. Inspect — through the API only.
    let (status, state) = get(
        &client,
        &base,
        &format!("/v1/threads/{thread_id}?tenant_id={tenant}"),
        &alice_id,
    )
    .await;
    assert_eq!(status, 200, "inspect thread: {state}");
    assert_eq!(state["state"]["state"], json!("closed"));
    assert_eq!(state["state"]["close_reason"], json!("decision reached"));
    assert_eq!(state["state"]["contributions"], json!(1));
    assert_eq!(state["state"]["revisions"], json!(1));
    assert_eq!(state["state"]["open_challenges"], json!(0));
    assert_eq!(
        state["state"]["participants"][&reviewer_id],
        json!("accepted"),
        "the explicit accept made the role a participant"
    );
    assert_eq!(
        state["state"]["invitations"][&reviewer_id]["expires_at"],
        json!(null),
        "the invitation record carries the offer facts (no expiry)"
    );

    let (status, events) = get(
        &client,
        &base,
        &format!("/v1/threads/{thread_id}/events?tenant_id={tenant}"),
        &alice_id,
    )
    .await;
    assert_eq!(status, 200, "inspect events: {events}");
    let types: Vec<&str> = events["events"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["event_type"].as_str().unwrap())
        .collect();
    assert_eq!(
        types,
        vec![
            "thread.created",
            "thread.participant_invited",
            "thread.invitation_accepted",
            "thread.contribution_submitted",
            "thread.challenge_posted",
            "thread.revision_submitted",
            "thread.closed",
        ],
        "the ordered audit timeline"
    );
    assert_eq!(events["next_cursor"], json!(7));

    // `.1.5.1`: the contribution event carries its structured kind and evidence
    // references — the inspection surface renders them, nothing is silently dropped.
    let contribution = events["events"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["event_type"] == json!("thread.contribution_submitted"))
        .expect("the contribution event exists");
    assert_eq!(contribution["body"]["kind"], json!("claim"));
    assert_eq!(
        contribution["body"]["evidence_refs"],
        json!([
            { "uri": "https://example.org/kill-risk-report", "digest": "sha256:abc123", "note": "the WP2 sweep" },
            { "uri": "https://example.org/journal-sweep" },
        ])
    );

    let (status, audit) = get(
        &client,
        &base,
        &format!("/v1/threads/{thread_id}/audit?tenant_id={tenant}"),
        &alice_id,
    )
    .await;
    assert_eq!(status, 200, "inspect audit: {audit}");
    let actions: Vec<&str> = audit["records"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["action"].as_str().unwrap())
        .collect();
    assert_eq!(
        actions,
        vec![
            "thread_invite",
            "thread_invitation_respond",
            "thread_contribute",
            "thread_contribute",
            "thread_contribute",
            "thread_close",
            "thread_inspect",
            "thread_inspect",
            "thread_inspect",
        ],
        "every command and read left an audit record (reads recorded too)"
    );
    for record in audit["records"].as_array().unwrap() {
        assert_eq!(record["decision"], json!("allowed"));
        assert_eq!(record["evaluation"], json!({"kind":"boundary_checked"}));
        assert_eq!(record["policy_digest"].as_str().unwrap().len(), 64);
    }

    let (status, listed) = get(
        &client,
        &base,
        &format!("/v1/threads?tenant_id={tenant}"),
        &alice_id,
    )
    .await;
    assert_eq!(status, 200);
    assert_eq!(listed["threads"].as_array().unwrap().len(), 1);
    assert_eq!(listed["threads"][0]["thread_id"], thread_id);
}

/// Deny-by-default: a role with only `thread_contribute` cannot create threads; the
/// denial is durable and effect-free.
#[tokio::test]
async fn denials_are_recorded_and_have_no_domain_effect() {
    let _guard = api_guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (_, alice) = enroll(&client, &base, json!({ "kind": "human", "name": "alice" })).await;
    let tenant = alice["tenant_id"].as_str().unwrap().to_string();
    let alice_id = alice["principal_id"].as_str().unwrap().to_string();
    let (_, reviewer) = enroll(
        &client,
        &base,
        json!({ "kind": "role", "name": "reviewer", "tenant_id": tenant }),
    )
    .await;
    let reviewer_id = reviewer["principal_id"].as_str().unwrap().to_string();

    let (status, denied) = command(
        &client,
        &base,
        "/v1/threads",
        &reviewer_id,
        &envelope(
            "thread.create",
            "k-denied-create",
            json!({ "tenant_id": tenant, "subject": "s", "objective": "o" }),
        ),
    )
    .await;
    assert_eq!(status, 403, "denied create: {denied}");
    assert_eq!(denied["code"], json!("unauthorized"));
    assert!(
        denied["message"].as_str().unwrap().contains("denied"),
        "the denial names the audit record: {denied}"
    );

    // No domain effect: the thread list stays empty.
    let (status, listed) = get(
        &client,
        &base,
        &format!("/v1/threads?tenant_id={tenant}"),
        &alice_id,
    )
    .await;
    assert_eq!(status, 200);
    assert_eq!(listed["threads"].as_array().unwrap().len(), 0);

    // A contributor without participation is refused too (grant ≠ standing).
    let (status, created) = command(
        &client,
        &base,
        "/v1/threads",
        &alice_id,
        &envelope(
            "thread.create",
            "k-thread-for-outsider",
            json!({ "tenant_id": tenant, "subject": "s", "objective": "o" }),
        ),
    )
    .await;
    assert_eq!(status, 200);
    let thread_id = created["thread_id"].as_str().unwrap().to_string();

    let (status, refused) = command(
        &client,
        &base,
        &format!("/v1/threads/{thread_id}/commands"),
        &reviewer_id,
        &envelope(
            "thread.contribute",
            "k-outsider",
            json!({ "tenant_id": tenant, "content": "hi" }),
        ),
    )
    .await;
    assert_eq!(status, 403, "outsider contribute: {refused}");
    assert!(refused["message"]
        .as_str()
        .unwrap()
        .contains("not a participant"));
}

/// Idempotency: a redelivery of the same key + body returns the ORIGINAL result; the
/// same key with a different body is a conflict.
#[tokio::test]
async fn replay_returns_the_original_result_and_conflicts_are_typed() {
    let _guard = api_guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (_, alice) = enroll(&client, &base, json!({ "kind": "human", "name": "alice" })).await;
    let tenant = alice["tenant_id"].as_str().unwrap().to_string();
    let alice_id = alice["principal_id"].as_str().unwrap().to_string();

    let create = || {
        envelope(
            "thread.create",
            "k-replay",
            json!({ "tenant_id": tenant, "subject": "s", "objective": "o" }),
        )
    };

    let (status, first) = command(&client, &base, "/v1/threads", &alice_id, &create()).await;
    assert_eq!(status, 200, "first create: {first}");
    let thread_id = first["thread_id"].as_str().unwrap().to_string();

    // Same key + same body: the ORIGINAL result, marked replayed.
    let (status, replay) = command(&client, &base, "/v1/threads", &alice_id, &create()).await;
    assert_eq!(status, 200, "replay: {replay}");
    assert_eq!(replay["replayed"], json!(true));
    assert_eq!(replay["thread_id"], first["thread_id"]);

    // Same key + different body: a typed conflict, never a silent overwrite.
    let (status, conflict) = command(
        &client,
        &base,
        "/v1/threads",
        &alice_id,
        &envelope(
            "thread.create",
            "k-replay",
            json!({ "tenant_id": tenant, "subject": "different", "objective": "o" }),
        ),
    )
    .await;
    assert_eq!(status, 409, "conflict: {conflict}");
    assert_eq!(conflict["code"], json!("idempotency_mismatch"));

    // Exactly one thread exists.
    let (status, listed) = get(
        &client,
        &base,
        &format!("/v1/threads?tenant_id={tenant}"),
        &alice_id,
    )
    .await;
    assert_eq!(status, 200);
    assert_eq!(listed["threads"].as_array().unwrap().len(), 1);
    assert_eq!(listed["threads"][0]["thread_id"], thread_id);
}

/// The core state machines bind the API: closing twice and contributing after close
/// are deterministic `invalid_transition` rejections.
#[tokio::test]
async fn invalid_transitions_are_rejected_deterministically() {
    let _guard = api_guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (_, alice) = enroll(&client, &base, json!({ "kind": "human", "name": "alice" })).await;
    let tenant = alice["tenant_id"].as_str().unwrap().to_string();
    let alice_id = alice["principal_id"].as_str().unwrap().to_string();

    let (_, created) = command(
        &client,
        &base,
        "/v1/threads",
        &alice_id,
        &envelope(
            "thread.create",
            "k-transitions",
            json!({ "tenant_id": tenant, "subject": "s", "objective": "o" }),
        ),
    )
    .await;
    let thread_id = created["thread_id"].as_str().unwrap().to_string();
    let path = format!("/v1/threads/{thread_id}/commands");

    let (status, _) = command(
        &client,
        &base,
        &path,
        &alice_id,
        &envelope(
            "thread.close",
            "k-close-1",
            json!({ "tenant_id": tenant, "reason": "done" }),
        ),
    )
    .await;
    assert_eq!(status, 200);

    let (status, second_close) = command(
        &client,
        &base,
        &path,
        &alice_id,
        &envelope(
            "thread.close",
            "k-close-2",
            json!({ "tenant_id": tenant, "reason": "again" }),
        ),
    )
    .await;
    assert_eq!(status, 409, "second close: {second_close}");
    assert_eq!(second_close["code"], json!("invalid_transition"));

    let (status, contribute_after_close) = command(
        &client,
        &base,
        &path,
        &alice_id,
        &envelope(
            "thread.contribute",
            "k-late",
            json!({ "tenant_id": tenant, "content": "too late" }),
        ),
    )
    .await;
    assert_eq!(
        status, 409,
        "contribute after close: {contribute_after_close}"
    );
    assert_eq!(contribute_after_close["code"], json!("invalid_transition"));
}

/// The wire contract rejects forged authoritative fields and wrong protocol versions.
#[tokio::test]
async fn forged_fields_and_version_mismatches_are_rejected() {
    let _guard = api_guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (_, alice) = enroll(&client, &base, json!({ "kind": "human", "name": "alice" })).await;
    let tenant = alice["tenant_id"].as_str().unwrap().to_string();
    let alice_id = alice["principal_id"].as_str().unwrap().to_string();

    // A client-supplied authoritative field is rejected, not ignored (§9.1): axum's
    // Json extractor answers with 422 and the serde rejection NAMES the forged field
    // (the .3.2 channel convention — the handler never sees the payload).
    let forged = json!({
        "protocol_version": PROTOCOL_VERSION,
        "operation": "thread.create",
        "request_id": RequestId::new().to_string(),
        "idempotency_key": "k-forged",
        "body": { "tenant_id": tenant, "subject": "s", "objective": "o" },
        "client_context": {},
        "actor_principal_id": "agt_00000000-0000-7000-8000-000000000001",
    });
    let response = client
        .post(format!("{base}/v1/threads"))
        .header(PRINCIPAL_HEADER, &alice_id)
        .json(&forged)
        .send()
        .await
        .expect("forged request");
    assert_eq!(response.status().as_u16(), 422, "forged field rejected");
    let text = response.text().await.unwrap();
    assert!(
        text.contains("unknown field"),
        "the rejection names the forged field: {text}"
    );

    // A wrong protocol version fails loudly, never silently downgrades.
    let mut wrong_version = envelope(
        "thread.create",
        "k-version",
        json!({ "tenant_id": tenant, "subject": "s", "objective": "o" }),
    );
    wrong_version.protocol_version = "reasonbraid/0.3".to_string();
    let (status, version_err) =
        command(&client, &base, "/v1/threads", &alice_id, &wrong_version).await;
    assert_eq!(status, 400, "version: {version_err}");
    assert_eq!(version_err["code"], json!("protocol_incompatible"));

    // No domain effect from any of it.
    let (status, listed) = get(
        &client,
        &base,
        &format!("/v1/threads?tenant_id={tenant}"),
        &alice_id,
    )
    .await;
    assert_eq!(status, 200);
    assert_eq!(listed["threads"].as_array().unwrap().len(), 0);
}

/// The dev profile trusts the principal header but still requires it to be present
/// and well-formed.
#[tokio::test]
async fn missing_or_malformed_principal_is_unauthenticated() {
    let _guard = api_guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (_, alice) = enroll(&client, &base, json!({ "kind": "human", "name": "alice" })).await;
    let tenant = alice["tenant_id"].as_str().unwrap().to_string();

    let env = envelope(
        "thread.create",
        "k-no-header",
        json!({ "tenant_id": tenant, "subject": "s", "objective": "o" }),
    );
    let response = client
        .post(format!("{base}/v1/threads"))
        .json(&env)
        .send()
        .await
        .expect("no-header request");
    assert_eq!(response.status().as_u16(), 401);
    let body: Value = response.json().await.unwrap();
    assert_eq!(body["code"], json!("unauthenticated"));

    let (status, malformed) = command(
        &client,
        &base,
        "/v1/threads",
        "not-a-principal",
        &envelope(
            "thread.create",
            "k-bad-header",
            json!({ "tenant_id": tenant, "subject": "s", "objective": "o" }),
        ),
    )
    .await;
    assert_eq!(status, 401, "malformed header: {malformed}");
    assert_eq!(malformed["code"], json!("unauthenticated"));
}

/// A challenge must target a contribution of the SAME thread; anything else is a
/// typed invalid command.
#[tokio::test]
async fn challenge_targets_are_checked() {
    let _guard = api_guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (_, alice) = enroll(&client, &base, json!({ "kind": "human", "name": "alice" })).await;
    let tenant = alice["tenant_id"].as_str().unwrap().to_string();
    let alice_id = alice["principal_id"].as_str().unwrap().to_string();
    let (_, reviewer) = enroll(
        &client,
        &base,
        json!({ "kind": "role", "name": "reviewer", "tenant_id": tenant }),
    )
    .await;
    let reviewer_id = reviewer["principal_id"].as_str().unwrap().to_string();

    let (_, created) = command(
        &client,
        &base,
        "/v1/threads",
        &alice_id,
        &envelope(
            "thread.create",
            "k-targets",
            json!({ "tenant_id": tenant, "subject": "s", "objective": "o" }),
        ),
    )
    .await;
    let thread_id = created["thread_id"].as_str().unwrap().to_string();
    let path = format!("/v1/threads/{thread_id}/commands");

    let (_, invited) = command(
        &client,
        &base,
        &path,
        &alice_id,
        &envelope(
            "thread.invite",
            "k-invite-targets",
            json!({ "tenant_id": tenant, "agent_role": reviewer_id }),
        ),
    )
    .await;
    let invite_event = invited["event_id"].as_str().unwrap().to_string();

    // Challenging a non-contribution (the invite event) is refused.
    let (status, wrong_target) = command(
        &client,
        &base,
        &path,
        &alice_id,
        &envelope(
            "thread.challenge",
            "k-challenge-wrong",
            json!({
                "tenant_id": tenant,
                "target_event_id": invite_event,
                "content": "challenging the invite",
            }),
        ),
    )
    .await;
    assert_eq!(status, 400, "wrong target: {wrong_target}");
    assert_eq!(wrong_target["code"], json!("invalid_command"));

    // Challenging an event that does not exist is refused too.
    let (status, missing_target) = command(
        &client,
        &base,
        &path,
        &alice_id,
        &envelope(
            "thread.challenge",
            "k-challenge-missing",
            json!({
                "tenant_id": tenant,
                "target_event_id": "evt_00000000-0000-7000-8000-000000000099",
                "content": "challenging nothing",
            }),
        ),
    )
    .await;
    assert_eq!(status, 400, "missing target: {missing_target}");
    assert_eq!(missing_target["code"], json!("invalid_command"));
}

/// `thread.cancel` (`PHASE-1.1.3`): the abandonment terminal lands on the core
/// machine's `open → cancelled` edge, is inspectable through the API only, records
/// its reason, and refuses both a second cancel and any content verb afterwards.
#[tokio::test]
async fn cancel_is_inspectable_terminal_and_audited() {
    let _guard = api_guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (_, alice) = enroll(&client, &base, json!({ "kind": "human", "name": "alice" })).await;
    let tenant = alice["tenant_id"].as_str().unwrap().to_string();
    let alice_id = alice["principal_id"].as_str().unwrap().to_string();

    let (status, created) = command(
        &client,
        &base,
        "/v1/threads",
        &alice_id,
        &envelope(
            "thread.create",
            "k-cancel-create",
            json!({ "tenant_id": tenant, "subject": "doomed", "objective": "to be cancelled" }),
        ),
    )
    .await;
    assert_eq!(status, 200, "create: {created}");
    let thread_id = created["thread_id"].as_str().unwrap().to_string();
    let path = format!("/v1/threads/{thread_id}/commands");

    let (status, cancelled) = command(
        &client,
        &base,
        &path,
        &alice_id,
        &envelope(
            "thread.cancel",
            "k-cancel",
            json!({ "tenant_id": tenant, "reason": "no longer needed" }),
        ),
    )
    .await;
    assert_eq!(status, 200, "cancel: {cancelled}");
    assert_eq!(cancelled["thread_state"], json!("cancelled"));

    // Inspect through the API: the projection carries the terminal state + reason.
    let (status, inspected) = get(
        &client,
        &base,
        &format!("/v1/threads/{thread_id}?tenant_id={tenant}"),
        &alice_id,
    )
    .await;
    assert_eq!(status, 200, "inspect: {inspected}");
    assert_eq!(inspected["state"]["state"], json!("cancelled"));
    assert_eq!(
        inspected["state"]["cancel_reason"],
        json!("no longer needed")
    );
    assert_eq!(
        inspected["state"]["close_reason"],
        json!(null),
        "cancel is not a close"
    );

    // A second cancel is a deterministic invalid transition.
    let (status, again) = command(
        &client,
        &base,
        &path,
        &alice_id,
        &envelope(
            "thread.cancel",
            "k-cancel-again",
            json!({ "tenant_id": tenant, "reason": "twice" }),
        ),
    )
    .await;
    assert_eq!(status, 409, "second cancel: {again}");
    assert_eq!(again["code"], json!("invalid_transition"));

    // Content verbs are refused on a cancelled thread.
    let (status, contribution) = command(
        &client,
        &base,
        &path,
        &alice_id,
        &envelope(
            "thread.contribute",
            "k-cancel-contribute",
            json!({ "tenant_id": tenant, "content": "too late" }),
        ),
    )
    .await;
    assert_eq!(status, 409, "contribute after cancel: {contribution}");
    assert_eq!(contribution["code"], json!("invalid_transition"));
}

/// The typed create fields (`PHASE-1.1.3`): classification / workflow profile /
/// participant rules land on the projection with deny-unknown typing, the stated
/// defaults apply when unnamed, and unknown or malformed values are rejected.
#[tokio::test]
async fn create_carries_typed_classification_profile_and_rules() {
    let _guard = api_guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (_, alice) = enroll(&client, &base, json!({ "kind": "human", "name": "alice" })).await;
    let tenant = alice["tenant_id"].as_str().unwrap().to_string();
    let alice_id = alice["principal_id"].as_str().unwrap().to_string();

    // All three fields named: the projection carries them verbatim.
    let (status, created) = command(
        &client,
        &base,
        "/v1/threads",
        &alice_id,
        &envelope(
            "thread.create",
            "k-profile-full",
            json!({
                "tenant_id": tenant,
                "subject": "profiled",
                "objective": "typed fields",
                "classification": "confidential",
                "workflow_profile": "critique",
                "participant_rules": { "allow_explicit_invites": true, "allow_join_requests": true },
            }),
        ),
    )
    .await;
    assert_eq!(status, 200, "create: {created}");
    let thread_id = created["thread_id"].as_str().unwrap().to_string();
    let (_, inspected) = get(
        &client,
        &base,
        &format!("/v1/threads/{thread_id}?tenant_id={tenant}"),
        &alice_id,
    )
    .await;
    assert_eq!(inspected["state"]["classification"], json!("confidential"));
    assert_eq!(inspected["state"]["workflow_profile"], json!("critique"));
    assert_eq!(
        inspected["state"]["participant_rules"]["allow_explicit_invites"],
        json!(true)
    );
    assert_eq!(
        inspected["state"]["participant_rules"]["allow_join_requests"],
        json!(true)
    );

    // Unnamed fields take the STATED defaults: general / quick_advice /
    // explicit-invites-only.
    let (status, defaulted) = command(
        &client,
        &base,
        "/v1/threads",
        &alice_id,
        &envelope(
            "thread.create",
            "k-profile-defaults",
            json!({ "tenant_id": tenant, "subject": "plain", "objective": "defaults" }),
        ),
    )
    .await;
    assert_eq!(status, 200, "create defaults: {defaulted}");
    let thread_id = defaulted["thread_id"].as_str().unwrap().to_string();
    let (_, inspected) = get(
        &client,
        &base,
        &format!("/v1/threads/{thread_id}?tenant_id={tenant}"),
        &alice_id,
    )
    .await;
    assert_eq!(inspected["state"]["classification"], json!("general"));
    assert_eq!(
        inspected["state"]["workflow_profile"],
        json!("quick_advice")
    );
    assert_eq!(
        inspected["state"]["participant_rules"]["allow_explicit_invites"],
        json!(true)
    );
    assert_eq!(
        inspected["state"]["participant_rules"]["allow_join_requests"],
        json!(false)
    );

    // deny-unknown typing: a foreign create field is rejected.
    let (status, unknown) = command(
        &client,
        &base,
        "/v1/threads",
        &alice_id,
        &envelope(
            "thread.create",
            "k-profile-unknown",
            json!({
                "tenant_id": tenant,
                "subject": "typed",
                "objective": "rejection",
                "classifiction": "confidential",
            }),
        ),
    )
    .await;
    assert_eq!(status, 400, "unknown field: {unknown}");
    assert_eq!(unknown["code"], json!("invalid_command"));

    // An out-of-registry profile id is rejected, not silently stored.
    let (status, bad_enum) = command(
        &client,
        &base,
        "/v1/threads",
        &alice_id,
        &envelope(
            "thread.create",
            "k-profile-bad-enum",
            json!({
                "tenant_id": tenant,
                "subject": "typed",
                "objective": "rejection",
                "workflow_profile": "committee_of_everyone",
            }),
        ),
    )
    .await;
    assert_eq!(status, 400, "bad enum: {bad_enum}");
    assert_eq!(bad_enum["code"], json!("invalid_command"));
}

/// The structured contribution body (`PHASE-1.5.1`): the kind defaults to
/// `position`, named kinds and evidence references ride the event, and an
/// out-of-registry kind or a foreign evidence-ref field is a typed refusal.
#[tokio::test]
async fn contribute_carries_a_typed_kind_and_evidence_refs() {
    let _guard = api_guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (_, alice) = enroll(&client, &base, json!({ "kind": "human", "name": "alice" })).await;
    let tenant = alice["tenant_id"].as_str().unwrap().to_string();
    let alice_id = alice["principal_id"].as_str().unwrap().to_string();

    let (status, created) = command(
        &client,
        &base,
        "/v1/threads",
        &alice_id,
        &envelope(
            "thread.create",
            "k-structured-create",
            json!({
                "tenant_id": tenant,
                "subject": "structured",
                "objective": "typed kinds",
            }),
        ),
    )
    .await;
    assert_eq!(status, 200, "create: {created}");
    let thread_id = created["thread_id"].as_str().unwrap().to_string();
    let path = format!("/v1/threads/{thread_id}/commands");

    // The stated default: no kind named means `position`.
    let (status, plain) = command(
        &client,
        &base,
        &path,
        &alice_id,
        &envelope(
            "thread.contribute",
            "k-structured-plain",
            json!({ "tenant_id": tenant, "content": "no kind named" }),
        ),
    )
    .await;
    assert_eq!(status, 200, "plain contribute: {plain}");

    // A named kind and an evidence reference ride the event.
    let (status, named) = command(
        &client,
        &base,
        &path,
        &alice_id,
        &envelope(
            "thread.contribute",
            "k-structured-named",
            json!({
                "tenant_id": tenant,
                "content": "named",
                "kind": "evidence_reference",
                "evidence_refs": [{ "uri": "https://example.org/spec" }],
            }),
        ),
    )
    .await;
    assert_eq!(status, 200, "named contribute: {named}");

    let (status, events) = get(
        &client,
        &base,
        &format!("/v1/threads/{thread_id}/events?tenant_id={tenant}"),
        &alice_id,
    )
    .await;
    assert_eq!(status, 200, "inspect events: {events}");
    let contributions: Vec<Value> = events["events"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["event_type"] == json!("thread.contribution_submitted"))
        .cloned()
        .collect();
    assert_eq!(contributions.len(), 2);
    assert_eq!(contributions[0]["body"]["kind"], json!("position"));
    assert_eq!(contributions[0]["body"]["evidence_refs"], json!([]));
    assert_eq!(
        contributions[1]["body"]["kind"],
        json!("evidence_reference")
    );
    assert_eq!(
        contributions[1]["body"]["evidence_refs"],
        json!([{ "uri": "https://example.org/spec" }])
    );

    // An out-of-registry kind is refused, never silently stored.
    let (status, bad_kind) = command(
        &client,
        &base,
        &path,
        &alice_id,
        &envelope(
            "thread.contribute",
            "k-structured-bad",
            json!({ "tenant_id": tenant, "content": "bad", "kind": "essay" }),
        ),
    )
    .await;
    assert_eq!(status, 400, "bad kind: {bad_kind}");
    assert_eq!(bad_kind["code"], json!("invalid_command"));

    // A foreign field on an evidence ref is refused too (deny-unknown at the ref).
    let (status, bad_ref) = command(
        &client,
        &base,
        &path,
        &alice_id,
        &envelope(
            "thread.contribute",
            "k-structured-bad-ref",
            json!({
                "tenant_id": tenant,
                "content": "bad ref",
                "evidence_refs": [{ "uri": "https://example.org/x", "password": "nope" }],
            }),
        ),
    )
    .await;
    assert_eq!(status, 400, "bad evidence ref: {bad_ref}");
    assert_eq!(bad_ref["code"], json!("invalid_command"));
}

/// Rounds (`PHASE-1.5.2`): SERVER-assigned — a new thread is round 1,
/// contributions land in the current round, `thread.advance_round` moves it
/// (event + projection), a role without the grant is refused, and a closed
/// thread refuses advancement.
#[tokio::test]
async fn rounds_are_server_assigned_and_advancement_is_gated() {
    let _guard = api_guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (_, alice) = enroll(&client, &base, json!({ "kind": "human", "name": "alice" })).await;
    let tenant = alice["tenant_id"].as_str().unwrap().to_string();
    let alice_id = alice["principal_id"].as_str().unwrap().to_string();
    let (_, reviewer) = enroll(
        &client,
        &base,
        json!({ "kind": "role", "name": "reviewer", "tenant_id": tenant }),
    )
    .await;
    let reviewer_id = reviewer["principal_id"].as_str().unwrap().to_string();

    let (status, created) = command(
        &client,
        &base,
        "/v1/threads",
        &alice_id,
        &envelope(
            "thread.create",
            "k-rounds-create",
            json!({ "tenant_id": tenant, "subject": "rounds", "objective": "typed rounds" }),
        ),
    )
    .await;
    assert_eq!(status, 200, "create: {created}");
    let thread_id = created["thread_id"].as_str().unwrap().to_string();
    let path = format!("/v1/threads/{thread_id}/commands");

    let (status, invited) = command(
        &client,
        &base,
        &path,
        &alice_id,
        &envelope(
            "thread.invite",
            "k-rounds-invite",
            json!({ "tenant_id": tenant, "agent_role": reviewer_id }),
        ),
    )
    .await;
    assert_eq!(status, 200, "invite: {invited}");
    let (status, accepted) = command(
        &client,
        &base,
        &path,
        &reviewer_id,
        &envelope(
            "thread.accept_invitation",
            "k-rounds-accept",
            json!({ "tenant_id": tenant }),
        ),
    )
    .await;
    assert_eq!(status, 200, "accept: {accepted}");

    // A new thread is round 1; the contribution lands in round 1.
    let (status, contributed) = command(
        &client,
        &base,
        &path,
        &reviewer_id,
        &envelope(
            "thread.contribute",
            "k-rounds-contribute-1",
            json!({ "tenant_id": tenant, "content": "first round" }),
        ),
    )
    .await;
    assert_eq!(status, 200, "contribute round 1: {contributed}");

    // The role has no thread_advance_round grant (deny-by-default).
    let (status, denied) = command(
        &client,
        &base,
        &path,
        &reviewer_id,
        &envelope(
            "thread.advance_round",
            "k-rounds-role-advance",
            json!({ "tenant_id": tenant }),
        ),
    )
    .await;
    assert_eq!(status, 403, "role advance: {denied}");
    assert_eq!(denied["code"], json!("unauthorized"));

    // The human advances: round 2, the event names it, the projection records it.
    let (status, advanced) = command(
        &client,
        &base,
        &path,
        &alice_id,
        &envelope(
            "thread.advance_round",
            "k-rounds-advance",
            json!({ "tenant_id": tenant }),
        ),
    )
    .await;
    assert_eq!(status, 200, "advance: {advanced}");
    assert_eq!(advanced["event_type"], json!("thread.round_advanced"));

    let (_, inspected) = get(
        &client,
        &base,
        &format!("/v1/threads/{thread_id}?tenant_id={tenant}"),
        &alice_id,
    )
    .await;
    assert_eq!(inspected["state"]["current_round"], json!(2));

    // The next contribution lands in round 2; the events view shows both rounds.
    let (status, contributed2) = command(
        &client,
        &base,
        &path,
        &reviewer_id,
        &envelope(
            "thread.contribute",
            "k-rounds-contribute-2",
            json!({ "tenant_id": tenant, "content": "second round" }),
        ),
    )
    .await;
    assert_eq!(status, 200, "contribute round 2: {contributed2}");

    let (status, events) = get(
        &client,
        &base,
        &format!("/v1/threads/{thread_id}/events?tenant_id={tenant}"),
        &alice_id,
    )
    .await;
    assert_eq!(status, 200, "inspect events: {events}");
    let rounds: Vec<u64> = events["events"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["event_type"] == json!("thread.contribution_submitted"))
        .map(|e| e["body"]["round"].as_u64().unwrap())
        .collect();
    assert_eq!(
        rounds,
        vec![1, 2],
        "contributions carry the round they landed in"
    );

    // A closed thread refuses advancement (the state machine boundary).
    let (status, closed) = command(
        &client,
        &base,
        &path,
        &alice_id,
        &envelope(
            "thread.close",
            "k-rounds-close",
            json!({ "tenant_id": tenant, "reason": "done" }),
        ),
    )
    .await;
    assert_eq!(status, 200, "close: {closed}");
    let (status, late) = command(
        &client,
        &base,
        &path,
        &alice_id,
        &envelope(
            "thread.advance_round",
            "k-rounds-late-advance",
            json!({ "tenant_id": tenant }),
        ),
    )
    .await;
    assert_eq!(status, 409, "advance after close: {late}");
    assert_eq!(late["code"], json!("invalid_transition"));
}

/// The honest close (`PHASE-1.5.3`): `outcome: inconclusive` lands the thread on
/// the core `Inconclusive` terminal with the unresolved register riding the
/// event; a decided close carrying unresolved items is a typed refusal; the
/// inconclusive terminal refuses further content verbs.
#[tokio::test]
async fn the_honest_inconclusive_close_carries_its_register() {
    let _guard = api_guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (_, alice) = enroll(&client, &base, json!({ "kind": "human", "name": "alice" })).await;
    let tenant = alice["tenant_id"].as_str().unwrap().to_string();
    let alice_id = alice["principal_id"].as_str().unwrap().to_string();

    let (status, created) = command(
        &client,
        &base,
        "/v1/threads",
        &alice_id,
        &envelope(
            "thread.create",
            "k-honest-create",
            json!({ "tenant_id": tenant, "subject": "honest", "objective": "no decision" }),
        ),
    )
    .await;
    assert_eq!(status, 200, "create: {created}");
    let thread_id = created["thread_id"].as_str().unwrap().to_string();
    let path = format!("/v1/threads/{thread_id}/commands");

    let (status, closed) = command(
        &client,
        &base,
        &path,
        &alice_id,
        &envelope(
            "thread.close",
            "k-honest-close",
            json!({
                "tenant_id": tenant,
                "reason": "the evidence did not converge",
                "outcome": "inconclusive",
                "unresolved": ["the budget gate blocked the revision", "the challenge stands"],
            }),
        ),
    )
    .await;
    assert_eq!(status, 200, "inconclusive close: {closed}");
    assert_eq!(closed["event_type"], json!("thread.closed"));
    assert_eq!(closed["thread_state"], json!("inconclusive"));

    // The register rides the event; the inspection view shows the terminal.
    let (status, events) = get(
        &client,
        &base,
        &format!("/v1/threads/{thread_id}/events?tenant_id={tenant}"),
        &alice_id,
    )
    .await;
    assert_eq!(status, 200, "inspect events: {events}");
    let close_event = events["events"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["event_type"] == json!("thread.closed"))
        .expect("the close event exists");
    // `.2.4.1`: the legacy wire word is the accepted alias — the event
    // persists the CANONICAL terminal (the aliases never persist).
    assert_eq!(close_event["body"]["outcome"], json!("deadlocked"));
    assert_eq!(
        close_event["body"]["unresolved"],
        json!([
            "the budget gate blocked the revision",
            "the challenge stands"
        ])
    );

    let (_, inspected) = get(
        &client,
        &base,
        &format!("/v1/threads/{thread_id}?tenant_id={tenant}"),
        &alice_id,
    )
    .await;
    assert_eq!(inspected["state"]["state"], json!("inconclusive"));

    // The inconclusive terminal refuses further content verbs.
    let (status, late) = command(
        &client,
        &base,
        &path,
        &alice_id,
        &envelope(
            "thread.contribute",
            "k-honest-late",
            json!({ "tenant_id": tenant, "content": "too late" }),
        ),
    )
    .await;
    assert_eq!(status, 409, "contribute after inconclusive: {late}");
    assert_eq!(late["code"], json!("invalid_transition"));

    // A DECIDED close that carries unresolved items is dishonest — typed refusal.
    let (status, created2) = command(
        &client,
        &base,
        "/v1/threads",
        &alice_id,
        &envelope(
            "thread.create",
            "k-honest-create-2",
            json!({ "tenant_id": tenant, "subject": "decided", "objective": "decision" }),
        ),
    )
    .await;
    assert_eq!(status, 200, "create 2: {created2}");
    let thread2 = created2["thread_id"].as_str().unwrap().to_string();
    let (status, dishonest) = command(
        &client,
        &base,
        &format!("/v1/threads/{thread2}/commands"),
        &alice_id,
        &envelope(
            "thread.close",
            "k-honest-dishonest",
            json!({
                "tenant_id": tenant,
                "reason": "decided, supposedly",
                "outcome": "decided",
                "unresolved": ["something still open"],
            }),
        ),
    )
    .await;
    assert_eq!(status, 400, "decided with unresolved: {dishonest}");
    assert_eq!(dishonest["code"], json!("invalid_command"));
}

/// The `.1.6.1` budget read surface: the ledger (ceiling + every reservation row —
/// held vs settled usage, denials with their reasons) is inspectable through the
/// API only, the §26.1 "spend and uncertainty are visible" fact — and the inspect
/// gate holds (a role without `thread_inspect` is a typed 403 with the audit row).
#[tokio::test]
async fn budget_read_surface_exposes_the_ledger_through_the_api_only() {
    let _guard = api_guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    // Bootstrap: alice (human admin), reviewer + skeptic (roles).
    let (status, alice) = enroll(&client, &base, json!({ "kind": "human", "name": "alice" })).await;
    assert_eq!(status, 200, "enroll alice: {alice}");
    let tenant = alice["tenant_id"].as_str().unwrap().to_string();
    let alice_id = alice["principal_id"].as_str().unwrap().to_string();
    let (status, reviewer) = enroll(
        &client,
        &base,
        json!({ "kind": "role", "name": "reviewer", "tenant_id": tenant }),
    )
    .await;
    assert_eq!(status, 200, "enroll reviewer: {reviewer}");
    let reviewer_id = reviewer["principal_id"].as_str().unwrap().to_string();
    let (status, skeptic) = enroll(
        &client,
        &base,
        json!({ "kind": "role", "name": "skeptic", "tenant_id": tenant }),
    )
    .await;
    assert_eq!(status, 200, "enroll skeptic: {skeptic}");
    let skeptic_id = skeptic["principal_id"].as_str().unwrap().to_string();

    // A thread whose ceiling meters ONE call: the first accept's dispatch reserves
    // it; the second accept's dispatch is DENIED — and the denial is a ledger row
    // the read surface must show (the §14.3 "denials are rows too" invariant).
    let (status, created) = command(
        &client,
        &base,
        "/v1/threads",
        &alice_id,
        &envelope(
            "thread.create",
            "k-budget-create",
            json!({
                "tenant_id": tenant,
                "subject": "budgeted deliberation",
                "objective": "prove spend is visible",
                "budget": { "calls": 1 },
            }),
        ),
    )
    .await;
    assert_eq!(status, 200, "create: {created}");
    let thread_id = created["thread_id"].as_str().unwrap().to_string();

    for (role, invite_key) in [
        (&reviewer_id, "k-budget-invite-a"),
        (&skeptic_id, "k-budget-invite-b"),
    ] {
        let (status, invited) = command(
            &client,
            &base,
            &format!("/v1/threads/{thread_id}/commands"),
            &alice_id,
            &envelope(
                "thread.invite",
                invite_key,
                json!({ "tenant_id": tenant, "agent_role": role }),
            ),
        )
        .await;
        assert_eq!(status, 200, "invite: {invited}");
        let (status, accepted) = command(
            &client,
            &base,
            &format!("/v1/threads/{thread_id}/commands"),
            role,
            &envelope(
                "thread.accept_invitation",
                &format!("{invite_key}-accept"),
                json!({ "tenant_id": tenant }),
            ),
        )
        .await;
        assert_eq!(status, 200, "accept: {accepted}");
    }

    // The read: the ceiling + one active hold + one denial with its reason.
    let (status, budget) = get(
        &client,
        &base,
        &format!("/v1/threads/{thread_id}/budget?tenant_id={tenant}"),
        &alice_id,
    )
    .await;
    assert_eq!(status, 200, "budget get: {budget}");
    assert_eq!(budget["thread_id"], json!(thread_id));
    assert_eq!(budget["ceiling"]["dimensions"]["calls"], json!(1));
    assert!(budget["ceiling"]["policy_version"].is_string());
    let rows = budget["reservations"]
        .as_array()
        .expect("reservations is an array");
    assert_eq!(rows.len(), 2, "one hold + one denial: {budget}");
    let active: Vec<&Value> = rows.iter().filter(|r| r["status"] == "active").collect();
    let denied: Vec<&Value> = rows.iter().filter(|r| r["status"] == "denied").collect();
    assert_eq!(active.len(), 1, "exactly one active hold: {budget}");
    assert_eq!(denied.len(), 1, "exactly one recorded denial: {budget}");
    assert!(
        denied[0]["reason"]
            .as_str()
            .unwrap_or("")
            .contains("the ceiling does not cover"),
        "the denial carries the budget engine's reason: {budget}"
    );
    // Absent optional facts are OMITTED, never null (the `.1.5.1` wire shape).
    assert!(
        !denied[0].as_object().unwrap().contains_key("usage"),
        "denied rows carry no usage key: {budget}"
    );

    // Settle the active hold through the real write path; the read shows the usage.
    let reservation_id = active[0]["reservation_id"].as_str().unwrap().to_string();
    reasonbraid_server::settle_reservation(
        &pool,
        &reservation_id,
        &reasonbraid_core::BudgetDimensions::attempt_usage(Some(50), Some(100)),
        chrono::Utc::now(),
    )
    .await
    .expect("settle the active reservation");
    let (status, after) = get(
        &client,
        &base,
        &format!("/v1/threads/{thread_id}/budget?tenant_id={tenant}"),
        &alice_id,
    )
    .await;
    assert_eq!(status, 200, "budget get after settle: {after}");
    let settled: Vec<&Value> = after["reservations"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| r["status"] == "settled")
        .collect();
    assert_eq!(settled.len(), 1, "the hold reads settled: {after}");
    assert_eq!(settled[0]["usage"]["input_tokens"], json!(50));
    assert_eq!(settled[0]["usage"]["output_tokens"], json!(100));
    assert!(settled[0]["settled_at"].is_string(), "settled_at: {after}");

    // The inspect gate: a role without `thread_inspect` is a typed 403 (audit row
    // inside the message) — the UI inherits this gate unchanged.
    let (status, role_denied) = get(
        &client,
        &base,
        &format!("/v1/threads/{thread_id}/budget?tenant_id={tenant}"),
        &skeptic_id,
    )
    .await;
    assert_eq!(status, 403, "role budget read: {role_denied}");
    assert_eq!(role_denied["code"], json!("unauthorized"));
    assert!(
        role_denied["message"]
            .as_str()
            .unwrap_or("")
            .contains("authorization denied"),
        "the denial names the audit record: {role_denied}"
    );
}

/// THE `.1.3.2` acceptance: revoking a grant refuses the subject's NEXT
/// authorization while other principals keep working; the refusal is audited,
/// the state is inspectable through the admin list, and the refusals are typed.
#[tokio::test]
async fn revoking_a_grant_refuses_its_next_authorization_while_others_work() {
    let _guard = api_guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, alice) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "rev-alice" }),
    )
    .await;
    assert_eq!(status, 200, "enroll alice: {alice}");
    let tenant = alice["tenant_id"].as_str().unwrap().to_string();
    let alice_id = alice["principal_id"].as_str().unwrap().to_string();
    let (status, role) = enroll(
        &client,
        &base,
        json!({ "kind": "role", "name": "rev-agent", "tenant_id": tenant }),
    )
    .await;
    assert_eq!(status, 200, "enroll role: {role}");
    let role_id = role["principal_id"].as_str().unwrap().to_string();
    let role_grant = role["grant_id"].as_str().unwrap().to_string();

    // A thread the role can act in: the human creates, the role contributes.
    let (status, created) = command(
        &client,
        &base,
        "/v1/threads",
        &alice_id,
        &envelope(
            "thread.create",
            "key-rev-c1",
            json!({ "tenant_id": tenant, "subject": "rev probe", "objective": "probe" }),
        ),
    )
    .await;
    assert_eq!(status, 200, "create: {created}");
    let thread = created["thread_id"].as_str().unwrap().to_string();

    // The explicit-participants contract (`.1.3`): the human invites, the role
    // accepts — only then may the role act.
    let (status, invited) = command(
        &client,
        &base,
        &format!("/v1/threads/{thread}/commands"),
        &alice_id,
        &envelope(
            "thread.invite",
            "key-rev-inv",
            json!({ "tenant_id": tenant, "agent_role": role_id }),
        ),
    )
    .await;
    assert_eq!(status, 200, "invite: {invited}");
    let (status, accepted) = command(
        &client,
        &base,
        &format!("/v1/threads/{thread}/commands"),
        &role_id,
        &envelope(
            "thread.accept_invitation",
            "key-rev-acc",
            json!({ "tenant_id": tenant }),
        ),
    )
    .await;
    assert_eq!(status, 200, "accept: {accepted}");

    let (status, contributed) = command(
        &client,
        &base,
        &format!("/v1/threads/{thread}/commands"),
        &role_id,
        &envelope(
            "thread.contribute",
            "key-rev-c2",
            json!({ "tenant_id": tenant, "content": "before the revocation" }),
        ),
    )
    .await;
    assert_eq!(
        status, 200,
        "the role contributes before the revocation: {contributed}"
    );

    // Revoke the role's grant (the human admin).
    let (status, revoked) = admin_revoke(
        &client,
        &base,
        &format!("/v1/admin/grants/{role_grant}/revoke"),
        &alice_id,
        &tenant,
    )
    .await;
    assert_eq!(status, 200, "the revocation succeeds: {revoked}");

    // The role's NEXT command is refused 403 (the evaluation's active filter)
    // and the refusal is audited.
    let (status, refused) = command(
        &client,
        &base,
        &format!("/v1/threads/{thread}/commands"),
        &role_id,
        &envelope(
            "thread.contribute",
            "key-rev-c3",
            json!({ "tenant_id": tenant, "content": "after the revocation" }),
        ),
    )
    .await;
    assert_eq!(
        status, 403,
        "the revoked grant refuses the next authorization: {refused}"
    );
    let (n_denied,): (i64,) = sqlx::query_as(
        "SELECT count(*) FROM authorization_records WHERE tenant_id = $1 AND decision = 'denied'",
    )
    .bind(&tenant)
    .fetch_one(&pool)
    .await
    .expect("count denials");
    assert!(n_denied >= 1, "the refusal is audited");

    // The human's own authority is untouched.
    let (status, listed) = get(
        &client,
        &base,
        &format!("/v1/threads/{thread}?tenant_id={tenant}"),
        &alice_id,
    )
    .await;
    assert_eq!(status, 200, "the human still inspects: {listed}");

    // The inspection surface shows the revoked status.
    let (status, grants) = get(
        &client,
        &base,
        &format!("/v1/admin/grants?tenant_id={tenant}"),
        &alice_id,
    )
    .await;
    assert_eq!(status, 200);
    let row = grants["grants"]
        .as_array()
        .unwrap()
        .iter()
        .find(|g| g["grant_id"] == json!(role_grant))
        .expect("the role's grant is listed");
    assert_eq!(row["status"], json!("revoked"));

    // Typed refusals: an unknown grant is 404; a second revocation is 409.
    let (status, unknown) = admin_revoke(
        &client,
        &base,
        "/v1/admin/grants/grt_nope/revoke",
        &alice_id,
        &tenant,
    )
    .await;
    assert_eq!(status, 404, "an unknown grant: {unknown}");
    let (status, again) = admin_revoke(
        &client,
        &base,
        &format!("/v1/admin/grants/{role_grant}/revoke"),
        &alice_id,
        &tenant,
    )
    .await;
    assert_eq!(status, 409, "a second revocation: {again}");
    let epoch: i64 =
        sqlx::query_scalar("SELECT revocation_epoch FROM tenants WHERE tenant_id = $1")
            .bind(&tenant)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(
        epoch, 1,
        "unknown and repeated targets cannot advance the epoch"
    );
}

/// The admin revoke POST (tenant_admin-audited server-side).
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

async fn assert_foreign_revocation_is_inert(boundary: bool) {
    let _guard = api_guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();
    let (status, attacker) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "revocation-attacker" }),
    )
    .await;
    assert_eq!(status, 200, "attacker enrollment: {attacker}");
    let (status, victim) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "revocation-victim" }),
    )
    .await;
    assert_eq!(status, 200, "victim enrollment: {victim}");
    let victim_tenant = victim["tenant_id"].as_str().unwrap();
    let (status, role) = enroll(
        &client,
        &base,
        json!({ "kind": "role", "name": "victim-role", "tenant_id": victim_tenant }),
    )
    .await;
    assert_eq!(status, 200, "victim role enrollment: {role}");
    let (collection, table, column, target) = if boundary {
        (
            "boundaries",
            "enrollment_boundaries",
            "boundary_id",
            victim["boundary_id"].as_str().unwrap(),
        )
    } else {
        (
            "grants",
            "authority_grants",
            "grant_id",
            role["grant_id"].as_str().unwrap(),
        )
    };
    // SQL identifiers come only from the two constant branches above.
    let snapshot_sql = format!(
        "SELECT a.status, t.revocation_epoch, \
         (SELECT COUNT(*) FROM authorization_records WHERE tenant_id = t.tenant_id) \
         FROM {table} a JOIN tenants t ON t.tenant_id = a.tenant_id WHERE a.{column} = $1"
    );
    let before: (String, i64, i64) = sqlx::query_as(&snapshot_sql)
        .bind(target)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(before.0, "active");
    let path = format!("/v1/admin/{collection}/{target}/revoke");
    let (status, refused) = admin_revoke(
        &client,
        &base,
        &path,
        attacker["principal_id"].as_str().unwrap(),
        attacker["tenant_id"].as_str().unwrap(),
    )
    .await;
    assert_eq!(status, 404, "foreign target stays hidden: {refused}");
    let after: (String, i64, i64) = sqlx::query_as(&snapshot_sql)
        .bind(target)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        after, before,
        "a foreign-target 404 must preserve victim status, epoch and authorization records"
    );

    // A real administrator in the target tenant can still revoke the same id.
    let (status, revoked) = admin_revoke(
        &client,
        &base,
        &path,
        victim["principal_id"].as_str().unwrap(),
        victim_tenant,
    )
    .await;
    assert_eq!(status, 200, "own-tenant revocation: {revoked}");
    let legitimate: (String, i64, i64) = sqlx::query_as(&snapshot_sql)
        .bind(target)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(legitimate, ("revoked".into(), before.1 + 1, before.2 + 1));
    let (status, repeated) = admin_revoke(
        &client,
        &base,
        &path,
        victim["principal_id"].as_str().unwrap(),
        victim_tenant,
    )
    .await;
    assert_eq!(
        status,
        if boundary { 403 } else { 409 },
        "a frozen boundary or already revoked grant refuses: {repeated}"
    );
    let repeated_state: (String, i64, i64) = sqlx::query_as(&snapshot_sql)
        .bind(target)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(repeated_state.0, "revoked");
    assert_eq!(repeated_state.1, legitimate.1, "refusal changes no epoch");
}

#[tokio::test]
async fn foreign_grant_revocation_leaves_victim_unchanged() {
    assert_foreign_revocation_is_inert(false).await;
}

#[tokio::test]
async fn foreign_boundary_revocation_leaves_victim_unchanged() {
    assert_foreign_revocation_is_inert(true).await;
}

#[tokio::test]
async fn concurrent_grant_revocations_transition_and_bump_the_epoch_once() {
    let _guard = api_guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();
    let (status, admin) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "concurrent-revoker" }),
    )
    .await;
    assert_eq!(status, 200, "admin enrollment: {admin}");
    let tenant = admin["tenant_id"].as_str().unwrap().to_owned();
    let principal = admin["principal_id"].as_str().unwrap().to_owned();
    let (status, role) = enroll(
        &client,
        &base,
        json!({ "kind": "role", "name": "concurrent-target", "tenant_id": tenant }),
    )
    .await;
    assert_eq!(status, 200, "role enrollment: {role}");
    let grant = role["grant_id"].as_str().unwrap();
    let path = format!("/v1/admin/grants/{grant}/revoke");

    // Hold the target until BOTH HTTP requests are waiting on database locks.
    // This proves contention instead of relying on scheduler timing.
    let mut blocker = pool.begin().await.unwrap();
    sqlx::query("SELECT grant_id FROM authority_grants WHERE grant_id = $1 FOR UPDATE")
        .bind(grant)
        .fetch_one(&mut *blocker)
        .await
        .unwrap();
    let blocker_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
        .fetch_one(&mut *blocker)
        .await
        .unwrap();
    let mut requests = Vec::new();
    for _ in 0..2 {
        let (client, base, path, principal, tenant) = (
            client.clone(),
            base.clone(),
            path.clone(),
            principal.clone(),
            tenant.clone(),
        );
        requests.push(tokio::spawn(async move {
            admin_revoke(&client, &base, &path, &principal, &tenant).await
        }));
    }
    // The first writer waits on the target; the second waits on its tenant
    // guard. Require both real dependencies rooted at our held target row.
    let observed = tokio::time::timeout(std::time::Duration::from_secs(4), async {
        loop {
            let waiting: (i64, i64, i64) = sqlx::query_as(
                "WITH RECURSIVE waiting AS MATERIALIZED (\
                    SELECT pid, pg_blocking_pids(pid) AS blockers, query FROM pg_stat_activity \
                    WHERE datname = current_database() AND wait_event_type = 'Lock' \
                    AND (query LIKE '%authority_grants%' OR query LIKE '%tenant_authority_guards%')\
                 ), dependent(pid) AS (\
                    SELECT pid FROM waiting WHERE $1 = ANY(blockers) \
                    UNION SELECT w.pid FROM waiting w JOIN dependent d ON d.pid = ANY(w.blockers)\
                 ) SELECT count(*), count(*) FILTER (WHERE query LIKE '%authority_grants%'), \
                    count(*) FILTER (WHERE query LIKE '%tenant_authority_guards%') \
                 FROM waiting JOIN dependent USING (pid)",
            )
            .bind(blocker_pid)
            .fetch_one(&pool)
            .await
            .unwrap();
            if waiting == (2, 1, 1) {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    })
    .await;
    let graph: Value = sqlx::query_scalar(
        "SELECT coalesce(jsonb_agg(jsonb_build_object('pid',pid,'blockers',pg_blocking_pids(pid),'query',query)), '[]'::jsonb) FROM pg_stat_activity WHERE datname=current_database() AND wait_event_type='Lock'")
        .fetch_one(&pool).await.unwrap();
    blocker.rollback().await.unwrap();
    let mut joined = Vec::new();
    for mut request in requests {
        match tokio::time::timeout(std::time::Duration::from_secs(4), &mut request).await {
            Ok(result) => joined.push(result.map_err(|error| error.to_string())),
            Err(_) => {
                request.abort();
                let _ = request.await;
                joined.push(Err("revocation did not finish after target release".into()));
            }
        }
    }
    assert!(
        observed.is_ok(),
        "both revocations must depend on held target {blocker_pid}: {graph}"
    );
    let mut results: Vec<_> = joined
        .into_iter()
        .map(|result| result.expect("revocation joins"))
        .collect();
    results.sort_by_key(|result| result.0);
    assert_eq!(
        [results[0].0, results[1].0],
        [200, 409],
        "exactly one transition succeeds: {results:?}"
    );
    let state: (String, i64) = sqlx::query_as(
        "SELECT g.status, t.revocation_epoch FROM authority_grants g \
         JOIN tenants t ON t.tenant_id = g.tenant_id WHERE grant_id = $1",
    )
    .bind(grant)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(state, ("revoked".into(), 1));
}

/// Revoking the boundary suspends the CEILING: every grant under it is refused
/// at the next decision (the core's revoked-boundary stance) — the nuclear
/// option, inspectable through the admin list.
#[tokio::test]
async fn revoking_the_boundary_suspends_the_ceiling() {
    let _guard = api_guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, alice) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "rev-alice-2" }),
    )
    .await;
    assert_eq!(status, 200, "enroll alice: {alice}");
    let tenant = alice["tenant_id"].as_str().unwrap().to_string();
    let alice_id = alice["principal_id"].as_str().unwrap().to_string();
    let boundary_id = alice["boundary_id"].as_str().unwrap().to_string();

    let (status, revoked) = admin_revoke(
        &client,
        &base,
        &format!("/v1/admin/boundaries/{boundary_id}/revoke"),
        &alice_id,
        &tenant,
    )
    .await;
    assert_eq!(status, 200, "the boundary revocation succeeds: {revoked}");

    // The next authorization fails: the active-boundary lookup finds no ceiling.
    let (status, denied) = command(
        &client,
        &base,
        "/v1/threads",
        &alice_id,
        &envelope(
            "thread.create",
            "key-rev-b1",
            json!({ "tenant_id": tenant, "subject": "after the boundary revocation", "objective": "probe" }),
        ),
    )
    .await;
    assert_eq!(
        status, 403,
        "the revoked boundary refuses the next decision: {denied}"
    );

    // The inspection surface shows the status.
    let (status, boundaries) = get(
        &client,
        &base,
        &format!("/v1/admin/boundaries?tenant_id={tenant}"),
        &alice_id,
    )
    .await;
    assert_eq!(status, 200);
    assert_eq!(boundaries["boundaries"][0]["status"], json!("revoked"));
}

const TENANT_INSPECTION_ROUTES: [&str; 7] = [
    "nodes/presence",
    "grants",
    "boundaries",
    "incarnations",
    "runs",
    "breakers",
    "usage",
];

const INSPECTION_RECEIPT_HEADER: &str = "x-reasonbraid-authorization";

async fn get_with_inspection_receipt(
    client: &reqwest::Client,
    base: &str,
    path: &str,
    principal: &str,
) -> (u16, Value, Option<String>) {
    let response = client
        .get(format!("{base}{path}"))
        .header(PRINCIPAL_HEADER, principal)
        .send()
        .await
        .unwrap();
    let status = response.status().as_u16();
    let receipt = response
        .headers()
        .get(INSPECTION_RECEIPT_HEADER)
        .map(|value| value.to_str().unwrap().to_string());
    let text = response.text().await.unwrap();
    let body = serde_json::from_str(&text).unwrap_or(Value::String(text));
    (status, body, receipt)
}

#[tokio::test]
async fn authorization_receipt_lookup_is_tenant_scoped_and_audits_exact_named_reads() {
    let _guard = api_guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();
    let (status, owner) = enroll(
        &client,
        &base,
        json!({"kind":"human","name":"lookup-owner"}),
    )
    .await;
    assert_eq!(status, 200, "{owner}");
    let tenant = owner["tenant_id"].as_str().unwrap();
    let principal = owner["principal_id"].as_str().unwrap();
    let (status, role) = enroll(
        &client,
        &base,
        json!({"kind":"role","name":"lookup-role","tenant_id":tenant}),
    )
    .await;
    assert_eq!(status, 200, "{role}");
    let (status, outsider) = enroll(
        &client,
        &base,
        json!({"kind":"human","name":"lookup-outsider"}),
    )
    .await;
    assert_eq!(status, 200, "{outsider}");
    let foreign_tenant = outsider["tenant_id"].as_str().unwrap();
    let foreign_principal = outsider["principal_id"].as_str().unwrap();
    sqlx::query("UPDATE authority_grants SET actions = '[\"tenant_admin\"]' WHERE grant_id = $1")
        .bind(role["grant_id"].as_str().unwrap())
        .execute(&pool)
        .await
        .unwrap();
    let (status, _, requested) = get_with_inspection_receipt(
        &client,
        &base,
        &format!("/v1/admin/grants?tenant_id={tenant}"),
        principal,
    )
    .await;
    assert_eq!(status, 200);
    let requested = requested.unwrap();
    let expected = serde_json::to_value(
        reasonbraid_server::load_authorization_record(&pool, &requested)
            .await
            .unwrap()
            .unwrap(),
    )
    .unwrap();
    let path = format!("/v1/admin/authorization-records/{requested}?tenant_id={tenant}");
    let mut admissions = std::collections::BTreeSet::new();
    for boundary_status in ["active", "suspended", "revoked"] {
        sqlx::query("UPDATE enrollment_boundaries SET status = $1 WHERE tenant_id = $2")
            .bind(boundary_status)
            .bind(tenant)
            .execute(&pool)
            .await
            .unwrap();
        for (kind, identity) in [("human", &owner), ("role", &role)] {
            let caller = identity["principal_id"].as_str().unwrap();
            let before: i64 = sqlx::query_scalar("SELECT count(*) FROM authorization_records")
                .fetch_one(&pool)
                .await
                .unwrap();
            let (status, body, admission) =
                get_with_inspection_receipt(&client, &base, &path, caller).await;
            assert_eq!(status, 200, "{kind}/{boundary_status}: {body}");
            assert_eq!(body, json!({"tenant_id":tenant,"authorization":expected}));
            let admission = admission.expect("lookup names its own committed admission");
            assert_ne!(admission, requested);
            assert!(admissions.insert(admission.clone()));
            let record = reasonbraid_server::load_authorization_record(&pool, &admission)
                .await
                .unwrap()
                .unwrap();
            assert_eq!(record.decision, reasonbraid_core::Decision::Allowed);
            assert_eq!(
                serde_json::to_value(record.evaluation).unwrap(),
                json!({
                    "kind":"tenant_admin_inspection",
                    "principal":{"kind":kind,"id":caller},
                    "inspection":{"kind":"authorization_record","record_id":requested},
                    "boundary_status":boundary_status,
                    "grant_selector":{"kind":"tenant_wide"}
                })
            );
            let after: i64 = sqlx::query_scalar("SELECT count(*) FROM authorization_records")
                .fetch_one(&pool)
                .await
                .unwrap();
            assert_eq!(after, before + 1, "one admission, no recursive lookup");
        }
    }
    let (status, _, foreign_record) = get_with_inspection_receipt(
        &client,
        &base,
        &format!("/v1/admin/grants?tenant_id={foreign_tenant}"),
        foreign_principal,
    )
    .await;
    assert_eq!(status, 200);
    let foreign_record = foreign_record.unwrap();
    // Malformed foreign evidence must be filtered before strict decoding. Restore
    // it before asserting responses, so the fault cannot affect later fixtures.
    sqlx::query(
        "UPDATE authorization_records SET decision = 'owned-malformed' WHERE record_id = $1",
    )
    .bind(&foreign_record)
    .execute(&pool)
    .await
    .unwrap();
    let missing = reasonbraid_core::AuthorizationRecordId::new().to_string();
    let hidden = get_with_inspection_receipt(
        &client,
        &base,
        &format!("/v1/admin/authorization-records/{foreign_record}?tenant_id={tenant}"),
        principal,
    )
    .await;
    let absent = get_with_inspection_receipt(
        &client,
        &base,
        &format!("/v1/admin/authorization-records/{missing}?tenant_id={tenant}"),
        principal,
    )
    .await;
    sqlx::query("UPDATE authorization_records SET decision = 'allowed' WHERE record_id = $1")
        .bind(&foreign_record)
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(hidden.0, 404, "{}", hidden.1);
    assert_eq!(absent.0, 404, "{}", absent.1);
    assert_eq!(
        hidden.1, absent.1,
        "foreign and absent evidence have one refusal"
    );
    assert_eq!(
        hidden.1,
        json!({"code":"not_found","message":"authorization record not found"})
    );
    for (receipt, target) in [(hidden.2, &foreign_record), (absent.2, &missing)] {
        let record = reasonbraid_server::load_authorization_record(&pool, &receipt.unwrap())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(record.tenant_id.to_string(), tenant);
        assert_eq!(record.decision, reasonbraid_core::Decision::Allowed);
        assert_eq!(
            serde_json::to_value(record.evaluation).unwrap()["inspection"],
            json!({"kind":"authorization_record","record_id":target})
        );
    }
    for target in [&requested, &missing] {
        let (status, body, receipt) = get_with_inspection_receipt(
            &client,
            &base,
            &format!("/v1/admin/authorization-records/{target}?tenant_id={tenant}"),
            foreign_principal,
        )
        .await;
        assert_eq!(status, 403, "{body}");
        assert!(body.get("authorization").is_none());
        let denied_receipt = receipt.unwrap();
        let record = reasonbraid_server::load_authorization_record(&pool, &denied_receipt)
            .await
            .unwrap()
            .unwrap();
        assert!(matches!(
            record.decision,
            reasonbraid_core::Decision::Denied { .. }
        ));
        assert_eq!(record.tenant_id.to_string(), tenant);
        // The tenant's administrator can inspect the denied attempt too. The
        // returned record keeps the original outsider and refusal unchanged.
        let (status, body, lookup_receipt) = get_with_inspection_receipt(
            &client,
            &base,
            &format!("/v1/admin/authorization-records/{denied_receipt}?tenant_id={tenant}"),
            principal,
        )
        .await;
        assert_eq!(status, 200, "{body}");
        assert_eq!(body["authorization"], serde_json::to_value(record).unwrap());
        assert_ne!(lookup_receipt.as_deref(), Some(denied_receipt.as_str()));
        assert!(lookup_receipt.is_some());
    }
    let (status, body, receipt) = get_with_inspection_receipt(
        &client,
        &base,
        &format!("/v1/admin/authorization-records/{requested}?tenant_id={foreign_tenant}"),
        foreign_principal,
    )
    .await;
    assert_eq!(status, 404, "{body}");
    assert_eq!(body, hidden.1);
    assert!(receipt.is_some());
    let unchanged = serde_json::to_value(
        reasonbraid_server::load_authorization_record(&pool, &requested)
            .await
            .unwrap()
            .unwrap(),
    )
    .unwrap();
    assert_eq!(
        unchanged, expected,
        "inspection never rewrites its source record"
    );
}

#[tokio::test]
async fn authorization_receipt_lookup_preserves_legacy_and_refuses_corrupt_evidence() {
    let _guard = api_guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();
    let (status, owner) = enroll(
        &client,
        &base,
        json!({"kind":"human","name":"legacy-lookup"}),
    )
    .await;
    assert_eq!(status, 200, "{owner}");
    let tenant = owner["tenant_id"].as_str().unwrap();
    let principal = owner["principal_id"].as_str().unwrap();
    let (status, _, requested) = get_with_inspection_receipt(
        &client,
        &base,
        &format!("/v1/admin/grants?tenant_id={tenant}"),
        principal,
    )
    .await;
    assert_eq!(status, 200);
    let requested = requested.unwrap();
    sqlx::query("UPDATE authorization_records SET evaluation = '{\"kind\":\"legacy_unspecified\"}' WHERE record_id = $1")
        .bind(&requested).execute(&pool).await.unwrap();
    let path = format!("/v1/admin/authorization-records/{requested}?tenant_id={tenant}");
    let (status, legacy, receipt) =
        get_with_inspection_receipt(&client, &base, &path, principal).await;
    assert_eq!(status, 200, "{legacy}");
    assert_eq!(
        legacy["authorization"]["evaluation"],
        json!({"kind":"legacy_unspecified"})
    );
    assert!(receipt.is_some());
    sqlx::query(
        "UPDATE authorization_records SET decision = 'owned-malformed' WHERE record_id = $1",
    )
    .bind(&requested)
    .execute(&pool)
    .await
    .unwrap();
    let (status, body, receipt) =
        get_with_inspection_receipt(&client, &base, &path, principal).await;
    sqlx::query("UPDATE authorization_records SET decision = 'allowed' WHERE record_id = $1")
        .bind(&requested)
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(status, 500, "{body}");
    assert_eq!(
        body,
        json!({"code":"dependency_unavailable","message":"internal server error"})
    );
    let admission = reasonbraid_server::load_authorization_record(&pool, &receipt.unwrap())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(admission.decision, reasonbraid_core::Decision::Allowed);
    assert_eq!(
        serde_json::to_value(admission.evaluation).unwrap()["inspection"],
        json!({"kind":"authorization_record","record_id":requested})
    );
    let (status, recovered, _) =
        get_with_inspection_receipt(&client, &base, &path, principal).await;
    assert_eq!(status, 200, "{recovered}");
    assert_eq!(recovered, legacy);
}

#[tokio::test]
async fn authorization_receipt_lookup_enforces_eligibility_and_extraction() {
    let _guard = api_guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();
    let (status, owner) = enroll(
        &client,
        &base,
        json!({"kind":"human","name":"ineligible-lookup"}),
    )
    .await;
    assert_eq!(status, 200, "{owner}");
    let tenant = owner["tenant_id"].as_str().unwrap();
    let principal = owner["principal_id"].as_str().unwrap();
    let grant = owner["grant_id"].as_str().unwrap();
    let (status, _, requested) = get_with_inspection_receipt(
        &client,
        &base,
        &format!("/v1/admin/grants?tenant_id={tenant}"),
        principal,
    )
    .await;
    assert_eq!(status, 200);
    let requested = requested.unwrap();
    let path = format!("/v1/admin/authorization-records/{requested}?tenant_id={tenant}");
    let (status, _, _) = get_with_inspection_receipt(&client, &base, &path, principal).await;
    assert_eq!(status, 200, "eligible positive control");
    for (grant_status, selector) in [
        ("revoked", json!({"kind":"tenant_wide"})),
        ("active", json!({"kind":"threads","threads":[]})),
    ] {
        sqlx::query("UPDATE authority_grants SET status = $1, selector = $2 WHERE grant_id = $3")
            .bind(grant_status)
            .bind(selector)
            .bind(grant)
            .execute(&pool)
            .await
            .unwrap();
        let (status, body, receipt) =
            get_with_inspection_receipt(&client, &base, &path, principal).await;
        assert_eq!(status, 403, "{body}");
        assert!(body.get("authorization").is_none());
        let admission = reasonbraid_server::load_authorization_record(&pool, &receipt.unwrap())
            .await
            .unwrap()
            .unwrap();
        assert!(matches!(
            admission.decision,
            reasonbraid_core::Decision::Denied { .. }
        ));
    }
    sqlx::query("UPDATE authority_grants SET status = 'active', selector = '{\"kind\":\"tenant_wide\"}' WHERE grant_id = $1")
        .bind(grant).execute(&pool).await.unwrap();
    let before: i64 = sqlx::query_scalar("SELECT count(*) FROM authorization_records")
        .fetch_one(&pool)
        .await
        .unwrap();
    for (path, caller, expected_status) in [
        (
            format!("/v1/admin/authorization-records/not-a-record?tenant_id={tenant}"),
            principal,
            400,
        ),
        (
            format!("/v1/admin/authorization-records/{requested}?tenant_id=not-a-tenant"),
            principal,
            400,
        ),
        (format!("{path}&unknown=value"), principal, 400),
        (format!("{path}&tenant_id={tenant}"), principal, 400),
        (path.clone(), "not-a-principal", 401),
    ] {
        let (status, _, receipt) = get_with_inspection_receipt(&client, &base, &path, caller).await;
        assert_eq!(status, expected_status);
        assert!(receipt.is_none());
    }
    let after: i64 = sqlx::query_scalar("SELECT count(*) FROM authorization_records")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        after, before,
        "extraction failures cannot invent admissions"
    );
    let (status, body, _) = get_with_inspection_receipt(&client, &base, &path, principal).await;
    assert_eq!(status, 200, "restored eligible authority: {body}");
}

#[tokio::test]
async fn administrative_inspections_commit_named_human_role_and_denial_receipts() {
    let _guard = api_guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();
    let (status, owner) = enroll(
        &client,
        &base,
        json!({"kind":"human","name":"receipt-owner"}),
    )
    .await;
    assert_eq!(status, 200, "{owner}");
    let tenant = owner["tenant_id"].as_str().unwrap();
    let (status, role) = enroll(
        &client,
        &base,
        json!({"kind":"role","name":"receipt-role","tenant_id":tenant}),
    )
    .await;
    assert_eq!(status, 200, "{role}");
    let (status, outsider) = enroll(
        &client,
        &base,
        json!({"kind":"human","name":"receipt-outsider"}),
    )
    .await;
    assert_eq!(status, 200, "{outsider}");
    sqlx::query("UPDATE authority_grants SET actions = '[\"tenant_admin\"]' WHERE grant_id = $1")
        .bind(role["grant_id"].as_str().unwrap())
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("UPDATE enrollment_boundaries SET status = 'revoked' WHERE tenant_id = $1")
        .bind(tenant)
        .execute(&pool)
        .await
        .unwrap();
    let mut failures = Vec::new();
    let mut receipt_ids = std::collections::BTreeSet::new();
    for (kind, identity, allowed) in [
        ("human", &owner, true),
        ("role", &role, true),
        ("human", &outsider, false),
    ] {
        let principal = identity["principal_id"].as_str().unwrap();
        for route in TENANT_INSPECTION_ROUTES {
            let before: chrono::DateTime<chrono::Utc> =
                sqlx::query_scalar("SELECT clock_timestamp()")
                    .fetch_one(&pool)
                    .await
                    .unwrap();
            let (status, body, receipt) = get_with_inspection_receipt(
                &client,
                &base,
                &format!("/v1/admin/{route}?tenant_id={tenant}"),
                principal,
            )
            .await;
            assert_eq!(
                status,
                if allowed { 200 } else { 403 },
                "{kind}/{route}: {body}"
            );
            if !allowed {
                assert!(body.get("tenant_id").is_none());
            }
            let Some(receipt) = receipt else {
                failures.push(format!(
                    "{kind}/{route}/{allowed}: missing committed receipt"
                ));
                continue;
            };
            assert!(
                receipt_ids.insert(receipt.clone()),
                "one fresh admission per request"
            );
            // A separate pooled read sees the committed record after HTTP returns.
            let record = reasonbraid_server::load_authorization_record(&pool, &receipt)
                .await
                .unwrap()
                .expect("the response names a durable record");
            let after: chrono::DateTime<chrono::Utc> =
                sqlx::query_scalar("SELECT clock_timestamp()")
                    .fetch_one(&pool)
                    .await
                    .unwrap();
            assert!(before <= record.decided_at && record.decided_at <= after);
            assert_eq!(record.tenant_id.to_string(), tenant);
            assert_eq!(
                record.decision == reasonbraid_core::Decision::Allowed,
                allowed
            );
            assert!(
                record.subject.is_none(),
                "inspection is direct, never delegated"
            );
            assert_eq!(record.action, reasonbraid_core::GrantAction::TenantAdmin);
            assert_eq!(
                record.boundary_id.as_deref(),
                allowed.then(|| owner["boundary_id"].as_str().unwrap())
            );
            assert_eq!(
                record.grant_id.as_deref(),
                allowed.then(|| identity["grant_id"].as_str().unwrap())
            );
            assert_eq!(
                serde_json::to_value(record.evaluation).unwrap(),
                json!({
                    "kind":"tenant_admin_inspection",
                    "principal":{"kind":kind,"id":principal},
                    "inspection":{"kind":route.replace('/', "_")},
                    "boundary_status":if allowed { json!("revoked") } else { Value::Null },
                    "grant_selector":if allowed { json!({"kind":"tenant_wide"}) } else { Value::Null },
                })
            );
        }
    }
    let (status, _, receipt) = get_with_inspection_receipt(
        &client,
        &base,
        &format!("/v1/admin/grants?tenant_id={tenant}"),
        "not-a-principal",
    )
    .await;
    assert_eq!(status, 401);
    assert!(
        receipt.is_none(),
        "extraction cannot invent an admission receipt"
    );
    let count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM authorization_records WHERE tenant_id = $1")
            .bind(tenant)
            .fetch_one(&pool)
            .await
            .unwrap();
    if count != 21 {
        failures.push(format!("expected 21 committed admissions, got {count}"));
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[tokio::test]
async fn inspection_audit_failure_refuses_reads_and_recovers_without_fabricated_receipts() {
    let _guard = api_guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();
    let (status, owner) = enroll(
        &client,
        &base,
        json!({"kind":"human","name":"receipt-fault-owner"}),
    )
    .await;
    assert_eq!(status, 200, "{owner}");
    let (status, outsider) = enroll(
        &client,
        &base,
        json!({"kind":"human","name":"receipt-fault-outsider"}),
    )
    .await;
    assert_eq!(status, 200, "{outsider}");
    let tenant = owner["tenant_id"].as_str().unwrap();
    let principal = owner["principal_id"].as_str().unwrap();
    sqlx::query("CREATE FUNCTION rb_test_refuse_inspection_audit() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN IF NEW.evaluation->>'kind' = 'tenant_admin_inspection' THEN RAISE EXCEPTION 'owned inspection audit fault'; END IF; RETURN NEW; END $$")
        .execute(&pool).await.unwrap();
    sqlx::query("CREATE TRIGGER rb_test_inspection_audit_fault BEFORE INSERT ON authorization_records FOR EACH ROW EXECUTE FUNCTION rb_test_refuse_inspection_audit()")
        .execute(&pool).await.unwrap();
    // The fault is scoped to inspection evidence; ordinary authority still records
    // boundary_checked and a real protected state marker is available to the read.
    let response = client
        .post(format!("{base}/v1/admin/breakers"))
        .header(PRINCIPAL_HEADER, principal)
        .json(&json!({"tenant_id":tenant,"threshold":{"calls":10}}))
        .send()
        .await
        .unwrap();
    let normal_status = response.status().as_u16();
    let normal_body: Value = response.json().await.unwrap();
    let mut failures = Vec::new();
    if normal_status != 200 {
        failures.push(format!(
            "normal admission failed: {normal_status} {normal_body}"
        ));
    }
    let ordinary_record: String =
        sqlx::query_scalar("SELECT record_id FROM authorization_records WHERE tenant_id = $1")
            .bind(tenant)
            .fetch_one(&pool)
            .await
            .unwrap();
    let mut routes: Vec<String> = TENANT_INSPECTION_ROUTES
        .iter()
        .map(|route| (*route).into())
        .collect();
    routes.push(format!("authorization-records/{ordinary_record}"));
    for caller in [principal, outsider["principal_id"].as_str().unwrap()] {
        for route in &routes {
            let (status, body, receipt) = get_with_inspection_receipt(
                &client,
                &base,
                &format!("/v1/admin/{route}?tenant_id={tenant}"),
                caller,
            )
            .await;
            if status != 500
                || body["code"] != "dependency_unavailable"
                || body.get("tenant_id").is_some()
                || receipt.is_some()
            {
                failures.push(format!(
                    "audit fault {route}: {status}, receipt={receipt:?}"
                ));
            }
        }
    }
    let counts: (i64, i64) = sqlx::query_as("SELECT count(*), count(*) FILTER (WHERE evaluation->>'kind' = 'boundary_checked') FROM authorization_records WHERE tenant_id = $1")
        .bind(tenant).fetch_one(&pool).await.unwrap();
    if counts != (1, 1) {
        failures.push(format!("audit fault changed evidence: {counts:?}"));
    }
    // Restore the owned fault before assertions can finish this test.
    sqlx::query("DROP TRIGGER rb_test_inspection_audit_fault ON authorization_records")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("DROP FUNCTION rb_test_refuse_inspection_audit()")
        .execute(&pool)
        .await
        .unwrap();
    let (status, body, receipt) = get_with_inspection_receipt(
        &client,
        &base,
        &format!("/v1/admin/breakers?tenant_id={tenant}"),
        principal,
    )
    .await;
    assert_eq!(status, 200, "recovery: {body}");
    assert_eq!(body["breaker"]["threshold"]["calls"], 10);
    if receipt.is_none() {
        failures.push("recovery has no committed receipt".into());
    }
    let (status, body, receipt) = get_with_inspection_receipt(
        &client,
        &base,
        &format!("/v1/admin/authorization-records/{ordinary_record}?tenant_id={tenant}"),
        principal,
    )
    .await;
    if status != 200
        || receipt.is_none()
        || body["authorization"]["record_id"] != ordinary_record
        || body["authorization"]["evaluation"] != json!({"kind":"boundary_checked"})
    {
        failures.push(format!(
            "receipt lookup recovery: {status} {body}, receipt={receipt:?}"
        ));
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[tokio::test]
async fn failed_response_query_retains_its_real_committed_inspection_admission() {
    let _guard = api_guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();
    let (status, owner) = enroll(
        &client,
        &base,
        json!({"kind":"human","name":"read-delivery-fault"}),
    )
    .await;
    assert_eq!(status, 200, "{owner}");
    let tenant = owner["tenant_id"].as_str().unwrap();
    let principal = owner["principal_id"].as_str().unwrap();
    // The first runs query cannot resolve its column. Renaming retains all data;
    // restore it immediately after the response, before inspecting the outcome.
    sqlx::query("ALTER TABLE runs RENAME COLUMN created_at TO created_at_inspection_fault")
        .execute(&pool)
        .await
        .unwrap();
    let (status, body, receipt) = get_with_inspection_receipt(
        &client,
        &base,
        &format!("/v1/admin/runs?tenant_id={tenant}"),
        principal,
    )
    .await;
    sqlx::query("ALTER TABLE runs RENAME COLUMN created_at_inspection_fault TO created_at")
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(status, 500, "{body}");
    assert_eq!(body["code"], "dependency_unavailable");
    assert!(body.get("tenant_id").is_none());
    let receipt = receipt.expect("failed delivery still names its committed admission");
    let record = reasonbraid_server::load_authorization_record(&pool, &receipt)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(record.decision, reasonbraid_core::Decision::Allowed);
    assert_eq!(
        serde_json::to_value(record.evaluation).unwrap()["inspection"],
        json!({"kind":"runs"})
    );
    let (status, _, recovered_receipt) = get_with_inspection_receipt(
        &client,
        &base,
        &format!("/v1/admin/runs?tenant_id={tenant}"),
        principal,
    )
    .await;
    assert_eq!(status, 200);
    assert_ne!(recovered_receipt.as_deref(), Some(receipt.as_str()));
    assert!(recovered_receipt.is_some());
}

/// The approved freeze exception is a direct own-tenant read, never a write.
#[tokio::test]
async fn tenant_inspection_survives_only_boundary_status_freezes() {
    let _guard = api_guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();
    let (status, alice) =
        enroll(&client, &base, json!({"kind":"human", "name":"read-owner"})).await;
    assert_eq!(status, 200, "{alice}");
    let (status, bob) = enroll(
        &client,
        &base,
        json!({"kind":"human", "name":"read-outsider"}),
    )
    .await;
    assert_eq!(status, 200, "{bob}");
    let tenant = alice["tenant_id"].as_str().unwrap();
    let principal = alice["principal_id"].as_str().unwrap();
    let outsider = bob["principal_id"].as_str().unwrap();
    let grant = alice["grant_id"].as_str().unwrap();
    let mut active_responses = Vec::new();
    for boundary_status in ["active", "suspended", "revoked"] {
        sqlx::query("UPDATE enrollment_boundaries SET status = $2 WHERE tenant_id = $1")
            .bind(tenant)
            .bind(boundary_status)
            .execute(&pool)
            .await
            .unwrap();
        for (i, route) in TENANT_INSPECTION_ROUTES.iter().enumerate() {
            let path = format!("/v1/admin/{route}?tenant_id={tenant}");
            let (status, body) = get(&client, &base, &path, principal).await;
            assert_eq!(status, 200, "{boundary_status} {route}: {body}");
            assert_eq!(body["tenant_id"], tenant, "{route}: {body}");
            if boundary_status == "active" {
                active_responses.push(body);
            } else if *route == "boundaries" {
                assert_eq!(body["boundaries"][0]["status"], boundary_status);
                let mut expected = active_responses[i].clone();
                expected["boundaries"][0]["status"] = json!(boundary_status);
                assert_eq!(body, expected);
            } else {
                assert_eq!(body, active_responses[i], "response shape/data: {route}");
            }
            let (status, denied) = get(&client, &base, &path, outsider).await;
            assert_eq!(status, 403, "foreign admin {route}: {denied}");
            assert!(
                denied.get("tenant_id").is_none(),
                "no protected response: {denied}"
            );
        }
        if boundary_status != "active" {
            let (status, denied) = admin_revoke(
                &client,
                &base,
                &format!("/v1/admin/grants/{grant}/revoke"),
                principal,
                tenant,
            )
            .await;
            assert_eq!(status, 403, "a read exception cannot revoke: {denied}");
            let stored: String =
                sqlx::query_scalar("SELECT status FROM authority_grants WHERE grant_id = $1")
                    .bind(grant)
                    .fetch_one(&pool)
                    .await
                    .unwrap();
            assert_eq!(stored, "active", "the refused write has no grant effect");
        }
    }
}

/// Legacy-invalid storage must not turn the status-only exception into a bypass.
#[tokio::test]
async fn tenant_inspection_rejects_invalid_grants_and_actual_parents() {
    let _guard = api_guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();
    let (status, outsider) = enroll(
        &client,
        &base,
        json!({"kind":"human", "name":"foreign-parent"}),
    )
    .await;
    assert_eq!(status, 200, "{outsider}");
    let cases = [
        ("thread selector", "UPDATE authority_grants SET selector = '{\"kind\":\"threads\",\"threads\":[]}' WHERE tenant_id = $1", 403),
        ("foreign parent", "UPDATE authority_grants SET boundary_id = (SELECT boundary_id FROM enrollment_boundaries WHERE tenant_id <> $1 ORDER BY boundary_id LIMIT 1) WHERE tenant_id = $1", 403),
        ("widened actions", "UPDATE enrollment_boundaries SET permitted_actions = '[\"tenant_admin\"]' WHERE tenant_id = $1", 403),
        ("widened risk", "UPDATE authority_grants SET risk_ceiling = 'high' WHERE tenant_id = $1", 403),
        ("widened spend", "UPDATE authority_grants SET spend_limits = '{\"amount\":1001}' WHERE tenant_id = $1", 403),
        ("widened delegation", "UPDATE authority_grants SET delegable = true WHERE tenant_id = $1", 403),
        ("grant precedes parent", "UPDATE authority_grants SET valid_from = valid_from - interval '1 day' WHERE tenant_id = $1", 403),
        ("grant outlives parent", "UPDATE authority_grants SET expires_at = expires_at + interval '1 day' WHERE tenant_id = $1", 403),
        ("expired grant", "UPDATE authority_grants SET expires_at = now() - interval '1 day' WHERE tenant_id = $1", 403),
        ("future grant", "UPDATE authority_grants SET valid_from = now() + interval '1 day' WHERE tenant_id = $1", 403),
        ("revoked grant", "UPDATE authority_grants SET status = 'revoked' WHERE tenant_id = $1", 403),
        ("expired parent", "UPDATE enrollment_boundaries SET expires_at = now() - interval '1 day' WHERE tenant_id = $1", 403),
        ("future parent", "UPDATE enrollment_boundaries SET valid_from = now() + interval '1 day' WHERE tenant_id = $1", 403),
        ("malformed selector", "UPDATE authority_grants SET selector = 'null' WHERE tenant_id = $1", 500),
        ("malformed parent", "UPDATE enrollment_boundaries SET max_delegation_depth = -1 WHERE tenant_id = $1", 500),
    ];
    let mut failures = Vec::new();
    for (label, mutation, expected) in cases {
        let (status, owner) = enroll(&client, &base, json!({"kind":"human", "name":label})).await;
        assert_eq!(status, 200, "{owner}");
        let tenant = owner["tenant_id"].as_str().unwrap();
        let principal = owner["principal_id"].as_str().unwrap();
        let path = format!("/v1/admin/grants?tenant_id={tenant}");
        assert_eq!(
            get(&client, &base, &path, principal).await.0,
            200,
            "eligible control: {label}"
        );
        // Freeze the actual original parent before corrupting the fixture. The
        // foreign-parent case deliberately points at a different tenant's parent.
        sqlx::query("UPDATE enrollment_boundaries SET status = 'revoked' WHERE tenant_id = $1")
            .bind(tenant)
            .execute(&pool)
            .await
            .unwrap();
        assert_eq!(
            sqlx::query(mutation)
                .bind(tenant)
                .execute(&pool)
                .await
                .unwrap()
                .rows_affected(),
            1,
            "{label}"
        );
        for route in TENANT_INSPECTION_ROUTES {
            let path = format!("/v1/admin/{route}?tenant_id={tenant}");
            let (status, body) = get(&client, &base, &path, principal).await;
            if status != expected || body.get("tenant_id").is_some() {
                failures.push(format!(
                    "{label}/{route}: expected {expected} without protected data, got {status}"
                ));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[tokio::test]
async fn tenant_inspection_selects_an_eligible_older_grant() {
    let _guard = api_guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();
    let (status, owner) = enroll(
        &client,
        &base,
        json!({"kind":"human", "name":"read-fallback"}),
    )
    .await;
    assert_eq!(status, 200, "{owner}");
    let tenant = owner["tenant_id"].as_str().unwrap();
    let principal = owner["principal_id"].as_str().unwrap();
    // More than a candidate page, all newer but not yet valid.
    sqlx::query(
        "INSERT INTO authority_grants SELECT 'grt_read_future_' || n, boundary_id, tenant_id, \
         issuer, subject_kind, subject_id, actions, selector, risk_ceiling, spend_limits, \
         delegable, valid_from + interval '1 day', expires_at, status \
         FROM authority_grants CROSS JOIN generate_series(1, 35) n WHERE tenant_id = $1",
    )
    .bind(tenant)
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query("UPDATE enrollment_boundaries SET status = 'revoked' WHERE tenant_id = $1")
        .bind(tenant)
        .execute(&pool)
        .await
        .unwrap();
    let mut failures = Vec::new();
    for route in TENANT_INSPECTION_ROUTES {
        let path = format!("/v1/admin/{route}?tenant_id={tenant}");
        let (status, body) = get(&client, &base, &path, principal).await;
        if status != 200 || body["tenant_id"] != tenant {
            failures.push(format!("{route}: expected 200, got {status}: {body}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// A command envelope carrying the `.1.4.2` delegation context (the scope is
/// either exactly `thread` or tenant-wide — the caller's own attenuation).
fn envelope_with_delegation(
    operation: &str,
    key: &str,
    body: Value,
    on_behalf_of: &str,
    thread: Option<&str>,
) -> CommandEnvelope {
    let scope = match thread {
        Some(id) => TargetSelector::Threads {
            threads: vec![id.parse().expect("thread id parses")],
        },
        None => TargetSelector::TenantWide,
    };
    CommandEnvelope {
        protocol_version: PROTOCOL_VERSION.to_string(),
        operation: operation.to_string(),
        request_id: RequestId::new(),
        idempotency_key: key.to_string(),
        expected_aggregate_version: None,
        body,
        authority_context: Some(AuthorityContext {
            on_behalf_of: on_behalf_of.to_string(),
            purpose: Some("test delegation".to_string()),
            scope,
        }),
        client_context: ClientContext::default(),
    }
}

/// THE `.1.4.2` acceptance: a delegation within the subject's grant succeeds
/// (and audits the subject), a widening scope is a typed 403 naming the
/// invariant, the caller's own authority is checked, and a revoked subject
/// grant refuses the delegation at the next decision (the `.1.3` freshness).
#[tokio::test]
async fn delegation_succeeds_within_the_subjects_grant_and_refuses_widening() {
    let _guard = api_guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, alice) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "del-alice" }),
    )
    .await;
    assert_eq!(status, 200, "enroll alice: {alice}");
    let tenant = alice["tenant_id"].as_str().unwrap().to_string();
    let alice_id = alice["principal_id"].as_str().unwrap().to_string();
    let (status, role) = enroll(
        &client,
        &base,
        json!({ "kind": "role", "name": "del-agent", "tenant_id": tenant }),
    )
    .await;
    assert_eq!(status, 200, "enroll role: {role}");
    let role_id = role["principal_id"].as_str().unwrap().to_string();
    let role_grant = role["grant_id"].as_str().unwrap().to_string();

    // Two threads: the delegation target + a sibling for the widening case.
    let (status, created) = command(
        &client,
        &base,
        "/v1/threads",
        &alice_id,
        &envelope(
            "thread.create",
            "key-del-c1",
            json!({ "tenant_id": tenant, "subject": "del target", "objective": "probe" }),
        ),
    )
    .await;
    assert_eq!(status, 200, "create: {created}");
    let thread = created["thread_id"].as_str().unwrap().to_string();
    let (status, created2) = command(
        &client,
        &base,
        "/v1/threads",
        &alice_id,
        &envelope(
            "thread.create",
            "key-del-c2",
            json!({ "tenant_id": tenant, "subject": "del sibling", "objective": "probe" }),
        ),
    )
    .await;
    assert_eq!(status, 200, "create sibling: {created2}");
    let sibling = created2["thread_id"].as_str().unwrap().to_string();

    // 1. Narrower succeeds: the human acts ON BEHALF OF the role with the
    //    scope = exactly the target thread.
    let (status, delegated) = command(
        &client,
        &base,
        &format!("/v1/threads/{thread}/commands"),
        &alice_id,
        &envelope_with_delegation(
            "thread.contribute",
            "key-del-1",
            json!({ "tenant_id": tenant, "content": "delegated contribution" }),
            &role_id,
            Some(&thread),
        ),
    )
    .await;
    assert_eq!(
        status, 200,
        "the delegation within the subject's grant succeeds: {delegated}"
    );
    // The audit row carries the SUBJECT.
    let (n_subject,): (i64,) = sqlx::query_as(
        "SELECT count(*) FROM authorization_records WHERE tenant_id = $1 AND subject_id = $2",
    )
    .bind(&tenant)
    .bind(&role_id)
    .fetch_one(&pool)
    .await
    .expect("count subject rows");
    assert!(n_subject >= 1, "the decision record carries the chain");

    // 2. Widening refused: the same delegation with the scope naming the
    //    SIBLING thread (outside the request's target).
    let (status, widened) = command(
        &client,
        &base,
        &format!("/v1/threads/{thread}/commands"),
        &alice_id,
        &envelope_with_delegation(
            "thread.contribute",
            "key-del-2",
            json!({ "tenant_id": tenant, "content": "widening attempt" }),
            &role_id,
            Some(&sibling),
        ),
    )
    .await;
    assert_eq!(
        status, 403,
        "a widening scope is refused with the invariant: {widened}"
    );
    assert!(
        widened["message"]
            .as_str()
            .unwrap_or("")
            .contains("widening"),
        "the refusal names the widening invariant: {widened}"
    );

    // 3. The caller's own authority is checked: the role delegates ON BEHALF
    //    OF the human for a thread-create (the role lacks thread_create).
    let (status, caller_refused) = command(
        &client,
        &base,
        "/v1/threads",
        &role_id,
        &envelope_with_delegation(
            "thread.create",
            "key-del-3",
            json!({ "tenant_id": tenant, "subject": "illegal delegation", "objective": "probe" }),
            &alice_id,
            None,
        ),
    )
    .await;
    assert_eq!(
        status, 403,
        "the caller's own authority gates the delegation: {caller_refused}"
    );

    // 4. Revocation freshness: revoke the subject's grant; the delegation is
    //    refused at the next decision (the `.1.3` filter, no new machinery).
    let (status, revoked) = admin_revoke(
        &client,
        &base,
        &format!("/v1/admin/grants/{role_grant}/revoke"),
        &alice_id,
        &tenant,
    )
    .await;
    assert_eq!(status, 200, "the subject grant is revoked: {revoked}");
    let (status, after_revoke) = command(
        &client,
        &base,
        &format!("/v1/threads/{thread}/commands"),
        &alice_id,
        &envelope_with_delegation(
            "thread.contribute",
            "key-del-4",
            json!({ "tenant_id": tenant, "content": "after the revocation" }),
            &role_id,
            Some(&thread),
        ),
    )
    .await;
    assert_eq!(
        status, 403,
        "a revoked subject grant refuses the delegation: {after_revoke}"
    );
}

fn dims(calls: Option<u64>, tokens: Option<u64>) -> reasonbraid_core::BudgetDimensions {
    reasonbraid_core::BudgetDimensions {
        calls,
        input_tokens: tokens,
        output_tokens: tokens,
        wall_clock_seconds: None,
    }
}

/// THE `.3.3` acceptance: the usage-reconciliation surface sums the LEDGER
/// rows (held vs settled vs overrun vs denied, per dimension) — measured:
/// the seeded rows' arithmetic is recomputed in the test and must match the
/// wire. tenant_admin-gated.
#[tokio::test]
async fn the_usage_surface_reconciles_the_ledger_rows() {
    let _guard = api_guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let client = reqwest::Client::new();
    let (status, alice) = enroll(
        &client,
        &server.base(),
        json!({ "kind": "human", "name": "usage-alice" }),
    )
    .await;
    assert_eq!(status, 200, "enroll: {alice}");
    let tenant = alice["tenant_id"].as_str().unwrap().to_string();
    let alice_id = alice["principal_id"].as_str().unwrap().to_string();

    // Seed the ledger directly (the budget engine's own rows): a thread with
    // a ceiling, one ACTIVE hold, one SETTLED with an overrun, one DENIED.
    let thread = "thr_00000000-0000-7000-8000-000000000900";
    sqlx::query(
        "INSERT INTO budget_ceilings (ceiling_id, tenant_id, thread_id, dimensions, policy_version) \
         VALUES ('ceil_recon', $1, $2, $3, 'dev-budget-1')",
    )
    .bind(&tenant)
    .bind(thread)
    .bind(serde_json::to_value(dims(Some(100), Some(100_000))).unwrap())
    .execute(&pool)
    .await
    .expect("ceiling");
    sqlx::query(
        "INSERT INTO budget_reservations \
         (reservation_id, ceiling_id, tenant_id, thread_id, dimensions, status, expires_at, created_at) \
         VALUES ('res_recon_hold', 'ceil_recon', $1, $2, $3, 'active', now() + interval '10 minutes', now())",
    )
    .bind(&tenant)
    .bind(thread)
    .bind(serde_json::to_value(dims(Some(2), Some(200))).unwrap())
    .execute(&pool)
    .await
    .expect("hold");
    sqlx::query(
        "INSERT INTO budget_reservations \
         (reservation_id, ceiling_id, tenant_id, thread_id, dimensions, usage, status, created_at, settled_at) \
         VALUES ('res_recon_settled', 'ceil_recon', $1, $2, $3, $4, 'settled', now(), now())",
    )
    .bind(&tenant)
    .bind(thread)
    .bind(serde_json::to_value(dims(Some(5), Some(500))).unwrap())
    .bind(serde_json::to_value(dims(Some(8), Some(500))).unwrap()) // 3 calls over the hold
    .execute(&pool)
    .await
    .expect("settled");
    sqlx::query(
        "INSERT INTO budget_reservations \
         (reservation_id, ceiling_id, tenant_id, thread_id, dimensions, status, reason, created_at) \
         VALUES ('res_recon_denied', 'ceil_recon', $1, $2, $3, 'denied', 'beyond the ceiling', now())",
    )
    .bind(&tenant)
    .bind(thread)
    .bind(serde_json::to_value(dims(Some(50), Some(50_000))).unwrap())
    .execute(&pool)
    .await
    .expect("denied");
    // An EXPIRED hold stops holding (the reconciliation counts it nowhere).
    sqlx::query(
        "INSERT INTO budget_reservations \
         (reservation_id, ceiling_id, tenant_id, thread_id, dimensions, status, expires_at, created_at) \
         VALUES ('res_recon_expired', 'ceil_recon', $1, $2, $3, 'active', now() - interval '1 minute', now())",
    )
    .bind(&tenant)
    .bind(thread)
    .bind(serde_json::to_value(dims(Some(9), Some(900))).unwrap())
    .execute(&pool)
    .await
    .expect("expired");

    let (status, usage) = get(
        &client,
        &server.base(),
        &format!("/v1/admin/usage?tenant_id={tenant}"),
        &alice_id,
    )
    .await;
    assert_eq!(status, 200, "the usage surface answers: {usage}");
    let aggregate = &usage["aggregate"];
    // Measured: held 2 calls (the expired 9 stops holding), settled 8 (the
    // overrun's actuals), overrun 3 calls (8 - 5), denied 1.
    assert_eq!(aggregate["held"]["calls"], json!(2), "held calls: {usage}");
    assert_eq!(
        aggregate["settled"]["calls"],
        json!(8),
        "settled calls: {usage}"
    );
    assert_eq!(aggregate["settled"]["output_tokens"], json!(500));
    assert_eq!(
        aggregate["overrun"]["calls"],
        json!(3),
        "overrun calls: {usage}"
    );
    assert_eq!(
        aggregate["overrun"]["output_tokens"],
        json!(0),
        "no token overrun"
    );
    assert_eq!(aggregate["denied"], json!(1));
    assert!(
        aggregate["denial_reasons"]
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r == "beyond the ceiling"),
        "the denial reason rides the surface: {usage}"
    );
    let threads = usage["threads"].as_array().unwrap();
    assert_eq!(threads.len(), 1, "one thread in the breakdown: {usage}");
    assert_eq!(threads[0]["held"]["calls"], json!(2));
    assert_eq!(threads[0]["settled"]["calls"], json!(8));
    assert_eq!(threads[0]["overrun"]["calls"], json!(3));
    assert_eq!(threads[0]["denied"], json!(1));
}

/// THE `.5.2` acceptance: the admin metrics surface counts what the server
/// observed — MEASURED: the denial counter's delta matches the new denied
/// authorization rows, the replay counter's delta matches the replayed
/// command. tenant_admin-gated.
#[tokio::test]
async fn the_metrics_surface_counts_match_the_records() {
    let _guard = api_guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let client = reqwest::Client::new();
    let base = server.base();

    // Bootstrap: the human holds tenant_admin (the gate); the role holds
    // thread_contribute only.
    let (status, alice) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "metrics-alice" }),
    )
    .await;
    assert_eq!(status, 200, "enroll: {alice}");
    let tenant = alice["tenant_id"].as_str().unwrap().to_string();
    let alice_id = alice["principal_id"].as_str().unwrap().to_string();
    let (status, role) = enroll(
        &client,
        &base,
        json!({ "kind": "role", "name": "metrics-role", "tenant_id": tenant, "actions": ["thread_contribute"] }),
    )
    .await;
    assert_eq!(status, 200, "enroll role: {role}");
    let role_id = role["principal_id"].as_str().unwrap().to_string();

    async fn metrics(client: &reqwest::Client, base: &str, alice: &str) -> Value {
        let response = client
            .get(format!("{base}/v1/admin/metrics"))
            .header(PRINCIPAL_HEADER, alice)
            .send()
            .await
            .expect("metrics request");
        assert_eq!(response.status().as_u16(), 200);
        response.json::<Value>().await.expect("metrics json")
    }
    let before = metrics(&client, &base, &alice_id).await;

    // One denial: the role (no tenant_admin) attempts an admin WRITE — the
    // authorize path writes the audited denial row (the record the counter
    // must agree with).
    let (status, refused) = admin_revoke(
        &client,
        &base,
        "/v1/admin/grants/grt_bogus/revoke",
        &role_id,
        &tenant,
    )
    .await;
    assert_eq!(status, 403, "the role is refused: {refused}");

    // One replay: the same enroll twice.
    let (status, _) = enroll(
        &client,
        &base,
        json!({ "kind": "role", "name": "metrics-role-2", "tenant_id": tenant }),
    )
    .await;
    assert_eq!(status, 200, "first role enroll");
    let (status, replayed) = enroll(
        &client,
        &base,
        json!({ "kind": "role", "name": "metrics-role-2", "tenant_id": tenant }),
    )
    .await;
    assert_eq!(status, 200, "replay enroll");
    assert_eq!(replayed["replayed"], json!(true));

    let after = metrics(&client, &base, &alice_id).await;
    let delta = |name: &str| {
        after[name]
            .as_u64()
            .unwrap_or(0)
            .saturating_sub(before[name].as_u64().unwrap_or(0))
    };
    assert_eq!(
        delta("authorization_denials"),
        1,
        "one denial counted: {after}"
    );

    // MEASURED against the records: the denied authorization rows for the
    // role's actor handle grew by exactly one (a direct authorization binds
    // the principal to the actor column; the subject columns are the
    // delegation split).
    let actor = reasonbraid_core::actor_handle_for_subject(&reasonbraid_core::GrantSubject::Role(
        role_id.parse().unwrap(),
    ))
    .to_string();
    let denied_rows: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM authorization_records WHERE actor = $1 AND decision = 'denied'",
    )
    .bind(&actor)
    .fetch_one(&pool)
    .await
    .expect("denied rows");
    assert_eq!(denied_rows, 1, "the record agrees with the counter");
}

/// Snapshot every provisional enrollment table; authorization records are not
/// enrollment effects and are tested at their separate admission boundary.
async fn enrollment_storage_snapshot(pool: &PgPool) -> Vec<Value> {
    let mut snapshot = Vec::new();
    for table in [
        "tenants",
        "enrollment_boundaries",
        "authority_grants",
        "human_principals",
        "agent_roles",
        "usage_quotas",
        "enrollments",
    ] {
        snapshot.push(sqlx::query_scalar::<_, Value>(&format!(
            "SELECT coalesce(jsonb_agg(row ORDER BY row::text), '[]'::jsonb) FROM (SELECT to_jsonb(t) AS row FROM {table} t) s"))
            .fetch_one(pool).await.unwrap());
    }
    snapshot
}

#[tokio::test]
async fn enrollment_grant_storage_failure_is_internal_and_atomic() {
    let _guard = api_guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let client = reqwest::Client::new();
    let (status, human) = enroll(
        &client,
        &server.base(),
        json!({"kind":"human", "name":"grant-fault-owner"}),
    )
    .await;
    assert_eq!(status, 200, "{human}");
    // Exercise bootstrap's provisional tenant/boundary/quotas as well as the
    // existing-tenant role path. Restore DDL before inspecting either outcome.
    let requests = [
        json!({"kind":"human", "name":"grant-fault-new"}),
        json!({"kind":"role", "name":"grant-fault-role", "tenant_id":human["tenant_id"]}),
    ];
    let before = enrollment_storage_snapshot(&pool).await;
    sqlx::query("CREATE FUNCTION rb_test_refuse_enrollment_grant() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RAISE EXCEPTION 'owned enrollment grant fault'; END $$")
        .execute(&pool).await.unwrap();
    sqlx::query("CREATE TRIGGER rb_test_enrollment_grant_fault BEFORE INSERT ON authority_grants FOR EACH ROW EXECUTE FUNCTION rb_test_refuse_enrollment_grant()")
        .execute(&pool).await.unwrap();
    let mut outcomes = Vec::new();
    for body in &requests {
        outcomes.push(
            client
                .post(format!("{}/v1/enrollments", server.base()))
                .json(body)
                .send()
                .await,
        );
    }
    sqlx::query("DROP TRIGGER rb_test_enrollment_grant_fault ON authority_grants")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("DROP FUNCTION rb_test_refuse_enrollment_grant()")
        .execute(&pool)
        .await
        .unwrap();
    let after = enrollment_storage_snapshot(&pool).await;
    for body in &requests {
        let (status, body) = enroll(&client, &server.base(), body.clone()).await;
        assert_eq!(status, 200, "recovery: {body}");
    }
    assert_eq!(
        before, after,
        "failed grant insertion leaves no enrollment effects"
    );
    for outcome in outcomes {
        let response = outcome.expect("storage faults produce HTTP responses");
        let status = response.status().as_u16();
        let body: Value = response.json().await.unwrap();
        assert_eq!(
            status, 500,
            "storage failure is not a policy refusal: {body}"
        );
        assert_eq!(
            body,
            json!({"code":"dependency_unavailable", "message":"internal server error"})
        );
    }
}

#[tokio::test]
async fn corrupt_active_enrollment_boundary_returns_internal_error() {
    let _guard = api_guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let client = reqwest::Client::new();
    let (status, human) = enroll(
        &client,
        &server.base(),
        json!({"kind":"human", "name":"corrupt-grant-owner"}),
    )
    .await;
    assert_eq!(status, 200, "{human}");
    let tenant = human["tenant_id"].as_str().unwrap();
    let actions: Value = sqlx::query_scalar(
        "SELECT permitted_actions FROM enrollment_boundaries WHERE tenant_id = $1",
    )
    .bind(tenant)
    .fetch_one(&pool)
    .await
    .unwrap();
    let before = enrollment_storage_snapshot(&pool).await;
    sqlx::query("UPDATE enrollment_boundaries SET permitted_actions = $2 WHERE tenant_id = $1")
        .bind(tenant)
        .bind(json!(["unknown_owned_fixture_action"]))
        .execute(&pool)
        .await
        .unwrap();
    let request = json!({"kind":"role", "name":"corrupt-grant-role", "tenant_id":tenant});
    let outcome = client
        .post(format!("{}/v1/enrollments", server.base()))
        .json(&request)
        .send()
        .await;
    sqlx::query("UPDATE enrollment_boundaries SET permitted_actions = $2 WHERE tenant_id = $1")
        .bind(tenant)
        .bind(actions)
        .execute(&pool)
        .await
        .unwrap();
    let after = enrollment_storage_snapshot(&pool).await;
    let (status, body) = enroll(&client, &server.base(), request).await;
    assert_eq!(status, 200, "recovery: {body}");
    assert_eq!(
        before, after,
        "corrupt authority cannot create a partial enrollment"
    );
    let response = outcome.expect("malformed stored authority must not panic the handler");
    assert_eq!(response.status().as_u16(), 500);
    assert_eq!(
        response.json::<Value>().await.unwrap(),
        json!({"code":"dependency_unavailable", "message":"internal server error"})
    );
}

#[tokio::test]
async fn enrollment_structural_grant_refusal_remains_a_bad_request() {
    let _guard = api_guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let client = reqwest::Client::new();
    let (status, human) = enroll(
        &client,
        &server.base(),
        json!({"kind":"human", "name":"narrow-grant-owner"}),
    )
    .await;
    assert_eq!(status, 200, "{human}");
    let tenant = human["tenant_id"].as_str().unwrap();
    let actions: Value = sqlx::query_scalar(
        "SELECT permitted_actions FROM enrollment_boundaries WHERE tenant_id = $1",
    )
    .bind(tenant)
    .fetch_one(&pool)
    .await
    .unwrap();
    let before = enrollment_storage_snapshot(&pool).await;
    sqlx::query("UPDATE enrollment_boundaries SET permitted_actions = $2 WHERE tenant_id = $1")
        .bind(tenant)
        .bind(json!(["tenant_admin"]))
        .execute(&pool)
        .await
        .unwrap();
    let request = json!({"kind":"role", "name":"narrow-grant-role", "tenant_id":tenant});
    let outcome = client
        .post(format!("{}/v1/enrollments", server.base()))
        .json(&request)
        .send()
        .await;
    sqlx::query("UPDATE enrollment_boundaries SET permitted_actions = $2 WHERE tenant_id = $1")
        .bind(tenant)
        .bind(actions)
        .execute(&pool)
        .await
        .unwrap();
    let after = enrollment_storage_snapshot(&pool).await;
    let (status, body) = enroll(&client, &server.base(), request).await;
    assert_eq!(status, 200, "recovery: {body}");
    assert_eq!(before, after, "structural refusal has no enrollment effect");
    let response = outcome.unwrap();
    assert_eq!(response.status().as_u16(), 400);
    let body: Value = response.json().await.unwrap();
    assert_eq!(body["code"], "invalid_command");
    assert!(
        body["message"]
            .as_str()
            .unwrap()
            .starts_with("the dev grant exceeds its boundary: "),
        "{body}"
    );
}
