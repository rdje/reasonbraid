# ADR-006 — Node transport and reconnect protocol: the authenticated outbound channel, accepted with evidence

- **Status:** `accepted` (evidence-gated — the decisions this record promotes were
  proven by WP3 and hardened by `PHASE-1.2.2` before this ADR was written)
- **Date:** `2026-09-07`
- **Leaf:** `PHASE-2.1.1` (reconciliation record — the transport itself was decided
  and shipped by the Phase-0/1 channel leaves)
- **Requirements:** `ROADMAP.md` §23 queue item 006; §11.1 (node responsibilities);
  §9.3 (node channel); §17.4 (reconnect)

## Context

The roadmap queued "node transport and reconnect protocol" as ADR-006 but the
implementation ran ahead of the record: WP3 built the outbound channel
(`docs/decisions/2026-09-06_node-channel.md`), and `.1.2.2` upgraded it to the
authenticated contract (`docs/decisions/2026-09-06_node-channel-auth.md`). The
record was never promoted to an ADR, leaving the queue item nominally open while
the transport is shipped and green. `PHASE-2.1` now touches the same boundary
(the certificate lifecycle), so the decision must exist as an ADR before the
wire changes again.

## Options

1. **Adopt the shipped transport as the ADR** (promote the decision records to
   an accepted ADR with their evidence).
2. Re-open the transport question (WebSocket/SSE, gRPC/HTTP-2, NATS) — nothing
   in the measured evidence asks for this: the channel survives real kill
   points and the LAN latency floor is the provider, not the transport.

## Evidence

- The WP3 channel: the node reports durable resume facts (cursor + pending
  operations), the server replays the tail and reconciles from receipts, and
  schedulability gates on the applied handshake (`tests/node_channel.rs`, 13
  tests pre-`.1.2.2`).
- The `.1.2.2` authenticated contract (`CHANNEL_VERSION` 2): HMAC key-proof
  handshake (constant-time; refused before any ledger read), lease + fencing
  token, POST-only poll (a credential never rides a query string), heartbeat,
  derived presence. 17 channel tests + the two-host demo's real SIGKILL beats.
- The demo proves reconnect end-to-end: server SIGKILL + restart loses no
  accepted command; node SIGKILL after dispatch → exactly one `outcome_unknown`.

## Choice

Option 1. The node channel is a plain axum HTTP/1 outbound channel with the
`.1.2.2` authenticated contract, in steady-state polling with a heartbeat; the
transport is not the identity (the credential is), so the wire contract survives
a transport-layer upgrade.

## Consequences

- A broker/streaming transport is NOT adopted (subtraction: no NATS, no
  WebSocket/SSE server). Revisit only from a measured need (Phase 3+ fan-out).
- Transport-level identity (mTLS) rides ADR-007's certificate lifecycle and
  changes the channel auth (CHANNEL_VERSION 3 in `PHASE-2.1.2`) without
  changing the reconnect semantics this ADR pins.
- The dev HMAC key-proof retires for enrolled nodes once `.1.2` lands; the
  replay/cursor/fencing semantics stay.

## Rollback / revisit trigger

- A measured delivery need that plain HTTP/1 polling cannot meet (storm
  fan-out, bidirectional push) — reopen with numbers, not anticipation.
- Any non-loopback exposure before `PHASE-2.1.2` lands (the dev trust store is
  loopback-only).
