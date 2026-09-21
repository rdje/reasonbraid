answers: should DOCTRINE_ENFORCEMENT.md have a size ceiling; why do the spine doctrine documents have no ceiling when the ledgers do; is it a defect that a governed document is bounded by nothing; how do I decide whether a document needs a rotation threshold; what would make us revisit the no-ceiling decision for a doctrine document; why can a ledger's threshold be derived and a doctrine document's not

# A doctrine document is deliberately unbounded, and the ledgers' threshold cannot be borrowed

- **Type:** `decision`
- **Date:** `2026-09-21`
- **Owner:** leaf `SIGNOFF-REPAIR.11.4.2.7.3.2`, opened by `.11.4.2.7.3.1` when a
  row's control sentence would otherwise have had to declare a ceiling nobody
  had derived.
- **Status:** accepted.

## The question

`SIGNOFF-REPAIR.11.4.2.7.3` reported that `DOCTRINE_ENFORCEMENT.md` has no size
control: 200,000 appended bytes, and the enforcer stays green. Probing its three
spine peers returned the same verdict for all of them — including two that have
carried governed rows for months.

| document | versions | grew | shrank | current | shape | +200 KB probe |
| --- | --- | --- | --- | --- | --- | --- |
| `DOCTRINE_ENFORCEMENT.md` | 41 | 40 | 0 | 56,424 | `append_only` | accepted |
| `TOOLBOX.md` | 46 | 44 | 1 | 52,434 | `at_all_time_high` | accepted |
| `MEMORY_ARCHITECTURE.md` | 2 | 1 | 0 | 24,325 | `append_only` | accepted |
| `COMMIT.md` | 6 | 5 | 0 | 8,077 | `append_only` | accepted |

Four of four, monotone, at their all-time high. Three sibling ledgers in the same
registry each carry a derived `ceiling=`. So: is the absence a defect?

Re-derive, never read from here:

```bash
python3 -B scripts/census_route_controls.py --shapes        # the four shapes
python3 -B scripts/census_mirror_numbers.py --growth        # size against population
```

## The answer: no ceiling, and the reason is that one cannot be derived here

⭐ **A ledger's threshold is derived from a window; a doctrine document has no
window.** `CHANGELOG.md`, `DEV_NOTES.md` and `LIVE_STATUS.md` each retire whole
records into git history, and their thresholds come from one pinned quantity —
how much history must stay *reachable in the file* (`LIVE_WINDOW` × the p90 entry
size). A doctrine document retires nothing: every clause in it describes a rule
that is in force. There is no quantity to derive a number from, so a ceiling here
would be **chosen**, and `SIGNOFF-REPAIR.11.6` forbids a threshold nobody derived.

⛔ **And the remedy a ceiling triggers is the wrong remedy.** A breach says
*rotate the oldest records into git*. For a ledger that is lossless: the record is
still retrievable and the file is the live window. For a reader document it means
deleting rationale that has no other home — and rationale is the content this
project most deliberately writes, because a rule stated without its founding
instance is reachable only by whoever already knows it.

## Why the growth is structural rather than accretion

The mirror's size tracks a population, and the population grew:

| | first version | current | ratio |
| --- | --- | --- | --- |
| `DOCTRINE_ENFORCEMENT.md` bytes | 6,700 | 56,424 | ×8.4 |
| registered doctrines | 13 | 24 | ×1.8 |
| bytes per doctrine | 515 | 2,351 | ×4.6 |

**92.8% of the file is one section** — `## The enforcer registry` — at 51,713 of
55,711 bytes, one row per doctrine. The other four sections total 3,998 bytes.

⚠️ **The ×4.6 is reported, not graded.** It says the per-row rationale has grown
faster than the population, which is a documentation choice rather than a second
information role moving in. `SIGNOFF-REPAIR.11.16` already measured that
population for a different property and **rejected three candidate gates over it**
(66%, 50% and 71% fire rates), on the standing rule that a gate people route
around is a gate that lies.

## ⛔ A ratchet on the ratio is declined, on the same ground

It would be calibrated against a single trajectory — one file, one direction —
which is the objection `.11.4.2.7.1` recorded when it declined a warning-count
ratchet over a population discharged to zero in the same commit. The instrument
reports the ratio on demand; that is the control.

## The trigger that would reopen this

Not a byte count. The condition worth watching is the one
`SIGNOFF-REPAIR.11.4.2.6.5` named on a different file: **a second information
role moving in beside the first.** `LIVE_STATUS.md` was a snapshot that had
become a ledger; `MEMORY.md` was a pointer that had become a warnings board. Both
were visible as a share — the thing the document was named for shrinking to a
fraction of itself.

> If `## The enforcer registry` stops being one row per registered doctrine — if
> its share falls sharply while the file grows, or a new section starts
> accumulating dated entries — the remedy is to **split the roles**, exactly as
> those two leaves did. It is not to cap the bytes.

`python3 -B scripts/census_mirror_numbers.py --growth` reports both halves of
that condition: the section shares and the per-population ratio.

## ⚠️ What this decision does not cover

`TOOLBOX.md`'s row declares its control in prose — *the tool-registry table grows
only with new diagnostic tools (deliberate)* — and nothing evaluates it. It is the
same mirror shape as this document and would be checkable the same way, against
the tracked instruments it lists. It is adjudicated NARRATIVE today and is named
here so the gap is recorded rather than implied by this document's absence from
the list.
