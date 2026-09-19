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

#[path = "support/cleanup.rs"]
mod pg_cleanup;

#[path = "support/site.rs"]
mod site_fixture;

use std::net::SocketAddr;
use std::sync::OnceLock;

use reasonbraid_server::{api_router, ca::ensure_server_ca, node_router, PRINCIPAL_HEADER};
use serde_json::{json, Value};
use sqlx::PgPool;

/// A principal of the right SHAPE that no enrolment ever mints. A malformed id
/// is refused by `resolve_principal` before any authorization runs, so a control
/// that wants to measure a gate must present a well-formed stranger.
const STRANGER: &str = "hpr_00000000-0000-7000-8000-00000000dead";

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
    pg_cleanup::delete_tables(
        &pool,
        &[
            "site_audit",
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
            "snapshot_objects",
            "reference_registrations",
            "resource_references",
            "quota_events",
            "usage_quotas",
            "federation_agreements",
            "cross_domain_receipts",
            "mcp_listen_state",
            "tenant_bootstrap_requests",
            "routing_recommendations",
            "routing_resolutions",
            "policy_reviews",
            "policy_outcomes",
            "policy_corrections",
            "policy_drift",
            "policy_publications",
            "policy_projections",
            "policy_approvals",
            "policy_decisions",
            "policy_proposals",
            "tenants",
            "idempotency",
            "event_log",
            "aggregate_state",
        ],
    )
    .await
    .expect("purge checked fixture plan");
    // This suite provisions site authority (`.7.4.3`), so it clears the site
    // tables the way the other suites that touch them do. The guard row is a
    // singleton the migration seeds and is deliberately left alone.
    sqlx::raw_sql(
        "DELETE FROM public.site_audit; \
         DELETE FROM public.site_grants; \
         DELETE FROM public.site_boundaries",
    )
    .execute(&pool)
    .await
    .expect("clear site authority fixtures");
    // ⛔ The opt-in R3/R5/RX gate is part of the world a test starts in, and it
    // was the one part this helper did not reset. `sync_gated_entries` writes
    // `resolver_capabilities` rows, which the migration seeds and the cleanup
    // plan above deliberately does not touch — so a test that opened the gate
    // handed it, open, to whichever test ran next, and
    // `the_gated_packs_resolve_only_while_the_gate_is_open` failed on its CLOSED
    // phase. Closing it at the END of the opening test is not enough: a test
    // that PANICS never reaches its own end, which is exactly how this was
    // found. Normalising it HERE makes the starting state independent of how the
    // previous test finished, and `false` is the product's own default.
    reasonbraid_server::sync_gated_entries(&pool, false)
        .await
        .expect("normalise the opt-in gate to its default");
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

    // `.7.1.2.1`: registering is a SITE act, so this suite's registrar holds the
    // explicitly issued `workflow_register` capability. Reading the registry
    // stays on enrolment — a tenant must see the profiles it may name.
    site_fixture::provision(
        &pool,
        &human_id,
        &[reasonbraid_server::site_authority::Action::WorkflowRegister],
    )
    .await;

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
            "reason": "the deliberate lane needs a critique step",
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
            .json(&json!({
                "profile_id": "custom_bad",
                "steps": steps,
                "reason": "an invalid composition must refuse by name",
            }))
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
async fn a_reference_submits_typed_and_the_locator_digest_pair_is_the_key() {
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

    // The pair is the key (`SIGNOFF-REPAIR.11.14.3.2`): the same locator at a
    // DIFFERENT digest is a SECOND reference. §12.6's live page changed, §12.1
    // forbids erasing that security-relevant distinction, and the 409 this
    // replaces told the caller that somebody else had pinned the locator to a
    // digest they were never shown — the cross-tenant existence §9.8 forbids.
    let mut changed = body.clone();
    changed["expected_digest"] = json!(format!("sha256:{}", "b".repeat(64)));
    let (status, second) = submit(&changed).await;
    assert_eq!(status, 200, "the second pin registers: {second}");
    assert_eq!(second["replayed"], json!(false));
    assert_ne!(
        second["resource_id"],
        json!(resource_id),
        "a different digest is a different reference: {second}"
    );

    // And the UNPINNED submission is its own row, replayed on repeat —
    // `migrations/0065` makes the constraint `NULLS NOT DISTINCT`, so a
    // digest-less reference cannot multiply.
    let mut unpinned = body.clone();
    unpinned.as_object_mut().unwrap().remove("expected_digest");
    let (status, fresh_unpinned) = submit(&unpinned).await;
    assert_eq!(status, 200, "the unpinned submit: {fresh_unpinned}");
    assert_eq!(fresh_unpinned["replayed"], json!(false));
    let (status, replayed_unpinned) = submit(&unpinned).await;
    assert_eq!(status, 200, "the unpinned replay: {replayed_unpinned}");
    assert_eq!(replayed_unpinned["replayed"], json!(true));
    assert_eq!(
        replayed_unpinned["resource_id"], fresh_unpinned["resource_id"],
        "one unpinned row, not one per submission: {replayed_unpinned}"
    );

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

/// `SIGNOFF-REPAIR.7.3.6.2`: a replace is COMPLETE, and the HTTP verb does not
/// perform one.
///
/// `resolvers::register` INSERTed 18 columns and its `ON CONFLICT DO UPDATE`
/// wrote 6, silently keeping the other eleven — every advertised policy,
/// `media_types`, the abilities, the authentication classes. So a corrected
/// advertisement never reached an existing row, while `advertised_media_types`
/// promises that narrowing a pack's advertisement narrows what it may acquire
/// *in the same act*.
///
/// The two halves are asserted separately because they are two callers with
/// two different trusts:
///
/// 1. **The product's own path must replace completely.** `sync_gated_entries`
///    runs at every boot, and the advertisement compiled into the binary is the
///    truth; drift in the row loses to it. Driven here by writing drift into the
///    row and re-running the sync.
/// 2. **The HTTP verb must not replace at all.** `resolver_capabilities` has no
///    tenant column, so any tenant administrator can address the built-in
///    packs' rows (`SIGNOFF-REPAIR.11.9.1.1.1`, owned by `.7.1`). It refuses an
///    existing id BY NAME rather than reporting a success it did not perform.
#[tokio::test]
async fn a_replace_is_complete_and_the_http_verb_does_not_perform_one() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    // ⛔ No server and no principal: this half is about the PRODUCT's own boot
    // path, which takes neither. The HTTP verb is the sibling test.

    // ── The product's own path replaces COMPLETELY ───────────────────────────
    // Open the gate so the R3 row exists, then write drift into the columns the
    // old upsert did not touch, then re-run the sync. The advertisement in the
    // binary must win.
    reasonbraid_server::sync_gated_entries(&pool, true)
        .await
        .expect("open the gate");
    sqlx::query(
        "UPDATE resolver_capabilities SET subresource_policy = 'allow', \
         javascript_policy = 'allow', media_types = '[\"application/x-drift\"]'::jsonb, \
         abilities = '[\"drift\"]'::jsonb WHERE resolver_id = 'r3-browser-worker'",
    )
    .execute(&pool)
    .await
    .expect("write drift into the row");
    reasonbraid_server::sync_gated_entries(&pool, true)
        .await
        .expect("re-run the startup sync");
    let (subresource, javascript, media, abilities): (String, String, Value, Value) =
        sqlx::query_as(
            "SELECT subresource_policy, javascript_policy, media_types, abilities \
             FROM resolver_capabilities WHERE resolver_id = 'r3-browser-worker'",
        )
        .fetch_one(&pool)
        .await
        .expect("read the R3 row back");
    assert_eq!(
        subresource, "deny",
        "the startup sync restores the advertised subresource policy — a \
         corrected advertisement that cannot reach the row is an advertisement \
         the registry does not carry",
    );
    assert_eq!(javascript, "allow-bounded", "…and the javascript policy");
    assert_eq!(
        media,
        json!([]),
        "…and `media_types`, which decides what the acquisition leg admits",
    );
    assert_eq!(abilities, json!(["render"]), "…and the abilities");
    reasonbraid_server::sync_gated_entries(&pool, false)
        .await
        .expect("close the gate again");
}

/// The other half of `SIGNOFF-REPAIR.7.3.6.2`, as its own test so its RED is
/// observed rather than hidden behind the first one's panic.
///
/// `POST /v1/resolvers` said "registers (or replaces)" over an upsert that
/// wrote 6 of 18 columns, so the documented use of the verb — narrowing a
/// pack's `media_types` — answered `200 {"registered": true}` and changed
/// nothing. Completing the replace here was REJECTED: the table has no tenant
/// column, so any tenant administrator addresses any row including the
/// built-in packs' (`SIGNOFF-REPAIR.11.9.1.1.1`, owned by `.7.1`), and
/// widening the upsert would have handed that unbound principal eleven more
/// columns. The verb refuses instead, by name.
#[tokio::test]
async fn the_resolver_verb_refuses_to_replace_an_existing_advertise() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, human) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "replace-verb-human" }),
    )
    .await;
    assert_eq!(status, 200, "the human enrolls: {human}");
    let human_id = human["principal_id"].as_str().unwrap().to_string();

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
    let advertise = |media: Value, policy: &str| {
        json!({
            "resolver_id": "rsv-replace-probe",
            "schemes": ["https"],
            "media_types": media,
            "egress_class": "listed",
            "sandbox_level": "process",
            "subresource_policy": policy,
            "version": "0.1.0",
        })
    };
    let (status, first) =
        register(advertise(json!(["text/html", "application/pdf"]), "allow")).await;
    assert_eq!(status, 200, "a new resolver registers: {first}");

    let (status, refused) = register(advertise(json!(["text/html"]), "deny")).await;
    assert_eq!(
        status, 409,
        "the narrowing re-registration is REFUSED, not silently ignored: {refused}",
    );
    assert_eq!(refused["code"], json!("invalid_transition"));
    assert!(
        refused["message"]
            .as_str()
            .unwrap_or_default()
            .contains("rsv-replace-probe"),
        "the refusal names the row so the caller knows nothing happened: {refused}",
    );

    // And the row is untouched — a refusal that half-applied would be worse
    // than the no-op it replaces.
    let (media, policy): (Value, String) = sqlx::query_as(
        "SELECT media_types, subresource_policy FROM resolver_capabilities \
         WHERE resolver_id = 'rsv-replace-probe'",
    )
    .fetch_one(&pool)
    .await
    .expect("read the probe row");
    assert_eq!(media, json!(["text/html", "application/pdf"]));
    assert_eq!(policy, "allow");

    // ⛔ The refusal is a bound, not a blackout: a DIFFERENT resolver still
    // registers, so this has not turned the verb off.
    let (status, other) = register(json!({
        "resolver_id": "rsv-replace-probe-2",
        "schemes": ["https"],
        "egress_class": "listed",
        "sandbox_level": "process",
        "version": "0.1.0",
    }))
    .await;
    assert_eq!(status, 200, "a different resolver still registers: {other}");

    sqlx::query("DELETE FROM resolver_capabilities WHERE resolver_id = ANY($1::text[])")
        .bind(["rsv-replace-probe", "rsv-replace-probe-2"])
        .execute(&pool)
        .await
        .expect("clear the probe rows");
}

/// `SIGNOFF-REPAIR.7.3.6.5`: a caller that requires VM or container isolation is
/// not served an ordinary child process.
///
/// The R3 browser pack advertised `sandbox_level: "vm_container"` — the TOP of
/// the ADR-018 ladder — while `crates/reasonbraid-browse` spawns an ordinary
/// child process in an owned process group. `security_evidence` carried
/// `"container_required": true`, which is a requirement ON THE DEPLOYMENT, not
/// a property the code establishes.
///
/// ⭐ Nothing in this repository had ever REQUIRED `vm_container`, which is why
/// a false claim at the ladder's top was never noticed: `git grep vm_container`
/// returned the ladder constant, one SDK vocabulary test, and the advertisement
/// itself. A rung nobody stands on holds any weight you like.
///
/// ⛔ ADR-018's exit clause is *the explicit failure, never the silent
/// downgrade*. Satisfying a `vm_container` requirement with a bare process is
/// that downgrade, performed silently, by the one pack that executes untrusted
/// JavaScript.
#[tokio::test]
async fn a_caller_requiring_container_isolation_is_not_served_a_bare_process() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    reasonbraid_server::sync_gated_entries(&pool, true)
        .await
        .expect("open the gate so the R3 row exists");

    let (status, human) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "sandbox-human" }),
    )
    .await;
    assert_eq!(status, 200, "the human enrolls: {human}");
    let human_id = human["principal_id"].as_str().unwrap().to_string();

    let submitted: Value = client
        .post(format!("{base}/v1/resources"))
        .header(PRINCIPAL_HEADER, &human_id)
        .json(&json!({
            "original_locator": "https://example.org/rendered",
            "scheme": "web+render",
        }))
        .send()
        .await
        .expect("submit request")
        .json()
        .await
        .expect("submit json");
    let resource_id = submitted["resource_id"].as_str().unwrap().to_string();

    let resolve = |required: &'static str| {
        let client = client.clone();
        let base = base.clone();
        let human_id = human_id.clone();
        let resource_id = resource_id.clone();
        async move {
            let response = client
                .post(format!("{base}/v1/resources/{resource_id}/resolve"))
                .header(PRINCIPAL_HEADER, &human_id)
                .json(&json!({ "required_sandbox": required, "required_egress": "listed" }))
                .send()
                .await
                .expect("resolve request");
            let parsed: Value = response.json().await.expect("resolve json");
            parsed
        }
    };

    // THE CLAIM. A caller that requires isolation this deployment does not
    // provide gets the explicit unresolvable-now, not the browser worker.
    for required in ["vm_container", "constrained_process"] {
        let refused = resolve(required).await;
        assert_eq!(
            refused["unresolvable_now"],
            json!(true),
            "requiring `{required}` must not be satisfied by a pack that runs an \
             ordinary child process: {refused}",
        );
        assert!(
            refused["resolvers"].as_array().unwrap().is_empty(),
            "…and no resolver is ranked for it: {refused}",
        );
    }

    // ⛔ A BOUND, NOT A BLACKOUT: the pack still resolves for a caller whose
    // requirement it genuinely meets. `process` is what the worker is.
    let served = resolve("process").await;
    assert_eq!(
        served["resolvers"],
        json!(["r3-browser-worker"]),
        "the pack still serves a caller requiring the isolation it actually \
         provides: {served}",
    );

    reasonbraid_server::sync_gated_entries(&pool, false)
        .await
        .expect("close the gate again");
}

/// `SIGNOFF-REPAIR.7.3.6.4`: a caller that bounds a pack's egress is not served
/// a pack that declares no bound.
///
/// ADR-018 states the egress class as *the allowed destinations … the claim is
/// the MAXIMUM, never the minimum*. `resolvers::resolve` filtered it with
/// `declared >= required`, the same test it uses for the sandbox ladder — and
/// the two ladders run in opposite safety directions. Higher sandbox is more
/// isolated, so a floor is right there; higher egress is more REACH, so the
/// same test admitted a pack that reaches further than the caller permitted.
///
/// Measured over the whole 4 × 6 matrix before the repair, the filter refused
/// exactly one combination out of 24, and it was the wrong one: a caller
/// requiring `listed` was served `rx-agent-mediated`, which declares `any`, and
/// no value of `required_egress` meant *do not give me a pack that can dial
/// anywhere*.
#[tokio::test]
async fn an_egress_bound_excludes_a_pack_that_declares_a_wider_one() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, human) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "egress-human" }),
    )
    .await;
    assert_eq!(status, 200, "the human enrolls: {human}");
    let human_id = human["principal_id"].as_str().unwrap().to_string();

    // Two resolvers on one scheme, differing ONLY in the egress they declare.
    // That is the discriminator: anything else that changed the outcome would
    // be a different finding.
    for (id, egress) in [("rsv-egress-listed", "listed"), ("rsv-egress-any", "any")] {
        let response = client
            .post(format!("{base}/v1/resolvers"))
            .header(PRINCIPAL_HEADER, &human_id)
            .json(&json!({
                "resolver_id": id,
                "schemes": ["egress-probe"],
                "egress_class": egress,
                "sandbox_level": "none",
                "latency_range_ms": { "min": 100, "max": 200 },
                "version": "0.1.0",
            }))
            .send()
            .await
            .expect("register request");
        assert_eq!(response.status().as_u16(), 200, "{id} registers");
    }

    let submitted: Value = client
        .post(format!("{base}/v1/resources"))
        .header(PRINCIPAL_HEADER, &human_id)
        .json(&json!({
            "original_locator": "https://example.org/egress-probe",
            "scheme": "egress-probe",
        }))
        .send()
        .await
        .expect("submit request")
        .json()
        .await
        .expect("submit json");
    let resource_id = submitted["resource_id"].as_str().unwrap().to_string();

    let resolve = |required: &'static str| {
        let client = client.clone();
        let base = base.clone();
        let human_id = human_id.clone();
        let resource_id = resource_id.clone();
        async move {
            let response = client
                .post(format!("{base}/v1/resources/{resource_id}/resolve"))
                .header(PRINCIPAL_HEADER, &human_id)
                .json(&json!({ "required_sandbox": "none", "required_egress": required }))
                .send()
                .await
                .expect("resolve request");
            let status = response.status().as_u16();
            let parsed: Value = response.json().await.expect("resolve json");
            (status, parsed)
        }
    };

    // THE BOUND. A caller permitting at most `listed` gets the `listed` pack
    // and NOT the one declaring `any`.
    let (status, bounded) = resolve("listed").await;
    assert_eq!(status, 200, "the bounded resolution: {bounded}");
    assert_eq!(
        bounded["resolvers"],
        json!(["rsv-egress-listed"]),
        "a caller that permits at most `listed` is not served a pack declaring \
         `any` — the egress claim is a MAXIMUM (ADR-018): {bounded}",
    );

    // ⛔ A BOUND, NOT A BLACKOUT: permitting `any` still admits both, so the
    // repair narrows what a caller ASKED to narrow and nothing else.
    let (status, unbounded) = resolve("any").await;
    assert_eq!(status, 200, "the unbounded resolution: {unbounded}");
    let mut ids: Vec<&str> = unbounded["resolvers"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    ids.sort_unstable();
    assert_eq!(
        ids,
        vec!["rsv-egress-any", "rsv-egress-listed"],
        "permitting `any` admits both packs: {unbounded}",
    );

    // ⛔ And a bound NO pack meets is the explicit unresolvable-now, never a
    // silent downgrade to a wider pack (ADR-018's own exit clause).
    let (status, impossible) = resolve("loopback").await;
    assert_eq!(
        status, 200,
        "the impossible bound still answers: {impossible}"
    );
    assert_eq!(impossible["unresolvable_now"], json!(true));
    assert!(impossible["resolvers"].as_array().unwrap().is_empty());

    // 🔴 AND A REQUIREMENT OUTSIDE THE ADR-018 VOCABULARY IS NAMED, not
    // silently answered as an absence. `position()` returned `None` for an
    // unknown class, every row became ineligible, and a typo was indistinguishable
    // from "no resolver available".
    let response = client
        .post(format!("{base}/v1/resources/{resource_id}/resolve"))
        .header(PRINCIPAL_HEADER, &human_id)
        .json(&json!({ "required_sandbox": "none", "required_egress": "public" }))
        .send()
        .await
        .expect("resolve request");
    let status = response.status().as_u16();
    let refused: Value = response.json().await.expect("resolve json");
    assert_eq!(
        status, 400,
        "an off-ladder required class is refused by name, not answered as an \
         empty result: {refused}",
    );
    assert_eq!(refused["code"], json!("invalid_command"));

    sqlx::query("DELETE FROM resolver_capabilities WHERE resolver_id = ANY($1::text[])")
        .bind(["rsv-egress-listed", "rsv-egress-any"])
        .execute(&pool)
        .await
        .expect("clear the probe rows");
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

// ── the R2 success join (`SIGNOFF-REPAIR.7.3.3.4.1`) ─────────────────────────

/// The document the local origin serves.
///
/// The R0 acquisition leg's sniff accepts `text/*` and HTML; the R2 media-type
/// HINT is what routes the acquired bytes to the feed parser. Those are two
/// separate decisions, and this control measures both — including the one that
/// refuses a feed served under its own `application/atom+xml` type.
const SERVED_FEED: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
  <title>Acquisition Join Feed</title>
  <subtitle>the served document</subtitle>
  <entry>
    <title>First entry</title>
    <summary>the first summary</summary>
    <content type="text">the first content</content>
  </entry>
  <entry>
    <title>Second entry</title>
    <summary>the second summary</summary>
  </entry>
</feed>"#;

/// The chunks `extract_feed` derives from `SERVED_FEED`: the feed title joined
/// to its subtitle, then one chunk per entry (title, summary, content). Stated
/// as literals rather than re-derived with the worker's own parser — a control
/// that asks the implementation what it should expect proves nothing.
const EXPECTED_CHUNKS: [&str; 3] = [
    "Acquisition Join Feed\nthe served document",
    "First entry\nthe first summary\nthe first content",
    "Second entry\nthe second summary",
];

/// The local origin: the same bytes on two paths, differing only in the type
/// they are served under.
async fn start_feed_origin() -> (SocketAddr, tokio::task::JoinHandle<()>) {
    use axum::http::header::CONTENT_TYPE;
    use axum::response::IntoResponse;
    let router = axum::Router::new()
        .route(
            "/feed.xml",
            axum::routing::get(|| async {
                ([(CONTENT_TYPE, "text/xml; charset=utf-8")], SERVED_FEED).into_response()
            }),
        )
        .route(
            "/typed-feed.xml",
            axum::routing::get(|| async {
                ([(CONTENT_TYPE, "application/atom+xml")], SERVED_FEED).into_response()
            }),
        )
        .route(
            "/unadvertised-feed.xml",
            axum::routing::get(|| async {
                ([(CONTENT_TYPE, "application/json")], SERVED_FEED).into_response()
            }),
        );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind the feed origin");
    let addr = listener.local_addr().expect("the origin's address");
    let handle = tokio::spawn(async move {
        axum::serve(listener, router).await.expect("serve the feed");
    });
    (addr, handle)
}

/// A fetcher for THIS deployment: the origin's scheme and port, with the
/// destination policy the caller states. Nothing here edits the shipped
/// policy — `Fetcher::new` still builds it, and `api_router` still uses that.
fn origin_fetcher(
    port: u16,
    policy: std::sync::Arc<
        dyn Fn(&std::net::IpAddr) -> reasonbraid_server::ssrf::SsrfVerdict + Send + Sync,
    >,
) -> std::sync::Arc<reasonbraid_server::fetcher::Fetcher> {
    use reasonbraid_server::fetcher::{FetchLimits, Fetcher, FetcherConfig};
    std::sync::Arc::new(
        Fetcher::from_config(FetcherConfig {
            limits: FetchLimits::default(),
            schemes: vec!["http"],
            ports: vec![port],
            policy,
            ..FetcherConfig::default()
        })
        .expect("the origin fetcher builds"),
    )
}

/// The deployment both R2 joins acquire through: the origin's scheme and port,
/// loopback admitted, and every other destination class still answering to the
/// shipped evaluation.
fn admitting_fetcher(port: u16) -> std::sync::Arc<reasonbraid_server::fetcher::Fetcher> {
    origin_fetcher(
        port,
        std::sync::Arc::new(|ip: &std::net::IpAddr| {
            if ip.is_loopback() {
                reasonbraid_server::ssrf::SsrfVerdict::Allowed
            } else {
                reasonbraid_server::ssrf::evaluate(*ip)
            }
        }),
    )
}

/// Declare that THIS deployment's R2 pack serves the origin's scheme.
///
/// The registry row is a deployment fact, not a code path: migration 0027 ships
/// `["https"]`, which the caller asserts as its baseline here. The row is shared
/// with every other suite in the same database, so the returned value must be
/// handed back to `restore_r2_schemes` before anything is asserted.
async fn widen_r2_to_http(pool: &PgPool) -> Value {
    let shipped: Value = sqlx::query_scalar(
        "SELECT schemes FROM resolver_capabilities WHERE resolver_id = 'r2-extract-worker'",
    )
    .fetch_one(pool)
    .await
    .expect("the R2 pack is installed");
    assert_eq!(
        shipped,
        json!(["https"]),
        "the migration's R2 schemes are the baseline this control widens"
    );
    sqlx::query(
        "UPDATE resolver_capabilities SET schemes = $1::jsonb \
         WHERE resolver_id = 'r2-extract-worker'",
    )
    .bind(json!(["https", "http"]))
    .execute(pool)
    .await
    .expect("this deployment's R2 pack also serves the local origin");
    shipped
}

async fn restore_r2_schemes(pool: &PgPool, shipped: &Value) {
    sqlx::query(
        "UPDATE resolver_capabilities SET schemes = $1::jsonb \
         WHERE resolver_id = 'r2-extract-worker'",
    )
    .bind(shipped)
    .execute(pool)
    .await
    .expect("the shipped R2 schemes are restored");
}

/// Submit an R2-hinted reference to the origin, naming the scheme this
/// deployment's R2 pack advertises — which for this control is `http`, the
/// origin's own.
///
/// ⚠️ **The earlier wording of this comment caused a wrong repair, so it is
/// corrected rather than tidied** (`SIGNOFF-REPAIR.11.14.3.5`). It said the
/// field "is NOT validated against the locator, so a control that wrote `https`
/// here would be routed on data that is simply false", and a leaf read that as a
/// missing check. The field is the RESOLVER-SELECTION key `resolvers::resolve`
/// matches against `resolver_capabilities.schemes`, not a claim about the
/// locator: `r1-git-fetcher` advertises `["git"]` and the R3 pack advertises
/// `["web+render"]`, both for `https://*` locators. What writing `https` here
/// would actually do is select a DIFFERENT pack — which is why this helper names
/// the one it means.
async fn submit_hinted(
    client: &reqwest::Client,
    base: &str,
    principal: &str,
    locator: &str,
) -> String {
    let (status, body) = post(
        client,
        base,
        "/v1/resources",
        principal,
        &json!({
            "original_locator": locator,
            "scheme": "http",
            "media_type_hint": "application/atom+xml",
        }),
    )
    .await;
    assert_eq!(status, 200, "the reference submits: {body}");
    body["resource_id"]
        .as_str()
        .expect("the submitted reference id")
        .to_string()
}

/// A repository-local, same-volume scratch path (§13): the root is discovered at
/// runtime from the current directory, so moving the checkout needs no edit and
/// nothing reaches for a system temporary directory.
fn control_scratch(name: &str) -> std::path::PathBuf {
    let root = std::env::current_dir()
        .expect("cwd")
        .ancestors()
        .find(|path| {
            path.join("Cargo.toml").is_file() && path.join("rust-toolchain.toml").is_file()
        })
        .expect("run inside the repository")
        .to_path_buf();
    let dir = root.join("target/r2-join-controls");
    std::fs::create_dir_all(&dir).expect("the control scratch is created");
    dir.join(format!("{name}-{}", std::process::id()))
}

/// The extraction worker both R2 joins need. A control whose whole purpose is
/// closing a stated coverage gap must not be able to report neither way, so an
/// absent worker stops rather than skipping.
fn require_extraction_worker() {
    let worker = reasonbraid_server::extraction::worker_path();
    assert!(
        worker.exists(),
        "the extraction worker is absent at {worker:?}: this control cannot be \
         reported either way without it — build the workspace binaries \
         (`cargo build --workspace --bins --locked`) or set R2_WORKER_BIN"
    );
}

/// The join `.7.3.3.3.2` left explicitly uncovered: a SUCCESSFUL R2 acquisition
/// driven through the real HTTP handler, all the way to the snapshot and the
/// derivations — and the persisted evidence asserted against the bytes the
/// origin actually served, not against a 200.
///
/// Three deployments resolve the SAME reference, differing only in the R0
/// fetcher their state was built with, so each production gate is isolated:
///
/// - `api_router` — the shipped state: refused at the scheme, no evidence;
/// - the origin's scheme with the shipped destination policy: refused at the
///   loopback class, no evidence;
/// - the origin's scheme with a loopback-admitting policy: acquired, extracted,
///   persisted.
///
/// The document's own advertised type is driven here too (`.7.3.3.5.2`): the
/// same feed served as `application/atom+xml` acquires, because the R2 pack
/// advertises that type and the acquisition leg admits the RANKED pack's own
/// advertisement. A type the pack does NOT advertise is still refused, which is
/// what keeps that widening bounded.
#[tokio::test]
async fn the_r2_acquisition_persists_the_served_document_s_own_evidence() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    require_extraction_worker();

    let (origin, _origin_handle) = start_feed_origin().await;
    let port = origin.port();
    let broker = || std::sync::Arc::new(reasonbraid_server::broker::Broker::default());

    // The shipped state, unchanged: https-only, the public-destination policy.
    let shipped = TestServer::start(&pool).await;
    // The origin's scheme, the SHIPPED destination policy.
    let public_policy = TestServer::start_with_router(
        &pool,
        reasonbraid_server::api_router_with_acquisition(
            pool.clone(),
            false,
            broker(),
            origin_fetcher(
                port,
                std::sync::Arc::new(|ip| reasonbraid_server::ssrf::evaluate(*ip)),
            ),
        ),
    )
    .await;
    // The origin's scheme AND a loopback-admitting destination policy.
    let admitting = TestServer::start_with_router(
        &pool,
        reasonbraid_server::api_router_with_acquisition(
            pool.clone(),
            false,
            broker(),
            admitting_fetcher(port),
        ),
    )
    .await;

    let client = reqwest::Client::new();
    let (status, human) = enroll(
        &client,
        &shipped.base(),
        json!({ "kind": "human", "name": "r2-join-human" }),
    )
    .await;
    assert_eq!(status, 200, "the human enrolls: {human}");
    let human_id = human["principal_id"].as_str().unwrap().to_string();

    let base = shipped.base();
    let reference = submit_hinted(
        &client,
        &base,
        &human_id,
        &format!("http://127.0.0.1:{port}/feed.xml"),
    )
    .await;
    let typed_reference = submit_hinted(
        &client,
        &base,
        &human_id,
        &format!("http://127.0.0.1:{port}/typed-feed.xml"),
    )
    .await;
    let unadvertised_reference = submit_hinted(
        &client,
        &base,
        &human_id,
        &format!("http://127.0.0.1:{port}/unadvertised-feed.xml"),
    )
    .await;

    // This deployment's R2 pack serves the origin's scheme. The row is restored
    // below before anything is asserted, so no later suite inherits it.
    let shipped_schemes = widen_r2_to_http(&pool).await;

    let resolve = |base: String, resource: String| {
        let client = client.clone();
        let human_id = human_id.clone();
        async move {
            let (status, body) = post(
                &client,
                &base,
                &format!("/v1/resources/{resource}/resolve"),
                &human_id,
                &json!({ "required_sandbox": "process", "required_egress": "listed" }),
            )
            .await;
            assert_eq!(status, 200, "the resolution answers: {body}");
            body
        }
    };
    let refused_scheme = resolve(shipped.base(), reference.clone()).await;
    let refused_destination = resolve(public_policy.base(), reference.clone()).await;
    let acquired = resolve(admitting.base(), reference.clone()).await;
    let acquired_typed = resolve(admitting.base(), typed_reference.clone()).await;
    let refused_type = resolve(admitting.base(), unadvertised_reference.clone()).await;

    restore_r2_schemes(&pool, &shipped_schemes).await;

    // Every deployment ranked the same pack: the acquisition leg is the only
    // difference between them.
    for outcome in [
        &refused_scheme,
        &refused_destination,
        &acquired,
        &acquired_typed,
        &refused_type,
    ] {
        assert_eq!(
            outcome["resolvers"],
            json!(["r2-extract-worker"]),
            "the R2 pack ranks the hinted reference: {outcome}"
        );
    }

    // The shipped state refuses at the scheme, before any socket opens.
    assert_eq!(
        refused_scheme["acquisition_error"]["kind"],
        json!("scheme_not_allowed"),
        "the shipped https-only fetcher refuses the origin: {refused_scheme}"
    );
    assert!(
        refused_scheme.get("acquisition").is_none(),
        "a refusal is not an acquisition: {refused_scheme}"
    );
    // The shipped DESTINATION policy refuses the loopback class by name — the
    // gate the seam exists to get past, isolated from the scheme gate.
    assert_eq!(
        refused_destination["acquisition_error"]["kind"],
        json!("destination_refused"),
        "the shipped destination policy refuses the origin: {refused_destination}"
    );
    assert!(
        refused_destination["acquisition_error"]["message"]
            .as_str()
            .unwrap()
            .contains("loopback"),
        "the refused class is named: {refused_destination}"
    );
    // The same feed under its OWN advertised type now acquires: the pack that
    // was ranked advertises `application/atom+xml`, so the leg admits it.
    assert!(
        acquired_typed["acquisition_error"].is_null(),
        "an advertised declared type acquires: {acquired_typed}"
    );
    assert_eq!(
        acquired_typed["acquisition"]["parent_digest"],
        json!(reasonbraid_server::fetcher::digest_sha256_hex(
            SERVED_FEED.as_bytes()
        )),
        "the advertised-type acquisition is bound to the same served bytes: {acquired_typed}"
    );
    // A type the pack does NOT advertise is still refused, so the widening is
    // the advertisement's and not "anything declared".
    assert_eq!(
        refused_type["acquisition_error"]["kind"],
        json!("media_type_refused"),
        "an unadvertised declared type stays refused: {refused_type}"
    );

    // The acquisition succeeded, and the receipt describes the served bytes.
    let served = SERVED_FEED.as_bytes();
    let served_digest = reasonbraid_server::fetcher::digest_sha256_hex(served);
    assert!(
        refused_type["acquisition"].is_null() && acquired["acquisition_error"].is_null(),
        "the admitting deployment acquired and the unadvertised one did not: \
         {acquired} / {refused_type}"
    );
    assert_eq!(
        acquired["acquisition"]["parent_digest"],
        json!(served_digest),
        "the receipt is bound to the served bytes: {acquired}"
    );

    // The SNAPSHOT describes the document the origin actually served.
    let snapshots: Vec<Value> =
        sqlx::query_scalar("SELECT to_jsonb(s) FROM evidence_snapshots s WHERE reference_id = $1")
            .bind(&reference)
            .fetch_all(&pool)
            .await
            .expect("read the snapshots back");
    assert_eq!(snapshots.len(), 1, "one snapshot: {snapshots:?}");
    let snapshot = &snapshots[0];
    assert_eq!(
        snapshot["raw_digest"],
        json!(served_digest),
        "the snapshot's raw digest is the served document's: {snapshot}"
    );
    assert_eq!(
        snapshot["byte_length"],
        json!(served.len()),
        "the snapshot's byte length is the served length: {snapshot}"
    );
    assert_eq!(
        snapshot["resolver_id"],
        json!("r2-extract-worker"),
        "the R2 pack is recorded as the resolver: {snapshot}"
    );
    assert_eq!(
        snapshot["original_locator"],
        json!(format!("http://127.0.0.1:{port}/feed.xml")),
        "the snapshot names the requested locator: {snapshot}"
    );

    // The DERIVATIONS are the chunks that feed derives — content and digest.
    let snapshot_id = snapshot["snapshot_id"].as_str().unwrap();
    let mut derived: Vec<(String, String)> = sqlx::query_as(
        "SELECT derived_digest, content FROM derivations WHERE parent_snapshot_id = $1",
    )
    .bind(snapshot_id)
    .fetch_all(&pool)
    .await
    .expect("read the derivations back");
    derived.sort();
    let mut expected: Vec<(String, String)> = EXPECTED_CHUNKS
        .iter()
        .map(|text| {
            (
                reasonbraid_server::fetcher::digest_sha256_hex(text.as_bytes()),
                (*text).to_owned(),
            )
        })
        .collect();
    expected.sort();
    assert_eq!(
        derived, expected,
        "the derivation rows are the chunks this feed derives"
    );

    // The advertised-type acquisition persisted its own evidence, and the
    // snapshot records the type the origin declared rather than a sniffed
    // stand-in.
    let typed_snapshots: Vec<Value> =
        sqlx::query_scalar("SELECT to_jsonb(s) FROM evidence_snapshots s WHERE reference_id = $1")
            .bind(&typed_reference)
            .fetch_all(&pool)
            .await
            .expect("read the advertised-type snapshot back");
    assert_eq!(
        typed_snapshots.len(),
        1,
        "one snapshot: {typed_snapshots:?}"
    );
    assert_eq!(
        typed_snapshots[0]["media_type"],
        json!("application/atom+xml"),
        "the snapshot records the declared type: {}",
        typed_snapshots[0]
    );
    assert_eq!(
        typed_snapshots[0]["raw_digest"],
        json!(served_digest),
        "and the same served bytes: {}",
        typed_snapshots[0]
    );

    // The refused reference persisted nothing.
    let unadvertised_snapshots: i64 =
        sqlx::query_scalar("SELECT count(*) FROM evidence_snapshots WHERE reference_id = $1")
            .bind(&unadvertised_reference)
            .fetch_one(&pool)
            .await
            .expect("count the refused reference's snapshots");
    assert_eq!(
        unadvertised_snapshots, 0,
        "a refused acquisition leaves no evidence behind"
    );
}

/// The mismatch refusal's live join (`SIGNOFF-REPAIR.7.3.3.4.2`): a worker that
/// describes bytes this request never supplied must persist NOTHING.
///
/// The seven controls `.7.3.3.3.2` added sit below the HTTP handler, on
/// `extract_acquired_bytes`. They prove the refusal happens; they cannot prove
/// the handler honours it, because they never reach a database. This one does,
/// and it asserts an ABSENCE by count over the exact reference — a non-200 is
/// not evidence that nothing was written, and neither is the refusal's own kind.
///
/// The dishonest worker is injected through the `R2_WORKER_BIN` override the
/// spawner already reads, so production is not touched. `R2_WORKER_BIN` is
/// process-wide: the suite's `guard()` serializes every test in this file, and
/// the override is removed before anything is asserted.
#[cfg(unix)]
#[tokio::test]
async fn the_r2_mismatch_refusal_persists_neither_snapshot_nor_derivation() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    require_extraction_worker();

    let (origin, _origin_handle) = start_feed_origin().await;
    let port = origin.port();
    let admitting = TestServer::start_with_router(
        &pool,
        reasonbraid_server::api_router_with_acquisition(
            pool.clone(),
            false,
            std::sync::Arc::new(reasonbraid_server::broker::Broker::default()),
            admitting_fetcher(port),
        ),
    )
    .await;
    let base = admitting.base();

    let client = reqwest::Client::new();
    let (status, human) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "r2-mismatch-human" }),
    )
    .await;
    assert_eq!(status, 200, "the human enrolls: {human}");
    let human_id = human["principal_id"].as_str().unwrap().to_string();
    let reference = submit_hinted(
        &client,
        &base,
        &human_id,
        &format!("http://127.0.0.1:{port}/feed.xml"),
    )
    .await;

    // A well-formed response for a document nobody supplied. It is well-formed
    // deliberately: a malformed reply would be refused by the parser, which is
    // a different refusal and would not exercise the digest binding at all.
    let stub = control_scratch("mismatch-worker.sh");
    std::fs::write(
        &stub,
        "#!/bin/sh\ncat > /dev/null\nprintf '%s\\n' '{\"parent_digest\":\
         \"sha256:0000000000000000000000000000000000000000000000000000000000000000\",\
         \"chunks\":[{\"digest\":\"sha256:aa\",\"text\":\"another document\"}],\
         \"excluded\":[],\"extractor_version\":\"0.1.0\"}'\n",
    )
    .expect("the stub is written");
    let mut permissions = std::fs::metadata(&stub)
        .expect("stub metadata")
        .permissions();
    std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o700);
    std::fs::set_permissions(&stub, permissions).expect("the stub is executable");

    let shipped_schemes = widen_r2_to_http(&pool).await;
    // The whole-table counts, so a row written under ANY reference is caught —
    // not only one written under this reference's id.
    let counts = |pool: PgPool| async move {
        let snapshots: i64 = sqlx::query_scalar("SELECT count(*) FROM evidence_snapshots")
            .fetch_one(&pool)
            .await
            .expect("count every snapshot");
        let derivations: i64 = sqlx::query_scalar("SELECT count(*) FROM derivations")
            .fetch_one(&pool)
            .await
            .expect("count every derivation");
        (snapshots, derivations)
    };
    let before = counts(pool.clone()).await;

    std::env::set_var("R2_WORKER_BIN", &stub);
    let (status, refused) = post(
        &client,
        &base,
        &format!("/v1/resources/{reference}/resolve"),
        &human_id,
        &json!({ "required_sandbox": "process", "required_egress": "listed" }),
    )
    .await;
    std::env::remove_var("R2_WORKER_BIN");
    restore_r2_schemes(&pool, &shipped_schemes).await;
    std::fs::remove_file(&stub).expect("the control removes its own stub");

    let after = counts(pool.clone()).await;

    assert_eq!(status, 200, "the resolution answers: {refused}");
    assert_eq!(
        refused["resolvers"],
        json!(["r2-extract-worker"]),
        "the R2 pack ranked the reference: {refused}"
    );
    // The acquisition SUCCEEDED and the extraction was refused: this control
    // fails for the wrong reason if it never got past the destination gate.
    assert_eq!(
        refused["acquisition_error"]["kind"],
        json!("extraction_source_mismatch"),
        "a response for other bytes is refused by name: {refused}"
    );
    let message = refused["acquisition_error"]["message"].as_str().unwrap();
    let served_digest = reasonbraid_server::fetcher::digest_sha256_hex(SERVED_FEED.as_bytes());
    assert!(
        message.contains(&served_digest) && message.contains("sha256:0000000000000000"),
        "the refusal names both digests: {refused}"
    );
    assert!(
        refused["acquisition"].is_null(),
        "a refusal is not an acquisition: {refused}"
    );

    // The absence, by count over the exact reference AND over the whole store.
    let reference_snapshots: i64 =
        sqlx::query_scalar("SELECT count(*) FROM evidence_snapshots WHERE reference_id = $1")
            .bind(&reference)
            .fetch_one(&pool)
            .await
            .expect("count this reference's snapshots");
    assert_eq!(
        reference_snapshots, 0,
        "the refused reference has no evidence snapshot"
    );
    let reference_derivations: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM derivations d \
         JOIN evidence_snapshots s ON s.snapshot_id = d.parent_snapshot_id \
         WHERE s.reference_id = $1",
    )
    .bind(&reference)
    .fetch_one(&pool)
    .await
    .expect("count this reference's derivations");
    assert_eq!(
        reference_derivations, 0,
        "the refused reference has no derivation"
    );
    assert_eq!(
        before, after,
        "the refusal wrote nothing anywhere: snapshots/derivations before vs after"
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
    // ⚠️ `required_egress: "any"`, changed from `"listed"` by
    // `SIGNOFF-REPAIR.7.3.6.4` and not to make a repair pass. RX declares
    // `egress_class: "any"`, and the egress claim is a MAXIMUM (ADR-018), so a
    // caller permitting at most `listed` must NOT be handed a pack that
    // declares no bound — that is the repair, and this call is a caller who
    // does permit it. ⛔ Recorded rather than quietly edited: under the old
    // `declared >= required` test this line passed `"listed"` and got RX,
    // which is the defect seen from the test suite's side.
    // 🔎 `.7.3.6.5` may yet find RX's `any` to be a MISDESCRIPTION — the pack
    // performs no egress at all — in which case this line changes again, for a
    // different reason.
    let response = client
        .post(format!("{base}/v1/resources/{agent_id}/resolve"))
        .header(PRINCIPAL_HEADER, &human_id)
        .json(&json!({ "required_sandbox": "none", "required_egress": "any" }))
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

/// `credential_binding_ref` SELECTS a credential, and the reference row it
/// lived on is SHARED (`SIGNOFF-REPAIR.11.14.3.10`). `UNIQUE (original_locator,
/// expected_digest)` means a second tenant registering the same pair replays
/// the FIRST tenant's row — so the binding came with it, and the R5 arm handed
/// one tenant's credential to another tenant's acquisition. ROADMAP §16.3
/// invariant 5: *target credentials are selected only after authorization for
/// the concrete target and action.*
///
/// The selection now lives on the tenant's OWN registration, so the four arms
/// read: the owner reaches its own binding; the replaying tenant that named
/// none is not even ROUTED to the credential pack (it used to reach the
/// owner's); a tenant that names its own binding gets ITS name back in the
/// refusal; and the owner is undisturbed by either — two tenants hold two
/// different bindings for one shared pair, which the pair key previously made
/// impossible.
///
/// ⭐ The arms DISCRIMINATE by construction, and each names WHICH signal carries
/// it. Arm 2 turns on the RESOLVER, because the binding is a ranking input:
/// binding-less ranks the `none`-class packs, so the credential pack is never
/// selected and the loopback refusal that comes back is R0's. Arms 1, 3 and 4
/// turn on the error KIND and its message: the owner's binding is registered
/// with the broker and the stranger's is not, so `destination_refused` (reached
/// only once a credential resolved) and `credential_unavailable` (the broker
/// refusing, quoting the binding it was asked for) cannot be confused.
#[tokio::test]
async fn a_credential_binding_is_not_inherited_by_another_tenant() {
    let _guard = guard().await;
    let Some(pool) = pool().await else {
        return;
    };
    reasonbraid_server::sync_gated_entries(&pool, true)
        .await
        .expect("the gate opens");

    let broker = std::sync::Arc::new(reasonbraid_server::broker::Broker::default());
    // Only the OWNER's binding exists in the broker. The second tenant's name
    // is deliberately absent, so a refusal that names it proves which binding
    // was consulted.
    broker.register(
        "cred_owner_only",
        reasonbraid_server::broker::Credential::new("owner-token-read", "tok_OWNER_SECRET"),
    );
    let server = TestServer::start_gated(&pool, true, broker).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, owner) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "binding-owner" }),
    )
    .await;
    assert_eq!(status, 200, "the owner enrolls: {owner}");
    let owner_id = owner["principal_id"].as_str().unwrap().to_string();
    let owner_tenant = owner["tenant_id"].as_str().unwrap().to_string();

    let (status, stranger) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "binding-stranger" }),
    )
    .await;
    assert_eq!(status, 200, "the stranger enrolls: {stranger}");
    let stranger_id = stranger["principal_id"].as_str().unwrap().to_string();
    let stranger_tenant = stranger["tenant_id"].as_str().unwrap().to_string();
    assert_ne!(
        owner_tenant, stranger_tenant,
        "the two principals are two TENANTS — otherwise this control proves nothing"
    );

    const LOCATOR: &str = "https://127.0.0.1/shared-private";
    let submit = |principal: String, binding: Option<&'static str>| {
        let client = client.clone();
        let base = base.clone();
        async move {
            let mut body = json!({ "original_locator": LOCATOR, "scheme": "https" });
            if let Some(binding) = binding {
                body["credential_binding_ref"] = json!(binding);
            }
            let response = client
                .post(format!("{base}/v1/resources"))
                .header(PRINCIPAL_HEADER, &principal)
                .json(&body)
                .send()
                .await
                .expect("submit request");
            let status = response.status().as_u16();
            let value: Value = response.json().await.expect("submit json");
            (status, value)
        }
    };
    let resolve = |principal: String, resource_id: String| {
        let client = client.clone();
        let base = base.clone();
        async move {
            let response = client
                .post(format!("{base}/v1/resources/{resource_id}/resolve"))
                .header(PRINCIPAL_HEADER, &principal)
                .json(&json!({ "required_sandbox": "none", "required_egress": "listed" }))
                .send()
                .await
                .expect("resolve request");
            response.json::<Value>().await.expect("resolve json")
        }
    };

    let (status, owned) = submit(owner_id.clone(), Some("cred_owner_only")).await;
    assert_eq!(status, 200, "the owner's reference submits: {owned}");
    let resource_id = owned["resource_id"].as_str().unwrap().to_string();

    // The stranger names the SAME pair and NO binding: the content-addressed
    // row replays, which is the point — one row, two tenants.
    let (status, replayed) = submit(stranger_id.clone(), None).await;
    assert_eq!(status, 200, "the stranger's reference submits: {replayed}");
    assert_eq!(
        replayed["resource_id"].as_str().unwrap(),
        resource_id,
        "the stranger REPLAYS the owner's shared row — without that this control \
         is two rows and proves nothing: {replayed}"
    );

    // Arm 1 — the owner reaches its own binding: the broker resolved it, so the
    // refusal comes from the loopback pre-flight AFTER the credential attached.
    let resolved = resolve(owner_id.clone(), resource_id.clone()).await;
    assert_eq!(
        resolved["resolvers"],
        json!(["r5-credential-broker"]),
        "the R5 pack ranks the owner's binding-carrying reference: {resolved}"
    );
    assert_eq!(
        resolved["acquisition_error"]["kind"],
        json!("destination_refused"),
        "the owner's OWN binding resolved and the loopback refused it: {resolved}"
    );

    // Arm 2 — THE DEFECT, and the repair reaches one layer FURTHER than denying
    // the credential. `resolvers::resolve` takes the binding as a RANKING input:
    // a binding ranks only the `credential`-class packs, and a binding-less
    // reference ranks only the `none`-class ones. So a stranger that names no
    // binding is not merely refused a credential — the credential pack is never
    // selected for it at all, and the refusal it gets is the plain R0 fetcher's.
    //
    // ⭐ The RESOLVER is the discriminator here, not the error kind: both the
    // authenticated and the unauthenticated path end at the same loopback
    // `destination_refused`, and only `resolvers` says which one ran. Before the
    // repair this arm read `["r5-credential-broker"]` — the stranger driving an
    // authenticated acquisition with the OWNER's credential.
    let resolved = resolve(stranger_id.clone(), resource_id.clone()).await;
    assert_eq!(
        resolved["resolvers"],
        json!(["r0-https-fetcher"]),
        "the stranger named NO binding, so no credential pack may be ranked for \
         it — `r5-credential-broker` here means it reached the owner's binding: \
         {resolved}"
    );

    // Arm 3 — the stranger names its OWN binding, which the broker does not
    // hold. The refusal quotes that name, so the row consulted was the
    // stranger's registration and not the owner's.
    let (status, own_binding) = submit(stranger_id.clone(), Some("cred_stranger_only")).await;
    assert_eq!(
        status, 200,
        "the stranger re-states its reference: {own_binding}"
    );
    assert_eq!(
        own_binding["resource_id"].as_str().unwrap(),
        resource_id,
        "still the one shared row: {own_binding}"
    );
    let resolved = resolve(stranger_id.clone(), resource_id.clone()).await;
    assert_eq!(
        resolved["acquisition_error"]["kind"],
        json!("credential_unavailable"),
        "the stranger's own binding is unknown to the broker: {resolved}"
    );
    assert!(
        resolved["acquisition_error"]["message"]
            .as_str()
            .unwrap()
            .contains("cred_stranger_only"),
        "the refusal names the STRANGER's binding, proving whose row was read: {resolved}"
    );

    // Arm 4 — and the owner is undisturbed by all of it. Two tenants now hold
    // two different bindings for ONE shared pair, which `UNIQUE (original_locator,
    // expected_digest)` previously made impossible.
    let resolved = resolve(owner_id, resource_id).await;
    assert_eq!(
        resolved["acquisition_error"]["kind"],
        json!("destination_refused"),
        "the stranger's registration did not overwrite the owner's binding: {resolved}"
    );
}

/// The evidence snapshot store (PHASE-4.6.1): the typed submission verifies
/// the content-addressing (the bytes MUST hash to the declared digest), the
/// same reference + digest is the REPLAY, and the deletion is the tombstone
/// + the reason — never a silent disappearance.
#[tokio::test]
async fn the_snapshot_store_roundtrips_replays_and_withdraws() {
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

    // A store fault is the SERVER's problem (`.7.4.2`). Before this repair
    // every storage failure was mapped to `ReferenceMissing` and rendered as
    // HTTP 400 `invalid_command`, telling the caller its own input was wrong.
    // The trigger is dropped BEFORE the assertions run, so a failing
    // expectation cannot leave this shared database rejecting snapshots.
    sqlx::raw_sql(
        "CREATE FUNCTION public.evidence_test_gate() RETURNS trigger LANGUAGE plpgsql AS $$ \
         BEGIN RAISE EXCEPTION 'injected snapshot storage failure'; END; $$; \
         CREATE TRIGGER evidence_test_gate BEFORE INSERT ON public.evidence_snapshots \
         FOR EACH ROW EXECUTE FUNCTION public.evidence_test_gate()",
    )
    .execute(&pool)
    .await
    .expect("install the injected storage fault");
    let faulted = client
        .post(format!("{base}/v1/snapshots"))
        .header(PRINCIPAL_HEADER, &human_id)
        .json(&snapshot_body(&digest))
        .send()
        .await
        .expect("snapshot request under the injected fault");
    let faulted_status = faulted.status().as_u16();
    let faulted_body: Value = faulted.json().await.unwrap();
    sqlx::raw_sql(
        "DROP TRIGGER IF EXISTS evidence_test_gate ON public.evidence_snapshots; \
         DROP FUNCTION IF EXISTS public.evidence_test_gate()",
    )
    .execute(&pool)
    .await
    .expect("remove the injected storage fault");
    assert_eq!(
        faulted_status, 500,
        "a store fault is the server's, not the caller's: {faulted_body}"
    );
    assert_eq!(
        faulted_body["code"],
        json!("dependency_unavailable"),
        "the store fault names itself: {faulted_body}"
    );
    assert_ne!(
        faulted_body["code"],
        json!("invalid_command"),
        "a store fault must never be reported as a bad request: {faulted_body}"
    );

    // And the repair did not turn every refusal into a 500: a genuinely
    // unknown reference is still the caller's error.
    let absent = client
        .post(format!("{base}/v1/snapshots"))
        .header(PRINCIPAL_HEADER, &human_id)
        .json(&{
            let mut body = snapshot_body(&digest);
            body["reference_id"] = json!("res_00000000-0000-7000-8000-0000000000ff");
            body
        })
        .send()
        .await
        .expect("snapshot request for an absent reference");
    let absent_status = absent.status().as_u16();
    let absent_body: Value = absent.json().await.unwrap();
    assert_eq!(
        absent_status, 400,
        "an absent reference is still the caller's error: {absent_body}"
    );
    assert_eq!(absent_body["code"], json!("invalid_command"));

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

    // ⚠️ CHANGED DELIBERATELY BY `SIGNOFF-REPAIR.7.4.4`, and the change is named
    // rather than absorbed. This block used to assert that `DELETE` tombstones
    // the row and that the caller then reads it back tombstoned. That verb now
    // WITHDRAWS the caller's citation, because the row is shared and tombstoning
    // it removed evidence other tenants cite. Three assertions moved:
    //
    //   * the reply field is `withdrawn`, not `tombstoned` — reporting a
    //     tombstone that did not happen would be the worse compatibility choice;
    //   * the caller's read is now 404, because a withdrawn citation is not a
    //     citation and a non-citer read is ABSENT rather than forbidden;
    //   * `deleted_at`/`deletion_reason` are asserted where they now belong —
    //     on the site-operator path, in
    //     `one_tenant_does_not_tombstone_evidence_another_tenant_cites`.
    //
    // ⛔ Nothing was deleted to make this green: the tombstone's own assertions
    // live in that control, over the authority that now performs it.
    let response = client
        .delete(format!("{base}/v1/snapshots/{snapshot_id}"))
        .header(PRINCIPAL_HEADER, &human_id)
        .json(&json!({ "reason": "this tenant no longer relies on it" }))
        .send()
        .await
        .expect("withdrawal request");
    assert_eq!(response.status().as_u16(), 200, "the withdrawal lands");
    let withdrawn: Value = response.json().await.unwrap();
    assert_eq!(withdrawn["withdrawn"], json!(true));

    let (status, stored) = get(
        &client,
        &base,
        &format!("/v1/snapshots/{snapshot_id}"),
        &human_id,
    )
    .await;
    assert_eq!(
        status, 404,
        "a withdrawn citation is not a citation, so the row is absent: {stored}"
    );
}

/// `SIGNOFF-REPAIR.7.4.4` — a snapshot two tenants cite is ONE row.
///
/// `resource_references` is UNIQUE on `(original_locator, expected_digest)` and
/// `snapshot_objects` is keyed by `digest` alone (ADR-011), so a second tenant
/// naming the same pair REPLAYS the first tenant's row and `record_citation`
/// adds it to the citer set. Sharing the row is the design, not the defect.
///
/// The defect is that `DELETE /v1/snapshots/{id}` tombstones that shared row.
/// `.11.14.1` bound the verb to the CITING tenant, which stopped a stranger
/// reaching it and does not reach this: both tenants here are citers. So
/// tenant A's deletion removes the evidence from tenant B's surface and stamps
/// B's receipt with A's reason — and irreversibly, since the `UPDATE` carries
/// `AND deleted_at IS NULL` and nothing in the tree clears the column.
///
/// This control asserts the REPAIRED behaviour: A withdraws its own citation,
/// B's reliance is untouched. Before the repair it fails at the final read,
/// with A's reason visible on B's row.
#[tokio::test]
async fn one_tenant_does_not_tombstone_evidence_another_tenant_cites() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, a) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "cocite-a" }),
    )
    .await;
    assert_eq!(status, 200, "tenant A enrolls: {a}");
    let a_id = a["principal_id"].as_str().unwrap().to_string();
    let a_tenant = a["tenant_id"].as_str().unwrap().to_string();

    let (status, b) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "cocite-b" }),
    )
    .await;
    assert_eq!(status, 200, "tenant B enrolls: {b}");
    let b_id = b["principal_id"].as_str().unwrap().to_string();
    let b_tenant = b["tenant_id"].as_str().unwrap().to_string();
    assert_ne!(
        a_tenant, b_tenant,
        "the two principals are two TENANTS — otherwise this control proves nothing"
    );

    const LOCATOR: &str = "https://example.org/co-cited-evidence";
    let submit_reference = |principal: String| {
        let client = client.clone();
        let base = base.clone();
        async move {
            let response = client
                .post(format!("{base}/v1/resources"))
                .header(PRINCIPAL_HEADER, &principal)
                .json(&json!({ "original_locator": LOCATOR, "scheme": "https" }))
                .send()
                .await
                .expect("submit request");
            let status = response.status().as_u16();
            (status, response.json::<Value>().await.expect("submit json"))
        }
    };

    let (status, a_ref) = submit_reference(a_id.clone()).await;
    assert_eq!(status, 200, "A's reference submits: {a_ref}");
    let reference_id = a_ref["resource_id"].as_str().unwrap().to_string();
    let (status, b_ref) = submit_reference(b_id.clone()).await;
    assert_eq!(status, 200, "B's reference submits: {b_ref}");
    assert_eq!(
        b_ref["resource_id"].as_str().unwrap(),
        reference_id,
        "the pair key replays: one reference row, two tenants"
    );

    let bytes = b"evidence two tenants rely on";
    let digest = reasonbraid_server::fetcher::digest_sha256_hex(bytes);
    let submit_snapshot = |principal: String| {
        let client = client.clone();
        let base = base.clone();
        let reference_id = reference_id.clone();
        let digest = digest.clone();
        async move {
            let response = client
                .post(format!("{base}/v1/snapshots"))
                .header(PRINCIPAL_HEADER, &principal)
                .json(&json!({
                    "reference_id": reference_id,
                    "original_locator": LOCATOR,
                    "final_locator": LOCATOR,
                    "resolver_id": "r0-https-fetcher",
                    "resolver_version": "0.1.0",
                    "raw_digest": digest,
                    "byte_length": bytes.len(),
                    "media_type": "text/plain",
                    "bytes_base64": base64_std(bytes),
                }))
                .send()
                .await
                .expect("snapshot request");
            let status = response.status().as_u16();
            (
                status,
                response.json::<Value>().await.expect("snapshot json"),
            )
        }
    };

    let (status, a_snap) = submit_snapshot(a_id.clone()).await;
    assert_eq!(status, 200, "A's snapshot submits: {a_snap}");
    assert_eq!(a_snap["replay"], json!(false));
    let snapshot_id = a_snap["snapshot_id"].as_str().unwrap().to_string();

    let (status, b_snap) = submit_snapshot(b_id.clone()).await;
    assert_eq!(status, 200, "B's snapshot submits: {b_snap}");
    assert_eq!(
        b_snap["replay"],
        json!(true),
        "the digest replays: ONE row, two citers — the premise of this control"
    );
    assert_eq!(b_snap["snapshot_id"], json!(snapshot_id));

    // Both citers read it live before anybody deletes anything.
    for (who, principal) in [("A", &a_id), ("B", &b_id)] {
        let (status, stored) = get(
            &client,
            &base,
            &format!("/v1/snapshots/{snapshot_id}"),
            principal,
        )
        .await;
        assert_eq!(status, 200, "{who} reads the shared snapshot: {stored}");
        assert_eq!(stored["deleted_at"], Value::Null, "{who}: {stored}");
    }

    // A is done with it and says so.
    const A_REASON: &str = "tenant A no longer relies on this";
    let response = client
        .delete(format!("{base}/v1/snapshots/{snapshot_id}"))
        .header(PRINCIPAL_HEADER, &a_id)
        .json(&json!({ "reason": A_REASON }))
        .send()
        .await
        .expect("A's delete request");
    assert_eq!(response.status().as_u16(), 200, "A's withdrawal lands");

    // ⛔ THE POINT, ASSERTED FIRST so a red lands on the sentence this leaf is
    // about. B never asked for anything and must not have been touched.
    let (status, b_after) = get(
        &client,
        &base,
        &format!("/v1/snapshots/{snapshot_id}"),
        &b_id,
    )
    .await;
    assert_eq!(status, 200, "B still cites the snapshot: {b_after}");
    assert_eq!(
        b_after["deleted_at"],
        Value::Null,
        "B's reliance survives A's withdrawal — one tenant does not tombstone \
         a row another cites: {b_after}"
    );
    assert_eq!(
        b_after["deletion_reason"],
        Value::Null,
        "B's receipt never carries A's reason: {b_after}"
    );

    // A's own surface: it withdrew, so the row is no longer A's to read. A
    // non-citer read is ABSENT rather than forbidden — the same rule
    // `.11.14.1` established, so withdrawing returns A to a stranger's view.
    let (status, a_after) = get(
        &client,
        &base,
        &format!("/v1/snapshots/{snapshot_id}"),
        &a_id,
    )
    .await;
    assert_eq!(
        status, 404,
        "A withdrew its citation, so the snapshot is absent for A: {a_after}"
    );

    // A cites the same bytes again: the withdrawal is an episode, not a wall.
    let (status, a_recite) = submit_snapshot(a_id.clone()).await;
    assert_eq!(status, 200, "A re-cites: {a_recite}");
    assert_eq!(a_recite["snapshot_id"], json!(snapshot_id));
    let (status, a_restored) = get(
        &client,
        &base,
        &format!("/v1/snapshots/{snapshot_id}"),
        &a_id,
    )
    .await;
    assert_eq!(status, 200, "re-citing restores A's read: {a_restored}");
    assert_eq!(a_restored["deleted_at"], Value::Null);

    // ── The authority MOVED; it did not vanish ────────────────────────────
    // Tombstoning a shared row is now a site act. An ordinary tenant is
    // refused, and B's evidence stays live while it is.
    let response = client
        .post(format!("{base}/v1/snapshots/{snapshot_id}/tombstone"))
        .header(PRINCIPAL_HEADER, &a_id)
        .json(&json!({ "reason": "A would like this gone for everybody" }))
        .send()
        .await
        .expect("unauthorized tombstone request");
    assert_eq!(
        response.status().as_u16(),
        403,
        "a tenant holds no site authority over a shared row"
    );
    let (status, still_live) = get(
        &client,
        &base,
        &format!("/v1/snapshots/{snapshot_id}"),
        &b_id,
    )
    .await;
    assert_eq!(status, 200, "{still_live}");
    assert_eq!(
        still_live["deleted_at"],
        Value::Null,
        "the refused tombstone wrote nothing: {still_live}"
    );

    // With the grant, the same request lands and every citer sees it — which
    // is what makes it a site act rather than a tenant one.
    site_fixture::provision(
        &pool,
        &a_id,
        &[reasonbraid_server::site_authority::Action::EvidenceExpire],
    )
    .await;
    const OPERATOR_REASON: &str = "the source withdrew the document";
    let response = client
        .post(format!("{base}/v1/snapshots/{snapshot_id}/tombstone"))
        .header(PRINCIPAL_HEADER, &a_id)
        .json(&json!({ "reason": OPERATOR_REASON }))
        .send()
        .await
        .expect("authorized tombstone request");
    assert_eq!(
        response.status().as_u16(),
        200,
        "the site act lands with the grant"
    );
    let receipt: Value = response.json().await.expect("receipt json");
    assert_eq!(receipt["tombstoned"], json!(true), "{receipt}");
    assert_eq!(receipt["snapshot_id"], json!(snapshot_id), "{receipt}");

    let (status, b_final) = get(
        &client,
        &base,
        &format!("/v1/snapshots/{snapshot_id}"),
        &b_id,
    )
    .await;
    assert_eq!(status, 200, "the tombstoned row stays readable: {b_final}");
    assert!(
        b_final["deleted_at"].is_string(),
        "a site tombstone reaches every citer: {b_final}"
    );
    assert_eq!(
        b_final["deletion_reason"],
        json!(OPERATOR_REASON),
        "the operator's own reason is on the row, not a canned one: {b_final}"
    );

    eprintln!(
        "co-citation: one row, two citers; A's withdrawal left B's read live and \
         unreasoned and made the row absent for A; re-citing restored it; a tenant's \
         tombstone was refused 403 writing nothing, and the granted site act \
         tombstoned it for both with the operator's own reason"
    );
}

/// Standard base64, for test bodies only — the API's decoder is under test.
fn base64_std(bytes: &[u8]) -> String {
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
    let claim_side = claim_side.as_array().expect("the array");
    assert_eq!(claim_side.len(), 1, "{claim_side:?}");
    // `.11.14.3.3`: this route is the NON-deliberation writer, so its rows say
    // so. `clm_budget` is the free-text identifier that made this namespace a
    // guessable one; the label is what keeps it from being mistaken for a claim
    // digest a thread's gates admitted.
    assert_eq!(
        claim_side[0]["claim_namespace"],
        json!("external"),
        "the standalone route writes the external namespace: {claim_side:?}"
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
/// TOMBSTONES the expired classes (the sweep is driven directly — the
/// caller's clock is not on the wire, `.7.4.3`),
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
                assert_eq!(response.status().as_u16(), 200, "snapshot submission");
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
    let audit = submit_snapshot(b"the audit bytes", "audit".to_owned(), None).await;
    let audit_id = audit["snapshot_id"].as_str().unwrap().to_string();

    // Expiry is measured from the stored creation time, not a calendar date
    // chosen when this test was written. Observe PostgreSQL's actual values.
    let created: std::collections::BTreeMap<String, chrono::DateTime<chrono::Utc>> =
        sqlx::query_as::<_, (String, chrono::DateTime<chrono::Utc>)>(
            "SELECT snapshot_id, created_at FROM evidence_snapshots WHERE reference_id=$1",
        )
        .bind(&reference_id)
        .fetch_all(&pool)
        .await
        .unwrap()
        .into_iter()
        .collect();
    assert_eq!(created.len(), 3);
    let temporary_due = created[&temporary_id] + chrono::Duration::days(1);
    let standard_due = created[&fresh_id] + chrono::Duration::days(30);
    let tick = chrono::Duration::microseconds(1); // PostgreSQL timestamp precision.
    assert!(temporary_due + tick < standard_due);
    let snapshot_rows = || async {
        sqlx::query_scalar::<_, Value>(
            "SELECT to_jsonb(s) FROM evidence_snapshots s WHERE reference_id=$1 ORDER BY snapshot_id",
        )
        .bind(&reference_id)
        .fetch_all(&pool)
        .await
        .unwrap()
    };
    // The TTL boundaries are a property of `(now, created_at, retention_class)`
    // and are driven directly, because the caller's clock is no longer on the
    // wire: `POST /v1/snapshots/expire-due` used to take an unbounded `at` from
    // the request body, which let any enrolled principal tombstone every
    // tenant's evidence (`SIGNOFF-REPAIR.7.4.3`). Every assertion below is the
    // one it made through HTTP; what the route is responsible for — the site
    // capability, the server clock and the audit record — is measured by
    // `the_retention_sweep_requires_site_authority_and_the_server_clock`.
    let expire_at = |at: chrono::DateTime<chrono::Utc>| {
        let pool = pool.clone();
        async move {
            let mut conn = pool.acquire().await.expect("a connection for the sweep");
            reasonbraid_server::snapshots::expire_due(&mut conn, at)
                .await
                .expect("the retention sweep")
        }
    };

    // The staleness surface lists ONLY the passed horizon.
    let (status, stale) = get(&client, &base, "/v1/snapshots/stale", &human_id).await;
    assert_eq!(status, 200, "the staleness reads: {stale}");
    let stale = stale.as_array().expect("the array");
    assert_eq!(stale.len(), 1, "{stale:?}");
    assert_eq!(stale[0]["snapshot_id"], json!(temporary_id));

    // A TTL must have passed: equality is not expiry. Neither the before nor
    // exact-boundary request may change any snapshot row.
    let original = snapshot_rows().await;
    assert!(original.iter().all(|row| row["deleted_at"].is_null()));
    assert_eq!(expire_at(temporary_due - tick).await, 0);
    assert_eq!(snapshot_rows().await, original);
    assert_eq!(expire_at(temporary_due).await, 0);
    assert_eq!(snapshot_rows().await, original);
    assert_eq!(expire_at(temporary_due + tick).await, 1);
    let expired_temporary = snapshot_rows().await;
    assert_eq!(expire_at(temporary_due + tick).await, 0);
    assert_eq!(snapshot_rows().await, expired_temporary);

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

    // Re-fetching identical bytes does not reset the original retention age.
    let refreshed_created: chrono::DateTime<chrono::Utc> =
        sqlx::query_scalar("SELECT created_at FROM evidence_snapshots WHERE snapshot_id=$1")
            .bind(&fresh_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(refreshed_created, created[&fresh_id]);
    let before_standard = snapshot_rows().await;
    assert_eq!(expire_at(standard_due).await, 0);
    assert_eq!(snapshot_rows().await, before_standard);
    assert_eq!(expire_at(standard_due + tick).await, 1);
    let after_standard = snapshot_rows().await;
    let standard = after_standard
        .iter()
        .find(|row| row["snapshot_id"] == fresh_id)
        .unwrap();
    assert!(standard["deleted_at"].is_string());
    assert_eq!(standard["deletion_reason"], "the retention expired");
    assert_eq!(standard["license"], "MIT OR Apache-2.0");

    // The audit class has no automatic TTL. Advancing this fixture's expiry
    // observation beyond both finite classes must preserve its exact row.
    assert_eq!(
        expire_at(created[&audit_id] + chrono::Duration::days(365)).await,
        0
    );
    assert_eq!(snapshot_rows().await, after_standard);
    let original_audit = original
        .iter()
        .find(|row| row["snapshot_id"] == audit_id)
        .unwrap();
    let final_audit = after_standard
        .iter()
        .find(|row| row["snapshot_id"] == audit_id)
        .unwrap();
    assert_eq!(final_audit, original_audit);
    eprintln!("retention fixture: temporary/standard exact TTL boundaries preserved; after-boundary tombstones 1/1; repeated expiry 0; audit row unchanged; replay retains creation time");
}

/// The tenant binding on the evidence reads (`SIGNOFF-REPAIR.11.14.1`,
/// `docs/decisions/2026-09-16_evidence-is-shared-the-read-is-tenant-bound.md`).
///
/// The snapshot ROW is shared by design: `resource_references` is UNIQUE on
/// `(original_locator, expected_digest)` and `snapshot_objects` is keyed by
/// digest alone, so two tenants citing the same URL at the same digest share
/// one row by construction. What ROADMAP §16.8 requires of such a row is that
/// the decision to DISCLOSE it names the tenant — and that decision was
/// missing. All four read surfaces admitted on ENROLMENT, so
/// `GET /v1/snapshots/stale` returned every tenant's rows to every principal:
/// an enumeration of which documents another tenant acquired, when, through
/// which resolver and under which credential class.
///
/// The control proves a BINDING and not a blackout — tenant A still receives
/// every row tenant A cited, including a row whose bytes tenant B acquired
/// first, because the citation is a SET and the replay records the second
/// citer (`SIGNOFF-REPAIR.6.1.5`'s trap: a tenant-scoped read over an
/// unscoped write hides rows from their own author).
#[tokio::test]
async fn the_evidence_reads_are_bound_to_the_citing_tenant() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    /// One citation: the reference, then the snapshot over its bytes. The
    /// freshness horizon is already past, so the row sits on the staleness
    /// surface the moment it lands.
    async fn cite(
        client: &reqwest::Client,
        base: &str,
        principal: &str,
        locator: &str,
        payload: &[u8],
        fresh_until: &str,
    ) -> (String, String) {
        let response = client
            .post(format!("{base}/v1/resources"))
            .header(PRINCIPAL_HEADER, principal)
            .json(&json!({ "original_locator": locator, "scheme": "https" }))
            .send()
            .await
            .expect("reference request");
        let status = response.status().as_u16();
        let reference: Value = response.json().await.expect("reference json");
        assert_eq!(status, 200, "the reference submits: {reference}");
        let reference_id = reference["resource_id"].as_str().unwrap().to_string();
        let response = client
            .post(format!("{base}/v1/snapshots"))
            .header(PRINCIPAL_HEADER, principal)
            .json(&json!({
                "reference_id": reference_id,
                "original_locator": locator,
                "final_locator": locator,
                "resolver_id": "r0-https-fetcher",
                "resolver_version": "0.1.0",
                "raw_digest": reasonbraid_server::fetcher::digest_sha256_hex(payload),
                "byte_length": payload.len(),
                "media_type": "text/plain",
                "fresh_until": fresh_until,
                "bytes_base64": util::base64(payload),
            }))
            .send()
            .await
            .expect("snapshot request");
        let status = response.status().as_u16();
        let snapshot: Value = response.json().await.expect("snapshot json");
        assert_eq!(status, 200, "the snapshot submits: {snapshot}");
        (
            reference_id,
            snapshot["snapshot_id"].as_str().unwrap().to_string(),
        )
    }

    // Two tenants, by construction: a human enrolment that names no tenant
    // MINTS one (`api.rs::enroll` — `TenantId::new()`), so these two humans
    // cannot share a tenant.
    let (status, alpha) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "evidence-alpha" }),
    )
    .await;
    assert_eq!(status, 200, "tenant A's human enrolls: {alpha}");
    let alpha_id = alpha["principal_id"].as_str().unwrap().to_string();
    let (status, beta) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "evidence-beta" }),
    )
    .await;
    assert_eq!(status, 200, "tenant B's human enrolls: {beta}");
    let beta_id = beta["principal_id"].as_str().unwrap().to_string();
    assert_ne!(
        alpha["tenant_id"], beta["tenant_id"],
        "the fixture needs two DISTINCT tenants: {alpha} / {beta}"
    );

    let past = (chrono::Utc::now() - chrono::Duration::hours(1)).to_rfc3339();
    let alpha_bytes = b"the alpha tenant evidence";
    let beta_bytes = b"the beta tenant evidence";
    let (_, alpha_snapshot) = cite(
        &client,
        &base,
        &alpha_id,
        "https://example.org/alpha-evidence",
        alpha_bytes,
        &past,
    )
    .await;
    let (_, beta_snapshot) = cite(
        &client,
        &base,
        &beta_id,
        "https://example.org/beta-evidence",
        beta_bytes,
        &past,
    )
    .await;
    assert_ne!(alpha_snapshot, beta_snapshot);

    // Tenant B's snapshot carries children on both child surfaces, so a leak
    // there discloses content and not merely an identifier.
    let derived = "the beta tenant derived chunk";
    let (status, derivation) = post(
        &client,
        &base,
        "/v1/derivations",
        &beta_id,
        &json!({
            "parent_snapshot_id": beta_snapshot,
            "derived_kind": "chunk",
            "derived_digest": reasonbraid_server::fetcher::digest_sha256_hex(derived.as_bytes()),
            "content": derived,
        }),
    )
    .await;
    assert_eq!(status, 200, "tenant B's derivation submits: {derivation}");
    let (status, assessment) = post(
        &client,
        &base,
        "/v1/assessments",
        &beta_id,
        &json!({
            "claim_id": "clm_beta",
            "snapshot_id": beta_snapshot,
            "assessment": "supports",
            "author": beta_id,
            "excerpt": "beta tenant evidence",
            "rationale": "the excerpt appears in the acquired bytes",
        }),
    )
    .await;
    assert_eq!(status, 200, "tenant B's assessment submits: {assessment}");

    let stale_ids = |principal: String| {
        let client = client.clone();
        let base = base.clone();
        async move {
            let (status, rows) = get(&client, &base, "/v1/snapshots/stale", &principal).await;
            assert_eq!(status, 200, "the staleness surface reads: {rows}");
            rows.as_array()
                .expect("the stale array")
                .iter()
                .map(|row| row["snapshot_id"].as_str().unwrap().to_string())
                .collect::<Vec<String>>()
        }
    };

    // Surface 1 — the ENUMERATION. This is the finding: before the repair
    // tenant A's list contained tenant B's row, with its locator, its
    // resolver and its credential class.
    let alpha_stale = stale_ids(alpha_id.clone()).await;
    assert!(
        alpha_stale.contains(&alpha_snapshot),
        "tenant A reads its OWN stale row — a binding, not a blackout: {alpha_stale:?}"
    );
    assert!(
        !alpha_stale.contains(&beta_snapshot),
        "tenant A enumerated tenant B's evidence trail: {alpha_stale:?}"
    );
    let beta_stale = stale_ids(beta_id.clone()).await;
    assert!(
        beta_stale.contains(&beta_snapshot),
        "tenant B reads its OWN stale row: {beta_stale:?}"
    );
    assert!(
        !beta_stale.contains(&alpha_snapshot),
        "tenant B enumerated tenant A's evidence trail: {beta_stale:?}"
    );

    // Surfaces 2–4 — the single row and its two child reads. A tenant that
    // did not cite the snapshot must not learn that it exists, so the refusal
    // is the absence (404) rather than a forbidden that confirms the id.
    let (status, body) = get(
        &client,
        &base,
        &format!("/v1/snapshots/{beta_snapshot}"),
        &alpha_id,
    )
    .await;
    assert_eq!(status, 404, "tenant A read tenant B's snapshot: {body}");
    let (status, body) = get(
        &client,
        &base,
        &format!("/v1/snapshots/{beta_snapshot}/derivations"),
        &alpha_id,
    )
    .await;
    assert_eq!(status, 404, "tenant A read tenant B's derivations: {body}");
    let (status, body) = get(
        &client,
        &base,
        &format!("/v1/snapshots/{beta_snapshot}/assessments"),
        &alpha_id,
    )
    .await;
    assert_eq!(status, 404, "tenant A read tenant B's assessments: {body}");

    // The same three surfaces for the row tenant A DID cite: a binding, not
    // a blackout.
    let (status, body) = get(
        &client,
        &base,
        &format!("/v1/snapshots/{alpha_snapshot}"),
        &alpha_id,
    )
    .await;
    assert_eq!(status, 200, "tenant A reads its own snapshot: {body}");
    assert_eq!(body["snapshot_id"], json!(alpha_snapshot));
    let (status, body) = get(
        &client,
        &base,
        &format!("/v1/snapshots/{alpha_snapshot}/derivations"),
        &alpha_id,
    )
    .await;
    assert_eq!(status, 200, "tenant A reads its own derivations: {body}");
    let (status, body) = get(
        &client,
        &base,
        &format!("/v1/snapshots/{alpha_snapshot}/assessments"),
        &alpha_id,
    )
    .await;
    assert_eq!(status, 200, "tenant A reads its own assessments: {body}");

    // The SHARED row, which is why the binding is a citation set and not a
    // column: tenant A cites the locator and digest tenant B acquired first.
    // The reference replays to B's row and the snapshot replays to B's id —
    // and both tenants now read it. A design that stored one owner on the
    // receipt would hide this row from one of its two authors.
    let (replayed_reference, replayed_snapshot) = cite(
        &client,
        &base,
        &alpha_id,
        "https://example.org/beta-evidence",
        beta_bytes,
        &past,
    )
    .await;
    assert_eq!(
        replayed_snapshot, beta_snapshot,
        "the shared row replays to one id: {replayed_reference}"
    );
    let alpha_stale = stale_ids(alpha_id.clone()).await;
    assert!(
        alpha_stale.contains(&beta_snapshot),
        "tenant A cited the shared row and must read it: {alpha_stale:?}"
    );
    let beta_stale = stale_ids(beta_id.clone()).await;
    assert!(
        beta_stale.contains(&beta_snapshot),
        "tenant B's own citation survives tenant A's: {beta_stale:?}"
    );
    let (status, body) = get(
        &client,
        &base,
        &format!("/v1/snapshots/{beta_snapshot}"),
        &alpha_id,
    )
    .await;
    assert_eq!(status, 200, "tenant A reads the row it now cites: {body}");

    // The enrolment gate stays where it was: an unenrolled principal reads
    // nothing at all, and the tenant binding did not replace that refusal.
    // ⚠️ The principal is WELL-FORMED and merely unknown. An id of the wrong
    // shape is refused by `resolve_principal` before any gate runs, so a
    // malformed value here would assert 401 while measuring the header parser.
    let (status, body) = get(&client, &base, "/v1/snapshots/stale", STRANGER).await;
    assert_eq!(
        status, 403,
        "an unenrolled principal reads no staleness: {body}"
    );
    assert_eq!(body["code"], json!("unauthorized"), "{body}");
    // The two refusals are distinct and the control names both, because
    // conflating them is how the first version of this leg passed for the
    // wrong reason: a malformed principal is `unauthenticated` (401) from
    // `resolve_principal`, and a well-formed unenrolled one is `unauthorized`
    // (403) from the gate.
    let (status, body) = get(
        &client,
        &base,
        "/v1/snapshots/stale",
        "hum_00000000000000000000000000000000",
    )
    .await;
    assert_eq!(
        status, 401,
        "a malformed principal is unauthenticated: {body}"
    );
    assert_eq!(body["code"], json!("unauthenticated"), "{body}");

    let citations: i64 =
        sqlx::query_scalar("SELECT count(*) FROM evidence_citations WHERE snapshot_id = $1")
            .bind(&beta_snapshot)
            .fetch_one(&pool)
            .await
            .expect("count the shared row's citations");
    assert_eq!(citations, 2, "the shared row carries BOTH citing tenants");
    eprintln!(
        "evidence binding: 4 read surfaces bound to the citing tenant; foreign reads 404/absent; own reads 200; the shared row carries {citations} citations"
    );
}

/// The retention sweep is a SITE-OPERATOR act (`SIGNOFF-REPAIR.7.4.3`).
///
/// `POST /v1/snapshots/expire-due` admitted any enrolled principal and read its
/// cutoff straight from the request body with no upper bound, while
/// `snapshots::expire_due` carries no tenant predicate. One request naming a
/// far-future instant therefore tombstoned every tenant's live `standard` and
/// `temporary` evidence — irreversibly, since nothing in the product clears
/// `deleted_at`.
///
/// Which rows are DUE is a property of the shared row's `retention_class`, not
/// of any one tenant's citation, so the sweep cannot be a tenant verb. It takes
/// the `evidence_expire` site capability, on
/// `docs/decisions/2026-09-09_site-operator-authority.md`'s shape, and it runs
/// on the server's own clock.
#[tokio::test]
async fn the_retention_sweep_requires_site_authority_and_the_server_clock() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, alpha) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "sweep-alpha" }),
    )
    .await;
    assert_eq!(status, 200, "tenant A's human enrolls: {alpha}");
    let alpha_id = alpha["principal_id"].as_str().unwrap().to_string();
    let (status, beta) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "sweep-beta" }),
    )
    .await;
    assert_eq!(status, 200, "tenant B's human enrolls: {beta}");
    let beta_id = beta["principal_id"].as_str().unwrap().to_string();
    assert_ne!(
        alpha["tenant_id"], beta["tenant_id"],
        "two distinct tenants"
    );

    // Tenant B's evidence, live and nowhere near its retention horizon.
    let submit = |principal: String, locator: &'static str, payload: &'static [u8]| {
        let client = client.clone();
        let base = base.clone();
        async move {
            let response = client
                .post(format!("{base}/v1/resources"))
                .header(PRINCIPAL_HEADER, &principal)
                .json(&json!({ "original_locator": locator, "scheme": "https" }))
                .send()
                .await
                .expect("reference request");
            let reference: Value = response.json().await.expect("reference json");
            let reference_id = reference["resource_id"].as_str().unwrap().to_string();
            let response = client
                .post(format!("{base}/v1/snapshots"))
                .header(PRINCIPAL_HEADER, &principal)
                .json(&json!({
                    "reference_id": reference_id,
                    "original_locator": locator,
                    "final_locator": locator,
                    "resolver_id": "r0-https-fetcher",
                    "resolver_version": "0.1.0",
                    "raw_digest": reasonbraid_server::fetcher::digest_sha256_hex(payload),
                    "byte_length": payload.len(),
                    "media_type": "text/plain",
                    "retention_class": "standard",
                    "bytes_base64": util::base64(payload),
                }))
                .send()
                .await
                .expect("snapshot request");
            let status = response.status().as_u16();
            let snapshot: Value = response.json().await.expect("snapshot json");
            assert_eq!(status, 200, "the snapshot submits: {snapshot}");
            snapshot["snapshot_id"].as_str().unwrap().to_string()
        }
    };
    let beta_snapshot = submit(
        beta_id.clone(),
        "https://example.org/sweep-beta",
        b"the beta tenant retained evidence",
    )
    .await;
    let alpha_snapshot = submit(
        alpha_id.clone(),
        "https://example.org/sweep-alpha",
        b"the alpha tenant retained evidence",
    )
    .await;

    let live = |snapshot_id: String| {
        let pool = pool.clone();
        async move {
            let deleted: Option<chrono::DateTime<chrono::Utc>> = sqlx::query_scalar(
                "SELECT deleted_at FROM evidence_snapshots WHERE snapshot_id = $1",
            )
            .bind(&snapshot_id)
            .fetch_one(&pool)
            .await
            .expect("read the tombstone state");
            deleted.is_none()
        }
    };
    assert!(live(beta_snapshot.clone()).await, "B's row starts live");
    assert!(live(alpha_snapshot.clone()).await, "A's row starts live");

    // The finding, replayed verbatim: tenant A, holding nothing but an
    // enrolment, names the year 3000. On the unrepaired route this returned
    // `200 {"tombstoned":2}` — both tenants' rows, on a clock A invented.
    // The clock is no longer a field this route has, so the request is refused
    // at the body: the attack is not expressible rather than merely denied.
    let response = client
        .post(format!("{base}/v1/snapshots/expire-due"))
        .header(PRINCIPAL_HEADER, &alpha_id)
        .json(&json!({ "at": "3000-01-01T00:00:00Z" }))
        .send()
        .await
        .expect("sweep request");
    let status = response.status().as_u16();
    let body: Value = response.json().await.expect("sweep json");
    assert_eq!(status, 400, "the caller's clock reached the sweep: {body}");
    assert!(
        live(beta_snapshot.clone()).await,
        "tenant A tombstoned tenant B's live evidence"
    );
    assert!(
        live(alpha_snapshot.clone()).await,
        "tenant A tombstoned its own live evidence on a clock it invented"
    );

    // And a WELL-FORMED sweep from the same principal is refused on authority,
    // which is the gate the clock was hiding: on the unrepaired route this body
    // returned 200 for any enrolled principal.
    let (status, body) = post(
        &client,
        &base,
        "/v1/snapshots/expire-due",
        &alpha_id,
        &json!({ "reason": "a sweep tenant A is not entitled to run" }),
    )
    .await;
    assert_eq!(
        status, 403,
        "an enrolled principal without site authority swept the site: {body}"
    );
    assert_eq!(body["code"], json!("site_authority_required"));
    assert!(
        body["audit_id"].is_string(),
        "the denial is recorded: {body}"
    );
    assert!(
        live(beta_snapshot.clone()).await,
        "B's row survives the denial"
    );
    assert!(
        live(alpha_snapshot.clone()).await,
        "A's row survives the denial"
    );

    // A principal enrolled nowhere is refused by the same gate, with the same
    // answer. The route deliberately does not distinguish "not enrolled" from
    // "not authorized": site authority is independent of tenant enrolment by
    // design, and a separate refusal would report who is enrolled.
    let (status, body) = post(
        &client,
        &base,
        "/v1/snapshots/expire-due",
        STRANGER,
        &json!({ "reason": "a sweep from nowhere" }),
    )
    .await;
    assert_eq!(status, 403, "an unknown principal swept the site: {body}");
    assert_eq!(body["code"], json!("site_authority_required"));
    assert!(
        body["audit_id"].is_string(),
        "the denial is recorded: {body}"
    );

    // The authority is a site-operator grant for this action, issued only
    // through the deployment-controlled service.
    site_fixture::provision(
        &pool,
        &alpha_id,
        &[reasonbraid_server::site_authority::Action::EvidenceExpire],
    )
    .await;

    // The sweep is a repair of AUTHORITY, not a removal: an authorized operator
    // still expires exactly what its retention class makes due. `standard` is
    // thirty days, so the fixture ages one row rather than inventing a clock.
    sqlx::query("UPDATE evidence_snapshots SET created_at = now() - interval '31 days' WHERE snapshot_id = $1")
        .bind(&beta_snapshot)
        .execute(&pool)
        .await
        .expect("age tenant B's row past its retention horizon");
    let (status, body) = post(
        &client,
        &base,
        "/v1/snapshots/expire-due",
        &alpha_id,
        &json!({ "reason": "the scheduled retention sweep" }),
    )
    .await;
    assert_eq!(status, 200, "the authorized sweep runs: {body}");
    assert_eq!(body["tombstoned"], json!(1), "exactly the due row: {body}");
    assert!(
        !live(beta_snapshot.clone()).await,
        "the aged row is tombstoned"
    );
    assert!(
        live(alpha_snapshot.clone()).await,
        "a row inside its horizon survives the sweep"
    );

    // The caller's clock is gone from the wire: an `at` field is no longer a
    // field this route accepts, so a fabricated retention expiry is not
    // expressible rather than merely unauthorized.
    let response = client
        .post(format!("{base}/v1/snapshots/expire-due"))
        .header(PRINCIPAL_HEADER, &alpha_id)
        .json(&json!({ "reason": "a sweep", "at": "3000-01-01T00:00:00Z" }))
        .send()
        .await
        .expect("clock-override request");
    assert_eq!(
        response.status().as_u16(),
        400,
        "the route still accepts a caller-supplied clock"
    );
    let missing_reason = client
        .post(format!("{base}/v1/snapshots/expire-due"))
        .header(PRINCIPAL_HEADER, &alpha_id)
        .json(&json!({}))
        .send()
        .await
        .expect("reason-less request");
    assert_eq!(
        missing_reason.status().as_u16(),
        400,
        "the sweep requires a reason"
    );
    assert!(
        live(alpha_snapshot.clone()).await,
        "the refused request changed nothing"
    );

    // The deletion record says what actually happened.
    let reason: String =
        sqlx::query_scalar("SELECT deletion_reason FROM evidence_snapshots WHERE snapshot_id = $1")
            .bind(&beta_snapshot)
            .fetch_one(&pool)
            .await
            .expect("read the deletion reason");
    assert_eq!(reason, "the retention expired");

    // The act is audited as a site decision, denial included.
    let decisions: Vec<(String, String)> = sqlx::query_as(
        "SELECT outcome, reason FROM public.site_audit \
         WHERE action = 'evidence_expire' ORDER BY decided_at",
    )
    .fetch_all(&pool)
    .await
    .expect("read the site audit");
    assert!(
        decisions.iter().any(|(outcome, _)| outcome == "denied"),
        "the unauthorized sweep left no denial record: {decisions:?}"
    );
    assert!(
        decisions.iter().any(|(outcome, _)| outcome == "applied"),
        "the authorized sweep left no applied record: {decisions:?}"
    );
    eprintln!(
        "retention sweep: unauthorized 403 with both tenants' rows live; authorized sweep tombstoned 1 due row and left 1 live; caller clock refused 400; {} site audit rows",
        decisions.len()
    );
}

/// An assessment is read by the tenant that AUTHORED it
/// (`SIGNOFF-REPAIR.11.14.2`).
///
/// `GET /v1/claims/{claim_id}/assessments` admitted any enrolled principal over
/// a namespace the server never mints: no `claims` table exists, `claim_id` is
/// caller-supplied `TEXT` with no key, and the shipped control's own id is
/// `clm_budget`. That makes it an ORACLE over guessable identifiers — the
/// property `.11.14.1`'s enumeration explicitly did not have.
///
/// The binding is the AUTHORING tenant rather than the parent snapshot's
/// citation, and the schema is why: `claim_assessments_replay_idx` is
/// `(claim_id, snapshot_id, assessment, author)`, so two tenants asserting the
/// same thing about the same evidence hold two SEPARATE rows. An assessment is
/// an authored opinion, not a shared receipt, so `.11.14`'s content-addressing
/// argument — which correctly forbids a column on `evidence_snapshots` and
/// `derivations` — does not reach it.
///
/// That also settles `.11.14.1`'s co-citation residual for assessments: the
/// last legs cite ONE shared snapshot from both tenants and require that
/// neither reads the other's position on it.
#[tokio::test]
async fn an_assessment_is_read_by_the_tenant_that_authored_it() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, alpha) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "assess-alpha" }),
    )
    .await;
    assert_eq!(status, 200, "tenant A's human enrolls: {alpha}");
    let alpha_id = alpha["principal_id"].as_str().unwrap().to_string();
    let (status, beta) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "assess-beta" }),
    )
    .await;
    assert_eq!(status, 200, "tenant B's human enrolls: {beta}");
    let beta_id = beta["principal_id"].as_str().unwrap().to_string();
    assert_ne!(
        alpha["tenant_id"], beta["tenant_id"],
        "two distinct tenants"
    );

    // One snapshot per tenant, plus one both of them cite.
    let cite = |principal: String, locator: &'static str, payload: &'static [u8]| {
        let client = client.clone();
        let base = base.clone();
        async move {
            let response = client
                .post(format!("{base}/v1/resources"))
                .header(PRINCIPAL_HEADER, &principal)
                .json(&json!({ "original_locator": locator, "scheme": "https" }))
                .send()
                .await
                .expect("reference request");
            let reference: Value = response.json().await.expect("reference json");
            let reference_id = reference["resource_id"].as_str().unwrap().to_string();
            let response = client
                .post(format!("{base}/v1/snapshots"))
                .header(PRINCIPAL_HEADER, &principal)
                .json(&json!({
                    "reference_id": reference_id,
                    "original_locator": locator,
                    "final_locator": locator,
                    "resolver_id": "r0-https-fetcher",
                    "resolver_version": "0.1.0",
                    "raw_digest": reasonbraid_server::fetcher::digest_sha256_hex(payload),
                    "byte_length": payload.len(),
                    "media_type": "text/plain",
                    "bytes_base64": util::base64(payload),
                }))
                .send()
                .await
                .expect("snapshot request");
            let status = response.status().as_u16();
            let snapshot: Value = response.json().await.expect("snapshot json");
            assert_eq!(status, 200, "the snapshot submits: {snapshot}");
            snapshot["snapshot_id"].as_str().unwrap().to_string()
        }
    };
    let beta_bytes = b"the beta tenant assessed evidence";
    let shared_bytes = b"the shared evidence both tenants acquired";
    let beta_snapshot = cite(
        beta_id.clone(),
        "https://example.org/assess-beta",
        beta_bytes,
    )
    .await;
    let shared_from_beta = cite(
        beta_id.clone(),
        "https://example.org/assess-shared",
        shared_bytes,
    )
    .await;
    let shared_from_alpha = cite(
        alpha_id.clone(),
        "https://example.org/assess-shared",
        shared_bytes,
    )
    .await;
    assert_eq!(
        shared_from_alpha, shared_from_beta,
        "the shared locator and digest replay to one snapshot"
    );
    let shared_snapshot = shared_from_alpha;

    // A claim identifier nobody mints. `clm_budget` is the id the shipped
    // control uses, which is the point: the namespace is guessable because the
    // server never generates it.
    let guessable = "clm_budget";
    let assess =
        |principal: String, snapshot: String, excerpt: &'static str, rationale: &'static str| {
            let client = client.clone();
            let base = base.clone();
            async move {
                let (status, body) = post(
                    &client,
                    &base,
                    "/v1/assessments",
                    &principal,
                    &json!({
                        "claim_id": guessable,
                        "snapshot_id": snapshot,
                        "assessment": "supports",
                        "author": principal,
                        "excerpt": excerpt,
                        "rationale": rationale,
                    }),
                )
                .await;
                assert_eq!(status, 200, "the assessment submits: {body}");
                body["assessment_id"].as_str().unwrap().to_string()
            }
        };
    let beta_private = assess(
        beta_id.clone(),
        beta_snapshot.clone(),
        "beta tenant assessed evidence",
        "tenant B's private analytical position",
    )
    .await;
    let beta_shared = assess(
        beta_id.clone(),
        shared_snapshot.clone(),
        "shared evidence both tenants",
        "tenant B's position on the shared evidence",
    )
    .await;
    let alpha_shared = assess(
        alpha_id.clone(),
        shared_snapshot.clone(),
        "shared evidence both tenants",
        "tenant A's own position on the shared evidence",
    )
    .await;
    assert_ne!(
        alpha_shared, beta_shared,
        "two tenants asserting the same thing hold SEPARATE rows — the replay key carries the author"
    );

    let ids = |body: &Value| {
        body.as_array()
            .expect("the assessment array")
            .iter()
            .map(|row| row["assessment_id"].as_str().unwrap().to_string())
            .collect::<Vec<String>>()
    };

    // The finding: the claim-keyed read, over an identifier tenant A guessed.
    let (status, body) = get(
        &client,
        &base,
        &format!("/v1/claims/{guessable}/assessments"),
        &alpha_id,
    )
    .await;
    assert_eq!(status, 200, "tenant A reads its own claim side: {body}");
    let seen = ids(&body);
    assert!(
        seen.contains(&alpha_shared),
        "tenant A must still read its OWN assessment — a binding, not a blackout: {seen:?}"
    );
    assert!(
        !seen.contains(&beta_private),
        "tenant A read tenant B's position on evidence A never cited: {seen:?}"
    );
    assert!(
        !seen.contains(&beta_shared),
        "tenant A read tenant B's position on the SHARED snapshot: {seen:?}"
    );

    // And the snapshot-keyed read over the row both tenants cite. `.11.14.1`
    // bound this surface on the parent's citation, which both tenants hold, so
    // before this repair it disclosed B's position to A.
    let (status, body) = get(
        &client,
        &base,
        &format!("/v1/snapshots/{shared_snapshot}/assessments"),
        &alpha_id,
    )
    .await;
    assert_eq!(
        status, 200,
        "tenant A reads the shared snapshot's side: {body}"
    );
    let seen = ids(&body);
    assert!(
        seen.contains(&alpha_shared),
        "tenant A reads its own position on the shared snapshot: {seen:?}"
    );
    assert!(
        !seen.contains(&beta_shared),
        "a co-citing tenant read the other's analytical position: {seen:?}"
    );

    // Symmetry: tenant B reads its own two and neither of A's.
    let (status, body) = get(
        &client,
        &base,
        &format!("/v1/claims/{guessable}/assessments"),
        &beta_id,
    )
    .await;
    assert_eq!(status, 200, "tenant B reads its own claim side: {body}");
    let seen = ids(&body);
    assert!(seen.contains(&beta_private), "{seen:?}");
    assert!(seen.contains(&beta_shared), "{seen:?}");
    assert!(
        !seen.contains(&alpha_shared),
        "tenant B read tenant A's position: {seen:?}"
    );

    // The gates below the binding are unchanged and attributed: a well-formed
    // stranger is refused by the enrolment gate, a malformed one by the parser.
    let (status, body) = get(
        &client,
        &base,
        &format!("/v1/claims/{guessable}/assessments"),
        STRANGER,
    )
    .await;
    assert_eq!(
        status, 403,
        "an unenrolled principal reads assessments: {body}"
    );
    assert_eq!(body["code"], json!("unauthorized"), "{body}");

    let authored: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM claim_assessments WHERE authored_by_tenant IS NULL",
    )
    .fetch_one(&pool)
    .await
    .expect("count unattributed assessments");
    assert_eq!(
        authored, 0,
        "every assessment this control wrote is attributed"
    );
    eprintln!(
        "assessment binding: the claim-keyed and snapshot-keyed reads are bound to the AUTHORING tenant; the shared snapshot carries one assessment per tenant and neither reads the other's; {authored} unattributed rows"
    );
}

/// The `assess` step records an assessment against the thread
/// (`SIGNOFF-REPAIR.11.14.3.1`,
/// `docs/decisions/2026-09-16_the-deliberation-flow-owns-the-evidence-chain.md`).
///
/// ROADMAP §13.2's deliberation flow is: step 2 register context and resource
/// references, step 5 normalize claims and requested evidence, step 6
/// acquire/assess evidence within the allowed plan. The product encodes it —
/// `STEP_KINDS` carries `assess`, and `evidence_review` and `policy_proposal`
/// declare the step — but `git grep -n '"assess"' -- crates/reasonbraid-server/src`
/// returned ONE hit before this repair: the vocabulary constant itself. A tenant
/// running the shipped profile for reviewing evidence advanced onto a step at
/// which nothing could be recorded, beside a complete assessment store.
///
/// The claim is named by its SERVER-COMPUTED digest and membership-checked
/// against this thread, so the identifier cannot be invented; the snapshot must
/// be one this tenant CITED (`SIGNOFF-REPAIR.11.14.1`); and the row is authored
/// by the thread's tenant (`SIGNOFF-REPAIR.11.14.2`).
#[tokio::test]
async fn the_assess_step_records_an_assessment_against_the_thread() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, human) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "assess-step-human" }),
    )
    .await;
    assert_eq!(status, 200, "the human enrolls: {human}");
    let human_id = human["principal_id"].as_str().unwrap().to_string();
    let tenant_id = human["tenant_id"].as_str().unwrap().to_string();

    // The shipped built-in whose steps are `solicit, evidence_request, assess,
    // decide` (`migrations/0032_workflow_profiles.sql`).
    let (status, created) = post(
        &client,
        &base,
        "/v1/threads",
        &human_id,
        &json!({
            "protocol_version": reasonbraid_core::PROTOCOL_VERSION,
            "operation": "thread.create",
            "request_id": reasonbraid_core::RequestId::new().to_string(),
            "idempotency_key": "as-create",
            "body": {
                "tenant_id": tenant_id,
                "subject": "as-assess",
                "objective": "probe the assess step",
                "workflow_profile": "evidence_review",
            },
            "client_context": {},
        }),
    )
    .await;
    assert_eq!(status, 200, "the evidence_review thread creates: {created}");
    let thread_id = created["thread_id"].as_str().unwrap().to_string();

    let command = |key: String, operation: &'static str, body: Value| {
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

    // The evidence this deliberation will assess: a reference, then a snapshot
    // over bytes the excerpt genuinely appears in. Submitting it as this tenant
    // is what records the citation (`.11.14.1`).
    let payload = b"the acquired report states the migration preserved every row";
    let (status, reference) = post(
        &client,
        &base,
        "/v1/resources",
        &human_id,
        &json!({ "original_locator": "https://example.org/assess-evidence", "scheme": "https" }),
    )
    .await;
    assert_eq!(status, 200, "the reference submits: {reference}");
    let reference_id = reference["resource_id"].as_str().unwrap().to_string();
    let (status, snapshot) = post(
        &client,
        &base,
        "/v1/snapshots",
        &human_id,
        &json!({
            "reference_id": reference_id,
            "original_locator": "https://example.org/assess-evidence",
            "final_locator": "https://example.org/assess-evidence",
            "resolver_id": "r0-https-fetcher",
            "resolver_version": "0.1.0",
            "raw_digest": reasonbraid_server::fetcher::digest_sha256_hex(payload),
            "byte_length": payload.len(),
            "media_type": "text/plain",
            "bytes_base64": util::base64(payload),
        }),
    )
    .await;
    assert_eq!(status, 200, "the snapshot submits: {snapshot}");
    let snapshot_id = snapshot["snapshot_id"].as_str().unwrap().to_string();

    // Step 0 `solicit`: the claim whose digest the assessment will name. The
    // digest is the SERVER's — the client sends content only.
    let claim_content = "the migration preserved every row";
    let claim_digest = reasonbraid_server::fetcher::digest_sha256_hex(claim_content.as_bytes());
    let (status, claimed) = command(
        "as-claim".into(),
        "thread.contribute",
        json!({
            "tenant_id": tenant_id,
            "content": "the claim under review",
            "kind": "claim",
            "claims": [ { "content": claim_content } ],
        }),
    )
    .await;
    assert_eq!(status, 200, "the claim contributes: {claimed}");

    let assessment_body = |digest: &str, snapshot: &str, excerpt: &str| {
        json!({
            "tenant_id": tenant_id,
            "content": "the evidence supports the claim",
            "kind": "assessment",
            "assessment": {
                "claim_digest": digest,
                "snapshot_id": snapshot,
                "assessment": "supports",
                "excerpt": excerpt,
                "rationale": "the report states it in the acquired bytes",
            },
        })
    };

    // The step gate, before the step: an assessment during `solicit` refuses.
    let (status, early) = command(
        "as-early".into(),
        "thread.contribute",
        assessment_body(&claim_digest, &snapshot_id, "preserved every row"),
    )
    .await;
    assert_eq!(status, 400, "the early assessment refuses: {early}");
    assert!(
        early["message"]
            .as_str()
            .unwrap_or_default()
            .contains("assess"),
        "the refusal names its step: {early}"
    );

    // solicit → evidence_request → assess.
    for (key, _) in [("as-advance-1", 0), ("as-advance-2", 0)] {
        let (status, advanced) = command(
            key.to_string(),
            "thread.advance_round",
            json!({ "tenant_id": tenant_id }),
        )
        .await;
        assert_eq!(status, 200, "the round advances: {advanced}");
    }

    // ── The finding: on its own step, the assessment is recorded ──
    let (status, assessed) = command(
        "as-assess".into(),
        "thread.contribute",
        assessment_body(&claim_digest, &snapshot_id, "preserved every row"),
    )
    .await;
    assert_eq!(
        status, 200,
        "the `assess` step records nothing — the step is declared by a shipped profile and wired to no contribution kind: {assessed}"
    );

    // The row exists, keyed by the MINTED digest and authored by this tenant.
    let stored: Vec<(String, String, String, Option<String>)> = sqlx::query_as(
        "SELECT claim_id, snapshot_id, assessment, authored_by_tenant \
         FROM claim_assessments WHERE claim_id = $1",
    )
    .bind(&claim_digest)
    .fetch_all(&pool)
    .await
    .expect("read the assessment the step recorded");
    assert_eq!(stored.len(), 1, "exactly one assessment: {stored:?}");
    assert_eq!(
        stored[0].1, snapshot_id,
        "it cites the snapshot: {stored:?}"
    );
    assert_eq!(stored[0].2, "supports", "{stored:?}");
    assert_eq!(
        stored[0].3.as_deref(),
        Some(tenant_id.as_str()),
        "authored by the thread's tenant: {stored:?}"
    );

    // And it is reachable through the claim-keyed read, whose identifier is now
    // a digest the server minted rather than a label a caller invented.
    let (status, listed) = get(
        &client,
        &base,
        &format!("/v1/claims/{claim_digest}/assessments"),
        &human_id,
    )
    .await;
    assert_eq!(status, 200, "the claim's assessments read: {listed}");
    let rows = listed.as_array().expect("the assessment array");
    assert_eq!(rows.len(), 1, "{rows:?}");
    assert_eq!(rows[0]["snapshot_id"], json!(snapshot_id));

    // The event carries the assessment's facts, so the timeline shows what was
    // asserted and against which evidence.
    let (status, timeline) = get(
        &client,
        &base,
        &format!("/v1/threads/{thread_id}/events?tenant_id={tenant_id}"),
        &human_id,
    )
    .await;
    assert_eq!(status, 200, "the timeline reads: {timeline}");
    let event = timeline["events"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| {
            e["event_type"] == json!("thread.contribution_submitted")
                && e["body"]["kind"] == json!("assessment")
        })
        .cloned()
        .expect("the assessment event is in the timeline");
    assert_eq!(
        event["body"]["assessment"]["claim_digest"],
        json!(claim_digest)
    );
    assert_eq!(
        event["body"]["assessment"]["snapshot_id"],
        json!(snapshot_id)
    );
    assert_eq!(event["body"]["assessment"]["assessment"], json!("supports"));

    // ── Each refusal by NAME, so a caller learns which gate stopped it ──

    // A forged claim digest is not a claim of this thread.
    let forged = reasonbraid_server::fetcher::digest_sha256_hex(b"never claimed here");
    let (status, refused) = command(
        "as-forged".into(),
        "thread.contribute",
        assessment_body(&forged, &snapshot_id, "preserved every row"),
    )
    .await;
    assert_eq!(status, 400, "the forged digest refuses: {refused}");
    assert!(
        refused["message"]
            .as_str()
            .unwrap_or_default()
            .contains("claim"),
        "{refused}"
    );

    // A snapshot this tenant never cited is not readable and not assessable.
    let (status, stranger) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "assess-step-other" }),
    )
    .await;
    assert_eq!(status, 200, "the other tenant enrols: {stranger}");
    let other_id = stranger["principal_id"].as_str().unwrap().to_string();
    let other_payload = b"another tenant's acquired document";
    let (status, other_reference) = post(
        &client,
        &base,
        "/v1/resources",
        &other_id,
        &json!({ "original_locator": "https://example.org/assess-other", "scheme": "https" }),
    )
    .await;
    assert_eq!(status, 200, "{other_reference}");
    let (status, other_snapshot) = post(
        &client,
        &base,
        "/v1/snapshots",
        &other_id,
        &json!({
            "reference_id": other_reference["resource_id"].as_str().unwrap(),
            "original_locator": "https://example.org/assess-other",
            "final_locator": "https://example.org/assess-other",
            "resolver_id": "r0-https-fetcher",
            "resolver_version": "0.1.0",
            "raw_digest": reasonbraid_server::fetcher::digest_sha256_hex(other_payload),
            "byte_length": other_payload.len(),
            "media_type": "text/plain",
            "bytes_base64": util::base64(other_payload),
        }),
    )
    .await;
    assert_eq!(status, 200, "{other_snapshot}");
    let (status, refused) = command(
        "as-uncited".into(),
        "thread.contribute",
        assessment_body(
            &claim_digest,
            other_snapshot["snapshot_id"].as_str().unwrap(),
            "another tenant's acquired",
        ),
    )
    .await;
    assert_eq!(status, 400, "the uncited snapshot refuses: {refused}");
    assert!(
        refused["message"]
            .as_str()
            .unwrap_or_default()
            .contains("snapshot"),
        "{refused}"
    );

    // The excerpt must appear in the acquired bytes — citation existence alone
    // never satisfies an evidence gate (ROADMAP §12.7).
    let (status, refused) = command(
        "as-fake-excerpt".into(),
        "thread.contribute",
        assessment_body(&claim_digest, &snapshot_id, "preserved NO rows at all"),
    )
    .await;
    assert_eq!(status, 400, "the fake excerpt refuses: {refused}");
    assert!(
        refused["message"]
            .as_str()
            .unwrap_or_default()
            .contains("excerpt"),
        "{refused}"
    );

    // An assessment payload on another kind is refused, the way a verdict is.
    let (status, refused) = command(
        "as-wrong-kind".into(),
        "thread.contribute",
        json!({
            "tenant_id": tenant_id,
            "content": "a position carrying an assessment",
            "kind": "position",
            "assessment": {
                "claim_digest": claim_digest,
                "snapshot_id": snapshot_id,
                "assessment": "supports",
                "excerpt": "preserved every row",
                "rationale": "misplaced",
            },
        }),
    )
    .await;
    assert_eq!(status, 400, "the misplaced assessment refuses: {refused}");

    // An unknown assessment kind is refused by name.
    let (status, refused) = command(
        "as-unknown-kind".into(),
        "thread.contribute",
        json!({
            "tenant_id": tenant_id,
            "content": "an unsupported judgement",
            "kind": "assessment",
            "assessment": {
                "claim_digest": claim_digest,
                "snapshot_id": snapshot_id,
                "assessment": "proves",
                "excerpt": "preserved every row",
                "rationale": "not one of the five",
            },
        }),
    )
    .await;
    assert_eq!(
        status, 400,
        "the unknown assessment kind refuses: {refused}"
    );

    // Nothing the refusals attempted was written.
    let total: i64 = sqlx::query_scalar("SELECT count(*) FROM claim_assessments")
        .fetch_one(&pool)
        .await
        .expect("count the assessments");
    assert_eq!(total, 1, "only the accepted assessment exists");
    eprintln!(
        "assess step: the shipped evidence_review profile records an assessment keyed by a minted claim digest over a cited snapshot; forged digest, uncited snapshot, fake excerpt, misplaced payload, wrong step and unknown kind each refused by name; {total} row written"
    );
}

/// The two assessment writers are two NAMESPACES, and the row says which
/// (`SIGNOFF-REPAIR.11.14.3.3`,
/// `docs/decisions/2026-09-17_the-assessment-namespace-is-part-of-the-row.md`).
///
/// `.11.14.3.1` made a deliberation's `claim_id` the SERVER-MINTED claim digest,
/// membership-checked against the thread. `POST /v1/assessments` was left
/// standing deliberately — an assessment made outside any deliberation may be
/// legitimate — and still takes a free-text `claim_id`. So one column holds two
/// kinds of identifier, and before this leaf nothing said which kind a row was.
///
/// Two consequences, and the second is the one that made this a defect rather
/// than an untidiness. A caller who types a real thread's digest into the
/// standalone route lands a row in that deliberation's claim-keyed read, having
/// passed neither the thread-membership gate nor the citation gate. And because
/// the replay key `(claim_id, snapshot_id, assessment, author)` did not carry
/// the namespace either, naming the deliberation's own author ALIASED its row:
/// the standalone route returned the deliberation's `assessment_id` to a caller
/// who never contributed to the thread.
///
/// The namespace joins the row's identity for the same reason the
/// `(locator, digest)` pair became the reference's in `.11.14.3.2`: an
/// identifier two writers mint differently is not one identifier.
#[tokio::test]
async fn the_two_assessment_writers_are_two_namespaces() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, human) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "namespace-human" }),
    )
    .await;
    assert_eq!(status, 200, "the human enrols: {human}");
    let human_id = human["principal_id"].as_str().unwrap().to_string();
    let tenant_id = human["tenant_id"].as_str().unwrap().to_string();

    let (status, created) = post(
        &client,
        &base,
        "/v1/threads",
        &human_id,
        &json!({
            "protocol_version": reasonbraid_core::PROTOCOL_VERSION,
            "operation": "thread.create",
            "request_id": reasonbraid_core::RequestId::new().to_string(),
            "idempotency_key": "ns-create",
            "body": {
                "tenant_id": tenant_id,
                "subject": "ns-assess",
                "objective": "probe the assessment namespaces",
                "workflow_profile": "evidence_review",
            },
            "client_context": {},
        }),
    )
    .await;
    assert_eq!(status, 200, "the evidence_review thread creates: {created}");
    let thread_id = created["thread_id"].as_str().unwrap().to_string();

    let command = |key: String, operation: &'static str, body: Value| {
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

    // The evidence this deliberation assesses, cited by this tenant (`.11.14.1`).
    let payload = b"the acquired report states the ledger balanced to the cent";
    let (status, reference) = post(
        &client,
        &base,
        "/v1/resources",
        &human_id,
        &json!({ "original_locator": "https://example.org/ns-evidence", "scheme": "https" }),
    )
    .await;
    assert_eq!(status, 200, "the reference submits: {reference}");
    let (status, snapshot) = post(
        &client,
        &base,
        "/v1/snapshots",
        &human_id,
        &json!({
            "reference_id": reference["resource_id"].as_str().unwrap(),
            "original_locator": "https://example.org/ns-evidence",
            "final_locator": "https://example.org/ns-evidence",
            "resolver_id": "r0-https-fetcher",
            "resolver_version": "0.1.0",
            "raw_digest": reasonbraid_server::fetcher::digest_sha256_hex(payload),
            "byte_length": payload.len(),
            "media_type": "text/plain",
            "bytes_base64": util::base64(payload),
        }),
    )
    .await;
    assert_eq!(status, 200, "the snapshot submits: {snapshot}");
    let snapshot_id = snapshot["snapshot_id"].as_str().unwrap().to_string();

    // The claim whose digest the deliberation's assessment names.
    let claim_content = "the ledger balanced to the cent";
    let claim_digest = reasonbraid_server::fetcher::digest_sha256_hex(claim_content.as_bytes());
    let (status, claimed) = command(
        "ns-claim".into(),
        "thread.contribute",
        json!({
            "tenant_id": tenant_id,
            "content": "the claim under review",
            "kind": "claim",
            "claims": [ { "content": claim_content } ],
        }),
    )
    .await;
    assert_eq!(status, 200, "the claim contributes: {claimed}");

    // solicit → evidence_request → assess.
    for key in ["ns-advance-1", "ns-advance-2"] {
        let (status, advanced) = command(
            key.to_string(),
            "thread.advance_round",
            json!({ "tenant_id": tenant_id }),
        )
        .await;
        assert_eq!(status, 200, "the round advances: {advanced}");
    }

    let (status, assessed) = command(
        "ns-assess".into(),
        "thread.contribute",
        json!({
            "tenant_id": tenant_id,
            "content": "the evidence supports the claim",
            "kind": "assessment",
            "assessment": {
                "claim_digest": claim_digest,
                "snapshot_id": snapshot_id,
                "assessment": "supports",
                "excerpt": "balanced to the cent",
                "rationale": "the report states it in the acquired bytes",
            },
        }),
    )
    .await;
    assert_eq!(status, 200, "the assess step records: {assessed}");
    let deliberated: String =
        sqlx::query_scalar("SELECT assessment_id FROM claim_assessments WHERE claim_id = $1")
            .bind(&claim_digest)
            .fetch_one(&pool)
            .await
            .expect("the deliberation's assessment row");

    // ── The finding: the standalone route, on the deliberation's own key ──
    //
    // Every field matches what the `assess` step wrote, `author` included —
    // which is the shape that ALIASED the deliberation's row through the
    // four-column replay key. The namespace is what makes these two rows two
    // assertions rather than one.
    let (status, external) = post(
        &client,
        &base,
        "/v1/assessments",
        &human_id,
        &json!({
            "claim_id": claim_digest,
            "snapshot_id": snapshot_id,
            "assessment": "supports",
            "author": human_id,
            "excerpt": "balanced to the cent",
            "rationale": "asserted outside the deliberation",
        }),
    )
    .await;
    assert_eq!(status, 200, "the standalone assessment submits: {external}");
    let external_id = external["assessment_id"].as_str().unwrap().to_string();
    assert_ne!(
        external_id, deliberated,
        "the standalone route aliased the deliberation's own row and returned its \
         assessment_id — the replay key does not carry the namespace"
    );

    // Both rows exist, and each says which writer minted its identifier.
    let namespaces: Vec<(String, Option<String>)> = sqlx::query_as(
        "SELECT assessment_id, claim_namespace FROM claim_assessments \
         WHERE claim_id = $1 ORDER BY created_at",
    )
    .bind(&claim_digest)
    .fetch_all(&pool)
    .await
    .expect("read both rows");
    assert_eq!(
        namespaces.len(),
        2,
        "two assertions, two rows: {namespaces:?}"
    );
    assert_eq!(
        namespaces
            .iter()
            .find(|(id, _)| id == &deliberated)
            .and_then(|(_, ns)| ns.as_deref()),
        Some("thread"),
        "the deliberation's row is in the thread namespace: {namespaces:?}"
    );
    assert_eq!(
        namespaces
            .iter()
            .find(|(id, _)| id == &external_id)
            .and_then(|(_, ns)| ns.as_deref()),
        Some("external"),
        "the standalone row is in the external namespace: {namespaces:?}"
    );

    // The claim-keyed read returns both and DISTINGUISHES them, so a reader can
    // tell an assessment the deliberation's gates admitted from one asserted
    // beside it. Nothing is filtered: filtering would make the standalone route
    // write-only, which is worse than the removal this leaf declined.
    let (status, listed) = get(
        &client,
        &base,
        &format!("/v1/claims/{claim_digest}/assessments"),
        &human_id,
    )
    .await;
    assert_eq!(status, 200, "the claim's assessments read: {listed}");
    let rows = listed.as_array().expect("the assessment array");
    assert_eq!(rows.len(), 2, "{rows:?}");
    let mut seen: Vec<&str> = rows
        .iter()
        .map(|r| {
            r["claim_namespace"]
                .as_str()
                .expect("the namespace rides the read")
        })
        .collect();
    seen.sort_unstable();
    assert_eq!(seen, vec!["external", "thread"], "{rows:?}");

    // The replay still holds WITHIN a namespace: the same standalone submission
    // returns the same id rather than a second row.
    let (status, replayed) = post(
        &client,
        &base,
        "/v1/assessments",
        &human_id,
        &json!({
            "claim_id": claim_digest,
            "snapshot_id": snapshot_id,
            "assessment": "supports",
            "author": human_id,
            "excerpt": "balanced to the cent",
            "rationale": "asserted outside the deliberation",
        }),
    )
    .await;
    assert_eq!(status, 200, "the standalone replay submits: {replayed}");
    assert_eq!(
        replayed["assessment_id"],
        json!(external_id),
        "the standalone replay returns its own id: {replayed}"
    );

    // ── The second finding, MEASURED rather than read ──
    //
    // The namespace alone does not bound the aliasing, because the replay
    // pre-check carries no tenant predicate: `.11.14.2`'s authoring gate binds
    // the two READS and never this. `migrations/0064` reasoned that the key
    // "carries the AUTHOR, so two tenants asserting the same thing about the
    // same evidence already hold two separate rows" — and stated eight lines
    // later, in the same file, that `author` is an unauthenticated caller
    // string. ⛔ Both cannot be true: on this route the CALLER types `author`,
    // so a second tenant can simply present the first tenant's label.
    let (status, stranger) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "namespace-stranger" }),
    )
    .await;
    assert_eq!(status, 200, "the second tenant enrols: {stranger}");
    let stranger_id = stranger["principal_id"].as_str().unwrap().to_string();
    assert_ne!(
        stranger["tenant_id"].as_str().unwrap(),
        tenant_id,
        "the stranger must be a DIFFERENT tenant for this to measure anything"
    );

    // ⭐ The stranger ACQUIRES the same bytes first, which records its own
    // citation on the replay. Until `SIGNOFF-REPAIR.11.14.3.8` this arm reached
    // the excerpt check without it, because the standalone route applied no
    // citation gate — so the aliasing below was measurable over evidence the
    // stranger had never touched. The gate does not close the aliasing question:
    // two tenants that BOTH cite one shared row still both reach it, which is
    // exactly what makes `authored_by_tenant` load-bearing rather than redundant.
    let (status, stranger_reference) = post(
        &client,
        &base,
        "/v1/resources",
        &stranger_id,
        &json!({ "original_locator": "https://example.org/ns-evidence", "scheme": "https" }),
    )
    .await;
    assert_eq!(
        status, 200,
        "the stranger's reference replays: {stranger_reference}"
    );
    let (status, stranger_snapshot) = post(
        &client,
        &base,
        "/v1/snapshots",
        &stranger_id,
        &json!({
            "reference_id": stranger_reference["resource_id"].as_str().unwrap(),
            "original_locator": "https://example.org/ns-evidence",
            "final_locator": "https://example.org/ns-evidence",
            "resolver_id": "r0-https-fetcher",
            "resolver_version": "0.1.0",
            "raw_digest": reasonbraid_server::fetcher::digest_sha256_hex(payload),
            "byte_length": payload.len(),
            "media_type": "text/plain",
            "bytes_base64": util::base64(payload),
        }),
    )
    .await;
    assert_eq!(
        status, 200,
        "the stranger's snapshot replays: {stranger_snapshot}"
    );
    assert_eq!(
        stranger_snapshot["snapshot_id"].as_str().unwrap(),
        snapshot_id,
        "the same locator and digest replay to ONE shared snapshot"
    );

    // Every field is the first tenant's, `author` included.
    let (status, forged) = post(
        &client,
        &base,
        "/v1/assessments",
        &stranger_id,
        &json!({
            "claim_id": claim_digest,
            "snapshot_id": snapshot_id,
            "assessment": "supports",
            "author": human_id,
            "excerpt": "balanced to the cent",
            "rationale": "asserted outside the deliberation",
        }),
    )
    .await;
    assert_eq!(status, 200, "the stranger's assessment submits: {forged}");
    assert_ne!(
        forged["assessment_id"].as_str().unwrap(),
        external_id,
        "a SECOND TENANT was handed the first tenant's assessment_id — the replay \
         key carries a caller-supplied `author` and no server-set tenant"
    );

    // Three assertions by two tenants in two namespaces: three rows, each
    // attributed to the tenant the SERVER recorded.
    let attributed: Vec<(String, Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT assessment_id, claim_namespace, authored_by_tenant \
         FROM claim_assessments WHERE claim_id = $1",
    )
    .bind(&claim_digest)
    .fetch_all(&pool)
    .await
    .expect("read all three rows");
    assert_eq!(attributed.len(), 3, "three assertions: {attributed:?}");
    assert!(
        attributed
            .iter()
            .all(|(_, ns, tenant)| ns.is_some() && tenant.is_some()),
        "every row carries both server-set columns: {attributed:?}"
    );

    let total: i64 = sqlx::query_scalar("SELECT count(*) FROM claim_assessments")
        .fetch_one(&pool)
        .await
        .expect("count the assessments");
    assert_eq!(total, 3, "the replay wrote no extra row");
    eprintln!(
        "assessment namespaces: the deliberation's row and the standalone row share a claim digest, a snapshot, a kind and an author and are TWO rows in two namespaces; a SECOND TENANT presenting the first's author label gets its own row rather than the first's assessment_id; the standalone replay still returns its own id; {total} rows"
    );
}

/// `SIGNOFF-REPAIR.11.14.3.8`: `POST /v1/assessments` is bound to the CITING
/// tenant, so it stops answering three distinguishable things about a snapshot
/// the caller never acquired.
///
/// The finding is the product's own asymmetry. `threads.rs`'s `assess` step
/// gates on `snapshots::is_cited_by` with the stated reason that *"without
/// this, an assessment would be a way to learn that a snapshot exists"*, and
/// both writers call the SAME `claims::submit` — which selected
/// `snapshot_objects.bytes` on `snapshot_id` alone, with no tenant predicate.
/// A census of every surface that names a `snapshot_id` found seven, six
/// citation-bound and exactly one not:
///
/// ```text
/// grep -n "cited_snapshot\|is_cited_by" crates/reasonbraid-server/src/*.rs
/// ```
///
/// The three answers a stranger could separate were the oracle, and the third
/// is the strongest: a 200 says a chosen substring APPEARS in bytes the caller
/// was never allowed to read.
///
/// ⭐ The compatibility objection — *"a principal legitimately assessing
/// evidence another team acquired starts being refused"* — was measured rather
/// than accepted. Every read of that snapshot (`GET /v1/snapshots/{id}`, its
/// `/derivations`, its `/assessments`, `/v1/snapshots/stale`, and the `DELETE`)
/// already answers a non-citing tenant `404`, so that workflow cannot function
/// today: the excerpt check was over bytes the caller cannot see. The arm below
/// where a SECOND tenant acquires the same bytes and then assesses them is the
/// supported path, and it is the bound that keeps this from being a blackout.
#[tokio::test]
async fn the_standalone_assessment_route_is_bound_to_the_citing_tenant() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, owner) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "citation-gate-owner" }),
    )
    .await;
    assert_eq!(status, 200, "the owning tenant enrols: {owner}");
    let owner_id = owner["principal_id"].as_str().unwrap().to_string();
    let owner_tenant = owner["tenant_id"].as_str().unwrap().to_string();

    let (status, stranger) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "citation-gate-stranger" }),
    )
    .await;
    assert_eq!(status, 200, "the second tenant enrols: {stranger}");
    let stranger_id = stranger["principal_id"].as_str().unwrap().to_string();
    assert_ne!(
        stranger["tenant_id"].as_str().unwrap(),
        owner_tenant,
        "the stranger must be a DIFFERENT tenant for this to measure anything"
    );

    // The evidence, acquired by the owner alone. `POST /v1/snapshots` records
    // the citation, which is the fact every other snapshot surface reads.
    let payload = b"the acquired report states the reserve was drawn down in March";
    let locator = "https://example.org/citation-gate-evidence";
    let acquire = |principal: String| {
        let client = client.clone();
        let base = base.clone();
        async move {
            let (status, reference) = post(
                &client,
                &base,
                "/v1/resources",
                &principal,
                &json!({ "original_locator": locator, "scheme": "https" }),
            )
            .await;
            assert_eq!(status, 200, "the reference submits: {reference}");
            let (status, snapshot) = post(
                &client,
                &base,
                "/v1/snapshots",
                &principal,
                &json!({
                    "reference_id": reference["resource_id"].as_str().unwrap(),
                    "original_locator": locator,
                    "final_locator": locator,
                    "resolver_id": "r0-https-fetcher",
                    "resolver_version": "0.1.0",
                    "raw_digest": reasonbraid_server::fetcher::digest_sha256_hex(payload),
                    "byte_length": payload.len(),
                    "media_type": "text/plain",
                    "bytes_base64": util::base64(payload),
                }),
            )
            .await;
            assert_eq!(status, 200, "the snapshot submits: {snapshot}");
            snapshot["snapshot_id"].as_str().unwrap().to_string()
        }
    };
    let snapshot_id = acquire(owner_id.clone()).await;

    // The three probes. Each is what a caller holding an `snp_` id can ask,
    // and before this repair each answered differently.
    let probe = |principal: String, snapshot: String, excerpt: &'static str| {
        let client = client.clone();
        let base = base.clone();
        async move {
            let (status, body) = post(
                &client,
                &base,
                "/v1/assessments",
                &principal,
                &json!({
                    "claim_id": "clm_citation_gate",
                    "snapshot_id": snapshot,
                    "assessment": "supports",
                    "author": principal,
                    "excerpt": excerpt,
                    "rationale": "the probe",
                }),
            )
            .await;
            (status, body)
        }
    };

    // (1) an identifier that names nothing;
    let absent = probe(
        stranger_id.clone(),
        "snp_00000000-0000-7000-8000-00000000dead".to_string(),
        "the reserve was drawn down",
    )
    .await;
    // (2) the owner's real snapshot, with an excerpt that is NOT in its bytes;
    let wrong_excerpt = probe(
        stranger_id.clone(),
        snapshot_id.clone(),
        "the reserve was REPLENISHED",
    )
    .await;
    // (3) the owner's real snapshot, with an excerpt that IS in its bytes —
    //     the content probe, which used to answer 200.
    let true_excerpt = probe(
        stranger_id.clone(),
        snapshot_id.clone(),
        "the reserve was drawn down",
    )
    .await;

    let answer = |(status, body): &(u16, Value)| {
        (
            *status,
            body["message"].as_str().unwrap_or_default().to_string(),
        )
    };
    let (absent_status, absent_message) = answer(&absent);
    let (wrong_status, wrong_message) = answer(&wrong_excerpt);
    let (true_status, true_message) = answer(&true_excerpt);

    assert_eq!(
        (absent_status, wrong_status, true_status),
        (400, 400, 400),
        "all three probes are refused identically: {absent:?} / {wrong_excerpt:?} / {true_excerpt:?}"
    );
    assert_eq!(
        absent_message, wrong_message,
        "a snapshot that does not exist and one the caller never cited must be \
         the SAME answer — separating them confirms the identifier exists"
    );
    assert_eq!(
        wrong_message, true_message,
        "a wrong excerpt and a TRUE one must be the same answer — a 200 here \
         says a chosen substring appears in bytes the caller never acquired"
    );
    assert!(
        !absent_message.contains("does not exist")
            && !absent_message.contains("excerpt")
            && absent_message.contains("not cited"),
        "the refusal names the caller's own missing citation and nothing about \
         the snapshot: {absent_message}"
    );

    // Nothing was written. A refusal that recorded a row would leave the
    // stranger's assertion in the store under a different name.
    let stranger_rows: i64 =
        sqlx::query_scalar("SELECT count(*) FROM claim_assessments WHERE authored_by_tenant = $1")
            .bind(stranger["tenant_id"].as_str().unwrap())
            .fetch_one(&pool)
            .await
            .expect("count the stranger's rows");
    assert_eq!(stranger_rows, 0, "the refused probes wrote nothing");

    // ── The bound: this is a binding, not a blackout ─────────────────────────
    //
    // The owner, which cited the snapshot, still assesses it.
    let (status, accepted) = probe(
        owner_id.clone(),
        snapshot_id.clone(),
        "the reserve was drawn down",
    )
    .await;
    assert_eq!(status, 200, "the citing tenant still assesses: {accepted}");
    let owner_assessment = accepted["assessment_id"].as_str().unwrap().to_string();

    // And the excerpt check still refuses the citing tenant by NAME, so the
    // gate did not swallow the §12.7 validation it sits in front of.
    let (status, fake) = probe(
        owner_id.clone(),
        snapshot_id.clone(),
        "the reserve was REPLENISHED",
    )
    .await;
    assert_eq!(
        status, 400,
        "the citing tenant's fake excerpt refuses: {fake}"
    );
    assert!(
        fake["message"].as_str().unwrap().contains("excerpt"),
        "the citing tenant keeps the diagnosis: {fake}"
    );

    // ── The supported cross-team path ────────────────────────────────────────
    //
    // A second tenant that ACQUIRES the same bytes records its own citation on
    // the replay (the book's "re-acquiring such a snapshot records the citation
    // and restores the read"), and then assesses the shared row normally.
    let replayed = acquire(stranger_id.clone()).await;
    assert_eq!(
        replayed, snapshot_id,
        "the same locator and digest replay to ONE shared snapshot"
    );
    let (status, shared) = probe(
        stranger_id.clone(),
        snapshot_id.clone(),
        "the reserve was drawn down",
    )
    .await;
    assert_eq!(
        status, 200,
        "the second tenant assesses evidence it has now cited: {shared}"
    );
    assert_ne!(
        shared["assessment_id"].as_str().unwrap(),
        owner_assessment,
        "two tenants asserting the same thing hold SEPARATE rows"
    );

    eprintln!(
        "standalone assessment citation gate: an absent id, a wrong excerpt and a TRUE excerpt over a snapshot the caller never cited are now ONE answer ({absent_status} `{absent_message}`) and wrote 0 rows; the citing tenant still assesses (200) and still gets the excerpt diagnosis (400); a second tenant that acquires the shared bytes assesses them normally"
    );
}

/// `SIGNOFF-REPAIR.11.14.3.2` (ROADMAP §13.2 step 2, "register context and
/// resource references"): a contribution's `EvidenceRef` resolves to a
/// registered `resource_references` row.
///
/// `EvidenceRef { uri, digest }` and `resource_references UNIQUE
/// (original_locator, expected_digest)` are the SAME key — a contributor naming
/// a URL at a digest is naming exactly the row the evidence store would hold —
/// and before this leaf nothing joined them: six `evidence_refs` hits in one
/// file, every one inert with respect to the store.
///
/// The key is the PAIR, which is what keeps one tenant's citation from blocking
/// another's: two tenants legitimately cite one locator at two digests (§12.6,
/// "a live Web page or branch can change"), and §12.1 forbids erasing that
/// security-relevant distinction. A digest-less citation is a §12.1 reference
/// with no pin, not a refusal.
#[tokio::test]
async fn the_contribution_citation_registers_a_resource_reference() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, human) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "cite-human" }),
    )
    .await;
    assert_eq!(status, 200, "the human enrolls: {human}");
    let human_id = human["principal_id"].as_str().unwrap().to_string();
    let tenant_id = human["tenant_id"].as_str().unwrap().to_string();

    let (status, created) = post(
        &client,
        &base,
        "/v1/threads",
        &human_id,
        &json!({
            "protocol_version": reasonbraid_core::PROTOCOL_VERSION,
            "operation": "thread.create",
            "request_id": reasonbraid_core::RequestId::new().to_string(),
            "idempotency_key": "cite-create",
            "body": {
                "tenant_id": tenant_id,
                "subject": "cite-subject",
                "objective": "probe the citation registration",
            },
            "client_context": {},
        }),
    )
    .await;
    assert_eq!(status, 200, "the thread creates: {created}");
    let thread_id = created["thread_id"].as_str().unwrap().to_string();

    let contribute =
        |principal: String, thread: String, tenant: String, key: String, refs: Value| {
            let client = client.clone();
            let base = base.clone();
            async move {
                let response = client
                    .post(format!("{base}/v1/threads/{thread}/commands"))
                    .header(PRINCIPAL_HEADER, &principal)
                    .json(&json!({
                        "protocol_version": reasonbraid_core::PROTOCOL_VERSION,
                        "operation": "thread.contribute",
                        "request_id": reasonbraid_core::RequestId::new().to_string(),
                        "idempotency_key": key,
                        "body": {
                            "tenant_id": tenant,
                            "content": "the position this citation supports",
                            "kind": "evidence_reference",
                            "evidence_refs": refs,
                        },
                        "client_context": {},
                    }))
                    .send()
                    .await
                    .expect("contribute request");
                let status = response.status().as_u16();
                let text = response.text().await.expect("contribute body");
                let value = serde_json::from_str(&text).unwrap_or_else(|_| json!({ "raw": text }));
                (status, value)
            }
        };

    let locator = "https://example.org/cited-report";
    let first_digest = format!("sha256:{}", "1".repeat(64));
    let second_digest = format!("sha256:{}", "2".repeat(64));

    // ── The finding: the citation registers the reference it names ──
    let (status, cited) = contribute(
        human_id.clone(),
        thread_id.clone(),
        tenant_id.clone(),
        "cite-first".into(),
        json!([{ "uri": locator, "digest": first_digest, "note": "the acquired report" }]),
    )
    .await;
    assert_eq!(status, 200, "the citing contribution commits: {cited}");

    let registered: Vec<(String, Option<String>, String, String)> = sqlx::query_as(
        "SELECT resource_id, expected_digest, scheme, submitted_by \
         FROM resource_references WHERE original_locator = $1 \
         ORDER BY created_at",
    )
    .bind(locator)
    .fetch_all(&pool)
    .await
    .expect("read the references the citation registered");
    assert_eq!(
        registered.len(),
        1,
        "a contribution's `EvidenceRef` resolves to nothing — the citation is a string beside an evidence store holding the row it describes: {registered:?}"
    );
    assert_eq!(
        registered[0].1.as_deref(),
        Some(first_digest.as_str()),
        "the citation's digest IS the reference's pin: {registered:?}"
    );
    assert_eq!(
        registered[0].2, "https",
        "the scheme is derived from the locator, never claimed beside it: {registered:?}"
    );
    // One namespace in `submitted_by`, whichever writer filled it: the citation
    // path computes the same `actor_handle_for_subject` the route does, rather
    // than storing the raw principal id beside the route's UUID handles.
    assert!(
        registered[0].3.starts_with("agt_"),
        "the citation records the route's own actor-handle shape: {registered:?}"
    );
    let first_resource_id = registered[0].0.clone();

    // The event carries the id, so the timeline's citation resolves.
    let (status, timeline) = get(
        &client,
        &base,
        &format!("/v1/threads/{thread_id}/events?tenant_id={tenant_id}"),
        &human_id,
    )
    .await;
    assert_eq!(status, 200, "the timeline reads: {timeline}");
    let event = timeline["events"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["event_type"] == json!("thread.contribution_submitted"))
        .cloned()
        .expect("the citing contribution is in the timeline");
    assert_eq!(
        event["body"]["evidence_refs"][0]["resource_id"],
        json!(first_resource_id),
        "the event names the registered row: {event}"
    );
    assert_eq!(event["body"]["evidence_refs"][0]["uri"], json!(locator));
    assert_eq!(
        event["body"]["evidence_refs"][0]["note"],
        json!("the acquired report"),
        "the contributor's own words survive the registration: {event}"
    );

    // The replay: the same pair cited again is the SAME row.
    let (status, again) = contribute(
        human_id.clone(),
        thread_id.clone(),
        tenant_id.clone(),
        "cite-replay".into(),
        json!([{ "uri": locator, "digest": first_digest }]),
    )
    .await;
    assert_eq!(status, 200, "the second citation commits: {again}");

    // ── The pair is the key, so another tenant's citation is never blocked ──
    let (status, other) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "cite-other" }),
    )
    .await;
    assert_eq!(status, 200, "the other human enrolls: {other}");
    let other_id = other["principal_id"].as_str().unwrap().to_string();
    let other_tenant = other["tenant_id"].as_str().unwrap().to_string();
    let (status, other_thread) = post(
        &client,
        &base,
        "/v1/threads",
        &other_id,
        &json!({
            "protocol_version": reasonbraid_core::PROTOCOL_VERSION,
            "operation": "thread.create",
            "request_id": reasonbraid_core::RequestId::new().to_string(),
            "idempotency_key": "cite-create-other",
            "body": {
                "tenant_id": other_tenant,
                "subject": "cite-subject-other",
                "objective": "cite the same page at a later version",
            },
            "client_context": {},
        }),
    )
    .await;
    assert_eq!(status, 200, "the other thread creates: {other_thread}");
    let other_thread_id = other_thread["thread_id"].as_str().unwrap().to_string();
    let (status, changed) = contribute(
        other_id.clone(),
        other_thread_id.clone(),
        other_tenant.clone(),
        "cite-changed".into(),
        json!([{ "uri": locator, "digest": second_digest }]),
    )
    .await;
    assert_eq!(
        status, 200,
        "a live page changes (§12.6) and the second digest is a second reference, never a refusal that leaks the first tenant's pin (§9.8): {changed}"
    );

    // ── The digest-less citation: a §12.1 reference with no pin ──
    let (status, unpinned) = contribute(
        human_id.clone(),
        thread_id.clone(),
        tenant_id.clone(),
        "cite-unpinned".into(),
        json!([{ "uri": locator }]),
    )
    .await;
    assert_eq!(status, 200, "the digest-less citation commits: {unpinned}");
    let (status, unpinned_again) = contribute(
        human_id.clone(),
        thread_id.clone(),
        tenant_id.clone(),
        "cite-unpinned-again".into(),
        json!([{ "uri": locator }]),
    )
    .await;
    assert_eq!(status, 200, "it replays: {unpinned_again}");

    let rows: Vec<(String, Option<String>)> = sqlx::query_as(
        "SELECT resource_id, expected_digest FROM resource_references \
         WHERE original_locator = $1 ORDER BY created_at",
    )
    .bind(locator)
    .fetch_all(&pool)
    .await
    .expect("read every reference for the locator");
    assert_eq!(
        rows.len(),
        3,
        "three distinct pins, each cited twice: {rows:?}"
    );
    assert_eq!(rows[0].1.as_deref(), Some(first_digest.as_str()));
    assert_eq!(rows[1].1.as_deref(), Some(second_digest.as_str()));
    assert_eq!(
        rows[2].1, None,
        "the unpinned citation registers with no digest: {rows:?}"
    );

    // ── One namespace, two writers: the shipped route replays the row the
    // citation minted, rather than conflicting with it ──
    let (status, resubmitted) = post(
        &client,
        &base,
        "/v1/resources",
        &human_id,
        &json!({
            "original_locator": locator,
            "scheme": "https",
            "expected_digest": first_digest,
        }),
    )
    .await;
    assert_eq!(
        status, 200,
        "the route replays the citation's row: {resubmitted}"
    );
    assert_eq!(resubmitted["resource_id"], json!(first_resource_id));
    assert_eq!(resubmitted["replayed"], json!(true));

    // ── Each refusal by NAME ──

    // A citation with no scheme is not a reference — §12.1's contract needs one.
    let (status, schemeless) = contribute(
        human_id.clone(),
        thread_id.clone(),
        tenant_id.clone(),
        "cite-schemeless".into(),
        json!([{ "uri": "see the internal wiki" }]),
    )
    .await;
    assert_eq!(status, 400, "the schemeless citation refuses: {schemeless}");
    assert!(
        schemeless["message"]
            .as_str()
            .unwrap_or_default()
            .contains("scheme"),
        "the refusal names what is missing: {schemeless}"
    );

    // A digest outside ADR-011 is refused with the scheme it should have used.
    let (status, malformed) = contribute(
        human_id.clone(),
        thread_id.clone(),
        tenant_id.clone(),
        "cite-malformed".into(),
        json!([{ "uri": "https://example.org/other", "digest": "md5:not-sha256" }]),
    )
    .await;
    assert_eq!(status, 400, "the malformed digest refuses: {malformed}");
    assert!(
        malformed["message"]
            .as_str()
            .unwrap_or_default()
            .contains("sha256"),
        "the refusal names the scheme: {malformed}"
    );

    // Nothing the refusals attempted was written.
    let total: i64 = sqlx::query_scalar("SELECT count(*) FROM resource_references")
        .fetch_one(&pool)
        .await
        .expect("count the references");
    assert_eq!(total, 3, "only the accepted citations registered");
    eprintln!(
        "citation registration: a contribution's evidence reference resolves to a §12.1 row keyed by the (locator, digest) PAIR; a changed page is a second reference rather than a cross-tenant refusal; a digest-less citation registers unpinned and replays; the shipped route replays the citation's own row; a schemeless uri and a non-ADR-011 digest are each refused by name; {total} rows written"
    );
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

    // `.7.1.2.1`: registering is a SITE act.
    site_fixture::provision(
        &pool,
        &human_id,
        &[reasonbraid_server::site_authority::Action::WorkflowRegister],
    )
    .await;

    // The custom profile composes the `moderate` step (the built-ins don't).
    let (status, registered) = post(
        &client,
        &base,
        "/v1/workflow-profiles",
        &human_id,
        &json!({
            "profile_id": "moderated_panel",
            "steps": ["solicit", "moderate", "decide"],
            "reason": "the moderation lane needs a moderate step",
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

// ── The role/version anchor (`SIGNOFF-REPAIR.3.3.4.11.1`) ─────────────────────
//
// The writer computed its next version as a read-then-write: read
// `current_version`, add one, insert under migration 0019's
// `UNIQUE (role_id, version)`. Two concurrent writers for one role therefore
// both read N, both wrote N+1, and the constraint turned the second into a
// RAISED violation the handler could only answer `500`. The repair serializes
// them at the role's own anchor row, which the writer now creates inside its
// acquisition because `SELECT … FOR UPDATE` over a row that does not exist yet
// locks nothing (`docs/knowledge/serializing-writers-at-a-row-that-may-not-exist.md`).

/// Concurrent writes for the SAME role all succeed, with consecutive versions
/// and every payload readable. This is the discriminating control: against the
/// superseded read-then-write it reports a `500` carrying
/// `duplicate key value violates unique constraint "profile_versions_role_id_version_key"`.
///
/// ⚠️ It is PROBABILISTIC, not deterministic, and that is a property of the
/// repair rather than of the fixture. No lock-holding fixture can discriminate
/// this one: the superseded writer's closing upsert and the repaired writer's
/// `SELECT … FOR UPDATE` contend on the SAME anchor row, so a held lock blocks
/// both. What changed is WHERE in each sequence the contention happens — the
/// repaired writer contends BEFORE it chooses a version number, the superseded
/// one after it had already written that number — and the only externally
/// visible consequence of that is the outcome under real concurrency. Four
/// writers over two rounds is what makes a false pass negligible rather than
/// merely unlikely; a single pair passed by luck on one baseline run.
#[tokio::test]
async fn concurrent_profile_writes_serialize_at_the_role_anchor() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, human) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "anchor-alice" }),
    )
    .await;
    assert_eq!(status, 200, "human enrolls: {human}");
    let tenant = human["tenant_id"].as_str().unwrap().to_string();
    let (status, role) = enroll(
        &client,
        &base,
        json!({ "kind": "role", "name": "anchor-agent", "tenant_id": tenant }),
    )
    .await;
    assert_eq!(status, 200, "role enrolls: {role}");
    let role_id = role["principal_id"].as_str().unwrap().to_string();
    let path = format!("/v1/profiles/{role_id}");

    // Each writer sends DISTINCT content, so a lost update is visible as a
    // missing content hash rather than hidden behind an identical one.
    let bodies: Vec<Value> = (0..4)
        .map(|n| {
            let mut body = profile(json!([{ "taxonomy_id": "code_review" }]));
            body["interests"] = json!([format!("writer {n}")]);
            body
        })
        .collect();

    // ROUND 1 races with NO anchor row in place — the case a lock-only repair
    // silently fails, because a row lock over an absent row is a no-op.
    let round_one = tokio::join!(
        put(&client, &base, &path, &role_id, &bodies[0]),
        put(&client, &base, &path, &role_id, &bodies[1]),
        put(&client, &base, &path, &role_id, &bodies[2]),
        put(&client, &base, &path, &role_id, &bodies[3]),
    );
    // ROUND 2 races with the anchor already present.
    let round_two = tokio::join!(
        put(&client, &base, &path, &role_id, &bodies[0]),
        put(&client, &base, &path, &role_id, &bodies[1]),
        put(&client, &base, &path, &role_id, &bodies[2]),
        put(&client, &base, &path, &role_id, &bodies[3]),
    );

    let mut versions = Vec::new();
    for (round, results) in [(1, round_one), (2, round_two)] {
        for (n, (status, body)) in [results.0, results.1, results.2, results.3]
            .iter()
            .enumerate()
        {
            assert_eq!(*status, 200, "round {round} writer {n}: {body}");
            versions.push(body["version"].as_i64().expect("a version"));
        }
    }
    versions.sort_unstable();
    assert_eq!(
        versions,
        (1..=8).collect::<Vec<i64>>(),
        "eight writers take eight consecutive versions"
    );

    // Serializing must not drop a payload: every version is stored, and the
    // four distinct contents are all present.
    let (status, listed) = get(
        &client,
        &base,
        &format!("/v1/profiles/{role_id}/versions"),
        &role_id,
    )
    .await;
    assert_eq!(status, 200, "the history reads: {listed}");
    let rows = listed["versions"].as_array().unwrap();
    assert_eq!(rows.len(), 8, "every version is stored: {listed}");
    let distinct: std::collections::BTreeSet<&str> = rows
        .iter()
        .map(|r| r["content_hash"].as_str().unwrap())
        .collect();
    assert_eq!(
        distinct.len(),
        4,
        "all four distinct payloads survived: {listed}"
    );
}

/// The write is ONE transaction now, so a refused lineage link leaves no
/// version row and no advanced anchor behind — the check and the write can no
/// longer see two different snapshots.
///
/// ⚠️ REGRESSION control, labelled as one rather than presented as proof: it
/// PASSES against the superseded code too, because that code also ran the
/// lineage check before calling the writer. What it defends is the ordering
/// staying that way now that both halves live in one transaction, where a
/// reordering would be silent. The discriminating control for this leaf is the
/// concurrency one above.
#[tokio::test]
async fn a_refused_lineage_link_leaves_no_version_and_no_anchor() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, human) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "lineage-alice" }),
    )
    .await;
    assert_eq!(status, 200, "human enrolls: {human}");
    let tenant = human["tenant_id"].as_str().unwrap().to_string();
    let (status, role) = enroll(
        &client,
        &base,
        json!({ "kind": "role", "name": "lineage-agent", "tenant_id": tenant }),
    )
    .await;
    assert_eq!(status, 200, "role enrolls: {role}");
    let role_id = role["principal_id"].as_str().unwrap().to_string();

    let mut body = profile(json!([]));
    body["incarnation_id"] = json!("inc_does_not_exist");
    let (status, refused) = put(
        &client,
        &base,
        &format!("/v1/profiles/{role_id}"),
        &role_id,
        &body,
    )
    .await;
    assert_eq!(status, 400, "the dangling lineage refuses: {refused}");

    let versions: i64 =
        sqlx::query_scalar("SELECT count(*) FROM profile_versions WHERE role_id = $1")
            .bind(&role_id)
            .fetch_one(&pool)
            .await
            .expect("count versions");
    assert_eq!(versions, 0, "the refusal wrote no version row");
    let anchors: i64 = sqlx::query_scalar("SELECT count(*) FROM agent_profiles WHERE role_id = $1")
        .bind(&role_id)
        .fetch_one(&pool)
        .await
        .expect("count anchors");
    assert_eq!(anchors, 0, "the refusal left no anchor row behind");
}

// ── Guarded owner attestation (`SIGNOFF-REPAIR.3.3.4.11.2`) ───────────────────
//
// The route ran three unconnected pieces: the role's tenant on the pool, an
// admission in its own transaction, then a read of the current profile on the
// pool and a write in a third transaction. The split read-modify-write is what
// made a concurrent attestation vanish.

/// 🔴 THE discriminating control: two administrators attesting two DIFFERENT
/// capabilities of the same role must BOTH survive. Against the superseded
/// route the second write published a profile carrying only its own upgrade,
/// and the first attestation was gone with no error reported to anyone.
#[tokio::test]
async fn two_concurrent_attestations_of_different_claims_both_survive() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, human) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "attest-owner" }),
    )
    .await;
    assert_eq!(status, 200, "human enrolls: {human}");
    let tenant = human["tenant_id"].as_str().unwrap().to_string();
    let owner = human["principal_id"].as_str().unwrap().to_string();
    let (status, role) = enroll(
        &client,
        &base,
        json!({ "kind": "role", "name": "attest-agent", "tenant_id": tenant }),
    )
    .await;
    assert_eq!(status, 200, "role enrolls: {role}");
    let role_id = role["principal_id"].as_str().unwrap().to_string();

    let body = profile(json!([
        { "taxonomy_id": "code_review", "confidence": "self_asserted" },
        { "taxonomy_id": "schema_design", "confidence": "self_asserted" },
    ]));
    let (status, written) = put(
        &client,
        &base,
        &format!("/v1/profiles/{role_id}"),
        &role_id,
        &body,
    )
    .await;
    assert_eq!(status, 200, "the role declares two claims: {written}");

    let attest = format!("/v1/profiles/{role_id}/attest");
    let review = json!({ "taxonomy_id": "code_review", "evidence_ref": "ev-review" });
    let schema = json!({ "taxonomy_id": "schema_design", "evidence_ref": "ev-schema" });
    let (left, right) = tokio::join!(
        post(&client, &base, &attest, &owner, &review),
        post(&client, &base, &attest, &owner, &schema),
    );
    assert_eq!(left.0, 200, "the first attestation: {:?}", left.1);
    assert_eq!(right.0, 200, "the second attestation: {:?}", right.1);

    // BOTH upgrades must be in the profile the role now publishes.
    let (status, current) = get(&client, &base, &format!("/v1/profiles/{role_id}"), &role_id).await;
    assert_eq!(status, 200, "the current profile reads: {current}");
    let claims = current["profile"]["capabilities"].as_array().unwrap();
    let attested: std::collections::BTreeMap<&str, &str> = claims
        .iter()
        .map(|c| {
            (
                c["taxonomy_id"].as_str().unwrap(),
                c["confidence"].as_str().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        attested.get("code_review"),
        Some(&"owner_attested"),
        "the first attestation survived: {current}"
    );
    assert_eq!(
        attested.get("schema_design"),
        Some(&"owner_attested"),
        "the second attestation survived: {current}"
    );
    let evidence: std::collections::BTreeMap<&str, &str> = claims
        .iter()
        .map(|c| {
            (
                c["taxonomy_id"].as_str().unwrap(),
                c["evidence_ref"].as_str().unwrap_or(""),
            )
        })
        .collect();
    assert_eq!(evidence.get("code_review"), Some(&"ev-review"), "{current}");
    assert_eq!(
        evidence.get("schema_design"),
        Some(&"ev-schema"),
        "{current}"
    );
}

/// The attestation records what it did, and both of its answers do: an applied
/// upgrade and a refusal that names no target the caller did not supply.
#[tokio::test]
async fn an_attestation_records_its_outcome_and_carries_its_receipt() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, human) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "attest-record-owner" }),
    )
    .await;
    assert_eq!(status, 200, "human enrolls: {human}");
    let tenant = human["tenant_id"].as_str().unwrap().to_string();
    let owner = human["principal_id"].as_str().unwrap().to_string();
    let (status, role) = enroll(
        &client,
        &base,
        json!({ "kind": "role", "name": "attest-record-agent", "tenant_id": tenant }),
    )
    .await;
    assert_eq!(status, 200, "role enrolls: {role}");
    let role_id = role["principal_id"].as_str().unwrap().to_string();
    let body = profile(json!([{ "taxonomy_id": "code_review", "confidence": "self_asserted" }]));
    let (status, _) = put(
        &client,
        &base,
        &format!("/v1/profiles/{role_id}"),
        &role_id,
        &body,
    )
    .await;
    assert_eq!(status, 200, "the role declares its claim");

    // The APPLIED answer.
    let attest = format!("/v1/profiles/{role_id}/attest");
    let response = client
        .post(format!("{base}{attest}"))
        .header(PRINCIPAL_HEADER, &owner)
        .json(&json!({ "taxonomy_id": "code_review", "evidence_ref": "ev-1" }))
        .send()
        .await
        .expect("attest request");
    assert_eq!(response.status().as_u16(), 200, "the attestation applies");
    let receipt = response
        .headers()
        .get("x-reasonbraid-authorization")
        .expect("the attestation carries its receipt")
        .to_str()
        .unwrap()
        .to_string();
    let written: Value = response.json().await.expect("attest json");
    let (kind, outcome, effected_at): (Value, Value, chrono::DateTime<chrono::Utc>) =
        sqlx::query_as(
            "SELECT operation, outcome, effected_at FROM administrative_effects WHERE record_id = $1",
        )
        .bind(&receipt)
        .fetch_one(&pool)
        .await
        .expect("the effect record exists");
    assert_eq!(kind["kind"], json!("capability_claim_attest"), "{kind}");
    assert_eq!(kind["role_id"], json!(role_id), "{kind}");
    assert_eq!(kind["taxonomy_id"], json!("code_review"), "{kind}");
    assert_eq!(outcome["kind"], json!("applied"), "{outcome}");
    // Compare INSTANTS, not their spellings: the response serializes UTC as `Z`
    // and chrono's `to_rfc3339` as `+00:00`, which are the same moment.
    let written_at: chrono::DateTime<chrono::Utc> = written["written_at"]
        .as_str()
        .unwrap()
        .parse()
        .expect("the response carries an RFC3339 instant");
    assert_eq!(
        written_at, effected_at,
        "the version is stamped with the transaction's own time"
    );

    // The REFUSED answer: same wire message for both idle states, and the
    // record is where they stop being the same fact.
    let response = client
        .post(format!("{base}{attest}"))
        .header(PRINCIPAL_HEADER, &owner)
        .json(&json!({ "taxonomy_id": "no_such_capability", "evidence_ref": "ev-2" }))
        .send()
        .await
        .expect("attest request");
    assert_eq!(response.status().as_u16(), 404, "the unknown claim refuses");
    let receipt = response
        .headers()
        .get("x-reasonbraid-authorization")
        .expect("the refusal carries its receipt too")
        .to_str()
        .unwrap()
        .to_string();
    let outcome: Value =
        sqlx::query_scalar("SELECT outcome FROM administrative_effects WHERE record_id = $1")
            .bind(&receipt)
            .fetch_one(&pool)
            .await
            .expect("the refusal's effect record exists");
    assert_eq!(outcome["kind"], json!("refused"), "{outcome}");
    assert_eq!(outcome["code"], json!("not_found"), "{outcome}");

    // The refusal wrote no version: the profile is still at the attested one.
    let (status, listed) = get(
        &client,
        &base,
        &format!("/v1/profiles/{role_id}/versions"),
        &role_id,
    )
    .await;
    assert_eq!(status, 200, "the history reads: {listed}");
    assert_eq!(
        listed["versions"].as_array().unwrap().len(),
        2,
        "the declaration and the attestation, and nothing from the refusal: {listed}"
    );
}

/// A foreign tenant's administrator attests nothing and learns nothing: the
/// admission is evaluated against the ROLE's tenant, so it is simply denied.
///
/// ⚠️ REGRESSION control, labelled rather than counted as proof: it PASSES
/// against the superseded route too, because that route already resolved the
/// role's tenant and admitted against it. `.11`'s census measured that this
/// family's tenant predicates were already correct — unlike `.10`'s — so what
/// this defends is that moving the admission inside the transaction did not
/// quietly widen it. The discriminating control for this leaf is the concurrent
/// attestation above.
#[tokio::test]
async fn a_foreign_administrator_cannot_attest_another_tenants_role() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, owner_human) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "attest-home" }),
    )
    .await;
    assert_eq!(status, 200, "the home human enrolls: {owner_human}");
    let home_tenant = owner_human["tenant_id"].as_str().unwrap().to_string();
    let (status, role) = enroll(
        &client,
        &base,
        json!({ "kind": "role", "name": "attest-home-agent", "tenant_id": home_tenant }),
    )
    .await;
    assert_eq!(status, 200, "role enrolls: {role}");
    let role_id = role["principal_id"].as_str().unwrap().to_string();
    let body = profile(json!([{ "taxonomy_id": "code_review", "confidence": "self_asserted" }]));
    let (status, _) = put(
        &client,
        &base,
        &format!("/v1/profiles/{role_id}"),
        &role_id,
        &body,
    )
    .await;
    assert_eq!(status, 200, "the role declares its claim");

    // A DIFFERENT tenant, with its own freshly bootstrapped administrator.
    let (status, stranger) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "attest-stranger" }),
    )
    .await;
    assert_eq!(status, 200, "the stranger enrolls: {stranger}");
    let stranger_id = stranger["principal_id"].as_str().unwrap().to_string();
    assert_ne!(
        stranger["tenant_id"].as_str().unwrap(),
        home_tenant,
        "the stranger administers a different tenant"
    );

    let (status, refused) = post(
        &client,
        &base,
        &format!("/v1/profiles/{role_id}/attest"),
        &stranger_id,
        &json!({ "taxonomy_id": "code_review", "evidence_ref": "forged" }),
    )
    .await;
    assert_eq!(
        status, 403,
        "the foreign administrator is denied: {refused}"
    );

    // Nothing moved: one version, still self_asserted.
    let (status, current) = get(&client, &base, &format!("/v1/profiles/{role_id}"), &role_id).await;
    assert_eq!(status, 200, "the profile reads: {current}");
    assert_eq!(current["version"], json!(1), "no version was written");
    assert_eq!(
        current["profile"]["capabilities"][0]["confidence"],
        json!("self_asserted"),
        "the claim was not upgraded: {current}"
    );
}

/// `SIGNOFF-REPAIR.11.14.3.4`: the §12.1 reference detail read is bound to the
/// tenant that REGISTERED it, and the pair replay is deliberately not.
///
/// `.11.14` marked `resource_references` *site-wide by design*, and that verdict
/// answered the COLUMN question — "can the row carry an owner?", to which
/// `UNIQUE (original_locator, expected_digest)` says no. It did not answer the
/// READ one. The three sibling tables got *"site-wide row, TENANT-BOUND READ"*
/// for a reason that applies here too: the locator is the research trail.
///
/// ⭐ **The two halves get different answers, and the difference is nameable.**
/// A snapshot's existence cannot be confirmed without presenting its BYTES, so
/// `.11.14.1`'s binding closed both halves at once. A reference's existence is
/// confirmed by presenting a LOCATOR, which anyone can type — so the replay is
/// structural and stays, while the detail read is bound.
///
/// ⭐ **Binding the detail read breaks no reachable caller**, and that is why it
/// is not a compatibility cost: every way to obtain a `res_…` id goes through
/// the pair replay, which now RECORDS the caller's registration. What changes is
/// the price of admission — a bare opaque handle used to be enough, and the
/// locator is now required, which is the very thing the row would disclose.
#[tokio::test]
async fn the_reference_read_is_bound_to_the_registering_tenant() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, owner) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "reference-owner" }),
    )
    .await;
    assert_eq!(status, 200, "the registering tenant enrols: {owner}");
    let owner_id = owner["principal_id"].as_str().unwrap().to_string();

    let (status, stranger) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "reference-stranger" }),
    )
    .await;
    assert_eq!(status, 200, "the second tenant enrols: {stranger}");
    let stranger_id = stranger["principal_id"].as_str().unwrap().to_string();
    assert_ne!(
        stranger["tenant_id"].as_str().unwrap(),
        owner["tenant_id"].as_str().unwrap(),
        "the stranger must be a DIFFERENT tenant for this to measure anything"
    );

    // The reference the owner registers. Every field here is one the read
    // discloses, and `purpose` is free text one tenant wrote about its own
    // research.
    let locator = "https://internal.example.org/q3-reserve-review";
    let digest = format!("sha256:{}", "a".repeat(64));
    let reference_body = json!({
        "original_locator": locator,
        "scheme": "https",
        "expected_digest": digest,
        "credential_binding_ref": "the-owner-binding",
        "purpose": "the reserve review the owner is running",
        "visibility_scope": "tenant",
        "risk_class": "high",
    });
    let (status, registered) =
        post(&client, &base, "/v1/resources", &owner_id, &reference_body).await;
    assert_eq!(status, 200, "the reference registers: {registered}");
    let resource_id = registered["resource_id"].as_str().unwrap().to_string();
    assert_eq!(
        registered["replayed"],
        json!(false),
        "the first registration is not a replay: {registered}"
    );

    // ── The finding: a bare opaque handle used to be the whole predicate ─────
    let (status, read) = get(
        &client,
        &base,
        &format!("/v1/resources/{resource_id}"),
        &stranger_id,
    )
    .await;
    assert_eq!(
        status, 404,
        "a tenant that did not register the reference reads nothing: {read}"
    );
    // The two answers must depend only on what the CALLER supplied, never on
    // whether the reference exists. Both echo the id the request itself named —
    // the same convention `cited_snapshot` already uses — so the assertion is
    // over the template rather than over the literal strings.
    let absent_id = "res_00000000-0000-7000-8000-00000000dead";
    let (absent_status, absent) = get(
        &client,
        &base,
        &format!("/v1/resources/{absent_id}"),
        &stranger_id,
    )
    .await;
    assert_eq!(absent_status, 404, "an absent id reads nothing: {absent}");
    assert_eq!(
        (read["code"].as_str(), read["message"].as_str()),
        (
            Some("not_found"),
            Some(format!("no reference `{resource_id}`").as_str())
        ),
        "a registered reference the caller did not register is reported ABSENT: {read}"
    );
    assert_eq!(
        (absent["code"].as_str(), absent["message"].as_str()),
        (
            Some("not_found"),
            Some(format!("no reference `{absent_id}`").as_str())
        ),
        "…in exactly the words a truly absent id gets, so the refusal cannot \
         confirm that an identifier exists: {absent}"
    );

    // The resolve verb reads the same row and takes the same binding. Without
    // it, a caller refused the READ could still drive an acquisition off the
    // reference — and, with the gated R5 pack on, off its credential binding.
    let (status, resolved) = post(
        &client,
        &base,
        &format!("/v1/resources/{resource_id}/resolve"),
        &stranger_id,
        &json!({ "required_sandbox": "none", "required_egress": "public" }),
    )
    .await;
    assert_eq!(
        status, 404,
        "the resolve verb is bound to the same registration: {resolved}"
    );

    // ── The bound: this is a binding, not a blackout ─────────────────────────
    let (status, own) = get(
        &client,
        &base,
        &format!("/v1/resources/{resource_id}"),
        &owner_id,
    )
    .await;
    assert_eq!(status, 200, "the registering tenant still reads: {own}");
    assert_eq!(
        own["reference"]["original_locator"],
        json!(locator),
        "the owner's own row is unchanged: {own}"
    );

    // ── The supported path, and the limit that is DELIBERATELY kept ──────────
    //
    // The second tenant registers the SAME pair. §12.1's key makes that one
    // shared row, so it replays — and the replay records the second
    // registration, exactly as a snapshot re-acquisition records the second
    // citation. ⚠️ The `replayed: true` IS an existence confirmation, and it
    // cannot be closed without breaking the pair key §12.1 and §12.6 require.
    // The caller must already know the locator AND the digest, which is the
    // width this control pins rather than leaves implicit.
    let (status, replayed) = post(
        &client,
        &base,
        "/v1/resources",
        &stranger_id,
        &reference_body,
    )
    .await;
    assert_eq!(status, 200, "the second registration replays: {replayed}");
    assert_eq!(
        replayed["resource_id"].as_str().unwrap(),
        resource_id,
        "the pair key still yields ONE shared row: {replayed}"
    );
    assert_eq!(
        replayed["replayed"],
        json!(true),
        "the pair replay is kept — it is the §12.1 identity, and closing it \
         would make one tenant's pin uncitable by another: {replayed}"
    );

    let (status, now_readable) = get(
        &client,
        &base,
        &format!("/v1/resources/{resource_id}"),
        &stranger_id,
    )
    .await;
    assert_eq!(
        status, 200,
        "the replay recorded the second registration, so the read is restored: {now_readable}"
    );

    let registrations: i64 =
        sqlx::query_scalar("SELECT count(*) FROM reference_registrations WHERE resource_id = $1")
            .bind(&resource_id)
            .fetch_one(&pool)
            .await
            .expect("count the registrations");
    assert_eq!(
        registrations, 2,
        "one shared row, two registering tenants — the dedupe the pair key \
         exists for is untouched"
    );

    eprintln!(
        "reference read binding: a second tenant holding the res_ id reads 404 — the same answer an absent id gets — and its resolve is refused identically; the registering tenant still reads its own row; registering the same pair replays to the SAME resource_id, records the second registration and restores the read; {registrations} registrations on one shared row"
    );
}

/// `SIGNOFF-REPAIR.11.14.3.6`: a reference's `expected_digest` constrains the
/// snapshots filed against it, and an unpinned reference still holds many.
///
/// The pin was a §12.1 field a caller supplied to say *"these are the bytes I
/// expect"*, and the census found **zero** readers of a STORED one:
/// `snapshots::submit` asked `SELECT EXISTS … WHERE resource_id = $1` and never
/// looked at the column. So a reference pinned to one digest accepted a snapshot
/// of entirely different bytes.
///
/// ⭐ **Enforcing it is what makes `.11.14.3.2`'s pair key mean something.** That
/// leaf made `(original_locator, expected_digest)` the reference's identity so a
/// changed page would be a SECOND reference rather than an erased distinction.
/// Without a checkpoint, both rows accepted any bytes and the distinction the
/// key was created to preserve was preserved nowhere.
///
/// ⚠️ **The plural is not forbidden — it is relocated.** `evidence_snapshots`
/// replays on `(reference_id, raw_digest)`, so one reference holds many
/// versions; that stays true of an UNPINNED reference, which is what §12.6's
/// changing page needs. A pin says the opposite about its own reference, and the
/// two compose.
#[tokio::test]
async fn a_pinned_reference_accepts_only_the_bytes_it_names() {
    const PINNED_REPORT: &str = "https://example.org/pinned-report";
    const LIVING_REPORT: &str = "https://example.org/living-report";
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, human) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "pin-human" }),
    )
    .await;
    assert_eq!(status, 200, "the human enrols: {human}");
    let human_id = human["principal_id"].as_str().unwrap().to_string();

    let pinned_bytes = b"the audited figure is 41.2 per cent";
    let other_bytes = b"the audited figure is 62.8 per cent";
    let pinned_digest = reasonbraid_server::fetcher::digest_sha256_hex(pinned_bytes);
    let other_digest = reasonbraid_server::fetcher::digest_sha256_hex(other_bytes);
    assert_ne!(pinned_digest, other_digest);

    let register = |locator: &'static str, digest: Option<String>| {
        let client = client.clone();
        let base = base.clone();
        let human_id = human_id.clone();
        async move {
            let mut body = json!({ "original_locator": locator, "scheme": "https" });
            if let Some(digest) = digest {
                body["expected_digest"] = json!(digest);
            }
            let (status, reference) = post(&client, &base, "/v1/resources", &human_id, &body).await;
            assert_eq!(status, 200, "the reference registers: {reference}");
            reference["resource_id"].as_str().unwrap().to_string()
        }
    };
    // ⚠️ The locator rides the call because a snapshot must name the SAME
    // `original_locator` as the reference it is filed against
    // (`SIGNOFF-REPAIR.11.14.3.13`). The first version of this control passed the
    // pinned report's locator for BOTH references, which was simply wrong about
    // which document the unpinned snapshot was of, and nothing checked it.
    let snapshot = |reference_id: String, locator: &'static str, bytes: &'static [u8]| {
        let client = client.clone();
        let base = base.clone();
        let human_id = human_id.clone();
        async move {
            post(
                &client,
                &base,
                "/v1/snapshots",
                &human_id,
                &json!({
                    "reference_id": reference_id,
                    "original_locator": locator,
                    "final_locator": locator,
                    "resolver_id": "r0-https-fetcher",
                    "resolver_version": "0.1.0",
                    "raw_digest": reasonbraid_server::fetcher::digest_sha256_hex(bytes),
                    "byte_length": bytes.len(),
                    "media_type": "text/plain",
                    "bytes_base64": util::base64(bytes),
                }),
            )
            .await
        }
    };

    // ── The finding: a pinned reference accepted bytes it does not name ──────
    let pinned_ref = register(PINNED_REPORT, Some(pinned_digest.clone())).await;
    let (status, refused) = snapshot(pinned_ref.clone(), PINNED_REPORT, other_bytes).await;
    assert_eq!(
        status, 400,
        "a snapshot of other bytes is refused against a pinned reference: {refused}"
    );
    assert!(
        refused["message"].as_str().unwrap().contains("pinned to"),
        "the refusal names the pin as its reason: {refused}"
    );
    // ⛔ And it names NEITHER digest. The pinned one belongs to a reference this
    // route does not check the caller may read, so quoting it would turn the
    // refusal into an oracle over a `res_…` id.
    let message = refused["message"].as_str().unwrap();
    assert!(
        !message.contains(&pinned_digest) && !message.contains(&other_digest),
        "the refusal carries no digest: {refused}"
    );
    let stored: i64 =
        sqlx::query_scalar("SELECT count(*) FROM evidence_snapshots WHERE reference_id = $1")
            .bind(&pinned_ref)
            .fetch_one(&pool)
            .await
            .expect("count the pinned reference's snapshots");
    assert_eq!(stored, 0, "the refused snapshot wrote nothing");

    // ── The bound: the pinned bytes themselves are accepted ─────────────────
    let (status, accepted) = snapshot(pinned_ref.clone(), PINNED_REPORT, pinned_bytes).await;
    assert_eq!(
        status, 200,
        "the reference's OWN bytes are accepted: {accepted}"
    );
    assert_eq!(accepted["replay"], json!(false), "{accepted}");

    // ── The plural is relocated, not forbidden ──────────────────────────────
    //
    // An UNPINNED reference still holds every version §12.6's changing page
    // produces. A repair that enforced a pin nobody declared would fail here.
    let unpinned_ref = register(LIVING_REPORT, None).await;
    for bytes in [pinned_bytes.as_slice(), other_bytes.as_slice()] {
        let (status, stored) = snapshot(unpinned_ref.clone(), LIVING_REPORT, bytes).await;
        assert_eq!(
            status, 200,
            "an unpinned reference holds this version too: {stored}"
        );
    }
    let versions: i64 =
        sqlx::query_scalar("SELECT count(*) FROM evidence_snapshots WHERE reference_id = $1")
            .bind(&unpinned_ref)
            .fetch_one(&pool)
            .await
            .expect("count the unpinned reference's snapshots");
    assert_eq!(
        versions, 2,
        "one unpinned reference, two versions — the changing page §12.6 describes"
    );

    // ── And the changed page's own route: a SECOND reference ────────────────
    //
    // This is what makes the refusal above a redirection rather than a dead end,
    // and it is `.11.14.3.2`'s key doing the work it was created for.
    let second_ref = register(PINNED_REPORT, Some(other_digest.clone())).await;
    assert_ne!(
        second_ref, pinned_ref,
        "the same locator at a DIFFERENT digest is a second reference"
    );
    let (status, moved) = snapshot(second_ref.clone(), PINNED_REPORT, other_bytes).await;
    assert_eq!(
        status, 200,
        "the changed page's bytes are acquired against the reference that names them: {moved}"
    );

    eprintln!(
        "reference pin: a pinned reference refuses a snapshot of other bytes (400, no digest quoted, 0 rows written) and accepts its own; an UNPINNED reference still holds {versions} versions; the same locator at the other digest is a second reference and acquires normally"
    );
}

/// Declare that THIS deployment's R0 pack serves the origin's scheme — the
/// `widen_r2_to_http` shape, for the acquisition-only pack.
async fn widen_r0_to_http(pool: &PgPool) -> Value {
    let shipped: Value = sqlx::query_scalar(
        "SELECT schemes FROM resolver_capabilities WHERE resolver_id = 'r0-https-fetcher'",
    )
    .fetch_one(pool)
    .await
    .expect("the R0 pack is installed");
    assert_eq!(
        shipped,
        json!(["https"]),
        "the migration's R0 schemes are the baseline this control widens"
    );
    sqlx::query(
        "UPDATE resolver_capabilities SET schemes = $1::jsonb \
         WHERE resolver_id = 'r0-https-fetcher'",
    )
    .bind(json!(["https", "http"]))
    .execute(pool)
    .await
    .expect("this deployment's R0 pack also serves the local origin");
    shipped
}

async fn restore_r0_schemes(pool: &PgPool, shipped: &Value) {
    sqlx::query(
        "UPDATE resolver_capabilities SET schemes = $1::jsonb \
         WHERE resolver_id = 'r0-https-fetcher'",
    )
    .bind(shipped)
    .execute(pool)
    .await
    .expect("the shipped R0 schemes are restored");
}

/// `SIGNOFF-REPAIR.11.14.3.12`: a resolution whose evidence did not persist says
/// so, on every arm rather than on one.
///
/// `.7.4.2` decided this for the R2 arm and wrote the reason at the call site —
/// *"A failed snapshot is NOT a successful acquisition … a caller was told the
/// document had been acquired while no evidence row and no derivation existed"*.
/// The R0 and R5 arms were never migrated onto it: both called
/// `let _ = crate::snapshots::submit(…)` and set `outcome.acquisition` regardless.
///
/// ⭐ **The disposition was therefore not an open question** — it was a shipped
/// decision with two unconverted call sites, which is a different and cheaper
/// thing to find. The leaf that opened this one posed it as a three-way choice;
/// reading `.7.4.2`'s own call site settled it.
///
/// The failure is made REACHABLE by `.11.14.3.6`: a pinned reference whose page
/// has drifted is a common, caller-meaningful reason for the store to refuse,
/// where before the only cause was a storage fault.
#[tokio::test]
async fn a_resolution_says_when_its_evidence_was_not_persisted() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };

    let (origin, _origin_handle) = start_feed_origin().await;
    let port = origin.port();
    let server = TestServer::start_with_router(
        &pool,
        reasonbraid_server::api_router_with_acquisition(
            pool.clone(),
            false,
            std::sync::Arc::new(reasonbraid_server::broker::Broker::default()),
            admitting_fetcher(port),
        ),
    )
    .await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, human) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "unstored-human" }),
    )
    .await;
    assert_eq!(status, 200, "the human enrols: {human}");
    let human_id = human["principal_id"].as_str().unwrap().to_string();

    // The reference is pinned to bytes the origin does NOT serve, so the store
    // refuses the snapshot while the acquisition itself succeeds.
    let locator = format!("http://127.0.0.1:{port}/feed.xml");
    let wrong_pin = reasonbraid_server::fetcher::digest_sha256_hex(b"not what the origin serves");
    let (status, reference) = post(
        &client,
        &base,
        "/v1/resources",
        &human_id,
        &json!({
            "original_locator": locator,
            "scheme": "http",
            "expected_digest": wrong_pin,
        }),
    )
    .await;
    assert_eq!(status, 200, "the pinned reference registers: {reference}");
    let resource_id = reference["resource_id"].as_str().unwrap().to_string();

    // The registry row is shared with every other suite in this database, so it
    // is restored before anything is asserted.
    let shipped_schemes = widen_r0_to_http(&pool).await;
    let (status, outcome) = post(
        &client,
        &base,
        &format!("/v1/resources/{resource_id}/resolve"),
        &human_id,
        &json!({ "required_sandbox": "none", "required_egress": "listed" }),
    )
    .await;
    restore_r0_schemes(&pool, &shipped_schemes).await;

    assert_eq!(status, 200, "the resolution answers: {outcome}");
    assert_eq!(
        outcome["resolvers"][0],
        json!("r0-https-fetcher"),
        "the R0 pack is the ranked resolver for this control: {outcome}"
    );

    // ── The finding: the receipt without the evidence ───────────────────────
    let stored: i64 =
        sqlx::query_scalar("SELECT count(*) FROM evidence_snapshots WHERE reference_id = $1")
            .bind(&resource_id)
            .fetch_one(&pool)
            .await
            .expect("count the reference's snapshots");
    assert_eq!(stored, 0, "the pin refused the snapshot: {outcome}");
    assert!(
        outcome.get("acquisition").is_none(),
        "no evidence row means no acquisition receipt — a caller must not be \
         told the document was acquired while nothing was stored: {outcome}"
    );
    assert_eq!(
        outcome["acquisition_error"]["kind"],
        json!("evidence_unstored"),
        "the resolution NAMES the persistence failure, in the vocabulary \
         `.7.4.2` already shipped on the R2 arm: {outcome}"
    );

    // ── The bound: an unpinned reference through the same deployment ────────
    //
    // A repair that reported `evidence_unstored` for every resolution would
    // fail here.
    let (status, unpinned) = post(
        &client,
        &base,
        "/v1/resources",
        &human_id,
        &json!({ "original_locator": locator, "scheme": "http" }),
    )
    .await;
    assert_eq!(status, 200, "the unpinned reference registers: {unpinned}");
    let unpinned_id = unpinned["resource_id"].as_str().unwrap().to_string();

    let shipped_schemes = widen_r0_to_http(&pool).await;
    let (status, ok) = post(
        &client,
        &base,
        &format!("/v1/resources/{unpinned_id}/resolve"),
        &human_id,
        &json!({ "required_sandbox": "none", "required_egress": "listed" }),
    )
    .await;
    restore_r0_schemes(&pool, &shipped_schemes).await;
    assert_eq!(status, 200, "the unpinned resolution answers: {ok}");
    assert!(
        ok.get("acquisition_error").is_none(),
        "a resolution whose evidence DID persist names no error: {ok}"
    );
    assert!(
        ok.get("acquisition").is_some(),
        "…and carries its receipt: {ok}"
    );
    let stored: i64 =
        sqlx::query_scalar("SELECT count(*) FROM evidence_snapshots WHERE reference_id = $1")
            .bind(&unpinned_id)
            .fetch_one(&pool)
            .await
            .expect("count the unpinned reference's snapshots");
    assert_eq!(stored, 1, "the evidence is there: {ok}");

    eprintln!(
        "resolution persistence: a pinned reference whose page drifted answers `evidence_unstored` with NO acquisition receipt and 0 snapshots, instead of a silent receipt; the unpinned reference through the same deployment acquires, persists 1 snapshot and names no error"
    );
}

/// `SIGNOFF-REPAIR.11.14.3.11`: a snapshot is filed against a reference THIS
/// TENANT registered, and a reference it did not register is indistinguishable
/// from one that does not exist.
///
/// ⭐ **This is a gap in `.11.14.3.4`'s own census, one commit earlier.** That
/// census enumerated the routes under `/v1/resources` — three, two of them
/// unbound, both bound — and `POST /v1/snapshots` names a `reference_id` in its
/// **body**, so a route-prefix enumeration cannot see it. The census was not
/// wrong; it was silent, which is the more dangerous failure.
/// (`docs/knowledge/a-census-is-as-wide-as-its-key.md`.)
///
/// ⚠️ **The leaf's own warning is answered rather than obeyed.** It said the
/// asymmetry that made the snapshot family's replay safe — a submission carries
/// the BYTES — cuts the other way here. It does, and it is still not a reason to
/// leave the write open: a `SnapshotSubmission` also carries `original_locator`,
/// so a caller submitting one already holds everything registration needs. The
/// bound arm below is that path.
#[tokio::test]
async fn a_snapshot_is_filed_against_a_reference_this_tenant_registered() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, owner) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "snapshot-write-owner" }),
    )
    .await;
    assert_eq!(status, 200, "the registering tenant enrols: {owner}");
    let owner_id = owner["principal_id"].as_str().unwrap().to_string();

    let (status, stranger) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "snapshot-write-stranger" }),
    )
    .await;
    assert_eq!(status, 200, "the second tenant enrols: {stranger}");
    let stranger_id = stranger["principal_id"].as_str().unwrap().to_string();
    assert_ne!(
        stranger["tenant_id"].as_str().unwrap(),
        owner["tenant_id"].as_str().unwrap()
    );

    let locator = "https://example.org/write-bound-report";
    let payload = b"the acquired report states the write surface was open";
    let (status, reference) = post(
        &client,
        &base,
        "/v1/resources",
        &owner_id,
        &json!({ "original_locator": locator, "scheme": "https" }),
    )
    .await;
    assert_eq!(status, 200, "the reference registers: {reference}");
    let resource_id = reference["resource_id"].as_str().unwrap().to_string();

    let submit = |principal: String, reference_id: String| {
        let client = client.clone();
        let base = base.clone();
        async move {
            post(
                &client,
                &base,
                "/v1/snapshots",
                &principal,
                &json!({
                    "reference_id": reference_id,
                    "original_locator": locator,
                    "final_locator": locator,
                    "resolver_id": "r0-https-fetcher",
                    "resolver_version": "0.1.0",
                    "raw_digest": reasonbraid_server::fetcher::digest_sha256_hex(payload),
                    "byte_length": payload.len(),
                    "media_type": "text/plain",
                    "bytes_base64": util::base64(payload),
                }),
            )
            .await
        }
    };

    // ── The finding: a foreign reference answered differently from an absent one
    let absent_id = "res_00000000-0000-7000-8000-00000000dead";
    let (foreign_status, foreign) = submit(stranger_id.clone(), resource_id.clone()).await;
    let (missing_status, missing) = submit(stranger_id.clone(), absent_id.to_string()).await;
    assert_eq!(
        (foreign_status, missing_status),
        (400, 400),
        "a foreign reference and an absent one are refused alike: {foreign} / {missing}"
    );
    assert_eq!(
        (foreign["code"].clone(), foreign["message"].clone()),
        (missing["code"].clone(), missing["message"].clone()),
        "…in the SAME words, so the write surface cannot confirm that a `res_` id \
         exists: {foreign} / {missing}"
    );
    let stored: i64 =
        sqlx::query_scalar("SELECT count(*) FROM evidence_snapshots WHERE reference_id = $1")
            .bind(&resource_id)
            .fetch_one(&pool)
            .await
            .expect("count the reference's snapshots");
    assert_eq!(
        stored, 0,
        "the refused submissions attached nothing to another tenant's reference"
    );

    // ── The bound: the registering tenant files its own snapshot ────────────
    let (status, own) = submit(owner_id.clone(), resource_id.clone()).await;
    assert_eq!(status, 200, "the registering tenant still files: {own}");
    let snapshot_id = own["snapshot_id"].as_str().unwrap().to_string();

    // ── The supported path for the second tenant ────────────────────────────
    //
    // A submission carries `original_locator`, so a caller that can make one can
    // register the pair — which returns the SAME reference and records the
    // registration. This is the arm that makes the binding a binding rather than
    // a blackout.
    let (status, replayed) = post(
        &client,
        &base,
        "/v1/resources",
        &stranger_id,
        &json!({ "original_locator": locator, "scheme": "https" }),
    )
    .await;
    assert_eq!(status, 200, "the second registration replays: {replayed}");
    assert_eq!(
        replayed["resource_id"].as_str().unwrap(),
        resource_id,
        "one shared reference row: {replayed}"
    );
    let (status, now_allowed) = submit(stranger_id.clone(), resource_id.clone()).await;
    assert_eq!(
        status, 200,
        "the registered second tenant files against the shared reference: {now_allowed}"
    );
    assert_eq!(
        now_allowed["snapshot_id"].as_str().unwrap(),
        snapshot_id,
        "the same reference and digest is the REPLAY — one shared snapshot row"
    );
    assert_eq!(now_allowed["replay"], json!(true), "{now_allowed}");

    let citations: i64 =
        sqlx::query_scalar("SELECT count(*) FROM evidence_citations WHERE snapshot_id = $1")
            .bind(&snapshot_id)
            .fetch_one(&pool)
            .await
            .expect("count the citations");
    assert_eq!(
        citations, 2,
        "both tenants cite the shared row — the replay records the second citation"
    );

    eprintln!(
        "snapshot write binding: a foreign reference and an absent one are ONE answer and attach nothing; the registering tenant files normally; the second tenant registers the same locator, replays to the same reference and then files — one shared snapshot row with {citations} citations"
    );
}

/// `SIGNOFF-REPAIR.11.14.3.13`: a snapshot names the SAME `original_locator` as
/// the reference it is filed against, and `final_locator` stays free.
///
/// `snapshots::submit` bound the submission's value straight into the insert and
/// never compared it with the reference's, so a snapshot could say it was an
/// acquisition of one document while its reference named another. §12.6 asks the
/// snapshot to carry "original reference and resolved final locator"; those were
/// two facts that need not agree.
///
/// ⚠️ **The comparison is byte equality on purpose.** §12.1 keeps the original
/// locator immutable and canonicalization separate and scheme-specific, so
/// normalising either side here would BE a canonicalization decision rather than
/// a check. Refusing a disagreement erases nothing.
///
/// ⭐ **Nothing in the product reads this column to make a decision** — it is
/// written, mapped and returned on the read surfaces, and no resolver or gate
/// consults it. That is what makes the defect an evidence-integrity one rather
/// than a routing one, and it is why the refusal is the whole repair.
#[tokio::test]
async fn a_snapshot_names_the_locator_its_reference_names() {
    const REGISTERED: &str = "https://example.org/the-registered-report";
    const CLAIMED: &str = "https://example.org/some-other-document";
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, human) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "locator-human" }),
    )
    .await;
    assert_eq!(status, 200, "the human enrols: {human}");
    let human_id = human["principal_id"].as_str().unwrap().to_string();

    let payload = b"the acquired report states the locator was never compared";
    let (status, reference) = post(
        &client,
        &base,
        "/v1/resources",
        &human_id,
        &json!({ "original_locator": REGISTERED, "scheme": "https" }),
    )
    .await;
    assert_eq!(status, 200, "the reference registers: {reference}");
    let resource_id = reference["resource_id"].as_str().unwrap().to_string();

    let file = |original: &'static str, final_locator: &'static str| {
        let client = client.clone();
        let base = base.clone();
        let human_id = human_id.clone();
        let resource_id = resource_id.clone();
        async move {
            post(
                &client,
                &base,
                "/v1/snapshots",
                &human_id,
                &json!({
                    "reference_id": resource_id,
                    "original_locator": original,
                    "final_locator": final_locator,
                    "resolver_id": "r0-https-fetcher",
                    "resolver_version": "0.1.0",
                    "raw_digest": reasonbraid_server::fetcher::digest_sha256_hex(payload),
                    "byte_length": payload.len(),
                    "media_type": "text/plain",
                    "bytes_base64": util::base64(payload),
                }),
            )
            .await
        }
    };

    // ── The finding: a snapshot that names a different document ─────────────
    let (status, refused) = file(CLAIMED, CLAIMED).await;
    assert_eq!(
        status, 400,
        "a snapshot cannot name a document its reference does not: {refused}"
    );
    let message = refused["message"].as_str().unwrap();
    assert!(
        message.contains(CLAIMED) && message.contains(REGISTERED),
        "the refusal names BOTH, because the caller registered the reference and \
         may read its locator: {refused}"
    );
    let stored: i64 =
        sqlx::query_scalar("SELECT count(*) FROM evidence_snapshots WHERE reference_id = $1")
            .bind(&resource_id)
            .fetch_one(&pool)
            .await
            .expect("count the reference's snapshots");
    assert_eq!(stored, 0, "the refused submission wrote nothing");

    // ── The bound: the agreeing submission, with a redirect ─────────────────
    //
    // `final_locator` is where the acquisition ENDED and is deliberately free —
    // a repair that compared it too would fail here, and it would be wrong:
    // §12.6 records both precisely because a redirect moves one of them.
    let (status, accepted) = file(REGISTERED, "https://cdn.example.net/the-report").await;
    assert_eq!(
        status, 200,
        "the agreeing submission is accepted, redirect and all: {accepted}"
    );
    let snapshot_id = accepted["snapshot_id"].as_str().unwrap().to_string();
    let (recorded_original, recorded_final): (String, String) = sqlx::query_as(
        "SELECT original_locator, final_locator FROM evidence_snapshots WHERE snapshot_id = $1",
    )
    .bind(&snapshot_id)
    .fetch_one(&pool)
    .await
    .expect("read the stored locators");
    assert_eq!(recorded_original, REGISTERED);
    assert_eq!(recorded_final, "https://cdn.example.net/the-report");

    eprintln!(
        "snapshot locator: a snapshot naming a document its reference does not is refused (400, naming both, 0 rows written); the agreeing submission is accepted and keeps a DIFFERENT final_locator, which a redirect legitimately moves"
    );
}

/// `SIGNOFF-REPAIR.11.14.3.5`: the §12.1 fields `POST /v1/resources` used to
/// take on trust — the `scheme` nothing validated, and the omissions that
/// bypassed `migrations/0023`'s declared column defaults.
///
/// Three mechanisms sit in that leaf's goal line and each gets its own arm here,
/// because a leaf that reproduces two of three and decides all three is the
/// over-reporting `.11.15` exists to catch.
///
/// 🔴 **Two of the three are REFUTATIONS rather than repairs**, and the first is
/// the one to read: `scheme` is the resolver-selection key, not the locator's URI
/// scheme, and a repair that treated it as the latter was written and then
/// refused by two of this suite's own controls.
#[tokio::test]
async fn a_reference_s_declared_fields_are_checked_or_defaulted() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, human) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "fields-human" }),
    )
    .await;
    assert_eq!(status, 200, "the human enrols: {human}");
    let human_id = human["principal_id"].as_str().unwrap().to_string();

    let submit = |body: Value| {
        let client = client.clone();
        let base = base.clone();
        let human_id = human_id.clone();
        async move { post(&client, &base, "/v1/resources", &human_id, &body).await }
    };

    // ── (1) `scheme` is the RESOLVER-SELECTION key, not the locator's ───────
    //
    // 🔴 This arm asserts a REFUTATION. The leaf was opened on a test helper's
    // comment — *"`resource_references.scheme` is caller-supplied and is NOT
    // validated against the locator"* — read as a defect, and the repair that
    // followed from it (refuse a scheme that is not `scheme_of(locator)`) was
    // written, shipped into a RED/GREEN cycle, and then refused by two of this
    // suite's own controls.
    //
    // Measured instead of inferred: `resolvers::resolve` selects on
    // `resolver_capabilities.schemes @> [$scheme]`, and two SHIPPED packs pair a
    // non-URI scheme with an `https://*` locator pattern — `r1-git-fetcher`
    // advertises `["git"]` (`migrations/0026`) and the R3 browser pack
    // advertises `["web+render"]`. A Git repository and a rendered page are both
    // reached over HTTPS; the field is how a caller asks for a CAPABILITY.
    //
    // So these two register, and a repair that validated the field against the
    // locator would refuse both:
    for (locator, scheme) in [
        ("https://127.0.0.1/repo.git#main", "git"),
        ("https://127.0.0.1/page", "web+render"),
    ] {
        let (status, capability) =
            submit(json!({ "original_locator": locator, "scheme": scheme })).await;
        assert_eq!(
            status, 200,
            "`{scheme}` selects a CAPABILITY for an https locator: {capability}"
        );
    }

    // ⚠️ And the field is validated against nothing else either, deliberately:
    // §3.7 says accepting a reference is not a promise the core can resolve it,
    // so an unknown scheme is `resource_unresolvable_now` at RESOLUTION rather
    // than a refusal at registration. This arm pins that, so a later reader does
    // not re-derive the repair this one refutes.
    let (status, unknown) = submit(json!({
        "original_locator": "ftp://example.org/archive.tar",
        "scheme": "no-pack-advertises-this",
    }))
    .await;
    assert_eq!(
        status, 200,
        "an unresolvable reference stays submitted (§3.7): {unknown}"
    );

    // ── (2) an omitted field takes the schema's DECLARED default ────────────
    //
    // `#[serde(default)]` was `String::default()` — the empty string — and
    // `submit` binds the field explicitly, so `migrations/0023`'s
    // `NOT NULL DEFAULT 'network'` / `'low'` never applied.
    let (status, defaulted) = submit(json!({
        "original_locator": "https://example.org/defaults",
        "scheme": "https",
    }))
    .await;
    assert_eq!(status, 200, "the reference registers: {defaulted}");
    let defaulted_id = defaulted["resource_id"].as_str().unwrap().to_string();
    let (scope, risk): (String, String) = sqlx::query_as(
        "SELECT visibility_scope, risk_class FROM resource_references WHERE resource_id = $1",
    )
    .bind(&defaulted_id)
    .fetch_one(&pool)
    .await
    .expect("read the stored defaults");
    assert_eq!(
        (scope.as_str(), risk.as_str()),
        ("network", "low"),
        "the omitted fields take the values `migrations/0023` declares, not `''`"
    );

    // And the two writers now agree. A contribution's citation has always
    // written the declared values; the route used to write `''`.
    let (status, created) = post(
        &client,
        &base,
        "/v1/threads",
        &human_id,
        &json!({
            "protocol_version": reasonbraid_core::PROTOCOL_VERSION,
            "operation": "thread.create",
            "request_id": reasonbraid_core::RequestId::new().to_string(),
            "idempotency_key": "fields-create",
            "body": {
                "tenant_id": human["tenant_id"].as_str().unwrap(),
                "subject": "field defaults",
                "objective": "compare the two reference writers",
            },
            "client_context": {},
        }),
    )
    .await;
    assert_eq!(status, 200, "the thread creates: {created}");
    let thread_id = created["thread_id"].as_str().unwrap().to_string();
    let cited = "https://example.org/cited-defaults";
    let (status, contributed) = post(
        &client,
        &base,
        &format!("/v1/threads/{thread_id}/commands"),
        &human_id,
        &json!({
            "protocol_version": reasonbraid_core::PROTOCOL_VERSION,
            "operation": "thread.contribute",
            "request_id": reasonbraid_core::RequestId::new().to_string(),
            "idempotency_key": "fields-contribute",
            "body": {
                "tenant_id": human["tenant_id"].as_str().unwrap(),
                "content": "the position this citation supports",
                "kind": "evidence_reference",
                "evidence_refs": [ { "uri": cited } ],
            },
            "client_context": {},
        }),
    )
    .await;
    assert_eq!(status, 200, "the citation contributes: {contributed}");
    let (cited_scope, cited_risk): (String, String) = sqlx::query_as(
        "SELECT visibility_scope, risk_class FROM resource_references \
         WHERE original_locator = $1",
    )
    .bind(cited)
    .fetch_one(&pool)
    .await
    .expect("read the citation path's row");
    assert_eq!(
        (cited_scope, cited_risk),
        (scope, risk),
        "the two writers produce the SAME row for the same omitted field"
    );

    // ── (3) the fragment stays in the locator, and that is DECIDED ──────────
    //
    // §12.1 lists `fragment_or_selector` beside `original_locator`, which
    // implies the locator excludes the fragment. ⛔ Splitting it IS
    // canonicalization, and §12.1 says canonicalization is scheme-specific and
    // "must not erase security-relevant distinctions" — merging three locators
    // onto one row is exactly such an erasure. So the conservative choice is
    // kept, and this arm pins the behaviour rather than a repair.
    let page = "https://example.org/page";
    let mut ids = Vec::new();
    for locator in [
        page,
        "https://example.org/page#section-a",
        "https://example.org/page#section-b",
    ] {
        let (status, body) =
            submit(json!({ "original_locator": locator, "scheme": "https" })).await;
        assert_eq!(
            status, 200,
            "the fragment-bearing reference registers: {body}"
        );
        ids.push(body["resource_id"].as_str().unwrap().to_string());
    }
    ids.sort();
    ids.dedup();
    assert_eq!(
        ids.len(),
        3,
        "a fragment is part of the locator: three spellings are three references, \
         because merging them would erase a distinction §12.1 protects"
    );

    eprintln!(
        "reference fields: `scheme` is the RESOLVER-SELECTION key — `git` and `web+render` register for https locators, and an unadvertised scheme stays submitted per §3.7 — so the locator check this leaf was opened on is REFUTED; an omitted visibility_scope/risk_class takes `network`/`low`, the values migrations/0023 declares, and the citation path writes the SAME row; and three fragment spellings remain three references, decided rather than defaulted"
    );
}

/// `SIGNOFF-REPAIR.11.14.3.7`: what a contribution's citation list costs, and
/// the one bound that can be taken without inventing a number.
///
/// `.11.14.3.2` made each citation register a §12.1 reference **inside the
/// thread's aggregate transaction**, which holds `FOR UPDATE` on the thread's
/// row — so one request became O(n) statements blocking every other command on
/// that thread. The leaf refused to invent a cap, and this control is why that
/// was right and what it leaves.
///
/// ⭐ **The de-duplication is DERIVED, not chosen.** A reference's identity is
/// the `(original_locator, expected_digest)` pair, so two citations naming the
/// same pair name ONE row and the second registration can only return what the
/// first just wrote. ⛔ It de-duplicates the WORK, never the RECORD: every
/// citation still rides the event, in order, with its own note.
///
/// ⚠️ **A windowed quota is the wrong instrument for this defect, and that is
/// measured rather than asserted.** The cost is inside ONE request, so a
/// per-hour call ceiling on `thread.contribute` bounds how many requests arrive
/// and not how long any one of them holds the row. What bounds a single request
/// today is its BODY, at `axum-core 0.5.6`'s `DEFAULT_LIMIT = 2_097_152`.
#[tokio::test]
async fn a_contribution_registers_each_cited_pair_once() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, human) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "citation-cost-human" }),
    )
    .await;
    assert_eq!(status, 200, "the human enrols: {human}");
    let human_id = human["principal_id"].as_str().unwrap().to_string();
    let tenant_id = human["tenant_id"].as_str().unwrap().to_string();

    let (status, created) = post(
        &client,
        &base,
        "/v1/threads",
        &human_id,
        &json!({
            "protocol_version": reasonbraid_core::PROTOCOL_VERSION,
            "operation": "thread.create",
            "request_id": reasonbraid_core::RequestId::new().to_string(),
            "idempotency_key": "cost-create",
            "body": {
                "tenant_id": tenant_id,
                "subject": "citation cost",
                "objective": "measure what a citation list registers",
            },
            "client_context": {},
        }),
    )
    .await;
    assert_eq!(status, 200, "the thread creates: {created}");
    let thread_id = created["thread_id"].as_str().unwrap().to_string();

    let contribute = |key: String, refs: Value| {
        let client = client.clone();
        let base = base.clone();
        let human_id = human_id.clone();
        let thread_id = thread_id.clone();
        let tenant_id = tenant_id.clone();
        async move {
            post(
                &client,
                &base,
                &format!("/v1/threads/{thread_id}/commands"),
                &human_id,
                &json!({
                    "protocol_version": reasonbraid_core::PROTOCOL_VERSION,
                    "operation": "thread.contribute",
                    "request_id": reasonbraid_core::RequestId::new().to_string(),
                    "idempotency_key": key,
                    "body": {
                        "tenant_id": tenant_id,
                        "content": "the position these citations support",
                        "kind": "evidence_reference",
                        "evidence_refs": refs,
                    },
                    "client_context": {},
                }),
            )
            .await
        }
    };
    let references = |locator_prefix: &'static str| {
        let pool = pool.clone();
        async move {
            sqlx::query_scalar::<_, i64>(
                "SELECT count(*) FROM resource_references WHERE original_locator LIKE $1",
            )
            .bind(format!("{locator_prefix}%"))
            .fetch_one(&pool)
            .await
            .expect("count the registered references")
        }
    };

    // ── The amplification, measured deterministically ───────────────────────
    //
    // Statement counts are not observable from here, but the ROWS are, and they
    // are one-for-one with the registrations: N distinct pairs is N trips to the
    // store inside the thread's locked transaction. A timing assertion would say
    // less and be flakier.
    const DISTINCT: usize = 64;
    let distinct: Vec<Value> = (0..DISTINCT)
        .map(|n| json!({ "uri": format!("https://example.org/distinct/{n}") }))
        .collect();
    let (status, many) = contribute("cost-distinct".into(), json!(distinct)).await;
    assert_eq!(status, 200, "the distinct citations contribute: {many}");
    assert_eq!(
        references("https://example.org/distinct/").await,
        DISTINCT as i64,
        "N distinct citations register N references — the O(n) the leaf measured"
    );

    // ── The de-duplication is NOT observable here, and that is recorded ─────
    //
    // ⚠️ The row count is **1 either way**: `resources::submit`'s pair replay
    // already returns the existing row, so a repeated citation writes nothing
    // new with or without the de-duplication. What it removes is the N−1 round
    // trips inside the locked transaction, and no product surface exposes those.
    //
    // ⛔ A `pg_stat_user_tables` scan-counter instrument was written for this
    // and DISCARDED: it reported the same value against the repaired and the
    // unrepaired handler, so its assertion could not fail. A control that passes
    // identically either way measures nothing, and shipping it would have been
    // worse than shipping none.
    //
    // The de-duplication is falsified DIRECTLY instead, as a unit test over the
    // pure function this handler calls —
    // `threads::tests::the_citation_pairs_are_de_duplicated_by_pair_not_by_locator`,
    // observed RED against a locator-keyed implementation. What THIS control
    // holds is everything the product does expose: the O(n) over distinct
    // citations above, the record below, and the pair semantics after it.
    const REPEATS: usize = 48;
    let repeated: Vec<Value> = (0..REPEATS)
        .map(|n| {
            json!({
                "uri": "https://example.org/repeated/report",
                "note": format!("the {n}th time this contributor cited it"),
            })
        })
        .collect();
    let (status, deduped) = contribute("cost-repeated".into(), json!(repeated)).await;
    assert_eq!(status, 200, "the repeated citations contribute: {deduped}");
    assert_eq!(
        references("https://example.org/repeated/").await,
        1,
        "one pair, one reference — true before this repair too, which is why the \
         row count cannot measure it"
    );

    // ⛔ And the RECORD is not de-duplicated. Every citation the contributor
    // wrote rides the event, in order, with its own note and the shared
    // `resource_id`.
    let (status, events) = get(
        &client,
        &base,
        &format!("/v1/threads/{thread_id}/events?tenant_id={tenant_id}"),
        &human_id,
    )
    .await;
    assert_eq!(status, 200, "the events read: {events}");
    // ⚠️ Selected by LOCATOR, not by length. An earlier version selected on
    // `len() == REPEATS` and matched the DISTINCT contribution's event instead,
    // because both carried 64 — a control defect of this session's own, and the
    // reason the two arms now differ in both locator and count.
    let refs = events["events"]
        .as_array()
        .expect("the event array")
        .iter()
        .filter_map(|event| event["body"]["evidence_refs"].as_array())
        .find(|refs| {
            refs.first().and_then(|r| r["uri"].as_str())
                == Some("https://example.org/repeated/report")
        })
        .unwrap_or_else(|| {
            panic!("the repeated contribution's event carries all {REPEATS}: {events}")
        });
    assert_eq!(
        refs.len(),
        REPEATS,
        "every citation the contributor wrote rides the event"
    );
    let resource_ids: std::collections::BTreeSet<&str> = refs
        .iter()
        .map(|r| r["resource_id"].as_str().expect("the resolved resource id"))
        .collect();
    assert_eq!(
        resource_ids.len(),
        1,
        "all {REPEATS} citations resolved to the SAME reference: {refs:?}"
    );
    let notes: std::collections::BTreeSet<&str> =
        refs.iter().filter_map(|r| r["note"].as_str()).collect();
    assert_eq!(
        notes.len(),
        REPEATS,
        "and each kept its own note — the work is shared, the record is not"
    );

    // ── The pair, not the locator: one locator at two digests is TWO ────────
    //
    // A de-duplication keyed on the locator alone would collapse these, and
    // §12.6 requires a changed page to stay a second reference.
    let digest_a = format!("sha256:{}", "a".repeat(64));
    let digest_b = format!("sha256:{}", "b".repeat(64));
    let (status, pairs) = contribute(
        "cost-pairs".into(),
        json!([
            { "uri": "https://example.org/pairs/page", "digest": digest_a },
            { "uri": "https://example.org/pairs/page", "digest": digest_b },
            { "uri": "https://example.org/pairs/page", "digest": digest_a },
        ]),
    )
    .await;
    assert_eq!(status, 200, "the pinned citations contribute: {pairs}");
    assert_eq!(
        references("https://example.org/pairs/").await,
        2,
        "one locator at two digests is TWO references, and the repeat of the \
         first is one — the de-duplication keys on the PAIR"
    );

    eprintln!(
        "citation cost: {DISTINCT} distinct citations register {DISTINCT} references (the O(n) inside the thread's locked transaction); the same pair cited {REPEATS} times registers ONE reference — which the row count cannot distinguish, so the de-duplication is falsified in a unit test instead — while all {REPEATS} still ride the event with their own notes; and one locator at two digests stays TWO references"
    );
}

/// `SIGNOFF-REPAIR.11.14.3.15`: a derivation is filed against a snapshot THIS
/// TENANT cited — the surface `.11.14.3.8`'s census did not see.
///
/// 🔴 That leaf published *"every surface that names a `snapshot_id`: seven, six
/// citation-bound, exactly one not"*. Re-derived on the IDENTIFIER rather than on
/// the route table, the population is **eight** and **two** were unbound:
/// `POST /v1/derivations` names its parent in the request **BODY**, so a census
/// built from `Path(snapshot_id)` extractors and `cited_snapshot` call sites
/// cannot see it.
///
/// ⭐ This is the FIRST instance of the blind spot
/// `docs/knowledge/a-census-is-as-wide-as-its-key.md` describes; that note was
/// written from the second (`.11.14.3.11`) and never applied backwards. The rule
/// was earned three times before it was used.
#[tokio::test]
async fn a_derivation_is_filed_against_a_snapshot_this_tenant_cited() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, owner) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "derivation-owner" }),
    )
    .await;
    assert_eq!(status, 200, "the acquiring tenant enrols: {owner}");
    let owner_id = owner["principal_id"].as_str().unwrap().to_string();

    let (status, stranger) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "derivation-stranger" }),
    )
    .await;
    assert_eq!(status, 200, "the second tenant enrols: {stranger}");
    let stranger_id = stranger["principal_id"].as_str().unwrap().to_string();
    assert_ne!(
        stranger["tenant_id"].as_str().unwrap(),
        owner["tenant_id"].as_str().unwrap()
    );

    // The owner acquires the evidence, which records its citation.
    let locator = "https://example.org/derivation-parent";
    let payload = b"the acquired report the derivation is taken from";
    let (status, reference) = post(
        &client,
        &base,
        "/v1/resources",
        &owner_id,
        &json!({ "original_locator": locator, "scheme": "https" }),
    )
    .await;
    assert_eq!(status, 200, "the reference registers: {reference}");
    let (status, snapshot) = post(
        &client,
        &base,
        "/v1/snapshots",
        &owner_id,
        &json!({
            "reference_id": reference["resource_id"].as_str().unwrap(),
            "original_locator": locator,
            "final_locator": locator,
            "resolver_id": "r0-https-fetcher",
            "resolver_version": "0.1.0",
            "raw_digest": reasonbraid_server::fetcher::digest_sha256_hex(payload),
            "byte_length": payload.len(),
            "media_type": "text/plain",
            "bytes_base64": util::base64(payload),
        }),
    )
    .await;
    assert_eq!(status, 200, "the snapshot submits: {snapshot}");
    let snapshot_id = snapshot["snapshot_id"].as_str().unwrap().to_string();

    let derive = |principal: String, parent: String, content: &'static str| {
        let client = client.clone();
        let base = base.clone();
        async move {
            post(
                &client,
                &base,
                "/v1/derivations",
                &principal,
                &json!({
                    "parent_snapshot_id": parent,
                    "derived_kind": "excerpt",
                    "derived_digest": reasonbraid_server::fetcher::digest_sha256_hex(
                        content.as_bytes()
                    ),
                    "content": content,
                }),
            )
            .await
        }
    };

    // ── The finding: a foreign parent answered differently, and SUCCEEDED ────
    let absent = "snp_00000000-0000-7000-8000-00000000dead";
    let (foreign_status, foreign) = derive(
        stranger_id.clone(),
        snapshot_id.clone(),
        "the stranger's excerpt",
    )
    .await;
    let (missing_status, missing) = derive(
        stranger_id.clone(),
        absent.to_string(),
        "the stranger's excerpt",
    )
    .await;
    assert_eq!(
        (foreign_status, missing_status),
        (400, 400),
        "a foreign parent and an absent one are refused alike: {foreign} / {missing}"
    );
    assert_eq!(
        (foreign["code"].clone(), foreign["message"].clone()),
        (missing["code"].clone(), missing["message"].clone()),
        "…in the SAME words, so the write surface cannot confirm that a `snp_` id \
         exists: {foreign} / {missing}"
    );
    let attached: i64 =
        sqlx::query_scalar("SELECT count(*) FROM derivations WHERE parent_snapshot_id = $1")
            .bind(&snapshot_id)
            .fetch_one(&pool)
            .await
            .expect("count the parent's derivations");
    assert_eq!(
        attached, 0,
        "the refused submissions attached nothing to another tenant's snapshot"
    );

    // ── The bound: the citing tenant still derives ──────────────────────────
    let (status, own) = derive(owner_id.clone(), snapshot_id.clone(), "the owner's excerpt").await;
    assert_eq!(status, 200, "the citing tenant still derives: {own}");

    // ── The supported path: acquire the same bytes, then derive ─────────────
    //
    // The replay records the second citation, exactly as it does for an
    // assessment — the arm that keeps this a binding rather than a blackout.
    let (status, replayed_ref) = post(
        &client,
        &base,
        "/v1/resources",
        &stranger_id,
        &json!({ "original_locator": locator, "scheme": "https" }),
    )
    .await;
    assert_eq!(
        status, 200,
        "the stranger's reference replays: {replayed_ref}"
    );
    let (status, replayed_snapshot) = post(
        &client,
        &base,
        "/v1/snapshots",
        &stranger_id,
        &json!({
            "reference_id": replayed_ref["resource_id"].as_str().unwrap(),
            "original_locator": locator,
            "final_locator": locator,
            "resolver_id": "r0-https-fetcher",
            "resolver_version": "0.1.0",
            "raw_digest": reasonbraid_server::fetcher::digest_sha256_hex(payload),
            "byte_length": payload.len(),
            "media_type": "text/plain",
            "bytes_base64": util::base64(payload),
        }),
    )
    .await;
    assert_eq!(
        status, 200,
        "the stranger's snapshot replays: {replayed_snapshot}"
    );
    assert_eq!(
        replayed_snapshot["snapshot_id"].as_str().unwrap(),
        snapshot_id,
        "one shared snapshot row"
    );
    let (status, now_allowed) = derive(
        stranger_id.clone(),
        snapshot_id.clone(),
        "the stranger's excerpt",
    )
    .await;
    assert_eq!(
        status, 200,
        "the second tenant derives from evidence it has now cited: {now_allowed}"
    );

    // ⚠️ And the derivation graph stays SHARED, which is the disposition
    // `.11.14.2` already took: a derivation is content-addressed in the way a
    // snapshot is, so both tenants read both children.
    let listed = |principal: String| {
        let client = client.clone();
        let base = base.clone();
        let snapshot_id = snapshot_id.clone();
        async move {
            let (status, body) = get(
                &client,
                &base,
                &format!("/v1/snapshots/{snapshot_id}/derivations"),
                &principal,
            )
            .await;
            assert_eq!(status, 200, "the derivations read: {body}");
            body.as_array().expect("the derivation array").len()
        }
    };
    assert_eq!(listed(owner_id).await, 2, "the owner reads both children");
    assert_eq!(
        listed(stranger_id).await,
        2,
        "and so does the second tenant, which now cites the parent — derivations \
         are content-addressed and stay shared (`.11.14.2`)"
    );

    eprintln!(
        "derivation write binding: a foreign parent and an absent one are ONE answer and attach nothing; the citing tenant derives normally; the second tenant acquires the same bytes, records its citation and then derives — and the derivation graph stays shared, both tenants reading both children"
    );
}

/// `SIGNOFF-REPAIR.11.14.3.14`: §16.11's two unwired quota scopes gain the
/// producer the section names — the acquisition path — and the fail-closed
/// contract survives a member space the server does not control.
///
/// `resolver` and `destination` shipped in `SCOPE_KINDS`, in migration 0047's
/// CHECK constraint, and in nothing else. Meanwhile
/// `POST /v1/resources/{id}/resolve` performed a real network acquisition per
/// call with no quota, no storm control and no breaker — which is exactly
/// §16.11's "resolver abuse" and "scraping".
///
/// ⭐ **The load-bearing finding is why they sat unwired.** `check_in_tx` is
/// fail-closed, and that works for the two scopes already bound because their
/// members are created by a path the server controls and can seed. Neither of
/// these is: the resolver space grows through `POST /v1/resolvers`, and the
/// destination space is the open internet. So each tenant gets a DEFAULT row at
/// the wildcard id, a specific row overrides it, and the absence of BOTH is the
/// same typed refusal as before.
#[tokio::test]
async fn the_acquisition_path_is_bounded_per_resolver_and_per_destination() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, human) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "acquisition-quota-human" }),
    )
    .await;
    assert_eq!(status, 200, "the human enrols: {human}");
    let human_id = human["principal_id"].as_str().unwrap().to_string();
    let tenant_id = human["tenant_id"].as_str().unwrap().to_string();

    // ── The tenant is seeded with a DEFAULT row for each open scope ─────────
    let defaults: Vec<(String, String, i64)> = sqlx::query_as(
        "SELECT scope_kind, scope_id, ceiling FROM usage_quotas \
         WHERE tenant_id = $1 AND scope_kind IN ('resolver', 'destination') \
         ORDER BY scope_kind",
    )
    .bind(&tenant_id)
    .fetch_all(&pool)
    .await
    .expect("read the seeded acquisition quotas");
    assert_eq!(
        defaults,
        vec![
            ("destination".to_owned(), "*".to_owned(), 1000),
            ("resolver".to_owned(), "*".to_owned(), 1000),
        ],
        "the enrol transaction seeds one DEFAULT row per open scope, not one per member"
    );

    let (status, reference) = post(
        &client,
        &base,
        "/v1/resources",
        &human_id,
        // ⚠️ A loopback locator on purpose: the R0 pack still RANKS on `https`,
        // so the quota is checked, and the shipped destination policy then
        // refuses the class before any socket opens. The control therefore
        // touches no network and also shows the ordering — the bound is
        // consumed by the ATTEMPT, ahead of the policy that refuses it.
        &json!({ "original_locator": "https://127.0.0.1/quota-probe", "scheme": "https" }),
    )
    .await;
    assert_eq!(status, 200, "the reference registers: {reference}");
    let resource_id = reference["resource_id"].as_str().unwrap().to_string();

    let resolve = || {
        let client = client.clone();
        let base = base.clone();
        let human_id = human_id.clone();
        let resource_id = resource_id.clone();
        async move {
            post(
                &client,
                &base,
                &format!("/v1/resources/{resource_id}/resolve"),
                &human_id,
                &json!({ "required_sandbox": "none", "required_egress": "listed" }),
            )
            .await
        }
    };

    // ── Under the ceiling: the resolution answers, and records its use ──────
    let (status, first) = resolve().await;
    assert_eq!(status, 200, "the first resolution answers: {first}");
    let uses: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM quota_events e JOIN usage_quotas q USING (quota_id) \
         WHERE q.tenant_id = $1 AND q.scope_kind IN ('resolver', 'destination') \
           AND e.kind = 'use'",
    )
    .bind(&tenant_id)
    .fetch_one(&pool)
    .await
    .expect("count the recorded uses");
    assert_eq!(
        uses, 2,
        "one resolution records one use per scope — the resolver and the destination"
    );

    // ── A SPECIFIC row overrides the default, and the bound then bites ──────
    //
    // ⚠️ A specific row starts its OWN count — `quota_events` is keyed by
    // `quota_id` — so the use recorded against the default above does not carry
    // over. That is the correct semantic (a narrowed bound is a new bound, not a
    // continuation of the old one) and it is why the ceiling here is `0` rather
    // than the count already spent: the first attempt under it is at the
    // ceiling. This is also how an operator narrows one noisy host without
    // touching the rest.
    sqlx::query(
        "INSERT INTO usage_quotas (quota_id, tenant_id, scope_kind, scope_id, ceiling, window_seconds) \
         VALUES ($1, $2, 'destination', '127.0.0.1', 0, 3600)",
    )
    .bind(format!("quo_{tenant_id}_loopback"))
    .bind(&tenant_id)
    .execute(&pool)
    .await
    .expect("declare a host-specific bound");

    let (status, refused) = resolve().await;
    assert_eq!(
        status, 429,
        "the host-specific ceiling refuses the next acquisition: {refused}"
    );
    assert_eq!(refused["code"], json!("quota_exceeded"), "{refused}");
    let denials: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM quota_events e JOIN usage_quotas q USING (quota_id) \
         WHERE q.tenant_id = $1 AND q.scope_id = '127.0.0.1' AND e.kind = 'denial'",
    )
    .bind(&tenant_id)
    .fetch_one(&pool)
    .await
    .expect("count the denials");
    assert_eq!(
        denials, 1,
        "the refusal is a RECORDED fact, never silent — the `0047` contract"
    );

    // ── The fail-closed contract survives: remove BOTH rows, get the refusal ─
    sqlx::query(
        "DELETE FROM quota_events WHERE quota_id IN \
         (SELECT quota_id FROM usage_quotas WHERE tenant_id = $1 AND scope_kind = 'destination')",
    )
    .bind(&tenant_id)
    .execute(&pool)
    .await
    .expect("clear the destination events");
    sqlx::query("DELETE FROM usage_quotas WHERE tenant_id = $1 AND scope_kind = 'destination'")
        .bind(&tenant_id)
        .execute(&pool)
        .await
        .expect("remove both destination rows");

    let (status, unconfigured) = resolve().await;
    assert_eq!(
        status, 503,
        "no specific row AND no default row is still the typed refusal: {unconfigured}"
    );
    assert_eq!(
        unconfigured["code"],
        json!("quota_unconfigured"),
        "the fail-closed contract is unchanged — what moved is that the default \
         is a ROW rather than an absence: {unconfigured}"
    );

    eprintln!(
        "acquisition quota: the enrol transaction seeds one DEFAULT row per open scope; one resolution records one use per scope (resolver + destination); a host-specific ceiling overrides the default and refuses with a RECORDED denial (429); and removing both rows is still the typed fail-closed 503"
    );
}

/// `SIGNOFF-REPAIR.7.1.2.1` — the workflow registry is site-wide configuration,
/// and before the repair any enrolled principal could rewrite it.
///
/// ⛔ **The NEGATIVE arm observes the takeover END TO END, not the write.** A
/// control that only asserted "tenant B's INSERT succeeded" would prove nothing:
/// the registry is versioned on purpose and a new row is not a defect. What
/// makes it one is that `workflows::resolve` takes the HIGHEST version of a
/// `profile_id` site-wide, with no `built_in` filter and no tenant predicate, so
/// tenant A's next BARE thread — which defaults to `quick_advice` — executes the
/// steps tenant B chose. That is what this asserts.
///
/// ⚠️ The vocabulary is closed (`STEP_KINDS`), so the override cannot introduce
/// a verb. `["solicit", "decide"]` is a legal profile whose point is what it
/// REMOVES: `synthesize` is gone from every tenant's default deliberation.
#[tokio::test]
async fn a_foreign_tenant_cannot_rewrite_the_default_workflow() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    // Two enrolments, two tenants: `human_principals.principal_id` carries one
    // `tenant_id` each, so a second enrolment is a second tenant by construction.
    let (status, alice) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "wfo-alice" }),
    )
    .await;
    assert_eq!(status, 200, "alice enrols: {alice}");
    let alice_id = alice["principal_id"].as_str().unwrap().to_string();
    let alice_tenant = alice["tenant_id"].as_str().unwrap().to_string();

    let (status, mallory) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "wfo-mallory" }),
    )
    .await;
    assert_eq!(status, 200, "mallory enrols: {mallory}");
    let mallory_id = mallory["principal_id"].as_str().unwrap().to_string();
    let mallory_tenant = mallory["tenant_id"].as_str().unwrap().to_string();
    assert_ne!(
        alice_tenant, mallory_tenant,
        "the two enrolments must be two tenants, or this control measures nothing"
    );

    // Mallory holds tenant authority in her OWN tenant and no site grant at all.
    let (status, refused) = post(
        &client,
        &base,
        "/v1/workflow-profiles",
        &mallory_id,
        &json!({
            "profile_id": "quick_advice",
            "steps": ["solicit", "decide"],
            "reason": "shorten the default deliberation",
        }),
    )
    .await;
    assert_eq!(
        status, 403,
        "an enrolled principal without the site capability registers no profile: {refused}"
    );
    assert_eq!(
        refused["code"],
        json!("site_authority_required"),
        "{refused}"
    );

    // The default is unchanged — the `.1` half of the acceptance, asserted from
    // the RESOLUTION rather than from the table, because the resolution is what
    // a thread actually runs.
    let (status, created) = post(
        &client,
        &base,
        "/v1/threads",
        &alice_id,
        &json!({
            "protocol_version": reasonbraid_core::PROTOCOL_VERSION,
            "operation": "thread.create",
            "request_id": reasonbraid_core::RequestId::new().to_string(),
            "idempotency_key": "wfo-create",
            "body": {
                "tenant_id": alice_tenant,
                "subject": "wfo",
                "objective": "a bare thread takes the default profile",
            },
            "client_context": {},
        }),
    )
    .await;
    assert_eq!(status, 200, "alice's bare thread is created: {created}");
    let thread_id = created["thread_id"].as_str().unwrap().to_string();

    let (status, thread) = get(
        &client,
        &base,
        &format!("/v1/threads/{thread_id}?tenant_id={alice_tenant}"),
        &alice_id,
    )
    .await;
    assert_eq!(status, 200, "alice reads her thread: {thread}");
    assert_eq!(
        thread["state"]["workflow_profile"],
        json!("quick_advice"),
        "{thread}"
    );
    assert_eq!(
        thread["state"]["workflow_steps"],
        json!(["solicit", "synthesize", "decide"]),
        "the shipped built-in steps survive a foreign registration attempt: {thread}"
    );
}

/// The POSITIVE arm, without which the repair above is indistinguishable from
/// deleting the route: an operator-issued `workflow_register` grant still
/// registers a profile, the receipt is audited, and a thread naming it runs it.
#[tokio::test]
async fn the_workflow_register_capability_still_registers_and_resolves() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, human) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "wfc-operator" }),
    )
    .await;
    assert_eq!(status, 200, "the operator enrols: {human}");
    let human_id = human["principal_id"].as_str().unwrap().to_string();
    let tenant_id = human["tenant_id"].as_str().unwrap().to_string();

    site_fixture::provision(
        &pool,
        &human_id,
        &[reasonbraid_server::site_authority::Action::WorkflowRegister],
    )
    .await;

    let response = client
        .post(format!("{base}/v1/workflow-profiles"))
        .header(PRINCIPAL_HEADER, &human_id)
        .json(&json!({
            "profile_id": "wfc_reviewed",
            "steps": ["solicit", "critique", "decide"],
            "reason": "the review lane needs a critique step",
        }))
        .send()
        .await
        .expect("register request");
    assert_eq!(
        response.status().as_u16(),
        200,
        "the capable operator registers"
    );
    assert!(
        response.headers().contains_key("x-reasonbraid-site-audit"),
        "a site act carries its audit id"
    );
    let registered: Value = response.json().await.unwrap();
    assert_eq!(
        registered["profile_id"],
        json!("wfc_reviewed"),
        "{registered}"
    );
    assert_eq!(registered["version"], json!(1), "{registered}");

    // The registration is REACHABLE, not merely stored.
    let (status, created) = post(
        &client,
        &base,
        "/v1/threads",
        &human_id,
        &json!({
            "protocol_version": reasonbraid_core::PROTOCOL_VERSION,
            "operation": "thread.create",
            "request_id": reasonbraid_core::RequestId::new().to_string(),
            "idempotency_key": "wfc-create",
            "body": {
                "tenant_id": tenant_id,
                "subject": "wfc",
                "objective": "the registered profile runs",
                "workflow_profile": "wfc_reviewed",
            },
            "client_context": {},
        }),
    )
    .await;
    assert_eq!(status, 200, "the thread names the new profile: {created}");
    let thread_id = created["thread_id"].as_str().unwrap().to_string();
    let (_, thread) = get(
        &client,
        &base,
        &format!("/v1/threads/{thread_id}?tenant_id={tenant_id}"),
        &human_id,
    )
    .await;
    assert_eq!(
        thread["state"]["workflow_steps"],
        json!(["solicit", "critique", "decide"]),
        "{thread}"
    );

    // ⛔ The capability is for THIS action only: the same grant does not become
    // site-wide authority. A registry inspection with a workflow-only grant is
    // refused, so the repair widened one verb rather than the boundary.
    let (status, refused) = get(&client, &base, "/v1/admin/adapters", &human_id).await;
    assert_eq!(
        status, 403,
        "a workflow_register grant inspects no registry: {refused}"
    );

    sqlx::query("DELETE FROM workflow_profiles WHERE profile_id = 'wfc_reviewed' AND NOT built_in")
        .execute(&pool)
        .await
        .expect("drop the fixture profile");
}
