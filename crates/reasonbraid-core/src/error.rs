//! Typed errors and the stable reason-code registry (`ROADMAP.md` §9.8).
//!
//! §9.8 makes errors machine-actionable: a stable code, a retryability, a safe human
//! message, a correlation id, and visibility-filtered details. [`KnownReasonCode`] is the
//! stable §9.8 registry (snake_case on the wire); [`ReasonCode`] wraps it so a code this
//! build does not recognize is *preserved* verbatim rather than dropped — the WP1
//! acceptance "unknown codes remain preservable".

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::id::CorrelationId;

/// The stable reason-code registry (`ROADMAP.md` §9.8), snake_case on the wire.
///
/// This is the *complete* §9.8 list, not a Phase-0 subset: a stable registry must not be
/// carved up and re-churned leaf by leaf. The demo consumes a subset; codes the demo never
/// emits are still part of the registry so a client and server can name them consistently.
/// Codes *beyond* this list (future versions) are handled by [`ReasonCode::Unknown`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum KnownReasonCode {
    Unauthenticated,
    Unauthorized,
    ScopeHidden,
    InvalidCommand,
    InvalidTransition,
    VersionConflict,
    IdempotencyMismatch,
    RateLimited,
    BudgetUnavailable,
    ResourceUnresolvable,
    ResourceDenied,
    EvidenceQuarantined,
    ProviderOutcomeUnknown,
    RetryRequiresAuthorization,
    NoQuorum,
    ApprovalExpired,
    PublicationConflict,
    DeploymentPartial,
    DependencyUnavailable,
    ProtocolIncompatible,
}

impl KnownReasonCode {
    /// The stable wire/record name (snake_case, matches the serde serialization).
    pub fn as_str(self) -> &'static str {
        match self {
            KnownReasonCode::Unauthenticated => "unauthenticated",
            KnownReasonCode::Unauthorized => "unauthorized",
            KnownReasonCode::ScopeHidden => "scope_hidden",
            KnownReasonCode::InvalidCommand => "invalid_command",
            KnownReasonCode::InvalidTransition => "invalid_transition",
            KnownReasonCode::VersionConflict => "version_conflict",
            KnownReasonCode::IdempotencyMismatch => "idempotency_mismatch",
            KnownReasonCode::RateLimited => "rate_limited",
            KnownReasonCode::BudgetUnavailable => "budget_unavailable",
            KnownReasonCode::ResourceUnresolvable => "resource_unresolvable",
            KnownReasonCode::ResourceDenied => "resource_denied",
            KnownReasonCode::EvidenceQuarantined => "evidence_quarantined",
            KnownReasonCode::ProviderOutcomeUnknown => "provider_outcome_unknown",
            KnownReasonCode::RetryRequiresAuthorization => "retry_requires_authorization",
            KnownReasonCode::NoQuorum => "no_quorum",
            KnownReasonCode::ApprovalExpired => "approval_expired",
            KnownReasonCode::PublicationConflict => "publication_conflict",
            KnownReasonCode::DeploymentPartial => "deployment_partial",
            KnownReasonCode::DependencyUnavailable => "dependency_unavailable",
            KnownReasonCode::ProtocolIncompatible => "protocol_incompatible",
        }
    }
}

impl std::fmt::Display for KnownReasonCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A machine-actionable reason code: a known §9.8 code, or a preserved unknown code.
///
/// `#[serde(untagged)]` tries the known registry first, then falls back to the raw string,
/// so a code from a newer build deserializes into [`ReasonCode::Unknown`] (round-trippable)
/// instead of erroring or collapsing to a generic name.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum ReasonCode {
    Known(KnownReasonCode),
    /// A code this build's registry does not recognize; the original string is preserved.
    Unknown(String),
}

impl ReasonCode {
    pub fn as_str(&self) -> &str {
        match self {
            ReasonCode::Known(k) => k.as_str(),
            ReasonCode::Unknown(s) => s.as_str(),
        }
    }
}

impl std::fmt::Display for ReasonCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<KnownReasonCode> for ReasonCode {
    fn from(code: KnownReasonCode) -> Self {
        ReasonCode::Known(code)
    }
}

/// Whether (and how) an error may be retried (`ROADMAP.md` §9.8: `retry_requires_authorization`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Retryability {
    No,
    Yes,
    RequiresAuthorization,
}

/// A typed, machine-actionable error (`ROADMAP.md` §9.8).
///
/// Internal secrets, policy internals, and cross-tenant existence are not carried here:
/// `message` is a safe human-facing string and `details` is already visibility-filtered by
/// the caller before it is constructed.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct DomainError {
    pub code: ReasonCode,
    pub retryability: Retryability,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<CorrelationId>,
    #[serde(default)]
    pub details: serde_json::Value,
}

impl DomainError {
    /// Construct a typed error with no correlation id and empty details.
    pub fn new(code: ReasonCode, retryability: Retryability, message: impl Into<String>) -> Self {
        DomainError {
            code,
            retryability,
            message: message.into(),
            correlation_id: None,
            details: serde_json::Value::Null,
        }
    }
}

impl std::fmt::Display for DomainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for DomainError {}

/// A deterministic state-machine rejection (`PHASE-0.1.3`) is a typed error whose reason
/// code is `invalid_transition` and which is not retryable as-is.
impl From<crate::state::TransitionError> for DomainError {
    fn from(e: crate::state::TransitionError) -> Self {
        DomainError::new(
            KnownReasonCode::InvalidTransition.into(),
            Retryability::No,
            e.to_string(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every known code round-trips to its snake_case name, and the registry holds exactly
    /// the §9.8 list (20 codes).
    #[test]
    fn known_codes_round_trip_to_snake_case() {
        let codes: &[KnownReasonCode] = &[
            KnownReasonCode::Unauthenticated,
            KnownReasonCode::Unauthorized,
            KnownReasonCode::ScopeHidden,
            KnownReasonCode::InvalidCommand,
            KnownReasonCode::InvalidTransition,
            KnownReasonCode::VersionConflict,
            KnownReasonCode::IdempotencyMismatch,
            KnownReasonCode::RateLimited,
            KnownReasonCode::BudgetUnavailable,
            KnownReasonCode::ResourceUnresolvable,
            KnownReasonCode::ResourceDenied,
            KnownReasonCode::EvidenceQuarantined,
            KnownReasonCode::ProviderOutcomeUnknown,
            KnownReasonCode::RetryRequiresAuthorization,
            KnownReasonCode::NoQuorum,
            KnownReasonCode::ApprovalExpired,
            KnownReasonCode::PublicationConflict,
            KnownReasonCode::DeploymentPartial,
            KnownReasonCode::DependencyUnavailable,
            KnownReasonCode::ProtocolIncompatible,
        ];
        assert_eq!(codes.len(), 20, "§9.8 registry is 20 codes");
        for &code in codes {
            let wire = serde_json::to_string(&code).unwrap();
            assert_eq!(wire, format!("\"{}\"", code.as_str()), "as_str/serde drift");
            let back: KnownReasonCode = serde_json::from_str(&wire).unwrap();
            assert_eq!(back, code);
        }
    }

    /// The WP1 acceptance: a code this build does not know is preserved verbatim.
    #[test]
    fn unknown_codes_are_preserved_verbatim() {
        let raw = "future_semantic_reason";
        let code: ReasonCode = serde_json::from_str(&format!("\"{raw}\"")).unwrap();
        assert_eq!(code, ReasonCode::Unknown(raw.to_string()));
        // Re-serialization preserves the exact string, not a collapsed name.
        assert_eq!(serde_json::to_string(&code).unwrap(), format!("\"{raw}\""));
    }

    #[test]
    fn known_and_unknown_are_discriminated() {
        let known: ReasonCode = serde_json::from_str("\"invalid_transition\"").unwrap();
        assert_eq!(known, ReasonCode::Known(KnownReasonCode::InvalidTransition));

        // A near-miss is preserved, not misclassified as a known code.
        let near: ReasonCode = serde_json::from_str("\"invalid_transition_typo\"").unwrap();
        assert!(matches!(near, ReasonCode::Unknown(_)));
    }

    #[test]
    fn retryability_serializes_snake_case() {
        assert_eq!(
            serde_json::to_string(&Retryability::RequiresAuthorization).unwrap(),
            "\"requires_authorization\""
        );
    }

    #[test]
    fn domain_error_round_trips() {
        let err = DomainError {
            code: KnownReasonCode::InvalidTransition.into(),
            retryability: Retryability::No,
            message: "invalid Thread transition: `closed` -(begin_close)-> rejected".to_string(),
            correlation_id: None,
            details: serde_json::Value::Null,
        };
        let wire = serde_json::to_value(&err).unwrap();
        let back: DomainError = serde_json::from_value(wire).unwrap();
        assert_eq!(back, err);
    }

    /// A state-machine rejection classifies as `invalid_transition`, wiring the §9.8
    /// registry to the deterministic `apply` from PHASE-0.1.3.
    #[test]
    fn transition_error_maps_to_invalid_transition() {
        use crate::state::{ThreadState, ThreadTransition};
        let te = ThreadState::Closed
            .apply(ThreadTransition::BeginClose)
            .unwrap_err();
        let err: DomainError = te.into();
        assert_eq!(
            err.code,
            ReasonCode::Known(KnownReasonCode::InvalidTransition)
        );
        assert_eq!(err.retryability, Retryability::No);
        assert!(err.message.contains("closed"));
    }
}
