//! Command and event envelopes (`ROADMAP.md` §9.1–§9.2).
//!
//! Clients submit *intent* through [`CommandEnvelope`]; they do not assign
//! authoritative event fields. The server authenticates the connection, resolves the
//! actor, assigns tenant/sequence/authorization/timestamps, and commits a
//! [`CommittedEvent`]. A command envelope carries `#[serde(deny_unknown_fields)]`, so a
//! client-supplied authoritative field (actor, tenant, sequence, timestamps, authority)
//! is *rejected* rather than ignored or trusted (`ROADMAP.md` §9.1).

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::authority::TargetSelector;
use crate::id::{
    ActorPrincipalId, AuthorizationRecordId, CorrelationId, EventId, RequestId, TenantId, ThreadId,
};

/// The wire protocol version this build speaks (`ROADMAP.md` §9.1: `"reasonbraid/0.4"`).
pub const PROTOCOL_VERSION: &str = "reasonbraid/0.4";

/// A command submission from a client (`ROADMAP.md` §9.1).
///
/// The client supplies only intent: an operation, an idempotency key, an optional
/// expected aggregate version for optimistic concurrency, an opaque `body`, and a
/// correlation context. Authoritative fields — actor, tenant, sequence, timestamps,
/// authority — are absent from this type and are rejected by `deny_unknown_fields` if a
/// client tries to send them.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CommandEnvelope {
    pub protocol_version: String,
    /// Operation name, e.g. `"thread.create"`. The full operation catalogue is backlog 6.
    pub operation: String,
    pub request_id: RequestId,
    /// Client-generated idempotency key; opaque, no prefix (`ROADMAP.md` §9.2).
    pub idempotency_key: String,
    /// Optimistic-concurrency hint: the aggregate version the client believes is current.
    #[serde(default)]
    pub expected_aggregate_version: Option<u64>,
    /// Operation-specific payload, interpreted by the owning aggregate (§8.6).
    pub body: serde_json::Value,
    /// The delegation context (`.1.4.2`, ADR-009 — chain-in-envelope): present
    /// when the authenticated actor acts ON BEHALF OF another principal whose
    /// grant is the authority source.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authority_context: Option<AuthorityContext>,
    pub client_context: ClientContext,
}

/// The delegation context a command may carry (`.1.4.2`, ADR-009). The subject
/// rides a STRING field (`rol_…`/`hpr_…`): `GrantSubject` is a serde tagged
/// newtype and is not a wire field.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AuthorityContext {
    /// The principal the actor acts on behalf of (the grant holder).
    pub on_behalf_of: String,
    /// Why (audit context; the authorization record carries it).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub purpose: Option<String>,
    /// The scope the delegation may touch — the request's target must be
    /// within it (the §16.3 widening invariant measures against it).
    pub scope: TargetSelector,
}

/// Correlation context a client may attach to a command (`ROADMAP.md` §9.1).
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ClientContext {
    #[serde(default)]
    pub correlation_id: Option<CorrelationId>,
    #[serde(default)]
    pub causation_id: Option<RequestId>,
}

/// A committed event the server publishes after accepting a command (`ROADMAP.md` §9.1).
///
/// Every field is server-assigned; none is taken from a client's submission.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CommittedEvent {
    pub protocol_version: String,
    pub event_id: EventId,
    /// Event name, e.g. `"thread.created"`. The full event catalogue is backlog 6.
    pub event_type: String,
    pub tenant_id: TenantId,
    /// The aggregate this event mutates; Phase 0's only aggregate is a thread.
    pub aggregate_id: ThreadId,
    pub aggregate_version: u64,
    pub thread_sequence: u64,
    pub actor_principal_id: ActorPrincipalId,
    /// RFC 3339 wall-clock time; ADR-010 pins the exact time type.
    pub occurred_at: String,
    pub committed_at: String,
    #[serde(default)]
    pub correlation_id: Option<CorrelationId>,
    #[serde(default)]
    pub causation_id: Option<RequestId>,
    pub authorization_record_id: AuthorizationRecordId,
    pub schema_version: u64,
    pub body: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;
    use schemars::schema_for;

    const COMMAND_FIXTURE: &str = include_str!("../fixtures/command-thread-create.json");
    const EVENT_FIXTURE: &str = include_str!("../fixtures/event-thread-created.json");
    const REJECT_FIXTURE: &str = include_str!("../fixtures/command-with-authoritative-fields.json");
    const COMMAND_SCHEMA: &str = include_str!("../schema/command-envelope.schema.json");
    const EVENT_SCHEMA: &str = include_str!("../schema/committed-event.schema.json");

    #[test]
    fn command_fixture_round_trips_canonically() {
        let parsed: CommandEnvelope = serde_json::from_str(COMMAND_FIXTURE).unwrap();
        let fixture: serde_json::Value = serde_json::from_str(COMMAND_FIXTURE).unwrap();
        assert_eq!(serde_json::to_value(parsed).unwrap(), fixture);
    }

    #[test]
    fn event_fixture_round_trips_canonically() {
        let parsed: CommittedEvent = serde_json::from_str(EVENT_FIXTURE).unwrap();
        let fixture: serde_json::Value = serde_json::from_str(EVENT_FIXTURE).unwrap();
        assert_eq!(serde_json::to_value(parsed).unwrap(), fixture);
    }

    #[test]
    fn command_rejects_client_supplied_authoritative_fields() {
        // The checked-in golden fixture injects several authoritative fields at once.
        let err = serde_json::from_str::<CommandEnvelope>(REJECT_FIXTURE).unwrap_err();
        assert!(err.to_string().contains("unknown field"), "got: {err}");

        // Each authoritative field is rejected on its own, not merely as a group.
        let base: serde_json::Value = serde_json::from_str(COMMAND_FIXTURE).unwrap();
        for field in [
            "tenant_id",
            "actor_principal_id",
            "thread_sequence",
            "authorization_record_id",
            "occurred_at",
        ] {
            let mut tampered = base.clone();
            tampered[field] = serde_json::json!("tampered");
            let result = serde_json::from_value::<CommandEnvelope>(tampered);
            assert!(
                result.is_err(),
                "client-supplied `{field}` was not rejected"
            );
        }
    }

    #[test]
    fn schema_goldens_are_in_sync_with_types() {
        let got: serde_json::Value = serde_json::to_value(schema_for!(CommandEnvelope)).unwrap();
        let golden: serde_json::Value = serde_json::from_str(COMMAND_SCHEMA).unwrap();
        assert_eq!(
            got, golden,
            "CommandEnvelope schema drifted from checked-in golden"
        );

        let got: serde_json::Value = serde_json::to_value(schema_for!(CommittedEvent)).unwrap();
        let golden: serde_json::Value = serde_json::from_str(EVENT_SCHEMA).unwrap();
        assert_eq!(
            got, golden,
            "CommittedEvent schema drifted from checked-in golden"
        );
    }

    /// Regenerates the checked-in golden schema files. Run manually after changing an
    /// envelope type:
    /// `cargo test -p reasonbraid-core -- --ignored write_schema_goldens`.
    #[test]
    #[ignore]
    fn write_schema_goldens() {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/schema");
        std::fs::create_dir_all(dir).unwrap();
        let cmd = serde_json::to_string_pretty(&schema_for!(CommandEnvelope)).unwrap();
        std::fs::write(
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/schema/command-envelope.schema.json"
            ),
            cmd,
        )
        .unwrap();
        let evt = serde_json::to_string_pretty(&schema_for!(CommittedEvent)).unwrap();
        std::fs::write(
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/schema/committed-event.schema.json"
            ),
            evt,
        )
        .unwrap();
    }
}
