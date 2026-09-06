# Identifier representation: branded newtypes over UUIDv7 with per-kind wire prefixes

- **Type:** `decision`
- **Date:** `2026-09-06`
- **Status:** `active`
- **Owner / source:** engineering decision during leaf `PHASE-0.1.1` (WP1 strong IDs)
answers: how are ReasonBraid identifiers represented so distinct things stay distinct in types and on the wire?

## The fact / decision

Every identifier is a **branded newtype** `Id<K>` over a **UUIDv7**, where `K` is a
zero-sized marker naming one family (tenant, human principal, host, node, agent role,
agent incarnation, run, thread). On the wire each serializes to `"{PREFIX}_{uuid}"`
with a **per-family prefix** validated on deserialization, so two families can never
serialize to the same form or be read back as each other. Construction is explicit
(`Id::<K>::new()` / `Id::<K>::from_uuid`) — there is deliberately **no blanket
`From<Uuid>`** for every kind, because that would let a caller silently re-brand any
UUID as any kind.

Prefixes (all three-letter, distinct):

| Family | Alias | Prefix | Label |
| --- | --- | --- | --- |
| Tenant | `TenantId` | `ten` | Tenant |
| HumanPrincipal | `HumanPrincipalId` | `hpr` | HumanPrincipal |
| Host | `HostId` | `hst` | Host |
| NodeInstance | `NodeId` | `nod` | NodeInstance |
| AgentRole | `AgentRoleId` | `rol` | AgentRole |
| AgentIncarnation | `AgentIncarnationId` | `inc` | AgentIncarnation |
| Run | `RunId` | `run` | Run |
| Thread | `ThreadId` | `thr` | Thread |

`agt` is deliberately **not** used: `ROADMAP.md` §9.1 sketches `agt_` for the *actor
principal reference* in the command/event envelope (WP1 `PHASE-0.1.2`) — a different
concept from the durable `AgentRole` identity.

## Why

`ROADMAP.md` §8.3 mandates "newtypes for every ID; never interchange plain UUID strings
inside domain code," and §17.2 prefers "UUIDv7 or an equivalently sortable opaque ID."
The branded-newtype pattern turns the confusable families (`AgentRole` vs
`AgentIncarnation` vs `Run`, and `Tenant` vs `Thread`) into *different Rust types*, so
mis-assignment is a compile error rather than a runtime bug; the per-family prefix makes
the same guarantee hold on the wire, where a bare UUID string would otherwise be
ambiguous.

## How to apply

- Add every future identifier family with the same `id_family!` macro in
  `crates/reasonbraid-core/src/id.rs`; choose a distinct three-letter prefix.
- Never add a blanket `From<Uuid> for Id<K>` impl; construction stays explicit by kind.
- Deserialization must keep validating the prefix — that check, not the type alone, is
  what makes the wire non-confusable.
- `ProviderAttemptId` belongs to `PHASE-0.1.3`, which owns the provider-attempt state
  machine; do not invent its ID here.
- This is ADR-010's eventual subject ("ID, time, causal ordering, clock-uncertainty
  rules"); if ADR-010 changes the underlying representation, only
  `crates/reasonbraid-core/src/id.rs` changes, because the UUID is hidden behind the
  newtype.
