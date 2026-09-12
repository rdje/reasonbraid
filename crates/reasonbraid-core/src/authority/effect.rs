//! What an admitted administrative request actually DID, kept distinct from the
//! admission that authorized it (`SIGNOFF-REPAIR.3.3.4.7.1`).
//!
//! An [`AuthorizationDecisionRecord`](super::AuthorizationDecisionRecord) says a
//! caller was admitted. It says nothing about whether the local mutation then
//! happened, was already satisfied, or was refused by a domain rule after
//! admission. Those are different facts, and collapsing them would let an
//! operator read "allowed" as "applied".
//!
//! # The closed sets are census-derived, not invented
//!
//! [`AdministrativeOperation`] contains exactly the fourteen administrative
//! mutations the `SIGNOFF-REPAIR.3.3.4` parent owns through its `.8`–`.12`
//! integration children. The population was measured before the set was written
//! (the census command and its classification are in that leaf); the four
//! remaining `tenant_admin`-gated mutations are routed to other owners and are
//! deliberately absent, because this parent cannot certify their gates.
//!
//! Each variant carries its own target, so an operation and a target that do not
//! belong together are unrepresentable rather than merely discouraged. Every
//! target is something the CALLER supplied, so it exists when the outcome is
//! [`AdministrativeOutcome::Refused`] just as it does when the mutation applied.
//!
//! # Declaring a variant is not implementing a route
//!
//! As with [`TenantAdminInspection`](super::TenantAdminInspection), a variant here
//! describes a representation. Until its integration child lands, no production
//! path writes it.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};

use super::evaluation::object_only;
use crate::error::KnownReasonCode;
use crate::id::{AgentRoleId, AuthorizationRecordId, TenantId};

/// Why a bounded administrative text field was refused.
///
/// Stored evidence is decoded with the same bounds it is constructed with, so a
/// row that grew past them is a storage failure rather than a value this build
/// silently accepts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AdministrativeTextError {
    /// Empty, or nothing but whitespace.
    Blank { field: &'static str },
    /// Longer than the field's byte limit (UTF-8 bytes, not characters).
    TooLong {
        field: &'static str,
        limit: usize,
        bytes: usize,
    },
    /// Contains a control character.
    ControlCharacter { field: &'static str },
}

impl std::fmt::Display for AdministrativeTextError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Blank { field } => write!(f, "{field} must be nonblank"),
            Self::TooLong {
                field,
                limit,
                bytes,
            } => write!(f, "{field} must be at most {limit} bytes, got {bytes}"),
            Self::ControlCharacter { field } => {
                write!(f, "{field} must contain no control characters")
            }
        }
    }
}

impl std::error::Error for AdministrativeTextError {}

fn bounded(
    field: &'static str,
    limit: usize,
    value: String,
) -> Result<String, AdministrativeTextError> {
    if value.trim().is_empty() {
        return Err(AdministrativeTextError::Blank { field });
    }
    if value.len() > limit {
        return Err(AdministrativeTextError::TooLong {
            field,
            limit,
            bytes: value.len(),
        });
    }
    if value.chars().any(char::is_control) {
        return Err(AdministrativeTextError::ControlCharacter { field });
    }
    Ok(value)
}

/// The identifier of an administrative target whose family has no typed
/// [`Id`](crate::Id) in this build — a grant (`grt_…`), an enrollment boundary
/// (`bnd_…`), a node (a `nod_…` id or the `rol_…` role wire id the dev profile
/// serves), an inbox command, a capability taxonomy, or a card digest.
///
/// Bounded exactly like the site-authority registry names it mirrors: nonblank,
/// at most 256 UTF-8 bytes, no control characters. Spelling is preserved
/// verbatim — this is evidence about what a caller named, not a normalized key.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct AdministrativeTargetId(String);

impl AdministrativeTargetId {
    pub const MAX_BYTES: usize = 256;

    pub fn new(value: impl Into<String>) -> Result<Self, AdministrativeTextError> {
        bounded("an administrative target id", Self::MAX_BYTES, value.into()).map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for AdministrativeTargetId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for AdministrativeTargetId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::new(String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

/// A bounded administrative reason: nonblank, at most 1 024 UTF-8 bytes, no
/// control characters — the same contract the site-authority reason already
/// publishes, so an operator meets one rule rather than two.
///
/// It carries two different senses, and the field name says which: a *submitted*
/// reason is what the caller wrote, and an outcome *detail* is what the server
/// determined. Neither is a reason code; that is [`KnownReasonCode`].
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct AdministrativeReason(String);

impl AdministrativeReason {
    pub const MAX_BYTES: usize = 1024;

    pub fn new(value: impl Into<String>) -> Result<Self, AdministrativeTextError> {
        bounded("an administrative reason", Self::MAX_BYTES, value.into()).map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for AdministrativeReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for AdministrativeReason {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::new(String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

/// One administrative mutation and the tenant-bound target it names.
///
/// The fourteen variants are the mutations owned by `SIGNOFF-REPAIR.3.3.4.8`
/// through `.12`. Breaker administration names no target field because its
/// target IS the tenant, which the enclosing record already carries.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AdministrativeOperation {
    // `.8` — grant and boundary revocation.
    GrantRevoke {
        grant_id: AdministrativeTargetId,
    },
    BoundaryRevoke {
        boundary_id: AdministrativeTargetId,
    },
    // `.9` — breaker administration; the target is the tenant itself.
    BreakerArm {},
    BreakerReset {},
    // `.10` — node administration.
    NodeEnrollTokenIssue {
        node_id: AdministrativeTargetId,
    },
    NodeRevoke {
        node_id: AdministrativeTargetId,
    },
    NodeCommandReplay {
        node_id: AdministrativeTargetId,
        command_id: AdministrativeTargetId,
    },
    NodeCommandQuarantine {
        node_id: AdministrativeTargetId,
        command_id: AdministrativeTargetId,
    },
    NodeInboxPrune {
        node_id: AdministrativeTargetId,
    },
    // `.11` — profile and card administration. The import's target is the card
    // the caller supplied, named by its digest: the local role the import may
    // create does not exist yet when the import is refused.
    ProfileCardImport {
        card_digest: AdministrativeTargetId,
    },
    CapabilityClaimAttest {
        role_id: AgentRoleId,
        taxonomy_id: AdministrativeTargetId,
    },
    // `.12` — federation direction administration. A direction is named by its
    // remote tenant, which is what `accept` and `revoke` actually address.
    FederationDirectionPropose {
        remote_tenant_id: TenantId,
    },
    FederationDirectionAccept {
        remote_tenant_id: TenantId,
    },
    FederationDirectionRevoke {
        remote_tenant_id: TenantId,
    },
}

impl AdministrativeOperation {
    /// The stored discriminant, which is also the database CHECK's vocabulary.
    pub fn kind(&self) -> &'static str {
        match self {
            Self::GrantRevoke { .. } => "grant_revoke",
            Self::BoundaryRevoke { .. } => "boundary_revoke",
            Self::BreakerArm {} => "breaker_arm",
            Self::BreakerReset {} => "breaker_reset",
            Self::NodeEnrollTokenIssue { .. } => "node_enroll_token_issue",
            Self::NodeRevoke { .. } => "node_revoke",
            Self::NodeCommandReplay { .. } => "node_command_replay",
            Self::NodeCommandQuarantine { .. } => "node_command_quarantine",
            Self::NodeInboxPrune { .. } => "node_inbox_prune",
            Self::ProfileCardImport { .. } => "profile_card_import",
            Self::CapabilityClaimAttest { .. } => "capability_claim_attest",
            Self::FederationDirectionPropose { .. } => "federation_direction_propose",
            Self::FederationDirectionAccept { .. } => "federation_direction_accept",
            Self::FederationDirectionRevoke { .. } => "federation_direction_revoke",
        }
    }

    /// Every discriminant, in declaration order. The migration's CHECK and this
    /// list are the same vocabulary; a control asserts they agree, so a variant
    /// added without its constraint cannot pass silently.
    pub const KINDS: [&'static str; 14] = [
        "grant_revoke",
        "boundary_revoke",
        "breaker_arm",
        "breaker_reset",
        "node_enroll_token_issue",
        "node_revoke",
        "node_command_replay",
        "node_command_quarantine",
        "node_inbox_prune",
        "profile_card_import",
        "capability_claim_attest",
        "federation_direction_propose",
        "federation_direction_accept",
        "federation_direction_revoke",
    ];
}

/// What the admitted operation finally did locally.
///
/// This is the fact an admission record cannot carry. `Applied` is the only
/// variant that asserts a protected change; the other two assert that protected
/// state and the tenant's revocation epoch are unchanged.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AdministrativeOutcome {
    /// The protected target changed. The domain tables carry WHAT changed; this
    /// record carries that it did, bound to the admission that permitted it.
    Applied {},
    /// The request was already satisfied, so nothing changed — the repeated
    /// revocation archetype. Protected state and the revocation epoch are
    /// unchanged, and the detail says why nothing was left to do.
    NoOp { detail: AdministrativeReason },
    /// A domain rule refused the operation AFTER admission. Protected state and
    /// the revocation epoch are unchanged. The code is the §9.8 registry name
    /// the request's own response used, so the record and the response cannot
    /// disagree about which refusal happened.
    ///
    /// ⛔ Closed on purpose: this build writes these codes, so it can only write
    /// ones it knows. A record naming a code outside the registry is a decode
    /// failure, never a guessed outcome — the same fail-closed stance stored
    /// authorization evidence already takes.
    Refused {
        code: KnownReasonCode,
        detail: AdministrativeReason,
    },
}

impl AdministrativeOutcome {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Applied {} => "applied",
            Self::NoOp { .. } => "no_op",
            Self::Refused { .. } => "refused",
        }
    }

    /// Whether this outcome asserts that a protected target changed.
    pub fn changed_protected_state(&self) -> bool {
        matches!(self, Self::Applied {})
    }

    pub const KINDS: [&'static str; 3] = ["applied", "no_op", "refused"];
}

/// The final effect of ONE admitted administrative request.
///
/// `record_id` is the admission's own id rather than a fresh identifier: the
/// link to the admission is then structural, and there is no second key to keep
/// synchronized with the first. `docs/CLAIM_VERIFICATION.md` leg 3 prefers one
/// derived source over N copies, and an effect record with no admission is not
/// a thing this model can represent.
///
/// `submitted_reason` is `Option` because the census found four of the fourteen
/// operations take a caller reason today. It is deliberately NOT `#[serde(default)]`:
/// this record has no history, so a missing field is malformed evidence rather
/// than an absent reason.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AdministrativeEffectRecord {
    /// The admission this effect is tied to; at most one effect per admission.
    pub record_id: AuthorizationRecordId,
    /// The tenant whose protected state the operation addressed.
    pub tenant_id: TenantId,
    pub operation: AdministrativeOperation,
    /// What the caller wrote, when the operation takes a reason at all.
    pub submitted_reason: Option<AdministrativeReason>,
    pub outcome: AdministrativeOutcome,
    /// Database time sampled when the effect was determined, inside the same
    /// transaction as the mutation.
    pub effected_at: DateTime<Utc>,
}

// ── Strict decoding ─────────────────────────────────────────────────────────────
//
// The pinned Serde internally tagged decoder also accepts a sequence
// representation, and a unit variant discards extra fields. Require a map first,
// then decode through empty-struct markers with `deny_unknown_fields`, passing
// `MapAccess` straight through so duplicate keys stay visible as evidence rather
// than being normalized away by a `serde_json::Value` round trip. This is the
// same shape `authority/evaluation.rs` uses, using its helper rather than a
// second copy of it.

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum OperationWire {
    GrantRevoke {
        grant_id: AdministrativeTargetId,
    },
    BoundaryRevoke {
        boundary_id: AdministrativeTargetId,
    },
    BreakerArm {},
    BreakerReset {},
    NodeEnrollTokenIssue {
        node_id: AdministrativeTargetId,
    },
    NodeRevoke {
        node_id: AdministrativeTargetId,
    },
    NodeCommandReplay {
        node_id: AdministrativeTargetId,
        command_id: AdministrativeTargetId,
    },
    NodeCommandQuarantine {
        node_id: AdministrativeTargetId,
        command_id: AdministrativeTargetId,
    },
    NodeInboxPrune {
        node_id: AdministrativeTargetId,
    },
    ProfileCardImport {
        card_digest: AdministrativeTargetId,
    },
    CapabilityClaimAttest {
        role_id: AgentRoleId,
        taxonomy_id: AdministrativeTargetId,
    },
    FederationDirectionPropose {
        remote_tenant_id: TenantId,
    },
    FederationDirectionAccept {
        remote_tenant_id: TenantId,
    },
    FederationDirectionRevoke {
        remote_tenant_id: TenantId,
    },
}

impl<'de> Deserialize<'de> for AdministrativeOperation {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(match object_only::<D, OperationWire>(deserializer)? {
            OperationWire::GrantRevoke { grant_id } => Self::GrantRevoke { grant_id },
            OperationWire::BoundaryRevoke { boundary_id } => Self::BoundaryRevoke { boundary_id },
            OperationWire::BreakerArm {} => Self::BreakerArm {},
            OperationWire::BreakerReset {} => Self::BreakerReset {},
            OperationWire::NodeEnrollTokenIssue { node_id } => {
                Self::NodeEnrollTokenIssue { node_id }
            }
            OperationWire::NodeRevoke { node_id } => Self::NodeRevoke { node_id },
            OperationWire::NodeCommandReplay {
                node_id,
                command_id,
            } => Self::NodeCommandReplay {
                node_id,
                command_id,
            },
            OperationWire::NodeCommandQuarantine {
                node_id,
                command_id,
            } => Self::NodeCommandQuarantine {
                node_id,
                command_id,
            },
            OperationWire::NodeInboxPrune { node_id } => Self::NodeInboxPrune { node_id },
            OperationWire::ProfileCardImport { card_digest } => {
                Self::ProfileCardImport { card_digest }
            }
            OperationWire::CapabilityClaimAttest {
                role_id,
                taxonomy_id,
            } => Self::CapabilityClaimAttest {
                role_id,
                taxonomy_id,
            },
            OperationWire::FederationDirectionPropose { remote_tenant_id } => {
                Self::FederationDirectionPropose { remote_tenant_id }
            }
            OperationWire::FederationDirectionAccept { remote_tenant_id } => {
                Self::FederationDirectionAccept { remote_tenant_id }
            }
            OperationWire::FederationDirectionRevoke { remote_tenant_id } => {
                Self::FederationDirectionRevoke { remote_tenant_id }
            }
        })
    }
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum OutcomeWire {
    Applied {},
    NoOp {
        detail: AdministrativeReason,
    },
    Refused {
        code: KnownReasonCode,
        detail: AdministrativeReason,
    },
}

impl<'de> Deserialize<'de> for AdministrativeOutcome {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(match object_only::<D, OutcomeWire>(deserializer)? {
            OutcomeWire::Applied {} => Self::Applied {},
            OutcomeWire::NoOp { detail } => Self::NoOp { detail },
            OutcomeWire::Refused { code, detail } => Self::Refused { code, detail },
        })
    }
}

// ⚠️ `Option<T>` alone does NOT make a field required: serde's missing-field
// path answers `visit_none` for an Option, so a writer that dropped the member
// would read back as "no reason was submitted". A `deserialize_with` field has
// no such fallback, so the member must be present — `null` when the operation
// takes no reason. Measured: the control for this refused the first draft.
fn explicit_reason<'de, D>(deserializer: D) -> Result<Option<AdministrativeReason>, D::Error>
where
    D: Deserializer<'de>,
{
    Option::deserialize(deserializer)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RecordWire {
    record_id: AuthorizationRecordId,
    tenant_id: TenantId,
    operation: AdministrativeOperation,
    #[serde(deserialize_with = "explicit_reason")]
    submitted_reason: Option<AdministrativeReason>,
    outcome: AdministrativeOutcome,
    effected_at: DateTime<Utc>,
}

impl<'de> Deserialize<'de> for AdministrativeEffectRecord {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let wire = object_only::<D, RecordWire>(deserializer)?;
        Ok(Self {
            record_id: wire.record_id,
            tenant_id: wire.tenant_id,
            operation: wire.operation,
            submitted_reason: wire.submitted_reason,
            outcome: wire.outcome,
            effected_at: wire.effected_at,
        })
    }
}
