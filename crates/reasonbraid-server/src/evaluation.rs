//! The evaluation-service core (`PHASE-5.4.2`, ADR-017): the versioned case
//! registry + the experiment run records. The service RECORDS; the WP7 bench
//! harness MEASURES (one grading implementation — the harness's outputs
//! persist here; a second judge would invite drift).
//!
//! The invariants (ADR-017, enforced here):
//! - the registry is versioned + content-addressed (the declared 64-hex
//!   digests; a re-run with the same version + seed must reproduce);
//! - the run DECLARES its seed — a non-deterministic run without one is the
//!   typed refusal (an undeclared randomness is never a silent guess);
//! - a run references a REGISTERED corpus version (never a phantom);
//! - a duplicate registry row or run id is the typed refusal, never an
//!   overwrite (the record's identity is its content).

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::PgPool;

/// The registry-row shape a client submits (`.4.2`): the corpus id + version,
/// the declared 64-hex digests, and the cases document.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CorpusRegistration {
    pub corpus_id: String,
    pub version: i64,
    pub cases_digest: String,
    pub prompts_digest: String,
    pub cases: Value,
}

/// The run-row shape a client submits (`.4.2`): the workflow arm, the corpus
/// reference, the declared seed + determinism, and the harness's results.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunRecord {
    pub run_id: String,
    pub workflow: String,
    pub corpus_id: String,
    pub corpus_version: i64,
    #[serde(default)]
    pub seed: Option<i64>,
    #[serde(default)]
    pub deterministic: bool,
    pub trial_count: i64,
    pub results: Value,
}

/// The register verb's outcome: the row as stored.
#[derive(Debug, Clone, Serialize)]
pub struct RegisteredCorpus {
    pub corpus_id: String,
    pub version: i64,
    pub cases_digest: String,
    pub prompts_digest: String,
    pub cases: Value,
}

/// A run row as stored.
#[derive(Debug, Clone, Serialize)]
pub struct StoredRun {
    pub run_id: String,
    pub workflow: String,
    pub corpus_id: String,
    pub corpus_version: i64,
    pub seed: Option<i64>,
    pub deterministic: bool,
    pub trial_count: i64,
    pub results: Value,
}

/// The typed refusal reasons — the caller maps them to an HTTP error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvaluationError {
    /// The digest is not a 64-hex string (the harness's shape).
    MalformedDigest(String),
    /// The (corpus, version) or the run id already exists — never an
    /// overwrite.
    Duplicate(String),
    /// The run references a corpus version that is not registered.
    UnknownCorpus(String),
    /// A non-deterministic run without a declared seed.
    UndeclaredSeed,
    /// The trial count is not positive.
    InvalidTrialCount(i64),
}

impl std::fmt::Display for EvaluationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvaluationError::MalformedDigest(d) => {
                write!(f, "digest `{d}` is not a 64-hex string")
            }
            EvaluationError::Duplicate(what) => {
                write!(
                    f,
                    "{what} already exists — the record's identity is its content, \
                     register a new version or run id instead of overwriting"
                )
            }
            EvaluationError::UnknownCorpus(c) => {
                write!(f, "corpus `{c}` at that version is not registered")
            }
            EvaluationError::UndeclaredSeed => {
                write!(
                    f,
                    "a non-deterministic run must declare its seed (or set \
                     `deterministic: true`) — an undeclared randomness is the typed refusal"
                )
            }
            EvaluationError::InvalidTrialCount(n) => {
                write!(f, "the trial count {n} is not positive")
            }
        }
    }
}

/// The 64-hex digest shape (the harness's sha256 hex over the file bytes).
fn is_hex64(digest: &str) -> bool {
    digest.len() == 64 && digest.bytes().all(|b| b.is_ascii_hexdigit())
}

/// Register one corpus version (the content-addressed registry row).
pub async fn register_corpus(
    pool: &PgPool,
    registration: &CorpusRegistration,
) -> Result<RegisteredCorpus, EvaluationError> {
    if !is_hex64(&registration.cases_digest) {
        return Err(EvaluationError::MalformedDigest(
            registration.cases_digest.clone(),
        ));
    }
    if !is_hex64(&registration.prompts_digest) {
        return Err(EvaluationError::MalformedDigest(
            registration.prompts_digest.clone(),
        ));
    }
    let inserted = sqlx::query(
        "INSERT INTO evaluation_corpora \
         (corpus_id, version, cases_digest, prompts_digest, cases) \
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(&registration.corpus_id)
    .bind(registration.version)
    .bind(&registration.cases_digest)
    .bind(&registration.prompts_digest)
    .bind(&registration.cases)
    .execute(pool)
    .await;
    match inserted {
        Ok(_) => Ok(RegisteredCorpus {
            corpus_id: registration.corpus_id.clone(),
            version: registration.version,
            cases_digest: registration.cases_digest.clone(),
            prompts_digest: registration.prompts_digest.clone(),
            cases: registration.cases.clone(),
        }),
        Err(_) => Err(EvaluationError::Duplicate(format!(
            "corpus `{}` version {}",
            registration.corpus_id, registration.version
        ))),
    }
}

/// Record one experiment run (the seed-declaring record).
pub async fn record_run(pool: &PgPool, run: &RunRecord) -> Result<StoredRun, EvaluationError> {
    if run.trial_count < 1 {
        return Err(EvaluationError::InvalidTrialCount(run.trial_count));
    }
    if !run.deterministic && run.seed.is_none() {
        return Err(EvaluationError::UndeclaredSeed);
    }
    let corpus_exists: Option<bool> = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM evaluation_corpora \
         WHERE corpus_id = $1 AND version = $2)",
    )
    .bind(&run.corpus_id)
    .bind(run.corpus_version)
    .fetch_one(pool)
    .await
    .map_err(|_| EvaluationError::UnknownCorpus(run.corpus_id.clone()))?;
    if !corpus_exists.unwrap_or(false) {
        return Err(EvaluationError::UnknownCorpus(run.corpus_id.clone()));
    }
    let inserted = sqlx::query(
        "INSERT INTO evaluation_runs \
         (run_id, workflow, corpus_id, corpus_version, seed, deterministic, trial_count, results) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
    )
    .bind(&run.run_id)
    .bind(&run.workflow)
    .bind(&run.corpus_id)
    .bind(run.corpus_version)
    .bind(run.seed)
    .bind(run.deterministic)
    .bind(run.trial_count)
    .bind(&run.results)
    .execute(pool)
    .await;
    match inserted {
        Ok(_) => Ok(StoredRun {
            run_id: run.run_id.clone(),
            workflow: run.workflow.clone(),
            corpus_id: run.corpus_id.clone(),
            corpus_version: run.corpus_version,
            seed: run.seed,
            deterministic: run.deterministic,
            trial_count: run.trial_count,
            results: run.results.clone(),
        }),
        Err(_) => Err(EvaluationError::Duplicate(format!("run `{}`", run.run_id))),
    }
}

/// The stored row shape (the query tuple — clippy's type-complexity line).
type CorpusRow = (String, i64, String, String, Value);
/// The stored run row shape.
type RunRow = (String, String, String, i64, Option<i64>, bool, i64, Value);
/// The stored trial row shape.
type TrialRow = (String, String, i64, i64, Value, Value, Value, Value);

/// The registry's latest versions (one row per corpus id).
pub async fn list_corpora(pool: &PgPool) -> Result<Vec<RegisteredCorpus>, sqlx::Error> {
    let rows: Vec<CorpusRow> = sqlx::query_as(
        "SELECT corpus_id, version, cases_digest, prompts_digest, cases \
         FROM evaluation_corpora ORDER BY corpus_id, version",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(
            |(corpus_id, version, cases_digest, prompts_digest, cases)| RegisteredCorpus {
                corpus_id,
                version,
                cases_digest,
                prompts_digest,
                cases,
            },
        )
        .collect())
}

/// The recorded runs, newest first.
pub async fn list_runs(pool: &PgPool) -> Result<Vec<StoredRun>, sqlx::Error> {
    let rows: Vec<RunRow> = sqlx::query_as(
        "SELECT run_id, workflow, corpus_id, corpus_version, seed, deterministic, \
         trial_count, results FROM evaluation_runs ORDER BY created_at DESC",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(
            |(
                run_id,
                workflow,
                corpus_id,
                corpus_version,
                seed,
                deterministic,
                trial_count,
                results,
            )| {
                StoredRun {
                    run_id,
                    workflow,
                    corpus_id,
                    corpus_version,
                    seed,
                    deterministic,
                    trial_count,
                    results,
                }
            },
        )
        .collect())
}

// ── The randomized routing trials + the cohorts (`.4.3`, ADR-017) ────────────────

/// One cohort record (`.4.3`): a recorded LABEL over the trial's subjects or
/// cases — never a derived claim (the reports aggregate only what is
/// recorded here).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CohortRecord {
    pub label: String,
    /// `case` (the case ids the cohort covers) or `subject` (the subject ids).
    pub kind: String,
    pub members: Vec<String>,
}

/// The trial submission (`.4.3`): the shadow experiment — the arms, the
/// recorded cohorts, and the cases. The SERVER computes the seeded
/// assignment (the client never supplies it, so the draw is reproducible
/// from the record alone).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TrialSubmission {
    pub trial_id: String,
    pub corpus_id: String,
    pub corpus_version: i64,
    pub seed: i64,
    pub arms: Vec<String>,
    #[serde(default)]
    pub cohorts: Vec<CohortRecord>,
    pub case_ids: Vec<String>,
}

/// The stored trial (the computed assignment rides it).
#[derive(Debug, Clone, Serialize)]
pub struct StoredTrial {
    pub trial_id: String,
    pub corpus_id: String,
    pub corpus_version: i64,
    pub seed: i64,
    pub arms: Vec<String>,
    pub cohorts: Vec<CohortRecord>,
    pub case_ids: Vec<String>,
    pub assignment: serde_json::Map<String, serde_json::Value>,
}

impl EvaluationError {
    fn empty_arms() -> Self {
        EvaluationError::MalformedDigest("the arms are empty".to_string())
    }
    fn empty_cases() -> Self {
        EvaluationError::MalformedDigest("the case ids are empty".to_string())
    }
    fn bad_cohort(label: &str) -> Self {
        EvaluationError::MalformedDigest(format!(
            "cohort `{label}` has an unknown kind (expected `case` or `subject`)"
        ))
    }
}

/// The stable splitmix64 over (seed, bytes) — a dependency-free deterministic
/// draw so the same seed + case re-draws the same arm on every run (the
/// `std` hasher is NOT stable across releases; the assignment must be).
fn splitmix64(seed: u64, input: &[u8]) -> u64 {
    let mut z = seed.wrapping_add(0x9e37_79b9_7f4a_7c15);
    for &b in input {
        z = z.wrapping_add(u64::from(b));
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^= z >> 31;
    }
    z
}

/// Create the shadow trial: the seeded assignment is SERVER-computed (the
/// record alone reproduces it — the client never supplies a draw).
pub async fn create_trial(
    pool: &PgPool,
    submission: &TrialSubmission,
) -> Result<StoredTrial, EvaluationError> {
    if submission.arms.is_empty() {
        return Err(EvaluationError::empty_arms());
    }
    if submission.case_ids.is_empty() {
        return Err(EvaluationError::empty_cases());
    }
    for cohort in &submission.cohorts {
        if cohort.kind != "case" && cohort.kind != "subject" {
            return Err(EvaluationError::bad_cohort(&cohort.label));
        }
    }
    let corpus_exists: Option<bool> = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM evaluation_corpora \
         WHERE corpus_id = $1 AND version = $2)",
    )
    .bind(&submission.corpus_id)
    .bind(submission.corpus_version)
    .fetch_one(pool)
    .await
    .map_err(|_| EvaluationError::UnknownCorpus(submission.corpus_id.clone()))?;
    if !corpus_exists.unwrap_or(false) {
        return Err(EvaluationError::UnknownCorpus(submission.corpus_id.clone()));
    }

    // The seeded draw: splitmix64 over the (case id, seed) — stable across
    // runs and platforms (the std hasher is not).
    let mut assignment = serde_json::Map::new();
    for case_id in &submission.case_ids {
        let draw = splitmix64(submission.seed as u64, case_id.as_bytes());
        let arm = &submission.arms[(draw as usize) % submission.arms.len()];
        assignment.insert(case_id.clone(), serde_json::Value::String(arm.clone()));
    }

    let inserted = sqlx::query(
        "INSERT INTO evaluation_trials \
         (trial_id, corpus_id, corpus_version, seed, arms, cohorts, case_ids, assignment) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
    )
    .bind(&submission.trial_id)
    .bind(&submission.corpus_id)
    .bind(submission.corpus_version)
    .bind(submission.seed)
    .bind(serde_json::to_value(&submission.arms).expect("the arms serialize"))
    .bind(serde_json::to_value(&submission.cohorts).expect("the cohorts serialize"))
    .bind(serde_json::to_value(&submission.case_ids).expect("the case ids serialize"))
    .bind(serde_json::to_value(&assignment).expect("the assignment serializes"))
    .execute(pool)
    .await;
    match inserted {
        Ok(_) => Ok(StoredTrial {
            trial_id: submission.trial_id.clone(),
            corpus_id: submission.corpus_id.clone(),
            corpus_version: submission.corpus_version,
            seed: submission.seed,
            arms: submission.arms.clone(),
            cohorts: submission.cohorts.clone(),
            case_ids: submission.case_ids.clone(),
            assignment,
        }),
        Err(_) => Err(EvaluationError::Duplicate(format!(
            "trial `{}`",
            submission.trial_id
        ))),
    }
}

/// Append one per-arm results row (append-only — never an overwrite).
pub async fn record_trial_results(
    pool: &PgPool,
    trial_id: &str,
    results: &Value,
) -> Result<(), EvaluationError> {
    let trial_exists: Option<bool> =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM evaluation_trials WHERE trial_id = $1)")
            .bind(trial_id)
            .fetch_one(pool)
            .await
            .map_err(|_| EvaluationError::Duplicate(trial_id.to_string()))?;
    if !trial_exists.unwrap_or(false) {
        return Err(EvaluationError::Duplicate(format!(
            "trial `{trial_id}` (the results append to a REGISTERED trial)"
        )));
    }
    sqlx::query("INSERT INTO evaluation_trial_results (trial_id, results) VALUES ($1, $2)")
        .bind(trial_id)
        .bind(results)
        .execute(pool)
        .await
        .map_err(|_| EvaluationError::Duplicate("trial result".to_string()))?;
    Ok(())
}

/// The trials, newest first.
pub async fn list_trials(pool: &PgPool) -> Result<Vec<StoredTrial>, sqlx::Error> {
    let rows: Vec<TrialRow> = sqlx::query_as(
        "SELECT trial_id, corpus_id, corpus_version, seed, arms, cohorts, case_ids, assignment \
         FROM evaluation_trials ORDER BY created_at DESC",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(
            |(trial_id, corpus_id, corpus_version, seed, arms, cohorts, case_ids, assignment)| {
                StoredTrial {
                    trial_id,
                    corpus_id,
                    corpus_version,
                    seed,
                    arms: serde_json::from_value(arms).expect("the arms parse"),
                    cohorts: serde_json::from_value(cohorts).expect("the cohorts parse"),
                    case_ids: serde_json::from_value(case_ids).expect("the case ids parse"),
                    assignment: assignment
                        .as_object()
                        .expect("the assignment is an object")
                        .clone(),
                }
            },
        )
        .collect())
}

/// The recorded per-arm results for one trial, oldest first.
pub async fn list_trial_results(pool: &PgPool, trial_id: &str) -> Result<Vec<Value>, sqlx::Error> {
    let rows: Vec<Value> = sqlx::query_scalar(
        "SELECT results FROM evaluation_trial_results WHERE trial_id = $1 \
         ORDER BY recorded_at",
    )
    .bind(trial_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

// ── The calibration + the regression gates (`.4.4`, ADR-017) ──────────────────────

/// The calibration submission (`.4.4`): the accumulation over the NAMED runs
/// (each must exist — the record accumulates real measurements, never a
/// fabrication).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CalibrationSubmission {
    pub calibration_id: String,
    pub corpus_id: String,
    pub corpus_version: i64,
    pub workflow: String,
    pub run_ids: Vec<String>,
    pub brier: Option<f64>,
    pub confidence: Value,
}

/// The gate submission (`.4.4`): the recorded baseline (case → score) + the
/// tolerance threshold. The gate only BLOCKS — it never mutates a result.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GateSubmission {
    pub gate_id: String,
    pub corpus_id: String,
    pub corpus_version: i64,
    pub workflow: String,
    pub baseline: Value,
    pub threshold: f64,
}

impl EvaluationError {
    fn ghost_run(run_id: &str) -> Self {
        EvaluationError::Duplicate(format!(
            "run `{run_id}` (the calibration accumulates REGISTERED runs only)"
        ))
    }
    fn out_of_range(field: &str, value: f64) -> Self {
        EvaluationError::MalformedDigest(format!("{field} {value} is outside [0, 1]"))
    }
    fn ghost_gate(gate_id: &str) -> Self {
        EvaluationError::Duplicate(format!(
            "gate `{gate_id}` (the evaluation targets a REGISTERED gate)"
        ))
    }
}

/// Record one calibration (the accumulation over the named runs).
pub async fn record_calibration(
    pool: &PgPool,
    submission: &CalibrationSubmission,
) -> Result<Value, EvaluationError> {
    if submission.run_ids.is_empty() {
        return Err(EvaluationError::MalformedDigest(
            "the calibration names at least one run".to_string(),
        ));
    }
    if let Some(brier) = submission.brier {
        if !(0.0..=1.0).contains(&brier) {
            return Err(EvaluationError::out_of_range("brier", brier));
        }
    }
    for run_id in &submission.run_ids {
        let exists: Option<bool> =
            sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM evaluation_runs WHERE run_id = $1)")
                .bind(run_id)
                .fetch_one(pool)
                .await
                .map_err(|_| EvaluationError::ghost_run(run_id))?;
        if !exists.unwrap_or(false) {
            return Err(EvaluationError::ghost_run(run_id));
        }
    }
    let inserted = sqlx::query(
        "INSERT INTO evaluation_calibrations \
         (calibration_id, corpus_id, corpus_version, workflow, run_ids, brier, confidence) \
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(&submission.calibration_id)
    .bind(&submission.corpus_id)
    .bind(submission.corpus_version)
    .bind(&submission.workflow)
    .bind(serde_json::to_value(&submission.run_ids).expect("the run ids serialize"))
    .bind(submission.brier)
    .bind(&submission.confidence)
    .execute(pool)
    .await;
    match inserted {
        Ok(_) => Ok(json!({
            "calibration_id": submission.calibration_id,
            "corpus_id": submission.corpus_id,
            "corpus_version": submission.corpus_version,
            "workflow": submission.workflow,
            "run_ids": submission.run_ids,
            "brier": submission.brier,
            "confidence": submission.confidence,
        })),
        Err(_) => Err(EvaluationError::Duplicate(format!(
            "calibration `{}`",
            submission.calibration_id
        ))),
    }
}

/// Record the gate (the baseline + the threshold).
pub async fn record_gate(
    pool: &PgPool,
    submission: &GateSubmission,
) -> Result<Value, EvaluationError> {
    if !(0.0..=1.0).contains(&submission.threshold) {
        return Err(EvaluationError::out_of_range(
            "threshold",
            submission.threshold,
        ));
    }
    let baseline = submission.baseline.as_object().ok_or_else(|| {
        EvaluationError::MalformedDigest("the baseline is a case→score object".to_string())
    })?;
    if baseline.is_empty() {
        return Err(EvaluationError::MalformedDigest(
            "the baseline names at least one case".to_string(),
        ));
    }
    for (case_id, score) in baseline {
        let score = score.as_f64().ok_or_else(|| {
            EvaluationError::MalformedDigest(format!(
                "the baseline score for `{case_id}` is not a number"
            ))
        })?;
        if !(0.0..=1.0).contains(&score) {
            return Err(EvaluationError::out_of_range(
                &format!("the baseline score for `{case_id}`"),
                score,
            ));
        }
    }
    let inserted = sqlx::query(
        "INSERT INTO evaluation_gates \
         (gate_id, corpus_id, corpus_version, workflow, baseline, threshold) \
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(&submission.gate_id)
    .bind(&submission.corpus_id)
    .bind(submission.corpus_version)
    .bind(&submission.workflow)
    .bind(&submission.baseline)
    .bind(submission.threshold)
    .execute(pool)
    .await;
    match inserted {
        Ok(_) => Ok(json!({
            "gate_id": submission.gate_id,
            "corpus_id": submission.corpus_id,
            "corpus_version": submission.corpus_version,
            "workflow": submission.workflow,
            "baseline": submission.baseline,
            "threshold": submission.threshold,
        })),
        Err(_) => Err(EvaluationError::Duplicate(format!(
            "gate `{}`",
            submission.gate_id
        ))),
    }
}

/// The gate evaluation (`.4.4`): each measured case score is compared against
/// the baseline minus the threshold — a drop below it is the typed FAILURE.
/// The result APPENDS (the gate never rewrites a result).
pub async fn evaluate_gate(
    pool: &PgPool,
    gate_id: &str,
    scores: &Value,
) -> Result<Value, EvaluationError> {
    let row: Option<(Value, f64)> =
        sqlx::query_as("SELECT baseline, threshold FROM evaluation_gates WHERE gate_id = $1")
            .bind(gate_id)
            .fetch_optional(pool)
            .await
            .map_err(|_| EvaluationError::ghost_gate(gate_id))?;
    let Some((baseline, threshold)) = row else {
        return Err(EvaluationError::ghost_gate(gate_id));
    };
    let baseline = baseline.as_object().ok_or_else(|| {
        EvaluationError::MalformedDigest("the stored baseline is corrupt".to_string())
    })?;
    let scores = scores.as_object().ok_or_else(|| {
        EvaluationError::MalformedDigest("the scores are a case→score object".to_string())
    })?;
    let mut failures = Vec::new();
    for (case_id, expected) in baseline {
        let Some(expected) = expected.as_f64() else {
            continue;
        };
        let Some(measured) = scores.get(case_id).and_then(|v| v.as_f64()) else {
            continue; // an unmeasured case is not compared (the caller owns the coverage)
        };
        let floor = expected - threshold;
        if measured < floor {
            failures.push(json!({
                "case_id": case_id,
                "baseline": expected,
                "measured": measured,
                "delta": expected - measured,
            }));
        }
    }
    let passed = failures.is_empty();
    let appended = sqlx::query(
        "INSERT INTO evaluation_gate_results (gate_id, passed, failures) VALUES ($1, $2, $3)",
    )
    .bind(gate_id)
    .bind(passed)
    .bind(serde_json::to_value(&failures).expect("the failures serialize"))
    .execute(pool)
    .await;
    if appended.is_err() {
        return Err(EvaluationError::Duplicate(format!(
            "gate result for `{gate_id}`"
        )));
    }
    Ok(json!({ "gate_id": gate_id, "passed": passed, "failures": failures }))
}

/// The calibrations + the gates, newest first.
pub async fn list_calibrations(pool: &PgPool) -> Result<Vec<Value>, sqlx::Error> {
    let rows: Vec<Value> = sqlx::query_scalar(
        "SELECT jsonb_build_object('calibration_id', calibration_id, 'corpus_id', corpus_id, \
         'corpus_version', corpus_version, 'workflow', workflow, 'run_ids', run_ids, \
         'brier', brier, 'confidence', confidence) \
         FROM evaluation_calibrations ORDER BY created_at DESC",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// The gate rows, newest first.
pub async fn list_gates(pool: &PgPool) -> Result<Vec<Value>, sqlx::Error> {
    let rows: Vec<Value> = sqlx::query_scalar(
        "SELECT jsonb_build_object('gate_id', gate_id, 'corpus_id', corpus_id, \
         'corpus_version', corpus_version, 'workflow', workflow, 'baseline', baseline, \
         'threshold', threshold) \
         FROM evaluation_gates ORDER BY created_at DESC",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// The gate's evaluation results, oldest first.
pub async fn list_gate_results(pool: &PgPool, gate_id: &str) -> Result<Vec<Value>, sqlx::Error> {
    let rows: Vec<Value> = sqlx::query_scalar(
        "SELECT jsonb_build_object('gate_id', gate_id, 'passed', passed, 'failures', failures) \
         FROM evaluation_gate_results WHERE gate_id = $1 ORDER BY evaluated_at",
    )
    .bind(gate_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}
