# MEMORY — resume pointer (layer A; overwrite-only)

## Current state (OVERWRITE this block each update — do not append)

- next_action: `SIGNOFF-REPAIR.11.2.1.3.2` — the families still unguarded, largest first: `cached-decision-live` 35,272 KiB (`reasonbraid-server/tests/node_work.rs`), `node-replacement` 32,652 KiB (`tests/node_replacement.rs`), then `r2-join-controls`, `conformance-stubs` and the `cli-*` set. Both leaders need a live cluster (`bash scripts/run_pg_tests.sh <suite>`, ~70 s each). Scope: `python3 -B scripts/census_fixture_population.py`.
- active_work_unit: `SIGNOFF-REPAIR` → frontier leaf: `.11.2.1.3.2` (`pending`); `journal-tests` is fully guarded at all 62 call sites.
- latest_commit: `REASONBRAID-REPAIR-0374`.
- in_flight_uncommitted: none.
- blockers: `SIGNOFF-REPAIR.13`.
