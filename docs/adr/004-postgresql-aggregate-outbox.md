# ADR-004 — PostgreSQL aggregate/event/outbox pattern: locked head, claim-first, one-transaction writes

- **Status:** `accepted` (evidence-gated — acceptance rests on the executable
  WP2 kill-point proofs and the `.1.1.1` re-proof; no gate signature is required
  for a pattern record)
- **Date:** `2026-09-06`
- **Leaf:** `PHASE-1.1.1`
- **Requirements:** `ROADMAP.md` §8.6, §17.2–§17.3, §9.2; KICKOFF WP2; backlog 9
- **Predecessors:** `docs/decisions/2026-09-06_atomic-transaction.md`,
  `docs/decisions/2026-09-06_outbox-worker-fencing.md` (the Phase 0 proofs this
  record formalizes)

## Context

Backlog 9 asks for an "aggregate transaction library: revision checks, events,
outbox, authorization/audit context, and test helpers". Phase 0 built the
machinery inline in `reasonbraid-server/src/tx.rs` and proved it under kill
points (WP2). Phase 1 needs the machinery as a typed, reusable library so the
coordinator's future aggregates (identity, membership, jobs) ride the same
durability spine instead of re-deriving it — while the modular-monolith stance
(ADR-002; KICKOFF §3: no new crate without a measured boundary need) forbids
premature extraction into a separate store crate.

## Options

1. **In-server library module + typed compatibility shim** — extract the six
   writes into `agg`, keep `tx` as the Phase 0 public shape delegating to it.
2. **New `reasonbraid-store-pg` crate** — the roadmap's §7.1 end-state, but a
   new crate boundary with no measured isolation/scale/ownership need yet.
3. **Leave it inline** — each future aggregate re-implements the writes (the
   drift risk the library removes).

## Evidence

- WP2 kill-point proofs (`.2.1`/`.2.2`): 5 atomic-transaction + 7 outbox-worker
  live-PG tests — successful responses imply committed state; same-key/same-hash
  replays; hash conflicts; stale leased workers cannot commit after a newer
  fencing value.
- `.1.1.1` re-proof through the library's own surface: `tests/aggregate_library.rs`
  (fresh-apply vs replay, conflict, the revision precondition, outbox→event
  integrity) green against live PostgreSQL 16.15.
- Zero-behavior-change proof: every Phase 0 caller keeps its exact `tx::` shape;
  the full offline + live-PG regression (all eight server suites, CLI e2e, the
  two-host demo) green at the leaf commit.

## Choice

**Option 1.** `reasonbraid-server::agg` is the single write path — idempotency
claim (the `(tenant_id, idempotency_key)` primary key is the serialization
point), locked-head revision (`SELECT … FOR UPDATE`; the next version derives
from it), ordered event append, current-state upsert, outbox enqueue (the FK
proves an outbox item implies its event is durable), and the semantic result —
all in one transaction, composed by callers with authorization and domain
validation in the same transaction. An optional `expected_revision` precondition
turns the lock into an optimistic-concurrency check; it defaults OFF, so
existing behavior is byte-for-byte preserved. The outbox remains the durable job
store until Phase 2's retry policy measures a need for a richer `jobs` table
(§20.4 owns that; a column arrives then, not a guessed shape now).

## Consequences

- Easier: one auditable write path; new aggregates (`.1.1.2` identity,
  `.1.1.3` thread API completion) compose over it; the revision precondition is
  available the moment a caller needs optimistic concurrency.
- Harder: two modules (`agg` + the `tx` shim) must stay in agreement — enforced
  by the fact that the shim contains no SQL and delegates every write.
- Forbidden: hand-rolled claim/apply SQL outside `agg`; a separate store crate
  before a measured boundary need.

## Rollback / revisit trigger

Revisit (extract `reasonbraid-store-pg`, or change the serialization point)
when a measured isolation, scale, security, or ownership need names it — per
ADR-002's extraction criteria, not before. The `expected_revision` default may
flip to mandatory optimistic concurrency if a future workload measures head-lock
contention.

## answers:

Recorded in the layer-C decision record
`docs/decisions/2026-09-06_aggregate-library.md` (`answers:` present) — the
durable lesson this leaf promoted: extraction-without-behavior-change (the
SQL-free shim + zero-call-site-churn proof), the library-is-the-spine-not-the-
domain boundary, and the fresh-aggregate serialization-point note.
