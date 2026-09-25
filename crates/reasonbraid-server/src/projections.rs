//! The policy-projection records (`PHASE-6.3.2`, ADR-033): the server
//! RESOLVES (the `.1.3` seven-step pipeline), the compiler RENDERS (the
//! hermetic crate — never a re-resolve), and the artifact is RECORDED with
//! its digest + its declared unrepresentable list.

use serde::{Deserialize, Serialize};
use sqlx::PgPool;

/// The projection request: the id, the target, and the resolution request.
///
/// ⛔ It carries NO lock rows (`SIGNOFF-REPAIR.9.1.4`). It used to, and the
/// `lock` target rendered them verbatim, so a published policy.lock could name a
/// policy that was never resolved, a digest nobody registered, or another
/// principal's grant. [`project`] writes the rows itself from the registry, and
/// `deny_unknown_fields` refuses a request that still sends them.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectionRequest {
    pub projection_id: String,
    pub target: String,
    pub resolution: crate::policy::ResolutionRequest,
}

/// The stored projection row.
#[derive(Debug, Clone, Serialize)]
pub struct StoredProjection {
    pub projection_id: String,
    pub target: String,
    pub digest: String,
    pub bytes: String,
    pub unrepresentable: Vec<reasonbraid_policy_compiler::Unrepresentable>,
    /// The `(policy_id, version)` pairs the seven-step resolution produced
    /// (`SIGNOFF-REPAIR.9.2.1.3.2`), so a publication can be required to carry
    /// the policy its proposal was approved for.
    ///
    /// ⚠️ `None` means the row predates `migrations/0082` — *the set was never
    /// recorded*, never *the set is empty*. Staging FAILS CLOSED on it, and the
    /// refusal says which of the two it is.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_policies: Option<Vec<crate::policy::PolicyRef>>,
}

/// The typed refusal reasons, and the one failure that is not a refusal.
#[derive(Debug)]
pub enum ProjectionError {
    Duplicate(String),
    Resolution(crate::policy::PolicyError),
    Compile(reasonbraid_policy_compiler::CompileError),
    /// A policy the lock would name, whose stored digest does not verify: a
    /// version registered before the server derived digests
    /// (`SIGNOFF-REPAIR.9.1.4`). The lock never publishes a digest that
    /// identifies nothing.
    UnverifiedDigest {
        policy_id: String,
        version: String,
    },
    /// The store failed to answer. Not a refusal, so it reaches the caller as
    /// `api::storage_failure` decides, never as a `400`.
    Storage(sqlx::Error),
}

impl std::fmt::Display for ProjectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProjectionError::Duplicate(what) => {
                write!(
                    f,
                    "{what} already exists — the record's identity is its content"
                )
            }
            ProjectionError::Resolution(e) => write!(f, "{e}"),
            ProjectionError::Compile(e) => write!(f, "{e}"),
            ProjectionError::UnverifiedDigest { policy_id, version } => write!(
                f,
                "policy `{policy_id}` version {version} carries a stored digest its document does \
                 not hash to (it was declared before the server derived digests), so no lock \
                 names it"
            ),
            ProjectionError::Storage(e) => write!(f, "the policy store failed: {e}"),
        }
    }
}

/// Project one resolved set: the resolve → the compile → the record.
///
/// ⛔ `tenant_id` is the AUTHOR's, and it is the one lifecycle tenant that is
/// authorship rather than lineage (`SIGNOFF-REPAIR.6.1.5.2`). A projection has
/// no ancestor: `ProjectionRequest` names a target, a resolution over the
/// SITE-WIDE library, and a lock — no thread, no proposal, no tenant. It is
/// still tenant work, because `bytes` and `unrepresentable` are a function of
/// the caller's own resolution request, so the row discloses which policies its
/// author compiled for which target. ⚠️ Which is also why `migrations/0073`
/// backfills every lifecycle table but this one: an authorship the old schema
/// never recorded cannot be recovered from it.
pub async fn project(
    pool: &PgPool,
    tenant_id: &str,
    request: &ProjectionRequest,
) -> Result<StoredProjection, ProjectionError> {
    let resolution = crate::policy::resolve(pool, &request.resolution)
        .await
        .map_err(ProjectionError::Resolution)?;
    let clauses: Vec<reasonbraid_policy_compiler::InputClause> = resolution
        .resolved
        .iter()
        .map(|clause| reasonbraid_policy_compiler::InputClause {
            policy_id: clause.policy_id.clone(),
            policy_version: clause.version.clone(),
            clause_id: clause.clause_id.clone(),
            statement: clause.statement.clone(),
        })
        .collect();
    // `SIGNOFF-REPAIR.9.1.4`: the lock rows are the SERVER'S. One per policy the
    // request named (the collection the resolution consumed, §15.1), each read
    // from the registry with its derived digest and its owner.
    let named = crate::policy::registered(pool, &request.resolution.policies)
        .await
        .map_err(ProjectionError::Storage)?;
    if request.target == "lock" {
        if let Some(unverified) = named.iter().find(|p| !p.digest_verified) {
            return Err(ProjectionError::UnverifiedDigest {
                policy_id: unverified.policy_id.clone(),
                version: unverified.version.clone(),
            });
        }
    }
    let lock = named
        .into_iter()
        .map(|p| reasonbraid_policy_compiler::LockedPolicy {
            policy_id: p.policy_id,
            version: p.version,
            digest: p.digest,
            owning_authority: p.owning_authority,
        })
        .collect();
    let artifact =
        reasonbraid_policy_compiler::compile(&reasonbraid_policy_compiler::CompileRequest {
            target: request.target.clone(),
            clauses,
            lock,
        })
        .map_err(ProjectionError::Compile)?;
    let unrepresentable =
        serde_json::to_value(&artifact.unrepresentable).expect("the unrepresentables serialize");
    // `SIGNOFF-REPAIR.9.2.1.3.2`: the resolved `(policy_id, version)` set, so a
    // publication can be required to carry the policy its proposal was
    // approved for. ⛔ Taken from the RESOLUTION rather than from the rendered
    // bytes: the compiler drops every unrepresentable clause before rendering,
    // so a policy that resolved and could not ride this target leaves no trace
    // in the text while genuinely being part of the set.
    let mut resolved_policies: Vec<crate::policy::PolicyRef> = Vec::new();
    for clause in &resolution.resolved {
        if !resolved_policies
            .iter()
            .any(|p| p.policy_id == clause.policy_id && p.version == clause.version)
        {
            resolved_policies.push(crate::policy::PolicyRef {
                policy_id: clause.policy_id.clone(),
                version: clause.version.clone(),
            });
        }
    }
    let resolved_policies_json =
        serde_json::to_value(&resolved_policies).expect("the resolved set serializes");
    let inserted = sqlx::query(
        "INSERT INTO policy_projections \
         (projection_id, target, digest, bytes, unrepresentable, tenant_id, resolved_policies) \
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(&request.projection_id)
    .bind(&artifact.target)
    .bind(&artifact.digest)
    .bind(&artifact.bytes)
    .bind(&unrepresentable)
    .bind(tenant_id)
    .bind(&resolved_policies_json)
    .execute(pool)
    .await;
    if inserted.is_err() {
        return Err(ProjectionError::Duplicate(format!(
            "projection `{}`",
            request.projection_id
        )));
    }
    Ok(StoredProjection {
        projection_id: request.projection_id.clone(),
        target: artifact.target,
        digest: artifact.digest,
        bytes: artifact.bytes,
        unrepresentable: artifact.unrepresentable,
        resolved_policies: Some(resolved_policies),
    })
}

/// Load one projection row (the `.4.3.2` publish verb reads the bundle +
/// the digest it publishes).
pub async fn load(pool: &PgPool, projection_id: &str) -> Result<StoredProjection, ProjectionError> {
    let row: Option<(
        String,
        String,
        String,
        String,
        serde_json::Value,
        Option<serde_json::Value>,
    )> = sqlx::query_as(
        "SELECT projection_id, target, digest, bytes, unrepresentable, resolved_policies \
         FROM policy_projections WHERE projection_id = $1",
    )
    .bind(projection_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| ProjectionError::Duplicate(projection_id.to_string()))?;
    let Some((projection_id, target, digest, bytes, unrepresentable, resolved_policies)) = row
    else {
        return Err(ProjectionError::Duplicate(format!(
            "projection `{projection_id}` (the publish references a REGISTERED projection)"
        )));
    };
    Ok(StoredProjection {
        projection_id,
        target,
        digest,
        bytes,
        unrepresentable: serde_json::from_value(unrepresentable)
            .expect("the unrepresentables parse"),
        resolved_policies: resolved_policies.and_then(|value| serde_json::from_value(value).ok()),
    })
}

/// The projections, newest first.
/// ⛔ `SIGNOFF-REPAIR.6.1.5.3`: bound to the caller's tenant. Until now this
/// returned every tenant's rows to any enrolled principal. ⚠️ The predicate
/// never matches NULL, so a row `migrations/0073` could not attribute is read
/// by NOBODY — `.7.1.2.2`'s disposition, and the reason the backfill's coverage
/// is published as a measured count rather than assumed complete.
pub async fn list(pool: &PgPool, tenant_id: &str) -> Result<Vec<StoredProjection>, sqlx::Error> {
    let rows: Vec<(
        String,
        String,
        String,
        String,
        serde_json::Value,
        Option<serde_json::Value>,
    )> = sqlx::query_as(
        "SELECT projection_id, target, digest, bytes, unrepresentable, resolved_policies \
         FROM policy_projections WHERE tenant_id = $1 ORDER BY created_at DESC",
    )
    .bind(tenant_id)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(
            |(projection_id, target, digest, bytes, unrepresentable, resolved_policies)| {
                StoredProjection {
                    projection_id,
                    target,
                    digest,
                    bytes,
                    unrepresentable: serde_json::from_value(unrepresentable)
                        .expect("the unrepresentables parse"),
                    resolved_policies: resolved_policies
                        .and_then(|value| serde_json::from_value(value).ok()),
                }
            },
        )
        .collect())
}
