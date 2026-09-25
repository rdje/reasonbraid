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

/// A repeated waiver is at least this many waivers IN FORCE for one
/// publication (`SIGNOFF-REPAIR.9.3.2`). One waiver is an exception; a second is
/// the pattern §15.11 names. It used to fire on the first waiver, and on waivers
/// that had long lapsed.
pub const REPEATED_WAIVER_THRESHOLD: i64 = 2;

/// ... recorded within this many days of the evaluation (the window).
pub const REPEATED_WAIVER_WINDOW_DAYS: i32 = 90;

/// Evaluate the DUE reviews: for each (publication, trigger) the records name,
/// its LATEST occurrence — the outcomes naming a trigger in the vocabulary, the
/// drift occurrences, and the repeated waivers — and a new review when that
/// occurrence is newer than the pair's latest review and none is due.
///
/// ⭐ `SIGNOFF-REPAIR.9.3.2`: the lifecycle RECURS. A review had the id
/// `rev_{publication}_{trigger}`, the primary key, so once the pair's first
/// review was done every later one collided — and `inserted.is_ok()` discarded
/// the error, which also turned a failing store into an empty schedule. A review
/// now has its own id; at most one DUE review per pair is the partial unique
/// index `policy_reviews_one_due` (`migrations/0113`), which the insert names,
/// so a concurrent schedule is a skip; and any other insert error is returned.
///
/// Occurrence times and the waiver window read the DATABASE clock, the one the
/// rows were stamped with.
///
/// ⛔ `SIGNOFF-REPAIR.6.1.5.2.1`: SCOPED to the caller's tenant, not gated by a
/// refusal, because this verb names no id at all — one POST used to materialise
/// review rows for EVERY tenant's publications, so a stranger decided which of
/// your publications were under review.
pub async fn schedule_reviews(
    pool: &PgPool,
    tenant_id: &str,
) -> Result<Vec<StoredReview>, sqlx::Error> {
    type Occurrence = (String, String, chrono::DateTime<chrono::Utc>);
    let outcomes: Vec<Occurrence> = sqlx::query_as(
        "SELECT publication_id, review_trigger, max(created_at) FROM policy_outcomes \
         WHERE review_trigger IS NOT NULL AND tenant_id = $1 GROUP BY 1, 2",
    )
    .bind(tenant_id)
    .fetch_all(pool)
    .await?;
    let drift: Vec<Occurrence> = sqlx::query_as(
        "SELECT publication_id, 'drift', max(created_at) FROM policy_drift \
         WHERE tenant_id = $1 GROUP BY 1",
    )
    .bind(tenant_id)
    .fetch_all(pool)
    .await?;
    // A lapsed waiver is not in force, and an old one is outside the window.
    let waivers: Vec<Occurrence> = sqlx::query_as(
        "SELECT publication_id, 'repeated_waiver', max(created_at) FROM policy_corrections \
         WHERE operation = 'waiver' AND tenant_id = $1 \
           AND (expires_at IS NULL OR expires_at > now()) \
           AND created_at > now() - make_interval(days => $2) \
         GROUP BY 1 HAVING count(*) >= $3",
    )
    .bind(tenant_id)
    .bind(REPEATED_WAIVER_WINDOW_DAYS)
    .bind(REPEATED_WAIVER_THRESHOLD)
    .fetch_all(pool)
    .await?;

    // Each pair's latest occurrence: an outcome naming `drift` and a drift row
    // are occurrences of ONE pair.
    let mut latest: std::collections::BTreeMap<(String, String), chrono::DateTime<chrono::Utc>> =
        std::collections::BTreeMap::new();
    for (publication_id, trigger, at) in outcomes.into_iter().chain(drift).chain(waivers) {
        if !REVIEW_TRIGGERS.contains(&trigger.as_str()) {
            continue;
        }
        let entry = latest.entry((publication_id, trigger)).or_insert(at);
        if at > *entry {
            *entry = at;
        }
    }

    let mut scheduled = Vec::new();
    for ((publication_id, trigger), occurred_at) in latest {
        // Covered: the pair's latest review was scheduled after this occurrence.
        let reviewed_at: Option<chrono::DateTime<chrono::Utc>> = sqlx::query_scalar(
            "SELECT max(created_at) FROM policy_reviews WHERE publication_id = $1 AND trigger = $2",
        )
        .bind(&publication_id)
        .bind(&trigger)
        .fetch_one(pool)
        .await?;
        if reviewed_at.is_some_and(|reviewed_at| reviewed_at >= occurred_at) {
            continue;
        }
        // `SIGNOFF-REPAIR.6.1.5.2`: a review belongs to the PUBLICATION it
        // reviews, never to whoever posted the schedule verb.
        //
        // ⚠️ The lookup conflates two absences deliberately: a publication that
        // does not exist and one staged before `migrations/0073` both yield
        // NULL, and both are unattributable.
        let owner: Option<String> = sqlx::query_scalar(
            "SELECT tenant_id FROM policy_publications WHERE publication_id = $1",
        )
        .bind(&publication_id)
        .fetch_optional(pool)
        .await?
        .flatten();
        let review_id = crate::snapshots::evidence_id("rev");
        let inserted = sqlx::query(
            "INSERT INTO policy_reviews (review_id, publication_id, trigger, status, tenant_id) \
             VALUES ($1, $2, $3, 'due', $4) \
             ON CONFLICT (publication_id, trigger) WHERE status = 'due' DO NOTHING",
        )
        .bind(&review_id)
        .bind(&publication_id)
        .bind(&trigger)
        .bind(&owner)
        .execute(pool)
        .await?;
        // Zero rows: a review for the pair is already due — this call's
        // occurrence is that review's to cover.
        if inserted.rows_affected() == 1 {
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
pub async fn mark_done(
    pool: &PgPool,
    tenant_id: &str,
    review_id: &str,
) -> Result<StoredReview, ReviewError> {
    let row: Option<(String, Option<String>)> =
        sqlx::query_as("SELECT status, tenant_id FROM policy_reviews WHERE review_id = $1")
            .bind(review_id)
            .fetch_optional(pool)
            .await
            .map_err(|_| ReviewError::UnknownReview(review_id.to_string()))?;
    let Some((status, owner)) = row else {
        return Err(ReviewError::UnknownReview(review_id.to_string()));
    };
    // ⛔ `SIGNOFF-REPAIR.6.1.5.2.1`: a foreign review answers as an absent one,
    // and a review with no owner is closed by nobody.
    if owner.as_deref() != Some(tenant_id) {
        return Err(ReviewError::UnknownReview(review_id.to_string()));
    }
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
/// ⛔ `SIGNOFF-REPAIR.6.1.5.3`: bound to the caller's tenant. Until now this
/// returned every tenant's rows to any enrolled principal. ⚠️ The predicate
/// never matches NULL, so a row `migrations/0073` could not attribute is read
/// by NOBODY — `.7.1.2.2`'s disposition, and the reason the backfill's coverage
/// is published as a measured count rather than assumed complete.
pub async fn list_reviews(
    pool: &PgPool,
    tenant_id: &str,
) -> Result<Vec<StoredReview>, sqlx::Error> {
    let rows: Vec<(String, String, String, String)> = sqlx::query_as(
        "SELECT review_id, publication_id, trigger, status \
         FROM policy_reviews WHERE tenant_id = $1 ORDER BY created_at DESC",
    )
    .bind(tenant_id)
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
