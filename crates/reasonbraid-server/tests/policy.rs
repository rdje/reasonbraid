//! The typed policy schema + the versioned registry (`PHASE-6.1.2`, ADR-019):
//! the policy is a versioned digest-pinned DOCUMENT — the stable clause ids,
//! the applicability + the non-applicability, the exception schema, and the
//! OWNERSHIP metadata (the owning authority is a GRANT reference — an
//! unresolvable owning authority is invalid at registration; the label
//! grants nothing).
//!
//! Run with `scripts/run_pg_tests.sh` locally or the `pg-tests` CI job.
//! Without `DATABASE_URL` these skip, so `make check` stays green offline.

use std::net::SocketAddr;
use std::sync::OnceLock;

use reasonbraid_server::{api_router, ca::ensure_server_ca, node_router, PRINCIPAL_HEADER};
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
    let url = match std::env::var("DATABASE_URL") {
        Ok(u) => u,
        Err(_) => {
            eprintln!(
                "SKIP: DATABASE_URL is unset — run scripts/run_pg_tests.sh for the policy proof"
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
        let router = api_router(pool.clone());
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
    let server = TestServer::start(&pool).await;
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

    // 2. The effective transition records the Git object ids.
    let (status, effective) = post(
        &client,
        &base,
        "/v1/policy-publications/pb-pub/effective",
        &human_id,
        &json!({ "git_object_ids": ["abc123", "def456"] }),
    )
    .await;
    assert_eq!(status, 200, "the publication marks effective: {effective}");
    assert_eq!(effective["state"], json!("effective"));
    assert_eq!(effective["git_object_ids"], json!(["abc123", "def456"]));

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
        &json!({ "git_object_ids": ["zzz"] }),
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
    let server = TestServer::start(&pool).await;
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

    // 1. The publish drives the Git half: the bare repo + the verb → the
    // record marks effective with the ref ids.
    let repo_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/policy-publish-tests/live");
    let _ = std::fs::remove_dir_all(&repo_dir);
    std::fs::create_dir_all(&repo_dir).expect("the dir creates");
    gix::init_bare(&repo_dir).expect("the bare repo inits");
    let (status, published) = post(
        &client,
        &base,
        "/v1/policy-publications/pu-pub/publish",
        &human_id,
        &json!({ "repo_path": repo_dir.to_string_lossy() }),
    )
    .await;
    assert_eq!(status, 200, "the publish drives the git half: {published}");
    assert_eq!(published["state"], json!("effective"));
    let object_ids = published["git_object_ids"].as_array().unwrap();
    assert_eq!(object_ids.len(), 2, "{published}");

    // 2. A publish on a NON-staged publication refuses (the effective is
    // terminal).
    let (status, refused) = post(
        &client,
        &base,
        "/v1/policy-publications/pu-pub/publish",
        &human_id,
        &json!({ "repo_path": repo_dir.to_string_lossy() }),
    )
    .await;
    assert_eq!(status, 400, "the terminal re-publish refuses: {refused}");
    assert!(
        refused["message"].as_str().unwrap().contains("staged"),
        "{refused}"
    );

    // 3. A publish into a NON-repository path refuses (the store contract's
    // open failure).
    let (status, refused) = post(
        &client,
        &base,
        "/v1/policy-publications/pu-pub/publish",
        &human_id,
        &json!({ "repo_path": "/nonexistent/repo" }),
    )
    .await;
    assert_eq!(status, 400, "the non-repository refuses: {refused}");
    let _ = std::fs::remove_dir_all(&repo_dir);
}

#[tokio::test]
async fn the_deployment_rides_the_effective_publication_per_target() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
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
                    &json!({ "git_object_ids": ["abc123", "def456"] }),
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
    let server = TestServer::start(&pool).await;
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
                &json!({ "git_object_ids": ["abc123", "def456"] }),
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
