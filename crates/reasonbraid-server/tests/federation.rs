//! The federation trust-agreement proof (`PHASE-8.1.2`, ADR-026, migration
//! 0048): the NAMED tenant-to-tenant pairing is the single capability
//! source. Measured:
//!   - WITHOUT an agreement, a cross-tenant reader sees the NETWORK view
//!     (the pseudonym class — the tenant fields ABSENT);
//!   - a ONE-SIDED proposal widens nothing;
//!   - the BOTH-SIDES accepted agreement widens the visibility to the
//!     TENANT view (the agreed scope — exactly what the agreement names);
//!   - a revocation falls back to the network view;
//!   - a THIRD tenant never inherits the agreement (no transitive
//!     default);
//!   - accepting without a proposal is the typed 409.
//!
//! Run with `scripts/run_pg_tests.sh` locally or the `pg-tests` CI job.
//! Without `DATABASE_URL` these skip, so `make check` stays green offline.

#[path = "support/mod.rs"]
mod pg_test_support;

#[path = "support/cleanup.rs"]
mod pg_cleanup;

use std::net::SocketAddr;
use std::sync::OnceLock;

use reasonbraid_server::{api_router, PRINCIPAL_HEADER};
use serde_json::{json, Value};
use sqlx::PgPool;

static FEDERATION_LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();

async fn guard() -> tokio::sync::MutexGuard<'static, ()> {
    FEDERATION_LOCK
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
            "federation_agreements",
            "quota_events",
            "usage_quotas",
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
            "spend_breakers",
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

async fn post(
    client: &reqwest::Client,
    base: &str,
    path: &str,
    principal: &str,
    body: &Value,
) -> (u16, Value) {
    let response = client
        .post(format!("{base}{path}"))
        .header(PRINCIPAL_HEADER, principal)
        .json(body)
        .send()
        .await
        .expect("post request");
    let status = response.status().as_u16();
    (status, response.json().await.expect("post json"))
}

async fn put(
    client: &reqwest::Client,
    base: &str,
    path: &str,
    principal: &str,
    body: &Value,
) -> (u16, Value) {
    let response = client
        .put(format!("{base}{path}"))
        .header(PRINCIPAL_HEADER, principal)
        .json(body)
        .send()
        .await
        .expect("put request");
    let status = response.status().as_u16();
    let text = response.text().await.expect("put body");
    let body = serde_json::from_str(&text).unwrap_or_else(|_| json!({ "raw": text }));
    (status, body)
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

/// A profile with the explicit per-field policy (the profiles suite's
/// shape): the capabilities stay tenant-scoped, the interests reach the
/// network — the agreement's widening is the observable difference.
fn visibility_profile() -> Value {
    json!({
        "display_label": "visible everywhere",
        "purpose": "probe the per-reader filtering",
        "conversation_modes": ["architecture_deliberation"],
        "capabilities": [{
            "taxonomy_id": "code_review",
            "confidence": "self_asserted",
        }],
        "interests": ["parser trivia"],
        "languages": ["en"],
        "structured_output_formats": ["json"],
        "scopes": ["repo:example/parser"],
        "confidentiality_classes": ["internal"],
        "cost_latency_class": "cheap",
        "resource_ceilings": { "calls": 100 },
        "visibility": {
            "display_label": "public",
            "purpose": "network",
            "conversation_modes": "tenant",
            "capabilities": "tenant",
            "interests": "network",
            "languages": "network",
            "structured_output_formats": "tenant",
            "scopes": "tenant",
            "confidentiality_classes": "self_only",
            "availability": "tenant",
            "resolver_tool_capabilities": "tenant",
            "cost_latency_class": "tenant",
            "resource_ceilings": "self_only",
            "grants_by_reference": "self_only",
        },
    })
}

#[tokio::test]
async fn the_agreement_widens_the_visibility_and_never_transitively() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    // The three tenants (A, B, C) + the role whose profile A hosts.
    let (status, human_a) =
        enroll(&client, &base, json!({ "kind": "human", "name": "fed-a" })).await;
    assert_eq!(status, 200, "A enrolls: {human_a}");
    let a_admin = human_a["principal_id"].as_str().unwrap().to_string();
    let tenant_a = human_a["tenant_id"].as_str().unwrap().to_string();
    let (status, human_b) =
        enroll(&client, &base, json!({ "kind": "human", "name": "fed-b" })).await;
    assert_eq!(status, 200, "B enrolls: {human_b}");
    let b_admin = human_b["principal_id"].as_str().unwrap().to_string();
    let tenant_b = human_b["tenant_id"].as_str().unwrap().to_string();
    let (status, human_c) =
        enroll(&client, &base, json!({ "kind": "human", "name": "fed-c" })).await;
    assert_eq!(status, 200, "C enrolls: {human_c}");
    let c_admin = human_c["principal_id"].as_str().unwrap().to_string();
    let (status, role) = enroll(
        &client,
        &base,
        json!({ "kind": "role", "name": "fed-role", "tenant_id": tenant_a }),
    )
    .await;
    assert_eq!(status, 200, "the role enrolls: {role}");
    let role_id = role["principal_id"].as_str().unwrap().to_string();

    let (status, written) = put(
        &client,
        &base,
        &format!("/v1/profiles/{role_id}"),
        &role_id,
        &visibility_profile(),
    )
    .await;
    assert_eq!(status, 200, "the role writes the profile: {written}");

    async fn read(
        client: &reqwest::Client,
        base: &str,
        role_id: &str,
        principal: &str,
    ) -> (u16, Value) {
        get(client, base, &format!("/v1/profiles/{role_id}"), principal).await
    }

    // 1. WITHOUT an agreement: the cross-tenant reader sees the NETWORK
    //    view (the tenant fields ABSENT).
    let (status, network) = read(&client, &base, &role_id, &b_admin).await;
    assert_eq!(status, 200, "B reads: {network}");
    assert_eq!(network["visibility"], json!("network"));
    assert!(
        !network["profile"]
            .as_object()
            .unwrap()
            .contains_key("capabilities"),
        "the network view absents the tenant fields: {network}"
    );

    // 2. The ONE-SIDED proposal (A proposes toward B) widens NOTHING.
    let (status, proposed) = post(
        &client,
        &base,
        "/v1/federation-agreements",
        &a_admin,
        &json!({
            "tenant_id": tenant_a,
            "remote_tenant_id": tenant_b,
            "directory_visibility": true,
            "recruitment": false,
        }),
    )
    .await;
    assert_eq!(status, 200, "A proposes: {proposed}");
    let (_, still_network) = read(&client, &base, &role_id, &b_admin).await;
    assert_eq!(still_network["visibility"], json!("network"));
    assert_eq!(
        network["profile"], still_network["profile"],
        "the one-sided proposal widens nothing"
    );

    // 3. Accepting WITHOUT a proposal is the typed 409.
    let (status, refused) = post(
        &client,
        &base,
        "/v1/federation-agreements/accept",
        &b_admin,
        &json!({ "tenant_id": tenant_b, "remote_tenant_id": tenant_a }),
    )
    .await;
    assert_eq!(
        status, 409,
        "the accept without a proposal refuses: {refused}"
    );

    // 4. The BOTH-SIDES pairing: B proposes back + both accept → the
    //    EFFECTIVE agreement widens B's read to the TENANT view.
    let (status, proposed) = post(
        &client,
        &base,
        "/v1/federation-agreements",
        &b_admin,
        &json!({
            "tenant_id": tenant_b,
            "remote_tenant_id": tenant_a,
            "directory_visibility": true,
            "recruitment": false,
        }),
    )
    .await;
    assert_eq!(status, 200, "B proposes back: {proposed}");
    let (status, accepted) = post(
        &client,
        &base,
        "/v1/federation-agreements/accept",
        &a_admin,
        &json!({ "tenant_id": tenant_a, "remote_tenant_id": tenant_b }),
    )
    .await;
    assert_eq!(status, 200, "A accepts: {accepted}");
    let (status, accepted) = post(
        &client,
        &base,
        "/v1/federation-agreements/accept",
        &b_admin,
        &json!({ "tenant_id": tenant_b, "remote_tenant_id": tenant_a }),
    )
    .await;
    assert_eq!(status, 200, "B accepts: {accepted}");
    let (status, tenant_view) = read(&client, &base, &role_id, &b_admin).await;
    assert_eq!(status, 200, "B re-reads: {tenant_view}");
    assert_eq!(tenant_view["visibility"], json!("tenant"));
    assert!(
        tenant_view["profile"]
            .as_object()
            .unwrap()
            .contains_key("capabilities"),
        "the agreement widens to the tenant view: {tenant_view}"
    );

    // 5. The revocation falls back (B revokes its direction).
    let (status, revoked) = post(
        &client,
        &base,
        "/v1/federation-agreements/revoke",
        &b_admin,
        &json!({ "tenant_id": tenant_b, "remote_tenant_id": tenant_a }),
    )
    .await;
    assert_eq!(status, 200, "B revokes: {revoked}");
    let (_, fallen_back) = read(&client, &base, &role_id, &b_admin).await;
    assert_eq!(fallen_back["visibility"], json!("network"));

    // 6. The THIRD tenant never inherits (the transitive-default refusal).
    let (_, stranger_view) = read(&client, &base, &role_id, &c_admin).await;
    assert_eq!(
        stranger_view["visibility"],
        json!("network"),
        "no agreement, no widening — C sees the pseudonym only"
    );
}
