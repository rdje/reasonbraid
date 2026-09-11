//! The claim-evidence graph + the citation validation (PHASE-4.6.3, ROADMAP
//! §12.7): claims link to evidence with ONE of the five assessments, and
//! the citation is VALIDATED — the excerpt must actually appear in the
//! snapshot's raw bytes (the claim must point at a REAL snapshot; citation
//! existence alone never satisfies an evidence gate).

use serde::{Deserialize, Serialize};
use sqlx::PgPool;

/// The five assessment kinds (§12.7).
pub const ASSESSMENT_KINDS: [&str; 5] = [
    "supports",
    "contradicts",
    "contextualizes",
    "source_only",
    "unverifiable",
];

/// The typed assessment submission.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct AssessmentSubmission {
    pub claim_id: String,
    pub snapshot_id: String,
    pub assessment: String,
    pub author: String,
    #[serde(default)]
    pub verifier: Option<String>,
    pub excerpt: String,
    #[serde(default)]
    pub selector: Option<String>,
    pub rationale: String,
    #[serde(default = "unassessed")]
    pub source_authority: String,
    #[serde(default = "unassessed")]
    pub freshness: String,
    #[serde(default = "unassessed")]
    pub independence: String,
    #[serde(default = "unassessed")]
    pub uncertainty: String,
}

fn unassessed() -> String {
    "unassessed".to_owned()
}

/// The stored assessment (the read surface).
#[derive(Debug, Clone, Serialize)]
pub struct StoredAssessment {
    pub assessment_id: String,
    pub claim_id: String,
    pub snapshot_id: String,
    pub assessment: String,
    pub author: String,
    pub verifier: Option<String>,
    pub excerpt: String,
    pub selector: Option<String>,
    pub rationale: String,
    pub source_authority: String,
    pub freshness: String,
    pub independence: String,
    pub uncertainty: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

// Only `Debug` derives: the storage variant carries the original SQLx error
// so it survives to `std::error::Error::source`, and that error is neither
// `Clone` nor `Eq`. This matches the `GrantCreateError` contract from `.3.3.4.3.1`.
#[derive(Debug)]
#[non_exhaustive]
pub enum AssessmentError {
    UnknownKind(String),
    SnapshotMissing,
    ExcerptAbsent,
    /// The store itself failed. A database fault does not prove anything
    /// about the caller's input, and must never be reported as though it did.
    Storage(sqlx::Error),
}

impl std::fmt::Display for AssessmentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Storage(_) => write!(f, "the evidence store is unavailable"),
            Self::UnknownKind(kind) => {
                write!(f, "the assessment `{kind}` is outside the §12.7 vocabulary")
            }
            Self::SnapshotMissing => write!(f, "the cited snapshot does not exist"),
            Self::ExcerptAbsent => write!(
                f,
                "the excerpt does not appear in the snapshot's bytes — the citation is refused"
            ),
        }
    }
}

impl std::error::Error for AssessmentError {}

/// Submit an assessment: the kind must be one of the five; the citation is
/// VALIDATED — the excerpt must appear in the snapshot's raw bytes. The
/// same claim + snapshot + kind + author is the REPLAY.
pub async fn submit(
    pool: &PgPool,
    submission: &AssessmentSubmission,
) -> Result<String, AssessmentError> {
    if !ASSESSMENT_KINDS.contains(&submission.assessment.as_str()) {
        return Err(AssessmentError::UnknownKind(submission.assessment.clone()));
    }
    let bytes: Option<Vec<u8>> = sqlx::query_scalar(
        "SELECT o.bytes FROM snapshot_objects o \
         JOIN evidence_snapshots s ON s.raw_digest = o.digest \
         WHERE s.snapshot_id = $1",
    )
    .bind(&submission.snapshot_id)
    .fetch_optional(pool)
    .await
    .map_err(AssessmentError::Storage)?;
    let bytes = bytes.ok_or(AssessmentError::SnapshotMissing)?;
    let excerpt = submission.excerpt.as_bytes();
    if excerpt.is_empty() || !bytes.windows(excerpt.len()).any(|window| window == excerpt) {
        return Err(AssessmentError::ExcerptAbsent);
    }
    let existing: Option<String> = sqlx::query_scalar(
        "SELECT assessment_id FROM claim_assessments \
         WHERE claim_id = $1 AND snapshot_id = $2 AND assessment = $3 AND author = $4 LIMIT 1",
    )
    .bind(&submission.claim_id)
    .bind(&submission.snapshot_id)
    .bind(&submission.assessment)
    .bind(&submission.author)
    .fetch_optional(pool)
    .await
    .map_err(AssessmentError::Storage)?;
    if let Some(existing) = existing {
        return Ok(existing);
    }
    let assessment_id = crate::snapshots::evidence_id("asn");
    sqlx::query(
        "INSERT INTO claim_assessments \
         (assessment_id, claim_id, snapshot_id, assessment, author, verifier, excerpt, \
          selector, rationale, source_authority, freshness, independence, uncertainty) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)",
    )
    .bind(&assessment_id)
    .bind(&submission.claim_id)
    .bind(&submission.snapshot_id)
    .bind(&submission.assessment)
    .bind(&submission.author)
    .bind(&submission.verifier)
    .bind(&submission.excerpt)
    .bind(&submission.selector)
    .bind(&submission.rationale)
    .bind(&submission.source_authority)
    .bind(&submission.freshness)
    .bind(&submission.independence)
    .bind(&submission.uncertainty)
    .execute(pool)
    .await
    .map_err(AssessmentError::Storage)?;
    Ok(assessment_id)
}

/// The read surface: the snapshot's assessments (oldest first).
pub async fn assessments_for_snapshot(
    pool: &PgPool,
    snapshot_id: &str,
) -> Result<Vec<StoredAssessment>, sqlx::Error> {
    rows(
        pool,
        "SELECT assessment_id, claim_id, snapshot_id, assessment, author, verifier, excerpt, \
                selector, rationale, source_authority, freshness, independence, uncertainty, created_at \
         FROM claim_assessments WHERE snapshot_id = $1 ORDER BY created_at",
        snapshot_id,
    )
    .await
}

/// The read surface: the claim's assessments (oldest first).
pub async fn assessments_of_claim(
    pool: &PgPool,
    claim_id: &str,
) -> Result<Vec<StoredAssessment>, sqlx::Error> {
    rows(
        pool,
        "SELECT assessment_id, claim_id, snapshot_id, assessment, author, verifier, excerpt, \
                selector, rationale, source_authority, freshness, independence, uncertainty, created_at \
         FROM claim_assessments WHERE claim_id = $1 ORDER BY created_at",
        claim_id,
    )
    .await
}

async fn rows(
    pool: &PgPool,
    query: &str,
    bind: &str,
) -> Result<Vec<StoredAssessment>, sqlx::Error> {
    let rows = sqlx::query_as::<_, AssessmentRow>(query)
        .bind(bind)
        .fetch_all(pool)
        .await?;
    Ok(rows.into_iter().map(Into::into).collect())
}

#[derive(sqlx::FromRow)]
struct AssessmentRow {
    assessment_id: String,
    claim_id: String,
    snapshot_id: String,
    assessment: String,
    author: String,
    verifier: Option<String>,
    excerpt: String,
    selector: Option<String>,
    rationale: String,
    source_authority: String,
    freshness: String,
    independence: String,
    uncertainty: String,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl From<AssessmentRow> for StoredAssessment {
    fn from(row: AssessmentRow) -> Self {
        Self {
            assessment_id: row.assessment_id,
            claim_id: row.claim_id,
            snapshot_id: row.snapshot_id,
            assessment: row.assessment,
            author: row.author,
            verifier: row.verifier,
            excerpt: row.excerpt,
            selector: row.selector,
            rationale: row.rationale,
            source_authority: row.source_authority,
            freshness: row.freshness,
            independence: row.independence,
            uncertainty: row.uncertainty,
            created_at: row.created_at,
        }
    }
}
