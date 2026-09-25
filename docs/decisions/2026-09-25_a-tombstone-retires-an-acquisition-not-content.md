---
answers:
  - What does a tombstone on an evidence snapshot refuse, and what does it leave alone?
  - What happens when the same content is acquired again after its snapshot was tombstoned?
  - Why is a re-acquisition of tombstoned content a new row rather than a refusal?
  - Where would a block on content itself belong?
---
# A tombstone retires an acquisition, not content

- **Type:** decision
- **Status:** active
- **Owner:** `SIGNOFF-REPAIR.7.4.6`
- **Date:** 2026-09-25
- **Work unit:** `REASONBRAID-REPAIR-0515`
- **Cites:** ROADMAP §12.6, §12.9; `docs/book/src/deployment.md` (*What a tombstoned snapshot refuses*)

## The fact / decision

A tombstone (`deleted_at` + `deletion_reason` on an `evidence_snapshots` row) is
written by two site acts: the retention sweep and the operator's named
tombstone. The book defines it as *"this evidence must not be relied upon by
anyone"*. It retires that ROW, one acquisition:

1. **Nothing new rests on it.** A derivation naming it as parent and an
   assessment citing it (the standalone route and the deliberation's `assess`
   step, which share `claims::submit`) are refused by name, with the reason.
   Re-filing a derivation made while the row was live is refused too: a replay
   is a caller relying on it again.
2. **A re-acquisition of the same content is a NEW row**, with its own
   provenance and retention clock. Both replay lookups read live rows only, and
   the external class's unique identity index is partial on live rows
   (`migrations/0112`).
3. **Reads are unchanged**: every citer still reads the row, its tombstone and
   its reason (§12.9, *never a silent disappearance*).

## Why

- **The retention sweep is the common writer.** Refusing a re-acquisition would
  turn a one-day or thirty-day expiry into a permanent ban on unchanged content,
  and nothing clears `deleted_at`, so the ban could never be lifted.
- **The other shape re-cited retired evidence.** Before the repair the replay
  lookup matched the tombstoned row and answered `"replay": true` with its id,
  so a caller was told its fresh acquisition was the retired one.
- **Blocking CONTENT is a different feature.** §12.6 gives a snapshot a
  quarantine status for that, and its gate does not exist (`SIGNOFF-REPAIR.7.4.10`,
  deferred; the qualification chapter states it). Making the tombstone carry it
  would have meant choosing, per tombstone, between "this acquisition is
  retired" and "these bytes are banned", with no field saying which.

## How to apply

- A writer that relies on a snapshot checks `deleted_at IS NULL` on the row it
  relies on, and names the tombstone when it refuses.
- A lookup that answers "is this acquisition already recorded?" reads live rows
  only, and any unique index behind it is partial on `deleted_at IS NULL`.
