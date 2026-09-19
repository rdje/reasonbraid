//! The policy-projection records (`PHASE-6.3.2`, ADR-033): the server
//! RESOLVES (the `.1.3` seven-step pipeline), the compiler RENDERS (the
//! hermetic crate — never a re-resolve), and the artifact is RECORDED with
//! its digest + its declared unrepresentable list.

use serde::{Deserialize, Serialize};
use sqlx::PgPool;

/// The projection request: the id, the target, the resolution request, and
/// the lock rows (the policy.lock's inputs).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectionRequest {
    pub projection_id: String,
    pub target: String,
    pub resolution: crate::policy::ResolutionRequest,
    #[serde(default)]
    pub lock: Vec<reasonbraid_policy_compiler::LockedPolicy>,
}

/// The stored projection row.
#[derive(Debug, Clone, Serialize)]
pub struct StoredProjection {
    pub projection_id: String,
    pub target: String,
    pub digest: String,
    pub bytes: String,
    pub unrepresentable: Vec<reasonbraid_policy_compiler::Unrepresentable>,
}

/// The typed refusal reasons.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectionError {
    Duplicate(String),
    Resolution(crate::policy::PolicyError),
    Compile(reasonbraid_policy_compiler::CompileError),
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
    let artifact =
        reasonbraid_policy_compiler::compile(&reasonbraid_policy_compiler::CompileRequest {
            target: request.target.clone(),
            clauses,
            lock: request.lock.clone(),
        })
        .map_err(ProjectionError::Compile)?;
    let unrepresentable =
        serde_json::to_value(&artifact.unrepresentable).expect("the unrepresentables serialize");
    let inserted = sqlx::query(
        "INSERT INTO policy_projections \
         (projection_id, target, digest, bytes, unrepresentable, tenant_id) \
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(&request.projection_id)
    .bind(&artifact.target)
    .bind(&artifact.digest)
    .bind(&artifact.bytes)
    .bind(&unrepresentable)
    .bind(tenant_id)
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
    })
}

/// Load one projection row (the `.4.3.2` publish verb reads the bundle +
/// the digest it publishes).
pub async fn load(pool: &PgPool, projection_id: &str) -> Result<StoredProjection, ProjectionError> {
    let row: Option<(String, String, String, String, serde_json::Value)> = sqlx::query_as(
        "SELECT projection_id, target, digest, bytes, unrepresentable          FROM policy_projections WHERE projection_id = $1",
    )
    .bind(projection_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| ProjectionError::Duplicate(projection_id.to_string()))?;
    let Some((projection_id, target, digest, bytes, unrepresentable)) = row else {
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
    })
}

/// The projections, newest first.
/// ⛔ `SIGNOFF-REPAIR.6.1.5.3`: bound to the caller's tenant. Until now this
/// returned every tenant's rows to any enrolled principal. ⚠️ The predicate
/// never matches NULL, so a row `migrations/0073` could not attribute is read
/// by NOBODY — `.7.1.2.2`'s disposition, and the reason the backfill's coverage
/// is published as a measured count rather than assumed complete.
pub async fn list(pool: &PgPool, tenant_id: &str) -> Result<Vec<StoredProjection>, sqlx::Error> {
    let rows: Vec<(String, String, String, String, serde_json::Value)> = sqlx::query_as(
        "SELECT projection_id, target, digest, bytes, unrepresentable \
         FROM policy_projections WHERE tenant_id = $1 ORDER BY created_at DESC",
    )
    .bind(tenant_id)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(
            |(projection_id, target, digest, bytes, unrepresentable)| StoredProjection {
                projection_id,
                target,
                digest,
                bytes,
                unrepresentable: serde_json::from_value(unrepresentable)
                    .expect("the unrepresentables parse"),
            },
        )
        .collect())
}
