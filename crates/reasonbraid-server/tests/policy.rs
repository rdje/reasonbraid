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

#[path = "support/site.rs"]
mod site_fixture;

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
            // `SIGNOFF-REPAIR.6.1.5.4`: registering a policy version is a site act,
            // so this suite now writes the site trail. Ahead of the policy tables
            // because the audit row outlives the act it records.
            "site_audit",
            "governance_charters",
            "policy_reviews",
            "policy_outcomes",
            "policy_corrections",
            "policy_drift",
            "deployment_receipts",
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
            "node_inbox_cursors",
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
            "node_ambiguous_attempts",
            "node_proof_nonces",
            "nodes",
            "hosts",
            "recruitment_panels",
            "recruitment_responses",
            "recruitment_offers",
            "recruitment_calls",
            "card_imports",
            "agent_roles",
            "human_principals",
            "evidence_citations",
            "claim_assessments",
            "derivations",
            "evidence_snapshots",
            // `SIGNOFF-REPAIR.11.36`: the malformed-body control posts to
            // `/v1/snapshots`. Refused before any write, but the plan purges what
            // the suite's routes can reach, not what one request happened to do.
            "snapshot_objects",
            "reference_registrations",
            "resource_references",
            "quota_events",
            "usage_quotas",
            "federation_agreements",
            "cross_domain_receipts",
            "mcp_listen_state",
            "tenant_bootstrap_requests",
            "storm_refusals",
            "resolution_refusals",
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

/// `POST /v1/policies` — a SITE act since `SIGNOFF-REPAIR.6.1.5.4`, so every
/// body carries the reason every site act carries.
///
/// ⭐ One helper rather than the same wire detail restated at twenty-six
/// fixtures, and it adds NOTHING else: it does not issue the capability, so a
/// caller that holds no `policy_register` grant is refused here exactly as it is
/// in production. A fixture that means to register seeds the grant explicitly
/// through `site_fixture::provision`, in its own body, where a reader can see
/// it. That the reason is REQUIRED — not merely accepted — is asserted by
/// `the_policy_library_takes_site_operator_authority`, which posts without one.
async fn register_policy(
    client: &reqwest::Client,
    base: &str,
    principal: &str,
    body: &Value,
) -> (u16, Value) {
    let mut body = body.clone();
    if body.get("reason").is_none() {
        body["reason"] = json!("the fixture registers a policy version");
    }
    post(client, base, "/v1/policies", principal, &body).await
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

/// The audited reason a registration is refused with when its owning authority
/// is not a live, covering grant the registrar holds (`SIGNOFF-REPAIR.9.1.2`).
/// One text for a ghost grant and for another principal's grant, so a registrar
/// learns nothing about grants it does not hold.
const OWNING_AUTHORITY_REFUSAL: &str =
    "the named owning authority is not a live grant the caller holds that covers policy_version_register";

/// `SIGNOFF-REPAIR.9.2.1.3`: a publication repository under a configured root,
/// and object ids READ BACK from it.
///
/// Three fixtures used to declare `["abc123", "def456"]`, which resolve to
/// nothing — the suite meant to qualify the effective transition was proving it
/// with ids that do not exist. ⛔ They are RE-SEEDED rather than relaxed or
/// deleted: each still asserts the same transition, and now asserts it
/// honestly. Returns the root to configure and the ids to declare.
fn seeded_publication_repository(name: &str) -> (std::path::PathBuf, Vec<String>) {
    // The root follows the PROCESS, not the build (§12): a compile-time
    // CARGO_MANIFEST_DIR names the checkout this binary was BUILT in
    // (SIGNOFF-REPAIR.11.2.1.2.2.1).
    let root = reasonbraid_core::repository_root()
        .expect("the tests run inside the repository")
        .join("target/policy-publish-tests")
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
    // `SIGNOFF-REPAIR.6.1.5.4`: registering a policy version is a site act.
    // This fixture seeds the governance library, so it holds the capability —
    // issued through the deployment-controlled service, never by a row insert.
    site_fixture::provision(
        &pool,
        &human_id,
        &[reasonbraid_server::site_authority::Action::PolicyRegister],
    )
    .await;
    // The dev enrollment's grant: `grt_<principal>` (the Phase-2 model) — the
    // policy's owning authority references it.
    let grant_id = format!("grt_{human_id}");

    let policy = |policy_id: &str, version: &str| {
        json!({
            "policy_id": policy_id,
            "version": version,
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
    let (status, registered) =
        register_policy(&client, &base, &human_id, &policy("org-baseline", "1.0.0")).await;
    assert_eq!(status, 200, "the policy registers: {registered}");
    assert_eq!(registered["policy_id"], json!("org-baseline"));
    assert_eq!(registered["version"], json!("1.0.0"));
    // `SIGNOFF-REPAIR.9.1.3`: the digest is the server's, derived from the
    // document; `expected_policy_digest` re-derives it by a second route.
    assert_eq!(
        registered["digest"],
        json!(expected_policy_digest(&policy("org-baseline", "1.0.0"))),
        "{registered}"
    );
    assert_eq!(registered["owning_authority"], json!(grant_id));
    assert_eq!(registered["clauses"].as_array().unwrap().len(), 2);

    // 2. A NEW version of the same policy registers (the versioned registry).
    let (status, upgraded) =
        register_policy(&client, &base, &human_id, &policy("org-baseline", "1.1.0")).await;
    assert_eq!(status, 200, "the new version registers: {upgraded}");

    // 3. The refusals: the duplicate version, the malformed digest, the
    // malformed semver, the unknown lifecycle, the duplicate clause ids, the
    // empty clauses, and the GHOST owning authority (the label grants
    // nothing).
    let (status, refused) =
        register_policy(&client, &base, &human_id, &policy("org-baseline", "1.0.0")).await;
    // ⚠️ **400 AGAIN SINCE `SIGNOFF-REPAIR.16`, WITH THE AUDIT ID BESIDE IT.**
    // `.6.1.5.4` moved this to 403 and its reason — *answering it before the
    // gate would hand a caller with no site authority an existence oracle* —
    // is correct about ORDER and does not reach RENDERING. Authority is still
    // checked first, so nothing leaks; but a caller who PASSED that check was
    // then told *a current site grant … is required* about a grant they hold,
    // and was counted in `authorization_denials`. ⛔ The order is unchanged;
    // only what an authorized caller is told changed back.
    assert_eq!(status, 400, "the duplicate refuses: {refused}");
    assert!(
        refused["audit_id"].is_string(),
        "and the domain refusal is still audited: {refused}"
    );
    assert_eq!(
        refused["code"],
        json!("that policy version is already registered"),
        "{refused}"
    );
    assert!(
        refused["audit_id"]
            .as_str()
            .is_some_and(|id| !id.is_empty()),
        "a refused site act still records why it was refused: {refused}"
    );

    let (status, refused) = register_policy(
        &client,
        &base,
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

    let (status, refused) = register_policy(
        &client,
        &base,
        &human_id,
        &json!({
            "policy_id": "bad-version",
            "version": "not.a.version",
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

    let (status, refused) = register_policy(
        &client,
        &base,
        &human_id,
        &json!({
            "policy_id": "bad-lifecycle",
            "version": "1.0.0",
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

    let (status, refused) = register_policy(
        &client,
        &base,
        &human_id,
        &json!({
            "policy_id": "dup-clauses",
            "version": "1.0.0",
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

    let (status, refused) = register_policy(
        &client,
        &base,
        &human_id,
        &json!({
            "policy_id": "no-clauses",
            "version": "1.0.0",
            "lifecycle": "draft",
            "title": "x",
            "owning_authority": grant_id,
            "clauses": [],
        }),
    )
    .await;
    assert_eq!(status, 400, "the empty clauses refuse: {refused}");

    let (status, refused) = register_policy(
        &client,
        &base,
        &human_id,
        &json!({
            "policy_id": "ghost-authority",
            "version": "1.0.0",
            "lifecycle": "draft",
            "title": "x",
            "owning_authority": "grt_ghost",
            "clauses": [ { "id": "c1", "statement": "s" } ],
        }),
    )
    .await;
    // ⚠️ **400 again since `SIGNOFF-REPAIR.16`**, and this is the sharper case,
    // so it is worth being explicit: whether `grt_ghost` names a live grant IS
    // an existence question about the site's own grants, and `.6.1.5.4` was
    // right that the pre-gate 400 answered anyone. ⭐ The gate still runs first
    // and still refuses an unauthorized caller with 403 — so the oracle stays
    // closed — and only a caller who already holds site authority reaches this
    // answer. Telling THEM 400 discloses nothing they could not already read.
    assert_eq!(status, 400, "the ghost authority refuses: {refused}");
    assert!(refused["audit_id"].is_string(), "{refused}");
    assert_eq!(
        refused["code"],
        json!(OWNING_AUTHORITY_REFUSAL),
        "the audit trail tells the two domain refusals apart: {refused}"
    );

    // 4. The list: the two registered rows, newest first (the refusals
    // persisted nothing).
    let (status, policies) = get(&client, &base, "/v1/policies", &human_id).await;
    assert_eq!(status, 200, "the policies list: {policies}");
    let policies = policies.as_array().unwrap();
    assert_eq!(policies.len(), 2, "{policies:?}");
    assert_eq!(policies[0]["version"], json!("1.1.0"), "newest first");

    // 5. A malformed principal is refused at the header, before anything else.
    // ⚠️ Since `.6.1.5.4` this leg proves the HEADER contract and no longer the
    // enrolment gate, which the site gate replaced: a WELL-FORMED principal that
    // is merely enrolled is now refused 403, and that is asserted where it
    // belongs, in `the_policy_library_takes_site_operator_authority`.
    let (status, _) = register_policy(&client, &base, "hpr_ghost", &policy("ghost", "1.0.0")).await;
    assert_eq!(status, 401, "a malformed principal refuses at the header");
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
    // `SIGNOFF-REPAIR.6.1.5.4`: registering a policy version is a site act.
    // This fixture seeds the governance library, so it holds the capability —
    // issued through the deployment-controlled service, never by a row insert.
    site_fixture::provision(
        &pool,
        &human_id,
        &[reasonbraid_server::site_authority::Action::PolicyRegister],
    )
    .await;
    let grant_id = format!("grt_{human_id}");

    let register = |policy: Value| {
        let client = client.clone();
        let base = base.clone();
        let human_id = human_id.clone();
        async move { register_policy(&client, &base, &human_id, &policy).await }
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
    // `SIGNOFF-REPAIR.6.1.5.4`: registering a policy version is a site act.
    // This fixture seeds the governance library, so it holds the capability —
    // issued through the deployment-controlled service, never by a row insert.
    site_fixture::provision(
        &pool,
        &human_id,
        &[reasonbraid_server::site_authority::Action::PolicyRegister],
    )
    .await;
    let tenant_id = human["tenant_id"].as_str().unwrap().to_string();
    allow_owner_decides(&pool, &tenant_id).await;
    let grant_id = format!("grt_{human_id}");

    // The policy the proposal targets.
    let (status, _) = register_policy(
        &client,
        &base,
        &human_id,
        &json!({
            "policy_id": "lc-policy",
            "version": "1.0.0",
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
                "decision_rule": "owner_decides",
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
    // `SIGNOFF-REPAIR.8.1.1.3`: this fixture used to record `rule: "unanimity"`
    // with `accepted_unanimously` — ONE adjudicator claiming a unanimous count
    // on a thread that declares `owner_decides`. Re-derived, not edited to
    // pass: the claim is now the refusal, and the verdict this lifecycle needs
    // as supporting evidence names only what it judges.
    let (status, refused) = command(
        "lc-verdict-count",
        "thread.contribute",
        json!({
            "tenant_id": tenant_id,
            "content": "the panel judged",
            "kind": "verdict",
            "verdict": { "target_digest": "sha256:00", "outcome": "accepted_unanimously" },
        }),
    )
    .await;
    assert_eq!(status, 400, "a verdict cannot claim a count: {refused}");
    assert!(
        refused["message"]
            .as_str()
            .unwrap()
            .contains("names a count of ballots"),
        "{refused}"
    );
    let (status, refused) = command(
        "lc-verdict-rule",
        "thread.contribute",
        json!({
            "tenant_id": tenant_id,
            "content": "the panel judged",
            "kind": "verdict",
            "verdict": { "target_digest": "sha256:00", "rule": "unanimity", "outcome": "accepted_by_rule" },
        }),
    )
    .await;
    assert_eq!(
        status, 400,
        "the rule is no longer the adjudicator's to state: {refused}"
    );
    let (status, verdict) = command(
        "lc-verdict",
        "thread.contribute",
        json!({
            "tenant_id": tenant_id,
            "content": "the panel judged",
            "kind": "verdict",
            "verdict": { "target_digest": "sha256:00", "outcome": "accepted_by_rule" },
        }),
    )
    .await;
    assert_eq!(status, 200, "the verdict contributes: {verdict}");
    let verdict_event = verdict["event_id"].as_str().unwrap().to_string();
    // The rule it applies is the THREAD's, derived by the server.
    let recorded: Value = sqlx::query_scalar("SELECT body FROM event_log WHERE event_id = $1")
        .bind(&verdict_event)
        .fetch_one(&pool)
        .await
        .expect("the verdict event");
    assert_eq!(
        recorded["verdict"]["rule"],
        json!("owner_decides"),
        "{recorded}"
    );
    close_as_owner(&client, &base, &human_id, &tenant_id, &thread_id).await;

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
            "rule": "owner_decides",
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
    // `SIGNOFF-REPAIR.11.4.7.2.1.2.3.1`: the stored rule is the THREAD's, and
    // the record says what it was derived from.
    assert_eq!(decision["rule"], json!("owner_decides"));
    assert_eq!(
        decision["derivation"]["outcome"],
        json!("accepted_by_rule"),
        "{decision}"
    );
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
            "rule": "owner_decides",
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
                "decision_rule": "owner_decides",
            },
            "client_context": {},
        }),
    )
    .await;
    let other_thread = created["thread_id"].as_str().unwrap().to_string();
    close_as_owner(&client, &base, &human_id, &tenant_id, &other_thread).await;
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
            "rule": "owner_decides",
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
            "rule": "owner_decides",
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
    // `SIGNOFF-REPAIR.6.1.5.4`: registering a policy version is a site act.
    // This fixture seeds the governance library, so it holds the capability —
    // issued through the deployment-controlled service, never by a row insert.
    site_fixture::provision(
        &pool,
        &human_id,
        &[reasonbraid_server::site_authority::Action::PolicyRegister],
    )
    .await;
    let tenant_id = human["tenant_id"].as_str().unwrap().to_string();
    allow_owner_decides(&pool, &tenant_id).await;
    let grant_id = format!("grt_{human_id}");

    // The chain: the policy → the thread + the verdict → the proposal →
    // the decision.
    let (status, _) = register_policy(
        &client,
        &base,
        &human_id,
        &json!({
            "policy_id": "ap-policy",
            "version": "1.0.0",
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
                "decision_rule": "owner_decides",
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
            "verdict": { "target_digest": "sha256:00", "outcome": "accepted_by_rule" },
        }),
    )
    .await;
    let verdict_event = verdict["event_id"].as_str().unwrap().to_string();
    close_as_owner(&client, &base, &human_id, &tenant_id, &thread_id).await;

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
                    "rule": "owner_decides",
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
    // `SIGNOFF-REPAIR.6.1.5.4`: registering a policy version is a site act.
    // This fixture seeds the governance library, so it holds the capability —
    // issued through the deployment-controlled service, never by a row insert.
    site_fixture::provision(
        &pool,
        &human_id,
        &[reasonbraid_server::site_authority::Action::PolicyRegister],
    )
    .await;
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
        let (status, registered) = register_policy(
            &client,
            &base,
            &human_id,
            &json!({
                "policy_id": policy_id,
                "version": "1.0.0",
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
        }),
    )
    .await;
    assert_eq!(status, 200, "the lock projects: {locked}");
    // `SIGNOFF-REPAIR.9.1.4`: the rows are the SERVER'S, read from the
    // registry; this request no longer supplies them.
    let (status, library) = get(&client, &base, "/v1/policies", &human_id).await;
    assert_eq!(status, 200, "the library answers: {library}");
    let base_digest = library
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["policy_id"] == json!("pj-base"))
        .and_then(|p| p["digest"].as_str())
        .expect("pj-base is registered")
        .to_string();
    assert!(
        locked["bytes"]
            .as_str()
            .unwrap()
            .contains(&format!("pj-base 1.0.0 {base_digest} {grant_id}")),
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
    // `SIGNOFF-REPAIR.6.1.5.4`: registering a policy version is a site act.
    // This fixture seeds the governance library, so it holds the capability —
    // issued through the deployment-controlled service, never by a row insert.
    site_fixture::provision(
        &pool,
        &human_id,
        &[reasonbraid_server::site_authority::Action::PolicyRegister],
    )
    .await;
    let grant_id = format!("grt_{human_id}");

    let (status, _) = register_policy(
        &client,
        &base,
        &human_id,
        &json!({
            "policy_id": "cc-policy",
            "version": "1.0.0",
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
    // `SIGNOFF-REPAIR.6.1.5.4`: registering a policy version is a site act.
    // This fixture seeds the governance library, so it holds the capability —
    // issued through the deployment-controlled service, never by a row insert.
    site_fixture::provision(
        &pool,
        &human_id,
        &[reasonbraid_server::site_authority::Action::PolicyRegister],
    )
    .await;
    let tenant_id = human["tenant_id"].as_str().unwrap().to_string();
    allow_owner_decides(&pool, &tenant_id).await;
    let grant_id = format!("grt_{human_id}");

    // The chain: the policy → the thread + the verdict → the proposal →
    // the decision → the approval → the projection.
    let (status, _) = register_policy(
        &client,
        &base,
        &human_id,
        &json!({
            "policy_id": "pb-policy",
            "version": "1.0.0",
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
                "decision_rule": "owner_decides",
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
            "verdict": { "target_digest": "sha256:00", "outcome": "accepted_by_rule" },
        }),
    )
    .await;
    let verdict_event = verdict["event_id"].as_str().unwrap().to_string();
    close_as_owner(&client, &base, &human_id, &tenant_id, &thread_id).await;

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
            "rule": "owner_decides",
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
    let projection_digest = projection["digest"].as_str().unwrap().to_string();

    // `.9.2.1.3.1`: the manifest digest this control INDEPENDENTLY derives.
    // ⛔ Recomputed here rather than read from the server, and deliberately not
    // by calling the production helper: a control that asks the code under test
    // what the answer is passes for an unrelated reason
    // (`docs/knowledge/a-control-that-passes-for-an-unrelated-reason.md`).
    let derived_digest = |publication_id: &str| -> String {
        let manifest = serde_json::to_string(&json!({
            "publication_id": publication_id,
            "proposal_id": "pb-prop",
            "decision_id": "pb-dec",
            "approval_id": "pb-app",
            "projection_id": "pb-proj",
            "projection_digest": projection_digest,
        }))
        .expect("the manifest serializes");
        use sha2::Digest as _;
        format!("sha256:{:x}", sha2::Sha256::digest(manifest.as_bytes()))
    };

    // 1. The publication stages (the references verified, the staged
    // state). ⛔ `.9.2.1.3.1`: the body no longer carries `manifest_digest`.
    // It used to send the PROJECTION's digest — which is not a manifest digest
    // and was stored, unread, as though it were.
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
            // `.9.2.1.2.2`: staging names a grant the caller holds, like its
            // three transitions.
            "owning_authority": grant_id,
        }),
    )
    .await;
    assert_eq!(status, 200, "the publication stages: {publication}");
    assert_eq!(publication["state"], json!("staged"));

    // 1a. `.9.2.1.3.1` — THE STORED DIGEST IS THE SERVER'S. ADR-020 §15.7
    // steps (2)-(4) make the manifest digest the server's product of compiling
    // and hashing the manifest, not an input to the transaction that stores
    // the row. It used to be whatever the caller typed, shape-checked and read
    // by nothing.
    assert_eq!(
        publication["manifest_digest"].as_str().unwrap(),
        derived_digest("pb-pub"),
        "the staged row carries the digest of its OWN manifest: {publication}"
    );
    // ⭐ AND THE STORED COLUMN, not only the answer. The first version of this
    // leg asserted the RESPONSE alone, and the falsification run caught it: a
    // neutralization that bound a different value into the INSERT while the
    // response kept the derived one left this assertion green and was found
    // two verbs later. The row is what `publish` reads, so the row is what
    // this leg must read.
    let stored: String = sqlx::query_scalar(
        "SELECT manifest_digest FROM policy_publications WHERE publication_id = 'pb-pub'",
    )
    .fetch_one(&pool)
    .await
    .expect("the staged row reads back");
    assert_eq!(
        stored,
        derived_digest("pb-pub"),
        "the COLUMN carries the derived digest, not merely the response"
    );
    assert_ne!(
        publication["manifest_digest"].as_str().unwrap(),
        projection_digest,
        "the manifest digest is not the projection digest — that confusion is \
         what this leaf repaired: {publication}"
    );

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
            "owning_authority": grant_id,
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
            "owning_authority": grant_id,
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
            "owning_authority": grant_id,
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
            "rule": "owner_decides",
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
            "owning_authority": grant_id,
        }),
    )
    .await;
    assert_eq!(status, 200, "the second publication stages");
    let (status, failed) = post(
        &client,
        &base,
        "/v1/policy-publications/pb-pub-2/failed",
        &human_id,
        // `.9.2.1.2.1`: `failed` now takes a held authority like its two
        // siblings, so the walk carries one rather than being refused before
        // it reaches the transition it means to measure.
        &json!({ "reason": "the fetch-back verification failed",
                 "owning_authority": grant_id }),
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

    // 6a. `.9.2.1.3.1` — A SUPPLIED digest is an ASSERTION checked against the
    // derivation —
    // the shape `expected_effective` already uses on the publish verb. The
    // projection digest, which every caller used to send, is refused by name.
    let (status, refused) = post(
        &client,
        &base,
        "/v1/policy-publications",
        &human_id,
        &json!({
            "publication_id": "pb-pub-asserted",
            "proposal_id": "pb-prop",
            "decision_id": "pb-dec",
            "approval_id": "pb-app",
            "projection_id": "pb-proj",
            "owning_authority": grant_id,
            "manifest_digest": projection_digest,
        }),
    )
    .await;
    assert_eq!(
        status, 400,
        "a manifest digest that does not describe the manifest refuses: {refused}"
    );
    assert!(
        refused["message"]
            .as_str()
            .unwrap()
            .contains("manifest digest"),
        "{refused}"
    );

    // 6b. THE MATCHED PAIR: the same request with the digest the server would
    // derive is ADMITTED. Without this leg, a repair that refused every
    // supplied digest would pass 1b.
    let (status, asserted) = post(
        &client,
        &base,
        "/v1/policy-publications",
        &human_id,
        &json!({
            "publication_id": "pb-pub-asserted",
            "proposal_id": "pb-prop",
            "decision_id": "pb-dec",
            "approval_id": "pb-app",
            "projection_id": "pb-proj",
            "owning_authority": grant_id,
            "manifest_digest": derived_digest("pb-pub-asserted"),
        }),
    )
    .await;
    assert_eq!(status, 200, "the correct assertion is admitted: {asserted}");
    assert_eq!(
        asserted["manifest_digest"].as_str().unwrap(),
        derived_digest("pb-pub-asserted"),
        "{asserted}"
    );

    // 7. `SIGNOFF-REPAIR.9.2.2`: the two TERMINAL transitions race. Each used to
    // read the stage, test it in Rust, then UPDATE with no stage predicate, so
    // both passed the check and the second overwrote the first — measured, a
    // publication answered EFFECTIVE while carrying the failed transition's
    // reason — and both callers were told they had won. A transaction holds the
    // row; the FIRST verb is sent and observed queued on it, then the SECOND
    // (observed in `pg_stat_activity`, never assumed from a sleep), so the row is
    // granted in that order when it is released. Both orders run, one per staged
    // publication, so each verb's own stage check is what refuses the loser.
    let (status, staged) = post(
        &client,
        &base,
        "/v1/policy-publications",
        &human_id,
        &json!({
            "publication_id": "pb-pub-race",
            "proposal_id": "pb-prop",
            "decision_id": "pb-dec",
            "approval_id": "pb-app",
            "projection_id": "pb-proj",
            "owning_authority": grant_id,
        }),
    )
    .await;
    assert_eq!(
        status, 200,
        "a second staged publication for the race: {staged}"
    );
    let body_for = |verb: &str| match verb {
        "effective" => json!({
            "git_object_ids": object_ids.clone(), "repo_path": "live", "owning_authority": grant_id,
        }),
        _ => json!({ "reason": "the race's other side", "owning_authority": grant_id }),
    };
    for (publication, first, second) in [
        ("pb-pub-asserted", "failed", "effective"),
        ("pb-pub-race", "effective", "failed"),
    ] {
        let mut holder = pool.begin().await.expect("begin the holder");
        sqlx::query("SELECT 1 FROM policy_publications WHERE publication_id = $1 FOR UPDATE")
            .bind(publication)
            .execute(&mut *holder)
            .await
            .expect("hold the publication row");
        let holder_pid: i32 = sqlx::query_scalar("SELECT pg_backend_pid()")
            .fetch_one(&mut *holder)
            .await
            .expect("the holder's backend");
        let send = |verb: &str| {
            let client = client.clone();
            let base = base.clone();
            let human_id = human_id.clone();
            let path = format!("/v1/policy-publications/{publication}/{verb}");
            let body = body_for(verb);
            tokio::spawn(async move { post(&client, &base, &path, &human_id, &body).await })
        };
        // ⚠️ Counted by the table, not by `pg_blocking_pids(pid) ∋ holder`: the
        // second waiter queues behind the FIRST (PostgreSQL's tuple-lock queue),
        // so only one of them names the holder as its blocker.
        let queued = |expected: i64| {
            let pool = pool.clone();
            async move {
                let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(10);
                loop {
                    let queued: i64 = sqlx::query_scalar(
                        "SELECT count(*) FROM pg_stat_activity \
                         WHERE datname = current_database() AND wait_event_type = 'Lock' \
                           AND query LIKE '%policy_publications%' AND pid <> $1",
                    )
                    .bind(holder_pid)
                    .fetch_one(&pool)
                    .await
                    .expect("pg_stat_activity");
                    if queued == expected {
                        return;
                    }
                    assert!(
                        tokio::time::Instant::now() < deadline,
                        "{expected} transition(s) queue on the held row"
                    );
                    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                }
            }
        };
        let first_request = send(first);
        queued(1).await;
        let second_request = send(second);
        queued(2).await;
        holder.commit().await.expect("release the row");
        let won = first_request.await.expect("the first request");
        let lost = second_request.await.expect("the second request");
        assert_eq!(
            won.0, 200,
            "{publication}: `{first}`, queued first, wins: {won:?}"
        );
        assert_eq!(won.1["state"], json!(first), "{publication}: {won:?}");
        assert_eq!(
            lost.0, 400,
            "{publication}: `{second}` is refused: {lost:?}"
        );
        assert!(
            lost.1["message"]
                .as_str()
                .unwrap_or_default()
                .contains(&format!("is at stage `{first}`")),
            "{publication}: the refusal names the stage the winner left: {lost:?}"
        );
        let stored: (String, Option<String>) = sqlx::query_as(
            "SELECT state, failed_reason FROM policy_publications WHERE publication_id = $1",
        )
        .bind(publication)
        .fetch_one(&pool)
        .await
        .expect("the stored stage");
        assert_eq!(
            stored.0, first,
            "{publication}: the row holds the winner's stage"
        );
        assert_eq!(
            stored.1.is_some(),
            first == "failed",
            "{publication}: a failure reason only on a failed row: {stored:?}"
        );
    }
}

#[tokio::test]
async fn the_publish_verb_drives_the_git_half() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    // `.9.2.1.1`: the server declares WHERE publications may be written, and
    // the request names a location inside it. This control is re-pointed at a
    // configured root rather than deleted — it still asserts the Git half.
    // Derived at RUNTIME (§12) (SIGNOFF-REPAIR.11.2.1.2.2.1).
    let repo_root = reasonbraid_core::repository_root()
        .expect("the tests run inside the repository")
        .join("target/policy-publish-tests");
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
    // `SIGNOFF-REPAIR.6.1.5.4`: registering a policy version is a site act.
    // This fixture seeds the governance library, so it holds the capability —
    // issued through the deployment-controlled service, never by a row insert.
    site_fixture::provision(
        &pool,
        &human_id,
        &[reasonbraid_server::site_authority::Action::PolicyRegister],
    )
    .await;
    let tenant_id = human["tenant_id"].as_str().unwrap().to_string();
    allow_owner_decides(&pool, &tenant_id).await;
    let grant_id = format!("grt_{human_id}");

    // The chain: the policy → the thread + the verdict → the proposal →
    // the decision → the approval → the projection → the staged publication.
    let (status, _) = register_policy(
        &client,
        &base,
        &human_id,
        &json!({
            "policy_id": "pu-policy",
            "version": "1.0.0",
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
                "decision_rule": "owner_decides",
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
            "verdict": { "target_digest": "sha256:00", "outcome": "accepted_by_rule" },
        }),
    )
    .await;
    let verdict_event = verdict["event_id"].as_str().unwrap().to_string();
    close_as_owner(&client, &base, &human_id, &tenant_id, &thread_id).await;
    for (proposal_id, decision_id, approval_id, publication_id) in [
        ("pu-prop", "pu-dec", "pu-app", "pu-pub"),
        // `SIGNOFF-REPAIR.9.3.5.1.1`: a second publication, stranded on
        // purpose below — only the reconciler may recover it.
        ("pu-prop-2", "pu-dec-2", "pu-app-2", "pu-pub-2"),
    ] {
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
                "rule": "owner_decides",
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
        // ⛔ `.9.2.1.3.1`: the staging body below no longer sends
        // `manifest_digest`, so this response is no longer read — it used to
        // supply `projection["digest"]`, which is not a manifest digest, and
        // the row stored it unread.
        let (status, _projection) = post(
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
                "owning_authority": grant_id,
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

    // 1a. `.9.2.1.3.1` — THE PUBLISH VERB VERIFIES THE MANIFEST IT IS ABOUT TO
    // WRITE against the digest staging derived. Before this leaf the stored
    // digest had no reader at all: `publish` composed its own manifest and
    // never consulted the row.
    //
    // ⭐ The tamper is REAL, not fabricated: the projection's digest column is
    // moved after the publication was staged, which is precisely the case a
    // stored digest exists to catch — the compiled inputs changing under a
    // staged publication. It is restored afterwards, so leg 2 still measures
    // the Git half and not this leg's leftovers.
    let true_projection_digest: String = sqlx::query_scalar(
        "SELECT digest FROM policy_projections WHERE projection_id = 'pu-prop-proj'",
    )
    .fetch_one(&pool)
    .await
    .expect("the projection digest reads");
    sqlx::query("UPDATE policy_projections SET digest = $2 WHERE projection_id = $1")
        .bind("pu-prop-proj")
        .bind(format!("sha256:{}", "b".repeat(64)))
        .execute(&pool)
        .await
        .expect("the projection digest moves under the staged publication");
    let (status, refused) = post(
        &client,
        &base,
        "/v1/policy-publications/pu-pub/publish",
        &human_id,
        &json!({ "repo_path": "live", "owning_authority": grant_id }),
    )
    .await;
    assert_eq!(
        status, 400,
        "a manifest that disagrees with the staged digest refuses: {refused}"
    );
    assert!(
        refused["message"]
            .as_str()
            .unwrap()
            .contains("manifest digest"),
        "the refusal names the digest comparison, not the repository: {refused}"
    );
    // ⛔ NOTHING WAS WRITTEN. A refusal that had already published and then
    // complained would satisfy the status assertion above and be the worse
    // outcome, so the state is re-read rather than assumed.
    let (status, still) = get(&client, &base, "/v1/policy-publications", &human_id).await;
    assert_eq!(status, 200, "{still}");
    assert_eq!(
        still[0]["state"],
        json!("staged"),
        "the refused publish left the publication staged: {still}"
    );
    sqlx::query("UPDATE policy_projections SET digest = $2 WHERE projection_id = $1")
        .bind("pu-prop-proj")
        .bind(&true_projection_digest)
        .execute(&pool)
        .await
        .expect("the projection digest is restored");

    // `SIGNOFF-REPAIR.9.3.5.1.1`: the refused publishes above wrote nothing,
    // so they recorded no Git operation either.
    let recorded = |listing: &Value, id: &str| {
        listing
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["publication_id"] == json!(id))
            .cloned()
            .unwrap_or_else(|| panic!("{id} is listed: {listing}"))
    };
    let (_, listing) = get(&client, &base, "/v1/policy-publications", &human_id).await;
    assert_eq!(
        recorded(&listing, "pu-pub")["repository"],
        Value::Null,
        "a publish refused before any write records no operation: {listing}"
    );

    // 1b. `SIGNOFF-REPAIR.9.3.5.1.1` — A PUBLISH THAT DIES AFTER ITS GIT WRITE
    // LEAVES A ROW NAMING WHERE IT WROTE. The effective channel does not exist
    // yet, so an `expected_effective` fails the compare-and-swap AFTER the
    // commit and the immutable ref are written — a real interruption between
    // the Git write and `mark_effective`, not a constructed one.
    let bogus_effective = "1".repeat(40);
    let (status, refused) = post(
        &client,
        &base,
        "/v1/policy-publications/pu-pub-2/publish",
        &human_id,
        &json!({ "repo_path": "live", "owning_authority": grant_id,
                 "expected_effective": bogus_effective }),
    )
    .await;
    assert_eq!(status, 400, "the stale CAS refuses: {refused}");
    let (_, listing) = get(&client, &base, "/v1/policy-publications", &human_id).await;
    let stranded = recorded(&listing, "pu-pub-2");
    assert_eq!(stranded["state"], json!("staged"), "{stranded}");
    assert_eq!(
        stranded["repository"],
        json!("live"),
        "the row names the repository, ROOT-RELATIVE: {stranded}"
    );
    assert_eq!(
        stranded["expected_effective"],
        json!(bogus_effective),
        "{stranded}"
    );
    {
        let repo = gix::open(repo_root.join("live")).expect("the repository opens");
        assert!(
            repo.find_reference("refs/rb/publications/pu-pub-2").is_ok(),
            "the Git write really happened before the failure"
        );
    }
    // A retry that asks for a DIFFERENT operation is refused, not re-pointed.
    let (status, refused) = post(
        &client,
        &base,
        "/v1/policy-publications/pu-pub-2/publish",
        &human_id,
        &json!({ "repo_path": "live", "owning_authority": grant_id }),
    )
    .await;
    assert_eq!(status, 400, "{refused}");
    assert!(
        refused["message"]
            .as_str()
            .unwrap()
            .contains("already recorded its Git operation"),
        "{refused}"
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
    assert_eq!(published["repository"], json!("live"), "{published}");

    // 2a. `SIGNOFF-REPAIR.9.3.5.2` — THE BUNDLE IS READABLE BY ITS MANIFEST
    // DIGEST, AND VERIFIED ON THE WAY OUT.
    let digest = published["manifest_digest"].as_str().unwrap().to_string();
    let bundle_path = format!("/v1/policy-bundles/{digest}");
    let (status, served) = get(&client, &base, &bundle_path, &human_id).await;
    assert_eq!(status, 200, "the bundle serves: {served}");
    let projection = reasonbraid_server::projections::load(&pool, "pu-prop-proj")
        .await
        .expect("the projection loads");
    assert_eq!(
        served["bundle"],
        json!(projection.bytes),
        "the bytes that were published"
    );
    assert_eq!(served["publication_id"], json!("pu-pub"));
    let true_manifest = served["manifest"].as_str().unwrap().to_string();
    let (status, v) = get(
        &client,
        &base,
        &format!("/v1/policy-bundles/sha256:{}", "e".repeat(64)),
        &human_id,
    )
    .await;
    assert_eq!(status, 404, "an unpublished digest: {v}");
    let (status, v) = get(&client, &base, "/v1/policy-bundles/not-a-digest", &human_id).await;
    assert_eq!(status, 400, "a malformed digest: {v}");
    let (_, outsider) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "pu-outsider" }),
    )
    .await;
    let (status, v) = get(
        &client,
        &base,
        &bundle_path,
        outsider["principal_id"].as_str().unwrap(),
    )
    .await;
    assert_eq!(
        status, 404,
        "another tenant is answered as if it were absent: {v}"
    );

    // THE CONTROL: the STORED content is tampered — the immutable ref is
    // re-pointed at a commit carrying another manifest, then at one carrying
    // the true manifest and another bundle. Both are refused, not served.
    let staged_at = reasonbraid_server::publications::load(&pool, "pu-pub")
        .await
        .unwrap()
        .staged_at_seconds;
    let point_at = |commit: gix::ObjectId| {
        let repo = gix::open(&repo_dir).expect("the repository opens");
        repo.edit_reference(gix::refs::transaction::RefEdit {
            change: gix::refs::transaction::Change::Update {
                log: Default::default(),
                expected: gix::refs::transaction::PreviousValue::Any,
                new: gix::refs::Target::Object(commit),
            },
            name: "refs/rb/publications/pu-pub".try_into().unwrap(),
            deref: false,
        })
        .expect("the ref is re-pointed");
    };
    let original: gix::ObjectId = published["git_object_ids"][0]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();
    for (manifest, bundle, file) in [
        ("{}", projection.bytes.as_str(), "manifest.json"),
        (true_manifest.as_str(), "# a tampered bundle", "bundle.txt"),
    ] {
        let forged = reasonbraid_server::publisher::expected_commit(
            &repo_dir, "pu-pub", manifest, bundle, staged_at,
        )
        .expect("the forged commit writes");
        point_at(forged);
        let (status, refused) = get(&client, &base, &bundle_path, &human_id).await;
        assert_eq!(status, 409, "tampered `{file}` is refused: {refused}");
        assert_eq!(refused["code"], json!("publication_conflict"), "{refused}");
        assert!(
            refused["message"].as_str().unwrap().contains(file),
            "the refusal names the file that failed: {refused}"
        );
    }
    point_at(original);
    let (status, _) = get(&client, &base, &bundle_path, &human_id).await;
    assert_eq!(status, 200, "the restored ref serves again");
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

    // Derived at RUNTIME (§12) (SIGNOFF-REPAIR.11.2.1.2.2.1).
    let fixture = reasonbraid_core::repository_root()
        .expect("the tests run inside the repository")
        .join("target/policy-publish-tests/containment");
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
            .contains("publication `pc-absent` does not exist"),
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
                .contains("publication `pc-absent` does not exist"),
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
    // ⛔ `tenant_id` is seeded too, because `.6.1.5.2` makes the real `stage`
    // store it and `.6.1.5.2.1` makes every later verb require it. A publication
    // with no owner is advanced by NOBODY, so a fixture that omitted it would
    // refuse alice as loudly as it refuses bob and this control would pass for
    // the wrong reason.
    sqlx::query(
        "INSERT INTO policy_publications \
         (publication_id, proposal_id, decision_id, approval_id, projection_id, state, \
          manifest_digest, tenant_id) \
         VALUES ('pa-pub', 'pa-prp', 'pa-dec', 'pa-app', 'pa-proj', 'staged', $1, $2)",
    )
    .bind(DIGEST)
    .bind(alice["tenant_id"].as_str().unwrap())
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
    // ⛔ `.9.2.1.2.3`: `publish` takes no `git_object_ids`, and since the verbs
    // are typed it REFUSES one rather than ignoring it. The two verbs
    // therefore need two bodies — which is the contract being honest rather
    // than an inconvenience: one of these requests was always nonsense for one
    // of the two verbs, and nothing said so.
    let publish_body = |grant: &str| {
        json!({
            "repo_path": "live",
            "owning_authority": grant,
        })
    };

    // Leg A — an enrolled principal naming NO authority is refused by both
    // verbs. Enrolment used to be the whole predicate.
    // ⛔ `.9.2.1.2.3` typed both verbs, so each leg sends THAT VERB'S shape:
    // `publish` does not accept `git_object_ids` and now says so rather than
    // ignoring it. The refusal is the strict wire boundary's `422`, naming the
    // field that is missing.
    for (verb, path, body) in [
        (
            "publish",
            "/v1/policy-publications/pa-pub/publish",
            json!({ "repo_path": "live" }),
        ),
        (
            "effective",
            "/v1/policy-publications/pa-pub/effective",
            json!({ "git_object_ids": object_ids.clone(), "repo_path": "live" }),
        ),
    ] {
        let (status, refused) = post(&client, &base, path, &alice_id, &body).await;
        assert_eq!(
            status, 422,
            "{verb} without an authority refuses at the boundary: {refused}"
        );
        assert!(
            format!("{refused}").contains("owning_authority"),
            "{verb}: {refused}"
        );
    }

    // Leg B — naming an authority that is real, active and held by SOMEONE
    // ELSE is refused by both verbs. ⭐ This is the leg that distinguishes
    // holding from naming: `bob_grant` is derivable from bob's principal id,
    // which alice can read off any response.
    for (verb, path, body) in [
        (
            "publish",
            "/v1/policy-publications/pa-pub/publish",
            publish_body(&bob_grant),
        ),
        (
            "effective",
            "/v1/policy-publications/pa-pub/effective",
            effective_body(&bob_grant),
        ),
    ] {
        let (status, refused) = post(&client, &base, path, &alice_id, &body).await;
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
        &publish_body(&alice_grant),
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

/// `SIGNOFF-REPAIR.9.2.1.2.1` — the THIRD publication transition took no
/// authority at all.
///
/// `.9.2.1.2` bound `publish` and `effective` to a grant the caller HOLDS and
/// left `failed` behind on enrolment plus tenant ownership — the very
/// predicate it had just removed from the other two. Its title says "both
/// publish verbs" and its reproduce line names those two by name, so `failed`
/// was never in that leaf's population.
///
/// ⛔ `staged → failed` is TERMINAL: `mark_effective` and `publish` each refuse
/// a publication whose state is not `staged`, so marking one failed is how a
/// governance publication is permanently taken off the table. The cheapest
/// destructive act on this surface was the unbound one.
///
/// ⭐ The three legs mirror `the_publication_verbs_require_an_authority_the_caller_holds`
/// on purpose — same questions, same seeded shape — so the two controls read
/// against each other and no later change can move one verb's binding without
/// the difference being visible.
#[tokio::test]
async fn the_failed_transition_requires_an_authority_the_caller_holds() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };

    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, alice) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "pf-alice" }),
    )
    .await;
    assert_eq!(status, 200, "alice enrolls: {alice}");
    let alice_id = alice["principal_id"].as_str().unwrap().to_string();
    let alice_grant = format!("grt_{alice_id}");

    let (status, bob) = enroll(&client, &base, json!({ "kind": "human", "name": "pf-bob" })).await;
    assert_eq!(status, 200, "bob enrolls: {bob}");
    let bob_id = bob["principal_id"].as_str().unwrap().to_string();
    let bob_grant = format!("grt_{bob_id}");
    assert_ne!(
        alice["tenant_id"], bob["tenant_id"],
        "the two enrolments mint distinct tenants"
    );

    // ⛔ `tenant_id` is seeded, for the reason the sibling control records: a
    // publication with no owner is advanced by NOBODY, so a fixture that
    // omitted it would refuse alice as loudly as it refuses bob and this
    // control would pass for the wrong reason.
    sqlx::query(
        "INSERT INTO policy_publications \
         (publication_id, proposal_id, decision_id, approval_id, projection_id, state, \
          manifest_digest, tenant_id) \
         VALUES ('pf-pub', 'pf-prp', 'pf-dec', 'pf-app', 'pf-proj', 'staged', $1, $2)",
    )
    .bind(DIGEST)
    .bind(alice["tenant_id"].as_str().unwrap())
    .execute(&pool)
    .await
    .expect("the staged publication seeds");

    let staged = |label: &'static str| {
        let pool = pool.clone();
        async move {
            let state: String = sqlx::query_scalar(
                "SELECT state FROM policy_publications WHERE publication_id = 'pf-pub'",
            )
            .fetch_one(&pool)
            .await
            .expect("the publication reads back");
            assert_eq!(
                state, "staged",
                "{label}: a refused transition must leave the row untouched"
            );
        }
    };

    // Leg A — an enrolled principal naming NO authority is refused. Enrolment
    // plus tenant ownership used to be the whole predicate.
    let (status, refused) = post(
        &client,
        &base,
        "/v1/policy-publications/pf-pub/failed",
        &alice_id,
        &json!({ "reason": "unauthorized failure" }),
    )
    .await;
    // ⛔ `422` since `.9.2.1.2.3` typed this verb: a missing required field is
    // the strict wire boundary's answer, not the handler's. The transition is
    // still not reached, which is what leg A is for.
    assert_eq!(
        status, 422,
        "failed without an authority refuses at the boundary: {refused}"
    );
    assert!(
        format!("{refused}").contains("owning_authority"),
        "{refused}"
    );
    staged("leg A").await;

    // Leg B — naming an authority that is real, active and held by SOMEONE
    // ELSE is refused. ⭐ The leg that distinguishes holding from naming:
    // `bob_grant` is derivable from bob's principal id, which alice reads off
    // any response.
    let (status, refused) = post(
        &client,
        &base,
        "/v1/policy-publications/pf-pub/failed",
        &alice_id,
        &json!({ "reason": "borrowed failure", "owning_authority": bob_grant }),
    )
    .await;
    assert_eq!(
        status, 403,
        "failed under another principal's grant refuses: {refused}"
    );
    assert_eq!(refused["code"], json!("unauthorized"), "{refused}");
    assert!(
        refused["message"].as_str().unwrap().contains("HOLDS"),
        "{refused}"
    );
    staged("leg B").await;

    // Leg C — THE MATCHED PAIR for leg B: the same request, the same verb, the
    // same everything except WHOSE grant is named, and the transition
    // completes. If either refusal above came from anything but the holding
    // check, this would be refused too.
    let (status, failed) = post(
        &client,
        &base,
        "/v1/policy-publications/pf-pub/failed",
        &alice_id,
        &json!({ "reason": "the holder's failure", "owning_authority": alice_grant }),
    )
    .await;
    assert_eq!(status, 200, "the holder marks failed: {failed}");
    assert_eq!(failed["state"], json!("failed"), "{failed}");
    assert_eq!(
        failed["failed_reason"],
        json!("the holder's failure"),
        "{failed}"
    );
}

/// `SIGNOFF-REPAIR.9.3.4.2` — HOLDING IS NOT COVERING, at all five
/// administrative surfaces.
///
/// `.9.3.1` and `.9.2.1.2` bound each surface to a grant the caller HOLDS.
/// `.9.3.4.1` made the narrower verbs expressible. Neither made a surface ask
/// whether the held grant's actions COVER the verb being attempted — so a
/// grant minted for one administrative purpose still carried every other one,
/// which is the gap `.9.3.4` opened on.
///
/// ⭐ THREE ARMS PER SURFACE, and the third is what makes the other two mean
/// something: a grant carrying an unrelated action is REFUSED, the same grant
/// carrying that surface's own action is ADMITTED, and the same grant
/// carrying only `tenant_admin` is ADMITTED TOO — the subsumption `.9.3.4.1`
/// chose as the disposition for every boundary stored before those names
/// existed, exercised end to end rather than only in a unit test.
///
/// ⛔ Every arm drives FRESH ids. Four of the five surfaces are one-shot (a
/// staged publication is spent, a target id is unique, a correction and an
/// approval are their own rows), so reusing ids would make arm 3 fail on a
/// duplicate and read as a coverage refusal.
#[tokio::test]
async fn a_held_grant_must_cover_the_administrative_verb_it_is_cited_for() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };

    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, human) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "cv-human" }),
    )
    .await;
    assert_eq!(status, 200, "the human enrolls: {human}");
    let human_id = human["principal_id"].as_str().unwrap().to_string();
    let tenant_id = human["tenant_id"].as_str().unwrap().to_string();
    let grant_id = format!("grt_{human_id}");
    site_fixture::provision(
        &pool,
        &human_id,
        &[reasonbraid_server::site_authority::Action::PolicyRegister],
    )
    .await;

    // ── The fixtures, seeded BEFORE the grant is narrowed ────────────────────
    // ⛔ A PROPOSAL PER ARM. An approval advances its proposal `decided →
    // approved`, so a second approval against one proposal is refused for a
    // STAGE reason — which would read as a coverage refusal and make arm 3
    // pass for the wrong reason. Found by the suite, not predicted.
    for arm in ["a", "b", "c"] {
        sqlx::query(
            "INSERT INTO policy_proposals \
             (proposal_id, policy_id, policy_version, thread_id, status, tenant_id) \
             VALUES ($2, 'cv-policy', '1.0.0', 'cv-thread', 'decided', $1)",
        )
        .bind(&tenant_id)
        .bind(format!("cv-prp-{arm}"))
        .execute(&pool)
        .await
        .expect("the proposal seeds");
        // `SIGNOFF-REPAIR.11.4.7.2.1.2.3.2`: an approval copies its decision's
        // DERIVED quorum, so the seeded decision carries a derivation and an
        // electorate EQUAL to the quorum the approval asserts — leaving the
        // authority check as the only thing that can refuse it.
        sqlx::query(
            "INSERT INTO policy_decisions \
             (decision_id, proposal_id, rule, electorate, verdict_event_id, tenant_id, derivation) \
             VALUES ($2, $3, 'owner_decides', $4, 'cv-evt', $1, '{\"fixture\": \"authority\"}'::jsonb)",
        )
        .bind(&tenant_id)
        .bind(format!("cv-dec-{arm}"))
        .bind(format!("cv-prp-{arm}"))
        .bind(json!({ "participants": [human_id], "denominator": 1, "abstentions": [] }))
        .execute(&pool)
        .await
        .expect("the decision seeds");
    }
    sqlx::query(
        "INSERT INTO aggregate_state \
         (tenant_id, aggregate_id, aggregate_type, aggregate_version, state) \
         VALUES ($1, 'cv-thread', 'thread', 1, '{}'::jsonb)",
    )
    .bind(&tenant_id)
    .execute(&pool)
    .await
    .expect("the thread aggregate seeds");
    for arm in ["a", "b", "c"] {
        sqlx::query(
            "INSERT INTO policy_publications \
             (publication_id, proposal_id, decision_id, approval_id, projection_id, state, \
              manifest_digest, tenant_id, owning_authority) \
             VALUES ($4, $5, $6, 'cv-app', 'cv-proj', 'staged', $1, $2, $3)",
        )
        .bind(DIGEST)
        .bind(&tenant_id)
        .bind(&grant_id)
        .bind(format!("cv-pub-{arm}"))
        .bind(format!("cv-prp-{arm}"))
        .bind(format!("cv-dec-{arm}"))
        .execute(&pool)
        .await
        .expect("the staged publication seeds");
    }

    // Narrow or widen the ONE grant every leg cites. ⭐ One column, one knob:
    // every other input is identical across the arms, so a difference in
    // outcome can only be the coverage check.
    let set_actions = |names: &'static str| {
        let pool = pool.clone();
        let grant_id = grant_id.clone();
        async move {
            sqlx::query("UPDATE authority_grants SET actions = $2::jsonb WHERE grant_id = $1")
                .bind(&grant_id)
                .bind(names)
                .execute(&pool)
                .await
                .expect("the grant's actions are set");
        }
    };

    // Each surface, parameterised by arm so every attempt drives fresh ids.
    let surfaces = |arm: &str| -> Vec<(&'static str, String, serde_json::Value)> {
        vec![
            (
                "publication",
                format!("/v1/policy-publications/cv-pub-{arm}/failed"),
                json!({ "owning_authority": grant_id, "reason": "r" }),
            ),
            (
                "deployment target",
                "/v1/deployment-targets".to_string(),
                json!({ "target_id": format!("cv-target-{arm}"),
                        "target_type": "repository", "owning_authority": grant_id,
                        "reporter": human_id }),
            ),
            (
                "correction",
                "/v1/policy-corrections".to_string(),
                json!({ "correction_id": format!("cv-corr-{arm}"),
                        "publication_id": format!("cv-pub-{arm}"),
                        "operation": "retraction", "authority_grant": grant_id,
                        "reason": "r" }),
            ),
            (
                "approval",
                "/v1/policy-approvals".to_string(),
                json!({ "approval_id": format!("cv-app-{arm}"),
                        "proposal_id": format!("cv-prp-{arm}"),
                        "decision_id": format!("cv-dec-{arm}"), "approver": human_id,
                        "grant_id": grant_id,
                        "quorum": { "participants": [human_id], "denominator": 1,
                                    "abstentions": [] } }),
            ),
            (
                "policy version",
                "/v1/policies".to_string(),
                json!({ "policy_id": format!("cv-reg-{arm}"), "version": "1.0.0",
                        "lifecycle": "draft", "title": "cv",
                        "owning_authority": grant_id,
                        "clauses": [ { "id": "cv-c1", "statement": "a rule" } ],
                        "reason": "the control registers a policy version" }),
            ),
        ]
    };

    // ⛔ The ORDER of the arms is chosen so the correction leg has a
    // publication to correct: the correction runs against the SAME arm's
    // publication, which the publication leg has just marked failed — a
    // correction is a record ABOUT a publication and does not require it
    // staged, so both legs of one arm are satisfiable.

    // ── ARM 1: an unrelated action is REFUSED at every surface ───────────────
    set_actions(r#"["thread_contribute"]"#).await;
    for (label, path, body) in surfaces("a") {
        let (status, refused) = post(&client, &base, &path, &human_id, &body).await;
        assert_ne!(
            status, 200,
            "{label}: a grant that does not cover the verb is refused: {refused}"
        );
    }

    // ── ARM 2: the surface's OWN action, and nothing else, is admitted ───────
    // ⛔ Each leg sets only that surface's action, so admission cannot come
    // from a grant that happens to carry everything.
    let covering = [
        r#"["policy_publication_write"]"#,
        r#"["deployment_target_register"]"#,
        r#"["policy_correction_record"]"#,
        r#"["policy_proposal_approve"]"#,
        r#"["policy_version_register"]"#,
    ];
    for (i, (label, path, body)) in surfaces("b").into_iter().enumerate() {
        set_actions(covering[i]).await;
        let (status, answered) = post(&client, &base, &path, &human_id, &body).await;
        assert_eq!(
            status, 200,
            "{label}: the covering grant is admitted: {answered}"
        );
    }

    // ── ARM 3: `tenant_admin` ALONE is admitted everywhere ───────────────────
    // ⭐ This is the stored-boundary disposition `.9.3.4.1` chose, exercised
    // END TO END over real routes: every grant written before the five names
    // existed carries exactly this, and if the subsumption were not honoured
    // here, adding the vocabulary would have silently revoked five verbs from
    // every already-enrolled tenant.
    set_actions(r#"["tenant_admin"]"#).await;
    for (label, path, body) in surfaces("c") {
        let (status, answered) = post(&client, &base, &path, &human_id, &body).await;
        assert_eq!(
            status, 200,
            "{label}: a pre-change `tenant_admin` grant still works: {answered}"
        );
    }
}

/// `SIGNOFF-REPAIR.9.2.1.2.3` — the three publication transitions took an
/// untyped `serde_json::Value`, so an unknown field was SILENTLY IGNORED and
/// every required field was graded by hand.
///
/// ⛔ §9.1 and this repository's own `command_api` control say a
/// client-supplied field is *rejected, not ignored*, and `422 at the strict
/// wire boundary` is how the other 46 typed extractors say it. These three
/// could not: an untyped body has no declared shape to reject against.
///
/// ⭐ The `git_object_ids` leg is the sharpest. The handler did `.as_array()`
/// then `filter_map(|v| v.as_str())`, so a non-string entry VANISHED before
/// `.9.2.1.3`'s existence check ever saw it.
#[tokio::test]
async fn the_publication_transitions_refuse_a_body_that_is_not_their_shape() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };

    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, human) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "wb-human" }),
    )
    .await;
    assert_eq!(status, 200, "the human enrolls: {human}");
    let human_id = human["principal_id"].as_str().unwrap().to_string();
    let grant_id = format!("grt_{human_id}");

    // ⛔ No publication is seeded, and none is needed: the boundary refuses
    // before the handler runs, so these legs prove the SHAPE is graded rather
    // than that some record was found. Leg 3 is what keeps that honest.
    let legs: [(&str, serde_json::Value); 3] = [
        (
            "/v1/policy-publications/wb-pub/effective",
            json!({ "owning_authority": grant_id, "git_object_ids": ["abc"],
                    "repo_path": "live" }),
        ),
        (
            "/v1/policy-publications/wb-pub/failed",
            json!({ "owning_authority": grant_id, "reason": "r" }),
        ),
        (
            "/v1/policy-publications/wb-pub/publish",
            json!({ "owning_authority": grant_id, "repo_path": "live" }),
        ),
    ];

    for (path, body) in legs {
        // Leg 1 — an UNKNOWN field is refused, not ignored.
        let mut forged = body.clone();
        forged["fabricated"] = json!(true);
        let (status, rejected) = post(&client, &base, path, &human_id, &forged).await;
        assert_eq!(
            status, 422,
            "{path}: the unknown field is refused: {rejected}"
        );
        assert!(
            format!("{rejected}").contains("fabricated"),
            "{path}: the rejection NAMES the forged field: {rejected}"
        );

        // Leg 2 — a MISSING required field is refused at the same boundary and
        // named. ⛔ `owning_authority` is the field dropped, so one assertion
        // serves all three verbs.
        let mut missing = body.clone();
        missing.as_object_mut().unwrap().remove("owning_authority");
        let (status, rejected) = post(&client, &base, path, &human_id, &missing).await;
        assert_eq!(
            status, 422,
            "{path}: the missing field is refused: {rejected}"
        );
        assert!(
            format!("{rejected}").contains("owning_authority"),
            "{path}: the rejection names the missing field: {rejected}"
        );
    }

    // Leg 3 — THE MATCHED PAIR, and it is what stops legs 1 and 2 passing for
    // a refuse-everything repair: a WELL-FORMED body of exactly this shape
    // gets PAST the boundary and is refused later, by the authority check,
    // with the typed `{code, message}` a semantic refusal uses.
    let (status, refused) = post(
        &client,
        &base,
        "/v1/policy-publications/wb-pub/failed",
        &human_id,
        &json!({ "owning_authority": "grt_nobody", "reason": "r" }),
    )
    .await;
    assert_eq!(
        status, 403,
        "a well-formed body reaches the handler: {refused}"
    );
    assert_eq!(refused["code"], json!("unauthorized"), "{refused}");

    // Leg 4 — `git_object_ids` is a list of STRINGS. A non-string entry used
    // to be silently dropped by `filter_map`, so a caller could shorten the
    // list `.9.2.1.3` then checked for existence. It is refused now.
    let (status, rejected) = post(
        &client,
        &base,
        "/v1/policy-publications/wb-pub/effective",
        &human_id,
        &json!({ "owning_authority": grant_id, "git_object_ids": ["abc", 7],
                 "repo_path": "live" }),
    )
    .await;
    assert_eq!(
        status, 422,
        "a non-string object id is refused rather than dropped: {rejected}"
    );
}

/// `SIGNOFF-REPAIR.9.2.1.3.2` — the publication verified that its projection
/// EXISTED and never that it was the proposal's, so an approval for one policy
/// published another's compiled bytes.
///
/// ⛔ `stage` matched the decision and the approval to `proposal_id` and left
/// the ONE reference that carries the bytes unmatched: the projection probe was
/// `projection_id = $1 AND tenant_id = $2`. `publish` writes
/// `projection.bytes`, so the authority for writing them was the approval of a
/// proposal that had nothing to do with them.
///
/// ⭐ Two policies, two projections, one approval — the matched pair is the
/// whole control: the SAME staging request differing only in which projection
/// it names is refused for the foreign one and admitted for the proposal's own.
#[tokio::test]
async fn a_publication_carries_the_policy_its_proposal_was_approved_for() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };

    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, human) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "cp-human" }),
    )
    .await;
    assert_eq!(status, 200, "the human enrolls: {human}");
    let human_id = human["principal_id"].as_str().unwrap().to_string();
    let tenant_id = human["tenant_id"].as_str().unwrap().to_string();
    let grant_id = format!("grt_{human_id}");
    site_fixture::provision(
        &pool,
        &human_id,
        &[reasonbraid_server::site_authority::Action::PolicyRegister],
    )
    .await;

    // TWO registered policies, so a projection of one is a real artefact of
    // the other's tenant rather than a fabricated row.
    for (policy_id, statement) in [
        ("cp-approved", "the approved rule"),
        ("cp-other", "a rule nobody approved"),
    ] {
        let (status, registered) = register_policy(
            &client,
            &base,
            &human_id,
            &json!({
                "policy_id": policy_id,
                "version": "1.0.0",
                "lifecycle": "draft",
                "title": policy_id,
                "owning_authority": grant_id,
                "clauses": [ { "id": format!("{policy_id}-c1"), "statement": statement } ],
            }),
        )
        .await;
        assert_eq!(status, 200, "{policy_id} registers: {registered}");
    }

    // A projection PER policy. Both belong to this tenant, so tenancy cannot
    // be what separates them — only the correspondence can.
    let mut digests = std::collections::HashMap::new();
    for policy_id in ["cp-approved", "cp-other"] {
        let (status, projection) = post(
            &client,
            &base,
            "/v1/policy-projections",
            &human_id,
            &json!({
                "projection_id": format!("{policy_id}-proj"),
                "target": "generic",
                "resolution": {
                    "policies": [ { "policy_id": policy_id, "version": "1.0.0" } ],
                    "target": { "layer": "organization", "target": "*" },
                },
            }),
        )
        .await;
        assert_eq!(
            status, 200,
            "the {policy_id} projection records: {projection}"
        );
        digests.insert(
            policy_id,
            projection["digest"].as_str().unwrap().to_string(),
        );
    }
    assert_ne!(
        digests["cp-approved"], digests["cp-other"],
        "the two projections are genuinely different artefacts"
    );

    // The approved chain, for `cp-approved` ONLY. Seeded directly for the
    // reason the sibling controls record: the full walk is driven end to end
    // elsewhere, and re-deriving it here would put the correspondence check
    // behind a second copy of that pipeline.
    sqlx::query(
        "INSERT INTO policy_proposals \
         (proposal_id, policy_id, policy_version, thread_id, status, tenant_id) \
         VALUES ('cp-prp', 'cp-approved', '1.0.0', 'cp-thread', 'approved', $1)",
    )
    .bind(&tenant_id)
    .execute(&pool)
    .await
    .expect("the approved proposal seeds");
    sqlx::query(
        "INSERT INTO policy_decisions \
         (decision_id, proposal_id, rule, electorate, verdict_event_id, tenant_id) \
         VALUES ('cp-dec', 'cp-prp', 'majority', '{}'::jsonb, 'cp-evt', $1)",
    )
    .bind(&tenant_id)
    .execute(&pool)
    .await
    .expect("the decision seeds");
    sqlx::query(
        "INSERT INTO policy_approvals \
         (approval_id, proposal_id, decision_id, approver, grant_id, quorum, tenant_id) \
         VALUES ('cp-app', 'cp-prp', 'cp-dec', $1, $2, '{}'::jsonb, $3)",
    )
    .bind(&human_id)
    .bind(&grant_id)
    .bind(&tenant_id)
    .execute(&pool)
    .await
    .expect("the approval seeds");

    let stage = |publication_id: &'static str, projection_id: String| {
        let client = client.clone();
        let base = base.clone();
        let human_id = human_id.clone();
        let grant_id = grant_id.clone();
        async move {
            post(
                &client,
                &base,
                "/v1/policy-publications",
                &human_id,
                &json!({
                    "publication_id": publication_id,
                    "proposal_id": "cp-prp",
                    "decision_id": "cp-dec",
                    "approval_id": "cp-app",
                    "projection_id": projection_id,
                    "owning_authority": grant_id,
                }),
            )
            .await
        }
    };

    // Leg A — the FOREIGN projection. `cp-other` was never proposed and never
    // approved, and this request would publish its bytes under `cp-approved`'s
    // approval.
    let (status, refused) = stage("cp-pub-foreign", "cp-other-proj".to_string()).await;
    assert_eq!(
        status, 400,
        "a projection that does not carry the approved policy refuses: {refused}"
    );
    let message = refused["message"].as_str().unwrap_or_default().to_string();
    assert!(
        message.contains("cp-approved") && message.contains("cp-other-proj"),
        "the refusal names the policy that is missing and the projection that \
         does not carry it: {refused}"
    );
    let n: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM policy_publications WHERE publication_id = 'cp-pub-foreign'",
    )
    .fetch_one(&pool)
    .await
    .expect("the publications count");
    assert_eq!(n, 0, "the refused staging wrote nothing");

    // Leg B — THE MATCHED PAIR: the same request, the same approval, one field
    // different, and it stages. If leg A's refusal came from the tenant check,
    // the approval chain, the authority or the manifest, this would be refused
    // too.
    let (status, staged) = stage("cp-pub-own", "cp-approved-proj".to_string()).await;
    assert_eq!(
        status, 200,
        "the proposal's OWN projection stages: {staged}"
    );
    assert_eq!(staged["state"], json!("staged"), "{staged}");

    // Leg C — a projection whose resolved set is UNKNOWN (the pre-migration
    // shape) cannot be shown to carry the approved policy, so it is refused.
    // ⛔ Fail CLOSED, the disposition `publications::owned_by` already takes
    // for an unattributable governance row: a publication that cannot be shown
    // to carry what was approved is worse admitted than frozen.
    sqlx::query("UPDATE policy_projections SET resolved_policies = NULL WHERE projection_id = $1")
        .bind("cp-approved-proj")
        .execute(&pool)
        .await
        .expect("the historical shape is restored on one row");
    let (status, refused) = stage("cp-pub-legacy", "cp-approved-proj".to_string()).await;
    assert_eq!(
        status, 400,
        "a projection with no recorded resolved set refuses: {refused}"
    );
    assert!(
        refused["message"]
            .as_str()
            .unwrap_or_default()
            .contains("records no resolved policy set"),
        "the refusal says WHY it cannot answer rather than claiming the policy \
         is absent: {refused}"
    );
}

/// `SIGNOFF-REPAIR.9.2.1.2.2` — the STAGING verb created a publication on
/// enrolment alone, while all three of its transitions required a grant the
/// caller HOLDS.
///
/// ⛔ The half-bound aggregate is not the whole reason it binds. `.9.2.1.3.2`
/// measured that `stage` never reads the proposal's policy and checks the
/// projection only for existence and tenant, so the stager chooses the bytes
/// the approval will publish — staging decides CONTENT, not bookkeeping.
///
/// Three legs, the same three questions the sibling controls ask, so the four
/// publication verbs can be read against each other.
#[tokio::test]
async fn the_staging_verb_requires_an_authority_the_caller_holds() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };

    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, alice) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "ps-alice" }),
    )
    .await;
    assert_eq!(status, 200, "alice enrolls: {alice}");
    let alice_id = alice["principal_id"].as_str().unwrap().to_string();
    let alice_tenant = alice["tenant_id"].as_str().unwrap().to_string();
    let alice_grant = format!("grt_{alice_id}");

    let (status, bob) = enroll(&client, &base, json!({ "kind": "human", "name": "ps-bob" })).await;
    assert_eq!(status, 200, "bob enrolls: {bob}");
    let bob_id = bob["principal_id"].as_str().unwrap().to_string();
    let bob_grant = format!("grt_{bob_id}");
    assert_ne!(alice["tenant_id"], bob["tenant_id"], "distinct tenants");

    // The approved chain, seeded directly. ⛔ The full proposal -> decision ->
    // approval -> projection walk is already driven end to end by
    // `the_publication_stages_and_marks_its_typed_state`; re-deriving it here
    // would put the thing under test — the authority binding — behind a second
    // copy of that pipeline, which is the shape `.9.2.1.2` recorded refusing.
    sqlx::query(
        "INSERT INTO policy_proposals \
         (proposal_id, policy_id, policy_version, thread_id, status, tenant_id) \
         VALUES ('ps-prp', 'ps-policy', '1.0.0', 'ps-thread', 'approved', $1)",
    )
    .bind(&alice_tenant)
    .execute(&pool)
    .await
    .expect("the approved proposal seeds");
    sqlx::query(
        "INSERT INTO policy_decisions \
         (decision_id, proposal_id, rule, electorate, verdict_event_id, tenant_id) \
         VALUES ('ps-dec', 'ps-prp', 'majority', '{}'::jsonb, 'ps-evt', $1)",
    )
    .bind(&alice_tenant)
    .execute(&pool)
    .await
    .expect("the decision seeds");
    sqlx::query(
        "INSERT INTO policy_approvals \
         (approval_id, proposal_id, decision_id, approver, grant_id, quorum, tenant_id) \
         VALUES ('ps-app', 'ps-prp', 'ps-dec', $1, $2, '{}'::jsonb, $3)",
    )
    .bind(&alice_id)
    .bind(&alice_grant)
    .bind(&alice_tenant)
    .execute(&pool)
    .await
    .expect("the approval seeds");
    // ⛔ `.9.2.1.3.2`: the seed records the RESOLVED SET, because a projection
    // that does not is frozen by design — and this fixture found that out the
    // hard way, failing leg C on the fail-closed refusal until the column was
    // seeded. The set matches the proposal's policy, since this control is
    // about AUTHORITY and must not be refused by the correspondence check.
    sqlx::query(
        "INSERT INTO policy_projections \
         (projection_id, target, digest, bytes, unrepresentable, tenant_id, resolved_policies) \
         VALUES ('ps-proj', 'generic', $1, 'body', '[]'::jsonb, $2, \
                 '[{\"policy_id\": \"ps-policy\", \"version\": \"1.0.0\"}]'::jsonb)",
    )
    .bind(DIGEST)
    .bind(&alice_tenant)
    .execute(&pool)
    .await
    .expect("the projection seeds");

    let body = |authority: Option<&str>| {
        let mut b = json!({
            "publication_id": "ps-pub",
            "proposal_id": "ps-prp",
            "decision_id": "ps-dec",
            "approval_id": "ps-app",
            "projection_id": "ps-proj",
        });
        if let Some(a) = authority {
            b["owning_authority"] = json!(a);
        }
        b
    };

    let staged_rows = |label: &'static str, want: i64| {
        let pool = pool.clone();
        async move {
            let n: i64 = sqlx::query_scalar(
                "SELECT count(*) FROM policy_publications WHERE publication_id = 'ps-pub'",
            )
            .fetch_one(&pool)
            .await
            .expect("the publications count");
            assert_eq!(n, want, "{label}: a refused stage must write nothing");
        }
    };

    // Leg A — an enrolled principal naming NO authority is refused. Enrolment
    // plus a tenant-owned approved chain used to be the whole predicate.
    let (status, refused) = post(
        &client,
        &base,
        "/v1/policy-publications",
        &alice_id,
        &body(None),
    )
    .await;
    // ⛔ `422`, THE STRICT WIRE BOUNDARY, and `.9.2.1.2.3` CORRECTED this leg.
    // It first asserted `400`, because `.9.2.1.2.2` hand-graded the absence so
    // that all four verbs answered alike — and the census says the other three
    // were the anomaly: they were UNTYPED and could not use the boundary at
    // all. 46 typed extractors answer `422` here, four suites assert it, and
    // `.4.2.2` depends on it. The rejection NAMES the field, which is the
    // convention's own point.
    assert_eq!(
        status, 422,
        "a missing required field is refused at the wire boundary: {refused}"
    );
    assert!(
        format!("{refused}").contains("owning_authority"),
        "the boundary rejection names the missing field: {refused}"
    );
    staged_rows("leg A", 0).await;

    // Leg B — a real, active grant held by SOMEONE ELSE is refused. ⭐ Grant
    // ids are derivable (`grt_<principal_id>`), so naming one is not holding
    // one — the conflation `.9.3.1` found on five surfaces.
    let (status, refused) = post(
        &client,
        &base,
        "/v1/policy-publications",
        &alice_id,
        &body(Some(&bob_grant)),
    )
    .await;
    assert_eq!(
        status, 403,
        "staging under another principal's grant refuses: {refused}"
    );
    assert_eq!(refused["code"], json!("unauthorized"), "{refused}");
    assert!(
        refused["message"].as_str().unwrap().contains("HOLDS"),
        "{refused}"
    );
    staged_rows("leg B", 0).await;

    // Leg C — THE MATCHED PAIR: the same request, one field different, and the
    // publication stages. The row RECORDS the authority it was staged under,
    // which is the half a check alone would not deliver.
    let (status, staged) = post(
        &client,
        &base,
        "/v1/policy-publications",
        &alice_id,
        &body(Some(&alice_grant)),
    )
    .await;
    assert_eq!(status, 200, "the holder stages: {staged}");
    assert_eq!(staged["state"], json!("staged"), "{staged}");
    let recorded: Option<String> = sqlx::query_scalar(
        "SELECT owning_authority FROM policy_publications WHERE publication_id = 'ps-pub'",
    )
    .fetch_one(&pool)
    .await
    .expect("the staged row reads back");
    assert_eq!(
        recorded.as_deref(),
        Some(alice_grant.as_str()),
        "the COLUMN records the authority the publication was staged under"
    );
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
    // `SIGNOFF-REPAIR.6.1.5.4`: registering a policy version is a site act.
    // This fixture seeds the governance library, so it holds the capability —
    // issued through the deployment-controlled service, never by a row insert.
    site_fixture::provision(
        &pool,
        &human_id,
        &[reasonbraid_server::site_authority::Action::PolicyRegister],
    )
    .await;
    let tenant_id = human["tenant_id"].as_str().unwrap().to_string();
    allow_owner_decides(&pool, &tenant_id).await;
    let grant_id = format!("grt_{human_id}");

    // The chain to the EFFECTIVE publication (the made-up object ids ride
    // the /effective verb — the git half is the `.4.3` lane's, already
    // proven).
    let (status, _) = register_policy(
        &client,
        &base,
        &human_id,
        &json!({
            "policy_id": "dp-policy",
            "version": "1.0.0",
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
                "decision_rule": "owner_decides",
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
            "verdict": { "target_digest": "sha256:00", "outcome": "accepted_by_rule" },
        }),
    )
    .await;
    let verdict_event = verdict["event_id"].as_str().unwrap().to_string();
    close_as_owner(&client, &base, &human_id, &tenant_id, &thread_id).await;

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
                    "rule": "owner_decides",
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
                    "owning_authority": grant_id,
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

    // 1. The target registers (the authority checked), naming its REPORTER — the
    // one principal that files its receipts (`SIGNOFF-REPAIR.9.3.3.2`): here an
    // agent role of the tenant, not the operator who registers and assigns.
    let (status, reporter) = enroll(
        &client,
        &base,
        json!({ "kind": "role", "name": "dp-reporter", "tenant_id": tenant_id }),
    )
    .await;
    assert_eq!(status, 200, "the reporter role enrols: {reporter}");
    let reporter_id = reporter["principal_id"].as_str().unwrap().to_string();
    let (status, _) = post(
        &client,
        &base,
        "/v1/deployment-targets",
        &human_id,
        &json!({
            "target_id": "dp-target",
            "target_type": "repository",
            "owning_authority": grant_id,
            "reporter": reporter_id,
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
            "reporter": reporter_id,
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
            "reporter": reporter_id,
        }),
    )
    .await;
    assert_eq!(status, 400, "the unknown type refuses: {refused}");
    // The reporter must be a principal that can file: a malformed id and an
    // unenrolled one are refused by name. ⛔ And only AFTER the authority: a
    // caller that may not register gets the authority refusal whatever reporter
    // it names, so the verb is no oracle over which principals are enrolled.
    const UNENROLLED: &str = "hpr_00000000-0000-7000-8000-000000009332";
    for (label, target, authority, named, expected) in [
        (
            "a malformed reporter",
            "dp-bad-reporter",
            grant_id.as_str(),
            "nobody",
            "not a principal id",
        ),
        (
            "an unenrolled reporter",
            "dp-ghost-reporter",
            grant_id.as_str(),
            UNENROLLED,
            "not an enrolled principal",
        ),
        (
            "an unauthorized caller",
            "dp-oracle",
            "grt_ghost",
            UNENROLLED,
            "owning authority",
        ),
    ] {
        let (status, refused) = post(
            &client,
            &base,
            "/v1/deployment-targets",
            &human_id,
            &json!({
                "target_id": target,
                "target_type": "repository",
                "owning_authority": authority,
                "reporter": named,
            }),
        )
        .await;
        assert_eq!(status, 400, "{label} refuses: {refused}");
        let message = refused["message"].as_str().unwrap_or_default();
        assert!(message.contains(expected), "{label}: {refused}");
    }
    let (status, targets) = get(&client, &base, "/v1/deployment-targets", &human_id).await;
    assert_eq!(status, 200, "the targets read: {targets}");
    let listed: Vec<&Value> = targets
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| {
            row["target_id"]
                .as_str()
                .is_some_and(|id| id.starts_with("dp-"))
        })
        .collect();
    assert_eq!(
        listed.len(),
        1,
        "only the valid target was stored: {targets}"
    );
    assert_eq!(listed[0]["reporter"], json!(reporter_id), "{targets}");
    // A store failure while looking the reporter up is the server's `500`, never
    // "not an enrolled principal". The reporter is an agent role and the caller a
    // human, so withholding `agent_roles` for one request fails only that lookup;
    // the table is restored before anything is asserted.
    sqlx::raw_sql("ALTER TABLE agent_roles RENAME TO agent_roles_withheld")
        .execute(&pool)
        .await
        .expect("withhold the table");
    let (status, answered) = post(
        &client,
        &base,
        "/v1/deployment-targets",
        &human_id,
        &json!({
            "target_id": "dp-store-fault",
            "target_type": "repository",
            "owning_authority": grant_id,
            "reporter": reporter_id,
        }),
    )
    .await;
    sqlx::raw_sql("ALTER TABLE agent_roles_withheld RENAME TO agent_roles")
        .execute(&pool)
        .await
        .expect("restore the table");
    assert_eq!(
        status, 500,
        "the reporter lookup's failure is the server's: {answered}"
    );
    assert_eq!(
        answered["code"],
        json!("dependency_unavailable"),
        "{answered}"
    );
    // `SIGNOFF-REPAIR.9.3.3.6`: a GENUINE duplicate is still the caller's `400`,
    // and the store failing at the authority read or at the write is the
    // server's `500` — both used to be answered as the caller's mistake.
    let target_body = |target_id: &str| {
        json!({ "target_id": target_id, "target_type": "repository",
                "owning_authority": grant_id, "reporter": reporter_id })
    };
    let (status, refused) = post(
        &client,
        &base,
        "/v1/deployment-targets",
        &human_id,
        &target_body("dp-target"),
    )
    .await;
    assert_eq!(
        status, 400,
        "a genuine duplicate target is the caller's: {refused}"
    );
    assert!(
        refused["message"]
            .as_str()
            .unwrap_or_default()
            .contains("already exists"),
        "{refused}"
    );
    // Input the store cannot hold is the caller's, reaching the WRITE.
    let (status, answered) = post(
        &client,
        &base,
        "/v1/deployment-targets",
        &human_id,
        &target_body("dp-nul\u{0}target"),
    )
    .await;
    assert_eq!(
        status, 400,
        "a NUL in the target id is the caller's: {answered}"
    );
    assert_eq!(
        answered["code"],
        json!("unrepresentable_input"),
        "{answered}"
    );
    for (table, fault, target_id) in [
        ("authority_grants", StoreFault::Unreadable, "dp-fault-grant"),
        (
            "deployment_targets",
            StoreFault::InsertRefused,
            "dp-fault-write",
        ),
    ] {
        let (status, answered) = post_under_fault(
            &pool,
            table,
            fault,
            &client,
            &base,
            "/v1/deployment-targets",
            &human_id,
            &target_body(target_id),
        )
        .await;
        assert_eq!(
            status, 500,
            "registering with `{table}` {fault:?}: {answered}"
        );
        assert_eq!(
            answered["code"],
            json!("dependency_unavailable"),
            "{answered}"
        );
    }

    // 2. The assignment rides the EFFECTIVE publication, and its desired pair
    // IS that publication's (`SIGNOFF-REPAIR.9.3.3.1`, ADR-021: *the effective
    // publication's ref id + the attested projection digest*). A digest the
    // projection does not have, or a ref the publication never recorded, used
    // to be accepted on its shape alone.
    for (label, desired_ref, desired_digest, named) in [
        (
            "another digest",
            object_ids[0].as_str(),
            DIGEST,
            "desired_digest",
        ),
        (
            "an unrecorded ref",
            "abc123",
            projection_digest.as_str(),
            "desired_ref",
        ),
    ] {
        let (status, refused) = post(
            &client,
            &base,
            "/v1/deployments",
            &human_id,
            &json!({
                "target_id": "dp-target",
                "publication_id": "dp-pub-1",
                "wave": 1,
                "desired_ref": desired_ref,
                "desired_digest": desired_digest,
            }),
        )
        .await;
        assert_eq!(status, 400, "{label} is refused: {refused}");
        assert!(
            refused["message"]
                .as_str()
                .unwrap_or_default()
                .contains(named),
            "{label}: the refusal names `{named}`: {refused}"
        );
    }
    let valid = json!({
        "target_id": "dp-target",
        "publication_id": "dp-pub-1",
        "wave": 1,
        "desired_ref": object_ids[0],
        "desired_digest": projection_digest,
    });
    // `SIGNOFF-REPAIR.9.3.3.7`: only the holder of the TARGET's authority points
    // it at a publication. The reporter role is of the same tenant and enrolled,
    // and holds no such grant: refused `403`, as an authority denial.
    let (status, refused) = post(&client, &base, "/v1/deployments", &reporter_id, &valid).await;
    assert_eq!(
        status, 403,
        "a same-tenant principal without the target's authority: {refused}"
    );
    assert_eq!(refused["code"], json!("unauthorized"), "{refused}");
    // A store failure in either new lookup is the server's `500`, never "no such
    // target" or an authority denial. Each table is withheld for one request and
    // restored before anything is asserted.
    for table in [
        "deployment_targets",
        "authority_grants",
        "policy_publications",
    ] {
        sqlx::raw_sql(&format!("ALTER TABLE {table} RENAME TO {table}_withheld"))
            .execute(&pool)
            .await
            .expect("withhold the table");
        let (status, answered) = post(&client, &base, "/v1/deployments", &human_id, &valid).await;
        sqlx::raw_sql(&format!("ALTER TABLE {table}_withheld RENAME TO {table}"))
            .execute(&pool)
            .await
            .expect("restore the table");
        assert_eq!(
            status, 500,
            "`{table}` unreadable is the server's failure: {answered}"
        );
        assert_eq!(
            answered["code"],
            json!("dependency_unavailable"),
            "{answered}"
        );
    }
    let (status, assignment) = post(&client, &base, "/v1/deployments", &human_id, &valid).await;
    assert_eq!(status, 200, "the assignment records: {assignment}");
    assert_eq!(assignment["observed_state"], json!("pending"));
    // The held grant must COVER `deployment_target_register`: the same holder,
    // with the grant narrowed to another verb, is refused on a second target it
    // registered. The grant's actions are restored before anything is asserted.
    let (status, registered) = post(
        &client,
        &base,
        "/v1/deployment-targets",
        &human_id,
        &json!({ "target_id": "dp-target-narrowed", "target_type": "repository",
                 "owning_authority": grant_id, "reporter": reporter_id }),
    )
    .await;
    assert_eq!(status, 200, "the second target registers: {registered}");
    let actions: Value =
        sqlx::query_scalar("SELECT actions FROM authority_grants WHERE grant_id = $1")
            .bind(&grant_id)
            .fetch_one(&pool)
            .await
            .expect("the grant's actions read");
    sqlx::query("UPDATE authority_grants SET actions = $2::jsonb WHERE grant_id = $1")
        .bind(&grant_id)
        .bind(r#"["policy_publication_write"]"#)
        .execute(&pool)
        .await
        .expect("narrow the grant");
    let mut narrowed = valid.clone();
    narrowed["target_id"] = json!("dp-target-narrowed");
    let (status, refused) = post(&client, &base, "/v1/deployments", &human_id, &narrowed).await;
    sqlx::query("UPDATE authority_grants SET actions = $2 WHERE grant_id = $1")
        .bind(&grant_id)
        .bind(&actions)
        .execute(&pool)
        .await
        .expect("restore the grant");
    assert_eq!(
        status, 403,
        "holding the grant is not covering the verb: {refused}"
    );
    // `SIGNOFF-REPAIR.9.3.3.6`: the same assignment twice is a genuine duplicate,
    // `400`; the write failing for any other reason is the server's `500`.
    let (status, refused) = post(&client, &base, "/v1/deployments", &human_id, &valid).await;
    assert_eq!(
        status, 400,
        "a genuine duplicate assignment is the caller's: {refused}"
    );
    assert!(
        refused["message"]
            .as_str()
            .unwrap_or_default()
            .contains("already exists"),
        "{refused}"
    );
    let (status, answered) = post_under_fault(
        &pool,
        "deployment_assignments",
        StoreFault::InsertRefused,
        &client,
        &base,
        "/v1/deployments",
        &human_id,
        &narrowed,
    )
    .await;
    assert_eq!(
        status, 500,
        "an assignment the store will not write: {answered}"
    );
    assert_eq!(
        answered["code"],
        json!("dependency_unavailable"),
        "{answered}"
    );
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
            "desired_ref": object_ids[0],
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
            "desired_ref": object_ids[0],
            "desired_digest": "not-a-digest",
        }),
    )
    .await;
    assert_eq!(status, 400, "the bad digest refuses: {refused}");

    // 3. The receipt attests the OBSERVED digest + the state — and only the
    // principal the TARGET names may file it (`SIGNOFF-REPAIR.9.3.3.2`). Any
    // principal of the owning tenant used to write the observed half, which is
    // the drift comparison's input.
    let (status, bystander) = enroll(
        &client,
        &base,
        json!({ "kind": "role", "name": "dp-bystander", "tenant_id": tenant_id }),
    )
    .await;
    assert_eq!(status, 200, "the bystander role enrols: {bystander}");
    let bystander_id = bystander["principal_id"].as_str().unwrap().to_string();
    // ⭐ The operator is refused too: it registered the target and assigned the
    // desired pair, and ADR-021's receipt is what the target OBSERVED, not what
    // the operator hoped.
    for (label, who) in [
        ("a same-tenant role the target does not name", &bystander_id),
        ("the operator who registered and assigned", &human_id),
    ] {
        let (status, refused) = post(
            &client,
            &base,
            "/v1/deployments/dp-target/dp-pub-1/receipt",
            who,
            &json!({
                "observed_digest": projection_digest,
                "observed_state": "applied",
            }),
        )
        .await;
        assert_eq!(status, 400, "{label} files no receipt: {refused}");
        assert!(
            refused["message"]
                .as_str()
                .unwrap_or_default()
                .contains("reporter"),
            "{label}: the refusal names the reporter rule: {refused}"
        );
    }
    let (status, receipt) = post(
        &client,
        &base,
        "/v1/deployments/dp-target/dp-pub-1/receipt",
        &reporter_id,
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
        &reporter_id,
        &json!({ "observed_digest": projection_digest, "observed_state": "vibes" }),
    )
    .await;
    assert_eq!(status, 400, "the unknown state refuses: {refused}");
    // A target registered before targets named a reporter (`migrations/0114`
    // left its `reporter` NULL) takes no receipt from anyone: nothing is
    // inferred for it.
    sqlx::query(
        "INSERT INTO deployment_targets (target_id, target_type, owning_authority) \
         VALUES ('dp-legacy', 'repository', $1)",
    )
    .bind(&grant_id)
    .execute(&pool)
    .await
    .expect("the pre-0114 target seeds");
    let (desired_ref, desired_digest) = desired_pair(&pool, "dp-pub-1").await;
    let (status, assigned) = post(
        &client,
        &base,
        "/v1/deployments",
        &human_id,
        &json!({
            "target_id": "dp-legacy", "publication_id": "dp-pub-1", "wave": 1,
            "desired_ref": desired_ref, "desired_digest": desired_digest,
        }),
    )
    .await;
    assert_eq!(status, 200, "the legacy target is assigned: {assigned}");
    for who in [&reporter_id, &human_id] {
        let (status, refused) = post(
            &client,
            &base,
            "/v1/deployments/dp-legacy/dp-pub-1/receipt",
            who,
            &json!({ "observed_digest": projection_digest, "observed_state": "applied" }),
        )
        .await;
        assert_eq!(
            status, 400,
            "a target naming no reporter takes no receipt: {refused}"
        );
        assert!(
            refused["message"]
                .as_str()
                .unwrap_or_default()
                .contains("names no reporter"),
            "{refused}"
        );
    }

    // 4. The list carries the desired/observed pair.
    let (status, deployments) = get(&client, &base, "/v1/deployments", &human_id).await;
    assert_eq!(status, 200, "the deployments read: {deployments}");
    let deployments = deployments.as_array().unwrap();
    assert_eq!(deployments.len(), 2, "{deployments:?}");
    // Ordered by target: `dp-legacy`, never reported, then `dp-target`.
    assert_eq!(deployments[0]["target_id"], json!("dp-legacy"));
    assert_eq!(deployments[0]["observed_digest"], Value::Null);
    assert_eq!(deployments[1]["desired_digest"], json!(projection_digest));
    assert_eq!(deployments[1]["observed_digest"], json!(projection_digest));

    // 5. A second receipt does not erase the first (`SIGNOFF-REPAIR.9.3.3.3`,
    // §12.9's *never a silent disappearance*): every receipt is a row naming its
    // reporter, in the order filed, and the assignment's pair is the latest. The
    // single observed pair used to be overwritten in place.
    let (status, second) = post(
        &client,
        &base,
        "/v1/deployments/dp-target/dp-pub-1/receipt",
        &reporter_id,
        &json!({ "observed_digest": DIGEST, "observed_state": "rejected" }),
    )
    .await;
    assert_eq!(status, 200, "the second receipt records: {second}");
    assert_eq!(second["observed_digest"], json!(DIGEST), "{second}");
    let (status, history) = get(
        &client,
        &base,
        "/v1/deployments/dp-target/dp-pub-1/receipts",
        &human_id,
    )
    .await;
    assert_eq!(status, 200, "the receipt history reads: {history}");
    let history = history.as_array().expect("the history is a list");
    assert_eq!(history.len(), 2, "both receipts are kept: {history:?}");
    assert_eq!(history[0]["observed_digest"], json!(projection_digest));
    assert_eq!(history[0]["observed_state"], json!("applied"));
    assert_eq!(history[1]["observed_digest"], json!(DIGEST));
    assert_eq!(history[1]["observed_state"], json!("rejected"));
    for row in history {
        assert_eq!(row["reporter"], json!(reporter_id), "{row}");
    }
    // Each kept row carries the PUBLICATION's tenant, and the history reads by
    // it: a row for the same pair under another tenant is not this tenant's.
    let owners: Vec<String> = sqlx::query_scalar(
        "SELECT DISTINCT tenant_id FROM deployment_receipts \
         WHERE target_id = 'dp-target' AND publication_id = 'dp-pub-1'",
    )
    .fetch_all(&pool)
    .await
    .expect("the owners read");
    assert_eq!(
        owners,
        vec![tenant_id.clone()],
        "a receipt carries its publication's tenant"
    );
    sqlx::query(
        "INSERT INTO deployment_receipts (receipt_id, target_id, publication_id, tenant_id, \
         reporter, observed_digest, observed_state) \
         VALUES ('drc_foreign', 'dp-target', 'dp-pub-1', 'tnt_elsewhere', $1, $2, 'applied')",
    )
    .bind(&reporter_id)
    .bind(DIGEST)
    .execute(&pool)
    .await
    .expect("a foreign-tenant row seeds");
    let (status, history) = get(
        &client,
        &base,
        "/v1/deployments/dp-target/dp-pub-1/receipts",
        &human_id,
    )
    .await;
    assert_eq!(status, 200, "{history}");
    assert_eq!(
        history.as_array().map(Vec::len),
        Some(2),
        "a row under another tenant is not in this tenant's history: {history}"
    );
    // An assignment never reported has an empty history, and one that does not
    // exist is refused like any absent assignment.
    let (status, empty) = get(
        &client,
        &base,
        "/v1/deployments/dp-legacy/dp-pub-1/receipts",
        &human_id,
    )
    .await;
    assert_eq!(status, 200, "a never-reported assignment reads: {empty}");
    assert_eq!(empty, json!([]), "{empty}");
    let (status, refused) = get(
        &client,
        &base,
        "/v1/deployments/dp-target/dp-pub-2/receipts",
        &human_id,
    )
    .await;
    assert_eq!(
        status, 400,
        "an absent assignment has no history: {refused}"
    );
    // ⛔ The database refuses to rewrite a kept receipt.
    let rewritten = sqlx::query(
        "UPDATE deployment_receipts SET observed_state = 'applied' \
         WHERE target_id = 'dp-target' AND observed_state = 'rejected'",
    )
    .execute(&pool)
    .await;
    assert!(
        rewritten
            .as_ref()
            .is_err_and(|e| e.to_string().contains("never rewritten")),
        "a receipt is never rewritten: {rewritten:?}"
    );
    // ⛔ The row and the pair are ONE write: with the history unwritable for one
    // request, the receipt is the server's `500` and the assignment's pair is
    // unchanged, never updated without its row. Restored before asserting.
    sqlx::raw_sql("ALTER TABLE deployment_receipts RENAME TO deployment_receipts_withheld")
        .execute(&pool)
        .await
        .expect("withhold the table");
    let (status, answered) = post(
        &client,
        &base,
        "/v1/deployments/dp-target/dp-pub-1/receipt",
        &reporter_id,
        &json!({ "observed_digest": projection_digest, "observed_state": "applied" }),
    )
    .await;
    sqlx::raw_sql("ALTER TABLE deployment_receipts_withheld RENAME TO deployment_receipts")
        .execute(&pool)
        .await
        .expect("restore the table");
    assert_eq!(
        status, 500,
        "an unwritable history is the server's failure: {answered}"
    );
    assert_eq!(
        answered["code"],
        json!("dependency_unavailable"),
        "{answered}"
    );
    let pair: (Option<String>, String) = sqlx::query_as(
        "SELECT observed_digest, observed_state FROM deployment_assignments \
         WHERE target_id = 'dp-target' AND publication_id = 'dp-pub-1'",
    )
    .fetch_one(&pool)
    .await
    .expect("the pair reads");
    assert_eq!(
        pair,
        (Some(DIGEST.to_string()), "rejected".to_string()),
        "a failed receipt leaves the pair as the last kept receipt"
    );
    // …and the other direction: with the PAIR unwritable for one request (a
    // trigger injected on the assignment, dropped before asserting), the row the
    // receipt already wrote is rolled back with it — no kept receipt names a pair
    // the assignment never took.
    let kept = |pool: PgPool| async move {
        sqlx::query_scalar::<_, i64>(
            "SELECT count(*) FROM deployment_receipts \
             WHERE target_id = 'dp-target' AND publication_id = 'dp-pub-1'",
        )
        .fetch_one(&pool)
        .await
        .expect("the history counts")
    };
    let before = kept(pool.clone()).await;
    sqlx::raw_sql(
        "CREATE FUNCTION dp_refuse_pair() RETURNS trigger LANGUAGE plpgsql AS \
         $$ BEGIN RAISE EXCEPTION 'the pair is withheld'; END; $$; \
         CREATE TRIGGER dp_refuse_pair BEFORE UPDATE ON deployment_assignments \
         FOR EACH ROW EXECUTE FUNCTION dp_refuse_pair();",
    )
    .execute(&pool)
    .await
    .expect("withhold the pair");
    let (status, answered) = post(
        &client,
        &base,
        "/v1/deployments/dp-target/dp-pub-1/receipt",
        &reporter_id,
        &json!({ "observed_digest": projection_digest, "observed_state": "applied" }),
    )
    .await;
    sqlx::raw_sql(
        "DROP TRIGGER dp_refuse_pair ON deployment_assignments; DROP FUNCTION dp_refuse_pair();",
    )
    .execute(&pool)
    .await
    .expect("restore the pair");
    assert_eq!(
        status, 500,
        "an unwritable pair is the server's failure: {answered}"
    );
    assert_eq!(
        kept(pool.clone()).await,
        before,
        "the receipt's row is rolled back with the pair it could not set"
    );
    // ⛔ Receipts to one assignment are taken ONE AT A TIME, so "the latest row"
    // and "the assignment's pair" are the same receipt: a receipt waits for the
    // assignment's lock BEFORE it writes its row. Observed by holding the lock,
    // waiting until the receipt is queued on it, and releasing at a known
    // database instant: the kept row must be recorded AFTER that instant. A
    // receipt that wrote its row first and queued only for the pair would record
    // it before, and two such receipts could leave the pair on the older one.
    // ⚠️ Such a receipt queues on the row's INSERT, not on the assignment: the
    // foreign key's check takes a share lock on the held assignment row. The
    // wait is therefore watched on both tables, or that receipt would never be
    // seen queued and the ordering assertion below would never be reached.
    let mut holder = pool.begin().await.expect("the holder begins");
    sqlx::query(
        "SELECT 1 FROM deployment_assignments \
         WHERE target_id = 'dp-target' AND publication_id = 'dp-pub-1' FOR UPDATE",
    )
    .execute(&mut *holder)
    .await
    .expect("the holder locks the assignment");
    let queued = tokio::spawn({
        let (client, base, reporter_id) = (client.clone(), base.clone(), reporter_id.clone());
        let digest = projection_digest.clone();
        async move {
            post(
                &client,
                &base,
                "/v1/deployments/dp-target/dp-pub-1/receipt",
                &reporter_id,
                &json!({ "observed_digest": digest, "observed_state": "applied" }),
            )
            .await
        }
    });
    let mut waiting = 0_i64;
    for _ in 0..500 {
        waiting = sqlx::query_scalar(
            "SELECT count(*) FROM pg_stat_activity WHERE wait_event_type = 'Lock' \
             AND (query LIKE '%deployment_assignments%' OR query LIKE '%deployment_receipts%')",
        )
        .fetch_one(&pool)
        .await
        .expect("the wait reads");
        if waiting > 0 {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    assert_eq!(waiting, 1, "the receipt queues on the assignment's lock");
    let released: chrono::DateTime<chrono::Utc> = sqlx::query_scalar("SELECT clock_timestamp()")
        .fetch_one(&mut *holder)
        .await
        .expect("the release instant reads");
    holder.commit().await.expect("the holder releases");
    let (status, filed) = queued.await.expect("the queued receipt joins");
    assert_eq!(
        status, 200,
        "the queued receipt records once released: {filed}"
    );
    let recorded: chrono::DateTime<chrono::Utc> = sqlx::query_scalar(
        "SELECT max(recorded_at) FROM deployment_receipts \
         WHERE target_id = 'dp-target' AND publication_id = 'dp-pub-1'",
    )
    .fetch_one(&pool)
    .await
    .expect("the latest receipt reads");
    assert!(
        recorded > released,
        "the receipt's row is written after the lock is released ({recorded} > {released})"
    );
    // Input the store cannot hold is the caller's, and permanent.
    let (status, answered) = post(
        &client,
        &base,
        "/v1/deployments/dp%00target/dp-pub-1/receipt",
        &reporter_id,
        &json!({ "observed_digest": projection_digest, "observed_state": "applied" }),
    )
    .await;
    assert_eq!(status, 400, "a NUL in the path is the caller's: {answered}");
    assert_eq!(
        answered["code"],
        json!("unrepresentable_input"),
        "{answered}"
    );
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
    // `SIGNOFF-REPAIR.6.1.5.4`: registering a policy version is a site act.
    // This fixture seeds the governance library, so it holds the capability —
    // issued through the deployment-controlled service, never by a row insert.
    site_fixture::provision(
        &pool,
        &human_id,
        &[reasonbraid_server::site_authority::Action::PolicyRegister],
    )
    .await;
    let tenant_id = human["tenant_id"].as_str().unwrap().to_string();
    allow_owner_decides(&pool, &tenant_id).await;
    let grant_id = format!("grt_{human_id}");

    // The chain to the effective publication + the target + the assignment
    // (the same path the `.5.2` test drives).
    let (status, _) = register_policy(
        &client,
        &base,
        &human_id,
        &json!({
            "policy_id": "cr-policy",
            "version": "1.0.0",
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
                "decision_rule": "owner_decides",
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
            "verdict": { "target_digest": "sha256:00", "outcome": "accepted_by_rule" },
        }),
    )
    .await;
    let verdict_event = verdict["event_id"].as_str().unwrap().to_string();
    close_as_owner(&client, &base, &human_id, &tenant_id, &thread_id).await;
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
                "rule": "owner_decides",
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
        let (status, _projection) = post(
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
                "owning_authority": grant_id,
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
        &json!({ "target_id": "cr-target", "target_type": "repository", "owning_authority": grant_id,
                 "reporter": human_id }),
    )
    .await;
    assert_eq!(status, 200, "the target registers");
    let (desired_ref, desired_digest) = desired_pair(&pool, "cr-pub-1").await;
    let (status, _) = post(
        &client,
        &base,
        "/v1/deployments",
        &human_id,
        &json!({
            "target_id": "cr-target",
            "publication_id": "cr-pub-1",
            "wave": 1,
            "desired_ref": desired_ref,
            "desired_digest": desired_digest,
        }),
    )
    .await;
    assert_eq!(status, 200, "the assignment records");

    // 1. The drift: the categorized pair. Its DESIRED half is the assignment's
    // (`SIGNOFF-REPAIR.9.3.3.5`): a drift record says a target is not running what
    // was published, and a desired digest the assignment does not carry makes
    // that a comparison against a value nobody published.
    let (status, refused) = post(
        &client,
        &base,
        "/v1/policy-drift",
        &human_id,
        &json!({
            "drift_id": "cr-drift-declared",
            "target_id": "cr-target",
            "publication_id": "cr-pub-1",
            "category": "pending_rollout",
            "desired_digest": DIGEST,
            "observed_digest": null,
        }),
    )
    .await;
    assert_eq!(
        status, 400,
        "a desired digest the assignment does not carry is refused: {refused}"
    );
    assert!(
        refused["message"]
            .as_str()
            .unwrap_or_default()
            .contains("desired_digest"),
        "the refusal names the field: {refused}"
    );
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
            "desired_digest": desired_digest,
            "observed_digest": null,
        }),
    )
    .await;
    assert_eq!(status, 200, "the drift records");
    // The assignment is the PAIR: `cr-pub-2` is this tenant's and effective, and
    // was never assigned to `cr-target`, so a drift naming the two is refused
    // even though the target carries another assignment with the same digest.
    let (status, refused) = post(
        &client,
        &base,
        "/v1/policy-drift",
        &human_id,
        &json!({
            "drift_id": "cr-drift-unassigned",
            "target_id": "cr-target",
            "publication_id": "cr-pub-2",
            "category": "pending_rollout",
            "desired_digest": desired_digest,
            "observed_digest": null,
        }),
    )
    .await;
    assert_eq!(
        status, 400,
        "a pair that was never assigned has no drift: {refused}"
    );
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
            "desired_digest": desired_digest,
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
            "desired_digest": desired_digest,
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
            // RELATIVE (`SIGNOFF-REPAIR.9.3.2`): the hard-coded date had passed.
            "expires_at": (chrono::Utc::now() + chrono::Duration::days(30)).to_rfc3339(),
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

    // 5. `SIGNOFF-REPAIR.9.3.3.6`: each writer tells the caller's mistake from the
    // store's failure. A genuine duplicate, and an expiry that is not a time, stay
    // the caller's `400`; the store failing at a read or at the write is the
    // server's `500` — every one of these used to answer "does not exist", "not an
    // active grant" or "already exists".
    let drift = |drift_id: &str| {
        json!({ "drift_id": drift_id, "target_id": "cr-target", "publication_id": "cr-pub-1",
                "category": "pending_rollout", "desired_digest": desired_digest,
                "observed_digest": null })
    };
    let correction = |correction_id: &str| {
        json!({ "correction_id": correction_id, "publication_id": "cr-pub-1",
                "operation": "retraction", "authority_grant": grant_id,
                "reason": "the owners withdrew" })
    };
    let outcome = |outcome_id: &str| {
        json!({ "outcome_id": outcome_id, "publication_id": "cr-pub-1",
                "kind": "observation", "note": "n" })
    };
    for (path, body, what) in [
        ("/v1/policy-drift", drift("cr-drift-1"), "drift"),
        (
            "/v1/policy-corrections",
            correction("cr-retract"),
            "correction",
        ),
        ("/v1/policy-outcomes", outcome("cr-out-1"), "outcome"),
    ] {
        let (status, refused) = post(&client, &base, path, &human_id, &body).await;
        assert_eq!(
            status, 400,
            "a genuine duplicate {what} is the caller's: {refused}"
        );
        assert!(
            refused["message"]
                .as_str()
                .unwrap_or_default()
                .contains("already exists"),
            "{refused}"
        );
    }
    let (status, refused) = post(
        &client,
        &base,
        "/v1/policy-corrections",
        &human_id,
        &json!({ "correction_id": "cr-suspend-badtime", "publication_id": "cr-pub-1",
                 "operation": "suspension", "authority_grant": grant_id,
                 "expires_at": "next tuesday", "reason": "r" }),
    )
    .await;
    assert_eq!(
        status, 400,
        "an expiry that is not a time is the caller's: {refused}"
    );
    assert!(
        refused["message"]
            .as_str()
            .unwrap_or_default()
            .contains("not an RFC 3339"),
        "a malformed expiry is not reported as a missing one: {refused}"
    );
    let mut nul = correction("cr-corr-nul");
    nul["reason"] = json!("with\u{0}nul");
    let (status, answered) = post(&client, &base, "/v1/policy-corrections", &human_id, &nul).await;
    assert_eq!(
        status, 400,
        "a NUL in the reason is the caller's: {answered}"
    );
    assert_eq!(
        answered["code"],
        json!("unrepresentable_input"),
        "{answered}"
    );
    for (table, fault, path, body) in [
        (
            "policy_publications",
            StoreFault::Unreadable,
            "/v1/policy-outcomes",
            outcome("cr-out-f1"),
        ),
        (
            "deployment_assignments",
            StoreFault::Unreadable,
            "/v1/policy-drift",
            drift("cr-drift-f1"),
        ),
        (
            "policy_drift",
            StoreFault::InsertRefused,
            "/v1/policy-drift",
            drift("cr-drift-f2"),
        ),
        (
            "authority_grants",
            StoreFault::Unreadable,
            "/v1/policy-corrections",
            correction("cr-corr-f1"),
        ),
        (
            "policy_corrections",
            StoreFault::InsertRefused,
            "/v1/policy-corrections",
            correction("cr-corr-f2"),
        ),
        (
            "policy_outcomes",
            StoreFault::InsertRefused,
            "/v1/policy-outcomes",
            outcome("cr-out-f2"),
        ),
    ] {
        let (status, answered) =
            post_under_fault(&pool, table, fault, &client, &base, path, &human_id, &body).await;
        assert_eq!(status, 500, "{path} with `{table}` {fault:?}: {answered}");
        assert_eq!(
            answered["code"],
            json!("dependency_unavailable"),
            "{answered}"
        );
    }
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
    // `SIGNOFF-REPAIR.6.1.5.4`: registering a policy version is a site act.
    // This fixture seeds the governance library, so it holds the capability —
    // issued through the deployment-controlled service, never by a row insert.
    site_fixture::provision(
        &pool,
        &human_id,
        &[reasonbraid_server::site_authority::Action::PolicyRegister],
    )
    .await;
    let tenant_id = human["tenant_id"].as_str().unwrap().to_string();
    allow_owner_decides(&pool, &tenant_id).await;
    let grant_id = format!("grt_{human_id}");

    // The chain to a STAGED publication (the outcomes only require the
    // existence).
    let (status, _) = register_policy(
        &client,
        &base,
        &human_id,
        &json!({
            "policy_id": "rv-policy",
            "version": "1.0.0",
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
                "decision_rule": "owner_decides",
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
            "verdict": { "target_digest": "sha256:00", "outcome": "accepted_by_rule" },
        }),
    )
    .await;
    let verdict_event = verdict["event_id"].as_str().unwrap().to_string();
    close_as_owner(&client, &base, &human_id, &tenant_id, &thread_id).await;
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
            "rule": "owner_decides",
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
    let (status, _projection) = post(
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
            "owning_authority": grant_id,
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
    // ⛔ RELATIVE clocks (`SIGNOFF-REPAIR.9.3.2`): this fixture hard-coded
    // `2026-09-15`, which had passed before the expiry predicate arrived.
    let in_force = (chrono::Utc::now() + chrono::Duration::days(30)).to_rfc3339();
    let lapsed = (chrono::Utc::now() - chrono::Duration::days(1)).to_rfc3339();
    let waiver = |id: &str, expires_at: &str| {
        json!({
            "correction_id": id,
            "publication_id": "rv-pub",
            "operation": "waiver",
            "authority_grant": grant_id,
            "expires_at": expires_at,
            "reason": "the bounded exception",
        })
    };
    let schedule = || {
        let client = client.clone();
        let base = base.clone();
        let human_id = human_id.clone();
        async move {
            post(
                &client,
                &base,
                "/v1/policy-reviews/schedule",
                &human_id,
                &json!({}),
            )
            .await
        }
    };
    // A LAPSED waiver, one recorded OUTSIDE the window, and ONE in force within
    // it: that is not a repeated waiver.
    for (id, expires_at) in [
        ("rv-waiver-lapsed", &lapsed),
        ("rv-waiver-old", &in_force),
        ("rv-waiver-1", &in_force),
    ] {
        let (status, recorded) = post(
            &client,
            &base,
            "/v1/policy-corrections",
            &human_id,
            &waiver(id, expires_at),
        )
        .await;
        assert_eq!(status, 200, "the waiver records: {recorded}");
    }
    // The window is measured from when a waiver was RECORDED, which only the
    // store sets; backdate one past it.
    sqlx::query(
        "UPDATE policy_corrections SET created_at = now() - interval '91 days' \
         WHERE correction_id = 'rv-waiver-old'",
    )
    .execute(&pool)
    .await
    .expect("backdate the old waiver");

    // 2. The schedule: the drift trigger (the outcome) alone. One waiver in
    // force used to satisfy `repeated_waiver`, and a lapsed one counted too.
    let (status, scheduled) = schedule().await;
    assert_eq!(status, 200, "the schedule evaluates: {scheduled}");
    let scheduled = scheduled.as_array().unwrap().clone();
    let triggers: Vec<&str> = scheduled
        .iter()
        .map(|r| r["trigger"].as_str().unwrap())
        .collect();
    assert_eq!(
        triggers,
        ["drift"],
        "one waiver in force is not a repeat: {scheduled:?}"
    );

    // 3. A SECOND waiver in force within the window IS a repeat.
    let (status, recorded) = post(
        &client,
        &base,
        "/v1/policy-corrections",
        &human_id,
        &waiver("rv-waiver-2", &in_force),
    )
    .await;
    assert_eq!(status, 200, "the second waiver records: {recorded}");
    let (status, repeated) = schedule().await;
    assert_eq!(status, 200, "{repeated}");
    let repeated = repeated.as_array().unwrap().clone();
    assert_eq!(repeated.len(), 1, "{repeated:?}");
    assert_eq!(
        repeated[0]["trigger"],
        json!("repeated_waiver"),
        "{repeated:?}"
    );

    // 4. The schedule is IDEMPOTENT while a review is due.
    let (status, again) = schedule().await;
    assert_eq!(status, 200, "the second schedule: {again}");
    assert_eq!(again.as_array().unwrap().len(), 0, "the dedupe holds");

    // 5. The done transition + the re-done refusal.
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
    // The completed review covered its occurrence: nothing new is due.
    let (status, covered) = schedule().await;
    assert_eq!(status, 200, "{covered}");
    assert_eq!(
        covered.as_array().unwrap().len(),
        0,
        "the review covered its occurrence"
    );

    // 6. The lifecycle RECURS: a new drift occurrence after the completed
    // review schedules a NEW review. The id used to be `rev_{publication}_{trigger}`,
    // the primary key, so every later insert for the pair collided and the
    // error was discarded — the pair could be reviewed once, for ever.
    let drift_outcome = |id: &str| {
        json!({
            "outcome_id": id,
            "publication_id": "rv-pub",
            "kind": "observation",
            "review_trigger": "drift",
            "note": "the target lagged again",
        })
    };
    let (status, _) = post(
        &client,
        &base,
        "/v1/policy-outcomes",
        &human_id,
        &drift_outcome("rv-out-2"),
    )
    .await;
    assert_eq!(status, 200, "the second drift outcome records");
    let (status, recurred) = schedule().await;
    assert_eq!(status, 200, "{recurred}");
    let recurred = recurred.as_array().unwrap().clone();
    assert_eq!(
        recurred.len(),
        1,
        "a new occurrence, a new review: {recurred:?}"
    );
    assert_eq!(recurred[0]["trigger"], json!("drift"), "{recurred:?}");
    assert_ne!(
        recurred[0]["review_id"], scheduled[0]["review_id"],
        "a NEW review, not the completed one"
    );
    // Another occurrence while that review is due folds into it: one due
    // review per publication and trigger.
    let (status, _) = post(
        &client,
        &base,
        "/v1/policy-outcomes",
        &human_id,
        &drift_outcome("rv-out-2b"),
    )
    .await;
    assert_eq!(status, 200, "another drift outcome records");
    let (status, folded) = schedule().await;
    assert_eq!(status, 200, "{folded}");
    assert_eq!(
        folded.as_array().unwrap().len(),
        0,
        "one due review per pair: {folded}"
    );

    // 7. The list: the completed drift review, the due repeated-waiver review,
    // and the recurred drift review.
    let (status, reviews) = get(&client, &base, "/v1/policy-reviews", &human_id).await;
    assert_eq!(status, 200, "the reviews read: {reviews}");
    let reviews = reviews.as_array().unwrap();
    assert_eq!(reviews.len(), 3, "{reviews:?}");
    assert!(
        reviews.iter().any(|r| r["status"] == json!("done")),
        "the done review rides the list"
    );

    // 8. An INSERT that fails is an error, not an empty success: every insert
    // error used to be discarded by `is_ok()`, so a failing store answered
    // "nothing was due". The failure is forced by a trigger on the table,
    // dropped before anything is asserted.
    let recurred_id = recurred[0]["review_id"].as_str().unwrap().to_string();
    let (status, _) = post(
        &client,
        &base,
        &format!("/v1/policy-reviews/{recurred_id}/done"),
        &human_id,
        &json!({}),
    )
    .await;
    assert_eq!(status, 200, "the recurred review marks done");
    let (status, _) = post(
        &client,
        &base,
        "/v1/policy-outcomes",
        &human_id,
        &drift_outcome("rv-out-3"),
    )
    .await;
    assert_eq!(status, 200, "the third drift outcome records");
    sqlx::raw_sql(
        "CREATE FUNCTION rv_forced_failure() RETURNS trigger LANGUAGE plpgsql AS \
         $$ BEGIN RAISE EXCEPTION 'forced insert failure'; END $$; \
         CREATE TRIGGER rv_forced_failure BEFORE INSERT ON policy_reviews \
         FOR EACH ROW EXECUTE FUNCTION rv_forced_failure();",
    )
    .execute(&pool)
    .await
    .expect("force the insert to fail");
    let (status, failed) = schedule().await;
    sqlx::raw_sql(
        "DROP TRIGGER rv_forced_failure ON policy_reviews; DROP FUNCTION rv_forced_failure();",
    )
    .execute(&pool)
    .await
    .expect("remove the forced failure");
    assert_eq!(
        status, 500,
        "a failed insert is the server's failure, not an empty schedule: {failed}"
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
    // `SIGNOFF-REPAIR.6.1.5.4`: registering a policy version is a site act.
    // This fixture seeds the governance library, so it holds the capability —
    // issued through the deployment-controlled service, never by a row insert.
    site_fixture::provision(
        &pool,
        &alice_id,
        &[reasonbraid_server::site_authority::Action::PolicyRegister],
    )
    .await;
    let alice_grant = format!("grt_{alice_id}");
    let alice_tenant = alice["tenant_id"].as_str().unwrap().to_string();

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
    let (status, registered) = register_policy(
        &client,
        &base,
        &alice_id,
        &json!({
            "policy_id": "cite-pol",
            "version": "1.0.0",
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
         (publication_id, proposal_id, decision_id, approval_id, projection_id, state, \
          manifest_digest, tenant_id) \
         VALUES ('cite-pub-1', 'cite-prp', 'cite-dec', 'cite-app', 'cite-proj', 'published', \
                 $1, $2)",
    )
    .bind(DIGEST)
    .bind(&alice_tenant)
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
            "reporter": alice_id,
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
        "INSERT INTO aggregate_state (tenant_id, aggregate_id, aggregate_type, aggregate_version, state) \
         VALUES ($1, 'cite-thread', 'thread', 1, '{}'::jsonb) ON CONFLICT DO NOTHING",
    )
    .bind(&alice_tenant)
    .execute(&pool)
    .await
    .expect("the fixture's thread exists, as `.6.1.5.1` requires of a real proposal");
    sqlx::query(
        "INSERT INTO policy_proposals \
         (proposal_id, policy_id, policy_version, thread_id, status, tenant_id) \
         VALUES ('cite-prp-1', 'cite-pol', '1.0.0', 'cite-thread', 'decided', $1)",
    )
    .bind(&alice_tenant)
    .execute(&pool)
    .await
    .expect("seed the decided proposal");
    // `SIGNOFF-REPAIR.11.4.7.2.1.2.3.2`: a DERIVED decision whose electorate
    // equals the quorum this leg asserts, so only authority can refuse it.
    sqlx::query(
        "INSERT INTO policy_decisions \
         (decision_id, proposal_id, rule, electorate, verdict_event_id, derivation) \
         VALUES ('cite-dec-1', 'cite-prp-1', 'owner_decides', $1, 'cite-evt', \
                 '{\"fixture\": \"authority\"}'::jsonb)",
    )
    .bind(json!({ "participants": [bob_id], "denominator": 1, "abstentions": [] }))
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
        "INSERT INTO policy_proposals \
         (proposal_id, policy_id, policy_version, thread_id, status, tenant_id) \
         VALUES ('cite-prp-2', 'cite-pol', '1.0.0', 'cite-thread', 'decided', $1)",
    )
    .bind(&alice_tenant)
    .execute(&pool)
    .await
    .expect("seed the second decided proposal");
    // `SIGNOFF-REPAIR.11.4.7.2.1.2.3.2`: a DERIVED decision whose electorate
    // equals the quorum this leg asserts, so only authority can refuse it.
    sqlx::query(
        "INSERT INTO policy_decisions \
         (decision_id, proposal_id, rule, electorate, verdict_event_id, derivation) \
         VALUES ('cite-dec-2', 'cite-prp-2', 'owner_decides', $1, 'cite-evt-2', \
                 '{\"fixture\": \"authority\"}'::jsonb)",
    )
    .bind(json!({ "participants": [alice_id], "denominator": 1, "abstentions": [] }))
    .execute(&pool)
    .await
    .expect("seed the second decision");

    // And a THIRD for leg G, for the same reason: a successful approval
    // advances its proposal, so two legs sharing one proposal report each
    // other's outcome. Invisible while the repair holds and immediately
    // visible under falsification — which is where it was found.
    sqlx::query(
        "INSERT INTO policy_proposals \
         (proposal_id, policy_id, policy_version, thread_id, status, tenant_id) \
         VALUES ('cite-prp-3', 'cite-pol', '1.0.0', 'cite-thread', 'decided', $1)",
    )
    .bind(&alice_tenant)
    .execute(&pool)
    .await
    .expect("seed the third decided proposal");
    // `SIGNOFF-REPAIR.11.4.7.2.1.2.3.2`: a DERIVED decision whose electorate
    // equals the quorum this leg asserts, so only authority can refuse it.
    sqlx::query(
        "INSERT INTO policy_decisions \
         (decision_id, proposal_id, rule, electorate, verdict_event_id, derivation) \
         VALUES ('cite-dec-3', 'cite-prp-3', 'owner_decides', $1, 'cite-evt-3', \
                 '{\"fixture\": \"authority\"}'::jsonb)",
    )
    .bind(json!({ "participants": [alice_id], "denominator": 1, "abstentions": [] }))
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
            "reporter": alice_id,
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

/// `SIGNOFF-REPAIR.6.1.5.1` — an approval is an act upon a proposal, so it is
/// bound to the tenant that owns the proposal's thread.
///
/// ⛔ **Not merely a disclosure defect.** `publications::stage` reads
/// `policy_approvals` to decide whether a publication may be STAGED, and refuses
/// only a `ForeignRecord` whose parent is the wrong PROPOSAL — never one whose
/// approver belongs to another tenant. So a foreign approval carried a foreign
/// publication forward.
///
/// ⚠️ The foreign proposal answers exactly as an ABSENT one does, per
/// `docs/decisions/2026-09-18_node-presence-is-read-by-its-own-tenant.md`:
/// distinguishing them would leave an existence oracle over every other tenant's
/// proposal ids.
#[tokio::test]
async fn an_approval_is_bound_to_the_proposals_own_tenant() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, alice) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "apt-alice" }),
    )
    .await;
    assert_eq!(status, 200, "alice enrols: {alice}");
    let alice_id = alice["principal_id"].as_str().unwrap().to_string();
    // `SIGNOFF-REPAIR.6.1.5.4`: registering a policy version is a site act.
    // This fixture seeds the governance library, so it holds the capability —
    // issued through the deployment-controlled service, never by a row insert.
    site_fixture::provision(
        &pool,
        &alice_id,
        &[reasonbraid_server::site_authority::Action::PolicyRegister],
    )
    .await;
    let alice_tenant = alice["tenant_id"].as_str().unwrap().to_string();
    allow_owner_decides(&pool, &alice_tenant).await;
    let alice_grant = format!("grt_{alice_id}");

    // ⭐ Mallory is enrolled in her OWN tenant and holds her OWN live grant, so
    // the control measures the tenant binding and nothing else: every other
    // check on this path — the grant's liveness, the approver being the
    // authenticated caller (`.9.3.1`), the decision's parentage — passes for her.
    let (status, mallory) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "apt-mallory" }),
    )
    .await;
    assert_eq!(status, 200, "mallory enrols: {mallory}");
    let mallory_id = mallory["principal_id"].as_str().unwrap().to_string();
    let mallory_grant = format!("grt_{mallory_id}");
    assert_ne!(
        alice_tenant,
        mallory["tenant_id"].as_str().unwrap(),
        "two enrolments must be two tenants, or this control measures nothing"
    );

    let (status, _) = register_policy(
        &client,
        &base,
        &alice_id,
        &json!({
            "policy_id": "apt-policy",
            "version": "1.0.0",
            "lifecycle": "draft",
            "title": "apt",
            "owning_authority": alice_grant,
            "clauses": [ { "id": "c1", "statement": "the approval clause" } ],
        }),
    )
    .await;
    assert_eq!(status, 200, "the policy registers");

    let (_status, created) = post(
        &client,
        &base,
        "/v1/threads",
        &alice_id,
        &json!({
            "protocol_version": reasonbraid_core::PROTOCOL_VERSION,
            "operation": "thread.create",
            "request_id": reasonbraid_core::RequestId::new().to_string(),
            "idempotency_key": "apt-create",
            "body": {
                "tenant_id": alice_tenant,
                "subject": "apt",
                "objective": "probe",
                "workflow_profile": "independent_panel",
                "decision_rule": "owner_decides",
            },
            "client_context": {},
        }),
    )
    .await;
    let thread_id = created["thread_id"].as_str().unwrap().to_string();
    let command = |key: &'static str, operation: &'static str, body: Value| {
        let client = client.clone();
        let base = base.clone();
        let alice_id = alice_id.clone();
        let thread_id = thread_id.clone();
        async move {
            let response = client
                .post(format!("{base}/v1/threads/{thread_id}/commands"))
                .header(PRINCIPAL_HEADER, &alice_id)
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
            (
                status,
                serde_json::from_str(&text).unwrap_or_else(|_| json!({ "raw": text })),
            )
        }
    };
    let (status, _) = command(
        "apt-advance",
        "thread.advance_round",
        json!({ "tenant_id": alice_tenant }),
    )
    .await;
    assert_eq!(status, 200, "the round advances");
    let (_status, verdict) = command(
        "apt-verdict",
        "thread.contribute",
        json!({
            "tenant_id": alice_tenant,
            "content": "judged",
            "kind": "verdict",
            "verdict": { "target_digest": "sha256:00", "outcome": "accepted_by_rule" },
        }),
    )
    .await;
    let verdict_event = verdict["event_id"].as_str().unwrap().to_string();
    close_as_owner(&client, &base, &alice_id, &alice_tenant, &thread_id).await;

    let (status, _) = post(
        &client,
        &base,
        "/v1/policy-proposals",
        &alice_id,
        &json!({
            "proposal_id": "apt-prop", "policy_id": "apt-policy",
            "policy_version": "1.0.0", "thread_id": thread_id,
        }),
    )
    .await;
    assert_eq!(status, 200, "alice's proposal registers");
    let (status, _) = post(
        &client,
        &base,
        "/v1/policy-decisions",
        &alice_id,
        &json!({
            "decision_id": "apt-dec", "proposal_id": "apt-prop", "rule": "owner_decides",
            "electorate": { "participants": [alice_id], "denominator": 1, "abstentions": [] },
            "verdict_event_id": verdict_event,
        }),
    )
    .await;
    assert_eq!(status, 200, "alice's decision records");

    // THE NEGATIVE ARM: mallory approves alice's proposal, as herself, with her
    // own live grant. Everything but the tenant is in order.
    let (status, refused) = post(
        &client,
        &base,
        "/v1/policy-approvals",
        &mallory_id,
        &json!({
            "approval_id": "apt-app-foreign", "proposal_id": "apt-prop",
            "decision_id": "apt-dec", "approver": mallory_id, "grant_id": mallory_grant,
            "quorum": { "participants": [mallory_id], "denominator": 1, "abstentions": [] },
        }),
    )
    .await;
    assert_eq!(
        status, 400,
        "a foreign tenant approves no proposal: {refused}"
    );
    assert!(
        refused["message"]
            .as_str()
            .unwrap_or_default()
            .contains("apt-prop"),
        "the refusal names the proposal as an ABSENT one would: {refused}"
    );

    // The stage did NOT advance — the assertion that makes this a control
    // defect rather than a rejected request.
    let (status, proposals) = get(&client, &base, "/v1/policy-proposals", &alice_id).await;
    assert_eq!(status, 200, "alice reads her proposals: {proposals}");
    assert_eq!(
        proposals[0]["status"],
        json!("decided"),
        "a foreign approval advances no stage: {proposals}"
    );

    // THE POSITIVE ARM: alice approves her own proposal and it advances.
    let (status, approval) = post(
        &client,
        &base,
        "/v1/policy-approvals",
        &alice_id,
        &json!({
            "approval_id": "apt-app-own", "proposal_id": "apt-prop",
            "decision_id": "apt-dec", "approver": alice_id, "grant_id": alice_grant,
            "quorum": { "participants": [alice_id], "denominator": 1, "abstentions": [] },
        }),
    )
    .await;
    assert_eq!(status, 200, "alice approves her own proposal: {approval}");
    let (_status, proposals) = get(&client, &base, "/v1/policy-proposals", &alice_id).await;
    assert_eq!(
        proposals[0]["status"],
        json!("approved"),
        "the owning tenant still advances the stage: {proposals}"
    );
}

/// `SIGNOFF-REPAIR.6.1.5.1.1` — the two lifecycle verbs whose tenant claim bound
/// nothing in the profile this repository runs.
///
/// ⛔ **Both gates were open AND unobservable**, which is the part worth stating.
/// `rls.rs` records that the dev profile's superuser connection bypasses RLS
/// regardless, and `migrations/0046`'s `FORCE ROW LEVEL SECURITY` does not reach
/// a superuser either — so no control could ever have watched these admit or
/// refuse anything, and none did.
///
/// ⚠️ **Not a claim that RLS is broken.** Under the app role the policies bind
/// exactly as `2026-09-08_rls-tenant-claim.md` describes. What was wrong is
/// relying on them ALONE for a tenant gate here.
#[tokio::test]
async fn the_lifecycle_verbs_refuse_a_foreign_tenants_thread() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, alice) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "lft-alice" }),
    )
    .await;
    assert_eq!(status, 200, "alice enrols: {alice}");
    let alice_id = alice["principal_id"].as_str().unwrap().to_string();
    // `SIGNOFF-REPAIR.6.1.5.4`: registering a policy version is a site act.
    // This fixture seeds the governance library, so it holds the capability —
    // issued through the deployment-controlled service, never by a row insert.
    site_fixture::provision(
        &pool,
        &alice_id,
        &[reasonbraid_server::site_authority::Action::PolicyRegister],
    )
    .await;
    let alice_tenant = alice["tenant_id"].as_str().unwrap().to_string();
    allow_owner_decides(&pool, &alice_tenant).await;
    let alice_grant = format!("grt_{alice_id}");

    let (status, mallory) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "lft-mallory" }),
    )
    .await;
    assert_eq!(status, 200, "mallory enrols: {mallory}");
    let mallory_id = mallory["principal_id"].as_str().unwrap().to_string();

    let (status, _) = register_policy(
        &client,
        &base,
        &alice_id,
        &json!({
            "policy_id": "lft-policy", "version": "1.0.0",
            "lifecycle": "draft", "title": "lft", "owning_authority": alice_grant,
            "clauses": [ { "id": "c1", "statement": "the lifecycle clause" } ],
        }),
    )
    .await;
    assert_eq!(status, 200, "the policy registers");

    // Alice's thread, and a verdict inside it. Both are hers alone.
    let (_status, created) = post(
        &client,
        &base,
        "/v1/threads",
        &alice_id,
        &json!({
            "protocol_version": reasonbraid_core::PROTOCOL_VERSION,
            "operation": "thread.create",
            "request_id": reasonbraid_core::RequestId::new().to_string(),
            "idempotency_key": "lft-create",
            "body": {
                "tenant_id": alice_tenant, "subject": "lft", "objective": "probe",
                "workflow_profile": "independent_panel",
                "decision_rule": "owner_decides",
            },
            "client_context": {},
        }),
    )
    .await;
    let thread_id = created["thread_id"].as_str().unwrap().to_string();
    let command = |key: &'static str, operation: &'static str, body: Value| {
        let client = client.clone();
        let base = base.clone();
        let alice_id = alice_id.clone();
        let thread_id = thread_id.clone();
        async move {
            let response = client
                .post(format!("{base}/v1/threads/{thread_id}/commands"))
                .header(PRINCIPAL_HEADER, &alice_id)
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
            let text = response.text().await.expect("command body");
            serde_json::from_str::<Value>(&text).unwrap_or_else(|_| json!({ "raw": text }))
        }
    };
    let _ = command(
        "lft-advance",
        "thread.advance_round",
        json!({ "tenant_id": alice_tenant }),
    )
    .await;
    let verdict = command(
        "lft-verdict",
        "thread.contribute",
        json!({
            "tenant_id": alice_tenant, "content": "judged", "kind": "verdict",
            "verdict": { "target_digest": "sha256:00", "outcome": "accepted_by_rule" },
        }),
    )
    .await;
    let verdict_event = verdict["event_id"].as_str().unwrap().to_string();
    close_as_owner(&client, &base, &alice_id, &alice_tenant, &thread_id).await;

    // ── ARM 1: `register_proposal` — mallory names ALICE's thread ──────────────
    let (status, refused) = post(
        &client,
        &base,
        "/v1/policy-proposals",
        &mallory_id,
        &json!({
            "proposal_id": "lft-prop-foreign", "policy_id": "lft-policy",
            "policy_version": "1.0.0", "thread_id": thread_id,
        }),
    )
    .await;
    assert_eq!(
        status, 400,
        "a foreign tenant proposes against no thread of another tenant's: {refused}"
    );

    // ── The owning tenant still registers (the positive arm for ARM 1) ────────
    let (status, own) = post(
        &client,
        &base,
        "/v1/policy-proposals",
        &alice_id,
        &json!({
            "proposal_id": "lft-prop-own", "policy_id": "lft-policy",
            "policy_version": "1.0.0", "thread_id": thread_id,
        }),
    )
    .await;
    assert_eq!(status, 200, "alice still proposes on her own thread: {own}");

    // ── ARM 2: `record_decision` — mallory cites a verdict from ALICE's thread ─
    // ⭐ The proposal is Alice's own and valid, so the ONLY thing under test is
    // whether the verdict event is reachable from mallory's tenant.
    let (status, refused) = post(
        &client,
        &base,
        "/v1/policy-decisions",
        &mallory_id,
        &json!({
            "decision_id": "lft-dec-foreign", "proposal_id": "lft-prop-own", "rule": "owner_decides",
            "electorate": { "participants": [mallory_id], "denominator": 1, "abstentions": [] },
            "verdict_event_id": verdict_event,
        }),
    )
    .await;
    assert_eq!(
        status, 400,
        "a foreign tenant cites no verdict from another tenant's thread: {refused}"
    );

    // ── The owning tenant still decides (the positive arm for ARM 2) ──────────
    let (status, decided) = post(
        &client,
        &base,
        "/v1/policy-decisions",
        &alice_id,
        &json!({
            "decision_id": "lft-dec-own", "proposal_id": "lft-prop-own", "rule": "owner_decides",
            "electorate": { "participants": [alice_id], "denominator": 1, "abstentions": [] },
            "verdict_event_id": verdict_event,
        }),
    )
    .await;
    assert_eq!(
        status, 200,
        "alice still decides on her own verdict: {decided}"
    );
}

/// `SIGNOFF-REPAIR.6.1.5.2` — every lifecycle row carries the tenant that OWNS
/// it, and for five of the nine that is not the tenant that wrote it.
///
/// ⭐ **The sharp half is the second arm.** Mallory records drift, an outcome and
/// a correction against ALICE's publication, and schedules the reviews. Each row
/// must come back stamped `alice`: a row about Alice's publication carrying
/// Mallory's tenant would vanish from the only party it concerns the moment
/// `.6.1.5.3` binds the reads — `.6.1.5`'s own trap, re-entered from the write
/// side.
///
/// ⚠️ **This leaf LABELS rows; it does not GATE writes.** Mallory is admitted
/// throughout, and the control asserts that she is: the gate is `.6.1.5.2.1`'s,
/// and a control that expected a refusal here would be testing that leaf's work
/// rather than this one's.
///
/// ⛔ **The reads are asserted UNCHANGED**, so this leaf cannot accidentally
/// deliver `.6.1.5.3`.
#[tokio::test]
async fn the_lifecycle_row_carries_the_tenant_that_owns_it() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let (repo_root, object_ids) = seeded_publication_repository("lto");
    let server = TestServer::start_with_publication_root(&pool, &repo_root).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, alice) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "lto-alice" }),
    )
    .await;
    assert_eq!(status, 200, "alice enrols: {alice}");
    let alice_id = alice["principal_id"].as_str().unwrap().to_string();
    // `SIGNOFF-REPAIR.6.1.5.4`: registering a policy version is a site act.
    // This fixture seeds the governance library, so it holds the capability —
    // issued through the deployment-controlled service, never by a row insert.
    site_fixture::provision(
        &pool,
        &alice_id,
        &[reasonbraid_server::site_authority::Action::PolicyRegister],
    )
    .await;
    let alice_tenant = alice["tenant_id"].as_str().unwrap().to_string();
    allow_owner_decides(&pool, &alice_tenant).await;
    let alice_grant = format!("grt_{alice_id}");

    let (status, mallory) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "lto-mallory" }),
    )
    .await;
    assert_eq!(status, 200, "mallory enrols: {mallory}");
    let mallory_id = mallory["principal_id"].as_str().unwrap().to_string();
    let mallory_tenant = mallory["tenant_id"].as_str().unwrap().to_string();
    assert_ne!(
        alice_tenant, mallory_tenant,
        "two enrolments must be two tenants, or this control measures nothing"
    );

    // The tenant a stored row actually carries. ⛔ Read straight from the table:
    // the READ verbs are still site-wide by design until `.6.1.5.3`, so they
    // could not tell these rows apart even if the column were wrong.
    let stored = |table: &'static str, key: &'static str, id: String| {
        let pool = pool.clone();
        async move {
            sqlx::query_scalar::<_, Option<String>>(&format!(
                "SELECT tenant_id FROM {table} WHERE {key} = $1"
            ))
            .bind(id)
            .fetch_one(&pool)
            .await
            .expect("the row exists")
        }
    };

    let (status, _) = register_policy(
        &client,
        &base,
        &alice_id,
        &json!({
            "policy_id": "lto-policy", "version": "1.0.0",
            "lifecycle": "draft", "title": "lto", "owning_authority": alice_grant,
            "clauses": [ { "id": "c1", "statement": "the owned clause" } ],
        }),
    )
    .await;
    assert_eq!(status, 200, "the policy registers");

    let (_status, created) = post(
        &client,
        &base,
        "/v1/threads",
        &alice_id,
        &json!({
            "protocol_version": reasonbraid_core::PROTOCOL_VERSION,
            "operation": "thread.create",
            "request_id": reasonbraid_core::RequestId::new().to_string(),
            "idempotency_key": "lto-create",
            "body": {
                "tenant_id": alice_tenant, "subject": "lto", "objective": "probe",
                "workflow_profile": "independent_panel",
                "decision_rule": "owner_decides",
            },
            "client_context": {},
        }),
    )
    .await;
    let thread_id = created["thread_id"].as_str().unwrap().to_string();
    let command = |key: &'static str, operation: &'static str, body: Value| {
        let client = client.clone();
        let base = base.clone();
        let alice_id = alice_id.clone();
        let thread_id = thread_id.clone();
        async move {
            let response = client
                .post(format!("{base}/v1/threads/{thread_id}/commands"))
                .header(PRINCIPAL_HEADER, &alice_id)
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
            let text = response.text().await.expect("command body");
            serde_json::from_str::<Value>(&text).unwrap_or_else(|_| json!({ "raw": text }))
        }
    };
    let _ = command(
        "lto-advance",
        "thread.advance_round",
        json!({ "tenant_id": alice_tenant }),
    )
    .await;
    let verdict = command(
        "lto-verdict",
        "thread.contribute",
        json!({
            "tenant_id": alice_tenant, "content": "judged", "kind": "verdict",
            "verdict": { "target_digest": "sha256:00", "outcome": "accepted_by_rule" },
        }),
    )
    .await;
    let verdict_event = verdict["event_id"].as_str().unwrap().to_string();
    close_as_owner(&client, &base, &alice_id, &alice_tenant, &thread_id).await;

    // ── ARM 1: the four rows whose tenant is the CALLER's ─────────────────────
    let (status, _) = post(
        &client,
        &base,
        "/v1/policy-proposals",
        &alice_id,
        &json!({
            "proposal_id": "lto-prop", "policy_id": "lto-policy",
            "policy_version": "1.0.0", "thread_id": thread_id,
        }),
    )
    .await;
    assert_eq!(status, 200, "alice's proposal registers");
    let (status, _) = post(
        &client,
        &base,
        "/v1/policy-decisions",
        &alice_id,
        &json!({
            "decision_id": "lto-dec", "proposal_id": "lto-prop", "rule": "owner_decides",
            "electorate": { "participants": [alice_id], "denominator": 1, "abstentions": [] },
            "verdict_event_id": verdict_event,
        }),
    )
    .await;
    assert_eq!(status, 200, "alice's decision records");
    let (status, _) = post(
        &client,
        &base,
        "/v1/policy-approvals",
        &alice_id,
        &json!({
            "approval_id": "lto-app", "proposal_id": "lto-prop", "decision_id": "lto-dec",
            "approver": alice_id, "grant_id": alice_grant,
            "quorum": { "participants": [alice_id], "denominator": 1, "abstentions": [] },
        }),
    )
    .await;
    assert_eq!(status, 200, "alice's approval records");

    for (table, key, id) in [
        ("policy_proposals", "proposal_id", "lto-prop"),
        ("policy_decisions", "decision_id", "lto-dec"),
        ("policy_approvals", "approval_id", "lto-app"),
    ] {
        assert_eq!(
            stored(table, key, id.to_string()).await,
            Some(alice_tenant.clone()),
            "{table} carries the caller's tenant"
        );
    }

    // ── ARM 2: the projection's tenant is its AUTHOR's, because it has no
    //    ancestor to inherit from. Mallory's own projection is Mallory's. ──────
    let (status, alice_projection) = post(
        &client,
        &base,
        "/v1/policy-projections",
        &alice_id,
        &json!({
            "projection_id": "lto-proj", "target": "generic",
            "resolution": {
                "policies": [ { "policy_id": "lto-policy", "version": "1.0.0" } ],
                "target": { "layer": "organization", "target": "*" },
            },
        }),
    )
    .await;
    assert_eq!(
        status, 200,
        "alice's projection records: {alice_projection}"
    );
    let (status, _) = post(
        &client,
        &base,
        "/v1/policy-projections",
        &mallory_id,
        &json!({
            "projection_id": "lto-proj-m", "target": "generic",
            "resolution": {
                "policies": [ { "policy_id": "lto-policy", "version": "1.0.0" } ],
                "target": { "layer": "organization", "target": "*" },
            },
        }),
    )
    .await;
    assert_eq!(status, 200, "mallory projects the shared library too");
    assert_eq!(
        stored("policy_projections", "projection_id", "lto-proj".into()).await,
        Some(alice_tenant.clone()),
        "a projection is its author's"
    );
    assert_eq!(
        stored("policy_projections", "projection_id", "lto-proj-m".into()).await,
        Some(mallory_tenant.clone()),
        "and the OTHER author's projection is hers — the library is shared, the \
         compiled artifact records who asked for it"
    );

    // ── ARM 3: the five rows whose tenant is their PARENT's. ─────────────────
    //
    // ⚠️ ALICE writes these, and the reason is a consequence of `.6.1.5.2.1`
    // worth stating: once every one of these verbs requires the caller to own
    // the parent, caller and owner ALWAYS coincide, so no black-box control can
    // any longer tell parent-derivation from caller-derivation at this surface.
    // This arm proves the column carries the owner; the LINEAGE is measured
    // where it is still observable — the backfill coverage in
    // `tests/migration_upgrade.rs` — and the gate that makes the two coincide is
    // `the_lifecycle_verbs_refuse_another_tenants_publication`.
    //
    // ⭐ The anchor stays PARENT rather than caller, deliberately: a gate can be
    // wrong, and a row derived from its parent cannot be mislabelled by a caller
    // that a faulty gate admitted. Defence in depth, not decoration.
    let (status, staged) = post(
        &client,
        &base,
        "/v1/policy-publications",
        &alice_id,
        &json!({
            "publication_id": "lto-pub", "proposal_id": "lto-prop",
            "decision_id": "lto-dec", "approval_id": "lto-app",
            "projection_id": "lto-proj", "owning_authority": alice_grant,
        }),
    )
    .await;
    assert_eq!(
        status, 200,
        "alice stages her own approved proposal: {staged}"
    );
    assert_eq!(
        stored("policy_publications", "publication_id", "lto-pub".into()).await,
        Some(alice_tenant.clone()),
        "a publication carries its PROPOSAL's tenant"
    );

    let (status, _) = post(
        &client,
        &base,
        &format!("/v1/policy-publications/{}/effective", "lto-pub"),
        &alice_id,
        &json!({ "git_object_ids": object_ids.clone(), "repo_path": "live", "owning_authority": alice_grant }),
    )
    .await;
    assert_eq!(status, 200, "the publication marks effective");
    let (status, _) = post(
        &client,
        &base,
        "/v1/deployment-targets",
        &alice_id,
        &json!({ "target_id": "lto-target", "target_type": "repository", "owning_authority": alice_grant,
                 "reporter": alice_id }),
    )
    .await;
    assert_eq!(status, 200, "the target registers");
    let (desired_ref, desired_digest) = desired_pair(&pool, "lto-pub").await;
    let (status, _) = post(
        &client,
        &base,
        "/v1/deployments",
        &alice_id,
        &json!({
            "target_id": "lto-target", "publication_id": "lto-pub", "wave": 1,
            "desired_ref": desired_ref, "desired_digest": desired_digest,
        }),
    )
    .await;
    assert_eq!(status, 200, "the assignment records");

    let (status, _) = post(
        &client,
        &base,
        "/v1/policy-drift",
        &alice_id,
        &json!({
            "drift_id": "lto-drift", "target_id": "lto-target", "publication_id": "lto-pub",
            "category": "pending_rollout", "desired_digest": desired_digest, "observed_digest": null,
        }),
    )
    .await;
    assert_eq!(status, 200, "alice records drift on her own publication");
    let (status, _) = post(
        &client,
        &base,
        "/v1/policy-corrections",
        &alice_id,
        &json!({
            "correction_id": "lto-corr", "publication_id": "lto-pub", "operation": "waiver",
            "authority_grant": alice_grant, "expires_at": "2030-01-01T00:00:00Z",
            "reason": "the foreign waiver",
        }),
    )
    .await;
    assert_eq!(status, 200, "alice waives her own publication");
    let (status, _) = post(
        &client,
        &base,
        "/v1/policy-outcomes",
        &alice_id,
        &json!({
            "outcome_id": "lto-out", "publication_id": "lto-pub", "kind": "incident",
            "review_trigger": "adverse_threshold", "note": "the foreign outcome",
        }),
    )
    .await;
    assert_eq!(
        status, 200,
        "alice records an outcome on her own publication"
    );
    let (status, scheduled) = post(
        &client,
        &base,
        "/v1/policy-reviews/schedule",
        &alice_id,
        &json!({}),
    )
    .await;
    assert_eq!(status, 200, "alice schedules her own reviews: {scheduled}");
    assert!(
        !scheduled.as_array().unwrap().is_empty(),
        "the schedule materialised rows, or the next assertion is vacuous: {scheduled}"
    );

    for (table, key, id) in [
        ("policy_drift", "drift_id", "lto-drift"),
        ("policy_corrections", "correction_id", "lto-corr"),
        ("policy_outcomes", "outcome_id", "lto-out"),
    ] {
        assert_eq!(
            stored(table, key, id.to_string()).await,
            Some(alice_tenant.clone()),
            "{table} carries its PUBLICATION's tenant"
        );
    }
    let review_tenants: Vec<Option<String>> =
        sqlx::query_scalar("SELECT tenant_id FROM policy_reviews ORDER BY review_id")
            .fetch_all(&pool)
            .await
            .expect("the reviews are readable");
    assert!(
        !review_tenants.is_empty()
            && review_tenants
                .iter()
                .all(|t| t.as_deref() == Some(alice_tenant.as_str())),
        "every scheduled review is its publication's, never the scheduler's: {review_tenants:?}"
    );

    // ── THE READS. ⭐ `.6.1.5.2` asserted here that they were UNCHANGED, which
    //    is what kept that leaf out of `.6.1.5.3`'s; `.6.1.5.3` has since bound
    //    them, so the same assertion now runs the other way and this line is the
    //    record of that discharge rather than a rewritten expectation.
    let (status, proposals) = get(&client, &base, "/v1/policy-proposals", &mallory_id).await;
    assert_eq!(status, 200, "mallory reads her own proposals: {proposals}");
    assert!(
        !proposals
            .as_array()
            .unwrap()
            .iter()
            .any(|p| p["proposal_id"] == json!("lto-prop")),
        "alice's proposal is absent from another tenant's list (`.6.1.5.3`): {proposals}"
    );
    let (status, proposals) = get(&client, &base, "/v1/policy-proposals", &alice_id).await;
    assert_eq!(status, 200, "alice reads her own proposals: {proposals}");
    assert!(
        proposals
            .as_array()
            .unwrap()
            .iter()
            .any(|p| p["proposal_id"] == json!("lto-prop")),
        "and the owner still reads it, so the binding is not a blackout: {proposals}"
    );
}

/// `SIGNOFF-REPAIR.6.1.5.2.1` — every mutating lifecycle verb refuses a record
/// that is not the caller's, and the owning tenant still performs all eleven.
///
/// ⛔ **ELEVEN, not the five the leaf was opened with.** `.6.1.5.2` touched five
/// write sites and noticed the gap at those five; the question is which verbs act
/// on a record named off the wire without checking it is the caller's, and the
/// answer is every mutating verb over a publication or a proposal.
///
/// ⛔ **The three `held_publication_authority` verbs are the sharpest, because
/// they look guarded and are not.** `.9.2.1.2` made them require a grant the
/// CALLER HOLDS — a real check answering a different question: *may this
/// principal act on publications at all*, never *is this publication theirs*.
/// Mallory holds her own live grant here, so she passes that gate and every
/// other one on the path; the control measures the tenant binding and nothing
/// else, exactly as `.6.1.5.1`'s did.
///
/// ⚠️ A foreign record answers exactly as an ABSENT one
/// (`docs/decisions/2026-09-18_node-presence-is-read-by-its-own-tenant.md`), so
/// the arms assert the refusal names the record rather than the tenant.
#[tokio::test]
async fn the_lifecycle_verbs_refuse_another_tenants_publication() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let (repo_root, object_ids) = seeded_publication_repository("gtn");
    let server = TestServer::start_with_publication_root(&pool, &repo_root).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, alice) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "gtn-alice" }),
    )
    .await;
    assert_eq!(status, 200, "alice enrols: {alice}");
    let alice_id = alice["principal_id"].as_str().unwrap().to_string();
    // `SIGNOFF-REPAIR.6.1.5.4`: registering a policy version is a site act.
    // This fixture seeds the governance library, so it holds the capability —
    // issued through the deployment-controlled service, never by a row insert.
    site_fixture::provision(
        &pool,
        &alice_id,
        &[reasonbraid_server::site_authority::Action::PolicyRegister],
    )
    .await;
    let alice_tenant = alice["tenant_id"].as_str().unwrap().to_string();
    allow_owner_decides(&pool, &alice_tenant).await;
    let alice_grant = format!("grt_{alice_id}");

    let (status, mallory) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "gtn-mallory" }),
    )
    .await;
    assert_eq!(status, 200, "mallory enrols: {mallory}");
    let mallory_id = mallory["principal_id"].as_str().unwrap().to_string();
    let mallory_grant = format!("grt_{mallory_id}");
    assert_ne!(
        alice_tenant,
        mallory["tenant_id"].as_str().unwrap(),
        "two enrolments must be two tenants, or this control measures nothing"
    );

    let (status, _) = register_policy(
        &client,
        &base,
        &alice_id,
        &json!({
            "policy_id": "gtn-policy", "version": "1.0.0",
            "lifecycle": "draft", "title": "gtn", "owning_authority": alice_grant,
            "clauses": [ { "id": "c1", "statement": "the gated clause" } ],
        }),
    )
    .await;
    assert_eq!(status, 200, "the policy registers");

    let (_status, created) = post(
        &client,
        &base,
        "/v1/threads",
        &alice_id,
        &json!({
            "protocol_version": reasonbraid_core::PROTOCOL_VERSION,
            "operation": "thread.create",
            "request_id": reasonbraid_core::RequestId::new().to_string(),
            "idempotency_key": "gtn-create",
            "body": {
                "tenant_id": alice_tenant, "subject": "gtn", "objective": "probe",
                "workflow_profile": "independent_panel",
                "decision_rule": "owner_decides",
            },
            "client_context": {},
        }),
    )
    .await;
    let thread_id = created["thread_id"].as_str().unwrap().to_string();
    let command = |key: &'static str, operation: &'static str, body: Value| {
        let client = client.clone();
        let base = base.clone();
        let alice_id = alice_id.clone();
        let thread_id = thread_id.clone();
        async move {
            let response = client
                .post(format!("{base}/v1/threads/{thread_id}/commands"))
                .header(PRINCIPAL_HEADER, &alice_id)
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
            let text = response.text().await.expect("command body");
            serde_json::from_str::<Value>(&text).unwrap_or_else(|_| json!({ "raw": text }))
        }
    };
    let _ = command(
        "gtn-advance",
        "thread.advance_round",
        json!({ "tenant_id": alice_tenant }),
    )
    .await;
    let verdict = command(
        "gtn-verdict",
        "thread.contribute",
        json!({
            "tenant_id": alice_tenant, "content": "judged", "kind": "verdict",
            "verdict": { "target_digest": "sha256:00", "outcome": "accepted_by_rule" },
        }),
    )
    .await;
    let verdict_event = verdict["event_id"].as_str().unwrap().to_string();
    close_as_owner(&client, &base, &alice_id, &alice_tenant, &thread_id).await;

    // ⭐ THREE publications, because three of the eleven verbs consume a STAGED
    // one and a staged publication can be spent exactly once: `failed` and
    // `publish` each end the stage, so sharing a publication would make one
    // verb's positive arm depend on another's — the coupling
    // `citing_an_authority_requires_holding_it` records finding the hard way.
    let stage_chain = |n: u8| {
        let client = client.clone();
        let base = base.clone();
        let alice_id = alice_id.clone();
        let alice_grant = alice_grant.clone();
        let thread_id = thread_id.clone();
        let verdict_event = verdict_event.clone();
        async move {
            let prop = format!("gtn-prop-{n}");
            let dec = format!("gtn-dec-{n}");
            let app = format!("gtn-app-{n}");
            let proj = format!("gtn-proj-{n}");
            let publication = format!("gtn-pub-{n}");
            let (status, _) = post(
                &client,
                &base,
                "/v1/policy-proposals",
                &alice_id,
                &json!({ "proposal_id": prop, "policy_id": "gtn-policy",
                         "policy_version": "1.0.0", "thread_id": thread_id }),
            )
            .await;
            assert_eq!(status, 200, "proposal {n} registers");
            let (status, _) = post(
                &client, &base, "/v1/policy-decisions", &alice_id,
                &json!({ "decision_id": dec, "proposal_id": prop, "rule": "owner_decides",
                         "electorate": { "participants": [alice_id], "denominator": 1, "abstentions": [] },
                         "verdict_event_id": verdict_event }),
            ).await;
            assert_eq!(status, 200, "decision {n} records");
            let (status, _) = post(
                &client, &base, "/v1/policy-approvals", &alice_id,
                &json!({ "approval_id": app, "proposal_id": prop, "decision_id": dec,
                         "approver": alice_id, "grant_id": alice_grant,
                         "quorum": { "participants": [alice_id], "denominator": 1, "abstentions": [] } }),
            ).await;
            assert_eq!(status, 200, "approval {n} records");
            let (status, _projection) = post(
                &client, &base, "/v1/policy-projections", &alice_id,
                &json!({ "projection_id": proj, "target": "generic",
                         "resolution": { "policies": [ { "policy_id": "gtn-policy", "version": "1.0.0" } ],
                                         "target": { "layer": "organization", "target": "*" } } }),
            ).await;
            assert_eq!(status, 200, "projection {n} records");
            (prop, dec, app, proj, publication)
        }
    };

    // ── ARM 1: `POST /v1/policy-publications` — stage another tenant's proposal
    let (prop1, dec1, app1, proj1, pub1) = stage_chain(1).await;
    // ⛔ `.9.2.1.2.2`: the caller names its OWN grant. ARM 1 measures TENANT
    // containment, so mallory must be PAST the authority gate — the shape
    // ARMs 2-4 already use, and without it this arm would keep asserting 400
    // while silently measuring the new authority refusal instead.
    let staging = |who: String,
                   grant: String,
                   prop: String,
                   dec: String,
                   app: String,
                   proj: String,
                   publication: String| {
        let client = client.clone();
        let base = base.clone();
        async move {
            post(
                &client,
                &base,
                "/v1/policy-publications",
                &who,
                &json!({ "publication_id": publication, "proposal_id": prop,
                      "decision_id": dec, "approval_id": app,
                      "projection_id": proj, "owning_authority": grant }),
            )
            .await
        }
    };
    let (status, refused) = staging(
        mallory_id.clone(),
        mallory_grant.clone(),
        prop1.clone(),
        dec1.clone(),
        app1.clone(),
        proj1.clone(),
        pub1.clone(),
    )
    .await;
    assert_eq!(
        status, 400,
        "a foreign tenant stages no publication: {refused}"
    );
    assert!(
        refused["message"]
            .as_str()
            .unwrap_or_default()
            .contains(&prop1),
        "the refusal names the PROPOSAL as an absent one would: {refused}"
    );
    let (status, staged) = staging(
        alice_id.clone(),
        alice_grant.clone(),
        prop1.clone(),
        dec1.clone(),
        app1.clone(),
        proj1.clone(),
        pub1.clone(),
    )
    .await;
    assert_eq!(status, 200, "the owner still stages: {staged}");

    // ── ARM 2: `/effective` — Mallory holds her OWN live grant, so she is past
    //    `.9.2.1.2`'s authority gate and only the tenant binding refuses her.
    let effective_body = |grant: &str| {
        json!({
            "git_object_ids": object_ids.clone(), "repo_path": "live", "owning_authority": grant,
        })
    };
    let (status, refused) = post(
        &client,
        &base,
        &format!("/v1/policy-publications/{pub1}/effective"),
        &mallory_id,
        &effective_body(&mallory_grant),
    )
    .await;
    assert_eq!(
        status, 400,
        "a foreign tenant marks nothing effective: {refused}"
    );
    assert!(
        !refused["message"]
            .as_str()
            .unwrap_or_default()
            .contains("HOLDS"),
        "mallory is PAST the authority gate — the refusal must be the tenant one: {refused}"
    );

    // ── ARM 3: `/failed`, on the second publication, still staged.
    let (prop2, dec2, app2, proj2, pub2) = stage_chain(2).await;
    let (status, _) = staging(
        alice_id.clone(),
        alice_grant.clone(),
        prop2,
        dec2,
        app2,
        proj2,
        pub2.clone(),
    )
    .await;
    assert_eq!(status, 200, "the second publication stages");
    let (status, refused) = post(
        &client,
        &base,
        &format!("/v1/policy-publications/{pub2}/failed"),
        &mallory_id,
        // `.9.2.1.2.1`: mallory names HER OWN grant, so she is past the
        // authority gate and the refusal this arm measures is still the TENANT
        // one — the shape ARM 2 above already uses for `/effective`. Without
        // it this arm would keep asserting 400 and would have stopped
        // measuring containment.
        &json!({ "reason": "the foreign failure", "owning_authority": mallory_grant }),
    )
    .await;
    assert_eq!(
        status, 400,
        "a foreign tenant marks nothing failed: {refused}"
    );
    assert!(
        !refused["message"]
            .as_str()
            .unwrap_or_default()
            .contains("HOLDS"),
        "mallory is PAST the authority gate — the refusal must be the tenant one: {refused}"
    );
    let (status, failed) = post(
        &client,
        &base,
        &format!("/v1/policy-publications/{pub2}/failed"),
        &alice_id,
        &json!({ "reason": "the owner's failure", "owning_authority": alice_grant }),
    )
    .await;
    assert_eq!(status, 200, "the owner still marks failed: {failed}");
    assert_eq!(failed["state"], json!("failed"), "{failed}");

    // ── ARM 4: `/publish`, on the third publication, still staged.
    let (prop3, dec3, app3, proj3, pub3) = stage_chain(3).await;
    let (status, _) = staging(
        alice_id.clone(),
        alice_grant.clone(),
        prop3,
        dec3,
        app3,
        proj3,
        pub3.clone(),
    )
    .await;
    assert_eq!(status, 200, "the third publication stages");
    // ⛔ THIS ARM ASSERTS THE REPOSITORY, NOT ONLY THE STATUS, AND IT HAD TO.
    // Falsification found it passing for an unrelated reason: with the handler's
    // ownership check removed, the publish still ended at `mark_effective`,
    // whose own check refused Mallory — so the status was 400 while
    // `publisher::publish` had ALREADY written `refs/rb/publications/…` into the
    // git repository. A 400 after a completed side effect is not a refusal, and
    // only the repository tells the two apart
    // (`docs/knowledge/a-control-that-passes-for-an-unrelated-reason.md`).
    let published_ref = repo_root
        .join("live")
        .join("refs/rb/publications")
        .join(&pub3);
    let (status, refused) = post(
        &client,
        &base,
        &format!("/v1/policy-publications/{pub3}/publish"),
        &mallory_id,
        &json!({ "repo_path": "live", "owning_authority": mallory_grant }),
    )
    .await;
    assert_eq!(status, 400, "a foreign tenant publishes nothing: {refused}");
    assert!(
        !published_ref.exists(),
        "and writes NOTHING to the repository — a 400 after the ref was written \
         is not a refusal: {}",
        published_ref.display()
    );
    let (status, published) = post(
        &client,
        &base,
        &format!("/v1/policy-publications/{pub3}/publish"),
        &alice_id,
        &json!({ "repo_path": "live", "owning_authority": alice_grant }),
    )
    .await;
    assert_eq!(status, 200, "the owner still publishes: {published}");
    assert!(
        published_ref.exists(),
        "the owner's publish DOES write the ref, or the assertion above passes \
         because nothing ever writes it: {}",
        published_ref.display()
    );

    // The owner completes ARM 2's positive half, which also makes `pub1`
    // effective so the deployment arms have something to deploy.
    let (status, marked) = post(
        &client,
        &base,
        &format!("/v1/policy-publications/{pub1}/effective"),
        &alice_id,
        &effective_body(&alice_grant),
    )
    .await;
    assert_eq!(status, 200, "the owner still marks effective: {marked}");

    // ── ARMS 5 and 6: the deployment assignment and its receipt.
    let (status, _) = post(
        &client,
        &base,
        "/v1/deployment-targets",
        &alice_id,
        &json!({ "target_id": "gtn-target", "target_type": "repository",
                 "owning_authority": alice_grant, "reporter": alice_id }),
    )
    .await;
    assert_eq!(status, 200, "the target registers");
    let (desired_ref, desired_digest) = desired_pair(&pool, &pub1).await;
    let assignment = json!({
        "target_id": "gtn-target", "publication_id": pub1, "wave": 1,
        "desired_ref": desired_ref, "desired_digest": desired_digest,
    });
    // `SIGNOFF-REPAIR.9.3.3.7`: mallory holds no authority over alice's target,
    // so she is refused there, `403`, before any publication is looked up.
    let (status, refused) = post(&client, &base, "/v1/deployments", &mallory_id, &assignment).await;
    assert_eq!(
        status, 403,
        "a foreign tenant points alice's target at nothing: {refused}"
    );
    // …so the TENANT binding is exercised from PAST that gate: mallory's own
    // target, which she holds, pointed at alice's publication, is refused as an
    // absent publication, `400`.
    let (status, registered) = post(
        &client,
        &base,
        "/v1/deployment-targets",
        &mallory_id,
        &json!({ "target_id": "gtn-target-m", "target_type": "repository",
                 "owning_authority": mallory_grant, "reporter": mallory_id }),
    )
    .await;
    assert_eq!(
        status, 200,
        "mallory registers her own target: {registered}"
    );
    let mut foreign = assignment.clone();
    foreign["target_id"] = json!("gtn-target-m");
    let (status, refused) = post(&client, &base, "/v1/deployments", &mallory_id, &foreign).await;
    assert_eq!(
        status, 400,
        "mallory is PAST the authority gate — the refusal must be the tenant one: {refused}"
    );
    assert!(
        refused["message"]
            .as_str()
            .unwrap_or_default()
            .contains("does not exist"),
        "another tenant's publication answers as an absent one: {refused}"
    );
    let (status, assigned) = post(&client, &base, "/v1/deployments", &alice_id, &assignment).await;
    assert_eq!(status, 200, "the owner still deploys: {assigned}");
    let receipt = json!({ "observed_digest": DIGEST, "observed_state": "applied" });
    let receipt_path = format!("/v1/deployments/gtn-target/{pub1}/receipt");
    let (status, refused) = post(&client, &base, &receipt_path, &mallory_id, &receipt).await;
    assert_eq!(status, 400, "a foreign tenant files no receipt: {refused}");
    let (status, filed) = post(&client, &base, &receipt_path, &alice_id, &receipt).await;
    assert_eq!(status, 200, "the owner still files a receipt: {filed}");
    // `SIGNOFF-REPAIR.9.3.3.3`: the history is read by the publication's tenant,
    // and another tenant gets the answer an absent assignment gets.
    let history_path = format!("/v1/deployments/gtn-target/{pub1}/receipts");
    let (status, refused) = get(&client, &base, &history_path, &mallory_id).await;
    assert_eq!(status, 400, "a foreign tenant reads no receipts: {refused}");
    let absent_path = format!("/v1/deployments/gtn-target-absent/{pub1}/receipts");
    let (status, absent) = get(&client, &base, &absent_path, &mallory_id).await;
    assert_eq!(status, 400, "{absent}");
    assert_eq!(
        absent["message"]
            .as_str()
            .map(|m| m.replace("gtn-target-absent", "gtn-target")),
        refused["message"].as_str().map(str::to_string),
        "a foreign tenant cannot tell a deployed pair from an absent one: {absent} vs {refused}"
    );
    let (status, history) = get(&client, &base, &history_path, &alice_id).await;
    assert_eq!(status, 200, "the owner reads the history: {history}");
    assert_eq!(history.as_array().map(Vec::len), Some(1), "{history}");

    // ── ARMS 7, 8 and 9: drift, correction and outcome.
    let drift = json!({
        "drift_id": "gtn-drift", "target_id": "gtn-target", "publication_id": pub1,
        "category": "pending_rollout", "desired_digest": desired_digest, "observed_digest": null,
    });
    let (status, refused) = post(&client, &base, "/v1/policy-drift", &mallory_id, &drift).await;
    assert_eq!(status, 400, "a foreign tenant records no drift: {refused}");
    // ⛔ `SIGNOFF-REPAIR.6.1.5.3`: the refusal must not depend on whether the pair
    // is DEPLOYED. The assignment probe used to answer first, so a foreign caller
    // got `UnknownAssignment` for an undeployed pair and `UnknownPublication` for
    // a deployed one — two different answers, which is an existence oracle over
    // another tenant's rollout state.
    let mut undeployed = drift.clone();
    undeployed["drift_id"] = json!("gtn-drift-undeployed");
    undeployed["target_id"] = json!("gtn-target-absent");
    let (status, other) = post(&client, &base, "/v1/policy-drift", &mallory_id, &undeployed).await;
    assert_eq!(status, 400, "the undeployed pair also refuses: {other}");
    assert_eq!(
        other["message"], refused["message"],
        "a deployed and an undeployed pair give another tenant the SAME answer, \
         or the refusal enumerates her rollout state: {other} vs {refused}"
    );
    let (status, ok) = post(&client, &base, "/v1/policy-drift", &alice_id, &drift).await;
    assert_eq!(status, 200, "the owner still records drift: {ok}");

    let correction = |id: &str, grant: &str| {
        json!({
            "correction_id": id, "publication_id": pub1, "operation": "waiver",
            "authority_grant": grant, "expires_at": "2030-01-01T00:00:00Z",
            "reason": "the waiver",
        })
    };
    let (status, refused) = post(
        &client,
        &base,
        "/v1/policy-corrections",
        &mallory_id,
        &correction("gtn-corr-foreign", &mallory_grant),
    )
    .await;
    assert_eq!(status, 400, "a foreign tenant corrects nothing: {refused}");
    let (status, ok) = post(
        &client,
        &base,
        "/v1/policy-corrections",
        &alice_id,
        &correction("gtn-corr-own", &alice_grant),
    )
    .await;
    assert_eq!(status, 200, "the owner still corrects: {ok}");

    let outcome = |id: &str| {
        json!({
            "outcome_id": id, "publication_id": pub1, "kind": "incident",
            "review_trigger": "adverse_threshold", "note": "the outcome",
        })
    };
    let (status, refused) = post(
        &client,
        &base,
        "/v1/policy-outcomes",
        &mallory_id,
        &outcome("gtn-out-foreign"),
    )
    .await;
    assert_eq!(
        status, 400,
        "a foreign tenant records no outcome: {refused}"
    );
    let (status, ok) = post(
        &client,
        &base,
        "/v1/policy-outcomes",
        &alice_id,
        &outcome("gtn-out-own"),
    )
    .await;
    assert_eq!(status, 200, "the owner still records an outcome: {ok}");

    // ── ARM 10: the review SCHEDULE, which names no id and so cannot refuse —
    //    it SCOPES. Mallory's schedule must materialise nothing at all, and the
    //    assertion is about the TABLE rather than the response, because a verb
    //    that silently wrote and returned nothing would pass a response check.
    let (status, scheduled) = post(
        &client,
        &base,
        "/v1/policy-reviews/schedule",
        &mallory_id,
        &json!({}),
    )
    .await;
    assert_eq!(
        status, 200,
        "a foreign tenant's schedule is admitted: {scheduled}"
    );
    assert_eq!(
        scheduled,
        json!([]),
        "and schedules NOTHING, because it reads only its own drift and outcomes: {scheduled}"
    );
    let rows: i64 = sqlx::query_scalar("SELECT count(*) FROM policy_reviews")
        .fetch_one(&pool)
        .await
        .expect("the reviews are countable");
    assert_eq!(
        rows, 0,
        "a foreign schedule materialises no row for anyone — the response alone \
         would not have caught a write it did not report"
    );
    let (status, scheduled) = post(
        &client,
        &base,
        "/v1/policy-reviews/schedule",
        &alice_id,
        &json!({}),
    )
    .await;
    assert_eq!(status, 200, "the owner still schedules: {scheduled}");
    let scheduled = scheduled.as_array().unwrap().clone();
    assert!(
        !scheduled.is_empty(),
        "the owner's schedule materialises her reviews, or ARM 11 is vacuous"
    );

    // ── ARM 11: closing a review.
    let review_id = scheduled[0]["review_id"].as_str().unwrap().to_string();
    let (status, refused) = post(
        &client,
        &base,
        &format!("/v1/policy-reviews/{review_id}/done"),
        &mallory_id,
        &json!({}),
    )
    .await;
    assert_eq!(status, 400, "a foreign tenant closes no review: {refused}");
    let (status, done) = post(
        &client,
        &base,
        &format!("/v1/policy-reviews/{review_id}/done"),
        &alice_id,
        &json!({}),
    )
    .await;
    assert_eq!(status, 200, "the owner still closes her review: {done}");
    assert_eq!(done["status"], json!("done"), "{done}");

    let _ = std::fs::remove_dir_all(&repo_root);
}

/// `SIGNOFF-REPAIR.6.1.5.3` — every lifecycle read is bound to the caller's
/// tenant, and the binding runs BOTH WAYS ROUND.
///
/// ⭐ **Two full chains, not one.** A repair that returned nothing to anybody
/// would pass a one-sided control: Alice's rows really would be absent from
/// Mallory's list. So both tenants build the whole chain — proposal, decision,
/// approval, projection, publication, deployment, receipt, drift, correction,
/// outcome, review — and every table is asserted twice: the other tenant's row
/// is absent AND this tenant's own row is present. That is `.3.5.5`'s shape and
/// `.7.1.2.2`'s, applied to ten tables at once.
///
/// ⚠️ `GET /v1/policies` and the MCP `policy_bundle` are deliberately NOT bound
/// and are asserted so: `policy_versions` is the governance LIBRARY, site-wide by
/// design (DOC-0071), and a policy only its author can read is not governance.
/// `.6.1.5.4` owns its WRITE, which is the half that is actually wrong.
#[tokio::test]
async fn every_lifecycle_read_is_bound_to_its_own_tenant() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let (repo_root, object_ids) = seeded_publication_repository("rdb");
    let server = TestServer::start_with_publication_root(&pool, &repo_root).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let enrol = |name: &'static str| {
        let client = client.clone();
        let base = base.clone();
        async move {
            let (status, who) =
                enroll(&client, &base, json!({ "kind": "human", "name": name })).await;
            assert_eq!(status, 200, "{name} enrols: {who}");
            (
                who["principal_id"].as_str().unwrap().to_string(),
                who["tenant_id"].as_str().unwrap().to_string(),
            )
        }
    };
    let (alice_id, alice_tenant) = enrol("rdb-alice").await;
    let (mallory_id, mallory_tenant) = enrol("rdb-mallory").await;
    allow_owner_decides(&pool, &alice_tenant).await;
    allow_owner_decides(&pool, &mallory_tenant).await;
    // `SIGNOFF-REPAIR.6.1.5.4`: registering a policy version is a site act.
    // This fixture seeds the governance library, so it holds the capability —
    // issued through the deployment-controlled service, never by a row insert.
    site_fixture::provision(
        &pool,
        &alice_id,
        &[reasonbraid_server::site_authority::Action::PolicyRegister],
    )
    .await;
    assert_ne!(
        alice_tenant, mallory_tenant,
        "two enrolments must be two tenants, or this control measures nothing"
    );

    // ⛔ ONE policy, registered once and named by both chains. `policy_versions`
    // is the shared library by DOC-0071's decision, so a control that gave each
    // tenant its own policy would quietly assume the opposite of what this leaf
    // deliberately leaves unbound.
    let (status, _) = register_policy(
        &client,
        &base,
        &alice_id,
        &json!({
            "policy_id": "rdb-policy", "version": "1.0.0",
            "lifecycle": "draft", "title": "rdb",
            "owning_authority": format!("grt_{alice_id}"),
            "clauses": [ { "id": "c1", "statement": "the shared clause" } ],
        }),
    )
    .await;
    assert_eq!(status, 200, "the shared policy registers");

    // The whole chain for one tenant, so both are built by the same code and a
    // difference between them cannot be an accident of the fixture.
    let chain = |who: String, tenant: String, tag: &'static str| {
        let client = client.clone();
        let base = base.clone();
        let pool = pool.clone();
        let object_ids = object_ids.clone();
        async move {
            let grant = format!("grt_{who}");
            let (_status, created) = post(
                &client,
                &base,
                "/v1/threads",
                &who,
                &json!({
                    "protocol_version": reasonbraid_core::PROTOCOL_VERSION,
                    "operation": "thread.create",
                    "request_id": reasonbraid_core::RequestId::new().to_string(),
                    "idempotency_key": format!("{tag}-create"),
                    "body": { "tenant_id": tenant, "subject": tag, "objective": "probe",
                              "workflow_profile": "independent_panel",
                              "decision_rule": "owner_decides" },
                    "client_context": {},
                }),
            )
            .await;
            let thread_id = created["thread_id"].as_str().unwrap().to_string();
            let command = |key: String, operation: &'static str, body: Value| {
                let client = client.clone();
                let base = base.clone();
                let who = who.clone();
                let thread_id = thread_id.clone();
                async move {
                    let response = client
                        .post(format!("{base}/v1/threads/{thread_id}/commands"))
                        .header(PRINCIPAL_HEADER, &who)
                        .json(&json!({
                            "protocol_version": reasonbraid_core::PROTOCOL_VERSION,
                            "operation": operation,
                            "request_id": reasonbraid_core::RequestId::new().to_string(),
                            "idempotency_key": key,
                            "body": body, "client_context": {},
                        }))
                        .send()
                        .await
                        .expect("command request");
                    let text = response.text().await.expect("command body");
                    serde_json::from_str::<Value>(&text).unwrap_or_else(|_| json!({ "raw": text }))
                }
            };
            let _ = command(
                format!("{tag}-advance"),
                "thread.advance_round",
                json!({ "tenant_id": tenant }),
            )
            .await;
            let verdict = command(
                format!("{tag}-verdict"),
                "thread.contribute",
                json!({ "tenant_id": tenant, "content": "judged", "kind": "verdict",
                        "verdict": { "target_digest": "sha256:00",
                                     "outcome": "accepted_by_rule" } }),
            )
            .await;
            let verdict_event = verdict["event_id"].as_str().unwrap().to_string();
            let _ = command(
                format!("{tag}-close"),
                "thread.close",
                json!({ "tenant_id": tenant, "reason": "the owner decides" }),
            )
            .await;

            let ids = |suffix: &str| format!("{tag}-{suffix}");
            for (path, body) in [
                (
                    "/v1/policy-proposals",
                    json!({
                    "proposal_id": ids("prop"), "policy_id": "rdb-policy",
                    "policy_version": "1.0.0", "thread_id": thread_id }),
                ),
                (
                    "/v1/policy-decisions",
                    json!({
                    "decision_id": ids("dec"), "proposal_id": ids("prop"), "rule": "owner_decides",
                    "electorate": { "participants": [who], "denominator": 1, "abstentions": [] },
                    "verdict_event_id": verdict_event }),
                ),
                (
                    "/v1/policy-approvals",
                    json!({
                    "approval_id": ids("app"), "proposal_id": ids("prop"),
                    "decision_id": ids("dec"), "approver": who, "grant_id": grant,
                    "quorum": { "participants": [who], "denominator": 1, "abstentions": [] } }),
                ),
            ] {
                let (status, out) = post(&client, &base, path, &who, &body).await;
                assert_eq!(status, 200, "{tag}: {path} — {out}");
            }
            let (status, projection) = post(
                &client, &base, "/v1/policy-projections", &who,
                &json!({ "projection_id": ids("proj"), "target": "generic",
                         "resolution": { "policies": [ { "policy_id": "rdb-policy", "version": "1.0.0" } ],
                                         "target": { "layer": "organization", "target": "*" } } }),
            ).await;
            assert_eq!(status, 200, "{tag}: the projection — {projection}");
            let (status, out) = post(
                &client,
                &base,
                "/v1/policy-publications",
                &who,
                &json!({ "publication_id": ids("pub"), "proposal_id": ids("prop"),
                         "decision_id": ids("dec"), "approval_id": ids("app"),
                         "projection_id": ids("proj"), "owning_authority": grant }),
            )
            .await;
            assert_eq!(status, 200, "{tag}: the publication — {out}");
            let (status, out) = post(
                &client,
                &base,
                &format!("/v1/policy-publications/{}/effective", ids("pub")),
                &who,
                &json!({ "git_object_ids": object_ids, "repo_path": "live",
                         "owning_authority": grant }),
            )
            .await;
            assert_eq!(status, 200, "{tag}: effective — {out}");
            let (desired_ref, desired_digest) = desired_pair(&pool, &ids("pub")).await;
            for (path, body) in [
                (
                    "/v1/deployment-targets".to_string(),
                    json!({
                    "target_id": ids("target"), "target_type": "repository",
                    "owning_authority": grant, "reporter": who }),
                ),
                (
                    "/v1/deployments".to_string(),
                    json!({
                    "target_id": ids("target"), "publication_id": ids("pub"), "wave": 1,
                    "desired_ref": desired_ref, "desired_digest": desired_digest }),
                ),
                (
                    format!("/v1/deployments/{}/{}/receipt", ids("target"), ids("pub")),
                    json!({
                    "observed_digest": DIGEST, "observed_state": "applied" }),
                ),
                (
                    "/v1/policy-drift".to_string(),
                    json!({
                    "drift_id": ids("drift"), "target_id": ids("target"),
                    "publication_id": ids("pub"), "category": "pending_rollout",
                    "desired_digest": desired_digest, "observed_digest": null }),
                ),
                (
                    "/v1/policy-corrections".to_string(),
                    json!({
                    "correction_id": ids("corr"), "publication_id": ids("pub"),
                    "operation": "waiver", "authority_grant": grant,
                    "expires_at": "2030-01-01T00:00:00Z", "reason": "the waiver" }),
                ),
                (
                    "/v1/policy-outcomes".to_string(),
                    json!({
                    "outcome_id": ids("out"), "publication_id": ids("pub"), "kind": "incident",
                    "review_trigger": "adverse_threshold", "note": "the outcome" }),
                ),
                ("/v1/policy-reviews/schedule".to_string(), json!({})),
            ] {
                let (status, out) = post(&client, &base, &path, &who, &body).await;
                assert_eq!(status, 200, "{tag}: {path} — {out}");
            }
            (thread_id, verdict_event)
        }
    };
    let (alice_thread, alice_verdict) =
        chain(alice_id.clone(), alice_tenant.clone(), "rdb-a").await;
    let _ = chain(mallory_id.clone(), mallory_tenant.clone(), "rdb-m").await;

    // ── `SIGNOFF-REPAIR.9.3.3.7`: a target is pointed at a publication by whoever
    //    holds the TARGET's owning authority. Targets are site-wide, so without
    //    that check mallory could assign HER OWN effective publication to alice's
    //    target: her tenant owns the assignment, and alice's target now carries a
    //    desired state alice's authority never set.
    let (desired_ref, desired_digest) = desired_pair(&pool, "rdb-m-pub").await;
    let (status, refused) = post(
        &client,
        &base,
        "/v1/deployments",
        &mallory_id,
        &json!({
            "target_id": "rdb-a-target", "publication_id": "rdb-m-pub", "wave": 1,
            "desired_ref": desired_ref, "desired_digest": desired_digest,
        }),
    )
    .await;
    assert_eq!(
        status, 403,
        "a principal without the target's authority points it at nothing: {refused}"
    );
    assert_eq!(refused["code"], json!("unauthorized"), "{refused}");

    // ── `publications::stage` reads `policy_projections`, and that read is one
    //    of the 32. ⛔ It probed EXISTENCE only, so a publication could be staged
    //    against ANOTHER tenant's projection — its compiled bytes and its
    //    declared unrepresentables, which are a function of that tenant's own
    //    resolution request. Falsification found this predicate untested: every
    //    other arm here reads a list, and no control staged across the boundary.
    for (suffix, projection, expected, note) in [
        (
            "x1",
            "rdb-m-proj",
            400_u16,
            "another tenant's projection is refused",
        ),
        (
            "x2",
            "rdb-a-proj",
            200_u16,
            "and the owner's own projection still stages",
        ),
    ] {
        let prop = format!("rdb-a-{suffix}-prop");
        let dec = format!("rdb-a-{suffix}-dec");
        let app = format!("rdb-a-{suffix}-app");
        for (path, payload) in [
            (
                "/v1/policy-proposals",
                json!({
                "proposal_id": prop, "policy_id": "rdb-policy",
                "policy_version": "1.0.0", "thread_id": alice_thread }),
            ),
            (
                "/v1/policy-decisions",
                json!({
                "decision_id": dec, "proposal_id": prop, "rule": "owner_decides",
                "electorate": { "participants": [alice_id], "denominator": 1, "abstentions": [] },
                "verdict_event_id": alice_verdict }),
            ),
            (
                "/v1/policy-approvals",
                json!({
                "approval_id": app, "proposal_id": prop, "decision_id": dec,
                "approver": alice_id, "grant_id": format!("grt_{alice_id}"),
                "quorum": { "participants": [alice_id], "denominator": 1, "abstentions": [] } }),
            ),
        ] {
            let (status, out) = post(&client, &base, path, &alice_id, &payload).await;
            assert_eq!(status, 200, "the {suffix} chain: {path} — {out}");
        }
        let (status, out) = post(
            &client,
            &base,
            "/v1/policy-publications",
            &alice_id,
            &json!({ "publication_id": format!("rdb-a-{suffix}-pub"), "proposal_id": prop,
                     "decision_id": dec, "approval_id": app, "projection_id": projection,
                     "owning_authority": format!("grt_{alice_id}") }),
        )
        .await;
        assert_eq!(status, expected, "{note}: {out}");
    }

    // Every list verb, with the id field that identifies its rows.
    let surfaces: [(&str, &str, &str); 10] = [
        ("/v1/policy-proposals", "proposal_id", "prop"),
        ("/v1/policy-decisions", "decision_id", "dec"),
        ("/v1/policy-approvals", "approval_id", "app"),
        ("/v1/policy-projections", "projection_id", "proj"),
        ("/v1/policy-publications", "publication_id", "pub"),
        ("/v1/deployments", "publication_id", "pub"),
        ("/v1/policy-drift", "drift_id", "drift"),
        ("/v1/policy-corrections", "correction_id", "corr"),
        ("/v1/policy-outcomes", "outcome_id", "out"),
        ("/v1/policy-reviews", "publication_id", "pub"),
    ];
    for (path, field, suffix) in surfaces {
        for (reader, own_tag, other_tag) in [
            (&alice_id, "rdb-a", "rdb-m"),
            (&mallory_id, "rdb-m", "rdb-a"),
        ] {
            let (status, rows) = get(&client, &base, path, reader).await;
            assert_eq!(status, 200, "{path} answers {reader}: {rows}");
            let rows = rows.as_array().expect("a list").clone();
            let mine = format!("{own_tag}-{suffix}");
            let theirs = format!("{other_tag}-{suffix}");
            assert!(
                rows.iter().any(|r| r[field] == json!(mine)),
                "{path}: the owner still reads its own `{mine}` — a repair that \
                 returned nothing to anybody would pass the next assertion: {rows:?}"
            );
            assert!(
                !rows.iter().any(|r| r[field] == json!(theirs)),
                "{path}: another tenant's `{theirs}` is absent: {rows:?}"
            );
        }
    }

    // ⛔ THE LIBRARY IS DELIBERATELY NOT BOUND, and the control says so out loud:
    // `policy_versions` is site-wide by design (DOC-0071) and both tenants read
    // the one policy. A future repair that bound this read would be reversing a
    // recorded decision, and it would fail here rather than silently.
    for reader in [&alice_id, &mallory_id] {
        let (status, policies) = get(&client, &base, "/v1/policies", reader).await;
        assert_eq!(status, 200, "the library answers {reader}: {policies}");
        assert!(
            policies
                .as_array()
                .unwrap()
                .iter()
                .any(|p| p["policy_id"] == json!("rdb-policy")),
            "the governance LIBRARY stays readable by the tenants it governs — \
             binding it would reverse DOC-0071: {policies}"
        );
    }

    let _ = std::fs::remove_dir_all(&repo_root);
}

/// `SIGNOFF-REPAIR.6.1.5.4` — the policy LIBRARY takes site-operator authority.
///
/// 🔴 **Reproduced at runtime against the shipped server before the repair was
/// written**, in all four legs this control now refuses: an enrolled principal
/// in tenant B registered `red-org-baseline 1.0.0` (200), tenant A's own
/// registration at that coordinate was refused 400 `already exists`, tenant A
/// then READ tenant B's clause `"a publication needs no authority"` as the
/// organization baseline, and tenant B appended a `2.0.0` that withdrew the
/// authority clause — with no site authority of any kind.
///
/// ⚠️ **Stated at its real width.** Unlike the workflow registry
/// (`SIGNOFF-REPAIR.7.1.2.1`), `policy::resolve` names an EXPLICIT
/// `(policy_id, version)` pair, so a foreign registration does not silently
/// re-shape someone else's deliberation. What it does is take a coordinate the
/// rightful author then cannot use, and put text under a governance id that
/// every enrolled principal reads. This control asserts that, and not more.
#[tokio::test]
async fn the_policy_library_takes_site_operator_authority() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, operator) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "lib-operator" }),
    )
    .await;
    assert_eq!(status, 200, "the operator enrols: {operator}");
    let operator_id = operator["principal_id"].as_str().unwrap().to_string();
    let (status, alice) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "lib-alice" }),
    )
    .await;
    assert_eq!(status, 200, "alice enrols: {alice}");
    let alice_id = alice["principal_id"].as_str().unwrap().to_string();
    let (status, mallory) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "lib-mallory" }),
    )
    .await;
    assert_eq!(status, 200, "mallory enrols: {mallory}");
    let mallory_id = mallory["principal_id"].as_str().unwrap().to_string();

    // Only the operator holds the capability. Alice and Mallory are ordinary
    // enrolled principals in two different tenants, exactly as before.
    site_fixture::provision(
        &pool,
        &operator_id,
        &[reasonbraid_server::site_authority::Action::PolicyRegister],
    )
    .await;

    let document = |who: &str, version: &str, statement: &str| {
        json!({
            "policy_id": "lib-org-baseline",
            "version": version,
            "lifecycle": "active",
            "title": "the organization baseline",
            "owning_authority": format!("grt_{who}"),
            "clauses": [ { "id": "c1", "statement": statement } ],
        })
    };

    // The library's legitimate content, registered by the operator.
    let (status, seeded) = register_policy(
        &client,
        &base,
        &operator_id,
        &document(
            &operator_id,
            "1.0.0",
            "every publication names its authority",
        ),
    )
    .await;
    assert_eq!(status, 200, "the capable operator registers: {seeded}");

    // ── THE REFUSAL ──────────────────────────────────────────────────────────
    // Leg 1: Mallory cannot take a coordinate in the shared namespace.
    let (status, refused) = register_policy(
        &client,
        &base,
        &mallory_id,
        &document(&mallory_id, "2.0.0", "the authority clause is withdrawn"),
    )
    .await;
    assert_eq!(
        status, 403,
        "an enrolled principal with no site grant registers no policy: {refused}"
    );
    assert_eq!(
        refused["code"],
        json!("site_authority_required"),
        "{refused}"
    );
    assert!(
        refused["audit_id"]
            .as_str()
            .is_some_and(|id| !id.is_empty()),
        "a refused site act still records that it was attempted: {refused}"
    );

    // Leg 2: and the refusal is a REFUSAL, not a silent success — the version
    // Mallory asked for is absent from the library every tenant reads.
    let (status, library) = get(&client, &base, "/v1/policies", &alice_id).await;
    assert_eq!(status, 200, "the library answers Alice: {library}");
    let rows: Vec<&Value> = library
        .as_array()
        .unwrap()
        .iter()
        .filter(|p| p["policy_id"] == json!("lib-org-baseline"))
        .collect();
    assert_eq!(
        rows.len(),
        1,
        "exactly the operator's version exists: {rows:?}"
    );
    assert_eq!(rows[0]["version"], json!("1.0.0"), "{rows:?}");
    assert_eq!(
        rows[0]["clauses"][0]["statement"],
        json!("every publication names its authority"),
        "the governance text is the operator's, not a foreign tenant's: {rows:?}"
    );

    // Leg 3: Alice is refused on exactly the same footing. ⛔ The gate is the
    // CAPABILITY, never the tenant — a control in which Alice succeeded would be
    // measuring tenancy and would pass under a repair that scoped the library
    // per tenant, which DOC-0071 explicitly rejected.
    let (status, refused) = register_policy(
        &client,
        &base,
        &alice_id,
        &document(&alice_id, "1.1.0", "alice's own amendment"),
    )
    .await;
    assert_eq!(status, 403, "alice holds no capability either: {refused}");

    // Leg 4: the reason is REQUIRED, not merely accepted — the helper injects
    // one, so this is the single place the wire contract is proved.
    let mut bodyless = document(&operator_id, "1.2.0", "no reason given");
    bodyless.as_object_mut().unwrap().remove("reason");
    let (status, rejected) = post(&client, &base, "/v1/policies", &operator_id, &bodyless).await;
    assert_eq!(
        status, 400,
        "a site act without a reason is not a site act: {rejected}"
    );

    // ── THE READS ARE UNCHANGED, and the control says so out loud ────────────
    // ⛔ DOC-0071 decided the library is READABLE by the tenants it governs. A
    // later repair that bound these would fail here rather than silently reverse
    // a recorded decision — the same guard `.6.1.5.3` put on the list read.
    let request = json!({
        "policies": [ { "policy_id": "lib-org-baseline", "version": "1.0.0" } ],
        "target": { "layer": "organization", "target": "anything" },
    });
    for (reader, who) in [(&alice_id, "alice"), (&mallory_id, "mallory")] {
        let (status, resolved) =
            post(&client, &base, "/v1/policies/resolve", reader, &request).await;
        assert_eq!(status, 200, "{who} resolves the shared library: {resolved}");
        assert_eq!(
            resolved["resolved"][0]["statement"],
            json!("every publication names its authority"),
            "{who} reads the governance it is governed by: {resolved}"
        );
        let (status, impact) = get(
            &client,
            &base,
            "/v1/policies/lib-org-baseline/1.0.0/impact",
            reader,
        )
        .await;
        assert_eq!(status, 200, "{who} reads the impact map: {impact}");
    }

    sqlx::query("DELETE FROM policy_versions WHERE policy_id = 'lib-org-baseline'")
        .execute(&pool)
        .await
        .expect("drop the fixture rows");
}

/// `SIGNOFF-REPAIR.9.1.2`: the owning authority a registrar names is a grant the
/// REGISTRAR HOLDS.
///
/// Holding `policy_register` lets a principal write the site's library. It
/// never let them attach somebody else's authority to what they wrote, yet
/// until this leaf `policy::register` asked only whether the named grant was
/// live and covering, never whose it was. A site operator could therefore name
/// another tenant's grant as the owner of a document that tenant never wrote,
/// and the publication verbs then treat that grant as the policy's owner.
/// Every other site that cites an authority already asks
/// `authority::grant_held_by` (`SIGNOFF-REPAIR.9.3.1`); this was the last one
/// that did not.
///
/// ⭐ The refusal and the admission are a matched pair: the same registrar, the
/// same document, the same coordinate, and only the grant's holder differs.
/// The last leg is the path the old comment worried the rule would close, a
/// policy owned by someone other than the operator. It stays open by the owner
/// acting for themselves.
#[tokio::test]
async fn the_owning_authority_is_a_grant_the_registrar_holds() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, operator) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "held-operator" }),
    )
    .await;
    assert_eq!(status, 200, "the operator enrols: {operator}");
    let operator_id = operator["principal_id"].as_str().unwrap().to_string();
    let (status, owner) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "held-owner" }),
    )
    .await;
    assert_eq!(status, 200, "the owner enrols: {owner}");
    let owner_id = owner["principal_id"].as_str().unwrap().to_string();
    site_fixture::provision(
        &pool,
        &operator_id,
        &[reasonbraid_server::site_authority::Action::PolicyRegister],
    )
    .await;

    let document = |authority: &str, version: &str| {
        json!({
            "policy_id": "held-owner-baseline",
            "version": version,
            "lifecycle": "active",
            "title": "a baseline its owner never wrote",
            "owning_authority": authority,
            "clauses": [ { "id": "c1", "statement": "every publication names its authority" } ],
        })
    };
    let owner_grant = format!("grt_{owner_id}");
    let operator_grant = format!("grt_{operator_id}");

    // Leg 1, THE REFUSAL: the operator names the other principal's live grant,
    // which covers `policy_version_register`, and is refused on the act's own
    // terms: 400 with an audit id, since the caller did pass the site gate.
    let (status, refused) = register_policy(
        &client,
        &base,
        &operator_id,
        &document(&owner_grant, "1.0.0"),
    )
    .await;
    assert_eq!(
        status, 400,
        "a registrar cannot name an authority it does not hold: {refused}"
    );
    assert!(refused["audit_id"].is_string(), "{refused}");
    assert_eq!(
        refused["code"],
        json!(OWNING_AUTHORITY_REFUSAL),
        "the refusal names the rule: {refused}"
    );

    // Leg 2: nothing was written, so the library carries no row attributing
    // that document to the other principal's authority.
    let (status, library) = get(&client, &base, "/v1/policies", &owner_id).await;
    assert_eq!(status, 200, "the library answers: {library}");
    assert!(
        !library
            .as_array()
            .unwrap()
            .iter()
            .any(|p| p["policy_id"] == json!("held-owner-baseline")),
        "the refused registration left no row: {library}"
    );

    // Leg 3, THE MATCHED ADMISSION: the same registrar, document and
    // coordinate, naming the grant it holds.
    let (status, registered) = register_policy(
        &client,
        &base,
        &operator_id,
        &document(&operator_grant, "1.0.0"),
    )
    .await;
    assert_eq!(status, 200, "a registrar names its own grant: {registered}");
    assert_eq!(registered["owning_authority"], json!(operator_grant));

    // Leg 4: a policy owned by someone other than the operator is still
    // possible, by its owner registering it.
    site_fixture::provision(
        &pool,
        &owner_id,
        &[reasonbraid_server::site_authority::Action::PolicyRegister],
    )
    .await;
    let (status, registered) =
        register_policy(&client, &base, &owner_id, &document(&owner_grant, "1.1.0")).await;
    assert_eq!(
        status, 200,
        "the owner registers under its own grant: {registered}"
    );
    assert_eq!(registered["owning_authority"], json!(owner_grant));

    sqlx::query("DELETE FROM policy_versions WHERE policy_id = 'held-owner-baseline'")
        .execute(&pool)
        .await
        .expect("drop the fixture rows");
}

/// The canonical form the book states for a policy digest, written out here by
/// a second route rather than imported from the server: compact JSON, object
/// keys sorted by byte order at every depth, arrays in submitted order.
fn canonical_policy_json(value: &Value, out: &mut String) {
    match value {
        Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            out.push('{');
            for (i, key) in keys.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                out.push_str(&serde_json::to_string(key).unwrap());
                out.push(':');
                canonical_policy_json(&map[key.as_str()], out);
            }
            out.push('}');
        }
        Value::Array(items) => {
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                canonical_policy_json(item, out);
            }
            out.push(']');
        }
        leaf => out.push_str(&serde_json::to_string(leaf).unwrap()),
    }
}

/// The digest the book says the server derives for a submitted document: every
/// field except `reason`, `digest` and `lifecycle`, with an omitted text field
/// read as `""` and an omitted list as `[]`, hashed over the canonical form.
fn expected_policy_digest(document: &Value) -> String {
    use sha2::Digest as _;
    let mut content = serde_json::Map::new();
    for field in [
        "policy_id",
        "version",
        "title",
        "intent",
        "rationale",
        "domain",
        "risk_class",
        "owning_authority",
    ] {
        let value = document.get(field).cloned().unwrap_or(json!(""));
        content.insert(field.to_string(), value);
    }
    for field in [
        "clauses",
        "applicability",
        "non_applicability",
        "dependencies",
        "conflicts",
        "precedence_hints",
        "exceptions",
        "provenance",
    ] {
        let value = document.get(field).cloned().unwrap_or(json!([]));
        content.insert(field.to_string(), value);
    }
    let mut canonical = String::new();
    canonical_policy_json(&Value::Object(content), &mut canonical);
    format!("sha256:{:x}", sha2::Sha256::digest(canonical.as_bytes()))
}

/// `SIGNOFF-REPAIR.9.1.3`: a policy's digest is DERIVED from its document.
///
/// §15.1 gives every policy version an *immutable digest*, and until this leaf
/// the registry stored whatever `sha256:<64 hex>` the caller typed: two versions
/// with different clauses could carry one digest, and a digest could hash
/// nothing at all. The server now derives it. A declared digest is optional,
/// and when present it must be the document's own.
///
/// ⭐ The expected digests here are computed by `expected_policy_digest`, which
/// implements the rule the book states without calling the server's code. A
/// server that hashed some other form would disagree with it.
#[tokio::test]
async fn the_policy_digest_is_derived_from_the_document() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, human) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "digest-human" }),
    )
    .await;
    assert_eq!(status, 200, "the human enrolls: {human}");
    let human_id = human["principal_id"].as_str().unwrap().to_string();
    site_fixture::provision(
        &pool,
        &human_id,
        &[reasonbraid_server::site_authority::Action::PolicyRegister],
    )
    .await;
    let grant_id = format!("grt_{human_id}");

    let document = |version: &str, statement: &str| {
        json!({
            "policy_id": "derived-digest",
            "version": version,
            "lifecycle": "active",
            "title": "the derived digest",
            "owning_authority": grant_id,
            "clauses": [ { "id": "c1", "statement": statement } ],
            // Unsorted keys, nesting and number forms, so a server hashing the
            // bytes as sent, or re-ordering differently, disagrees here.
            "provenance": [ { "z": 1, "a": { "y": 0.1, "b": 1e2, "big": 9007199254740993u64 } } ],
        })
    };

    // Leg 1, THE REFUSAL: a declared digest that is not the document's.
    let mut forged = document("1.0.0", "every thread declares its objective");
    forged["digest"] = json!(DIGEST);
    let (status, refused) = register_policy(&client, &base, &human_id, &forged).await;
    assert_eq!(
        status, 400,
        "a digest that is not the document's refuses: {refused}"
    );
    let first = document("1.0.0", "every thread declares its objective");
    let first_digest = expected_policy_digest(&first);
    assert!(
        refused["message"]
            .as_str()
            .is_some_and(|m| m.contains(DIGEST) && m.contains(&first_digest)),
        "the refusal names the declared and the derived digest: {refused}"
    );

    // Leg 2: with no digest declared, the server derives it.
    let (status, registered) = register_policy(&client, &base, &human_id, &first).await;
    assert_eq!(status, 200, "an undeclared digest is derived: {registered}");
    assert_eq!(registered["digest"], json!(first_digest), "{registered}");

    // Leg 3: different clauses under a new version get a different digest.
    let second = document("1.1.0", "every publication names its authority");
    let (status, registered) = register_policy(&client, &base, &human_id, &second).await;
    assert_eq!(status, 200, "the second version registers: {registered}");
    let second_digest = expected_policy_digest(&second);
    assert_ne!(first_digest, second_digest);
    assert_eq!(registered["digest"], json!(second_digest), "{registered}");

    // Leg 4: declaring the document's own digest is accepted.
    let mut pinned = document("1.2.0", "every publication names its authority");
    let pinned_digest = expected_policy_digest(&pinned);
    pinned["digest"] = json!(pinned_digest);
    let (status, registered) = register_policy(&client, &base, &human_id, &pinned).await;
    assert_eq!(
        status, 200,
        "a correct declaration is accepted: {registered}"
    );
    assert_eq!(registered["digest"], json!(pinned_digest), "{registered}");

    // Leg 5: a row stored before this leaf, with a declared digest nobody
    // derived, reads as unverified; every derived row reads as verified.
    sqlx::query(
        "INSERT INTO policy_versions (policy_id, version, digest, lifecycle, title, \
         owning_authority, clauses) \
         VALUES ('derived-digest', '0.9.0', $1, 'active', 'a legacy row', $2, \
         '[{\"id\": \"c1\", \"statement\": \"stored with a declared digest\"}]'::jsonb)",
    )
    .bind(DIGEST)
    .bind(&grant_id)
    .execute(&pool)
    .await
    .expect("the legacy row inserts");
    let (status, library) = get(&client, &base, "/v1/policies", &human_id).await;
    assert_eq!(status, 200, "the library answers: {library}");
    let rows: Vec<&Value> = library
        .as_array()
        .unwrap()
        .iter()
        .filter(|p| p["policy_id"] == json!("derived-digest"))
        .collect();
    assert_eq!(rows.len(), 4, "{rows:?}");
    for row in rows {
        let expected = row["version"] != json!("0.9.0");
        assert_eq!(
            row["digest_verified"],
            json!(expected),
            "only a digest the stored document hashes to is verified: {row}"
        );
    }

    sqlx::query("DELETE FROM policy_versions WHERE policy_id = 'derived-digest'")
        .execute(&pool)
        .await
        .expect("drop the fixture rows");
}

/// `SIGNOFF-REPAIR.9.1.4`: the published `policy.lock` records what was
/// RESOLVED, from the registry, and nothing the caller wrote.
///
/// §15.1 makes the lock the record of every version, digest and grant basis a
/// resolution consumed. The projection request used to carry its own `lock`
/// rows and the `lock` target rendered them verbatim, so a tenant could publish
/// a lock naming a policy it never resolved, a digest nobody registered, or
/// another principal's grant. The server now writes the rows itself: one per
/// policy the request named, with the registry's derived digest and owner.
#[tokio::test]
async fn the_published_lock_records_what_was_resolved() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, alice) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "lock-alice" }),
    )
    .await;
    assert_eq!(status, 200, "alice enrols: {alice}");
    let alice_id = alice["principal_id"].as_str().unwrap().to_string();
    let (status, bob) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "lock-bob" }),
    )
    .await;
    assert_eq!(status, 200, "bob enrols: {bob}");
    let bob_id = bob["principal_id"].as_str().unwrap().to_string();
    site_fixture::provision(
        &pool,
        &alice_id,
        &[reasonbraid_server::site_authority::Action::PolicyRegister],
    )
    .await;
    let alice_grant = format!("grt_{alice_id}");
    let (status, registered) = register_policy(
        &client,
        &base,
        &alice_id,
        &json!({
            "policy_id": "lock-real",
            "version": "1.0.0",
            "lifecycle": "active",
            "title": "the policy the lock must name",
            "owning_authority": alice_grant,
            "clauses": [ { "id": "c1", "statement": "every publication names its authority" } ],
        }),
    )
    .await;
    assert_eq!(status, 200, "the policy registers: {registered}");
    let digest = registered["digest"].as_str().unwrap().to_string();
    // A second policy at the SAME version, so the lock is shown to carry every
    // named policy, in its stable order, and not merely the first.
    let (status, second) = register_policy(
        &client,
        &base,
        &alice_id,
        &json!({
            "policy_id": "lock-also",
            "version": "1.0.0",
            "lifecycle": "active",
            "title": "the second policy the lock must name",
            "owning_authority": alice_grant,
            "clauses": [ { "id": "c2", "statement": "every thread declares its objective" } ],
        }),
    )
    .await;
    assert_eq!(status, 200, "the second policy registers: {second}");
    let second_digest = second["digest"].as_str().unwrap().to_string();
    let resolution = json!({
        "policies": [
            { "policy_id": "lock-real", "version": "1.0.0" },
            { "policy_id": "lock-also", "version": "1.0.0" },
        ],
        "target": { "layer": "organization", "target": "org-acme" },
    });

    // Leg 1, THE REFUSAL: a caller-written lock naming a policy that was never
    // resolved, a digest nobody registered and another principal's grant.
    let (status, refused) = post(
        &client,
        &base,
        "/v1/policy-projections",
        &alice_id,
        &json!({
            "projection_id": "lock-forged",
            "target": "lock",
            "resolution": resolution,
            "lock": [ {
                "policy_id": "never-resolved",
                "version": "9.9.9",
                "digest": format!("sha256:{}", "f".repeat(64)),
                "owning_authority": format!("grt_{bob_id}"),
            } ],
        }),
    )
    .await;
    assert_eq!(
        status, 422,
        "a projection request may not supply its own lock rows: {refused}"
    );
    let (status, projections) = get(&client, &base, "/v1/policy-projections", &alice_id).await;
    assert_eq!(status, 200, "the projections list: {projections}");
    assert!(
        !projections
            .as_array()
            .unwrap()
            .iter()
            .any(|p| p["projection_id"] == json!("lock-forged")),
        "the refused projection stored nothing: {projections}"
    );

    // Leg 2: the lock the server writes is exactly the resolved policy, with
    // the registry's derived digest and its owner.
    let (status, locked) = post(
        &client,
        &base,
        "/v1/policy-projections",
        &alice_id,
        &json!({ "projection_id": "lock-derived", "target": "lock", "resolution": resolution }),
    )
    .await;
    assert_eq!(status, 200, "the lock projects: {locked}");
    assert_eq!(
        locked["bytes"],
        json!(format!(
            "# policy.lock (deterministic projection)\n\
             lock-also 1.0.0 {second_digest} {alice_grant}\n\
             lock-real 1.0.0 {digest} {alice_grant}\n"
        )),
        "{locked}"
    );

    // Leg 3: a policy whose stored digest does not verify is refused by the
    // lock, naming it; the lock never publishes a digest that identifies nothing.
    sqlx::query(
        "INSERT INTO policy_versions (policy_id, version, digest, lifecycle, title, \
         owning_authority, clauses) \
         VALUES ('lock-legacy', '1.0.0', $1, 'active', 'a legacy row', $2, \
         '[{\"id\": \"c9\", \"statement\": \"stored with a declared digest\"}]'::jsonb)",
    )
    .bind(DIGEST)
    .bind(&alice_grant)
    .execute(&pool)
    .await
    .expect("the legacy row inserts");
    let (status, refused) = post(
        &client,
        &base,
        "/v1/policy-projections",
        &alice_id,
        &json!({
            "projection_id": "lock-legacy",
            "target": "lock",
            "resolution": {
                "policies": [ { "policy_id": "lock-legacy", "version": "1.0.0" } ],
                "target": { "layer": "organization", "target": "org-acme" },
            },
        }),
    )
    .await;
    assert_eq!(status, 400, "an unverified digest is not locked: {refused}");
    assert!(
        refused["message"]
            .as_str()
            .is_some_and(|m| m.contains("lock-legacy") && m.contains("digest")),
        "the refusal names the policy and the digest: {refused}"
    );

    sqlx::query(
        "DELETE FROM policy_versions WHERE policy_id IN ('lock-real', 'lock-also', 'lock-legacy')",
    )
    .execute(&pool)
    .await
    .expect("drop the fixture rows");
}

/// `SIGNOFF-REPAIR.9.1.4`: the projection example in `policy-lifecycle.md`,
/// sent as the book writes it. Only the identifiers differ, because the book's
/// are elided. A change that breaks the example breaks this control.
#[tokio::test]
async fn the_books_projection_example_runs() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, human) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "book-projection" }),
    )
    .await;
    assert_eq!(status, 200, "the human enrols: {human}");
    let human_id = human["principal_id"].as_str().unwrap().to_string();
    site_fixture::provision(
        &pool,
        &human_id,
        &[reasonbraid_server::site_authority::Action::PolicyRegister],
    )
    .await;
    let grant_id = format!("grt_{human_id}");
    let (status, registered) = register_policy(
        &client,
        &base,
        &human_id,
        &json!({
            "policy_id": "pol_retention",
            "version": "2.1.0",
            "lifecycle": "active",
            "title": "evidence retention",
            "owning_authority": grant_id,
            "clauses": [ { "id": "c1", "statement": "evidence is retained for 90 days" } ],
            "applicability": [ { "layer": "organization", "target": "*" } ],
        }),
    )
    .await;
    assert_eq!(status, 200, "the example's policy registers: {registered}");
    let digest = registered["digest"].as_str().unwrap().to_string();

    // The book's generic example, verbatim apart from the identifiers.
    let (status, generic) = post(
        &client,
        &base,
        "/v1/policy-projections",
        &human_id,
        &json!({
            "projection_id": "prj_book_generic",
            "target": "generic",
            "resolution": {
                "policies": [ { "policy_id": "pol_retention", "version": "2.1.0" } ],
                "target": { "layer": "organization", "target": "org-acme" },
            },
        }),
    )
    .await;
    assert_eq!(
        status, 200,
        "the book's generic example projects: {generic}"
    );
    assert_eq!(generic["target"], json!("generic"), "{generic}");
    assert!(generic["digest"].as_str().unwrap().starts_with("sha256:"));
    assert_eq!(
        generic["bytes"],
        json!(
            "# Policy bundle (deterministic projection)\n## c1 [pol_retention 2.1.0]\nevidence is retained for 90 days\n"
        ),
        "{generic}"
    );
    assert_eq!(generic["unrepresentable"], json!([]), "{generic}");
    assert_eq!(
        generic["resolved_policies"],
        json!([ { "policy_id": "pol_retention", "version": "2.1.0" } ]),
        "{generic}"
    );

    // And its lock example.
    let (status, locked) = post(
        &client,
        &base,
        "/v1/policy-projections",
        &human_id,
        &json!({
            "projection_id": "prj_book_lock",
            "target": "lock",
            "resolution": {
                "policies": [ { "policy_id": "pol_retention", "version": "2.1.0" } ],
                "target": { "layer": "organization", "target": "org-acme" },
            },
        }),
    )
    .await;
    assert_eq!(status, 200, "the book's lock example projects: {locked}");
    assert_eq!(
        locked["bytes"],
        json!(format!(
            "# policy.lock (deterministic projection)\npol_retention 2.1.0 {digest} {grant_id}\n"
        )),
        "{locked}"
    );

    sqlx::query("DELETE FROM policy_versions WHERE policy_id = 'pol_retention'")
        .execute(&pool)
        .await
        .expect("drop the fixture rows");
}

/// `SIGNOFF-REPAIR.9.1.5`: a selector that does not say what it selects is
/// REFUSED, never read as a wildcard.
///
/// The resolver read `layer` and `target` with `.unwrap_or("*")`, so a missing
/// field, a non-string field, a misspelt key or a bare string all matched EVERY
/// target: an applicability typo applied a policy everywhere, and a
/// non-applicability typo removed it everywhere. A selector is now exactly
/// `{"layer": …, "target": …}`, both non-empty strings, with `"*"` written out.
/// Registration refuses anything else, and a stored selector that does not
/// parse fails the resolution closed, naming its policy.
#[tokio::test]
async fn a_malformed_selector_is_refused_not_widened() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, human) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "selector-human" }),
    )
    .await;
    assert_eq!(status, 200, "the human enrols: {human}");
    let human_id = human["principal_id"].as_str().unwrap().to_string();
    site_fixture::provision(
        &pool,
        &human_id,
        &[reasonbraid_server::site_authority::Action::PolicyRegister],
    )
    .await;
    let grant_id = format!("grt_{human_id}");
    let document = |policy_id: &str, field: &str, selectors: Value| {
        let mut body = json!({
            "policy_id": policy_id,
            "version": "1.0.0",
            "lifecycle": "active",
            "title": "the selector control",
            "owning_authority": grant_id,
            "clauses": [ { "id": format!("{policy_id}-c1"), "statement": "every thread declares its objective" } ],
        });
        body[field] = selectors;
        body
    };

    // Legs 1–5, THE REFUSALS: each malformed form, in each selector list.
    for (label, selectors) in [
        ("a missing target", json!([ { "layer": "organization" } ])),
        ("a bare string", json!(["tenant:*"])),
        (
            "a misspelt key",
            json!([ { "layr": "organization", "target": "*" } ]),
        ),
        (
            "a non-string field",
            json!([ { "layer": 5, "target": "*" } ]),
        ),
        ("an empty field", json!([ { "layer": "", "target": "*" } ])),
    ] {
        for field in ["applicability", "non_applicability"] {
            let (status, refused) = register_policy(
                &client,
                &base,
                &human_id,
                &document("sel-refused", field, selectors.clone()),
            )
            .await;
            assert_eq!(
                status, 400,
                "{label} in {field} is refused, not read as a wildcard: {refused}"
            );
            assert!(
                refused["message"]
                    .as_str()
                    .is_some_and(|m| m.contains(field) && m.contains("selector")),
                "the refusal names the list and the rule: {refused}"
            );
        }
    }

    // Leg 6, the matched positive: a well-formed selector still selects, and a
    // non-applicability excludes only the target it names.
    let (status, registered) = register_policy(
        &client,
        &base,
        &human_id,
        &json!({
            "policy_id": "sel-good",
            "version": "1.0.0",
            "lifecycle": "active",
            "title": "the selector control",
            "owning_authority": grant_id,
            "clauses": [ { "id": "sel-good-c1", "statement": "every thread declares its objective" } ],
            "applicability": [ { "layer": "organization", "target": "*" } ],
            "non_applicability": [ { "layer": "organization", "target": "eu-restricted" } ],
        }),
    )
    .await;
    assert_eq!(
        status, 200,
        "a well-formed selector registers: {registered}"
    );
    let resolve_for = |target: &str| {
        json!({
            "policies": [ { "policy_id": "sel-good", "version": "1.0.0" } ],
            "target": { "layer": "organization", "target": target },
        })
    };
    let (status, applies) = post(
        &client,
        &base,
        "/v1/policies/resolve",
        &human_id,
        &resolve_for("org-acme"),
    )
    .await;
    assert_eq!(status, 200, "{applies}");
    assert_eq!(
        applies["resolved"].as_array().unwrap().len(),
        1,
        "{applies}"
    );
    let (status, excluded) = post(
        &client,
        &base,
        "/v1/policies/resolve",
        &human_id,
        &resolve_for("eu-restricted"),
    )
    .await;
    assert_eq!(status, 200, "{excluded}");
    assert_eq!(excluded["resolved"], json!([]), "{excluded}");

    // Leg 7: a stored selector that does not parse (a row written before the
    // rule) fails the resolution closed and names its policy, rather than
    // silently excluding it from every target.
    sqlx::query(
        "INSERT INTO policy_versions (policy_id, version, digest, lifecycle, title, \
         owning_authority, clauses, non_applicability) \
         VALUES ('sel-legacy', '1.0.0', $1, 'active', 'a legacy row', $2, \
         '[{\"id\": \"sel-legacy-c1\", \"statement\": \"stored before the rule\"}]'::jsonb, \
         '[\"region:eu-restricted\"]'::jsonb)",
    )
    .bind(DIGEST)
    .bind(&grant_id)
    .execute(&pool)
    .await
    .expect("the legacy row inserts");
    let (status, refused) = post(
        &client,
        &base,
        "/v1/policies/resolve",
        &human_id,
        &json!({
            "policies": [ { "policy_id": "sel-legacy", "version": "1.0.0" } ],
            "target": { "layer": "organization", "target": "org-acme" },
        }),
    )
    .await;
    assert_eq!(
        status, 400,
        "a stored malformed selector fails closed: {refused}"
    );
    assert!(
        refused["message"]
            .as_str()
            .is_some_and(|m| m.contains("sel-legacy") && m.contains("selector")),
        "the refusal names the policy: {refused}"
    );

    sqlx::query("DELETE FROM policy_versions WHERE policy_id IN ('sel-good', 'sel-legacy')")
        .execute(&pool)
        .await
        .expect("drop the fixture rows");
}

/// `SIGNOFF-REPAIR.9.1.6`: the resolution's dependency, precedence and
/// set-shape steps refuse the sets the design refuses.
///
/// Measured before this leaf: a dependency was satisfied by ANY loaded policy
/// with its id (its version never read, its applicability never consulted);
/// precedence refused only a two-policy cycle while the explanation asserted
/// *"the precedence edges form a DAG"*; and a policy named twice was reported
/// as a binding conflict with itself. Steps 3 and 4 now run over the
/// APPLICABLE policies, in §15.3's order, and the shapes those steps read are
/// validated at registration.
#[tokio::test]
async fn the_resolution_steps_refuse_what_the_design_refuses() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, human) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "steps-human" }),
    )
    .await;
    assert_eq!(status, 200, "the human enrols: {human}");
    let human_id = human["principal_id"].as_str().unwrap().to_string();
    site_fixture::provision(
        &pool,
        &human_id,
        &[reasonbraid_server::site_authority::Action::PolicyRegister],
    )
    .await;
    let grant_id = format!("grt_{human_id}");
    let policy = |policy_id: &str, version: &str, extra: Value| {
        let mut body = json!({
            "policy_id": policy_id,
            "version": version,
            "lifecycle": "draft",
            "title": policy_id,
            "owning_authority": grant_id,
            "clauses": [ { "id": format!("{policy_id}-{version}"), "statement": "a clause of its own" } ],
        });
        for (key, value) in extra.as_object().unwrap() {
            body[key] = value.clone();
        }
        body
    };
    for (policy_id, version, extra) in [
        ("st-dep", "1.0.0", json!({})),
        ("st-dep", "2.0.0", json!({})),
        (
            "st-needs-v2",
            "1.0.0",
            json!({ "dependencies": [ { "policy": "st-dep", "version": "2.0.0" } ] }),
        ),
        (
            "st-elsewhere",
            "1.0.0",
            json!({ "applicability": [ { "layer": "project", "target": "prj-other" } ] }),
        ),
        (
            "st-needs-elsewhere",
            "1.0.0",
            json!({ "dependencies": [ { "policy": "st-elsewhere", "version": "1.0.0" } ] }),
        ),
        (
            "st-x",
            "1.0.0",
            json!({ "precedence_hints": [ { "over": "st-y" } ] }),
        ),
        (
            "st-y",
            "1.0.0",
            json!({ "precedence_hints": [ { "over": "st-z" } ] }),
        ),
        (
            "st-z",
            "1.0.0",
            json!({ "precedence_hints": [ { "over": "st-x" } ] }),
        ),
        (
            "st-conf-a",
            "1.0.0",
            json!({ "conflicts": [ { "policy": "st-conf-b" } ] }),
        ),
        ("st-conf-b", "1.0.0", json!({})),
        (
            "st-conf-moot",
            "1.0.0",
            json!({ "conflicts": [ { "policy": "st-elsewhere" } ] }),
        ),
        (
            "st-self",
            "1.0.0",
            json!({ "precedence_hints": [ { "over": "st-self" } ] }),
        ),
        (
            "st-p",
            "1.0.0",
            json!({ "precedence_hints": [ { "over": "st-q" } ] }),
        ),
        (
            "st-q",
            "1.0.0",
            json!({
                "applicability": [ { "layer": "project", "target": "prj-other" } ],
                "precedence_hints": [ { "over": "st-p" } ],
            }),
        ),
    ] {
        let (status, registered) = register_policy(
            &client,
            &base,
            &human_id,
            &policy(policy_id, version, extra),
        )
        .await;
        assert_eq!(status, 200, "{policy_id} {version} registers: {registered}");
    }
    // Every leg is RECORDED and judged at the end, so one run against the
    // unrepaired code shows each defect, not only the first.
    let mut breaches: Vec<String> = Vec::new();
    let resolve = |refs: &[(&str, &str)]| {
        json!({
            "policies": refs.iter().map(|(p, v)| json!({ "policy_id": p, "version": v })).collect::<Vec<_>>(),
            "target": { "layer": "organization", "target": "org-acme" },
        })
    };
    let refused_with = |status: u16, body: &Value, needles: &[&str]| {
        status == 400
            && body["message"]
                .as_str()
                .is_some_and(|m| needles.iter().all(|n| m.contains(n)))
    };

    // Leg 1: a dependency on version 2.0.0 is not satisfied by version 1.0.0.
    let (status, body) = post(
        &client,
        &base,
        "/v1/policies/resolve",
        &human_id,
        &resolve(&[("st-needs-v2", "1.0.0"), ("st-dep", "1.0.0")]),
    )
    .await;
    if !refused_with(status, &body, &["st-dep", "2.0.0"]) {
        breaches.push(format!(
            "leg 1: version 1.0.0 satisfied a dependency on 2.0.0 ({status}): {body}"
        ));
    }

    // Leg 2: a dependency that does not apply to this target does not satisfy.
    let (status, body) = post(
        &client,
        &base,
        "/v1/policies/resolve",
        &human_id,
        &resolve(&[("st-needs-elsewhere", "1.0.0"), ("st-elsewhere", "1.0.0")]),
    )
    .await;
    if !refused_with(status, &body, &["st-elsewhere"]) {
        breaches.push(format!(
            "leg 2: a dependency that does not apply here satisfied ({status}): {body}"
        ));
    }

    // Leg 3, the positive: the named version, applicable, satisfies.
    let (status, body) = post(
        &client,
        &base,
        "/v1/policies/resolve",
        &human_id,
        &resolve(&[("st-needs-v2", "1.0.0"), ("st-dep", "2.0.0")]),
    )
    .await;
    if status != 200 {
        breaches.push(format!(
            "leg 3: the named, applicable version did not satisfy ({status}): {body}"
        ));
    }

    // Leg 4: a three-policy precedence cycle is refused, although no clause id
    // collides (each policy's clause is its own).
    let (status, body) = post(
        &client,
        &base,
        "/v1/policies/resolve",
        &human_id,
        &resolve(&[("st-x", "1.0.0"), ("st-y", "1.0.0"), ("st-z", "1.0.0")]),
    )
    .await;
    if !refused_with(status, &body, &["cycle"]) {
        breaches.push(format!(
            "leg 4: a three-policy precedence cycle was accepted ({status}): {body}"
        ));
    }

    // Leg 5: the same policy named twice, and two versions of one policy, are
    // refused as such rather than as a binding conflict with itself.
    for refs in [
        vec![("st-dep", "1.0.0"), ("st-dep", "1.0.0")],
        vec![("st-dep", "1.0.0"), ("st-dep", "2.0.0")],
    ] {
        let (status, body) = post(
            &client,
            &base,
            "/v1/policies/resolve",
            &human_id,
            &resolve(&refs),
        )
        .await;
        if !refused_with(status, &body, &["more than once"]) {
            breaches.push(format!(
                "leg 5: {refs:?} was not refused as a repeated policy ({status}): {body}"
            ));
        }
    }

    // Leg 6: the shapes steps 3 and 4 read are validated at registration.
    // Each case takes its own version, so a case the unrepaired code ADMITS
    // cannot make the next fail for the unrelated reason of a taken coordinate.
    for (case, (field, entry)) in [
        ("dependencies", json!([ { "policy": "st-dep" } ])),
        ("dependencies", json!(["st-dep"])),
        ("conflicts", json!([ { "polic": "st-dep" } ])),
        ("precedence_hints", json!([ { "over": "" } ])),
    ]
    .into_iter()
    .enumerate()
    {
        let version = format!("1.0.{case}");
        let (status, body) = register_policy(
            &client,
            &base,
            &human_id,
            &policy("st-malformed", &version, json!({ field: entry.clone() })),
        )
        .await;
        if !refused_with(status, &body, &[field]) {
            breaches.push(format!(
                "leg 6: {field} {entry} was not refused at registration ({status}): {body}"
            ));
        }
    }

    // Leg 7: an explicit conflict with a policy that applies here is refused;
    // one with a policy that does not apply here is moot (§15.3: step 2
    // filters before step 3 checks). ⚠️ The second half LOOSENS the old
    // behaviour, which refused a conflict with any loaded policy.
    let (status, body) = post(
        &client,
        &base,
        "/v1/policies/resolve",
        &human_id,
        &resolve(&[("st-conf-a", "1.0.0"), ("st-conf-b", "1.0.0")]),
    )
    .await;
    if !refused_with(status, &body, &["st-conf-a conflicts with st-conf-b"]) {
        breaches.push(format!(
            "leg 7: an applicable conflict was not refused ({status}): {body}"
        ));
    }
    let (status, body) = post(
        &client,
        &base,
        "/v1/policies/resolve",
        &human_id,
        &resolve(&[("st-conf-moot", "1.0.0"), ("st-elsewhere", "1.0.0")]),
    )
    .await;
    if status != 200 {
        breaches.push(format!(
            "leg 7: a conflict with a policy that does not apply here refused ({status}): {body}"
        ));
    }

    // Leg 8: a self-hint is a no-op, and a cycle that runs through a policy
    // that does not apply here is moot, for the same reason as leg 7.
    let (status, body) = post(
        &client,
        &base,
        "/v1/policies/resolve",
        &human_id,
        &resolve(&[("st-self", "1.0.0")]),
    )
    .await;
    if status != 200 {
        breaches.push(format!("leg 8: a self-hint refused ({status}): {body}"));
    }
    let (status, body) = post(
        &client,
        &base,
        "/v1/policies/resolve",
        &human_id,
        &resolve(&[("st-p", "1.0.0"), ("st-q", "1.0.0")]),
    )
    .await;
    if status != 200 {
        breaches.push(format!(
            "leg 8: a cycle through a policy that does not apply here refused ({status}): {body}"
        ));
    }

    assert!(
        breaches.is_empty(),
        "{} breach(es):\n{}",
        breaches.len(),
        breaches.join("\n")
    );

    sqlx::query("DELETE FROM policy_versions WHERE policy_id LIKE 'st-%'")
        .execute(&pool)
        .await
        .expect("drop the fixture rows");
}

/// `SIGNOFF-REPAIR.9.1.7`: the policy registry's refusals say what happened.
///
/// Measured before this leaf: five resolution refusals were built on
/// `PolicyError::Duplicate`, whose text appends *"already exists — … register a
/// new version"*, so an unregistered reference answered that it *already
/// exists*; the owner refusal nested its own sentence inside itself; an empty
/// request answered about *normative statements*; a database that could not
/// answer was reported as a domain refusal (`400`) by `resolve`, `impact` and
/// the projection insert; and `is_semver` was not SemVer. Every leg is
/// RECORDED and judged at the end. The three outage legs break a column or a
/// constraint for exactly one request and restore it BEFORE judging anything,
/// so a failing leg cannot leave the schema broken for the next test.
#[tokio::test]
async fn the_policy_refusals_say_what_happened() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, human) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "texts-human" }),
    )
    .await;
    assert_eq!(status, 200, "the human enrols: {human}");
    let human_id = human["principal_id"].as_str().unwrap().to_string();
    site_fixture::provision(
        &pool,
        &human_id,
        &[reasonbraid_server::site_authority::Action::PolicyRegister],
    )
    .await;
    let grant_id = format!("grt_{human_id}");
    let policy = |policy_id: &str, version: &str, clause: &str, extra: Value| {
        let mut body = json!({
            "policy_id": policy_id,
            "version": version,
            "lifecycle": "active",
            "title": policy_id,
            "owning_authority": grant_id,
            "clauses": [ { "id": clause, "statement": "a clause" } ],
        });
        for (key, value) in extra.as_object().unwrap() {
            body[key] = value.clone();
        }
        body
    };
    for body in [
        policy(
            "tx-a",
            "1.0.0",
            "shared",
            json!({ "exceptions": [ { "waiver": "wv-ok" } ] }),
        ),
        policy("tx-b", "1.0.0", "shared", json!({})),
        policy(
            "tx-needs",
            "1.0.0",
            "own",
            json!({ "dependencies": [ { "policy": "tx-absent", "version": "1.0.0" } ] }),
        ),
    ] {
        let (status, registered) = register_policy(&client, &base, &human_id, &body).await;
        assert_eq!(status, 200, "the fixture registers: {registered}");
    }
    let resolve = |refs: &[(&str, &str)], extra: Value| {
        let mut body = json!({
            "policies": refs.iter().map(|(p, v)| json!({ "policy_id": p, "version": v })).collect::<Vec<_>>(),
            "target": { "layer": "organization", "target": "org-acme" },
        });
        for (key, value) in extra.as_object().unwrap() {
            body[key] = value.clone();
        }
        body
    };
    let mut breaches: Vec<String> = Vec::new();
    let message = |body: &Value| body["message"].as_str().unwrap_or_default().to_string();

    // Legs 1–4: each resolution refusal says what it is, and none says a
    // missing thing *already exists*.
    for (label, request, needle) in [
        (
            "an unregistered reference",
            resolve(&[("tx-ghost", "1.0.0")], json!({})),
            "not registered",
        ),
        (
            "a missing dependency",
            resolve(&[("tx-needs", "1.0.0")], json!({})),
            "tx-absent 1.0.0",
        ),
        (
            "an unknown waiver",
            resolve(
                &[("tx-a", "1.0.0")],
                json!({ "exception_grants": ["wv-nope"] }),
            ),
            "exception schema",
        ),
        (
            "a binding conflict",
            resolve(&[("tx-a", "1.0.0"), ("tx-b", "1.0.0")], json!({})),
            "binding conflict",
        ),
    ] {
        let (status, body) =
            post(&client, &base, "/v1/policies/resolve", &human_id, &request).await;
        let text = message(&body);
        if status != 400 || !text.contains(needle) || text.contains("already exists") {
            breaches.push(format!("{label}: ({status}) {text}"));
        }
    }

    // Leg 5: an empty request says so.
    let (status, body) = post(
        &client,
        &base,
        "/v1/policies/resolve",
        &human_id,
        &resolve(&[], json!({})),
    )
    .await;
    let text = message(&body);
    if status != 400 || !text.contains("names no policy") {
        breaches.push(format!("an empty request: ({status}) {text}"));
    }

    // Leg 6: the version rule is SemVer 2.0.0, in both directions.
    for (version, admitted) in [
        ("1", false),
        ("1.0", false),
        ("01.2.3", false),
        ("1.2.3-rc.1+build.5", true),
        ("1.2.3-alpha", true),
    ] {
        let (status, body) = register_policy(
            &client,
            &base,
            &human_id,
            &policy("tx-version", version, "own", json!({})),
        )
        .await;
        let ok = if admitted {
            status == 200
        } else {
            status == 400 && message(&body).contains("semantic version")
        };
        if !ok {
            breaches.push(format!(
                "version `{version}` (admitted {admitted}): ({status}) {body}"
            ));
        }
    }

    // Leg 7: an owner whose grant is no longer live is named once, plainly.
    let (status, body) = register_policy(
        &client,
        &base,
        &human_id,
        &policy("tx-owned", "1.0.0", "owned", json!({})),
    )
    .await;
    assert_eq!(status, 200, "the owned policy registers: {body}");

    // Legs 8–10, THE OUTAGES: a store that cannot answer is a 500, never a
    // refusal. Each fault is undone before anything is judged.
    sqlx::query("ALTER TABLE policy_versions RENAME COLUMN clauses TO clauses_elsewhere")
        .execute(&pool)
        .await
        .expect("the column moves");
    let resolved = post(
        &client,
        &base,
        "/v1/policies/resolve",
        &human_id,
        &resolve(&[("tx-a", "1.0.0")], json!({})),
    )
    .await;
    let impact = get(&client, &base, "/v1/policies/tx-a/1.0.0/impact", &human_id).await;
    sqlx::query("ALTER TABLE policy_versions RENAME COLUMN clauses_elsewhere TO clauses")
        .execute(&pool)
        .await
        .expect("the column returns");
    if resolved.0 != 500 {
        breaches.push(format!(
            "resolve under an outage: ({}) {}",
            resolved.0, resolved.1
        ));
    }
    if impact.0 != 500 {
        breaches.push(format!(
            "impact under an outage: ({}) {}",
            impact.0, impact.1
        ));
    }
    // The owner-liveness read (step 1) failing, with the load succeeding.
    sqlx::query("ALTER TABLE authority_grants RENAME COLUMN actions TO actions_elsewhere")
        .execute(&pool)
        .await
        .expect("the column moves");
    let liveness = post(
        &client,
        &base,
        "/v1/policies/resolve",
        &human_id,
        &resolve(&[("tx-a", "1.0.0")], json!({})),
    )
    .await;
    sqlx::query("ALTER TABLE authority_grants RENAME COLUMN actions_elsewhere TO actions")
        .execute(&pool)
        .await
        .expect("the column returns");
    if liveness.0 != 500 {
        breaches.push(format!(
            "resolve when the owner-liveness read fails: ({}) {}",
            liveness.0, liveness.1
        ));
    }
    sqlx::query(
        "ALTER TABLE policy_projections ADD CONSTRAINT tx_refuse_all CHECK (false) NOT VALID",
    )
    .execute(&pool)
    .await
    .expect("the constraint is added");
    let projected = post(
        &client,
        &base,
        "/v1/policy-projections",
        &human_id,
        &json!({ "projection_id": "tx-projection", "target": "generic",
                 "resolution": resolve(&[("tx-a", "1.0.0")], json!({})) }),
    )
    .await;
    sqlx::query("ALTER TABLE policy_projections DROP CONSTRAINT tx_refuse_all")
        .execute(&pool)
        .await
        .expect("the constraint is dropped");
    if projected.0 != 500 {
        breaches.push(format!(
            "a projection whose insert fails for a reason other than a taken id: ({}) {}",
            projected.0, projected.1
        ));
    }

    // Leg 7, judged after the outages (it retires the grant): the owner's
    // refusal names the grant once and does not nest its own sentence.
    sqlx::query("UPDATE authority_grants SET status = 'revoked' WHERE grant_id = $1")
        .bind(&grant_id)
        .execute(&pool)
        .await
        .expect("the grant is revoked");
    let (status, body) = post(
        &client,
        &base,
        "/v1/policies/resolve",
        &human_id,
        &resolve(&[("tx-owned", "1.0.0")], json!({})),
    )
    .await;
    let text = message(&body);
    if status != 400
        || text.matches(&grant_id).count() != 1
        || text.contains("`the owning authority")
    {
        breaches.push(format!("an owner that is not live: ({status}) {text}"));
    }

    sqlx::query("DELETE FROM policy_versions WHERE policy_id LIKE 'tx-%'")
        .execute(&pool)
        .await
        .expect("drop the fixture rows");
    assert!(
        breaches.is_empty(),
        "{} breach(es):\n{}",
        breaches.len(),
        breaches.join("\n")
    );
}

/// The POSITIVE arm, without which the repair above is indistinguishable from
/// deleting the route: a `policy_register` holder still registers, the receipt
/// is audited, the document is REACHABLE through every read, and the same grant
/// is refused at an unrelated site verb — so one verb widened, not the boundary.
#[tokio::test]
async fn the_policy_register_capability_still_registers_and_resolves() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, operator) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "prc-operator" }),
    )
    .await;
    assert_eq!(status, 200, "the operator enrols: {operator}");
    let operator_id = operator["principal_id"].as_str().unwrap().to_string();

    site_fixture::provision(
        &pool,
        &operator_id,
        &[reasonbraid_server::site_authority::Action::PolicyRegister],
    )
    .await;

    let body = json!({
        "reason": "the review lane needs a published baseline",
        "policy_id": "prc-baseline",
        "version": "1.0.0",
        "lifecycle": "active",
        "title": "the review baseline",
        "owning_authority": format!("grt_{operator_id}"),
        "clauses": [ { "id": "c1", "statement": "every review names its trigger" } ],
        "applicability": [ { "layer": "organization", "target": "*" } ],
    });
    let response = client
        .post(format!("{base}/v1/policies"))
        .header(PRINCIPAL_HEADER, &operator_id)
        .json(&body)
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
        registered["policy_id"],
        json!("prc-baseline"),
        "{registered}"
    );
    assert_eq!(registered["version"], json!("1.0.0"), "{registered}");

    // ⚠️ Appending a version is deliberately still possible for a holder: the
    // registry is versioned by design, and forbidding it would have removed the
    // feature rather than repaired the authority. The defect was never that a
    // policy can gain a version — only that anyone could give it one.
    let (status, appended) = register_policy(
        &client,
        &base,
        &operator_id,
        &json!({
            "policy_id": "prc-baseline",
            "version": "1.1.0",
            "lifecycle": "active",
            "title": "the review baseline",
            "owning_authority": format!("grt_{operator_id}"),
            "clauses": [ { "id": "c1", "statement": "every review names its trigger and its owner" } ],
        }),
    )
    .await;
    assert_eq!(status, 200, "a holder still appends a version: {appended}");

    // The registration is REACHABLE, not merely stored: an ORDINARY enrolled
    // principal in another tenant reads it, resolves it and maps its impact.
    let (status, reader) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "prc-reader" }),
    )
    .await;
    assert_eq!(status, 200, "the reader enrols: {reader}");
    let reader_id = reader["principal_id"].as_str().unwrap().to_string();

    let (status, library) = get(&client, &base, "/v1/policies", &reader_id).await;
    assert_eq!(status, 200, "the library answers the reader: {library}");
    assert_eq!(
        library
            .as_array()
            .unwrap()
            .iter()
            .filter(|p| p["policy_id"] == json!("prc-baseline"))
            .count(),
        2,
        "both versions are readable by a principal who registered neither: {library}"
    );

    let (status, resolved) = post(
        &client,
        &base,
        "/v1/policies/resolve",
        &reader_id,
        &json!({
            "policies": [ { "policy_id": "prc-baseline", "version": "1.1.0" } ],
            "target": { "layer": "organization", "target": "anything" },
        }),
    )
    .await;
    assert_eq!(status, 200, "the reader resolves it: {resolved}");
    assert_eq!(
        resolved["resolved"][0]["statement"],
        json!("every review names its trigger and its owner"),
        "{resolved}"
    );

    // ⛔ The capability is for THIS action only: the same grant does not become
    // site-wide authority. A registry inspection with a policy-only grant is
    // refused, so the repair widened one verb rather than the boundary.
    let (status, refused) = get(&client, &base, "/v1/admin/adapters", &operator_id).await;
    assert_eq!(
        status, 403,
        "a policy_register grant inspects no registry: {refused}"
    );

    sqlx::query("DELETE FROM policy_versions WHERE policy_id = 'prc-baseline'")
        .execute(&pool)
        .await
        .expect("drop the fixture rows");
}

/// `SIGNOFF-REPAIR.11.4.7.2.1.2.3.1`: a policy decision is its thread's COUNTED
/// close, so every proposal thread here declares `owner_decides` — the one
/// family that needs no ballot — and its tenant's charter must allow it. The
/// registration is direct because the site gate is `.11.4.7.2.1.2.1`'s
/// control, not this suite's; the boundary rebind is what a site operator's
/// reissue does.
/// `SIGNOFF-REPAIR.11.36` — a request body that does not deserialize is refused
/// with a reason code, on every surface. `errors.md` promises every refusal a
/// stable `code`, and 52 handlers took a bare `Json<T>`, whose rejection is axum's
/// default: a plain-text `422` for a body of the wrong shape and a plain-text
/// `400` for one that does not parse, neither carrying a `code`. ⭐ The STATUSES
/// stay (the `422` is the strict wire boundary's deliberate level, `.9.2.1.2.3`);
/// the body gains `{code, message}`. The sample spans
/// the control API (enrolment, threads, deployments, profiles, evidence) and the
/// node channel; the guard `api::json_extraction::no_handler_takes_a_bare_json`
/// holds every other handler to the same shape.
#[tokio::test]
async fn a_malformed_body_is_refused_with_a_reason_code() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();
    let routes = [
        (reqwest::Method::POST, "/v1/enrollments"),
        (reqwest::Method::POST, "/v1/threads"),
        (reqwest::Method::POST, "/v1/deployments"),
        (
            reqwest::Method::PUT,
            "/v1/profiles/rol_00000000-0000-7000-8000-000000000036",
        ),
        (reqwest::Method::POST, "/v1/snapshots"),
        (reqwest::Method::POST, "/v1/nodes/events"),
        (reqwest::Method::POST, "/v1/nodes/heartbeat"),
    ];
    for (method, path) in routes {
        for (shape, body, expected) in [
            ("the wrong shape", "[]", 422_u16),
            ("no JSON at all", "{", 400_u16),
        ] {
            let response = client
                .request(method.clone(), format!("{base}{path}"))
                .header("content-type", "application/json")
                .body(body)
                .send()
                .await
                .expect("the request is sent");
            let status = response.status().as_u16();
            let text = response.text().await.expect("the body reads");
            let answer: Value =
                serde_json::from_str(&text).unwrap_or_else(|_| json!({ "raw": text }));
            assert_eq!(status, expected, "{method} {path} with {shape}: {answer}");
            assert_eq!(
                answer["code"],
                json!("invalid_command"),
                "{method} {path} with {shape}: {answer}"
            );
            assert!(
                answer["message"].as_str().is_some_and(|m| !m.is_empty()),
                "{method} {path} with {shape}: the parser's own sentence rides the message: {answer}"
            );
        }
    }
}

/// The publication's own desired pair, `(ref, digest)` — ADR-021: its first
/// recorded Git object id and the digest of its projection. A fixture deploys
/// what its publication deploys (`SIGNOFF-REPAIR.9.3.3.1`), never a value
/// typed into the test.
async fn desired_pair(pool: &PgPool, publication_id: &str) -> (String, String) {
    sqlx::query_as(
        "SELECT p.git_object_ids->>0, pr.digest FROM policy_publications p \
         JOIN policy_projections pr ON pr.projection_id = p.projection_id \
         WHERE p.publication_id = $1",
    )
    .bind(publication_id)
    .fetch_one(pool)
    .await
    .expect("an effective publication has a desired pair")
}

/// How a store fault is injected for one request (`SIGNOFF-REPAIR.9.3.3.6`).
#[derive(Clone, Copy, Debug)]
enum StoreFault {
    /// The table is renamed away, so every statement touching it fails.
    Unreadable,
    /// A `BEFORE INSERT` trigger raises, so only the write fails.
    InsertRefused,
}

/// POST once with `table` faulted, restoring it BEFORE returning, so no later
/// arm or suite inherits the fault and nothing is asserted against it.
#[allow(clippy::too_many_arguments)]
async fn post_under_fault(
    pool: &PgPool,
    table: &str,
    fault: StoreFault,
    client: &reqwest::Client,
    base: &str,
    path: &str,
    principal: &str,
    body: &Value,
) -> (u16, Value) {
    let (apply, restore) = match fault {
        StoreFault::Unreadable => (
            format!("ALTER TABLE {table} RENAME TO {table}_withheld"),
            format!("ALTER TABLE {table}_withheld RENAME TO {table}"),
        ),
        StoreFault::InsertRefused => (
            format!(
                "CREATE FUNCTION rb_refuse_insert() RETURNS trigger LANGUAGE plpgsql AS \
                 $$ BEGIN RAISE EXCEPTION 'the write is withheld'; END; $$; \
                 CREATE TRIGGER rb_refuse_insert BEFORE INSERT ON {table} \
                 FOR EACH ROW EXECUTE FUNCTION rb_refuse_insert();"
            ),
            format!("DROP TRIGGER rb_refuse_insert ON {table}; DROP FUNCTION rb_refuse_insert();"),
        ),
    };
    sqlx::raw_sql(&apply)
        .execute(pool)
        .await
        .expect("inject the fault");
    let answered = post(client, base, path, principal, body).await;
    sqlx::raw_sql(&restore)
        .execute(pool)
        .await
        .expect("restore the table");
    answered
}

async fn allow_owner_decides(pool: &PgPool, tenant: &str) {
    allow_rules(pool, tenant, &["owner_decides"]).await;
}

async fn allow_rules(pool: &PgPool, tenant: &str, rules: &[&str]) {
    let stored = reasonbraid_server::charters::register(
        pool,
        &reasonbraid_server::charters::CharterInput {
            tenant_id: tenant.to_owned(),
            allowed_decision_rules: rules.iter().map(|r| (*r).to_owned()).collect(),
            approval_thresholds: Default::default(),
            charter_digest: None,
            reason: reasonbraid_server::site_authority::Reason::new("the policy suite's charter")
                .unwrap(),
        },
    )
    .await
    .expect("the charter registers");
    let bound = sqlx::query(
        "UPDATE enrollment_boundaries SET charter_digest = $1 \
         WHERE tenant_id = $2 AND status = 'active'",
    )
    .bind(&stored.charter_digest)
    .bind(tenant)
    .execute(pool)
    .await
    .expect("rebind the boundary")
    .rows_affected();
    assert_eq!(bound, 1, "the tenant has exactly one active boundary");
}

/// Close a proposal thread as its owner: under `owner_decides` that close is
/// the derived `accepted_by_rule` a policy decision records.
async fn close_as_owner(
    client: &reqwest::Client,
    base: &str,
    owner: &str,
    tenant: &str,
    thread_id: &str,
) {
    let response = client
        .post(format!("{base}/v1/threads/{thread_id}/commands"))
        .header(PRINCIPAL_HEADER, owner)
        .json(&json!({
            "protocol_version": reasonbraid_core::PROTOCOL_VERSION,
            "operation": "thread.close",
            "request_id": reasonbraid_core::RequestId::new().to_string(),
            "idempotency_key": format!("close-{thread_id}"),
            "body": { "tenant_id": tenant, "reason": "the owner decides" },
            "client_context": {},
        }))
        .send()
        .await
        .expect("close request");
    let status = response.status().as_u16();
    let body = response.text().await.unwrap_or_default();
    assert_eq!(status, 200, "the owner closes the proposal thread: {body}");
}

/// `SIGNOFF-REPAIR.11.4.7.2.1.2.3.1`: a policy decision is the proposal
/// thread's counted close. A thread that is still open, one that declares no
/// rule, and one that closed without a binding acceptance hold no decision; an
/// asserted rule or electorate that disagrees with the derived one is refused;
/// an omitted one records the derived record.
#[tokio::test]
async fn a_policy_decision_is_its_threads_counted_close() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, human) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "pdc-human" }),
    )
    .await;
    assert_eq!(status, 200, "the human enrolls: {human}");
    let human_id = human["principal_id"].as_str().unwrap().to_string();
    site_fixture::provision(
        &pool,
        &human_id,
        &[reasonbraid_server::site_authority::Action::PolicyRegister],
    )
    .await;
    let tenant_id = human["tenant_id"].as_str().unwrap().to_string();
    allow_rules(&pool, &tenant_id, &["owner_decides", "advisory_synthesis"]).await;
    let (status, _) = register_policy(
        &client,
        &base,
        &human_id,
        &json!({
            "policy_id": "pdc-policy",
            "version": "1.0.0",
            "lifecycle": "draft",
            "title": "pdc",
            "owning_authority": format!("grt_{human_id}"),
            "clauses": [ { "id": "c1", "statement": "the clause" } ],
        }),
    )
    .await;
    assert_eq!(status, 200, "the policy registers");

    // One thread per case, each with its own proposal.
    let thread = |key: &'static str, rule: Option<&'static str>| {
        let (client, base, human_id, tenant_id) = (
            client.clone(),
            base.clone(),
            human_id.clone(),
            tenant_id.clone(),
        );
        async move {
            let mut body = json!({ "tenant_id": tenant_id, "subject": key, "objective": "probe" });
            if let Some(rule) = rule {
                body["decision_rule"] = json!(rule);
            }
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
                    "body": body,
                    "client_context": {},
                }),
            )
            .await;
            assert_eq!(status, 200, "{key} creates: {created}");
            let thread_id = created["thread_id"].as_str().unwrap().to_string();
            let (status, proposal) = post(
                &client,
                &base,
                "/v1/policy-proposals",
                &human_id,
                &json!({
                    "proposal_id": format!("{key}-prop"),
                    "policy_id": "pdc-policy",
                    "policy_version": "1.0.0",
                    "thread_id": thread_id,
                }),
            )
            .await;
            assert_eq!(status, 200, "{key}'s proposal registers: {proposal}");
            thread_id
        }
    };
    let decide = |key: &'static str, extra: Value| {
        let (client, base, human_id) = (client.clone(), base.clone(), human_id.clone());
        async move {
            let mut body = json!({
                "decision_id": format!("{key}-dec"),
                "proposal_id": format!("{key}-prop"),
            });
            for (k, v) in extra.as_object().unwrap() {
                body[k] = v.clone();
            }
            post(&client, &base, "/v1/policy-decisions", &human_id, &body).await
        }
    };
    let refused = |status: u16, value: &Value, needle: &str, what: &str| {
        assert_eq!(status, 400, "{what}: {value}");
        assert!(
            value["message"]
                .as_str()
                .unwrap_or_default()
                .contains(needle),
            "{what} names `{needle}`: {value}"
        );
    };

    // An OPEN thread holds no decision.
    thread("pdc-open", Some("owner_decides")).await;
    let (status, value) = decide("pdc-open", json!({})).await;
    refused(status, &value, "not closed", "an open thread");

    // A RULE-LESS thread's close is the closer's claim. It asserted
    // `accepted_unanimously` until `SIGNOFF-REPAIR.8.1.1.5` made that word a
    // refusal on a thread that counted nothing; `accepted_by_rule` is still the
    // closer's claim, which is what this control needs.
    let ruleless = thread("pdc-ruleless", None).await;
    let (status, closed) = post(
        &client,
        &base,
        &format!("/v1/threads/{ruleless}/commands"),
        &human_id,
        &json!({
            "protocol_version": reasonbraid_core::PROTOCOL_VERSION,
            "operation": "thread.close",
            "request_id": reasonbraid_core::RequestId::new().to_string(),
            "idempotency_key": "pdc-ruleless-close",
            "body": { "tenant_id": tenant_id, "reason": "asserted",
                      "outcome": "accepted_by_rule" },
            "client_context": {},
        }),
    )
    .await;
    assert_eq!(status, 200, "{closed}");
    let (status, value) = decide("pdc-ruleless", json!({})).await;
    refused(
        status,
        &value,
        "declares no decision rule",
        "a rule-less thread",
    );

    // A close that is not a binding acceptance holds no decision.
    let deadlocked = thread("pdc-deadlocked", Some("owner_decides")).await;
    let (status, closed) = post(
        &client,
        &base,
        &format!("/v1/threads/{deadlocked}/commands"),
        &human_id,
        &json!({
            "protocol_version": reasonbraid_core::PROTOCOL_VERSION,
            "operation": "thread.close",
            "request_id": reasonbraid_core::RequestId::new().to_string(),
            "idempotency_key": "pdc-deadlocked-close",
            "body": { "tenant_id": tenant_id, "reason": "no decision", "outcome": "deadlocked" },
            "client_context": {},
        }),
    )
    .await;
    assert_eq!(status, 200, "{closed}");
    let (status, value) = decide("pdc-deadlocked", json!({})).await;
    refused(
        status,
        &value,
        "`inconclusive`, not closed",
        "a deadlocked close",
    );

    // An advisory close is closed and derived — and binds nothing.
    let advisory = thread("pdc-advisory", Some("advisory_synthesis")).await;
    close_as_owner(&client, &base, &human_id, &tenant_id, &advisory).await;
    let (status, value) = decide("pdc-advisory", json!({})).await;
    refused(
        status,
        &value,
        "not a binding acceptance",
        "an advisory close",
    );

    // THE CONTROL: a decided thread, and a decision that inflates its electorate.
    let decided = thread("pdc-decided", Some("owner_decides")).await;
    close_as_owner(&client, &base, &human_id, &tenant_id, &decided).await;
    let (status, value) = decide(
        "pdc-decided",
        json!({ "electorate": { "participants": [human_id, "hpr_00000000-0000-7000-8000-00000000beef"] } }),
    )
    .await;
    refused(status, &value, "electorate", "an inflated electorate");
    let (status, value) = decide("pdc-decided", json!({ "rule": "unanimity" })).await;
    refused(
        status,
        &value,
        "owner_decides",
        "a rule the thread did not declare",
    );

    // Omitted, the derived record is stored.
    let (status, decision) = decide("pdc-decided", json!({})).await;
    assert_eq!(status, 200, "the derived decision records: {decision}");
    assert_eq!(decision["rule"], json!("owner_decides"));
    assert_eq!(
        decision["electorate"],
        json!({ "participants": [human_id], "denominator": 1, "abstentions": [] })
    );
    assert_eq!(decision["verdict_event_id"], Value::Null);
    assert_eq!(decision["derivation"]["outcome"], json!("accepted_by_rule"));
    assert_eq!(decision["derivation"]["thread_id"], json!(decided));
    assert!(
        decision["derivation"]["charter_digest"]
            .as_str()
            .is_some_and(|d| d.starts_with("sha256:")),
        "the decision names the charter it was taken under: {decision}"
    );

    // `SIGNOFF-REPAIR.11.4.7.2.1.2.3.2`: the approval copies that decision's
    // derived quorum. THE CONTROL: an approver who inflates it is refused.
    let approve = |id: &'static str,
                   proposal: &'static str,
                   decision: &'static str,
                   quorum: Option<Value>| {
        let (client, base, human_id) = (client.clone(), base.clone(), human_id.clone());
        async move {
            let mut body = json!({
                "approval_id": id,
                "proposal_id": proposal,
                "decision_id": decision,
                "approver": human_id,
                "grant_id": format!("grt_{human_id}"),
            });
            if let Some(quorum) = quorum {
                body["quorum"] = quorum;
            }
            post(&client, &base, "/v1/policy-approvals", &human_id, &body).await
        }
    };
    let (status, value) = approve(
        "pdc-app-inflated",
        "pdc-decided-prop",
        "pdc-decided-dec",
        Some(json!({ "participants": [human_id, "hpr_00000000-0000-7000-8000-00000000beef"] })),
    )
    .await;
    refused(status, &value, "quorum", "an inflated quorum");
    let (status, approval) = approve("pdc-app", "pdc-decided-prop", "pdc-decided-dec", None).await;
    assert_eq!(status, 200, "the approval records: {approval}");
    assert_eq!(
        approval["quorum"], decision["electorate"],
        "the approval's quorum IS the decision's derived electorate"
    );
    // ⛔ Read back from the STORE, not the response: the response is built
    // from the derived value whatever the row holds, so a control on it alone
    // cannot see what was written (a mutation storing the request's quorum
    // survived exactly that way).
    let (status, approvals) = get(&client, &base, "/v1/policy-approvals", &human_id).await;
    assert_eq!(status, 200, "{approvals}");
    let stored = approvals
        .as_array()
        .unwrap()
        .iter()
        .find(|a| a["approval_id"] == json!("pdc-app"))
        .cloned()
        .expect("the approval is listed");
    assert_eq!(
        stored["quorum"], decision["electorate"],
        "the STORED quorum is the decision's derived electorate: {stored}"
    );

    // A decision recorded before decisions were derived has no quorum to copy.
    sqlx::query(
        "INSERT INTO policy_proposals \
         (proposal_id, policy_id, policy_version, thread_id, status, tenant_id) \
         VALUES ('pdc-legacy-prop', 'pdc-policy', '1.0.0', $1, 'decided', $2)",
    )
    .bind(&decided)
    .bind(&tenant_id)
    .execute(&pool)
    .await
    .expect("seed the legacy proposal");
    sqlx::query(
        "INSERT INTO policy_decisions \
         (decision_id, proposal_id, rule, electorate, verdict_event_id, tenant_id) \
         VALUES ('pdc-legacy-dec', 'pdc-legacy-prop', 'majority', $1, 'evt-legacy', $2)",
    )
    .bind(json!({ "participants": [human_id], "denominator": 1, "abstentions": [] }))
    .bind(&tenant_id)
    .execute(&pool)
    .await
    .expect("seed the legacy decision");
    let (status, value) =
        approve("pdc-app-legacy", "pdc-legacy-prop", "pdc-legacy-dec", None).await;
    refused(
        status,
        &value,
        "recorded before decisions were derived",
        "an underived decision",
    );
}

/// `SIGNOFF-REPAIR.9.3.5.1.2` (ROADMAP §15.8): the reconciler recovers the two
/// states a machine may recover, reports the rest, and changes nothing on a
/// second pass. Each kill point is the real sequence STOPPED at the named step:
/// the operation recorded and the Git write never made (killed before the
/// write), or the Git write made and `mark_effective` never run (killed after
/// it). Every case has its own bare repository, so one case's effective channel
/// cannot answer for another's.
#[tokio::test]
async fn the_reconciler_recovers_what_it_may_and_reports_the_rest() {
    use reasonbraid_server::reconciler::Action;
    use reasonbraid_server::reconciliation::Outcome;

    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let repo_root = reasonbraid_core::repository_root()
        .expect("the tests run inside the repository")
        .join("target/policy-publish-tests");
    let _ = std::fs::remove_dir_all(&repo_root);
    for case in ["rc-a", "rc-b", "rc-c", "rc-d", "rc-e"] {
        let dir = repo_root.join(case);
        std::fs::create_dir_all(&dir).expect("the dir creates");
        gix::init_bare(&dir).expect("the bare repo inits");
    }
    let server = TestServer::start_with_publication_root(&pool, &repo_root).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, human) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "rc-human" }),
    )
    .await;
    assert_eq!(status, 200, "the human enrolls: {human}");
    let human_id = human["principal_id"].as_str().unwrap().to_string();
    site_fixture::provision(
        &pool,
        &human_id,
        &[reasonbraid_server::site_authority::Action::PolicyRegister],
    )
    .await;
    let tenant_id = human["tenant_id"].as_str().unwrap().to_string();
    allow_owner_decides(&pool, &tenant_id).await;
    let grant_id = format!("grt_{human_id}");
    let (status, _) = register_policy(
        &client,
        &base,
        &human_id,
        &json!({
            "policy_id": "rc-policy", "version": "1.0.0",
            "lifecycle": "draft", "title": "rc", "owning_authority": grant_id,
            "clauses": [ { "id": "c1", "statement": "the reconciled clause" } ],
        }),
    )
    .await;
    assert_eq!(status, 200, "the policy registers");
    let (_, created) = post(
        &client,
        &base,
        "/v1/threads",
        &human_id,
        &json!({
            "protocol_version": reasonbraid_core::PROTOCOL_VERSION,
            "operation": "thread.create",
            "request_id": reasonbraid_core::RequestId::new().to_string(),
            "idempotency_key": "rc-create",
            "body": { "tenant_id": tenant_id, "subject": "rc", "objective": "probe",
                      "decision_rule": "owner_decides" },
            "client_context": {},
        }),
    )
    .await;
    let thread_id = created["thread_id"].as_str().unwrap().to_string();
    close_as_owner(&client, &base, &human_id, &tenant_id, &thread_id).await;

    // One staged publication per case: rc-pub-<x>.
    for x in ["a", "b", "c", "d", "e", "f"] {
        let prop = format!("rc-prop-{x}");
        let calls: [(&str, Value); 5] = [
            (
                "/v1/policy-proposals",
                json!({ "proposal_id": prop, "policy_id": "rc-policy",
                "policy_version": "1.0.0", "thread_id": thread_id }),
            ),
            (
                "/v1/policy-decisions",
                json!({ "decision_id": format!("rc-dec-{x}"),
                "proposal_id": prop }),
            ),
            (
                "/v1/policy-approvals",
                json!({ "approval_id": format!("rc-app-{x}"),
                "proposal_id": prop, "decision_id": format!("rc-dec-{x}"),
                "approver": human_id, "grant_id": grant_id }),
            ),
            (
                "/v1/policy-projections",
                json!({ "projection_id": format!("rc-proj-{x}"),
                "target": "generic", "resolution": {
                    "policies": [ { "policy_id": "rc-policy", "version": "1.0.0" } ],
                    "target": { "layer": "organization", "target": "*" } } }),
            ),
            (
                "/v1/policy-publications",
                json!({ "publication_id": format!("rc-pub-{x}"),
                "proposal_id": prop, "decision_id": format!("rc-dec-{x}"),
                "approval_id": format!("rc-app-{x}"), "projection_id": format!("rc-proj-{x}"),
                "owning_authority": grant_id }),
            ),
        ];
        for (path, body) in calls {
            let (status, value) = post(&client, &base, path, &human_id, &body).await;
            assert_eq!(status, 200, "{path} for {x}: {value}");
        }
    }

    let reconcile = |id: &'static str| {
        let (pool, root) = (pool.clone(), repo_root.clone());
        async move {
            reasonbraid_server::reconciliation::reconcile_publication(&pool, Some(&root), id)
                .await
                .expect("the pass itself does not fail")
        }
    };
    let refs = |dir: &str, id: &str| {
        reasonbraid_server::reconciler::observe(&repo_root.join(dir), id).expect("observes")
    };
    let state = |id: &'static str| {
        let pool = pool.clone();
        async move {
            reasonbraid_server::publications::load(&pool, id)
                .await
                .expect("the publication loads")
        }
    };
    let record = |id: &'static str, dir: &'static str, expected: Option<&'static str>| {
        let (pool, tenant) = (pool.clone(), tenant_id.clone());
        async move {
            reasonbraid_server::publications::record_git_operation(
                &pool, &tenant, id, dir, expected,
            )
            .await
            .expect("the operation records")
        }
    };
    // The Git half a publish performs, run directly — exactly what exists when
    // the process dies after it and before `mark_effective`.
    let write_git_half = |id: &'static str, dir: &'static str| {
        let (pool, root) = (pool.clone(), repo_root.clone());
        async move {
            let publication = reasonbraid_server::publications::load(&pool, id)
                .await
                .unwrap();
            let projection =
                reasonbraid_server::projections::load(&pool, &publication.projection_id)
                    .await
                    .unwrap();
            let manifest = reasonbraid_server::publications::manifest(
                &publication.publication_id,
                &publication.proposal_id,
                &publication.decision_id,
                &publication.approval_id,
                &publication.projection_id,
                &projection.digest,
            );
            reasonbraid_server::publisher::publish(
                &root.join(dir),
                id,
                &manifest,
                &projection.bytes,
                None,
                publication.staged_at_seconds,
            )
            .expect("the Git half writes")
        }
    };

    // A — killed BEFORE the Git write: the reconciler retries it.
    record("rc-pub-a", "rc-a", None).await;
    assert_eq!(
        refs("rc-a", "rc-pub-a").immutable,
        None,
        "nothing was written"
    );
    assert_eq!(
        reconcile("rc-pub-a").await,
        Outcome::Applied(Action::RetryStagedWrite)
    );
    assert_eq!(state("rc-pub-a").await.state, "effective");
    let settled = refs("rc-a", "rc-pub-a");
    assert!(settled.immutable.is_some());
    // …and a second pass over it changes nothing (§15.8's idempotence).
    assert_eq!(reconcile("rc-pub-a").await, Outcome::Consistent);
    assert_eq!(refs("rc-a", "rc-pub-a"), settled, "the refs did not move");
    assert_eq!(state("rc-pub-a").await.state, "effective");

    // B — killed AFTER the Git write, before `mark_effective`: verify and advance.
    record("rc-pub-b", "rc-b", None).await;
    let written = write_git_half("rc-pub-b", "rc-b").await;
    assert_eq!(
        state("rc-pub-b").await.state,
        "staged",
        "the row never heard"
    );
    assert_eq!(
        reconcile("rc-pub-b").await,
        Outcome::Applied(Action::VerifyAndAdvance)
    );
    let advanced = state("rc-pub-b").await;
    assert_eq!(advanced.state, "effective");
    assert_eq!(
        advanced.git_object_ids[0], written.publication_ref_id,
        "the SAME commit"
    );
    assert_eq!(reconcile("rc-pub-b").await, Outcome::Consistent);

    // C — a DIFFERENT commit already holds the immutable ref: stop, never pick.
    record("rc-pub-c", "rc-c", None).await;
    let foreign = reasonbraid_server::publisher::publish(
        &repo_root.join("rc-c"),
        "rc-pub-c",
        "{}",
        "# not this publication",
        None,
        0,
    )
    .expect("the foreign write lands");
    assert!(matches!(
        reconcile("rc-pub-c").await,
        Outcome::RequiresHuman {
            action: Action::StopSecurityAlert,
            ..
        }
    ));
    assert_eq!(
        state("rc-pub-c").await.state,
        "staged",
        "nothing was adjudicated"
    );
    assert_eq!(
        refs("rc-c", "rc-pub-c").immutable.map(|id| id.to_string()),
        Some(foreign.publication_ref_id),
        "the conflicting ref was not touched"
    );

    // D — failed, and its write appeared later: quarantine, NEVER promote.
    record("rc-pub-d", "rc-d", None).await;
    reasonbraid_server::publications::mark_failed(
        &pool,
        &tenant_id,
        "rc-pub-d",
        "the publisher died",
    )
    .await
    .expect("the publication fails");
    write_git_half("rc-pub-d", "rc-d").await;
    assert!(matches!(
        reconcile("rc-pub-d").await,
        Outcome::RequiresHuman {
            action: Action::QuarantineAndAdjudicate,
            ..
        }
    ));
    assert_eq!(
        state("rc-pub-d").await.state,
        "failed",
        "never silently promoted"
    );

    // E — the recorded compare-and-swap can no longer hold: reported, not forced.
    record(
        "rc-pub-e",
        "rc-e",
        Some("1111111111111111111111111111111111111111"),
    )
    .await;
    assert!(matches!(
        reconcile("rc-pub-e").await,
        Outcome::RequiresHuman {
            action: Action::RetryStagedWrite,
            ..
        }
    ));
    assert_eq!(state("rc-pub-e").await.state, "staged");

    // F — no recorded operation: nothing to observe, and nothing guessed.
    assert!(matches!(
        reconcile("rc-pub-f").await,
        Outcome::Unreconcilable(_)
    ));

    // One pass visits exactly the publications that recorded an operation.
    let mut visited = reasonbraid_server::reconciliation::candidates(&pool)
        .await
        .unwrap();
    visited.sort();
    assert_eq!(
        visited,
        ["rc-pub-a", "rc-pub-b", "rc-pub-c", "rc-pub-d", "rc-pub-e"]
    );

    let _ = std::fs::remove_dir_all(&repo_root);
}

/// `SIGNOFF-REPAIR.9.2.3`: the governance records answered a DATABASE failure as
/// a missing record, so a caller was told its policy, proposal or decision does
/// not exist when the store had failed. The failure is produced on cue: the
/// table a handler reads is renamed away for one request (this suite runs its
/// tests one at a time) and restored before anything is asserted.
#[tokio::test]
async fn a_governance_store_failure_is_the_servers_not_a_missing_record() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();
    let (status, human) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "gv-store-failure" }),
    )
    .await;
    assert_eq!(status, 200, "{human}");
    let human_id = human["principal_id"].as_str().unwrap().to_string();
    let grant_id = format!("grt_{human_id}");

    for (table, path, body) in [
        (
            "policy_versions",
            "/v1/policy-proposals",
            json!({
                "proposal_id": "gv-prop",
                "policy_id": "gv-policy",
                "policy_version": "1.0.0",
                "thread_id": "thr_gv",
            }),
        ),
        (
            "policy_proposals",
            "/v1/policy-decisions",
            json!({
                "decision_id": "gv-dec",
                "proposal_id": "gv-prop",
                "rule": "owner_decides",
                "electorate": { "participants": [human_id], "denominator": 1, "abstentions": [] },
            }),
        ),
        (
            "policy_proposals",
            "/v1/policy-approvals",
            json!({
                "approval_id": "gv-app",
                "proposal_id": "gv-prop",
                "decision_id": "gv-dec",
                "approver": human_id,
                "grant_id": grant_id,
                "quorum": { "participants": [human_id], "denominator": 1, "abstentions": [] },
            }),
        ),
        (
            "policy_proposals",
            "/v1/policy-publications",
            json!({
                "publication_id": "gv-pub",
                "proposal_id": "gv-prop",
                "decision_id": "gv-dec",
                "approval_id": "gv-app",
                "projection_id": "gv-proj",
                "owning_authority": grant_id,
            }),
        ),
    ] {
        sqlx::raw_sql(&format!("ALTER TABLE {table} RENAME TO {table}_withheld"))
            .execute(&pool)
            .await
            .expect("withhold the table");
        let (status, answered) = post(&client, &base, path, &human_id, &body).await;
        sqlx::raw_sql(&format!("ALTER TABLE {table}_withheld RENAME TO {table}"))
            .execute(&pool)
            .await
            .expect("restore the table");
        assert_eq!(
            status, 500,
            "{path} with `{table}` unreadable is the server's failure: {answered}"
        );
        assert_eq!(
            answered["code"],
            json!("dependency_unavailable"),
            "{answered}"
        );
        assert!(
            !answered["message"]
                .as_str()
                .unwrap_or_default()
                .contains("does not exist"),
            "{path}: a store failure is not reported as a missing record: {answered}"
        );
    }
}
