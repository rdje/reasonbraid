---
answers:
  - Where does §10.6's `offered` state come from?
  - Why is `offered_at` written once instead of bumped on every re-offer?
  - Why is there no offer count?
  - Why does `acknowledged` stay underived?
  - Why must a migration that adds a `node_inbox` column drop and recreate the view?
---
# The tail read is the offer

- **Type:** decision
- **Status:** accepted; implemented
- **Owner:** `SIGNOFF-REPAIR.11.24.1.1.1`
- **Date:** 2026-09-20
- **Raised by:** `SIGNOFF-REPAIR.11.24.1.1`'s disposition table, which left two
  of §10.6's rungs underived and named each one's obstacle
- **Related:** `docs/decisions/2026-09-19_a-transport-receipt-is-not-an-acknowledgement.md`,
  `docs/decisions/2026-09-20_a-command-cannot-outlive-its-authority.md`,
  `docs/decisions/2026-09-20_a-retention-window-measures-time-in-the-state-it-retains.md`

## §10.6, quoted

> ```text
> queued → offered → transport_received → acknowledged → consumed
>                   ↘ expired / revoked / dead_lettered
> ```
>
> Transport receipt does not mean an agent read or acted. Acknowledgement
> semantics are explicit per event type.

`migrations/0076` derived every rung of that ladder except `offered` and
`acknowledged`. This record decides both, independently, as the leaf required.

## `offered` — taken

🔴 **The gap.** Without the rung, a row the server had handed to a transport
read `queued` — the same answer as a row it had never tried to deliver. Those
are two different faults: rows stuck at `queued` say the node is not polling,
and rows reaching `offered` and stopping say it is polling and the responses
are not arriving. The surface an operator reads could not tell them apart.

✅ **The producer is the tail read itself**, because that is where the fact
exists. `NodeChannelState::replay` is the one function the handshake and the
poll both read the tail through, and the rows it returns are the rows a
response carries. The mark rides the **same statement** as the read, as a
data-modifying CTE: two statements would be two snapshots, and the pair is not
in a transaction, so a row could enter the tail between them and be delivered
unmarked.

⛔ **A withheld row is not marked.** The mark is scoped to the rows the tail
actually returns, so quarantine and an ended authority exclude a row from the
offer exactly as they exclude it from the response. A statement that marked the
whole inbox would be indistinguishable from this one on a node with nothing
withheld, which is why the control keeps a withheld row beside the live one.

⚠️ **What the mark can and cannot mean.** The server knows the row went into a
response. It cannot know the bytes arrived — that is what `transport_received`
is for. So a response lost in flight leaves the row at `offered`, which is the
state's whole purpose, and a response the caller fails to send after `replay`
returns overstates by one rung. Both residuals are the safe direction: the
cursor did not move, the row is offered again, and the node's journal
deduplicates by command id.

### Write-once, and that is derived

`offered_at` records when the row **entered** the state, not when it was last
re-offered. `SIGNOFF-REPAIR.11.24.1.1.2.1.1` established that a window measures
time in the state being retained; an instant a re-offer bumps silently resets
every age measured from it, so a node polling in a loop would keep its oldest
outstanding work looking new. The `offered_at IS NULL` guard in the writing
statement is what makes it write-once, and a control drives a second poll and
asserts the instant does not move.

### No count, and the trigger that would add one

⛔ The leaf asked for "`offered_at`, or a count". The count is **not** added.
The question the state exists to answer — a quiet node versus one losing its
responses — is answered by the *state*, and **a column is not missing until
something needs it** is this tree's own promoted rule. ⚠️ Trigger: a reader
that must distinguish one offer from many; §10.7's storm controls are the
likely one.

### No backfill, and that refusal is derived twice

⛔ A row already at `transport_received` or above outranks `offered` in the
view, so dating it would change nothing observable — while the only instant
available (`acknowledged_at`) is provably later than the real offer. A row
still at `queued` is exactly where the answer would matter, and nothing in the
schema records whether it was ever offered. So the backfill is unnecessary
where it is derivable and impossible where it would be informative. A NULL
`offered_at` means *not offered since this migration*, never *never offered* —
and a migration control drives the absence, because a refusal nothing exercises
is indistinguishable from an oversight.

## `acknowledged` — deferred, with a measured trigger

⛔ §10.6 defines it as the state whose *acknowledgement semantics are explicit
per event type*, and the channel has one cursor ack covering every row up to a
cursor whatever their types. The obstacle is not an implementation gap: it is
that **the per-event-type contract does not exist**, and inventing one to fill
a ladder slot is a design commitment made to satisfy a table.

⭐ **The deferral is measured, not assumed.** The dispatch has exactly two
kinds today — `threads::WORK_CONTRIBUTE` (`contribute`) and
`threads::WORK_REVISE` (`revise`) — and confirmation means the same thing for
both: the node holds the command. There is nothing for a per-type contract to
distinguish.

⚠️ **The trigger a person can evaluate:** a command kind on the node channel
whose acknowledgement means something different from a work item's. At that
point the contract has content, and the rung can be derived from it rather than
declared.

## The view-replacement hazard this uncovered

🔴 `node_inbox_state` is `SELECT i.*, … AS delivery_state`. The `*` expanded
positionally when the view was created, so `delivery_state` is its last column.
`ALTER TABLE node_inbox ADD COLUMN` inserts the new column into that expansion
ahead of it, and `CREATE OR REPLACE VIEW` may only **append** columns —
PostgreSQL refuses with `42P16 cannot change name of view column
"delivery_state" to "offered_at"`, and every suite that applies migrations fails
at once. `0078` therefore drops and recreates the view.

⚠️ **Standing hazard, recorded rather than gated.** While that view selects
`i.*`, every future `ALTER TABLE node_inbox ADD COLUMN` must drop and recreate
it. Measured population rather than a guess: the schema has two views, and this
is the only one that expands a star (`node_presence` names its columns). One
instance is not a rule, so no gate is proposed; the trigger for one is a second
table gaining a star-expanding view.

⭐ Worth stating beside `SIGNOFF-REPAIR.13.4.3.1`, two commits earlier: that
leaf's point was that *applying* a migration proves only that its SQL runs. This
one is the other half — applying it caught this instantly, because the error is
in the SQL's validity rather than in its meaning. Application catches syntax and
dependency; only a driver catches semantics.

## Verification

`node_inbox` 10, `node_work` 12, `node_channel` 40, `migration_upgrade` 7,
`mcp_listen` 6 — 0 failed. Falsified four ways, each file restored
byte-identical: removing the mark, removing its write-once guard, unscoping it
from the tail, and removing the rung from the view each turn the control RED.
