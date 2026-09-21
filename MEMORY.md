# MEMORY — resume pointer (layer A; overwrite-only)

## Current state (OVERWRITE this block each update — do not append)

- next_action: `SIGNOFF-REPAIR.11.4.6.2` — the per-member judgement for the book-coverage population, carried as a TRACKED TABLE keyed to each of the 30 route families and 10 binaries (never prose), guarded so a member nothing judged refuses. Start where the measurement points: **9 of 30 families are mentioned by no chapter and SIX of the nine are `policy-*`**, so the likely answer is one policy-lifecycle chapter. Scope: `python3 -B scripts/census_book_coverage.py`.
- active_work_unit: `SIGNOFF-REPAIR` → frontier leaf: `.11.4.6` (`pending`); its open child is `.11.4.6.2` (`pending`).
- latest_commit: `REASONBRAID-REPAIR-0377`.
- in_flight_uncommitted: none.
- blockers: `SIGNOFF-REPAIR.13`.
- ⚠️ Standing note from the last verification: the fixture census answers TWO questions and both are needed — `python3 -B scripts/census_fixture_population.py` reports families (what WILL accumulate) AND orphans (what HAS). At `50a9712` the total is 2,619,172 KiB; 11 orphans hold 197,268 KiB, the largest `target/claude-stubs` at 270 directories with no producer. Nothing has been removed; removal has never been this lane's act.
