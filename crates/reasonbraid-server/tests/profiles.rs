//! The directory profile write surface (`PHASE-3.1.2`, backlog 26): a role
//! declares its OWN profile (the §10.1 registration fields, typed at the
//! boundary), every write is a NEW content-addressed version (identical content
//! hashes identically; the old versions stay readable), the owner attests
//! capability claims with the provenance upgrade (audited), and the gates hold:
//! a stranger cannot write the profile, a dangling lineage reference fails
//! closed, and an unknown field is a typed rejection.
//!
//! Run with `scripts/run_pg_tests.sh` locally or the `pg-tests` CI job. Without
//! `DATABASE_URL` these skip, so `make check` stays green offline.

#[path = "support/mod.rs"]
mod pg_test_support;

use std::net::SocketAddr;
use std::sync::OnceLock;

use reasonbraid_server::{api_router, ca::ensure_server_ca, node_router, PRINCIPAL_HEADER};
use serde_json::{json, Value};
use sqlx::PgPool;

static PROFILE_LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();

async fn guard() -> tokio::sync::MutexGuard<'static, ()> {
    PROFILE_LOCK
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
    for table in [
        "profile_versions",
        "agent_profiles",
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
        "recruitment_panels",
        "recruitment_responses",
        "recruitment_offers",
        "recruitment_calls",
        "agent_roles",
        "human_principals",
        "resource_references",
        "quota_events",
        "usage_quotas",
        "federation_agreements",
        "cross_domain_receipts",
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
        Self::start_with_router(pool, api_router(pool.clone())).await
    }

    /// The `.5.3` seam: an explicit gate + a pre-populated broker.
    async fn start_gated(
        pool: &PgPool,
        enabled: bool,
        broker: std::sync::Arc<reasonbraid_server::broker::Broker>,
    ) -> Self {
        let router = reasonbraid_server::api_router_gated(pool.clone(), enabled, broker);
        Self::start_with_router(pool, router).await
    }

    async fn start_with_router(pool: &PgPool, router: axum::Router) -> Self {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind ephemeral loopback port");
        let addr = listener.local_addr().unwrap();
        let router = router.merge(node_router(
            pool.clone(),
            std::sync::Arc::new(ensure_server_ca(pool).await.expect("server CA")),
        ));
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

/// Enroll a node through the PUBLIC surface (`.1.2.1`; the dev wiring
/// collapses node==role — the node id IS the role wire id).
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
            "host_claim": "dir-host",
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
            "host_claim": "dir-host",
            "nonce": issued["nonce"],
            "key_secret": "dir-secret",
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

/// A minimal §10.1 profile (the typed boundary accepts the full shape; the
/// test drives the minimal one plus the fields the assertions need).
fn profile(claims: Value) -> Value {
    json!({
        "display_label": "directory probe",
        "purpose": "probe the profile surface",
        "conversation_modes": ["architecture_deliberation"],
        "capabilities": claims,
        "interests": ["parser trivia"],
        "languages": ["en"],
        "structured_output_formats": ["json"],
        "scopes": ["repo:example/parser"],
        "confidentiality_classes": ["internal"],
        "cost_latency_class": "cheap",
    })
}

/// THE write + history acceptance: the role declares its profile, the version
/// is content-addressed (identical content → identical hash; changed content →
/// a new version with a new hash), and the history stays readable.
#[tokio::test]
async fn a_role_writes_its_profile_and_the_history_is_content_addressed() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, human) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "prof-alice" }),
    )
    .await;
    assert_eq!(status, 200, "human enrolls: {human}");
    let tenant = human["tenant_id"].as_str().unwrap().to_string();
    let (status, role) = enroll(
        &client,
        &base,
        json!({ "kind": "role", "name": "prof-agent", "tenant_id": tenant }),
    )
    .await;
    assert_eq!(status, 200, "role enrolls: {role}");
    let role_id = role["principal_id"].as_str().unwrap().to_string();

    // Write v1.
    let body = profile(json!([{
        "taxonomy_id": "code_review",
        "confidence": "self_asserted",
    }]));
    let (status, written) = put(
        &client,
        &base,
        &format!("/v1/profiles/{role_id}"),
        &role_id,
        &body,
    )
    .await;
    assert_eq!(status, 200, "the role writes its profile: {written}");
    assert_eq!(written["version"], json!(1), "the first version");
    let hash1 = written["content_hash"].as_str().unwrap().to_string();
    assert_eq!(hash1.len(), 64, "the content hash is a sha256 hex");

    // Re-writing the IDENTICAL content is a new version with the SAME hash
    // (the content addressing, measured).
    let (status, rewritten) = put(
        &client,
        &base,
        &format!("/v1/profiles/{role_id}"),
        &role_id,
        &body,
    )
    .await;
    assert_eq!(status, 200, "the rewrite: {rewritten}");
    assert_eq!(rewritten["version"], json!(2), "the second version");
    assert_eq!(
        rewritten["content_hash"],
        json!(hash1),
        "identical content hashes identically"
    );

    // Changed content → version 3 with a DIFFERENT hash.
    let mut changed = body.clone();
    changed["interests"] = json!(["parser trivia", "schema drift"]);
    let (status, updated) = put(
        &client,
        &base,
        &format!("/v1/profiles/{role_id}"),
        &role_id,
        &changed,
    )
    .await;
    assert_eq!(status, 200, "the update: {updated}");
    assert_eq!(updated["version"], json!(3), "the third version");
    assert_ne!(
        updated["content_hash"],
        json!(hash1),
        "changed content hashes differently"
    );

    // The history is inspectable: the version list names every write, and the
    // OLD version's content stays readable.
    let (status, versions) = get(
        &client,
        &base,
        &format!("/v1/profiles/{role_id}/versions"),
        &role_id,
    )
    .await;
    assert_eq!(status, 200, "the version list: {versions}");
    assert_eq!(versions["versions"].as_array().unwrap().len(), 3);
    let (status, v1) = get(
        &client,
        &base,
        &format!("/v1/profiles/{role_id}/versions/1"),
        &role_id,
    )
    .await;
    assert_eq!(status, 200, "the old version reads: {v1}");
    assert_eq!(v1["profile"]["interests"], json!(["parser trivia"]));
    assert_eq!(v1["content_hash"], json!(hash1));
}

/// THE gates: a stranger cannot write the profile (only the role itself), a
/// dangling lineage reference fails closed, and an unknown field is a typed
/// rejection (the deny-unknown-fields stance).
#[tokio::test]
async fn the_profile_gates_hold_for_strangers_lineage_and_forged_fields() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, human) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "prof-bob" }),
    )
    .await;
    assert_eq!(status, 200, "human enrolls: {human}");
    let tenant = human["tenant_id"].as_str().unwrap().to_string();
    let (status, role) = enroll(
        &client,
        &base,
        json!({ "kind": "role", "name": "prof-agent-b", "tenant_id": tenant }),
    )
    .await;
    assert_eq!(status, 200, "role enrolls: {role}");
    let role_id = role["principal_id"].as_str().unwrap().to_string();
    let (status, stranger) = enroll(
        &client,
        &base,
        json!({ "kind": "role", "name": "prof-other", "tenant_id": tenant }),
    )
    .await;
    assert_eq!(status, 200, "the stranger enrolls: {stranger}");
    let stranger_id = stranger["principal_id"].as_str().unwrap().to_string();
    let human_id = human["principal_id"].as_str().unwrap().to_string();

    let body = profile(json!([]));

    // The owner (a human) and a sibling role are NOT the role: 403.
    let (status, refused) = put(
        &client,
        &base,
        &format!("/v1/profiles/{role_id}"),
        &human_id,
        &body,
    )
    .await;
    assert_eq!(
        status, 403,
        "the owner cannot self-declare for the role: {refused}"
    );
    let (status, refused) = put(
        &client,
        &base,
        &format!("/v1/profiles/{role_id}"),
        &stranger_id,
        &body,
    )
    .await;
    assert_eq!(status, 403, "a sibling role cannot: {refused}");

    // The read gate (`.1.3` classifies readers; `.1.2` gated self/owner only):
    // no profile was written, so the classified read finds NOTHING.
    let (status, _peeked) = get(
        &client,
        &base,
        &format!("/v1/profiles/{role_id}"),
        &stranger_id,
    )
    .await;
    assert_eq!(status, 404, "the classified read finds no profile");

    // A dangling lineage reference fails closed.
    let mut dangling = body.clone();
    dangling["incarnation_id"] = json!("inc_00000000-0000-7000-8000-000000000000");
    let (status, refused) = put(
        &client,
        &base,
        &format!("/v1/profiles/{role_id}"),
        &role_id,
        &dangling,
    )
    .await;
    assert_eq!(
        status, 400,
        "a dangling incarnation reference refuses: {refused}"
    );
    assert!(
        refused["message"]
            .as_str()
            .unwrap_or("")
            .contains("incarnation"),
        "the refusal names the lineage: {refused}"
    );

    // An unknown field is a typed rejection, never silently dropped.
    let mut forged = body.clone();
    forged["self_declared_authority"] = json!(["ThreadClose"]);
    let (status, rejected) = put(
        &client,
        &base,
        &format!("/v1/profiles/{role_id}"),
        &role_id,
        &forged,
    )
    .await;
    assert_eq!(status, 422, "an unknown field is rejected: {rejected}");
    assert!(
        rejected.to_string().contains("self_declared_authority"),
        "the rejection names the forged field: {rejected}"
    );

    // Nothing landed.
    let (status, _missing) =
        get(&client, &base, &format!("/v1/profiles/{role_id}"), &role_id).await;
    assert_eq!(
        status, 404,
        "no profile was written by the refused attempts"
    );
}

/// THE attestation path: the owner upgrades a self-asserted claim to
/// `owner_attested` with the evidence reference — a new version, audited.
#[tokio::test]
async fn the_owner_attests_a_capability_claim_with_provenance() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, human) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "prof-carol" }),
    )
    .await;
    assert_eq!(status, 200, "human enrolls: {human}");
    let tenant = human["tenant_id"].as_str().unwrap().to_string();
    let human_id = human["principal_id"].as_str().unwrap().to_string();
    let (status, role) = enroll(
        &client,
        &base,
        json!({ "kind": "role", "name": "prof-agent-c", "tenant_id": tenant }),
    )
    .await;
    assert_eq!(status, 200, "role enrolls: {role}");
    let role_id = role["principal_id"].as_str().unwrap().to_string();

    let (status, written) = put(
        &client,
        &base,
        &format!("/v1/profiles/{role_id}"),
        &role_id,
        &profile(json!([{ "taxonomy_id": "code_review", "confidence": "self_asserted" }])),
    )
    .await;
    assert_eq!(status, 200, "the role declares: {written}");

    // The owner attests: the claim's provenance upgrades with the evidence
    // (the tenant is DERIVED from the role's row — the body is typed).
    let (status, attested) = post(
        &client,
        &base,
        &format!("/v1/profiles/{role_id}/attest"),
        &human_id,
        &json!({
            "taxonomy_id": "code_review",
            "evidence_ref": "evt_demo/20260907",
        }),
    )
    .await;
    assert_eq!(status, 200, "the attestation: {attested}");
    assert_eq!(
        attested["version"],
        json!(2),
        "a new version for the attestation"
    );
    let claims = attested["profile"]["capabilities"].as_array().unwrap();
    assert_eq!(claims[0]["confidence"], json!("owner_attested"));
    assert_eq!(claims[0]["evidence_ref"], json!("evt_demo/20260907"));

    // The attestation is AUDITED (the tenant-admin authorization record).
    let (n_admin,): (i64,) = sqlx::query_as(
        "SELECT count(*) FROM authorization_records WHERE tenant_id = $1 AND decision = 'allowed' AND action = 'tenant_admin'",
    )
    .bind(&tenant)
    .fetch_one(&pool)
    .await
    .expect("count the attestation audit");
    assert!(n_admin >= 1, "the attestation is audited");

    // The stranger cannot attest.
    let (status, refused) = post(
        &client,
        &base,
        &format!("/v1/profiles/{role_id}/attest"),
        &role_id,
        &json!({ "taxonomy_id": "code_review", "evidence_ref": "x" }),
    )
    .await;
    assert_eq!(
        status, 403,
        "the role cannot attest its own claims: {refused}"
    );
}

/// A profile with an EXPLICIT per-field visibility policy: the display label
/// travels everywhere, the capabilities stay tenant-scoped, the interests
/// reach the network, and the confidential/ceiling fields stay self-only.
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

/// THE `.1.3` acceptance, measured: the SAME profile read by the role, the
/// owner, a tenant sibling, and a stranger yields exactly the allowed fields
/// each — a hidden field is ABSENT, never nulled, and the response names the
/// applied class.
#[tokio::test]
async fn the_same_profile_read_by_four_readers_yields_exactly_the_allowed_fields() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, owner) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "vis-owner" }),
    )
    .await;
    assert_eq!(status, 200, "the owner enrolls: {owner}");
    let tenant_a = owner["tenant_id"].as_str().unwrap().to_string();
    let owner_id = owner["principal_id"].as_str().unwrap().to_string();
    let (status, role) = enroll(
        &client,
        &base,
        json!({ "kind": "role", "name": "vis-agent", "tenant_id": tenant_a }),
    )
    .await;
    assert_eq!(status, 200, "the role enrolls: {role}");
    let role_id = role["principal_id"].as_str().unwrap().to_string();
    let (status, sibling) = enroll(
        &client,
        &base,
        json!({ "kind": "role", "name": "vis-sibling", "tenant_id": tenant_a }),
    )
    .await;
    assert_eq!(status, 200, "the sibling enrolls: {sibling}");
    let sibling_id = sibling["principal_id"].as_str().unwrap().to_string();
    let (status, stranger) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "vis-stranger" }),
    )
    .await;
    assert_eq!(status, 200, "the stranger enrolls: {stranger}");
    let stranger_id = stranger["principal_id"].as_str().unwrap().to_string();

    let (status, written) = put(
        &client,
        &base,
        &format!("/v1/profiles/{role_id}"),
        &role_id,
        &visibility_profile(),
    )
    .await;
    assert_eq!(status, 200, "the role writes the policy: {written}");

    let read = |principal: String| {
        let client = client.clone();
        let base = base.clone();
        let role_id = role_id.clone();
        async move {
            let (status, body) = get(
                &client,
                &base,
                &format!("/v1/profiles/{role_id}"),
                &principal,
            )
            .await;
            assert_eq!(status, 200, "the read succeeds: {body}");
            body
        }
    };

    // The role itself: FULL, every field.
    let own = read(role_id.clone()).await;
    assert_eq!(own["visibility"], json!("full"));
    let own_fields = own["profile"].as_object().unwrap();
    assert!(
        own_fields.contains_key("confidentiality_classes"),
        "the self read is full"
    );
    assert!(
        own_fields.contains_key("resource_ceilings"),
        "the self read is full"
    );

    // The owner (tenant_admin): FULL (the audited admin read).
    let owners = read(owner_id.clone()).await;
    assert_eq!(owners["visibility"], json!("full"));
    assert!(owners["profile"]
        .as_object()
        .unwrap()
        .contains_key("confidentiality_classes"));

    // The tenant sibling: TENANT view — the network+public+tenant fields
    // exist; the self-only fields are ABSENT (not nulled).
    let tenant_view = read(sibling_id.clone()).await;
    assert_eq!(tenant_view["visibility"], json!("tenant"));
    let fields = tenant_view["profile"].as_object().unwrap();
    for key in [
        "display_label",
        "purpose",
        "conversation_modes",
        "capabilities",
        "interests",
        "languages",
        "structured_output_formats",
        "scopes",
        "cost_latency_class",
    ] {
        assert!(
            fields.contains_key(key),
            "the tenant view carries `{key}`: {fields:?}"
        );
    }
    for hidden in [
        "confidentiality_classes",
        "resource_ceilings",
        "grants_by_reference",
    ] {
        assert!(
            !fields.contains_key(hidden),
            "the tenant view ABSENTS `{hidden}` (never nulls it): {fields:?}"
        );
    }

    // The stranger (another tenant): NETWORK view — only public+network fields.
    let network_view = read(stranger_id.clone()).await;
    assert_eq!(network_view["visibility"], json!("network"));
    let fields = network_view["profile"].as_object().unwrap();
    for key in ["display_label", "purpose", "interests", "languages"] {
        assert!(
            fields.contains_key(key),
            "the network view carries `{key}`: {fields:?}"
        );
    }
    for hidden in [
        "capabilities",
        "scopes",
        "conversation_modes",
        "cost_latency_class",
        "confidentiality_classes",
        "resource_ceilings",
    ] {
        assert!(
            !fields.contains_key(hidden),
            "the network view ABSENTS `{hidden}`: {fields:?}"
        );
    }

    // The provenance rides the visible claims (never flattened).
    let claim = tenant_view["profile"]["capabilities"][0].clone();
    assert_eq!(claim["confidence"], json!("self_asserted"));
}

/// The history stays FULL-only: a tenant sibling or a stranger cannot read
/// the version history (the past versions may carry fields later reclassified).
#[tokio::test]
async fn the_version_history_stays_full_only() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, owner) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "vis-owner-2" }),
    )
    .await;
    assert_eq!(status, 200, "the owner enrolls: {owner}");
    let tenant = owner["tenant_id"].as_str().unwrap().to_string();
    let owner_id = owner["principal_id"].as_str().unwrap().to_string();
    let (status, role) = enroll(
        &client,
        &base,
        json!({ "kind": "role", "name": "vis-agent-2", "tenant_id": tenant }),
    )
    .await;
    assert_eq!(status, 200, "the role enrolls: {role}");
    let role_id = role["principal_id"].as_str().unwrap().to_string();
    let (status, stranger) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "vis-stranger-2" }),
    )
    .await;
    assert_eq!(status, 200, "the stranger enrolls: {stranger}");
    let stranger_id = stranger["principal_id"].as_str().unwrap().to_string();

    let (status, _) = put(
        &client,
        &base,
        &format!("/v1/profiles/{role_id}"),
        &role_id,
        &visibility_profile(),
    )
    .await;
    assert_eq!(status, 200, "the role writes");

    // The stranger reads the NETWORK view of the current profile but NOT the
    // history.
    let (status, versions) = get(
        &client,
        &base,
        &format!("/v1/profiles/{role_id}/versions"),
        &stranger_id,
    )
    .await;
    assert_eq!(status, 403, "the history is full-only: {versions}");
    let (status, _) = get(
        &client,
        &base,
        &format!("/v1/profiles/{role_id}/versions/1"),
        &stranger_id,
    )
    .await;
    assert_eq!(status, 403, "a past version is full-only");

    // The owner reads the history.
    let (status, versions) = get(
        &client,
        &base,
        &format!("/v1/profiles/{role_id}/versions"),
        &owner_id,
    )
    .await;
    assert_eq!(status, 200, "the owner reads the history: {versions}");
}

/// THE `.3.2.3` acceptance, measured: the SAME directory read by the owner, a
/// tenant member, and a stranger yields the allowed shapes — the owner sees
/// the FULL own-tenant fields, the member the TENANT-filtered fields, every
/// enrolled principal the network pseudonyms, and a zero-visibility profile
/// contributes nothing at all (not even a count).
#[tokio::test]
async fn the_directory_reads_yield_the_allowed_shapes_per_reader() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    // Tenant A: the owner, the role+node, and a plain member (no node).
    let (status, owner_a) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "dir-owner-a" }),
    )
    .await;
    assert_eq!(status, 200, "owner A enrolls: {owner_a}");
    let tenant_a = owner_a["tenant_id"].as_str().unwrap().to_string();
    let owner_a_id = owner_a["principal_id"].as_str().unwrap().to_string();
    let (status, role_a) = enroll(
        &client,
        &base,
        json!({ "kind": "role", "name": "dir-agent-a", "tenant_id": tenant_a }),
    )
    .await;
    assert_eq!(status, 200, "role A enrolls: {role_a}");
    let role_a_id = role_a["principal_id"].as_str().unwrap().to_string();
    enroll_node(&client, &base, &owner_a_id, &tenant_a, &role_a_id).await;
    let (status, _) = put(
        &client,
        &base,
        &format!("/v1/profiles/{role_a_id}"),
        &role_a_id,
        &visibility_profile(),
    )
    .await;
    assert_eq!(status, 200, "role A writes its profile");
    let (status, member_a) = enroll(
        &client,
        &base,
        json!({ "kind": "role", "name": "dir-member-a", "tenant_id": tenant_a }),
    )
    .await;
    assert_eq!(status, 200, "the member enrolls: {member_a}");
    let member_a_id = member_a["principal_id"].as_str().unwrap().to_string();

    // Tenant B: the owner + a role+node with the same profile shape.
    let (status, owner_b) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "dir-owner-b" }),
    )
    .await;
    assert_eq!(status, 200, "owner B enrolls: {owner_b}");
    let tenant_b = owner_b["tenant_id"].as_str().unwrap().to_string();
    let owner_b_id = owner_b["principal_id"].as_str().unwrap().to_string();
    let (status, role_b) = enroll(
        &client,
        &base,
        json!({ "kind": "role", "name": "dir-agent-b", "tenant_id": tenant_b }),
    )
    .await;
    assert_eq!(status, 200, "role B enrolls: {role_b}");
    let role_b_id = role_b["principal_id"].as_str().unwrap().to_string();
    enroll_node(&client, &base, &owner_b_id, &tenant_b, &role_b_id).await;
    let (status, _) = put(
        &client,
        &base,
        &format!("/v1/profiles/{role_b_id}"),
        &role_b_id,
        &visibility_profile(),
    )
    .await;
    assert_eq!(status, 200, "role B writes its profile");

    // Tenant C: a role+node whose profile exposes NOTHING to the network
    // (every field self_only or tenant) — the zero-visibility rule.
    let (status, owner_c) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "dir-owner-c" }),
    )
    .await;
    assert_eq!(status, 200, "owner C enrolls: {owner_c}");
    let tenant_c = owner_c["tenant_id"].as_str().unwrap().to_string();
    let owner_c_id = owner_c["principal_id"].as_str().unwrap().to_string();
    let (status, role_c) = enroll(
        &client,
        &base,
        json!({ "kind": "role", "name": "dir-agent-c", "tenant_id": tenant_c }),
    )
    .await;
    assert_eq!(status, 200, "role C enrolls: {role_c}");
    let role_c_id = role_c["principal_id"].as_str().unwrap().to_string();
    enroll_node(&client, &base, &owner_c_id, &tenant_c, &role_c_id).await;
    let mut hidden_profile = visibility_profile();
    hidden_profile["visibility"] = json!({
        "display_label": "self_only",
        "purpose": "self_only",
        "conversation_modes": "self_only",
        "capabilities": "self_only",
        "interests": "self_only",
        "languages": "self_only",
        "structured_output_formats": "self_only",
        "scopes": "self_only",
        "confidentiality_classes": "self_only",
        "availability": "self_only",
        "resolver_tool_capabilities": "self_only",
        "cost_latency_class": "self_only",
        "resource_ceilings": "self_only",
        "grants_by_reference": "self_only",
    });
    let (status, _) = put(
        &client,
        &base,
        &format!("/v1/profiles/{role_c_id}"),
        &role_c_id,
        &hidden_profile,
    )
    .await;
    assert_eq!(status, 200, "role C writes the zero-visibility profile");

    let read_directory = |principal: String| {
        let client = client.clone();
        let base = base.clone();
        async move {
            let (status, body) = get(&client, &base, "/v1/directory/presence", &principal).await;
            assert_eq!(status, 200, "the directory read succeeds: {body}");
            body
        }
    };

    // THE OWNER: the FULL own-tenant fields (the self-only resource_ceilings
    // ride the owner's view) + the network pseudonyms.
    let owners = read_directory(owner_a_id.clone()).await;
    assert_eq!(owners["own_tenant"]["tenant_id"], json!(tenant_a));
    let own_nodes = owners["own_tenant"]["nodes"].as_array().unwrap();
    assert_eq!(own_nodes.len(), 1, "the owner's own view: {owners}");
    let node_a = &own_nodes[0];
    assert_eq!(node_a["node_id"], json!(role_a_id));
    assert!(
        node_a["profile"]
            .as_object()
            .unwrap()
            .contains_key("resource_ceilings"),
        "the owner's own-tenant view is FULL: {node_a}"
    );
    let network = owners["network"]["nodes"].as_array().unwrap();
    let network_ids: Vec<&str> = network
        .iter()
        .map(|n| n["node_id"].as_str().unwrap())
        .collect();
    assert!(
        network_ids.contains(&role_b_id.as_str()),
        "the network carries B: {network_ids:?}"
    );
    assert!(
        !network_ids.contains(&role_c_id.as_str()),
        "the zero-visibility profile contributes nothing: {network_ids:?}"
    );
    let node_b = network
        .iter()
        .find(|n| n["node_id"] == json!(role_b_id))
        .unwrap();
    let b_fields = node_b["profile"].as_object().unwrap();
    assert!(
        b_fields.contains_key("display_label"),
        "the network view carries B's public label"
    );
    assert!(
        !b_fields.contains_key("capabilities"),
        "B's tenant-scoped capabilities are absent from the network view: {b_fields:?}"
    );

    // THE TENANT MEMBER: the own-tenant view is TENANT-filtered (the self-only
    // fields absent), the network view identical.
    let members = read_directory(member_a_id.clone()).await;
    let own_nodes = members["own_tenant"]["nodes"].as_array().unwrap();
    assert_eq!(own_nodes.len(), 1, "the member's own view: {members}");
    let member_fields = own_nodes[0]["profile"].as_object().unwrap();
    assert!(
        member_fields.contains_key("capabilities"),
        "the member sees the tenant-scoped capabilities: {member_fields:?}"
    );
    assert!(
        !member_fields.contains_key("resource_ceilings"),
        "the member's view ABSENTS the self-only fields: {member_fields:?}"
    );

    // THE STRANGER: their own tenant's view + A's network pseudonyms.
    let strangers = read_directory(owner_b_id.clone()).await;
    assert_eq!(strangers["own_tenant"]["tenant_id"], json!(tenant_b));
    let stranger_network = strangers["network"]["nodes"].as_array().unwrap();
    let stranger_ids: Vec<&str> = stranger_network
        .iter()
        .map(|n| n["node_id"].as_str().unwrap())
        .collect();
    assert!(
        stranger_ids.contains(&role_a_id.as_str()),
        "the stranger sees A's network pseudonym: {stranger_ids:?}"
    );
    assert!(
        !stranger_ids.contains(&role_c_id.as_str()),
        "the zero-visibility profile is invisible to the stranger too: {stranger_ids:?}"
    );
}

/// THE `.3.3.3` acceptance: the match query resolves server-side — the
/// tenant-scope expression returns the ranked eligible candidates with the
/// reader-visible fields + the reasons; the zero-visibility profile never
/// appears; and the scope clamp refuses a wider-than-classified expression.
/// The provenance gate rides along: a role's own write may declare only
/// self_asserted claims (the owner attests the upgrade).
#[tokio::test]
async fn the_match_query_resolves_the_expression_and_clamps_the_scope() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    // Tenant A: the owner + two role nodes.
    let (status, owner_a) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "match-owner-a" }),
    )
    .await;
    assert_eq!(status, 200, "owner A enrolls: {owner_a}");
    let tenant_a = owner_a["tenant_id"].as_str().unwrap().to_string();
    let owner_a_id = owner_a["principal_id"].as_str().unwrap().to_string();
    let (status, role_a) = enroll(
        &client,
        &base,
        json!({ "kind": "role", "name": "match-agent-a", "tenant_id": tenant_a }),
    )
    .await;
    assert_eq!(status, 200, "role A enrolls: {role_a}");
    let role_a_id = role_a["principal_id"].as_str().unwrap().to_string();
    enroll_node(&client, &base, &owner_a_id, &tenant_a, &role_a_id).await;
    let mut profile_a = visibility_profile();
    profile_a["scopes"] = json!(["repo:example/parser"]);
    let (status, _) = put(
        &client,
        &base,
        &format!("/v1/profiles/{role_a_id}"),
        &role_a_id,
        &profile_a,
    )
    .await;
    assert_eq!(status, 200, "role A writes");
    // The provenance gate: the role's OWN write cannot self-declare the
    // upgrade — the owner attests it (the audited path).
    let mut forged_upgrade = visibility_profile();
    forged_upgrade["capabilities"][0]["confidence"] = json!("certified");
    let (status, refused) = put(
        &client,
        &base,
        &format!("/v1/profiles/{role_a_id}"),
        &role_a_id,
        &forged_upgrade,
    )
    .await;
    assert_eq!(
        status, 400,
        "a self-declared provenance upgrade is refused: {refused}"
    );
    assert!(
        refused["message"]
            .as_str()
            .unwrap_or("")
            .contains("self_asserted"),
        "the refusal names the gate: {refused}"
    );
    let (status, _) = post(
        &client,
        &base,
        &format!("/v1/profiles/{role_a_id}/attest"),
        &owner_a_id,
        &json!({
            "taxonomy_id": "code_review",
            "evidence_ref": "evt_match/20260907",
        }),
    )
    .await;
    assert_eq!(status, 200, "the owner attests A's claim");

    // Role B: the same capability + interest, no domain scope.
    let (status, role_b) = enroll(
        &client,
        &base,
        json!({ "kind": "role", "name": "match-agent-b", "tenant_id": tenant_a }),
    )
    .await;
    assert_eq!(status, 200, "role B enrolls: {role_b}");
    let role_b_id = role_b["principal_id"].as_str().unwrap().to_string();
    enroll_node(&client, &base, &owner_a_id, &tenant_a, &role_b_id).await;
    let mut profile_b = visibility_profile();
    profile_b["scopes"] = json!([]);
    let (status, _) = put(
        &client,
        &base,
        &format!("/v1/profiles/{role_b_id}"),
        &role_b_id,
        &profile_b,
    )
    .await;
    assert_eq!(status, 200, "role B writes");
    let (status, _) = post(
        &client,
        &base,
        &format!("/v1/profiles/{role_b_id}/attest"),
        &owner_a_id,
        &json!({
            "taxonomy_id": "code_review",
            "evidence_ref": "evt_match/20260907",
        }),
    )
    .await;
    assert_eq!(status, 200, "the owner attests B's claim");

    // Tenant B: the zero-visibility profile.
    let (status, owner_b) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "match-owner-b" }),
    )
    .await;
    assert_eq!(status, 200, "owner B enrolls: {owner_b}");
    let tenant_b = owner_b["tenant_id"].as_str().unwrap().to_string();
    let owner_b_id = owner_b["principal_id"].as_str().unwrap().to_string();
    let (status, role_c) = enroll(
        &client,
        &base,
        json!({ "kind": "role", "name": "match-agent-c", "tenant_id": tenant_b }),
    )
    .await;
    assert_eq!(status, 200, "role C enrolls: {role_c}");
    let role_c_id = role_c["principal_id"].as_str().unwrap().to_string();
    enroll_node(&client, &base, &owner_b_id, &tenant_b, &role_c_id).await;
    let mut hidden_profile = visibility_profile();
    hidden_profile["visibility"] = json!({
        "display_label": "self_only",
        "purpose": "self_only",
        "conversation_modes": "self_only",
        "capabilities": "self_only",
        "interests": "self_only",
        "languages": "self_only",
        "structured_output_formats": "self_only",
        "scopes": "self_only",
        "confidentiality_classes": "self_only",
        "availability": "self_only",
        "resolver_tool_capabilities": "self_only",
        "cost_latency_class": "self_only",
        "resource_ceilings": "self_only",
        "grants_by_reference": "self_only",
    });
    let (status, _) = put(
        &client,
        &base,
        &format!("/v1/profiles/{role_c_id}"),
        &role_c_id,
        &hidden_profile,
    )
    .await;
    assert_eq!(status, 200, "role C writes the zero-visibility profile");

    // THE match: the owner's tenant-scope expression.
    let expression = json!({
        "scope": "tenant",
        "capabilities": [{ "taxonomy_id": "code_review", "min_confidence": "owner_attested" }],
        "interests": ["parser trivia"],
        "domains": ["repo:example/parser"],
        // The drill's nodes hold no leases: the initiator WIDENS the presence
        // gate explicitly (the default is available-only).
        "presence_states": ["available", "offline"],
    });
    let response = client
        .post(format!("{base}/v1/directory/match"))
        .header(PRINCIPAL_HEADER, &owner_a_id)
        .json(&json!({ "expression": expression }))
        .send()
        .await
        .expect("match request");
    assert_eq!(response.status().as_u16(), 200, "the match resolves");
    let matched: Value = response.json().await.unwrap();
    let candidates = matched["candidates"].as_array().unwrap();
    assert_eq!(
        candidates.len(),
        2,
        "only the eligible candidates: {matched}"
    );
    assert_eq!(
        candidates[0]["role_id"],
        json!(role_a_id),
        "the full match ranks first"
    );
    assert_eq!(candidates[1]["role_id"], json!(role_b_id));
    assert!(
        candidates[0]["total"].as_f64().unwrap() > candidates[1]["total"].as_f64().unwrap(),
        "the affinity separates the ranking: {matched}"
    );
    // The candidate profiles carry only the reader-class-visible fields.
    let a_fields = candidates[0]["profile"].as_object().unwrap();
    assert!(
        a_fields.contains_key("capabilities"),
        "the owner sees the capabilities"
    );
    // The zero-visibility profile never appears (no candidate, no count).
    for candidate in candidates {
        assert_ne!(candidate["role_id"], json!(role_c_id));
    }
    // The stage-1 reasons ride every candidate.
    assert!(
        candidates[0]["stage1_reasons"]
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r.as_str().unwrap().contains("code_review")),
        "the reasons ride: {matched}"
    );

    // THE scope clamp: a NON-owner (a plain member of B) demanding the FULL
    // scope — refused with the typed reason (an owner's own class IS full).
    let (status, stranger) = enroll(
        &client,
        &base,
        json!({ "kind": "role", "name": "match-stranger", "tenant_id": tenant_b }),
    )
    .await;
    assert_eq!(status, 200, "the stranger enrolls: {stranger}");
    let stranger_id = stranger["principal_id"].as_str().unwrap().to_string();
    let forged = json!({
        "scope": "full",
        "capabilities": [],
    });
    let response = client
        .post(format!("{base}/v1/directory/match"))
        .header(PRINCIPAL_HEADER, &stranger_id)
        .json(&json!({ "expression": forged }))
        .send()
        .await
        .expect("match request");
    assert_eq!(response.status().as_u16(), 403, "the scope clamp refuses");
    let refused: Value = response.json().await.unwrap();
    assert!(
        refused["message"].as_str().unwrap_or("").contains("scope"),
        "the refusal names the clamp: {refused}"
    );
}

/// THE `.3.4.2` acceptance: the call rides the thread's invitation machinery —
/// the human opens a call with the eligibility expression, the eligible role
/// joins, the ineligible role's join refuses with the stage-1 reasons, the
/// decline carries its reason, and the close snapshots the ranked panel with
/// the selection explanation.
#[tokio::test]
async fn the_call_artifact_rides_the_invitation_machinery() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, human) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "call-human" }),
    )
    .await;
    assert_eq!(status, 200, "the human enrolls: {human}");
    let tenant = human["tenant_id"].as_str().unwrap().to_string();
    let human_id = human["principal_id"].as_str().unwrap().to_string();

    // A thread the call rides (the human holds the invite authority).
    let envelope = |operation: &str, key: &str, body: Value| reasonbraid_core::CommandEnvelope {
        protocol_version: reasonbraid_core::PROTOCOL_VERSION.to_string(),
        operation: operation.to_string(),
        request_id: reasonbraid_core::RequestId::new(),
        idempotency_key: key.to_string(),
        expected_aggregate_version: None,
        body,
        authority_context: None,
        client_context: Default::default(),
    };
    let command = |path: String, principal: String, env: reasonbraid_core::CommandEnvelope| {
        let client = client.clone();
        let base = base.clone();
        async move {
            let response = client
                .post(format!("{base}{path}"))
                .header(PRINCIPAL_HEADER, principal)
                .json(&env)
                .send()
                .await
                .expect("command");
            let status = response.status().as_u16();
            let body: Value = response.json().await.expect("command json");
            (status, body)
        }
    };
    let (status, created) = command(
        "/v1/threads".to_string(),
        human_id.clone(),
        envelope(
            "thread.create",
            "key-call-t1",
            json!({ "tenant_id": tenant, "subject": "open call probe", "objective": "probe" }),
        ),
    )
    .await;
    assert_eq!(status, 200, "the thread creates: {created}");
    let thread_id = created["thread_id"].as_str().unwrap().to_string();

    // Role A (eligible) + Role B (no profile → ineligible).
    let (status, role_a) = enroll(
        &client,
        &base,
        json!({ "kind": "role", "name": "call-agent-a", "tenant_id": tenant }),
    )
    .await;
    assert_eq!(status, 200, "role A enrolls: {role_a}");
    let role_a_id = role_a["principal_id"].as_str().unwrap().to_string();
    enroll_node(&client, &base, &human_id, &tenant, &role_a_id).await;
    let (status, _) = put(
        &client,
        &base,
        &format!("/v1/profiles/{role_a_id}"),
        &role_a_id,
        &visibility_profile(),
    )
    .await;
    assert_eq!(status, 200, "role A writes its profile");
    let (status, _) = post(
        &client,
        &base,
        &format!("/v1/profiles/{role_a_id}/attest"),
        &human_id,
        &json!({ "taxonomy_id": "code_review", "evidence_ref": "evt_call/20260907" }),
    )
    .await;
    assert_eq!(status, 200, "the owner attests A");
    let (status, role_b) = enroll(
        &client,
        &base,
        json!({ "kind": "role", "name": "call-agent-b", "tenant_id": tenant }),
    )
    .await;
    assert_eq!(status, 200, "role B enrolls: {role_b}");
    let role_b_id = role_b["principal_id"].as_str().unwrap().to_string();
    enroll_node(&client, &base, &human_id, &tenant, &role_b_id).await;

    // The call: the human's invite authority gates the open.
    let deadline = (chrono::Utc::now() + chrono::Duration::hours(1)).to_rfc3339();
    let expiry = (chrono::Utc::now() + chrono::Duration::hours(2)).to_rfc3339();
    let response = client
        .post(format!("{base}/v1/calls"))
        .header(PRINCIPAL_HEADER, &human_id)
        .json(&json!({
            "tenant_id": tenant,
            "thread_id": thread_id,
            "expression": {
                "scope": "tenant",
                "capabilities": [{ "taxonomy_id": "code_review", "min_confidence": "owner_attested" }],
                "presence_states": ["available", "offline"],
            },
            "min_participants": 1,
            "max_participants": 2,
            "join_deadline": deadline,
            "expires_at": expiry,
        }))
        .send()
        .await
        .expect("open request");
    assert_eq!(response.status().as_u16(), 200, "the call opens");
    let opened: Value = response.json().await.unwrap();
    let call_id = opened["call_id"].as_str().unwrap().to_string();

    // Role A joins (the eligible participation claim).
    let response = client
        .post(format!("{base}/v1/calls/{call_id}/respond"))
        .header(PRINCIPAL_HEADER, &role_a_id)
        .json(&json!({ "kind": "join" }))
        .send()
        .await
        .expect("respond request");
    assert_eq!(response.status().as_u16(), 200, "the eligible role joins");
    let joined: Value = response.json().await.unwrap();
    assert_eq!(joined["response"], json!("join"));

    // Role B's JOIN refuses with the stage-1 reasons (the eligibility gate).
    let response = client
        .post(format!("{base}/v1/calls/{call_id}/respond"))
        .header(PRINCIPAL_HEADER, &role_b_id)
        .json(&json!({ "kind": "join" }))
        .send()
        .await
        .expect("respond request");
    assert_eq!(
        response.status().as_u16(),
        403,
        "the ineligible join refuses"
    );
    let refused: Value = response.json().await.unwrap();
    assert!(
        refused["message"]
            .as_str()
            .unwrap_or("")
            .contains("ineligible"),
        "the refusal names the gate: {refused}"
    );

    // Role B's DECLINE carries its reason (the informational responses are
    // NOT refused — a decline is exactly the ineligible declaring why).
    let response = client
        .post(format!("{base}/v1/calls/{call_id}/respond"))
        .header(PRINCIPAL_HEADER, &role_b_id)
        .json(&json!({ "kind": "decline", "reason": "no code-review capability" }))
        .send()
        .await
        .expect("respond request");
    assert_eq!(response.status().as_u16(), 200, "the decline rides");
    let declined: Value = response.json().await.unwrap();
    assert_eq!(declined["response"], json!("decline"));

    // The close: the panel snapshots the joiners + the explanation.
    let response = client
        .post(format!("{base}/v1/calls/{call_id}/close"))
        .header(PRINCIPAL_HEADER, &human_id)
        .send()
        .await
        .expect("close request");
    assert_eq!(response.status().as_u16(), 200, "the close snapshots");
    let closed: Value = response.json().await.unwrap();
    assert_eq!(
        closed["panel"],
        json!([role_a_id]),
        "the panel is the joiner"
    );
    assert_eq!(closed["status"], json!("closed"));

    // The inspection: the responses + the panel + the explanation.
    let (status, inspected) = get(&client, &base, &format!("/v1/calls/{call_id}"), &human_id).await;
    assert_eq!(status, 200, "the inspection: {inspected}");
    assert_eq!(inspected["responses"].as_array().unwrap().len(), 2);
    let explanation = inspected["explanation"].as_object().unwrap();
    let per_panelist = explanation["per_panelist"].as_array().unwrap();
    assert!(
        per_panelist[0]["stage1_reasons"]
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r.as_str().unwrap().contains("code_review")),
        "the selection explanation rides: {inspected}"
    );
    assert!(
        explanation.contains_key("dependence_indicators"),
        "the snapshot carries the dependence indicators: {inspected}"
    );
}

/// THE `.3.4.3` acceptance at the dev scale: the open-call fan-out caps hold
/// (the initiator's 5th open is the typed 429) and an expired call refuses
/// the responses — the storm controls' buildable core, measured.
#[tokio::test]
async fn the_open_call_storm_controls_hold_at_the_dev_scale() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, human) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "storm-human" }),
    )
    .await;
    assert_eq!(status, 200, "the human enrolls: {human}");
    let tenant = human["tenant_id"].as_str().unwrap().to_string();
    let human_id = human["principal_id"].as_str().unwrap().to_string();
    let (status, role) = enroll(
        &client,
        &base,
        json!({ "kind": "role", "name": "storm-agent", "tenant_id": tenant }),
    )
    .await;
    assert_eq!(status, 200, "the role enrolls: {role}");
    let role_id = role["principal_id"].as_str().unwrap().to_string();
    enroll_node(&client, &base, &human_id, &tenant, &role_id).await;
    let (status, _) = put(
        &client,
        &base,
        &format!("/v1/profiles/{role_id}"),
        &role_id,
        &visibility_profile(),
    )
    .await;
    assert_eq!(status, 200, "the role writes its profile");

    let open = |key: &str, deadline: &str, expiry: &str| {
        let client = client.clone();
        let base = base.clone();
        let human_id = human_id.clone();
        let tenant = tenant.clone();
        let thread_id = "thr_00000000-0000-7000-8000-000000000001".to_string();
        let key = key.to_string();
        let deadline = deadline.to_string();
        let expiry = expiry.to_string();
        async move {
            let response = client
                .post(format!("{base}/v1/calls"))
                .header(PRINCIPAL_HEADER, &human_id)
                .json(&json!({
                    "tenant_id": tenant,
                    "thread_id": thread_id,
                    "expression": { "scope": "tenant", "presence_states": ["available", "offline"] },
                    "min_participants": 1,
                    "join_deadline": deadline,
                    "expires_at": expiry,
                }))
                .send()
                .await
                .expect("open request");
            let status = response.status().as_u16();
            let body: Value = response.json().await.expect("open json");
            (status, body, key)
        }
    };
    let future_deadline = (chrono::Utc::now() + chrono::Duration::hours(1)).to_rfc3339();
    let future_expiry = (chrono::Utc::now() + chrono::Duration::hours(2)).to_rfc3339();

    // Four opens: the initiator's cap (4) is reached exactly.
    for i in 1..=4 {
        let (status, body, _key) =
            open(&format!("key-storm-{i}"), &future_deadline, &future_expiry).await;
        assert_eq!(status, 200, "the open {i}: {body}");
    }
    // The FIFTH open: the typed 429 names the fan-out limit.
    let (status, refused, _key) = open("key-storm-5", &future_deadline, &future_expiry).await;
    assert_eq!(status, 429, "the fan-out cap refuses: {refused}");
    assert!(
        refused["message"]
            .as_str()
            .unwrap_or("")
            .contains("fan-out"),
        "the refusal names the limit: {refused}"
    );

    // The expiry enforcement: rewind one call's expiry to the past (the
    // test's clock control — the same row-level seed the lease tests use),
    // then the response refuses with the typed reason.
    sqlx::query(
        "UPDATE recruitment_calls SET expires_at = now() - interval '1 minute' \
         WHERE tenant_id = $1 AND status = 'open'",
    )
    .bind(&tenant)
    .execute(&pool)
    .await
    .expect("rewind the expiry");
    let call_ids: Vec<String> = sqlx::query_scalar(
        "SELECT call_id FROM recruitment_calls WHERE tenant_id = $1 AND status = 'open' LIMIT 1",
    )
    .bind(&tenant)
    .fetch_all(&pool)
    .await
    .expect("the call ids");
    let response = client
        .post(format!("{base}/v1/calls/{}/respond", call_ids[0]))
        .header(PRINCIPAL_HEADER, &role_id)
        .json(&json!({ "kind": "join" }))
        .send()
        .await
        .expect("respond request");
    assert_eq!(
        response.status().as_u16(),
        409,
        "the expired call refuses the response"
    );
    let refused: Value = response.json().await.unwrap();
    assert!(
        refused["message"]
            .as_str()
            .unwrap_or("")
            .contains("expired"),
        "the refusal names the expiry: {refused}"
    );
}

/// THE `.3.5.2` subscription acceptance: the open call's topic tags MATCH the
/// subscribers' declared interests — the server records the offers (the
/// advertisement window's durable trace) and the inspection shows them.
#[tokio::test]
async fn the_open_call_advertises_to_the_subscribers() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, human) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "subs-human" }),
    )
    .await;
    assert_eq!(status, 200, "the human enrolls: {human}");
    let tenant = human["tenant_id"].as_str().unwrap().to_string();
    let human_id = human["principal_id"].as_str().unwrap().to_string();

    // Two roles declaring the SAME interest; one also declares a second.
    let profile = |extra: &[&str]| {
        let mut p = visibility_profile();
        p["interests"] = json!(["parser trivia"]);
        for e in extra {
            p["interests"].as_array_mut().unwrap().push(json!(e));
        }
        p
    };
    let (status, role_a) = enroll(
        &client,
        &base,
        json!({ "kind": "role", "name": "subs-agent-a", "tenant_id": tenant }),
    )
    .await;
    assert_eq!(status, 200, "role A enrolls: {role_a}");
    let role_a_id = role_a["principal_id"].as_str().unwrap().to_string();
    enroll_node(&client, &base, &human_id, &tenant, &role_a_id).await;
    let (status, _) = put(
        &client,
        &base,
        &format!("/v1/profiles/{role_a_id}"),
        &role_a_id,
        &profile(&[]),
    )
    .await;
    assert_eq!(status, 200, "role A writes");
    let (status, role_b) = enroll(
        &client,
        &base,
        json!({ "kind": "role", "name": "subs-agent-b", "tenant_id": tenant }),
    )
    .await;
    assert_eq!(status, 200, "role B enrolls: {role_b}");
    let role_b_id = role_b["principal_id"].as_str().unwrap().to_string();
    enroll_node(&client, &base, &human_id, &tenant, &role_b_id).await;
    let (status, _) = put(
        &client,
        &base,
        &format!("/v1/profiles/{role_b_id}"),
        &role_b_id,
        &profile(&["schema drift"]),
    )
    .await;
    assert_eq!(status, 200, "role B writes");

    let deadline = (chrono::Utc::now() + chrono::Duration::hours(1)).to_rfc3339();
    let expiry = (chrono::Utc::now() + chrono::Duration::hours(2)).to_rfc3339();
    let response = client
        .post(format!("{base}/v1/calls"))
        .header(PRINCIPAL_HEADER, &human_id)
        .json(&json!({
            "tenant_id": tenant,
            "thread_id": "thr_00000000-0000-7000-8000-000000000002",
            "expression": {
                "scope": "tenant",
                "interests": ["parser trivia"],
                "presence_states": ["available", "offline"],
            },
            "join_deadline": deadline,
            "expires_at": expiry,
        }))
        .send()
        .await
        .expect("open request");
    assert_eq!(response.status().as_u16(), 200, "the call opens");
    let opened: Value = response.json().await.unwrap();
    let call_id = opened["call_id"].as_str().unwrap().to_string();
    assert_eq!(
        opened["offered_to"],
        json!(2),
        "BOTH matching subscribers are offered: {opened}"
    );

    // The inspection lists the offers (the advertisement's durable trace).
    let (status, inspected) = get(&client, &base, &format!("/v1/calls/{call_id}"), &human_id).await;
    assert_eq!(status, 200, "the inspection: {inspected}");
    let offers = inspected["offers"].as_array().unwrap();
    assert_eq!(offers.len(), 2, "{inspected}");
    for offer in offers {
        let role = offer.as_str().unwrap();
        assert!(
            role == role_a_id || role == role_b_id,
            "the offer names a real subscriber: {offer}"
        );
    }
}

/// THE `.3.5.3` acceptance: the node-initiated thread creation needs the
/// EXPLICIT `thread_create_auto` grant, the §11.5 checklist gates server-side
/// (the topic gate, the confidentiality match, the concurrency gate, the
/// spend bound), and replies do NOT inherit the permission — the plain
/// thread.create stays denied for the role.
#[tokio::test]
async fn the_auto_initiation_lands_under_the_grant_and_the_checklist() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, human) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "auto-human" }),
    )
    .await;
    assert_eq!(status, 200, "the human enrolls: {human}");
    let tenant = human["tenant_id"].as_str().unwrap().to_string();
    let human_id = human["principal_id"].as_str().unwrap().to_string();
    let boundary_id = human["boundary_id"].as_str().unwrap().to_string();
    let (status, role) = enroll(
        &client,
        &base,
        json!({ "kind": "role", "name": "auto-agent", "tenant_id": tenant }),
    )
    .await;
    assert_eq!(status, 200, "the role enrolls: {role}");
    let role_id = role["principal_id"].as_str().unwrap().to_string();
    enroll_node(&client, &base, &human_id, &tenant, &role_id).await;
    let (status, _) = put(
        &client,
        &base,
        &format!("/v1/profiles/{role_id}"),
        &role_id,
        &visibility_profile(),
    )
    .await;
    assert_eq!(status, 200, "the role writes its profile");

    // Without the grant: the auto-initiation refuses (the explicit-grant rule).
    let auto = |body: Value| {
        let client = client.clone();
        let base = base.clone();
        let role_id = role_id.clone();
        async move {
            let response = client
                .post(format!("{base}/v1/threads/auto"))
                .header(PRINCIPAL_HEADER, &role_id)
                .json(&body)
                .send()
                .await
                .expect("auto request");
            let status = response.status().as_u16();
            let parsed: Value = response.json().await.expect("auto json");
            (status, parsed)
        }
    };
    let (status, refused) = auto(json!({
        "tenant_id": tenant,
        "subject": "auto probe",
        "objective": "probe",
        "topics": ["parser trivia"],
    }))
    .await;
    assert_eq!(status, 403, "no auto grant, no initiation: {refused}");

    // The boundary must permit the auto action (the ceiling the grant's
    // subset check rides), then the EXPLICIT grant.
    sqlx::query(
        "UPDATE enrollment_boundaries \
         SET permitted_actions = permitted_actions || '[\"thread_create_auto\"]'::jsonb \
         WHERE boundary_id = $1",
    )
    .bind(&boundary_id)
    .execute(&pool)
    .await
    .expect("the boundary permits the auto action");
    sqlx::query(
        "INSERT INTO authority_grants \
         (grant_id, boundary_id, tenant_id, issuer, subject_kind, subject_id, actions, selector, \
          risk_ceiling, spend_limits, delegable, valid_from, expires_at, status) \
         VALUES ('grt_auto_probe', $1, $2, $3, 'role', $4, '[\"thread_create_auto\"]', \
                 '{\"kind\":\"tenant_wide\"}', 'low', '{\"amount\": 100.0}', false, \
                 now(), now() + interval '1 day', 'active')",
    )
    .bind(&boundary_id)
    .bind(&tenant)
    .bind(&human_id)
    .bind(&role_id)
    .execute(&pool)
    .await
    .expect("seed the auto grant");

    // With the grant: the initiation lands (the topic rides the interests).
    let (status, created) = auto(json!({
        "tenant_id": tenant,
        "subject": "auto probe",
        "objective": "probe",
        "topics": ["parser trivia"],
        "budget_amount": 50.0,
    }))
    .await;
    assert_eq!(status, 200, "the auto-initiation lands: {created}");
    let thread_id = created["thread_id"].as_str().unwrap().to_string();
    assert!(thread_id.starts_with("thr_"), "{created}");

    // The checklist refusals are typed:
    // the topic gate.
    let (status, refused) = auto(json!({
        "tenant_id": tenant,
        "subject": "auto probe 2",
        "objective": "probe",
        "topics": ["undeclared topic"],
    }))
    .await;
    assert_eq!(status, 403, "the topic gate refuses: {refused}");
    assert!(
        refused["message"].as_str().unwrap_or("").contains("topic"),
        "the refusal names the topic gate: {refused}"
    );
    // the spend bound.
    let (status, refused) = auto(json!({
        "tenant_id": tenant,
        "subject": "auto probe 3",
        "objective": "probe",
        "budget_amount": 500.0,
    }))
    .await;
    assert_eq!(status, 403, "the spend bound refuses: {refused}");
    assert!(
        refused["message"].as_str().unwrap_or("").contains("spend"),
        "the refusal names the spend bound: {refused}"
    );

    // Replies do NOT inherit: the plain thread.create stays denied (the role
    // holds only the AUTO action).
    let response = client
        .post(format!("{base}/v1/threads"))
        .header(PRINCIPAL_HEADER, &role_id)
        .json(&serde_json::json!({
            "protocol_version": reasonbraid_core::PROTOCOL_VERSION,
            "operation": "thread.create",
            "request_id": reasonbraid_core::RequestId::new().to_string(),
            "idempotency_key": "key-auto-plain",
            "body": { "tenant_id": tenant, "subject": "plain", "objective": "probe" },
            "client_context": {},
        }))
        .send()
        .await
        .expect("plain create");
    assert_eq!(
        response.status().as_u16(),
        403,
        "the plain create stays denied — no inherited permission"
    );
}

/// The workflow-profile registry + the validation (PHASE-5.1.2, ADR-016):
/// the built-ins list, the custom registration validates the composition
/// (the unknown step / the non-terminal last / the adjudicate-without-blind
/// refusals), and the thread-create boundary resolves the reference (the
/// unknown id is the typed refusal; the known id rides the projection).
#[tokio::test]
async fn the_workflow_profile_registry_validates_and_resolves() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, human) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "wfp-human" }),
    )
    .await;
    assert_eq!(status, 200, "the human enrolls: {human}");
    let human_id = human["principal_id"].as_str().unwrap().to_string();

    // The registry's built-ins list (the eight §13.1 entries).
    let (status, profiles) = get(&client, &base, "/v1/workflow-profiles", &human_id).await;
    assert_eq!(status, 200, "the profiles list: {profiles}");
    let profiles = profiles.as_array().expect("the array");
    assert_eq!(profiles.len(), 8, "{profiles:?}");
    assert!(profiles
        .iter()
        .any(|p| p["profile_id"] == json!("quick_advice")));

    // The custom registration: the VALID steps land.
    let response = client
        .post(format!("{base}/v1/workflow-profiles"))
        .header(PRINCIPAL_HEADER, &human_id)
        .json(&json!({
            "profile_id": "custom_deliberate",
            "steps": ["solicit", "critique", "decide"],
        }))
        .send()
        .await
        .expect("register request");
    assert_eq!(
        response.status().as_u16(),
        200,
        "the valid profile registers"
    );
    let registered: Value = response.json().await.unwrap();
    assert_eq!(registered["version"], json!(1));

    // The invalid compositions refuse with their names.
    for (steps, expected) in [
        (json!(["solicit", "teleport", "decide"]), "vocabulary"),
        (json!(["solicit", "synthesize"]), "terminal"),
        (json!(["solicit", "adjudicate", "decide"]), "blind"),
    ] {
        let response = client
            .post(format!("{base}/v1/workflow-profiles"))
            .header(PRINCIPAL_HEADER, &human_id)
            .json(&json!({ "profile_id": "custom_bad", "steps": steps }))
            .send()
            .await
            .expect("invalid register request");
        assert_eq!(
            response.status().as_u16(),
            400,
            "the invalid profile refuses"
        );
        let refused: Value = response.json().await.unwrap();
        assert!(
            refused["message"].as_str().unwrap().contains(expected),
            "{refused}"
        );
    }

    // The thread-create boundary: the UNKNOWN id is the typed refusal.
    let tenant_id = human["tenant_id"].as_str().unwrap().to_string();
    let response = client
        .post(format!("{base}/v1/threads"))
        .header(PRINCIPAL_HEADER, &human_id)
        .json(&json!({
            "protocol_version": reasonbraid_core::PROTOCOL_VERSION,
            "operation": "thread.create",
            "request_id": reasonbraid_core::RequestId::new().to_string(),
            "idempotency_key": "wfp-unknown",
            "body": {
                "tenant_id": tenant_id,
                "subject": "wfp",
                "objective": "probe",
                "workflow_profile": "no_such_profile",
            },
            "client_context": {},
        }))
        .send()
        .await
        .expect("unknown-profile create");
    assert_eq!(
        response.status().as_u16(),
        400,
        "the unknown profile refuses"
    );
    let refused: Value = response.json().await.unwrap();
    assert!(
        refused["message"]
            .as_str()
            .unwrap()
            .contains("no_such_profile"),
        "{refused}"
    );

    // The KNOWN id rides the projection.
    let response = client
        .post(format!("{base}/v1/threads"))
        .header(PRINCIPAL_HEADER, &human_id)
        .json(&json!({
            "protocol_version": reasonbraid_core::PROTOCOL_VERSION,
            "operation": "thread.create",
            "request_id": reasonbraid_core::RequestId::new().to_string(),
            "idempotency_key": "wfp-known",
            "body": {
                "tenant_id": tenant_id,
                "subject": "wfp-known",
                "objective": "probe",
                "workflow_profile": "independent_panel",
            },
            "client_context": {},
        }))
        .send()
        .await
        .expect("known-profile create");
    assert_eq!(response.status().as_u16(), 200, "the known profile creates");
    let created: Value = response.json().await.unwrap();
    let thread_id = created["thread_id"]
        .as_str()
        .or_else(|| created["thread"]["thread_id"].as_str())
        .unwrap()
        .to_string();
    let (status, thread) = get(
        &client,
        &base,
        &format!("/v1/threads/{thread_id}?tenant_id={tenant_id}"),
        &human_id,
    )
    .await;
    assert_eq!(status, 200, "the thread reads: {thread}");
    assert_eq!(
        thread["state"]["workflow_profile"],
        json!("independent_panel"),
        "the validated reference rides the projection: {thread}"
    );
}

/// The profile-driven execution (PHASE-5.1.3, ADR-016): the projection
/// carries the resolved step sequence + the current index — the create
/// seats step 0, the close advances to the terminal step — and the
/// inspection shows the plan.
#[tokio::test]
async fn the_profile_steps_ride_the_projection_through_the_close() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, human) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "stp-human" }),
    )
    .await;
    assert_eq!(status, 200, "the human enrolls: {human}");
    let human_id = human["principal_id"].as_str().unwrap().to_string();
    let tenant_id = human["tenant_id"].as_str().unwrap().to_string();

    let response = client
        .post(format!("{base}/v1/threads"))
        .header(PRINCIPAL_HEADER, &human_id)
        .json(&json!({
            "protocol_version": reasonbraid_core::PROTOCOL_VERSION,
            "operation": "thread.create",
            "request_id": reasonbraid_core::RequestId::new().to_string(),
            "idempotency_key": "stp-create",
            "body": {
                "tenant_id": tenant_id,
                "subject": "stp",
                "objective": "probe",
                "workflow_profile": "independent_panel",
            },
            "client_context": {},
        }))
        .send()
        .await
        .expect("create request");
    assert_eq!(response.status().as_u16(), 200, "the create succeeds");
    let created: Value = response.json().await.unwrap();
    let thread_id = created["thread_id"].as_str().unwrap().to_string();

    let inspect = || {
        let client = client.clone();
        let base = base.clone();
        let human_id = human_id.clone();
        let thread_id = thread_id.clone();
        let tenant_id = tenant_id.clone();
        async move {
            let (_, state) = get(
                &client,
                &base,
                &format!("/v1/threads/{thread_id}?tenant_id={tenant_id}"),
                &human_id,
            )
            .await;
            state["state"].clone()
        }
    };

    // The create seats step 0 with the resolved sequence.
    let state = inspect().await;
    assert_eq!(state["workflow_profile"], json!("independent_panel"));
    assert_eq!(
        state["workflow_steps"],
        json!(["blind_solicit", "adjudicate", "decide"])
    );
    assert_eq!(state["workflow_step"], json!(0));

    // The close advances to the terminal step (index 2 = `decide`).
    let response = client
        .post(format!("{base}/v1/threads/{thread_id}/commands"))
        .header(PRINCIPAL_HEADER, &human_id)
        .json(&json!({
            "protocol_version": reasonbraid_core::PROTOCOL_VERSION,
            "operation": "thread.close",
            "request_id": reasonbraid_core::RequestId::new().to_string(),
            "idempotency_key": "stp-close",
            "body": { "tenant_id": tenant_id, "reason": "the panel decided" },
            "client_context": {},
        }))
        .send()
        .await
        .expect("close request");
    assert_eq!(response.status().as_u16(), 200, "the close succeeds");
    let state = inspect().await;
    assert_eq!(state["workflow_step"], json!(2));
    assert_eq!(state["workflow_steps"][2], json!("decide"));
}

/// THE `.3.6.2` panel-wiring acceptance: two joiners sharing the provider
/// produce the named overlap group in the panel snapshot's dependence
/// indicators.
#[tokio::test]
async fn the_panel_snapshot_carries_the_dependence_indicators() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, human) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "dep-human" }),
    )
    .await;
    assert_eq!(status, 200, "the human enrolls: {human}");
    let tenant = human["tenant_id"].as_str().unwrap().to_string();
    let human_id = human["principal_id"].as_str().unwrap().to_string();

    let mut roles = Vec::new();
    for name in ["dep-agent-a", "dep-agent-b"] {
        let (status, role) = enroll(
            &client,
            &base,
            json!({ "kind": "role", "name": name, "tenant_id": tenant }),
        )
        .await;
        assert_eq!(status, 200, "the role enrolls: {role}");
        let role_id = role["principal_id"].as_str().unwrap().to_string();
        enroll_node(&client, &base, &human_id, &tenant, &role_id).await;
        let (status, _) = put(
            &client,
            &base,
            &format!("/v1/profiles/{role_id}"),
            &role_id,
            &visibility_profile(),
        )
        .await;
        assert_eq!(status, 200, "the role writes");
        let (status, _) = post(
            &client,
            &base,
            &format!("/v1/profiles/{role_id}/attest"),
            &human_id,
            &json!({ "taxonomy_id": "code_review", "evidence_ref": "evt_dep/20260907" }),
        )
        .await;
        assert_eq!(status, 200, "the owner attests");
        // The incarnation lineage: both roles share the provider (the
        // dependence fact the indicator computes).
        sqlx::query("UPDATE incarnations SET provider = 'openai' WHERE role_id = $1")
            .bind(&role_id)
            .execute(&pool)
            .await
            .expect("seed the lineage");
        roles.push(role_id);
    }

    let deadline = (chrono::Utc::now() + chrono::Duration::hours(1)).to_rfc3339();
    let expiry = (chrono::Utc::now() + chrono::Duration::hours(2)).to_rfc3339();
    let response = client
        .post(format!("{base}/v1/calls"))
        .header(PRINCIPAL_HEADER, &human_id)
        .json(&json!({
            "tenant_id": tenant,
            "thread_id": "thr_00000000-0000-7000-8000-000000000003",
            "expression": {
                "scope": "tenant",
                "capabilities": [{ "taxonomy_id": "code_review", "min_confidence": "owner_attested" }],
                "presence_states": ["available", "offline"],
            },
            "join_deadline": deadline,
            "expires_at": expiry,
        }))
        .send()
        .await
        .expect("open request");
    assert_eq!(response.status().as_u16(), 200);
    let opened: Value = response.json().await.unwrap();
    let call_id = opened["call_id"].as_str().unwrap().to_string();

    for role_id in &roles {
        let response = client
            .post(format!("{base}/v1/calls/{call_id}/respond"))
            .header(PRINCIPAL_HEADER, role_id)
            .json(&json!({ "kind": "join" }))
            .send()
            .await
            .expect("join request");
        assert_eq!(response.status().as_u16(), 200, "the role joins");
    }
    let response = client
        .post(format!("{base}/v1/calls/{call_id}/close"))
        .header(PRINCIPAL_HEADER, &human_id)
        .send()
        .await
        .expect("close request");
    assert_eq!(response.status().as_u16(), 200, "the close snapshots");

    let (status, inspected) = get(&client, &base, &format!("/v1/calls/{call_id}"), &human_id).await;
    assert_eq!(status, 200, "the inspection: {inspected}");
    let indicators = inspected["explanation"]["dependence_indicators"]
        .as_array()
        .unwrap();
    let provider = indicators
        .iter()
        .find(|i| i["attribute"] == json!("provider"))
        .expect("the provider indicator");
    assert_eq!(
        provider["groups"][0]["value"],
        json!("openai"),
        "the shared provider rides the named group: {inspected}"
    );
    assert_eq!(
        provider["groups"][0]["members"].as_array().unwrap().len(),
        2,
        "both members ride: {inspected}"
    );
}

/// THE `.4.1.2` acceptance: the typed §12.1 reference submits; the SAME
/// locator + digest is the replay; the same locator with a DIFFERENT digest
/// is the typed immutability conflict; an unknown field and a malformed
/// digest are typed refusals; the inspection reads the submitted shape back.
#[tokio::test]
async fn a_reference_submits_typed_and_the_locator_is_immutable() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, human) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "res-human" }),
    )
    .await;
    assert_eq!(status, 200, "the human enrolls: {human}");
    let human_id = human["principal_id"].as_str().unwrap().to_string();

    let digest = format!("sha256:{}", "a".repeat(64));
    let body = json!({
        "original_locator": "https://example.org/parser-guidance",
        "scheme": "https",
        "media_type_hint": "text/html",
        "expected_digest": digest,
        "fragment_or_selector": "#rationale",
        "credential_binding_ref": "cb_ref_1",
        "owning_node_or_capability": "nod_00000000-0000-7000-8000-000000000001",
        "visibility_scope": "tenant",
        "purpose": "the parser trivia deliberation's guidance",
        "retention_class": "standard",
        "risk_class": "low",
    });
    let submit = |body: &Value| {
        let client = client.clone();
        let base = base.clone();
        let human_id = human_id.clone();
        let body = body.clone();
        async move {
            let response = client
                .post(format!("{base}/v1/resources"))
                .header(PRINCIPAL_HEADER, &human_id)
                .json(&body)
                .send()
                .await
                .expect("submit request");
            let status = response.status().as_u16();
            let text = response.text().await.expect("submit body");
            let parsed = serde_json::from_str(&text).unwrap_or_else(|_| json!({ "raw": text }));
            (status, parsed)
        }
    };

    // The fresh submit.
    let (status, submitted) = submit(&body).await;
    assert_eq!(status, 200, "the reference submits: {submitted}");
    let resource_id = submitted["resource_id"].as_str().unwrap().to_string();
    assert!(resource_id.starts_with("res_"), "{submitted}");
    assert_eq!(submitted["replayed"], json!(false));

    // The replay: the same locator + digest returns the SAME id.
    let (status, replayed) = submit(&body).await;
    assert_eq!(status, 200, "the replay: {replayed}");
    assert_eq!(replayed["resource_id"], json!(resource_id));
    assert_eq!(replayed["replayed"], json!(true));

    // The immutability: the same locator with a DIFFERENT digest conflicts.
    let mut changed = body.clone();
    changed["expected_digest"] = json!(format!("sha256:{}", "b".repeat(64)));
    let (status, conflicted) = submit(&changed).await;
    assert_eq!(
        status, 409,
        "the locator's digest is immutable: {conflicted}"
    );
    assert_eq!(conflicted["code"], json!("locator_digest_conflict"));

    // An unknown field is the typed 422.
    let mut forged = body.clone();
    forged["fabricated"] = json!(true);
    let (status, rejected) = submit(&forged).await;
    assert_eq!(status, 422, "the unknown field is rejected: {rejected}");

    // A malformed digest is the typed 400.
    let mut malformed = body.clone();
    malformed["expected_digest"] = json!("md5:not-sha256");
    let (status, refused) = submit(&malformed).await;
    assert_eq!(status, 400, "the malformed digest is refused: {refused}");
    assert!(
        refused["message"].as_str().unwrap_or("").contains("sha256"),
        "the refusal names the scheme: {refused}"
    );

    // The inspection reads the submitted shape back.
    let (status, inspected) = get(
        &client,
        &base,
        &format!("/v1/resources/{resource_id}"),
        &human_id,
    )
    .await;
    assert_eq!(status, 200, "the inspection: {inspected}");
    assert_eq!(
        inspected["reference"]["original_locator"],
        json!("https://example.org/parser-guidance")
    );
    assert_eq!(inspected["reference"]["expected_digest"], json!(digest));
}

/// THE `.4.1.3` acceptance: the registry ships the §12.2 shape — the
/// operator registers two resolvers (https + git), the resolution applies
/// the scheme + the ADR-018 isolation filters FIRST (the weaker sandbox is
/// ineligible), then the latency rank; an unsupported scheme is the
/// explicit `resource_unresolvable_now` — the reference stays submitted.
#[tokio::test]
async fn the_resolver_registry_resolves_and_fails_explicitly() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, human) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "rsv-human" }),
    )
    .await;
    assert_eq!(status, 200, "the human enrolls: {human}");
    let human_id = human["principal_id"].as_str().unwrap().to_string();

    // The operator registers two resolvers.
    let register = |body: Value| {
        let client = client.clone();
        let base = base.clone();
        let human_id = human_id.clone();
        async move {
            let response = client
                .post(format!("{base}/v1/resolvers"))
                .header(PRINCIPAL_HEADER, &human_id)
                .json(&body)
                .send()
                .await
                .expect("register request");
            let status = response.status().as_u16();
            let text = response.text().await.expect("register body");
            let parsed = serde_json::from_str(&text).unwrap_or_else(|_| json!({ "raw": text }));
            (status, parsed)
        }
    };
    let (status, _) = register(json!({
        "resolver_id": "rsv-https-fast",
        "schemes": ["https"],
        "media_types": ["text/html"],
        "egress_class": "listed",
        "sandbox_level": "constrained_process",
        "latency_range_ms": { "min": 100, "max": 200 },
        "version": "0.1.0",
    }))
    .await;
    assert_eq!(status, 200, "the first resolver registers");
    let (status, _) = register(json!({
        "resolver_id": "rsv-https-slow",
        "schemes": ["https"],
        "media_types": ["text/html"],
        "egress_class": "any",
        "sandbox_level": "process",
        "latency_range_ms": { "min": 5000, "max": 10000 },
        "version": "0.1.0",
    }))
    .await;
    assert_eq!(status, 200, "the second resolver registers");
    let (status, _) = register(json!({
        "resolver_id": "rsv-git",
        "schemes": ["git"],
        "egress_class": "listed",
        "sandbox_level": "constrained_process",
        "version": "0.1.0",
    }))
    .await;
    assert_eq!(status, 200, "the git resolver registers");

    // The ADR-018 vocabulary refuses the off-ladder claim.
    let (status, refused) = register(json!({
        "resolver_id": "rsv-bad",
        "schemes": ["https"],
        "egress_class": "mars",
        "sandbox_level": "constrained_process",
        "version": "0.1.0",
    }))
    .await;
    assert_eq!(status, 400, "the off-ladder egress is refused: {refused}");

    // Submit an https reference.
    let (status, submitted): (u16, Value) = {
        let response = client
            .post(format!("{base}/v1/resources"))
            .header(PRINCIPAL_HEADER, &human_id)
            .json(&json!({
                "original_locator": "https://example.org/guidance-2",
                "scheme": "https",
            }))
            .send()
            .await
            .expect("submit request");
        (
            response.status().as_u16(),
            response.json().await.expect("submit json"),
        )
    };
    assert_eq!(status, 200, "the reference submits: {submitted}");
    let resource_id = submitted["resource_id"].as_str().unwrap().to_string();

    // The resolution: the required constrained_process filters the SLOW
    // resolver out (it declares `process` — below the requirement); the
    // fast one ranks first anyway (the latency order).
    let response = client
        .post(format!("{base}/v1/resources/{resource_id}/resolve"))
        .header(PRINCIPAL_HEADER, &human_id)
        .json(&json!({ "required_sandbox": "constrained_process" }))
        .send()
        .await
        .expect("resolve request");
    assert_eq!(response.status().as_u16(), 200, "the resolution");
    let resolved: Value = response.json().await.unwrap();
    assert_eq!(
        resolved["resolvers"],
        json!(["rsv-https-fast"]),
        "the filter + the rank: {resolved}"
    );
    assert_eq!(resolved["unresolvable_now"], json!(false));

    // The unsupported scheme: the explicit unresolvable-now (the reference
    // stays submitted — the inspection still reads it).
    let (status, submitted): (u16, Value) = {
        let response = client
            .post(format!("{base}/v1/resources"))
            .header(PRINCIPAL_HEADER, &human_id)
            .json(&json!({
                "original_locator": "ftp://example.org/archive",
                "scheme": "ftp",
            }))
            .send()
            .await
            .expect("submit request");
        (
            response.status().as_u16(),
            response.json().await.expect("submit json"),
        )
    };
    assert_eq!(status, 200, "the ftp reference submits: {submitted}");
    let ftp_id = submitted["resource_id"].as_str().unwrap().to_string();
    let response = client
        .post(format!("{base}/v1/resources/{ftp_id}/resolve"))
        .header(PRINCIPAL_HEADER, &human_id)
        .json(&json!({}))
        .send()
        .await
        .expect("resolve request");
    assert_eq!(response.status().as_u16(), 200, "the explicit failure");
    let unresolved: Value = response.json().await.unwrap();
    assert_eq!(
        unresolved["unresolvable_now"],
        json!(true),
        "the unsupported scheme fails explicitly: {unresolved}"
    );
    assert!(unresolved["resolvers"].as_array().unwrap().is_empty());
    // The reference is PRESERVED (still submitted, never fabricated).
    let (status, still_there) = get(
        &client,
        &base,
        &format!("/v1/resources/{ftp_id}"),
        &human_id,
    )
    .await;
    assert_eq!(status, 200, "the reference stays submitted: {still_there}");
    assert_eq!(
        still_there["reference"]["original_locator"],
        json!("ftp://example.org/archive")
    );
}

/// The built-in R0 pack (PHASE-4.2.3): the seeded registry entry resolves
/// the https references under its OWN claimed classes, the acquisition runs
/// under the fetcher's real policy — the loopback/private literals refuse
/// with the class NAMED (the SSRF proof through the resolution path) — and
/// the reference stays submitted either way. Requiring MORE isolation than
/// the built-in honestly declares is the explicit unresolvable-now, never a
/// silent downgrade.
#[tokio::test]
async fn the_r0_resolver_resolves_https_and_the_execution_names_the_refused_class() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, human) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "r0-human" }),
    )
    .await;
    assert_eq!(status, 200, "the human enrolls: {human}");
    let human_id = human["principal_id"].as_str().unwrap().to_string();

    let submit = |locator: String| {
        let client = client.clone();
        let base = base.clone();
        let human_id = human_id.clone();
        async move {
            let response = client
                .post(format!("{base}/v1/resources"))
                .header(PRINCIPAL_HEADER, &human_id)
                .json(&json!({
                    "original_locator": locator,
                    "scheme": "https",
                }))
                .send()
                .await
                .expect("submit request");
            let parsed: Value = response.json().await.expect("submit json");
            parsed["resource_id"].as_str().unwrap().to_string()
        }
    };
    let resolve = |resource_id: String, body: Value| {
        let client = client.clone();
        let base = base.clone();
        let human_id = human_id.clone();
        async move {
            let response = client
                .post(format!("{base}/v1/resources/{resource_id}/resolve"))
                .header(PRINCIPAL_HEADER, &human_id)
                .json(&body)
                .send()
                .await
                .expect("resolve request");
            let status = response.status().as_u16();
            let parsed: Value = response.json().await.expect("resolve json");
            (status, parsed)
        }
    };

    // The https reference under the built-in's OWN classes: the R0 entry is
    // the ranked resolver, and the acquisition refuses the loopback literal
    // with the class NAMED — before any socket opens (the `.2.2` proof,
    // now through the resolution path).
    let resource_id = submit("https://127.0.0.1/guidance".to_string()).await;
    let (status, resolved) = resolve(
        resource_id.clone(),
        json!({ "required_sandbox": "none", "required_egress": "listed" }),
    )
    .await;
    assert_eq!(status, 200, "the resolution");
    assert_eq!(
        resolved["resolvers"],
        json!(["r0-https-fetcher"]),
        "the built-in ranks: {resolved}"
    );
    assert_eq!(resolved["unresolvable_now"], json!(false));
    assert_eq!(
        resolved["acquisition_error"]["kind"],
        json!("destination_refused"),
        "the refusal names its kind: {resolved}"
    );
    assert!(
        resolved["acquisition_error"]["message"]
            .as_str()
            .unwrap()
            .contains("loopback"),
        "the refusal names the class: {resolved}"
    );

    // The private literal names its class too.
    let private_id = submit("https://10.0.0.1/escape".to_string()).await;
    let (_, refused) = resolve(
        private_id,
        json!({ "required_sandbox": "none", "required_egress": "listed" }),
    )
    .await;
    assert!(
        refused["acquisition_error"]["message"]
            .as_str()
            .unwrap()
            .contains("private"),
        "the private class is named: {refused}"
    );

    // The reference is PRESERVED through the refusal (submitted, never
    // fabricated).
    let (status, still_there) = get(
        &client,
        &base,
        &format!("/v1/resources/{resource_id}"),
        &human_id,
    )
    .await;
    assert_eq!(status, 200, "the reference stays submitted: {still_there}");
    assert_eq!(
        still_there["reference"]["original_locator"],
        json!("https://127.0.0.1/guidance")
    );

    // Requiring MORE isolation than the built-in declares (sandbox `none`)
    // is the explicit unresolvable-now — the ADR-018 filter holds on the
    // real entry, never a silent downgrade.
    let (_, strict) = resolve(
        resource_id,
        json!({ "required_sandbox": "constrained_process", "required_egress": "listed" }),
    )
    .await;
    assert_eq!(
        strict["unresolvable_now"],
        json!(true),
        "the stricter requirement filters the built-in out: {strict}"
    );
    assert!(strict["resolvers"].as_array().unwrap().is_empty());
}

/// The built-in R1 pack (PHASE-4.3.3): the seeded `git` entry resolves the
/// git references under its own claimed classes, the acquisition runs under
/// the real policy — the loopback literal refuses with the class NAMED (the
/// R1 SSRF proof through the resolution path) — and the reference stays
/// submitted.
#[tokio::test]
async fn the_r1_resolver_resolves_git_and_the_execution_names_the_refused_class() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, human) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "r1-human" }),
    )
    .await;
    assert_eq!(status, 200, "the human enrolls: {human}");
    let human_id = human["principal_id"].as_str().unwrap().to_string();

    // The git reference (the https transport URL + the fragment-carried
    // ref) under the built-in's own classes.
    let (status, submitted): (u16, Value) = {
        let response = client
            .post(format!("{base}/v1/resources"))
            .header(PRINCIPAL_HEADER, &human_id)
            .json(&json!({
                "original_locator": "https://127.0.0.1/repo.git#main",
                "scheme": "git",
            }))
            .send()
            .await
            .expect("submit request");
        (
            response.status().as_u16(),
            response.json().await.expect("submit json"),
        )
    };
    assert_eq!(status, 200, "the git reference submits: {submitted}");
    let resource_id = submitted["resource_id"].as_str().unwrap().to_string();

    let response = client
        .post(format!("{base}/v1/resources/{resource_id}/resolve"))
        .header(PRINCIPAL_HEADER, &human_id)
        .json(&json!({ "required_sandbox": "none", "required_egress": "listed" }))
        .send()
        .await
        .expect("resolve request");
    assert_eq!(response.status().as_u16(), 200, "the resolution");
    let resolved: Value = response.json().await.unwrap();
    assert_eq!(
        resolved["resolvers"],
        json!(["r1-git-fetcher"]),
        "the R1 built-in ranks: {resolved}"
    );
    assert_eq!(resolved["unresolvable_now"], json!(false));
    assert_eq!(
        resolved["acquisition_error"]["kind"],
        json!("destination_refused"),
        "the refusal names its kind: {resolved}"
    );
    assert!(
        resolved["acquisition_error"]["message"]
            .as_str()
            .unwrap()
            .contains("loopback"),
        "the refusal names the class: {resolved}"
    );

    // The reference is PRESERVED through the refusal.
    let (status, still_there) = get(
        &client,
        &base,
        &format!("/v1/resources/{resource_id}"),
        &human_id,
    )
    .await;
    assert_eq!(status, 200, "the reference stays submitted: {still_there}");
    assert_eq!(
        still_there["reference"]["original_locator"],
        json!("https://127.0.0.1/repo.git#main")
    );

    // The stricter requirement is the explicit unresolvable-now (the
    // honest `none` claim, never a silent downgrade).
    let response = client
        .post(format!("{base}/v1/resources/{resource_id}/resolve"))
        .header(PRINCIPAL_HEADER, &human_id)
        .json(&json!({ "required_sandbox": "constrained_process", "required_egress": "listed" }))
        .send()
        .await
        .expect("strict resolve request");
    let strict: Value = response.json().await.unwrap();
    assert_eq!(
        strict["unresolvable_now"],
        json!(true),
        "the stricter requirement filters the R1 built-in out: {strict}"
    );
}

/// The built-in R2 pack (PHASE-4.4.3): a reference CARRYING an extraction
/// media-type hint ranks the R2 worker under its own classes, and the
/// pipeline's acquisition leg refuses the loopback literal with the class
/// NAMED (the SSRF proof through the R2 pipeline). The hintless reference
/// stays the acquisition-only path (the R0 built-in ranks).
#[tokio::test]
async fn the_r2_resolver_ranks_the_hinted_reference_and_the_pipeline_names_the_refusal() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, human) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "r2-human" }),
    )
    .await;
    assert_eq!(status, 200, "the human enrolls: {human}");
    let human_id = human["principal_id"].as_str().unwrap().to_string();

    let submit = |body: Value| {
        let client = client.clone();
        let base = base.clone();
        let human_id = human_id.clone();
        async move {
            let response = client
                .post(format!("{base}/v1/resources"))
                .header(PRINCIPAL_HEADER, &human_id)
                .json(&body)
                .send()
                .await
                .expect("submit request");
            let parsed: Value = response.json().await.expect("submit json");
            parsed["resource_id"].as_str().unwrap().to_string()
        }
    };

    // The HINTED reference (the extraction media type) ranks the R2 worker
    // under its own classes; the pipeline's acquisition leg refuses the
    // loopback with the class named.
    let hinted = submit(json!({
        "original_locator": "https://127.0.0.1/feed.xml",
        "scheme": "https",
        "media_type_hint": "application/atom+xml",
    }))
    .await;
    let response = client
        .post(format!("{base}/v1/resources/{hinted}/resolve"))
        .header(PRINCIPAL_HEADER, &human_id)
        .json(&json!({ "required_sandbox": "process", "required_egress": "listed" }))
        .send()
        .await
        .expect("resolve request");
    assert_eq!(response.status().as_u16(), 200, "the resolution");
    let resolved: Value = response.json().await.unwrap();
    assert_eq!(
        resolved["resolvers"],
        json!(["r2-extract-worker"]),
        "the R2 built-in ranks the hinted reference: {resolved}"
    );
    assert_eq!(
        resolved["acquisition_error"]["kind"],
        json!("destination_refused"),
        "the pipeline's acquisition leg names its refusal: {resolved}"
    );
    assert!(
        resolved["acquisition_error"]["message"]
            .as_str()
            .unwrap()
            .contains("loopback"),
        "the class is named: {resolved}"
    );

    // The reference is PRESERVED through the refusal.
    let (status, still_there) = get(
        &client,
        &base,
        &format!("/v1/resources/{hinted}"),
        &human_id,
    )
    .await;
    assert_eq!(status, 200, "the reference stays submitted: {still_there}");

    // The HINTLESS reference stays the acquisition-only path: the R0
    // built-in ranks (its latency midpoint is the lowest).
    let hintless = submit(json!({
        "original_locator": "https://127.0.0.1/page",
        "scheme": "https",
    }))
    .await;
    let response = client
        .post(format!("{base}/v1/resources/{hintless}/resolve"))
        .header(PRINCIPAL_HEADER, &human_id)
        .json(&json!({ "required_sandbox": "none", "required_egress": "listed" }))
        .send()
        .await
        .expect("resolve request");
    let resolved: Value = response.json().await.unwrap();
    assert_eq!(
        resolved["resolvers"],
        json!(["r0-https-fetcher"]),
        "the hintless reference keeps the acquisition-only path: {resolved}"
    );

    // The stricter requirement (above the R2 worker's `process` claim) is
    // the explicit unresolvable-now.
    let response = client
        .post(format!("{base}/v1/resources/{hinted}/resolve"))
        .header(PRINCIPAL_HEADER, &human_id)
        .json(&json!({ "required_sandbox": "constrained_process", "required_egress": "listed" }))
        .send()
        .await
        .expect("strict resolve request");
    let strict: Value = response.json().await.unwrap();
    assert_eq!(
        strict["unresolvable_now"],
        json!(true),
        "the stricter requirement filters the R2 built-in out: {strict}"
    );
}

/// The OPT-IN gate (PHASE-4.5.3): the R3/R5/RX packs resolve ONLY while the
/// gate is open — closed, the binding-carrying reference is the explicit
/// unresolvable-now (the disabled pack has no row). Open, the R5 pack ranks
/// the binding-carrying reference and the authenticated acquisition refuses
/// the loopback with the class NAMED (the SSRF proof through the
/// authenticated path); the R3 render and the RX capability call rank their
/// references too.
#[tokio::test]
async fn the_gated_packs_resolve_only_while_the_gate_is_open() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };

    // The gate CLOSED (the default everywhere): the binding-carrying
    // reference has no row to rank — the explicit unresolvable-now.
    {
        let server = TestServer::start(&pool).await;
        let base = server.base();
        let client = reqwest::Client::new();
        let (status, human) = enroll(
            &client,
            &base,
            json!({ "kind": "human", "name": "gate-human" }),
        )
        .await;
        assert_eq!(status, 200, "the human enrolls: {human}");
        let human_id = human["principal_id"].as_str().unwrap().to_string();
        let (status, submitted): (u16, Value) = {
            let response = client
                .post(format!("{base}/v1/resources"))
                .header(PRINCIPAL_HEADER, &human_id)
                .json(&json!({
                    "original_locator": "https://private.example/report",
                    "scheme": "https",
                    "credential_binding_ref": "cred_gate_test",
                }))
                .send()
                .await
                .expect("submit request");
            (
                response.status().as_u16(),
                response.json().await.expect("submit json"),
            )
        };
        assert_eq!(
            status, 200,
            "the binding-carrying reference submits: {submitted}"
        );
        let resource_id = submitted["resource_id"].as_str().unwrap().to_string();
        let response = client
            .post(format!("{base}/v1/resources/{resource_id}/resolve"))
            .header(PRINCIPAL_HEADER, &human_id)
            .json(&json!({ "required_sandbox": "none", "required_egress": "listed" }))
            .send()
            .await
            .expect("resolve request");
        let resolved: Value = response.json().await.unwrap();
        assert_eq!(
            resolved["unresolvable_now"],
            json!(true),
            "the disabled pack has no row: {resolved}"
        );
    }

    // The gate OPEN: the startup sync registers the rows.
    reasonbraid_server::sync_gated_entries(&pool, true)
        .await
        .expect("the gate opens");

    let broker = std::sync::Arc::new(reasonbraid_server::broker::Broker::default());
    broker.register(
        "cred_gate_test",
        reasonbraid_server::broker::Credential::new("test-token-read", "tok_GATE_SECRET"),
    );
    let server = TestServer::start_gated(&pool, true, broker).await;
    let base = server.base();
    let client = reqwest::Client::new();
    let (status, human) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "gate-open-human" }),
    )
    .await;
    assert_eq!(status, 200, "the human enrolls: {human}");
    let human_id = human["principal_id"].as_str().unwrap().to_string();

    // The R5 pack ranks the binding-carrying reference; the authenticated
    // acquisition refuses the loopback with the class named.
    let (status, submitted): (u16, Value) = {
        let response = client
            .post(format!("{base}/v1/resources"))
            .header(PRINCIPAL_HEADER, &human_id)
            .json(&json!({
                "original_locator": "https://127.0.0.1/private",
                "scheme": "https",
                "credential_binding_ref": "cred_gate_test",
            }))
            .send()
            .await
            .expect("submit request");
        (
            response.status().as_u16(),
            response.json().await.expect("submit json"),
        )
    };
    assert_eq!(
        status, 200,
        "the binding-carrying reference submits: {submitted}"
    );
    let resource_id = submitted["resource_id"].as_str().unwrap().to_string();
    let response = client
        .post(format!("{base}/v1/resources/{resource_id}/resolve"))
        .header(PRINCIPAL_HEADER, &human_id)
        .json(&json!({ "required_sandbox": "none", "required_egress": "listed" }))
        .send()
        .await
        .expect("resolve request");
    let resolved: Value = response.json().await.unwrap();
    assert_eq!(
        resolved["resolvers"],
        json!(["r5-credential-broker"]),
        "the R5 pack ranks the binding-carrying reference: {resolved}"
    );
    assert_eq!(
        resolved["acquisition_error"]["kind"],
        json!("destination_refused"),
        "the authenticated acquisition names its refusal: {resolved}"
    );
    assert!(
        resolved["acquisition_error"]["message"]
            .as_str()
            .unwrap()
            .contains("loopback"),
        "the class is named: {resolved}"
    );

    // The R3 pack ranks the render reference; the pre-flight refuses the
    // loopback BEFORE any worker spawn.
    let (status, render_ref): (u16, Value) = {
        let response = client
            .post(format!("{base}/v1/resources"))
            .header(PRINCIPAL_HEADER, &human_id)
            .json(&json!({
                "original_locator": "https://127.0.0.1/page",
                "scheme": "web+render",
            }))
            .send()
            .await
            .expect("submit request");
        (
            response.status().as_u16(),
            response.json().await.expect("submit json"),
        )
    };
    assert_eq!(status, 200, "the render reference submits: {render_ref}");
    let render_id = render_ref["resource_id"].as_str().unwrap().to_string();
    let response = client
        .post(format!("{base}/v1/resources/{render_id}/resolve"))
        .header(PRINCIPAL_HEADER, &human_id)
        .json(&json!({ "required_sandbox": "none", "required_egress": "listed" }))
        .send()
        .await
        .expect("resolve request");
    let resolved: Value = response.json().await.unwrap();
    assert_eq!(
        resolved["resolvers"],
        json!(["r3-browser-worker"]),
        "the R3 pack ranks the render reference: {resolved}"
    );
    assert_eq!(
        resolved["acquisition_error"]["kind"],
        json!("destination_refused"),
        "the pre-flight refuses before any worker spawn: {resolved}"
    );

    // The RX pack ranks the agent-mediated reference and publishes the
    // §12.8 capability call.
    let (status, agent_ref): (u16, Value) = {
        let response = client
            .post(format!("{base}/v1/resources"))
            .header(PRINCIPAL_HEADER, &human_id)
            .json(&json!({
                "original_locator": "https://internal.example/data",
                "scheme": "web+agent",
            }))
            .send()
            .await
            .expect("submit request");
        (
            response.status().as_u16(),
            response.json().await.expect("submit json"),
        )
    };
    assert_eq!(status, 200, "the agent reference submits: {agent_ref}");
    let agent_id = agent_ref["resource_id"].as_str().unwrap().to_string();
    let response = client
        .post(format!("{base}/v1/resources/{agent_id}/resolve"))
        .header(PRINCIPAL_HEADER, &human_id)
        .json(&json!({ "required_sandbox": "none", "required_egress": "listed" }))
        .send()
        .await
        .expect("resolve request");
    let resolved: Value = response.json().await.unwrap();
    assert_eq!(
        resolved["resolvers"],
        json!(["rx-agent-mediated"]),
        "the RX pack ranks the agent reference: {resolved}"
    );
    assert_eq!(
        resolved["acquisition_call"]["locator"],
        json!("https://internal.example/data"),
        "the capability call publishes the locator: {resolved}"
    );

    // The gate CLOSED again: the sync REMOVES the rows — the same
    // reference is the explicit unresolvable-now.
    reasonbraid_server::sync_gated_entries(&pool, false)
        .await
        .expect("the gate closes");
    let response = client
        .post(format!("{base}/v1/resources/{resource_id}/resolve"))
        .header(PRINCIPAL_HEADER, &human_id)
        .json(&json!({ "required_sandbox": "none", "required_egress": "listed" }))
        .send()
        .await
        .expect("resolve request");
    let resolved: Value = response.json().await.unwrap();
    assert_eq!(
        resolved["unresolvable_now"],
        json!(true),
        "the closed gate has no rows to rank: {resolved}"
    );
}

/// The evidence snapshot store (PHASE-4.6.1): the typed submission verifies
/// the content-addressing (the bytes MUST hash to the declared digest), the
/// same reference + digest is the REPLAY, and the deletion is the tombstone
/// + the reason — never a silent disappearance.
#[tokio::test]
async fn the_snapshot_store_roundtrips_replays_and_tombstones() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, human) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "snp-human" }),
    )
    .await;
    assert_eq!(status, 200, "the human enrolls: {human}");
    let human_id = human["principal_id"].as_str().unwrap().to_string();

    // The reference the snapshot points at.
    let (status, submitted): (u16, Value) = {
        let response = client
            .post(format!("{base}/v1/resources"))
            .header(PRINCIPAL_HEADER, &human_id)
            .json(&json!({
                "original_locator": "https://example.org/evidence",
                "scheme": "https",
            }))
            .send()
            .await
            .expect("submit request");
        (
            response.status().as_u16(),
            response.json().await.expect("submit json"),
        )
    };
    assert_eq!(status, 200, "the reference submits: {submitted}");
    let reference_id = submitted["resource_id"].as_str().unwrap().to_string();

    // The bytes + their ADR-011 digest.
    let bytes = b"the acquired evidence bytes";
    let digest = reasonbraid_server::fetcher::digest_sha256_hex(bytes);
    let snapshot_body = |digest: &str| {
        json!({
            "reference_id": reference_id,
            "original_locator": "https://example.org/evidence",
            "final_locator": "https://example.org/evidence",
            "resolver_id": "r0-https-fetcher",
            "resolver_version": "0.1.0",
            "raw_digest": digest,
            "byte_length": bytes.len(),
            "media_type": "text/plain",
            "bytes_base64": base64(bytes),
        })
    };
    // A tiny local base64 helper (the test's own — the API's decoder is
    // under test, not this one).
    fn base64(bytes: &[u8]) -> String {
        const TABLE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut out = String::new();
        for chunk in bytes.chunks(3) {
            let b0 = chunk[0] as u32;
            let b1 = chunk.get(1).copied().map(|b| b as u32).unwrap_or(0);
            let b2 = chunk.get(2).copied().map(|b| b as u32).unwrap_or(0);
            let triple = (b0 << 16) | (b1 << 8) | b2;
            out.push(TABLE[(triple >> 18) as usize & 63] as char);
            out.push(TABLE[(triple >> 12) as usize & 63] as char);
            out.push(if chunk.len() > 1 {
                TABLE[(triple >> 6) as usize & 63] as char
            } else {
                '='
            });
            out.push(if chunk.len() > 2 {
                TABLE[triple as usize & 63] as char
            } else {
                '='
            });
        }
        out
    }

    // The submit → the read-back.
    let response = client
        .post(format!("{base}/v1/snapshots"))
        .header(PRINCIPAL_HEADER, &human_id)
        .json(&snapshot_body(&digest))
        .send()
        .await
        .expect("snapshot request");
    let submit_status = response.status().as_u16();
    let outcome: Value = response.json().await.unwrap();
    assert_eq!(submit_status, 200, "the snapshot submits: {outcome}");
    assert_eq!(outcome["replay"], json!(false));
    let snapshot_id = outcome["snapshot_id"].as_str().unwrap().to_string();

    let (status, stored) = get(
        &client,
        &base,
        &format!("/v1/snapshots/{snapshot_id}"),
        &human_id,
    )
    .await;
    assert_eq!(status, 200, "the snapshot reads back: {stored}");
    assert_eq!(stored["raw_digest"], json!(digest));
    assert_eq!(stored["byte_length"], json!(bytes.len()));
    assert_eq!(stored["deleted_at"], Value::Null);

    // The replay: the same reference + digest returns the SAME id.
    let response = client
        .post(format!("{base}/v1/snapshots"))
        .header(PRINCIPAL_HEADER, &human_id)
        .json(&snapshot_body(&digest))
        .send()
        .await
        .expect("replay request");
    let replay: Value = response.json().await.unwrap();
    assert_eq!(replay["replay"], json!(true));
    assert_eq!(replay["snapshot_id"], json!(snapshot_id));

    // The digest mismatch: the content-addressing is verified, not trusted.
    let response = client
        .post(format!("{base}/v1/snapshots"))
        .header(PRINCIPAL_HEADER, &human_id)
        .json(&snapshot_body(
            "sha256:0000000000000000000000000000000000000000000000000000000000000000",
        ))
        .send()
        .await
        .expect("mismatch request");
    assert_eq!(response.status().as_u16(), 400, "the mismatch refuses");
    let refused: Value = response.json().await.unwrap();
    assert!(
        refused["message"].as_str().unwrap().contains("hash"),
        "{refused}"
    );

    // The tombstone: the deletion records the reason + the time.
    let response = client
        .delete(format!("{base}/v1/snapshots/{snapshot_id}"))
        .header(PRINCIPAL_HEADER, &human_id)
        .json(&json!({ "reason": "the retention expired" }))
        .send()
        .await
        .expect("tombstone request");
    assert_eq!(response.status().as_u16(), 200, "the tombstone lands");
    let tombstoned: Value = response.json().await.unwrap();
    assert_eq!(tombstoned["tombstoned"], json!(true));

    let (status, stored) = get(
        &client,
        &base,
        &format!("/v1/snapshots/{snapshot_id}"),
        &human_id,
    )
    .await;
    assert_eq!(
        status, 200,
        "the tombstoned snapshot stays readable: {stored}"
    );
    assert!(stored["deleted_at"].is_string(), "{stored}");
    assert_eq!(stored["deletion_reason"], json!("the retention expired"));
}

/// The derivation graph (PHASE-4.6.2): every transformation is a
/// Derivation edge — the content MUST hash to its declared digest, the
/// parent snapshot must exist, the same parent + kind + digest is the
/// REPLAY, and the children traversal lists the derived edges.
#[tokio::test]
async fn the_derivation_graph_edges_roundtrip_and_replay() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, human) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "drv-human" }),
    )
    .await;
    assert_eq!(status, 200, "the human enrolls: {human}");
    let human_id = human["principal_id"].as_str().unwrap().to_string();

    // The parent: a reference + its snapshot.
    let (status, submitted): (u16, Value) = {
        let response = client
            .post(format!("{base}/v1/resources"))
            .header(PRINCIPAL_HEADER, &human_id)
            .json(&json!({
                "original_locator": "https://example.org/derivation-parent",
                "scheme": "https",
            }))
            .send()
            .await
            .expect("submit request");
        (
            response.status().as_u16(),
            response.json().await.expect("submit json"),
        )
    };
    assert_eq!(status, 200, "the reference submits: {submitted}");
    let reference_id = submitted["resource_id"].as_str().unwrap().to_string();
    let bytes = b"the parent bytes";
    let parent_digest = reasonbraid_server::fetcher::digest_sha256_hex(bytes);
    let response = client
        .post(format!("{base}/v1/snapshots"))
        .header(PRINCIPAL_HEADER, &human_id)
        .json(&json!({
            "reference_id": reference_id,
            "original_locator": "https://example.org/derivation-parent",
            "final_locator": "https://example.org/derivation-parent",
            "resolver_id": "r0-https-fetcher",
            "resolver_version": "0.1.0",
            "raw_digest": parent_digest,
            "byte_length": bytes.len(),
            "media_type": "text/plain",
            "bytes_base64": "dGhlIHBhcmVudCBieXRlcw==",
        }))
        .send()
        .await
        .expect("snapshot request");
    let snapshot_outcome: Value = response.json().await.unwrap();
    let snapshot_id = snapshot_outcome["snapshot_id"]
        .as_str()
        .unwrap()
        .to_string();

    // The derivation: the derived chunk with its own digest.
    let content = "the derived chunk text";
    let chunk_digest = reasonbraid_server::fetcher::digest_sha256_hex(content.as_bytes());
    let body = |digest: &str| {
        json!({
            "parent_snapshot_id": snapshot_id,
            "derived_kind": "chunk",
            "derived_digest": digest,
            "content": content,
            "extraction_version": "0.1.0",
        })
    };
    let response = client
        .post(format!("{base}/v1/derivations"))
        .header(PRINCIPAL_HEADER, &human_id)
        .json(&body(&chunk_digest))
        .send()
        .await
        .expect("derivation request");
    assert_eq!(response.status().as_u16(), 200, "the derivation submits");
    let outcome: Value = response.json().await.unwrap();
    let derivation_id = outcome["derivation_id"].as_str().unwrap().to_string();

    // The replay: the same parent + kind + digest → the same id.
    let response = client
        .post(format!("{base}/v1/derivations"))
        .header(PRINCIPAL_HEADER, &human_id)
        .json(&body(&chunk_digest))
        .send()
        .await
        .expect("replay request");
    let replay: Value = response.json().await.unwrap();
    assert_eq!(replay["derivation_id"], json!(derivation_id));

    // The traversal: the snapshot's children list the edge.
    let (status, children) = get(
        &client,
        &base,
        &format!("/v1/snapshots/{snapshot_id}/derivations"),
        &human_id,
    )
    .await;
    assert_eq!(status, 200, "the children read: {children}");
    let children = children.as_array().expect("the children array");
    assert_eq!(children.len(), 1, "{children:?}");
    assert_eq!(children[0]["derivation_id"], json!(derivation_id));
    assert_eq!(children[0]["derived_kind"], json!("chunk"));
    assert_eq!(children[0]["content"], json!(content));

    // The digest mismatch: the content must hash to the declared digest.
    let response = client
        .post(format!("{base}/v1/derivations"))
        .header(PRINCIPAL_HEADER, &human_id)
        .json(&body(
            "sha256:0000000000000000000000000000000000000000000000000000000000000000",
        ))
        .send()
        .await
        .expect("mismatch request");
    assert_eq!(response.status().as_u16(), 400, "the mismatch refuses");

    // The missing parent refuses with its name.
    let response = client
        .post(format!("{base}/v1/derivations"))
        .header(PRINCIPAL_HEADER, &human_id)
        .json(&json!({
            "parent_snapshot_id": "snp_missing",
            "derived_kind": "chunk",
            "derived_digest": chunk_digest,
            "content": content,
        }))
        .send()
        .await
        .expect("missing-parent request");
    assert_eq!(
        response.status().as_u16(),
        400,
        "the missing parent refuses"
    );
    let refused: Value = response.json().await.unwrap();
    assert!(
        refused["message"].as_str().unwrap().contains("parent"),
        "{refused}"
    );
}

/// The claim-evidence graph + the citation validation (PHASE-4.6.3): the
/// five assessments link claims to snapshots, and the citation is
/// VALIDATED — the excerpt must appear in the snapshot's raw bytes (the
/// fake excerpt is refused; citation existence alone never satisfies an
/// evidence gate).
#[tokio::test]
async fn the_claim_assessments_validate_the_citation() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, human) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "clm-human" }),
    )
    .await;
    assert_eq!(status, 200, "the human enrolls: {human}");
    let human_id = human["principal_id"].as_str().unwrap().to_string();

    // The evidence: a reference + a snapshot whose bytes are KNOWN.
    let (status, submitted): (u16, Value) = {
        let response = client
            .post(format!("{base}/v1/resources"))
            .header(PRINCIPAL_HEADER, &human_id)
            .json(&json!({
                "original_locator": "https://example.org/claim-evidence",
                "scheme": "https",
            }))
            .send()
            .await
            .expect("submit request");
        (
            response.status().as_u16(),
            response.json().await.expect("submit json"),
        )
    };
    assert_eq!(status, 200, "the reference submits: {submitted}");
    let reference_id = submitted["resource_id"].as_str().unwrap().to_string();
    let bytes = b"the evidence says the budget is exhausted";
    let raw_digest = reasonbraid_server::fetcher::digest_sha256_hex(bytes);
    let response = client
        .post(format!("{base}/v1/snapshots"))
        .header(PRINCIPAL_HEADER, &human_id)
        .json(&json!({
            "reference_id": reference_id,
            "original_locator": "https://example.org/claim-evidence",
            "final_locator": "https://example.org/claim-evidence",
            "resolver_id": "r0-https-fetcher",
            "resolver_version": "0.1.0",
            "raw_digest": raw_digest,
            "byte_length": bytes.len(),
            "media_type": "text/plain",
            "bytes_base64": "dGhlIGV2aWRlbmNlIHNheXMgdGhlIGJ1ZGdldCBpcyBleGhhdXN0ZWQ=",
        }))
        .send()
        .await
        .expect("snapshot request");
    let snapshot_outcome: Value = response.json().await.unwrap();
    let snapshot_id = snapshot_outcome["snapshot_id"]
        .as_str()
        .unwrap()
        .to_string();

    let body = |excerpt: &str| {
        json!({
            "claim_id": "clm_budget",
            "snapshot_id": snapshot_id,
            "assessment": "supports",
            "author": human_id,
            "excerpt": excerpt,
            "rationale": "the evidence states the exhaustion",
            "source_authority": "primary",
            "freshness": "current",
            "independence": "independent",
            "uncertainty": "low",
        })
    };

    // The TRUE excerpt (present in the bytes) is accepted.
    let response = client
        .post(format!("{base}/v1/assessments"))
        .header(PRINCIPAL_HEADER, &human_id)
        .json(&body("the budget is exhausted"))
        .send()
        .await
        .expect("assessment request");
    assert_eq!(response.status().as_u16(), 200, "the true excerpt accepts");
    let outcome: Value = response.json().await.unwrap();
    let assessment_id = outcome["assessment_id"].as_str().unwrap().to_string();

    // The replay: the same claim + snapshot + kind + author → the same id.
    let response = client
        .post(format!("{base}/v1/assessments"))
        .header(PRINCIPAL_HEADER, &human_id)
        .json(&body("the budget is exhausted"))
        .send()
        .await
        .expect("replay request");
    let replay: Value = response.json().await.unwrap();
    assert_eq!(replay["assessment_id"], json!(assessment_id));

    // The FAKE excerpt is refused — the citation must point at the real
    // bytes (citation existence alone never satisfies an evidence gate).
    let response = client
        .post(format!("{base}/v1/assessments"))
        .header(PRINCIPAL_HEADER, &human_id)
        .json(&body("the budget is INCREASED"))
        .send()
        .await
        .expect("fake-excerpt request");
    assert_eq!(response.status().as_u16(), 400, "the fake excerpt refuses");
    let refused: Value = response.json().await.unwrap();
    assert!(
        refused["message"].as_str().unwrap().contains("excerpt"),
        "{refused}"
    );

    // The unknown assessment kind refuses with its name.
    let response = client
        .post(format!("{base}/v1/assessments"))
        .header(PRINCIPAL_HEADER, &human_id)
        .json(&json!({
            "claim_id": "clm_budget",
            "snapshot_id": snapshot_id,
            "assessment": "proves",
            "author": human_id,
            "excerpt": "the budget is exhausted",
            "rationale": "nope",
        }))
        .send()
        .await
        .expect("unknown-kind request");
    assert_eq!(response.status().as_u16(), 400, "the unknown kind refuses");

    // The read surfaces: the snapshot's + the claim's assessments.
    let (status, snapshot_side) = get(
        &client,
        &base,
        &format!("/v1/snapshots/{snapshot_id}/assessments"),
        &human_id,
    )
    .await;
    assert_eq!(
        status, 200,
        "the snapshot assessments read: {snapshot_side}"
    );
    let snapshot_side = snapshot_side.as_array().expect("the array");
    assert_eq!(snapshot_side.len(), 1, "{snapshot_side:?}");
    assert_eq!(snapshot_side[0]["assessment"], json!("supports"));

    let (status, claim_side) = get(
        &client,
        &base,
        "/v1/claims/clm_budget/assessments",
        &human_id,
    )
    .await;
    assert_eq!(status, 200, "the claim assessments read: {claim_side}");
    assert_eq!(
        claim_side.as_array().expect("the array").len(),
        1,
        "{claim_side:?}"
    );
}

/// The test's own base64 (the API's decoder is under test, not this one).
mod util {
    pub fn base64(bytes: &[u8]) -> String {
        const TABLE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut out = String::new();
        for chunk in bytes.chunks(3) {
            let b0 = chunk[0] as u32;
            let b1 = chunk.get(1).copied().map(|b| b as u32).unwrap_or(0);
            let b2 = chunk.get(2).copied().map(|b| b as u32).unwrap_or(0);
            let triple = (b0 << 16) | (b1 << 8) | b2;
            out.push(TABLE[(triple >> 18) as usize & 63] as char);
            out.push(TABLE[(triple >> 12) as usize & 63] as char);
            out.push(if chunk.len() > 1 {
                TABLE[(triple >> 6) as usize & 63] as char
            } else {
                '='
            });
            out.push(if chunk.len() > 2 {
                TABLE[triple as usize & 63] as char
            } else {
                '='
            });
        }
        out
    }
}

/// The license/retention + the freshness (PHASE-4.6.4): the license +
/// the freshness horizon ride the snapshot, the retention enforcement
/// TOMBSTONES the expired classes (the `at` override drives the test),
/// the staleness surface lists the passed horizons, and the replay
/// REFRESHES the freshness record (the re-fetch policy).
#[tokio::test]
async fn the_retention_enforcement_and_the_freshness_surface() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, human) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "rtn-human" }),
    )
    .await;
    assert_eq!(status, 200, "the human enrolls: {human}");
    let human_id = human["principal_id"].as_str().unwrap().to_string();

    let (status, submitted): (u16, Value) = {
        let response = client
            .post(format!("{base}/v1/resources"))
            .header(PRINCIPAL_HEADER, &human_id)
            .json(&json!({
                "original_locator": "https://example.org/retention",
                "scheme": "https",
            }))
            .send()
            .await
            .expect("submit request");
        (
            response.status().as_u16(),
            response.json().await.expect("submit json"),
        )
    };
    assert_eq!(status, 200, "the reference submits: {submitted}");
    let reference_id = submitted["resource_id"].as_str().unwrap().to_string();
    let submit_snapshot =
        |bytes: &'static [u8], retention_class: String, fresh_until: Option<String>| {
            let client = client.clone();
            let base = base.clone();
            let human_id = human_id.clone();
            let reference_id = reference_id.clone();
            async move {
                let response = client
                    .post(format!("{base}/v1/snapshots"))
                    .header(PRINCIPAL_HEADER, &human_id)
                    .json(&json!({
                        "reference_id": reference_id,
                        "original_locator": "https://example.org/retention",
                        "final_locator": "https://example.org/retention",
                        "resolver_id": "r0-https-fetcher",
                        "resolver_version": "0.1.0",
                        "raw_digest": reasonbraid_server::fetcher::digest_sha256_hex(bytes),
                        "byte_length": bytes.len(),
                        "media_type": "text/plain",
                        "retention_class": retention_class,
                        "license": "MIT OR Apache-2.0",
                        "fresh_until": fresh_until,
                        "bytes_base64": util::base64(bytes),
                    }))
                    .send()
                    .await
                    .expect("snapshot request");
                response.json::<Value>().await.unwrap()
            }
        };

    // The temporary snapshot with a PASSED horizon + the fresh one with a
    // FUTURE horizon — the dates are RELATIVE to now (the `.4.3.3`
    // clock-crossing repair: the hardcoded horizons went stale at the
    // midnight boundary and the freshness list grew an extra row).
    let horizon = |offset_days: i64| {
        let day = chrono::Utc::now().date_naive() + chrono::Duration::days(offset_days);
        format!("{}T00:00:00Z", day.format("%Y-%m-%d"))
    };
    let temporary = submit_snapshot(
        b"the temporary bytes",
        "temporary".to_owned(),
        Some(horizon(-1)),
    )
    .await;
    let temporary_id = temporary["snapshot_id"].as_str().unwrap().to_string();
    // The fresh one (distinct bytes — a distinct snapshot).
    let fresh = submit_snapshot(
        b"the fresh standard bytes",
        "standard".to_owned(),
        Some(horizon(1)),
    )
    .await;
    let fresh_id = fresh["snapshot_id"].as_str().unwrap().to_string();

    // The staleness surface lists ONLY the passed horizon.
    let (status, stale) = get(&client, &base, "/v1/snapshots/stale", &human_id).await;
    assert_eq!(status, 200, "the staleness reads: {stale}");
    let stale = stale.as_array().expect("the array");
    assert_eq!(stale.len(), 1, "{stale:?}");
    assert_eq!(stale[0]["snapshot_id"], json!(temporary_id));

    // The retention enforcement: the `at` override tombstones the
    // temporary class (the TTL = 1 day) — the audit/standard stay.
    let response = client
        .post(format!("{base}/v1/snapshots/expire-due"))
        .header(PRINCIPAL_HEADER, &human_id)
        .json(&json!({ "at": "2026-09-10T00:00:00Z" }))
        .send()
        .await
        .expect("expire request");
    let outcome: Value = response.json().await.unwrap();
    assert!(outcome["tombstoned"].as_u64().unwrap() >= 1, "{outcome}");

    let (status, stored) = get(
        &client,
        &base,
        &format!("/v1/snapshots/{temporary_id}"),
        &human_id,
    )
    .await;
    assert_eq!(
        status, 200,
        "the tombstoned snapshot stays readable: {stored}"
    );
    assert!(stored["deleted_at"].is_string(), "{stored}");
    assert_eq!(stored["deletion_reason"], json!("the retention expired"));
    // The license metadata rides the row.
    assert_eq!(stored["license"], json!("MIT OR Apache-2.0"));

    // The fresh standard snapshot is NOT tombstoned (30-day TTL).
    let (status, stored) = get(
        &client,
        &base,
        &format!("/v1/snapshots/{fresh_id}"),
        &human_id,
    )
    .await;
    assert_eq!(status, 200, "the fresh snapshot reads: {stored}");
    assert_eq!(stored["deleted_at"], Value::Null);

    // The re-fetch policy: the replay REFRESHES the freshness record.
    let replayed = submit_snapshot(b"the fresh standard bytes", "standard".to_owned(), None).await;
    assert_eq!(replayed["replay"], json!(true));
    let (status, stored) = get(
        &client,
        &base,
        &format!("/v1/snapshots/{fresh_id}"),
        &human_id,
    )
    .await;
    assert_eq!(status, 200, "the refreshed snapshot reads: {stored}");
    assert!(stored["refreshed_at"].is_string(), "{stored}");
}

/// The G4 hostile-content suite (PHASE-4.7.1): ONE gate-citable test
/// assembling the hostile scenarios end-to-end — every refusal NAMES its
/// reason (the unsupported, denied, mutable, and non-reproducible paths
/// fail explicitly rather than becoming fabricated evidence).
#[tokio::test]
async fn the_g4_hostile_suite_names_every_refusal() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, human) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "g4-human" }),
    )
    .await;
    assert_eq!(status, 200, "the human enrolls: {human}");
    let human_id = human["principal_id"].as_str().unwrap().to_string();

    let submit = |body: Value| {
        let client = client.clone();
        let base = base.clone();
        let human_id = human_id.clone();
        async move {
            let response = client
                .post(format!("{base}/v1/resources"))
                .header(PRINCIPAL_HEADER, &human_id)
                .json(&body)
                .send()
                .await
                .expect("submit request");
            let parsed: Value = response.json().await.expect("submit json");
            parsed["resource_id"].as_str().unwrap().to_string()
        }
    };
    let resolve = |resource_id: &str| {
        let client = client.clone();
        let base = base.clone();
        let human_id = human_id.clone();
        let resource_id = resource_id.to_owned();
        async move {
            let response = client
                .post(format!("{base}/v1/resources/{resource_id}/resolve"))
                .header(PRINCIPAL_HEADER, &human_id)
                .json(&json!({ "required_sandbox": "none", "required_egress": "listed" }))
                .send()
                .await
                .expect("resolve request");
            response.json::<Value>().await.unwrap()
        }
    };

    // 1. The loopback literal refuses with its class named.
    let id = submit(json!({
        "original_locator": "https://127.0.0.1/hostile",
        "scheme": "https",
    }))
    .await;
    let resolved = resolve(&id).await;
    assert_eq!(
        resolved["acquisition_error"]["kind"],
        json!("destination_refused"),
        "{resolved}"
    );
    assert!(
        resolved["acquisition_error"]["message"]
            .as_str()
            .unwrap()
            .contains("loopback"),
        "{resolved}"
    );

    // 2. The private literal.
    let id = submit(json!({
        "original_locator": "https://10.0.0.1/hostile",
        "scheme": "https",
    }))
    .await;
    let resolved = resolve(&id).await;
    assert!(
        resolved["acquisition_error"]["message"]
            .as_str()
            .unwrap()
            .contains("private"),
        "{resolved}"
    );

    // 3. The mapped-form loopback (the `.2.1` re-classification).
    let id = submit(json!({
        "original_locator": "https://[::ffff:127.0.0.1]/hostile",
        "scheme": "https",
    }))
    .await;
    let resolved = resolve(&id).await;
    assert!(
        resolved["acquisition_error"]["message"]
            .as_str()
            .unwrap()
            .contains("loopback"),
        "{resolved}"
    );

    // 4. The userinfo URL refuses with its name.
    let id = submit(json!({
        "original_locator": "https://user@127.0.0.1/hostile",
        "scheme": "https",
    }))
    .await;
    let resolved = resolve(&id).await;
    assert_eq!(
        resolved["acquisition_error"]["kind"],
        json!("userinfo_forbidden"),
        "{resolved}"
    );

    // 5. The unsupported scheme is the explicit unresolvable-now.
    let id = submit(json!({
        "original_locator": "ftp://example.org/hostile",
        "scheme": "ftp",
    }))
    .await;
    let resolved = resolve(&id).await;
    assert_eq!(resolved["unresolvable_now"], json!(true), "{resolved}");

    // 6. The fake digest: the snapshot submit refuses (the
    // content-addressing is verified, not trusted).
    let id = submit(json!({
        "original_locator": "https://example.org/g4-evidence",
        "scheme": "https",
    }))
    .await;
    let response = client
        .post(format!("{base}/v1/snapshots"))
        .header(PRINCIPAL_HEADER, &human_id)
        .json(&json!({
            "reference_id": id,
            "original_locator": "https://example.org/g4-evidence",
            "final_locator": "https://example.org/g4-evidence",
            "resolver_id": "r0-https-fetcher",
            "resolver_version": "0.1.0",
            "raw_digest": "sha256:0000000000000000000000000000000000000000000000000000000000000000",
            "byte_length": 10,
            "media_type": "text/plain",
            "bytes_base64": "aG9zdGlsZQ==",
        }))
        .send()
        .await
        .expect("fake-digest request");
    assert_eq!(response.status().as_u16(), 400, "the fake digest refuses");
    let refused: Value = response.json().await.unwrap();
    assert!(
        refused["message"].as_str().unwrap().contains("hash"),
        "{refused}"
    );

    // 7. The unknown assessment kind refuses with its name.
    let response = client
        .post(format!("{base}/v1/assessments"))
        .header(PRINCIPAL_HEADER, &human_id)
        .json(&json!({
            "claim_id": "clm_g4",
            "snapshot_id": "snp_missing",
            "assessment": "proves",
            "author": human_id,
            "excerpt": "x",
            "rationale": "nope",
        }))
        .send()
        .await
        .expect("unknown-kind request");
    assert_eq!(response.status().as_u16(), 400, "the unknown kind refuses");
    let refused: Value = response.json().await.unwrap();
    assert!(
        refused["message"].as_str().unwrap().contains("vocabulary"),
        "{refused}"
    );

    // 8. The unknown-field boundary: the submission's deny-unknown
    // refuses the forged field.
    let response = client
        .post(format!("{base}/v1/resources"))
        .header(PRINCIPAL_HEADER, &human_id)
        .json(&json!({
            "original_locator": "https://example.org/g4",
            "scheme": "https",
            "forged_field": true,
        }))
        .send()
        .await
        .expect("forged-field request");
    assert_eq!(response.status().as_u16(), 422, "the forged field refuses");
}

#[tokio::test]
async fn the_structured_claims_and_objections_ride_the_wire() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, human) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "sc-human" }),
    )
    .await;
    assert_eq!(status, 200, "the human enrolls: {human}");
    let human_id = human["principal_id"].as_str().unwrap().to_string();
    let tenant_id = human["tenant_id"].as_str().unwrap().to_string();

    let created = post(
        &client,
        &base,
        "/v1/threads",
        &human_id,
        &json!({
            "protocol_version": reasonbraid_core::PROTOCOL_VERSION,
            "operation": "thread.create",
            "request_id": reasonbraid_core::RequestId::new().to_string(),
            "idempotency_key": "sc-create",
            "body": {
                "tenant_id": tenant_id,
                "subject": "sc",
                "objective": "probe",
            },
            "client_context": {},
        }),
    )
    .await;
    assert_eq!(created.0, 200, "the create succeeds: {created:?}");
    let thread_id = created.1["thread_id"].as_str().unwrap().to_string();

    let command = |key: &'static str, operation: &'static str, body: Value| {
        let client = client.clone();
        let base = base.clone();
        let human_id = human_id.clone();
        let thread_id = thread_id.clone();
        async move {
            let response = client
                .post(format!("{base}/v1/threads/{thread_id}/commands"))
                .header(PRINCIPAL_HEADER, &human_id)
                .json(&json!({
                    "protocol_version": reasonbraid_core::PROTOCOL_VERSION,
                    "operation": operation,
                    "request_id": reasonbraid_core::RequestId::new().to_string(),
                    "idempotency_key": key,
                    "body": body,
                    "client_context": {},
                }))
                .send()
                .await
                .expect("command request");
            let status = response.status().as_u16();
            let text = response.text().await.expect("command body");
            let value = serde_json::from_str(&text).unwrap_or_else(|_| json!({ "raw": text }));
            (status, value)
        }
    };

    // 1. The contribute with two structured claims: the server computes the
    // digests (the client supplies content only).
    let (status, contributed) = command(
        "sc-contribute",
        "thread.contribute",
        json!({
            "tenant_id": tenant_id,
            "content": "the claims ride the contribution",
            "kind": "claim",
            "claims": [
                { "content": "the registry ships" },
                { "content": "the invariant is the boundary" },
            ],
        }),
    )
    .await;
    assert_eq!(
        status, 200,
        "the structured contribute succeeds: {contributed}"
    );
    let contribution_event = contributed["event_id"].as_str().unwrap().to_string();

    // 2. The projection carries the structure (the ADR-029 counter).
    let (status, state) = get(
        &client,
        &base,
        &format!("/v1/threads/{thread_id}?tenant_id={tenant_id}"),
        &human_id,
    )
    .await;
    assert_eq!(status, 200, "the thread reads: {state}");
    assert_eq!(state["state"]["structured_claims"], json!(2));

    // 3. The event body carries the records with the SERVER-computed
    // digests (re-derived here from the claim content).
    let (status, timeline) = get(
        &client,
        &base,
        &format!("/v1/threads/{thread_id}/events?tenant_id={tenant_id}"),
        &human_id,
    )
    .await;
    assert_eq!(status, 200, "the timeline reads: {timeline}");
    let contribution = timeline["events"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["event_id"] == json!(contribution_event))
        .cloned()
        .expect("the contribute event is in the timeline");
    let claims = contribution["body"]["claims"].as_array().unwrap();
    assert_eq!(claims.len(), 2, "two claim records: {claims:?}");
    let digest_one =
        reasonbraid_server::fetcher::digest_sha256_hex("the registry ships".as_bytes());
    let digest_two =
        reasonbraid_server::fetcher::digest_sha256_hex("the invariant is the boundary".as_bytes());
    assert_eq!(claims[0]["digest"], json!(digest_one));
    assert_eq!(claims[1]["digest"], json!(digest_two));
    assert_ne!(digest_one, digest_two, "the digests differ");

    // 4. The structured objection: the challenge names ONE claim of the
    // target contribution and rides the event body.
    let (status, challenged) = command(
        "sc-challenge",
        "thread.challenge",
        json!({
            "tenant_id": tenant_id,
            "target_event_id": contribution_event,
            "claim_digest": digest_one,
            "content": "which registry, exactly?",
        }),
    )
    .await;
    assert_eq!(
        status, 200,
        "the structured challenge succeeds: {challenged}"
    );
    let challenge_event = challenged["event_id"].as_str().unwrap().to_string();
    let (_, timeline) = get(
        &client,
        &base,
        &format!("/v1/threads/{thread_id}/events?tenant_id={tenant_id}"),
        &human_id,
    )
    .await;
    let challenge = timeline["events"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["event_id"] == json!(challenge_event))
        .cloned()
        .expect("the challenge event is in the timeline");
    assert_eq!(challenge["body"]["claim_digest"], json!(digest_one));

    // 5. A digest that is NOT a claim of the target is the typed refusal.
    let foreign = reasonbraid_server::fetcher::digest_sha256_hex("not a claim".as_bytes());
    let (status, refused) = command(
        "sc-challenge-foreign",
        "thread.challenge",
        json!({
            "tenant_id": tenant_id,
            "target_event_id": contribution_event,
            "claim_digest": foreign,
            "content": "nope",
        }),
    )
    .await;
    assert_eq!(status, 400, "the foreign digest refuses: {refused}");
    assert!(
        refused["message"].as_str().unwrap().contains("not a claim"),
        "{refused}"
    );

    // 6. A claimless contribution refuses any targeted digest; and a
    // non-claim contribution refuses the claims field.
    let (status, plain) = command(
        "sc-plain",
        "thread.contribute",
        json!({
            "tenant_id": tenant_id,
            "content": "plain position",
        }),
    )
    .await;
    assert_eq!(status, 200, "the plain contribute succeeds: {plain}");
    let plain_event = plain["event_id"].as_str().unwrap().to_string();
    let (status, refused) = command(
        "sc-challenge-claimless",
        "thread.challenge",
        json!({
            "tenant_id": tenant_id,
            "target_event_id": plain_event,
            "claim_digest": digest_one,
            "content": "nope",
        }),
    )
    .await;
    assert_eq!(status, 400, "the claimless target refuses: {refused}");
    let (status, refused) = command(
        "sc-position-claims",
        "thread.contribute",
        json!({
            "tenant_id": tenant_id,
            "content": "positions cannot carry claims",
            "kind": "position",
            "claims": [ { "content": "x" } ],
        }),
    )
    .await;
    assert_eq!(status, 400, "the position+claims refuses: {refused}");
    assert!(
        refused["message"].as_str().unwrap().contains("claim"),
        "{refused}"
    );

    // 7. The revision answers the objection — the register stays honest.
    let (status, revised) = command(
        "sc-revise",
        "thread.revise",
        json!({
            "tenant_id": tenant_id,
            "target_event_id": challenge_event,
            "content": "the profile registry (migration 0032).",
        }),
    )
    .await;
    assert_eq!(status, 200, "the revision succeeds: {revised}");
    let (status, state) = get(
        &client,
        &base,
        &format!("/v1/threads/{thread_id}?tenant_id={tenant_id}"),
        &human_id,
    )
    .await;
    assert_eq!(status, 200, "the thread reads: {state}");
    assert_eq!(
        state["state"]["open_challenges"],
        json!(0),
        "the revision closed the register: {state}"
    );
}

#[tokio::test]
async fn the_blind_contributions_commit_at_the_round_advance() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    // A bootstraps the tenant; B enrolls as a ROLE into the SAME tenant with
    // thread_inspect + thread_contribute (the role's default lacks inspect —
    // the actions name it explicitly), then A invites B and B accepts: B is a
    // participant (challenges) AND a reader (the blind rule's non-author).
    let (status, human_a) =
        enroll(&client, &base, json!({ "kind": "human", "name": "bl-a" })).await;
    assert_eq!(status, 200, "A enrolls: {human_a}");
    let a_id = human_a["principal_id"].as_str().unwrap().to_string();
    let tenant_id = human_a["tenant_id"].as_str().unwrap().to_string();
    let (status, human_b) = enroll(
        &client,
        &base,
        json!({
            "kind": "role",
            "name": "bl-b",
            "tenant_id": tenant_id,
            "actions": ["thread_contribute", "thread_inspect", "thread_invitation_respond"],
        }),
    )
    .await;
    assert_eq!(status, 200, "B enrolls: {human_b}");
    let b_id = human_b["principal_id"].as_str().unwrap().to_string();

    // The independent_panel profile seats step 0 = blind_solicit (the .1.3
    // proof) — a contribution posted now is blind.
    let (status, created) = post(
        &client,
        &base,
        "/v1/threads",
        &a_id,
        &json!({
            "protocol_version": reasonbraid_core::PROTOCOL_VERSION,
            "operation": "thread.create",
            "request_id": reasonbraid_core::RequestId::new().to_string(),
            "idempotency_key": "bl-create",
            "body": {
                "tenant_id": tenant_id,
                "subject": "bl",
                "objective": "probe",
                "workflow_profile": "independent_panel",
            },
            "client_context": {},
        }),
    )
    .await;
    assert_eq!(status, 200, "the create succeeds: {created}");
    let thread_id = created["thread_id"].as_str().unwrap().to_string();

    // A invites B; B accepts — B becomes a participant (challenge-eligible).
    let invite =
        |key: &'static str, operation: &'static str, principal: &'static str, body: Value| {
            let client = client.clone();
            let base = base.clone();
            let thread_id = thread_id.clone();
            let a_id = a_id.clone();
            let b_id = b_id.clone();
            async move {
                let principal = if principal == "A" { a_id } else { b_id };
                let response = client
                    .post(format!("{base}/v1/threads/{thread_id}/commands"))
                    .header(PRINCIPAL_HEADER, &principal)
                    .json(&json!({
                        "protocol_version": reasonbraid_core::PROTOCOL_VERSION,
                        "operation": operation,
                        "request_id": reasonbraid_core::RequestId::new().to_string(),
                        "idempotency_key": key,
                        "body": body,
                        "client_context": {},
                    }))
                    .send()
                    .await
                    .expect("command request");
                let status = response.status().as_u16();
                let text = response.text().await.expect("command body");
                let value = serde_json::from_str(&text).unwrap_or_else(|_| json!({ "raw": text }));
                (status, value)
            }
        };
    let (status, invited) = invite(
        "bl-invite",
        "thread.invite",
        "A",
        json!({ "tenant_id": tenant_id, "agent_role": b_id }),
    )
    .await;
    assert_eq!(status, 200, "the invite succeeds: {invited}");
    let (status, accepted) = invite(
        "bl-accept",
        "thread.accept_invitation",
        "B",
        json!({ "tenant_id": tenant_id }),
    )
    .await;
    assert_eq!(status, 200, "the accept succeeds: {accepted}");

    let command =
        |key: &'static str, operation: &'static str, principal: &'static str, body: Value| {
            let client = client.clone();
            let base = base.clone();
            let thread_id = thread_id.clone();
            let a_id = a_id.clone();
            let b_id = b_id.clone();
            async move {
                let principal = if principal == "A" { a_id } else { b_id };
                let response = client
                    .post(format!("{base}/v1/threads/{thread_id}/commands"))
                    .header(PRINCIPAL_HEADER, &principal)
                    .json(&json!({
                        "protocol_version": reasonbraid_core::PROTOCOL_VERSION,
                        "operation": operation,
                        "request_id": reasonbraid_core::RequestId::new().to_string(),
                        "idempotency_key": key,
                        "body": body,
                        "client_context": {},
                    }))
                    .send()
                    .await
                    .expect("command request");
                let status = response.status().as_u16();
                let text = response.text().await.expect("command body");
                let value = serde_json::from_str(&text).unwrap_or_else(|_| json!({ "raw": text }));
                (status, value)
            }
        };
    let read_events = |principal: &'static str| {
        let client = client.clone();
        let base = base.clone();
        let thread_id = thread_id.clone();
        let tenant_id = tenant_id.clone();
        let a_id = a_id.clone();
        let b_id = b_id.clone();
        async move {
            let principal = if principal == "A" { a_id } else { b_id };
            get(
                &client,
                &base,
                &format!("/v1/threads/{thread_id}/events?tenant_id={tenant_id}"),
                &principal,
            )
            .await
        }
    };

    // 1. A contributes during the blind phase — the event carries the marker.
    let content = "A's blind initial position";
    let (status, contributed) = command(
        "bl-contribute",
        "thread.contribute",
        "A",
        json!({
            "tenant_id": tenant_id,
            "content": content,
        }),
    )
    .await;
    assert_eq!(status, 200, "the blind contribute succeeds: {contributed}");

    // 2. The AUTHOR reads the full body; the marker rides it.
    let (status, timeline_a) = read_events("A").await;
    assert_eq!(status, 200, "A reads the timeline: {timeline_a}");
    let contribution = timeline_a["events"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["event_type"] == json!("thread.contribution_submitted"))
        .cloned()
        .expect("the contribute event is in the timeline");
    assert_eq!(contribution["body"]["blind"], json!(true));
    assert_eq!(contribution["body"]["content"], json!(content));

    // 3. The NON-AUTHOR reader sees the digest + the marker, NOT the content.
    let (status, timeline_b) = read_events("B").await;
    assert_eq!(status, 200, "B reads the timeline: {timeline_b}");
    let redacted = timeline_b["events"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["event_type"] == json!("thread.contribution_submitted"))
        .cloned()
        .expect("the contribute event is in the timeline");
    assert_eq!(redacted["body"]["blind"], json!(true));
    assert_eq!(redacted["body"]["blind_until"], json!("round_advance"));
    assert_eq!(
        redacted["body"]["content_digest"],
        json!(reasonbraid_server::fetcher::digest_sha256_hex(
            content.as_bytes()
        ))
    );
    assert!(redacted["body"].get("content").is_none(), "{redacted}");
    assert!(redacted["body"].get("claims").is_none(), "{redacted}");

    // 4. A non-author cannot challenge a still-blind contribution.
    let (status, refused) = command(
        "bl-challenge-blind",
        "thread.challenge",
        "B",
        json!({
            "tenant_id": tenant_id,
            "target_event_id": contributed["event_id"],
            "content": "which position, exactly?",
        }),
    )
    .await;
    assert_eq!(status, 400, "the blind target refuses: {refused}");
    assert!(
        refused["message"].as_str().unwrap().contains("blind until"),
        "{refused}"
    );

    // 5. The round advance IS the commitment point: the step moves past
    // blind_solicit and the event records it.
    let (status, advanced) = command(
        "bl-advance",
        "thread.advance_round",
        "A",
        json!({ "tenant_id": tenant_id }),
    )
    .await;
    assert_eq!(status, 200, "the round advances: {advanced}");
    let (status, timeline_a) = read_events("A").await;
    assert_eq!(status, 200, "A reads the timeline: {timeline_a}");
    let advance_event = timeline_a["events"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["event_type"] == json!("thread.round_advanced"))
        .cloned()
        .expect("the round-advanced event is in the timeline");
    assert_eq!(advance_event["body"]["blind_committed"], json!(true));
    let (status, state) = get(
        &client,
        &base,
        &format!("/v1/threads/{thread_id}?tenant_id={tenant_id}"),
        &a_id,
    )
    .await;
    assert_eq!(status, 200, "the thread reads: {state}");
    assert_eq!(state["state"]["workflow_step"], json!(1));

    // 6. Post-commitment, the non-author reads the FULL body and may
    // challenge it.
    let (status, timeline_b) = read_events("B").await;
    assert_eq!(status, 200, "B reads the timeline: {timeline_b}");
    let revealed = timeline_b["events"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["event_type"] == json!("thread.contribution_submitted"))
        .cloned()
        .expect("the contribute event is in the timeline");
    assert_eq!(revealed["body"]["content"], json!(content));
    let (status, challenged) = command(
        "bl-challenge-open",
        "thread.challenge",
        "B",
        json!({
            "tenant_id": tenant_id,
            "target_event_id": contributed["event_id"],
            "content": "which position, exactly?",
        }),
    )
    .await;
    assert_eq!(
        status, 200,
        "the committed target accepts the challenge: {challenged}"
    );
}

#[tokio::test]
async fn the_twelve_terminals_and_the_minority_report_ride_the_close() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, human) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "tw-human" }),
    )
    .await;
    assert_eq!(status, 200, "the human enrolls: {human}");
    let human_id = human["principal_id"].as_str().unwrap().to_string();
    let tenant_id = human["tenant_id"].as_str().unwrap().to_string();

    // The §13.4 table: canonical terminal → the decision family (closed) or
    // the failure family (inconclusive).
    let terminals: &[(&str, bool)] = &[
        ("accepted_unanimously", true),
        ("accepted_with_recorded_objections", true),
        ("accepted_by_rule", true),
        ("advisory_answer_only", true),
        ("deadlocked", false),
        ("no_quorum", false),
        ("insufficient_evidence", false),
        ("budget_exhausted", false),
        ("expired", false),
        ("cancelled", false),
        ("human_decision_required", false),
        ("unsafe_to_continue", false),
    ];

    for (i, (outcome, decided_family)) in terminals.iter().enumerate() {
        let key = format!("tw-create-{i}");
        let (status, created) = post(
            &client,
            &base,
            "/v1/threads",
            &human_id,
            &json!({
                "protocol_version": reasonbraid_core::PROTOCOL_VERSION,
                "operation": "thread.create",
                "request_id": reasonbraid_core::RequestId::new().to_string(),
                "idempotency_key": key,
                "body": {
                    "tenant_id": tenant_id,
                    "subject": format!("tw-{i}"),
                    "objective": "probe",
                },
                "client_context": {},
            }),
        )
        .await;
        assert_eq!(status, 200, "the create succeeds: {created}");
        let thread_id = created["thread_id"].as_str().unwrap().to_string();

        let response = client
            .post(format!("{base}/v1/threads/{thread_id}/commands"))
            .header(PRINCIPAL_HEADER, &human_id)
            .json(&json!({
                "protocol_version": reasonbraid_core::PROTOCOL_VERSION,
                "operation": "thread.close",
                "request_id": reasonbraid_core::RequestId::new().to_string(),
                "idempotency_key": format!("tw-close-{i}"),
                "body": {
                    "tenant_id": tenant_id,
                    "reason": "the terminal probe",
                    "outcome": outcome,
                },
                "client_context": {},
            }))
            .send()
            .await
            .expect("close request");
        assert_eq!(response.status().as_u16(), 200, "close {outcome}");
        let closed: Value = response.json().await.unwrap();
        assert_eq!(
            closed["thread_state"],
            json!(if *decided_family {
                "closed"
            } else {
                "inconclusive"
            }),
            "{outcome}"
        );

        let (status, state) = get(
            &client,
            &base,
            &format!("/v1/threads/{thread_id}?tenant_id={tenant_id}"),
            &human_id,
        )
        .await;
        assert_eq!(status, 200, "the thread reads: {state}");
        assert_eq!(
            state["state"]["close_outcome"],
            json!(outcome),
            "the canonical terminal persists on the projection"
        );
    }

    // The legacy aliases stay accepted on the wire but NEVER persist — the
    // canonical names do.
    let legacy: &[(&str, &str)] = &[
        ("decided", "accepted_by_rule"),
        ("inconclusive", "deadlocked"),
    ];
    for (i, (wire, canonical)) in legacy.iter().enumerate() {
        let (_status, created) = post(
            &client,
            &base,
            "/v1/threads",
            &human_id,
            &json!({
                "protocol_version": reasonbraid_core::PROTOCOL_VERSION,
                "operation": "thread.create",
                "request_id": reasonbraid_core::RequestId::new().to_string(),
                "idempotency_key": format!("tw-legacy-{i}"),
                "body": {
                    "tenant_id": tenant_id,
                    "subject": format!("tw-legacy-{i}"),
                    "objective": "probe",
                },
                "client_context": {},
            }),
        )
        .await;
        let thread_id = created["thread_id"].as_str().unwrap().to_string();
        let response = client
            .post(format!("{base}/v1/threads/{thread_id}/commands"))
            .header(PRINCIPAL_HEADER, &human_id)
            .json(&json!({
                "protocol_version": reasonbraid_core::PROTOCOL_VERSION,
                "operation": "thread.close",
                "request_id": reasonbraid_core::RequestId::new().to_string(),
                "idempotency_key": format!("tw-legacy-close-{i}"),
                "body": {
                    "tenant_id": tenant_id,
                    "reason": "the alias probe",
                    "outcome": wire,
                },
                "client_context": {},
            }))
            .send()
            .await
            .expect("close request");
        assert_eq!(response.status().as_u16(), 200, "the alias {wire} accepts");
        let closed: Value = response.json().await.unwrap();
        let (_, timeline) = get(
            &client,
            &base,
            &format!("/v1/threads/{thread_id}/events?tenant_id={tenant_id}"),
            &human_id,
        )
        .await;
        let close_event = timeline["events"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["event_type"] == json!("thread.closed"))
            .cloned()
            .expect("the close event exists");
        assert_eq!(
            close_event["body"]["outcome"],
            json!(canonical),
            "the alias never persists"
        );
        assert!(closed["thread_id"].is_string());
    }

    // The family rule: a decision terminal refuses the unresolved register; a
    // failure terminal accepts (and carries) it.
    let (_status, created) = post(
        &client,
        &base,
        "/v1/threads",
        &human_id,
        &json!({
            "protocol_version": reasonbraid_core::PROTOCOL_VERSION,
            "operation": "thread.create",
            "request_id": reasonbraid_core::RequestId::new().to_string(),
            "idempotency_key": "tw-refusal",
            "body": {
                "tenant_id": tenant_id,
                "subject": "tw-refusal",
                "objective": "probe",
            },
            "client_context": {},
        }),
    )
    .await;
    let thread_id = created["thread_id"].as_str().unwrap().to_string();
    let response = client
        .post(format!("{base}/v1/threads/{thread_id}/commands"))
        .header(PRINCIPAL_HEADER, &human_id)
        .json(&json!({
            "protocol_version": reasonbraid_core::PROTOCOL_VERSION,
            "operation": "thread.close",
            "request_id": reasonbraid_core::RequestId::new().to_string(),
            "idempotency_key": "tw-refusal-close",
            "body": {
                "tenant_id": tenant_id,
                "reason": "dishonest",
                "outcome": "accepted_unanimously",
                "unresolved": ["an objection stands"],
            },
            "client_context": {},
        }))
        .send()
        .await
        .expect("close request");
    assert_eq!(
        response.status().as_u16(),
        400,
        "the decision family refuses"
    );
    let refused: Value = response.json().await.unwrap();
    assert!(
        refused["message"]
            .as_str()
            .unwrap()
            .contains("failure terminal"),
        "{refused}"
    );

    let response = client
        .post(format!("{base}/v1/threads/{thread_id}/commands"))
        .header(PRINCIPAL_HEADER, &human_id)
        .json(&json!({
            "protocol_version": reasonbraid_core::PROTOCOL_VERSION,
            "operation": "thread.close",
            "request_id": reasonbraid_core::RequestId::new().to_string(),
            "idempotency_key": "tw-failure-close",
            "body": {
                "tenant_id": tenant_id,
                "reason": "the evidence did not converge",
                "outcome": "insufficient_evidence",
                "unresolved": ["the objection stands"],
                "minority_report": {
                    "synthesizer": "hpr-tw-human",
                    "input_event_range": "1..3",
                    "sources": ["https://example.org/a"],
                    "coverage": [
                        { "item": "the objection", "included": true },
                        { "item": "the weak claim", "included": false, "reason": "uncited" },
                    ],
                },
            },
            "client_context": {},
        }))
        .send()
        .await
        .expect("close request");
    assert_eq!(
        response.status().as_u16(),
        200,
        "the failure family accepts"
    );
    let (_, timeline) = get(
        &client,
        &base,
        &format!("/v1/threads/{thread_id}/events?tenant_id={tenant_id}"),
        &human_id,
    )
    .await;
    let close_event = timeline["events"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["event_type"] == json!("thread.closed"))
        .cloned()
        .expect("the close event exists");
    assert_eq!(
        close_event["body"]["outcome"],
        json!("insufficient_evidence")
    );
    let report = &close_event["body"]["minority_report"];
    assert_eq!(report["synthesizer"], json!("hpr-tw-human"));
    assert_eq!(report["input_event_range"], json!("1..3"));
    assert_eq!(report["sources"], json!(["https://example.org/a"]));
    assert_eq!(report["coverage"].as_array().unwrap().len(), 2);
    assert_eq!(report["coverage"][1]["included"], json!(false));
    assert_eq!(report["coverage"][1]["reason"], json!("uncited"));
}

#[tokio::test]
async fn the_evidence_requests_and_verdicts_execute_on_their_steps() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, human) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "ev-human" }),
    )
    .await;
    assert_eq!(status, 200, "the human enrolls: {human}");
    let human_id = human["principal_id"].as_str().unwrap().to_string();
    let tenant_id = human["tenant_id"].as_str().unwrap().to_string();

    let create = |profile: &'static str, subject: &'static str| {
        let client = client.clone();
        let base = base.clone();
        let human_id = human_id.clone();
        let tenant_id = tenant_id.clone();
        async move {
            post(
                &client,
                &base,
                "/v1/threads",
                &human_id,
                &json!({
                    "protocol_version": reasonbraid_core::PROTOCOL_VERSION,
                    "operation": "thread.create",
                    "request_id": reasonbraid_core::RequestId::new().to_string(),
                    "idempotency_key": format!("ev-create-{subject}"),
                    "body": {
                        "tenant_id": tenant_id,
                        "subject": subject,
                        "objective": "probe",
                        "workflow_profile": profile,
                    },
                    "client_context": {},
                }),
            )
            .await
        }
    };

    // ── the evidence_review profile: solicit → evidence_request → assess ──
    let (status, created) = create("evidence_review", "ev-request").await;
    assert_eq!(status, 200, "the evidence_review create: {created}");
    let thread_id = created["thread_id"].as_str().unwrap().to_string();

    let command = |key: &'static str, operation: &'static str, body: Value| {
        let client = client.clone();
        let base = base.clone();
        let human_id = human_id.clone();
        let thread_id = thread_id.clone();
        async move {
            let response = client
                .post(format!("{base}/v1/threads/{thread_id}/commands"))
                .header(PRINCIPAL_HEADER, &human_id)
                .json(&json!({
                    "protocol_version": reasonbraid_core::PROTOCOL_VERSION,
                    "operation": operation,
                    "request_id": reasonbraid_core::RequestId::new().to_string(),
                    "idempotency_key": key,
                    "body": body,
                    "client_context": {},
                }))
                .send()
                .await
                .expect("command request");
            let status = response.status().as_u16();
            let text = response.text().await.expect("command body");
            let value = serde_json::from_str(&text).unwrap_or_else(|_| json!({ "raw": text }));
            (status, value)
        }
    };
    let read_events = || {
        let client = client.clone();
        let base = base.clone();
        let human_id = human_id.clone();
        let thread_id = thread_id.clone();
        let tenant_id = tenant_id.clone();
        async move {
            get(
                &client,
                &base,
                &format!("/v1/threads/{thread_id}/events?tenant_id={tenant_id}"),
                &human_id,
            )
            .await
        }
    };

    // 1. The claim (step 0: solicit) — the request will target its digest.
    let claim_content = "the citation validation holds";
    let claim_digest = reasonbraid_server::fetcher::digest_sha256_hex(claim_content.as_bytes());
    let (status, claimed) = command(
        "ev-claim",
        "thread.contribute",
        json!({
            "tenant_id": tenant_id,
            "content": "the claim",
            "kind": "claim",
            "claims": [ { "content": claim_content } ],
        }),
    )
    .await;
    assert_eq!(status, 200, "the claim contributes: {claimed}");

    // 2. The step gate: a request during `solicit` is the typed refusal.
    let (status, refused) = command(
        "ev-request-early",
        "thread.contribute",
        json!({
            "tenant_id": tenant_id,
            "content": "show the evidence",
            "kind": "evidence_request",
            "target_claim_digest": claim_digest,
        }),
    )
    .await;
    assert_eq!(status, 400, "the early request refuses: {refused}");
    assert!(
        refused["message"]
            .as_str()
            .unwrap()
            .contains("evidence_request"),
        "{refused}"
    );

    // 3. The round advance moves the step: solicit → evidence_request.
    let (status, advanced) = command(
        "ev-advance",
        "thread.advance_round",
        json!({ "tenant_id": tenant_id }),
    )
    .await;
    assert_eq!(status, 200, "the round advances: {advanced}");

    // 4. The request on its step: accepted, the target rides the event.
    let (status, requested) = command(
        "ev-request",
        "thread.contribute",
        json!({
            "tenant_id": tenant_id,
            "content": "show the evidence",
            "kind": "evidence_request",
            "target_claim_digest": claim_digest,
        }),
    )
    .await;
    assert_eq!(status, 200, "the request succeeds: {requested}");
    let (status, timeline) = read_events().await;
    assert_eq!(status, 200, "the timeline reads: {timeline}");
    let request_event = timeline["events"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| {
            e["event_type"] == json!("thread.contribution_submitted")
                && e["body"]["kind"] == json!("evidence_request")
        })
        .cloned()
        .expect("the request event exists");
    assert_eq!(
        request_event["body"]["target_claim_digest"],
        json!(claim_digest)
    );

    // 5. A digest that is not a claim of THIS thread is the typed refusal.
    let foreign = reasonbraid_server::fetcher::digest_sha256_hex("never claimed".as_bytes());
    let (status, refused) = command(
        "ev-request-foreign",
        "thread.contribute",
        json!({
            "tenant_id": tenant_id,
            "content": "show the evidence",
            "kind": "evidence_request",
            "target_claim_digest": foreign,
        }),
    )
    .await;
    assert_eq!(status, 400, "the foreign target refuses: {refused}");
    assert!(
        refused["message"].as_str().unwrap().contains("not a claim"),
        "{refused}"
    );

    // 6. The evidence_reference kind carries evidence — empty refs refuse.
    let (status, refused) = command(
        "ev-ref-empty",
        "thread.contribute",
        json!({
            "tenant_id": tenant_id,
            "content": "empty",
            "kind": "evidence_reference",
        }),
    )
    .await;
    assert_eq!(status, 400, "the empty reference refuses: {refused}");
    assert!(
        refused["message"]
            .as_str()
            .unwrap()
            .contains("at least one"),
        "{refused}"
    );
    let (status, referenced) = command(
        "ev-ref",
        "thread.contribute",
        json!({
            "tenant_id": tenant_id,
            "content": "the evidence",
            "kind": "evidence_reference",
            "evidence_refs": [ { "uri": "https://example.org/evidence" } ],
        }),
    )
    .await;
    assert_eq!(status, 200, "the reference contributes: {referenced}");

    // 7. The kind-specific field rides its kind: a claim cannot carry the target.
    let (status, refused) = command(
        "ev-claim-target",
        "thread.contribute",
        json!({
            "tenant_id": tenant_id,
            "content": "misplaced",
            "kind": "claim",
            "target_claim_digest": claim_digest,
        }),
    )
    .await;
    assert_eq!(status, 400, "the misplaced target refuses: {refused}");

    // ── the independent_panel profile: blind_solicit → adjudicate → decide ──
    let (status, created) = create("independent_panel", "ev-verdict").await;
    assert_eq!(status, 200, "the panel create: {created}");
    let thread2_id = created["thread_id"].as_str().unwrap().to_string();
    let command2 = |key: &'static str, operation: &'static str, body: Value| {
        let client = client.clone();
        let base = base.clone();
        let human_id = human_id.clone();
        let thread2_id = thread2_id.clone();
        async move {
            let response = client
                .post(format!("{base}/v1/threads/{thread2_id}/commands"))
                .header(PRINCIPAL_HEADER, &human_id)
                .json(&json!({
                    "protocol_version": reasonbraid_core::PROTOCOL_VERSION,
                    "operation": operation,
                    "request_id": reasonbraid_core::RequestId::new().to_string(),
                    "idempotency_key": key,
                    "body": body,
                    "client_context": {},
                }))
                .send()
                .await
                .expect("command request");
            let status = response.status().as_u16();
            let text = response.text().await.expect("command body");
            let value = serde_json::from_str(&text).unwrap_or_else(|_| json!({ "raw": text }));
            (status, value)
        }
    };

    // 8. A verdict during `blind_solicit` refuses (the adjudicate step gate).
    let (status, refused) = command2(
        "ev-verdict-early",
        "thread.contribute",
        json!({
            "tenant_id": tenant_id,
            "content": "too early",
            "kind": "verdict",
            "verdict": {
                "target_digest": "sha256:00",
                "rule": "majority",
                "outcome": "accepted_with_recorded_objections",
            },
        }),
    )
    .await;
    assert_eq!(status, 400, "the early verdict refuses: {refused}");
    assert!(
        refused["message"].as_str().unwrap().contains("adjudicate"),
        "{refused}"
    );

    // 9. The round advance commits the blind phase AND seats `adjudicate`.
    let (status, advanced) = command2(
        "ev-advance-2",
        "thread.advance_round",
        json!({ "tenant_id": tenant_id }),
    )
    .await;
    assert_eq!(status, 200, "the panel advances: {advanced}");

    // 10. The verdict on its step: the attributable record rides the event
    // with the CANONICAL outcome (the alias never persists).
    let (status, verdict) = command2(
        "ev-verdict",
        "thread.contribute",
        json!({
            "tenant_id": tenant_id,
            "content": "the panel judged",
            "kind": "verdict",
            "verdict": {
                "target_digest": "sha256:00",
                "rule": "majority",
                "outcome": "decided",
            },
        }),
    )
    .await;
    assert_eq!(status, 200, "the verdict succeeds: {verdict}");
    let (status, timeline) = get(
        &client,
        &base,
        &format!("/v1/threads/{thread2_id}/events?tenant_id={tenant_id}"),
        &human_id,
    )
    .await;
    assert_eq!(status, 200, "the timeline reads: {timeline}");
    let verdict_event = timeline["events"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| {
            e["event_type"] == json!("thread.contribution_submitted")
                && e["body"]["kind"] == json!("verdict")
        })
        .cloned()
        .expect("the verdict event exists");
    assert_eq!(
        verdict_event["body"]["verdict"]["outcome"],
        json!("accepted_by_rule")
    );
    assert_eq!(verdict_event["body"]["verdict"]["rule"], json!("majority"));

    // 11. The verdict field rides its kind only.
    let (status, refused) = command2(
        "ev-verdict-misplaced",
        "thread.contribute",
        json!({
            "tenant_id": tenant_id,
            "content": "misplaced",
            "kind": "position",
            "verdict": {
                "target_digest": "sha256:00",
                "rule": "majority",
                "outcome": "accepted_by_rule",
            },
        }),
    )
    .await;
    assert_eq!(status, 400, "the misplaced verdict refuses: {refused}");
}

#[tokio::test]
async fn the_moderation_actions_are_bounded_contributions() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, human) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "md-human" }),
    )
    .await;
    assert_eq!(status, 200, "the human enrolls: {human}");
    let human_id = human["principal_id"].as_str().unwrap().to_string();
    let tenant_id = human["tenant_id"].as_str().unwrap().to_string();

    // The custom profile composes the `moderate` step (the built-ins don't).
    let (status, registered) = post(
        &client,
        &base,
        "/v1/workflow-profiles",
        &human_id,
        &json!({
            "profile_id": "moderated_panel",
            "steps": ["solicit", "moderate", "decide"],
        }),
    )
    .await;
    assert_eq!(status, 200, "the profile registers: {registered}");

    let (status, created) = post(
        &client,
        &base,
        "/v1/threads",
        &human_id,
        &json!({
            "protocol_version": reasonbraid_core::PROTOCOL_VERSION,
            "operation": "thread.create",
            "request_id": reasonbraid_core::RequestId::new().to_string(),
            "idempotency_key": "md-create",
            "body": {
                "tenant_id": tenant_id,
                "subject": "md",
                "objective": "probe",
                "workflow_profile": "moderated_panel",
            },
            "client_context": {},
        }),
    )
    .await;
    assert_eq!(status, 200, "the create succeeds: {created}");
    let thread_id = created["thread_id"].as_str().unwrap().to_string();

    let command = |key: &'static str, operation: &'static str, body: Value| {
        let client = client.clone();
        let base = base.clone();
        let human_id = human_id.clone();
        let thread_id = thread_id.clone();
        async move {
            let response = client
                .post(format!("{base}/v1/threads/{thread_id}/commands"))
                .header(PRINCIPAL_HEADER, &human_id)
                .json(&json!({
                    "protocol_version": reasonbraid_core::PROTOCOL_VERSION,
                    "operation": operation,
                    "request_id": reasonbraid_core::RequestId::new().to_string(),
                    "idempotency_key": key,
                    "body": body,
                    "client_context": {},
                }))
                .send()
                .await
                .expect("command request");
            let status = response.status().as_u16();
            let text = response.text().await.expect("command body");
            let value = serde_json::from_str(&text).unwrap_or_else(|_| json!({ "raw": text }));
            (status, value)
        }
    };

    // 1. The to-be-moderated contribution (step 0: solicit).
    let (status, posted) = command(
        "md-position",
        "thread.contribute",
        json!({
            "tenant_id": tenant_id,
            "content": "the panel's position",
        }),
    )
    .await;
    assert_eq!(status, 200, "the position contributes: {posted}");
    let position_event = posted["event_id"].as_str().unwrap().to_string();

    // 2. The step gate: a moderation action during `solicit` refuses.
    let (status, refused) = command(
        "md-early",
        "thread.contribute",
        json!({
            "tenant_id": tenant_id,
            "content": "classified",
            "kind": "classify",
            "ref_event_id": position_event,
        }),
    )
    .await;
    assert_eq!(status, 400, "the early moderation refuses: {refused}");
    assert!(
        refused["message"].as_str().unwrap().contains("moderate"),
        "{refused}"
    );

    // 3. The round advance seats `moderate`.
    let (status, advanced) = command(
        "md-advance",
        "thread.advance_round",
        json!({ "tenant_id": tenant_id }),
    )
    .await;
    assert_eq!(status, 200, "the round advances: {advanced}");

    // 4. The moderation action on its step: accepted, the reference rides it.
    let (status, classified) = command(
        "md-classify",
        "thread.contribute",
        json!({
            "tenant_id": tenant_id,
            "content": "this is the proposal",
            "kind": "classify",
            "ref_event_id": position_event,
        }),
    )
    .await;
    assert_eq!(status, 200, "the classify succeeds: {classified}");
    let classify_event = classified["event_id"].as_str().unwrap().to_string();

    // 5. A forged reference refuses.
    let (status, refused) = command(
        "md-forged-ref",
        "thread.contribute",
        json!({
            "tenant_id": tenant_id,
            "content": "nothing",
            "kind": "classify",
            "ref_event_id": "evt_does_not_exist",
        }),
    )
    .await;
    assert_eq!(status, 400, "the forged reference refuses: {refused}");
    assert!(
        refused["message"]
            .as_str()
            .unwrap()
            .contains("does not exist"),
        "{refused}"
    );

    // 6. The capability-shaped fields refuse on the moderation kinds — the
    // prohibition is the vocabulary's negative space.
    let (status, refused) = command(
        "md-verdict",
        "thread.contribute",
        json!({
            "tenant_id": tenant_id,
            "content": "not a verdict",
            "kind": "classify",
            "verdict": { "target_digest": "sha256:00", "rule": "majority", "outcome": "accepted_by_rule" },
        }),
    )
    .await;
    assert_eq!(status, 400, "the verdict field refuses: {refused}");
    assert!(
        refused["message"]
            .as_str()
            .unwrap()
            .contains("negative space"),
        "{refused}"
    );
    let (status, refused) = command(
        "md-evidence",
        "thread.contribute",
        json!({
            "tenant_id": tenant_id,
            "content": "no evidence",
            "kind": "classify",
            "evidence_refs": [ { "uri": "https://example.org/x" } ],
        }),
    )
    .await;
    assert_eq!(status, 400, "the evidence field refuses: {refused}");

    // 7. The reference rides the moderation kinds only.
    let (status, refused) = command(
        "md-misplaced-ref",
        "thread.contribute",
        json!({
            "tenant_id": tenant_id,
            "content": "misplaced",
            "kind": "position",
            "ref_event_id": position_event,
        }),
    )
    .await;
    assert_eq!(status, 400, "the misplaced reference refuses: {refused}");

    // 8. Another moderation kind (no reference needed) succeeds.
    let (status, clarified) = command(
        "md-clarify",
        "thread.contribute",
        json!({
            "tenant_id": tenant_id,
            "content": "what does the panel mean by this?",
            "kind": "request_clarification",
        }),
    )
    .await;
    assert_eq!(status, 200, "the clarification succeeds: {clarified}");

    // 9. The moderation action is challengeable — the appeal IS the
    // existing challenge verb (the action is a contribution).
    let (status, appealed) = command(
        "md-appeal",
        "thread.challenge",
        json!({
            "tenant_id": tenant_id,
            "target_event_id": classify_event,
            "content": "the classification mislabels the position",
        }),
    )
    .await;
    assert_eq!(
        status, 200,
        "the moderation action is challengeable: {appealed}"
    );

    // 10. The custom profile is THIS test's row — the registry is global
    // (no tenant scoping) and the `.1.2` built-in count must stay 8, so
    // the test removes its own registration.
    sqlx::query(
        "DELETE FROM workflow_profiles WHERE profile_id = 'moderated_panel' AND NOT built_in",
    )
    .execute(&pool)
    .await
    .expect("the custom profile row deletes");
}

#[tokio::test]
async fn the_synthesis_record_is_derived_content() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, human) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "sy-human" }),
    )
    .await;
    assert_eq!(status, 200, "the human enrolls: {human}");
    let human_id = human["principal_id"].as_str().unwrap().to_string();
    let tenant_id = human["tenant_id"].as_str().unwrap().to_string();

    // The DEFAULT quick_advice profile: solicit → synthesize → decide.
    let (status, created) = post(
        &client,
        &base,
        "/v1/threads",
        &human_id,
        &json!({
            "protocol_version": reasonbraid_core::PROTOCOL_VERSION,
            "operation": "thread.create",
            "request_id": reasonbraid_core::RequestId::new().to_string(),
            "idempotency_key": "sy-create",
            "body": {
                "tenant_id": tenant_id,
                "subject": "sy",
                "objective": "probe",
            },
            "client_context": {},
        }),
    )
    .await;
    assert_eq!(status, 200, "the create succeeds: {created}");
    let thread_id = created["thread_id"].as_str().unwrap().to_string();

    let command = |key: &'static str, operation: &'static str, body: Value| {
        let client = client.clone();
        let base = base.clone();
        let human_id = human_id.clone();
        let thread_id = thread_id.clone();
        async move {
            let response = client
                .post(format!("{base}/v1/threads/{thread_id}/commands"))
                .header(PRINCIPAL_HEADER, &human_id)
                .json(&json!({
                    "protocol_version": reasonbraid_core::PROTOCOL_VERSION,
                    "operation": operation,
                    "request_id": reasonbraid_core::RequestId::new().to_string(),
                    "idempotency_key": key,
                    "body": body,
                    "client_context": {},
                }))
                .send()
                .await
                .expect("command request");
            let status = response.status().as_u16();
            let text = response.text().await.expect("command body");
            let value = serde_json::from_str(&text).unwrap_or_else(|_| json!({ "raw": text }));
            (status, value)
        }
    };

    // 1. The input contribution (version 2).
    let (status, posted) = command(
        "sy-position",
        "thread.contribute",
        json!({
            "tenant_id": tenant_id,
            "content": "the advice",
        }),
    )
    .await;
    assert_eq!(status, 200, "the position contributes: {posted}");

    // 2. The kind + step gates: a synthesis during `solicit`, and on a
    // non-summary kind, both refuse.
    let synthesis = json!({
        "synthesizer": "hpr-sy-human",
        "input_from": 1,
        "input_to": 2,
        "sources": ["https://example.org/source"],
        "coverage": [
            { "item": "the objection", "included": true },
            { "item": "the weak claim", "included": false, "reason": "uncited" },
        ],
    });
    let (status, refused) = command(
        "sy-early",
        "thread.contribute",
        json!({
            "tenant_id": tenant_id,
            "content": "too early",
            "kind": "summary",
            "synthesis": synthesis,
        }),
    )
    .await;
    assert_eq!(status, 400, "the early synthesis refuses: {refused}");
    assert!(
        refused["message"].as_str().unwrap().contains("synthesize"),
        "{refused}"
    );
    let (status, refused) = command(
        "sy-wrong-kind",
        "thread.contribute",
        json!({
            "tenant_id": tenant_id,
            "content": "wrong kind",
            "kind": "position",
            "synthesis": synthesis,
        }),
    )
    .await;
    assert_eq!(status, 400, "the wrong-kind synthesis refuses: {refused}");
    assert!(
        refused["message"].as_str().unwrap().contains("summary"),
        "{refused}"
    );

    // 3. The round advance seats `synthesize`.
    let (status, advanced) = command(
        "sy-advance",
        "thread.advance_round",
        json!({ "tenant_id": tenant_id }),
    )
    .await;
    assert_eq!(status, 200, "the round advances: {advanced}");

    // 4. The synthesis on its step: accepted, the record rides the event.
    let (status, synthesized) = command(
        "sy-synthesize",
        "thread.contribute",
        json!({
            "tenant_id": tenant_id,
            "content": "the synthesis",
            "kind": "summary",
            "synthesis": synthesis,
        }),
    )
    .await;
    assert_eq!(status, 200, "the synthesis succeeds: {synthesized}");
    let (status, timeline) = get(
        &client,
        &base,
        &format!("/v1/threads/{thread_id}/events?tenant_id={tenant_id}"),
        &human_id,
    )
    .await;
    assert_eq!(status, 200, "the timeline reads: {timeline}");
    let synthesis_event = timeline["events"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| {
            e["event_type"] == json!("thread.contribution_submitted")
                && e["body"]["kind"] == json!("summary")
        })
        .cloned()
        .expect("the synthesis event exists");
    assert_eq!(
        synthesis_event["body"]["synthesis"]["synthesizer"],
        json!("hpr-sy-human")
    );
    assert_eq!(synthesis_event["body"]["synthesis"]["input_from"], json!(1));
    assert_eq!(synthesis_event["body"]["synthesis"]["input_to"], json!(2));
    assert_eq!(
        synthesis_event["body"]["synthesis"]["coverage"]
            .as_array()
            .unwrap()
            .len(),
        2
    );

    // 5. An input range that names events beyond the log refuses (the
    // transformation must be re-derivable — never a claim over nothing).
    let (status, refused) = command(
        "sy-beyond",
        "thread.contribute",
        json!({
            "tenant_id": tenant_id,
            "content": "overreach",
            "kind": "summary",
            "synthesis": {
                "synthesizer": "hpr-sy-human",
                "input_from": 1,
                "input_to": 99,
                "sources": [],
                "coverage": [],
            },
        }),
    )
    .await;
    assert_eq!(status, 400, "the overreaching range refuses: {refused}");
    assert!(
        refused["message"].as_str().unwrap().contains("exceeds"),
        "{refused}"
    );
    let (status, refused) = command(
        "sy-inverted",
        "thread.contribute",
        json!({
            "tenant_id": tenant_id,
            "content": "inverted",
            "kind": "summary",
            "synthesis": {
                "synthesizer": "hpr-sy-human",
                "input_from": 3,
                "input_to": 2,
                "sources": [],
                "coverage": [],
            },
        }),
    )
    .await;
    assert_eq!(status, 400, "the inverted range refuses: {refused}");
    assert!(
        refused["message"].as_str().unwrap().contains("invalid"),
        "{refused}"
    );
}
