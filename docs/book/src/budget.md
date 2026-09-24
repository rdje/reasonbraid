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

## Wall-clock time is spent, like tokens

Every dimension of a ceiling is cumulative: the server holds it against active
reservations **plus settled usage**. Wall-clock time is charged the same way
(`SIGNOFF-REPAIR.4.4.6.1`). The node's supervisor times each attempt from its
dispatch record to its end, rounds **up** to whole seconds (at least one, so an
attempt never costs less than it took), and charges its local ledger. It
reports the seconds to the control plane beside the tokens:

```json
"usage": { "input_tokens": 41, "output_tokens": 17, "wall_clock_seconds": 2 }
```

The settlement records them on the reservation, and later reservations meet
them against the ceiling's clock.

Before this repair every settlement recorded `wall_clock_seconds: null`. Each
attempt handed its reserved time back when it settled, so a ceiling's clock
limited only the attempts running *at the same moment*. However long the
thread's attempts took in total, its clock never ran out. The per-attempt
limit is a separate matter: that is the request's deadline, enforced since
`SIGNOFF-REPAIR.4.4.6` (the adapter chapter).

## The hold has a window, and the node reads it

A reservation holds its dimensions until `expires_at` and no longer (with one
exception, the next section): the server's held-amount query stops counting an
active row the instant that passes, and the allowance returns to the ceiling
for other work. A work item's
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

## A hold whose outcome is unknown outlives its window

When a provider response is lost, the node records the attempt as
`outcome_unknown` and refuses to retry it without authorization. No result will
ever settle its reservation. The call may still have been charged, and §14.6
releases *only amounts not potentially consumed*.

Until `SIGNOFF-REPAIR.4.5.1` the window alone decided. Ten minutes after
dispatch the lost call's allowance went back to the ceiling and was lent again,
so a possible-duplicate re-run could fit under a ceiling that had room for one
call only.

Now the node's `retry_requires_authorization` dead letter is the server's cue.
In the same guarded transaction that quarantines the inbox row, the server
stamps that row's reservation with `outcome_unknown_at`. It uses the
reservation the server stored on the row, never one the node names. The rule
for "still held" lives in one place (`budget::holding!`), and admission, the
spend breaker and `GET /v1/admin/usage` all use it:

```sql
r.status = 'active' AND (r.expires_at > $2 OR r.outcome_unknown_at IS NOT NULL)
```

`GET /v1/threads/{thread_id}/budget` shows the stamp on the reservation row. A
hold whose window had already closed when the report arrived is stamped anyway,
which can take the ledger over its ceiling. That errs toward counting a
possible charge, never toward lending it twice.

> ⚠️ **Nothing releases a stamped hold yet.** An operator's adjudication should
> release it (`failed_known`) or charge it (`completed`). That is
> `SIGNOFF-REPAIR.4.5.1.1`. Until then it stays counted, which is the safe
> direction. A node that never reports again gives the server nothing to act on,
> so its hold lapses at the end of its window, as before.

## Admission is serialized

Each admission check sums what is already committed and then records its own
row: the ceiling's held amount, the breaker's tenant-wide spend, and a quota's
uses in its window. Until `SIGNOFF-REPAIR.4.5.3` none of them locked anything.
Command paths hold the tenant's authority guard in shared mode, so two
admissions could interleave. Each would sum the rows committed so far, see room
for one, and both would record it: the ceiling, the breaker or the quota
overshot by one admission per race.

Each check now locks the row it decides against before it sums:

| Check | Lock |
| --- | --- |
| Ceiling (`create_reservation_in_tx`) | `budget_ceilings … FOR UPDATE` |
| Spend breaker (`check_spend_breaker_in_tx`) | `spend_breakers … FOR UPDATE`, after the ceiling |
| Quota (`quota::check_in_tx`) | `usage_quotas … FOR UPDATE` |

A second admission waits on the lock. Under READ COMMITTED its later sum sees
what the first committed, so it decides on the first's hold or use. The breaker
lock is the one that matters across ceilings: two admissions against different
threads of one tenant share no ceiling, but the breaker sums both. Every
admission takes the ceiling first and the breaker second. The breaker's arm and
reset verbs run under the tenant's exclusive guard, which shared-guard
admissions already exclude.

The live controls watch the second admission wait in `pg_stat_activity`
rather than sleeping and hoping, then commit the first, and require the second
to refuse on it.

## Settle, release, overrun

A node's result settles the reservation the **server** stored on the work
item's inbox row. The `reservation_id` a node puts in its result is never read,
and a live control proves it: a result naming another tenant's reservation
leaves that hold untouched (`SIGNOFF-REPAIR.4.5.4`).

Settlement records **actual** usage: lower than the reservation frees the
difference; higher is an **overrun reported in full** (estimate errors feed
routing, §14.3 — never clamped). Release returns the unused hold. Expired
reservations stop holding. An **indeterminate** attempt keeps its hold
(§14.6: release only amounts not potentially consumed).

### A usage too large to count

Settled usage is what a node *reported*, recorded in full, so the ledger can
hold numbers no honest provider produces. Until `SIGNOFF-REPAIR.4.5.2`, its sums
used plain `u64` addition, and the workspace declares no `[profile.release]`,
so a release build does not check for overflow. Two reports of 2⁶³ tokens
panicked a debug server's admission. In a release build they would have wrapped
to a small held sum, and the ceiling would have lent again. That last part
follows from Rust's release default; it was not run.

`BudgetDimensions::add` is now fallible, like `subtract` beside it. A sum past
`u64` is a typed `BudgetError::Overflow` naming the dimension; it never wraps
and never saturates. The unchecked form no longer exists. Every reader fails
closed:

| Reader | Answer on overflow |
| --- | --- |
| Admission's held sum | `Unavailable`: *the ceiling's held sum cannot be counted* |
| The breaker's spend, and spend plus the request | `Unavailable`: *the tenant's recorded spend cannot be counted* |
| `GET /v1/admin/usage` | `500 ledger_overflow`, naming the dimension |
| The node's `LocalBudget` | reservation refused; a settlement leaves the headroom spent beyond counting, so a later settlement cannot reopen it |

Settlement itself does not reject a large report. The work happened, and
§14.3 records overruns without clamping. A usage too large to count now stops
the ceiling from lending, which is the safe reading of it.

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

A breaker constrains **only the dimensions its threshold names**. Armed at
`{"calls": 100}`, it trips at the hundred-and-first call, whatever tokens those
calls use. Until `SIGNOFF-REPAIR.4.5.6` it compared with `covers`, which a
ceiling uses and which refuses any dimension it does not meter, so a partial
breaker tripped on the first work item, since every work item also asks for
tokens and time. It now compares the projection restricted to the threshold's
dimensions (`BudgetDimensions::restricted_to`). ⚠️ The arm verb accepts a
threshold that names **nothing**, and such a breaker can never trip; refusing
it is `SIGNOFF-REPAIR.4.5.6.1`.

## Honest limits (Phase 0)

- Reservation references are unsigned (dev profile); workload-identity
  signatures arrive with WP7.
- The node's local ledger is in-memory; the server's ceiling is the durable
  counterpart.
- `.5.1`'s grant evaluation precedes this engine: a reservation requires an
  applicable grant, and a dispatch requires both.
