# ADR-013 — Budget units and settlement invariants, accepted with evidence

- **Status:** `accepted` (evidence-gated — the budget engine this record
  formalizes was built by the Phase-0 WP5 leaf and carried by every dispatch
  since)
- **Date:** `2026-09-07`
- **Leaf:** `PHASE-2.3.1` (reconciliation record — the budget model and the
  settlement invariants were decided and shipped by `PHASE-0.5.2` and the
  WP6 wiring)
- **Requirements:** `ROADMAP.md` §23 queue item 013; §14.1 (budget
  dimensions), §14.3 step 4 (reserve before dispatch), §14.6 (holds on
  indeterminate attempts)

## Context

The roadmap queued "budget units, pricing snapshots, and settlement
invariants" as ADR-013. The engine shipped first: the WP5 budget leaf
(`docs/decisions/2026-09-06_budget-reservation.md`) landed the model
(`BudgetDimensions`), the server-side engine, and the node-side dispatch gate,
and the `.6.2` wiring settled reservations with actual usage at the result
receipt. Pricing snapshots remain open — the honest dev slice is the
invariants; the snapshot machinery is named as the trigger below.

## Options

1. **Adopt the shipped budget invariants as the ADR**, naming the pricing-
   snapshot machinery as the deferred half.
2. Build pricing snapshots now (per-provider price tables, historical
   versions, settlement at snapshot prices) — nothing in the dev profile
   measures cost against multiple price versions yet; the single-tenant dev
   ledger has no such need.

## Evidence

- **The WP5 engine** (`docs/decisions/2026-09-06_budget-reservation.md`):
  dimensions are unknown-or-measured never zero; `covers` fails closed (a
  requested dimension the ceiling does not meter is refused); the server
  holds active reservations + settled usage against the ceiling in ONE
  transaction with a denial row for every refusal; settlement records ACTUAL
  usage — lower frees the difference, higher is an OVERRUN reported never
  clamped; release returns the hold; expired reservations stop holding
  (caller-supplied clock).
- **Both boundaries are one invariant twice** (§14.3 step 4): the server
  refuses to ISSUE what the ceiling cannot cover; the node refuses to
  DISPATCH what it has not been issued — an indeterminate attempt KEEPS its
  hold (§14.6: release only amounts not potentially consumed).
- **Measured, re-derived live** (`bash scripts/run_pg_tests.sh` → the
  `budget` suite): within-ceiling holds, beyond-ceiling denied AND recorded,
  settlement frees the remainder, overruns reported never clamped, double-
  settlement a no-op; node-side: a reservation covering no dispatch refuses
  BEFORE the boundary record, completed attempts settle actual usage, an
  indeterminate attempt keeps its hold.
- **The `.6.2` settlement** rides the result receipt: a node-reported usage
  settles the reservation in the SAME transaction as the contribution fold.

## Choice

Option 1. The dev-profile budget contract is the shipped engine: reserve
before dispatch at both boundaries, settle with actual usage, overruns
reported never clamped, denials recorded, holds on indeterminate attempts.
Pricing snapshots are deferred to Phase 4+ (below).

## Consequences

- Subtraction: no pricing-snapshot tables, no per-provider price versions,
  no settlement-at-snapshot-price machinery.
- The invariants are the ADR — any future pricing work must preserve them
  (a snapshot change never rewrites a settled reservation).

## Rollback / revisit trigger

- Multi-provider spend with prices that change between estimate and
  settlement (Phase 4 resource economics) — reopen pricing snapshots with
  the versioned-price machinery, keeping the settlement invariants this ADR
  pins.
