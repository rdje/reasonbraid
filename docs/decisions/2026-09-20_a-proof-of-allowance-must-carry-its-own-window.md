---
answers:
  - What happens to a work item delivered after its budget reservation expired?
  - Why does the reservation reference carry `expires_at` and not just `issued_at`?
  - Why is the answer a refusal rather than a re-reservation at delivery?
  - Is ten minutes the right hold for a dispatched work item?
---
# A proof of allowance must carry its own window

- **Type:** decision
- **Status:** accepted; implemented
- **Owner:** `SIGNOFF-REPAIR.11.24.1.1.2.2`
- **Date:** 2026-09-20
- **Raised by:** `SIGNOFF-REPAIR.11.24.1.1.2`, while censusing what already bounds
  an undelivered command's useful life
- **Related:** `docs/decisions/2026-09-20_a-command-cannot-outlive-its-authority.md`,
  `docs/decisions/2026-09-20_revoking-an-authority-records-when.md`

## The defect

`dispatch_work_in_tx` creates a work item's budget reservation with a
**ten-minute** hold and puts the `ReservationReference` in the work payload.
The server's held-amount query counts an active reservation only while its
window is open —

```sql
CASE WHEN r.status = 'active' AND r.expires_at > $2
     THEN r.dimensions ELSE r.usage END
```

— and `usage` is NULL until settlement, so a lapsed, unsettled reservation
holds **nothing**. That is deliberate, and the server suite's
`expired_reservations_stop_holding` is the control that decided it.

🔴 **Nothing bound DELIVERY to the same window.** The node's §14.3 step 4 gate
called `ReservationReference::applicable()`, which checked that the reference
had an id and covered a call — and the reference carried `issued_at` and **no
window at all**. So a node polling at minute eleven received a work item whose
allowance the ceiling had already re-lent to other work, verified a proof it
could not date, and dispatched to a provider against a ceiling that reserved
nothing for it.

⛔ **The two halves of one rule read different clocks.** §14.3: *reservations
prevent two concurrent threads from each assuming the same remaining budget.*
Here one thread assumed it twice — once when the hold was issued, once when the
hold was gone and the work item still claimed it — and §14.6's *reservation plus
settled/estimated charges never silently exceeds hard ceiling* is what that
breaks. The overspend self-heals at settlement, which records the usage; the
invariant it breaks is the one that had to hold **before** the provider was
contacted.

## What is not claimed

⚠️ **That ten minutes is wrong.** The number is `PHASE-0.6.2`'s, and `.11.6`
forbids revising a number before measuring the population it quantifies over.
This record changes no duration.

⚠️ **That the server's ledger is wrong.** Stopping the hold at `expires_at` is
the documented and controlled behaviour. The gap was entirely on the other side
of the wire.

## The decision

✅ **`ReservationReference` carries `expires_at` — the ledger's own instant,
written by the same statement that issues the row** — and the node's gate
becomes `applicable_at(now)`, refusing a reservation whose hold has lapsed
before the adapter is contacted, journaling `failed_before_dispatch` with the
reason. That is the shape a budget denial at dispatch already takes
(`budget_denial_enqueues_work_without_a_reservation`).

⭐ **One value, written once.** The reference's window is read from the same
variable the `INSERT` binds, not recomputed from a duration, so the proof and
the ledger cannot drift apart. This is `revoking-an-authority-records-when`'s
rule applied one layer out: *the row the path loads must be able to date the
fact it reports.*

⛔ **The boundary is the ledger's.** `expires_at > $2` holds, so an instant
exactly equal to `expires_at` is already outside the window on both sides, and
the control drives the equality case rather than a point near it.

## The two answers refused, and why

⛔ **Re-reserve at delivery.** Minting a fresh hold when the node polls would
make the delivery path a budget authority: it creates an allowance outside the
transaction that admitted the dispatch and evaluated the caller's grant, on a
read path that is not the engine's single-writer serialization point. Its
failure leg is the refusal anyway, so it buys a success case at the cost of the
property that makes the reservation meaningful.

⛔ **Refuse the delivery (hold the row back).** The work item is already
durable and `.11.24.1.1.2` gave undelivered rows exactly two terminals, both
derived from the admitting grant's liveness. A budget window is not an
authority ending, and a third terminal derived from a different clock would put
a row in §10.6's vocabulary for a reason §10.6 does not name.

✅ **Deliver, and refuse at the node's own gate** — the answer §14.4 states
directly: *surface partial result and missing work instead of consuming an
unauthorized overrun.* The refusal is journaled at the node, audited, and
terminal, and the inbox row reaches a terminal rather than being held back.

## The wire change, stated

⚠️ `reservation.expires_at` is a **required** field of the work payload's
reservation object, which is decoded with `deny_unknown_fields`. A node and a
server across this change do not interoperate in either direction: an old node
refuses the new field, and a new node refuses a payload that omits it. Both
sides upgrade together. Dev profile, single deployment — recorded because the
next such change may not be.

## Verification

Falsified in both halves, each file restored byte-identical: removing the window
clause from `applicable_at` turns the core units and the node's
`supervisor_budget` control RED; making the issuer carry `issued_at` in place of
the window turns the server's `node_work` control RED. The node control drives
both sides of the boundary — one minute inside the window the same reservation
dispatches and completes.
