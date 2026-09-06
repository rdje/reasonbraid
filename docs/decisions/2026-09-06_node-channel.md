# The WP3 node channel: the node reports its durable resume facts, the server replays the tail, and reconciliation gates schedulability

- **Type:** `decision`
- **Date:** `2026-09-06`
- **Status:** `active`
- **Owner / source:** engineering decision during leaf `PHASE-0.3.2` (WP3 outbound node channel with cursor resume + reconciliation handshake)
answers: how does the outbound node channel make reconnect exchange the last acknowledged server cursor and the pending local operation ids, guarantee that a duplicated command never creates a second local operation, and keep the node unschedulable until reconciliation completes?

## The fact / decision

**Server side** (`crates/reasonbraid-server/src/node_channel.rs`,
`migrations/0003_node_inbox.sql`): the control plane keeps a durable per-node inbox
(`node_inbox` — a monotonic per-node cursor, the command payload, acknowledgement state)
and deduplicated receipts of node-emitted events (`node_events`, keyed on the
NODE-assigned event id). The HTTP/1 JSON surface (axum) is
`POST /v1/nodes/handshake`, `POST /v1/nodes/events`, `POST /v1/nodes/ack`,
`GET /v1/nodes/poll`, all versioned (`channel_version: 1`) and `deny_unknown_fields`-strict.

**Node side** (`reasonbraid-node`: `src/channel.rs` client via reqwest, `src/node.rs`
lifecycle facade): `Node::reconcile` walks the reconnect protocol end to end — recover
crashed attempts, report the resume facts, journal the replay (deduplicated by command
id), apply the directives, re-emit pending events with their ORIGINAL ids (skipping the
ones the server reports as already held), acknowledge the cursor both sides, and ONLY
THEN become `Schedulable`. Any failure along the way returns the node to `Offline`; the
protocol is idempotent, so retrying is always safe.

The load-bearing rules:

- **The node is authoritative for what it durably holds.** Replay is computed from the
  cursor the node REPORTS, never from the server's own ack bookkeeping; the node's
  journal deduplicates. A node reporting a cursor ahead of the server's ledger is
  REFUSED with a typed `version_conflict` — a `journal_lost`-class anomaly, never a
  silent re-base.
- **Reconciliation concludes from server receipts.** An ambiguous attempt whose
  operation has a server-side event receipt is `adjudicated` (node marks it
  `reconciled` with the receipt as evidence); without a receipt the directive is
  `needs_adjudication` and the attempt stays `outcome_unknown` — bounded and visible,
  never silently retried. The pending-operation exchange is load-bearing: `known_events`
  lets the node skip re-sending already-delivered results.
- **Schedulability is the gate, not a suggestion.** `emit_event` refuses before
  reconciliation; the state machine is `Offline → Reconciling → Schedulable`, with the
  terminal step written only after the full handshake is applied (`§17.4` step 7).

## Why

`KICKOFF.md` WP3 acceptance items 2–4 and `ROADMAP.md` §17.4 (reconnect steps 1–7) /
§9.3 (node channel profile). The channel is the experiment for kill-risk question 1
("can a Rust control plane and Rust node preserve accepted work across crashes and
reconnects?") on the node leg — WP2 proved the control-plane leg.

## Measured behavior (legs: re-derive · falsify · durable)

- Re-derived against live PostgreSQL 16.15 + real `127.0.0.1` sockets:
  `bash scripts/run_pg_tests.sh` → `test result: ok. 13 passed` (`node_channel`) plus
  the `5 passed` (atomic) + `7 passed` (outbox worker) suites. Tests: fresh handshake
  plays the whole inbox; reconnect replays ONLY the tail after the reported cursor;
  **duplicate delivery leaves the same operation ids** (crash-window cursor rewind);
  **the node refuses new work until reconciliation completes** (and a failed reconcile
  keeps it unschedulable); both directive cases (no receipt → stays `outcome_unknown`;
  receipt → `reconciled`); pending events re-emitted with original ids, known events
  NOT re-sent; server restart resumes from the durable inbox; cursor-ahead refusal;
  poll tail; version mismatch (400) and forged fields (422) rejected; double event
  emission → one receipt.
- Falsified: the first channel run failed ONE test — not a protocol defect but a test
  fixture that emitted for an operation id the journal had never created (the FK
  correctly refused it). The protocol itself passed 11/11 on the first live run.
- Durable: the producer (both crates, migration 0003, the test suite) is tracked; the
  proof command is `scripts/run_pg_tests.sh`, mirrored by the `pg-tests` CI job, which
  now runs all three suites.

## Rejected designs

- **In-memory server inbox** — a server restart (a WP3 exercise) would silently lose
  the delivery ledger; the inbox is PostgreSQL-durable like everything the control
  plane owns (§17.1).
- **A bespoke TCP line protocol** — real sockets but throwaway code; axum/reqwest is
  the planned §9.3 HTTP stack, and the protocol semantics (cursor replay, original-id
  re-emission, directives) are transport-neutral.
- **Push-only delivery (SSE) in `.3.2`** — the poll endpoint is the minimal live path;
  the streaming profile arrives with ADR-006's formal record and WP6.
- **Node-command leases in `.3.2`** — WP3's lease exercise needs reservation semantics
  (WP5); dedupe by command id already makes redelivery safe. Deferred, recorded.
- **A shared wire crate** — the channel DTOs are duplicated per side with independent
  `deny_unknown_fields`; `reasonbraid-protocol` is a later roadmap step.
- **Schedulability blocked on every attempt being terminal** — ambiguity is bounded
  and visible, not blocking; only the mechanical reconciliation must complete. The
  node may hold `outcome_unknown` attempts and still be schedulable.
- **Trusting the server's own ack bookkeeping for replay** — the node's report is the
  resume point; server-side ack state exists for observability/operations, not replay.

## How to apply

- Never replay from the server's cursor — the node's reported cursor is the resume point.
- The channel is dev-only and unauthenticated until WP5 identity; never wire it to a
  non-loopback surface before then.
- The inbox ledger is append-only; new columns arrive via new root migrations.
- Keep `known_events` computed from the reported pending operations — that exchange is
  what makes lost event-acks recoverable without re-sending.
- Related: [[2026-09-06_node-journal]] (the durable leg), [[2026-09-06_outbox-worker-fencing]]
  (the control-plane delivery leg). ADR-006 (node transport) remains deferred to WP8's ADR set.
