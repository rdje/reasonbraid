# A leaf's verification set must re-run the suites of every crate it changed (`PHASE-2.1.5.1`)

- Date: 2026-09-07 · Leaf: `PHASE-2.1.5.1` · Fact record (the `.1.4.2`
  golden drift, found by the `.1.5.1` verification)

## Context

`.1.5.1`'s verification (`cargo test --all` — its NO REGRESSION set) failed
at `schema_goldens_are_in_sync_with_types`: the checked-in
`crates/reasonbraid-core/schema/command-envelope.schema.json` did not match
the live `CommandEnvelope` schema. The envelope gained the optional
`authority_context` in `.1.4.2` — a change to the CORE crate — but the golden
was never regenerated and the `.1.4.2` NO REGRESSION box ran only the live
PostgreSQL suites + CLI e2e + the demo (`bash scripts/run_pg_tests.sh`): the
core crate's own offline suite — the home of the golden-drift test — was not
re-run, so the drift sat undetected for exactly one leaf. The fix
(`cargo test -p reasonbraid-core -- --ignored write_schema_goldens`) rides
the `.1.5.1` commit; the core suite is green again (`test result: ok. 44
passed`).

## answers:

- **A NO REGRESSION set must include the test suites of every crate the
  leaf changed**, not only the live/demo set — the `.1.5.1`/`.1.4.1`
  precedent (`cargo test --all` when the core crate changed) is the rule,
  the live-only set is the exception that must be named and justified in the
  box.
- **A generated golden is a contract, not a convenience**: the schema golden
  drift test is exactly the `.1.5.2`-relevant safety net that a cache wire
  shape will rely on (a deny-unknown-fields envelope change must refuse, not
  silently accept). Regenerate goldens in the same commit as the type change.
- **The doctrine gates did not catch this** — `make gate` checks docs and
  ownership, not test-set selection; the selection discipline lives in the
  leaf's own NO REGRESSION box, so it is the writer's judgment that
  protects it.
