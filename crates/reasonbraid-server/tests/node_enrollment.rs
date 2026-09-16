//! Integration tests for dev-profile node enrollment (`PHASE-1.2.1`; backlog 11).
//!
//! The flow: a principal holding `TenantAdmin` — a human OR an agent role
//! (`SIGNOFF-REPAIR.4.1.4`) — issues a ONE-TIME token (bound to tenant, node id,
//! host claim, nonce, expiry — §16.2), then the node consumes it at
//! `/v1/nodes/enroll` with its dev signing secret. The token is the credential; the
//! server lands the host + node + key rows in the 0007/0008 identity tables and
//! audits every attempt. Certificate issuance is deferred to Phase 2 (ADR-007).
//! Like the other PostgreSQL suites, these tests skip without `DATABASE_URL`.

#[path = "support/mod.rs"]
mod pg_test_support;

#[path = "support/cleanup.rs"]
mod pg_cleanup;

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

/// Issue a token and wait for the PRODUCT's own expiry to pass it
/// (`SIGNOFF-REPAIR.4.1.2.1`).
///
/// This suite used to mint a lapsed token by asking for `ttl_seconds: -1`, for a
/// good reason `.4.1.1` wrote down: the lapse must happen through the product's
/// own expiry rather than through a test-side write to the very row under
/// measurement. That reason is preserved here and the mechanism is not, because
/// the route now refuses a lifetime outside `1..=86_400` — a token born expired
/// is an operator error that can never be redeemed, and silently accepting one
/// is how `.4.1.1`'s lockout was reachable in the first place.
///
/// So the token is issued with the shortest LEGAL lifetime and the test waits
/// for the database's own clock to pass it. No write touches the row, and the
/// resulting state is byte-for-byte the state production reaches: `expires_at`
/// in the past, `used_at IS NULL`, `superseded_at IS NULL`.
async fn issue_and_let_it_lapse(
    client: &reqwest::Client,
    base: &str,
    pool: &PgPool,
    principal: &str,
    tenant: &str,
    node_id: &str,
    host_claim: &str,
) -> Value {
    let token = issue_token(
        client,
        base,
        principal,
        tenant,
        node_id,
        host_claim,
        Some(1),
    )
    .await;
    let token_id = token["token_id"].as_str().expect("a token id").to_owned();
    for _ in 0..60 {
        let lapsed: bool = sqlx::query_scalar(
            "SELECT now() > expires_at + interval '500 milliseconds' \
             FROM node_enrollment_tokens WHERE token_id = $1",
        )
        .bind(&token_id)
        .fetch_one(pool)
        .await
        .expect("read the token's own expiry");
        if lapsed {
            return token;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    panic!("the token never lapsed — its expiry is not moving as the route reports it");
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
    let (stored_fp, stored_der, stored_expires): (String, Vec<u8>, chrono::DateTime<chrono::Utc>) =
        sqlx::query_as(
            "SELECT cert_fingerprint, cert_der, expires_at FROM node_certificates \
             WHERE node_id = $1",
        )
        .bind(node_id)
        .fetch_one(&pool)
        .await
        .expect("read cert row");
    assert_eq!(stored_fp, enrolled["cert_fingerprint"].as_str().unwrap());
    // `SIGNOFF-REPAIR.3.4.3.1.1`: the stored and returned expiry must be the one
    // the CERTIFICATE carries — the only expiry a verifier enforces. This path
    // previously re-derived it from a `now` sampled BEFORE the enrollment
    // transaction's database work, so the stored value under-reported the
    // certificate's real validity by however long that work took.
    let signed = reasonbraid_server::ca::leaf_not_after(&stored_der).expect("the leaf parses");
    assert_eq!(
        stored_expires, signed,
        "the stored expiry must be the certificate's own not_after"
    );
    let returned: chrono::DateTime<chrono::Utc> = enrolled["cert_expires_at"]
        .as_str()
        .expect("cert_expires_at")
        .parse()
        .expect("an RFC 3339 instant");
    assert_eq!(
        returned, signed,
        "the expiry handed to the node must be the certificate's own not_after"
    );
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
/// `SIGNOFF-REPAIR.3.4.3.1.1` — the server has TWO clocks, and their agreement
/// is load-bearing rather than incidental.
///
/// The instants it hands a node come from both: `decided_at` from PostgreSQL
/// `clock_timestamp()`, the certificate's `not_after` from the server process.
/// `.3.4.3.1.2` will let a node correct for its own offset against "the
/// server's time" — a phrase that only means something if these two agree. On
/// one host they do; nothing in the code said so, and no control would have
/// noticed if they stopped.
///
/// ⚠️ What this measures is the divergence NOT explained by the round trip: the
/// database instant is sampled between two process instants, so a value inside
/// that interval is indistinguishable from zero skew, and only a value outside
/// it is skew at all. That makes the measurement sound without needing the
/// round trip to be fast.
#[tokio::test]
async fn the_servers_database_and_process_clocks_agree() {
    let _guard = enroll_guard().await;
    let Some(pool) = pool().await else { return };

    /// The declared tolerance. `.3.4.3.1.2` may assume the server's two clocks
    /// agree to within this; anything larger makes "the server's time"
    /// ambiguous and that leaf's correction unsound.
    const TOLERANCE: chrono::Duration = chrono::Duration::seconds(1);

    let mut worst = chrono::Duration::zero();
    for _ in 0..5 {
        let before = chrono::Utc::now();
        let db: chrono::DateTime<chrono::Utc> = sqlx::query_scalar("SELECT clock_timestamp()")
            .fetch_one(&pool)
            .await
            .expect("the database states its clock");
        let after = chrono::Utc::now();
        let divergence = if db < before {
            before - db
        } else if db > after {
            db - after
        } else {
            chrono::Duration::zero()
        };
        if divergence > worst {
            worst = divergence;
        }
    }
    assert!(
        worst <= TOLERANCE,
        "the server's database and process clocks diverge by {worst} beyond the \
         request round trip, past the declared tolerance of {TOLERANCE}. \
         `SIGNOFF-REPAIR.3.4.3.1.2` assumes they agree; fix the deployment's \
         clocks or re-open that assumption rather than raising this bound."
    );
}

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
    let leaf = reasonbraid_server::ca::issue_node_leaf(
        &first,
        "nod_00000000-0000-7000-8000-000000000999",
        "host-persist",
    )
    .expect("the fixture host claim is a valid SAN");
    assert!(!leaf.cert_der.is_empty() && !leaf.key_der.is_empty());
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

    // Expired token — a distinct node id: one UNUSED token per node is the 0008
    // invariant. The lapse is the product's own expiry, waited for rather than
    // written (`SIGNOFF-REPAIR.4.1.2.1`).
    let expired_node = "nod_00000000-0000-7000-8000-000000000103";
    let expired = issue_and_let_it_lapse(
        &client,
        &base,
        &pool,
        &alice,
        &tenant,
        expired_node,
        "host-b",
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

// ── Token issuance as one guarded transaction (`SIGNOFF-REPAIR.3.3.4.10.1`) ──
//
// The FIRST of this parent's three children. What is under test is the
// transaction, not the token: the admission, the INSERT and the final effect
// record now share one commit under the tenant's SHARED authority guard, where
// the admission used to commit in its own transaction and the INSERT then ran on
// the connection pool with nothing recording what it did.
//
// ⚠️ The guard here is SHARED, so a lock-holding fixture cannot discriminate this
// repair in either mode — the superseded admission took the shared guard too.
// What discriminates is ATOMICITY, and the evidence-rollback control below is
// the one that goes red on the old shape. The ordering control that follows it
// is a REGRESSION control, and is labelled as one rather than presented as proof.

use reasonbraid_core::{
    AdministrativeOperation, AdministrativeOutcome, AdministrativeRefusal, TenantId,
};
use reasonbraid_server::load_tenant_administrative_effect;

/// The issuance POST, returning status, the receipt and the body. Every answer
/// that reached an admission carries the receipt — that is the contract.
async fn issue(
    client: &reqwest::Client,
    base: &str,
    principal: &str,
    body: Value,
) -> (u16, Option<String>, Value) {
    let response = client
        .post(format!("{base}/v1/nodes/enroll-tokens"))
        .header(PRINCIPAL_HEADER, principal)
        .json(&body)
        .send()
        .await
        .expect("issuance request");
    let status = response.status().as_u16();
    let receipt = response
        .headers()
        .get("x-reasonbraid-authorization")
        .map(|v| v.to_str().expect("an ASCII receipt").to_owned());
    (
        status,
        receipt,
        response.json().await.expect("issuance json"),
    )
}

async fn token_count(pool: &PgPool) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM node_enrollment_tokens")
        .fetch_one(pool)
        .await
        .expect("count tokens")
}

async fn effect_count(pool: &PgPool) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM administrative_effects")
        .fetch_one(pool)
        .await
        .expect("count effects")
}

async fn effect_of(
    pool: &PgPool,
    tenant: &str,
    receipt: Option<String>,
) -> Option<reasonbraid_core::AdministrativeEffectRecord> {
    let tenant: TenantId = tenant.parse().expect("a tenant id");
    load_tenant_administrative_effect(
        pool,
        tenant,
        receipt
            .expect("every admitted answer carries its receipt")
            .parse()
            .expect("the receipt is a record id"),
    )
    .await
    .expect("read the effect back")
}

#[tokio::test]
async fn issuance_commits_its_admission_token_and_effect_together() {
    let _guard = enroll_guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();
    let (tenant, alice) = bootstrap_admin(&client, &base).await;
    let node_id = "nod_00000000-0000-7000-8000-0000000001a1";

    let (status, receipt, body) = issue(
        &client,
        &base,
        &alice,
        json!({ "tenant_id": tenant, "node_id": node_id, "host_claim": "host-a" }),
    )
    .await;
    assert_eq!(status, 200, "an eligible administrator issues: {body}");
    assert!(body["token_id"].as_str().unwrap().starts_with("ntk_"));
    assert!(!body["nonce"].as_str().unwrap().is_empty());
    assert_eq!(token_count(&pool).await, 1);

    let effect = effect_of(&pool, &tenant, receipt)
        .await
        .expect("the effect committed with its token");
    assert_eq!(
        effect.operation,
        AdministrativeOperation::NodeEnrollTokenIssue {
            node_id: reasonbraid_core::AdministrativeTargetId::new(node_id).unwrap()
        }
    );
    assert!(effect.outcome.changed_protected_state());
    assert_eq!(
        effect.submitted_reason, None,
        "issuance takes no caller reason, so none is invented"
    );

    // `expires_at` is now database time from inside the transaction plus the
    // TTL, so it agrees with the instant the decision was made rather than with
    // a process clock read before the guard wait. The default TTL is 3600 s.
    let expires: chrono::DateTime<chrono::Utc> = body["expires_at"]
        .as_str()
        .unwrap()
        .parse()
        .expect("an RFC3339 expiry");
    let gap = (expires - effect.effected_at).num_seconds();
    assert_eq!(
        gap, 3600,
        "the expiry is exactly the TTL after the transaction's own decision time"
    );
}

#[tokio::test]
async fn an_outstanding_token_is_refused_and_the_refusal_is_recorded() {
    let _guard = enroll_guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();
    let (tenant, alice) = bootstrap_admin(&client, &base).await;
    let node_id = "nod_00000000-0000-7000-8000-0000000001a2";
    let body = json!({ "tenant_id": tenant, "node_id": node_id, "host_claim": "host-a" });

    let (status, _, _) = issue(&client, &base, &alice, body.clone()).await;
    assert_eq!(status, 200);

    // ⛔ The established contract, byte for byte: 409 `invalid_command` with the
    // same message. Its NEW property is that the refusal now commits — under a
    // raising unique violation the transaction would abort and take the
    // admission and the effect record with it.
    let (status, receipt, refused) = issue(&client, &base, &alice, body).await;
    assert_eq!(status, 409, "a re-issue is refused: {refused}");
    assert_eq!(refused["code"], json!("invalid_command"));
    assert!(refused["message"]
        .as_str()
        .unwrap()
        .contains("already exists"));
    assert_eq!(token_count(&pool).await, 1, "the refusal issued no token");

    let effect = effect_of(&pool, &tenant, receipt)
        .await
        .expect("an admitted caller's refused operation is recorded");
    let AdministrativeOutcome::Refused { code, detail } = &effect.outcome else {
        panic!("expected a recorded refusal, got {:?}", effect.outcome)
    };
    // The record names the code the RESPONSE carries.
    assert_eq!(*code, AdministrativeRefusal::InvalidCommand);
    assert_eq!(refused["code"], json!(code.as_str()));
    assert!(detail.as_str().contains("already outstanding"), "{detail}");
    assert!(!effect.outcome.changed_protected_state());
}

/// `SIGNOFF-REPAIR.4.1.1` — a token that LAPSES unused must not lock its node
/// out of ever being issued another one.
///
/// Migration 0018's partial index keys `(node_id) WHERE used_at IS NULL`, and an
/// EXPIRED token still has `used_at IS NULL` — so it stayed in that index for
/// ever and every later issuance for the node answered `409`. The 409's own
/// message told the operator to "consume or expire it before issuing another":
/// expiring it was the advertised recovery, and it was the thing that did not
/// work. The lapsed token is itself unusable, so the node's normal enrollment
/// path was closed permanently.
///
/// Three arms, and the middle one is the defect:
///   1. a lapsed token really is dead — it enrolls nothing;
///   2. issuance for that node succeeds again;
///   3. two LIVE unused tokens still cannot coexist — 0018's invariant is intact.
#[tokio::test]
async fn a_lapsed_token_is_superseded_rather_than_locking_the_node_out() {
    let _guard = enroll_guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();
    let (tenant, alice) = bootstrap_admin(&client, &base).await;
    let node_id = "nod_00000000-0000-7000-8000-0000000001a6";
    let reissue = json!({ "tenant_id": tenant, "node_id": node_id, "host_claim": "host-a" });

    // The lapse happens through the product's own expiry rather than through a
    // test-side write to the very row under measurement — `.4.1.1`'s reason,
    // kept, with `.4.1.2.1`'s legal minimum lifetime as the mechanism.
    let first =
        issue_and_let_it_lapse(&client, &base, &pool, &alice, &tenant, node_id, "host-a").await;
    let lapsed_id = first["token_id"].as_str().unwrap().to_owned();
    let lapsed_nonce = first["nonce"].as_str().unwrap().to_owned();

    // Arm 1 — the lapsed token is dead. Nothing about the lockout is softened by
    // the old token still working, because it does not.
    let (status, dead) = enroll_node(
        &client,
        &base,
        &lapsed_id,
        node_id,
        "host-a",
        &lapsed_nonce,
        "dev-secret",
    )
    .await;
    assert_eq!(status, 401, "a lapsed token must enroll nothing: {dead}");
    assert!(
        dead["message"].as_str().unwrap().contains("expired"),
        "{dead}"
    );

    // Arm 2 — THE DEFECT. Before the repair this answered `409 invalid_command`,
    // for ever, for this node id.
    let (status, _, issued) = issue(&client, &base, &alice, reissue.clone()).await;
    assert_eq!(
        status, 200,
        "a lapsed token must not block reissuance: {issued}"
    );
    let live_id = issued["token_id"].as_str().unwrap().to_owned();
    assert_ne!(
        live_id, lapsed_id,
        "reissuance mints a NEW token; it never rewrites the lapsed row, whose id \
         the enrollment audit already names"
    );

    // The lapsed row is RETAINED and marked — not deleted, and not dishonestly
    // marked `used_at`, which it never was.
    let (superseded, used): (
        Option<chrono::DateTime<chrono::Utc>>,
        Option<chrono::DateTime<chrono::Utc>>,
    ) = sqlx::query_as(
        "SELECT superseded_at, used_at FROM node_enrollment_tokens WHERE token_id = $1",
    )
    .bind(&lapsed_id)
    .fetch_one(&pool)
    .await
    .expect("the lapsed row is retained, not deleted");
    assert!(
        superseded.is_some(),
        "the lapsed token is stamped superseded"
    );
    assert_eq!(
        used, None,
        "a token that was never redeemed is never recorded as used"
    );

    // Arm 3 — 0018's invariant, unchanged: the token just issued is LIVE, so a
    // third issuance is still the typed refusal. A repair that let two live
    // tokens coexist would re-open what 0018 closed.
    let (status, _, refused) = issue(&client, &base, &alice, reissue).await;
    assert_eq!(
        status, 409,
        "a LIVE unused token still blocks another: {refused}"
    );
    assert_eq!(refused["code"], json!("invalid_command"));
}

/// `SIGNOFF-REPAIR.4.1.1`'s stated BOUND, pinned so it cannot rot unnoticed: the
/// supersede is scoped to the ADMITTED tenant and never stamps a row it does not
/// own — a foreign stamp would be a write into another tenant's rows while holding
/// only this tenant's guard.
///
/// ⛔ This control deliberately does NOT assert the status the foreign tenant
/// receives. `SIGNOFF-REPAIR.3.3.4.10.1` reproduced the cross-tenant `409` (0018's
/// index is keyed on `node_id` alone, with no tenant column) and removed its probe
/// rather than commit a control that would enshrine the defect; `.3.5` owns it and
/// may well make that answer a `200`. What must hold under BOTH the current global
/// index and that future repair is everything below.
#[tokio::test]
async fn a_foreign_tenants_issuance_never_supersedes_a_row_it_does_not_own() {
    let _guard = enroll_guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();
    let (tenant_a, alice) = bootstrap_admin(&client, &base).await;
    let (tenant_b, bob) = bootstrap_admin(&client, &base).await;
    let node_id = "nod_00000000-0000-7000-8000-0000000001a7";

    // Alice's tenant lapses a token for the node.
    issue_and_let_it_lapse(&client, &base, &pool, &alice, &tenant_a, node_id, "host-a").await;

    // Bob, fully admitted in his OWN tenant, issues for the same node identity.
    // Whatever he is answered, he must not have touched Alice's row.
    let (bob_status, _, bob_body) = issue(
        &client,
        &base,
        &bob,
        json!({ "tenant_id": tenant_b, "node_id": node_id, "host_claim": "host-b" }),
    )
    .await;

    let alice_rows: Vec<(Option<chrono::DateTime<chrono::Utc>>,)> = sqlx::query_as(
        "SELECT superseded_at FROM node_enrollment_tokens WHERE node_id = $1 AND tenant_id = $2",
    )
    .bind(node_id)
    .bind(&tenant_a)
    .fetch_all(&pool)
    .await
    .expect("alice's rows for the node");
    assert_eq!(
        alice_rows.len(),
        1,
        "the foreign issuance neither added nor removed a row in alice's tenant \
         (bob was answered {bob_status}: {bob_body})"
    );
    assert_eq!(
        alice_rows[0].0, None,
        "a foreign tenant's issuance never stamps a row it does not own \
         (bob was answered {bob_status}: {bob_body})"
    );

    // And the tenant that DOES own the lapsed token recovers, which is the repair.
    let (status, _, issued) = issue(
        &client,
        &base,
        &alice,
        json!({ "tenant_id": tenant_a, "node_id": node_id, "host_claim": "host-a" }),
    )
    .await;
    assert_eq!(status, 200, "the owning tenant reissues: {issued}");
}

#[tokio::test]
async fn a_denied_issuance_records_no_effect_and_no_token() {
    let _guard = enroll_guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();
    let (tenant, _alice) = bootstrap_admin(&client, &base).await;
    let (status, role) = post_json(
        &client,
        format!("{base}/v1/enrollments"),
        None,
        json!({ "kind": "role", "name": "ordinary", "tenant_id": tenant }),
    )
    .await;
    assert_eq!(status, 200, "enroll role: {role}");
    let outsider = role["principal_id"].as_str().unwrap().to_string();

    let (status, receipt, body) = issue(
        &client,
        &base,
        &outsider,
        json!({
            "tenant_id": tenant,
            "node_id": "nod_00000000-0000-7000-8000-0000000001a3",
            "host_claim": "host-a",
        }),
    )
    .await;
    assert_eq!(status, 403, "a role without tenant_admin is denied: {body}");
    assert_eq!(token_count(&pool).await, 0, "the denial issued no token");
    assert_eq!(
        effect_count(&pool).await,
        0,
        "a denial never became an operation; the admission record already says denied"
    );
    assert!(effect_of(&pool, &tenant, receipt).await.is_none());
}

#[tokio::test]
async fn a_malformed_node_identity_is_refused_before_any_admission_exists() {
    let _guard = enroll_guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();
    let (tenant, alice) = bootstrap_admin(&client, &base).await;

    let (status, receipt, body) = issue(
        &client,
        &base,
        &alice,
        json!({ "tenant_id": tenant, "node_id": "not-a-node", "host_claim": "host-a" }),
    )
    .await;
    assert_eq!(status, 400, "a malformed node identity is refused: {body}");
    // Deliberately NO receipt: this refusal happens before the transaction that
    // would create an admission, so there is no record to name. An answer that
    // named one would be advertising a record that does not exist.
    assert!(
        receipt.is_none(),
        "a pre-admission refusal names no admission"
    );
    assert_eq!(token_count(&pool).await, 0);
    assert_eq!(effect_count(&pool).await, 0);
}

#[tokio::test]
async fn a_foreign_tenant_issues_nothing_and_leaves_the_other_tenant_untouched() {
    let _guard = enroll_guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();
    let (tenant_a, alice) = bootstrap_admin(&client, &base).await;
    // A second tenant with its own administrator.
    let (status, bob) = post_json(
        &client,
        format!("{base}/v1/enrollments"),
        None,
        json!({ "kind": "human", "name": "bob" }),
    )
    .await;
    assert_eq!(status, 200, "bootstrap bob: {bob}");
    let tenant_b = bob["tenant_id"].as_str().unwrap().to_string();
    let bob_id = bob["principal_id"].as_str().unwrap().to_string();

    // Bob issues in his own tenant.
    let (status, _, _) = issue(
        &client,
        &base,
        &bob_id,
        json!({
            "tenant_id": tenant_b,
            "node_id": "nod_00000000-0000-7000-8000-0000000001b1",
            "host_claim": "host-b",
        }),
    )
    .await;
    assert_eq!(status, 200);
    let before = token_count(&pool).await;
    let b_effects = effect_count(&pool).await;

    // Alice names BOB's tenant. Her grant does not select it.
    let (status, _, body) = issue(
        &client,
        &base,
        &alice,
        json!({
            "tenant_id": tenant_b,
            "node_id": "nod_00000000-0000-7000-8000-0000000001b2",
            "host_claim": "host-b",
        }),
    )
    .await;
    assert_eq!(status, 403, "issuing into another tenant is denied: {body}");
    assert_eq!(token_count(&pool).await, before, "no token was created");
    assert_eq!(
        effect_count(&pool).await,
        b_effects,
        "a denial records no effect in any tenant"
    );
    // Alice's own tenant is untouched by the attempt.
    let a_tokens: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM node_enrollment_tokens WHERE tenant_id = $1")
            .bind(&tenant_a)
            .fetch_one(&pool)
            .await
            .expect("count alice's tokens");
    assert_eq!(a_tokens, 0);
}

#[tokio::test]
async fn an_evidence_failure_rolls_the_token_back_and_the_route_recovers() {
    let _guard = enroll_guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();
    let (tenant, alice) = bootstrap_admin(&client, &base).await;
    let body = json!({
        "tenant_id": tenant,
        "node_id": "nod_00000000-0000-7000-8000-0000000001c1",
        "host_claim": "host-a",
    });

    // ⛔ THE discriminating control for this child. The guard mode is shared, so
    // no lock-holding fixture can tell this repair from the one it replaced;
    // what changed is that the token and its evidence are ONE commit. Under the
    // superseded shape the token was inserted on the pool and no effect was
    // written at all, so this control cannot even be expressed against it.
    //
    // NOT VALID applies to new rows only, leaving existing evidence untouched.
    sqlx::query(
        "ALTER TABLE administrative_effects \
         ADD CONSTRAINT signoff_repair_3_3_4_10_1_evidence_fault CHECK (false) NOT VALID",
    )
    .execute(&pool)
    .await
    .expect("install the evidence fault");

    // Observe, REMOVE THE FAULT, then assert: an assertion that fires while the
    // fault stands leaks it into every later test in the suite.
    let (status, receipt, refused) = issue(&client, &base, &alice, body.clone()).await;
    let tokens_after_fault = token_count(&pool).await;

    sqlx::query(
        "ALTER TABLE administrative_effects \
         DROP CONSTRAINT signoff_repair_3_3_4_10_1_evidence_fault",
    )
    .execute(&pool)
    .await
    .expect("remove the evidence fault");

    assert_eq!(
        status, 500,
        "an evidence failure is not reported as success: {refused}"
    );
    assert!(
        receipt.is_none(),
        "a transaction that did not commit advertises no receipt"
    );
    assert_eq!(
        tokens_after_fault, 0,
        "the token did not survive its evidence"
    );

    // Exact recovery: with the fault gone the same request succeeds and records.
    let (status, receipt, body2) = issue(&client, &base, &alice, body).await;
    assert_eq!(status, 200, "the route recovers: {body2}");
    assert_eq!(token_count(&pool).await, 1);
    let effect = effect_of(&pool, &tenant, receipt).await.expect("recorded");
    assert!(effect.outcome.changed_protected_state());
}

/// `SIGNOFF-REPAIR.3.5.1` — a node id enrolled by one tenant, claimed by another.
///
/// `nodes.node_id` is a GLOBAL primary key, so a node identity belongs to at
/// most one tenant ever. The redemption path knows a unique violation is fatal —
/// its own comment says "a unique-violation probe would abort the transaction" —
/// and guards with `EXISTS(SELECT 1 FROM nodes WHERE node_id = $1 AND
/// tenant_id = $2)`. That guard is TENANT-SCOPED against a GLOBAL key: when the
/// node exists in a DIFFERENT tenant it reports false, the insert runs anyway,
/// and the primary key aborts the transaction.
///
/// ⚠️ Reachable today. The one-unused-token index blocks a second UNUSED token,
/// not a second token: once the first tenant has enrolled, its token is `used`,
/// so the second tenant's issuance succeeds and only the redemption fails.
#[tokio::test]
async fn a_node_id_enrolled_by_one_tenant_refuses_the_next_tenant_cleanly() {
    let _guard = enroll_guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let client = reqwest::Client::new();
    let base = server.base();

    let node_id = "nod_00000000-0000-7000-8000-0000000009f1";

    // Tenant A issues, redeems, and owns the node.
    let (tenant_a, alice) = bootstrap_admin(&client, &base).await;
    let token_a = issue_token(&client, &base, &alice, &tenant_a, node_id, "host-a", None).await;
    let (status, enrolled) = enroll_node(
        &client,
        &base,
        token_a["token_id"].as_str().unwrap(),
        node_id,
        "host-a",
        token_a["nonce"].as_str().unwrap(),
        "secret-a",
    )
    .await;
    assert_eq!(status, 200, "tenant A enrols the node: {enrolled}");

    // Tenant B. Its token for the SAME node id is issued without complaint,
    // because the partial index only forbids a second UNUSED token and A's is
    // now used — so the collision is not caught at issuance.
    let (tenant_b, bob) = bootstrap_admin(&client, &base).await;
    assert_ne!(tenant_a, tenant_b, "two distinct tenants");
    // Raw: this issuance is itself part of what is being measured, and a helper
    // that decodes JSON would panic on an empty body instead of reporting it.
    let issue = client
        .post(format!("{base}/v1/nodes/enroll-tokens"))
        .header(PRINCIPAL_HEADER, &bob)
        .json(&json!({ "tenant_id": tenant_b, "node_id": node_id, "host_claim": "host-b" }))
        .send()
        .await
        .expect("the second tenant's issuance answers at all");
    let issue_status = issue.status().as_u16();
    let issue_body = issue.text().await.unwrap_or_default();
    assert_eq!(
        issue_status, 200,
        "the second tenant's token issuance: {issue_status} {issue_body:?}"
    );
    let token_b: Value = serde_json::from_str(&issue_body).expect("issuance json");

    // Raw, not `enroll_node`: the point of this control is what comes back, and
    // an aborted transaction may answer with no body at all — which a JSON
    // decode would turn into a panic in the helper instead of a measurement.
    let response = client
        .post(format!("{base}/v1/nodes/enroll"))
        .json(&json!({
            "token_id": token_b["token_id"].as_str().unwrap(),
            "node_id": node_id,
            "host_claim": "host-b",
            "nonce": token_b["nonce"].as_str().unwrap(),
            "key_secret": "secret-b",
        }))
        .send()
        .await
        .expect("the second tenant's enrollment answers at all");
    let status = response.status().as_u16();
    let body = response.text().await.unwrap_or_default();

    // The identity must not move tenants, whatever else happens.
    let (owner,): (String,) = sqlx::query_as("SELECT tenant_id FROM nodes WHERE node_id = $1")
        .bind(node_id)
        .fetch_one(&pool)
        .await
        .expect("read the node row");
    assert_eq!(
        owner, tenant_a,
        "the node stays with the tenant that enrolled it"
    );

    assert_ne!(
        status, 500,
        "claiming a node id owned by another tenant must be a REFUSAL, not an \
         unhandled primary-key violation — got {status}: {body}"
    );
    assert!(
        body.contains("already enrolled"),
        "the refusal reuses the same-tenant duplicate's wording and names no \
         owner — got {status}: {body}"
    );

    // ⛔ THE DANGEROUS PATH, and the reason the repair reads the OWNER rather
    // than merely widening the existence test. The replacement branch swaps the
    // node's key and issues it a fresh workload certificate WITHOUT a tenant
    // check, and it fires once every certificate is revoked. A check that only
    // asked "does this node exist anywhere" would hand tenant B the node the
    // moment tenant A revoked — a takeover, worse than the 500 being repaired.
    let revoked = sqlx::query("UPDATE node_certificates SET revoked_at = now() WHERE node_id = $1")
        .bind(node_id)
        .execute(&pool)
        .await
        .expect("revoke every certificate");
    assert!(
        revoked.rows_affected() > 0,
        "there was a certificate to revoke"
    );

    // B's token is still UNUSED — its redemption was refused — so the
    // one-unused-token index correctly declines to issue a second, and the
    // takeover attempt reuses the one it holds.
    let token_b2 = &token_b;
    let takeover = client
        .post(format!("{base}/v1/nodes/enroll"))
        .json(&json!({
            "token_id": token_b2["token_id"].as_str().unwrap(),
            "node_id": node_id,
            "host_claim": "host-b",
            "nonce": token_b2["nonce"].as_str().unwrap(),
            "key_secret": "secret-b-takeover",
        }))
        .send()
        .await
        .expect("the takeover attempt answers");
    let takeover_status = takeover.status().as_u16();
    let takeover_body = takeover.text().await.unwrap_or_default();
    assert!(
        (400..500).contains(&takeover_status),
        "a foreign tenant must be REFUSED the replacement path, not served and \
         not 500 — got {takeover_status}: {takeover_body}"
    );
    let (still_owned_by,): (String,) =
        sqlx::query_as("SELECT tenant_id FROM nodes WHERE node_id = $1")
            .bind(node_id)
            .fetch_one(&pool)
            .await
            .expect("read the node row");
    assert_eq!(
        still_owned_by, tenant_a,
        "the node must not change tenants when its certificates are revoked"
    );
}

/// `SIGNOFF-REPAIR.4.1.2.1` — the token's lifetime is validated before anything
/// computes with it.
///
/// REPRODUCED FIRST, against the live route, and the measurement was three
/// different failures rather than the one that was suspected:
///
/// | sent | before the repair |
/// | --- | --- |
/// | `i64::MAX`, UNAUTHORIZED caller | **panic**, connection dropped — `TimeDelta::seconds` is documented to panic above `i64::MAX / 1_000`, and it ran before any authorization |
/// | `1_000_000_000_000_000`, admin | **panic** inside the tenant's authority guard — `at + ttl` leaves chrono's date range |
/// | a century, admin | `200`, `expires_at: 2126-09-14` — a bearer credential outliving everyone who could reason about it |
///
/// ⚠️ The unauthorized arm is the sharpest of the three: `resolve_principal`
/// only parses the header, so the arithmetic sat in front of the gate. The two
/// middling values answered `403` correctly, which is exactly why reading the
/// source was not enough — only the extreme value crossed the panic threshold,
/// and only from outside the guard.
#[tokio::test]
async fn the_token_lifetime_is_validated_before_anything_computes_with_it() {
    let _guard = enroll_guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();
    let (tenant, admin) = bootstrap_admin(&client, &base).await;
    // A syntactically valid principal that holds NO grant in this tenant.
    let stranger = "hpr_00000000-0000-7000-8000-0000000004f1";

    // `node_id` differs per case so a refusal is never explained by the
    // one-live-token index instead of by the lifetime.
    let ask = |principal: String, node: String, ttl: Option<i64>| {
        let client = client.clone();
        let base = base.clone();
        let tenant = tenant.clone();
        async move {
            let mut body = json!({
                "tenant_id": tenant,
                "node_id": node,
                "host_claim": "ttl-host",
            });
            if let Some(t) = ttl {
                body["ttl_seconds"] = json!(t);
            }
            let r = client
                .post(format!("{base}/v1/nodes/enroll-tokens"))
                .header(PRINCIPAL_HEADER, principal)
                .json(&body)
                .send()
                .await
                .expect(
                    "the route must ANSWER — a dropped connection is the panic \
                     this leaf repaired",
                );
            let status = r.status().as_u16();
            let body: Value = r.json().await.expect("a JSON body");
            (status, body)
        }
    };

    // THE REPAIR. Each of these dropped the connection or issued a century.
    for (label, ttl) in [
        ("i64::MAX", i64::MAX),
        ("i64::MIN", i64::MIN),
        ("past chrono's date range", 1_000_000_000_000_000_i64),
        ("a century", 3_155_760_000_i64),
        ("one second past the cap", 86_401_i64),
        ("zero", 0_i64),
        ("negative", -1_i64),
    ] {
        let (status, body) = ask(
            admin.clone(),
            "nod_00000000-0000-7000-8000-0000000004a1".to_string(),
            Some(ttl),
        )
        .await;
        assert_eq!(
            status, 400,
            "{label}: a lifetime outside the range is a typed refusal — got \
             {status}: {body}"
        );
        assert_eq!(body["code"], json!("invalid_command"), "{label}: {body}");
    }

    // The unauthorized arm: the panic sat in FRONT of the guard, so this is the
    // case that made it reachable without any authority at all.
    let (status, body) = ask(
        stranger.to_string(),
        "nod_00000000-0000-7000-8000-0000000004a2".to_string(),
        Some(i64::MAX),
    )
    .await;
    assert_eq!(
        status, 400,
        "an unauthorized caller gets a typed refusal, not a dropped \
         connection: {body}"
    );

    // ⛔ And the refusal must not have swallowed the authorization one: with a
    // LEGAL lifetime the stranger is still refused by the guard, unchanged.
    let (status, body) = ask(
        stranger.to_string(),
        "nod_00000000-0000-7000-8000-0000000004a3".to_string(),
        Some(60),
    )
    .await;
    assert_eq!(
        status, 403,
        "a legal lifetime still reaches the authority guard: {body}"
    );

    // The boundary is inclusive, and the default still issues — the repair
    // bounds the range, it does not narrow ordinary use.
    let (status, body) = ask(
        admin.clone(),
        "nod_00000000-0000-7000-8000-0000000004a4".to_string(),
        Some(86_400),
    )
    .await;
    assert_eq!(status, 200, "the cap itself is issuable: {body}");
    let (status, body) = ask(
        admin.clone(),
        "nod_00000000-0000-7000-8000-0000000004a5".to_string(),
        None,
    )
    .await;
    assert_eq!(status, 200, "the default lifetime is unchanged: {body}");
    let expires: chrono::DateTime<chrono::Utc> = body["expires_at"]
        .as_str()
        .expect("an expiry")
        .parse()
        .expect("an RFC3339 expiry");
    let ahead = (expires - chrono::Utc::now()).num_seconds();
    assert!(
        (3_000..=3_600).contains(&ahead),
        "the default is still an hour, not a re-tuned value: {ahead}s"
    );
}

/// `SIGNOFF-REPAIR.4.1.2` — a token does not outlive the authority that issued it.
///
/// The decision, and its three load-bearing properties:
///
///  1. revoking the grant that issued a token **voids** it, and redemption says
///     so in its own words — waiting does not help and neither does re-issuing
///     under the same authority;
///  2. the node's enrollment path is **not** closed by that, because a voided
///     token releases the one-live-token index slot. ⛔ Without that the operator
///     revokes a compromised administrator and can never issue a replacement
///     token for the node — `SIGNOFF-REPAIR.4.1.1`'s lockout, through a
///     different door;
///  3. the selection is **exact**. Both administrators here are in the SAME
///     tenant deliberately: a tenant-wide implementation would pass a
///     cross-tenant check and fail this one.
#[tokio::test]
async fn a_token_does_not_outlive_the_authority_that_issued_it() {
    let _guard = enroll_guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();
    let (tenant, alice) = bootstrap_admin(&client, &base).await;

    // A SECOND administrator in Alice's own tenant — the colleague who is still
    // there after Alice is withdrawn, and the reason property (2) matters.
    let (status, surviving) = post_json(
        &client,
        format!("{base}/v1/enrollments"),
        None,
        json!({
            "kind": "role",
            "name": "surviving-admin",
            "tenant_id": tenant,
            "actions": ["tenant_admin"],
        }),
    )
    .await;
    assert_eq!(status, 200, "the second administrator enrolls: {surviving}");
    let surviving = surviving["principal_id"].as_str().unwrap().to_owned();

    let node_id = "nod_00000000-0000-7000-8000-0000000005a1";
    let other_node = "nod_00000000-0000-7000-8000-0000000005a2";
    let token = issue_token(&client, &base, &alice, &tenant, node_id, "host-void", None).await;
    let token_id = token["token_id"].as_str().unwrap().to_owned();
    let nonce = token["nonce"].as_str().unwrap().to_owned();
    let untouched = issue_token(
        &client,
        &base,
        &surviving,
        &tenant,
        other_node,
        "host-other",
        None,
    )
    .await;

    // The link the whole design rests on, resolved exactly as the product does:
    // the token names its admission, and the admission names the grant it chose.
    let (grant_id,): (String,) = sqlx::query_as(
        "SELECT r.grant_id FROM node_enrollment_tokens t \
         JOIN authorization_records r ON r.record_id = t.issued_under \
         WHERE t.token_id = $1",
    )
    .bind(&token_id)
    .fetch_one(&pool)
    .await
    .expect("the token names the admission that issued it, and it names a grant");

    let revoked = client
        .post(format!("{base}/v1/admin/grants/{grant_id}/revoke"))
        .header(PRINCIPAL_HEADER, &alice)
        .json(&json!({ "tenant_id": tenant, "reason": "the administrator is withdrawn" }))
        .send()
        .await
        .expect("the revocation answers");
    assert_eq!(
        revoked.status().as_u16(),
        200,
        "the revocation succeeds: {}",
        revoked.text().await.unwrap_or_default()
    );

    // (1) The token is dead, and the refusal is its OWN, not "expired".
    let (status, refused) = enroll_node(
        &client,
        &base,
        &token_id,
        node_id,
        "host-void",
        &nonce,
        "dev-secret",
    )
    .await;
    assert_eq!(status, 401, "a voided token enrolls nothing: {refused}");
    let message = refused["message"].as_str().unwrap_or_default();
    assert!(
        message.contains("authority that issued this token has been revoked"),
        "the operator is told the AUTHORITY was withdrawn — re-issuing under it \
         will not help, and waiting will not either: {refused}"
    );
    let (nodes,): (i64,) = sqlx::query_as("SELECT count(*) FROM nodes WHERE node_id = $1")
        .bind(node_id)
        .fetch_one(&pool)
        .await
        .expect("count the node rows");
    assert_eq!(nodes, 0, "and nothing was enrolled");
    let (audited,): (i64,) = sqlx::query_as(
        "SELECT count(*) FROM node_enroll_audit WHERE token_id = $1 AND decision = 'refused'",
    )
    .bind(&token_id)
    .fetch_one(&pool)
    .await
    .expect("count the audit rows");
    assert_eq!(audited, 1, "the refusal leaves an audit row");

    // (2) 🔴 THE LOCKOUT THAT MUST NOT RETURN. The surviving administrator issues
    // again for the same node id: the voided token must have released the slot.
    let (status, _receipt, reissued) = issue(
        &client,
        &base,
        &surviving,
        json!({ "tenant_id": tenant, "node_id": node_id, "host_claim": "host-void" }),
    )
    .await;
    assert_eq!(
        status, 200,
        "a voided token must NOT hold the node's one-live-token slot — that is \
         `.4.1.1`'s lockout re-entered through a different door: {reissued}"
    );

    // (3) The other administrator's token, in the SAME tenant, is untouched.
    let (still_live,): (Option<chrono::DateTime<chrono::Utc>>,) =
        sqlx::query_as("SELECT voided_at FROM node_enrollment_tokens WHERE token_id = $1")
            .bind(untouched["token_id"].as_str().unwrap())
            .fetch_one(&pool)
            .await
            .expect("read the other authority's token");
    assert!(
        still_live.is_none(),
        "revoking one authority voids only the tokens IT issued, even inside the \
         same tenant: {still_live:?}"
    );
}

/// `SIGNOFF-REPAIR.4.1.4` — who may issue a node enrollment token, DRIVEN rather
/// than read off a doc comment.
///
/// Two documentation sites said "an **authorized human** issues a ONE-TIME
/// enrollment token". The handler resolves a principal that may be
/// `GrantSubject::Human(hpr_…)` **or** `GrantSubject::Role(rol_…)` and asks only
/// whether it holds `GrantAction::TenantAdmin`. So the sentence and the code
/// disagreed, and which of the two is wrong is a governance question, not a
/// typo.
///
/// ⭐ The census that decides it: across the server's 117 `resolve_principal`
/// call sites, authorization NEVER depends on the principal's KIND. Every
/// branch on `GrantSubject::Human` — `reader_tenant`, the unreachable invite
/// arm, the bootstrap insert, and MCP's own helper — selects which identity
/// TABLE to read. "Human-only" is not a concept this authority model has, and
/// `ROADMAP.md` §16.4 says why: authorization is over typed actions and
/// resources, deny-by-default. §16.3 goes further and states outright that "a
/// human, service, or agent may delegate a strict subset of its own authority".
///
/// So the CODE is consistent with the roadmap and with every other route, and
/// the sentence is the outlier. This control exists because narrowing a claim
/// to match the code is only honest if the code's behaviour is actually
/// asserted: it drives an agent role holding `tenant_admin` at the route and
/// shows the token it gets.
///
/// ⚠️ This is a real governance property and it is now stated rather than
/// implied by a sentence that said the opposite: **an agent role granted
/// `tenant_admin` can extend the node population.**
#[tokio::test]
async fn an_agent_role_holding_tenant_admin_may_issue_an_enrollment_token() {
    let _guard = enroll_guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    // A tenant must exist before a role can join it.
    let (tenant, _alice) = bootstrap_admin(&client, &base).await;

    // An AGENT role, enrolled with the tenant-admin action explicitly named.
    let (status, agent) = post_json(
        &client,
        format!("{base}/v1/enrollments"),
        None,
        json!({
            "kind": "role",
            "name": "an-administrative-agent",
            "tenant_id": tenant,
            "actions": ["tenant_admin"],
        }),
    )
    .await;
    assert_eq!(
        status, 200,
        "enrol an agent role with tenant_admin: {agent}"
    );
    let role = agent["principal_id"]
        .as_str()
        .expect("the role's wire id")
        .to_string();
    assert!(
        role.starts_with("rol_"),
        "the principal really is an agent role, not a human: {role}"
    );

    // THE MEASUREMENT: the route, driven with that role as the principal.
    let node_id = "nod_00000000-0000-7000-8000-000000000414";
    let (status, issued) = post_json(
        &client,
        format!("{base}/v1/nodes/enroll-tokens"),
        Some(&role),
        json!({ "tenant_id": tenant, "node_id": node_id, "host_claim": "host-agent" }),
    )
    .await;
    println!("  .4.1.4 an agent role issuing an enrollment token: {status}");
    assert_eq!(
        status, 200,
        "an agent role holding tenant_admin issues a token — the code gates on \
         the GRANT, not on the principal's kind, which is what ROADMAP §16.4 \
         specifies and what the narrowed documentation now says: {issued}"
    );

    // Asserted on the STORE as well as the wire, and resolved exactly as the
    // product does: the token names its admission, the admission names the
    // GRANT it chose, and that grant's subject is the agent role.
    //
    // ⛔ Deliberately NOT asserted through `authorization_records.actor`: that
    // column holds a v5 UUID derived from the subject's describe() handle, so checking
    // it would mean re-deriving a contract in a fixture — the third copy
    // `.4.2.6` refused to write. `subject_kind`/`subject_id` on the GRANT are
    // stored verbatim, so they say the same thing without a second derivation.
    let (kind, subject): (String, String) = sqlx::query_as(
        "SELECT g.subject_kind, g.subject_id FROM node_enrollment_tokens t \
         JOIN authorization_records r ON r.record_id = t.issued_under \
         JOIN authority_grants g ON g.grant_id = r.grant_id \
         WHERE t.token_id = $1",
    )
    .bind(issued["token_id"].as_str().expect("the token id"))
    .fetch_one(&pool)
    .await
    .expect("the token names the admission, and the admission names its grant");
    assert_eq!(
        (kind.as_str(), subject.as_str()),
        ("role", role.as_str()),
        "the ledger records an AGENT ROLE as the grant subject that issued this \
         node's enrollment token — the governance property is visible in the \
         audit trail, not only in the response"
    );

    // ⛔ The other half, so this control is not merely "everything is allowed":
    // a role WITHOUT the grant is still refused. The gate is the grant.
    let (status, plain) = post_json(
        &client,
        format!("{base}/v1/enrollments"),
        None,
        json!({ "kind": "role", "name": "an-ordinary-agent", "tenant_id": tenant }),
    )
    .await;
    assert_eq!(status, 200, "enrol an ordinary role: {plain}");
    let ordinary = plain["principal_id"]
        .as_str()
        .expect("the role id")
        .to_string();
    let (status, refused) = post_json(
        &client,
        format!("{base}/v1/nodes/enroll-tokens"),
        Some(&ordinary),
        json!({
            "tenant_id": tenant,
            "node_id": "nod_00000000-0000-7000-8000-000000000415",
            "host_claim": "host-ordinary",
        }),
    )
    .await;
    println!("  .4.1.4 an agent role WITHOUT tenant_admin: {status}");
    assert_eq!(
        status, 403,
        "an agent role without the grant is refused — the kind was never the \
         gate, and the grant still is: {refused}"
    );
    let (tokens,): (i64,) =
        sqlx::query_as("SELECT count(*) FROM node_enrollment_tokens WHERE node_id = $1")
            .bind("nod_00000000-0000-7000-8000-000000000415")
            .fetch_one(&pool)
            .await
            .expect("count the refused node's tokens");
    assert_eq!(tokens, 0, "and the refusal wrote no token");
}

/// `SIGNOFF-REPAIR.4.1.6`: a host claim the certificate library refuses is
/// refused where a human typed it, and never reaches the node that would have
/// carried the panic.
///
/// Reproduced against the unchanged source before this control existed: issuance
/// returned `200`, redemption panicked at `ca.rs:142` — the caller received a
/// TRANSPORT error rather than an answer, the server kept serving, `nodes` held
/// zero rows, and the token was left `used_at = NULL`, so that node id could not
/// be enrolled again until the token lapsed.
///
/// The arms are ordered so a repair that simply refused everything cannot pass:
/// the ordinary claim must still enroll.
#[tokio::test]
async fn a_host_claim_the_library_refuses_is_refused_at_issuance() {
    let _guard = enroll_guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();
    let (tenant, admin) = bootstrap_admin(&client, &base).await;

    // 1. The POSITIVE arm first: an ordinary host claim still issues AND enrolls.
    let good_node = "nod_00000000-0000-7000-8000-0000004160b1";
    let token = issue_token(
        &client,
        &base,
        &admin,
        &tenant,
        good_node,
        "host-4160",
        None,
    )
    .await;
    let (status, enrolled) = enroll_node(
        &client,
        &base,
        token["token_id"].as_str().unwrap(),
        good_node,
        "host-4160",
        token["nonce"].as_str().unwrap(),
        "secret-4160",
    )
    .await;
    assert_eq!(
        status, 200,
        "an ordinary host claim still enrolls: {enrolled}"
    );

    // 2. The refused claim is refused AT ISSUANCE, with a typed answer.
    let bad_node = "nod_00000000-0000-7000-8000-0000004160b2";
    let bad = "h\u{e9}llo";
    let (status, body) = post_json(
        &client,
        format!("{base}/v1/nodes/enroll-tokens"),
        Some(&admin),
        json!({ "tenant_id": tenant, "node_id": bad_node, "host_claim": bad }),
    )
    .await;
    assert_eq!(status, 400, "the refused host claim is refused: {body}");
    assert_eq!(body["code"], json!("invalid_command"), "typed: {body}");
    assert!(
        body["message"]
            .as_str()
            .unwrap_or_default()
            .contains("subject alternative name"),
        "the message names what is wrong: {body}"
    );

    // 3. And it left NOTHING behind — no token, so the node id stays enrollable.
    //    This is the assertion that distinguishes the repair from a late refusal.
    let tokens: i64 =
        sqlx::query_scalar("SELECT count(*) FROM node_enrollment_tokens WHERE node_id = $1")
            .bind(bad_node)
            .fetch_one(&pool)
            .await
            .expect("count tokens");
    assert_eq!(tokens, 0, "a refused issuance writes no token");

    // 4. The proof that it stays enrollable: the SAME node id takes a good claim.
    let token = issue_token(
        &client,
        &base,
        &admin,
        &tenant,
        bad_node,
        "host-4160-ok",
        None,
    )
    .await;
    let (status, enrolled) = enroll_node(
        &client,
        &base,
        token["token_id"].as_str().unwrap(),
        bad_node,
        "host-4160-ok",
        token["nonce"].as_str().unwrap(),
        "secret-4160-b",
    )
    .await;
    assert_eq!(
        status, 200,
        "the node id the bad claim named is still enrollable: {enrolled}"
    );
}

/// `SIGNOFF-REPAIR.4.1.6`, the other half: when a refused host claim DOES reach
/// redemption, the node receives a typed answer instead of a dropped connection.
///
/// ⛔ Issuance now refuses such a claim, so no supported sequence can reach this
/// path — which is exactly why it needs a control. The token row is written
/// directly, modelling the one case that can still produce it: a row that
/// predates the issuance check. Measured on the unrepaired source, this same
/// request panicked at `ca.rs:142` and the client got a transport error.
#[tokio::test]
async fn a_refused_host_claim_reaching_redemption_is_answered_not_dropped() {
    let _guard = enroll_guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();
    let (tenant, admin) = bootstrap_admin(&client, &base).await;
    let node_id = "nod_00000000-0000-7000-8000-0000004160c1";
    let bad = "h\u{e9}llo";

    // A token row as it could only exist from before the issuance check: the
    // issuance ROUTE refuses this claim, so it is written directly.
    let (status, refused) = post_json(
        &client,
        format!("{base}/v1/nodes/enroll-tokens"),
        Some(&admin),
        json!({ "tenant_id": tenant, "node_id": node_id, "host_claim": bad }),
    )
    .await;
    assert_eq!(
        status, 400,
        "the premise: the route itself will not mint this token: {refused}"
    );
    sqlx::query(
        "INSERT INTO node_enrollment_tokens \
         (token_id, tenant_id, node_id, host_claim, nonce, expires_at) \
         VALUES ($1, $2, $3, $4, $5, now() + interval '1 hour')",
    )
    .bind("ntk_4160c1")
    .bind(&tenant)
    .bind(node_id)
    .bind(bad)
    .bind("nonce-4160c1")
    .execute(&pool)
    .await
    .expect("seed the pre-check token row");

    // Redemption ANSWERS. On the unrepaired source this call did not return a
    // response at all, so asserting a status is itself the discriminator.
    let (status, body) = enroll_node(
        &client,
        &base,
        "ntk_4160c1",
        node_id,
        bad,
        "nonce-4160c1",
        "secret-4160c1",
    )
    .await;
    assert_eq!(
        status, 400,
        "a typed refusal, not a dropped connection: {body}"
    );
    assert_eq!(body["code"], json!("invalid_command"), "typed: {body}");

    // And the refusal is effect-free: no node, and the token is still unconsumed
    // rather than burned by a request that could never have succeeded.
    let nodes: i64 = sqlx::query_scalar("SELECT count(*) FROM nodes WHERE node_id = $1")
        .bind(node_id)
        .fetch_one(&pool)
        .await
        .expect("count nodes");
    assert_eq!(nodes, 0, "the refusal wrote no node row");
    let used: Option<chrono::DateTime<chrono::Utc>> = sqlx::query_scalar(
        "SELECT used_at FROM node_enrollment_tokens WHERE token_id = 'ntk_4160c1'",
    )
    .fetch_one(&pool)
    .await
    .expect("read the token");
    assert!(used.is_none(), "the refusal consumed no token: {used:?}");
}

/// `SIGNOFF-REPAIR.4.1.6`: the unit contract the two handlers rest on — the
/// issuer reports a refused claim instead of unwinding, and the checker's
/// verdict is the issuer's own.
#[tokio::test]
async fn the_leaf_issuer_reports_a_refused_host_claim() {
    let Some(pool) = pool().await else { return };
    let ca = ensure_server_ca(&pool).await.expect("server CA");
    let node = "nod_00000000-0000-7000-8000-0000004160d1";

    // ⛔ `.err().expect(...)` rather than `expect_err`: `IssuedLeaf` carries
    // `key_der` and deliberately derives no `Debug`, which `expect_err` would
    // require (ROADMAP §16.5 — key material belongs in no crash report).
    let refusal = reasonbraid_server::ca::issue_node_leaf(&ca, node, "h\u{e9}llo")
        .err()
        .expect("the library refuses this claim");
    assert_eq!(refusal.host_claim, "h\u{e9}llo");
    assert!(
        reasonbraid_server::ca::check_host_claim("h\u{e9}llo").is_err(),
        "the checker agrees with the issuer — one rule, asked once"
    );

    let leaf = reasonbraid_server::ca::issue_node_leaf(&ca, node, "host-4160d")
        .expect("an ordinary claim still issues");
    assert!(!leaf.cert_der.is_empty() && !leaf.key_der.is_empty());
    assert!(reasonbraid_server::ca::check_host_claim("host-4160d").is_ok());
}
