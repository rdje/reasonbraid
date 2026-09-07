//! The evaluation-service core (`PHASE-5.4.2`, ADR-017): the versioned corpus
//! registry (the declared 64-hex digests — the harness re-derives them at run
//! time) and the experiment run records (the workflow arm, the corpus
//! reference, the DECLARED seed — an undeclared randomness is the typed
//! refusal — the trial count, the harness's result rows). The service
//! RECORDS; the WP7 harness MEASURES (one grading implementation).
//!
//! Run with `scripts/run_pg_tests.sh` locally or the `pg-tests` CI job.
//! Without `DATABASE_URL` these skip, so `make check` stays green offline.

use std::net::SocketAddr;
use std::sync::OnceLock;

use reasonbraid_server::{api_router, ca::ensure_server_ca, node_router, PRINCIPAL_HEADER};
use serde_json::{json, Value};
use sqlx::PgPool;

static EVAL_LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();

async fn guard() -> tokio::sync::MutexGuard<'static, ()> {
    EVAL_LOCK
        .get_or_init(|| tokio::sync::Mutex::new(()))
        .lock()
        .await
}

async fn pool() -> Option<PgPool> {
    let url = match std::env::var("DATABASE_URL") {
        Ok(u) => u,
        Err(_) => {
            eprintln!(
                "SKIP: DATABASE_URL is unset — run scripts/run_pg_tests.sh for the evaluation proof"
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

const DIGEST_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const DIGEST_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

#[tokio::test]
async fn the_evaluation_service_records_the_registry_and_the_runs() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, human) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "evl-human" }),
    )
    .await;
    assert_eq!(status, 200, "the human enrolls: {human}");
    let human_id = human["principal_id"].as_str().unwrap().to_string();

    // 1. Register one corpus version (the declared digests + the cases).
    let (status, registered) = post(
        &client,
        &base,
        "/v1/evaluations/corpora",
        &human_id,
        &json!({
            "corpus_id": "bench-v1",
            "version": 1,
            "cases_digest": DIGEST_A,
            "prompts_digest": DIGEST_B,
            "cases": {
                "cases": [
                    { "id": "c1", "class": "factual", "statement": "2+2" },
                ]
            },
        }),
    )
    .await;
    assert_eq!(status, 200, "the corpus registers: {registered}");
    assert_eq!(registered["corpus_id"], json!("bench-v1"));
    assert_eq!(registered["cases_digest"], json!(DIGEST_A));

    // 2. A duplicate (same id + version) is the typed refusal — never an
    // overwrite.
    let (status, refused) = post(
        &client,
        &base,
        "/v1/evaluations/corpora",
        &human_id,
        &json!({
            "corpus_id": "bench-v1",
            "version": 1,
            "cases_digest": DIGEST_B,
            "prompts_digest": DIGEST_A,
            "cases": { "cases": [] },
        }),
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

    // 3. A malformed digest is the typed refusal.
    let (status, refused) = post(
        &client,
        &base,
        "/v1/evaluations/corpora",
        &human_id,
        &json!({
            "corpus_id": "bench-bad",
            "version": 1,
            "cases_digest": "not-hex",
            "prompts_digest": DIGEST_B,
            "cases": { "cases": [] },
        }),
    )
    .await;
    assert_eq!(status, 400, "the malformed digest refuses: {refused}");
    assert!(
        refused["message"].as_str().unwrap().contains("64-hex"),
        "{refused}"
    );

    // 4. An undeclared randomness is the typed refusal: a non-deterministic
    // run without a seed refuses; with the seed it records.
    let (status, refused) = post(
        &client,
        &base,
        "/v1/evaluations/runs",
        &human_id,
        &json!({
            "run_id": "run-1",
            "workflow": "blind",
            "corpus_id": "bench-v1",
            "corpus_version": 1,
            "trial_count": 5,
            "results": { "cases": [] },
        }),
    )
    .await;
    assert_eq!(status, 400, "the seedless run refuses: {refused}");
    assert!(
        refused["message"]
            .as_str()
            .unwrap()
            .contains("declare its seed"),
        "{refused}"
    );
    let (status, recorded) = post(
        &client,
        &base,
        "/v1/evaluations/runs",
        &human_id,
        &json!({
            "run_id": "run-1",
            "workflow": "blind",
            "corpus_id": "bench-v1",
            "corpus_version": 1,
            "seed": 42,
            "trial_count": 5,
            "results": { "cases": [ { "id": "c1", "score": 1.0 } ] },
        }),
    )
    .await;
    assert_eq!(status, 200, "the seeded run records: {recorded}");
    assert_eq!(recorded["seed"], json!(42));
    assert_eq!(recorded["trial_count"], json!(5));

    // 5. A deterministic run may omit the seed; a run referencing an
    // unregistered corpus refuses; a duplicate run id refuses.
    let (status, deterministic) = post(
        &client,
        &base,
        "/v1/evaluations/runs",
        &human_id,
        &json!({
            "run_id": "run-2",
            "workflow": "single",
            "corpus_id": "bench-v1",
            "corpus_version": 1,
            "deterministic": true,
            "trial_count": 1,
            "results": { "cases": [] },
        }),
    )
    .await;
    assert_eq!(
        status, 200,
        "the deterministic run records: {deterministic}"
    );
    assert_eq!(deterministic["seed"], Value::Null);
    let (status, refused) = post(
        &client,
        &base,
        "/v1/evaluations/runs",
        &human_id,
        &json!({
            "run_id": "run-3",
            "workflow": "blind",
            "corpus_id": "ghost",
            "corpus_version": 9,
            "seed": 1,
            "trial_count": 1,
            "results": { "cases": [] },
        }),
    )
    .await;
    assert_eq!(status, 400, "the phantom corpus refuses: {refused}");
    assert!(
        refused["message"]
            .as_str()
            .unwrap()
            .contains("not registered"),
        "{refused}"
    );
    let (status, refused) = post(
        &client,
        &base,
        "/v1/evaluations/runs",
        &human_id,
        &json!({
            "run_id": "run-1",
            "workflow": "single",
            "corpus_id": "bench-v1",
            "corpus_version": 1,
            "deterministic": true,
            "trial_count": 1,
            "results": { "cases": [] },
        }),
    )
    .await;
    assert_eq!(status, 400, "the duplicate run id refuses: {refused}");

    // 6. The reads: the registry + the runs, newest first.
    let (status, corpora) = get(&client, &base, "/v1/evaluations/corpora", &human_id).await;
    assert_eq!(status, 200, "the corpora list: {corpora}");
    assert_eq!(corpora.as_array().unwrap().len(), 1);
    assert_eq!(corpora[0]["corpus_id"], json!("bench-v1"));
    let (status, runs) = get(&client, &base, "/v1/evaluations/runs", &human_id).await;
    assert_eq!(status, 200, "the runs list: {runs}");
    let runs = runs.as_array().unwrap();
    assert_eq!(runs.len(), 2, "{runs:?}");
    assert_eq!(runs[0]["run_id"], json!("run-2"), "newest first");
    assert_eq!(runs[1]["run_id"], json!("run-1"));

    // 7. An unenrolled principal reads nothing (the enroll gate).
    let (status, _) = get(&client, &base, "/v1/evaluations/runs", "hpr_ghost").await;
    assert_eq!(status, 401, "the unenrolled read refuses");
}
