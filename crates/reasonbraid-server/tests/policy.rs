//! The typed policy schema + the versioned registry (`PHASE-6.1.2`, ADR-019):
//! the policy is a versioned digest-pinned DOCUMENT — the stable clause ids,
//! the applicability + the non-applicability, the exception schema, and the
//! OWNERSHIP metadata (the owning authority is a GRANT reference — an
//! unresolvable owning authority is invalid at registration; the label
//! grants nothing).
//!
//! Run with `scripts/run_pg_tests.sh` locally or the `pg-tests` CI job.
//! Without `DATABASE_URL` these skip, so `make check` stays green offline.

#[path = "support/mod.rs"]
mod pg_test_support;

#[path = "support/cleanup.rs"]
mod pg_cleanup;

use std::net::SocketAddr;
use std::sync::OnceLock;

use reasonbraid_server::{
    api_router, api_router_with_publication_root, ca::ensure_server_ca, node_router,
    PRINCIPAL_HEADER,
};
use serde_json::{json, Value};
use sqlx::PgPool;

static POLICY_LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();

async fn guard() -> tokio::sync::MutexGuard<'static, ()> {
    POLICY_LOCK
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
            "routing_recommendations",
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
            "reference_registrations",
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
        Self::start_with_router(pool, api_router(pool.clone())).await
    }

    /// The `.9.2.1.1` seam: a server whose publication repository root is
    /// DECLARED. `start` declares none, which is what the publish verb now
    /// refuses — so a control that means to exercise the verb has to say
    /// where publications live.
    async fn start_with_publication_root(pool: &PgPool, root: &std::path::Path) -> Self {
        Self::start_with_router(
            pool,
            api_router_with_publication_root(pool.clone(), Some(root.to_path_buf())),
        )
        .await
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

const DIGEST: &str = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

/// `SIGNOFF-REPAIR.9.2.1.3`: a publication repository under a configured root,
/// and object ids READ BACK from it.
///
/// Three fixtures used to declare `["abc123", "def456"]`, which resolve to
/// nothing — the suite meant to qualify the effective transition was proving it
/// with ids that do not exist. ⛔ They are RE-SEEDED rather than relaxed or
/// deleted: each still asserts the same transition, and now asserts it
/// honestly. Returns the root to configure and the ids to declare.
fn seeded_publication_repository(name: &str) -> (std::path::PathBuf, Vec<String>) {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/policy-publish-tests")
        .join(name);
    let _ = std::fs::remove_dir_all(&root);
    let repo_dir = root.join("live");
    std::fs::create_dir_all(&repo_dir).expect("the repository dir creates");
    let repo = gix::init_bare(&repo_dir).expect("the bare repository inits");
    let ids = [b"first".as_slice(), b"second".as_slice()]
        .iter()
        .map(|data| {
            repo.write_object(gix::objs::BlobRef { data })
                .expect("the blob writes")
                .to_string()
        })
        .collect();
    (root, ids)
}

#[tokio::test]
async fn the_policy_registry_validates_the_digest_pinned_document() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, human) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "pol-human" }),
    )
    .await;
    assert_eq!(status, 200, "the human enrolls: {human}");
    let human_id = human["principal_id"].as_str().unwrap().to_string();
    // The dev enrollment's grant: `grt_<principal>` (the Phase-2 model) — the
    // policy's owning authority references it.
    let grant_id = format!("grt_{human_id}");

    let policy = |policy_id: &str, version: &str| {
        json!({
            "policy_id": policy_id,
            "version": version,
            "digest": DIGEST,
            "lifecycle": "draft",
            "title": "the baseline",
            "intent": "the org baseline policy",
            "domain": "deliberation",
            "risk_class": "low",
            "owning_authority": grant_id,
            "clauses": [
                { "id": "c1", "statement": "every thread declares its objective" },
                { "id": "c2", "statement": "every publication names its authority" },
            ],
            "applicability": [ { "layer": "organization", "target": "*" } ],
            "exceptions": [],
        })
    };

    // 1. The typed document registers; the row echoes the fields.
    let (status, registered) = post(
        &client,
        &base,
        "/v1/policies",
        &human_id,
        &policy("org-baseline", "1.0.0"),
    )
    .await;
    assert_eq!(status, 200, "the policy registers: {registered}");
    assert_eq!(registered["policy_id"], json!("org-baseline"));
    assert_eq!(registered["version"], json!("1.0.0"));
    assert_eq!(registered["digest"], json!(DIGEST));
    assert_eq!(registered["owning_authority"], json!(grant_id));
    assert_eq!(registered["clauses"].as_array().unwrap().len(), 2);

    // 2. A NEW version of the same policy registers (the versioned registry).
    let (status, upgraded) = post(
        &client,
        &base,
        "/v1/policies",
        &human_id,
        &policy("org-baseline", "1.1.0"),
    )
    .await;
    assert_eq!(status, 200, "the new version registers: {upgraded}");

    // 3. The refusals: the duplicate version, the malformed digest, the
    // malformed semver, the unknown lifecycle, the duplicate clause ids, the
    // empty clauses, and the GHOST owning authority (the label grants
    // nothing).
    let (status, refused) = post(
        &client,
        &base,
        "/v1/policies",
        &human_id,
        &policy("org-baseline", "1.0.0"),
    )
    .await;
    assert_eq!(status, 400, "the duplicate refuses: {refused}");
    assert!(
        refused["message"]
            .as_str()
            .unwrap()
            .contains("already exists"),
        "{refused}"
    );

    let (status, refused) = post(
        &client,
        &base,
        "/v1/policies",
        &human_id,
        &json!({
            "policy_id": "bad-digest",
            "version": "1.0.0",
            "digest": "not-a-digest",
            "lifecycle": "draft",
            "title": "x",
            "owning_authority": grant_id,
            "clauses": [ { "id": "c1", "statement": "s" } ],
        }),
    )
    .await;
    assert_eq!(status, 400, "the bad digest refuses: {refused}");
    assert!(
        refused["message"].as_str().unwrap().contains("sha256:"),
        "{refused}"
    );

    let (status, refused) = post(
        &client,
        &base,
        "/v1/policies",
        &human_id,
        &json!({
            "policy_id": "bad-version",
            "version": "not.a.version",
            "digest": DIGEST,
            "lifecycle": "draft",
            "title": "x",
            "owning_authority": grant_id,
            "clauses": [ { "id": "c1", "statement": "s" } ],
        }),
    )
    .await;
    assert_eq!(status, 400, "the bad semver refuses: {refused}");
    assert!(
        refused["message"]
            .as_str()
            .unwrap()
            .contains("semantic version"),
        "{refused}"
    );

    let (status, refused) = post(
        &client,
        &base,
        "/v1/policies",
        &human_id,
        &json!({
            "policy_id": "bad-lifecycle",
            "version": "1.0.0",
            "digest": DIGEST,
            "lifecycle": "vibes",
            "title": "x",
            "owning_authority": grant_id,
            "clauses": [ { "id": "c1", "statement": "s" } ],
        }),
    )
    .await;
    assert_eq!(status, 400, "the unknown lifecycle refuses: {refused}");
    assert!(
        refused["message"].as_str().unwrap().contains("vocabulary"),
        "{refused}"
    );

    let (status, refused) = post(
        &client,
        &base,
        "/v1/policies",
        &human_id,
        &json!({
            "policy_id": "dup-clauses",
            "version": "1.0.0",
            "digest": DIGEST,
            "lifecycle": "draft",
            "title": "x",
            "owning_authority": grant_id,
            "clauses": [
                { "id": "c1", "statement": "s" },
                { "id": "c1", "statement": "t" },
            ],
        }),
    )
    .await;
    assert_eq!(status, 400, "the duplicate clause ids refuse: {refused}");
    assert!(
        refused["message"].as_str().unwrap().contains("repeats"),
        "{refused}"
    );

    let (status, refused) = post(
        &client,
        &base,
        "/v1/policies",
        &human_id,
        &json!({
            "policy_id": "no-clauses",
            "version": "1.0.0",
            "digest": DIGEST,
            "lifecycle": "draft",
            "title": "x",
            "owning_authority": grant_id,
            "clauses": [],
        }),
    )
    .await;
    assert_eq!(status, 400, "the empty clauses refuse: {refused}");

    let (status, refused) = post(
        &client,
        &base,
        "/v1/policies",
        &human_id,
        &json!({
            "policy_id": "ghost-authority",
            "version": "1.0.0",
            "digest": DIGEST,
            "lifecycle": "draft",
            "title": "x",
            "owning_authority": "grt_ghost",
            "clauses": [ { "id": "c1", "statement": "s" } ],
        }),
    )
    .await;
    assert_eq!(status, 400, "the ghost authority refuses: {refused}");
    assert!(
        refused["message"]
            .as_str()
            .unwrap()
            .contains("label grants nothing"),
        "{refused}"
    );

    // 4. The list: the two registered rows, newest first (the refusals
    // persisted nothing).
    let (status, policies) = get(&client, &base, "/v1/policies", &human_id).await;
    assert_eq!(status, 200, "the policies list: {policies}");
    let policies = policies.as_array().unwrap();
    assert_eq!(policies.len(), 2, "{policies:?}");
    assert_eq!(policies[0]["version"], json!("1.1.0"), "newest first");

    // 5. An unenrolled principal registers nothing.
    let (status, _) = post(
        &client,
        &base,
        "/v1/policies",
        "hpr_ghost",
        &policy("ghost", "1.0.0"),
    )
    .await;
    assert_eq!(status, 401, "the unenrolled register refuses");
}

#[tokio::test]
async fn the_seven_step_resolution_fails_closed() {
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
    let grant_id = format!("grt_{human_id}");

    let register = |policy: Value| {
        let client = client.clone();
        let base = base.clone();
        let human_id = human_id.clone();
        async move { post(&client, &base, "/v1/policies", &human_id, &policy).await }
    };
    let resolve = |request: Value| {
        let client = client.clone();
        let base = base.clone();
        let human_id = human_id.clone();
        async move { post(&client, &base, "/v1/policies/resolve", &human_id, &request).await }
    };
    let base_policy = |policy_id: &str, version: &str, clauses: Value, extra: Value| {
        json!({
            "policy_id": policy_id,
            "version": version,
            "digest": DIGEST,
            "lifecycle": "draft",
            "title": policy_id,
            "owning_authority": grant_id,
            "clauses": clauses,
            "applicability": extra.get("applicability").cloned().unwrap_or(json!([])),
            "dependencies": extra.get("dependencies").cloned().unwrap_or(json!([])),
            "conflicts": extra.get("conflicts").cloned().unwrap_or(json!([])),
            "precedence_hints": extra.get("precedence_hints").cloned().unwrap_or(json!([])),
            "exceptions": extra.get("exceptions").cloned().unwrap_or(json!([])),
        })
    };

    // The org baseline (the c1 + c2 clauses; one allowed waiver) + the
    // project policy (the c1 override via the precedence hint + the
    // dependency on the baseline).
    let (status, _) = register(base_policy(
        "org-baseline",
        "1.0.0",
        json!([
            { "id": "c1", "statement": "every thread declares its objective" },
            { "id": "c2", "statement": "every publication names its authority" },
        ]),
        json!({ "exceptions": [ { "waiver": "wv-1" } ] }),
    ))
    .await;
    assert_eq!(status, 200, "the baseline registers");
    let (status, _) = register(base_policy(
        "project-x",
        "1.0.0",
        json!([
            { "id": "c1", "statement": "the project requires the evidence gate" },
        ]),
        json!({
            "applicability": [ { "layer": "project", "target": "prj-x" } ],
            "precedence_hints": [ { "over": "org-baseline" } ],
            "dependencies": [ { "policy": "org-baseline", "version": "1.0.0" } ],
        }),
    ))
    .await;
    assert_eq!(status, 200, "the project registers");

    // 1. The happy resolution: both policies apply; the c1 collision settles
    // by the precedence (project-x over the baseline); c2 rides the
    // baseline; the explanation tree carries the seven steps.
    let (status, resolved) = resolve(json!({
        "policies": [
            { "policy_id": "org-baseline", "version": "1.0.0" },
            { "policy_id": "project-x", "version": "1.0.0" },
        ],
        "target": { "layer": "project", "target": "prj-x" },
    }))
    .await;
    assert_eq!(status, 200, "the resolution: {resolved}");
    let clauses = resolved["resolved"].as_array().unwrap();
    assert_eq!(clauses.len(), 2, "{resolved}");
    let c1 = clauses
        .iter()
        .find(|c| c["clause_id"] == json!("c1"))
        .unwrap();
    assert_eq!(c1["policy_id"], json!("project-x"), "the precedence won");
    assert_eq!(
        c1["statement"],
        json!("the project requires the evidence gate")
    );
    let c2 = clauses
        .iter()
        .find(|c| c["clause_id"] == json!("c2"))
        .unwrap();
    assert_eq!(c2["policy_id"], json!("org-baseline"));
    assert_eq!(resolved["explanation"].as_array().unwrap().len(), 7);
    assert_eq!(resolved["conflicts"].as_array().unwrap().len(), 0);

    // 2. The FAIL-CLOSED: the same clause id without the precedence is the
    // typed refusal (never a silent pick).
    let (status, _) = register(base_policy(
        "project-y",
        "1.0.0",
        json!([
            { "id": "c1", "statement": "a rival objective clause" },
        ]),
        json!({
            "applicability": [ { "layer": "project", "target": "prj-y" } ],
        }),
    ))
    .await;
    assert_eq!(status, 200, "the rival registers");
    let (status, refused) = resolve(json!({
        "policies": [
            { "policy_id": "org-baseline", "version": "1.0.0" },
            { "policy_id": "project-y", "version": "1.0.0" },
        ],
        "target": { "layer": "project", "target": "prj-y" },
    }))
    .await;
    assert_eq!(status, 400, "the binding conflict refuses: {refused}");
    assert!(
        refused["message"]
            .as_str()
            .unwrap()
            .contains("binding conflict"),
        "{refused}"
    );

    // 3. The missing dependency refuses.
    let (status, _) = register(base_policy(
        "project-z",
        "1.0.0",
        json!([ { "id": "cz", "statement": "z" } ]),
        json!({
            "dependencies": [ { "policy": "missing-policy", "version": "1.0.0" } ],
        }),
    ))
    .await;
    assert_eq!(status, 200, "the dependent registers");
    let (status, refused) = resolve(json!({
        "policies": [
            { "policy_id": "org-baseline", "version": "1.0.0" },
            { "policy_id": "project-z", "version": "1.0.0" },
        ],
        "target": { "layer": "project", "target": "prj-z" },
    }))
    .await;
    assert_eq!(status, 400, "the missing dependency refuses: {refused}");
    assert!(
        refused["message"].as_str().unwrap().contains("depends on"),
        "{refused}"
    );

    // 4. The requested exception must ride a policy's exception schema.
    let (status, refused) = resolve(json!({
        "policies": [ { "policy_id": "org-baseline", "version": "1.0.0" } ],
        "target": { "layer": "organization", "target": "*" },
        "exception_grants": ["wv-9"],
    }))
    .await;
    assert_eq!(status, 400, "the unknown waiver refuses: {refused}");
    assert!(
        refused["message"]
            .as_str()
            .unwrap()
            .contains("exception schema"),
        "{refused}"
    );
    let (status, resolved) = resolve(json!({
        "policies": [ { "policy_id": "org-baseline", "version": "1.0.0" } ],
        "target": { "layer": "organization", "target": "*" },
        "exception_grants": ["wv-1"],
    }))
    .await;
    assert_eq!(status, 200, "the allowed waiver resolves: {resolved}");

    // 5. A ghost policy reference refuses.
    let (status, refused) = resolve(json!({
        "policies": [ { "policy_id": "ghost", "version": "1.0.0" } ],
        "target": { "layer": "organization", "target": "*" },
    }))
    .await;
    assert_eq!(status, 400, "the ghost reference refuses: {refused}");

    // 6. A SUSPENDED policy is excluded by the applicability step (no
    // conflict, no clause).
    let (status, _) = register(json!({
        "policy_id": "suspended-p",
        "version": "1.0.0",
        "digest": DIGEST,
        "lifecycle": "suspended",
        "title": "suspended-p",
        "owning_authority": grant_id,
        "clauses": [ { "id": "c1", "statement": "a suspended clause" } ],
        "applicability": [ { "layer": "organization", "target": "*" } ],
    }))
    .await;
    assert_eq!(status, 200, "the suspended registers");
    let (status, resolved) = resolve(json!({
        "policies": [
            { "policy_id": "org-baseline", "version": "1.0.0" },
            { "policy_id": "suspended-p", "version": "1.0.0" },
        ],
        "target": { "layer": "organization", "target": "*" },
    }))
    .await;
    assert_eq!(status, 200, "the suspended set resolves: {resolved}");
    let clauses = resolved["resolved"].as_array().unwrap();
    assert_eq!(
        clauses.len(),
        2,
        "the suspended policy contributes nothing: {resolved}"
    );

    // 7. The precedence CYCLE refuses (the DAG check).
    let (status, _) = register(base_policy(
        "cycle-a",
        "1.0.0",
        json!([ { "id": "ca", "statement": "a" } ]),
        json!({ "precedence_hints": [ { "over": "cycle-b" } ] }),
    ))
    .await;
    assert_eq!(status, 200, "cycle-a registers");
    let (status, _) = register(base_policy(
        "cycle-b",
        "1.0.0",
        json!([ { "id": "cb", "statement": "b" } ]),
        json!({ "precedence_hints": [ { "over": "cycle-a" } ] }),
    ))
    .await;
    assert_eq!(status, 200, "cycle-b registers");
    let (status, refused) = resolve(json!({
        "policies": [
            { "policy_id": "cycle-a", "version": "1.0.0" },
            { "policy_id": "cycle-b", "version": "1.0.0" },
        ],
        "target": { "layer": "organization", "target": "*" },
    }))
    .await;
    assert_eq!(status, 400, "the precedence cycle refuses: {refused}");
    assert!(
        refused["message"].as_str().unwrap().contains("conflict"),
        "{refused}"
    );

    // 8. The impact map: the clauses × the declared coverage.
    let (status, impact) = get(
        &client,
        &base,
        "/v1/policies/org-baseline/1.0.0/impact",
        &human_id,
    )
    .await;
    assert_eq!(status, 200, "the impact map reads: {impact}");
    let map = impact.as_array().unwrap();
    assert_eq!(map.len(), 2, "{impact}");
    assert_eq!(map[0]["clause_id"], json!("c1"));
    let (status, refused) = get(&client, &base, "/v1/policies/ghost/1.0.0/impact", &human_id).await;
    assert_eq!(status, 400, "the ghost impact refuses: {refused}");
}

#[tokio::test]
async fn the_proposal_and_the_decision_stay_separate_records() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, human) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "lc-human" }),
    )
    .await;
    assert_eq!(status, 200, "the human enrolls: {human}");
    let human_id = human["principal_id"].as_str().unwrap().to_string();
    let tenant_id = human["tenant_id"].as_str().unwrap().to_string();
    let grant_id = format!("grt_{human_id}");

    // The policy the proposal targets.
    let (status, _) = post(
        &client,
        &base,
        "/v1/policies",
        &human_id,
        &json!({
            "policy_id": "lc-policy",
            "version": "1.0.0",
            "digest": DIGEST,
            "lifecycle": "draft",
            "title": "lc",
            "owning_authority": grant_id,
            "clauses": [ { "id": "c1", "statement": "the lifecycle clause" } ],
        }),
    )
    .await;
    assert_eq!(status, 200, "the policy registers");

    // The deliberation thread: independent_panel → advance → the verdict
    // contribution (the decision references it).
    let (status, created) = post(
        &client,
        &base,
        "/v1/threads",
        &human_id,
        &json!({
            "protocol_version": reasonbraid_core::PROTOCOL_VERSION,
            "operation": "thread.create",
            "request_id": reasonbraid_core::RequestId::new().to_string(),
            "idempotency_key": "lc-create",
            "body": {
                "tenant_id": tenant_id,
                "subject": "lc",
                "objective": "probe",
                "workflow_profile": "independent_panel",
            },
            "client_context": {},
        }),
    )
    .await;
    assert_eq!(status, 200, "the thread creates: {created}");
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
    let (status, advanced) = command(
        "lc-advance",
        "thread.advance_round",
        json!({ "tenant_id": tenant_id }),
    )
    .await;
    assert_eq!(status, 200, "the round advances: {advanced}");
    let (status, verdict) = command(
        "lc-verdict",
        "thread.contribute",
        json!({
            "tenant_id": tenant_id,
            "content": "the panel judged",
            "kind": "verdict",
            "verdict": {
                "target_digest": "sha256:00",
                "rule": "unanimity",
                "outcome": "accepted_unanimously",
            },
        }),
    )
    .await;
    assert_eq!(status, 200, "the verdict contributes: {verdict}");
    let verdict_event = verdict["event_id"].as_str().unwrap().to_string();

    // 1. The proposal registers (the draft stage; a REFERENCE).
    let (status, proposal) = post(
        &client,
        &base,
        "/v1/policy-proposals",
        &human_id,
        &json!({
            "proposal_id": "lc-prop-1",
            "policy_id": "lc-policy",
            "policy_version": "1.0.0",
            "thread_id": thread_id,
        }),
    )
    .await;
    assert_eq!(status, 200, "the proposal registers: {proposal}");
    assert_eq!(proposal["status"], json!("draft"));

    // 2. The ghost policy + the ghost thread refuse.
    let (status, refused) = post(
        &client,
        &base,
        "/v1/policy-proposals",
        &human_id,
        &json!({
            "proposal_id": "lc-ghost-policy",
            "policy_id": "ghost",
            "policy_version": "1.0.0",
            "thread_id": thread_id,
        }),
    )
    .await;
    assert_eq!(status, 400, "the ghost policy refuses: {refused}");
    let (status, refused) = post(
        &client,
        &base,
        "/v1/policy-proposals",
        &human_id,
        &json!({
            "proposal_id": "lc-ghost-thread",
            "policy_id": "lc-policy",
            "policy_version": "1.0.0",
            "thread_id": "thr_ghost",
        }),
    )
    .await;
    assert_eq!(status, 400, "the ghost thread refuses: {refused}");

    // 3. The decision: the frozen electorate snapshot + the verdict
    // reference; the proposal advances to `decided`.
    let (status, decision) = post(
        &client,
        &base,
        "/v1/policy-decisions",
        &human_id,
        &json!({
            "decision_id": "lc-dec-1",
            "proposal_id": "lc-prop-1",
            "rule": "unanimity",
            "electorate": {
                "participants": [human_id],
                "denominator": 1,
                "abstentions": [],
            },
            "verdict_event_id": verdict_event,
        }),
    )
    .await;
    assert_eq!(status, 200, "the decision records: {decision}");
    assert_eq!(decision["rule"], json!("unanimity"));
    let (status, proposals) = get(&client, &base, "/v1/policy-proposals", &human_id).await;
    assert_eq!(status, 200, "the proposals read: {proposals}");
    assert_eq!(
        proposals[0]["status"],
        json!("decided"),
        "the stage advanced"
    );

    // 4. A second decision on the SAME proposal refuses (one proposal, one
    // decision — the stage gate).
    let (status, refused) = post(
        &client,
        &base,
        "/v1/policy-decisions",
        &human_id,
        &json!({
            "decision_id": "lc-dec-2",
            "proposal_id": "lc-prop-1",
            "rule": "unanimity",
            "electorate": { "participants": [human_id], "denominator": 1, "abstentions": [] },
            "verdict_event_id": verdict_event,
        }),
    )
    .await;
    assert_eq!(status, 400, "the second decision refuses: {refused}");
    assert!(
        refused["message"].as_str().unwrap().contains("draft"),
        "{refused}"
    );

    // 5. A verdict from ANOTHER thread refuses (the reference is scoped to
    // the proposal's thread).
    let (_status, created) = post(
        &client,
        &base,
        "/v1/threads",
        &human_id,
        &json!({
            "protocol_version": reasonbraid_core::PROTOCOL_VERSION,
            "operation": "thread.create",
            "request_id": reasonbraid_core::RequestId::new().to_string(),
            "idempotency_key": "lc-other-thread",
            "body": {
                "tenant_id": tenant_id,
                "subject": "lc-other",
                "objective": "probe",
                "workflow_profile": "independent_panel",
            },
            "client_context": {},
        }),
    )
    .await;
    let other_thread = created["thread_id"].as_str().unwrap().to_string();
    let (status, _) = post(
        &client,
        &base,
        "/v1/policy-proposals",
        &human_id,
        &json!({
            "proposal_id": "lc-prop-2",
            "policy_id": "lc-policy",
            "policy_version": "1.0.0",
            "thread_id": other_thread,
        }),
    )
    .await;
    assert_eq!(status, 200, "the second proposal registers");
    let (status, refused) = post(
        &client,
        &base,
        "/v1/policy-decisions",
        &human_id,
        &json!({
            "decision_id": "lc-dec-3",
            "proposal_id": "lc-prop-2",
            "rule": "unanimity",
            "electorate": { "participants": [human_id], "denominator": 1, "abstentions": [] },
            "verdict_event_id": verdict_event,
        }),
    )
    .await;
    assert_eq!(status, 400, "the foreign verdict refuses: {refused}");
    assert!(
        refused["message"].as_str().unwrap().contains("verdict"),
        "{refused}"
    );

    // 6. The empty electorate refuses; the lists carry both records.
    let (status, refused) = post(
        &client,
        &base,
        "/v1/policy-decisions",
        &human_id,
        &json!({
            "decision_id": "lc-dec-4",
            "proposal_id": "lc-prop-2",
            "rule": "unanimity",
            "electorate": { "participants": [], "denominator": 0, "abstentions": [] },
            "verdict_event_id": verdict_event,
        }),
    )
    .await;
    assert_eq!(status, 400, "the empty electorate refuses: {refused}");
    let (status, decisions) = get(&client, &base, "/v1/policy-decisions", &human_id).await;
    assert_eq!(status, 200, "the decisions read: {decisions}");
    let decisions = decisions.as_array().unwrap();
    assert_eq!(decisions.len(), 1, "{decisions:?}");
    assert_eq!(decisions[0]["decision_id"], json!("lc-dec-1"));
}

#[tokio::test]
async fn the_approval_carries_its_authority_proof() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, human) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "ap-human" }),
    )
    .await;
    assert_eq!(status, 200, "the human enrolls: {human}");
    let human_id = human["principal_id"].as_str().unwrap().to_string();
    let tenant_id = human["tenant_id"].as_str().unwrap().to_string();
    let grant_id = format!("grt_{human_id}");

    // The chain: the policy → the thread + the verdict → the proposal →
    // the decision.
    let (status, _) = post(
        &client,
        &base,
        "/v1/policies",
        &human_id,
        &json!({
            "policy_id": "ap-policy",
            "version": "1.0.0",
            "digest": DIGEST,
            "lifecycle": "draft",
            "title": "ap",
            "owning_authority": grant_id,
            "clauses": [ { "id": "c1", "statement": "the approval clause" } ],
        }),
    )
    .await;
    assert_eq!(status, 200, "the policy registers");
    let (_status, created) = post(
        &client,
        &base,
        "/v1/threads",
        &human_id,
        &json!({
            "protocol_version": reasonbraid_core::PROTOCOL_VERSION,
            "operation": "thread.create",
            "request_id": reasonbraid_core::RequestId::new().to_string(),
            "idempotency_key": "ap-create",
            "body": {
                "tenant_id": tenant_id,
                "subject": "ap",
                "objective": "probe",
                "workflow_profile": "independent_panel",
            },
            "client_context": {},
        }),
    )
    .await;
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
    let (status, _) = command(
        "ap-advance",
        "thread.advance_round",
        json!({ "tenant_id": tenant_id }),
    )
    .await;
    assert_eq!(status, 200, "the round advances");
    let (_status, verdict) = command(
        "ap-verdict",
        "thread.contribute",
        json!({
            "tenant_id": tenant_id,
            "content": "judged",
            "kind": "verdict",
            "verdict": { "target_digest": "sha256:00", "rule": "majority", "outcome": "accepted_by_rule" },
        }),
    )
    .await;
    let verdict_event = verdict["event_id"].as_str().unwrap().to_string();

    let register_proposal = |proposal_id: &'static str| {
        let client = client.clone();
        let base = base.clone();
        let human_id = human_id.clone();
        let thread_id = thread_id.clone();
        async move {
            post(
                &client,
                &base,
                "/v1/policy-proposals",
                &human_id,
                &json!({
                    "proposal_id": proposal_id,
                    "policy_id": "ap-policy",
                    "policy_version": "1.0.0",
                    "thread_id": thread_id,
                }),
            )
            .await
        }
    };
    let decide = |decision_id: &'static str, proposal_id: &'static str| {
        let client = client.clone();
        let base = base.clone();
        let human_id = human_id.clone();
        let verdict_event = verdict_event.clone();
        async move {
            post(
                &client,
                &base,
                "/v1/policy-decisions",
                &human_id,
                &json!({
                    "decision_id": decision_id,
                    "proposal_id": proposal_id,
                    "rule": "majority",
                    "electorate": { "participants": [human_id], "denominator": 1, "abstentions": [] },
                    "verdict_event_id": verdict_event,
                }),
            )
            .await
        }
    };

    let (status, _) = register_proposal("ap-prop-1").await;
    assert_eq!(status, 200, "the proposal registers");
    let (status, _) = decide("ap-dec-1", "ap-prop-1").await;
    assert_eq!(status, 200, "the decision records");

    // 1. The approval: the matching grant (the approver IS the holder) →
    // the proof passes, the proposal advances to `approved`.
    let (status, approval) = post(
        &client,
        &base,
        "/v1/policy-approvals",
        &human_id,
        &json!({
            "approval_id": "ap-app-1",
            "proposal_id": "ap-prop-1",
            "decision_id": "ap-dec-1",
            "approver": human_id,
            "grant_id": grant_id,
            "quorum": { "participants": [human_id], "denominator": 1, "abstentions": [] },
        }),
    )
    .await;
    assert_eq!(status, 200, "the approval records: {approval}");
    assert_eq!(approval["grant_id"], json!(grant_id));
    let (status, proposals) = get(&client, &base, "/v1/policy-proposals", &human_id).await;
    assert_eq!(status, 200, "the proposals read: {proposals}");
    assert_eq!(
        proposals[0]["status"],
        json!("approved"),
        "the stage advanced"
    );

    // 2. A second approval on the approved proposal refuses (the stage gate).
    let (status, refused) = post(
        &client,
        &base,
        "/v1/policy-approvals",
        &human_id,
        &json!({
            "approval_id": "ap-app-2",
            "proposal_id": "ap-prop-1",
            "decision_id": "ap-dec-1",
            "approver": human_id,
            "grant_id": grant_id,
            "quorum": { "participants": [human_id], "denominator": 1, "abstentions": [] },
        }),
    )
    .await;
    assert_eq!(status, 400, "the second approval refuses: {refused}");
    assert!(
        refused["message"].as_str().unwrap().contains("approved"),
        "the refusal names the stage: {refused}"
    );

    // 3. The authority proof: a grant NOT held by the approver refuses (the
    // §4.5 identity check at the action time).
    let (status, _) = register_proposal("ap-prop-2").await;
    assert_eq!(status, 200, "the second proposal registers");
    let (status, _) = decide("ap-dec-2", "ap-prop-2").await;
    assert_eq!(status, 200, "the second decision records");
    let (status, refused) = post(
        &client,
        &base,
        "/v1/policy-approvals",
        &human_id,
        &json!({
            "approval_id": "ap-app-3",
            "proposal_id": "ap-prop-2",
            "decision_id": "ap-dec-2",
            "approver": human_id,
            "grant_id": "grt_hpr_ghost",
            "quorum": { "participants": [human_id], "denominator": 1, "abstentions": [] },
        }),
    )
    .await;
    assert_eq!(status, 400, "the mismatched proof refuses: {refused}");
    assert!(
        refused["message"]
            .as_str()
            .unwrap()
            .contains("authority proof"),
        "{refused}"
    );

    // 4. The FOREIGN decision refuses (the decision must belong to the
    // proposal).
    let (status, refused) = post(
        &client,
        &base,
        "/v1/policy-approvals",
        &human_id,
        &json!({
            "approval_id": "ap-app-4",
            "proposal_id": "ap-prop-2",
            "decision_id": "ap-dec-1",
            "approver": human_id,
            "grant_id": grant_id,
            "quorum": { "participants": [human_id], "denominator": 1, "abstentions": [] },
        }),
    )
    .await;
    assert_eq!(status, 400, "the foreign decision refuses: {refused}");
    assert!(
        refused["message"]
            .as_str()
            .unwrap()
            .contains("does not belong"),
        "{refused}"
    );

    // 5. The approval on a DRAFT proposal refuses (no decision yet).
    let (status, _) = register_proposal("ap-prop-3").await;
    assert_eq!(status, 200, "the third proposal registers");
    let (status, refused) = post(
        &client,
        &base,
        "/v1/policy-approvals",
        &human_id,
        &json!({
            "approval_id": "ap-app-5",
            "proposal_id": "ap-prop-3",
            "decision_id": "ap-dec-1",
            "approver": human_id,
            "grant_id": grant_id,
            "quorum": { "participants": [human_id], "denominator": 1, "abstentions": [] },
        }),
    )
    .await;
    assert_eq!(status, 400, "the draft-stage approval refuses: {refused}");
    assert!(
        refused["message"].as_str().unwrap().contains("draft"),
        "{refused}"
    );

    // 6. The empty quorum refuses; the list carries the one approval.
    let (status, refused) = post(
        &client,
        &base,
        "/v1/policy-approvals",
        &human_id,
        &json!({
            "approval_id": "ap-app-6",
            "proposal_id": "ap-prop-2",
            "decision_id": "ap-dec-2",
            "approver": human_id,
            "grant_id": grant_id,
            "quorum": { "participants": [], "denominator": 0, "abstentions": [] },
        }),
    )
    .await;
    assert_eq!(status, 400, "the empty quorum refuses: {refused}");
    let (status, approvals) = get(&client, &base, "/v1/policy-approvals", &human_id).await;
    assert_eq!(status, 200, "the approvals read: {approvals}");
    let approvals = approvals.as_array().unwrap();
    assert_eq!(approvals.len(), 1, "{approvals:?}");
    assert_eq!(approvals[0]["approval_id"], json!("ap-app-1"));
}

#[tokio::test]
async fn the_projection_compiles_the_resolved_set_byte_identical() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, human) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "pj-human" }),
    )
    .await;
    assert_eq!(status, 200, "the human enrolls: {human}");
    let human_id = human["principal_id"].as_str().unwrap().to_string();
    let grant_id = format!("grt_{human_id}");

    // Two policies: the baseline (the c1/c2 clauses) + the shape policy
    // (the c3 clause with a CONTROL character — the compiler declares it
    // unrepresentable instead of silently dropping it).
    for (policy_id, clauses) in [
        (
            "pj-base",
            json!([
                { "id": "c1", "statement": "every thread declares its objective" },
                { "id": "c2", "statement": "every publication names its authority" },
            ]),
        ),
        (
            "pj-shape",
            json!([ { "id": "c3", "statement": "a control \u{0001} character" } ]),
        ),
    ] {
        let (status, registered) = post(
            &client,
            &base,
            "/v1/policies",
            &human_id,
            &json!({
                "policy_id": policy_id,
                "version": "1.0.0",
                "digest": DIGEST,
                "lifecycle": "draft",
                "title": policy_id,
                "owning_authority": grant_id,
                "clauses": clauses,
            }),
        )
        .await;
        assert_eq!(status, 200, "the policy registers: {registered}");
    }

    // 1. The generic projection: the resolved set renders byte-identically
    // (the digest pins it), and the control-char clause DECLARES itself.
    let (status, projected) = post(
        &client,
        &base,
        "/v1/policy-projections",
        &human_id,
        &json!({
            "projection_id": "pj-1",
            "target": "generic",
            "resolution": {
                "policies": [
                    { "policy_id": "pj-base", "version": "1.0.0" },
                    { "policy_id": "pj-shape", "version": "1.0.0" },
                ],
                "target": { "layer": "organization", "target": "*" },
            },
            "lock": [],
        }),
    )
    .await;
    assert_eq!(status, 200, "the projection records: {projected}");
    assert!(projected["digest"].as_str().unwrap().starts_with("sha256:"));
    assert!(
        projected["bytes"]
            .as_str()
            .unwrap()
            .contains("## c1 [pj-base 1.0.0]"),
        "the bundle renders the clause: {projected}"
    );
    assert!(
        projected["bytes"]
            .as_str()
            .unwrap()
            .contains("## c2 [pj-base 1.0.0]"),
        "the second clause renders"
    );
    let unrepresentable = projected["unrepresentable"].as_array().unwrap();
    assert_eq!(unrepresentable.len(), 1, "{projected}");
    assert_eq!(unrepresentable[0]["clause_id"], json!("c3"));
    assert!(
        unrepresentable[0]["reason"]
            .as_str()
            .unwrap()
            .contains("control"),
        "{projected}"
    );

    // 2. The SAME request re-projects byte-identically (a second id, the
    // identical digest).
    let (status, repeated) = post(
        &client,
        &base,
        "/v1/policy-projections",
        &human_id,
        &json!({
            "projection_id": "pj-1-repeat",
            "target": "generic",
            "resolution": {
                "policies": [
                    { "policy_id": "pj-base", "version": "1.0.0" },
                    { "policy_id": "pj-shape", "version": "1.0.0" },
                ],
                "target": { "layer": "organization", "target": "*" },
            },
            "lock": [],
        }),
    )
    .await;
    assert_eq!(status, 200, "the repeat projects: {repeated}");
    assert_eq!(
        repeated["digest"], projected["digest"],
        "the byte-identical guarantee"
    );

    // 3. The policy.lock projection renders the stable rows.
    let (status, locked) = post(
        &client,
        &base,
        "/v1/policy-projections",
        &human_id,
        &json!({
            "projection_id": "pj-lock",
            "target": "lock",
            "resolution": {
                "policies": [ { "policy_id": "pj-base", "version": "1.0.0" } ],
                "target": { "layer": "organization", "target": "*" },
            },
            "lock": [
                { "policy_id": "pj-base", "version": "1.0.0", "digest": DIGEST, "owning_authority": grant_id },
            ],
        }),
    )
    .await;
    assert_eq!(status, 200, "the lock projects: {locked}");
    assert!(
        locked["bytes"]
            .as_str()
            .unwrap()
            .contains(&format!("pj-base 1.0.0 {DIGEST} {grant_id}")),
        "{locked}"
    );

    // 4. An unknown target refuses (the named deferral — no adapter).
    let (status, refused) = post(
        &client,
        &base,
        "/v1/policy-projections",
        &human_id,
        &json!({
            "projection_id": "pj-mcp",
            "target": "mcp",
            "resolution": {
                "policies": [ { "policy_id": "pj-base", "version": "1.0.0" } ],
                "target": { "layer": "organization", "target": "*" },
            },
        }),
    )
    .await;
    assert_eq!(status, 400, "the unknown target refuses: {refused}");
    assert!(
        refused["message"].as_str().unwrap().contains("vocabulary"),
        "{refused}"
    );

    // 5. The duplicate projection id refuses; the list carries the three.
    let (status, refused) = post(
        &client,
        &base,
        "/v1/policy-projections",
        &human_id,
        &json!({
            "projection_id": "pj-1",
            "target": "generic",
            "resolution": {
                "policies": [ { "policy_id": "pj-base", "version": "1.0.0" } ],
                "target": { "layer": "organization", "target": "*" },
            },
        }),
    )
    .await;
    assert_eq!(status, 400, "the duplicate refuses: {refused}");
    let (status, projections) = get(&client, &base, "/v1/policy-projections", &human_id).await;
    assert_eq!(status, 200, "the projections read: {projections}");
    assert_eq!(projections.as_array().unwrap().len(), 3, "{projections:?}");
}

#[tokio::test]
async fn the_codex_and_claude_projections_ride_the_verb() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, human) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "cc-human" }),
    )
    .await;
    assert_eq!(status, 200, "the human enrolls: {human}");
    let human_id = human["principal_id"].as_str().unwrap().to_string();
    let grant_id = format!("grt_{human_id}");

    let (status, _) = post(
        &client,
        &base,
        "/v1/policies",
        &human_id,
        &json!({
            "policy_id": "cc-policy",
            "version": "1.0.0",
            "digest": DIGEST,
            "lifecycle": "draft",
            "title": "cc",
            "owning_authority": grant_id,
            "clauses": [ { "id": "c1", "statement": "the harness clause" } ],
        }),
    )
    .await;
    assert_eq!(status, 200, "the policy registers");

    for (target, marker) in [
        ("codex", "- `c1` [cc-policy 1.0.0]: the harness clause"),
        ("claude", "- c1 [cc-policy 1.0.0]: the harness clause"),
    ] {
        let (status, projected) = post(
            &client,
            &base,
            "/v1/policy-projections",
            &human_id,
            &json!({
                "projection_id": format!("cc-{target}"),
                "target": target,
                "resolution": {
                    "policies": [ { "policy_id": "cc-policy", "version": "1.0.0" } ],
                    "target": { "layer": "organization", "target": "*" },
                },
            }),
        )
        .await;
        assert_eq!(status, 200, "the {target} projection: {projected}");
        assert!(
            projected["bytes"].as_str().unwrap().contains(marker),
            "the {target} harness shape renders: {projected}"
        );
        assert!(projected["digest"].as_str().unwrap().starts_with("sha256:"));
    }
}

#[tokio::test]
async fn the_publication_stages_and_marks_its_typed_state() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    // `.9.2.1.3`: this fixture drives the effective transition, so it needs a
    // configured repository and ids READ BACK from it — it used to declare
    // `abc123`, which resolves to nothing.
    let (repo_root, object_ids) = seeded_publication_repository("staging");
    let server = TestServer::start_with_publication_root(&pool, &repo_root).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, human) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "pb-human" }),
    )
    .await;
    assert_eq!(status, 200, "the human enrolls: {human}");
    let human_id = human["principal_id"].as_str().unwrap().to_string();
    let tenant_id = human["tenant_id"].as_str().unwrap().to_string();
    let grant_id = format!("grt_{human_id}");

    // The chain: the policy → the thread + the verdict → the proposal →
    // the decision → the approval → the projection.
    let (status, _) = post(
        &client,
        &base,
        "/v1/policies",
        &human_id,
        &json!({
            "policy_id": "pb-policy",
            "version": "1.0.0",
            "digest": DIGEST,
            "lifecycle": "draft",
            "title": "pb",
            "owning_authority": grant_id,
            "clauses": [ { "id": "c1", "statement": "the publication clause" } ],
        }),
    )
    .await;
    assert_eq!(status, 200, "the policy registers");
    let (_status, created) = post(
        &client,
        &base,
        "/v1/threads",
        &human_id,
        &json!({
            "protocol_version": reasonbraid_core::PROTOCOL_VERSION,
            "operation": "thread.create",
            "request_id": reasonbraid_core::RequestId::new().to_string(),
            "idempotency_key": "pb-create",
            "body": {
                "tenant_id": tenant_id,
                "subject": "pb",
                "objective": "probe",
                "workflow_profile": "independent_panel",
            },
            "client_context": {},
        }),
    )
    .await;
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
    let (status, _) = command(
        "pb-advance",
        "thread.advance_round",
        json!({ "tenant_id": tenant_id }),
    )
    .await;
    assert_eq!(status, 200, "the round advances");
    let (_status, verdict) = command(
        "pb-verdict",
        "thread.contribute",
        json!({
            "tenant_id": tenant_id,
            "content": "judged",
            "kind": "verdict",
            "verdict": { "target_digest": "sha256:00", "rule": "majority", "outcome": "accepted_by_rule" },
        }),
    )
    .await;
    let verdict_event = verdict["event_id"].as_str().unwrap().to_string();

    let (status, _) = post(
        &client,
        &base,
        "/v1/policy-proposals",
        &human_id,
        &json!({
            "proposal_id": "pb-prop",
            "policy_id": "pb-policy",
            "policy_version": "1.0.0",
            "thread_id": thread_id,
        }),
    )
    .await;
    assert_eq!(status, 200, "the proposal registers");
    let (status, _) = post(
        &client,
        &base,
        "/v1/policy-decisions",
        &human_id,
        &json!({
            "decision_id": "pb-dec",
            "proposal_id": "pb-prop",
            "rule": "majority",
            "electorate": { "participants": [human_id], "denominator": 1, "abstentions": [] },
            "verdict_event_id": verdict_event,
        }),
    )
    .await;
    assert_eq!(status, 200, "the decision records");
    let (status, _) = post(
        &client,
        &base,
        "/v1/policy-approvals",
        &human_id,
        &json!({
            "approval_id": "pb-app",
            "proposal_id": "pb-prop",
            "decision_id": "pb-dec",
            "approver": human_id,
            "grant_id": grant_id,
            "quorum": { "participants": [human_id], "denominator": 1, "abstentions": [] },
        }),
    )
    .await;
    assert_eq!(status, 200, "the approval records");
    let (status, projection) = post(
        &client,
        &base,
        "/v1/policy-projections",
        &human_id,
        &json!({
            "projection_id": "pb-proj",
            "target": "generic",
            "resolution": {
                "policies": [ { "policy_id": "pb-policy", "version": "1.0.0" } ],
                "target": { "layer": "organization", "target": "*" },
            },
        }),
    )
    .await;
    assert_eq!(status, 200, "the projection records: {projection}");
    let manifest_digest = projection["digest"].as_str().unwrap().to_string();

    // 1. The publication stages (the references verified, the staged
    // state).
    let (status, publication) = post(
        &client,
        &base,
        "/v1/policy-publications",
        &human_id,
        &json!({
            "publication_id": "pb-pub",
            "proposal_id": "pb-prop",
            "decision_id": "pb-dec",
            "approval_id": "pb-app",
            "projection_id": "pb-proj",
            "manifest_digest": manifest_digest,
        }),
    )
    .await;
    assert_eq!(status, 200, "the publication stages: {publication}");
    assert_eq!(publication["state"], json!("staged"));

    // `.9.2.1.3` — a declared object id that resolves to nothing is REFUSED,
    // and this is the exact shape the fixture itself used to send. Until that
    // leaf the transition recorded whatever arrived: the suite meant to
    // qualify it was proving the transition with ids that do not exist.
    let (status, refused) = post(
        &client,
        &base,
        "/v1/policy-publications/pb-pub/effective",
        &human_id,
        &json!({ "git_object_ids": ["abc123", "def456"], "repo_path": "live", "owning_authority": grant_id }),
    )
    .await;
    assert_eq!(status, 400, "a fabricated object id refuses: {refused}");
    assert!(
        refused["message"]
            .as_str()
            .unwrap()
            .contains("name nothing in the publication repository"),
        "{refused}"
    );

    // THE MATCHED PAIR, and it is per-id rather than per-request: swap ONE of
    // the two for a real id and it is still refused, naming only the one that
    // is missing. A check that asked "does any declared id resolve" would pass
    // this, and would let a real id launder a fabricated one beside it.
    let (status, refused) = post(
        &client,
        &base,
        "/v1/policy-publications/pb-pub/effective",
        &human_id,
        &json!({ "git_object_ids": [object_ids[0].clone(), "def456"], "repo_path": "live", "owning_authority": grant_id }),
    )
    .await;
    assert_eq!(
        status, 400,
        "one real id does not launder the other: {refused}"
    );
    let message = refused["message"].as_str().unwrap();
    assert!(message.contains("def456"), "{refused}");
    assert!(
        !message.contains(&object_ids[0]),
        "the real id is not named as missing: {refused}"
    );

    // A refused transition wrote nothing: the publication is still staged, so
    // the leg below is still exercising staged -> effective.
    let (status, listed) = get(&client, &base, "/v1/policy-publications", &human_id).await;
    assert_eq!(status, 200, "{listed}");
    assert_eq!(
        listed[0]["state"],
        json!("staged"),
        "the refusal left the publication staged: {listed}"
    );

    // 2. The effective transition records the Git object ids.
    let (status, effective) = post(
        &client,
        &base,
        "/v1/policy-publications/pb-pub/effective",
        &human_id,
        &json!({ "git_object_ids": object_ids.clone(), "repo_path": "live", "owning_authority": grant_id }),
    )
    .await;
    assert_eq!(status, 200, "the publication marks effective: {effective}");
    assert_eq!(effective["state"], json!("effective"));
    assert_eq!(effective["git_object_ids"], json!(object_ids));

    // 3. The refusals: the bad manifest digest, the ghost projection, the
    // foreign decision, the non-approved proposal, the wrong-stage
    // transitions, the empty object ids, the duplicate.
    let (status, refused) = post(
        &client,
        &base,
        "/v1/policy-publications",
        &human_id,
        &json!({
            "publication_id": "pb-bad-digest",
            "proposal_id": "pb-prop",
            "decision_id": "pb-dec",
            "approval_id": "pb-app",
            "projection_id": "pb-proj",
            "manifest_digest": "not-a-digest",
        }),
    )
    .await;
    assert_eq!(status, 400, "the bad digest refuses: {refused}");
    let (status, refused) = post(
        &client,
        &base,
        "/v1/policy-publications",
        &human_id,
        &json!({
            "publication_id": "pb-ghost-proj",
            "proposal_id": "pb-prop",
            "decision_id": "pb-dec",
            "approval_id": "pb-app",
            "projection_id": "ghost",
            "manifest_digest": manifest_digest,
        }),
    )
    .await;
    assert_eq!(status, 400, "the ghost projection refuses: {refused}");
    let (status, refused) = post(
        &client,
        &base,
        "/v1/policy-publications",
        &human_id,
        &json!({
            "publication_id": "pb-foreign",
            "proposal_id": "pb-prop",
            "decision_id": "ghost-dec",
            "approval_id": "pb-app",
            "projection_id": "pb-proj",
            "manifest_digest": manifest_digest,
        }),
    )
    .await;
    assert_eq!(status, 400, "the ghost decision refuses: {refused}");
    // The effective → effective again refuses (the terminal state).
    let (status, refused) = post(
        &client,
        &base,
        "/v1/policy-publications/pb-pub/effective",
        &human_id,
        &json!({ "git_object_ids": ["zzz"], "repo_path": "live", "owning_authority": grant_id }),
    )
    .await;
    assert_eq!(status, 400, "the terminal re-transition refuses: {refused}");
    assert!(
        refused["message"].as_str().unwrap().contains("effective"),
        "{refused}"
    );

    // 4. The typed failure: a second publication on a NEW proposal chain
    // (the same thread + a new proposal) stages, then marks FAILED with
    // the reason.
    let (status, _) = post(
        &client,
        &base,
        "/v1/policy-proposals",
        &human_id,
        &json!({
            "proposal_id": "pb-prop-2",
            "policy_id": "pb-policy",
            "policy_version": "1.0.0",
            "thread_id": thread_id,
        }),
    )
    .await;
    assert_eq!(status, 200, "the second proposal registers");
    let (status, _) = post(
        &client,
        &base,
        "/v1/policy-decisions",
        &human_id,
        &json!({
            "decision_id": "pb-dec-2",
            "proposal_id": "pb-prop-2",
            "rule": "majority",
            "electorate": { "participants": [human_id], "denominator": 1, "abstentions": [] },
            "verdict_event_id": verdict_event,
        }),
    )
    .await;
    assert_eq!(status, 200, "the second decision records");
    let (status, _) = post(
        &client,
        &base,
        "/v1/policy-approvals",
        &human_id,
        &json!({
            "approval_id": "pb-app-2",
            "proposal_id": "pb-prop-2",
            "decision_id": "pb-dec-2",
            "approver": human_id,
            "grant_id": grant_id,
            "quorum": { "participants": [human_id], "denominator": 1, "abstentions": [] },
        }),
    )
    .await;
    assert_eq!(status, 200, "the second approval records");
    let (status, _) = post(
        &client,
        &base,
        "/v1/policy-publications",
        &human_id,
        &json!({
            "publication_id": "pb-pub-2",
            "proposal_id": "pb-prop-2",
            "decision_id": "pb-dec-2",
            "approval_id": "pb-app-2",
            "projection_id": "pb-proj",
            "manifest_digest": manifest_digest,
        }),
    )
    .await;
    assert_eq!(status, 200, "the second publication stages");
    let (status, failed) = post(
        &client,
        &base,
        "/v1/policy-publications/pb-pub-2/failed",
        &human_id,
        &json!({ "reason": "the fetch-back verification failed" }),
    )
    .await;
    assert_eq!(status, 200, "the publication marks failed: {failed}");
    assert_eq!(failed["state"], json!("failed"));
    assert_eq!(
        failed["failed_reason"],
        json!("the fetch-back verification failed")
    );

    // 5. The list: the two publications, newest first.
    let (status, publications) = get(&client, &base, "/v1/policy-publications", &human_id).await;
    assert_eq!(status, 200, "the publications read: {publications}");
    let publications = publications.as_array().unwrap();
    assert_eq!(publications.len(), 2, "{publications:?}");
    assert_eq!(publications[0]["publication_id"], json!("pb-pub-2"));
}

#[tokio::test]
async fn the_publish_verb_drives_the_git_half() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    // `.9.2.1.1`: the server declares WHERE publications may be written, and
    // the request names a location inside it. This control is re-pointed at a
    // configured root rather than deleted — it still asserts the Git half.
    let repo_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/policy-publish-tests");
    let repo_dir = repo_root.join("live");
    let not_a_repository = repo_root.join("not-a-repository");
    let _ = std::fs::remove_dir_all(&repo_root);
    std::fs::create_dir_all(&repo_dir).expect("the dir creates");
    std::fs::create_dir_all(&not_a_repository).expect("the non-repository dir creates");
    gix::init_bare(&repo_dir).expect("the bare repo inits");
    let server = TestServer::start_with_publication_root(&pool, &repo_root).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, human) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "pu-human" }),
    )
    .await;
    assert_eq!(status, 200, "the human enrolls: {human}");
    let human_id = human["principal_id"].as_str().unwrap().to_string();
    let tenant_id = human["tenant_id"].as_str().unwrap().to_string();
    let grant_id = format!("grt_{human_id}");

    // The chain: the policy → the thread + the verdict → the proposal →
    // the decision → the approval → the projection → the staged publication.
    let (status, _) = post(
        &client,
        &base,
        "/v1/policies",
        &human_id,
        &json!({
            "policy_id": "pu-policy",
            "version": "1.0.0",
            "digest": DIGEST,
            "lifecycle": "draft",
            "title": "pu",
            "owning_authority": grant_id,
            "clauses": [ { "id": "c1", "statement": "the published clause" } ],
        }),
    )
    .await;
    assert_eq!(status, 200, "the policy registers");
    let (_status, created) = post(
        &client,
        &base,
        "/v1/threads",
        &human_id,
        &json!({
            "protocol_version": reasonbraid_core::PROTOCOL_VERSION,
            "operation": "thread.create",
            "request_id": reasonbraid_core::RequestId::new().to_string(),
            "idempotency_key": "pu-create",
            "body": {
                "tenant_id": tenant_id,
                "subject": "pu",
                "objective": "probe",
                "workflow_profile": "independent_panel",
            },
            "client_context": {},
        }),
    )
    .await;
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
    let (status, _) = command(
        "pu-advance",
        "thread.advance_round",
        json!({ "tenant_id": tenant_id }),
    )
    .await;
    assert_eq!(status, 200, "the round advances");
    let (_status, verdict) = command(
        "pu-verdict",
        "thread.contribute",
        json!({
            "tenant_id": tenant_id,
            "content": "judged",
            "kind": "verdict",
            "verdict": { "target_digest": "sha256:00", "rule": "majority", "outcome": "accepted_by_rule" },
        }),
    )
    .await;
    let verdict_event = verdict["event_id"].as_str().unwrap().to_string();
    for (proposal_id, decision_id, approval_id, publication_id) in
        [("pu-prop", "pu-dec", "pu-app", "pu-pub")]
    {
        let (status, _) = post(
            &client,
            &base,
            "/v1/policy-proposals",
            &human_id,
            &json!({
                "proposal_id": proposal_id,
                "policy_id": "pu-policy",
                "policy_version": "1.0.0",
                "thread_id": thread_id,
            }),
        )
        .await;
        assert_eq!(status, 200, "the proposal registers");
        let (status, _) = post(
            &client,
            &base,
            "/v1/policy-decisions",
            &human_id,
            &json!({
                "decision_id": decision_id,
                "proposal_id": proposal_id,
                "rule": "majority",
                "electorate": { "participants": [human_id], "denominator": 1, "abstentions": [] },
                "verdict_event_id": verdict_event,
            }),
        )
        .await;
        assert_eq!(status, 200, "the decision records");
        let (status, _) = post(
            &client,
            &base,
            "/v1/policy-approvals",
            &human_id,
            &json!({
                "approval_id": approval_id,
                "proposal_id": proposal_id,
                "decision_id": decision_id,
                "approver": human_id,
                "grant_id": grant_id,
                "quorum": { "participants": [human_id], "denominator": 1, "abstentions": [] },
            }),
        )
        .await;
        assert_eq!(status, 200, "the approval records");
        let (status, projection) = post(
            &client,
            &base,
            "/v1/policy-projections",
            &human_id,
            &json!({
                "projection_id": format!("{proposal_id}-proj"),
                "target": "generic",
                "resolution": {
                    "policies": [ { "policy_id": "pu-policy", "version": "1.0.0" } ],
                    "target": { "layer": "organization", "target": "*" },
                },
            }),
        )
        .await;
        assert_eq!(status, 200, "the projection records");
        let (status, _) = post(
            &client,
            &base,
            "/v1/policy-publications",
            &human_id,
            &json!({
                "publication_id": publication_id,
                "proposal_id": proposal_id,
                "decision_id": decision_id,
                "approval_id": approval_id,
                "projection_id": format!("{proposal_id}-proj"),
                "manifest_digest": projection["digest"],
            }),
        )
        .await;
        assert_eq!(status, 200, "the publication stages");
    }

    // 1. A publish into a NON-repository location refuses with the store
    // contract's own open failure. ⛔ THIS LEG RUNS FIRST, AND THAT IS THE
    // WHOLE POINT. It used to run third, after the publication had already
    // been driven to `effective`, so the STAGE check answered it and it never
    // reached `gix::open` at all — green for the wrong reason, for as long as
    // it has existed. `.9.2.1.1` found it by pointing the leg at a location
    // inside the configured root and watching it fail on the stage instead.
    // The location exists and is a directory, so the containment check passes
    // it through and only the repository open can refuse it.
    assert!(
        not_a_repository.is_dir(),
        "the non-repository leg must exercise an existing location"
    );
    let (status, refused) = post(
        &client,
        &base,
        "/v1/policy-publications/pu-pub/publish",
        &human_id,
        &json!({ "repo_path": "not-a-repository", "owning_authority": grant_id }),
    )
    .await;
    assert_eq!(status, 400, "the non-repository refuses: {refused}");
    assert!(
        refused["message"]
            .as_str()
            .unwrap()
            .contains("does not open"),
        "the refusal is the store contract's, not the stage check's or the \
         containment check's: {refused}"
    );
    // The stage is asserted rather than assumed, so reordering this leg behind
    // a successful publish fails loudly instead of silently changing what it
    // measures.
    let (status, staged) = get(&client, &base, "/v1/policy-publications", &human_id).await;
    assert_eq!(status, 200, "{staged}");
    assert_eq!(
        staged[0]["state"],
        json!("staged"),
        "the refused publish left the publication staged: {staged}"
    );

    // 2. The publish drives the Git half: the bare repo + the verb → the
    // record marks effective with the ref ids. The location is named RELATIVE
    // to the configured root, which is the shape the root exists to support.
    let (status, published) = post(
        &client,
        &base,
        "/v1/policy-publications/pu-pub/publish",
        &human_id,
        &json!({ "repo_path": "live", "owning_authority": grant_id }),
    )
    .await;
    assert_eq!(status, 200, "the publish drives the git half: {published}");
    assert_eq!(published["state"], json!("effective"));
    let object_ids = published["git_object_ids"].as_array().unwrap();
    assert_eq!(object_ids.len(), 2, "{published}");

    // 3. A publish on a NON-staged publication refuses (the effective is
    // terminal).
    let (status, refused) = post(
        &client,
        &base,
        "/v1/policy-publications/pu-pub/publish",
        &human_id,
        &json!({ "repo_path": "live", "owning_authority": grant_id }),
    )
    .await;
    assert_eq!(status, 400, "the terminal re-publish refuses: {refused}");
    assert!(
        refused["message"].as_str().unwrap().contains("staged"),
        "{refused}"
    );

    let _ = std::fs::remove_dir_all(&repo_root);
}

/// `SIGNOFF-REPAIR.9.2.1.1` — the publish verb used to take its repository
/// location from the request body and hand it straight to `gix::open`, so any
/// enrolled principal named any path on the server's filesystem.
///
/// Four legs, each observed RED before the repair. ⭐ Every leg drives a
/// publication id that does not exist, and that is deliberate: it proves the
/// path is refused BEFORE the publication is loaded, so an untrusted path never
/// reaches the database. The fifth leg is the positive arm the four refusals
/// need — an inside location gets PAST the containment check and fails on the
/// missing publication instead, which no "refuse everything" repair could do.
#[tokio::test]
async fn the_publish_verb_stays_inside_the_configured_repository_root() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };

    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/policy-publish-tests/containment");
    let root = fixture.join("root");
    let outside = fixture.join("outside");
    let _ = std::fs::remove_dir_all(&fixture);
    std::fs::create_dir_all(&root).expect("the root creates");
    std::fs::create_dir_all(&outside).expect("the outside directory creates");
    gix::init_bare(&outside).expect("the outside bare repository inits");
    std::os::unix::fs::symlink(&outside, root.join("escape")).expect("the symlink creates");
    let inside = root.join("live");
    std::fs::create_dir_all(&inside).expect("the inside directory creates");
    gix::init_bare(&inside).expect("the inside bare repository inits");

    let client = reqwest::Client::new();
    let configured = TestServer::start_with_publication_root(&pool, &root).await;
    let base = configured.base();
    let (status, human) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "pc-human" }),
    )
    .await;
    assert_eq!(status, 200, "the human enrolls: {human}");
    let human_id = human["principal_id"].as_str().unwrap().to_string();
    // `.9.2.1.2`: the verb is now behind a held authority, so every leg below
    // carries one — otherwise they would all be refused before the path is
    // ever looked at, and this control would stop measuring containment.
    let grant_id = format!("grt_{human_id}");

    let publish = |base: String, principal: String, grant: String, path: &'static str| {
        let client = client.clone();
        async move {
            post(
                &client,
                &base,
                "/v1/policy-publications/pc-absent/publish",
                &principal,
                &json!({ "repo_path": path, "owning_authority": grant }),
            )
            .await
        }
    };

    // Leg A — `..` walks out of the configured root.
    let (status, refused) = publish(
        base.clone(),
        human_id.clone(),
        grant_id.clone(),
        "../outside",
    )
    .await;
    assert_eq!(status, 400, "the `..` escape refuses: {refused}");
    assert_eq!(refused["code"], json!("invalid_command"), "{refused}");
    assert!(
        refused["message"]
            .as_str()
            .unwrap()
            .contains("outside the configured publication repository root"),
        "{refused}"
    );

    // Leg B — a SYMLINK inside the root walks out of it. Every component the
    // caller named is inside the root, so a string containment test admits it.
    let (status, refused) =
        publish(base.clone(), human_id.clone(), grant_id.clone(), "escape").await;
    assert_eq!(status, 400, "the symlink escape refuses: {refused}");
    assert!(
        refused["message"]
            .as_str()
            .unwrap()
            .contains("outside the configured publication repository root"),
        "{refused}"
    );

    // Leg C — an ABSOLUTE path outside the root, the shape the verb used to
    // accept verbatim.
    let (status, refused) = post(
        &client,
        &base,
        "/v1/policy-publications/pc-absent/publish",
        &human_id,
        &json!({ "repo_path": outside.to_string_lossy(), "owning_authority": grant_id }),
    )
    .await;
    assert_eq!(status, 400, "the absolute escape refuses: {refused}");
    assert!(
        refused["message"]
            .as_str()
            .unwrap()
            .contains("outside the configured publication repository root"),
        "{refused}"
    );

    // Leg D — the POSITIVE arm: an inside location passes containment and is
    // refused by the publication lookup instead. Without this the three
    // refusals above are equally consistent with a verb that refuses
    // everything.
    let (status, refused) = publish(base.clone(), human_id.clone(), grant_id.clone(), "live").await;
    assert_eq!(
        status, 400,
        "an inside location reaches the lookup: {refused}"
    );
    assert!(
        refused["message"]
            .as_str()
            .unwrap()
            .contains("does not exist"),
        "an inside location must be refused by the RECORD, not by containment: {refused}"
    );

    // Leg E — a server that declares NO root closes the verb outright, and
    // says so with a code a client cannot fix by retrying with another path.
    // ⭐ This is leg D's matched pair: the same request, against a server that
    // differs only in whether a root is declared. If the 503 came from
    // anywhere else, leg D would not answer 400.
    let unconfigured = TestServer::start(&pool).await;
    let (status, refused) = publish(
        unconfigured.base(),
        human_id.clone(),
        grant_id.clone(),
        "live",
    )
    .await;
    assert_eq!(status, 503, "an unconfigured deployment refuses: {refused}");
    assert_eq!(
        refused["code"],
        json!("publication_repository_unconfigured"),
        "{refused}"
    );

    // Leg F — THE FALSIFICATION of legs A to C, as a matched pair rather than
    // as prose: the SAME three locations, against a server whose declared root
    // is the directory ABOVE, legitimately contains them — and they are now
    // accepted, reaching the record exactly as leg D does. One knob changes.
    // If those refusals came from anything but containment, these would stay
    // refused. ⭐ Each is spelled absolutely, because a relative location is
    // joined to the root and so would not be the same location twice.
    let wider = TestServer::start_with_publication_root(&pool, &fixture).await;
    for (label, requested) in [
        ("the `..` walk", root.join("../outside")),
        ("the symlink", root.join("escape")),
        ("the absolute location", outside.clone()),
    ] {
        let (status, answered) = post(
            &client,
            &wider.base(),
            "/v1/policy-publications/pc-absent/publish",
            &human_id,
            &json!({ "repo_path": requested.to_string_lossy(), "owning_authority": grant_id }),
        )
        .await;
        assert_eq!(status, 400, "{label}: {answered}");
        assert!(
            answered["message"]
                .as_str()
                .unwrap()
                .contains("does not exist"),
            "{label} must reach the record once the declared root contains it: {answered}"
        );
    }

    let _ = std::fs::remove_dir_all(&fixture);
}

/// `SIGNOFF-REPAIR.9.2.1.2` — both publication verbs checked
/// `reader_tenant(…).is_some()` and nothing else, so enrolment in ANY tenant
/// was the whole predicate for writing a publication into a Git repository and
/// for declaring it effective.
///
/// Six legs over two tenants, on BOTH verbs. ⭐ The load-bearing pair is
/// "names a grant" against "holds a grant": grant ids here are derivable
/// (`grt_<principal_id>`), so a check that only asked whether an active grant
/// EXISTS would be no check at all — which is exactly what `.9.3.1` found
/// conflated on three other surfaces.
#[tokio::test]
async fn the_publication_verbs_require_an_authority_the_caller_holds() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };

    let (repo_root, object_ids) = seeded_publication_repository("authority");
    let server = TestServer::start_with_publication_root(&pool, &repo_root).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, alice) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "pa-alice" }),
    )
    .await;
    assert_eq!(status, 200, "alice enrolls: {alice}");
    let alice_id = alice["principal_id"].as_str().unwrap().to_string();
    let alice_grant = format!("grt_{alice_id}");

    let (status, bob) = enroll(&client, &base, json!({ "kind": "human", "name": "pa-bob" })).await;
    assert_eq!(status, 200, "bob enrolls: {bob}");
    let bob_id = bob["principal_id"].as_str().unwrap().to_string();
    let bob_grant = format!("grt_{bob_id}");
    assert_ne!(
        alice["tenant_id"], bob["tenant_id"],
        "the two enrolments mint distinct tenants"
    );

    // A STAGED publication, seeded directly. The full proposal -> decision ->
    // approval -> projection -> publication chain is already driven end to end
    // by `the_publish_verb_drives_the_git_half`; re-deriving it here would put
    // the thing under test — the authority binding — behind a second copy of
    // that pipeline (the shape `citing_an_authority_requires_holding_it` uses).
    sqlx::query(
        "INSERT INTO policy_publications \
         (publication_id, proposal_id, decision_id, approval_id, projection_id, state, manifest_digest) \
         VALUES ('pa-pub', 'pa-prp', 'pa-dec', 'pa-app', 'pa-proj', 'staged', $1)",
    )
    .bind(DIGEST)
    .execute(&pool)
    .await
    .expect("the staged publication seeds");

    let effective_body = |grant: &str| {
        json!({
            "git_object_ids": object_ids.clone(),
            "repo_path": "live",
            "owning_authority": grant,
        })
    };

    // Leg A — an enrolled principal naming NO authority is refused by both
    // verbs. Enrolment used to be the whole predicate.
    for (verb, path) in [
        ("publish", "/v1/policy-publications/pa-pub/publish"),
        ("effective", "/v1/policy-publications/pa-pub/effective"),
    ] {
        let (status, refused) = post(
            &client,
            &base,
            path,
            &alice_id,
            &json!({ "git_object_ids": object_ids.clone(), "repo_path": "live" }),
        )
        .await;
        assert_eq!(
            status, 400,
            "{verb} without an authority refuses: {refused}"
        );
        assert!(
            refused["message"]
                .as_str()
                .unwrap()
                .contains("owning_authority"),
            "{verb}: {refused}"
        );
    }

    // Leg B — naming an authority that is real, active and held by SOMEONE
    // ELSE is refused by both verbs. ⭐ This is the leg that distinguishes
    // holding from naming: `bob_grant` is derivable from bob's principal id,
    // which alice can read off any response.
    for (verb, path) in [
        ("publish", "/v1/policy-publications/pa-pub/publish"),
        ("effective", "/v1/policy-publications/pa-pub/effective"),
    ] {
        let (status, refused) =
            post(&client, &base, path, &alice_id, &effective_body(&bob_grant)).await;
        assert_eq!(
            status, 403,
            "{verb} under another principal's grant refuses: {refused}"
        );
        assert_eq!(refused["code"], json!("unauthorized"), "{verb}: {refused}");
        assert!(
            refused["message"].as_str().unwrap().contains("HOLDS"),
            "{verb}: {refused}"
        );
    }

    // Leg C — THE MATCHED PAIR for leg B: the same request, the same verb, the
    // same everything except WHOSE grant is named, now passes the gate. If the
    // refusals above came from anything but the holding check, this would be
    // refused too.
    //
    // `publish` gets past the gate and is stopped by the projection the seeded
    // row names, which does not exist — a LATER refusal, and therefore proof
    // that the authority check admitted it.
    let (status, admitted) = post(
        &client,
        &base,
        "/v1/policy-publications/pa-pub/publish",
        &alice_id,
        &effective_body(&alice_grant),
    )
    .await;
    assert_eq!(status, 400, "the holder is admitted: {admitted}");
    assert_eq!(
        admitted["code"],
        json!("invalid_command"),
        "the holder's refusal is not an authority refusal: {admitted}"
    );
    assert!(
        !admitted["message"].as_str().unwrap().contains("HOLDS"),
        "the holder is past the authority gate: {admitted}"
    );

    // And `effective` completes for the holder, so the positive arm is a real
    // success rather than only a different refusal.
    let (status, marked) = post(
        &client,
        &base,
        "/v1/policy-publications/pa-pub/effective",
        &alice_id,
        &effective_body(&alice_grant),
    )
    .await;
    assert_eq!(status, 200, "the holder marks effective: {marked}");
    assert_eq!(marked["state"], json!("effective"), "{marked}");

    let _ = std::fs::remove_dir_all(&repo_root);
}

#[tokio::test]
async fn the_deployment_rides_the_effective_publication_per_target() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    // `.9.2.1.3`: this fixture drives the effective transition, so it needs a
    // configured repository and ids READ BACK from it — it used to declare
    // `abc123`, which resolves to nothing.
    let (repo_root, object_ids) = seeded_publication_repository("deployment");
    let server = TestServer::start_with_publication_root(&pool, &repo_root).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, human) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "dp-human" }),
    )
    .await;
    assert_eq!(status, 200, "the human enrolls: {human}");
    let human_id = human["principal_id"].as_str().unwrap().to_string();
    let tenant_id = human["tenant_id"].as_str().unwrap().to_string();
    let grant_id = format!("grt_{human_id}");

    // The chain to the EFFECTIVE publication (the made-up object ids ride
    // the /effective verb — the git half is the `.4.3` lane's, already
    // proven).
    let (status, _) = post(
        &client,
        &base,
        "/v1/policies",
        &human_id,
        &json!({
            "policy_id": "dp-policy",
            "version": "1.0.0",
            "digest": DIGEST,
            "lifecycle": "draft",
            "title": "dp",
            "owning_authority": grant_id,
            "clauses": [ { "id": "c1", "statement": "the deployed clause" } ],
        }),
    )
    .await;
    assert_eq!(status, 200, "the policy registers");
    let (_status, created) = post(
        &client,
        &base,
        "/v1/threads",
        &human_id,
        &json!({
            "protocol_version": reasonbraid_core::PROTOCOL_VERSION,
            "operation": "thread.create",
            "request_id": reasonbraid_core::RequestId::new().to_string(),
            "idempotency_key": "dp-create",
            "body": {
                "tenant_id": tenant_id,
                "subject": "dp",
                "objective": "probe",
                "workflow_profile": "independent_panel",
            },
            "client_context": {},
        }),
    )
    .await;
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
    let (status, _) = command(
        "dp-advance",
        "thread.advance_round",
        json!({ "tenant_id": tenant_id }),
    )
    .await;
    assert_eq!(status, 200, "the round advances");
    let (_status, verdict) = command(
        "dp-verdict",
        "thread.contribute",
        json!({
            "tenant_id": tenant_id,
            "content": "judged",
            "kind": "verdict",
            "verdict": { "target_digest": "sha256:00", "rule": "majority", "outcome": "accepted_by_rule" },
        }),
    )
    .await;
    let verdict_event = verdict["event_id"].as_str().unwrap().to_string();

    let make_chain = |suffix: &'static str, publication_id: &'static str, effective: bool| {
        let client = client.clone();
        let base = base.clone();
        let human_id = human_id.clone();
        let thread_id = thread_id.clone();
        let verdict_event = verdict_event.clone();
        let grant_id = grant_id.clone();
        // `.9.2.1.3`: cloned here like every other capture, so the closure
        // stays `Fn` and can build more than one chain.
        let object_ids = object_ids.clone();
        async move {
            let proposal_id = format!("dp-prop-{suffix}");
            let decision_id = format!("dp-dec-{suffix}");
            let approval_id = format!("dp-app-{suffix}");
            let (status, _) = post(
                &client,
                &base,
                "/v1/policy-proposals",
                &human_id,
                &json!({
                    "proposal_id": proposal_id,
                    "policy_id": "dp-policy",
                    "policy_version": "1.0.0",
                    "thread_id": thread_id,
                }),
            )
            .await;
            assert_eq!(status, 200, "the proposal registers");
            let (status, _) = post(
                &client,
                &base,
                "/v1/policy-decisions",
                &human_id,
                &json!({
                    "decision_id": decision_id,
                    "proposal_id": proposal_id,
                    "rule": "majority",
                    "electorate": { "participants": [human_id], "denominator": 1, "abstentions": [] },
                    "verdict_event_id": verdict_event,
                }),
            )
            .await;
            assert_eq!(status, 200, "the decision records");
            let (status, _) = post(
                &client,
                &base,
                "/v1/policy-approvals",
                &human_id,
                &json!({
                    "approval_id": approval_id,
                    "proposal_id": proposal_id,
                    "decision_id": decision_id,
                    "approver": human_id,
                    "grant_id": grant_id,
                    "quorum": { "participants": [human_id], "denominator": 1, "abstentions": [] },
                }),
            )
            .await;
            assert_eq!(status, 200, "the approval records");
            let (status, projection) = post(
                &client,
                &base,
                "/v1/policy-projections",
                &human_id,
                &json!({
                    "projection_id": format!("{proposal_id}-proj"),
                    "target": "generic",
                    "resolution": {
                        "policies": [ { "policy_id": "dp-policy", "version": "1.0.0" } ],
                        "target": { "layer": "organization", "target": "*" },
                    },
                }),
            )
            .await;
            assert_eq!(status, 200, "the projection records");
            let projection_digest = projection["digest"].as_str().unwrap().to_string();
            let (status, _) = post(
                &client,
                &base,
                "/v1/policy-publications",
                &human_id,
                &json!({
                    "publication_id": publication_id,
                    "proposal_id": proposal_id,
                    "decision_id": decision_id,
                    "approval_id": approval_id,
                    "projection_id": format!("{proposal_id}-proj"),
                    "manifest_digest": projection_digest.clone(),
                }),
            )
            .await;
            assert_eq!(status, 200, "the publication stages");
            if effective {
                let (status, _) = post(
                    &client,
                    &base,
                    &format!("/v1/policy-publications/{publication_id}/effective"),
                    &human_id,
                    &json!({ "git_object_ids": object_ids.clone(), "repo_path": "live", "owning_authority": grant_id }),
                )
                .await;
                assert_eq!(status, 200, "the publication marks effective");
            }
            (proposal_id, projection_digest)
        }
    };
    let (_, projection_digest) = make_chain("1", "dp-pub-1", true).await;
    let _ = make_chain("2", "dp-pub-2", false).await;

    // 1. The target registers (the authority checked).
    let (status, _) = post(
        &client,
        &base,
        "/v1/deployment-targets",
        &human_id,
        &json!({
            "target_id": "dp-target",
            "target_type": "repository",
            "owning_authority": grant_id,
        }),
    )
    .await;
    assert_eq!(status, 200, "the target registers");
    let (status, refused) = post(
        &client,
        &base,
        "/v1/deployment-targets",
        &human_id,
        &json!({
            "target_id": "dp-ghost-authority",
            "target_type": "repository",
            "owning_authority": "grt_ghost",
        }),
    )
    .await;
    assert_eq!(status, 400, "the ghost authority refuses: {refused}");
    let (status, refused) = post(
        &client,
        &base,
        "/v1/deployment-targets",
        &human_id,
        &json!({
            "target_id": "dp-bad-type",
            "target_type": "not_a_type",
            "owning_authority": grant_id,
        }),
    )
    .await;
    assert_eq!(status, 400, "the unknown type refuses: {refused}");

    // 2. The assignment rides the EFFECTIVE publication (the desired pair).
    let (status, assignment) = post(
        &client,
        &base,
        "/v1/deployments",
        &human_id,
        &json!({
            "target_id": "dp-target",
            "publication_id": "dp-pub-1",
            "wave": 1,
            "desired_ref": "abc123",
            "desired_digest": projection_digest,
        }),
    )
    .await;
    assert_eq!(status, 200, "the assignment records: {assignment}");
    assert_eq!(assignment["observed_state"], json!("pending"));
    // The STAGED publication refuses (the chain gate).
    let (status, refused) = post(
        &client,
        &base,
        "/v1/deployments",
        &human_id,
        &json!({
            "target_id": "dp-target",
            "publication_id": "dp-pub-2",
            "wave": 1,
            "desired_ref": "zzz",
            "desired_digest": projection_digest,
        }),
    )
    .await;
    assert_eq!(status, 400, "the staged publication refuses: {refused}");
    assert!(
        refused["message"].as_str().unwrap().contains("effective"),
        "{refused}"
    );
    // The ghost target + the bad digest refuse.
    let (status, refused) = post(
        &client,
        &base,
        "/v1/deployments",
        &human_id,
        &json!({
            "target_id": "ghost-target",
            "publication_id": "dp-pub-1",
            "wave": 1,
            "desired_ref": "abc123",
            "desired_digest": projection_digest,
        }),
    )
    .await;
    assert_eq!(status, 400, "the ghost target refuses: {refused}");
    let (status, refused) = post(
        &client,
        &base,
        "/v1/deployments",
        &human_id,
        &json!({
            "target_id": "dp-target",
            "publication_id": "dp-pub-1",
            "wave": 2,
            "desired_ref": "abc123",
            "desired_digest": "not-a-digest",
        }),
    )
    .await;
    assert_eq!(status, 400, "the bad digest refuses: {refused}");

    // 3. The receipt attests the OBSERVED digest + the state.
    let (status, receipt) = post(
        &client,
        &base,
        "/v1/deployments/dp-target/dp-pub-1/receipt",
        &human_id,
        &json!({
            "observed_digest": projection_digest,
            "observed_state": "applied",
        }),
    )
    .await;
    assert_eq!(status, 200, "the receipt records: {receipt}");
    assert_eq!(receipt["observed_digest"], json!(projection_digest));
    assert_eq!(receipt["observed_state"], json!("applied"));
    let (status, refused) = post(
        &client,
        &base,
        "/v1/deployments/dp-target/dp-pub-1/receipt",
        &human_id,
        &json!({ "observed_digest": projection_digest, "observed_state": "vibes" }),
    )
    .await;
    assert_eq!(status, 400, "the unknown state refuses: {refused}");

    // 4. The list carries the desired/observed pair.
    let (status, deployments) = get(&client, &base, "/v1/deployments", &human_id).await;
    assert_eq!(status, 200, "the deployments read: {deployments}");
    let deployments = deployments.as_array().unwrap();
    assert_eq!(deployments.len(), 1, "{deployments:?}");
    assert_eq!(deployments[0]["desired_digest"], json!(projection_digest));
    assert_eq!(deployments[0]["observed_digest"], json!(projection_digest));
}

#[tokio::test]
async fn the_drift_corrections_and_outcomes_ride_the_records() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    // `.9.2.1.3`: this fixture drives the effective transition, so it needs a
    // configured repository and ids READ BACK from it — it used to declare
    // `abc123`, which resolves to nothing.
    let (repo_root, object_ids) = seeded_publication_repository("drift");
    let server = TestServer::start_with_publication_root(&pool, &repo_root).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, human) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "cr-human" }),
    )
    .await;
    assert_eq!(status, 200, "the human enrolls: {human}");
    let human_id = human["principal_id"].as_str().unwrap().to_string();
    let tenant_id = human["tenant_id"].as_str().unwrap().to_string();
    let grant_id = format!("grt_{human_id}");

    // The chain to the effective publication + the target + the assignment
    // (the same path the `.5.2` test drives).
    let (status, _) = post(
        &client,
        &base,
        "/v1/policies",
        &human_id,
        &json!({
            "policy_id": "cr-policy",
            "version": "1.0.0",
            "digest": DIGEST,
            "lifecycle": "draft",
            "title": "cr",
            "owning_authority": grant_id,
            "clauses": [ { "id": "c1", "statement": "the corrected clause" } ],
        }),
    )
    .await;
    assert_eq!(status, 200, "the policy registers");
    let (_status, created) = post(
        &client,
        &base,
        "/v1/threads",
        &human_id,
        &json!({
            "protocol_version": reasonbraid_core::PROTOCOL_VERSION,
            "operation": "thread.create",
            "request_id": reasonbraid_core::RequestId::new().to_string(),
            "idempotency_key": "cr-create",
            "body": {
                "tenant_id": tenant_id,
                "subject": "cr",
                "objective": "probe",
                "workflow_profile": "independent_panel",
            },
            "client_context": {},
        }),
    )
    .await;
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
    let (status, _) = command(
        "cr-advance",
        "thread.advance_round",
        json!({ "tenant_id": tenant_id }),
    )
    .await;
    assert_eq!(status, 200, "the round advances");
    let (_status, verdict) = command(
        "cr-verdict",
        "thread.contribute",
        json!({
            "tenant_id": tenant_id,
            "content": "judged",
            "kind": "verdict",
            "verdict": { "target_digest": "sha256:00", "rule": "majority", "outcome": "accepted_by_rule" },
        }),
    )
    .await;
    let verdict_event = verdict["event_id"].as_str().unwrap().to_string();
    for (publication_id, effective) in [("cr-pub-1", true), ("cr-pub-2", true)] {
        let proposal_id = format!("{publication_id}-prop");
        let decision_id = format!("{publication_id}-dec");
        let approval_id = format!("{publication_id}-app");
        let (status, _) = post(
            &client,
            &base,
            "/v1/policy-proposals",
            &human_id,
            &json!({
                "proposal_id": proposal_id,
                "policy_id": "cr-policy",
                "policy_version": "1.0.0",
                "thread_id": thread_id,
            }),
        )
        .await;
        assert_eq!(status, 200, "the proposal registers");
        let (status, _) = post(
            &client,
            &base,
            "/v1/policy-decisions",
            &human_id,
            &json!({
                "decision_id": decision_id,
                "proposal_id": proposal_id,
                "rule": "majority",
                "electorate": { "participants": [human_id], "denominator": 1, "abstentions": [] },
                "verdict_event_id": verdict_event,
            }),
        )
        .await;
        assert_eq!(status, 200, "the decision records");
        let (status, _) = post(
            &client,
            &base,
            "/v1/policy-approvals",
            &human_id,
            &json!({
                "approval_id": approval_id,
                "proposal_id": proposal_id,
                "decision_id": decision_id,
                "approver": human_id,
                "grant_id": grant_id,
                "quorum": { "participants": [human_id], "denominator": 1, "abstentions": [] },
            }),
        )
        .await;
        assert_eq!(status, 200, "the approval records");
        let (status, projection) = post(
            &client,
            &base,
            "/v1/policy-projections",
            &human_id,
            &json!({
                "projection_id": format!("{publication_id}-proj"),
                "target": "generic",
                "resolution": {
                    "policies": [ { "policy_id": "cr-policy", "version": "1.0.0" } ],
                    "target": { "layer": "organization", "target": "*" },
                },
            }),
        )
        .await;
        assert_eq!(status, 200, "the projection records");
        let (status, _) = post(
            &client,
            &base,
            "/v1/policy-publications",
            &human_id,
            &json!({
                "publication_id": publication_id,
                "proposal_id": proposal_id,
                "decision_id": decision_id,
                "approval_id": approval_id,
                "projection_id": format!("{publication_id}-proj"),
                "manifest_digest": projection["digest"],
            }),
        )
        .await;
        assert_eq!(status, 200, "the publication stages");
        if effective {
            let (status, _) = post(
                &client,
                &base,
                &format!("/v1/policy-publications/{publication_id}/effective"),
                &human_id,
                &json!({ "git_object_ids": object_ids.clone(), "repo_path": "live", "owning_authority": grant_id }),
            )
            .await;
            assert_eq!(status, 200, "the publication marks effective");
        }
    }
    let (status, _) = post(
        &client,
        &base,
        "/v1/deployment-targets",
        &human_id,
        &json!({ "target_id": "cr-target", "target_type": "repository", "owning_authority": grant_id }),
    )
    .await;
    assert_eq!(status, 200, "the target registers");
    let (status, _) = post(
        &client,
        &base,
        "/v1/deployments",
        &human_id,
        &json!({
            "target_id": "cr-target",
            "publication_id": "cr-pub-1",
            "wave": 1,
            "desired_ref": "abc123",
            "desired_digest": DIGEST,
        }),
    )
    .await;
    assert_eq!(status, 200, "the assignment records");

    // 1. The drift: the categorized pair.
    let (status, _) = post(
        &client,
        &base,
        "/v1/policy-drift",
        &human_id,
        &json!({
            "drift_id": "cr-drift-1",
            "target_id": "cr-target",
            "publication_id": "cr-pub-1",
            "category": "pending_rollout",
            "desired_digest": DIGEST,
            "observed_digest": null,
        }),
    )
    .await;
    assert_eq!(status, 200, "the drift records");
    let (status, refused) = post(
        &client,
        &base,
        "/v1/policy-drift",
        &human_id,
        &json!({
            "drift_id": "cr-drift-bad",
            "target_id": "cr-target",
            "publication_id": "cr-pub-1",
            "category": "vibes",
            "desired_digest": DIGEST,
            "observed_digest": null,
        }),
    )
    .await;
    assert_eq!(status, 400, "the unknown category refuses: {refused}");
    let (status, refused) = post(
        &client,
        &base,
        "/v1/policy-drift",
        &human_id,
        &json!({
            "drift_id": "cr-drift-ghost",
            "target_id": "ghost",
            "publication_id": "cr-pub-1",
            "category": "pending_rollout",
            "desired_digest": DIGEST,
            "observed_digest": null,
        }),
    )
    .await;
    assert_eq!(status, 400, "the ghost assignment refuses: {refused}");

    // 2. The corrections: the §4.7 operations with the authority proof.
    // The suspension REQUIRES the expiry (the expiring rule).
    let (status, refused) = post(
        &client,
        &base,
        "/v1/policy-corrections",
        &human_id,
        &json!({
            "correction_id": "cr-suspend-noexpiry",
            "publication_id": "cr-pub-1",
            "operation": "suspension",
            "authority_grant": grant_id,
            "reason": "the adverse outcome",
        }),
    )
    .await;
    assert_eq!(status, 400, "the expiry-less suspension refuses: {refused}");
    assert!(
        refused["message"].as_str().unwrap().contains("expires_at"),
        "{refused}"
    );
    let (status, suspended) = post(
        &client,
        &base,
        "/v1/policy-corrections",
        &human_id,
        &json!({
            "correction_id": "cr-suspend",
            "publication_id": "cr-pub-1",
            "operation": "suspension",
            "authority_grant": grant_id,
            "expires_at": "2026-09-15T00:00:00Z",
            "reason": "the adverse outcome",
        }),
    )
    .await;
    assert_eq!(status, 200, "the suspension records: {suspended}");

    // The RETRACTION preserves the original (the correction is a NEW row,
    // the publication stays readable).
    let (status, retracted) = post(
        &client,
        &base,
        "/v1/policy-corrections",
        &human_id,
        &json!({
            "correction_id": "cr-retract",
            "publication_id": "cr-pub-1",
            "operation": "retraction",
            "authority_grant": grant_id,
            "reason": "the owners withdrew",
            "remediation": "revert to the previous publication",
        }),
    )
    .await;
    assert_eq!(status, 200, "the retraction records: {retracted}");
    let (_status, publications) = get(&client, &base, "/v1/policy-publications", &human_id).await;
    assert!(
        publications
            .as_array()
            .unwrap()
            .iter()
            .any(|p| p["publication_id"] == json!("cr-pub-1")),
        "the original survives the retraction"
    );

    // The SUPERSESSION links the old/new.
    let (status, refused) = post(
        &client,
        &base,
        "/v1/policy-corrections",
        &human_id,
        &json!({
            "correction_id": "cr-supersede-nolink",
            "publication_id": "cr-pub-2",
            "operation": "supersession",
            "authority_grant": grant_id,
            "reason": "the replacement",
        }),
    )
    .await;
    assert_eq!(status, 400, "the link-less supersession refuses: {refused}");
    let (status, superseded) = post(
        &client,
        &base,
        "/v1/policy-corrections",
        &human_id,
        &json!({
            "correction_id": "cr-supersede",
            "publication_id": "cr-pub-2",
            "operation": "supersession",
            "authority_grant": grant_id,
            "supersedes": "cr-pub-1",
            "reason": "the replacement",
        }),
    )
    .await;
    assert_eq!(status, 200, "the supersession records: {superseded}");

    // The ghost authority refuses (the §4.7 proof).
    let (status, refused) = post(
        &client,
        &base,
        "/v1/policy-corrections",
        &human_id,
        &json!({
            "correction_id": "cr-ghost",
            "publication_id": "cr-pub-1",
            "operation": "retraction",
            "authority_grant": "grt_ghost",
            "reason": "nope",
        }),
    )
    .await;
    assert_eq!(status, 400, "the ghost authority refuses: {refused}");

    // 3. The outcomes: the §15.11 link.
    let (status, _) = post(
        &client,
        &base,
        "/v1/policy-outcomes",
        &human_id,
        &json!({
            "outcome_id": "cr-out-1",
            "publication_id": "cr-pub-1",
            "kind": "observation",
            "review_trigger": "drift",
            "note": "the target lagged the desired digest",
        }),
    )
    .await;
    assert_eq!(status, 200, "the outcome records");
    let (status, refused) = post(
        &client,
        &base,
        "/v1/policy-outcomes",
        &human_id,
        &json!({
            "outcome_id": "cr-out-bad",
            "publication_id": "cr-pub-1",
            "kind": "vibes",
            "note": "nope",
        }),
    )
    .await;
    assert_eq!(status, 400, "the unknown outcome kind refuses: {refused}");

    // 4. The lists.
    let (status, corrections) = get(&client, &base, "/v1/policy-corrections", &human_id).await;
    assert_eq!(status, 200, "the corrections read: {corrections}");
    assert_eq!(corrections.as_array().unwrap().len(), 3, "{corrections:?}");
    let (status, outcomes) = get(&client, &base, "/v1/policy-outcomes", &human_id).await;
    assert_eq!(status, 200, "the outcomes read: {outcomes}");
    assert_eq!(outcomes.as_array().unwrap().len(), 1, "{outcomes:?}");
}

#[tokio::test]
async fn the_scheduled_reviews_evaluate_the_triggers() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, human) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "rv-human" }),
    )
    .await;
    assert_eq!(status, 200, "the human enrolls: {human}");
    let human_id = human["principal_id"].as_str().unwrap().to_string();
    let tenant_id = human["tenant_id"].as_str().unwrap().to_string();
    let grant_id = format!("grt_{human_id}");

    // The chain to a STAGED publication (the outcomes only require the
    // existence).
    let (status, _) = post(
        &client,
        &base,
        "/v1/policies",
        &human_id,
        &json!({
            "policy_id": "rv-policy",
            "version": "1.0.0",
            "digest": DIGEST,
            "lifecycle": "draft",
            "title": "rv",
            "owning_authority": grant_id,
            "clauses": [ { "id": "c1", "statement": "the reviewed clause" } ],
        }),
    )
    .await;
    assert_eq!(status, 200, "the policy registers");
    let (_status, created) = post(
        &client,
        &base,
        "/v1/threads",
        &human_id,
        &json!({
            "protocol_version": reasonbraid_core::PROTOCOL_VERSION,
            "operation": "thread.create",
            "request_id": reasonbraid_core::RequestId::new().to_string(),
            "idempotency_key": "rv-create",
            "body": {
                "tenant_id": tenant_id,
                "subject": "rv",
                "objective": "probe",
                "workflow_profile": "independent_panel",
            },
            "client_context": {},
        }),
    )
    .await;
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
    let (status, _) = command(
        "rv-advance",
        "thread.advance_round",
        json!({ "tenant_id": tenant_id }),
    )
    .await;
    assert_eq!(status, 200, "the round advances");
    let (_status, verdict) = command(
        "rv-verdict",
        "thread.contribute",
        json!({
            "tenant_id": tenant_id,
            "content": "judged",
            "kind": "verdict",
            "verdict": { "target_digest": "sha256:00", "rule": "majority", "outcome": "accepted_by_rule" },
        }),
    )
    .await;
    let verdict_event = verdict["event_id"].as_str().unwrap().to_string();
    let proposal_id = "rv-prop";
    let (status, _) = post(
        &client,
        &base,
        "/v1/policy-proposals",
        &human_id,
        &json!({
            "proposal_id": proposal_id,
            "policy_id": "rv-policy",
            "policy_version": "1.0.0",
            "thread_id": thread_id,
        }),
    )
    .await;
    assert_eq!(status, 200, "the proposal registers");
    let (status, _) = post(
        &client,
        &base,
        "/v1/policy-decisions",
        &human_id,
        &json!({
            "decision_id": "rv-dec",
            "proposal_id": proposal_id,
            "rule": "majority",
            "electorate": { "participants": [human_id], "denominator": 1, "abstentions": [] },
            "verdict_event_id": verdict_event,
        }),
    )
    .await;
    assert_eq!(status, 200, "the decision records");
    let (status, _) = post(
        &client,
        &base,
        "/v1/policy-approvals",
        &human_id,
        &json!({
            "approval_id": "rv-app",
            "proposal_id": proposal_id,
            "decision_id": "rv-dec",
            "approver": human_id,
            "grant_id": grant_id,
            "quorum": { "participants": [human_id], "denominator": 1, "abstentions": [] },
        }),
    )
    .await;
    assert_eq!(status, 200, "the approval records");
    let (status, projection) = post(
        &client,
        &base,
        "/v1/policy-projections",
        &human_id,
        &json!({
            "projection_id": "rv-proj",
            "target": "generic",
            "resolution": {
                "policies": [ { "policy_id": "rv-policy", "version": "1.0.0" } ],
                "target": { "layer": "organization", "target": "*" },
            },
        }),
    )
    .await;
    assert_eq!(status, 200, "the projection records");
    let (status, _) = post(
        &client,
        &base,
        "/v1/policy-publications",
        &human_id,
        &json!({
            "publication_id": "rv-pub",
            "proposal_id": proposal_id,
            "decision_id": "rv-dec",
            "approval_id": "rv-app",
            "projection_id": "rv-proj",
            "manifest_digest": projection["digest"],
        }),
    )
    .await;
    assert_eq!(status, 200, "the publication stages");

    // 1. The outcomes + the waiver feed the triggers. The OUTCOME's
    // trigger rides the §15.11 vocabulary (the `.6` back-fill).
    let (status, _) = post(
        &client,
        &base,
        "/v1/policy-outcomes",
        &human_id,
        &json!({
            "outcome_id": "rv-out-1",
            "publication_id": "rv-pub",
            "kind": "observation",
            "review_trigger": "drift",
            "note": "the target lagged",
        }),
    )
    .await;
    assert_eq!(status, 200, "the outcome records");
    let (status, refused) = post(
        &client,
        &base,
        "/v1/policy-outcomes",
        &human_id,
        &json!({
            "outcome_id": "rv-out-bad",
            "publication_id": "rv-pub",
            "kind": "observation",
            "review_trigger": "vibes",
            "note": "nope",
        }),
    )
    .await;
    assert_eq!(status, 400, "the unknown trigger refuses: {refused}");
    let (status, _) = post(
        &client,
        &base,
        "/v1/policy-corrections",
        &human_id,
        &json!({
            "correction_id": "rv-waiver",
            "publication_id": "rv-pub",
            "operation": "waiver",
            "authority_grant": grant_id,
            "expires_at": "2026-09-15T00:00:00Z",
            "reason": "the bounded exception",
        }),
    )
    .await;
    assert_eq!(status, 200, "the waiver records");

    // 2. The schedule: the drift trigger (the outcome) + the
    // repeated_waiver trigger (the waiver) → two due reviews.
    let (status, scheduled) = post(
        &client,
        &base,
        "/v1/policy-reviews/schedule",
        &human_id,
        &json!({}),
    )
    .await;
    assert_eq!(status, 200, "the schedule evaluates: {scheduled}");
    let scheduled = scheduled.as_array().unwrap();
    assert_eq!(scheduled.len(), 2, "{scheduled:?}");
    let triggers: Vec<&str> = scheduled
        .iter()
        .map(|r| r["trigger"].as_str().unwrap())
        .collect();
    assert!(triggers.contains(&"drift"), "{triggers:?}");
    assert!(triggers.contains(&"repeated_waiver"), "{triggers:?}");

    // 3. The schedule is IDEMPOTENT (the dedupe: the second run adds
    // nothing).
    let (status, again) = post(
        &client,
        &base,
        "/v1/policy-reviews/schedule",
        &human_id,
        &json!({}),
    )
    .await;
    assert_eq!(status, 200, "the second schedule: {again}");
    assert_eq!(again.as_array().unwrap().len(), 0, "the dedupe holds");

    // 4. The done transition + the re-done refusal.
    let review_id = scheduled[0]["review_id"].as_str().unwrap().to_string();
    let (status, done) = post(
        &client,
        &base,
        &format!("/v1/policy-reviews/{review_id}/done"),
        &human_id,
        &json!({}),
    )
    .await;
    assert_eq!(status, 200, "the review marks done: {done}");
    assert_eq!(done["status"], json!("done"));
    let (status, refused) = post(
        &client,
        &base,
        &format!("/v1/policy-reviews/{review_id}/done"),
        &human_id,
        &json!({}),
    )
    .await;
    assert_eq!(status, 400, "the re-done refuses: {refused}");

    // 5. The list.
    let (status, reviews) = get(&client, &base, "/v1/policy-reviews", &human_id).await;
    assert_eq!(status, 200, "the reviews read: {reviews}");
    let reviews = reviews.as_array().unwrap();
    assert_eq!(reviews.len(), 2, "{reviews:?}");
    assert!(
        reviews.iter().any(|r| r["status"] == json!("done")),
        "the done review rides the list"
    );
}

/// The cited-authority control (`SIGNOFF-REPAIR.9.3.1`): citing a grant must
/// require HOLDING it, on every surface where a caller names one.
///
/// ⛔ The grant id is DERIVABLE, which is what makes this reachable rather
/// than theoretical: the dev enrolment mints `grt_<principal_id>`, so any
/// caller who has seen another principal's id can name that principal's
/// grant. The control derives Bob's exactly as a caller would.
///
/// ⛔ The `valid_from` leg is separate on purpose. Before the repair NO site
/// in the family consulted that column at all — `git grep -n valid_from` over
/// `corrections.rs`, `deployments.rs`, `policy.rs` and `lifecycle.rs`
/// returned rc=1 — so a grant that has not begun was as good as a live one.
#[tokio::test]
async fn citing_an_authority_requires_holding_it() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    // Two tenants, each with its own bootstrap human and its own dev grant.
    let (status, alice) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "cite-alice" }),
    )
    .await;
    assert_eq!(status, 200, "alice enrolls: {alice}");
    let alice_id = alice["principal_id"].as_str().unwrap().to_string();
    let alice_grant = format!("grt_{alice_id}");

    let (status, bob) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "cite-bob" }),
    )
    .await;
    assert_eq!(status, 200, "bob enrolls: {bob}");
    let bob_id = bob["principal_id"].as_str().unwrap().to_string();
    let bob_grant = format!("grt_{bob_id}");
    assert_ne!(
        alice["tenant_id"], bob["tenant_id"],
        "the two enrolments mint distinct tenants"
    );

    // A publication for the correction to name, owned by ALICE's own grant so
    // that nothing but the cited authority distinguishes the legs below.
    let (status, registered) = post(
        &client,
        &base,
        "/v1/policies",
        &alice_id,
        &json!({
            "policy_id": "cite-pol",
            "version": "1.0.0",
            "digest": DIGEST,
            "lifecycle": "active",
            "title": "the cited-authority control",
            "intent": "the grant binding",
            "domain": "deliberation",
            "risk_class": "low",
            "owning_authority": alice_grant,
            "clauses": [ { "id": "c1", "statement": "a cited authority is a held one" } ],
            "applicability": [ { "layer": "organization", "target": "*" } ],
            "exceptions": [],
        }),
    )
    .await;
    assert_eq!(status, 200, "the policy registers: {registered}");

    // The publication the correction names, seeded directly. The full
    // proposal -> decision -> approval -> projection -> publication chain is
    // already driven end to end by
    // `the_drift_corrections_and_outcomes_ride_the_records`; re-deriving it
    // here would add a second copy of that pipeline to maintain and would put
    // the thing under test — the authority binding — behind it.
    sqlx::query(
        "INSERT INTO policy_publications \
         (publication_id, proposal_id, decision_id, approval_id, projection_id, state, manifest_digest) \
         VALUES ('cite-pub-1', 'cite-prp', 'cite-dec', 'cite-app', 'cite-proj', 'published', $1)",
    )
    .bind(DIGEST)
    .execute(&pool)
    .await
    .expect("seed the publication the correction names");

    let mut breaches: Vec<String> = Vec::new();

    // Leg A — the correction surface: alice cites BOB's grant.
    let (status, body) = post(
        &client,
        &base,
        "/v1/policy-corrections",
        &alice_id,
        &json!({
            "correction_id": "cite-corr-foreign",
            "publication_id": "cite-pub-1",
            "operation": "retraction",
            "authority_grant": bob_grant,
            "reason": "recorded in an authority the caller does not hold",
        }),
    )
    .await;
    if status == 200 {
        breaches.push(format!(
            "A: alice recorded a correction in bob's authority: {body}"
        ));
    }

    // Leg B — the deployment surface: the same citation, the same caller.
    let (status, body) = post(
        &client,
        &base,
        "/v1/deployment-targets",
        &alice_id,
        &json!({
            "target_id": "cite-target-foreign",
            "target_type": "repository",
            "owning_authority": bob_grant,
        }),
    )
    .await;
    if status == 200 {
        breaches.push(format!(
            "B: alice registered a target owned by bob's authority: {body}"
        ));
    }

    // Leg C — `valid_from`: a grant that has NOT BEGUN is not a live one.
    // Alice's own grant is moved into the future, so the only thing separating
    // this leg from the passing leg D below is the column nothing consulted.
    sqlx::query(
        "UPDATE authority_grants SET valid_from = now() + interval '1 day' WHERE grant_id = $1",
    )
    .bind(&alice_grant)
    .execute(&pool)
    .await
    .expect("postdate alice's grant");
    let (status, body) = post(
        &client,
        &base,
        "/v1/policy-corrections",
        &alice_id,
        &json!({
            "correction_id": "cite-corr-notyet",
            "publication_id": "cite-pub-1",
            "operation": "retraction",
            "authority_grant": alice_grant,
            "reason": "recorded in an authority that has not begun",
        }),
    )
    .await;
    if status == 200 {
        breaches.push(format!(
            "C: alice recorded a correction in a grant that has not begun: {body}"
        ));
    }
    sqlx::query(
        "UPDATE authority_grants SET valid_from = now() - interval '1 hour' WHERE grant_id = $1",
    )
    .bind(&alice_grant)
    .execute(&pool)
    .await
    .expect("restore alice's grant");

    // Leg D — the legitimate correction still lands, with its record intact.
    // Without this the repair could be a blanket refusal and every leg above
    // would still pass.
    let (status, body) = post(
        &client,
        &base,
        "/v1/policy-corrections",
        &alice_id,
        &json!({
            "correction_id": "cite-corr-own",
            "publication_id": "cite-pub-1",
            "operation": "retraction",
            "authority_grant": alice_grant,
            "reason": "recorded in the caller's own authority",
        }),
    )
    .await;
    if status != 200 {
        breaches.push(format!("D: alice's OWN authority was refused: {body}"));
    }
    let (status, listed) = get(&client, &base, "/v1/policy-corrections", &alice_id).await;
    assert_eq!(status, 200, "the corrections list: {listed}");
    let recorded = listed
        .as_array()
        .map(|rows| {
            rows.iter()
                .any(|r| r["correction_id"] == json!("cite-corr-own"))
        })
        .unwrap_or(false);
    if !recorded {
        breaches.push(format!(
            "D: the legitimate correction left no row: {listed}"
        ));
    }
    let foreign_landed = listed
        .as_array()
        .map(|rows| {
            rows.iter().any(|r| {
                r["correction_id"] == json!("cite-corr-foreign")
                    || r["correction_id"] == json!("cite-corr-notyet")
            })
        })
        .unwrap_or(false);
    if foreign_landed {
        breaches.push(format!(
            "D: a refused correction left a row anyway: {listed}"
        ));
    }

    // Leg F — the APPROVAL surface, the third instance of the same mechanism
    // and the one the leaf did not name. It was the safest of the five sites
    // (it alone matched the grant to a subject) and still half-bound: the
    // subject it matched was `approver`, a string off the wire. So alice
    // approves AS bob, citing bob's grant, and nothing tied either to her.
    sqlx::query(
        "INSERT INTO policy_proposals (proposal_id, policy_id, policy_version, thread_id, status) \
         VALUES ('cite-prp-1', 'cite-pol', '1.0.0', 'cite-thread', 'decided')",
    )
    .execute(&pool)
    .await
    .expect("seed the decided proposal");
    sqlx::query(
        "INSERT INTO policy_decisions (decision_id, proposal_id, rule, electorate, verdict_event_id) \
         VALUES ('cite-dec-1', 'cite-prp-1', 'consensus', '[]'::jsonb, 'cite-evt')",
    )
    .execute(&pool)
    .await
    .expect("seed the decision");

    // ⛔ A SECOND proposal for the positive leg, deliberately. Sharing one
    // would couple the two: a successful foreign approval advances the
    // proposal to `approved`, and the legitimate approval that follows then
    // fails on the STAGE rather than on anything this leaf repairs. Measured
    // exactly that way against the unrepaired code, which is how the coupling
    // was found — a positive leg that can only pass when the negative leg
    // already did is measuring them jointly.
    sqlx::query(
        "INSERT INTO policy_proposals (proposal_id, policy_id, policy_version, thread_id, status) \
         VALUES ('cite-prp-2', 'cite-pol', '1.0.0', 'cite-thread', 'decided')",
    )
    .execute(&pool)
    .await
    .expect("seed the second decided proposal");
    sqlx::query(
        "INSERT INTO policy_decisions (decision_id, proposal_id, rule, electorate, verdict_event_id) \
         VALUES ('cite-dec-2', 'cite-prp-2', 'consensus', '[]'::jsonb, 'cite-evt-2')",
    )
    .execute(&pool)
    .await
    .expect("seed the second decision");

    // And a THIRD for leg G, for the same reason: a successful approval
    // advances its proposal, so two legs sharing one proposal report each
    // other's outcome. Invisible while the repair holds and immediately
    // visible under falsification — which is where it was found.
    sqlx::query(
        "INSERT INTO policy_proposals (proposal_id, policy_id, policy_version, thread_id, status) \
         VALUES ('cite-prp-3', 'cite-pol', '1.0.0', 'cite-thread', 'decided')",
    )
    .execute(&pool)
    .await
    .expect("seed the third decided proposal");
    sqlx::query(
        "INSERT INTO policy_decisions (decision_id, proposal_id, rule, electorate, verdict_event_id) \
         VALUES ('cite-dec-3', 'cite-prp-3', 'consensus', '[]'::jsonb, 'cite-evt-3')",
    )
    .execute(&pool)
    .await
    .expect("seed the third decision");

    let (status, body) = post(
        &client,
        &base,
        "/v1/policy-approvals",
        &alice_id,
        &json!({
            "approval_id": "cite-app-foreign",
            "proposal_id": "cite-prp-1",
            "decision_id": "cite-dec-1",
            "approver": bob_id,
            "grant_id": bob_grant,
            "quorum": { "participants": [bob_id] },
        }),
    )
    .await;
    if status == 200 {
        breaches.push(format!("F: alice recorded an approval AS bob: {body}"));
    }

    // Leg G — alice cites her OWN grant and claims BOB as the approver.
    //
    // ⚠️ This leg was NOT red against the unrepaired code, and saying so is
    // the point of it. The original predicate bound the grant to the CLAIMED
    // APPROVER (`subject_id = $2`), so this exact request was refused — while
    // the request leg F makes, where both the grant and the claim are bob's,
    // sailed through. The repair moves the binding to the AUTHENTICATED
    // CALLER, which closes F; this leg guards the link that move needs in its
    // place, because grant-to-caller alone would let a caller file an
    // approval under her own authority and attribute it to somebody else.
    let (status, body) = post(
        &client,
        &base,
        "/v1/policy-approvals",
        &alice_id,
        &json!({
            "approval_id": "cite-app-misattributed",
            "proposal_id": "cite-prp-3",
            "decision_id": "cite-dec-3",
            "approver": bob_id,
            "grant_id": alice_grant,
            "quorum": { "participants": [alice_id] },
        }),
    )
    .await;
    if status == 200 {
        breaches.push(format!(
            "G: alice filed an approval under her own grant and attributed it to bob: {body}"
        ));
    }

    // And alice approving as HERSELF, with her own grant, still lands.
    let (status, body) = post(
        &client,
        &base,
        "/v1/policy-approvals",
        &alice_id,
        &json!({
            "approval_id": "cite-app-own",
            "proposal_id": "cite-prp-2",
            "decision_id": "cite-dec-2",
            "approver": alice_id,
            "grant_id": alice_grant,
            "quorum": { "participants": [alice_id] },
        }),
    )
    .await;
    if status != 200 {
        breaches.push(format!("F: alice's OWN approval was refused: {body}"));
    }

    // Leg E — the legitimate target still registers.
    let (status, body) = post(
        &client,
        &base,
        "/v1/deployment-targets",
        &alice_id,
        &json!({
            "target_id": "cite-target-own",
            "target_type": "repository",
            "owning_authority": alice_grant,
        }),
    )
    .await;
    if status != 200 {
        breaches.push(format!("E: alice's OWN authority was refused: {body}"));
    }

    assert!(
        breaches.is_empty(),
        "{} of the cited-authority legs breach:\n{}",
        breaches.len(),
        breaches.join("\n")
    );
}
