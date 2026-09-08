# Source census — part 5

Owner: `SIGNOFF-REPAIR.1`. Baseline: `9c2d2ba`. Status of all records: pending reproduction or explicit refutation. Repair contracts: `docs/tasks/SIGNOFF-REPAIR.md`.

## R-83-1

- Repair candidates: `SIGNOFF-REPAIR.5.2`, `SIGNOFF-REPAIR.7.4`, `SIGNOFF-REPAIR.11.1`, `SIGNOFF-REPAIR.11.4`.
- State: open source-review record; runtime pending.

Source review candidate: web/app.js el() converts only strings to text nodes; viewEvents passes numeric aggregate_version into table/td and appendChild(number) throws. Audit actor or other object-valued fields may likewise throw; inspect response types. Concurrent render() calls share same output node with no generation/cancellation: slower old view can append into new view; identity switch does not clear selected thread. deny.toml introductory zero-dependencies claim obsolete despite later reviewed dependency skips. External dependency ledger freshness and roadmap entries need claim alignment after full read.

## R-84-1

- Repair candidates: `SIGNOFF-REPAIR.6.1`, `SIGNOFF-REPAIR.6.3`, `SIGNOFF-REPAIR.7.4`, `SIGNOFF-REPAIR.8.2`, `SIGNOFF-REPAIR.9.1`, `SIGNOFF-REPAIR.11.2`, `SIGNOFF-REPAIR.11.4`, `SIGNOFF-REPAIR.2.1`.
- State: open source-review record; runtime pending.

Source-only: tracked task-acceptance and waiver-routing probe scripts both WORK=$(mktemp -d) rely on ambient TMPDIR and create throwaway git repositories off-volume by default; guards themselves may mktemp similarly. Fix repo-derived local scratch and safe cleanup before running. External-ledger MCP/A2A rows remain tested_versions empty despite claimed subsequent implementation; compare evidence and update within tracking.

## R-85-1

- Repair candidates: `SIGNOFF-REPAIR.3.3`, `SIGNOFF-REPAIR.3.5`, `SIGNOFF-REPAIR.4.1`, `SIGNOFF-REPAIR.4.3`, `SIGNOFF-REPAIR.4.4`, `SIGNOFF-REPAIR.4.5`, `SIGNOFF-REPAIR.5.1`, `SIGNOFF-REPAIR.9.3`, `SIGNOFF-REPAIR.11.4`.
- State: open source-review record; runtime pending.

Source/migration-only: migrations0007 tenant_id independent FK from parent IDs permits cross-tenant host->node, role->incarnation, incarnation->run inconsistencies (fixtures already exploit host/node). authority_grants boundary FK is by boundary_id only, tenant not composite. 0018 unused enrollment-token unique index ignores expired status, preventing new token after unused expired token. 0021 node_inbox_state work_result EXISTS matches operation_id alone, not node_id, so foreign node result with same command_id can mark another inbox consumed. Migration0017 suspended ignores cert expiry: expired unrevoked leaf prevents suspended even no usable leaf. Determine intended observability contract and fix drift conservatively.

## R-86-1

- Repair candidates: `SIGNOFF-REPAIR.4.1`, `SIGNOFF-REPAIR.7.2`, `SIGNOFF-REPAIR.7.4`, `SIGNOFF-REPAIR.8.2`, `SIGNOFF-REPAIR.9.1`, `SIGNOFF-REPAIR.9.2`, `SIGNOFF-REPAIR.11.3`, `SIGNOFF-REPAIR.11.4`, `SIGNOFF-REPAIR.2.1`, `SIGNOFF-REPAIR.2.2`.
- State: open source-review record; runtime pending.

Source-only: scripts/backup.sh echoes DATABASE_URL including password, generates second-resolution destination can overwrite concurrent same-second backup, no restrictive umask or atomic final publication. New operator CLI must never echo credential URLs. Migration snapshots ON DELETE CASCADE allows reference deletion silently remove snapshot/derivation/assessment despite tombstone claims (no current delete verb? track invariant guard). Many policy/evaluation records lack tenant columns/FKs, reinforcing isolation design work, cannot claim simply adding handler enrollment fixes it.

## R-87-1

- Repair candidates: `SIGNOFF-REPAIR.4.1`, `SIGNOFF-REPAIR.4.5`, `SIGNOFF-REPAIR.11.2`, `SIGNOFF-REPAIR.11.4`, `SIGNOFF-REPAIR.2.1`.
- State: open source-review record; runtime pending.

Source-only: scripts/bootstrap.sh de-template uses MAINTAINING.md presence as pristine sentinel, but this active project retains it; named invocation can erase real MEMORY/tree index/decision index and docs with no clean/pristine guard. Also creates owning task-tree only after changes; violates latest user ownership-before-all rule, sed -i GNU-only fails macOS partial destructive sequence. Guard bootstrap explicit pristine state and portable mutation, preserve existing projects. check_docpaths only /Users|/home misses /Volumes and other checkout paths; check_compatibility_matrix token accepts occurrence in any table cell and test-name tokens not measured report integrity. Track bounded doctrine fixes separate from authority repair.

## R-88-1

- Repair candidates: `SIGNOFF-REPAIR.4.4`, `SIGNOFF-REPAIR.7.3`, `SIGNOFF-REPAIR.7.4`, `SIGNOFF-REPAIR.10.1`, `SIGNOFF-REPAIR.11.2`, `SIGNOFF-REPAIR.11.4`.
- State: open source-review record; runtime pending.

Source-only: handoff check suppresses lsof/ps errors under set -uo without -e, can claim no jobs when census unavailable; broad */.codex/* command exclusion can mask actual project worker launched via toolchain path, and ps includes other uid while docs says this uid. Verify reproducible controls before adjusting. Table-arity parser treats pipes inside code as nonseparators, but GFM tables split unescaped pipes even code spans (must verify primary GFM spec when repairing), current selftest may encode falsegreen. Multiple doctrine checks read worktree rather than staged blobs; overall staged evidence can be satisfied by unstaged text. Maintain explicit evidence-limit doctrine, strengthen material bypasses.

## R-89-1

- Repair candidates: `SIGNOFF-REPAIR.4.1`, `SIGNOFF-REPAIR.5.1`, `SIGNOFF-REPAIR.7.4`, `SIGNOFF-REPAIR.11.3`, `SIGNOFF-REPAIR.11.4`, `SIGNOFF-REPAIR.2.1`, `SIGNOFF-REPAIR.2.2`.
- State: open source-review record; runtime pending.

Source-only: check_task_acceptance examines FIRST matching checklist in whole tree rather than active leaf, so prior completed leaf can satisfy new unchecked leaf; reads worktree not index; temp defaults /tmp. Ownership check ignores deletions and scripts/migrations/hooks despite user all-change rule, bypass env should not be used. demo_two_host arbitrary run-id path traversal feeds cleanup paths; weak quoting remote_args/$* permits shell injection; remote binary copies rootworkdir but node_exec uses ./rb-node from a/b subdir (absent). Persists absolute project paths and full DATABASE_URL/secrets in env/channel evidence default permissions. Presence probe unauthenticated while current API enrolled, likely fails after gate changes. Several doublequoted log literals include raw backticks causing shell command substitutions for .1.x labels. Need validate script with focused mocks plus current local integration before claims.

## R-90-1

- Repair candidates: `SIGNOFF-REPAIR.3.4`, `SIGNOFF-REPAIR.4.4`, `SIGNOFF-REPAIR.5.2`, `SIGNOFF-REPAIR.7.2`, `SIGNOFF-REPAIR.7.3`, `SIGNOFF-REPAIR.7.4`, `SIGNOFF-REPAIR.8.1`, `SIGNOFF-REPAIR.9.2`, `SIGNOFF-REPAIR.11.2`, `SIGNOFF-REPAIR.11.3`, `SIGNOFF-REPAIR.11.4`, `SIGNOFF-REPAIR.2.1`, `SIGNOFF-REPAIR.2.2`.
- State: open source-review record; runtime pending.

Source-only: restore.sh --clean deletes target database objects despite isolated-target promise, no identity/live refusal guard and echoes full credential URL. dev/run_pg scripts cleanup unconditionally rm cluster even pg_ctl stop failed; no verified process stop/residue for pg harness. dev.sh no cdROOT then cargo builds caller repo but runs ownbinary; inherited local cache/temp not set. Demo marks threadA closed decision with durable open_challenges1, explicit historical acceptance contract => changing decision close must reconcile deliberate behavior. Demo no-revision check uses ! pipeline without pipefail in bash -c; CLI failure satisfies negative. curl probes often ignore HTTP status, not evidence of successful auth; role ID null checks absent. Load harness rounds up N/C (may exceed requested commands), unvalidatedzero/negative args, fixedport/run files concurrent clobber, readiness may hit unrelated existing server, worker curl/timeouts unconstrained and exit errors not aggregated; summary includes failures in latency, cleanup only kills parent no wait. run_pg harness30 suites not31 exact verify census; migr_upgrade running alphabetical before someothers resets. update_scaffold lists docs/TASK_TREE.md as neutral despite containing project index => overwrite continuity, defaultmktemp offvolume and local cp-R includes target artifacts/caches/possiblysecrets.


