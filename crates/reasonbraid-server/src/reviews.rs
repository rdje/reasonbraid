//! The scheduled reviews (`PHASE-6.6`, §15.11): the seven review-trigger
//! vocabulary, the DUE evaluation over the `.5.3` records (the outcomes'
//! named triggers + the drift occurrences + the repeated waivers), the
//! dedupe (one due review per (publication, trigger)), and the done
//! transition.

use serde::Serialize;
use sqlx::PgPool;

/// The §15.11 review-trigger vocabulary.
pub const REVIEW_TRIGGERS: [&str; 7] = [
    "elapsed_interval",
    "dependency_change",
    "adverse_threshold",
    "external_standard_change",
    "repeated_waiver",
    "drift",
    "evaluator_regression",
];

/// The stored review row.
#[derive(Debug, Clone, Serialize)]
pub struct StoredReview {
    pub review_id: String,
    pub publication_id: String,
    pub trigger: String,
    pub status: String,
}

/// The typed refusal reasons.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReviewError {
    UnknownReview(String),
    UnknownTrigger(String),
    WrongStatus { review_id: String, status: String },
    Duplicate(String),
}

impl std::fmt::Display for ReviewError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReviewError::UnknownReview(r) => write!(f, "review `{r}` does not exist"),
            ReviewError::UnknownTrigger(t) => {
                write!(
                    f,
                    "trigger `{t}` is not in the vocabulary ({})",
                    REVIEW_TRIGGERS.join(", ")
                )
            }
            ReviewError::WrongStatus { review_id, status } => {
                write!(f, "review `{review_id}` is at status `{status}` — the done transition rides a due review")
            }
            ReviewError::Duplicate(what) => {
                write!(
                    f,
                    "{what} already exists — the record's identity is its content"
                )
            }
        }
    }
}

/// Evaluate the DUE reviews: the outcomes' named triggers (where the
/// trigger is in the vocabulary) + the drift occurrences + the repeated
/// waivers — one due review per (publication, trigger), the dedupe.
pub async fn schedule_reviews(pool: &PgPool) -> Result<Vec<StoredReview>, sqlx::Error> {
    let outcome_rows: Vec<(String, String)> = sqlx::query_as(
        "SELECT publication_id, review_trigger FROM policy_outcomes \
         WHERE review_trigger IS NOT NULL",
    )
    .fetch_all(pool)
    .await?;
    let drift_rows: Vec<String> = sqlx::query_scalar("SELECT publication_id FROM policy_drift")
        .fetch_all(pool)
        .await?;
    let waiver_rows: Vec<String> = sqlx::query_scalar(
        "SELECT publication_id FROM policy_corrections WHERE operation = 'waiver'",
    )
    .fetch_all(pool)
    .await?;

    // The (publication, trigger) pairs, deduped + the existing due rows
    // excluded (the schedule is idempotent).
    let mut pairs: Vec<(String, String)> = outcome_rows
        .into_iter()
        .filter(|(_, trigger)| REVIEW_TRIGGERS.contains(&trigger.as_str()))
        .collect();
    for publication_id in drift_rows {
        pairs.push((publication_id, "drift".to_string()));
    }
    for publication_id in waiver_rows {
        pairs.push((publication_id, "repeated_waiver".to_string()));
    }
    pairs.sort();
    pairs.dedup();

    let mut scheduled = Vec::new();
    for (publication_id, trigger) in pairs {
        let already: Option<bool> = sqlx::query_scalar(
            "SELECT EXISTS (SELECT 1 FROM policy_reviews \
             WHERE publication_id = $1 AND trigger = $2 AND status = 'due')",
        )
        .bind(&publication_id)
        .bind(&trigger)
        .fetch_one(pool)
        .await?;
        if already.unwrap_or(false) {
            continue;
        }
        let review_id = format!("rev_{}_{}", publication_id, trigger.replace('_', "-"));
        // `SIGNOFF-REPAIR.6.1.5.2`: a review belongs to the PUBLICATION it
        // reviews, never to whoever posted the schedule verb. This function
        // reads outcomes, drift and corrections with no predicate at all, so a
        // single caller materialises rows for every tenant's publications —
        // stamping that caller's tenant on all of them would attribute each
        // tenant's review trail to one stranger.
        //
        // ⚠️ The lookup conflates two absences deliberately: a publication that
        // does not exist and one staged before `migrations/0073` both yield
        // NULL, and both are unattributable. ⛔ It does not REFUSE the first —
        // that would change which review rows exist, which is `.6.1.5.3`'s
        // question about this same function, not this leaf's.
        let tenant_id: Option<String> = sqlx::query_scalar(
            "SELECT tenant_id FROM policy_publications WHERE publication_id = $1",
        )
        .bind(&publication_id)
        .fetch_optional(pool)
        .await?
        .flatten();
        let inserted = sqlx::query(
            "INSERT INTO policy_reviews (review_id, publication_id, trigger, status, tenant_id) \
             VALUES ($1, $2, $3, 'due', $4)",
        )
        .bind(&review_id)
        .bind(&publication_id)
        .bind(&trigger)
        .bind(&tenant_id)
        .execute(pool)
        .await;
        if inserted.is_ok() {
            scheduled.push(StoredReview {
                review_id,
                publication_id,
                trigger,
                status: "due".to_string(),
            });
        }
    }
    Ok(scheduled)
}

/// Mark one review DONE (the due → done transition).
pub async fn mark_done(pool: &PgPool, review_id: &str) -> Result<StoredReview, ReviewError> {
    let row: Option<String> =
        sqlx::query_scalar("SELECT status FROM policy_reviews WHERE review_id = $1")
            .bind(review_id)
            .fetch_optional(pool)
            .await
            .map_err(|_| ReviewError::UnknownReview(review_id.to_string()))?;
    let Some(status) = row else {
        return Err(ReviewError::UnknownReview(review_id.to_string()));
    };
    if status != "due" {
        return Err(ReviewError::WrongStatus {
            review_id: review_id.to_string(),
            status,
        });
    }
    sqlx::query("UPDATE policy_reviews SET status = 'done', done_at = now() WHERE review_id = $1")
        .bind(review_id)
        .execute(pool)
        .await
        .map_err(|_| ReviewError::UnknownReview(review_id.to_string()))?;
    let row: Option<(String, String, String, String)> = sqlx::query_as(
        "SELECT review_id, publication_id, trigger, status FROM policy_reviews WHERE review_id = $1",
    )
    .bind(review_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| ReviewError::UnknownReview(review_id.to_string()))?;
    let (review_id, publication_id, trigger, status) =
        row.expect("the row exists after the update");
    Ok(StoredReview {
        review_id,
        publication_id,
        trigger,
        status,
    })
}

/// The reviews, newest first.
pub async fn list_reviews(pool: &PgPool) -> Result<Vec<StoredReview>, sqlx::Error> {
    let rows: Vec<(String, String, String, String)> = sqlx::query_as(
        "SELECT review_id, publication_id, trigger, status \
         FROM policy_reviews ORDER BY created_at DESC",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(
            |(review_id, publication_id, trigger, status)| StoredReview {
                review_id,
                publication_id,
                trigger,
                status,
            },
        )
        .collect())
}
