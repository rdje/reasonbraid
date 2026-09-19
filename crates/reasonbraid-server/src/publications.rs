//! The publication records + the staging (`PHASE-6.4.2`, ADR-020): the
//! publication is the nine-step §15.7 machine's AGGREGATE — its own row
//! (never folded into the approval) carrying the decision/approval/
//! projection references, the manifest digest, the typed state
//! (staged → effective | failed), the Git object ids, and the failure
//! reason. A step that cannot complete is the typed failure, never a skip.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;

/// The publication-state vocabulary (the §15.7 machine's typed states).
pub const PUBLICATION_STATES: [&str; 3] = ["staged", "effective", "failed"];

/// The publication submission (`.4.2`): the references the machine verifies
/// and the manifest digest (the ADR-011 shape over the compiled bundle, the
/// lock, and the authority basis).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicationInput {
    pub publication_id: String,
    pub proposal_id: String,
    pub decision_id: String,
    pub approval_id: String,
    pub projection_id: String,
    pub manifest_digest: String,
}

/// The stored publication row.
#[derive(Debug, Clone, Serialize)]
pub struct StoredPublication {
    pub publication_id: String,
    pub proposal_id: String,
    pub decision_id: String,
    pub approval_id: String,
    pub projection_id: String,
    pub state: String,
    pub manifest_digest: String,
    pub git_object_ids: Vec<String>,
    pub failed_reason: Option<String>,
}

/// The typed refusal reasons.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PublicationError {
    UnknownProposal(String),
    UnknownDecision(String),
    UnknownApproval(String),
    UnknownProjection(String),
    ForeignRecord {
        kind: String,
        id: String,
        proposal_id: String,
    },
    WrongStage {
        publication_id: String,
        state: String,
    },
    MalformedDigest(String),
    EmptyObjectIds,
    /// Declared Git object ids that name nothing in the publication
    /// repository (`SIGNOFF-REPAIR.9.2.1.3`).
    UnknownObjects(Vec<String>),
    /// The publication repository could not be read at all — a deployment
    /// fault, and deliberately NOT the same answer as an absent object.
    RepositoryUnreadable(String),
    Duplicate(String),
}

impl std::fmt::Display for PublicationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PublicationError::UnknownProposal(p) => {
                write!(f, "proposal `{p}` does not exist")
            }
            PublicationError::UnknownDecision(d) => {
                write!(f, "decision `{d}` does not exist")
            }
            PublicationError::UnknownApproval(a) => {
                write!(f, "approval `{a}` does not exist")
            }
            PublicationError::UnknownProjection(p) => {
                write!(f, "projection `{p}` does not exist")
            }
            PublicationError::ForeignRecord {
                kind,
                id,
                proposal_id,
            } => {
                write!(
                    f,
                    "the {kind} `{id}` does not belong to proposal `{proposal_id}`"
                )
            }
            PublicationError::WrongStage {
                publication_id,
                state,
            } => {
                write!(f, "publication `{publication_id}` is at stage `{state}` — the transition does not apply")
            }
            PublicationError::MalformedDigest(d) => {
                write!(f, "digest `{d}` is not the ADR-011 `sha256:<64 hex>` shape")
            }
            PublicationError::EmptyObjectIds => {
                write!(f, "the effective publication records its Git object ids")
            }
            PublicationError::UnknownObjects(ids) => {
                write!(
                    f,
                    "the declared Git object ids name nothing in the publication repository: {}",
                    ids.join(", ")
                )
            }
            PublicationError::RepositoryUnreadable(why) => {
                write!(f, "the publication repository cannot be read: {why}")
            }
            PublicationError::Duplicate(what) => {
                write!(
                    f,
                    "{what} already exists — the record's identity is its content"
                )
            }
        }
    }
}

fn is_sha256_hex(digest: &str) -> bool {
    digest
        .strip_prefix("sha256:")
        .map(|hex| hex.len() == 64 && hex.bytes().all(|b| b.is_ascii_hexdigit()))
        .unwrap_or(false)
}

/// The stored publication row shape (the query tuple).
type PublicationRow = (
    String,
    String,
    String,
    String,
    String,
    String,
    String,
    Value,
    Option<String>,
);

/// Stage one publication (the §15.7 steps 1–4's record half): the
/// references must resolve AND belong to the proposal; the proposal must be
/// APPROVED (the publication follows the approval — the ADR-032 chain).
pub async fn stage(
    pool: &PgPool,
    input: &PublicationInput,
) -> Result<StoredPublication, PublicationError> {
    if !is_sha256_hex(&input.manifest_digest) {
        return Err(PublicationError::MalformedDigest(
            input.manifest_digest.clone(),
        ));
    }
    // ⛔ `tenant_id` joins the read `SIGNOFF-REPAIR.6.1.5.2`: a publication's
    // tenant is its PROPOSAL's, not its caller's. The publication is staged FROM
    // this proposal — every reference below is checked against it — so stamping
    // the caller's tenant on a publication of another tenant's proposal would
    // hide it from the only party it concerns, which is `.6.1.5`'s own trap
    // re-entered from the write side.
    //
    // ⚠️ This LABELS the row; it does not GATE the write. `stage` still admits
    // any enrolled principal to stage another tenant's approved proposal —
    // measured, and owned by `.6.1.5.2.1` rather than quietly widened here.
    let proposal: Option<(String, Option<String>)> =
        sqlx::query_as("SELECT status, tenant_id FROM policy_proposals WHERE proposal_id = $1")
            .bind(&input.proposal_id)
            .fetch_optional(pool)
            .await
            .map_err(|_| PublicationError::UnknownProposal(input.proposal_id.clone()))?;
    let Some((status, tenant_id)) = proposal else {
        return Err(PublicationError::UnknownProposal(input.proposal_id.clone()));
    };
    if status != "approved" {
        return Err(PublicationError::WrongStage {
            publication_id: input.proposal_id.clone(),
            state: status,
        });
    }
    let decision_proposal: Option<String> =
        sqlx::query_scalar("SELECT proposal_id FROM policy_decisions WHERE decision_id = $1")
            .bind(&input.decision_id)
            .fetch_optional(pool)
            .await
            .map_err(|_| PublicationError::UnknownDecision(input.decision_id.clone()))?;
    match decision_proposal {
        None => return Err(PublicationError::UnknownDecision(input.decision_id.clone())),
        Some(proposal) if proposal != input.proposal_id => {
            return Err(PublicationError::ForeignRecord {
                kind: "decision".to_string(),
                id: input.decision_id.clone(),
                proposal_id: input.proposal_id.clone(),
            })
        }
        _ => {}
    }
    let approval_proposal: Option<String> =
        sqlx::query_scalar("SELECT proposal_id FROM policy_approvals WHERE approval_id = $1")
            .bind(&input.approval_id)
            .fetch_optional(pool)
            .await
            .map_err(|_| PublicationError::UnknownApproval(input.approval_id.clone()))?;
    match approval_proposal {
        None => return Err(PublicationError::UnknownApproval(input.approval_id.clone())),
        Some(proposal) if proposal != input.proposal_id => {
            return Err(PublicationError::ForeignRecord {
                kind: "approval".to_string(),
                id: input.approval_id.clone(),
                proposal_id: input.proposal_id.clone(),
            })
        }
        _ => {}
    }
    let projection: Option<bool> = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM policy_projections WHERE projection_id = $1)",
    )
    .bind(&input.projection_id)
    .fetch_one(pool)
    .await
    .map_err(|_| PublicationError::UnknownProjection(input.projection_id.clone()))?;
    if !projection.unwrap_or(false) {
        return Err(PublicationError::UnknownProjection(
            input.projection_id.clone(),
        ));
    }
    let inserted = sqlx::query(
        "INSERT INTO policy_publications \
         (publication_id, proposal_id, decision_id, approval_id, projection_id, state, \
          manifest_digest, tenant_id) VALUES ($1, $2, $3, $4, $5, 'staged', $6, $7)",
    )
    .bind(&input.publication_id)
    .bind(&input.proposal_id)
    .bind(&input.decision_id)
    .bind(&input.approval_id)
    .bind(&input.projection_id)
    .bind(&input.manifest_digest)
    .bind(&tenant_id)
    .execute(pool)
    .await;
    if inserted.is_err() {
        return Err(PublicationError::Duplicate(format!(
            "publication `{}`",
            input.publication_id
        )));
    }
    Ok(StoredPublication {
        publication_id: input.publication_id.clone(),
        proposal_id: input.proposal_id.clone(),
        decision_id: input.decision_id.clone(),
        approval_id: input.approval_id.clone(),
        projection_id: input.projection_id.clone(),
        state: "staged".to_string(),
        manifest_digest: input.manifest_digest.clone(),
        git_object_ids: Vec::new(),
        failed_reason: None,
    })
}

/// Mark one publication EFFECTIVE (the §15.7 step 8's record half): the
/// staged → effective transition with the Git object ids recorded.
pub async fn mark_effective(
    pool: &PgPool,
    publication_id: &str,
    git_object_ids: Vec<String>,
    repository: &crate::publisher::PublicationRepository,
) -> Result<StoredPublication, PublicationError> {
    if git_object_ids.is_empty() {
        return Err(PublicationError::EmptyObjectIds);
    }
    let row: Option<(String, String)> = sqlx::query_as(
        "SELECT state, publication_id FROM policy_publications WHERE publication_id = $1",
    )
    .bind(publication_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| PublicationError::UnknownProposal(publication_id.to_string()))?;
    let Some((state, _)) = row else {
        return Err(PublicationError::UnknownProposal(
            publication_id.to_string(),
        ));
    };
    if state != "staged" {
        return Err(PublicationError::WrongStage {
            publication_id: publication_id.to_string(),
            state,
        });
    }
    // `SIGNOFF-REPAIR.9.2.1.3`: a declared object id is a CLAIM about the
    // repository, and nothing used to check it — the row recorded whatever the
    // caller sent, and three of this project's own fixtures drove the
    // transition with `abc123`, which is not even a well-formed id. ⛔ The
    // check lives HERE, in the core both publish verbs go through, rather than
    // in either handler: a seam-level repair would leave the sibling caller
    // open and the seam's own suite green (`.6.1.2`). The repository is a
    // `PublicationRepository`, which has no constructor but
    // `publisher::resolve_repository`, so it has already passed containment.
    // ⚠️ Ordered after the stage read on purpose, so the existing empty-list
    // and wrong-stage refusals answer first and a doomed request never pays
    // for a repository open.
    let missing = crate::publisher::missing_objects(repository, &git_object_ids)
        .map_err(|e| PublicationError::RepositoryUnreadable(e.to_string()))?;
    if !missing.is_empty() {
        return Err(PublicationError::UnknownObjects(missing));
    }
    sqlx::query(
        "UPDATE policy_publications SET state = 'effective', git_object_ids = $2 \
         WHERE publication_id = $1",
    )
    .bind(publication_id)
    .bind(serde_json::to_value(&git_object_ids).expect("the object ids serialize"))
    .execute(pool)
    .await
    .map_err(|_| PublicationError::UnknownProposal(publication_id.to_string()))?;
    load(pool, publication_id).await
}

/// Mark one publication FAILED (the typed failure — never a skip): the
/// staged → failed transition with the reason.
pub async fn mark_failed(
    pool: &PgPool,
    publication_id: &str,
    reason: &str,
) -> Result<StoredPublication, PublicationError> {
    let row: Option<String> =
        sqlx::query_scalar("SELECT state FROM policy_publications WHERE publication_id = $1")
            .bind(publication_id)
            .fetch_optional(pool)
            .await
            .map_err(|_| PublicationError::UnknownProposal(publication_id.to_string()))?;
    let Some(state) = row else {
        return Err(PublicationError::UnknownProposal(
            publication_id.to_string(),
        ));
    };
    if state != "staged" {
        return Err(PublicationError::WrongStage {
            publication_id: publication_id.to_string(),
            state,
        });
    }
    sqlx::query(
        "UPDATE policy_publications SET state = 'failed', failed_reason = $2 \
         WHERE publication_id = $1",
    )
    .bind(publication_id)
    .bind(reason)
    .execute(pool)
    .await
    .map_err(|_| PublicationError::UnknownProposal(publication_id.to_string()))?;
    load(pool, publication_id).await
}

/// Load one publication row (pub — the `.4.3.2` publish verb reads it).
pub async fn load(
    pool: &PgPool,
    publication_id: &str,
) -> Result<StoredPublication, PublicationError> {
    let row: Option<PublicationRow> = sqlx::query_as(
        "SELECT publication_id, proposal_id, decision_id, approval_id, projection_id, state, \
             manifest_digest, git_object_ids, failed_reason \
             FROM policy_publications WHERE publication_id = $1",
    )
    .bind(publication_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| PublicationError::UnknownProposal(publication_id.to_string()))?;
    let Some((
        publication_id,
        proposal_id,
        decision_id,
        approval_id,
        projection_id,
        state,
        manifest_digest,
        git_object_ids,
        failed_reason,
    )) = row
    else {
        return Err(PublicationError::UnknownProposal(
            publication_id.to_string(),
        ));
    };
    Ok(StoredPublication {
        publication_id,
        proposal_id,
        decision_id,
        approval_id,
        projection_id,
        state,
        manifest_digest,
        git_object_ids: serde_json::from_value(git_object_ids).expect("the object ids parse"),
        failed_reason,
    })
}

/// The publications, newest first.
pub async fn list(pool: &PgPool) -> Result<Vec<StoredPublication>, sqlx::Error> {
    let rows: Vec<PublicationRow> = sqlx::query_as(
        "SELECT publication_id, proposal_id, decision_id, approval_id, projection_id, state, \
             manifest_digest, git_object_ids, failed_reason \
             FROM policy_publications ORDER BY created_at DESC",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(
            |(
                publication_id,
                proposal_id,
                decision_id,
                approval_id,
                projection_id,
                state,
                manifest_digest,
                git_object_ids,
                failed_reason,
            )| StoredPublication {
                publication_id,
                proposal_id,
                decision_id,
                approval_id,
                projection_id,
                state,
                manifest_digest,
                git_object_ids: serde_json::from_value(git_object_ids)
                    .expect("the object ids parse"),
                failed_reason,
            },
        )
        .collect())
}
