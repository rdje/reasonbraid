//! The drift + the corrections + the outcomes (`PHASE-6.5.3`, ADR-021):
//! the drift names the §15.10 category over the desired/observed pair; the
//! correction is its OWN row with the §4.7 operation + the AUTHORITY PROOF
//! (the grant re-check — the reversal is fast, the authority is NOT
//! universally lower) — the retraction NEVER deletes the original; the
//! outcome links the publication to what happened after (§15.11).

use reasonbraid_core::GrantSubject;
use serde::Deserialize;
use serde_json::Value;
use sqlx::PgPool;

/// The §15.10 drift-category vocabulary.
pub const DRIFT_CATEGORIES: [&str; 6] = [
    "expected_override",
    "pending_rollout",
    "unauthorized_modification",
    "unsupported_target",
    "unverifiable_load",
    "stale_agent_incarnation",
];

/// The §4.7 correction-operation vocabulary.
pub const CORRECTION_OPERATIONS: [&str; 4] = ["suspension", "supersession", "retraction", "waiver"];

/// The §15.11 outcome-kind vocabulary.
pub const OUTCOME_KINDS: [&str; 6] = [
    "observation",
    "measurement",
    "incident",
    "complaint",
    "reversal",
    "unintended_effect",
];

/// The drift submission (`.5.3`): the assignment's pair + the category.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DriftInput {
    pub drift_id: String,
    pub target_id: String,
    pub publication_id: String,
    pub category: String,
    pub desired_digest: String,
    pub observed_digest: Option<String>,
}

/// The correction submission (`.5.3`): the §4.7 operation + the authority
/// proof + the operation-specific fields.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CorrectionInput {
    pub correction_id: String,
    pub publication_id: String,
    pub operation: String,
    pub authority_grant: String,
    #[serde(default)]
    pub supersedes: Option<String>,
    #[serde(default)]
    pub expires_at: Option<String>,
    pub reason: String,
    #[serde(default)]
    pub evidence: Vec<Value>,
    #[serde(default)]
    pub remediation: Option<String>,
}

/// The outcome submission (`.5.3`): the link + the kind + the trigger.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OutcomeInput {
    pub outcome_id: String,
    pub publication_id: String,
    pub kind: String,
    #[serde(default)]
    pub review_trigger: Option<String>,
    pub note: String,
}

/// The typed refusal reasons.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CorrectionError {
    UnknownCategory(String),
    UnknownOperation(String),
    UnknownKind(String),
    UnknownPublication(String),
    UnknownAssignment(String),
    GhostAuthority(String),
    MissingExpiry,
    MissingSupersedes,
    UnknownSupersedes(String),
    Duplicate(String),
}

impl std::fmt::Display for CorrectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CorrectionError::UnknownCategory(c) => {
                write!(
                    f,
                    "category `{c}` is not in the drift vocabulary ({})",
                    DRIFT_CATEGORIES.join(", ")
                )
            }
            CorrectionError::UnknownOperation(o) => {
                write!(
                    f,
                    "operation `{o}` is not in the correction vocabulary ({})",
                    CORRECTION_OPERATIONS.join(", ")
                )
            }
            CorrectionError::UnknownKind(k) => {
                write!(
                    f,
                    "outcome kind `{k}` is not in the vocabulary ({})",
                    OUTCOME_KINDS.join(", ")
                )
            }
            CorrectionError::UnknownPublication(p) => write!(f, "publication `{p}` does not exist"),
            CorrectionError::UnknownAssignment(a) => write!(f, "assignment `{a}` does not exist"),
            CorrectionError::GhostAuthority(g) => {
                write!(
                    f,
                    "the authority grant `{g}` is not an active, unexpired grant"
                )
            }
            CorrectionError::MissingExpiry => {
                write!(
                    f,
                    "the suspension and the waiver require the `expires_at` (the expiring rule)"
                )
            }
            CorrectionError::MissingSupersedes => {
                write!(
                    f,
                    "the supersession requires the `supersedes` reference (the linked old/new)"
                )
            }
            CorrectionError::UnknownSupersedes(p) => {
                write!(f, "the superseded publication `{p}` does not exist")
            }
            CorrectionError::Duplicate(what) => {
                write!(
                    f,
                    "{what} already exists — the record's identity is its content"
                )
            }
        }
    }
}

/// The publication must exist — and the check RETURNS ITS TENANT, which is the
/// tenant every record about it carries (`SIGNOFF-REPAIR.6.1.5.2`).
///
/// ⛔ The drift, the correction and the outcome are all observations ABOUT a
/// publication, so their owner is the publication's owner and not the caller: a
/// drift row about Alice's publication stamped with Mallory's tenant would be
/// invisible to Alice while she is the only party it concerns.
///
/// ⚠️ `Option<String>`, not `String`. A publication staged before
/// `migrations/0073` carries no tenant, and a record about it inherits that
/// absence rather than being given an owner it never had.
async fn publication_tenant(
    pool: &PgPool,
    publication_id: &str,
    tenant_id: &str,
) -> Result<Option<String>, CorrectionError> {
    let row: Option<(String, Option<String>)> = sqlx::query_as(
        "SELECT publication_id, tenant_id FROM policy_publications WHERE publication_id = $1",
    )
    .bind(publication_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| CorrectionError::UnknownPublication(publication_id.to_string()))?;
    let Some((_, owner)) = row else {
        return Err(CorrectionError::UnknownPublication(
            publication_id.to_string(),
        ));
    };
    // ⛔ `SIGNOFF-REPAIR.6.1.5.2.1`: the publication must be the CALLER's.
    // `.6.1.5.2` labelled these rows correctly and gated nothing, so a foreign
    // tenant's drift, waiver or incident report landed in the owner's own trail,
    // authorized by nothing. ⚠️ A foreign publication answers exactly as an
    // ABSENT one, and a publication with no owner is actionable by nobody — the
    // reasoning is recorded once, at `publications::owned_by`.
    if owner.as_deref() != Some(tenant_id) {
        return Err(CorrectionError::UnknownPublication(
            publication_id.to_string(),
        ));
    }
    Ok(owner)
}

/// The cited authority must be one the CALLER HOLDS, not merely one that
/// exists (`SIGNOFF-REPAIR.9.3.1`).
///
/// ⛔ This function used to take the grant id and nothing else — no caller, no
/// action, no scope — so any enrolled principal who could name any active
/// grant recorded a suspension, a retraction or a waiver in its name. The
/// grant id is derivable (`grt_<principal_id>`), so naming one needs nothing
/// but another principal's id. Measured before the repair: a principal in one
/// tenant recorded a retraction under another tenant's grant.
///
/// ⭐ The error type was already called `GhostAuthority`: the concept was
/// present and only the check was missing.
async fn authority_holds(
    pool: &PgPool,
    grant_id: &str,
    principal: &GrantSubject,
) -> Result<(), CorrectionError> {
    let held = crate::authority::grant_held_by(pool, grant_id, principal)
        .await
        .map_err(|_| CorrectionError::GhostAuthority(grant_id.to_string()))?;
    if !held {
        return Err(CorrectionError::GhostAuthority(grant_id.to_string()));
    }
    Ok(())
}

/// Record one drift observation (the categorized pair).
pub async fn record_drift(
    pool: &PgPool,
    tenant_id: &str,
    input: &DriftInput,
) -> Result<(), CorrectionError> {
    if !DRIFT_CATEGORIES.contains(&input.category.as_str()) {
        return Err(CorrectionError::UnknownCategory(input.category.clone()));
    }
    // ⛔ ORDER CHANGED BY `SIGNOFF-REPAIR.6.1.5.3`, and the order is the finding.
    // The assignment probe used to answer FIRST, so a foreign caller naming
    // another tenant's publication got `UnknownAssignment` when the pair was not
    // deployed and `UnknownPublication` when it was — two refusals that differ,
    // which is an existence oracle over another tenant's rollout state. The
    // ownership check now answers first, so a foreign publication gives ONE
    // answer whatever its deployment.
    let tenant_id = publication_tenant(pool, &input.publication_id, tenant_id).await?;
    let assignment: Option<bool> = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM deployment_assignments \
         WHERE target_id = $1 AND publication_id = $2)",
    )
    .bind(&input.target_id)
    .bind(&input.publication_id)
    .fetch_one(pool)
    .await
    .map_err(|_| CorrectionError::UnknownAssignment(input.target_id.clone()))?;
    if !assignment.unwrap_or(false) {
        return Err(CorrectionError::UnknownAssignment(format!(
            "({}, {})",
            input.target_id, input.publication_id
        )));
    }
    let inserted = sqlx::query(
        "INSERT INTO policy_drift \
         (drift_id, target_id, publication_id, category, desired_digest, observed_digest, \
          tenant_id) VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(&input.drift_id)
    .bind(&input.target_id)
    .bind(&input.publication_id)
    .bind(&input.category)
    .bind(&input.desired_digest)
    .bind(&input.observed_digest)
    .bind(&tenant_id)
    .execute(pool)
    .await;
    match inserted {
        Ok(_) => Ok(()),
        Err(_) => Err(CorrectionError::Duplicate(format!(
            "drift `{}`",
            input.drift_id
        ))),
    }
}

/// Record one correction (the §4.7 operation + the authority proof). The
/// retraction NEVER deletes the original — the correction is a NEW row.
pub async fn record_correction(
    pool: &PgPool,
    principal: &GrantSubject,
    caller_tenant: &str,
    input: &CorrectionInput,
) -> Result<Value, CorrectionError> {
    if !CORRECTION_OPERATIONS.contains(&input.operation.as_str()) {
        return Err(CorrectionError::UnknownOperation(input.operation.clone()));
    }
    let tenant_id = publication_tenant(pool, &input.publication_id, caller_tenant).await?;
    authority_holds(pool, &input.authority_grant, principal).await?;
    match input.operation.as_str() {
        "suspension" | "waiver" => {
            if input.expires_at.is_none() {
                return Err(CorrectionError::MissingExpiry);
            }
        }
        "supersession" => {
            let Some(supersedes) = input.supersedes.as_deref() else {
                return Err(CorrectionError::MissingSupersedes);
            };
            // The superseded publication must exist; its tenant is not the
            // correction's — the correction belongs to the publication it
            // corrects, which is `input.publication_id` above.
            // The superseded publication must be the caller's too: one tenant
            // does not supersede another's.
            publication_tenant(pool, supersedes, caller_tenant).await?;
        }
        _ => {}
    }
    // The expiry parses to the typed DateTime (a raw string bind would
    // fail the TIMESTAMPTZ coercion — the mislabeled "duplicate" trap the
    // first live pass caught).
    let expires_at: Option<chrono::DateTime<chrono::Utc>> = input
        .expires_at
        .as_deref()
        .map(|raw| {
            chrono::DateTime::parse_from_rfc3339(raw)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .map_err(|_| CorrectionError::MissingExpiry)
        })
        .transpose()?;
    let inserted = sqlx::query(
        "INSERT INTO policy_corrections \
         (correction_id, publication_id, operation, authority_grant, supersedes, expires_at, \
          reason, evidence, remediation, tenant_id) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
    )
    .bind(&input.correction_id)
    .bind(&input.publication_id)
    .bind(&input.operation)
    .bind(&input.authority_grant)
    .bind(&input.supersedes)
    .bind(expires_at)
    .bind(&input.reason)
    .bind(serde_json::to_value(&input.evidence).expect("the evidence serializes"))
    .bind(&input.remediation)
    .bind(&tenant_id)
    .execute(pool)
    .await;
    match inserted {
        Ok(_) => Ok(serde_json::json!({
            "correction_id": input.correction_id,
            "publication_id": input.publication_id,
            "operation": input.operation,
            "authority_grant": input.authority_grant,
            "supersedes": input.supersedes,
            "expires_at": input.expires_at,
            "reason": input.reason,
            "evidence": input.evidence,
            "remediation": input.remediation,
        })),
        Err(_) => Err(CorrectionError::Duplicate(format!(
            "correction `{}`",
            input.correction_id
        ))),
    }
}

/// Record one outcome (the §15.11 link).
pub async fn record_outcome(
    pool: &PgPool,
    tenant_id: &str,
    input: &OutcomeInput,
) -> Result<(), CorrectionError> {
    if !OUTCOME_KINDS.contains(&input.kind.as_str()) {
        return Err(CorrectionError::UnknownKind(input.kind.clone()));
    }
    // `.6`: the review trigger rides the §15.11 vocabulary (the schedule's
    // seven names) — the `.6` back-fill tightens the `.5.3` free string.
    if let Some(trigger) = input.review_trigger.as_deref() {
        if !crate::reviews::REVIEW_TRIGGERS.contains(&trigger) {
            return Err(CorrectionError::UnknownKind(format!(
                "review trigger `{trigger}` (the vocabulary: {})",
                crate::reviews::REVIEW_TRIGGERS.join(", ")
            )));
        }
    }
    let tenant_id = publication_tenant(pool, &input.publication_id, tenant_id).await?;
    let inserted = sqlx::query(
        "INSERT INTO policy_outcomes \
         (outcome_id, publication_id, kind, review_trigger, note, tenant_id) \
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(&input.outcome_id)
    .bind(&input.publication_id)
    .bind(&input.kind)
    .bind(&input.review_trigger)
    .bind(&input.note)
    .bind(&tenant_id)
    .execute(pool)
    .await;
    match inserted {
        Ok(_) => Ok(()),
        Err(_) => Err(CorrectionError::Duplicate(format!(
            "outcome `{}`",
            input.outcome_id
        ))),
    }
}

/// The drift records, newest first.
/// ⛔ `SIGNOFF-REPAIR.6.1.5.3`: bound to the caller's tenant. Until now this
/// returned every tenant's rows to any enrolled principal. ⚠️ The predicate
/// never matches NULL, so a row `migrations/0073` could not attribute is read
/// by NOBODY — `.7.1.2.2`'s disposition, and the reason the backfill's coverage
/// is published as a measured count rather than assumed complete.
pub async fn list_drift(pool: &PgPool, tenant_id: &str) -> Result<Vec<Value>, sqlx::Error> {
    let rows: Vec<Value> = sqlx::query_scalar(
        "SELECT jsonb_build_object('drift_id', drift_id, 'target_id', target_id, \
         'publication_id', publication_id, 'category', category, 'desired_digest', desired_digest, \
         'observed_digest', observed_digest) \
         FROM policy_drift WHERE tenant_id = $1 ORDER BY created_at DESC",
    )
    .bind(tenant_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// The corrections, newest first.
/// ⛔ `SIGNOFF-REPAIR.6.1.5.3`: bound to the caller's tenant. Until now this
/// returned every tenant's rows to any enrolled principal. ⚠️ The predicate
/// never matches NULL, so a row `migrations/0073` could not attribute is read
/// by NOBODY — `.7.1.2.2`'s disposition, and the reason the backfill's coverage
/// is published as a measured count rather than assumed complete.
pub async fn list_corrections(pool: &PgPool, tenant_id: &str) -> Result<Vec<Value>, sqlx::Error> {
    let rows: Vec<Value> = sqlx::query_scalar(
        "SELECT jsonb_build_object('correction_id', correction_id, 'publication_id', publication_id, \
         'operation', operation, 'authority_grant', authority_grant, 'supersedes', supersedes, \
         'expires_at', expires_at, 'reason', reason, 'evidence', evidence, 'remediation', remediation) \
         FROM policy_corrections WHERE tenant_id = $1 ORDER BY created_at DESC",
    )
    .bind(tenant_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// The outcomes, newest first.
/// ⛔ `SIGNOFF-REPAIR.6.1.5.3`: bound to the caller's tenant. Until now this
/// returned every tenant's rows to any enrolled principal. ⚠️ The predicate
/// never matches NULL, so a row `migrations/0073` could not attribute is read
/// by NOBODY — `.7.1.2.2`'s disposition, and the reason the backfill's coverage
/// is published as a measured count rather than assumed complete.
pub async fn list_outcomes(pool: &PgPool, tenant_id: &str) -> Result<Vec<Value>, sqlx::Error> {
    let rows: Vec<Value> = sqlx::query_scalar(
        "SELECT jsonb_build_object('outcome_id', outcome_id, 'publication_id', publication_id, \
         'kind', kind, 'review_trigger', review_trigger, 'note', note) \
         FROM policy_outcomes WHERE tenant_id = $1 ORDER BY created_at DESC",
    )
    .bind(tenant_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}
