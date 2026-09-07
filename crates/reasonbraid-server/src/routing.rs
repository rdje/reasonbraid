//! The rule-based routing policy (`PHASE-5.5.2`, ADR-031): the deterministic
//! class→arm table (the §13.8 rows). The case class is a SUBMITTED input
//! (never a derived judgment — the service never classifies); the resolution
//! is a lookup (the same class always resolves to the same arm); every
//! resolution rides the append-only audit table; the human authority
//! outranks the rule (the create boundary applies the policy ONLY when no
//! explicit profile is named).

use serde::Serialize;
use sqlx::PgPool;

/// The §13.8 case-class vocabulary (the closed set).
pub const CASE_CLASSES: [&str; 7] = [
    "simple",
    "factual",
    "uncertain",
    "design_policy",
    "governed",
    "correlated",
    "diminishing",
];

/// The deterministic resolution: the class → the arm + the rule id.
#[derive(Debug, Clone, Serialize)]
pub struct ResolvedRoute {
    pub case_class: String,
    pub arm: String,
    pub rule_id: String,
}

/// The typed refusal reasons.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RoutingError {
    /// The class is not in the closed vocabulary.
    UnknownClass(String),
    /// The stored rule's arm is not a registered profile (a corrupt row —
    /// fail closed, never route to a phantom arm).
    PhantomArm(String),
}

impl std::fmt::Display for RoutingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RoutingError::UnknownClass(c) => {
                write!(
                    f,
                    "case class `{c}` is not in the routing vocabulary ({})",
                    CASE_CLASSES.join(", ")
                )
            }
            RoutingError::PhantomArm(a) => {
                write!(
                    f,
                    "the policy's arm `{a}` is not a registered profile — fail closed"
                )
            }
        }
    }
}

/// Resolve one class through the rule table (the deterministic lookup). The
/// arm must be a REGISTERED profile (a phantom arm is a corrupt-row failure,
/// never a route).
pub async fn resolve(pool: &PgPool, case_class: &str) -> Result<ResolvedRoute, RoutingError> {
    if !CASE_CLASSES.contains(&case_class) {
        return Err(RoutingError::UnknownClass(case_class.to_string()));
    }
    let row: Option<(String, String)> =
        sqlx::query_as("SELECT rule_id, arm FROM routing_rules WHERE case_class = $1")
            .bind(case_class)
            .fetch_optional(pool)
            .await
            .map_err(|_| RoutingError::UnknownClass(case_class.to_string()))?;
    let Some((rule_id, arm)) = row else {
        return Err(RoutingError::UnknownClass(case_class.to_string()));
    };
    let registered: Option<bool> =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM workflow_profiles WHERE profile_id = $1)")
            .bind(&arm)
            .fetch_one(pool)
            .await
            .map_err(|_| RoutingError::PhantomArm(arm.clone()))?;
    if !registered.unwrap_or(false) {
        return Err(RoutingError::PhantomArm(arm));
    }
    Ok(ResolvedRoute {
        case_class: case_class.to_string(),
        arm,
        rule_id,
    })
}

/// Append the resolution audit row (the surface names where the resolution
/// happened: the `resolve` verb or the create boundary).
pub async fn record_resolution(
    pool: &PgPool,
    route: &ResolvedRoute,
    caller: &str,
    surface: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO routing_resolutions (case_class, arm, rule_id, caller, surface) \
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(&route.case_class)
    .bind(&route.arm)
    .bind(&route.rule_id)
    .bind(caller)
    .bind(surface)
    .execute(pool)
    .await?;
    Ok(())
}

/// The rule rows (the deterministic table, as stored).
pub async fn list_rules(pool: &PgPool) -> Result<Vec<serde_json::Value>, sqlx::Error> {
    let rows: Vec<serde_json::Value> = sqlx::query_scalar(
        "SELECT jsonb_build_object('rule_id', rule_id, 'case_class', case_class, \
         'arm', arm, 'built_in', built_in) FROM routing_rules ORDER BY rule_id",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// The resolution audit rows, newest first.
pub async fn list_resolutions(pool: &PgPool) -> Result<Vec<serde_json::Value>, sqlx::Error> {
    let rows: Vec<serde_json::Value> = sqlx::query_scalar(
        "SELECT jsonb_build_object('case_class', case_class, 'arm', arm, 'rule_id', rule_id, \
         'caller', caller, 'surface', surface) \
         FROM routing_resolutions ORDER BY resolved_at DESC",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

// ── The shadow recommendations (`.5.3`, ADR-031) ───────────────────────────────────

/// The recommendation submission (`.5.3`): the class → the arm + the evidence
/// reference (the `.4` trial/gate id it rests on). The arm must be an
/// EXISTING registered profile — the recommendation can re-order what exists,
/// never raise authority/spend/access/side-effect scope.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecommendationSubmission {
    pub recommendation_id: String,
    pub case_class: String,
    pub arm: String,
    pub evidence_ref: String,
}

impl RoutingError {
    fn unknown_evidence(reference: &str) -> Self {
        RoutingError::UnknownClass(format!(
            "evidence reference `{reference}` (the recommendation names a `.4` trial or gate)"
        ))
    }
}

/// Record one shadow recommendation. It is NEVER applied — the create
/// boundary keeps resolving the rule table; this record is the evidence a
/// future policy gate weighs.
pub async fn record_recommendation(
    pool: &PgPool,
    submission: &RecommendationSubmission,
) -> Result<serde_json::Value, RoutingError> {
    if !CASE_CLASSES.contains(&submission.case_class.as_str()) {
        return Err(RoutingError::UnknownClass(submission.case_class.clone()));
    }
    // The never-a-raise constraint: the arm must be a REGISTERED profile.
    let registered: Option<bool> =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM workflow_profiles WHERE profile_id = $1)")
            .bind(&submission.arm)
            .fetch_one(pool)
            .await
            .map_err(|_| RoutingError::PhantomArm(submission.arm.clone()))?;
    if !registered.unwrap_or(false) {
        return Err(RoutingError::PhantomArm(submission.arm.clone()));
    }
    // The evidence: a `.4` trial or gate the recommendation rests on.
    let evidence: Option<bool> = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM evaluation_trials WHERE trial_id = $1) \
         OR EXISTS (SELECT 1 FROM evaluation_gates WHERE gate_id = $1)",
    )
    .bind(&submission.evidence_ref)
    .fetch_one(pool)
    .await
    .map_err(|_| RoutingError::unknown_evidence(&submission.evidence_ref))?;
    if !evidence.unwrap_or(false) {
        return Err(RoutingError::unknown_evidence(&submission.evidence_ref));
    }
    let inserted = sqlx::query(
        "INSERT INTO routing_recommendations \
         (recommendation_id, case_class, arm, evidence_ref) VALUES ($1, $2, $3, $4)",
    )
    .bind(&submission.recommendation_id)
    .bind(&submission.case_class)
    .bind(&submission.arm)
    .bind(&submission.evidence_ref)
    .execute(pool)
    .await;
    match inserted {
        Ok(_) => Ok(serde_json::json!({
            "recommendation_id": submission.recommendation_id,
            "case_class": submission.case_class,
            "arm": submission.arm,
            "evidence_ref": submission.evidence_ref,
            "applied": false,
        })),
        Err(_) => Err(RoutingError::UnknownClass(format!(
            "recommendation `{}` (the id already exists — record a new one)",
            submission.recommendation_id
        ))),
    }
}

/// The recommendations, newest first.
pub async fn list_recommendations(pool: &PgPool) -> Result<Vec<serde_json::Value>, sqlx::Error> {
    let rows: Vec<serde_json::Value> = sqlx::query_scalar(
        "SELECT jsonb_build_object('recommendation_id', recommendation_id, \
         'case_class', case_class, 'arm', arm, 'evidence_ref', evidence_ref, \
         'applied', false) FROM routing_recommendations ORDER BY recorded_at DESC",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}
