//! Guarded standalone authority services. These locks coordinate the named tenant;
//! they do not authenticate an issuer or join a separate HTTP admission transaction.

use reasonbraid_core::{
    boundary_active_at, grant_exceeds_boundary, AuthorityGrant, EnrollmentAuthorityBoundary,
    TenantId,
};
use sqlx::{PgConnection, PgPool};

use super::transaction::{
    transact, transact_with_error, GuardError, GuardMode, Limits, TenantTransaction,
};
use super::{
    boundary_from_row, insert_boundary_in_tx, insert_grant_row, AuthorityTransactionError,
    BoundaryRow, GrantCreateError, GrantRefused,
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
    let mut violations = grant_exceeds_boundary(&boundary, grant);
    // The auto bounds are checked HERE, in the one path every producer takes
    // (`SIGNOFF-REPAIR.11.4.7.2.1.5.3.2.3.1`), so a row can only carry what an
    // issuer could have declared, and the refusal is the same typed refusal a
    // boundary overrun gives.
    violations.extend(auto_bounds_violations(grant));
    violations.extend(decision_rule_violations(grant));
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

/// What a grant's `auto_bounds` may say. Each rule refuses a declaration that
/// would bind NOTHING or bind more than the site allows — the two ways a bound
/// stops being a bound:
/// - bounds on a grant without `thread_create_auto` bind no initiation;
/// - an empty `topics` list admits no topic (omit the bound, or the action);
/// - `max_depth` is 1 or more (zero admits no initiation) and never above
///   `MAX_AUTONOMOUS_DEPTH`, the site ceiling `auto_lineage` applies.
fn auto_bounds_violations(grant: &AuthorityGrant) -> Vec<reasonbraid_core::BoundaryViolation> {
    use reasonbraid_core::BoundaryViolation;
    let Some(bounds) = &grant.auto_bounds else {
        return Vec::new();
    };
    let mut violations = Vec::new();
    if !grant
        .actions
        .contains(&reasonbraid_core::GrantAction::ThreadCreateAuto)
    {
        violations.push(BoundaryViolation {
            field: "grant.auto_bounds",
            detail: "auto bounds on a grant without the thread_create_auto action bind nothing"
                .into(),
        });
    }
    if let Some(topics) = &bounds.topics {
        if topics.is_empty() {
            violations.push(BoundaryViolation {
                field: "grant.auto_bounds.topics",
                detail: "an empty topic bound admits no topic — omit the bound, or omit the action"
                    .into(),
            });
        }
    }
    if let Some(depth) = bounds.max_depth {
        let ceiling = crate::threads::MAX_AUTONOMOUS_DEPTH;
        if depth == 0 {
            violations.push(BoundaryViolation {
                field: "grant.auto_bounds.max_depth",
                detail: "a depth bound of zero admits no initiation — omit the bound, or omit the action"
                    .into(),
            });
        } else if depth > ceiling {
            violations.push(BoundaryViolation {
                field: "grant.auto_bounds.max_depth",
                detail: format!("a depth bound of {depth} exceeds the site ceiling of {ceiling}"),
            });
        }
    }
    violations
}

/// What a grant's `decision_rule_constraints` may say
/// (`SIGNOFF-REPAIR.11.4.7.2.1.5.4.1`): a constraint on a grant that creates no
/// thread binds nothing; an empty list admits no rule (issue the grant without
/// the action instead); every name is a charter wire name the server knows.
fn decision_rule_violations(grant: &AuthorityGrant) -> Vec<reasonbraid_core::BoundaryViolation> {
    use reasonbraid_core::{BoundaryViolation, GrantAction};
    let Some(rules) = &grant.decision_rule_constraints else {
        return Vec::new();
    };
    let mut violations = Vec::new();
    let creates = grant
        .actions
        .iter()
        .any(|a| matches!(a, GrantAction::ThreadCreate | GrantAction::ThreadCreateAuto));
    if !creates {
        violations.push(BoundaryViolation {
            field: "grant.decision_rule_constraints",
            detail: "a decision-rule constraint on a grant that creates no thread binds nothing"
                .into(),
        });
    }
    if rules.is_empty() {
        violations.push(BoundaryViolation {
            field: "grant.decision_rule_constraints",
            detail: "an empty decision-rule constraint admits no rule — omit the constraint, or the action"
                .into(),
        });
    }
    for name in rules {
        if crate::charters::DecisionRule::parse(name).is_none() {
            violations.push(BoundaryViolation {
                field: "grant.decision_rule_constraints",
                detail: format!("`{name}` is not a decision rule the charter vocabulary knows"),
            });
        }
    }
    violations
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

// `load_active_boundary_for_tenant` used to live here: a pool-taking shared-guard
// read whose own comment called it "an explicit temporary bridge until its
// complete integration" for card import. `SIGNOFF-REPAIR.3.3.4.11.3` integrated
// it — the import reads its boundary with `load_active_boundary_in_guard` inside
// the transaction that then issues the grant against it, so a boundary revoked
// between the read and the issuance is no longer possible. Removed rather than
// left for a future caller to reintroduce the gap.

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

// `revoke_grant` and `revoke_boundary` used to live here: a second guarded
// transaction that the HTTP handler ran AFTER its own admission transaction.
// `SIGNOFF-REPAIR.3.3.4.8` replaced both with one transaction that holds the
// admission, the target selection, the status change, the epoch bump and the
// final effect record together (`authority/revocation.rs`), so these are removed
// rather than left as a second, unordered way to revoke the same rows.
