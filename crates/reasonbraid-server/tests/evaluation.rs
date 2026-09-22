//! The evaluation-service core (`PHASE-5.4.2`, ADR-017): the versioned corpus
//! registry (the declared 64-hex digests — the harness re-derives them at run
//! time) and the experiment run records (the workflow arm, the corpus
//! reference, the DECLARED seed — an undeclared randomness is the typed
//! refusal — the trial count, the harness's result rows). The service
//! RECORDS; the WP7 harness MEASURES (one grading implementation).
//!
//! Run with `scripts/run_pg_tests.sh` locally or the `pg-tests` CI job.
//! Without `DATABASE_URL` these skip, so `make check` stays green offline.

#[path = "support/mod.rs"]
mod pg_test_support;

#[path = "support/cleanup.rs"]
mod pg_cleanup;

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
    let pool = pg_test_support::pool().await?;
    sqlx::migrate!("../../migrations")
        .run(&pool)
        .await
        .expect("apply migrations");
    pg_cleanup::delete_tables(
        &pool,
        &[
            "evaluation_trials",
            "evaluation_trial_results",
            "evaluation_gates",
            "evaluation_gate_results",
            "evaluation_calibrations",
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

#[tokio::test]
async fn the_shadow_trials_record_the_seeded_assignment_and_the_cohorts() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, human) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "tri-human" }),
    )
    .await;
    assert_eq!(status, 200, "the human enrolls: {human}");
    let human_id = human["principal_id"].as_str().unwrap().to_string();

    // The trial's corpus must exist.
    let (status, _) = post(
        &client,
        &base,
        "/v1/evaluations/corpora",
        &human_id,
        &json!({
            "corpus_id": "tri-corpus",
            "version": 1,
            "cases_digest": DIGEST_A,
            "prompts_digest": DIGEST_B,
            "cases": { "cases": [] },
        }),
    )
    .await;
    assert_eq!(status, 200, "the corpus registers");

    // 1. The trial creation: the SERVER computes the seeded assignment (the
    // client supplies only the arms, the cohorts, and the case ids).
    let (status, trial) = post(
        &client,
        &base,
        "/v1/evaluations/trials",
        &human_id,
        &json!({
            "trial_id": "tri-1",
            "corpus_id": "tri-corpus",
            "corpus_version": 1,
            "seed": 7,
            "arms": ["single", "blind", "critique"],
            "cohorts": [
                { "label": "factual", "kind": "case", "members": ["c1", "c2"] },
                { "label": "expert-humans", "kind": "subject", "members": ["hpr-x"] },
            ],
            "case_ids": ["c1", "c2", "c3", "c4"],
        }),
    )
    .await;
    assert_eq!(status, 200, "the trial creates: {trial}");
    assert_eq!(trial["trial_id"], json!("tri-1"));
    let assignment = trial["assignment"].as_object().unwrap();
    assert_eq!(assignment.len(), 4, "every case is assigned: {trial}");
    for case_id in ["c1", "c2", "c3", "c4"] {
        let arm = assignment[case_id].as_str().unwrap();
        assert!(
            ["single", "blind", "critique"].contains(&arm),
            "the arm is one of the declared arms: {trial}"
        );
    }
    assert_eq!(trial["cohorts"].as_array().unwrap().len(), 2);

    // 2. The reproducibility: the SAME seed + the same cases re-draw the
    // SAME assignment (a new trial id, the identical draw).
    let (status, repeat) = post(
        &client,
        &base,
        "/v1/evaluations/trials",
        &human_id,
        &json!({
            "trial_id": "tri-1-repeat",
            "corpus_id": "tri-corpus",
            "corpus_version": 1,
            "seed": 7,
            "arms": ["single", "blind", "critique"],
            "cohorts": [],
            "case_ids": ["c1", "c2", "c3", "c4"],
        }),
    )
    .await;
    assert_eq!(status, 200, "the repeat trial creates: {repeat}");
    assert_eq!(
        repeat["assignment"], trial["assignment"],
        "the draw reproduces"
    );

    // 3. A different seed may (and generally does) draw differently — the
    // assignment is the seed's, not a guess.
    let (status, different) = post(
        &client,
        &base,
        "/v1/evaluations/trials",
        &human_id,
        &json!({
            "trial_id": "tri-2",
            "corpus_id": "tri-corpus",
            "corpus_version": 1,
            "seed": 8,
            "arms": ["single", "blind", "critique"],
            "cohorts": [],
            "case_ids": ["c1", "c2", "c3", "c4"],
        }),
    )
    .await;
    assert_eq!(status, 200, "the second trial creates: {different}");
    assert_ne!(different["assignment"], trial["assignment"]);

    // 4. The refusals: the empty arms, the empty cases, the unknown cohort
    // kind, the phantom corpus, the duplicate trial.
    for (key, body, needle) in [
        (
            "tri-empty-arms",
            json!({
                "trial_id": "tri-empty-arms",
                "corpus_id": "tri-corpus",
                "corpus_version": 1,
                "seed": 7,
                "arms": [],
                "case_ids": ["c1"],
            }),
            "arms are empty",
        ),
        (
            "tri-empty-cases",
            json!({
                "trial_id": "tri-empty-cases",
                "corpus_id": "tri-corpus",
                "corpus_version": 1,
                "seed": 7,
                "arms": ["single"],
                "case_ids": [],
            }),
            "case ids are empty",
        ),
        (
            "tri-bad-cohort",
            json!({
                "trial_id": "tri-bad-cohort",
                "corpus_id": "tri-corpus",
                "corpus_version": 1,
                "seed": 7,
                "arms": ["single"],
                "cohorts": [ { "label": "x", "kind": "nope", "members": [] } ],
                "case_ids": ["c1"],
            }),
            "unknown kind",
        ),
        (
            "tri-phantom",
            json!({
                "trial_id": "tri-phantom",
                "corpus_id": "ghost",
                "corpus_version": 1,
                "seed": 7,
                "arms": ["single"],
                "case_ids": ["c1"],
            }),
            "not registered",
        ),
    ] {
        let (status, refused) =
            post(&client, &base, "/v1/evaluations/trials", &human_id, &body).await;
        assert_eq!(status, 400, "{key}: {refused}");
        assert!(
            refused["message"].as_str().unwrap().contains(needle),
            "{refused}"
        );
    }
    let (status, refused) = post(
        &client,
        &base,
        "/v1/evaluations/trials",
        &human_id,
        &json!({
            "trial_id": "tri-1",
            "corpus_id": "tri-corpus",
            "corpus_version": 1,
            "seed": 7,
            "arms": ["single"],
            "case_ids": ["c1"],
        }),
    )
    .await;
    assert_eq!(status, 400, "the duplicate trial refuses: {refused}");

    // 5. The per-arm results are APPEND-ONLY: two submissions accumulate
    // (the record's identity is its content, never an overwrite).
    let (status, appended) = post(
        &client,
        &base,
        "/v1/evaluations/trials/tri-1/results",
        &human_id,
        &json!({ "single": { "mean": 0.9 }, "blind": { "mean": 0.8 } }),
    )
    .await;
    assert_eq!(status, 200, "the results append: {appended}");
    let (status, appended) = post(
        &client,
        &base,
        "/v1/evaluations/trials/tri-1/results",
        &human_id,
        &json!({ "critique": { "mean": 0.95 } }),
    )
    .await;
    assert_eq!(status, 200, "the second results row appends: {appended}");
    let (status, results) = get(
        &client,
        &base,
        "/v1/evaluations/trials/tri-1/results",
        &human_id,
    )
    .await;
    assert_eq!(status, 200, "the results read: {results}");
    assert_eq!(
        results.as_array().unwrap().len(),
        2,
        "both rows survive: {results}"
    );

    // 6. A result append to an unknown trial refuses.
    let (status, refused) = post(
        &client,
        &base,
        "/v1/evaluations/trials/ghost/results",
        &human_id,
        &json!({ "single": { "mean": 0.5 } }),
    )
    .await;
    assert_eq!(status, 400, "the ghost trial refuses: {refused}");

    // 7. The trials list, newest first.
    let (status, trials) = get(&client, &base, "/v1/evaluations/trials", &human_id).await;
    assert_eq!(status, 200, "the trials list: {trials}");
    let trials = trials.as_array().unwrap();
    assert_eq!(trials.len(), 3, "{trials:?}");
    assert_eq!(trials[0]["trial_id"], json!("tri-2"), "newest first");
}

#[tokio::test]
async fn the_calibration_accumulates_and_the_gate_only_blocks() {
    let _guard = guard().await;
    let Some(pool) = pool().await else { return };
    let server = TestServer::start(&pool).await;
    let base = server.base();
    let client = reqwest::Client::new();

    let (status, human) = enroll(
        &client,
        &base,
        json!({ "kind": "human", "name": "cal-human" }),
    )
    .await;
    assert_eq!(status, 200, "the human enrolls: {human}");
    let human_id = human["principal_id"].as_str().unwrap().to_string();

    let (status, _) = post(
        &client,
        &base,
        "/v1/evaluations/corpora",
        &human_id,
        &json!({
            "corpus_id": "cal-corpus",
            "version": 1,
            "cases_digest": DIGEST_A,
            "prompts_digest": DIGEST_B,
            "cases": { "cases": [] },
        }),
    )
    .await;
    assert_eq!(status, 200, "the corpus registers");
    for (run_id, seed) in [("cal-run-1", Some(1)), ("cal-run-2", Some(2))] {
        let (status, _) = post(
            &client,
            &base,
            "/v1/evaluations/runs",
            &human_id,
            &json!({
                "run_id": run_id,
                "workflow": "blind",
                "corpus_id": "cal-corpus",
                "corpus_version": 1,
                "seed": seed,
                "trial_count": 5,
                "results": { "cases": [] },
            }),
        )
        .await;
        assert_eq!(status, 200, "the run records: {run_id}");
    }

    // 1. The calibration accumulates over the NAMED runs.
    let (status, calibration) = post(
        &client,
        &base,
        "/v1/evaluations/calibrations",
        &human_id,
        &json!({
            "calibration_id": "cal-1",
            "corpus_id": "cal-corpus",
            "corpus_version": 1,
            "workflow": "blind",
            "run_ids": ["cal-run-1", "cal-run-2"],
            "brier": 0.21,
            "confidence": { "c1": { "declared": 0.9, "actual": 0.8 } },
        }),
    )
    .await;
    assert_eq!(status, 200, "the calibration records: {calibration}");
    assert_eq!(calibration["brier"], json!(0.21));

    // 2. A calibration over a GHOST run refuses (the accumulation is over
    // real measurements, never a fabrication); an out-of-range brier too.
    let (status, refused) = post(
        &client,
        &base,
        "/v1/evaluations/calibrations",
        &human_id,
        &json!({
            "calibration_id": "cal-ghost",
            "corpus_id": "cal-corpus",
            "corpus_version": 1,
            "workflow": "blind",
            "run_ids": ["ghost-run"],
            "brier": 0.5,
            "confidence": {},
        }),
    )
    .await;
    assert_eq!(status, 400, "the ghost run refuses: {refused}");
    assert!(
        refused["message"]
            .as_str()
            .unwrap()
            .contains("REGISTERED runs"),
        "{refused}"
    );
    let (status, refused) = post(
        &client,
        &base,
        "/v1/evaluations/calibrations",
        &human_id,
        &json!({
            "calibration_id": "cal-brier",
            "corpus_id": "cal-corpus",
            "corpus_version": 1,
            "workflow": "blind",
            "run_ids": ["cal-run-1"],
            "brier": 1.5,
            "confidence": {},
        }),
    )
    .await;
    assert_eq!(status, 400, "the out-of-range brier refuses: {refused}");

    // 3. The gate records the baseline + the threshold.
    let (status, gate) = post(
        &client,
        &base,
        "/v1/evaluations/gates",
        &human_id,
        &json!({
            "gate_id": "g5-blind",
            "corpus_id": "cal-corpus",
            "corpus_version": 1,
            "workflow": "blind",
            "baseline": { "c1": 0.9, "c2": 0.8 },
            "threshold": 0.1,
        }),
    )
    .await;
    assert_eq!(status, 200, "the gate records: {gate}");

    // 4. The evaluation: a case below (the baseline − the threshold) FAILS;
    // the result carries the delta; the failures name the case.
    let (status, evaluated) = post(
        &client,
        &base,
        "/v1/evaluations/gates/g5-blind/evaluations",
        &human_id,
        &json!({ "c1": 0.85, "c2": 0.5 }),
    )
    .await;
    assert_eq!(status, 200, "the evaluation appends: {evaluated}");
    assert_eq!(evaluated["passed"], json!(false));
    let failures = evaluated["failures"].as_array().unwrap();
    assert_eq!(failures.len(), 1, "{evaluated}");
    assert_eq!(failures[0]["case_id"], json!("c2"));
    assert_eq!(failures[0]["baseline"], json!(0.8));
    assert_eq!(failures[0]["measured"], json!(0.5));
    // The float subtraction's epsilon (0.8 - 0.5 in f64).
    let delta = failures[0]["delta"].as_f64().unwrap();
    assert!((delta - 0.3).abs() < 1e-9, "the delta is 0.3: {evaluated}");

    // 5. A second evaluation appends (the gate never rewrites a result).
    let (status, evaluated) = post(
        &client,
        &base,
        "/v1/evaluations/gates/g5-blind/evaluations",
        &human_id,
        &json!({ "c1": 0.95, "c2": 0.85 }),
    )
    .await;
    assert_eq!(status, 200, "the second evaluation appends: {evaluated}");
    assert_eq!(evaluated["passed"], json!(true));
    let (status, results) = get(
        &client,
        &base,
        "/v1/evaluations/gates/g5-blind/evaluations",
        &human_id,
    )
    .await;
    assert_eq!(status, 200, "the gate results read: {results}");
    assert_eq!(
        results.as_array().unwrap().len(),
        2,
        "both rows survive: {results}"
    );

    // 6. The refusals: the out-of-range threshold, the empty baseline, the
    // ghost gate.
    let (status, refused) = post(
        &client,
        &base,
        "/v1/evaluations/gates",
        &human_id,
        &json!({
            "gate_id": "g5-bad",
            "corpus_id": "cal-corpus",
            "corpus_version": 1,
            "workflow": "blind",
            "baseline": { "c1": 0.9 },
            "threshold": 2.0,
        }),
    )
    .await;
    assert_eq!(status, 400, "the out-of-range threshold refuses: {refused}");
    let (status, refused) = post(
        &client,
        &base,
        "/v1/evaluations/gates",
        &human_id,
        &json!({
            "gate_id": "g5-empty",
            "corpus_id": "cal-corpus",
            "corpus_version": 1,
            "workflow": "blind",
            "baseline": {},
            "threshold": 0.1,
        }),
    )
    .await;
    assert_eq!(status, 400, "the empty baseline refuses: {refused}");
    let (status, refused) = post(
        &client,
        &base,
        "/v1/evaluations/gates/ghost/evaluations",
        &human_id,
        &json!({ "c1": 0.9 }),
    )
    .await;
    assert_eq!(status, 400, "the ghost gate refuses: {refused}");

    // 6b. `SIGNOFF-REPAIR.8.2.1` — the read side is held to the write side's
    // rules. Before the repair each of the three below returned 200 with
    // `passed: true`, and the first of them APPENDED that pass to
    // `evaluation_gate_results`: a durable row claiming success over nothing.
    let (status, refused) = post(
        &client,
        &base,
        "/v1/evaluations/gates/g5-blind/evaluations",
        &human_id,
        &json!({}),
    )
    .await;
    assert_eq!(
        status, 400,
        "an evaluation that compares NO case refuses: {refused}"
    );
    let (status, refused) = post(
        &client,
        &base,
        "/v1/evaluations/gates/g5-blind/evaluations",
        &human_id,
        &json!({ "c1": 0.95, "c2": "oops" }),
    )
    .await;
    assert_eq!(
        status, 400,
        "a non-numeric measurement refuses rather than dropping the case: {refused}"
    );
    let (status, refused) = post(
        &client,
        &base,
        "/v1/evaluations/gates/g5-blind/evaluations",
        &human_id,
        &json!({ "c1": 0.95, "c2": 5.0 }),
    )
    .await;
    assert_eq!(
        status, 400,
        "an out-of-range measurement refuses instead of clearing every threshold: {refused}"
    );
    // ⭐ NEGATIVE: a PARTIAL evaluation is still legal — `the caller owns the
    // coverage` is a declared contract, and only the empty intersection is
    // refused. Its result names how many cases it compared, which is what a
    // reader of a `passed: true` row could not tell before.
    let (status, partial) = post(
        &client,
        &base,
        "/v1/evaluations/gates/g5-blind/evaluations",
        &human_id,
        &json!({ "c1": 0.95 }),
    )
    .await;
    assert_eq!(
        status, 200,
        "a partial evaluation still succeeds: {partial}"
    );
    assert_eq!(partial["passed"], json!(true), "{partial}");
    assert_eq!(partial["compared"], json!(1), "{partial}");
    assert_eq!(partial["unmeasured"], json!(1), "{partial}");
    // ⭐ NEGATIVE: a score at each end of the closed range is accepted, so the
    // bound is `[0, 1]` and not `(0, 1)`.
    let (status, edges) = post(
        &client,
        &base,
        "/v1/evaluations/gates/g5-blind/evaluations",
        &human_id,
        &json!({ "c1": 1.0, "c2": 0.0 }),
    )
    .await;
    assert_eq!(status, 200, "the closed range accepts 0.0 and 1.0: {edges}");
    assert_eq!(edges["compared"], json!(2), "{edges}");

    // 6c. `SIGNOFF-REPAIR.8.2.2` — a store fault is the SERVER's problem.
    // Before this repair every storage failure in `evaluation.rs` collapsed
    // into `Duplicate` or `UnknownCorpus`, so a connection loss reached the
    // caller as HTTP 400 "already exists" or "the corpus is not registered".
    // ⛔ The trigger is dropped BEFORE the assertions run, so a failing
    // expectation cannot leave this shared database rejecting corpus writes
    // (`.7.4.2`'s discipline, which this control copies deliberately).
    sqlx::raw_sql(
        "CREATE FUNCTION public.evaluation_test_gate() RETURNS trigger LANGUAGE plpgsql AS $$ \
         BEGIN RAISE EXCEPTION 'injected evaluation storage failure'; END; $$; \
         CREATE TRIGGER evaluation_test_gate BEFORE INSERT ON public.evaluation_corpora \
         FOR EACH ROW EXECUTE FUNCTION public.evaluation_test_gate()",
    )
    .execute(&pool)
    .await
    .expect("install the injected storage fault");
    let (faulted_status, faulted_body) = post(
        &client,
        &base,
        "/v1/evaluations/corpora",
        &human_id,
        &json!({
            "corpus_id": "cal-corpus-faulted",
            "version": 1,
            "cases_digest": DIGEST_A,
            "prompts_digest": DIGEST_B,
            "cases": { "cases": [] },
        }),
    )
    .await;
    sqlx::raw_sql(
        "DROP TRIGGER IF EXISTS evaluation_test_gate ON public.evaluation_corpora; \
         DROP FUNCTION IF EXISTS public.evaluation_test_gate()",
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

    // ⭐ NEGATIVE 1: the repair did not turn every refusal into a 500. A
    // GENUINE duplicate is still the caller's error, and it is the exact
    // message the store fault used to borrow.
    let (dup_status, dup_body) = post(
        &client,
        &base,
        "/v1/evaluations/corpora",
        &human_id,
        &json!({
            "corpus_id": "cal-corpus",
            "version": 1,
            "cases_digest": DIGEST_A,
            "prompts_digest": DIGEST_B,
            "cases": { "cases": [] },
        }),
    )
    .await;
    assert_eq!(
        dup_status, 400,
        "a real duplicate is still the caller's error: {dup_body}"
    );
    assert_eq!(dup_body["code"], json!("invalid_command"), "{dup_body}");
    // ⭐ NEGATIVE 2: and so is a genuinely unregistered corpus, which reaches
    // the caller through the existence BOOLEAN rather than through the query's
    // error arm — the distinction this repair rests on.
    let (ghost_status, ghost_body) = post(
        &client,
        &base,
        "/v1/evaluations/runs",
        &human_id,
        &json!({
            "run_id": "cal-run-ghost",
            "workflow": "blind",
            "corpus_id": "cal-corpus-absent",
            "corpus_version": 1,
            "seed": 7,
            "deterministic": true,
            "trial_count": 1,
            "results": {},
        }),
    )
    .await;
    assert_eq!(
        ghost_status, 400,
        "an unregistered corpus is still the caller's error: {ghost_body}"
    );
    assert_eq!(ghost_body["code"], json!("invalid_command"), "{ghost_body}");

    // 7. The lists.
    let (status, calibrations) =
        get(&client, &base, "/v1/evaluations/calibrations", &human_id).await;
    assert_eq!(status, 200, "the calibrations list: {calibrations}");
    assert_eq!(calibrations.as_array().unwrap().len(), 1);
    let (status, gates) = get(&client, &base, "/v1/evaluations/gates", &human_id).await;
    assert_eq!(status, 200, "the gates list: {gates}");
    assert_eq!(gates.as_array().unwrap().len(), 1);
    assert_eq!(gates[0]["gate_id"], json!("g5-blind"));

    // ⛔ 6d RUNS LAST, DELIBERATELY. Its negative controls REGISTER a gate and a
    // calibration, and the list assertions above pin exact counts — so placing
    // it earlier would have made this leaf's additions rewrite an existing
    // expectation, which is how a walk quietly stops asserting what it did.
    // `SIGNOFF-REPAIR.8.2.4` — a row that names a corpus BINDS to it.
    // Before this repair both of the next two returned 200: a gate could be
    // registered against a corpus version that does not exist, and a
    // calibration could accumulate a run taken against a different workflow,
    // while each stored row asserted a provenance nothing held.
    let (status, refused) = post(
        &client,
        &base,
        "/v1/evaluations/gates",
        &human_id,
        &json!({
            "gate_id": "g5-unbound",
            "corpus_id": "cal-corpus-absent",
            "corpus_version": 1,
            "workflow": "blind",
            "baseline": { "c1": 0.9 },
            "threshold": 0.1,
        }),
    )
    .await;
    assert_eq!(
        status, 400,
        "a gate against an unregistered corpus refuses: {refused}"
    );
    // A run that EXISTS but belongs to another workflow is not eligible, and
    // the refusal must NAME what disagreed rather than saying no.
    let (status, _) = post(
        &client,
        &base,
        "/v1/evaluations/runs",
        &human_id,
        &json!({
            "run_id": "cal-run-other-workflow",
            "workflow": "sighted",
            "corpus_id": "cal-corpus",
            "corpus_version": 1,
            "seed": 9,
            "deterministic": true,
            "trial_count": 1,
            "results": {},
        }),
    )
    .await;
    assert_eq!(status, 200, "the other-workflow run records");
    let (status, refused) = post(
        &client,
        &base,
        "/v1/evaluations/calibrations",
        &human_id,
        &json!({
            "calibration_id": "cal-ineligible",
            "corpus_id": "cal-corpus",
            "corpus_version": 1,
            "workflow": "blind",
            "run_ids": ["cal-run-1", "cal-run-other-workflow"],
            "brier": 0.2,
            "confidence": 0.8,
        }),
    )
    .await;
    assert_eq!(
        status, 400,
        "a calibration over an ineligible run refuses: {refused}"
    );
    let message = refused["message"].as_str().unwrap_or_default();
    assert!(
        message.contains("cal-run-other-workflow") && message.contains("sighted"),
        "the refusal names the run AND what disagreed: {refused}"
    );
    // ⭐ NEGATIVE: a check that refuses everything is this defect mirrored. A
    // gate against the REGISTERED corpus and a calibration over ELIGIBLE runs
    // both still succeed.
    let (status, bound) = post(
        &client,
        &base,
        "/v1/evaluations/gates",
        &human_id,
        &json!({
            "gate_id": "g5-bound",
            "corpus_id": "cal-corpus",
            "corpus_version": 1,
            "workflow": "blind",
            "baseline": { "c1": 0.9 },
            "threshold": 0.1,
        }),
    )
    .await;
    assert_eq!(
        status, 200,
        "a gate against a registered corpus still records: {bound}"
    );
    let (status, eligible) = post(
        &client,
        &base,
        "/v1/evaluations/calibrations",
        &human_id,
        &json!({
            "calibration_id": "cal-eligible",
            "corpus_id": "cal-corpus",
            "corpus_version": 1,
            "workflow": "blind",
            "run_ids": ["cal-run-1", "cal-run-2"],
            "brier": 0.2,
            "confidence": 0.8,
        }),
    )
    .await;
    assert_eq!(
        status, 200,
        "a calibration over eligible runs still records: {eligible}"
    );
}
