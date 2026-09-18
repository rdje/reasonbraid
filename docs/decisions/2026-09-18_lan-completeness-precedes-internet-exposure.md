# LAN completeness precedes Internet exposure

- Date: 2026-09-18
- Status: accepted
- Owner: director instruction, 2026-09-18
- Related: `SIGNOFF-REPAIR.13` (the blocker register), `SIGNOFF-REPAIR.14` (the
  frozen exposure candidate), `docs/book/src/blockers.md`,
  `docs/decisions/2026-09-16_internet-qualification-route.md` (the route this
  re-ranks but does not cancel), ROADMAP §16.12, Phase 7.

## The instruction, verbatim in substance

> The internet exposure is not high priority right now. It needs to fully work
> on the local network first.

## What it changes

**Priority, not ownership.** B1 (externally reviewed threat model), B2
(penetration test) and B3 (prompt-injection action-boundary suite) keep
`Owed here? = yes` — in-repo work genuinely remains for each. What changes is
that they no longer ride the frontier as current work.

`SIGNOFF-REPAIR.13`'s re-surfacing rule said a `yes` row "belongs in the
frontier and is worked like any other leaf". That rule was written when nothing
ranked the rows against each other. It now admits a third state, and the
register records it explicitly rather than leaving the rows to look abandoned:

| Column value | Meaning | How it is surfaced |
| --- | --- | --- |
| `yes` | in-repo work is owed AND is current | rides the frontier |
| `yes (deferred)` | in-repo work is owed and is NOT current, by a recorded ruling | surfaced once per session with its ruling and resumption trigger |
| `no` | parked on someone outside this repository | surfaced once per session with the external party and trigger |

⛔ **`yes (deferred)` is not `no`.** The distinction `.13` was built around is that
an owned-but-unsurfaced blocker is harder to notice than an unowned one. A
deferred row is still owed here, still has a leaf, and is still surfaced — it is
simply not what the next commit should be about.

## What it does NOT change

- `SIGNOFF-REPAIR.14`'s standing prohibition is untouched: nothing in that lane
  may deploy the profile or claim G6/G7. Deferring the work does not licence a
  claim about it.
- Phase 7's G6/G7 remains **NOT MET** and must keep being reported that way.
  This ruling changes what is worked on, never what may be claimed.
- B4 (name clearance) and C1/C2 are unaffected.
- `2026-09-16_internet-qualification-route.md` stands as the route; it is
  re-ranked, not superseded.

## The resumption trigger, and the open question it leaves

The trigger is "the local network deployment fully works". ⚠️ **That is not yet
measurable, and this record says so rather than inventing a definition the
instruction did not give.** The candidate the tree already supports is: every
corrective repair under Phases 1–6 closed, plus Phase 8's LAN-relevant children
(`.5.3` store-and-forward, `.5.4` export/import) — leaving Phase 7 and 9 as the
Internet-facing remainder. That candidate is proposed, not adopted; the director
confirms or replaces it before it becomes a gate condition.

Until then the practical rule is unambiguous and needs no definition: **work the
LAN-path repairs, and do not spend a commit on Internet-exposure work.**
