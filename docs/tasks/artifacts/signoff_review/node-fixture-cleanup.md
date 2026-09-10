# Adopt checked cleanup in fourteen node-fixture plans

Owner: `SIGNOFF-REPAIR.11.4.3.1.2.7.2`; REPAIR-0049. Predecessor:
`4e087d525055cb6ef7b02ff1fad52388dddd02da`. Raw controls:
`target/node-fixture-controls`; preserved baselines:
`target/fixture-dependency-controls`. Fixture qualification is complete; the two reproduced pre-existing feature/test
failures have mandatory repair owners before full-checkpoint resumption.

## Scope

The fourteen fixtures are command_api, escalation, evaluation, identity_store,
invitations, node_channel, node_enrollment, node_inbox, node_replacement, node_work,
policy, profiles, routing and CLI cli_end_to_end. All now call the qualified
shared test-only checker after obtaining their ownership-verified pool and applying
migrations. They explicitly delete mcp_listen_state before tenants; the CLI also
deletes spend_breakers. All explicitly name claim_assessments, derivations and
evidence_snapshots before resource_references, making existing cascade effects
visible without changing production referential actions.

Exact source reconstruction verifies that only each private module import and
cleanup loop changed. Original table relative order, test assertions, fixture
seeding and unrelated-state policies are byte-for-byte preserved outside that
scope. In particular identity_store still preserves server_ca; routing's seeded
routing_rules and workflow_profiles remain outside cleanup. The existing
certificate-residue regression is unchanged. The other 325 existing non-Markdown
source files are unchanged, including the shared checker, production code,
migrations, suite registry and manifests.

## Verification boundaries

The prior real MCP→identity and selected spend-breaker→CLI failures are preserved
with their exact logs and stopped databases. Corrected sequences reuse those
actual producers, then the full consumer suites. A third owned database starts all fourteen affected suites in their existing
registry-relative order, but stops at the invitation application failure. The
seven unstarted suites are qualified in separate fresh owned databases so that
any further failure evidence is retained independently. No suite is
reordered to conceal residue and no application assertion is weakened.

The live 41-FK graph is checked independently against all fourteen final plans.
Eleven original explicit-loop plans remain unchanged: six contain sixty restrictive
dependency candidates, owned by `.2.7.3`; remaining adoption and census reconciliation
are `.2.7.4`. These are schema edges, not sixty newly reproduced product defects.
The original node-work→MCP-partial-fixture failure remains pending that next repair.

The MCP producer exercises internal durable database state. This does not qualify
MCP transport or LLM-agent debugging. Full checkpoint, public push and remote CI
remain incomplete.

## Compiler observation

During identity compilation the recorded cargo/rustc group was observed at zero
sampled CPU, with no group child. The bounded native sample completed capture but
exceeded its 45-second supervisor timeout during symbol processing; no report was
written. Its exception, driver log and confirmed group absence are retained.
The identity command later passed without intervention. No stack cause, OS cause
or executable-startup attribution is inferred; broader host diagnosis remains
`.11.2`.

## Application failure discovered during qualification

The first affected run passes six suites, then invitations passes three tests and
fails the participant-removal lifecycle: the valid administrator receives 403,
with `tenant_admin does not support this target kind`. The cleanup completed
before the feature assertion. Seven later suites do not start. Retain stopped
`target/pg-tests/run-9u7iz6xo` and the original receipts/logs.

Source inspection finds that OP_REMOVE_PARTICIPANT chooses TenantAdmin while the
handler constructs a Thread target; the evaluator intentionally requires Tenant
for that action. The new prerequisite `.11.4.3.1.2.8` owns reproduction and repair
after the current fixture unit is committed. The exact pre-edit fixture comparison now reproduces the same 403/200 assertion
(exit 101, one failure/three filtered) on fresh run-6o213qj7, which is stopped and
retained. Its finally guard restores and verifies the current fixture bytes
against the saved hash. Production code is unchanged. This establishes that the
application defect predates this fixture migration; no full affected-suite pass
is claimed. The remaining isolated suites are consumed; their exact results follow.

The isolated profiles suite also returns 101: thirty tests pass and the retention
control receives `tombstoned=0`. It still supplies the fixed
`2026-09-10T00:00:00Z` expiry clock against newly created temporary snapshots.
Source compares creation time to that cutoff minus one day; exact original-fixture
and database-time reproduction is owned by `.11.4.3.1.2.9`. Preserve stopped
`target/pg-tests/run-7d88gdkg` and the log. Later tests continued and cleaned their
fixtures, so this final database is not evidence that the failed test's individual
snapshot rows survived. Production retention scope/client-clock defects retain
their existing `.7.4` ownership.

## Observed retention clock

The exact original profiles fixture reproduces the single retention failure on
fresh `target/pg-tests/run-jgx210ws` (exit 101, thirty other tests filtered). Both
snapshot rows are retained with deleted_at null. PostgreSQL observes temporary
created_at `2026-09-10T06:01:03.342530Z` and standard created_at
`2026-09-10T06:01:03.344175Z`; database observation time is
`2026-09-10T06:01:03.435009Z`. The supplied expiry instant is midnight that day.
Both independent predicates, creation before that instant minus one day and minus
thirty days, are false. Zero tombstones is correct for these rows at that supplied
instant; the fixture's calendar assumption is wrong. Current fixture bytes are
restored and hash-verified after the comparison. The failed database remains
stopped and preserved for `.2.9`; no production retention fix is claimed here.

## Affected-suite results

All fourteen actual cleanup callers complete without a dependency-plan or DELETE
failure. Twelve complete suites pass; invitations and profiles each retain the
known single assertion failure, independently reproduced with their original
fixtures. Across this one affected-suite census there are 135 passing assertions
and two failures, with no skips or ignores. This is **not** an all-green run.

| Suite | Passed | Failed | Exit | Test body seconds |
| --- | --- | --- | --- | --- |
| node_channel | 25 | 0 | 0 | 14.87 |
| command_api | 33 | 0 | 0 | 16.27 |
| node_work | 8 | 0 | 0 | 10.96 |
| identity_store | 4 | 0 | 0 | 9.15 |
| node_enrollment | 5 | 0 | 0 | 10.59 |
| node_inbox | 3 | 0 | 0 | 10.47 |
| invitations | 3 | 1 | 101 | 10.41 |
| escalation | 4 | 0 | 0 | 18.17 |
| node_replacement | 1 | 0 | 0 | 9.24 |
| profiles | 30 | 1 | 101 | 15.74 |
| evaluation | 3 | 0 | 0 | 9.47 |
| routing | 2 | 0 | 0 | 9.52 |
| policy | 11 | 0 | 0 | 11.35 |
| cli_end_to_end | 3 | 0 | 0 | 13.72 |

Separately, MCP listen→identity passes 1+4 tests, and selected spend-breaker→CLI
passes 1+3 (six budget tests deliberately filtered). Row snapshots show one
listener/breaker before each consumer and zero afterward. These repeated consumer
runs are not added to the affected-suite count as distinct test coverage. In the
ordered node-work→identity portion, node/certificate rows clear and the deployment
CA count remains one. All command outcomes and owned-cluster shutdowns are consumed.

Eight successful databases are removed. Four failed databases remain stopped:
run-9u7iz6xo, run-6o213qj7, run-7d88gdkg and run-jgx210ws under target/pg-tests.
Earlier .2.7.1 failures remain preserved separately. Format returns zero in 0.560585 seconds, strict all-target/all-feature server/CLI
lint in 177.305769 seconds, and book in 0.187892 seconds. Final independent
verification returns zero: thirty-six recorded groups absent, all expected
successful/failure residues verified, exact fourteen import/loop-only changes,
325 other source files and eleven remaining plans unchanged, all 41 FKs and twelve
rendered markers reconciled. README stays 52 lines/2,017 bytes; LIVE_STATUS
categories and production/schema/registry remain unchanged. Diff checks pass.
The verifier requires both pre-existing assertion failures and their exact
original-fixture reproductions; its success never relabels those tests as passing.
