//! The adapter-allowlist ledger's verbs (`PHASE-8.4.4`, ADR-027): the
//! rung-1 registry the operator manages — the list, the allow (with the
//! recorded reason), the revoke (the NEXT ladder run refuses at rung 1,
//! never a silent untrust), all site-grant-gated. Measured:
//!   - the seeded dev three list;
//!   - the allow adds the fourth row (the idempotent no-op on the
//!     re-allow);
//!   - the revoke removes the row;
//!   - the non-admin principal is refused the ledger.
//!
//! Run with `scripts/run_pg_tests.sh` locally or the `pg-tests` CI job.
//! Without `DATABASE_URL` these skip, so `make check` stays green offline.

#[path = "support/mod.rs"]
mod pg_test_support;

#[path = "support/cleanup.rs"]
mod pg_cleanup;
#[path = "support/site.rs"]
mod site_fixture;

use std::net::SocketAddr;
use std::sync::OnceLock;

use reasonbraid_server::{api_router, PRINCIPAL_HEADER};
use serde_json::{json, Value};
use sqlx::PgPool;

static ALLOW_LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();

async fn guard() -> tokio::sync::MutexGuard<'static, ()> {
    ALLOW_LOCK
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
            "site_audit",
            "site_grants",
            "site_boundaries",
            "adapter_allowlist",
            "policy_outcomes",
            "policy_corrections",
            "policy_drift",
            "deployment_assignments",
            "deployment_targets",
            "policy_publications",
            "policy_projections",
            "policy_approvals",
            "policy_decisions",
            "policy_proposals",
            "policy_versions",
            "routing_resolutions",
            "evaluation_runs",
            "evaluation_corpora",
            "profile_versions",
            "agent_profiles",
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
    // The purge removes the migration's seeds; the dev three return (the
    // ledger's baseline the legs assert against).
    for (id, reason) in [
        (
            "fake",
            "the deterministic conformance oracle (the §11.6 contract)",
        ),
        ("codex", "the .4.2 real harness (the qualified CLI adapter)"),
        (
            "claude",
            "the .1.4.1 real harness (the qualified CLI adapter)",
        ),
    ] {
        sqlx::query(
            "INSERT INTO adapter_allowlist (adapter_id, added_by, reason) VALUES ($1, 'dev-seed', $2)",
        )
        .bind(id)
        .bind(reason)
        .execute(&pool)
        .await
        .expect("re-seed the dev three");
    }
    Some(pool)
}

struct TestServer {
    addr: SocketAddr,
    _handle: tokio::task::JoinHandle<()>,
}

impl Drop for TestServer {
    fn drop(&mut self) {
        self._handle.abort();
    }
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

async fn get(client: &reqwest::Client, base: &str, path: &str, principal: &str) -> (u16, Value) {
    let response = client
        .get(format!("{base}{path}"))
        .header(PRINCIPAL_HEADER, principal)
        .send()
        .await
        .expect("get request");
    let status = response.status().as_u16();
    let text = response.text().await.expect("get body");
    let body = serde_json::from_str(&text).unwrap_or_else(|_| json!({ "raw": text }));
    (status, body)
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
    let text = response.text().await.expect("post body");
    let body = serde_json::from_str(&text).unwrap_or_else(|_| json!({ "raw": text }));
    (status, body)
}

async fn listed_ids(client: &reqwest::Client, base: &str, principal: &str) -> Vec<String> {
    let (status, body) = get(client, base, "/v1/admin/adapters", principal).await;
    assert_eq!(status, 200, "the list: {body}");
    body["adapters"]
        .as_array()
        .expect("the adapter list")
        .iter()
        .map(|row| row["adapter_id"].as_str().unwrap().to_string())
        .collect()
}

/// The ledger's verbs: the seeded three, the allow (the idempotent no-op),
/// the revoke, and the admin gate.
#[tokio::test]
async fn the_allowlist_verbs_manage_the_rung_one_ledger() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let client = reqwest::Client::new();
    let base = server.base();

    let (status, human) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "allow-human" }),
    )
    .await;
    assert_eq!(status, 200, "the human enrolls: {human}");
    let human_id = human["principal_id"].as_str().unwrap().to_string();
    site_fixture::provision(&pool, &human_id, site_fixture::ALL).await;

    // 1. The seeded dev three list.
    let ids = listed_ids(&client, &base, &human_id).await;
    assert_eq!(ids, vec!["claude", "codex", "fake"], "the dev three list");

    // 2. The allow adds the fourth row; the re-allow is the idempotent
    //    no-op (the row stays, the reason stays the original's).
    let (status, allowed) = post(
        &client,
        &base,
        "/v1/admin/adapters",
        &human_id,
        &json!({ "adapter_id": "vendor-4", "reason": "the qualified third party" }),
    )
    .await;
    assert_eq!(status, 200, "the allow: {allowed}");
    let (status, _) = post(
        &client,
        &base,
        "/v1/admin/adapters",
        &human_id,
        &json!({ "adapter_id": "vendor-4", "reason": "a second reason (ignored)" }),
    )
    .await;
    assert_eq!(status, 200, "the re-allow is the no-op");
    let ids = listed_ids(&client, &base, &human_id).await;
    assert!(
        ids.contains(&"vendor-4".to_string()),
        "the fourth row lists"
    );

    // 3. The revoke removes the row — the NEXT ladder run refuses at
    //    rung 1.
    let (status, revoked) = post(
        &client,
        &base,
        "/v1/admin/adapters/vendor-4/revoke",
        &human_id,
        &json!({"reason": "explicit registry maintenance"}),
    )
    .await;
    assert_eq!(status, 200, "the revoke: {revoked}");
    let ids = listed_ids(&client, &base, &human_id).await;
    assert!(
        !ids.contains(&"vendor-4".to_string()),
        "the revoked row is gone"
    );
}

/// The ledger is site-grant-gated: the non-admin role is refused.
#[tokio::test]
async fn the_allowlist_refuses_the_non_admin() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let client = reqwest::Client::new();
    let base = server.base();

    let (status, human) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "allow-owner" }),
    )
    .await;
    assert_eq!(status, 200, "the human enrolls: {human}");
    let tenant = human["tenant_id"].as_str().unwrap().to_string();
    let (status, role) = enroll(
        &client,
        &base,
        json!({ "kind": "role", "name": "allow-role", "tenant_id": tenant, "actions": [] }),
    )
    .await;
    assert_eq!(status, 200, "the role enrolls: {role}");
    let role_id = role["principal_id"].as_str().unwrap().to_string();

    let (status, _) = get(&client, &base, "/v1/admin/adapters", &role_id).await;
    assert_eq!(status, 403, "the non-admin list refuses");
    let (status, _) = post(
        &client,
        &base,
        "/v1/admin/adapters",
        &role_id,
        &json!({ "adapter_id": "vendor-5", "reason": "the role tries" }),
    )
    .await;
    assert_eq!(status, 403, "the non-admin allow refuses");
}
