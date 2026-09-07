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
    let url = match std::env::var("DATABASE_URL") {
        Ok(u) => u,
        Err(_) => {
            eprintln!(
                "SKIP: DATABASE_URL is unset — run scripts/run_pg_tests.sh for the profile proof"
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
        let ca = std::sync::Arc::new(ensure_server_ca(pool).await.expect("server CA"));
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
    let envelope = |operation: &str, key: &str, body: Value| {
        let env = reasonbraid_core::CommandEnvelope {
            protocol_version: reasonbraid_core::PROTOCOL_VERSION.to_string(),
            operation: operation.to_string(),
            request_id: reasonbraid_core::RequestId::new(),
            idempotency_key: key.to_string(),
            expected_aggregate_version: None,
            body,
            authority_context: None,
            client_context: Default::default(),
        };
        env
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
    let explanation = inspected["explanation"].as_array().unwrap();
    assert!(
        explanation[0]["stage1_reasons"]
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r.as_str().unwrap().contains("code_review")),
        "the selection explanation rides: {inspected}"
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
    let mut profile = |extra: &[&str]| {
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
