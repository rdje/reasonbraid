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
    boundary_active_at, delegation_scope_is_subset, grant_active_at, grant_exceeds_boundary,
    policy_digest, ActorPrincipalId, AuthorityGrant, AuthorizationEvaluation,
    AuthorizationRecordId, BoundaryStatus, Decision, EnrollmentAuthorityBoundary, GrantAction,
    GrantStatus, GrantSubject, ResourceTarget, RiskClass, TargetSelector,
};
use serde_json::Value;
use sqlx::PgPool;

use crate::tx::{self, ApplyError, Command, CommandOutcome};

#[cfg(test)]
mod evaluation_tests;
mod issuance;
pub(crate) mod transaction;

pub use issuance::{create_boundary, create_grant};
pub(crate) use issuance::{create_grant_in_guard, load_active_boundary_in_guard};
pub use transaction::GuardError as AuthorityTransactionError;
pub(crate) use transaction::{transact_with_error, GuardMode, Limits, TenantTransaction};

mod breaker;
mod conditions;
mod effects;
mod federation_admin;
mod node_admin;
mod profile_admin;
mod records;
mod revocation;
mod selection;

pub(crate) use breaker::{administer_breaker_in_one_transaction, BreakerCommand, BreakerResult};
pub use effects::{load_tenant_administrative_effect, record_administrative_effect_in_tx};
pub(crate) use federation_admin::{
    accept_direction_in_one_transaction, propose_direction_in_one_transaction,
    revoke_direction_in_one_transaction, AcceptResult, ProposeResult, RevokeDirectionResult,
};
pub(crate) use node_admin::{
    adjudicate_ambiguous_attempt_in_one_transaction, issue_enrollment_token_in_one_transaction,
    prune_node_inbox_in_one_transaction, quarantine_command_in_one_transaction,
    replay_command_in_one_transaction, replay_command_with_possible_duplicate_in_one_transaction,
    revoke_node_in_one_transaction, AdjudicationResult, DuplicateReplayResult, NodeRevokeResult,
    PruneResult, QuarantineResult, ReplayResult, TokenIssueResult,
};
pub(crate) use profile_admin::{
    attest_capability_in_one_transaction, import_card_in_one_transaction, AttestResult,
    CardImportResult,
};
pub use records::load_authorization_record;
pub(crate) use records::{load_tenant_authorization_record, load_thread_authorization_records};
pub(crate) use revocation::{revoke_in_one_transaction, RevocationResult, RevocationTarget};

/// One request's delegation: the subject the actor acts on behalf of, and the
/// attenuation the delegator applied — held together because neither is
/// meaningful alone (`SIGNOFF-REPAIR.3.4.1.1`).
///
/// ⛔ These were two independent `Option` fields on [`CommandAuthz`], which made
/// a subject-without-scope WRITABLE. The §16.3.1 widening check read the scope
/// under `if let Some(scope)`, so that state would have delegated with the
/// subject's FULL grant selector — no attenuation at all. It was never
/// reachable, because the one producer always set both; pairing them removes
/// the state rather than relying on every future producer to remember.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Delegation {
    /// The original subject the actor acts on behalf of.
    pub subject: GrantSubject,
    /// The attenuation the delegator applied (the `.1.4.2` scope): the
    /// request's target must be WITHIN it and it must be within the subject's
    /// grant selector (§16.3.1 — a delegate cannot widen).
    pub scope: TargetSelector,
}

/// The authorization context of one command: the authenticated actor, the grant
/// holder (the actor, or the delegating subject's principal), the delegation
/// when one applies, and the requested action/target.
pub struct CommandAuthz {
    /// Who the request authenticates as (the audit actor).
    pub actor: ActorPrincipalId,
    /// The grant holder — the principal whose grant is evaluated.
    pub principal: GrantSubject,
    /// Present exactly when the actor acts on another subject's behalf.
    pub delegation: Option<Delegation>,
    pub action: GrantAction,
    pub target: ResourceTarget,
}

impl CommandAuthz {
    /// The subject whose grant is evaluated: the delegated one when the request
    /// delegates, else the caller's own principal.
    pub(crate) fn evaluated_subject(&self) -> &GrantSubject {
        self.delegation
            .as_ref()
            .map(|d| &d.subject)
            .unwrap_or(&self.principal)
    }
}

/// The outcome of [`authorize_in_tx`]. An allowance carries what the delivery
/// needs (`.1.5.2`, ADR-008): the record id, the policy digest the record bound,
/// and the decision time (the node-side freshness TTL runs from it).
#[derive(Debug, Clone, PartialEq)]
pub enum AuthorizationOutcome {
    Allowed {
        record_id: String,
        policy_digest: String,
        decided_at: DateTime<Utc>,
    },
    Denied {
        reason: String,
        record_id: String,
    },
}

/// The message a DOMAIN grant refusal shows, composed once so the HTTP response
/// and the administrative effect record cannot drift apart.
///
/// `None` for the storage and transaction variants: those are not refusals of an
/// admitted operation, they roll it back, and inventing a message for them would
/// be exactly the disagreement the effect record exists to prevent
/// (`SIGNOFF-REPAIR.3.3.4.11.3`).
pub(crate) fn grant_refusal_message(
    error: &GrantCreateError,
    refusal_context: &str,
) -> Option<String> {
    match error {
        GrantCreateError::MissingBoundary { boundary_id } => {
            Some(format!("boundary `{boundary_id}` does not exist"))
        }
        GrantCreateError::Refused(GrantRefused { violations }) => Some(format!(
            "{refusal_context}: {}",
            violations
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join("; ")
        )),
        GrantCreateError::BoundaryNotLive { boundary_id } => Some(format!(
            "boundary `{boundary_id}` is not live for grant issuance"
        )),
        GrantCreateError::Storage(_) | GrantCreateError::Transaction(_) => None,
    }
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

/// Failure to create a grant. Only [`Self::Refused`] describes a proved
/// structural ceiling violation; missing or malformed authority is distinct.
#[derive(Debug)]
#[non_exhaustive]
pub enum GrantCreateError {
    /// The named parent was absent when read. No grant was inserted.
    MissingBoundary { boundary_id: String },
    /// The supplied grant exceeds its actual parent's structural ceiling.
    Refused(GrantRefused),
    /// The structurally valid parent is outside its live window at the guarded
    /// issuance evaluation. No new grant is inserted by this refusal.
    BoundaryNotLive { boundary_id: String },
    /// A guarded transaction failed outside an ordinary storage operation.
    /// In particular, commit failures must not be interpreted as confirmed rollback.
    Transaction(AuthorityTransactionError),
    /// Connection, query, insertion or stored-data decoding failed. The
    /// original SQLx error remains available through [`std::error::Error::source`].
    Storage(sqlx::Error),
}

impl std::fmt::Display for GrantCreateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingBoundary { boundary_id } => {
                write!(f, "boundary `{boundary_id}` does not exist")
            }
            Self::Refused(error) => std::fmt::Display::fmt(error, f),
            Self::BoundaryNotLive { boundary_id } => {
                write!(f, "boundary `{boundary_id}` is not live for grant issuance")
            }
            Self::Transaction(error) => std::fmt::Display::fmt(error, f),
            Self::Storage(_) => f.write_str("grant creation storage failure"),
        }
    }
}

impl std::error::Error for GrantCreateError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::MissingBoundary { .. } | Self::BoundaryNotLive { .. } => None,
            Self::Refused(error) => Some(error),
            Self::Transaction(error) => Some(error),
            Self::Storage(error) => Some(error),
        }
    }
}

impl From<sqlx::Error> for GrantCreateError {
    fn from(error: sqlx::Error) -> Self {
        Self::Storage(error)
    }
}

impl From<AuthorityTransactionError> for GrantCreateError {
    fn from(error: AuthorityTransactionError) -> Self {
        match error {
            // Keep the original SQLx cause directly available to existing callers.
            AuthorityTransactionError::Storage(error) => Self::Storage(error),
            error => Self::Transaction(error),
        }
    }
}

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

/// Boundary row insertion on the supplied executor; this does not acquire a
/// tenant guard. The standalone service and complete enrollment transaction
/// supply their scoped guarded connection.
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

// `create_grant_unordered_in_tx` used to live here: a temporary bridge that
// loaded a boundary, checked the grant structurally and inserted it on the
// caller's executor, with NO tenant guard and none of the guarded service's
// issuance-time liveness check. Its own comment named `.3.3.4.11` as the leaf
// that would migrate its last caller. `SIGNOFF-REPAIR.3.3.4.11.3` did: the card
// import now creates its grant with `create_grant_in_guard` inside the same
// exclusive-guard transaction that admits the caller and writes every row. The
// bridge is removed rather than left as a second, unordered way to issue
// authority — the same disposal `.8` gave the two superseded revocation services.

/// Row insertion only: callers must establish the relevant authority contract.
async fn insert_grant_row<E>(mut tx: E, grant: &AuthorityGrant) -> Result<(), sqlx::Error>
where
    E: std::ops::DerefMut,
    for<'c> &'c mut <E as std::ops::Deref>::Target: sqlx::Executor<'c, Database = sqlx::Postgres>,
{
    let (subject_kind, subject_id) = subject_parts(&grant.subject);
    sqlx::query(
        "INSERT INTO authority_grants \
         (grant_id, boundary_id, tenant_id, issuer, subject_kind, subject_id, actions, selector, \
          risk_ceiling, spend_limits, delegable, valid_from, expires_at, status, auto_bounds, \
          decision_rule_constraints, conditions) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)",
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
    .bind(
        grant
            .auto_bounds
            .as_ref()
            .map(|bounds| serde_json::to_value(bounds).expect("auto bounds serialize")),
    )
    .bind(
        grant
            .decision_rule_constraints
            .as_ref()
            .map(|rules| serde_json::to_value(rules).expect("rule names serialize")),
    )
    .bind(
        grant
            .conditions
            .as_ref()
            .map(|conditions| serde_json::to_value(conditions).expect("conditions serialize")),
    )
    .execute(&mut *tx)
    .await?;
    Ok(())
}

/// Is this grant LIVE right now — `active`, BEGUN, and not expired?
///
/// ⭐ The one definition of that question (`SIGNOFF-REPAIR.9.3.1`). Five sites
/// asked it in four different spellings: `policy::register` checked `status`
/// alone, `policy::resolve`, `corrections` and `deployments` added expiry, and
/// `lifecycle` added expiry plus the subject. **None of the five consulted
/// `valid_from`** — `git grep -n valid_from` over those four modules returned
/// rc=1 — so a grant that had not begun was as good as a live one everywhere.
///
/// ⛔ The `expires_at IS NULL` branch four of them carried is NOT reproduced:
/// `migrations/0004_authority.sql` declares the column `NOT NULL`, so that arm
/// was dead, and copying it forward would keep teaching the next reader that
/// the column is optional.
/// ⚠️ Executor-generic since `SIGNOFF-REPAIR.6.1.5.4`, so the one definition
/// serves both a pool and a CONNECTION. `policy::register` now runs inside the
/// site act's transaction, and asking this question on a second connection would
/// read the grants from outside the transaction that is about to commit the
/// write it gates.
/// ⭐ `SIGNOFF-REPAIR.9.3.4.2` adds `wanted`: does the grant's action set COVER
/// the verb being attempted? `None` asks only about liveness, and there is
/// exactly one caller entitled to it — `policy::resolve`, which asks this of
/// every LOADED policy's owner rather than of a caller's citation, so there is
/// no verb to cover. Every other site names its action.
/// 🔴 `SIGNOFF-REPAIR.11.55`: LIVE includes the BOUNDARY. Both checks read the
/// grant alone, so a grant whose enrollment boundary had been revoked, had
/// expired or had not begun still proved seven policy verbs (approvals,
/// corrections, both deployment verbs, the publication grant, registration and
/// resolution's owner check), while the guarded evaluator refused it and
/// `authority.md` says a revoked boundary freezes the next administrative
/// write. Measured live before the repair: after the owner revoked the
/// boundary, an approval under it was accepted and recorded.
pub(crate) async fn grant_is_live<'e, E>(
    executor: E,
    grant_id: &str,
    wanted: Option<GrantAction>,
) -> Result<bool, sqlx::Error>
where
    E: sqlx::PgExecutor<'e>,
{
    let actions: Option<Value> = sqlx::query_scalar(LIVE_GRANT_ACTIONS)
        .bind(grant_id)
        .fetch_optional(executor)
        .await?;
    Ok(covers(actions, wanted))
}

/// The one liveness predicate both checks read (`SIGNOFF-REPAIR.11.55`): the
/// grant is active and inside its window, and so is the enrollment boundary it
/// was issued under, the same two facts the guarded evaluator requires. A
/// grant with no boundary row cannot be live, which the foreign key already
/// guarantees and the inner join restates.
const LIVE_GRANT_ACTIONS: &str = "SELECT g.actions FROM authority_grants g \
     JOIN enrollment_boundaries b ON b.boundary_id = g.boundary_id \
     WHERE g.grant_id = $1 AND g.status = 'active' \
     AND g.valid_from <= now() AND g.expires_at > now() \
     AND b.status = 'active' AND b.valid_from <= now() AND b.expires_at > now()";

/// The coverage decision over a grant row's stored `actions`, shared by both
/// predicates above (`SIGNOFF-REPAIR.9.3.4.2`) — and it defers to
/// `reasonbraid_core::action_covered` rather than restating the subsumption
/// rule, in SQL or anywhere else. A `tenant_admin` grant covering an
/// administrative verb is ONE rule with ONE definition (`.9.3.4.1`).
///
/// ⛔ A row whose `actions` array does not parse covers NOTHING. That is
/// fail-closed and it matches `boundary_from_row`, which returns `None` on the
/// same failure: a build that cannot read a stored authority must not act on a
/// guess about it. ⚠️ `Vec<GrantAction>` is strict, so ONE unknown name —
/// a wire name written by a newer build — makes the whole grant unreadable and
/// therefore inert here. Fail-closed in the right direction, and stated rather
/// than discovered.
fn covers(actions: Option<Value>, wanted: Option<GrantAction>) -> bool {
    let Some(actions) = actions else {
        return false;
    };
    let Some(wanted) = wanted else {
        return true;
    };
    match serde_json::from_value::<Vec<GrantAction>>(actions) {
        Ok(held) => reasonbraid_core::action_covered(&held, wanted),
        Err(_) => false,
    }
}

/// Is this grant live, HELD BY this principal, and does it COVER this verb?
///
/// The predicate for every surface where a CALLER cites an authority. Citing
/// one is not holding one: the dev enrolment mints `grt_<principal_id>`, so a
/// grant id is derivable from any principal id a caller has seen, and a check
/// that only asks whether the grant exists admits anyone who can name it.
///
/// ⭐ `SIGNOFF-REPAIR.9.3.4.2` adds the THIRD term, and it is the one `.9.3.1`
/// named without being able to ask: holding is not covering. Until the
/// administrative verbs existed (`.9.3.4.1`) there was nothing to ask it of —
/// every administrative grant carried the single `tenant_admin` action, so a
/// grant minted to record a correction equally authorized publishing.
///
/// ⚠️ Executor-generic since `SIGNOFF-REPAIR.9.1.2`, for the reason
/// [`grant_is_live`] became so: `policy::register` asks it inside the site act's
/// transaction, and asking on a second connection would read the grants from
/// outside the transaction that is about to commit the write it gates.
pub(crate) async fn grant_held_by<'e, E>(
    executor: E,
    grant_id: &str,
    principal: &GrantSubject,
    wanted: GrantAction,
) -> Result<bool, sqlx::Error>
where
    E: sqlx::PgExecutor<'e>,
{
    let (kind, id) = subject_parts(principal);
    let held = format!("{LIVE_GRANT_ACTIONS} AND g.subject_kind = $2 AND g.subject_id = $3");
    let actions: Option<Value> = sqlx::query_scalar(&held)
        .bind(grant_id)
        .bind(kind)
        .bind(id)
        .fetch_optional(executor)
        .await?;
    Ok(covers(actions, Some(wanted)))
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
        max_delegation_depth: max_delegation_depth.try_into().ok()?,
        valid_from,
        expires_at,
        charter_digest,
        policy_version,
        status: status.parse::<BoundaryStatus>().ok()?,
    })
}

/// One stored grant row, decoded by column name. A named struct rather than a
/// tuple because sqlx decodes tuples of at most sixteen columns and the grant
/// carries seventeen since `conditions` (`SIGNOFF-REPAIR.11.4.7.2.1.5.4.3`).
#[derive(sqlx::FromRow)]
struct GrantRow {
    grant_id: String,
    boundary_id: String,
    tenant_id: String,
    issuer: String,
    subject_kind: String,
    subject_id: String,
    actions: Value,
    selector: Value,
    risk_ceiling: String,
    spend_limits: Option<Value>,
    auto_bounds: Option<Value>,
    decision_rule_constraints: Option<Value>,
    conditions: Option<Value>,
    delegable: bool,
    valid_from: DateTime<Utc>,
    expires_at: DateTime<Utc>,
    status: String,
}

fn grant_from_row(row: GrantRow) -> Option<AuthorityGrant> {
    let GrantRow {
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
        auto_bounds,
        decision_rule_constraints,
        conditions,
        delegable,
        valid_from,
        expires_at,
        status,
    } = row;
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
        // A stored block that is not the typed struct is a malformed grant, not
        // an unbounded one: `None` here fails the whole row, as every other
        // field does.
        auto_bounds: auto_bounds.map(serde_json::from_value).transpose().ok()?,
        decision_rule_constraints: decision_rule_constraints
            .map(serde_json::from_value)
            .transpose()
            .ok()?,
        // A stored condition list that is not the typed vocabulary is a
        // malformed grant, not an unconditional one: the row fails whole.
        conditions: conditions.map(serde_json::from_value).transpose().ok()?,
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
    boundary_from_row(row)
        .ok_or_else(|| sqlx::Error::Protocol("stored enrollment boundary is malformed".into()))
}

// ── Evaluation ──────────────────────────────────────────────────────────────────

/// The narrowly approved inspection exception ignores only boundary status. The
/// original boundary remains unchanged for source attribution. Even a malformed
/// internal context cannot use this purpose for delegation or another target.
fn evaluate_tenant_admin_read(
    boundary: Option<&EnrollmentAuthorityBoundary>,
    grant: Option<&AuthorityGrant>,
    authz: &CommandAuthz,
    at: DateTime<Utc>,
) -> Decision {
    if authz.action != GrantAction::TenantAdmin
        || !matches!(authz.target, ResourceTarget::Tenant { .. })
        || authz.delegation.is_some()
    {
        return Decision::Denied {
            reason: "the frozen-read exception requires direct tenant administration inspection"
                .into(),
        };
    }
    let projected = boundary.map(|boundary| {
        let mut projected = boundary.clone();
        projected.status = BoundaryStatus::Active;
        projected
    });
    evaluate(projected.as_ref(), grant, authz, at)
}

/// Commit the admission for a named, direct own-tenant inspection. The actual
/// parent status and selected scope describe the read exception explicitly.
/// This transaction does not cover response queries or serialize revocation.
pub(crate) async fn authorize_tenant_admin_inspection(
    pool: &PgPool,
    principal: &GrantSubject,
    tenant_id: reasonbraid_core::TenantId,
    inspection: reasonbraid_core::TenantAdminInspection,
) -> Result<AuthorizationOutcome, sqlx::Error> {
    let mut tx = pool.begin().await?;
    // The tenant's authority guard, before any authority row is read
    // (`SIGNOFF-REPAIR.3.3.4.6`). Shared: inspections read authority and must
    // run concurrently with one another, while a revocation's exclusive mode
    // fences every admission that has not already selected its evidence. The
    // record this commits NAMES the selected parent's status and the grant's
    // scope, so without the guard that evidence could be read either side of a
    // writer changing it.
    transaction::acquire_in_tx(&mut tx, tenant_id, transaction::GuardMode::Shared).await?;
    // Sample the database clock AFTER the guard wait; time spent waiting for the
    // pool or for an authority writer is not part of this decision's validity,
    // and authority that ended during the wait must be evaluated as ended.
    let at: DateTime<Utc> = transaction::database_now_in_tx(&mut tx).await?;
    let authz = CommandAuthz {
        actor: reasonbraid_core::actor_handle_for_subject(principal),
        principal: principal.clone(),
        delegation: None,
        action: GrantAction::TenantAdmin,
        target: ResourceTarget::Tenant { tenant_id },
    };
    let selected = selection::select_authority_in_tx(
        &mut *tx,
        &authz,
        at,
        selection::EvaluationUse::TenantAdminRead,
    )
    .await?;
    let evaluation = AuthorizationEvaluation::TenantAdminInspection {
        principal: principal.clone(),
        inspection,
        boundary_status: selected.boundary.as_ref().map(|boundary| boundary.status),
        grant_selector: selected.grant.as_ref().map(|grant| grant.selector.clone()),
    };
    let outcome = record_selection_in_tx(&mut *tx, &authz, at, selected, evaluation).await?;
    tx.commit().await?;
    Ok(outcome)
}

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
    let expected_subject = authz.evaluated_subject();
    if &grant.subject != expected_subject {
        return Decision::Denied {
            reason: "the grant does not belong to the evaluated subject".to_string(),
        };
    }
    if *authz.target.tenant_id() != grant.tenant_id {
        return Decision::Denied {
            reason: "the target tenant is outside the grant's scope".to_string(),
        };
    }
    // `SIGNOFF-REPAIR.9.3.4.1`: the SHARED predicate, so a stored `tenant_admin`
    // means the same thing here as it does in the boundary checker below.
    if !reasonbraid_core::action_covered(&grant.actions, authz.action) {
        return Decision::Denied {
            reason: format!(
                "action `{}` is not granted to this principal",
                authz.action.as_str()
            ),
        };
    }
    // §4.2's `conditions[]` (`SIGNOFF-REPAIR.11.4.7.2.1.5.4.3`): every listed
    // condition holds at this admission, or the grant denies naming it — after
    // the action and before the target kind, because a condition is a fact
    // about the moment or the target that a covered action must still satisfy.
    if let Some(conditions) = &grant.conditions {
        for condition in conditions {
            if let Err(reason) = conditions::holds(condition, at) {
                return Decision::Denied { reason };
            }
        }
    }
    let target_kind_allowed = match authz.action {
        // ⚠️ `SIGNOFF-REPAIR.9.3.4.1`: the five administrative verbs join the
        // TENANT arm, because that is the target their surfaces act on — a
        // policy version, a proposal's approval, a publication, a correction
        // and a deployment target are all tenant-level records, and none of
        // them is reachable through a thread target.
        GrantAction::ThreadCreate
        | GrantAction::ThreadCreateAuto
        | GrantAction::TenantAdmin
        | GrantAction::PolicyVersionRegister
        | GrantAction::PolicyProposalApprove
        | GrantAction::PolicyPublicationWrite
        | GrantAction::PolicyCorrectionRecord
        | GrantAction::DeploymentTargetRegister => {
            matches!(authz.target, ResourceTarget::Tenant { .. })
        }
        GrantAction::ThreadInspect => true,
        GrantAction::ThreadInvite
        | GrantAction::ThreadContribute
        | GrantAction::ThreadClose
        | GrantAction::ThreadCancel
        | GrantAction::ThreadInvitationRespond
        | GrantAction::ThreadAdvanceRound => {
            matches!(authz.target, ResourceTarget::Thread { .. })
        }
    };
    if !target_kind_allowed {
        return Decision::Denied {
            reason: format!(
                "action `{}` does not support this target kind",
                authz.action
            ),
        };
    }
    if !delegation_scope_is_subset(&target_to_selector(&authz.target), &grant.selector) {
        return Decision::Denied {
            reason: "the target is outside the grant's scope".to_string(),
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

/// The LIVE standalone admission (`SIGNOFF-REPAIR.3.3.4.6`): the same selection
/// and audit record as [`authorize`], with the tenant's SHARED authority guard
/// held across it and the decision time sampled from the database AFTER that
/// wait.
///
/// [`authorize`] keeps its explicit timestamp and takes no guard, because it is
/// the standalone evaluation-time API the core controls drive with a chosen
/// instant; this is the entry point a live request uses, where the instant is
/// not the caller's to choose and the evidence must not be readable either side
/// of an authority writer.
///
/// ⛔ It opens its OWN transaction, so a caller must not already hold a guard on
/// another connection when it calls this — every production call site is an HTTP
/// handler that admits first and acts afterwards, in that order.
pub(crate) async fn authorize_guarded(
    pool: &PgPool,
    authz: &CommandAuthz,
) -> Result<AuthorizationOutcome, sqlx::Error> {
    let mut tx = pool.begin().await?;
    transaction::acquire_in_tx(
        &mut tx,
        *authz.target.tenant_id(),
        transaction::GuardMode::Shared,
    )
    .await?;
    let at = transaction::database_now_in_tx(&mut tx).await?;
    let outcome = authorize_in_tx(&mut *tx, authz, at).await?;
    tx.commit().await?;
    Ok(outcome)
}

/// The request's target as a selector, for the delegation subset check.
fn target_to_selector(target: &ResourceTarget) -> TargetSelector {
    match target {
        ResourceTarget::Tenant { .. } => TargetSelector::TenantWide,
        ResourceTarget::Thread { thread_id, .. } => TargetSelector::Threads {
            threads: vec![*thread_id],
        },
    }
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
    // Caller permission and delegated authority each choose a usable grant with
    // its own parent. The record continues to name the delegated authority source;
    // an absent source cannot borrow the caller's grant or a tenant-level boundary.
    let selected = match &authz.delegation {
        Some(_) => {
            let caller = CommandAuthz {
                actor: authz.actor,
                principal: authz.principal.clone(),
                delegation: None,
                action: authz.action,
                target: authz.target.clone(),
            };
            let caller_selection = selection::select_authority_in_tx(
                &mut *pool,
                &caller,
                at,
                selection::EvaluationUse::Command,
            )
            .await?;
            let mut source = selection::select_authority_in_tx(
                &mut *pool,
                authz,
                at,
                selection::EvaluationUse::Command,
            )
            .await?;
            if let Decision::Denied { reason } = caller_selection.decision {
                source.decision = Decision::Denied {
                    reason: format!("the caller's own authority failed: {reason}"),
                };
            }
            source
        }
        None => {
            selection::select_authority_in_tx(
                &mut *pool,
                authz,
                at,
                selection::EvaluationUse::Command,
            )
            .await?
        }
    };
    record_selection_in_tx(
        &mut *pool,
        authz,
        at,
        selected,
        AuthorizationEvaluation::BoundaryChecked {},
    )
    .await
}

/// The common durable record writer. Evaluation provenance is supplied by the
/// closed entrypoints; selected source references and policy inputs stay paired.
async fn record_selection_in_tx<E>(
    mut pool: E,
    authz: &CommandAuthz,
    at: DateTime<Utc>,
    selected: selection::AuthoritySelection,
    evaluation: AuthorizationEvaluation,
) -> Result<AuthorizationOutcome, sqlx::Error>
where
    E: std::ops::DerefMut,
    for<'c> &'c mut <E as std::ops::Deref>::Target: sqlx::Executor<'c, Database = sqlx::Postgres>,
{
    let tenant = authz.target.tenant_id().to_string();
    let selection::AuthoritySelection {
        boundary,
        grant,
        decision,
    } = selected;
    let record_id = AuthorizationRecordId::new().to_string();

    let digest = policy_digest(
        boundary.as_ref(),
        grant.as_ref(),
        Some(authz.evaluated_subject()),
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
    let (subject_kind, subject_id) = match &authz.delegation {
        Some(delegation) => {
            let (k, i) = subject_parts(&delegation.subject);
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
          policy_version, decided_at, evaluation) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)",
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
    .bind(sqlx::types::Json(evaluation))
    .execute(&mut *pool)
    .await?;

    Ok(match decision {
        Decision::Allowed => AuthorizationOutcome::Allowed {
            record_id,
            policy_digest: digest,
            decided_at: at,
        },
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
    // The tenant guard before the aggregate and idempotency locks, and database
    // time sampled after that wait (`SIGNOFF-REPAIR.3.3.4.4`). Shared: this
    // authorizes and applies, it does not mutate authority.
    transaction::acquire_in_tx(
        &mut tx,
        *authz.target.tenant_id(),
        transaction::GuardMode::Shared,
    )
    .await?;
    let now = transaction::database_now_in_tx(&mut tx).await?;
    match authorize_in_tx(&mut *tx, authz, now).await? {
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

// ── Revocation write paths (`.1.3.2`; the epoch bump is `.1.5.2`, ADR-008) ──

/// Bump the tenant's revocation epoch — the cached-decision invalidation signal.
/// MUST run in the same transaction as the revocation write it accompanies: the
/// epoch and the status change commit together, so a node holding a cached
/// decision recorded under the old epoch is invalidated the moment the
/// revocation is durable. Grant/boundary services supply their guarded connection;
/// the node-certificate caller remains an unordered bridge owned by `.3.3.4.10`.
pub(crate) async fn bump_revocation_epoch(
    tx: &mut sqlx::PgConnection,
    tenant_id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE tenants SET revocation_epoch = revocation_epoch + 1 WHERE tenant_id = $1")
        .bind(tenant_id)
        .execute(&mut *tx)
        .await?;
    Ok(())
}
