//! The rule-based routing policy (`PHASE-5.5.2`, ADR-031): the deterministic
//! class→arm table (the §13.8 rows as the built-in rules). The case class is
//! a SUBMITTED input; the resolution is a lookup (the same class always
//! resolves to the same arm); every resolution rides the audit table; the
//! create boundary applies the policy ONLY when no explicit profile is named
//! (the human authority outranks the rule).
//!
//! Run with `scripts/run_pg_tests.sh` locally or the `pg-tests` CI job.
//! Without `DATABASE_URL` these skip, so `make check` stays green offline.

use std::net::SocketAddr;
use std::sync::OnceLock;

use reasonbraid_server::{api_router, ca::ensure_server_ca, node_router, PRINCIPAL_HEADER};
use serde_json::{json, Value};
use sqlx::PgPool;

static ROUTING_LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();

async fn guard() -> tokio::sync::MutexGuard<'static, ()> {
    ROUTING_LOCK
        .get_or_init(|| tokio::sync::Mutex::new(()))
        .lock()
        .await
}

async fn pool() -> Option<PgPool> {
    let url = match std::env::var("DATABASE_URL") {
        Ok(u) => u,
        Err(_) => {
            eprintln!(
                "SKIP: DATABASE_URL is unset — run scripts/run_pg_tests.sh for the routing proof"
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
    // NOTE: `routing_rules` is the migration's seeded constant — NOT purged
    // (the built-in rules must survive between tests). `workflow_profiles`
    // is the `.1.2` registry's seeded set — also NOT purged.
    for table in [
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

#[tokio::test]
async fn the_rule_based_policy_routes_deterministically() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, human) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "rt-human" }),
    )
    .await;
    assert_eq!(status, 200, "the human enrolls: {human}");
    let human_id = human["principal_id"].as_str().unwrap().to_string();
    let tenant_id = human["tenant_id"].as_str().unwrap().to_string();

    // 1. The rule table: the seven §13.8 rows.
    let (status, rules) = get(&client, &base, "/v1/routing/rules", &human_id).await;
    assert_eq!(status, 200, "the rules read: {rules}");
    let rules = rules.as_array().unwrap();
    assert_eq!(rules.len(), 7, "{rules:?}");
    let rule_for = |case_class: &str| {
        rules
            .iter()
            .find(|r| r["case_class"] == json!(case_class))
            .cloned()
            .expect("the rule exists")
    };
    assert_eq!(rule_for("simple")["arm"], json!("quick_advice"));
    assert_eq!(rule_for("factual")["arm"], json!("evidence_review"));
    assert_eq!(rule_for("uncertain")["arm"], json!("independent_panel"));
    assert_eq!(rule_for("design_policy")["arm"], json!("critique"));
    assert_eq!(rule_for("governed")["arm"], json!("policy_proposal"));
    assert_eq!(rule_for("correlated")["arm"], json!("independent_panel"));
    assert_eq!(rule_for("diminishing")["arm"], json!("quick_advice"));

    // 2. The deterministic resolution: the same class → the same arm + the
    // same rule id (the lookup is the policy's whole arithmetic).
    let (status, resolved) = post(
        &client,
        &base,
        "/v1/routing/resolve",
        &human_id,
        &json!({ "case_class": "uncertain" }),
    )
    .await;
    assert_eq!(status, 200, "the resolution: {resolved}");
    assert_eq!(resolved["arm"], json!("independent_panel"));
    assert_eq!(resolved["rule_id"], json!("rule_uncertain"));
    let (status, again) = post(
        &client,
        &base,
        "/v1/routing/resolve",
        &human_id,
        &json!({ "case_class": "uncertain" }),
    )
    .await;
    assert_eq!(status, 200, "the second resolution: {again}");
    assert_eq!(again["arm"], resolved["arm"], "the lookup repeats");

    // 3. An unknown class is the typed refusal (the vocabulary is closed).
    let (status, refused) = post(
        &client,
        &base,
        "/v1/routing/resolve",
        &human_id,
        &json!({ "case_class": "not_a_class" }),
    )
    .await;
    assert_eq!(status, 400, "the unknown class refuses: {refused}");
    assert!(
        refused["message"].as_str().unwrap().contains("vocabulary"),
        "{refused}"
    );

    // 4. The create boundary: the class + NO explicit profile → the policy's
    // arm (the projection carries it with its steps).
    let (status, created) = post(
        &client,
        &base,
        "/v1/threads",
        &human_id,
        &json!({
            "protocol_version": reasonbraid_core::PROTOCOL_VERSION,
            "operation": "thread.create",
            "request_id": reasonbraid_core::RequestId::new().to_string(),
            "idempotency_key": "rt-routed",
            "body": {
                "tenant_id": tenant_id,
                "subject": "routed",
                "objective": "probe",
                "routing_class": "uncertain",
            },
            "client_context": {},
        }),
    )
    .await;
    assert_eq!(status, 200, "the routed create: {created}");
    let routed_id = created["thread_id"].as_str().unwrap().to_string();
    let (status, state) = get(
        &client,
        &base,
        &format!("/v1/threads/{routed_id}?tenant_id={tenant_id}"),
        &human_id,
    )
    .await;
    assert_eq!(status, 200, "the routed thread reads: {state}");
    assert_eq!(
        state["state"]["workflow_profile"],
        json!("independent_panel")
    );
    assert_eq!(
        state["state"]["workflow_steps"],
        json!(["blind_solicit", "adjudicate", "decide"])
    );

    // 5. The explicit profile outranks the rule (the human authority wins).
    let (status, created) = post(
        &client,
        &base,
        "/v1/threads",
        &human_id,
        &json!({
            "protocol_version": reasonbraid_core::PROTOCOL_VERSION,
            "operation": "thread.create",
            "request_id": reasonbraid_core::RequestId::new().to_string(),
            "idempotency_key": "rt-explicit",
            "body": {
                "tenant_id": tenant_id,
                "subject": "explicit",
                "objective": "probe",
                "routing_class": "uncertain",
                "workflow_profile": "critique",
            },
            "client_context": {},
        }),
    )
    .await;
    assert_eq!(status, 200, "the explicit create: {created}");
    let explicit_id = created["thread_id"].as_str().unwrap().to_string();
    let (status, state) = get(
        &client,
        &base,
        &format!("/v1/threads/{explicit_id}?tenant_id={tenant_id}"),
        &human_id,
    )
    .await;
    assert_eq!(status, 200, "the explicit thread reads: {state}");
    assert_eq!(state["state"]["workflow_profile"], json!("critique"));

    // 6. The bare create stays the default (the policy never overrides).
    let (status, created) = post(
        &client,
        &base,
        "/v1/threads",
        &human_id,
        &json!({
            "protocol_version": reasonbraid_core::PROTOCOL_VERSION,
            "operation": "thread.create",
            "request_id": reasonbraid_core::RequestId::new().to_string(),
            "idempotency_key": "rt-bare",
            "body": {
                "tenant_id": tenant_id,
                "subject": "bare",
                "objective": "probe",
            },
            "client_context": {},
        }),
    )
    .await;
    assert_eq!(status, 200, "the bare create: {created}");
    let bare_id = created["thread_id"].as_str().unwrap().to_string();
    let (status, state) = get(
        &client,
        &base,
        &format!("/v1/threads/{bare_id}?tenant_id={tenant_id}"),
        &human_id,
    )
    .await;
    assert_eq!(status, 200, "the bare thread reads: {state}");
    assert_eq!(state["state"]["workflow_profile"], json!("quick_advice"));

    // 7. An unknown class on the create boundary is the typed refusal.
    let (status, refused) = post(
        &client,
        &base,
        "/v1/threads",
        &human_id,
        &json!({
            "protocol_version": reasonbraid_core::PROTOCOL_VERSION,
            "operation": "thread.create",
            "request_id": reasonbraid_core::RequestId::new().to_string(),
            "idempotency_key": "rt-unknown-class",
            "body": {
                "tenant_id": tenant_id,
                "subject": "unknown",
                "objective": "probe",
                "routing_class": "not_a_class",
            },
            "client_context": {},
        }),
    )
    .await;
    assert_eq!(status, 400, "the unknown class create refuses: {refused}");

    // 8. The audit: the resolve verb's two rows + the routed create's
    // create_boundary row (the explicit + the bare creates recorded none).
    let (status, resolutions) = get(&client, &base, "/v1/routing/resolutions", &human_id).await;
    assert_eq!(status, 200, "the resolutions read: {resolutions}");
    let resolutions = resolutions.as_array().unwrap();
    assert_eq!(resolutions.len(), 3, "{resolutions:?}");
    let surfaces: Vec<&str> = resolutions
        .iter()
        .map(|r| r["surface"].as_str().unwrap())
        .collect();
    assert!(surfaces.contains(&"create_boundary"), "{surfaces:?}");
    assert!(surfaces.contains(&"resolve_verb"), "{surfaces:?}");
}

#[tokio::test]
async fn the_shadow_recommendation_records_and_never_applies() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, human) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "rec-human" }),
    )
    .await;
    assert_eq!(status, 200, "the human enrolls: {human}");
    let human_id = human["principal_id"].as_str().unwrap().to_string();
    let tenant_id = human["tenant_id"].as_str().unwrap().to_string();

    // The evidence: a `.4` trial (the recommendation names it).
    let digest_a = "a".repeat(64);
    let digest_b = "b".repeat(64);
    let (status, _) = post(
        &client,
        &base,
        "/v1/evaluations/corpora",
        &human_id,
        &json!({
            "corpus_id": "rec-corpus",
            "version": 1,
            "cases_digest": digest_a,
            "prompts_digest": digest_b,
            "cases": { "cases": [] },
        }),
    )
    .await;
    assert_eq!(status, 200, "the corpus registers");
    let (status, trial) = post(
        &client,
        &base,
        "/v1/evaluations/trials",
        &human_id,
        &json!({
            "trial_id": "rec-trial",
            "corpus_id": "rec-corpus",
            "corpus_version": 1,
            "seed": 7,
            "arms": ["single", "blind"],
            "case_ids": ["c1"],
        }),
    )
    .await;
    assert_eq!(status, 200, "the trial creates: {trial}");

    // 1. The recommendation records with `applied: false` — the shadow is
    // stated on the record itself.
    let (status, recorded) = post(
        &client,
        &base,
        "/v1/routing/recommendations",
        &human_id,
        &json!({
            "recommendation_id": "rec-1",
            "case_class": "uncertain",
            "arm": "critique",
            "evidence_ref": "rec-trial",
        }),
    )
    .await;
    assert_eq!(status, 200, "the recommendation records: {recorded}");
    assert_eq!(recorded["applied"], json!(false));

    // 2. The never-a-raise constraint: an arm outside the registered set
    // refuses (the recommendation can re-order what exists, not invent).
    let (status, refused) = post(
        &client,
        &base,
        "/v1/routing/recommendations",
        &human_id,
        &json!({
            "recommendation_id": "rec-2",
            "case_class": "uncertain",
            "arm": "not_a_profile",
            "evidence_ref": "rec-trial",
        }),
    )
    .await;
    assert_eq!(status, 400, "the phantom arm refuses: {refused}");
    assert!(
        refused["message"]
            .as_str()
            .unwrap()
            .contains("registered profile"),
        "{refused}"
    );

    // 3. A ghost evidence reference refuses (the recommendation names the
    // evidence it rests on).
    let (status, refused) = post(
        &client,
        &base,
        "/v1/routing/recommendations",
        &human_id,
        &json!({
            "recommendation_id": "rec-3",
            "case_class": "uncertain",
            "arm": "critique",
            "evidence_ref": "ghost-trial",
        }),
    )
    .await;
    assert_eq!(status, 400, "the ghost evidence refuses: {refused}");

    // 4. An unknown class refuses.
    let (status, refused) = post(
        &client,
        &base,
        "/v1/routing/recommendations",
        &human_id,
        &json!({
            "recommendation_id": "rec-4",
            "case_class": "not_a_class",
            "arm": "critique",
            "evidence_ref": "rec-trial",
        }),
    )
    .await;
    assert_eq!(status, 400, "the unknown class refuses: {refused}");

    // 5. The recommendation is NEVER applied: the create boundary keeps
    // resolving the RULE table (the class `uncertain` still routes to
    // `independent_panel`, NOT the recommended `critique`).
    let (status, created) = post(
        &client,
        &base,
        "/v1/threads",
        &human_id,
        &json!({
            "protocol_version": reasonbraid_core::PROTOCOL_VERSION,
            "operation": "thread.create",
            "request_id": reasonbraid_core::RequestId::new().to_string(),
            "idempotency_key": "rec-shadow-create",
            "body": {
                "tenant_id": tenant_id,
                "subject": "shadow",
                "objective": "probe",
                "routing_class": "uncertain",
            },
            "client_context": {},
        }),
    )
    .await;
    assert_eq!(status, 200, "the shadow create: {created}");
    let thread_id = created["thread_id"].as_str().unwrap().to_string();
    let (status, state) = get(
        &client,
        &base,
        &format!("/v1/threads/{thread_id}?tenant_id={tenant_id}"),
        &human_id,
    )
    .await;
    assert_eq!(status, 200, "the shadow thread reads: {state}");
    assert_eq!(
        state["state"]["workflow_profile"],
        json!("independent_panel"),
        "the RULE's arm, not the recommendation's"
    );

    // 6. The list shows the record with the stated non-application.
    let (status, recommendations) =
        get(&client, &base, "/v1/routing/recommendations", &human_id).await;
    assert_eq!(status, 200, "the recommendations read: {recommendations}");
    let recommendations = recommendations.as_array().unwrap();
    assert_eq!(recommendations.len(), 1, "{recommendations:?}");
    assert_eq!(recommendations[0]["arm"], json!("critique"));
    assert_eq!(recommendations[0]["applied"], json!(false));
}
