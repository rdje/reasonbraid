# Keyed bootstrap server recovery

Owner: SIGNOFF-REPAIR.3.3.4.3.3.3.2. Product baseline: 8407501.
Final status: server protocol qualified by 73 selected controls (72 live / one pure), final eleven-control fixture rerun and strict lint. Every result/shutdown consumed; four owned clusters absent. Durable CLI persistence remains next. Progress notes below retain the chronology; final results supersede their earlier pending states.

## Matched baseline

`python3 -B scripts/project_env.py python3 -B scripts/run_pg_tests.py bootstrap_recovery`
returned rc=101: zero passed, two failed; build 1m49s, execution 9.15s.
Valid keyed bootstrap returns 422 unknown bootstrap_request_id, rather than 200.
All nine malformed-key/mode cases also return unknown-field 422 instead of semantic
400 invalid_command; the eight-table effect snapshot is unchanged.
Runner target/pg-tests/run-h9lfykn0 stopped; result/shutdown consumed.
Exact cleanup proved stopped metadata, no postmaster file/live owned PID or
matching postgres/pg_ctl command, no symlinks and all entries on repo volume.
Census: 1608 files / 51176471 bytes. Entire command log occurs byte-for-byte
in retained target/bootstrap-server-controls/baseline.log. SHA-256: `33155f937edac62dd8f2c4cf9e2303fac4af00c9a1a808d470c5a9482fe6fb9e`.

## Schema / fixture census before edits

Only api.rs constructs EnrollRequest/EnrollResponse (two response constructors).
The selected table has a tenant identity foreign key, unique tenant mapping,
canonical request key, positive outcome version and JSON object outcome. Service
paths only insert/read bindings; no update/delete/reassignment endpoint is added.
Maintenance/retention must preserve the immutable binding while the tenant is
recoverable; database-owner corruption remains explicitly tested and refused.

Add the request table before tenants in these exact existing fixture purges:

- crates/reasonbraid-cli/tests/cli_end_to_end.rs
- crates/reasonbraid-mcp/src/lib.rs
- crates/reasonbraid-server/tests/allowlist.rs
- crates/reasonbraid-server/tests/cards.rs
- crates/reasonbraid-server/tests/classification.rs
- crates/reasonbraid-server/tests/command_api.rs
- crates/reasonbraid-server/tests/escalation.rs
- crates/reasonbraid-server/tests/evaluation.rs
- crates/reasonbraid-server/tests/federation.rs
- crates/reasonbraid-server/tests/identity_store.rs
- crates/reasonbraid-server/tests/invitations.rs
- crates/reasonbraid-server/tests/mcp_listen.rs
- crates/reasonbraid-server/tests/mcp_write.rs
- crates/reasonbraid-server/tests/node_channel.rs
- crates/reasonbraid-server/tests/node_enrollment.rs
- crates/reasonbraid-server/tests/node_inbox.rs
- crates/reasonbraid-server/tests/node_replacement.rs
- crates/reasonbraid-server/tests/node_work.rs
- crates/reasonbraid-server/tests/policy.rs
- crates/reasonbraid-server/tests/profiles.rs
- crates/reasonbraid-server/tests/quarantine.rs
- crates/reasonbraid-server/tests/quota.rs
- crates/reasonbraid-server/tests/regions.rs
- crates/reasonbraid-server/tests/routing.rs

The migration_upgrade tenants list, authority_transaction count list and
enrollment_transaction/command_api snapshots are observation lists, not purges.
Add migration 0057 upgrade/constraint controls without weakening the FK; extend
the enrollment snapshot to include the new outcome table after the baseline.

Exact baseline cluster removal completed; residue absent.

## Implementation progress

Migration 0057 and private api/bootstrap.rs implement canonical UUIDv7 request
validation, one typed abort/reacquire redirect, shared 15s budget, strict version-one
complete outcome and own-tenant identity/parent bindings. Stored replay ignores
current liveness and creates no authority. Twenty-four named FK purge lists are
updated; normal enrollment snapshots now include the outcome table.

Initial strict server/lib/bootstrap lint passed rc=0 (31.47s); lint with the
first eight bootstrap and upgrade controls passed rc=0 (8.89s). Initial live
run-zil26n1k has eight bootstrap passes (30.22s build / 31.36s execution);
migration test result and shutdown remain pending. Three later controls cover
lost caller acknowledgment, mapping disappearance and the shared operation budget;
they are not part of that initial binary. Final qualification remains pending.

Initial run-zil26n1k completed rc=0: all eight bootstrap controls and all four
migration controls pass. Migration build 8.28s / execution 13.15s. Runner result
and shutdown consumed, cluster removed. A process census observed the migration
executable 3m19s old with 1.47s CPU before test output; a requested one-second
sample returned rc=255 because the process had already exited. No stack sample
or new root-cause evidence was obtained; the existing host-startup investigation
remains .11.2. No unrelated process was altered.

Final runner run-w3qxtbga executes bootstrap_recovery, enrollment_transaction,
command_api, authority_transaction, migration_upgrade. Final strict lint checks
server/CLI/MCP all targets because the named FK fixture lists span these crates.
Both final results remain pending. Evidence: target/bootstrap-server-controls/
qualification.log and lint-final.log with their exit companions.

## Native trust-store wait distinguished from loader delay

An additional one-second sample returned rc=0 for owned bootstrap test PID 71828
(parent cargo 54798); process age 1m29s, CPU 3.32s. All 756 samples of the test
thread reached fixture() → Reqwest Client::new → rustls_native_certs → macOS
SecTrustSettingsCopyCertificates and dispatch wait. The process was already in
the test body by sampling time. It does not establish a dyld stall or identify
why the OS trust service waited. The raw file is named bootstrap-loader.sample
because that was the pre-sample hypothesis; the evidence refutes that attribution
for this captured interval. Sample result is consumed.

Pinned local Reqwest 0.12.28 source confirms the native-root loading branch is
gated by tls_built_in_certs_native, tls_built_in_root_certs(false) clears that
flag, and no_proxy() disables automatic system proxy use. The new fixture serves
only plaintext 127.0.0.1; it now explicitly disables both unrelated lookups.
Production TLS and host settings are unchanged. The broader existing-fixture
census/qualification has an explicit .11.2 follow-up. Final fixture strict lint
returned rc=0, 25.94s; all eleven controls are rerunning in run-4sy2tk7u.

Sample SHA-256: `31f85889daad705472181bf2ec2c6c250872de16ccc052db6a6fd109bf77cbec`.

## Final fixture qualification

`python3 -B scripts/project_env.py python3 -B scripts/run_pg_tests.py bootstrap_recovery`
returned rc=0 in run-4sy2tk7u: all eleven controls pass, build 32.16s
(including Cargo lock wait), execution 75.13s. The runner stopped/removed the
cluster and its full result/shutdown are consumed. No performance improvement is
claimed from a non-benchmark run; the fixture source removes the specifically
observed unnecessary native-trust branch, while other host/fixture waits remain
subject to the .11.2 census.

Actual uncertain COMMIT backend 78482 returned commit_outcome_unconfirmed, followed
by the original complete keyed result; same-key replay returned those IDs without
new rows. The shared-budget control observed sleeping backend 80274, a later
recorded-tenant guard wait, then safe pre-commit failure at 15.00139875s. The earlier
run observed the whole-operation deadline variant at 15.003497583s; the final run
observed statement timeout as shortened statement/total limits meet. Both are
pre-commit safe failures, and all losing provisional rows are absent.

Collision pairs (winner/loser): 80047/80046 committed with matching replay;
80051/80040 committed with conflicting-name 409; 80056/80055 rolled back the first
request and let the second create exactly one result. All required dependencies
were observed through pg_blocking_pids, not inferred from elapsed time.

## Final selected regression qualification

`python3 -B scripts/project_env.py python3 -B scripts/run_pg_tests.py bootstrap_recovery enrollment_transaction command_api authority_transaction migration_upgrade`
returned rc=0 in run-w3qxtbga. All 73 selected controls pass (72 live / one pure),
none ignored or filtered; result and shutdown are consumed, cluster absent.
Production code is the final code; the later fixture-only change is covered by
its separate final eleven-control run above, not counted as new distinct tests.

| Suite | Passed | Build | Execution |
| --- | ---: | ---: | ---: |
| bootstrap_recovery | 11 | 2m33s including Cargo lock wait | 57.55s |
| enrollment_transaction | 9 | 13.47s | 42.78s |
| command_api | 33 | 31.33s including Cargo lock wait | 30.90s |
| authority_transaction | 16 (one pure) | 12.41s | 4.47s |
| migration_upgrade | 4 | 13.65s | 24.86s |

`python3 -B scripts/project_env.py cargo clippy --locked -p reasonbraid-server -p reasonbraid-cli -p reasonbraid-mcp --all-targets -- -D warnings`
passed rc=0 (2m21s); the final fixture-only strict Clippy check passed rc=0
(25.94s). All lint results are consumed. No full CI or push in this leaf.

## Coverage and limits

The eleven keyed controls prove original complete replay, exact name conflict,
ignored human actions, nine semantic invalid key/mode cases, both concurrent
commit and rollback outcomes with actual unique-key dependencies, rollback of
all losing rows/anchors, guard-before-outcome decoding, independent tenant
progress, frozen historical replay, missing/null/extra/wrong/foreign fields,
unsupported version, malformed route and six broken source-reference bindings.
Receipt INSERT and deferred COMMIT faults roll back and recover the same key.
A disconnected caller and a real unconfirmed COMMIT both recover the original
committed result. Mapping disappearance during redirect refuses; a twelve-second
provisional body followed by a guard wait consumes one original fifteen-second
budget. Existing no-key and tenant/name replay wire contracts remain unchanged.

Migration controls retain legacy rows exactly, create no inferred receipt,
reject malformed keys, absent tenant identity, nonpositive versions/non-object
outcomes and duplicate key/tenant mapping, and preserve the receipt on migration
replay. The identity FK is retained and 24 exact fixture purge lists are updated.
RLS migration 0046 remains explicitly limited to the command core; no new RLS or
caller-authentication claim follows from this private bootstrap API. Database
owner mutation is a corruption control, not a supported key reassignment path.

The initial twelve passes and final eleven rerun supplement the selected
regression evidence without inflating the distinct-control count. No all-client
recovery, unrestricted tenant authority, complete introspection or zero-defect
claim is made. Semantic introspection remains the director's tracked .6.4
proposal; the observed native trust-store delay has a concrete .11.2 follow-up.

## Retained evidence hashes

- baseline.log: `33155f937edac62dd8f2c4cf9e2303fac4af00c9a1a808d470c5a9482fe6fb9e`.
- live-initial.log: `576b6fd868a4d1f96c2e449fb534dad98043a079c968f836fdfa5cabd33214de`.
- qualification.log: `edc6507a856b160a3c8fe16eb0107dfcb0ab369d7473737fc138ee215ec9acd2`.
- lint-final.log: `cf48920216738894d5f65aaa1f1d3f0b44ac05f7eb9a4a6c24a54209ac391d4e`.
- fixture-final.log: `1e40320f71c3b80dc10444615048f2a9ae96360128baf118748862591eae4ff6`.
- lint-fixture.log: `0001de0796811bdbdc9662b2ed979e1b6dbc5f477b35fda83c19e66b768a7a54`.

Final cargo fmt/diff checks and make book pass, rc=0; sixteen rendered contract/progress/proposal markers and both parsed JSON examples pass. LIVE_STATUS categories are unchanged; README remains 2054 bytes/52 lines. All required verification and cleanup are complete before REPAIR-0027.
