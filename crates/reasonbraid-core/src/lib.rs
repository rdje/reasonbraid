//! # reasonbraid-core
//!
//! The ReasonBraid domain-model crate (`KICKOFF.md` §3): strong identifiers,
//! command/event envelopes, and the minimal thread/participation/provider-attempt
//! state machines.
//!
//! At this point (WP1) the strong identifiers, the command/event envelopes, and the
//! minimal state machines are implemented. This is the first real crate in the
//! repository.
//!
//! # Strong identifiers
//!
//! Every identifier is a branded newtype over a UUIDv7 ([`uuid::Uuid`]). Distinct
//! identifier families are distinct Rust types and serialize to distinct,
//! prefix-checked wire forms, so a tenant ID can never be mistaken for a thread ID
//! in code or on the wire. See [`id`] and
//! `docs/decisions/2026-09-06_id-representation.md`.
//!
//! # Envelopes
//!
//! Clients submit intent through [`CommandEnvelope`]; the server commits authoritative
//! [`CommittedEvent`]s. See [`envelope`] and
//! `docs/decisions/2026-09-06_envelope-representation.md`.
//!
//! # State machines
//!
//! [`ThreadState`], [`ParticipationState`], and [`ProviderAttemptState`] are minimal
//! orthogonal lifecycles whose only operation is a fallible, deterministic `apply`.
//! See [`state`] and `docs/decisions/2026-09-06_state-transitions.md`.
//!
//! # Errors and reason codes
//!
//! [`KnownReasonCode`] is the stable §9.8 registry, [`ReasonCode`] preserves unknown
//! codes, and [`DomainError`] is the typed, machine-actionable error. See [`error`] and
//! `docs/decisions/2026-09-06_reason-codes.md`.

mod authority;
mod budget;
mod envelope;
mod error;
mod id;
mod state;

pub use authority::{
    boundary_active_at, grant_active_at, grant_exceeds_boundary, policy_digest, AuthorityGrant,
    AuthorizationDecisionRecord, BoundaryStatus, BoundaryViolation, Decision,
    EnrollmentAuthorityBoundary, GrantAction, GrantStatus, GrantSubject, ResourceTarget, RiskClass,
    TargetSelector, UnknownAuthorityName,
};
pub use budget::{BudgetDimensions, BudgetError, ReservationReference};
pub use envelope::{ClientContext, CommandEnvelope, CommittedEvent, PROTOCOL_VERSION};
pub use error::{DomainError, KnownReasonCode, ReasonCode, Retryability};
pub use id::{
    ActorPrincipal, ActorPrincipalId, AgentIncarnation, AgentIncarnationId, AgentRole, AgentRoleId,
    AuthorizationRecord, AuthorizationRecordId, Correlation, CorrelationId, Event, EventId, Host,
    HostId, HumanPrincipal, HumanPrincipalId, Id, IdKind, IdParseError, NodeId, NodeInstance,
    ProviderAttempt, ProviderAttemptId, Request, RequestId, Run, RunId, Tenant, TenantId, Thread,
    ThreadId,
};
pub use state::{
    ParticipationState, ParticipationTransition, ProviderAttemptState, ProviderAttemptTransition,
    ThreadState, ThreadTransition, TransitionError, UnknownProviderAttemptState,
};
