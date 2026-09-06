//! # reasonbraid-core
//!
//! The ReasonBraid domain-model crate (`KICKOFF.md` §3): strong identifiers,
//! command/event envelopes, and minimal thread and provider-attempt state.
//!
//! At this point (WP1) only the strong identifiers are implemented — envelopes and
//! state machines arrive in later WP1 leaves. This is the first real crate in the
//! repository.
//!
//! # Strong identifiers
//!
//! Every identifier is a branded newtype over a UUIDv7 ([`uuid::Uuid`]). Distinct
//! identifier families are distinct Rust types and serialize to distinct,
//! prefix-checked wire forms, so a tenant ID can never be mistaken for a thread ID
//! in code or on the wire. See [`id`] and
//! `docs/decisions/2026-09-06_id-representation.md`.

mod id;

pub use id::{
    AgentIncarnation, AgentIncarnationId, AgentRole, AgentRoleId, Host, HostId, HumanPrincipal,
    HumanPrincipalId, Id, IdKind, IdParseError, NodeId, NodeInstance, Run, RunId, Tenant, TenantId,
    Thread, ThreadId,
};
