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
/// (the grant reference — the ADR-021 authority, checked like the policies') +
/// the REPORTER, the one principal that files the target's receipts
/// (`SIGNOFF-REPAIR.9.3.3.2`).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TargetInput {
    pub target_id: String,
    pub target_type: String,
    pub owning_authority: String,
    pub reporter: String,
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

/// One kept receipt (`SIGNOFF-REPAIR.9.3.3.3`): who filed it, what it
/// reported, and when, by the database's clock.
#[derive(Debug, Clone, Serialize)]
pub struct StoredReceipt {
    pub receipt_id: String,
    pub reporter: String,
    pub observed_digest: String,
    pub observed_state: String,
    pub recorded_at: chrono::DateTime<chrono::Utc>,
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
    /// The declared digest is not the digest of the publication's projection
    /// (`SIGNOFF-REPAIR.9.3.3.1`, ADR-021: the desired pair IS the publication's).
    DesiredDigestNotProjection {
        declared: String,
        projection: String,
    },
    /// The declared ref is none of the Git object ids the publication recorded
    /// when it became effective.
    DesiredRefNotRecorded(String),
    UnknownState(String),
    UnknownAssignment {
        target_id: String,
        publication_id: String,
    },
    /// The reporter named at registration is not a principal id.
    MalformedReporter(String),
    /// The reporter named at registration is not enrolled.
    UnenrolledReporter(String),
    /// The target was registered before targets named a reporter
    /// (`migrations/0114`), so it takes no receipt.
    NoReporter(String),
    /// The caller is not the principal the target names.
    NotTheReporter {
        principal: String,
        target_id: String,
    },
    Duplicate(String),
    /// The store could not answer: the server's fault, never the caller's
    /// (`api::deployment_refusal` answers it `500`).
    Storage(String),
    /// The caller's input holds a character the store cannot represent
    /// (`api::unrepresentable_input`): the caller's, and permanent.
    UnrepresentableInput,
}

impl DeploymentError {
    /// A store error, classified while its SQLSTATE is still readable: the
    /// server's, unless it is the caller's own unrepresentable input.
    fn storage(error: sqlx::Error) -> Self {
        if crate::api::unrepresentable_input(&error) {
            Self::UnrepresentableInput
        } else {
            Self::Storage(error.to_string())
        }
    }
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
            DeploymentError::DesiredDigestNotProjection {
                declared,
                projection,
            } => write!(
                f,
                "desired_digest `{declared}` is not the publication's projection digest \
                 `{projection}` — a target is assigned what its publication deploys"
            ),
            DeploymentError::DesiredRefNotRecorded(r) => write!(
                f,
                "desired_ref `{r}` is not a Git object id the publication recorded when it \
                 became effective"
            ),
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
            DeploymentError::Storage(detail) => write!(f, "the deployment store failed: {detail}"),
            DeploymentError::UnrepresentableInput => {
                write!(f, "the input holds a character the store cannot represent")
            }
            DeploymentError::MalformedReporter(r) => write!(
                f,
                "reporter `{r}` is not a principal id (expected hpr_… | rol_…)"
            ),
            DeploymentError::UnenrolledReporter(r) => {
                write!(f, "reporter `{r}` is not an enrolled principal")
            }
            DeploymentError::NoReporter(t) => write!(
                f,
                "target `{t}` names no reporter — it was registered before targets named \
                 one, so it takes no receipt"
            ),
            DeploymentError::NotTheReporter {
                principal,
                target_id,
            } => write!(
                f,
                "principal `{principal}` is not target `{target_id}`'s reporter — only the \
                 principal the target names files its receipts"
            ),
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
    // `.9.3.4.2`: HELD **and COVERING** `deployment_target_register`.
    let held = crate::authority::grant_held_by(
        pool,
        &input.owning_authority,
        principal,
        reasonbraid_core::GrantAction::DeploymentTargetRegister,
    )
    .await
    .map_err(|_| DeploymentError::GhostAuthority(input.owning_authority.clone()))?;
    if !held {
        return Err(DeploymentError::GhostAuthority(
            input.owning_authority.clone(),
        ));
    }
    // `SIGNOFF-REPAIR.9.3.3.2`: the target names the one principal that files its
    // receipts, and it must be one that can
    // (`docs/decisions/2026-09-25_a-target-names-its-reporter.md`). ⛔ AFTER the
    // authority, so a caller that may not register learns nothing about which
    // principals are enrolled; and a store failure here is the server's
    // (`Storage`), not an unenrolled reporter.
    let reporter = crate::api::parse_principal(&input.reporter)
        .ok_or_else(|| DeploymentError::MalformedReporter(input.reporter.clone()))?;
    let enrolled = crate::api::reader_tenant(pool, &reporter)
        .await
        .map_err(DeploymentError::storage)?;
    if enrolled.is_none() {
        return Err(DeploymentError::UnenrolledReporter(input.reporter.clone()));
    }
    let inserted = sqlx::query(
        "INSERT INTO deployment_targets (target_id, target_type, owning_authority, reporter) \
         VALUES ($1, $2, $3, $4)",
    )
    .bind(&input.target_id)
    .bind(&input.target_type)
    .bind(&input.owning_authority)
    .bind(reporter.id_string())
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
    tenant_id: &str,
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
    // ⛔ `SIGNOFF-REPAIR.6.1.5.2.1`: an assignment is tenant-owned by its
    // PUBLICATION (DOC-0071), and until now any enrolled principal could deploy
    // another tenant's effective publication to a site target. ⚠️ A foreign
    // publication answers exactly as an absent one; the reasoning is recorded
    // once, at `publications::owned_by`.
    //
    // The same read carries the publication's desired pair (`.9.3.3.1`, below),
    // so the check adds no second lookup that could disagree with this one.
    let publication: Option<(String, Option<String>, serde_json::Value, Option<String>)> =
        sqlx::query_as(
            "SELECT p.state, p.tenant_id, p.git_object_ids, pr.digest \
             FROM policy_publications p \
             LEFT JOIN policy_projections pr ON pr.projection_id = p.projection_id \
             WHERE p.publication_id = $1",
        )
        .bind(&input.publication_id)
        .fetch_optional(pool)
        .await
        .map_err(|_| DeploymentError::UnknownPublication(input.publication_id.clone()))?;
    let Some((state, owner, recorded, projection)) = publication else {
        return Err(DeploymentError::UnknownPublication(
            input.publication_id.clone(),
        ));
    };
    if owner.as_deref() != Some(tenant_id) {
        return Err(DeploymentError::UnknownPublication(
            input.publication_id.clone(),
        ));
    }
    if state != "effective" {
        return Err(DeploymentError::NotEffective(input.publication_id.clone()));
    }
    // ⛔ The desired pair is the PUBLICATION's (`SIGNOFF-REPAIR.9.3.3.1`). ADR-021
    // defines it as *the effective publication's ref id + the attested
    // projection digest*; only the digest's shape was checked, so a target could
    // be assigned content its publication never had, and the drift record then
    // compared an observation against a value nobody published. A publication
    // whose projection row is absent has no digest to deploy, so nothing matches.
    if projection.as_deref() != Some(input.desired_digest.as_str()) {
        return Err(DeploymentError::DesiredDigestNotProjection {
            declared: input.desired_digest.clone(),
            projection: projection.unwrap_or_else(|| "none recorded".to_string()),
        });
    }
    let recorded_ref = recorded
        .as_array()
        .is_some_and(|ids| ids.iter().any(|id| id.as_str() == Some(&input.desired_ref)));
    if !recorded_ref {
        return Err(DeploymentError::DesiredRefNotRecorded(
            input.desired_ref.clone(),
        ));
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

/// Record the receipt (the OBSERVED digest + the state — the attestation),
/// filed by the principal the target names (`SIGNOFF-REPAIR.9.3.3.2`).
pub async fn record_receipt(
    pool: &PgPool,
    principal: &GrantSubject,
    tenant_id: &str,
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
    // ⛔ `SIGNOFF-REPAIR.6.1.5.2.1`: `deployment_assignments` carries no tenant
    // column by DOC-0071's decision — it is tenant-owned BY ITS PUBLICATION, and
    // a target is site-wide by design — so the join is the ownership check, and
    // an assignment whose publication is not the caller's answers as an absent
    // assignment does.
    //
    // ⛔ `SIGNOFF-REPAIR.9.3.3.2`: the same read returns the target's REPORTER,
    // and only that principal files the receipt. The tenant join answers first,
    // so a caller of another tenant gets the unknown-assignment answer it always
    // got; a caller of the right tenant that the target does not name is refused
    // by name. The owner is deliberately not enough: ADR-021's receipt is what the
    // target OBSERVED, and the operator who assigned the desired pair is the hope.
    //
    // ⛔ `SIGNOFF-REPAIR.9.3.3.3`: a receipt is KEPT. It is a row of its own and
    // the assignment's observed pair becomes the latest one, both in this
    // transaction and under a lock on the assignment, so two receipts filed at
    // once are ordered and the pair is always the last row. A receipt used to
    // overwrite the pair in place, destroying the one before it.
    let mut tx = pool.begin().await.map_err(DeploymentError::storage)?;
    let found: Option<(Option<String>, String)> = sqlx::query_as(
        "SELECT t.reporter, p.tenant_id FROM deployment_assignments a \
         JOIN policy_publications p ON p.publication_id = a.publication_id \
         JOIN deployment_targets t ON t.target_id = a.target_id \
         WHERE a.target_id = $1 AND a.publication_id = $2 AND p.tenant_id = $3 \
         FOR UPDATE OF a",
    )
    .bind(target_id)
    .bind(publication_id)
    .bind(tenant_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(DeploymentError::storage)?;
    let Some((reporter, owner)) = found else {
        return Err(DeploymentError::UnknownAssignment {
            target_id: target_id.to_string(),
            publication_id: publication_id.to_string(),
        });
    };
    let Some(reporter) = reporter else {
        return Err(DeploymentError::NoReporter(target_id.to_string()));
    };
    if reporter != principal.id_string() {
        return Err(DeploymentError::NotTheReporter {
            principal: principal.id_string(),
            target_id: target_id.to_string(),
        });
    }
    sqlx::query(
        "INSERT INTO deployment_receipts \
         (receipt_id, target_id, publication_id, tenant_id, reporter, observed_digest, \
          observed_state) VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(crate::snapshots::evidence_id("drc"))
    .bind(target_id)
    .bind(publication_id)
    .bind(&owner)
    .bind(&reporter)
    .bind(&input.observed_digest)
    .bind(&input.observed_state)
    .execute(&mut *tx)
    .await
    .map_err(DeploymentError::storage)?;
    let row: AssignmentRow = sqlx::query_as(
        "UPDATE deployment_assignments SET observed_digest = $3, observed_state = $4 \
         WHERE target_id = $1 AND publication_id = $2 \
         RETURNING target_id, publication_id, wave, desired_ref, desired_digest, \
             observed_digest, observed_state",
    )
    .bind(target_id)
    .bind(publication_id)
    .bind(&input.observed_digest)
    .bind(&input.observed_state)
    .fetch_one(&mut *tx)
    .await
    .map_err(DeploymentError::storage)?;
    tx.commit().await.map_err(DeploymentError::storage)?;
    Ok(stored_assignment(row))
}

/// The stored assignment row shape (the query tuple).
type AssignmentRow = (String, String, i64, String, String, Option<String>, String);

fn stored_assignment(row: AssignmentRow) -> StoredAssignment {
    let (
        target_id,
        publication_id,
        wave,
        desired_ref,
        desired_digest,
        observed_digest,
        observed_state,
    ) = row;
    StoredAssignment {
        target_id,
        publication_id,
        wave,
        desired_ref,
        desired_digest,
        observed_digest,
        observed_state,
    }
}

/// One assignment's receipts, in the order they were filed
/// (`SIGNOFF-REPAIR.9.3.3.3`). Bound like the list: an assignment whose
/// publication is not the caller's tenant answers as an absent one does. An
/// assignment that exists and was never reported answers an empty history. The
/// rows are read by their own `tenant_id` too, copied from the publication when
/// each was written, so the history's read carries its own predicate.
///
/// ⚠️ A pair reported before `migrations/0115` has no row: that history was
/// already gone when the table was created.
pub async fn list_receipts(
    pool: &PgPool,
    tenant_id: &str,
    target_id: &str,
    publication_id: &str,
) -> Result<Vec<StoredReceipt>, DeploymentError> {
    let mut tx = pool.begin().await.map_err(DeploymentError::storage)?;
    let owned: Option<bool> = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM deployment_assignments a \
         JOIN policy_publications p ON p.publication_id = a.publication_id \
         WHERE a.target_id = $1 AND a.publication_id = $2 AND p.tenant_id = $3)",
    )
    .bind(target_id)
    .bind(publication_id)
    .bind(tenant_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(DeploymentError::storage)?;
    if !owned.unwrap_or(false) {
        return Err(DeploymentError::UnknownAssignment {
            target_id: target_id.to_string(),
            publication_id: publication_id.to_string(),
        });
    }
    let rows: Vec<(
        String,
        String,
        String,
        String,
        chrono::DateTime<chrono::Utc>,
    )> = sqlx::query_as(
        "SELECT receipt_id, reporter, observed_digest, observed_state, recorded_at \
             FROM deployment_receipts \
             WHERE target_id = $1 AND publication_id = $2 AND tenant_id = $3 \
             ORDER BY recorded_at, receipt_id",
    )
    .bind(target_id)
    .bind(publication_id)
    .bind(tenant_id)
    .fetch_all(&mut *tx)
    .await
    .map_err(DeploymentError::storage)?;
    tx.commit().await.map_err(DeploymentError::storage)?;
    Ok(rows
        .into_iter()
        .map(
            |(receipt_id, reporter, observed_digest, observed_state, recorded_at)| StoredReceipt {
                receipt_id,
                reporter,
                observed_digest,
                observed_state,
                recorded_at,
            },
        )
        .collect())
}

/// The assignments, newest first.
/// ⛔ `SIGNOFF-REPAIR.6.1.5.3`, and the SCOPE EXTENSION is deliberate rather than
/// accidental. `deployment_assignments` is not one of the nine lifecycle tables,
/// so this leaf was not opened over it — but DOC-0029 deferred it to `.6.1.5` BY
/// NAME, DOC-0071 discharged that deferral by ruling it **tenant-owned by its
/// PUBLICATION, not by its target**, and no child of `.6.1.5` names it. Leaving a
/// decided table with an unbound read and no owner is how a data model gets
/// decided by accident, which is the failure DOC-0029 exists to prevent.
///
/// ⚠️ The join IS the ownership, because the table carries no tenant column —
/// that was DOC-0071's decision, a target being site-wide by design.
pub async fn list_assignments(
    pool: &PgPool,
    tenant_id: &str,
) -> Result<Vec<StoredAssignment>, sqlx::Error> {
    let rows: Vec<AssignmentRow> = sqlx::query_as(
        "SELECT a.target_id, a.publication_id, a.wave, a.desired_ref, a.desired_digest, \
             a.observed_digest, a.observed_state \
             FROM deployment_assignments a \
             JOIN policy_publications p ON p.publication_id = a.publication_id \
             WHERE p.tenant_id = $1 ORDER BY a.target_id, a.publication_id",
    )
    .bind(tenant_id)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(stored_assignment).collect())
}
