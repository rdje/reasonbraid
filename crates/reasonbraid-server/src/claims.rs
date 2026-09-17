//! The claim-evidence graph + the citation validation (PHASE-4.6.3, ROADMAP
//! §12.7): claims link to evidence with ONE of the five assessments, and
//! the citation is VALIDATED — the excerpt must actually appear in the
//! snapshot's raw bytes (the claim must point at a REAL snapshot; citation
//! existence alone never satisfies an evidence gate).
//!
//! The evidence a tenant may assess is the evidence it ACQUIRED
//! (`SIGNOFF-REPAIR.11.14.3.8`): [`submit`] refuses a snapshot this tenant's
//! `evidence_citations` row does not cover, before it reads anything about that
//! snapshot, so the write surface answers the same question the five read
//! surfaces answer.

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

/// Which writer minted a row's `claim_id` (`SIGNOFF-REPAIR.11.14.3.3`).
///
/// `claim_id` is one column holding two kinds of identifier, and before this
/// type nothing said which kind a row carried. It is part of the row's IDENTITY
/// rather than a label beside it: it joins the replay key in `migrations/0066`
/// and the pre-check below, because an identifier two writers mint differently
/// is not one identifier.
///
/// ⛔ Server-set, and deliberately not a field of [`AssessmentSubmission`]. A
/// caller that could choose its own namespace would reopen exactly the
/// collision the column closes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClaimNamespace {
    /// The identifier is a claim digest the SERVER computed and
    /// membership-checked against the thread (`SIGNOFF-REPAIR.11.14.3.1`).
    Thread,
    /// The identifier is a caller label carried by `POST /v1/assessments`, the
    /// non-deliberation path. Bound only by the authoring tenant.
    External,
}

impl ClaimNamespace {
    /// The stored spelling — the vocabulary `migrations/0066`'s CHECK pins.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Thread => "thread",
            Self::External => "external",
        }
    }
}

/// The stored assessment (the read surface).
#[derive(Debug, Clone, Serialize)]
pub struct StoredAssessment {
    pub assessment_id: String,
    pub claim_id: String,
    /// Which writer minted `claim_id`; `None` for a row written before
    /// `migrations/0066` (`SIGNOFF-REPAIR.11.14.3.3`).
    pub claim_namespace: Option<String>,
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
    /// The authoring tenant has no citation for the snapshot, so nothing about
    /// that snapshot — its existence included — may be reported to it
    /// (`SIGNOFF-REPAIR.11.14.3.8`).
    SnapshotNotCited,
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
            // ⛔ Deliberately carries NO identifier and no fact about the
            // snapshot. An identifier that names nothing and one that names a
            // snapshot this tenant never cited must be the SAME bytes, or the
            // refusal is the existence oracle this gate closes.
            Self::SnapshotNotCited => write!(
                f,
                "the snapshot is not cited by this tenant — assess evidence this tenant acquired"
            ),
            Self::SnapshotMissing => write!(f, "the cited snapshot does not exist"),
            Self::ExcerptAbsent => write!(
                f,
                "the excerpt does not appear in the snapshot's bytes — the citation is refused"
            ),
        }
    }
}

impl std::error::Error for AssessmentError {}

/// Submit an assessment: the tenant must have CITED the snapshot; the kind must
/// be one of the five; the citation is VALIDATED — the excerpt must appear in
/// the snapshot's raw bytes. The same claim + snapshot + kind + author IN THE
/// SAME NAMESPACE is the REPLAY.
///
/// ⛔ The citation gate is this function's (`SIGNOFF-REPAIR.11.14.3.8`), because
/// this function is what both writers reach. Without it the standalone route
/// answered three distinguishable things about a snapshot the caller never
/// acquired — `SnapshotMissing`, `ExcerptAbsent`, or a 200 saying a chosen
/// substring appears in bytes it was never allowed to read — while every one of
/// the five snapshot READ surfaces already answered such a caller `404`.
///
/// The AUTHORING tenant is recorded by the server and is what the read
/// surfaces are bound to (`SIGNOFF-REPAIR.11.14.2`). ⛔ It is not
/// `submission.author`, which is an unauthenticated caller label — a
/// predicate over a field the caller controls is not an authorization.
/// Generic over the executor for the same reason [`crate::snapshots::is_cited_by`]
/// is: the `assess` step records an assessment inside the thread's own
/// transaction, so the contribution event and the row it produces commit
/// together or not at all (`SIGNOFF-REPAIR.11.14.3.1`).
///
/// ⛔ The `namespace` is the CALLER SITE's, never the submission's
/// (`SIGNOFF-REPAIR.11.14.3.3`). It joins the replay pre-check below for the
/// same reason it joins `claim_assessments_replay_idx`: without it, a caller
/// naming a deliberation's minted claim digest, its snapshot, its kind and its
/// author matched all four columns and was handed the DELIBERATION's
/// `assessment_id` — a row it never contributed. The index alone would not
/// close that, because this pre-check short-circuits before the insert runs.
///
/// ⛔ `authored_by_tenant` is in the pre-check for the SAME reason and it is a
/// separate defect. `author` is caller-supplied here, so a second TENANT
/// presenting the first tenant's label matched every other column — the
/// namespace included — and was handed that tenant's row id. `migrations/0066`
/// records the premise in `0064` this corrects. ⚠️ Two principals inside ONE
/// tenant can still alias each other on the standalone route; that is a
/// deduplication question rather than a disclosure, because the authoring gate
/// already admits both of them to the row.
pub async fn submit<'e, E>(
    mut executor: E,
    submission: &AssessmentSubmission,
    authored_by_tenant: &str,
    namespace: ClaimNamespace,
) -> Result<String, AssessmentError>
where
    E: std::ops::DerefMut,
    for<'c> &'c mut <E as std::ops::Deref>::Target: sqlx::Executor<'c, Database = sqlx::Postgres>,
{
    if !ASSESSMENT_KINDS.contains(&submission.assessment.as_str()) {
        return Err(AssessmentError::UnknownKind(submission.assessment.clone()));
    }
    // ⛔ The citation gate, BEFORE anything is read about the snapshot
    // (`SIGNOFF-REPAIR.11.14.3.8`). The select below joins on `snapshot_id`
    // alone — it has no tenant predicate and cannot have one, because a
    // snapshot row is shared by every tenant that acquired the same bytes —
    // so the binding has to be this separate question, asked first.
    //
    // It lives in the STORE rather than at each writer because the store is
    // what both writers reach: the `assess` step asks the same question
    // itself, one gate earlier, so that a deliberation's refusal can name the
    // step and the snapshot. That check is the named refusal; this one is the
    // invariant, and for the deliberation path it should never fire.
    if !crate::snapshots::is_cited_by(&mut *executor, &submission.snapshot_id, authored_by_tenant)
        .await
        .map_err(AssessmentError::Storage)?
    {
        return Err(AssessmentError::SnapshotNotCited);
    }
    let bytes: Option<Vec<u8>> = sqlx::query_scalar(
        "SELECT o.bytes FROM snapshot_objects o \
         JOIN evidence_snapshots s ON s.raw_digest = o.digest \
         WHERE s.snapshot_id = $1",
    )
    .bind(&submission.snapshot_id)
    .fetch_optional(&mut *executor)
    .await
    .map_err(AssessmentError::Storage)?;
    let bytes = bytes.ok_or(AssessmentError::SnapshotMissing)?;
    let excerpt = submission.excerpt.as_bytes();
    if excerpt.is_empty() || !bytes.windows(excerpt.len()).any(|window| window == excerpt) {
        return Err(AssessmentError::ExcerptAbsent);
    }
    let existing: Option<String> = sqlx::query_scalar(
        "SELECT assessment_id FROM claim_assessments \
         WHERE claim_id = $1 AND snapshot_id = $2 AND assessment = $3 AND author = $4 \
           AND claim_namespace = $5 AND authored_by_tenant = $6 LIMIT 1",
    )
    .bind(&submission.claim_id)
    .bind(&submission.snapshot_id)
    .bind(&submission.assessment)
    .bind(&submission.author)
    .bind(namespace.as_str())
    .bind(authored_by_tenant)
    .fetch_optional(&mut *executor)
    .await
    .map_err(AssessmentError::Storage)?;
    if let Some(existing) = existing {
        return Ok(existing);
    }
    let assessment_id = crate::snapshots::evidence_id("asn");
    sqlx::query(
        "INSERT INTO claim_assessments \
         (assessment_id, claim_id, snapshot_id, assessment, author, verifier, excerpt, \
          selector, rationale, source_authority, freshness, independence, uncertainty, \
          authored_by_tenant, claim_namespace) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)",
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
    .bind(authored_by_tenant)
    .bind(namespace.as_str())
    .execute(&mut *executor)
    .await
    .map_err(AssessmentError::Storage)?;
    Ok(assessment_id)
}

/// The stored assessment's columns — one definition for both bound reads.
const ASSESSMENT_COLUMNS: &str =
    "assessment_id, claim_id, claim_namespace, snapshot_id, assessment, author, verifier, \
     excerpt, selector, rationale, source_authority, freshness, independence, uncertainty, \
     created_at";

/// The read surface: the snapshot's assessments THIS TENANT AUTHORED (oldest
/// first).
///
/// ⭐ Bound on the author rather than on the parent snapshot's citation, which
/// is what settles `SIGNOFF-REPAIR.11.14.1`'s co-citation residual here: two
/// tenants that cite the same shared snapshot both hold the citation, so a
/// citation-based gate would have disclosed each one's analytical position to
/// the other. ⚠️ It settles it for assessments only — `derivations` are
/// content-addressed and remain shared.
pub async fn assessments_for_snapshot(
    pool: &PgPool,
    snapshot_id: &str,
    tenant_id: &str,
) -> Result<Vec<StoredAssessment>, sqlx::Error> {
    rows(
        pool,
        &format!(
            "SELECT {ASSESSMENT_COLUMNS} FROM claim_assessments \
             WHERE snapshot_id = $1 AND authored_by_tenant = $2 ORDER BY created_at"
        ),
        snapshot_id,
        tenant_id,
    )
    .await
}

/// The read surface: the claim's assessments THIS TENANT AUTHORED (oldest
/// first).
///
/// ⛔ `claim_id` is not one namespace: the `assess` step mints a digest and
/// membership-checks it against a thread, while `POST /v1/assessments` takes a
/// caller label. Both land here, because filtering one out would make the
/// standalone route write-only — worse than the removal `SIGNOFF-REPAIR.11.14.3.3`
/// declined. Each row carries `claim_namespace`, so a reader can tell an
/// assessment a deliberation's gates admitted from one asserted beside it. The
/// authoring binding (`.11.14.2`) is what makes a GUESSED identifier useless.
pub async fn assessments_of_claim(
    pool: &PgPool,
    claim_id: &str,
    tenant_id: &str,
) -> Result<Vec<StoredAssessment>, sqlx::Error> {
    rows(
        pool,
        &format!(
            "SELECT {ASSESSMENT_COLUMNS} FROM claim_assessments \
             WHERE claim_id = $1 AND authored_by_tenant = $2 ORDER BY created_at"
        ),
        claim_id,
        tenant_id,
    )
    .await
}

async fn rows(
    pool: &PgPool,
    query: &str,
    bind: &str,
    tenant_id: &str,
) -> Result<Vec<StoredAssessment>, sqlx::Error> {
    let rows = sqlx::query_as::<_, AssessmentRow>(query)
        .bind(bind)
        .bind(tenant_id)
        .fetch_all(pool)
        .await?;
    Ok(rows.into_iter().map(Into::into).collect())
}

#[derive(sqlx::FromRow)]
struct AssessmentRow {
    assessment_id: String,
    claim_id: String,
    claim_namespace: Option<String>,
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
            claim_namespace: row.claim_namespace,
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
