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
