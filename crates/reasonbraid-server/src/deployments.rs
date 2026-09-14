//! The target-deployment records + the waves (`PHASE-6.5.2`, ADR-021): the
//! deployment is PER-TARGET, never globally atomic — the desired/observed
//! pair rides each assignment; the receipt attests the OBSERVED digest (the
//! drift's comparison input, never the hope).

use reasonbraid_core::GrantSubject;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

/// The target-type vocabulary (the §15.9 eligible targets' initial set).
pub const TARGET_TYPES: [&str; 3] = ["repository", "node_policy", "service_config"];

/// The observed-state vocabulary.
pub const OBSERVED_STATES: [&str; 4] = ["pending", "applied", "waived", "rejected"];

/// The target submission (`.5.2`): the id + the type + the OWNING AUTHORITY
/// (the grant reference — the ADR-021 authority, checked like the policies').
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TargetInput {
    pub target_id: String,
    pub target_type: String,
    pub owning_authority: String,
}

/// The assignment submission (`.5.2`): the target + the effective
/// publication + the canary wave + the DESIRED pair (the ref + the
/// digest).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssignmentInput {
    pub target_id: String,
    pub publication_id: String,
    pub wave: i64,
    pub desired_ref: String,
    pub desired_digest: String,
}

/// The receipt submission (`.5.2`): the OBSERVED digest + the observed
/// state (the attestation).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReceiptInput {
    pub observed_digest: String,
    pub observed_state: String,
}

/// The stored assignment row.
#[derive(Debug, Clone, Serialize)]
pub struct StoredAssignment {
    pub target_id: String,
    pub publication_id: String,
    pub wave: i64,
    pub desired_ref: String,
    pub desired_digest: String,
    pub observed_digest: Option<String>,
    pub observed_state: String,
}

/// The typed refusal reasons.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeploymentError {
    UnknownTarget(String),
    UnknownType(String),
    GhostAuthority(String),
    UnknownPublication(String),
    NotEffective(String),
    MalformedDigest(String),
    UnknownState(String),
    UnknownAssignment {
        target_id: String,
        publication_id: String,
    },
    Duplicate(String),
}

impl std::fmt::Display for DeploymentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeploymentError::UnknownTarget(t) => write!(f, "target `{t}` does not exist"),
            DeploymentError::UnknownType(t) => {
                write!(
                    f,
                    "target type `{t}` is not in the vocabulary ({})",
                    TARGET_TYPES.join(", ")
                )
            }
            DeploymentError::GhostAuthority(g) => {
                write!(
                    f,
                    "the owning authority `{g}` is not an active, unexpired grant"
                )
            }
            DeploymentError::UnknownPublication(p) => write!(f, "publication `{p}` does not exist"),
            DeploymentError::NotEffective(p) => {
                write!(f, "publication `{p}` is not effective — the deployment rides the EFFECTIVE publication")
            }
            DeploymentError::MalformedDigest(d) => {
                write!(f, "digest `{d}` is not the ADR-011 `sha256:<64 hex>` shape")
            }
            DeploymentError::UnknownState(s) => {
                write!(
                    f,
                    "observed state `{s}` is not in the vocabulary ({})",
                    OBSERVED_STATES.join(", ")
                )
            }
            DeploymentError::UnknownAssignment {
                target_id,
                publication_id,
            } => {
                write!(
                    f,
                    "assignment ({target_id}, {publication_id}) does not exist"
                )
            }
            DeploymentError::Duplicate(what) => {
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

/// Register one deployment target: the owning authority must be a grant the
/// CALLER HOLDS (`SIGNOFF-REPAIR.9.3.1`).
///
/// ⛔ The check used to ask only whether the named grant existed and was
/// active, so a caller owned a target with any grant it could name — the same
/// defect as `corrections::authority_holds`, in a second module, and repaired
/// with the same predicate rather than a second spelling of it.
pub async fn register_target(
    pool: &PgPool,
    principal: &GrantSubject,
    input: &TargetInput,
) -> Result<(), DeploymentError> {
    if !TARGET_TYPES.contains(&input.target_type.as_str()) {
        return Err(DeploymentError::UnknownType(input.target_type.clone()));
    }
    let held = crate::authority::grant_held_by(pool, &input.owning_authority, principal)
        .await
        .map_err(|_| DeploymentError::GhostAuthority(input.owning_authority.clone()))?;
    if !held {
        return Err(DeploymentError::GhostAuthority(
            input.owning_authority.clone(),
        ));
    }
    let inserted = sqlx::query(
        "INSERT INTO deployment_targets (target_id, target_type, owning_authority) \
         VALUES ($1, $2, $3)",
    )
    .bind(&input.target_id)
    .bind(&input.target_type)
    .bind(&input.owning_authority)
    .execute(pool)
    .await;
    match inserted {
        Ok(_) => Ok(()),
        Err(_) => Err(DeploymentError::Duplicate(format!(
            "target `{}`",
            input.target_id
        ))),
    }
}

/// Assign one publication to one target (the canary wave + the desired
/// pair). The publication must be EFFECTIVE.
pub async fn assign(
    pool: &PgPool,
    input: &AssignmentInput,
) -> Result<StoredAssignment, DeploymentError> {
    if !is_sha256_hex(&input.desired_digest) {
        return Err(DeploymentError::MalformedDigest(
            input.desired_digest.clone(),
        ));
    }
    let target: Option<bool> =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM deployment_targets WHERE target_id = $1)")
            .bind(&input.target_id)
            .fetch_one(pool)
            .await
            .map_err(|_| DeploymentError::UnknownTarget(input.target_id.clone()))?;
    if !target.unwrap_or(false) {
        return Err(DeploymentError::UnknownTarget(input.target_id.clone()));
    }
    let publication: Option<String> =
        sqlx::query_scalar("SELECT state FROM policy_publications WHERE publication_id = $1")
            .bind(&input.publication_id)
            .fetch_optional(pool)
            .await
            .map_err(|_| DeploymentError::UnknownPublication(input.publication_id.clone()))?;
    let Some(state) = publication else {
        return Err(DeploymentError::UnknownPublication(
            input.publication_id.clone(),
        ));
    };
    if state != "effective" {
        return Err(DeploymentError::NotEffective(input.publication_id.clone()));
    }
    let inserted = sqlx::query(
        "INSERT INTO deployment_assignments \
         (target_id, publication_id, wave, desired_ref, desired_digest, observed_digest, observed_state) \
         VALUES ($1, $2, $3, $4, $5, NULL, 'pending')",
    )
    .bind(&input.target_id)
    .bind(&input.publication_id)
    .bind(input.wave)
    .bind(&input.desired_ref)
    .bind(&input.desired_digest)
    .execute(pool)
    .await;
    if inserted.is_err() {
        return Err(DeploymentError::Duplicate(format!(
            "assignment ({}, {})",
            input.target_id, input.publication_id
        )));
    }
    Ok(StoredAssignment {
        target_id: input.target_id.clone(),
        publication_id: input.publication_id.clone(),
        wave: input.wave,
        desired_ref: input.desired_ref.clone(),
        desired_digest: input.desired_digest.clone(),
        observed_digest: None,
        observed_state: "pending".to_string(),
    })
}

/// Record the receipt (the OBSERVED digest + the state — the attestation).
pub async fn record_receipt(
    pool: &PgPool,
    target_id: &str,
    publication_id: &str,
    input: &ReceiptInput,
) -> Result<StoredAssignment, DeploymentError> {
    if !is_sha256_hex(&input.observed_digest) {
        return Err(DeploymentError::MalformedDigest(
            input.observed_digest.clone(),
        ));
    }
    if !OBSERVED_STATES.contains(&input.observed_state.as_str()) {
        return Err(DeploymentError::UnknownState(input.observed_state.clone()));
    }
    let exists: Option<bool> = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM deployment_assignments \
         WHERE target_id = $1 AND publication_id = $2)",
    )
    .bind(target_id)
    .bind(publication_id)
    .fetch_one(pool)
    .await
    .map_err(|_| DeploymentError::UnknownAssignment {
        target_id: target_id.to_string(),
        publication_id: publication_id.to_string(),
    })?;
    if !exists.unwrap_or(false) {
        return Err(DeploymentError::UnknownAssignment {
            target_id: target_id.to_string(),
            publication_id: publication_id.to_string(),
        });
    }
    sqlx::query(
        "UPDATE deployment_assignments SET observed_digest = $3, observed_state = $4 \
         WHERE target_id = $1 AND publication_id = $2",
    )
    .bind(target_id)
    .bind(publication_id)
    .bind(&input.observed_digest)
    .bind(&input.observed_state)
    .execute(pool)
    .await
    .map_err(|_| DeploymentError::UnknownAssignment {
        target_id: target_id.to_string(),
        publication_id: publication_id.to_string(),
    })?;
    load_assignment(pool, target_id, publication_id).await
}

/// The stored assignment row shape (the query tuple).
type AssignmentRow = (String, String, i64, String, String, Option<String>, String);

async fn load_assignment(
    pool: &PgPool,
    target_id: &str,
    publication_id: &str,
) -> Result<StoredAssignment, DeploymentError> {
    let row: Option<AssignmentRow> = sqlx::query_as(
        "SELECT target_id, publication_id, wave, desired_ref, desired_digest, \
             observed_digest, observed_state \
             FROM deployment_assignments WHERE target_id = $1 AND publication_id = $2",
    )
    .bind(target_id)
    .bind(publication_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| DeploymentError::UnknownAssignment {
        target_id: target_id.to_string(),
        publication_id: publication_id.to_string(),
    })?;
    let Some((
        target_id,
        publication_id,
        wave,
        desired_ref,
        desired_digest,
        observed_digest,
        observed_state,
    )) = row
    else {
        return Err(DeploymentError::UnknownAssignment {
            target_id: target_id.to_string(),
            publication_id: publication_id.to_string(),
        });
    };
    Ok(StoredAssignment {
        target_id,
        publication_id,
        wave,
        desired_ref,
        desired_digest,
        observed_digest,
        observed_state,
    })
}

/// The assignments, newest first.
pub async fn list_assignments(pool: &PgPool) -> Result<Vec<StoredAssignment>, sqlx::Error> {
    let rows: Vec<AssignmentRow> = sqlx::query_as(
        "SELECT target_id, publication_id, wave, desired_ref, desired_digest, \
             observed_digest, observed_state \
             FROM deployment_assignments ORDER BY target_id, publication_id",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(
            |(
                target_id,
                publication_id,
                wave,
                desired_ref,
                desired_digest,
                observed_digest,
                observed_state,
            )| {
                StoredAssignment {
                    target_id,
                    publication_id,
                    wave,
                    desired_ref,
                    desired_digest,
                    observed_digest,
                    observed_state,
                }
            },
        )
        .collect())
}
