# MEMORY — resume pointer (layer A; overwrite-only)

## Current state (OVERWRITE this block each update — do not append)

- next_action: `SIGNOFF-REPAIR.11.2.1.3.2` — give the fixture producers cleanup-on-success (a guard whose `Drop` skips removal while panicking), starting with `journal-tests` (218,268 KiB / 2,375 fixtures, the largest family by 6x) and its **17** call sites in `crates/reasonbraid-node/src/journal.rs`. The scope is derived, not typed: `python3 -B scripts/census_fixture_population.py`.
- active_work_unit: `SIGNOFF-REPAIR` → frontier leaf: `.11.2.1.3.2` (`pending`).
- latest_commit: `REASONBRAID-REPAIR-0370`.
- in_flight_uncommitted: none.
- blockers: `SIGNOFF-REPAIR.13`.
