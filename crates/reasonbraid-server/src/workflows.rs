//! The workflow-profile registry + the validation (PHASE-5.1.2, ADR-016):
//! a profile is VERSIONED CONFIGURATION over the thread aggregates — the
//! steps compose the EXISTING verbs, never a new capability. The eight
//! §13.1 built-ins ship as the registry's versioned entries; the custom
//! profiles validate against the same composition rules; the thread's
//! `workflow_profile` is a VALIDATED reference (the unknown profile is the
//! typed refusal — never a stored string).

use serde::Serialize;
use sqlx::PgPool;

/// The step vocabulary (the composition over the existing verbs): each
/// kind names an existing contribution/terminal/budget surface — nothing
/// else is expressible, by construction.
pub const STEP_KINDS: [&str; 12] = [
    "solicit",
    "blind_solicit",
    "synthesize",
    "critique",
    "revise",
    "adjudicate",
    "decide",
    "evidence_request",
    "assess",
    "vote",
    "approve",
    "retrospect",
];

/// The terminal steps: the profile's LAST step must be one of these (the
/// lifecycle's terminal verbs).
pub const TERMINAL_KINDS: [&str; 3] = ["decide", "approve", "vote"];

/// The resolved profile reference (the thread stores this).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ResolvedProfile {
    pub profile_id: String,
    pub version: i32,
    pub steps: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProfileError {
    UnknownProfile(String),
    EmptySteps,
    UnknownStep(String),
    NonTerminalLast(String),
    AdjudicateWithoutBlind,
}

impl std::fmt::Display for ProfileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownProfile(id) => {
                write!(f, "the workflow profile `{id}` is not registered")
            }
            Self::EmptySteps => write!(f, "the profile carries no steps"),
            Self::UnknownStep(kind) => {
                write!(f, "the step `{kind}` is outside the workflow vocabulary")
            }
            Self::NonTerminalLast(kind) => {
                write!(f, "the profile's last step `{kind}` is not a terminal step")
            }
            Self::AdjudicateWithoutBlind => write!(
                f,
                "the `adjudicate` step requires a preceding `blind_solicit`"
            ),
        }
    }
}

impl std::error::Error for ProfileError {}

/// The composition validation (the ADR-016 invariants): the known kinds
/// only, the terminal last, the adjudicate-after-blind rule. The
/// authorization/budget/lifecycle invariants live in the VERBS (the `.1.3`
/// execution re-runs them per step — the profile cannot express a
/// bypass because the vocabulary has no such step).
pub fn validate_steps(steps: &[String]) -> Result<(), ProfileError> {
    if steps.is_empty() {
        return Err(ProfileError::EmptySteps);
    }
    for step in steps {
        if !STEP_KINDS.contains(&step.as_str()) {
            return Err(ProfileError::UnknownStep(step.clone()));
        }
    }
    let last = steps.last().expect("non-empty");
    if !TERMINAL_KINDS.contains(&last.as_str()) {
        return Err(ProfileError::NonTerminalLast(last.clone()));
    }
    if steps.contains(&"adjudicate".to_owned())
        && !steps[..steps
            .iter()
            .position(|s| s == "adjudicate")
            .expect("present")]
            .contains(&"blind_solicit".to_owned())
    {
        return Err(ProfileError::AdjudicateWithoutBlind);
    }
    Ok(())
}

/// The default profile (the ADR's answer: a bare thread runs `quick_advice`).
pub const DEFAULT_PROFILE_ID: &str = "quick_advice";

/// Resolve the thread's profile reference: the latest version of the
/// registered profile; `None` → the default. The unknown profile is the
/// typed refusal.
pub async fn resolve(
    pool: &PgPool,
    profile_id: Option<&str>,
) -> Result<ResolvedProfile, ProfileError> {
    let profile_id = profile_id.unwrap_or(DEFAULT_PROFILE_ID);
    let row: Option<(i32, serde_json::Value)> = sqlx::query_as(
        "SELECT version, steps FROM workflow_profiles \
         WHERE profile_id = $1 ORDER BY version DESC LIMIT 1",
    )
    .bind(profile_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| ProfileError::UnknownProfile(profile_id.to_owned()))?;
    let (version, steps) =
        row.ok_or_else(|| ProfileError::UnknownProfile(profile_id.to_owned()))?;
    let steps: Vec<String> = serde_json::from_value(steps)
        .map_err(|_| ProfileError::UnknownProfile(profile_id.to_owned()))?;
    validate_steps(&steps)?;
    Ok(ResolvedProfile {
        profile_id: profile_id.to_owned(),
        version,
        steps,
    })
}

/// Register a custom profile (the operator's verb): the steps MUST pass
/// the composition validation — the registry never stores an invalid
/// profile.
pub async fn register(
    pool: &PgPool,
    profile_id: &str,
    steps: &[String],
) -> Result<ResolvedProfile, ProfileError> {
    validate_steps(steps)?;
    let version: i32 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(version), 0) + 1 FROM workflow_profiles WHERE profile_id = $1",
    )
    .bind(profile_id)
    .fetch_one(pool)
    .await
    .map_err(|_| ProfileError::UnknownProfile(profile_id.to_owned()))?;
    sqlx::query(
        "INSERT INTO workflow_profiles (profile_id, version, steps, built_in) \
         VALUES ($1, $2, $3, false)",
    )
    .bind(profile_id)
    .bind(version)
    .bind(serde_json::to_value(steps).expect("the steps serialize"))
    .execute(pool)
    .await
    .map_err(|_| ProfileError::UnknownProfile(profile_id.to_owned()))?;
    Ok(ResolvedProfile {
        profile_id: profile_id.to_owned(),
        version,
        steps: steps.to_vec(),
    })
}

/// The read surface: every registered profile's latest version.
pub async fn list(pool: &PgPool) -> Result<Vec<ResolvedProfile>, sqlx::Error> {
    let rows: Vec<(String, i32, serde_json::Value)> = sqlx::query_as(
        "SELECT p.profile_id, p.version, p.steps FROM workflow_profiles p \
         JOIN (SELECT profile_id, MAX(version) AS version FROM workflow_profiles GROUP BY profile_id) m \
         ON p.profile_id = m.profile_id AND p.version = m.version \
         ORDER BY p.profile_id",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .filter_map(|(profile_id, version, steps)| {
            serde_json::from_value::<Vec<String>>(steps)
                .ok()
                .map(|steps| ResolvedProfile {
                    profile_id,
                    version,
                    steps,
                })
        })
        .collect())
}
