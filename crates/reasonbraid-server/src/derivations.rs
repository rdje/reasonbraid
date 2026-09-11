//! The derivation graph (PHASE-4.6.2, ROADMAP §12.6): every transformation
//! is a `Derivation` edge — the derived content carries its OWN ADR-011
//! digest (verified over the content, never trusted) and the parent link to
//! the snapshot it came from. A quote, a summary, an OCR result, or a
//! model-generated caption is NEVER the original — the graph says so, and
//! the parent stays addressable.

use serde::{Deserialize, Serialize};
use sqlx::PgPool;

/// The typed derivation submission.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct DerivationSubmission {
    pub parent_snapshot_id: String,
    /// The transformation kind (the §12.6 named set): `chunk`, `text`,
    /// `excerpt`, `summary`, `ocr`, `caption`, `analysis`.
    pub derived_kind: String,
    pub derived_digest: String,
    pub content: String,
    #[serde(default)]
    pub extraction_version: Option<String>,
    #[serde(default)]
    pub source_selector: Option<String>,
}

/// The stored derivation (the read surface).
#[derive(Debug, Clone, Serialize)]
pub struct StoredDerivation {
    pub derivation_id: String,
    pub parent_snapshot_id: String,
    pub derived_kind: String,
    pub derived_digest: String,
    pub content: String,
    pub extraction_version: Option<String>,
    pub source_selector: Option<String>,
    pub derived_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DerivationError {
    InvalidDigest,
    DigestMismatch { declared: String, actual: String },
    ParentMissing,
}

impl std::fmt::Display for DerivationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidDigest => write!(f, "the derived digest is not the ADR-011 shape"),
            Self::DigestMismatch { declared, actual } => write!(
                f,
                "the content hashes to `{actual}`, not the declared `{declared}`"
            ),
            Self::ParentMissing => write!(f, "the parent snapshot does not exist"),
        }
    }
}

impl std::error::Error for DerivationError {}

/// Submit a derivation: the content MUST hash to the declared digest; the
/// parent snapshot must exist; the same parent + kind + digest is the
/// REPLAY (the same id).
pub async fn submit(
    pool: &PgPool,
    submission: &DerivationSubmission,
) -> Result<String, DerivationError> {
    if !submission.derived_digest.starts_with("sha256:")
        || submission.derived_digest.len() != 7 + 64
        || !submission.derived_digest[7..]
            .chars()
            .all(|c| c.is_ascii_hexdigit())
    {
        return Err(DerivationError::InvalidDigest);
    }
    let actual = crate::fetcher::digest_sha256_hex(submission.content.as_bytes());
    if actual != submission.derived_digest {
        return Err(DerivationError::DigestMismatch {
            declared: submission.derived_digest.clone(),
            actual,
        });
    }
    let parent_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM evidence_snapshots WHERE snapshot_id = $1)",
    )
    .bind(&submission.parent_snapshot_id)
    .fetch_one(pool)
    .await
    .map_err(|_| DerivationError::ParentMissing)?;
    if !parent_exists {
        return Err(DerivationError::ParentMissing);
    }
    let existing: Option<String> = sqlx::query_scalar(
        "SELECT derivation_id FROM derivations \
         WHERE parent_snapshot_id = $1 AND derived_kind = $2 AND derived_digest = $3 LIMIT 1",
    )
    .bind(&submission.parent_snapshot_id)
    .bind(&submission.derived_kind)
    .bind(&submission.derived_digest)
    .fetch_optional(pool)
    .await
    .map_err(|_| DerivationError::ParentMissing)?;
    if let Some(existing) = existing {
        return Ok(existing);
    }
    let derivation_id = crate::snapshots::evidence_id("drv");
    sqlx::query(
        "INSERT INTO derivations \
         (derivation_id, parent_snapshot_id, derived_kind, derived_digest, content, \
          extraction_version, source_selector) \
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(&derivation_id)
    .bind(&submission.parent_snapshot_id)
    .bind(&submission.derived_kind)
    .bind(&submission.derived_digest)
    .bind(&submission.content)
    .bind(&submission.extraction_version)
    .bind(&submission.source_selector)
    .execute(pool)
    .await
    .map_err(|_| DerivationError::ParentMissing)?;
    Ok(derivation_id)
}

/// The parent/derived traversal: every derivation of the snapshot, oldest
/// first.
pub async fn children_of(
    pool: &PgPool,
    parent_snapshot_id: &str,
) -> Result<Vec<StoredDerivation>, sqlx::Error> {
    let rows = sqlx::query_as::<_, DerivationRow>(
        "SELECT derivation_id, parent_snapshot_id, derived_kind, derived_digest, content, \
                extraction_version, source_selector, derived_at \
         FROM derivations WHERE parent_snapshot_id = $1 ORDER BY derived_at",
    )
    .bind(parent_snapshot_id)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(Into::into).collect())
}

#[derive(sqlx::FromRow)]
struct DerivationRow {
    derivation_id: String,
    parent_snapshot_id: String,
    derived_kind: String,
    derived_digest: String,
    content: String,
    extraction_version: Option<String>,
    source_selector: Option<String>,
    derived_at: chrono::DateTime<chrono::Utc>,
}

impl From<DerivationRow> for StoredDerivation {
    fn from(row: DerivationRow) -> Self {
        Self {
            derivation_id: row.derivation_id,
            parent_snapshot_id: row.parent_snapshot_id,
            derived_kind: row.derived_kind,
            derived_digest: row.derived_digest,
            content: row.content,
            extraction_version: row.extraction_version,
            source_selector: row.source_selector,
            derived_at: row.derived_at,
        }
    }
}
