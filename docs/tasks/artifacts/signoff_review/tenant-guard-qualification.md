# Tenant guard primitive qualification

Owner: `SIGNOFF-REPAIR.3.3.4.2`; predecessor `3031a8c`. This artifact qualifies
the dedicated migration and private transaction owner, compiled by the isolated
integration test. No application authority/effect path is integrated in this leaf.
The earlier 101-file / 42-call source census retains its `1ba6184` baseline.

## Contract exercised

Migration 0056 backfills the exact union of tenant IDs in identities, boundaries,
grants and authorization records. Both union comparison and the guard primary key
use C collation. Guard rows create neither identities nor permissions; first use
inserts the missing anchor in the same transaction. Do not delete live anchors.

The caller declares one to eight typed tenant/mode entries before access. Duplicate
keys strengthen to exclusive; canonical full keys are sorted before insertion and
locking. Shared means FOR SHARE; exclusive means FOR NO KEY UPDATE. The context
checks tenant/mode before lending its executor and has no lock-upgrade API.
Permission checks and tenant-bound SQL remain the integrating caller's obligation.

Explicit READ COMMITTED and database clock_timestamp() at evaluation preserve
freshness after preceding waits. The callback and its transaction cannot escape
the operation runner. Production ceilings are lock 5s, statement 10s, whole
operation 15s including pool acquisition and commit. Internal policies can only
shorten them, in whole milliseconds, with lock <= statement <= total.

Callback/storage failure or cancellation closes the owned connection and cannot
report a success value. A domain Result returned as a successful callback value
can commit a recorded refusal. A health query before COMMIT detects swallowed SQL
errors instead of accepting PostgreSQL's aborted-transaction COMMIT-as-ROLLBACK.
Commit errors/deadlines conservatively report an unconfirmed outcome; no automatic
retry or receipt is invented. Cleanup is asynchronous, and an already-sent COMMIT
can still succeed after the acknowledgment deadline.

## Matched baselines and root causes

1. `run-ro1pp00f`: two existing upgrade controls passed; the new fixed 0055-to-0056
   control failed only because tenant_authority_guards was absent, rc=101. This
   establishes feature absence, not the existing application race. The fixture
   preserves a legacy grant/parent tenant mismatch and all original table fields;
   the migration must not silently repair or omit historical namespaces.
2. `run-hbyms3ia` and diagnostic `run-oe5h8ktw`: 11 primitive controls passed and
   two blocking observations failed, rc=101. Actual diagnostic graph: shared
   holders 68877/68881 remained idle in transaction, while exclusive waiter 68886
   reported only 68881 as its transactionid blocker. In the second control,
   exclusive holder 72339 blocked shared waiter 72341, which blocked exclusive
   waiter 72342 on a tuple lock. The tests incorrectly required every conflict to
   appear as a simultaneous direct edge. The correction traverses observed wait
   dependencies, releases the reported shared blocker first, and observes the
   remaining holder before release. No production lock algorithm changed for
   these fixture failures. PostgreSQL documents both held-lock and ahead-in-queue
   blockers. ([PostgreSQL 16 system information](https://www.postgresql.org/docs/16/functions-info.html))
3. `run-kk8wkcs9`: the corrected contention controls passed; 13 controls passed
   and the delayed-BEGIN control failed, rc=101. Backend 93734 was reused as
   backend 93734, still **idle in transaction**, after the runner returned Deadline.
   The test rolled it back before asserting. The fixed test-only injection delays
   acknowledgment after the server opens BEGIN; production uses a fixed BEGIN
   statement and cannot enable this injection.

The third baseline is a confirmed setup-cancellation defect in the new primitive's
initial reliance on SQLx 0.8.6 pool.begin_with. That pinned driver's begin method
increments transaction depth only after readiness, while cancellation rollback
requires positive depth. The connection lease now exists **before** awaiting
BEGIN and permits reuse only after an acknowledged successful commit; every other
exit marks the connection for closure. This is a local ownership repair, not a
dependency upgrade or a claim about every other SQLx caller in the repository.
([Pinned SQLx PostgreSQL transaction implementation](https://raw.githubusercontent.com/launchbadge/sqlx/v0.8.6/sqlx-postgres/src/transaction.rs))

## Focused control coverage

| Control | Independent observation or recovery |
| --- | --- |
| Compatible shared holders | Two actual backend admissions; exclusive waiter observed; each remaining shared holder excludes it after the other releases. |
| Exclusive holder | Shared and exclusive waiters observed through actual wait dependencies; another tenant completes independently. |
| Full-key order and strongest duplicate | Reversed input waits on B while already excluding A's reader; release B, then observe A still excluded until the multi-key owner commits. |
| Concurrent first use | The second insertion waits on the uncommitted first anchor; exactly one anchor and no tenant/boundary/grant/audit rows are created. |
| Scope, callback failure and refusal | Foreign/stronger-mode borrows refused; original SQL error retained; protected insert rolls back; explicit domain refusal commits its probe. Swallowed SQL error is refused with SQLSTATE 25P02. |
| Body cancellation | Actual backend exits; provisional first-use anchor and protected row are absent; a new exclusive transaction succeeds. |
| BEGIN cancellation | Former backend exits, next pool borrower receives a different idle backend; callback never runs. |
| Decision clock and isolation | Reader observed behind writer, then sees its committed value under READ COMMITTED; database evaluation time follows guard and later statement waits. |
| Lock and statement limits | Actual guard wait ends with original 55P03; long statement ends with 57014; callback/effect and recovery checks distinguish them. |
| Whole deadline | Exhausted ten-connection pool never reaches callback; a waiting callback is bounded and its protected insert rolls back. |
| Local setting reset | Nine pool connections held aside; the same tenth backend uses shortened limits, then returns to its original settings after confirmed commit. |
| Deferred commit failure | Deferred foreign-key failure returns Commit with original 23503 and no success value; protected row and dependent child remain absent. |
| Commit deadline | Actual COMMIT/PgSleep state observed; result is CommitDeadline, then authoritative readback finds the one original committed row without retry. |
| Pure limit validation | Zero, fractional-millisecond, enlarged or inverted limits refused; exact default ceilings and the minimum valid policy accepted. |
| Fixed and moving upgrades | Full old-row/field equality, all four namespaces, deduplication and replay; preceding provenance upgrade and existing-data/latest API enrollment remain valid. |

Initial corrected run `run-1r93kg4l` passed fourteen guard controls (thirteen live,
one pure) and three live upgrade controls, rc=0; guard execution 3.85s and upgrade
execution 9.23s. Strict focused Clippy passed, rc=0 (6.41s). Results and successful
cluster shutdown/removal were consumed. The final backend-exit/anchor assertions
and explicit union collation passed the same fourteen guard and three upgrade
controls in run-js102ri7 (4.26s / 9.43s). Its adjacent authority suite failed all
eighteen cases before assertions with VersionMissing(56), rc=101: Cargo reused a
cached binary embedding only the older migration set. This is a build-dependency
failure, not eighteen newly reproduced authority policy failures.

All four embedding crates lacked migration-directory build scripts. Server, MCP
and CLI embed the shared schema; node embeds its own journal schema. SQLx documents
that stable proc macros track existing migration files but cannot detect added
files without a build dependency on the directory. Their new build scripts derive
the current root at execution and declare the appropriate migration directory.
The directory-dependency probe passed all 28 Cargo check invocations across seven
phases (warm, unchanged, root addition/removal, journal addition/removal, final
unchanged), rc=0. Cargo JSON identifies rb-server, authority, reasonbraid_mcp,
cli_end_to_end and reasonbraid_node artifacts. Both unchanged phases report all
five fresh. Root directory addition/removal rebuilds the four server-schema
artifacts while the node stays fresh. Journal addition/removal rebuilds node;
server authority also rebuilds through its node dev dependency, while rb-server,
MCP and CLI remain fresh. Both temporary non-SQL probe files were removed and
residue absence checked. No source or dependency file was touched to force rebuild.
Commands use cargo check --locked --message-format=json with respectively
-p reasonbraid-server --bin rb-server --test authority; -p reasonbraid-mcp --tests;
-p reasonbraid-cli --test cli_end_to_end; and -p reasonbraid-node --lib.
The added/removed entries are .tenant-guard-dependency-probe in migrations and
crates/reasonbraid-node/migrations. All commands run through project_env.py, offline,
with local JSON logs/results under target/tenant-guard-controls. The probe script
is 2,682 bytes, SHA-256 82d10b6e18fc0346238e764a11a4d95a18bef72e081b31b8c0dd58cd7028894f;
results JSON is 6,539 bytes, SHA-256 ef157bba19ae516e7aab6862c1b705aef7e173dfbb470442385061eccd9d24ea.
([SQLx 0.8.6 migration recompilation](https://docs.rs/sqlx/0.8.6/sqlx/macro.migrate.html))

Final run `run-kypgia2p` passed all 35 controls: thirteen live guard controls, one
pure limit control, three live upgrades and eighteen existing live authority
controls, rc=0. Execution times: guard 4.26s, upgrade 12.60s, authority 0.21s;
build times including Cargo lock waits were 49.19s, 13.71s and 32.04s. Four focused
strict Clippy commands passed, rc=0: server lib/guard/upgrade/authority targets,
MCP tests, CLI end-to-end target and node lib. Their elapsed times were 24.19s,
1m29s, 43.62s and 23.05s, including build-directory waits where reported.
All results and successful cluster shutdown/removal were consumed. All seven
owned cluster residues and both temporary directory probes are absent.

The final live log is 4,406 bytes, SHA-256
4bd85f056c82bb9cece2d29456898b6029d5c8503c530530b3734017a495c61e;
strict lint log 1,575 bytes, SHA-256
e8ea7cd943bec646b59f36e0366d16b7da0ffb5262cbbbd2de8ea6d2fee687e7;
dependency probe log 3,160 bytes, SHA-256
1a861e1ae2a57c9660e1812e93d127759c38983f4181632b2993c9d0e03c9dc0.

## Exact failed-run cleanup evidence

The first commit attempt was also blocked by TASK-ACCEPTANCE, which recursively
treated the evidence index and this artifact as owning task-tree files. The
documented tree location is docs/tasks/<TREE-ID>.md. Its selector now permits only
that direct file level, preserving the existing TEMPLATE exclusion. No artifact
was given a fictitious ownership checklist and no gate was disabled.

Seven isolated Git-fixture controls reproduce both selector errors: five passed
and two failed before the correction, rc=1. The checker wrongly rejected a real
owner co-staged with nested evidence and accepted nested evidence as its sole
purported owner. The latter is this isolated checker's behavior, not a demonstrated
bypass of every commit hook. Corrected controls pass all seven, rc=0 (0.604s),
including unchecked boxes, missing real-tree boxes, unrelated-tree/prose evidence,
template exclusion and docs-only behavior. Actual staged acceptance and bash -n
also pass, rc=0. All seven repository-local Git fixtures and checker scratch
directories were removed; broader staged-snapshot/actual-leaf accuracy stays `.11.2`.
Re-run with project_env.py python3 -B -m unittest scripts.tests.test_task_acceptance -v.
Baseline log: 3,757 bytes, SHA-256
e4686b1c23a462d31470b8b5233d1e9a366f104383d6071b9b5b41899e5b2e8b;
final log: 1,242 bytes, SHA-256
63b1774705dd42b575bf18bc5f6407e9a00c051cbd18c4c9ee663e715ff2f522.

All five failed clusters were verified stopped with no postmaster.pid or matching
PostgreSQL process; all entries were non-symlink repository-volume data on device
16777244. Counts/hashes were captured before exact removal; each residue check was
absent. Logs and machine-readable captures remain under target/tenant-guard-controls
as local evidence, while the durable results are retained here.

| Owned cluster | Files / bytes | Command log SHA-256 | PostgreSQL log SHA-256 |
| --- | --- | --- | --- |
| run-ro1pp00f | 1,605 / 53,051,482 | 306814c89563085402da0c91ba5447c2652501b1ee8c529db3d76b4d2c233fa3 | 7223171ee4ec291648e0d977eaa76418d1911b97b343d7dc7083940cb78cdb1e |
| run-hbyms3ia | 1,624 / 51,285,199 | a73d440cd27b6e51eee9286a176763e50185484734e728c50be0f1bb1114cd87 | a46510e2f3623d3a7855e01b046a27a3960dd7f15f7b2cf68649cd3efc09b391 |
| run-oe5h8ktw | 1,624 / 51,286,452 | c483a86218e28796af9f01f8e6ac75fca614871a7b8db728d8a636f7628c81b8 | 36219028e4b4b1820473f9550ea82bc9e67bdbe98db309194d0ffbadd5471552 |
| run-kk8wkcs9 | 1,624 / 51,284,652 | 5ec53c7404e508063f5faaaac38e938f0b4d26aeb1f8c679a84eb6ba4b00314f | 34d6b18bc951510656f18ae7ade16a40cc50a412e074de7af45da639b11526a6 |
| run-js102ri7 | 1,612 / 70,886,726 | guard: 7cd5c9e147136833190321f53b4c105a3bbaddbcc6f167ddd39deaefb0717723; upgrade: 623aaa4d032f1129eddeb3551c75055e8cc44725578a12bd64c2d1b2c39f1052; authority: c7e75deab9312f136c9558d2c0515bfc7c3832225b4e094953102c8b2d2949bb | c9780fb8020d76eb6ad55f3e5fcaba0007daa1c30d885bbdb8265e4713d251f3 |

No full CI or push in this primitive leaf. Application revocation races and final
effect evidence retain the following children; these probe-table results do not
certify existing administrative or thread/node transaction paths.
