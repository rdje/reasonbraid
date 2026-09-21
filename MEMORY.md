# MEMORY — resume pointer (layer A; overwrite-only)

## Current state (OVERWRITE this block each update — do not append)

- next_action: `SIGNOFF-REPAIR.11.2.1.3.2.3` — bind the cleanup guard at `crates/reasonbraid-server/tests/node_channel.rs`'s 12 remaining `journal-tests` call sites (**1,308 fixtures, 55% of the family**); it needs a live cluster via `scripts/run_pg_tests.sh`. Then the families still unguarded outside `reasonbraid-node`: `node-replacement`, `cached-decision-live`, `r2-join-controls`, `conformance-stubs`, `cli-*`. Scope: `python3 -B scripts/census_fixture_population.py`.
- active_work_unit: `SIGNOFF-REPAIR` → frontier leaf: `.11.2.1.3.2` (`pending`); its open child is `.11.2.1.3.2.3` (`pending`).
- latest_commit: `REASONBRAID-REPAIR-0372`.
- in_flight_uncommitted: none.
- blockers: `SIGNOFF-REPAIR.13`.
