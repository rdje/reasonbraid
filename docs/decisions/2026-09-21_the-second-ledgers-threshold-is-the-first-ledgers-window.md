---
answers:
  - Where does DEV_NOTES.md's 76,000-byte threshold come from?
  - Why is a rotation threshold a fixed number instead of derived on read?
  - A proposed rule fires on 90% of history — is it a quarantine or a debt?
  - Why is DEV_NOTES.md not in the LEDGER-RUNWAY gate yet?
  - Why does the rotation tool refuse to perform a first rotation?
---
# The second ledger's threshold is the first ledger's live window, in the second ledger's units

- **Type:** decision
- **Status:** accepted
- **Owner:** `SIGNOFF-REPAIR.11.4.2.6.2`
- **Date:** 2026-09-21
- **Related:** `docs/decisions/2026-09-21_the-second-ledgers-record-boundary-is-quoted-not-chosen.md`,
  `docs/decisions/2026-09-09_changelog-rotation.md`

## The question

`DEV_NOTES.md` needs a rotation threshold. Every obvious way of picking one is
forbidden: the adoption guide rules out copying a donor's numbers, rules out
fitting a ceiling to current bloat, and rules out calling a quarantine ceiling
healthy. So what is the number, and what authority stands behind it?

## The answer

> **The threshold is the live window the ledger already in production runs on,
> expressed in the second ledger's own measured entry size.** The authority is an
> existing reviewed decision, not a fresh preference.

Measured at `7fc8913` with `scripts/rotate_changelog.py`'s own `entry_size_p90`,
so both figures come from one producer in matched units — bytes added per
non-rotation commit that touched the file:

| Ledger | p90 entry | threshold | live window |
| --- | --- | --- | --- |
| `CHANGELOG.md` | 4,734 B | 96,000 B | **20.279** p90-entries |
| `DEV_NOTES.md` | 3,763 B | **76,000 B** | 20.279 p90-entries |

`20.279 × 3,763 = 76,309`, **rounded DOWN to 76,000**. Rounding up would grant
headroom the derivation does not support, and a ceiling may only ever move that
way under an explicit reviewed decision.

⚠️ **The number is derived ONCE, at a named commit, and then held fixed.** It is
deliberately not re-derived on read: p90 grows as entries grow, so a ceiling
recomputed at run time would let the ledger widen its own bound by growing —
which is the failure the ceiling exists to prevent, wearing the clothes of a
derivation.

## The calibration, and why 90% is a debt rather than a quarantine

Over all **430** versions of `DEV_NOTES.md`, the runway rule at this threshold
would fire on **385 — 90%**. That number alone would condemn it: this repository
has already rejected a gate for firing on 114 of 131 (`SIGNOFF-REPAIR.11.9`).

⭐ **The shape is what distinguishes them, and it was measured rather than
argued.** The first **45** versions are clean. Version 46 (`b643c24`, 73,261
bytes) crosses, and **every one of the 385 versions after it fires, with no
recovery at any point**. A rule that fires on a scattered 90% is describing the
rule; a rule that fires on a contiguous tail after a single crossing is
describing the file — a ledger that went over its bound once and was never
rotated, which is exactly what `SIGNOFF-REPAIR.11.4.2.5` measured when it found
427 of 427 versions growing and zero bytes ever removed.

✅ **The discriminating evidence is that the remedy clears it.**
`--ledger dev-notes --plan` retires 427 records and leaves **36,774 bytes —
39,226 bytes of headroom, 10 commits of runway — and the rule does not fire.**
`SIGNOFF-REPAIR.11.9`'s rejected gate had no such remedy. Here it is one command.

## Why the ledger is not in the gate yet

`Ledger.enforced` is `False` for `DEV_NOTES.md`, and the field is declared rather
than omitted. Admitting a ledger to the gate before its first rotation would make
the enforcer red on every commit until the migration lands, and a gate that is
always red is one people route around (`SIGNOFF-REPAIR.11.5`) — not a bound.
`--check-all` judges every ledger regardless, so the debt is measurable on demand
without being in the commit path. Registration follows the rotation.

## Why the tool refuses to perform the first rotation

`scripts/rotate_changelog.py` continues a chain; it cannot start one. Each
rotation notice takes its ordinal from its predecessor's, so every notice names
the one before it back to the first transition's recorded evidence. A ledger that
has never been rotated has no predecessor notice, and inventing an ordinal would
break the one property the chain exists for. The tool now says so and exits 1.

⛔ It used to raise an uncaught `ValueError: substring not found` from
`split_ledger`, because a missing rotation footer was treated as a malformed file
rather than as the normal state of a ledger before its first rotation. Fixed
here: an absent footer is the end of the text, so `--plan` answers correctly for
a first rotation and `--apply` reaches its own refusal instead of a traceback.
