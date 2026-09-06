//! The development authority model (`PHASE-0.5.1`; `ROADMAP.md` §4.4): an
//! [`EnrollmentAuthorityBoundary`] is the root/parent-granted ceiling, an
//! [`AuthorityGrant`] is a scoped mandate under it, and every command's decision is an
//! [`AuthorizationRecord`] bound to the policy it was evaluated against.
//!
//! # The invariant this module enforces
//!
//! **A grant can never exceed its boundary** ([`grant_exceeds_boundary`]): actions,
//! risk ceiling, spend limits, delegation, and the validity window must each be a
//! subset of the applicable boundary. Tenant membership alone grants nothing — a
//! mandate exists only through a grant that names it, and administrative authority
//! (`TenantAdmin`) must be explicitly granted; it is never implied by other actions.
//!
//! # What is deliberately NOT here
//!
//! Authentication and credential issuance (`WP5`: "use development credentials; do not
//! build the final certificate issuer") — the evaluator receives an already-resolved
//! actor id. Crypto-signed records and full delegation chains are later phases; the
//! development profile trusts the control plane's own database as the authority
//! store. Policy engines (OPA/Cedar) remain §16.4 candidates for Phase 2.
//!
//! # The policy digest
//!
//! [`policy_digest`] is SHA-256 over a documented field-order input (boundary id,
//! charter digest, policy version, grant id, subject, action, target, decision) — so
//! an authorization record provably names WHICH policy it was decided under, and the
//! digest changes whenever any of those inputs changes. No secrets enter the digest.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::id::{
    ActorPrincipalId, AgentRoleId, AuthorizationRecordId, HumanPrincipalId, TenantId, ThreadId,
};

// ── Actions, risk, targets ─────────────────────────────────────────────────────

/// The actions a grant may authorize in the development profile (`KICKOFF.md` WP5:
/// create a thread, invite a named agent, contribute, inspect; `PHASE-0.6.1` adds
/// `thread_close` for the WP6 CLI flow; `PHASE-1.1.3` adds `thread_cancel` — the
/// abandonment terminal, distinct from a decided close).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GrantAction {
    ThreadCreate,
    ThreadInvite,
    ThreadContribute,
    ThreadInspect,
    /// Close a thread (the lifecycle authority; distinct from contributing to one).
    ThreadClose,
    /// Cancel a thread — the `open|closing → cancelled` terminal, with a reason.
    ThreadCancel,
    /// Administrative authority — NEVER implied by membership or other actions.
    TenantAdmin,
}

impl GrantAction {
    pub fn as_str(self) -> &'static str {
        match self {
            GrantAction::ThreadCreate => "thread_create",
            GrantAction::ThreadInvite => "thread_invite",
            GrantAction::ThreadContribute => "thread_contribute",
            GrantAction::ThreadInspect => "thread_inspect",
            GrantAction::ThreadClose => "thread_close",
            GrantAction::ThreadCancel => "thread_cancel",
            GrantAction::TenantAdmin => "tenant_admin",
        }
    }

    /// Parse a stored/wire name (the server persists actions as strings).
    pub fn from_wire_name(s: &str) -> Option<Self> {
        match s {
            "thread_create" => Some(GrantAction::ThreadCreate),
            "thread_invite" => Some(GrantAction::ThreadInvite),
            "thread_contribute" => Some(GrantAction::ThreadContribute),
            "thread_inspect" => Some(GrantAction::ThreadInspect),
            "thread_close" => Some(GrantAction::ThreadClose),
            "thread_cancel" => Some(GrantAction::ThreadCancel),
            "tenant_admin" => Some(GrantAction::TenantAdmin),
            _ => None,
        }
    }
}

impl std::fmt::Display for GrantAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A stored/wire name this build's authority registry does not know (preserved for
/// the diagnostic).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownAuthorityName(pub String);

impl std::fmt::Display for UnknownAuthorityName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "unknown authority name `{}`", self.0)
    }
}

impl std::error::Error for UnknownAuthorityName {}

macro_rules! from_str_via {
    ($ty:ty) => {
        impl std::str::FromStr for $ty {
            type Err = UnknownAuthorityName;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                <$ty>::from_wire_name(s).ok_or_else(|| UnknownAuthorityName(s.to_string()))
            }
        }
    };
}

/// A deterministic risk class (dev profile: ordered so a ceiling comparison is total).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskClass {
    Low,
    Medium,
    High,
}

impl RiskClass {
    pub fn as_str(self) -> &'static str {
        match self {
            RiskClass::Low => "low",
            RiskClass::Medium => "medium",
            RiskClass::High => "high",
        }
    }

    pub fn from_wire_name(s: &str) -> Option<Self> {
        match s {
            "low" => Some(RiskClass::Low),
            "medium" => Some(RiskClass::Medium),
            "high" => Some(RiskClass::High),
            _ => None,
        }
    }
}

/// A resource a command targets. `ThreadCreate` targets the tenant; the other actions
/// target a thread.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ResourceTarget {
    Tenant {
        tenant_id: TenantId,
    },
    Thread {
        tenant_id: TenantId,
        thread_id: ThreadId,
    },
}

impl ResourceTarget {
    pub fn tenant_id(&self) -> &TenantId {
        match self {
            ResourceTarget::Tenant { tenant_id } | ResourceTarget::Thread { tenant_id, .. } => {
                tenant_id
            }
        }
    }

    /// The canonical, digest-stable description (used in the policy digest input).
    pub fn describe(&self) -> String {
        match self {
            ResourceTarget::Tenant { tenant_id } => format!("tenant:{tenant_id}"),
            ResourceTarget::Thread {
                tenant_id,
                thread_id,
            } => {
                format!("thread:{thread_id}@tenant:{tenant_id}")
            }
        }
    }
}

/// Which resources a grant's actions reach: everything in the tenant, or a named
/// thread set.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum TargetSelector {
    TenantWide,
    /// (Struct-like on purpose: serde's tagged representation cannot carry a
    /// newtype variant wrapping a sequence.)
    Threads {
        threads: Vec<ThreadId>,
    },
}

// ── Boundary and grant ─────────────────────────────────────────────────────────

/// The enrollment ceiling (§4.4): root/parent-granted, visible to the enrolled target,
/// and the hard limit every grant and charter provision must stay within.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnrollmentAuthorityBoundary {
    pub boundary_id: String,
    pub tenant_id: TenantId,
    pub parent_or_root_authority: String,
    pub target_owner: String,
    pub permitted_actions: Vec<GrantAction>,
    pub permitted_domains: Vec<String>,
    pub risk_ceiling: RiskClass,
    pub spend_ceiling: Option<Value>,
    pub delegable: bool,
    pub max_delegation_depth: u32,
    pub valid_from: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub charter_digest: String,
    pub policy_version: String,
    pub status: BoundaryStatus,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BoundaryStatus {
    Active,
    Suspended,
    Revoked,
}

impl BoundaryStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            BoundaryStatus::Active => "active",
            BoundaryStatus::Suspended => "suspended",
            BoundaryStatus::Revoked => "revoked",
        }
    }

    pub fn from_wire_name(s: &str) -> Option<Self> {
        match s {
            "active" => Some(BoundaryStatus::Active),
            "suspended" => Some(BoundaryStatus::Suspended),
            "revoked" => Some(BoundaryStatus::Revoked),
            _ => None,
        }
    }
}

/// The grant subject: a durable human principal or an agent role (§8.1 — roles, not
/// models, hold authority).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum GrantSubject {
    Human(HumanPrincipalId),
    Role(AgentRoleId),
}

impl GrantSubject {
    /// The canonical, digest-stable description.
    pub fn describe(&self) -> String {
        match self {
            GrantSubject::Human(id) => format!("human:{id}"),
            GrantSubject::Role(id) => format!("role:{id}"),
        }
    }

    pub fn id_string(&self) -> String {
        match self {
            GrantSubject::Human(id) => id.to_string(),
            GrantSubject::Role(id) => id.to_string(),
        }
    }
}

/// A scoped mandate under one boundary (§4.2/§4.4).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthorityGrant {
    pub grant_id: String,
    pub boundary_id: String,
    pub tenant_id: TenantId,
    pub issuer: HumanPrincipalId,
    pub subject: GrantSubject,
    pub actions: Vec<GrantAction>,
    pub selector: TargetSelector,
    pub risk_ceiling: RiskClass,
    pub spend_limits: Option<Value>,
    pub delegable: bool,
    pub valid_from: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub status: GrantStatus,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GrantStatus {
    Active,
    Revoked,
}

impl GrantStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            GrantStatus::Active => "active",
            GrantStatus::Revoked => "revoked",
        }
    }

    pub fn from_wire_name(s: &str) -> Option<Self> {
        match s {
            "active" => Some(GrantStatus::Active),
            "revoked" => Some(GrantStatus::Revoked),
            _ => None,
        }
    }
}

// ── The subset rule ────────────────────────────────────────────────────────────

/// One way a grant exceeds its boundary (§4.4: "every charter provision and grant
/// must be a subset of the applicable boundary").
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundaryViolation {
    pub field: &'static str,
    pub detail: String,
}

impl std::fmt::Display for BoundaryViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.field, self.detail)
    }
}

/// Does a grant's spend limit stay within the boundary's ceiling? Both shapes are
/// JSON objects with a numeric `amount` in the dev profile; anything un-comparable is
/// a violation (fail closed), never an assumption of compliance.
fn spend_within(ceiling: &Value, limits: &Value) -> bool {
    match (
        ceiling.get("amount").and_then(|v| v.as_f64()),
        limits.get("amount").and_then(|v| v.as_f64()),
    ) {
        (Some(ceiling), Some(limit)) => limit <= ceiling,
        _ => false,
    }
}

/// The deterministic subset check: every dimension of the grant must fit inside the
/// boundary. An empty result means the grant is within the ceiling — the temporal
/// window INCLUDED (a grant may not outlive its boundary).
pub fn grant_exceeds_boundary(
    boundary: &EnrollmentAuthorityBoundary,
    grant: &AuthorityGrant,
) -> Vec<BoundaryViolation> {
    let mut violations = Vec::new();

    if boundary.status != BoundaryStatus::Active {
        violations.push(BoundaryViolation {
            field: "boundary.status",
            detail: format!(
                "boundary `{}` is `{}`, not `active`",
                boundary.boundary_id,
                boundary.status.as_str()
            ),
        });
    }

    for action in &grant.actions {
        if !boundary.permitted_actions.contains(action) {
            violations.push(BoundaryViolation {
                field: "grant.actions",
                detail: format!(
                    "action `{}` is not permitted by boundary `{}`",
                    action.as_str(),
                    boundary.boundary_id
                ),
            });
        }
    }

    if grant.risk_ceiling > boundary.risk_ceiling {
        violations.push(BoundaryViolation {
            field: "grant.risk_ceiling",
            detail: format!(
                "grant risk ceiling `{}` exceeds the boundary's `{}`",
                grant.risk_ceiling.as_str(),
                boundary.risk_ceiling.as_str()
            ),
        });
    }

    match (&boundary.spend_ceiling, &grant.spend_limits) {
        (Some(ceiling), Some(limits)) if !spend_within(ceiling, limits) => {
            violations.push(BoundaryViolation {
                field: "grant.spend_limits",
                detail: "the grant's spend limits are not within the boundary's ceiling"
                    .to_string(),
            });
        }
        (None, Some(_)) => violations.push(BoundaryViolation {
            field: "grant.spend_limits",
            detail: "the boundary pins no spend ceiling, so a grant cannot carry spend limits"
                .to_string(),
        }),
        _ => {}
    }

    if grant.delegable && !boundary.delegable {
        violations.push(BoundaryViolation {
            field: "grant.delegable",
            detail: "the boundary does not permit delegation".to_string(),
        });
    }
    let _ = boundary.max_delegation_depth; // dev profile: only direct grants exist; chains are Phase 2

    if grant.valid_from < boundary.valid_from {
        violations.push(BoundaryViolation {
            field: "grant.valid_from",
            detail: "the grant starts before its boundary".to_string(),
        });
    }
    if grant.expires_at > boundary.expires_at {
        violations.push(BoundaryViolation {
            field: "grant.expires_at",
            detail: "the grant outlives its boundary".to_string(),
        });
    }

    violations
}

/// Time- and status-based liveness at evaluation time (separate from the structural
/// subset rule, which is checked at grant creation AND re-checked at evaluation).
pub fn boundary_active_at(boundary: &EnrollmentAuthorityBoundary, at: DateTime<Utc>) -> bool {
    boundary.status == BoundaryStatus::Active
        && at >= boundary.valid_from
        && at <= boundary.expires_at
}

pub fn grant_active_at(grant: &AuthorityGrant, at: DateTime<Utc>) -> bool {
    grant.status == GrantStatus::Active && at >= grant.valid_from && at <= grant.expires_at
}

/// The development profile's actor handle for a presented principal
/// (`PHASE-0.6.1`): a deterministic `ActorPrincipalId` (UUIDv5, namespaced) derived
/// from the subject's canonical description.
///
/// Authentication and certificate-bound identity are out of Phase 0 scope
/// (`ID-003`; WP5 "development credentials"), so the control API resolves a presented
/// principal into a *stable* opaque actor handle instead of a per-request random one —
/// audit rows for the same principal stay linkable across requests. UUIDv5 uses
/// SHA-1; it is used here for stable namespacing of a dev-profile handle, NOT as a
/// security boundary (the dev profile trusts the presented principal anyway).
pub fn actor_handle_for_subject(subject: &GrantSubject) -> ActorPrincipalId {
    let uuid = uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_OID, subject.describe().as_bytes());
    ActorPrincipalId::from_uuid(uuid)
}

from_str_via!(GrantAction);
from_str_via!(RiskClass);
from_str_via!(BoundaryStatus);
from_str_via!(GrantStatus);

// ── Decision and record ────────────────────────────────────────────────────────

/// A deterministic authorization decision.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "decision", rename_all = "snake_case")]
pub enum Decision {
    Allowed,
    Denied { reason: String },
}

impl Decision {
    pub fn as_str(&self) -> &'static str {
        match self {
            Decision::Allowed => "allowed",
            Decision::Denied { .. } => "denied",
        }
    }

    /// The canonical, digest-stable description (the denial reason is part of what
    /// the digest binds).
    pub fn describe(&self) -> String {
        match self {
            Decision::Allowed => "allowed".to_string(),
            Decision::Denied { reason } => format!("denied:{reason}"),
        }
    }
}

/// The audit record every command leaves (§4.5/§5 WP5 acceptance): actor, subject if
/// delegated, the grant and boundary the decision referenced, the decision itself,
/// and the policy digest + version it was evaluated against.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthorizationDecisionRecord {
    pub record_id: AuthorizationRecordId,
    pub tenant_id: TenantId,
    pub actor: ActorPrincipalId,
    #[serde(default)]
    pub subject: Option<GrantSubject>,
    #[serde(default)]
    pub boundary_id: Option<String>,
    #[serde(default)]
    pub grant_id: Option<String>,
    pub action: GrantAction,
    pub target: ResourceTarget,
    pub decision: Decision,
    pub policy_digest: String,
    pub policy_version: String,
    pub decided_at: DateTime<Utc>,
}

/// SHA-256 over the documented field-order input, hex-encoded. The digest names the
/// exact policy inputs + decision an [`AuthorizationRecord`] was produced from — it
/// changes whenever any of them changes, and it contains no secrets.
pub fn policy_digest(
    boundary: Option<&EnrollmentAuthorityBoundary>,
    grant: Option<&AuthorityGrant>,
    subject: Option<&GrantSubject>,
    action: GrantAction,
    target: &ResourceTarget,
    decision: &Decision,
) -> String {
    let input = format!(
        "{}|{}|{}|{}|{}|{}|{}|{}",
        boundary
            .map(|b| b.boundary_id.as_str())
            .unwrap_or("no-boundary"),
        boundary
            .map(|b| b.charter_digest.as_str())
            .unwrap_or("no-charter"),
        boundary
            .map(|b| b.policy_version.as_str())
            .unwrap_or("no-policy"),
        grant.map(|g| g.grant_id.as_str()).unwrap_or("no-grant"),
        subject
            .map(GrantSubject::describe)
            .unwrap_or_else(|| "no-subject".to_string()),
        action.as_str(),
        target.describe(),
        decision.describe(),
    );
    let digest = Sha256::digest(input.as_bytes());
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn boundary(
        permitted_actions: Vec<GrantAction>,
        risk_ceiling: RiskClass,
        delegable: bool,
        spend_ceiling: Option<Value>,
    ) -> EnrollmentAuthorityBoundary {
        let valid_from = Utc.with_ymd_and_hms(2026, 9, 1, 0, 0, 0).unwrap();
        EnrollmentAuthorityBoundary {
            boundary_id: "bnd_test".to_string(),
            tenant_id: "ten_00000000-0000-7000-8000-000000000001".parse().unwrap(),
            parent_or_root_authority: "dev-root-2026-09-06".to_string(),
            target_owner: "owner@example.org".to_string(),
            permitted_actions,
            permitted_domains: vec!["deliberation".to_string()],
            risk_ceiling,
            spend_ceiling,
            delegable,
            max_delegation_depth: 1,
            valid_from,
            expires_at: Utc.with_ymd_and_hms(2027, 9, 1, 0, 0, 0).unwrap(),
            charter_digest: "dev-charter-digest".to_string(),
            policy_version: "dev-authz-1".to_string(),
            status: BoundaryStatus::Active,
        }
    }

    fn grant(
        actions: Vec<GrantAction>,
        risk_ceiling: RiskClass,
        delegable: bool,
    ) -> AuthorityGrant {
        AuthorityGrant {
            grant_id: "grt_test".to_string(),
            boundary_id: "bnd_test".to_string(),
            tenant_id: "ten_00000000-0000-7000-8000-000000000001".parse().unwrap(),
            issuer: "hpr_00000000-0000-7000-8000-000000000001".parse().unwrap(),
            subject: GrantSubject::Role(
                "rol_00000000-0000-7000-8000-000000000001".parse().unwrap(),
            ),
            actions,
            selector: TargetSelector::TenantWide,
            risk_ceiling,
            spend_limits: None,
            delegable,
            valid_from: Utc.with_ymd_and_hms(2026, 9, 1, 0, 0, 0).unwrap(),
            expires_at: Utc.with_ymd_and_hms(2027, 9, 1, 0, 0, 0).unwrap(),
            status: GrantStatus::Active,
        }
    }

    /// A valid grant produces no violations.
    #[test]
    fn a_grant_inside_its_boundary_has_no_violations() {
        let b = boundary(
            vec![GrantAction::ThreadContribute],
            RiskClass::Medium,
            false,
            None,
        );
        let g = grant(vec![GrantAction::ThreadContribute], RiskClass::Low, false);
        assert!(grant_exceeds_boundary(&b, &g).is_empty());
    }

    /// The subset rule: actions, risk, delegation, and the temporal window are each
    /// checked independently — one excess produces exactly its violation.
    #[test]
    fn every_dimension_of_the_subset_rule_is_enforced() {
        let b = boundary(
            vec![GrantAction::ThreadContribute],
            RiskClass::Medium,
            false,
            Some(serde_json::json!({ "amount": 100.0 })),
        );

        let extra_action = grant(
            vec![GrantAction::ThreadContribute, GrantAction::TenantAdmin],
            RiskClass::Low,
            false,
        );
        let violations = grant_exceeds_boundary(&b, &extra_action);
        assert_eq!(violations.len(), 1);
        assert!(
            violations[0].detail.contains("tenant_admin"),
            "got: {violations:?}"
        );

        let high_risk = grant(vec![GrantAction::ThreadContribute], RiskClass::High, false);
        let violations = grant_exceeds_boundary(&b, &high_risk);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].detail.contains("risk"), "got: {violations:?}");

        let delegating = grant(vec![GrantAction::ThreadContribute], RiskClass::Low, true);
        let violations = grant_exceeds_boundary(&b, &delegating);
        assert_eq!(violations.len(), 1);
        assert!(
            violations[0].detail.contains("delegation"),
            "got: {violations:?}"
        );

        let mut overspending = grant(vec![GrantAction::ThreadContribute], RiskClass::Low, false);
        overspending.spend_limits = Some(serde_json::json!({ "amount": 200.0 }));
        let violations = grant_exceeds_boundary(&b, &overspending);
        assert_eq!(violations.len(), 1);
        assert!(
            violations[0].detail.contains("spend"),
            "got: {violations:?}"
        );

        let mut early = grant(vec![GrantAction::ThreadContribute], RiskClass::Low, false);
        early.valid_from = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
        let violations = grant_exceeds_boundary(&b, &early);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].field, "grant.valid_from");

        let mut outliving = grant(vec![GrantAction::ThreadContribute], RiskClass::Low, false);
        outliving.expires_at = Utc.with_ymd_and_hms(2028, 1, 1, 0, 0, 0).unwrap();
        let violations = grant_exceeds_boundary(&b, &outliving);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].field, "grant.expires_at");
    }

    /// A suspended/revoked boundary is not a ceiling anything may sit under.
    #[test]
    fn an_inactive_boundary_violates_everything() {
        let mut b = boundary(
            vec![GrantAction::ThreadContribute],
            RiskClass::Medium,
            false,
            None,
        );
        b.status = BoundaryStatus::Suspended;
        let g = grant(vec![GrantAction::ThreadContribute], RiskClass::Low, false);
        let violations = grant_exceeds_boundary(&b, &g);
        assert_eq!(violations.len(), 1);
        assert!(
            violations[0].detail.contains("suspended"),
            "got: {violations:?}"
        );
        assert!(!boundary_active_at(&b, Utc::now()));
    }

    /// Liveness at evaluation time: windows and statuses bind the decision.
    #[test]
    fn liveness_helpers_respect_windows_and_status() {
        let b = boundary(vec![], RiskClass::Low, false, None);
        assert!(boundary_active_at(
            &b,
            Utc.with_ymd_and_hms(2026, 12, 1, 0, 0, 0).unwrap()
        ));
        assert!(!boundary_active_at(
            &b,
            Utc.with_ymd_and_hms(2028, 1, 1, 0, 0, 0).unwrap()
        ));

        let g = grant(vec![], RiskClass::Low, false);
        assert!(grant_active_at(
            &g,
            Utc.with_ymd_and_hms(2026, 12, 1, 0, 0, 0).unwrap()
        ));
        assert!(!grant_active_at(
            &g,
            Utc.with_ymd_and_hms(2026, 8, 1, 0, 0, 0).unwrap()
        ));
    }

    /// The digest is deterministic and binds every input: the same inputs hash the
    /// same, and changing any input — including the decision and its reason — changes
    /// the digest.
    #[test]
    fn policy_digest_is_deterministic_and_binds_all_inputs() {
        let b = boundary(
            vec![GrantAction::ThreadContribute],
            RiskClass::Medium,
            false,
            None,
        );
        let g = grant(vec![GrantAction::ThreadContribute], RiskClass::Low, false);
        let target = ResourceTarget::Thread {
            tenant_id: "ten_00000000-0000-7000-8000-000000000001".parse().unwrap(),
            thread_id: "thr_00000000-0000-7000-8000-000000000001".parse().unwrap(),
        };
        let decision = Decision::Allowed;

        let digest = || {
            policy_digest(
                Some(&b),
                Some(&g),
                Some(&g.subject),
                GrantAction::ThreadContribute,
                &target,
                &decision,
            )
        };

        let first = digest();
        assert_eq!(first.len(), 64, "sha256 hex");
        assert_eq!(first, digest(), "same inputs → same digest");

        let other_grant = AuthorityGrant {
            grant_id: "grt_other".to_string(),
            ..g.clone()
        };
        let changed = policy_digest(
            Some(&b),
            Some(&other_grant),
            Some(&g.subject),
            GrantAction::ThreadContribute,
            &target,
            &decision,
        );
        assert_ne!(first, changed, "a different grant changes the digest");

        let denied = policy_digest(
            Some(&b),
            Some(&g),
            Some(&g.subject),
            GrantAction::ThreadContribute,
            &target,
            &Decision::Denied {
                reason: "scope".to_string(),
            },
        );
        assert_ne!(first, denied, "the decision binds the digest");
    }

    /// Stored wire names parse back to their variants (the server persists strings).
    #[test]
    fn wire_names_parse_back() {
        for (s, expected) in [
            ("thread_create", GrantAction::ThreadCreate),
            ("thread_invite", GrantAction::ThreadInvite),
            ("thread_contribute", GrantAction::ThreadContribute),
            ("thread_inspect", GrantAction::ThreadInspect),
            ("thread_close", GrantAction::ThreadClose),
            ("thread_cancel", GrantAction::ThreadCancel),
            ("tenant_admin", GrantAction::TenantAdmin),
        ] {
            assert_eq!(s.parse::<GrantAction>(), Ok(expected));
            assert_eq!(s.parse::<GrantAction>().unwrap().as_str(), s);
        }
        assert!(matches!(
            "thread_delete".parse::<GrantAction>(),
            Err(UnknownAuthorityName(_))
        ));
        assert_eq!("high".parse::<RiskClass>(), Ok(RiskClass::High));
        assert_eq!(
            "suspended".parse::<BoundaryStatus>(),
            Ok(BoundaryStatus::Suspended)
        );
        assert_eq!("revoked".parse::<GrantStatus>(), Ok(GrantStatus::Revoked));
    }

    /// Subject and target descriptions are stable and digest-safe.
    #[test]
    fn descriptions_are_stable_and_distinct() {
        let human =
            GrantSubject::Human("hpr_00000000-0000-7000-8000-000000000001".parse().unwrap());
        let role = GrantSubject::Role("rol_00000000-0000-7000-8000-000000000001".parse().unwrap());
        assert_eq!(
            human.describe(),
            "human:hpr_00000000-0000-7000-8000-000000000001"
        );
        assert_ne!(human.describe(), role.describe());

        let tenant = ResourceTarget::Tenant {
            tenant_id: "ten_00000000-0000-7000-8000-000000000001".parse().unwrap(),
        };
        let thread = ResourceTarget::Thread {
            tenant_id: "ten_00000000-0000-7000-8000-000000000001".parse().unwrap(),
            thread_id: "thr_00000000-0000-7000-8000-000000000001".parse().unwrap(),
        };
        assert_ne!(tenant.describe(), thread.describe());
    }

    /// The dev-profile actor handle (`PHASE-0.6.1`): deterministic per subject,
    /// distinct across subject kinds, and a valid `agt_` identifier — so audit rows
    /// for the same principal stay linkable without a certificate issuer.
    #[test]
    fn actor_handles_are_deterministic_and_distinct() {
        let human =
            GrantSubject::Human("hpr_00000000-0000-7000-8000-000000000001".parse().unwrap());
        let role = GrantSubject::Role("rol_00000000-0000-7000-8000-000000000001".parse().unwrap());

        let a = actor_handle_for_subject(&human);
        let b = actor_handle_for_subject(&human);
        assert_eq!(a, b, "the same subject derives the same handle");
        assert_ne!(a, actor_handle_for_subject(&role), "kinds stay distinct");
        assert!(a.to_string().starts_with("agt_"));
    }
}
