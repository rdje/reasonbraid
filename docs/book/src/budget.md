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

## The hold has a window, and the node reads it

A reservation holds its dimensions until `expires_at` and no longer: the
server's held-amount query stops counting an active row the instant that
passes, and the allowance returns to the ceiling for other work. A work item's
reservation is created with a ten-minute hold at dispatch.

Delivery is not instant. A node that is offline, slow to poll, or replaying a
backlog can receive a work item long after the hold lapsed — and until
`SIGNOFF-REPAIR.11.24.1.1.2.2` the reference it was handed carried only
`issued_at`, so the node ran the *verify an applicable reservation* check
against a proof it could not date. The ledger had re-lent the allowance while
the work item still claimed it, which is exactly what §14.3 says reservations
exist to prevent.

The reference now carries **`expires_at`, the ledger's own instant** — read
back out of the row with `RETURNING`, not computed a second time in the server,
so the proof and the ledger cannot disagree. A node handed a work item whose
hold has lapsed refuses it before the
adapter is contacted and journals `failed_before_dispatch` with the reason — the
same shape a budget denial at dispatch already takes. That is §14.4's rule:
*surface partial result and missing work instead of consuming an unauthorized
overrun.*

The refusal is deliberately **not** a re-reservation at delivery. Minting a
fresh hold on a poll would make the delivery path a budget authority, creating
an allowance outside the transaction that admitted the dispatch and evaluated
the caller's grant.

> **Wire change.** `reservation.expires_at` is a required field of the work
> payload's reservation object, which is decoded with `deny_unknown_fields`. A
> node and a server across this change do not interoperate: upgrade both.

> **Why `RETURNING` and not arithmetic** (`SIGNOFF-REPAIR.11.30`). The first
> version of this computed `at + held_for` in the server and put that value in
> both the insert and the reference. PostgreSQL `TIMESTAMPTZ` is
> **microsecond**-precision, so the row truncated it while the reference kept
> the nanosecond original — and the held-amount query stops counting an active
> reservation at `expires_at > $2`, reading the **stored** column. The ceiling
> therefore re-lent the capacity up to 999 ns before the node stopped honouring
> the proof: the same *two halves of one rule read different clocks* failure
> this section exists to describe, one precision further down. `issued_at` is
> read back for the same reason. The rule is general: **a value a node will
> verify against the ledger is read out of the store, never recomputed beside
> it.** The authority path reached the same conclusion independently — see
> *Expiry, suspension and revocation*, where windows are normalized to
> PostgreSQL microseconds before comparison.

## Settle, release, overrun

Settlement records **actual** usage: lower than the reservation frees the
difference; higher is an **overrun reported in full** (estimate errors feed
routing, §14.3 — never clamped). Release returns the unused hold. Expired
reservations stop holding. An **indeterminate** attempt keeps its hold
(§14.6: release only amounts not potentially consumed).

## Administering the spend breaker

The per-tenant spend latch is armed and reset through two `tenant_admin` routes.
Since `SIGNOFF-REPAIR.3.3.4.9` each runs **one** transaction under the tenant's
exclusive authority guard — the admission, the breaker write and a durable record
of what the operation finally did share a single commit — so an arm is ordered
against every in-flight reservation in the tenant and against any change to the
caller's own authority. The contract, the outcomes and what the `409` deliberately
does not tell the caller are in
[arming and resetting a spend breaker](authority.md#arming-and-resetting-a-spend-breaker).

The **trip** is a different thing and keeps its own path: it flips inside the
caller's reservation transaction the moment recorded spend plus the request
crosses the threshold, so the latch never lags the ledger it guards.

## Honest limits (Phase 0)

- Reservation references are unsigned (dev profile); workload-identity
  signatures arrive with WP7.
- The node's local ledger is in-memory; the server's ceiling is the durable
  counterpart.
- `.5.1`'s grant evaluation precedes this engine: a reservation requires an
  applicable grant, and a dispatch requires both.
