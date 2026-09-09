//! Guarded standalone authority services. These locks coordinate the named tenant;
//! they do not authenticate an issuer or join a separate HTTP admission transaction.

use reasonbraid_core::{
    boundary_active_at, grant_exceeds_boundary, AuthorityGrant, BoundaryStatus,
    EnrollmentAuthorityBoundary, GrantStatus, TenantId,
};
use sqlx::{PgConnection, PgPool};

use super::transaction::{
    transact, transact_with_error, GuardError, GuardMode, Limits, TenantTransaction,
};
use super::{
    boundary_from_row, bump_revocation_epoch, insert_boundary_in_tx, insert_grant_row,
    AuthorityTransactionError, BoundaryRow, GrantCreateError, GrantRefused,
};

/// Insert an enrollment boundary under the tenant's exclusive authority guard.
/// This does not re-activate or replace an existing row. The unique partial index
/// still refuses a second active boundary. Standalone namespaces need no identity.
/// Commit errors preserve uncertainty through [`AuthorityTransactionError`].
pub async fn create_boundary(
    pool: &PgPool,
    boundary: &EnrollmentAuthorityBoundary,
) -> Result<(), AuthorityTransactionError> {
    let boundary = boundary.clone();
    let tenant = boundary.tenant_id;
    transact(pool, &[(tenant, GuardMode::Exclusive)], move |tx| {
        Box::pin(async move {
            insert_boundary_in_tx(tx.connection(tenant, GuardMode::Exclusive)?, &boundary).await?;
            Ok(())
        })
    })
    .await
}

/// Create a grant under the candidate tenant's exclusive authority guard. The
/// actual own-tenant parent must pass the structural checks and be live at the
/// database-time evaluation after guard/parent lookup, immediately before INSERT.
/// Scheduled grants remain supported. Missing/policy refusals roll back the
/// transaction, including a first-use anchor; existing anchors remain intact.
/// Transaction failures preserve their phase and never imply automatic retry.
pub async fn create_grant(pool: &PgPool, grant: &AuthorityGrant) -> Result<(), GrantCreateError> {
    let grant = grant.clone();
    let tenant = grant.tenant_id;
    transact_with_error(
        pool,
        &[(tenant, GuardMode::Exclusive)],
        Limits::default(),
        move |tx| Box::pin(async move { create_grant_in_guard(tx, &grant).await }),
    )
    .await
}

/// Same-context grant creation for complete guarded transactions, including
/// development enrollment. The caller must have declared the exclusive tenant.
pub(crate) async fn create_grant_in_guard(
    tx: &mut TenantTransaction<'_>,
    grant: &AuthorityGrant,
) -> Result<(), GrantCreateError> {
    let tenant = grant.tenant_id;
    let boundary = issuance_parent(tx.connection(tenant, GuardMode::Exclusive)?, grant).await?;
    let violations = grant_exceeds_boundary(&boundary, grant);
    if !violations.is_empty() {
        return Err(GrantCreateError::Refused(GrantRefused { violations }));
    }
    let at = tx.database_now().await?;
    if !boundary_active_at(&boundary, at) {
        return Err(GrantCreateError::BoundaryNotLive {
            boundary_id: grant.boundary_id.clone(),
        });
    }
    insert_grant_row(tx.connection(tenant, GuardMode::Exclusive)?, grant).await?;
    Ok(())
}

/// Load policy only inside the guarded tenant. The foreign-ID existence probe
/// identifies a binding refusal; it neither decodes nor evaluates foreign policy.
async fn issuance_parent(
    conn: &mut PgConnection,
    grant: &AuthorityGrant,
) -> Result<EnrollmentAuthorityBoundary, GrantCreateError> {
    let row: Option<BoundaryRow> = sqlx::query_as(
        "SELECT boundary_id, tenant_id, parent_or_root_authority, target_owner, permitted_actions, \
         permitted_domains, risk_ceiling, spend_ceiling, delegable, max_delegation_depth, \
         valid_from, expires_at, charter_digest, policy_version, status \
         FROM enrollment_boundaries WHERE boundary_id = $1 AND tenant_id = $2")
        .bind(&grant.boundary_id).bind(grant.tenant_id.to_string()).fetch_optional(&mut *conn).await?;
    if let Some(row) = row {
        return boundary_from_row(row).ok_or_else(|| {
            sqlx::Error::Protocol("stored enrollment boundary is malformed".into()).into()
        });
    }
    let foreign: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM enrollment_boundaries WHERE boundary_id = $1 AND tenant_id <> $2)")
        .bind(&grant.boundary_id).bind(grant.tenant_id.to_string()).fetch_one(&mut *conn).await?;
    if foreign {
        Err(GrantCreateError::Refused(GrantRefused {
            violations: vec![reasonbraid_core::BoundaryViolation {
                field: "grant.tenant_id",
                detail: "the grant and its boundary belong to different tenants".into(),
            }],
        }))
    } else {
        Err(GrantCreateError::MissingBoundary {
            boundary_id: grant.boundary_id.clone(),
        })
    }
}

/// Read the current active boundary in a shared guarded transaction. Callers that
/// use it after this returns must provide their own complete effect transaction;
/// card import remains an explicit temporary bridge until its complete integration.
pub(crate) async fn load_active_boundary_for_tenant(
    pool: &PgPool,
    tenant: &TenantId,
) -> Result<Option<EnrollmentAuthorityBoundary>, AuthorityTransactionError> {
    let tenant = *tenant;
    transact(pool, &[(tenant, GuardMode::Shared)], move |tx| {
        Box::pin(async move { load_active_boundary_in_guard(tx, tenant).await })
    })
    .await
}

/// Load active policy through the current transaction. An exclusive issuance
/// context satisfies this read scope without another pool acquisition/guard.
pub(crate) async fn load_active_boundary_in_guard(
    tx: &mut TenantTransaction<'_>,
    tenant: TenantId,
) -> Result<Option<EnrollmentAuthorityBoundary>, AuthorityTransactionError> {
    let row: Option<BoundaryRow> = sqlx::query_as(
        "SELECT boundary_id, tenant_id, parent_or_root_authority, target_owner, permitted_actions, \
         permitted_domains, risk_ceiling, spend_ceiling, delegable, max_delegation_depth, \
         valid_from, expires_at, charter_digest, policy_version, status \
         FROM enrollment_boundaries WHERE tenant_id = $1 AND status = 'active'")
        .bind(tenant.to_string()).fetch_optional(tx.connection(tenant, GuardMode::Shared)?).await?;
    row.map(|row| {
        boundary_from_row(row).ok_or_else(|| {
            GuardError::Storage(sqlx::Error::Protocol(
                "stored enrollment boundary is malformed".into(),
            ))
        })
    })
    .transpose()
}

/// Guard before the matching target lock; status and epoch commit together.
/// Missing/foreign targets and already-revoked targets have no epoch effect.
/// Malformed status is storage failure, not an inferred transitionable state.
pub(crate) async fn revoke_grant(
    pool: &PgPool,
    grant_id: &str,
    expected_tenant: TenantId,
) -> Result<Option<(String, GrantStatus)>, AuthorityTransactionError> {
    let grant_id = grant_id.to_owned();
    transact(pool, &[(expected_tenant, GuardMode::Exclusive)], move |tx| {
        Box::pin(async move {
            let conn = tx.connection(expected_tenant, GuardMode::Exclusive)?;
            let tenant_key = expected_tenant.to_string();
            let row: Option<(String, String)> = sqlx::query_as(
                "SELECT tenant_id, status FROM authority_grants WHERE grant_id = $1 AND tenant_id = $2 FOR UPDATE")
                .bind(&grant_id).bind(&tenant_key).fetch_optional(&mut *conn).await?;
            let Some((tenant, status)) = row else { return Ok(None) };
            let previous = status.parse::<GrantStatus>().map_err(|_| sqlx::Error::Protocol(
                "stored authority grant status is malformed".into()))?;
            if previous != GrantStatus::Revoked {
                sqlx::query("UPDATE authority_grants SET status = 'revoked' WHERE grant_id = $1 AND tenant_id = $2")
                    .bind(&grant_id).bind(&tenant_key).execute(&mut *conn).await?;
                bump_revocation_epoch(conn, &tenant).await?;
            }
            Ok(Some((tenant, previous)))
        })
    }).await
}

/// Boundary revocation uses the same exclusive tenant guard as grant issuance.
/// This does not join the caller's separate HTTP admission until `.3.3.4.8`.
pub(crate) async fn revoke_boundary(
    pool: &PgPool,
    boundary_id: &str,
    expected_tenant: TenantId,
) -> Result<Option<(String, BoundaryStatus)>, AuthorityTransactionError> {
    let boundary_id = boundary_id.to_owned();
    transact(pool, &[(expected_tenant, GuardMode::Exclusive)], move |tx| {
        Box::pin(async move {
            let conn = tx.connection(expected_tenant, GuardMode::Exclusive)?;
            let tenant_key = expected_tenant.to_string();
            let row: Option<(String, String)> = sqlx::query_as(
                "SELECT tenant_id, status FROM enrollment_boundaries WHERE boundary_id = $1 AND tenant_id = $2 FOR UPDATE")
                .bind(&boundary_id).bind(&tenant_key).fetch_optional(&mut *conn).await?;
            let Some((tenant, status)) = row else { return Ok(None) };
            let previous = status.parse::<BoundaryStatus>().map_err(|_| sqlx::Error::Protocol(
                "stored enrollment boundary status is malformed".into()))?;
            if previous != BoundaryStatus::Revoked {
                sqlx::query("UPDATE enrollment_boundaries SET status = 'revoked' WHERE boundary_id = $1 AND tenant_id = $2")
                    .bind(&boundary_id).bind(&tenant_key).execute(&mut *conn).await?;
                bump_revocation_epoch(conn, &tenant).await?;
            }
            Ok(Some((tenant, previous)))
        })
    }).await
}
