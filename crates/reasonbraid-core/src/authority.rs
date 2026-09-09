//! The development authority model (`PHASE-0.5.1`; `ROADMAP.md` §4.4): an
//! [`EnrollmentAuthorityBoundary`] is the root/parent-granted ceiling, an
//! [`AuthorityGrant`] is a scoped mandate under it, and every command's decision is an
//! [`AuthorizationRecord`] bound to the policy it was evaluated against.
//!
//! # The invariant this module enforces
//!
//! **A grant can never exceed its boundary** ([`grant_exceeds_boundary`]): parent
//! identity and tenant must match; actions,
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
    /// Respond to a PENDING invitation (accept or decline — the invitation is the
    /// real capability; this grant is the "may participate in invitation flows"
    /// gate every invited role's default carries, `.1.3.1`).
    ThreadInvitationRespond,
    /// Advance a thread to its next round (`.1.5.2`): rounds are SERVER-assigned —
    /// contributions land in the current round, and advancement is this explicit,
    /// auditable act. Humans carry it via the dev admin set; roles stay
    /// deny-by-default (they shape content, humans shape the process).
    ThreadAdvanceRound,
    /// Node-initiated thread creation (§11.5, `PHASE-3.5.3`): a role initiates a
    /// NEW thread under this EXPLICIT grant — never implied by membership,
    /// never inherited by replies (a child thread needs its own grant).
    ThreadCreateAuto,
    /// Administrative authority — NEVER implied by membership or other actions.
    TenantAdmin,
}

impl GrantAction {
    pub fn as_str(self) -> &'static str {
        match self {
            GrantAction::ThreadCreate => "thread_create",
            GrantAction::ThreadCreateAuto => "thread_create_auto",
            GrantAction::ThreadInvite => "thread_invite",
            GrantAction::ThreadContribute => "thread_contribute",
            GrantAction::ThreadInspect => "thread_inspect",
            GrantAction::ThreadClose => "thread_close",
            GrantAction::ThreadCancel => "thread_cancel",
            GrantAction::ThreadInvitationRespond => "thread_invitation_respond",
            GrantAction::ThreadAdvanceRound => "thread_advance_round",
            GrantAction::TenantAdmin => "tenant_admin",
        }
    }

    /// Parse a stored/wire name (the server persists actions as strings).
    pub fn from_wire_name(s: &str) -> Option<Self> {
        match s {
            "thread_create" => Some(GrantAction::ThreadCreate),
            "thread_create_auto" => Some(GrantAction::ThreadCreateAuto),
            "thread_invite" => Some(GrantAction::ThreadInvite),
            "thread_contribute" => Some(GrantAction::ThreadContribute),
            "thread_inspect" => Some(GrantAction::ThreadInspect),
            "thread_close" => Some(GrantAction::ThreadClose),
            "thread_cancel" => Some(GrantAction::ThreadCancel),
            "thread_invitation_respond" => Some(GrantAction::ThreadInvitationRespond),
            "thread_advance_round" => Some(GrantAction::ThreadAdvanceRound),
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

/// A resource a command targets. Creation and tenant administration target the
/// tenant. Inspection supports a thread or the tenant's thread listing; other
/// thread actions target a thread.
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
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
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
/// models, hold authority). JSON is an explicit kind/id object; the typed ID
/// must match its human/role kind. This is distinct from the public command
/// envelope's existing `on_behalf_of` string field.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "id", rename_all = "snake_case")]
pub enum GrantSubject {
    Human(HumanPrincipalId),
    Role(AgentRoleId),
}

// Require an object instead of accepting the sequence representation that a
// derived adjacently tagged deserializer also supports. Derived field parsing
// retains duplicate/missing/unknown-field checks without collapsing a JSON map.
impl<'de> Deserialize<'de> for GrantSubject {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct SubjectVisitor;
        impl<'de> serde::de::Visitor<'de> for SubjectVisitor {
            type Value = GrantSubject;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a subject object with kind and id")
            }

            fn visit_map<A: serde::de::MapAccess<'de>>(
                self,
                map: A,
            ) -> Result<Self::Value, A::Error> {
                #[derive(Deserialize)]
                #[serde(deny_unknown_fields)]
                struct Fields {
                    kind: String,
                    id: String,
                }
                let fields =
                    Fields::deserialize(serde::de::value::MapAccessDeserializer::new(map))?;
                match fields.kind.as_str() {
                    "human" => fields
                        .id
                        .parse()
                        .map(GrantSubject::Human)
                        .map_err(serde::de::Error::custom),
                    "role" => fields
                        .id
                        .parse()
                        .map(GrantSubject::Role)
                        .map_err(serde::de::Error::custom),
                    unknown => Err(serde::de::Error::unknown_variant(
                        unknown,
                        &["human", "role"],
                    )),
                }
            }
        }
        deserializer.deserialize_map(SubjectVisitor)
    }
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
/// named boundary in the same tenant. An empty result means the grant is within
/// the ceiling, including a nonempty validity window contained in its parent's.
pub fn grant_exceeds_boundary(
    boundary: &EnrollmentAuthorityBoundary,
    grant: &AuthorityGrant,
) -> Vec<BoundaryViolation> {
    let mut violations = Vec::new();

    if grant.boundary_id != boundary.boundary_id {
        violations.push(BoundaryViolation {
            field: "grant.boundary_id",
            detail: "the grant does not name the supplied boundary".to_string(),
        });
    }
    if grant.tenant_id != boundary.tenant_id {
        violations.push(BoundaryViolation {
            field: "grant.tenant_id",
            detail: "the grant and its boundary belong to different tenants".to_string(),
        });
    }
    if boundary.valid_from >= boundary.expires_at {
        violations.push(BoundaryViolation {
            field: "boundary.validity_window",
            detail: "the boundary's validity window must be nonempty".to_string(),
        });
    }
    if grant.valid_from >= grant.expires_at {
        violations.push(BoundaryViolation {
            field: "grant.validity_window",
            detail: "the grant's validity window must be nonempty".to_string(),
        });
    }

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
/// Validity includes the start and excludes expiration: [valid_from, expires_at).
pub fn boundary_active_at(boundary: &EnrollmentAuthorityBoundary, at: DateTime<Utc>) -> bool {
    boundary.status == BoundaryStatus::Active
        && at >= boundary.valid_from
        && at < boundary.expires_at
}

/// The grant's status and half-open validity window at the decision instant.
pub fn grant_active_at(grant: &AuthorityGrant, at: DateTime<Utc>) -> bool {
    grant.status == GrantStatus::Active && at >= grant.valid_from && at < grant.expires_at
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

    #[test]
    fn grants_are_bound_to_the_named_boundary_and_its_tenant() {
        let b = boundary(
            vec![GrantAction::ThreadInspect],
            RiskClass::Low,
            false,
            None,
        );
        let g = grant(vec![GrantAction::ThreadInspect], RiskClass::Low, false);
        assert!(grant_exceeds_boundary(&b, &g).is_empty());
        let mut unrelated = b.clone();
        unrelated.boundary_id = "bnd_unrelated".into();
        assert!(grant_exceeds_boundary(&unrelated, &g)
            .iter()
            .any(|v| v.field == "grant.boundary_id"));
        let mut foreign = b;
        foreign.tenant_id = TenantId::new();
        assert!(grant_exceeds_boundary(&foreign, &g)
            .iter()
            .any(|v| v.field == "grant.tenant_id"));
    }

    #[test]
    fn authority_windows_are_nonempty_and_expiration_is_exclusive() {
        let b = boundary(
            vec![GrantAction::ThreadInspect],
            RiskClass::Low,
            false,
            None,
        );
        let g = grant(vec![GrantAction::ThreadInspect], RiskClass::Low, false);
        let tick = chrono::Duration::nanoseconds(1);
        for (at, live) in [
            (b.valid_from - tick, false),
            (b.valid_from, true),
            (b.expires_at - tick, true),
            (b.expires_at, false),
            (b.expires_at + tick, false),
        ] {
            assert_eq!(boundary_active_at(&b, at), live, "boundary at {at}");
            assert_eq!(grant_active_at(&g, at), live, "grant at {at}");
        }
        for end in [g.valid_from, g.valid_from - tick] {
            let mut invalid = g.clone();
            invalid.expires_at = end;
            assert!(grant_exceeds_boundary(&b, &invalid)
                .iter()
                .any(|v| v.field == "grant.validity_window"));
            assert!(!grant_active_at(&invalid, invalid.valid_from));
        }
        let mut empty = b;
        empty.expires_at = empty.valid_from;
        assert!(grant_exceeds_boundary(&empty, &g)
            .iter()
            .any(|v| v.field == "boundary.validity_window"));
        assert!(!boundary_active_at(&empty, empty.valid_from));
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
            (
                "thread_invitation_respond",
                GrantAction::ThreadInvitationRespond,
            ),
            ("thread_advance_round", GrantAction::ThreadAdvanceRound),
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

// ── Delegation constraints (`.1.4.1`; the ADR-009 spike) ──────────────────────

/// The delegation context a request may carry (the `.1.4.1` dev shape — the
/// full §16.3 `AuthorityContext` collapses to these dimensions at the dev
/// profile; the issuer chain rides the grant rows and the audit record).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DelegationConstraints {
    /// The principal the actor acts on behalf of (the grant holder).
    pub on_behalf_of: GrantSubject,
    /// Why (audit context; the authorization record carries it).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub purpose: Option<String>,
    /// The scope the delegation may touch: the request's target must be
    /// WITHIN this — a subset of the subject's grant selector, never wider.
    pub scope: TargetSelector,
}

/// The widening invariant (§16.3.1), as a PURE decision: a request's scope is
/// within a grant when every requested thread is in the grant's selector (a
/// tenant-wide grant contains anything; a tenant-wide REQUEST is never within
/// a thread-scoped grant). The action dimension rides the caller's own check
/// (`grant.actions.contains`), so this function owns the TARGET dimension.
pub fn delegation_scope_is_subset(requested: &TargetSelector, granted: &TargetSelector) -> bool {
    match (requested, granted) {
        (_, TargetSelector::TenantWide) => true,
        (TargetSelector::TenantWide, TargetSelector::Threads { .. }) => false,
        (TargetSelector::Threads { threads: want }, TargetSelector::Threads { threads: have }) => {
            want.iter().all(|t| have.contains(t))
        }
    }
}

// ── Cached decisions (`.1.5.1`; the ADR-008 spike) ─────────────────────────

/// The freshness TTL of a cached ADMISSION allow (the rule the `.1.5.1` spike
/// declares): `decided_at + TTL` bounds the window a cached allow may stand
/// without a re-ask. Short on purpose — the node's poll cadence is seconds,
/// so a stale allow refreshes at the next poll.
pub const CACHED_ALLOW_TTL_SECONDS: i64 = 60;

/// The server's admission decision, as the node caches it (ROADMAP §16.4 —
/// only explicitly cacheable decisions; §11.1 — the minimum state). The node
/// caches ONLY the decisions riding its delivery: the allowing admission for
/// a dispatched command, or a re-ask's denial. `revocation_epoch` is the
/// tenant's epoch AT DECISION TIME — a later bump invalidates the entry,
/// however fresh it still looks (the `.1.3`/`.1.4` freshness model: a cache
/// must never outlive a revocation).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CachedDecision {
    pub kind: CachedDecisionKind,
    pub decided_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub revocation_epoch: u64,
    pub policy_digest: String,
}

/// What a cached decision says.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "decision", rename_all = "snake_case", deny_unknown_fields)]
pub enum CachedDecisionKind {
    Allow,
    Deny { reason: String },
}

/// The verdict of evaluating a cached decision at the dispatch boundary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CacheVerdict {
    /// The cached allow stands — fresh AND epoch-current: dispatch may
    /// proceed without a round trip.
    Allow,
    /// The cached deny stands — a deny is never widened by time; the
    /// dispatch is refused (cheaply fail-closed).
    Deny { reason: String },
    /// Expired or epoch-invalidated: RE-ASK the authority store. If the
    /// store is unreachable, the action class's fail rule (§16.4) applies.
    Stale,
}

impl CachedDecision {
    /// An allow built from the admission decision (the `.1.5.1` declared
    /// freshness rule: the TTL runs from the decision time).
    pub fn allow(decided_at: DateTime<Utc>, revocation_epoch: u64, policy_digest: String) -> Self {
        Self {
            kind: CachedDecisionKind::Allow,
            decided_at,
            expires_at: decided_at + chrono::Duration::seconds(CACHED_ALLOW_TTL_SECONDS),
            revocation_epoch,
            policy_digest,
        }
    }

    /// Still inside the declared freshness window?
    pub fn is_fresh(&self, now: DateTime<Utc>) -> bool {
        now < self.expires_at
    }

    /// Has the tenant's revocation epoch moved since this decision? A bump
    /// invalidates the cache — a revocation must never be outlived.
    pub fn is_invalidated(&self, current_epoch: u64) -> bool {
        self.revocation_epoch != current_epoch
    }

    /// The dispatch-boundary evaluation: a cached allow stands only while it
    /// is fresh AND the epoch is unchanged; a cached deny always stands (it
    /// is never widened by time — refusing is safe); everything else is a
    /// re-ask.
    pub fn evaluate(&self, now: DateTime<Utc>, current_epoch: u64) -> CacheVerdict {
        match &self.kind {
            CachedDecisionKind::Deny { reason } => CacheVerdict::Deny {
                reason: reason.clone(),
            },
            CachedDecisionKind::Allow => {
                if self.is_fresh(now) && !self.is_invalidated(current_epoch) {
                    CacheVerdict::Allow
                } else {
                    CacheVerdict::Stale
                }
            }
        }
    }
}

/// An action class for the §16.4 fail rule. The dev profile's classes are
/// declared HERE so the rule is a table lookup, not prose.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionClass {
    /// Dispatch of a delivered command — the provider contact, §16.4's
    /// irreversible write (a silent retry could duplicate a provider effect).
    IrreversibleWrite,
    /// Administrative writes (grants, boundaries, revocation).
    AdministrativeWrite,
    /// Read-only inspection (the node's local journal surfaces; authority-
    /// gated reads are server-side — unreachable means nothing to serve).
    Read,
}

/// The declared fail mode when the authority store is unreachable.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FailMode {
    FailClosed,
    FailOpen,
}

impl ActionClass {
    /// §16.4's rule: publication, secret access, grant changes, and
    /// irreversible writes FAIL CLOSED; the dev profile's reads (local
    /// journal inspection) fail open.
    pub fn fail_mode_when_unreachable(self) -> FailMode {
        match self {
            ActionClass::IrreversibleWrite | ActionClass::AdministrativeWrite => {
                FailMode::FailClosed
            }
            ActionClass::Read => FailMode::FailOpen,
        }
    }
}

#[cfg(test)]
mod delegation_tests {
    use super::*;
    use uuid::Uuid;

    fn thread(id: u64) -> ThreadId {
        ThreadId::from_uuid(Uuid::from_u64_pair(0x7000_8000, id))
    }

    #[test]
    fn a_narrower_scope_is_a_subset_and_equal_is_included() {
        let granted = TargetSelector::Threads {
            threads: vec![thread(1), thread(2), thread(3)],
        };
        // A narrower request (one of the granted threads) passes.
        assert!(delegation_scope_is_subset(
            &TargetSelector::Threads {
                threads: vec![thread(1)],
            },
            &granted,
        ));
        // The exact set passes (equality is a subset).
        assert!(delegation_scope_is_subset(
            &TargetSelector::Threads {
                threads: vec![thread(1), thread(2), thread(3)],
            },
            &granted,
        ));
        // An EMPTY request is a subset (ask for nothing — always safe).
        assert!(delegation_scope_is_subset(
            &TargetSelector::Threads { threads: vec![] },
            &granted,
        ));
        // Anything is within a tenant-wide grant.
        assert!(delegation_scope_is_subset(
            &TargetSelector::Threads {
                threads: vec![thread(9)],
            },
            &TargetSelector::TenantWide,
        ));
    }

    #[test]
    fn a_widening_scope_is_refused_per_dimension() {
        let granted = TargetSelector::Threads {
            threads: vec![thread(1), thread(2)],
        };
        // A thread OUTSIDE the grant's set — the widening attempt.
        assert!(!delegation_scope_is_subset(
            &TargetSelector::Threads {
                threads: vec![thread(1), thread(3)],
            },
            &granted,
        ));
        // A tenant-wide request is never within a thread-scoped grant.
        assert!(!delegation_scope_is_subset(
            &TargetSelector::TenantWide,
            &granted
        ));
    }

    #[test]
    fn public_authority_context_keeps_its_string_subject_and_scope() {
        let context = crate::envelope::AuthorityContext {
            on_behalf_of: "rol_00000000-0000-7000-8000-000000000001".into(),
            purpose: Some("delegated contribution".into()),
            scope: TargetSelector::Threads {
                threads: vec![
                    "thr_00000000-0000-7000-8000-000000000001".parse().unwrap(),
                    "thr_00000000-0000-7000-8000-000000000002".parse().unwrap(),
                ],
            },
        };
        let encoded = serde_json::to_vec(&context).unwrap();
        let value: Value = serde_json::from_slice(&encoded).unwrap();
        assert_eq!(
            value,
            serde_json::json!({
                "on_behalf_of": "rol_00000000-0000-7000-8000-000000000001",
                "purpose": "delegated contribution",
                "scope": { "kind": "threads", "threads": ["thr_00000000-0000-7000-8000-000000000001", "thr_00000000-0000-7000-8000-000000000002"] },
            })
        );
        let decoded: crate::envelope::AuthorityContext = serde_json::from_slice(&encoded).unwrap();
        assert_eq!(decoded.on_behalf_of, context.on_behalf_of);
        assert_eq!(decoded.purpose, context.purpose);
        assert_eq!(decoded.scope, context.scope);
        // This measures the shipped fixture, not a hypothetical token encoding.
        println!(
            "public AuthorityContext fixture: {} bytes; no token comparison",
            encoded.len()
        );
    }
}

#[cfg(test)]
mod cached_decision_tests {
    use super::*;
    use chrono::TimeZone;

    fn t(s: i64) -> DateTime<Utc> {
        Utc.timestamp_opt(s, 0).unwrap()
    }

    #[test]
    fn a_fresh_epoch_current_cached_allow_dispatches() {
        let decided = t(1_000);
        let allow = CachedDecision::allow(decided, 7, "digest-a".to_string());
        // Inside the TTL AND the epoch unchanged → the cached allow stands.
        assert_eq!(allow.evaluate(decided, 7), CacheVerdict::Allow);
        assert_eq!(
            allow.evaluate(
                decided + chrono::Duration::seconds(CACHED_ALLOW_TTL_SECONDS - 1),
                7
            ),
            CacheVerdict::Allow
        );
    }

    #[test]
    fn an_expired_cached_allow_is_stale() {
        let decided = t(1_000);
        let allow = CachedDecision::allow(decided, 7, "digest-a".to_string());
        // The TTL boundary is exclusive: AT expires_at the allow is stale.
        assert_eq!(
            allow.evaluate(
                decided + chrono::Duration::seconds(CACHED_ALLOW_TTL_SECONDS),
                7
            ),
            CacheVerdict::Stale
        );
        assert_eq!(
            allow.evaluate(
                decided + chrono::Duration::seconds(CACHED_ALLOW_TTL_SECONDS * 10),
                7
            ),
            CacheVerdict::Stale
        );
    }

    #[test]
    fn an_epoch_bump_invalidates_a_fresh_cached_allow() {
        let decided = t(1_000);
        let allow = CachedDecision::allow(decided, 7, "digest-a".to_string());
        // Still fresh — but a revocation bumped the epoch: the cache must
        // NOT outlive the revocation, so the entry is stale.
        assert_eq!(allow.evaluate(decided, 8), CacheVerdict::Stale);
        // A bump in either direction invalidates (the stored epoch is an
        // exact-generation marker, not an ordering).
        assert_eq!(allow.evaluate(decided, 6), CacheVerdict::Stale);
    }

    #[test]
    fn a_cached_deny_is_never_widened_by_time() {
        let decided = t(1_000);
        let deny = CachedDecision {
            kind: CachedDecisionKind::Deny {
                reason: "boundary frozen".to_string(),
            },
            decided_at: decided,
            expires_at: decided,
            revocation_epoch: 7,
            policy_digest: "digest-a".to_string(),
        };
        // A deny stands at ANY age and ANY epoch — refusing is always safe.
        assert_eq!(
            deny.evaluate(t(2_000_000), 99),
            CacheVerdict::Deny {
                reason: "boundary frozen".to_string()
            }
        );
    }

    #[test]
    fn irreversible_and_admin_writes_fail_closed_reads_fail_open() {
        // The §16.4 table: the irreversible dispatch + admin writes refuse
        // when the store is unreachable; the local-journal reads don't.
        assert_eq!(
            ActionClass::IrreversibleWrite.fail_mode_when_unreachable(),
            FailMode::FailClosed
        );
        assert_eq!(
            ActionClass::AdministrativeWrite.fail_mode_when_unreachable(),
            FailMode::FailClosed
        );
        assert_eq!(
            ActionClass::Read.fail_mode_when_unreachable(),
            FailMode::FailOpen
        );
    }
}
