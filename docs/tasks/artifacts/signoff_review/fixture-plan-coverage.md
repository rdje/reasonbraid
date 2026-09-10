# Reconcile the original fixture-plan population

Owner: `SIGNOFF-REPAIR.11.4.3.1.2.7.4`; REPAIR-0054. Predecessor:
`ba9ed845205431861cfce4bede25f5ee3f7a300c`. Raw source reconstruction,
case plans, receipts and catalog observations: `target/fixture-coverage-controls`.
All selected runtime, strict lint/book and final source/process checks pass.

## Scope and implementation

The last five original literal-array DELETE-loop fixtures are regions, allowlist,
rls, mcp_write and the MCP library's live_pool. The first, second, fourth and fifth
already delete resource_references; PostgreSQL's existing CASCADE effects delete
three evidence tables. The qualified shared helper requires those children to be
explicit: claim_assessments, derivations and evidence_snapshots, in that order,
before resource_references. This declaration is not a new product defect or a
change to production referential actions. Their existing server_ca deletion also
stays intact. RLS retains its five-table event/outbox scope.

Original/selected plan sizes are regions 60/63, allowlist 58/61, rls 5/5,
mcp_write 54/57 and MCP 54/57. Each selected set is exactly the minimum incoming-FK
closure of its original set against the captured 41-FK catalog, and every original
table retains its relative order. MCP imports the same private helper under
cfg(test), following its existing ownership-support module pattern. Only five
imports and cleanup blocks change; original feature assertions and all 336 other
existing non-Markdown files are unchanged. No production/schema/manifest/runner
change, new test function or new Cargo target.

The rederived original population has 25 checked plans and zero remaining legacy
loops. An independent scan finds no remaining matching loop, and each population
member contains exactly one checked literal plan with valid captured dependencies.
This is not a Rust parser, a census of arbitrary direct SQL or coverage of implicit
relationships without FKs. Runtime schema observations must agree independently.

## Runtime qualification

First execute real node_work, then rls, regions, allowlist, mcp_write and mcp on
one owned database. Observe the RLS fixture's retained node hierarchy/CA and the
broader regions fixture's explicit removal independently of feature assertions.
Then run the original 25 suites, plus the shared guard target, consecutively in
their registered relative order. This includes the repaired invitation lifecycle,
retention boundary test and actual CLI removal/delegation controls. Do not count
repeated tests in both runs as distinct coverage; the MCP library also has pure
schema/parsing tests and the guard target has pure metadata tests.

Any failure must retain its stopped database and command log, get an exact
mechanism diagnosis and concrete repair owner. Prior failed databases and their
receipts/logs remain preserved. No full-checkpoint or MCP-wire/agent qualification
follows from internal/database fixture coverage.

The first sequence is consumed and passes all 22 tests (including four pure MCP
schema/parsing controls); its successful database is removed. Independent SQL
observes nodes/certificates/hosts/incarnations and CA at one after node_work and
unchanged after RLS, including the exact CA row fingerprint. All five counts are
zero after regions, as that broader plan requests. Every schema observation
matches the selected 41-FK catalog.

| Suite | Tests passed | Test body seconds | Command seconds |
| --- | --- | --- | --- |
| node_work | 8 | 14.21 | 15.147814 |
| rls | 1 | 0.05 | 24.716005 |
| regions | 2 | 15.34 | 44.876107 |
| allowlist | 2 | 15.19 | 39.871911 |
| mcp_write | 4 | 15.75 | 40.476259 |
| mcp | 5 | 13.05 | 133.597510 |

These command times include compilation/startup and are observations, not bounds.
The subsequent affected collection includes these same tests; they are repeated
executions, not additional distinct coverage.

## Reproduce and falsify

Use the tracked ownership-gated runner, in the repository's local environment:

```bash
python3 -B scripts/project_env.py bash scripts/run_pg_tests.sh node_work rls regions allowlist mcp_write mcp
python3 -B scripts/project_env.py bash scripts/run_pg_tests.sh pg_guard node_channel command_api node_work identity_store node_enrollment node_inbox invitations escalation node_replacement profiles evaluation routing policy rls quota quarantine classification federation cards mcp_listen mcp_write allowlist regions mcp cli_end_to_end
python3 -B scripts/project_env.py cargo clippy -p reasonbraid-server -p reasonbraid-mcp -p reasonbraid-cli --all-targets --all-features --locked -- -D warnings
```

Prior unchanged producer/consumer failures establish the repeated dependency
mechanism; the shared guard's adversarial controls prove incomplete scopes refuse
before deletion, including empty children and existing CASCADE edges. Live catalog
observations, exact source reconstruction and actual sequential feature outcomes
provide independent re-derivation. The existing suites and mandatory shared
preflight are durable controls. Ignored diagnostic scripts/timings are historical
receipts, not portable future performance or cleanliness guarantees.

## Final verification

All 25 original fixture suites and the guard target pass consecutively: 169 distinct tests, zero skips/ignores/filters. The earlier 22-test consumer sequence repeats part of that coverage (191 total executions). All 32 command snapshots match the 41-FK graph. Format, strict all-target/all-feature server/MCP/CLI lint and book pass. Final verification returns zero: 37 recorded groups absent, both successful databases removed, twenty captured historical failure databases stopped/preserved with unchanged receipts/logs, exact five-import/cleanup reconstruction and 336 other non-Markdown files unchanged. Nine rendered markers match, README remains 52 lines/2017 bytes and LIVE_STATUS categories unchanged.

| Suite | Tests passed | Test body seconds | Command seconds |
| --- | --- | --- | --- |
| pg_guard | 8 | 15.44 | 47.623200 |
| node_channel | 25 | 22.87 | 52.877094 |
| command_api | 33 | 25.22 | 34.732342 |
| node_work | 8 | 12.40 | 12.773068 |
| identity_store | 4 | 15.20 | 40.128305 |
| node_enrollment | 5 | 15.64 | 40.875125 |
| node_inbox | 3 | 14.86 | 39.928656 |
| invitations | 6 | 15.79 | 25.026904 |
| escalation | 4 | 15.87 | 39.949127 |
| node_replacement | 1 | 15.64 | 40.849838 |
| profiles | 31 | 16.31 | 16.621340 |
| evaluation | 3 | 16.09 | 40.246026 |
| routing | 2 | 17.18 | 41.701578 |
| policy | 11 | 18.99 | 42.514075 |
| rls | 1 | 0.03 | 0.296882 |
| quota | 1 | 8.75 | 9.028694 |
| quarantine | 1 | 8.70 | 8.978353 |
| classification | 1 | 16.36 | 25.164976 |
| federation | 1 | 8.80 | 9.092633 |
| cards | 1 | 17.17 | 26.003274 |
| mcp_listen | 1 | 0.03 | 0.298822 |
| mcp_write | 4 | 9.36 | 9.611259 |
| allowlist | 2 | 8.89 | 9.162761 |
| regions | 2 | 8.97 | 9.262827 |
| mcp | 5 | 8.77 | 13.467498 |
| cli_end_to_end | 5 | 21.49 | 30.186316 |

Format: 0.838360s; strict three-crate Clippy: 207.299283s; book: 0.289707s, all exit zero. The 169-test total includes four pure MCP schema/parsing tests and two pure guard metadata tests. The original 25-suite population contributes 161 tests; the additional guard target contributes eight. These are distinct test identities, not new tests added by this leaf.

The consecutive run again observes a node certificate after node_work and none after identity_store, then listener state after mcp_listen and none after mcp_write. Together with the five-consumer scope observations, this binds the source census to actual predecessor cleanup. The full checkpoint remains incomplete and is the next action before any authorized public push/remote CI. Preserve its original failure and host-startup diagnostics.
