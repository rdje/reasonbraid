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
    /// The site act's audit reason (`SIGNOFF-REPAIR.8.2.5.3`). ⚠️ A documented
    /// wire ADDITION: every `evaluation_*` write is a site act now, and a site
    /// act records why it was performed.
    pub reason: crate::site_authority::Reason,
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
    /// The site act's audit reason (`SIGNOFF-REPAIR.8.2.5.3`). ⚠️ A documented
    /// wire ADDITION: every `evaluation_*` write is a site act now, and a site
    /// act records why it was performed.
    pub reason: crate::site_authority::Reason,
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
///
/// ⛔ **`Debug` ONLY, and `#[non_exhaustive]`** (`SIGNOFF-REPAIR.8.2.2`, following
/// `.7.4.2`'s contract): [`EvaluationError::Storage`] carries an `sqlx::Error`,
/// which is neither `Clone` nor `Eq`, and preserving the store's own cause
/// matters more than deriving equality nothing in this crate used.
#[derive(Debug)]
#[non_exhaustive]
pub enum EvaluationError {
    /// The digest is not a 64-hex string (the harness's shape).
    MalformedDigest(String),
    /// A request the harness's own rules refuse; the message is the whole
    /// account (`SIGNOFF-REPAIR.8.2.6`). ⛔ These refusals used to ride
    /// `MalformedDigest`, whose message wraps its text as a digest sentence, so
    /// a caller whose arms were empty read *"digest `the arms are empty` is not
    /// a 64-hex string"*.
    Invalid(String),
    /// The (corpus, version) or the run id already exists — never an
    /// overwrite.
    Duplicate(String),
    /// The run references a corpus version that is not registered.
    UnknownCorpus(String),
    /// A non-deterministic run without a declared seed.
    UndeclaredSeed,
    /// The trial count is not positive.
    InvalidTrialCount(i64),
    /// A record this act NAMED is not registered.
    ///
    /// 🔴 **`SIGNOFF-REPAIR.8.2.5.3` found these modelled as
    /// [`EvaluationError::Duplicate`]** — `ghost_run`, `ghost_gate` and the
    /// absent-trial refusal all returned the *already exists* variant, so one
    /// enum arm meant both *this identity is taken* and *this identity does not
    /// exist*. ⛔ In prose that was merely confusing; under the site gate, which
    /// renders a refusal by its CLASS, it became a caller being told *that
    /// calibration id already exists* when the defect was a run that does not.
    NotRegistered(String),
    /// The store itself failed. ⛔ A database fault proves NOTHING about the
    /// caller's input and must never be reported as though it did
    /// (`SIGNOFF-REPAIR.8.2.2`). Every arm here used to collapse into
    /// [`EvaluationError::Duplicate`] or [`EvaluationError::UnknownCorpus`], so
    /// a connection loss reached the caller as *already exists* or *the corpus
    /// is not registered* — with an HTTP 400 blaming them for it.
    Storage(sqlx::Error),
}

/// Classify a failed write: the database's own **unique violation** is the
/// caller's duplicate; anything else is the store failing.
///
/// ⛔ The old code took `Err(_)` for granted as a duplicate, which is true of
/// exactly one `sqlx::Error` variant out of all of them.
fn write_failure(error: sqlx::Error, duplicate: impl FnOnce() -> String) -> EvaluationError {
    let is_duplicate = error
        .as_database_error()
        .is_some_and(|db| db.is_unique_violation());
    if is_duplicate {
        EvaluationError::Duplicate(duplicate())
    } else {
        EvaluationError::Storage(error)
    }
}

impl std::fmt::Display for EvaluationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvaluationError::MalformedDigest(d) => {
                write!(f, "digest `{d}` is not a 64-hex string")
            }
            EvaluationError::Invalid(message) => f.write_str(message),
            EvaluationError::Duplicate(what) => {
                write!(
                    f,
                    "{what} already exists — the record's identity is its content, \
                     register a new version or run id instead of overwriting"
                )
            }
            EvaluationError::NotRegistered(what) => {
                write!(f, "{what} is not registered")
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
            // ⛔ The wire message says nothing about the caller's input,
            // because the store's failure is not about it. The cause travels
            // through `source` for the server-side log.
            EvaluationError::Storage(_) => {
                write!(f, "the evaluation store is unavailable")
            }
        }
    }
}

impl std::error::Error for EvaluationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            EvaluationError::Storage(cause) => Some(cause),
            _ => None,
        }
    }
}

/// The 64-hex digest shape (the harness's sha256 hex over the file bytes).
fn is_hex64(digest: &str) -> bool {
    digest.len() == 64 && digest.bytes().all(|b| b.is_ascii_hexdigit())
}

/// The pure part of [`register_corpus`]'s contract, callable without a database.
///
/// ⛔ It exists so the gate can run AFTER validation (`SIGNOFF-REPAIR.8.2.5.2`).
/// A site act renders its refusal as an authorization reason, so a validation
/// failure wrapped inside one becomes a 403 about authority for a 400 about
/// input — and `site_authority::workflows` states the rule in its own words:
/// validate in the HTTP layer first, because a caller that fails validation
/// learns nothing about authority and one that passes it still meets the gate.
/// ⚠️ [`register_corpus`] calls this itself regardless, so the registry cannot
/// store an invalid row whatever called it.
pub fn validate_corpus(registration: &CorpusRegistration) -> Result<(), EvaluationError> {
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
    Ok(())
}

/// Register one corpus version (the content-addressed registry row).
pub async fn register_corpus(
    conn: &mut sqlx::PgConnection,
    registration: &CorpusRegistration,
) -> Result<RegisteredCorpus, EvaluationError> {
    validate_corpus(registration)?;
    let inserted = sqlx::query(
        "INSERT INTO evaluation_corpora \
         (corpus_id, version, cases_digest, prompts_digest, cases) \
         VALUES ($1, $2, $3, $4, $5) \
         ON CONFLICT (corpus_id, version) DO NOTHING",
    )
    .bind(&registration.corpus_id)
    .bind(registration.version)
    .bind(&registration.cases_digest)
    .bind(&registration.prompts_digest)
    .bind(&registration.cases)
    .execute(&mut *conn)
    .await;
    // ⭐ THE CONSTRAINT STILL ARBITRATES, THROUGH `rows_affected` RATHER THAN
    // THROUGH AN ERROR (`SIGNOFF-REPAIR.8.2.5.3`): zero rows is the coordinate
    // already being taken, and nothing else produces zero. ⚠️ `write_failure`'s
    // duplicate branch is therefore UNREACHABLE at this site now, and it is kept
    // rather than simplified away: it is what stops an `Err` being assumed to be
    // a duplicate again if the `ON CONFLICT` clause is ever removed, which is the
    // defect `SIGNOFF-REPAIR.8.2.2` repaired.
    match inserted {
        Ok(done) if done.rows_affected() == 0 => Err(EvaluationError::Duplicate({
            format!(
                "corpus `{}` version {}",
                registration.corpus_id, registration.version
            )
        })),
        Ok(_) => Ok(RegisteredCorpus {
            corpus_id: registration.corpus_id.clone(),
            version: registration.version,
            cases_digest: registration.cases_digest.clone(),
            prompts_digest: registration.prompts_digest.clone(),
            cases: registration.cases.clone(),
        }),
        Err(error) => Err(write_failure(error, || {
            format!(
                "corpus `{}` version {}",
                registration.corpus_id, registration.version
            )
        })),
    }
}

/// The pure part of [`record_run`]'s contract, callable without a database —
/// `SIGNOFF-REPAIR.8.2.5.2`, so the gate runs AFTER validation. The write
/// calls it itself regardless, so an invalid row cannot be stored.
pub fn validate_run(run: &RunRecord) -> Result<(), EvaluationError> {
    if run.trial_count < 1 {
        return Err(EvaluationError::InvalidTrialCount(run.trial_count));
    }
    if !run.deterministic && run.seed.is_none() {
        return Err(EvaluationError::UndeclaredSeed);
    }
    Ok(())
}

/// Record one experiment run (the seed-declaring record).
pub async fn record_run(
    conn: &mut sqlx::PgConnection,
    run: &RunRecord,
) -> Result<StoredRun, EvaluationError> {
    validate_run(run)?;
    let corpus_exists: Option<bool> = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM evaluation_corpora \
         WHERE corpus_id = $1 AND version = $2)",
    )
    .bind(&run.corpus_id)
    .bind(run.corpus_version)
    .fetch_one(&mut *conn)
    .await
    .map_err(EvaluationError::Storage)?;
    if !corpus_exists.unwrap_or(false) {
        return Err(EvaluationError::UnknownCorpus(run.corpus_id.clone()));
    }
    let inserted = sqlx::query(
        "INSERT INTO evaluation_runs \
         (run_id, workflow, corpus_id, corpus_version, seed, deterministic, trial_count, results) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8) \
         ON CONFLICT (run_id) DO NOTHING",
    )
    .bind(&run.run_id)
    .bind(&run.workflow)
    .bind(&run.corpus_id)
    .bind(run.corpus_version)
    .bind(run.seed)
    .bind(run.deterministic)
    .bind(run.trial_count)
    .bind(&run.results)
    .execute(&mut *conn)
    .await;
    // ⭐ THE CONSTRAINT STILL ARBITRATES, THROUGH `rows_affected` RATHER THAN
    // THROUGH AN ERROR (`SIGNOFF-REPAIR.8.2.5.3`): zero rows is the coordinate
    // already being taken, and nothing else produces zero. ⚠️ `write_failure`'s
    // duplicate branch is therefore UNREACHABLE at this site now, and it is kept
    // rather than simplified away: it is what stops an `Err` being assumed to be
    // a duplicate again if the `ON CONFLICT` clause is ever removed, which is the
    // defect `SIGNOFF-REPAIR.8.2.2` repaired.
    match inserted {
        Ok(done) if done.rows_affected() == 0 => {
            Err(EvaluationError::Duplicate(format!("run `{}`", run.run_id)))
        }
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
        Err(error) => Err(write_failure(error, || format!("run `{}`", run.run_id))),
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
    /// The site act's audit reason (`SIGNOFF-REPAIR.8.2.5.3`). ⚠️ A documented
    /// wire ADDITION: every `evaluation_*` write is a site act now, and a site
    /// act records why it was performed.
    pub reason: crate::site_authority::Reason,
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
        EvaluationError::Invalid("the arms are empty".to_string())
    }
    fn empty_cases() -> Self {
        EvaluationError::Invalid("the case ids are empty".to_string())
    }
    fn bad_cohort(label: &str) -> Self {
        EvaluationError::Invalid(format!(
            "cohort `{label}` has an unknown kind (expected `case` or `subject`)"
        ))
    }
    fn repeated_arm(arm: &str) -> Self {
        EvaluationError::Invalid(format!(
            "the arm `{arm}` is listed twice; each listing is one share of the seeded \
             assignment, so a repeat would bias it"
        ))
    }
    fn repeated_case(case_id: &str) -> Self {
        EvaluationError::Invalid(format!(
            "the case id `{case_id}` is listed twice; a case is assigned one arm"
        ))
    }
}

/// The first value listed twice, in submission order.
fn first_repeat(values: &[String]) -> Option<&str> {
    let mut seen = std::collections::HashSet::new();
    values
        .iter()
        .map(String::as_str)
        .find(|value| !seen.insert(*value))
}

/// Choose an arm index from a `u64` draw.
///
/// ⛔ **THE MODULO HAPPENS IN `u64`, AND THE NARROWING ONLY AFTERWARDS**
/// (`SIGNOFF-REPAIR.8.2.3`). The old expression was `(draw as usize) % arms`,
/// and on a 32-bit target `draw as usize` TRUNCATES to the low 32 bits — so the
/// arm chosen for one `(case_id, seed)` depended on the host, while the comment
/// on [`splitmix64`] promised the opposite as this function's whole purpose.
/// ⭐ After the modulo the value is strictly less than `arms`, which is already
/// a `usize`, so the cast here cannot lose anything on any target.
///
/// # Panics
/// Never called with `arms == 0`: `create_trial` refuses an empty arm list
/// before it reaches the draw.
fn arm_index(draw: u64, arms: usize) -> usize {
    (draw % arms as u64) as usize
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

/// The pure part of [`create_trial`]'s contract, callable without a database —
/// `SIGNOFF-REPAIR.8.2.5.2`, so the gate runs AFTER validation. The write
/// calls it itself regardless, so an invalid row cannot be stored.
pub fn validate_trial(submission: &TrialSubmission) -> Result<(), EvaluationError> {
    if submission.arms.is_empty() {
        return Err(EvaluationError::empty_arms());
    }
    if submission.case_ids.is_empty() {
        return Err(EvaluationError::empty_cases());
    }
    // ⛔ `SIGNOFF-REPAIR.8.2.6`: the draw indexes into `arms`, so a repeated arm
    // took a double share of the cases, and a repeated case id was stored twice
    // while the assignment map kept one entry. Both were accepted silently.
    if let Some(arm) = first_repeat(&submission.arms) {
        return Err(EvaluationError::repeated_arm(arm));
    }
    if let Some(case_id) = first_repeat(&submission.case_ids) {
        return Err(EvaluationError::repeated_case(case_id));
    }
    for cohort in &submission.cohorts {
        if cohort.kind != "case" && cohort.kind != "subject" {
            return Err(EvaluationError::bad_cohort(&cohort.label));
        }
    }
    Ok(())
}

/// Create the shadow trial: the seeded assignment is SERVER-computed (the
/// record alone reproduces it — the client never supplies a draw).
pub async fn create_trial(
    conn: &mut sqlx::PgConnection,
    submission: &TrialSubmission,
) -> Result<StoredTrial, EvaluationError> {
    validate_trial(submission)?;
    let corpus_exists: Option<bool> = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM evaluation_corpora \
         WHERE corpus_id = $1 AND version = $2)",
    )
    .bind(&submission.corpus_id)
    .bind(submission.corpus_version)
    .fetch_one(&mut *conn)
    .await
    .map_err(EvaluationError::Storage)?;
    if !corpus_exists.unwrap_or(false) {
        return Err(EvaluationError::UnknownCorpus(submission.corpus_id.clone()));
    }

    // The seeded draw: splitmix64 over the (case id, seed) — stable across
    // runs and platforms (the std hasher is not), and stable across POINTER
    // WIDTHS because `arm_index` takes the modulo in u64 before narrowing
    // (`SIGNOFF-REPAIR.8.2.3`; the old `draw as usize` did not).
    let mut assignment = serde_json::Map::new();
    for case_id in &submission.case_ids {
        let draw = splitmix64(submission.seed as u64, case_id.as_bytes());
        let arm = &submission.arms[arm_index(draw, submission.arms.len())];
        assignment.insert(case_id.clone(), serde_json::Value::String(arm.clone()));
    }

    let inserted = sqlx::query(
        "INSERT INTO evaluation_trials \
         (trial_id, corpus_id, corpus_version, seed, arms, cohorts, case_ids, assignment) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8) \
         ON CONFLICT (trial_id) DO NOTHING",
    )
    .bind(&submission.trial_id)
    .bind(&submission.corpus_id)
    .bind(submission.corpus_version)
    .bind(submission.seed)
    .bind(serde_json::to_value(&submission.arms).expect("the arms serialize"))
    .bind(serde_json::to_value(&submission.cohorts).expect("the cohorts serialize"))
    .bind(serde_json::to_value(&submission.case_ids).expect("the case ids serialize"))
    .bind(serde_json::to_value(&assignment).expect("the assignment serializes"))
    .execute(&mut *conn)
    .await;
    // ⭐ THE CONSTRAINT STILL ARBITRATES, THROUGH `rows_affected` RATHER THAN
    // THROUGH AN ERROR (`SIGNOFF-REPAIR.8.2.5.3`): zero rows is the coordinate
    // already being taken, and nothing else produces zero. ⚠️ `write_failure`'s
    // duplicate branch is therefore UNREACHABLE at this site now, and it is kept
    // rather than simplified away: it is what stops an `Err` being assumed to be
    // a duplicate again if the `ON CONFLICT` clause is ever removed, which is the
    // defect `SIGNOFF-REPAIR.8.2.2` repaired.
    match inserted {
        Ok(done) if done.rows_affected() == 0 => Err(EvaluationError::Duplicate({
            format!("trial `{}`", submission.trial_id)
        })),
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
        Err(error) => Err(write_failure(error, || {
            format!("trial `{}`", submission.trial_id)
        })),
    }
}

/// Append one per-arm results row (append-only — never an overwrite).
pub async fn record_trial_results(
    conn: &mut sqlx::PgConnection,
    trial_id: &str,
    results: &Value,
) -> Result<(), EvaluationError> {
    let trial_exists: Option<bool> =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM evaluation_trials WHERE trial_id = $1)")
            .bind(trial_id)
            .fetch_one(&mut *conn)
            .await
            .map_err(EvaluationError::Storage)?;
    if !trial_exists.unwrap_or(false) {
        return Err(EvaluationError::NotRegistered(format!(
            "trial `{trial_id}` (the results append to a REGISTERED trial)"
        )));
    }
    sqlx::query("INSERT INTO evaluation_trial_results (trial_id, results) VALUES ($1, $2)")
        .bind(trial_id)
        .bind(results)
        .execute(&mut *conn)
        .await
        .map_err(|error| write_failure(error, || "trial result".to_string()))?;
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
    /// The site act's audit reason (`SIGNOFF-REPAIR.8.2.5.3`). ⚠️ A documented
    /// wire ADDITION: every `evaluation_*` write is a site act now, and a site
    /// act records why it was performed.
    pub reason: crate::site_authority::Reason,
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
    /// The site act's audit reason (`SIGNOFF-REPAIR.8.2.5.3`). ⚠️ A documented
    /// wire ADDITION: every `evaluation_*` write is a site act now, and a site
    /// act records why it was performed.
    pub reason: crate::site_authority::Reason,
}

impl EvaluationError {
    /// A run that EXISTS but was taken against another corpus version or
    /// workflow (`SIGNOFF-REPAIR.8.2.4`). ⛔ Named separately from
    /// [`EvaluationError::ghost_run`] because the two send the caller to
    /// different places: one run id is wrong, the other is real and in the
    /// wrong calibration.
    fn ineligible_run(run_id: &str, run_scope: &str, calibration_scope: &str) -> Self {
        EvaluationError::UnknownCorpus(format!(
            "run `{run_id}` was taken against `{run_scope}` and this calibration is \
             `{calibration_scope}` — a calibration accumulates ELIGIBLE runs, not \
             merely registered ones"
        ))
    }
    fn ghost_run(run_id: &str) -> Self {
        EvaluationError::NotRegistered(format!(
            "run `{run_id}` (the calibration accumulates REGISTERED runs only)"
        ))
    }
    fn out_of_range(field: &str, value: f64) -> Self {
        EvaluationError::Invalid(format!("{field} {value} is outside [0, 1]"))
    }
    fn ghost_gate(gate_id: &str) -> Self {
        EvaluationError::NotRegistered(format!(
            "gate `{gate_id}` (the evaluation targets a REGISTERED gate)"
        ))
    }
}

/// The pure part of [`record_calibration`]'s contract, callable without a database —
/// `SIGNOFF-REPAIR.8.2.5.2`, so the gate runs AFTER validation. The write
/// calls it itself regardless, so an invalid row cannot be stored.
pub fn validate_calibration(submission: &CalibrationSubmission) -> Result<(), EvaluationError> {
    if submission.run_ids.is_empty() {
        return Err(EvaluationError::Invalid(
            "the calibration names at least one run".to_string(),
        ));
    }
    if let Some(brier) = submission.brier {
        if !(0.0..=1.0).contains(&brier) {
            return Err(EvaluationError::out_of_range("brier", brier));
        }
    }
    Ok(())
}

/// Record one calibration (the accumulation over the named runs).
pub async fn record_calibration(
    conn: &mut sqlx::PgConnection,
    submission: &CalibrationSubmission,
) -> Result<Value, EvaluationError> {
    validate_calibration(submission)?;
    // ⛔ ELIGIBLE, NOT MERELY PRESENT (`SIGNOFF-REPAIR.8.2.4`). This loop used
    // to ask only whether the run row existed, so a calibration could
    // accumulate runs taken against a DIFFERENT corpus version or a different
    // workflow — while its own row asserts all three. `.8.2`'s goal line says
    // *derive calibration from eligible runs*, and existence is not eligibility.
    // ⭐ The query returns the run's own (corpus, version, workflow) rather than
    // a boolean, so the refusal can NAME what disagreed instead of saying no.
    for run_id in &submission.run_ids {
        let row: Option<(String, i64, String)> = sqlx::query_as(
            "SELECT corpus_id, corpus_version, workflow FROM evaluation_runs WHERE run_id = $1",
        )
        .bind(run_id)
        .fetch_optional(&mut *conn)
        .await
        .map_err(EvaluationError::Storage)?;
        let Some((corpus_id, corpus_version, workflow)) = row else {
            return Err(EvaluationError::ghost_run(run_id));
        };
        if corpus_id != submission.corpus_id
            || corpus_version != submission.corpus_version
            || workflow != submission.workflow
        {
            return Err(EvaluationError::ineligible_run(
                run_id,
                &format!("{corpus_id}@{corpus_version}/{workflow}"),
                &format!(
                    "{}@{}/{}",
                    submission.corpus_id, submission.corpus_version, submission.workflow
                ),
            ));
        }
    }
    let inserted = sqlx::query(
        "INSERT INTO evaluation_calibrations \
         (calibration_id, corpus_id, corpus_version, workflow, run_ids, brier, confidence) \
         VALUES ($1, $2, $3, $4, $5, $6, $7) \
         ON CONFLICT (calibration_id) DO NOTHING",
    )
    .bind(&submission.calibration_id)
    .bind(&submission.corpus_id)
    .bind(submission.corpus_version)
    .bind(&submission.workflow)
    .bind(serde_json::to_value(&submission.run_ids).expect("the run ids serialize"))
    .bind(submission.brier)
    .bind(&submission.confidence)
    .execute(&mut *conn)
    .await;
    // ⭐ THE CONSTRAINT STILL ARBITRATES, THROUGH `rows_affected` RATHER THAN
    // THROUGH AN ERROR (`SIGNOFF-REPAIR.8.2.5.3`): zero rows is the coordinate
    // already being taken, and nothing else produces zero. ⚠️ `write_failure`'s
    // duplicate branch is therefore UNREACHABLE at this site now, and it is kept
    // rather than simplified away: it is what stops an `Err` being assumed to be
    // a duplicate again if the `ON CONFLICT` clause is ever removed, which is the
    // defect `SIGNOFF-REPAIR.8.2.2` repaired.
    match inserted {
        Ok(done) if done.rows_affected() == 0 => Err(EvaluationError::Duplicate({
            format!("calibration `{}`", submission.calibration_id)
        })),
        Ok(_) => Ok(json!({
            "calibration_id": submission.calibration_id,
            "corpus_id": submission.corpus_id,
            "corpus_version": submission.corpus_version,
            "workflow": submission.workflow,
            "run_ids": submission.run_ids,
            "brier": submission.brier,
            "confidence": submission.confidence,
        })),
        Err(error) => Err(write_failure(error, || {
            format!("calibration `{}`", submission.calibration_id)
        })),
    }
}

/// The pure part of [`record_gate`]'s contract, callable without a database —
/// `SIGNOFF-REPAIR.8.2.5.2`, so the gate runs AFTER validation. The write
/// calls it itself regardless, so an invalid row cannot be stored.
pub fn validate_gate(submission: &GateSubmission) -> Result<(), EvaluationError> {
    if !(0.0..=1.0).contains(&submission.threshold) {
        return Err(EvaluationError::out_of_range(
            "threshold",
            submission.threshold,
        ));
    }
    let baseline = submission.baseline.as_object().ok_or_else(|| {
        EvaluationError::Invalid("the baseline is a case→score object".to_string())
    })?;
    if baseline.is_empty() {
        return Err(EvaluationError::Invalid(
            "the baseline names at least one case".to_string(),
        ));
    }
    for (case_id, score) in baseline {
        let score = score.as_f64().ok_or_else(|| {
            EvaluationError::Invalid(format!(
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
    Ok(())
}

/// Record the gate (the baseline + the threshold).
pub async fn record_gate(
    conn: &mut sqlx::PgConnection,
    submission: &GateSubmission,
) -> Result<Value, EvaluationError> {
    validate_gate(submission)?;
    // ⛔ THE GATE NAMES A CORPUS, SO IT BINDS TO ONE (`SIGNOFF-REPAIR.8.2.4`).
    // `record_run` and `create_trial` have asked this question since they were
    // written; `record_gate` carried `corpus_id` and `corpus_version` and
    // validated neither, so a gate could be registered against a corpus version
    // that does not exist and its row would assert a provenance nothing held.
    let corpus_exists: Option<bool> = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM evaluation_corpora \
         WHERE corpus_id = $1 AND version = $2)",
    )
    .bind(&submission.corpus_id)
    .bind(submission.corpus_version)
    .fetch_one(&mut *conn)
    .await
    .map_err(EvaluationError::Storage)?;
    if !corpus_exists.unwrap_or(false) {
        return Err(EvaluationError::UnknownCorpus(submission.corpus_id.clone()));
    }
    let inserted = sqlx::query(
        "INSERT INTO evaluation_gates \
         (gate_id, corpus_id, corpus_version, workflow, baseline, threshold) \
         VALUES ($1, $2, $3, $4, $5, $6) \
         ON CONFLICT (gate_id) DO NOTHING",
    )
    .bind(&submission.gate_id)
    .bind(&submission.corpus_id)
    .bind(submission.corpus_version)
    .bind(&submission.workflow)
    .bind(&submission.baseline)
    .bind(submission.threshold)
    .execute(&mut *conn)
    .await;
    // ⭐ THE CONSTRAINT STILL ARBITRATES, THROUGH `rows_affected` RATHER THAN
    // THROUGH AN ERROR (`SIGNOFF-REPAIR.8.2.5.3`): zero rows is the coordinate
    // already being taken, and nothing else produces zero. ⚠️ `write_failure`'s
    // duplicate branch is therefore UNREACHABLE at this site now, and it is kept
    // rather than simplified away: it is what stops an `Err` being assumed to be
    // a duplicate again if the `ON CONFLICT` clause is ever removed, which is the
    // defect `SIGNOFF-REPAIR.8.2.2` repaired.
    match inserted {
        Ok(done) if done.rows_affected() == 0 => Err(EvaluationError::Duplicate({
            format!("gate `{}`", submission.gate_id)
        })),
        Ok(_) => Ok(json!({
            "gate_id": submission.gate_id,
            "corpus_id": submission.corpus_id,
            "corpus_version": submission.corpus_version,
            "workflow": submission.workflow,
            "baseline": submission.baseline,
            "threshold": submission.threshold,
        })),
        Err(error) => Err(write_failure(error, || {
            format!("gate `{}`", submission.gate_id)
        })),
    }
}

/// The pure part of [`evaluate_gate`]'s contract, callable without a database —
/// `SIGNOFF-REPAIR.8.2.5.2`, so the gate runs AFTER validation. `evaluate_gate`
/// calls it itself regardless.
///
/// ⚠️ An EMPTY score object passes here and is refused later, inside the write:
/// *compared no case* is a fact about the BASELINE's intersection with these
/// scores, so it is not decidable without the stored row.
pub fn validate_gate_scores(scores: &Value) -> Result<(), EvaluationError> {
    let scores = scores.as_object().ok_or_else(|| {
        EvaluationError::Invalid("the scores are a case→score object".to_string())
    })?;
    // ⛔ A MEASUREMENT IS HELD TO THE SAME SHAPE AS A BASELINE SCORE
    // (`SIGNOFF-REPAIR.8.2.1`). `record_gate`, twenty lines up in this file,
    // refuses a non-numeric baseline score and one outside [0, 1] by name.
    // This side used to SKIP a non-numeric measurement — so a case could be
    // dropped from the comparison by sending a string — and to compare an
    // out-of-range one as written, so `5.0` cleared every threshold. The write
    // side's rule is the read side's rule.
    for (case_id, measured) in scores.iter() {
        let value = measured.as_f64().ok_or_else(|| {
            EvaluationError::Invalid(format!(
                "the measured score for `{case_id}` is not a number"
            ))
        })?;
        if !(0.0..=1.0).contains(&value) {
            return Err(EvaluationError::out_of_range(
                &format!("the measured score for `{case_id}`"),
                value,
            ));
        }
    }
    Ok(())
}

/// The gate evaluation (`.4.4`): each measured case score is compared against
/// the baseline minus the threshold — a drop below it is the typed FAILURE.
/// The result APPENDS (the gate never rewrites a result).
pub async fn evaluate_gate(
    conn: &mut sqlx::PgConnection,
    gate_id: &str,
    scores: &Value,
) -> Result<Value, EvaluationError> {
    validate_gate_scores(scores)?;
    let row: Option<(Value, f64)> =
        sqlx::query_as("SELECT baseline, threshold FROM evaluation_gates WHERE gate_id = $1")
            .bind(gate_id)
            .fetch_optional(&mut *conn)
            .await
            .map_err(EvaluationError::Storage)?;
    let Some((baseline, threshold)) = row else {
        return Err(EvaluationError::ghost_gate(gate_id));
    };
    let baseline = baseline
        .as_object()
        .ok_or_else(|| EvaluationError::Invalid("the stored baseline is corrupt".to_string()))?;
    let scores = scores.as_object().ok_or_else(|| {
        EvaluationError::Invalid("the scores are a case→score object".to_string())
    })?;
    let mut failures = Vec::new();
    let mut compared = 0_u64;
    for (case_id, expected) in baseline {
        let Some(expected) = expected.as_f64() else {
            continue;
        };
        let Some(measured) = scores.get(case_id).and_then(|v| v.as_f64()) else {
            continue; // an unmeasured case is not compared (the caller owns the coverage)
        };
        compared += 1;
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
    // ⛔ ZERO COMPARISONS IS NOT A VERDICT, and the old code made it the BEST
    // one: with no case compared, `failures` is empty and the gate reported
    // `passed: true` — then APPENDED that pass to `evaluation_gate_results`,
    // so a row claiming success over nothing became the durable record.
    // ⚠️ A PARTIAL evaluation is still legal: the loop above deliberately keeps
    // skipping an unmeasured case, because `the caller owns the coverage` is a
    // declared contract rather than an oversight. Only the empty intersection
    // is refused, and the counts below are what make a partial result readable.
    if compared == 0 {
        return Err(EvaluationError::Invalid(format!(
            "the evaluation of gate `{gate_id}` compared no case —              the scores name at least one case the baseline carries"
        )));
    }
    let unmeasured = baseline.len() as u64 - compared;
    let passed = failures.is_empty();
    let appended = sqlx::query(
        "INSERT INTO evaluation_gate_results (gate_id, passed, failures) VALUES ($1, $2, $3)",
    )
    .bind(gate_id)
    .bind(passed)
    .bind(serde_json::to_value(&failures).expect("the failures serialize"))
    .execute(&mut *conn)
    .await;
    if let Err(error) = appended {
        return Err(write_failure(error, || {
            format!("gate result for `{gate_id}`")
        }));
    }
    Ok(json!({
        "gate_id": gate_id,
        "passed": passed,
        "failures": failures,
        "compared": compared,
        "unmeasured": unmeasured,
    }))
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

#[cfg(test)]
mod tests {
    use super::*;

    fn trial(arms: &[&str], case_ids: &[&str]) -> TrialSubmission {
        TrialSubmission {
            trial_id: "tri".to_owned(),
            corpus_id: "corpus".to_owned(),
            corpus_version: 1,
            seed: 7,
            arms: arms.iter().map(|arm| (*arm).to_owned()).collect(),
            cohorts: Vec::new(),
            case_ids: case_ids.iter().map(|case| (*case).to_owned()).collect(),
            reason: crate::site_authority::Reason::new("a unit control").expect("a reason"),
        }
    }

    /// `SIGNOFF-REPAIR.8.2.6`: each arm is one share of the seeded assignment,
    /// so `["a", "a", "b"]` gave `a` two thirds of the cases, silently.
    #[test]
    fn a_repeated_arm_is_refused_by_name() {
        let refused = validate_trial(&trial(&["a", "b", "a"], &["c1"])).unwrap_err();
        assert_eq!(
            refused.to_string(),
            "the arm `a` is listed twice; each listing is one share of the seeded \
             assignment, so a repeat would bias it"
        );
    }

    /// A repeated case id was stored twice while the assignment kept one entry.
    #[test]
    fn a_repeated_case_id_is_refused_by_name() {
        let refused = validate_trial(&trial(&["a", "b"], &["c1", "c2", "c1"])).unwrap_err();
        assert_eq!(
            refused.to_string(),
            "the case id `c1` is listed twice; a case is assigned one arm"
        );
    }

    /// Both cohort kinds are valid and anything else is refused by label. Only
    /// the live suite covered this, so a library-scoped mutation run could not
    /// see it (three mutants MISSED at `SIGNOFF-REPAIR.8.2.6`).
    #[test]
    fn a_cohort_is_a_case_or_a_subject_cohort() {
        for kind in ["case", "subject"] {
            let mut valid = trial(&["a"], &["c1"]);
            valid.cohorts.push(CohortRecord {
                label: format!("{kind} cohort"),
                kind: kind.to_owned(),
                members: Vec::new(),
            });
            validate_trial(&valid).expect("a known cohort kind");
        }
        let mut unknown = trial(&["a"], &["c1"]);
        unknown.cohorts.push(CohortRecord {
            label: "x".to_owned(),
            kind: "nope".to_owned(),
            members: Vec::new(),
        });
        let refused = validate_trial(&unknown).unwrap_err();
        assert!(refused.to_string().contains("cohort `x`"), "{refused}");
    }

    /// A validation refusal says what is wrong and nothing else. They used to be
    /// wrapped as a digest sentence (`SIGNOFF-REPAIR.8.2.6`), so the caller read
    /// *"digest `the arms are empty` is not a 64-hex string"*.
    #[test]
    fn a_validation_refusal_is_not_worded_as_a_digest() {
        assert_eq!(
            validate_trial(&trial(&[], &["c1"]))
                .unwrap_err()
                .to_string(),
            "the arms are empty"
        );
        assert_eq!(
            validate_gate_scores(&serde_json::json!(["not", "an", "object"]))
                .unwrap_err()
                .to_string(),
            "the scores are a case→score object"
        );
        assert_eq!(
            validate_gate_scores(&serde_json::json!({ "c1": "oops" }))
                .unwrap_err()
                .to_string(),
            "the measured score for `c1` is not a number"
        );
        assert_eq!(
            validate_gate_scores(&serde_json::json!({ "c1": 5.0 }))
                .unwrap_err()
                .to_string(),
            "the measured score for `c1` 5 is outside [0, 1]"
        );
        validate_gate_scores(&serde_json::json!({ "c1": 0.0, "c2": 1.0 }))
            .expect("scores inside [0, 1], bounds included, are valid");
    }

    /// The positive control: distinct arms and cases, even sharing a spelling
    /// across the two lists, are a valid trial.
    #[test]
    fn distinct_arms_and_cases_are_valid() {
        validate_trial(&trial(&["a", "b"], &["a", "b", "c"])).expect("a valid trial");
    }

    /// ⛔ A DRAW ABOVE `u32::MAX`, chosen so the truncated and untruncated
    /// answers DISAGREE — otherwise the control passes under the defect.
    /// `0x1_0000_0003 % 5 == 4`, while its low 32 bits are `3` and `3 % 5 == 3`.
    const WIDE_DRAW: u64 = 0x1_0000_0003;

    #[test]
    fn the_arm_index_is_taken_in_u64_space() {
        assert_eq!(
            arm_index(WIDE_DRAW, 5),
            4,
            "the modulo is over the whole u64"
        );
        assert_ne!(
            arm_index(WIDE_DRAW, 5),
            ((WIDE_DRAW as u32) as usize) % 5,
            "a 32-bit narrowing must give a DIFFERENT answer, or this control              cannot detect the defect it exists for"
        );
    }

    #[test]
    fn the_index_is_always_inside_the_arm_list() {
        for arms in 1..=8_usize {
            for draw in [0, 1, u32::MAX as u64, WIDE_DRAW, u64::MAX] {
                assert!(
                    arm_index(draw, arms) < arms,
                    "draw {draw} over {arms} arms escaped the list"
                );
            }
        }
    }

    #[test]
    fn the_draw_is_stable_for_one_case_and_seed() {
        // The reproducibility claim the assignment record rests on: the same
        // (seed, case id) yields the same draw, and a different seed does not.
        assert_eq!(splitmix64(7, b"case-a"), splitmix64(7, b"case-a"));
        assert_ne!(splitmix64(7, b"case-a"), splitmix64(8, b"case-a"));
        assert_ne!(splitmix64(7, b"case-a"), splitmix64(7, b"case-b"));
    }
}
