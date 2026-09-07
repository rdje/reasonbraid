# ADR-022 — Audit hash-chain: the shipped linkage is the groundwork; the chain itself is a named trigger

- **Status:** `accepted` (evidence-gated — every binding this record pins is
  the SHIPPED design: the authorization decision's actor/subject/grant/digest
  bindings, the deterministic handles, the per-thread monotonic sequence, the
  audit reconstruction over the read surfaces)
- **Date:** `2026-09-07`
- **Leaf:** `PHASE-2.7.3`
- **Requirements:** `ROADMAP.md` §23 queue item 022 (audit hash-chain/
  checkpoint and verification policy); §5's audit attribute ("every binding
  transition links actor, authority, proposal, and prior state")

## Context

The roadmap queued "audit hash-chain/checkpoint and verification policy" as
ADR-022. The exit-lane census found the audit LINKAGE shipped but the CHAIN
unbuilt: every authorization decision commits an `authorization_records` row
(actor, subject, grant/boundary reference, decision, policy digest +
version), every committed event carries a unique monotonically increasing
thread sequence, the actor handles are deterministically derived (UUIDv5) so
rows stay linkable, and the audit reconstruction (the demo's leg) rebuilds
the whole story through the supported read surfaces. What does NOT exist: a
hash chain linking consecutive audit rows, a checkpoint scheme, and the
verification policy that consumes them. The open question is what the ADR
should PIN now: the chain's design against the linkage that already exists.

## Options

1. **Accept the linkage as the groundwork; defer the chain with its trigger**
   — the shipped bindings ARE the chain's content-addressing substrate (each
   row already names its actor, authority, proposal, and prior state); the
   hash chain + checkpoint machinery lands when a deployment profile makes
   tamper-evidence of the DURABLE row stream operationally necessary.
2. Build the hash chain now — a `prev_hash` column, a checkpoint table, and
   a verification CLI for a single-node, trusted-LAN dev profile whose
   database is the private, unshared store: machinery with no threat model
   to answer to (the subtraction doctrine's placeholder-infrastructure lie).

## Evidence

- **The linkage is shipped**: `authorization_records` carries the actor
  (the UUIDv5 handle — deterministic, linkable), the subject (the
  delegation chain binds the principal), the grant/boundary reference, the
  decision, and the policy digest + version (`.5.1`; `docs/decisions/
  2026-09-06_control-api-cli.md` records the handle derivation).
- **The sequence is shipped**: every committed event carries a unique
  monotonically increasing thread sequence (§5's per-thread order —
  `aggregate_state`/`event_log`).
- **The verification surface is shipped**: the audit reconstruction (the
  demo's `.1.8.1` leg, 34 checks) rebuilds commands, participants,
  authority, costs, and stop reasons through the supported read surfaces
  only — the dev profile's audit-verification story.

## Decision

Accept option 1. The hash chain is the STRENGTHENING of an already-durable
linkage, not a missing control: the trigger that re-opens it is the first
deployment profile whose audit row stream is exposed to a threat (a
non-loopback deployment or the G7 ops gate — the ADR-023 trigger family),
at which point the chain design (row-level `prev_hash` over the
deterministic canonical serialization, checkpoint rows, and the verification
policy) lands WITH its consumer.

## Consequences

- The dev profile claims linkage + reconstruction, NOT tamper-evidence of
  the durable stream; the G6–G7 feed records exactly that boundary.
- The chain's canonical serialization is pre-shaped by the shipped fields
  (the actor/subject/grant/digest bindings) — the future chain has nothing
  to re-derive.
- No `prev_hash` column, no checkpoint table, no verification CLI in the dev
  profile (the subtraction doctrine).

## Revisit trigger

The first non-loopback deployment or the G7 ops gate (whichever fires
first) — the chain + checkpoint + verification policy land as their own
leaf with the ADR-022 evidence package.
