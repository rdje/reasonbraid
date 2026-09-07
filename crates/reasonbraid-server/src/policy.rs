//! The typed policy schema + the versioned registry (`PHASE-6.1.2`, ADR-019):
//! the policy is a versioned, digest-pinned DOCUMENT — the stable clause ids,
//! the applicability + the explicit non-applicability, the exception schema,
//! and the OWNERSHIP metadata (the owning authority is a GRANT reference — the
//! label grants nothing; an unresolvable owning authority is invalid at
//! registration). The registry pattern mirrors the workflow profiles
//! (ADR-016): the unknown version is the typed refusal, never a stored guess.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;

/// The lifecycle vocabulary (the closed set).
pub const LIFECYCLES: [&str; 6] = [
    "draft",
    "active",
    "suspended",
    "superseded",
    "deprecated",
    "retracted",
];

/// One normative statement: a STABLE clause id + the statement text. The id
/// is the resolution's + the provenance's anchor — it never changes between
/// versions of the same policy.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClauseStatement {
    pub id: String,
    pub statement: String,
}

/// The policy-version submission (`.1.2`, ADR-019): the §15.1 fields. The
/// digest is the DECLARED ADR-011 digest over the canonical document bytes
/// (the `.4.2` corpus precedent — the consumer re-derives at use time); the
/// owning authority is a grant id that must EXIST.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyVersionInput {
    pub policy_id: String,
    pub version: String,
    pub digest: String,
    pub lifecycle: String,
    pub title: String,
    #[serde(default)]
    pub intent: String,
    #[serde(default)]
    pub rationale: String,
    #[serde(default)]
    pub domain: String,
    #[serde(default)]
    pub risk_class: String,
    pub owning_authority: String,
    pub clauses: Vec<ClauseStatement>,
    #[serde(default)]
    pub applicability: Vec<Value>,
    #[serde(default)]
    pub non_applicability: Vec<Value>,
    #[serde(default)]
    pub dependencies: Vec<Value>,
    #[serde(default)]
    pub conflicts: Vec<Value>,
    #[serde(default)]
    pub precedence_hints: Vec<Value>,
    #[serde(default)]
    pub exceptions: Vec<Value>,
    #[serde(default)]
    pub provenance: Vec<Value>,
}

/// The registered row, as stored.
#[derive(Debug, Clone, Serialize)]
pub struct RegisteredPolicy {
    pub policy_id: String,
    pub version: String,
    pub digest: String,
    pub lifecycle: String,
    pub title: String,
    pub intent: String,
    pub rationale: String,
    pub domain: String,
    pub risk_class: String,
    pub owning_authority: String,
    pub clauses: Vec<ClauseStatement>,
    pub applicability: Vec<Value>,
    pub non_applicability: Vec<Value>,
    pub dependencies: Vec<Value>,
    pub conflicts: Vec<Value>,
    pub precedence_hints: Vec<Value>,
    pub exceptions: Vec<Value>,
    pub provenance: Vec<Value>,
}

/// The typed refusal reasons — the caller maps them to an HTTP error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyError {
    MalformedDigest(String),
    MalformedVersion(String),
    UnknownLifecycle(String),
    DuplicateClause(String),
    GhostAuthority(String),
    Duplicate(String),
    EmptyClauses,
}

impl std::fmt::Display for PolicyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PolicyError::MalformedDigest(d) => {
                write!(f, "digest `{d}` is not the ADR-011 `sha256:<64 hex>` shape")
            }
            PolicyError::MalformedVersion(v) => {
                write!(
                    f,
                    "version `{v}` is not a semantic version (digits and dots, 1–3 parts)"
                )
            }
            PolicyError::UnknownLifecycle(l) => {
                write!(
                    f,
                    "lifecycle `{l}` is not in the vocabulary ({})",
                    LIFECYCLES.join(", ")
                )
            }
            PolicyError::DuplicateClause(id) => {
                write!(
                    f,
                    "clause id `{id}` repeats — the ids are STABLE anchors, never duplicated"
                )
            }
            PolicyError::GhostAuthority(grant) => {
                write!(f, "the owning authority `{grant}` is not an active grant — the label grants nothing")
            }
            PolicyError::Duplicate(what) => {
                write!(f, "{what} already exists — the record's identity is its content, register a new version")
            }
            PolicyError::EmptyClauses => {
                write!(f, "the policy carries at least one normative statement")
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

fn is_semver(version: &str) -> bool {
    let parts: Vec<&str> = version.split('.').collect();
    (1..=3).contains(&parts.len())
        && parts
            .iter()
            .all(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()))
}

/// Register one policy version (the typed, validated registry row).
pub async fn register(
    pool: &PgPool,
    input: &PolicyVersionInput,
) -> Result<RegisteredPolicy, PolicyError> {
    if !is_sha256_hex(&input.digest) {
        return Err(PolicyError::MalformedDigest(input.digest.clone()));
    }
    if !is_semver(&input.version) {
        return Err(PolicyError::MalformedVersion(input.version.clone()));
    }
    if !LIFECYCLES.contains(&input.lifecycle.as_str()) {
        return Err(PolicyError::UnknownLifecycle(input.lifecycle.clone()));
    }
    if input.clauses.is_empty() {
        return Err(PolicyError::EmptyClauses);
    }
    let mut seen = std::collections::BTreeSet::new();
    for clause in &input.clauses {
        if !seen.insert(clause.id.as_str()) {
            return Err(PolicyError::DuplicateClause(clause.id.clone()));
        }
    }
    // The ownership = the authority binding (ADR-019): the owning authority
    // must be an ACTIVE grant — a label-only policy fails at registration.
    let authority: Option<bool> = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM authority_grants WHERE grant_id = $1 AND status = 'active')",
    )
    .bind(&input.owning_authority)
    .fetch_one(pool)
    .await
    .map_err(|_| PolicyError::GhostAuthority(input.owning_authority.clone()))?;
    if !authority.unwrap_or(false) {
        return Err(PolicyError::GhostAuthority(input.owning_authority.clone()));
    }
    let inserted = sqlx::query(
        "INSERT INTO policy_versions \
         (policy_id, version, digest, lifecycle, title, intent, rationale, domain, risk_class, \
          owning_authority, clauses, applicability, non_applicability, dependencies, conflicts, \
          precedence_hints, exceptions, provenance) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)",
    )
    .bind(&input.policy_id)
    .bind(&input.version)
    .bind(&input.digest)
    .bind(&input.lifecycle)
    .bind(&input.title)
    .bind(&input.intent)
    .bind(&input.rationale)
    .bind(&input.domain)
    .bind(&input.risk_class)
    .bind(&input.owning_authority)
    .bind(serde_json::to_value(&input.clauses).expect("the clauses serialize"))
    .bind(serde_json::to_value(&input.applicability).expect("the applicability serializes"))
    .bind(serde_json::to_value(&input.non_applicability).expect("the non-applicability serializes"))
    .bind(serde_json::to_value(&input.dependencies).expect("the dependencies serialize"))
    .bind(serde_json::to_value(&input.conflicts).expect("the conflicts serialize"))
    .bind(serde_json::to_value(&input.precedence_hints).expect("the precedence serializes"))
    .bind(serde_json::to_value(&input.exceptions).expect("the exceptions serialize"))
    .bind(serde_json::to_value(&input.provenance).expect("the provenance serializes"))
    .execute(pool)
    .await;
    match inserted {
        Ok(_) => Ok(RegisteredPolicy {
            policy_id: input.policy_id.clone(),
            version: input.version.clone(),
            digest: input.digest.clone(),
            lifecycle: input.lifecycle.clone(),
            title: input.title.clone(),
            intent: input.intent.clone(),
            rationale: input.rationale.clone(),
            domain: input.domain.clone(),
            risk_class: input.risk_class.clone(),
            owning_authority: input.owning_authority.clone(),
            clauses: input.clauses.clone(),
            applicability: input.applicability.clone(),
            non_applicability: input.non_applicability.clone(),
            dependencies: input.dependencies.clone(),
            conflicts: input.conflicts.clone(),
            precedence_hints: input.precedence_hints.clone(),
            exceptions: input.exceptions.clone(),
            provenance: input.provenance.clone(),
        }),
        Err(_) => Err(PolicyError::Duplicate(format!(
            "policy `{}` version {}",
            input.policy_id, input.version
        ))),
    }
}

/// The stored policy row (a FromRow struct — the 18 columns exceed the
/// tuple impl's 16-column ceiling).
#[derive(Debug, sqlx::FromRow)]
struct PolicyRow {
    policy_id: String,
    version: String,
    digest: String,
    lifecycle: String,
    title: String,
    intent: String,
    rationale: String,
    domain: String,
    risk_class: String,
    owning_authority: String,
    clauses: Value,
    applicability: Value,
    non_applicability: Value,
    dependencies: Value,
    conflicts: Value,
    precedence_hints: Value,
    exceptions: Value,
    provenance: Value,
}

/// The registered policies, newest first.
pub async fn list(pool: &PgPool) -> Result<Vec<RegisteredPolicy>, sqlx::Error> {
    let rows: Vec<PolicyRow> = sqlx::query_as(
        "SELECT policy_id, version, digest, lifecycle, title, intent, rationale, domain, \
         risk_class, owning_authority, clauses, applicability, non_applicability, dependencies, \
         conflicts, precedence_hints, exceptions, provenance \
         FROM policy_versions ORDER BY created_at DESC",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| RegisteredPolicy {
            policy_id: row.policy_id,
            version: row.version,
            digest: row.digest,
            lifecycle: row.lifecycle,
            title: row.title,
            intent: row.intent,
            rationale: row.rationale,
            domain: row.domain,
            risk_class: row.risk_class,
            owning_authority: row.owning_authority,
            clauses: serde_json::from_value(row.clauses).expect("the clauses parse"),
            applicability: serde_json::from_value(row.applicability)
                .expect("the applicability parses"),
            non_applicability: serde_json::from_value(row.non_applicability)
                .expect("the non-applicability parses"),
            dependencies: serde_json::from_value(row.dependencies).expect("the dependencies parse"),
            conflicts: serde_json::from_value(row.conflicts).expect("the conflicts parse"),
            precedence_hints: serde_json::from_value(row.precedence_hints)
                .expect("the precedence parses"),
            exceptions: serde_json::from_value(row.exceptions).expect("the exceptions parse"),
            provenance: serde_json::from_value(row.provenance).expect("the provenance parses"),
        })
        .collect())
}
