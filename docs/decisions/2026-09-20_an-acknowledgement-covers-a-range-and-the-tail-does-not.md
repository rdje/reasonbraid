# An acknowledgement covers a range and the tail does not

- Date: 2026-09-20
- Status: accepted
- Leaf: `SIGNOFF-REPAIR.11.24.1.1.2.1`
- Extends: `2026-09-19_a-transport-receipt-is-not-an-acknowledgement.md`, which
  established what `acknowledged_at` means. This record repairs a place where the
  column recorded that meaning falsely.

## The defect

`acknowledge` marks every row up to the acknowledged cursor:

```sql
UPDATE node_inbox SET acknowledged_at = $3
WHERE node_id = $1 AND cursor <= $2 AND acknowledged_at IS NULL
```

`replay` — the one function the handshake and the poll both read the tail
through — withholds rows the node must not receive. The two predicates disagree,
so **a withheld row gains a transport receipt the moment the node acknowledges a
later row it really did receive**, and `migrations/0075` defines that column as
*the node process durably holds this command*.

Reproduced through the public surface before anything was changed: quarantine row
2 of three, let the tail hand the node rows 1 and 3, acknowledge cursor 3 — row 2
carries an `acknowledged_at` for a command the transport never carried.

## Why it is not a cosmetic column

`a_fenced_acknowledgement_cannot_mark_another_sessions_delivery`
(`SIGNOFF-REPAIR.4.2.4`) already recorded the harm model for this exact shape:
`acknowledged_at` is the retention prune's `DELETE` predicate, so a false receipt
*makes work that was never delivered eligible for deletion*.

The quarantine case is shielded by that predicate's own `quarantined_at IS NULL`
— and `POST /v1/nodes/replay` clears the quarantine while leaving the false
receipt behind, which takes the shield with it. The replayed command then reads
`transport_received` without ever having been delivered, and is prunable. That
chain is the second half of the control.

## The decision

The acknowledgement excludes **the rows the tail withheld for a reason about that
row** — a quarantine, and authority that has ended (`migrations/0076`). Both
call sites bind one statement, `ACKNOWLEDGE_SQL`, so they cannot drift.

**The two per-node reasons are deliberately not part of the exclusion.** `replay`
also withholds when the node holds no usable certificate (`.4.1.3.1`) and when
its profile declares zero concurrency. Those describe the node's standing *now*,
not whether any row was carried; including them would suppress **true** receipts
for rows the node demonstrably holds, because a node can acknowledge during the
lease tail after its certificate has lapsed.

## The residual error, and why it is the safe one

Without an `offered` column — still owed at `SIGNOFF-REPAIR.11.24.1.1.1` — the
server cannot distinguish *withheld now* from *withheld when it was offered*. A
row offered before its quarantine and acknowledged after it therefore goes
unmarked.

That under-records a receipt. The row is re-delivered on the next replay and the
node's journal deduplicates it by command id, which is what `migrations/0003`
states the channel relies on. The behaviour this replaces over-records one, and
over-recording destroys work. **Failing to record a receipt costs a redelivery;
recording one that never happened costs the command.**

## What the `.1.2.3` sentence actually was

The leaf opened expecting to argue against a recorded decision: `replay`'s doc
comment says *a node that never saw it simply has a hole in its ledger — cursor
acknowledgement still marks it terminal*, attributed to `.1.2.3`.

Measured rather than assumed: the sentence entered with that leaf's commit
(`0e47b27`) **in the code comment only**. `PHASE-1.2.3`'s goal and acceptance say
*a quarantined command is never re-delivered* and nothing about acknowledgement,
and no task-tree record carries the claim. So it is a comment describing what the
code did, not a decision weighed and taken — and the leaf's own premise that
*the prune depends on it* is wrong in the other direction: the prune excludes
quarantined rows outright, so marking them bought it nothing.

## The prune's eligibility does not move, and that is stated rather than skipped

The leaf's acceptance said that if the column stops being written, the prune's
rule moves with it. It does not, for a reason worth recording:

- **Quarantined rows** were never prunable (`quarantined_at IS NULL` in the
  predicate, the §16.11 preservation rule). Nothing changes.
- **Authority-terminal rows** are now never prunable. They were prunable before
  only *via the false receipt this repair removes*, so the reach has not shrunk
  — it was never legitimate.

Those rows therefore accumulate, and the obstacle is not a missing timestamp:
`node_inbox.decided_at` exists and would age them. It is that
`POST /v1/nodes/inbox/prune` is documented and controlled as deleting **delivered
rows**, and teaching it to delete never-delivered ones changes what an operator's
receipt means. That needs its own argument and its own control:
`SIGNOFF-REPAIR.11.24.1.1.2.1.1`.

## Evidence

- `node_inbox` **9 passed, 0 failed** (8 at `60affee`, measured on both sides);
  `node_work` **10 passed, 0 failed**, its revoked control extended with the
  column assertion the state assertion could not reach.
- Neighbours unchanged: `node_channel` 40/40, `quarantine` 1/1,
  `node_replacement` 2/2, `mcp_listen` 6/6, `node_result_ordering` 6/6.
- **Reproduced as RED before the repair**, which is the order this project
  requires: the new control failed on the unrepaired tree with *a row the tail
  withheld carries NO transport receipt*, `8 passed; 1 failed`.
- **Falsified twice after it, each clause alone, restored byte-identical**
  (`sha256` matched): dropping `AND quarantined_at IS NULL` fails the quarantine
  control; dropping the authority clause fails the `node_work` control with
  `Some(2026-09-19T22:09:17Z)` where `None` was required.
- No PRODUCTION code consumes the reported count: `git grep -n "\.acknowledged\b"`
  over `crates/*/src` returns no hit. 🔴 **CORRECTED by `SIGNOFF-REPAIR.13.4.3`:
  this line first said "over the node and server sources returns no hit" and used
  it to conclude the narrowing changes no caller.** Over `crates` — tests included
  — the same command returns **2** hits, and one of them ASSERTS the value
  (`crates/reasonbraid-server/tests/node_channel.rs:4434`, `marked.acknowledged == 3`). That control passes,
  because its rows are neither quarantined nor authority-ended and 3 is still 3 —
  so the conclusion holds. But the evidence was scoped to exclude the one place
  that could have refuted it, and the sentence did not say so.
