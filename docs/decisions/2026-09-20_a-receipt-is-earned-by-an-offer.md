---
answers:
  - What does the cursor acknowledgement mark, now that the channel records offers?
  - Could a node make the server receipt work it never received?
  - What happens to inbox rows enqueued before migrations/0078?
  - Why does a revoked row that earned a receipt still read `revoked`?
---
# A receipt is earned by an offer

- **Type:** decision
- **Status:** accepted; implemented
- **Owner:** `SIGNOFF-REPAIR.11.24.1.1.1.1`
- **Date:** 2026-09-20
- **Raised by:** `SIGNOFF-REPAIR.11.24.1.1.1`, which gave the channel the fact
  this needed
- **Related:** `docs/decisions/2026-09-20_the-tail-read-is-the-offer.md`,
  `docs/decisions/2026-09-20_an-acknowledgement-covers-a-range-and-the-tail-does-not.md`,
  `docs/decisions/2026-09-20_a-command-cannot-outlive-its-authority.md`

## The proxy, and both directions it was wrong in

`SIGNOFF-REPAIR.11.24.1.1.2.1` stopped the cursor acknowledgement writing a
transport receipt for rows the tail withheld, by excluding rows withheld **for
a reason about that row** — quarantined, or past their authority. That is a
proxy for *was this row carried*, evaluated at **ack time**. `migrations/0078`
records the fact it was standing in for, and `offered_at IS NOT NULL` replaces
both clauses.

🔴 **It under-recorded — the residual that leaf named itself.** A row offered
and then quarantined was excluded although the node demonstrably held it, so
the receipt was suppressed and the row re-delivered on the next replay.

🔴 **It over-recorded, which is the direction that destroys work, and that was
not noticed at the time.** A cursor acknowledgement is a wire input bounded
only by the server's `current_cursor`, so a node may acknowledge **past** rows
this tail never offered it — one enqueued after its last poll, for instance.
Those rows are neither quarantined nor authority-ended, so the proxy admitted
them; and `acknowledged_at` is the retention prune's `DELETE` predicate.
**Undelivered work became eligible for deletion on a number the node supplied.**
That is `SIGNOFF-REPAIR.4.2.4`'s harm model reached by a second route — there a
*fenced* session could mark another session's delivery, here any session can
mark a delivery that never happened.

## The pre-`0078` cohort, answered rather than omitted

⛔ Rows enqueued before the migration carry `offered_at IS NULL` whatever their
history, and that NULL means *not offered since the migration* — never *never
offered*. An unscoped fallback to the old proxy was considered and **refused**:
nothing in the schema distinguishes a legacy row from one simply not yet
offered, so the fallback would re-admit the over-recording hole for every row.

✅ **It costs nothing, and the reason is the tail's own shape rather than an
estimate.** A row with no acknowledgement is still in the tail, so the node's
next poll offers it, records the offer, and the next acknowledgement receipts
it. The only pre-`0078` rows that never regain a receipt are ones the tail
withholds — which the superseded clauses excluded too — or ones never carried,
which should never have been marked.

## The per-node reasons

⚠️ `replay` also withholds for two facts about the **node** rather than the
row: no usable certificate (`.4.1.3.1`), and a profile declaring zero declared
concurrency. `.11.24.1.1.2.1` kept those out of the exclusion deliberately,
because they describe the node's standing now and excluding them would suppress
true receipts. Keying on a recorded offer is exactly the distinction that clause
was reaching for, and it makes it by construction: a row carried **before** the
node lost its certificate keeps its receipt, and a row never carried does not
gain one.

## One interaction, stated rather than left to be discovered

⚠️ A row offered, then revoked, then acknowledged now earns a true
`acknowledged_at`. Its `delivery_state` still reads `revoked`, because
`migrations/0076` ranks the **act** above the receipt deliberately — it is the
more informative answer to an operator asking why a command was never completed
— while the retention prune now ages it as a **delivered** row, by the receipt,
rather than by the revocation instant. Both are right for the question each
answers, and a control pins the pair so the agreement cannot drift silently.

## A control that asserted the opposite of what it had just watched happen

🔴 `a_revoked_grant_makes_its_undelivered_command_revoked_and_withholds_it`
asserted `acknowledged_at IS NULL` with the comment *it never received this
one* — and its own positive arm had handed the row to the node one step
earlier, because that arm exists to prove the tail was capable of carrying it.
The assertion passed under the proxy for the **wrong reason**: the row was
excluded for being authority-ended at ack time, not for never having been
carried. It now asserts the receipt **together with the offer that earns it**,
ordered, so the pair cannot come apart — a receipt without a recorded offer is
the defect, and a receipt with one is the repair.

## Verification

`node_inbox` 11, `node_work` 12, `node_channel` 40, `node_result_ordering` 6,
`mcp_listen` 6 — **75 passed, 0 failed**.

⭐ **Falsified in two directions, and the asymmetry is the point.** Restoring
the superseded proxy turns `node_inbox`'s new control RED (the node acking past
an unoffered row) *and* `node_work`'s corrected assertion RED (the receipt the
offered row now earns). Removing the offer clause entirely turns `node_inbox`
RED — over-marking — and leaves `node_work` **GREEN**, because that assertion is
positive and cannot be broken by marking more. Neither suite covers both
directions alone; together they close it. Each mutation restored byte-identical,
verified by SHA-256.
