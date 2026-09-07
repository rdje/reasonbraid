# ADR-005 — Event transport: the PostgreSQL queue, accepted with evidence

- **Status:** `accepted` (evidence-gated — the decision this record promotes was
  built and proven by the Phase-0 WP2 outbox worker and shipped by every
  subsequent delivery leaf before this ADR was written)
- **Date:** `2026-09-07`
- **Leaf:** `PHASE-2.2.1` (reconciliation record — the transport itself was
  decided and shipped by the Phase-0/1 delivery leaves)
- **Requirements:** `ROADMAP.md` §23 queue item 005; §17.3 (the durable
  outbox); §9.3 (the node channel); backlog 5

## Context

The roadmap queued "event transport: PostgreSQL queue initially vs NATS
JetStream or other broker" as ADR-005, but the implementation ran ahead of the
record: WP2 built the leased outbox worker over the PostgreSQL queue
(`docs/decisions/2026-09-06_outbox-worker-fencing.md`), ADR-004 formalized the
aggregate/event/outbox pattern, and ADR-006 accepted the node channel whose
pull surface consumes it. The record was never promoted, leaving the queue item
nominally open while the transport is shipped and green across two phases.

## Options

1. **Adopt the shipped PostgreSQL queue as the ADR** (promote the decision
   records to an accepted ADR with their evidence).
2. **Adopt a broker** (NATS JetStream or similar) — a new infrastructure
   dependency, a second durability store beside PostgreSQL, and a migration of
   the fencing/lease/acknowledgement semantics the WP2 worker already proves.

## Evidence

- **The Phase-0 WP2 outbox worker is the PostgreSQL queue, proven**: claim with
  a per-row fencing token (`FOR UPDATE SKIP LOCKED`, concurrent workers cannot
  double-claim), deliver into a deduped sink, acknowledge only with the current
  token AND a live lease; kill points 3–5 proven with deterministic time
  (`docs/decisions/2026-09-06_outbox-worker-fencing.md`).
- **ADR-004** formalized the aggregate/event/outbox pattern whose outbox IS the
  queue — one transaction writes the event and the delivery row (an outbox item
  implies its event is durable).
- **ADR-006** accepted the node channel that consumes the queue: the per-node
  inbox is a PostgreSQL table, the node pulls its tail by cursor and resumes
  from its durably-reported facts — replay, quarantine, and acknowledgement all
  ride the queue's rows (`CHANNEL_VERSION` 4 after `.1.5.2`).
- **Two phases of delivery leaves shipped on it**: invitations/dispatch,
  retention/quarantine (`.1.2.3`), revocation refusals, the cached-decision
  metadata (`.1.5.2`), and the incarnation/run linkage (`.1.6`) — the queue
  has carried every delivery contract without a broker.

## Choice

Option 1. The event transport is the PostgreSQL queue: the WP2 leased outbox
worker for server-side jobs and the per-node inbox consumed by the pull-based
channel. No broker is adopted.

## Consequences

- Subtraction: no NATS/JetStream deployment, no second durability store, no
  broker ops surface — the queue's operational story is PostgreSQL's (backup,
  PITR, monitoring: `.4`).
- The fencing/lease semantics of the WP2 worker are the queue's production
  contract (a stale worker can never commit after a newer claim).

## Rollback / revisit trigger

- A measured need the PostgreSQL queue cannot meet — storm fan-out with
  sub-second delivery to many consumers, bidirectional push, or
  cross-datacenter replication requirements — reopen the broker option with
  numbers, not anticipation (the ADR-006 precedent).
