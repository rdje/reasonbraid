//! The classification-driven controls proof (`PHASE-7.1.4.3`, the `.1.4.1`
//! contract, ADR-034): the evaluator-access decision at the DISPATCH — a
//! confidential thread's work delivery refuses without a
//! confidential-qualified evaluator profile (the dev registry registers
//! none) — the typed refusal, never a silent general. Measured:
//!   - a confidential thread's ACCEPT refuses with the typed
//!     `classification_unqualified` code (409) — the invitation stays
//!     pending, no work item lands;
//!   - a general thread's accept still dispatches (the control is
//!     classification-scoped, not global);
//!   - the confidential thread itself stays creatable + inspectable (the
//!     controls refuse the PROVIDER use, never the thread).
//!
//! Run with `scripts/run_pg_tests.sh` locally or the `pg-tests` CI job.
//! Without `DATABASE_URL` these skip, so `make check` stays green offline.

#[path = "support/mod.rs"]
mod pg_test_support;

#[path = "support/cleanup.rs"]
mod pg_cleanup;

use std::net::SocketAddr;
use std::sync::OnceLock;

use reasonbraid_core::{ClientContext, CommandEnvelope, RequestId, PROTOCOL_VERSION};
use reasonbraid_server::{api_router, PRINCIPAL_HEADER};
use serde_json::{json, Value};
use sqlx::PgPool;

static CLASSIFICATION_LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();

async fn guard() -> tokio::sync::MutexGuard<'static, ()> {
    CLASSIFICATION_LOCK
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
            "quota_events",
            "usage_quotas",
            "federation_agreements",
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
            "cross_domain_receipts",
            "tenant_bootstrap_requests",
            "node_certificates",
            "node_keys",
            "node_leases",
            "nodes",
            "hosts",
            "mcp_listen_state",
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

#[tokio::test]
async fn the_confidential_dispatch_refuses_until_a_qualified_evaluator_exists() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    // 1. The bootstrap human + one role.
    let (status, human) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "cls-human" }),
    )
    .await;
    assert_eq!(status, 200, "the human enrolls: {human}");
    let human_id = human["principal_id"].as_str().unwrap().to_string();
    let tenant = human["tenant_id"].as_str().unwrap().to_string();
    let (status, role) = enroll(
        &client,
        &base,
        json!({ "kind": "role", "name": "cls-role", "tenant_id": tenant }),
    )
    .await;
    assert_eq!(status, 200, "the role enrolls: {role}");
    let role_id = role["principal_id"].as_str().unwrap().to_string();

    // 2. The CONFIDENTIAL thread creates (the classification stays creatable —
    //    the control refuses the provider use, never the thread) and inspects
    //    with the classification intact.
    let (status, created) = command(
        &client,
        &base,
        "/v1/threads",
        &human_id,
        &envelope(
            "thread.create",
            "cls-conf-create",
            json!({
                "tenant_id": tenant,
                "subject": "the confidential plan",
                "objective": "decide quietly",
                "budget": { "calls": 2 },
                "classification": "confidential",
                "participant_rules": { "allow_explicit_invites": true, "allow_join_requests": false },
            }),
        ),
    )
    .await;
    assert_eq!(status, 200, "the confidential thread creates: {created}");
    let thread_id = created["thread_id"].as_str().unwrap().to_string();
    let (status, inspected) = get(
        &client,
        &base,
        &format!("/v1/threads/{thread_id}?tenant_id={tenant}"),
        &human_id,
    )
    .await;
    assert_eq!(status, 200, "the thread inspects: {inspected}");
    assert_eq!(
        inspected["state"]["classification"],
        json!("confidential"),
        "the classification is a recorded fact"
    );

    // 3. The invitation records (the invite is a human decision, not a
    //    delivery); the ACCEPT — the delivery decision point — refuses with
    //    the typed code. The invitation stays pending, no work item lands.
    let (status, invited) = command(
        &client,
        &base,
        &format!("/v1/threads/{thread_id}/commands"),
        &human_id,
        &envelope(
            "thread.invite",
            "cls-invite",
            json!({ "tenant_id": tenant, "agent_role": role_id }),
        ),
    )
    .await;
    assert_eq!(status, 200, "the invite records: {invited}");

    let (status, refused) = command(
        &client,
        &base,
        &format!("/v1/threads/{thread_id}/commands"),
        &role_id,
        &envelope(
            "thread.accept_invitation",
            "cls-accept",
            json!({ "tenant_id": tenant }),
        ),
    )
    .await;
    assert_eq!(status, 409, "the confidential accept refuses: {refused}");
    assert_eq!(refused["code"], json!("classification_unqualified"));

    let work: i64 =
        sqlx::query_scalar("SELECT count(*) FROM node_inbox WHERE thread_id = $1 AND node_id = $2")
            .bind(&thread_id)
            .bind(&role_id)
            .fetch_one(&pool)
            .await
            .expect("count the work items");
    assert_eq!(work, 0, "no work item landed for the refused delivery");

    // 4. The control is classification-scoped: a GENERAL thread's
    //    invite/accept still dispatches.
    let (status, general) = command(
        &client,
        &base,
        "/v1/threads",
        &human_id,
        &envelope(
            "thread.create",
            "cls-general-create",
            json!({
                "tenant_id": tenant,
                "subject": "the open plan",
                "objective": "decide openly",
                "budget": { "calls": 2 },
                "participant_rules": { "allow_explicit_invites": true, "allow_join_requests": false },
            }),
        ),
    )
    .await;
    assert_eq!(status, 200, "the general thread creates: {general}");
    let general_id = general["thread_id"].as_str().unwrap().to_string();
    let (status, invited) = command(
        &client,
        &base,
        &format!("/v1/threads/{general_id}/commands"),
        &human_id,
        &envelope(
            "thread.invite",
            "cls-general-invite",
            json!({ "tenant_id": tenant, "agent_role": role_id }),
        ),
    )
    .await;
    assert_eq!(status, 200, "the general invite records: {invited}");
    let (status, accepted) = command(
        &client,
        &base,
        &format!("/v1/threads/{general_id}/commands"),
        &role_id,
        &envelope(
            "thread.accept_invitation",
            "cls-general-accept",
            json!({ "tenant_id": tenant }),
        ),
    )
    .await;
    assert_eq!(status, 200, "the general accept dispatches: {accepted}");
    let work: i64 =
        sqlx::query_scalar("SELECT count(*) FROM node_inbox WHERE thread_id = $1 AND node_id = $2")
            .bind(&general_id)
            .bind(&role_id)
            .fetch_one(&pool)
            .await
            .expect("count the general work items");
    assert_eq!(work, 1, "the qualified classification dispatched its work");
}
