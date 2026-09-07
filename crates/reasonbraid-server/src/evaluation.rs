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
use serde_json::Value;
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
