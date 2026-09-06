# Envelope representation: clients express intent, the server assigns every authoritative field

- **Type:** `decision`
- **Date:** `2026-09-06`
- **Status:** `active`
- **Owner / source:** engineering decision during leaf `PHASE-0.1.2` (WP1 envelopes)
answers: how do commands and events cross the wire so a client can only express intent and every authoritative event field is server-assigned and client-forgery-rejected?

## The fact / decision

ReasonBraid separates the *command* a client sends from the *event* the server commits. A
`CommandEnvelope` carries only intent — protocol version, an operation name, a
client-assigned `request_id` and idempotency key, an optional expected aggregate version,
an opaque `body`, and a `ClientContext` (`correlation_id`/`causation_id`). A
`CommittedEvent` carries only server-assigned authority — `event_id`, tenant, aggregate,
sequence, actor principal, RFC 3339 timestamps, correlation/causation, an
authorization-record reference, and a schema version. Both envelopes use
`#[serde(deny_unknown_fields)]`, so a client that smuggles in an authoritative field
(actor, tenant, sequence, timestamps, authority) is **rejected at deserialization**, not
silently ignored or trusted (`ROADMAP.md` §9.1: "client-supplied actor/timestamp/authority/
sequence/tenant ignored or rejected").

The wire protocol version is `"reasonbraid/0.4"` (`PROTOCOL_VERSION`). Envelope-scoped
identifiers add five families to `crates/reasonbraid-core/src/id.rs`:

| Family | Alias | Prefix | Role |
| --- | --- | --- | --- |
| Event | `EventId` | `evt` | a committed event's identity |
| Request | `RequestId` | `req` | a client request; also `causation_id` |
| Correlation | `CorrelationId` | `corr` | a correlation scope across commands/events |
| ActorPrincipal | `ActorPrincipalId` | `agt` | the authenticated actor the server resolved |
| AuthorizationRecord | `AuthorizationRecordId` | `authz` | a server-assigned authorization audit reference |

## Why

`ROADMAP.md` §9.1 already sketches both envelopes and the rule that the client supplies
only intent while the server supplies authority — but nothing enforced it. A plain struct
that merely *omits* authoritative fields would still accept them if a client sent extra
keys (serde ignores unknown fields by default), turning the security property into a
convention. `deny_unknown_fields` makes it mechanical: the same deserialization that
accepts a valid command rejects a forged one. Optional fields are nullable (`null` in
JSON, `#[serde(default)]` on read) so the wire form matches §9.1's explicit `null`s and
stays a single canonical shape.

## How to apply

- Never add a field to `CommandEnvelope` that a client could use to spoof authority; new
  authoritative fields belong on `CommittedEvent` (server-assigned) only.
- Keep `deny_unknown_fields` on both envelopes and on `ClientContext`; the acceptance test
  `command_rejects_client_supplied_authoritative_fields` in `envelope.rs` is the guard.
- Every wire payload the demo uses must have a golden fixture under
  `crates/reasonbraid-core/fixtures/` and a matching schema under
  `crates/reasonbraid-core/schema/`; regenerate schemas after a type change with
  `cargo test -p reasonbraid-core -- --ignored write_schema_goldens` (the drift test
  `schema_goldens_are_in_sync_with_types` fails CI otherwise).
- Timestamps are `String` (RFC 3339) for now; ADR-010 will pin the exact time type and
  then only this module's `occurred_at`/`committed_at` change.
- `ActorPrincipalId` is deliberately opaque: which principal kind (`hpr`/`rol`) an `agt`
  names is a WP5 (`PHASE-0.5`) identity concern — do not decode it here.
- See [[2026-09-06_id-representation]] for the identifier-newtype rule these families extend.
