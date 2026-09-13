//! The regional-routing proof (`PHASE-8.5.2`, ADR-035 §20.10): the
//! regions are DECLARED (the declaration is the fail-closed seam) and
//! the cross-region delivery rides the EXPLICIT pair allowlist.
//! Measured:
//!   - the dev profile declares `dev-local`; the admin declares more;
//!   - the routing decision: the same-region routes; the undeclared
//!     region refuses with its OWN name; the unpaired cross-region
//!     refuses until the pair row lands; the unpair makes the NEXT
//!     delivery refuse;
//!   - the registry verbs are site-grant-gated.
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

static REGION_LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();

async fn guard() -> tokio::sync::MutexGuard<'static, ()> {
    REGION_LOCK
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
            "region_pairs",
            "site_regions",
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
    sqlx::query("INSERT INTO site_regions (region_id) VALUES ('dev-local')")
        .execute(&pool)
        .await
        .expect("re-seed the dev region");
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

/// The routing decision + the registry verbs: the declared seam, the
/// pair allowlist, the typed refusals, the unpair.
#[tokio::test]
async fn the_regional_routing_refuses_the_undeclared_and_the_unpaired() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let client = reqwest::Client::new();
    let base = server.base();

    let (status, human) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "region-human" }),
    )
    .await;
    assert_eq!(status, 200, "the human enrolls: {human}");
    let human_id = human["principal_id"].as_str().unwrap().to_string();
    site_fixture::provision(&pool, &human_id, site_fixture::ALL).await;

    // 1. The same-region delivery routes (the dev-local seed).
    reasonbraid_server::regions_internal::route(&pool, "dev-local", "dev-local")
        .await
        .expect("the same-region routes");

    // 2. The undeclared region refuses with its OWN name.
    let refusal = reasonbraid_server::regions_internal::route(&pool, "dev-local", "nowhere")
        .await
        .expect_err("the undeclared refuses");
    assert!(
        matches!(
            refusal,
            reasonbraid_server::regions_internal::RegionRefusal::UndeclaredRegion { .. }
        ),
        "the refusal names the undeclared: {refusal:?}"
    );

    // 3. The admin declares the second region + the pair; the cross-region
    //    delivery routes ONLY through the pair row.
    let (status, declared) = post(
        &client,
        &base,
        "/v1/admin/regions",
        &human_id,
        &json!({ "region": "eu-site", "reason": "declare the qualified site" }),
    )
    .await;
    assert_eq!(status, 200, "the declare: {declared}");
    let refusal = reasonbraid_server::regions_internal::route(&pool, "dev-local", "eu-site")
        .await
        .expect_err("the unpaired cross-region refuses");
    assert!(
        matches!(
            refusal,
            reasonbraid_server::regions_internal::RegionRefusal::CrossRegionRefused { .. }
        ),
        "the refusal names the cross-region: {refusal:?}"
    );
    let (status, paired) = post(
        &client,
        &base,
        "/v1/admin/regions/dev-local/pair/eu-site",
        &human_id,
        &json!({"reason": "explicit registry maintenance"}),
    )
    .await;
    assert_eq!(status, 200, "the pair: {paired}");
    reasonbraid_server::regions_internal::route(&pool, "dev-local", "eu-site")
        .await
        .expect("the paired cross-region routes");

    // 4. The unpair makes the NEXT cross-region delivery refuse.
    let (status, unpaired) = post(
        &client,
        &base,
        "/v1/admin/regions/dev-local/unpair/eu-site",
        &human_id,
        &json!({"reason": "explicit registry maintenance"}),
    )
    .await;
    assert_eq!(status, 200, "the unpair: {unpaired}");
    let refusal = reasonbraid_server::regions_internal::route(&pool, "dev-local", "eu-site")
        .await
        .expect_err("the unpaired refuses again");
    assert!(matches!(
        refusal,
        reasonbraid_server::regions_internal::RegionRefusal::CrossRegionRefused { .. }
    ));

    // 5. The registry lists the declared region.
    let response = client
        .get(format!("{base}/v1/admin/regions"))
        .header(PRINCIPAL_HEADER, &human_id)
        .send()
        .await
        .expect("the list");
    assert_eq!(response.status().as_u16(), 200, "the list");
    let body: Value = response.json().await.expect("the list json");
    let regions: Vec<String> = body["regions"]
        .as_array()
        .expect("the regions")
        .iter()
        .map(|r| r.as_str().unwrap().to_string())
        .collect();
    assert!(regions.contains(&"eu-site".to_string()), "the region lists");
}

/// The registry verbs are site-grant-gated.
#[tokio::test]
async fn the_region_registry_refuses_the_non_admin() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let client = reqwest::Client::new();
    let base = server.base();

    let (status, human) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "region-owner" }),
    )
    .await;
    assert_eq!(status, 200, "the human enrolls: {human}");
    let tenant = human["tenant_id"].as_str().unwrap().to_string();
    let (status, role) = enroll(
        &client,
        &base,
        json!({ "kind": "role", "name": "region-role", "tenant_id": tenant, "actions": [] }),
    )
    .await;
    assert_eq!(status, 200, "the role enrolls: {role}");
    let role_id = role["principal_id"].as_str().unwrap().to_string();

    let (status, _) = post(
        &client,
        &base,
        "/v1/admin/regions",
        &role_id,
        &json!({ "region": "role-site", "reason": "unprivileged attempt" }),
    )
    .await;
    assert_eq!(status, 403, "the non-admin declare refuses");
}
