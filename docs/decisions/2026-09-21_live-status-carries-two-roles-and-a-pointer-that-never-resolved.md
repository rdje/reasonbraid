---
answers:
  - Why is LIVE_STATUS.md's current-status table buried 92% of the way down the file?
  - What lifecycle does LIVE_STATUS.md get, and why is it not the ledger rotation?
  - Can the DEV_NOTES record boundary be reused for LIVE_STATUS.md?
  - Is the LIVE_STATUS.md:1719 citation in the G4/G5 record still valid?
  - Why does POSITIONAL-REF report unresolved=0 while a stale citation exists?
  - What may I cite by line number, and what may I not?
---
# `LIVE_STATUS.md` carries two information roles, and a pointer into it never resolved

- **Type:** decision
- **Status:** accepted
- **Owner:** `SIGNOFF-REPAIR.11.4.2.6.5`
- **Date:** 2026-09-21
- **Corrects (does not mutate):** `docs/decisions/2026-09-19_g4-g5-four-claims-re-derived.md`,
  which stays byte-unchanged — `docs/decisions/` supersedes rather than mutates.
- **Related:** `docs/decisions/2026-09-21_the-second-ledgers-record-boundary-is-quoted-not-chosen.md`

## The measurement

`LIVE_STATUS.md` calls itself *"authoritative live progress tracker … This is a
current snapshot."* Measured:

| | |
| --- | --- |
| File | 615,710 bytes, 3,156 lines |
| Top-level sections | **one** — `## Qualification correction`, holding 615,412 bytes |
| Correction entries under it | **318**, prepended newest-first |
| The current-status table | **14 rows, 9,726 bytes — 1.58% of the file** |
| Bytes before that table | **570,411 — 92.6% of the file** |
| Widest single line | **7,305 bytes**, wider than the entire layer-A pointer cap |

⛔ **The reader question the file exists to answer is answered by 1.58% of its
bytes, at line 2,639 of 3,156.** `README.md` points a reader here for current
progress, and 92.6% of what they load is a chronological log they did not ask
for.

## The decision

> **The file carries two information roles and needs two lifecycles.** The
> status table is a `bounded_snapshot` — overwritten, small, and FIRST. The
> correction log is a `rolling_ledger` — rotated through git history like the
> two ledgers already running.

⛔ **The ledger rotation shipped for `DEV_NOTES.md` is NOT this file's answer as
it stands, and the reason is specific rather than aesthetic.** That rotation
splits on a record boundary QUOTED from the gate that already governs the file.
`LIVE_STATUS.md` has no gate defining a record: its 318 entries are `✅ **…**`
paragraphs under a single `##`, and nothing enforces that shape. A boundary here
must be CREATED, not quoted, and creating one is a different act with a different
burden of proof. That act belongs to the migration leaf, not here.

## The consumer census, taken before any history moves

Whole-file references, all of which survive any restructuring: `README.md`,
`ROADMAP.md`, `COMMIT.md`, `DOCTRINE_ENFORCEMENT.md`,
`docs/book/src/qualification-review.md`, `docs/book/src/roadmap.md`,
`.doctrine/readme_routes.txt`, `scripts/check_lockstep_claim.sh`,
`scripts/census_registry_read_reach.py`, `scripts/census_shared_registry_writes.py`,
`scripts/census_live_documents.py`, two `docs/knowledge/` notes and three other
decision records.

**Exactly one positional reference exists — and it is broken.**

## The correction

`docs/decisions/2026-09-19_g4-g5-four-claims-re-derived.md` line 112 reads:

> All **3** are withdrawals, not claims: `LIVE_STATUS.md:1719` (*"Historical G5
> subtraction gate withdrew the quality-lift claim"*) …

🔴 **`LIVE_STATUS.md:1719` does not carry that text today, and it did not carry it
at the commit that wrote the sentence.** At `b204c87` — the record's own and only
commit — line 1719 is `| --- | --- | --- |`, a table separator, and the cited
text is at line **1728**, nine lines further down. Today it is at line **2,647**,
928 lines lower again, because 928 lines of correction entries have been
prepended above it in two days.

✅ **The CLAIM is unaffected and nothing is withdrawn.** The text exists, in the
Phase-5 row of the status table, and it still says what the record says it says.
What failed is the pointer, twice over: wrong by 9 at birth, wrong by 928 now.

⭐ The record whose own title ends *"and G4's single test citation no longer
resolves"* contains a citation that never resolved.

> **Do not cite a line number into a prepend-only file.** Every entry added at
> the top invalidates every reference below it, so such a citation is broken by
> construction rather than by neglect. Cite the row, the heading, or the content.

## Why no gate caught it

`scripts/census_positional_refs.py` censuses positional references to **source**
files — a `.rs` path followed by a line number — and reports `unresolved=0` across 597 occurrences in 29
files. A positional reference whose target is a tracked **Markdown** file is
outside its population entirely, so the gate was green over a stale citation the
whole time. ⛔ Extending that population is owned by a named leaf with its own
acceptance, not routed: the rule needs calibrating over the corpus before it can
judge a commit, exactly as `SIGNOFF-REPAIR.11.9`'s rejected gate required.

## What is deliberately not decided here

⛔ No ceiling, no record boundary, and no history moved. The migration leaf owns
the boundary it must create, the ceiling derived from a reviewed survivor, and
the lossless transition.
