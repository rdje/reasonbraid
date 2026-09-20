---
answers:
  - What counts as one record in DEV_NOTES.md, and who decided?
  - Why can the changelog rotation not be pointed at a second ledger unchanged?
  - Does widening the rotation's heading parser change how CHANGELOG.md rotates?
  - What is DEV_NOTES.md's lifecycle, and what is its archive terminal?
  - Who reads DEV_NOTES.md, and would rotating it break a consumer?
  - Why is no rotation threshold set in this record?
---
# The second ledger's record boundary is quoted from the gate that already governs it, not chosen

- **Type:** decision
- **Status:** accepted
- **Owner:** `SIGNOFF-REPAIR.11.4.2.6.1`
- **Date:** 2026-09-21
- **Raised by:** `SIGNOFF-REPAIR.11.4.2.5`, which measured `DEV_NOTES.md` at 894,723 bytes with no size bound and no routed-destination row
- **Related:** `docs/decisions/2026-09-09_changelog-rotation.md`

## The question

`DEV_NOTES.md` is an append-only ledger — measured over its whole history, **427
of 427 version transitions grew it and not one byte has ever been removed**. The
repository already contains a working, calibrated rotation for an ordered ledger,
running on `CHANGELOG.md`. Before pointing it at a second file: **what is one
record on that file, and who gets to say?**

## The answer

> **The record boundary is not a choice available to the rotation. It is already
> defined, by the gate that already governs the file, and the rotation adopts
> that definition.**

`scripts/check_lesson_promotion.sh` has governed `DEV_NOTES.md` since it was
ported, and its rule names a record as an `## …` line carrying a `YYYY-MM-DD`
date — `^## .*[0-9]{4}-[0-9]{2}-[0-9]{2}`. Its own self-test pins **both**
spellings the file actually uses, `## 2026-09-04 — …` and `## _(2026-09-04)_ — …`.
That is this repository's existing, enforced definition of a `DEV_NOTES.md`
record, and a second definition invented for the rotation would be a second copy
of one fact — the defect class `SCAFFOLD-COVERAGE` and `INDEX-FRONTIER` both
exist to stop.

## The measurement that makes this load-bearing

`scripts/rotate_changelog.py` uses a **narrower** parser:
`HEADING = ^## \d{4}-\d{2}-\d{2}`. Pointed at `DEV_NOTES.md` unchanged, the two
parsers disagree about the file's contents:

| Parser | Source | Records seen | Bytes in records |
| --- | --- | --- | --- |
| narrow | `scripts/rotate_changelog.py` | 291 | 717,211 |
| wide | `scripts/check_lesson_promotion.sh` | **439** | **895,767** |

⛔ **148 records and 178,556 bytes are invisible to the narrow parser.** A
rotation splits a ledger at record boundaries and writes a notice stating how
many records it retired; run on a parser that cannot see a third of them, it
would retire content it did not count and publish a false figure in its own
chain notice. That is precisely the failure
`docs/decisions/2026-09-09_changelog-rotation.md` requires losslessness to
prevent.

⭐ **Exactly two `## ` headings match neither parser, and both are correctly not
records**: `## clause-1 [org-baseline 1.0.0]` and the template placeholder
`## _(YYYY-MM-DD)_ — bootstrap`, which carries no real date. The wide parser is
not merely wider — on this file it is exact.

## Adopting the wide parser does not change how `CHANGELOG.md` rotates

Proved over that ledger's entire history rather than asserted: across **all 664
versions of `CHANGELOG.md`, the two parsers return the same heading count in
every single one — 0 disagreements.** The narrow parser has never been wrong
there, and widening it is therefore behaviour-preserving for the ledger already
in production, which is the only way a build with no red can be made safe.

## Lifecycle and consumers

`DEV_NOTES.md` is a `rolling_ledger`; its archive terminal is **git history**,
the same terminal `CHANGELOG.md` uses, reached by `git show <commit>:DEV_NOTES.md`.

The consumer census, taken before any history is moved — every tracked reference
outside the file itself:

- `COMMIT.md` — the per-commit update mandate.
- `DOCTRINE_ENFORCEMENT.md` — the `LESSON-PROMOTION` registry row and the
  `LOCKSTEP` checklist item.
- `scripts/check_lesson_promotion.sh`, `scripts/check_lockstep_claim.sh` — the
  two gates.
- `.doctrine/readme_routes.txt` — the debt row added by `.11.4.2.5`.

⭐ **Not one consumer cites an individual record.** There are no deep links into
`DEV_NOTES.md` from code, tests, the book, the runbooks or the task trees: every
consumer is a rule about the file as a whole. And the durable content has a
separate retrievable home already — `LESSON-PROMOTION` exists to force each
lesson into `docs/knowledge/` or an `answers:` record. Retiring old records to
git therefore removes no reader's access path.

## What is deliberately NOT decided here

⛔ **No threshold, ceiling or retirement amount is set by this record.** A
ceiling chosen in the slice that migrates under it is fitted to current bloat.
The record-size distribution is published as an input for the leaf that derives
it — median **2,073** bytes, p90 **3,199**, max **6,802**, over 439 records —
and the derivation itself is `SIGNOFF-REPAIR.11.4.2.6.2`'s, using the runway
formula `LEDGER-RUNWAY` already calibrates for `CHANGELOG.md`.
