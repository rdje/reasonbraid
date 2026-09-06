# Budgets

The control plane enforces multi-dimensional budgets (`ROADMAP.md` §14):
calls, tokens in/out, and wall-clock — and a dimension that is not metered is
refused when requested, never silently allowed (fail closed, §14.6).

## Reserve before dispatch

The invariant is one sentence: **no provider dispatch begins without an
applicable reservation** — enforced at BOTH boundaries:

- **Server**: `create_reservation` checks the ceiling against everything
  currently held (active reservations + settled usage) in one transaction. A
  refusal is itself a recorded row.
- **Node**: the supervisor refuses to dispatch without a server-issued
  `ReservationReference`, and against its own `LocalBudget` headroom. A
  refusal is journaled `failed_before_dispatch` and the adapter is never
  invoked.

## Settle, release, overrun

Settlement records **actual** usage: lower than the reservation frees the
difference; higher is an **overrun reported in full** (estimate errors feed
routing, §14.3 — never clamped). Release returns the unused hold. Expired
reservations stop holding. An **indeterminate** attempt keeps its hold
(§14.6: release only amounts not potentially consumed).

## Honest limits (Phase 0)

- Reservation references are unsigned (dev profile); workload-identity
  signatures arrive with WP7.
- The node's local ledger is in-memory; the server's ceiling is the durable
  counterpart.
- `.5.1`'s grant evaluation precedes this engine: a reservation requires an
  applicable grant, and a dispatch requires both.
