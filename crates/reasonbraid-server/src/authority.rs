//! The development authorization engine (`PHASE-0.5.1`; `ROADMAP.md` §4.4/§4.5).
//!
//! # What this enforces
//!
//! - **A grant cannot exceed its boundary** — [`create_grant`] refuses at creation,
//!   and every evaluation RE-CHECKS the subset rule (`grant_exceeds_boundary`) as
//!   defense in depth.
//! - **Tenant membership alone grants nothing** — a mandate exists only through a
//!   grant that names it; `TenantAdmin` is never implied by other actions.
//! - **Every command records its authorization** — [`authorize`] writes an
//!   `authorization_records` row (actor, subject if delegated, grant + boundary
//!   references, decision, policy digest + version) whether the decision is allowed
//!   or denied; [`apply_authorized_command`] runs that record and the WP2 durability
//!   writes in ONE transaction, so an accepted command implies its audit record.
//!
//! # Development-profile limits (stated, not hidden)
//!
//! Authentication and credential issuance are out of scope (WP5: development
//! credentials; no certificate issuer) — the caller supplies an already-resolved
//! actor. Delegation CHAINS are out of scope (direct grants only; depth is recorded
//! on the boundary for Phase 2). The store is the control plane's own PostgreSQL —
//! the dev profile trusts it as the authority store.

use chrono::{DateTime, Utc};
use reasonbraid_core::{
    boundary_active_at, grant_active_at, grant_exceeds_boundary, policy_digest, ActorPrincipalId,
    AuthorityGrant, AuthorizationDecisionRecord, AuthorizationRecordId, BoundaryStatus, Decision,
    EnrollmentAuthorityBoundary, GrantAction, GrantStatus, GrantSubject, ResourceTarget, RiskClass,
    TargetSelector,
};
use serde_json::Value;
use sqlx::PgPool;

use crate::tx::{self, ApplyError, Command, CommandOutcome};

/// The authorization context of one command: the authenticated actor, the grant
/// holder (the actor, or the delegating subject's principal), the delegated subject
/// when delegation applies, and the requested action/target.
pub struct CommandAuthz {
    /// Who the request authenticates as (the audit actor).
    pub actor: ActorPrincipalId,
    /// The grant holder — the principal whose grant is evaluated.
    pub principal: GrantSubject,
    /// The original subject when the actor acts on its behalf (delegation).
    pub delegate_subject: Option<GrantSubject>,
    pub action: GrantAction,
    pub target: ResourceTarget,
}

/// The outcome of [`authorize`].
#[derive(Debug, Clone, PartialEq)]
pub enum AuthorizationOutcome {
    Allowed { record_id: String },
    Denied { reason: String, record_id: String },
}

/// The typed failure of [`create_grant`] when the grant exceeds its boundary.
#[derive(Debug)]
pub struct GrantRefused {
    pub violations: Vec<reasonbraid_core::BoundaryViolation>,
}

impl std::fmt::Display for GrantRefused {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "grant refused: {}",
            self.violations
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join("; ")
        )
    }
}

impl std::error::Error for GrantRefused {}

/// The typed failure of [`apply_authorized_command`].
#[derive(Debug)]
pub enum AuthorizedApplyError {
    /// The command was denied — the denial record IS durable, the command was not
    /// applied. `reason` is the safe, human-readable denial.
    Denied {
        reason: String,
        record_id: String,
    },
    Apply(ApplyError),
    Sql(sqlx::Error),
}

impl std::fmt::Display for AuthorizedApplyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthorizedApplyError::Denied { reason, record_id } => {
                write!(f, "authorization denied ({record_id}): {reason}")
            }
            AuthorizedApplyError::Apply(e) => write!(f, "command apply failed: {e}"),
            AuthorizedApplyError::Sql(e) => write!(f, "database error: {e}"),
        }
    }
}

impl std::error::Error for AuthorizedApplyError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AuthorizedApplyError::Apply(e) => Some(e),
            AuthorizedApplyError::Sql(e) => Some(e),
            _ => None,
        }
    }
}

impl From<sqlx::Error> for AuthorizedApplyError {
    fn from(e: sqlx::Error) -> Self {
        AuthorizedApplyError::Sql(e)
    }
}

impl From<ApplyError> for AuthorizedApplyError {
    fn from(e: ApplyError) -> Self {
        AuthorizedApplyError::Apply(e)
    }
}

// ── Repository ──────────────────────────────────────────────────────────────────

/// Create (or re-activate) an enrollment boundary. The dev profile holds one ACTIVE
/// boundary per tenant — the unique partial index refuses a second active one.
pub async fn create_boundary(
    pool: &PgPool,
    boundary: &EnrollmentAuthorityBoundary,
) -> Result<(), sqlx::Error> {
    let mut conn = pool.acquire().await?;
    insert_boundary_in_tx(&mut *conn, boundary).await
}

/// The transactional body of [`create_boundary`] — shared with the `.6.1` enroll
/// bootstrap so a new tenant's boundary, grant, and enrollment row commit together.
pub(crate) async fn insert_boundary_in_tx<'e, E>(
    mut tx: E,
    boundary: &EnrollmentAuthorityBoundary,
) -> Result<(), sqlx::Error>
where
    E: std::ops::DerefMut,
    for<'c> &'c mut <E as std::ops::Deref>::Target: sqlx::Executor<'c, Database = sqlx::Postgres>,
{
    sqlx::query(
        "INSERT INTO enrollment_boundaries \
         (boundary_id, tenant_id, parent_or_root_authority, target_owner, permitted_actions, \
          permitted_domains, risk_ceiling, spend_ceiling, delegable, max_delegation_depth, \
          valid_from, expires_at, charter_digest, policy_version, status) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)",
    )
    .bind(&boundary.boundary_id)
    .bind(boundary.tenant_id.to_string())
    .bind(&boundary.parent_or_root_authority)
    .bind(&boundary.target_owner)
    .bind(
        serde_json::to_value(
            boundary
                .permitted_actions
                .iter()
                .map(|a| a.as_str())
                .collect::<Vec<_>>(),
        )
        .expect("actions serialize"),
    )
    .bind(serde_json::to_value(&boundary.permitted_domains).expect("domains serialize"))
    .bind(boundary.risk_ceiling.as_str())
    .bind(boundary.spend_ceiling.as_ref())
    .bind(boundary.delegable)
    .bind(boundary.max_delegation_depth as i32)
    .bind(boundary.valid_from)
    .bind(boundary.expires_at)
    .bind(&boundary.charter_digest)
    .bind(&boundary.policy_version)
    .bind(boundary.status.as_str())
    .execute(&mut *tx)
    .await?;
    Ok(())
}

/// Create a grant — REFUSED (no row) when it exceeds its boundary in any dimension
/// (§4.4: "a grant cannot exceed the enrollment ceiling").
pub async fn create_grant(pool: &PgPool, grant: &AuthorityGrant) -> Result<(), GrantRefused> {
    let mut conn = pool.acquire().await.map_err(|_| GrantRefused {
        violations: vec![reasonbraid_core::BoundaryViolation {
            field: "grant",
            detail: format!("grant `{}` could not be stored", grant.grant_id),
        }],
    })?;
    create_grant_in_tx(&mut *conn, grant).await
}

/// The transactional body of [`create_grant`] — shared with the `.6.1` enroll
/// bootstrap (boundary load + subset check + insert on the caller's executor).
pub(crate) async fn create_grant_in_tx<'e, E>(
    mut tx: E,
    grant: &AuthorityGrant,
) -> Result<(), GrantRefused>
where
    E: std::ops::DerefMut,
    for<'c> &'c mut <E as std::ops::Deref>::Target: sqlx::Executor<'c, Database = sqlx::Postgres>,
{
    let boundary = load_boundary_by_id_in_tx(&mut *tx, &grant.boundary_id)
        .await
        .map_err(|_| GrantRefused {
            violations: vec![reasonbraid_core::BoundaryViolation {
                field: "boundary_id",
                detail: format!("boundary `{}` does not exist", grant.boundary_id),
            }],
        })?;
    let violations = grant_exceeds_boundary(&boundary, grant);
    if !violations.is_empty() {
        return Err(GrantRefused { violations });
    }

    let (subject_kind, subject_id) = subject_parts(&grant.subject);
    sqlx::query(
        "INSERT INTO authority_grants \
         (grant_id, boundary_id, tenant_id, issuer, subject_kind, subject_id, actions, selector, \
          risk_ceiling, spend_limits, delegable, valid_from, expires_at, status) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)",
    )
    .bind(&grant.grant_id)
    .bind(&grant.boundary_id)
    .bind(grant.tenant_id.to_string())
    .bind(grant.issuer.to_string())
    .bind(subject_kind)
    .bind(subject_id)
    .bind(
        serde_json::to_value(grant.actions.iter().map(|a| a.as_str()).collect::<Vec<_>>())
            .expect("actions serialize"),
    )
    .bind(serde_json::to_value(&grant.selector).expect("selector serializes"))
    .bind(grant.risk_ceiling.as_str())
    .bind(grant.spend_limits.as_ref())
    .bind(grant.delegable)
    .bind(grant.valid_from)
    .bind(grant.expires_at)
    .bind(grant.status.as_str())
    .execute(&mut *tx)
    .await
    .map_err(|_| GrantRefused {
        violations: vec![reasonbraid_core::BoundaryViolation {
            field: "grant",
            detail: format!("grant `{}` could not be stored", grant.grant_id),
        }],
    })?;
    Ok(())
}

fn subject_parts(subject: &GrantSubject) -> (&'static str, String) {
    match subject {
        GrantSubject::Human(id) => ("human", id.to_string()),
        GrantSubject::Role(id) => ("role", id.to_string()),
    }
}

fn subject_from_parts(kind: &str, id: &str) -> Option<GrantSubject> {
    match kind {
        "human" => id.parse().ok().map(GrantSubject::Human),
        "role" => id.parse().ok().map(GrantSubject::Role),
        _ => None,
    }
}

type BoundaryRow = (
    String,
    String,
    String,
    String,
    Value,
    Value,
    String,
    Option<Value>,
    bool,
    i32,
    DateTime<Utc>,
    DateTime<Utc>,
    String,
    String,
    String,
);

fn boundary_from_row(row: BoundaryRow) -> Option<EnrollmentAuthorityBoundary> {
    let (
        boundary_id,
        tenant_id,
        parent_or_root_authority,
        target_owner,
        permitted_actions,
        permitted_domains,
        risk_ceiling,
        spend_ceiling,
        delegable,
        max_delegation_depth,
        valid_from,
        expires_at,
        charter_digest,
        policy_version,
        status,
    ) = row;
    Some(EnrollmentAuthorityBoundary {
        boundary_id,
        tenant_id: tenant_id.parse().ok()?,
        parent_or_root_authority,
        target_owner,
        permitted_actions: serde_json::from_value(permitted_actions).ok()?,
        permitted_domains: serde_json::from_value(permitted_domains).ok()?,
        risk_ceiling: risk_ceiling.parse::<RiskClass>().ok()?,
        spend_ceiling,
        delegable,
        max_delegation_depth: max_delegation_depth as u32,
        valid_from,
        expires_at,
        charter_digest,
        policy_version,
        status: status.parse::<BoundaryStatus>().ok()?,
    })
}

type GrantRow = (
    String,
    String,
    String,
    String,
    String,
    String,
    Value,
    Value,
    String,
    Option<Value>,
    bool,
    DateTime<Utc>,
    DateTime<Utc>,
    String,
);

fn grant_from_row(row: GrantRow) -> Option<AuthorityGrant> {
    let (
        grant_id,
        boundary_id,
        tenant_id,
        issuer,
        subject_kind,
        subject_id,
        actions,
        selector,
        risk_ceiling,
        spend_limits,
        delegable,
        valid_from,
        expires_at,
        status,
    ) = row;
    Some(AuthorityGrant {
        grant_id,
        boundary_id,
        tenant_id: tenant_id.parse().ok()?,
        issuer: issuer.parse().ok()?,
        subject: subject_from_parts(&subject_kind, &subject_id)?,
        actions: serde_json::from_value(actions).ok()?,
        selector: serde_json::from_value(selector).ok()?,
        risk_ceiling: risk_ceiling.parse::<RiskClass>().ok()?,
        spend_limits,
        delegable,
        valid_from,
        expires_at,
        status: status.parse::<GrantStatus>().ok()?,
    })
}

/// The executor-generic boundary loader (`PHASE-0.6.1`): the grant path reads the
/// boundary inside the caller's transaction.
pub(crate) async fn load_boundary_by_id_in_tx<'e, E>(
    mut tx: E,
    boundary_id: &str,
) -> Result<EnrollmentAuthorityBoundary, sqlx::Error>
where
    E: std::ops::DerefMut,
    for<'c> &'c mut <E as std::ops::Deref>::Target: sqlx::Executor<'c, Database = sqlx::Postgres>,
{
    let row: BoundaryRow = sqlx::query_as(
        "SELECT boundary_id, tenant_id, parent_or_root_authority, target_owner, permitted_actions, \
                permitted_domains, risk_ceiling, spend_ceiling, delegable, max_delegation_depth, \
                valid_from, expires_at, charter_digest, policy_version, status \
         FROM enrollment_boundaries WHERE boundary_id = $1",
    )
    .bind(boundary_id)
    .fetch_one(&mut *tx)
    .await?;
    Ok(boundary_from_row(row).expect("stored boundary parses"))
}

/// Load a tenant's ACTIVE enrollment boundary, if one exists (`PHASE-0.6.1` enroll
/// path for existing tenants).
pub(crate) async fn load_active_boundary_for_tenant(
    pool: &PgPool,
    tenant_id: &reasonbraid_core::TenantId,
) -> Result<Option<EnrollmentAuthorityBoundary>, sqlx::Error> {
    let row: Option<BoundaryRow> = sqlx::query_as(
        "SELECT boundary_id, tenant_id, parent_or_root_authority, target_owner, permitted_actions, \
                permitted_domains, risk_ceiling, spend_ceiling, delegable, max_delegation_depth, \
                valid_from, expires_at, charter_digest, policy_version, status \
         FROM enrollment_boundaries WHERE tenant_id = $1 AND status = 'active'",
    )
    .bind(tenant_id.to_string())
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|r| boundary_from_row(r).expect("stored boundary parses")))
}

// ── Evaluation ──────────────────────────────────────────────────────────────────

/// The deterministic evaluation: which decision does the dev profile reach for this
/// context, and why.
fn evaluate(
    boundary: Option<&EnrollmentAuthorityBoundary>,
    grant: Option<&AuthorityGrant>,
    authz: &CommandAuthz,
    at: DateTime<Utc>,
) -> Decision {
    let Some(boundary) = boundary else {
        return Decision::Denied {
            reason: "the tenant has no active enrollment boundary".to_string(),
        };
    };
    if !boundary_active_at(boundary, at) {
        return Decision::Denied {
            reason: "the enrollment boundary is not active".to_string(),
        };
    }
    let Some(grant) = grant else {
        return Decision::Denied {
            reason: "no applicable grant: tenant membership alone grants no authority".to_string(),
        };
    };
    if !grant_active_at(grant, at) {
        return Decision::Denied {
            reason: "the grant is revoked or outside its validity window".to_string(),
        };
    }
    if !grant.actions.contains(&authz.action) {
        return Decision::Denied {
            reason: format!(
                "action `{}` is not granted to this principal",
                authz.action.as_str()
            ),
        };
    }
    match (&authz.target, &grant.selector) {
        (ResourceTarget::Tenant { .. }, _) => {}
        (
            ResourceTarget::Thread {
                tenant_id,
                thread_id,
            },
            TargetSelector::TenantWide,
        ) if *tenant_id == grant.tenant_id => {}
        (
            ResourceTarget::Thread {
                tenant_id,
                thread_id,
            },
            TargetSelector::Threads { threads },
        ) if *tenant_id == grant.tenant_id && threads.contains(thread_id) => {}
        (ResourceTarget::Thread { .. }, _) => {
            return Decision::Denied {
                reason: "the target thread is outside the grant's scope".to_string(),
            };
        }
    }
    if *authz.target.tenant_id() != grant.tenant_id {
        return Decision::Denied {
            reason: "the target tenant is outside the grant's scope".to_string(),
        };
    }
    if authz.action == GrantAction::ThreadCreate
        && matches!(authz.target, ResourceTarget::Thread { .. })
    {
        return Decision::Denied {
            reason: "thread_create targets the tenant, not a thread".to_string(),
        };
    }
    // Defense in depth: the subset rule is enforced at grant creation AND re-checked
    // at every evaluation.
    let violations = grant_exceeds_boundary(boundary, grant);
    if !violations.is_empty() {
        return Decision::Denied {
            reason: format!(
                "the grant exceeds its boundary: {}",
                violations
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>()
                    .join("; ")
            ),
        };
    }
    Decision::Allowed
}

/// Authorize one command: evaluate, compute the policy digest, write the audit
/// record, and return the decision. Denials are recorded exactly like allowances.
pub async fn authorize(
    pool: &PgPool,
    authz: &CommandAuthz,
    at: DateTime<Utc>,
) -> Result<AuthorizationOutcome, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let outcome = authorize_in_tx(&mut *tx, authz, at).await?;
    tx.commit().await?;
    Ok(outcome)
}

/// The transactional body of [`authorize`] — shared with [`apply_authorized_command`]
/// so the audit record and the command's writes are one commit.
pub(crate) async fn authorize_in_tx<E>(
    mut pool: E,
    authz: &CommandAuthz,
    at: DateTime<Utc>,
) -> Result<AuthorizationOutcome, sqlx::Error>
where
    E: std::ops::DerefMut,
    for<'c> &'c mut <E as std::ops::Deref>::Target: sqlx::Executor<'c, Database = sqlx::Postgres>,
{
    let tenant = authz.target.tenant_id().to_string();

    // The two lookups, inlined on the executor (a `&PgPool` executor and a
    // `&mut Transaction` executor have different shapes, so the shared loaders stay
    // pool-based and the transactional path carries its own queries).
    let boundary_row: Option<BoundaryRow> = sqlx::query_as(
        "SELECT boundary_id, tenant_id, parent_or_root_authority, target_owner, permitted_actions, \
                permitted_domains, risk_ceiling, spend_ceiling, delegable, max_delegation_depth, \
                valid_from, expires_at, charter_digest, policy_version, status \
         FROM enrollment_boundaries WHERE tenant_id = $1 AND status = 'active'",
    )
    .bind(&tenant)
    .fetch_optional(&mut *pool)
    .await?;
    let boundary = boundary_row.map(|r| boundary_from_row(r).expect("stored boundary parses"));

    let (subject_kind, subject_id) = subject_parts(&authz.principal);
    let grant_row: Option<GrantRow> = sqlx::query_as(
        "SELECT grant_id, boundary_id, tenant_id, issuer, subject_kind, subject_id, actions, \
                selector, risk_ceiling, spend_limits, delegable, valid_from, expires_at, status \
         FROM authority_grants \
         WHERE tenant_id = $1 AND subject_kind = $2 AND subject_id = $3 AND status = 'active' \
         ORDER BY valid_from DESC LIMIT 1",
    )
    .bind(&tenant)
    .bind(subject_kind)
    .bind(subject_id)
    .fetch_optional(&mut *pool)
    .await?;
    let grant = grant_row.map(|r| grant_from_row(r).expect("stored grant parses"));

    let decision = evaluate(boundary.as_ref(), grant.as_ref(), authz, at);
    let record_id = AuthorizationRecordId::new().to_string();

    let digest = policy_digest(
        boundary.as_ref(),
        grant.as_ref(),
        Some(&authz.principal),
        authz.action,
        &authz.target,
        &decision,
    );

    let (target_kind, target_tenant, target_thread) = match &authz.target {
        ResourceTarget::Tenant { tenant_id } => ("tenant", tenant_id.to_string(), None),
        ResourceTarget::Thread {
            tenant_id,
            thread_id,
        } => ("thread", tenant_id.to_string(), Some(thread_id.to_string())),
    };
    let (subject_kind, subject_id) = match &authz.delegate_subject {
        Some(s) => {
            let (k, i) = subject_parts(s);
            (Some(k), Some(i))
        }
        None => (None, None),
    };
    let reason = match &decision {
        Decision::Allowed => None,
        Decision::Denied { reason } => Some(reason.clone()),
    };

    sqlx::query(
        "INSERT INTO authorization_records \
         (record_id, tenant_id, actor, subject_kind, subject_id, boundary_id, grant_id, action, \
          target_kind, target_tenant, target_thread, decision, reason, policy_digest, \
          policy_version, decided_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)",
    )
    .bind(&record_id)
    .bind(&tenant)
    .bind(authz.actor.to_string())
    .bind(subject_kind)
    .bind(subject_id)
    .bind(boundary.as_ref().map(|b| b.boundary_id.as_str()))
    .bind(grant.as_ref().map(|g| g.grant_id.as_str()))
    .bind(authz.action.as_str())
    .bind(target_kind)
    .bind(target_tenant)
    .bind(target_thread)
    .bind(decision.as_str())
    .bind(reason)
    .bind(&digest)
    .bind(
        boundary
            .as_ref()
            .map(|b| b.policy_version.as_str())
            .unwrap_or("no-policy"),
    )
    .bind(at)
    .execute(&mut *pool)
    .await?;

    Ok(match decision {
        Decision::Allowed => AuthorizationOutcome::Allowed { record_id },
        Decision::Denied { reason } => AuthorizationOutcome::Denied { reason, record_id },
    })
}

/// Authorize AND apply a command in ONE transaction: the audit record (allow or
/// deny) and the WP2 state/event/idempotency/outbox writes commit together. A denial
/// commits the denial record and applies NOTHING.
pub async fn apply_authorized_command(
    pool: &PgPool,
    authz: &CommandAuthz,
    cmd: &Command,
) -> Result<CommandOutcome, AuthorizedApplyError> {
    let mut tx = pool.begin().await?;
    match authorize_in_tx(&mut *tx, authz, Utc::now()).await? {
        AuthorizationOutcome::Allowed { .. } => {
            let outcome = tx::apply_command_in_tx(&mut *tx, cmd).await?;
            tx.commit().await?;
            Ok(outcome)
        }
        AuthorizationOutcome::Denied { reason, record_id } => {
            // The denial record IS durable; the command wrote nothing.
            tx.commit().await?;
            Err(AuthorizedApplyError::Denied { reason, record_id })
        }
    }
}

/// The audit record assembled from a stored row (the acceptance's "every command
/// records…" is verified by reading it back).
pub async fn load_authorization_record(
    pool: &PgPool,
    record_id: &str,
) -> Result<Option<AuthorizationDecisionRecord>, sqlx::Error> {
    type Row = (
        String,
        String,
        String,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        String,
        String,
        String,
        Option<String>,
        String,
        Option<String>,
        String,
        String,
        DateTime<Utc>,
    );
    let row: Option<Row> = sqlx::query_as(
        "SELECT record_id, tenant_id, actor, subject_kind, subject_id, boundary_id, grant_id, \
                action, target_kind, target_tenant, target_thread, decision, reason, \
                policy_digest, policy_version, decided_at \
         FROM authorization_records WHERE record_id = $1",
    )
    .bind(record_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(
        |(
            record_id,
            tenant_id,
            actor,
            subject_kind,
            subject_id,
            boundary_id,
            grant_id,
            action,
            target_kind,
            target_tenant,
            target_thread,
            decision,
            reason,
            policy_digest,
            policy_version,
            decided_at,
        )| {
            let decision = match decision.as_str() {
                "allowed" => Decision::Allowed,
                _ => Decision::Denied {
                    reason: reason.unwrap_or_default(),
                },
            };
            let target = match target_kind.as_str() {
                "thread" => ResourceTarget::Thread {
                    tenant_id: target_tenant.parse().expect("stored tenant id parses"),
                    thread_id: target_thread
                        .as_deref()
                        .expect("thread target has a thread id")
                        .parse()
                        .expect("stored thread id parses"),
                },
                _ => ResourceTarget::Tenant {
                    tenant_id: target_tenant.parse().expect("stored tenant id parses"),
                },
            };
            AuthorizationDecisionRecord {
                record_id: record_id.parse().expect("stored record id parses"),
                tenant_id: tenant_id.parse().expect("stored tenant id parses"),
                actor: actor.parse().expect("stored actor id parses"),
                subject: subject_kind
                    .as_deref()
                    .and_then(|k| subject_from_parts(k, subject_id.as_deref().unwrap_or_default())),
                boundary_id,
                grant_id,
                action: action.parse::<GrantAction>().expect("stored action parses"),
                target,
                decision,
                policy_digest,
                policy_version,
                decided_at,
            }
        },
    ))
}
