# 2026-09-06_aggregate-library.md

## Context

`PHASE-1.1.1` extracted the proven WP2 transaction machinery (claim → locked
head → ordered event → state → outbox → result, one transaction) out of
`reasonbraid-server/src/tx.rs` and into the aggregate/event/outbox library
`reasonbraid-server::agg` — the single write path every future coordinator
aggregate composes over. The pattern decision is ADR-004
(`docs/adr/004-postgresql-aggregate-outbox.md`, `accepted`, evidence-gated).

## Decision

`agg` owns the six durability writes; `tx` stays as the Phase 0 public shape
delegating to it; authorization (`authority`) and domain validation (`threads`)
compose OVER the library inside one transaction — the modular-monolith pattern
made explicit. The `expected_revision` precondition defaults OFF.

## Consequences

- One auditable write path; hand-rolled claim/apply SQL outside `agg` is a
  review refusal.
- A separate store crate stays forbidden until a measured boundary need
  (ADR-002 extraction criteria).
- The shim owns no SQL, so the two modules cannot silently diverge.

answers:

- **How to extract a proven write path without changing behavior:** keep the
  old public shape as a thin shim that owns NO SQL — it only converts types and
  delegates to the library — and prove zero churn by switching no call site and
  re-running the full regression (offline suites + all live-PG suites + the
  two-host demo rode the new library with every acceptance check green). The
  shim documents its invariant (`expected_revision` always `None`) with an
  `unreachable!` on the impossible arm, so a future drift is a crash, not a
  silent divergence.
- **The library is the durability spine, not the domain:** the six writes are
  transaction mechanics; authorization and validation stay with their modules
  and compose over the library in the same transaction. Backlog 9's
  "authorization/audit context" belongs to the COMPOSITION (`crate::api`'s
  claim → authorize → validate → apply), not to the write path itself.
- **A revision precondition defaults OFF and stays honest:** `FOR UPDATE`
  takes no row lock when the aggregate has no row yet, so a fresh aggregate's
  serialization point is the primary-key insert — the doc names it instead of
  leaving it implicit.
