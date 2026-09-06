//! Integration tests for dev-profile node enrollment (`PHASE-1.2.1`; backlog 11).
//!
//! The flow: an authorized human issues a ONE-TIME token (bound to tenant, node id,
//! host claim, nonce, expiry — §16.2), then the node consumes it at
//! `/v1/nodes/enroll` with its dev signing secret. The token is the credential; the
//! server lands the host + node + key rows in the 0007/0008 identity tables and
//! audits every attempt. Certificate issuance is deferred to Phase 2 (ADR-007).
//! Like the other PostgreSQL suites, these tests skip without `DATABASE_URL`.

use std::net::SocketAddr;
use std::sync::OnceLock;

use reasonbraid_server::{api_router, node_router, PRINCIPAL_HEADER};
use serde_json::{json, Value};
use sqlx::PgPool;

static ENROLL_LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();

async fn enroll_guard() -> tokio::sync::MutexGuard<'static, ()> {
    ENROLL_LOCK
        .get_or_init(|| tokio::sync::Mutex::new(()))
        .lock()
        .await
}

async fn pool() -> Option<PgPool> {
    let url = match std::env::var("DATABASE_URL") {
        Ok(u) => u,
        Err(_) => {
            eprintln!(
                "SKIP: DATABASE_URL is unset — run scripts/run_pg_tests.sh for the real PostgreSQL proof"
            );
            return None;
        }
    };
    let pool = PgPool::connect(&url)
        .await
        .expect("connect to DATABASE_URL");
    sqlx::migrate!("../../migrations")
        .run(&pool)
        .await
        .expect("apply migrations");
    for table in [
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
        "node_enroll_audit",
        "node_keys",
        "node_leases",
        "node_enrollment_tokens",
        "runs",
        "incarnations",
        "nodes",
        "hosts",
        "agent_roles",
        "human_principals",
        "tenants",
        "idempotency",
        "event_log",
        "aggregate_state",
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
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind ephemeral loopback port");
        let addr = listener.local_addr().unwrap();
        let router = api_router(pool.clone()).merge(node_router(pool.clone()));
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

async fn post_json(
    client: &reqwest::Client,
    url: String,
    principal: Option<&str>,
    body: Value,
) -> (u16, Value) {
    let mut request = client.post(url).json(&body);
    if let Some(p) = principal {
        request = request.header(PRINCIPAL_HEADER, p);
    }
    let response = request.send().await.expect("request");
    let status = response.status().as_u16();
    (status, response.json().await.expect("json response"))
}

/// Bootstrap a human admin (the token issuer) and return (tenant, principal id).
async fn bootstrap_admin(client: &reqwest::Client, base: &str) -> (String, String) {
    let (status, alice) = post_json(
        client,
        format!("{base}/v1/enrollments"),
        None,
        json!({ "kind": "human", "name": "alice" }),
    )
    .await;
    assert_eq!(status, 200, "bootstrap admin: {alice}");
    (
        alice["tenant_id"].as_str().unwrap().to_string(),
        alice["principal_id"].as_str().unwrap().to_string(),
    )
}

async fn issue_token(
    client: &reqwest::Client,
    base: &str,
    principal: &str,
    tenant: &str,
    node_id: &str,
    host_claim: &str,
    ttl: Option<i64>,
) -> Value {
    let mut body = json!({ "tenant_id": tenant, "node_id": node_id, "host_claim": host_claim });
    if let Some(t) = ttl {
        body["ttl_seconds"] = json!(t);
    }
    let (status, token) = post_json(
        client,
        format!("{base}/v1/nodes/enroll-tokens"),
        Some(principal),
        body,
    )
    .await;
    assert_eq!(status, 200, "issue token: {token}");
    token
}

async fn enroll_node(
    client: &reqwest::Client,
    base: &str,
    token_id: &str,
    node_id: &str,
    host_claim: &str,
    nonce: &str,
    secret: &str,
) -> (u16, Value) {
    post_json(
        client,
        format!("{base}/v1/nodes/enroll"),
        None,
        json!({
            "token_id": token_id,
            "node_id": node_id,
            "host_claim": host_claim,
            "nonce": nonce,
            "key_secret": secret,
        }),
    )
    .await
}

/// A token enrolls EXACTLY ONCE: the host + node + key rows land together with the
/// consumed token and the audit row; a second use of the same token is refused AND
/// audited, with nothing duplicated.
#[tokio::test]
async fn a_token_enrolls_exactly_once_and_lands_identity_rows() {
    let _guard = enroll_guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (tenant, alice) = bootstrap_admin(&client, &base).await;
    let node_id = "nod_00000000-0000-7000-8000-000000000101";
    let token = issue_token(&client, &base, &alice, &tenant, node_id, "host-a", None).await;
    let token_id = token["token_id"].as_str().unwrap().to_string();
    let nonce = token["nonce"].as_str().unwrap().to_string();

    let (status, enrolled) = enroll_node(
        &client,
        &base,
        &token_id,
        node_id,
        "host-a",
        &nonce,
        "dev-secret-101",
    )
    .await;
    assert_eq!(status, 200, "enroll: {enrolled}");
    assert_eq!(enrolled["node_id"], json!(node_id));
    let host_id = enrolled["host_id"].as_str().unwrap().to_string();
    assert!(host_id.starts_with("hst_"));

    // Identity rows, from a SEPARATE connection: host, node, key, consumed token,
    // audit — all committed.
    let (n_hosts,): (i64,) = sqlx::query_as("SELECT count(*) FROM hosts WHERE host_id = $1")
        .bind(&host_id)
        .fetch_one(&pool)
        .await
        .expect("count hosts");
    assert_eq!(n_hosts, 1);
    let (host_tenant, host_name): (String, String) =
        sqlx::query_as("SELECT tenant_id, name FROM hosts WHERE host_id = $1")
            .bind(&host_id)
            .fetch_one(&pool)
            .await
            .expect("read host row");
    assert_eq!(host_tenant, tenant);
    assert_eq!(host_name, "host-a");

    let (n_nodes,): (i64,) = sqlx::query_as("SELECT count(*) FROM nodes WHERE node_id = $1")
        .bind(node_id)
        .fetch_one(&pool)
        .await
        .expect("count nodes");
    assert_eq!(n_nodes, 1);

    let (fingerprint, stored_secret): (String, String) =
        sqlx::query_as("SELECT key_fingerprint, key_secret FROM node_keys WHERE node_id = $1")
            .bind(node_id)
            .fetch_one(&pool)
            .await
            .expect("read node key");
    assert_eq!(
        stored_secret, "dev-secret-101",
        "the dev trust store holds the secret"
    );
    assert_eq!(fingerprint.len(), 64, "sha256 hex");

    let (used,): (bool,) = sqlx::query_as(
        "SELECT (used_at IS NOT NULL) FROM node_enrollment_tokens WHERE token_id = $1",
    )
    .bind(&token_id)
    .fetch_one(&pool)
    .await
    .expect("read token");
    assert!(used, "the token is consumed");

    let (n_audit,): (i64,) = sqlx::query_as(
        "SELECT count(*) FROM node_enroll_audit WHERE token_id = $1 AND decision = 'enrolled'",
    )
    .bind(&token_id)
    .fetch_one(&pool)
    .await
    .expect("count audit");
    assert_eq!(n_audit, 1);

    // The second use is REFUSED and audited; nothing is duplicated.
    let (status, again) = enroll_node(
        &client,
        &base,
        &token_id,
        node_id,
        "host-a",
        &nonce,
        "dev-secret-101",
    )
    .await;
    assert_eq!(status, 401, "second use: {again}");
    assert_eq!(again["code"], json!("unauthorized"));
    let (n_refused,): (i64,) = sqlx::query_as(
        "SELECT count(*) FROM node_enroll_audit WHERE token_id = $1 AND decision = 'refused'",
    )
    .bind(&token_id)
    .fetch_one(&pool)
    .await
    .expect("count refusals");
    assert_eq!(n_refused, 1, "the refusal is audited");
    let (n_keys,): (i64,) = sqlx::query_as("SELECT count(*) FROM node_keys WHERE node_id = $1")
        .bind(node_id)
        .fetch_one(&pool)
        .await
        .expect("count keys");
    assert_eq!(n_keys, 1, "the second use wrote no key row");
}

/// Every refusal class is audited and effect-free: unknown token, expired token,
/// node-id mismatch, nonce mismatch.
#[tokio::test]
async fn refusals_are_audited_and_effect_free() {
    let _guard = enroll_guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (tenant, alice) = bootstrap_admin(&client, &base).await;
    let node_id = "nod_00000000-0000-7000-8000-000000000102";
    let token = issue_token(&client, &base, &alice, &tenant, node_id, "host-b", None).await;
    let token_id = token["token_id"].as_str().unwrap().to_string();
    let nonce = token["nonce"].as_str().unwrap().to_string();

    // Unknown token.
    let (status, unknown) = enroll_node(
        &client,
        &base,
        "ntk_does_not_exist",
        node_id,
        "host-b",
        &nonce,
        "s",
    )
    .await;
    assert_eq!(status, 401, "unknown token: {unknown}");
    assert!(unknown["message"].as_str().unwrap().contains("unknown"));

    // Node-id mismatch.
    let (status, mismatched) = enroll_node(
        &client,
        &base,
        &token_id,
        "nod_00000000-0000-7000-8000-000000000999",
        "host-b",
        &nonce,
        "s",
    )
    .await;
    assert_eq!(status, 401, "node mismatch: {mismatched}");
    assert!(mismatched["message"]
        .as_str()
        .unwrap()
        .contains("different node"));

    // Nonce mismatch.
    let (status, bad_nonce) = enroll_node(
        &client,
        &base,
        &token_id,
        node_id,
        "host-b",
        "wrong-nonce",
        "s",
    )
    .await;
    assert_eq!(status, 401, "nonce mismatch: {bad_nonce}");
    assert!(bad_nonce["message"].as_str().unwrap().contains("nonce"));

    // Re-issuing while an unused token is outstanding is a TYPED refusal (one
    // UNUSED token per node — the 0008 unique index), never a 500.
    let (status, reissue) = post_json(
        &client,
        format!("{base}/v1/nodes/enroll-tokens"),
        Some(&alice),
        json!({ "tenant_id": tenant, "node_id": node_id, "host_claim": "host-b" }),
    )
    .await;
    assert_eq!(status, 409, "re-issue: {reissue}");
    assert_eq!(reissue["code"], json!("invalid_command"));
    assert!(reissue["message"]
        .as_str()
        .unwrap()
        .contains("already exists"));

    // Expired token (a negative ttl expires immediately) — a distinct node id:
    // one UNUSED token per node is the 0008 invariant.
    let expired_node = "nod_00000000-0000-7000-8000-000000000103";
    let expired = issue_token(
        &client,
        &base,
        &alice,
        &tenant,
        expired_node,
        "host-b",
        Some(-1),
    )
    .await;
    let (status, expired_use) = enroll_node(
        &client,
        &base,
        expired["token_id"].as_str().unwrap(),
        expired_node,
        "host-b",
        expired["nonce"].as_str().unwrap(),
        "s",
    )
    .await;
    assert_eq!(status, 401, "expired token: {expired_use}");
    assert!(expired_use["message"].as_str().unwrap().contains("expired"));

    // Every refusal left an audit row; no node row exists.
    let (n_refused,): (i64,) =
        sqlx::query_as("SELECT count(*) FROM node_enroll_audit WHERE decision = 'refused'")
            .fetch_one(&pool)
            .await
            .expect("count refusals");
    assert_eq!(n_refused, 4, "all four refusals audited");
    let (n_nodes,): (i64,) = sqlx::query_as("SELECT count(*) FROM nodes WHERE node_id = $1")
        .bind(node_id)
        .fetch_one(&pool)
        .await
        .expect("count nodes");
    assert_eq!(n_nodes, 0, "no refusal created a node row");
}

/// Token issuance is `tenant_admin` authority: a role with the default
/// `thread_contribute` grant is denied, and no token exists.
#[tokio::test]
async fn only_tenant_admin_issues_tokens() {
    let _guard = enroll_guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (tenant, _) = bootstrap_admin(&client, &base).await;
    let (status, role) = post_json(
        &client,
        format!("{base}/v1/enrollments"),
        None,
        json!({ "kind": "role", "name": "reviewer", "tenant_id": tenant }),
    )
    .await;
    assert_eq!(status, 200, "enroll role: {role}");
    let role_id = role["principal_id"].as_str().unwrap().to_string();

    let (status, refused) = post_json(
        &client,
        format!("{base}/v1/nodes/enroll-tokens"),
        Some(&role_id),
        json!({
            "tenant_id": tenant,
            "node_id": "nod_00000000-0000-7000-8000-000000000103",
            "host_claim": "host-c",
        }),
    )
    .await;
    assert_eq!(status, 403, "role token issuance: {refused}");
    assert_eq!(refused["code"], json!("unauthorized"));

    let (n_tokens,): (i64,) = sqlx::query_as("SELECT count(*) FROM node_enrollment_tokens")
        .fetch_one(&pool)
        .await
        .expect("count tokens");
    assert_eq!(n_tokens, 0, "the denial issued no token");
}
