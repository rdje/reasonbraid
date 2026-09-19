# A retention window measures time in the state it retains

- Date: 2026-09-20
- Status: accepted
- Leaf: `SIGNOFF-REPAIR.11.24.1.1.2.1.1`
- Follows: `2026-09-20_an-acknowledgement-covers-a-range-and-the-tail-does-not.md`,
  which created the gap this record closes, and
  `2026-09-20_a-command-cannot-outlive-its-authority.md`, which created the states.

## The gap

`POST /v1/nodes/inbox/prune` deletes on `acknowledged_at IS NOT NULL AND
acknowledged_at <= $3 AND quarantined_at IS NULL`. A row that reached §10.6's
`expired` or `revoked` was **never delivered**, so it carries no
`acknowledged_at` — and once `.11.24.1.1.2.1` stopped the cursor acknowledgement
writing a receipt it had not earned, no operator verb could remove such a row at
all.

That matters because those rows are exactly the ones a broken deployment
produces: commands queued for a node that never came back. Solving *held forever*
for delivery and recreating it for storage is not a repair.

## The rule this decision rests on

**A retention window measures time in the state being retained.** An operator who
asks to delete rows older than seven days is making a statement about how long
they want to be able to see a finished row, not about when that row happened to
be created. Ageing by anything else silently deletes rows they expected to still
be there.

Applied to the two terminals, that rule does not give the same answer twice, and
the difference is the whole decision:

- **`expired` has an exact timestamp.** The row entered the terminal at the
  admitting grant's `expires_at`. The window is `g.expires_at <= cutoff`, and it
  measures precisely time-in-terminal.
- **`revoked` has none.** `authority/revocation.rs` writes `SET status =
  'revoked'` and `authority_grants` carries no `revoked_at`. `node_inbox.decided_at`
  exists and would age the row, but it records when the row was *created*, which
  is a different quantity: a command queued thirty days ago under a grant revoked
  this morning would vanish under a seven-day window on the day it entered the
  terminal.

So `expired` becomes prunable and `revoked` does not. The trigger for revisiting
is a `revoked_at` column on `authority_grants`, at
`SIGNOFF-REPAIR.11.24.1.1.2.1.1.1`.

## What the operator is told

`deleted` is now the **total**, with `deleted_delivered` and `deleted_expired`
beside it.

⚠️ That widens `deleted`'s meaning. It used to be reachable only by delivered
rows, so a client reading it as *delivered rows removed* was accidentally right.
The breakdown is what keeps the receipt true, and the CLI summary moved with it —
`pruned N delivered row(s)` became `pruned N row(s) … M delivered, K expired
undelivered`, because *delivered* is a false word for work that was never handed
over. Pre-1.0, development profile.

## The oracles, and where they pushed

§12.9 says deletion *creates a tombstone and reason, not silent disappearance*,
and §16.11 preserves evidence through quarantine. Both lean towards retention,
and they are why the answer is not simply "sweep the terminals".

The prune is not silent: it is an explicit, authorized operator act that takes the
tenant's guard, writes an `administrative_effects` record with its reason, and
returns a measured before/after. §16.11's preservation rule is untouched — a
quarantined row is still never deleted. What this decision adds is a window that
an operator chooses, over a class whose entry instant is exactly known.

## Evidence

- `node_work` **12 passed, 0 failed** (10 at `71dccb0`, measured on both sides);
  `node_inbox` 9/9, `node_channel` 40/40, `quarantine` 1/1, `mcp_listen` 6/6,
  `cli_end_to_end` 5/5, `administrative_effects` 25/25.
- The window arm runs **before** the deletion arm: a grant that expired two hours
  ago is *not* removed by a one-day window, and the row is still there. Without
  that arm a passing deletion would prove only that the row can be destroyed.
- 🔴 **The first falsification found a real gap rather than confirming the
  design.** Removing `AND g.status = 'active'` left the revoked control green,
  because revocation does not move `expires_at` — so a merely-revoked grant was
  excluded by the window alone and the clause was dead. The control now ages the
  revoked grant past its expiry as well, which is the case where the two differ:
  the view calls that row `revoked` (the act outranks the lapse) and the prune
  must agree with the view rather than delete it as an expiry. With that arm the
  same deletion reproduces as `{"deleted":1,"deleted_expired":1,"after":0}`.
- Falsified in two places, each alone, restored byte-identical (`sha256
  14413608…` matched): `g.expires_at <= $3` → `<= now()` fails the window arm;
  dropping `AND g.status = 'active'` fails the revoked arm.
