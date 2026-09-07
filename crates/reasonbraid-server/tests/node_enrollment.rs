//! Integration tests for dev-profile node enrollment (`PHASE-1.2.1`; backlog 11).
//!
//! The flow: an authorized human issues a ONE-TIME token (bound to tenant, node id,
//! host claim, nonce, expiry — §16.2), then the node consumes it at
//! `/v1/nodes/enroll` with its dev signing secret. The token is the credential; the
//! server lands the host + node + key rows in the 0007/0008 identity tables and
//! audits every attempt. Certificate issuance is deferred to Phase 2 (ADR-007).
//! Like the other PostgreSQL suites, these tests skip without `DATABASE_URL`.

use std::net::SocketAddr;
use std::sync::{Arc, OnceLock};

use reasonbraid_server::{api_router, ca::ensure_server_ca, node_router, PRINCIPAL_HEADER};
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

    // The workload certificate (`.1.2.1`, ADR-007): the response carries the
    // leaf + the dev-escrowed key, and the cert row lands with the same
    // fingerprint the server computed.
    let cert_hex = enrolled["cert_der"].as_str().expect("cert_der").to_string();
    let key_hex = enrolled["key_der"].as_str().expect("key_der").to_string();
    assert!(!cert_hex.is_empty(), "the response carries the leaf");
    assert!(!key_hex.is_empty(), "the response carries the escrowed key");
    assert_eq!(
        enrolled["cert_fingerprint"].as_str().unwrap().len(),
        64,
        "sha256 hex"
    );
    assert!(enrolled["cert_expires_at"].is_string());
    let (stored_fp,): (String,) =
        sqlx::query_as("SELECT cert_fingerprint FROM node_certificates WHERE node_id = $1")
            .bind(node_id)
            .fetch_one(&pool)
            .await
            .expect("read cert row");
    assert_eq!(stored_fp, enrolled["cert_fingerprint"].as_str().unwrap());
    let (n_ca,): (i64,) = sqlx::query_as("SELECT count(*) FROM server_ca WHERE ca_id = 1")
        .fetch_one(&pool)
        .await
        .expect("count ca");
    assert_eq!(n_ca, 1, "the CA row exists (one per deployment)");

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
    let (n_certs,): (i64,) =
        sqlx::query_as("SELECT count(*) FROM node_certificates WHERE node_id = $1")
            .bind(node_id)
            .fetch_one(&pool)
            .await
            .expect("count certs");
    assert_eq!(n_certs, 1, "the second use issued no second certificate");
}

/// The CA survives a server rebuild: two `ensure_server_ca` passes over the same
/// store return the SAME CA (the demo kills and restarts the server — previously
/// issued leaves must keep chaining to the same anchor).
#[tokio::test]
async fn the_ca_survives_a_server_rebuild() {
    let _guard = enroll_guard().await;
    let Some(pool) = pool().await else { return };

    let first = ensure_server_ca(&pool).await.expect("first pass");
    let second = ensure_server_ca(&pool)
        .await
        .expect("second pass (the rebuild)");
    assert_eq!(first.cert_der, second.cert_der, "the same CA certificate");
    assert_eq!(first.key_der, second.key_der, "the same CA key");

    // A leaf issued by the FIRST handle (pre-rebuild material) is a well-formed
    // DER artifact; the chain verification itself lands with `.1.2.2`.
    let (cert_der, key_der) = reasonbraid_server::ca::issue_node_leaf(
        &first,
        "nod_00000000-0000-7000-8000-000000000999",
        "host-persist",
    );
    assert!(!cert_der.is_empty() && !key_der.is_empty());
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

/// THE `.1.6.1` acceptance (deferral #4's first half): a role-serving node's
/// enrollment writes the `incarnations` row with the §8.1 facts the node
/// declared, the row is inspectable through the tenant_admin surface, a
/// re-enrollment cannot duplicate it (the nodes PK refuses before the writer),
/// and a plain `nod_…` node (no role) records no incarnation.
#[tokio::test]
async fn enrollment_writes_the_incarnation_row_with_its_facts() {
    let _guard = enroll_guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (tenant, alice) = bootstrap_admin(&client, &base).await;

    // The dev wiring: the node id IS the role wire id it serves.
    let (status, role) = post_json(
        &client,
        format!("{base}/v1/enrollments"),
        None,
        json!({ "kind": "role", "name": "incarnating", "tenant_id": tenant }),
    )
    .await;
    assert_eq!(status, 200, "enroll role: {role}");
    let role_id = role["principal_id"].as_str().unwrap().to_string();

    let token = issue_token(&client, &base, &alice, &tenant, &role_id, "host-inc", None).await;
    let (status, enrolled) = post_json(
        &client,
        format!("{base}/v1/nodes/enroll"),
        None,
        json!({
            "token_id": token["token_id"],
            "node_id": role_id,
            "host_claim": "host-inc",
            "nonce": token["nonce"],
            "key_secret": "dev-secret-inc",
            "provider": "fake",
            "model": "scripted-1",
            "harness": "fake",
            "config": { "temperature": 0.2 },
        }),
    )
    .await;
    assert_eq!(status, 200, "enroll node: {enrolled}");
    let incarnation_id = enrolled["incarnation_id"]
        .as_str()
        .expect("the role node's enrollment returns its incarnation id")
        .to_string();
    assert!(
        incarnation_id.starts_with("inc_"),
        "the incarnation id is branded: {incarnation_id}"
    );

    // The row exists with the declared facts (a separate connection).
    let row: (
        String,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<Value>,
    ) = sqlx::query_as(
        "SELECT role_id, provider, model, harness, config FROM incarnations \
             WHERE incarnation_id = $1",
    )
    .bind(&incarnation_id)
    .fetch_one(&pool)
    .await
    .expect("the incarnation row");
    assert_eq!(row.0, role_id);
    assert_eq!(row.1.as_deref(), Some("fake"));
    assert_eq!(row.2.as_deref(), Some("scripted-1"));
    assert_eq!(row.3.as_deref(), Some("fake"));
    assert_eq!(
        row.4.as_ref().and_then(|c| c.get("temperature")),
        Some(&serde_json::json!(0.2)),
        "the config rides verbatim"
    );

    // Inspectable through the tenant_admin surface (no database surgery).
    let response = client
        .get(format!("{base}/v1/admin/incarnations?tenant_id={tenant}"))
        .header(PRINCIPAL_HEADER, &alice)
        .send()
        .await
        .expect("list request");
    assert_eq!(response.status().as_u16(), 200);
    let list: Value = response.json().await.expect("list json");
    let incarnations = list["incarnations"].as_array().expect("array");
    assert_eq!(incarnations.len(), 1, "exactly one incarnation: {list}");
    assert_eq!(incarnations[0]["incarnation_id"], json!(incarnation_id));
    assert_eq!(incarnations[0]["harness"], json!("fake"));

    // A re-enrollment attempt is refused BEFORE the incarnation writer — the
    // one-token-per-node index makes a second token unissuable, so the only
    // re-enrollment attempt the dev profile can make is the CONSUMED token's
    // reuse ("the token was already used") — and no second incarnation row can
    // ever be written (the writer sits inside the enroll transaction, after
    // the refusal ladder). Rotation never touches this table (no writer).
    let (status2, refused) = post_json(
        &client,
        format!("{base}/v1/nodes/enroll"),
        None,
        json!({
            "token_id": token["token_id"],
            "node_id": role_id,
            "host_claim": "host-inc",
            "nonce": token["nonce"],
            "key_secret": "another-secret",
            "harness": "fake",
        }),
    )
    .await;
    assert_eq!(status2, 401, "re-enrollment is refused: {refused}");
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM incarnations WHERE role_id = $1")
        .bind(&role_id)
        .fetch_one(&pool)
        .await
        .expect("count incarnations");
    assert_eq!(count, 1, "re-enrollment duplicated nothing");

    // A plain nod_ node (no role) records NO incarnation — the hierarchy's
    // role_id is NOT NULL, and the node serves no role.
    let plain = "nod_00000000-0000-7000-8000-000000000161";
    let token3 = issue_token(&client, &base, &alice, &tenant, plain, "host-plain", None).await;
    let (status3, plain_enroll) = post_json(
        &client,
        format!("{base}/v1/nodes/enroll"),
        None,
        json!({
            "token_id": token3["token_id"],
            "node_id": plain,
            "host_claim": "host-plain",
            "nonce": token3["nonce"],
            "key_secret": "plain-secret",
            "harness": "fake",
        }),
    )
    .await;
    assert_eq!(status3, 200, "the plain node enrolls: {plain_enroll}");
    assert_eq!(
        plain_enroll.get("incarnation_id"),
        Some(&Value::Null),
        "a role-less node records no incarnation"
    );
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM incarnations WHERE tenant_id = $1")
        .bind(&tenant)
        .fetch_one(&pool)
        .await
        .expect("count tenant incarnations");
    assert_eq!(total, 1, "still exactly the role node's incarnation");
}
