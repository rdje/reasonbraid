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
    // must be a LIVE grant — a label-only policy fails at registration.
    //
    // ⭐ `SIGNOFF-REPAIR.9.3.1` settled the asymmetry `.11.9.1.2.2` measured:
    // this site admitted on `status = 'active'` alone while `resolve` below,
    // in the same module, also required the grant to be unexpired — so a
    // lapsed grant registered a policy version the resolver would then refuse.
    // Both now ask `authority::grant_is_live`, which is also the first time
    // either consulted `valid_from`.
    //
    // ⛔ What this does NOT decide: whether the owning authority must be a
    // grant the REGISTRAR holds. A policy may legitimately be owned by an
    // authority other than the caller's, so that binding is a semantic
    // question and stays `SIGNOFF-REPAIR.9.1`'s.
    if !crate::authority::grant_is_live(pool, &input.owning_authority)
        .await
        .map_err(|_| PolicyError::GhostAuthority(input.owning_authority.clone()))?
    {
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

// ── The layering + the precedence (`.1.3`, ADR-019) ────────────────────────────────

/// One policy reference in the resolution request (the id + the version).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyRef {
    pub policy_id: String,
    pub version: String,
}

/// The resolution target: the layer + the target the clauses must apply to
/// (the §15.3 applicability filter's inputs).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResolutionTarget {
    pub layer: String,
    pub target: String,
}

/// The resolution request (`.1.3`): the collection to resolve + the target +
/// the requested waiver/exception references.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResolutionRequest {
    pub policies: Vec<PolicyRef>,
    pub target: ResolutionTarget,
    #[serde(default)]
    pub exception_grants: Vec<String>,
}

/// One resolved clause: the winning policy/version, the clause, and the
/// explanation trail (the path the seven steps took to it).
#[derive(Debug, Clone, Serialize)]
pub struct ResolvedClause {
    pub policy_id: String,
    pub version: String,
    pub clause_id: String,
    pub statement: String,
    pub path: Vec<String>,
}

/// The resolution result: the resolved clauses + the explanation tree. The
/// conflicts list is EMPTY on success — an unresolved binding conflict is
/// the typed refusal (the fail-closed step), never a silent pick.
#[derive(Debug, Clone, Serialize)]
pub struct Resolution {
    pub target: ResolutionTarget,
    pub resolved: Vec<ResolvedClause>,
    pub explanation: Vec<String>,
    pub conflicts: Vec<String>,
}

impl PolicyError {
    fn unknown_ref(policy_id: &str, version: &str) -> Self {
        PolicyError::Duplicate(format!(
            "policy `{policy_id}` version {version} is not registered"
        ))
    }
    fn expired_authority(policy_id: &str, grant: &str) -> Self {
        PolicyError::GhostAuthority(format!(
            "the owning authority `{grant}` of `{policy_id}` is not an active, unexpired grant"
        ))
    }
    fn missing_dependency(policy_id: &str, dependency: &str) -> Self {
        PolicyError::Duplicate(format!(
            "`{policy_id}` depends on `{dependency}`, which is not in the resolved set"
        ))
    }
    fn conflicting_pair(a: &str, b: &str) -> Self {
        PolicyError::Duplicate(format!(
            "the precedence hints conflict: `{a}` over `{b}` AND `{b}` over `{a}`"
        ))
    }
    fn unknown_waiver(waiver: &str) -> Self {
        PolicyError::Duplicate(format!(
            "the requested exception `{waiver}` is not allowed by any policy's exception schema"
        ))
    }
    fn binding_conflict(clause_id: &str) -> Self {
        PolicyError::Duplicate(format!(
            "the unresolved binding conflict: clause `{clause_id}` is carried by multiple \
             applicable policies and no precedence settles it (fail-closed)"
        ))
    }
}

/// The stored resolution row (the id, the version, the lifecycle, the digest,
/// the owning authority, the clauses, the applicability, the
/// non-applicability, the dependencies, the conflicts, the precedence hints,
/// the exceptions).
#[derive(Debug, sqlx::FromRow)]
struct ResolutionRow {
    policy_id: String,
    version: String,
    lifecycle: String,
    #[allow(dead_code)]
    // the digest rides the registry rows; the resolution reads the facts it needs
    digest: String,
    owning_authority: String,
    clauses: Value,
    applicability: Value,
    non_applicability: Value,
    dependencies: Value,
    conflicts: Value,
    precedence_hints: Value,
    exceptions: Value,
}

/// The seven-step resolution (ADR-019's §15.3 pipeline, fail-closed).
pub async fn resolve(
    pool: &PgPool,
    request: &ResolutionRequest,
) -> Result<Resolution, PolicyError> {
    if request.policies.is_empty() {
        return Err(PolicyError::EmptyClauses);
    }
    let mut explanation = Vec::new();

    // Load the named policies; a ghost reference is the typed refusal.
    let mut loaded = Vec::new();
    for reference in &request.policies {
        let row: Option<ResolutionRow> = sqlx::query_as(
            "SELECT policy_id, version, lifecycle, digest, owning_authority, clauses, \
             applicability, non_applicability, dependencies, conflicts, precedence_hints, \
             exceptions \
             FROM policy_versions WHERE policy_id = $1 AND version = $2",
        )
        .bind(&reference.policy_id)
        .bind(&reference.version)
        .fetch_optional(pool)
        .await
        .map_err(|_| PolicyError::unknown_ref(&reference.policy_id, &reference.version))?;
        let row =
            row.ok_or_else(|| PolicyError::unknown_ref(&reference.policy_id, &reference.version))?;
        loaded.push(row);
    }
    let ids: Vec<&str> = loaded.iter().map(|r| r.policy_id.as_str()).collect();
    explanation.push(format!(
        "loaded {} policies: {}",
        loaded.len(),
        ids.join(", ")
    ));

    // Step 1: the issuer authority — each owning grant must be ACTIVE and
    // unexpired (the label grants nothing; an expired grant grants nothing).
    for row in &loaded {
        let valid = crate::authority::grant_is_live(pool, &row.owning_authority)
            .await
            .map_err(|_| PolicyError::expired_authority(&row.policy_id, &row.owning_authority))?;
        if !valid {
            return Err(PolicyError::expired_authority(
                &row.policy_id,
                &row.owning_authority,
            ));
        }
    }
    explanation.push("step 1: every owning authority is an active, unexpired grant".to_string());

    // Step 2: the applicability filter — a policy applies when SOME
    // applicability selector matches the target AND NO non-applicability
    // selector matches. The empty applicability matches everything (the
    // baseline policy).
    let selector_matches = |selector: &Value| -> bool {
        let layer = selector
            .get("layer")
            .and_then(|v| v.as_str())
            .unwrap_or("*");
        let target = selector
            .get("target")
            .and_then(|v| v.as_str())
            .unwrap_or("*");
        (layer == "*" || layer == request.target.layer)
            && (target == "*" || target == request.target.target)
    };
    let selectors: Vec<Vec<Value>> = loaded
        .iter()
        .map(|row| {
            serde_json::from_value::<Vec<Value>>(row.applicability.clone())
                .expect("the applicability parses")
        })
        .collect();
    let non_selectors: Vec<Vec<Value>> = loaded
        .iter()
        .map(|row| {
            serde_json::from_value::<Vec<Value>>(row.non_applicability.clone())
                .expect("the non-applicability parses")
        })
        .collect();
    let applicable: Vec<bool> = loaded
        .iter()
        .enumerate()
        .map(|(i, row)| {
            let positive = selectors[i].is_empty() || selectors[i].iter().any(selector_matches);
            let negative = non_selectors[i].iter().any(selector_matches);
            positive && !negative && row.lifecycle != "suspended" && row.lifecycle != "retracted"
        })
        .collect();
    explanation.push(format!(
        "step 2: the applicability filter left {} of {} policies applying",
        applicable.iter().filter(|a| **a).count(),
        loaded.len()
    ));

    // Step 3: the dependencies + the explicit conflicts — every dependency
    // must be IN the set; every explicit conflict must be OUT.
    for (i, row) in loaded.iter().enumerate() {
        let dependencies: Vec<Value> =
            serde_json::from_value(row.dependencies.clone()).expect("the dependencies parse");
        for dependency in &dependencies {
            let policy = dependency
                .get("policy")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            if !ids.contains(&policy) {
                return Err(PolicyError::missing_dependency(&row.policy_id, policy));
            }
        }
        let conflicts: Vec<Value> =
            serde_json::from_value(row.conflicts.clone()).expect("the conflicts parse");
        for conflict in &conflicts {
            let policy = conflict
                .get("policy")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            if ids.contains(&policy) {
                return Err(PolicyError::binding_conflict(&format!(
                    "{} conflicts with {}",
                    row.policy_id, policy
                )));
            }
        }
        let _ = i;
    }
    explanation.push(
        "step 3: every dependency is in the set; no explicit conflict is present".to_string(),
    );

    // Step 4: the precedence hints — the `over` edges; a cycle is the
    // refusal (the charter precedence must be a DAG).
    let mut precedence: std::collections::BTreeMap<String, Vec<String>> =
        std::collections::BTreeMap::new();
    for row in &loaded {
        let hints: Vec<Value> =
            serde_json::from_value(row.precedence_hints.clone()).expect("the hints parse");
        for hint in &hints {
            let over = hint
                .get("over")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            if over == row.policy_id {
                continue; // a self-hint is a no-op, not a conflict
            }
            precedence
                .entry(row.policy_id.clone())
                .or_default()
                .push(over.to_string());
            if precedence
                .get(over)
                .map(|list| list.contains(&row.policy_id))
                .unwrap_or(false)
            {
                return Err(PolicyError::conflicting_pair(&row.policy_id, over));
            }
        }
    }
    // The transitive winner for a pair: A wins over B when A can reach B
    // through the `over` edges.
    let wins_over = |a: &str, b: &str| -> bool {
        let mut stack = vec![a.to_string()];
        let mut seen = std::collections::BTreeSet::new();
        while let Some(current) = stack.pop() {
            if current == b {
                return true;
            }
            if !seen.insert(current.clone()) {
                continue;
            }
            if let Some(next) = precedence.get(&current) {
                stack.extend(next.iter().cloned());
            }
        }
        false
    };
    explanation.push(format!(
        "step 4: the precedence edges form a DAG ({} edges)",
        precedence.values().map(|v| v.len()).sum::<usize>()
    ));

    // Step 5: the exceptions/waivers — each requested exception must be
    // allowed by SOME policy's exception schema.
    for waiver in &request.exception_grants {
        let allowed = loaded.iter().any(|row| {
            let exceptions: Vec<Value> =
                serde_json::from_value(row.exceptions.clone()).expect("the exceptions parse");
            exceptions
                .iter()
                .any(|e| e.get("waiver").and_then(|v| v.as_str()) == Some(waiver.as_str()))
        });
        if !allowed {
            return Err(PolicyError::unknown_waiver(waiver));
        }
    }
    explanation
        .push("step 5: every requested exception rides a policy's exception schema".to_string());

    // Step 6: the clause-id collisions across the APPLICABLE policies — the
    // precedence settles them; the unresolved binding conflict FAILS CLOSED.
    let mut by_clause: std::collections::BTreeMap<String, Vec<usize>> =
        std::collections::BTreeMap::new();
    for (i, row) in loaded.iter().enumerate() {
        if !applicable[i] {
            continue;
        }
        let clauses: Vec<ClauseStatement> =
            serde_json::from_value(row.clauses.clone()).expect("the clauses parse");
        for clause in &clauses {
            by_clause.entry(clause.id.clone()).or_default().push(i);
        }
    }
    let mut resolved = Vec::new();
    let mut conflicts = Vec::new();
    for (clause_id, carriers) in &by_clause {
        let winner = if carriers.len() == 1 {
            carriers[0]
        } else {
            // The candidate that wins over ALL the others transitively.
            let candidates: Vec<usize> = carriers
                .iter()
                .copied()
                .filter(|&candidate| {
                    carriers.iter().copied().all(|other| {
                        other == candidate
                            || wins_over(&loaded[candidate].policy_id, &loaded[other].policy_id)
                    })
                })
                .collect();
            if candidates.len() != 1 {
                conflicts.push(clause_id.clone());
                continue;
            }
            candidates[0]
        };
        let clauses: Vec<ClauseStatement> =
            serde_json::from_value(loaded[winner].clauses.clone()).expect("the clauses parse");
        let clause = clauses
            .iter()
            .find(|c| c.id == *clause_id)
            .expect("the clause is present");
        resolved.push(ResolvedClause {
            policy_id: loaded[winner].policy_id.clone(),
            version: loaded[winner].version.clone(),
            clause_id: clause.id.clone(),
            statement: clause.statement.clone(),
            path: vec![
                "authority: active grant".to_string(),
                "applicability: matched".to_string(),
                "precedence: the winner over the carriers".to_string(),
            ],
        });
    }
    if !conflicts.is_empty() {
        return Err(PolicyError::binding_conflict(&conflicts.join(", ")));
    }
    explanation.push(format!(
        "step 6: {} clauses resolved with no unresolved binding conflict",
        resolved.len()
    ));

    // Step 7: the explanation tree rides the result.
    Ok(Resolution {
        target: ResolutionTarget {
            layer: request.target.layer.clone(),
            target: request.target.target.clone(),
        },
        resolved,
        explanation,
        conflicts,
    })
}

/// The impact map (`.1.3`): the derivable coverage — the clauses × the
/// applicability selectors the policy DECLARES (never an achievement claim).
pub async fn impact(
    pool: &PgPool,
    policy_id: &str,
    version: &str,
) -> Result<Vec<Value>, PolicyError> {
    let row: Option<ResolutionRow> = sqlx::query_as(
        "SELECT policy_id, version, lifecycle, digest, owning_authority, clauses, \
         applicability, non_applicability, dependencies, conflicts, precedence_hints, \
         exceptions \
         FROM policy_versions WHERE policy_id = $1 AND version = $2",
    )
    .bind(policy_id)
    .bind(version)
    .fetch_optional(pool)
    .await
    .map_err(|_| PolicyError::unknown_ref(policy_id, version))?;
    let row = row.ok_or_else(|| PolicyError::unknown_ref(policy_id, version))?;
    let clauses: Vec<ClauseStatement> =
        serde_json::from_value(row.clauses).expect("the clauses parse");
    Ok(clauses
        .into_iter()
        .map(|clause| {
            serde_json::json!({
                "clause_id": clause.id,
                "statement": clause.statement,
                "applicability": row.applicability,
                "non_applicability": row.non_applicability,
            })
        })
        .collect())
}
