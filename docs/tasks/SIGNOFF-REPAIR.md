# SIGNOFF-REPAIR: restore the roadmap's enforced guarantees

## Metadata

- Tree ID: `SIGNOFF-REPAIR`
- Status: `active`
- Created: `2026-09-09` (startup read began 2026-09-08).
- Owner: repo-local engineering; architecture decision delegated by the director.
- Parent: `PROGRAM`; prerequisite repairs for `PHASE-8.5.3`.
- Roadmap: `ROADMAP.md` §§4, 10–20, 25; existing guarantees, defect correction and qualification.
- Baseline: `9c2d2ba`; clean `main`, 269 commits ahead of the locally recorded origin/main.

## Goal

Repair the authority defect in shared registries and the related invariant failures identified by the full startup read. Every finding must be reproduced and fixed, or explicitly refuted with evidence. A historical passing suite is evidence for its exercised assertions, not a blanket production qualification.

## Read prerequisite and ownership

Roadmap: Yes. Tracked codebase: Yes. mdBook: Yes. All were read before this file, the first repository change of this session. The non-Markdown corpus was read in full: 3,276,496 Python characters including path separators, in 92 consecutive 36,000-character pages over sorted `git ls-files`. Markdown roadmap and book sources were read separately. No runtime verification was run during that read. The baseline corpus digest and file count are preserved in `docs/tasks/artifacts/signoff_review/INDEX.md`.

This tree is created before the census artifact, decision record, or live documentation changes it owns. Leaf `.1` owns those documentation changes and their verification. All later changes require their specific leaf to be active before editing. The source-review notes are candidates and source evidence; none is marked runtime-confirmed by this census.

## Execution contract

Run one bounded leaf and commit it through `COMMIT.md` before the next. Expand a leaf into smaller children before implementation if its safe scope requires it. Each code leaf must carry REPRODUCE, ROOT CAUSE, FIX, ADDRESSED, NO REGRESSION and LOCKSTEP evidence. Denial tests assert unchanged protected state as well as status; concurrency tests prove the relevant serialization. Focused checks accompany ordinary commits; full CI runs before push and selected major steps. Preserve the director's PNT instruction and approximately 300-commit push cadence.

The registry decision is explicit site-operator authority, issued only through operator-controlled tooling. Tenant administrator grants remain tenant-scoped. A site grant is bound to an actual enrollment boundary; revoking that boundary freezes its writes. Authorization and mutation serialize with revocation and produce durable audit evidence. This is the selected target contract, not a claim that it already ships.

## Task Tree

### SIGNOFF-REPAIR.1 — Review ownership and authoritative status

- Status: `done`.
- Sources / owned surfaces: `docs/tasks, MEMORY.md, LIVE_STATUS.md, README.md, docs/book`.
- Goal and acceptance: Preserve the full-read census, record the site-operator decision, correct current progress pointers, and map every finding to repair work. This leaf changes documentation only.
- Verification: passed documentation checks and controls; see Leaf .1 closure evidence. Runtime product reproduction remains owned by the implementation leaves.
- Commit: `REASONBRAID-REPAIR-0001` (resolve with `git log --grep`).

### SIGNOFF-REPAIR.2.1 — Repository-local execution environment

- Status: `done`.
- Sources / owned surfaces: `Makefile, scripts, Cargo configuration`.
- Goal and acceptance: Derive caches, scratch, build and tool stores from the current repository; identify required read-only system/toolchain dependencies; inventory off-volume project data and use copy/verify/use/delete only for proven ownership.
- Bounded implementation: a repository-derived command launcher, Makefile integration, verified seeding of the locked Cargo archives/index into a local cache, focused locality/escape/corruption controls, and documented usage. This establishes the execution environment used by subsequent repairs. Remaining direct script/CI entrypoints and intrinsic worker storage behavior are explicitly owned by `.11.2`, `.11.3`, `.7.3` and `.4.2`.
- Measured cause: CARGO_HOME/RUSTUP_HOME unset; default Cargo store and TMPDIR on device `16777232`, repository and target on `16777244`. Cargo.lock has 484 crates.io packages. Shared Cargo registry is approximately 984 MiB and is not uniquely project-owned: copy the required records, never delete the shared source. Installed Rust 1.98.0, Homebrew PostgreSQL/Python/mdBook are read-only toolchain exceptions.
- Verification: 5 launcher controls passed; offline metadata resolved 496 local packages; core tests 49 passed / 0 failed / 1 intentionally ignored schema writer. See acceptance below.
- Commit: `REASONBRAID-REPAIR-0002`.

### SIGNOFF-REPAIR.2.2 — Disposable PostgreSQL verification

- Status: `done`; children `.2.2.1` and `.2.2.2` completed in separate commits.
- Sources / owned surfaces: `scripts/run_pg_tests.sh, tests pool helpers, migration_upgrade, backup_restore, rls`.
- Goal and acceptance: Refuse destructive tests against an unowned database; create isolated local cluster/database/roles, support focused suites, serialize shared fixtures, verify shutdown before cleanup, and prove failure-path residue handling.
- Verification: runner lifecycle and live controls, matched missing-proof and malformed-input reproductions, 40 focused cross-crate tests, final restore and three environment controls passed; evidence in both children.
- Commits: `REASONBRAID-REPAIR-0003` and `REASONBRAID-REPAIR-0004`.

#### SIGNOFF-REPAIR.2.2.1 — Owned PostgreSQL runner and focused selection

- Status: `done`.
- Owns: `scripts/run_pg_tests.sh`, new Python runner and its lifecycle controls, related toolbox/book/CI documentation and live pointers.
- Scope: create a unique repository-local cluster, ignore caller database targets, select named Cargo integration suites, serialize shared fixtures, identify the actual server before any database mutation, supervise/reap PostgreSQL and test commands, and preserve evidence when shutdown cannot be proven. The existing broad suite remains available.
- Measured source cause: the old runner uses fixed port 55432, invokes ambient Cargo without changing cwd, and unconditionally removes the cluster after ignoring `pg_ctl stop` failure. Its single broad command prevents focused authority reproductions. The 32 database-consuming test entrypoints still need independent refusal outside this runner; child `.2.2.2` owns that separate Rust/CI change.
- Acceptance: real focused PostgreSQL suite succeeds; caller URL is not used; lifecycle controls prove failure and signal cleanup, occupied-port refusal and retention on unresolved shutdown. All runner-owned data stays on the repository volume.
- Baseline reproduction: `git show ae99e01:scripts/run_pg_tests.sh` executed unchanged in a repository-local fixture using Bash functions named `stub/initdb`, `stub/createdb`, `stub/pg_ctl` and `cargo`; executable placeholder files satisfy its tool preflight. The initdb stub creates a witness, the stop stub returns 9 and the other stubs succeed; `RB_DEMO=0`. Output: `pg_ctl stop injected exit 9 ; runner exit 0 ; retained cluster directories 0`, rc=0 for the assertion probe. No real server was started. The fixture was removed afterward.
- Signal regression found during verification: SIGTERM between OS process creation and Python handle publication left a test sleeper alive until its 60-second timeout, while PostgreSQL stopped. A deterministic Popen injection reproduced `spawn interrupted before handle publication left a child alive` (1 test failed, rc=1); after deferring terminal signals until handle assignment and restoring child signals through an explicit Python exec trampoline, that test passed (1 test, OK, rc=0). The exact stopped failure fixture was removed after process/receipt verification: 1,271 files, residue False. No outside process was signalled.
- Verification: final lifecycle controls 12 passed; real PostgreSQL controls 4 passed (simultaneous isolation, failure log/receipt preservation and SIGTERM cleanup). `CARGO_NET_OFFLINE=true RB_DEMO=0 DATABASE_URL=postgres://unowned.invalid/never_use bash scripts/run_pg_tests.sh authority` passed all 9 tests, rc=0, through the final runner. Both owned product-test clusters were stopped and removed; the final run was `target/pg-tests/run-srkgcbhd`. No Rust source changed.
- Commit: `REASONBRAID-REPAIR-0003`.

#### SIGNOFF-REPAIR.2.2.2 — Test-side disposable database proof

- Status: `done`.
- Owns: server/CLI/MCP test pool helpers, destructive migration/restore/RLS exercises, CI PostgreSQL invocation, documentation.
- Scope: verify runner-issued ownership against the connected server before migrations, purges, schema drops or role changes; ordinary cargo tests with an arbitrary DATABASE_URL must refuse before destructive SQL. Preserve offline skips and prove negative controls. Wire CI through the same isolated runner, including PostgreSQL tools and repository-local environment.
- Acceptance: real owned tests pass, missing/forged/wrong-server ownership fails before effects, cross-suite shared fixtures serialize, restore and RLS helpers remain isolated.
- Implementation boundary: share a test-only pool helper across the 30 server database suites, CLI integration suite and MCP unit-test module. Preflight the runner's relative workspace, live receipt and exact connection target before connecting; validate server identity on every newly opened pool connection before exposing it to destructive fixtures. Keep the helper out of production library exports. Add focused negative controls and wire the CI PG job through the owned runner. Runner receipts also clear the previous command's exit field when publishing a new active command, so a crash reader sees no stale success attached to in-flight work.
- Historical alignment: annotate the matched census mechanisms with the repaired fixture/runner behavior and its evidence; keep combined records open wherever their other product or operations findings remain unresolved.
- Baseline reproduction: the previously verified `target/debug/deps/authority-32bfc3fd75141971` binary ran its tenant-membership test against a new controlled disposable cluster with RB_TEST_CLUSTER/RB_TEST_OWNER/RB_TEST_DATABASE removed. It returned `1 passed`, rc=0, and wrote 2 authorization rows. The controlled cluster `target/pg-tests/run-30jhvrq6` was stopped and removed. This proves missing ownership metadata did not prevent writes in the baseline; no unowned real database was contacted.
- Corrected missing-metadata control: the rebuilt authority binary returned exit 101 with the ownership-required refusal, and `information_schema.tables` reported zero public tables. The fresh controlled cluster `target/pg-tests/run-xnlq_y59` was stopped and removed. This is the matched negative leg for the baseline's 2 writes.
- Restore malformed-input baseline: the already-built `backup_restore-502fc31ddecf0c57` binary received DATABASE_URL containing byte 0xff (three ownership variables absent). It printed `SKIP: DATABASE_URL is unset` and `1 passed`, exit 0, proving non-Unicode input was misreported as absent. No database was started or contacted. The source now delegates environment validation to the shared helper before reading the validated URL; the rebuilt control then returned exit 101 with the ownership refusal; the absent-variable control still skipped explicitly with exit 0.
- Initial guard verification: `bash scripts/run_pg_tests.sh pg_guard` through the launcher passed 3 controls in 15.03 seconds, rc=0 (unsafe target/receipt/symlink preflight, forged live proof, and denial of a new connection after a role-default marker change). Cluster `target/pg-tests/run-7k9u5c0p` stopped and removed.
- Focused regression: `pg_guard authority command_api backup_restore migration_upgrade rls mcp cli_end_to_end` passed 40 tests (3+9+18+1+1+1+5+2), no failures/ignored tests, rc=0. Cluster `target/pg-tests/run-3hydt24g` stopped and removed. YAML parsed through installed system Ruby/Psych (read-only tool input); inline Python AST and stale command-exit receipt control passed. Rust format check passed. The final source simplifies the restore entrypoint to call the common guard before reading its validated URL, preserving refusal for non-Unicode environment data. Strict all-target/all-feature Clippy passed with warnings denied, rc=0, in 35m05s; the final restore recheck passed (1 test, rc=0), and `target/pg-tests/run-x6yk9h25` was stopped and removed. Three rebuilt-binary environment controls passed: non-Unicode and supplied-without-proof returned 101 with the ownership refusal; absent URL returned 0 with an explicit skip. No database was started or contacted by those environment controls.
- Commit: `REASONBRAID-REPAIR-0004`.

#### Historical census dispositions for `.2.2`

These dispositions apply to the named mechanisms in the baseline source records;
combined records retain their other open findings. Their original observations
remain preserved under `docs/tasks/artifacts/signoff_review/`.

| Record | Corrected mechanism and evidence | Remaining scope |
| --- | --- | --- |
| `R-58-2` | Initial fixture pools require owned receipts; missing-metadata control changes from 2 writes to zero public tables; 40 focused tests pass. | Tenant-admin registry authority and its coverage remain `.3.2`–`.3.3`. |
| `R-58-3` | Supported runner invocations use distinct clusters and serial suites/threads; simultaneous-cluster control passed in `.2.2.1`. | Deliberately sharing a runner's private environment across manual test processes is outside the supported invocation. |
| `R-59-2` | Guard restricts the restore exercise to the generated canonical URL; tmp defaults are repository-local; CREATE/DROP use verified connections and errors fail the test. Actual restore passes. | General backup/restore script safety and failure-artifact handling remain `.11.3`; arbitrary credential/IPv6 URLs are refused by this fixture. |
| `R-66-2` | Schema-drop upgrade exercise now obtains a verified pool first; upgrade test passes. | Historical migration/backfill coverage claims remain `.4.5` / `.11.4`. |
| `R-80-82-1` | RLS role/table cleanup executes only in its owned cluster; real RLS refusal test passes. | The combined product authority, attribution, lifecycle and registry findings remain open under their listed owners. |
| `R-90-1` | PG runner now verifies process shutdown before deletion, records failure evidence and supports explicit ordering. Baseline had 30 server suites; the new guard makes the current count 31. | The other dev/demo/load/restore/scaffold findings remain `.11.2`–`.11.4` and the record's product owners. |

### SIGNOFF-REPAIR.3.1 — Tenant-bound revocation

- Status: `done`.
- Sources / owned surfaces: `api.rs, authority.rs revoke_grant/revoke_boundary`.
- Goal and acceptance: Reproduce a foreign grant/boundary revocation using an own-tenant admin; bind target tenant before mutation and audit; prove victim status and epoch unchanged on refusal, with legitimate revoke preserved.
- Verification: corrected `command_api authority escalation` run passed 34 tests (21+9+4), 0 failed/ignored, rc=0. Cluster `target/pg-tests/run-od1t45mh` stopped and removed. Both matched foreign-target controls preserve victim status/epoch/audit count; rightful revocation succeeds; rejected repeats keep the epoch stable. Two HTTP requests were observed waiting on locks before release in the contention control: one 200, one 409 and epoch 1. Strict `cargo clippy --offline --locked -p reasonbraid-server --lib --test command_api -- -D warnings` passed, rc=0, in 6m35s; Rust format rc=0.
- Commit: `REASONBRAID-REPAIR-0005`.

- Implementation boundary: add the expected tenant to both revocation services, lock and inspect only a matching target before changing status/epoch, and refuse repeated revocation without another epoch bump. Keep the HTTP target hidden by the existing typed 404. Prove both grant and boundary refusals against two independently enrolled tenants, including unchanged victim status/epoch and victim authorization-record count; preserve legitimate revocation and its existing caller-tenant admission audit. Add a concurrent duplicate-grant revoke control to prove one transition/epoch increment.
- Audit qualification boundary: the existing admission audit describes tenant-admin permission, not the final revocation outcome or submitted reason. This leaf must not relabel that record as an effect audit. Atomic authorization/effect audit, reason persistence and serialization with administrator revocation remain explicitly owned by `.3.3` alongside the existing authorization transaction repair; no production qualification advances here.
- Baseline reproduction: `CARGO_NET_OFFLINE=true RB_DEMO=0 bash scripts/run_pg_tests.sh command_api` returned 101, `18 passed; 2 failed`. Both new foreign-grant and foreign-boundary tests observed HTTP 404 followed by victim tuple `(status, epoch, authorization_count)` changing from `(active, 0, 0)` to `(revoked, 1, 0)`. The owned cluster `target/pg-tests/run-7mqjygqr` was stopped, then its stopped receipt and both absent process groups were verified before exact cleanup: 1,590 files / 51,501,466 bytes removed, residue False. This confirms `R-36-39-1` in census-1's committed-mutation ordering at runtime.
- Rejected-repeat reproduction: the unchanged baseline `command_api-4d4db744bfbd2952` binary ran its existing grant-revocation test alone in a new owned cluster (`target/pg-tests/run-00piqqnv`): `1 passed`, rc=0, but a subsequent SQL query measured epoch 2 after one successful revoke and the rejected repeat. The probe asserted that faulty baseline, rc=0, and the cluster was stopped and removed. The same leaf now prevents another epoch bump when the locked target is already revoked, with a repeated-request assertion and a forced two-request contention control.
- Historical disposition: `R-36-39-1` in `docs/tasks/artifacts/signoff_review/census-1.md` is runtime-confirmed; its foreign-target committed-mutation mechanism is corrected by this leaf. The baseline source record remains historical. Admission/effect audit and administrator-revocation races remain open under `.3.3`; no claim is inferred from a 404 or a passing sibling suite.
- promotion: declined (the DEV_NOTES revocation entry records this measured repair of an existing tenant-isolation contract; the leaf preserves its reproductions, fix and re-verification commands, without introducing a new authority policy).

### SIGNOFF-REPAIR.3.2 — Site-operator registry authority

- Status: `active`; execute and commit `.3.2.1`, `.3.2.2`, then `.3.2.3`.
- Sources / owned surfaces: `api.rs require_admin_any_tenant, allowlist.rs, regions.rs, operator tooling, new migration`.
- Goal and acceptance: Implement explicitly issued site authority for adapter and region mutations, deny tenant-admin escalation, check actual bound boundary and grant liveness, serialize revocation with effects, and record durable actor/action/target/reason/decision audit. Operator-controlled issuance and revocation must be tested; no HTTP enrollment minting.
- Verification: pending; capture the failing case, corrected case, and independent control in this leaf or its children before closure.
- Commit: pending.

- Implementation policy: separate site boundaries/grants from tenant authority; no migration seeding or tenant-enrollment minting. Site registry inspection requires an explicit `registry_inspect` capability with a live grant and its actual live boundary. Tenant-scoped frozen-admin inspection remains unchanged. Operator tooling derives its issuer/audit identity from the authenticated database session and requires database superuser authority or explicit membership of a deployment-managed `reasonbraid_site_operator` role; tenant grants never satisfy that gate. No host/database role is created automatically by migration.

#### SIGNOFF-REPAIR.3.2.1 — Site authority and transactional registry service

- Status: `done`.
- Owns: new migration for site boundaries/grants/audit/serialization guard, server site-authority module/exports and direct UUID dependency plus Cargo.lock, repository-owned baseline probes, focused live service tests, shared test-helper verification access for restricted-role connections, and runner suite registration, decision/book/live-doc synchronization.
- Scope: explicit typed site actions and subjects; immutable grant-to-boundary binding and scope/window subsets; protected database-session issuance and irreversible revoke/suspend operations; inspectable durable issuer/reason/outcome records; a single registry service that checks the actual grant/boundary and writes the effect plus audit in one transaction. Candidate grants must not shadow another usable grant. Serialize short site-configuration transactions through one database guard row before reading the actual grant/boundary; operator status changes take the same guard. Use READ COMMITTED, a fresh database decision time after lock acquisition, bounded lock/statement timeouts, and no external work inside the transaction. Preserve no-op attribution and fail closed on audit errors.
- Acceptance: reproduce legacy any-tenant/frozen-boundary registry writes in an owned cluster; prove explicit operator service success and tenant-grant-only refusal, actual boundary binding, all liveness windows/states, safe multiple-grant selection, no-op attribution, audit rollback and deterministic grant/boundary revocation races in both orders. No HTTP routing change or public operator CLI yet; those remain the following children.
- Verification: `CARGO_NET_OFFLINE=true RB_DEMO=0 bash scripts/run_pg_tests.sh site_authority migration_upgrade allowlist regions command_api` passed 36 tests (10+1+2+2+21), zero failures/ignored, rc=0; owned cluster `run-ky9ees5v` stopped/removed. Focused strict Clippy for server lib/site test passed, rc=0; format, book build, rendered-content inspection and diff check passed. Legacy HTTP tests remain compatibility controls, not site-isolation qualification.
- Commit: `REASONBRAID-REPAIR-0006` (this commit).

- Legacy runtime reproduction: the existing `target/debug/rb-server` ran against owned cluster `target/pg-tests/run-ddzcfr8_`; its PID-owned loopback listener was identified by `lsof` before HTTP writes. Two independently enrolled tenant administrators added a shared adapter/region. After the second administrator revoked its tenant boundary through HTTP, it still added both another adapter and another region (all four shared writes returned 200). SQL measured `revoked boundaries|new adapters|new regions|authorization records = 1|2|2|1`; the only admission record was the tenant-boundary revocation. Probe rc=0. The server process group was stopped/reaped, then the PG cluster stopped/removed. Probe source remains repository-local at `target/site-authority-controls/legacy_registry_probe.py`; this summary is the durable baseline evidence.
- Initial service verification is **failed**, not qualified: `CARGO_NET_OFFLINE=true RB_DEMO=0 bash scripts/run_pg_tests.sh site_authority` compiled successfully but returned 101 (1 passed, 8 failed). Seven tests hit the existing internally tagged primitive `GrantSubject::Human` serialization defect when the new issuance audit used `json!`; the new service will encode an explicit kind/id object, and the core contract repair is owned by `.3.3`. The other failure is a queued database-member issuance after committed membership revocation. An isolated exact-test run returned 101 at test line 614: the ordinary outsider was refused, but the queued member produced a second allowed boundary issuance while a separate session reported both test roles non-superuser/non-member. The isolated cluster `run-vu3e3zsk` was stopped/removed; sanitized transcript is local at `target/site-authority-controls/role-isolation.log`. This leaf owns a minimal PostgreSQL prepared-statement/catalog visibility probe to root-cause that surprising result before any qualification claim. Original failed cluster `run-ab49o7ev` is stopped and retained pending evidence consumption/cleanup.
- Role-check root cause isolated: the simple-protocol psql control in `run-0poban9b` reported membership true/one direct edge before the guard wait and false/zero afterward under READ COMMITTED. A native libpq prepared-query control in `run-gvzqw7p2` reproduced true after committed REVOKE and guard release; a freshly parsed simple query then saw zero membership edges and the subsequent prepared query returned false. Both probes returned rc=0 and stopped/removed their clusters. PostgreSQL 16.15's `acl.c` caches role membership with syscache invalidation callbacks; repeated prepared execution alone did not refresh it in this control. The service gate now uses a static text Executor statement for a freshly parsed simple-protocol identity query before and after the wait. No caller data is interpolated. A diagnostic service rerun in `run-6cubiypy` passed the seven previously panicking tests and the remaining ordinary control (8 passed), while queued issuance still failed even after a separate connection explicitly proved membership false before releasing the guard (1 failed, rc=101). Temporary identity logging was removed before the corrected run. The regression retains the independent visibility assertion and reuses one physical operator connection.
- Compilation control: the first fresh-query `raw_sql(...).fetch_one(...)` spelling failed Executor/Send lifetime requirements in spawned callers (`run-rz4sy0sg`, compile rc=101, no tests executed). Calling the text Executor directly preserves simple protocol and `cargo check --offline --locked -p reasonbraid-server --test site_authority` passed, rc=0. The failed-run receipt is stopped; its evidence is consumed and the exact owned workspace may be removed after the process-group census. The role-cache lesson is promoted to `docs/decisions/2026-09-09_operator-role-query-freshness.md`.
- Cleanup disposition: after consuming the failure records and proving both recorded process groups absent, exact stopped workspaces were removed with no residue: `run-ab49o7ev` (1,603 files / 51,365,895 bytes), `run-6cubiypy` (1,606 / 51,673,698), `run-rz4sy0sg` (1,272 / 48,216,926). Command-log SHA-256 values respectively `aaceaff87bbb110473f2ca4d41d379ed64d1ec399889936101dabd46b0b4701a`, `4afdf7fae27b217d2aa1a5a1552ac11d2bab15b9991d99a3e0b84b35f362085c`, `103df40bf61d76f38e2ac2884d5e350e8dd607a62f979f6f03f5799ecd40d5ad`. The census refused symlinks/off-volume objects and required stopped receipts; cleanup rc=0.
- Current implementation: migration 0054 and the site service are additive. No tenant grant is upgraded, no database role is auto-created, and no HTTP handler is switched in this child. Human/role payloads, actual-parent and every-usable-grant selection, irreversible status transitions, liveness, domain refusals, effect/no-op/read audits, rollback injection, both revocation orders and queued expiry have live controls. Database-role gate freshness has the independent native libpq reproduction and corrected reused-connection regression. README layout/standard commands are unchanged; the site manual, qualification/authority/progress pages, LIVE_STATUS evidence, task/programme indexes, decisions, TOOLBOX and live memory stay aligned. LIVE_STATUS category values do not advance.

#### SIGNOFF-REPAIR.3.2.2 — Protected operator CLI

- Status: `pending`.
- Owns: dedicated server-crate operator binary, CLI integration controls, operator runbook and examples.
- Scope: issue/inspect/revoke/suspend site boundaries and grants through the protected service, requiring explicit subjects/actions/windows/reasons and deriving issuer identity from the database session. Inspect audit history through that same deployment-controlled surface. Use repository-derived output/storage and avoid credential disclosure. No tenant HTTP enrollment path may invoke issuance.
- Acceptance: real CLI issuance/use/revocation, restricted database-role refusal with unchanged authority/audit state where no audit privilege exists, deterministic argument/input refusals, and replay/no-op history. Book examples must be exercised.
- Verification: pending.
- Commit: pending.

#### SIGNOFF-REPAIR.3.2.3 — Registry HTTP enforcement and qualification

- Status: `pending`.
- Owns: adapter/region handlers, request validation and reason inputs, legacy fixture replacement, HTTP cross-tenant/freeze/audit/race controls, history and live-book correction.
- Scope: route all shared adapter/region reads and mutations through the verified site service, remove the any-tenant-admin helper for these routes, require a reason on mutations, preserve existing successful response shapes, and expose attributable refusal references. Distinguish domain refusal from SQL failure. Keep tenant-owned inspection behavior.
- Acceptance: operator success across every verb; two distinct tenant admins refused with unchanged registry rows; revoked/suspended/future/expired site grant or actual boundary refused; no enrollment-based minting; mutation/inspection read separation; audit/no-op/error persistence and deterministic HTTP revocation races. Run the affected registry/authority/CLI checks and the selected broader security gate; no Internet qualification claim.
- Verification: pending.
- Commit: pending.

### SIGNOFF-REPAIR.3.3 — Bound-boundary authorization and grant selection

- Status: `pending`.
- Sources / owned surfaces: `core authority, server authority.rs`.
- Goal and acceptance: Resolve a grant's actual boundary, enforce identity/tenant/subset/window correspondence, reject thread-only selectors for tenant actions, avoid latest-grant shadowing, and serialize all relevant authorization/mutation paths against revocation. Revocation administrative paths must persist the submitted reason and final outcome in an attributable effect audit atomically with status/epoch changes; the current tenant-admin admission audit is not that effect record.
- Additional runtime-confirmed defect owned here: `GrantSubject` derives internally tagged serde encoding over transparent primitive ID newtypes (`crates/reasonbraid-core/src/authority.rs`), so serializing `Human` fails with `cannot serialize tagged newtype variant GrantSubject::Human containing a string`; `json!` panics on that error. The initial `.3.2.1` live service run reproduced it in seven tests. Audit all consumers and establish an explicit, tested human/role wire contract with round-trip and enclosing-payload controls; the site service's explicit kind/id encoding does not close this core defect.
- Verification: pending; capture the failing case, corrected case, and independent control in this leaf or its children before closure.
- Commit: pending.

### SIGNOFF-REPAIR.3.4 — Delegation and cache freshness

- Status: `pending`.
- Sources / owned surfaces: `core delegation/cache, authority.rs, command envelopes`.
- Goal and acceptance: Enforce delegability, bounded depth, actor/subject participation and consent; bind replay hashes to target and authority context while preserving approved committed-replay semantics; make cached decision expiry and future-clock behavior explicit.
- Verification: pending; capture the failing case, corrected case, and independent control in this leaf or its children before closure.
- Commit: pending.

### SIGNOFF-REPAIR.3.5 — Tenant-owned administration

- Status: `pending`.
- Sources / owned surfaces: `api.rs node inbox/prune/quarantine/replay, breaker, enrollment, admin services`.
- Goal and acceptance: Use real target ownership inside the mutation transaction; replace foreign/nonexistent-node success fixtures and prove foreign reads/writes leave all affected rows unchanged; retain the approved own-tenant frozen-admin read carve-out.
- Verification: pending; capture the failing case, corrected case, and independent control in this leaf or its children before closure.
- Commit: pending.

### SIGNOFF-REPAIR.4.1 — Node enrollment and certificate lifecycle

- Status: `pending`.
- Sources / owned surfaces: `node_enrollment, node_channel, ca, rb-node, migrations`.
- Goal and acceptance: Bind token use to current issuing authority; recover expired unused token issuance; enforce host/node/incarnation tenant lineage; make replacement lineage and lease effects consistent; serialize rotation/revocation and bound renewal after revocation.
- Verification: pending; capture the failing case, corrected case, and independent control in this leaf or its children before closure.
- Commit: pending.

### SIGNOFF-REPAIR.4.2 — Handshake and lease fencing

- Status: `pending`.
- Sources / owned surfaces: `node_channel, channel client, certificate/key storage`.
- Goal and acceptance: Reproduce proof replay and fence races; prevent replay-based private-key recovery; validate current lease atomically for every fenced write; secure and atomically persist keys and rotations; distinguish shipped HTTP proof from TLS capability.
- Verification: pending; capture the failing case, corrected case, and independent control in this leaf or its children before closure.
- Commit: pending.

### SIGNOFF-REPAIR.4.3 — Durable inbox identity and cursors

- Status: `pending`.
- Sources / owned surfaces: `node_inbox, node_events, node_inbox_state, channel polling`.
- Goal and acceptance: Bind receipts/reconciliation/dedup to tenant and node; preserve monotonic cursors through pruning; serialize enqueue; prove command-id collisions across nodes cannot consume or hide foreign work.
- Verification: pending; capture the failing case, corrected case, and independent control in this leaf or its children before closure.
- Commit: pending.

### SIGNOFF-REPAIR.4.4 — Node recovery and unknown outcomes

- Status: `pending`.
- Sources / owned surfaces: `node journal/supervisor/worker, node_work/replacement tests`.
- Goal and acceptance: Close terminal-result/outgoing-event crash gaps, reconnect transport failures, bound streams/deadlines, reconcile actual evidence, settle refused results, and require explicit duplicate-risk authorization before replaying unknown provider outcomes.
- Verification: pending; capture the failing case, corrected case, and independent control in this leaf or its children before closure.
- Commit: pending.

### SIGNOFF-REPAIR.4.5 — Budget and quota serialization

- Status: `pending`.
- Sources / owned surfaces: `core budget, budget.rs, quota.rs, outbox worker`.
- Goal and acceptance: Reject arithmetic overflow/negative usage; bind reservations to tenant/thread/attempt; lock ceiling/breaker/quota admission, preserve uncertain holds, validate settlement transitions, fence delivery effects and completion, and prove concurrent ceilings.
- Verification: pending; capture the failing case, corrected case, and independent control in this leaf or its children before closure.
- Commit: pending.

### SIGNOFF-REPAIR.5.1 — Directory and profile isolation

- Status: `pending`.
- Sources / owned surfaces: `profiles, matching, presence, directory endpoints`.
- Goal and acceptance: Apply visibility per candidate tenant, use equally qualified foreign fixtures, prevent private-feature leaks, serialize updates/attestations, validate ranking bounds and missing dependence facts, and enforce capability expiry/concurrency.
- Verification: pending; capture the failing case, corrected case, and independent control in this leaf or its children before closure.
- Commit: pending.

### SIGNOFF-REPAIR.5.2 — Recruitment and autonomous initiation

- Status: `pending`.
- Sources / owned surfaces: `calls, offers, panels, auto-thread endpoints`.
- Goal and acceptance: Bind call/thread/tenant/actor and grants, make panel/close/offer transitions atomic, enforce post-filter minimums and concurrent caps, and make repeated legitimate auto initiation possible with full budget/topics/classification semantics.
- Verification: pending; capture the failing case, corrected case, and independent control in this leaf or its children before closure.
- Commit: pending.

### SIGNOFF-REPAIR.5.3 — Federation and portable cards

- Status: `pending`.
- Sources / owned surfaces: `agreements, card import/export, receipts`.
- Goal and acceptance: Complete remote recruitment under explicit local grants, bind receipts to actual digest references, transact identity/profile/quota/receipt import together, isolate replay and provenance, and prove revocation races cannot widen visibility or effects.
- Verification: pending; capture the failing case, corrected case, and independent control in this leaf or its children before closure.
- Commit: pending.

### SIGNOFF-REPAIR.6.1 — MCP read and write authorization

- Status: `pending`.
- Sources / owned surfaces: `mcp_read, mcp_write, reasonbraid-mcp`.
- Goal and acceptance: Require per-target tenant and grant checks on inbox/thread/policy reads and all writes; reject scalar payloads without panic; prove policy/recruitment handlers cannot use enrollment as authority; keep quota admission semantics accurately documented.
- Verification: pending; capture the failing case, corrected case, and independent control in this leaf or its children before closure.
- Commit: pending.

### SIGNOFF-REPAIR.6.2 — MCP durable delivery and transport

- Status: `pending`.
- Sources / owned surfaces: `mcp_listen, reasonbraid-mcp`.
- Goal and acceptance: Retain the latest bounded dedup window, serialize first delivery and monotonic cursor updates, validate state instead of dropping malformed data, and implement the documented reauthorization/reconnect/transport profile with real wire evidence.
- Verification: pending; capture the failing case, corrected case, and independent control in this leaf or its children before closure.
- Commit: pending.

### SIGNOFF-REPAIR.6.3 — A2A executable interoperability

- Status: `pending`.
- Sources / owned surfaces: `reasonbraid-a2a, compatibility records`.
- Goal and acceptance: Make semantic-loss defaults honest, wire mapped messages through local grants, and demonstrate a real independently connected peer; serialization-only tests must remain labeled as such.
- Verification: pending; capture the failing case, corrected case, and independent control in this leaf or its children before closure.
- Commit: pending.

### SIGNOFF-REPAIR.7.1 — Authenticated resource and resolver ownership

- Status: `pending`.
- Sources / owned surfaces: `resource_references, resolvers, API registration`.
- Goal and acceptance: Bind writes/reads to explicit tenant or site authority; prevent global URL first-writer poisoning and partial-upsert stale claims; enforce expected content digests, declared capabilities, deterministic ranking and supported execution.
- Verification: pending; capture the failing case, corrected case, and independent control in this leaf or its children before closure.
- Commit: pending.

### SIGNOFF-REPAIR.7.2 — HTTPS and Git acquisition safety

- Status: `pending`.
- Sources / owned surfaces: `fetcher, ssrf, git_acquire`.
- Goal and acceptance: Validate destinations and credential forwarding at every redirect and dial, including numeric IPv4/IPv6; bound download/decode/object/time consumption before allocation; prevent pipe deadlocks and remove owned scratch on every path; verify supported refs and canonical digest framing.
- Verification: pending; capture the failing case, corrected case, and independent control in this leaf or its children before closure.
- Commit: pending.

### SIGNOFF-REPAIR.7.3 — Extraction, browser and credential workers

- Status: `pending`.
- Sources / owned surfaces: `extract worker, browser worker, acquisition pipelines`.
- Goal and acceptance: Wire PDF/archive/feed inputs to supported acquisition; bound parse/decode/output, drain pipes concurrently, reap descendants, enforce browser redirect/subresource isolation and credential origin binding, and prove malicious local fixtures cannot escape.
- Verification: pending; capture the failing case, corrected case, and independent control in this leaf or its children before closure.
- Commit: pending.

### SIGNOFF-REPAIR.7.4 — Evidence integrity and retention

- Status: `pending`.
- Sources / owned surfaces: `snapshots, derivations, claim_assessments`.
- Goal and acceptance: Bind metadata and authors to authenticated actions, make object+snapshot writes atomic, refresh actual freshness horizons, retain tombstone honesty, restrict expiry clocks/scope, and distinguish excerpt existence from claim entailment with explicit evidence gates.
- Verification: pending; capture the failing case, corrected case, and independent control in this leaf or its children before closure.
- Commit: pending.

### SIGNOFF-REPAIR.8.1 — Workflow and thread state invariants

- Status: `pending`.
- Sources / owned surfaces: `thread engine, workflow registry, profiles tests`.
- Goal and acceptance: Version and authorize shared workflow registration, prevent built-in override and MAX+1 races, track each challenge's resolution once, recover expired invitations, validate duration bounds and record attribution, and reconcile durable unresolved challenges with close contracts.
- Verification: pending; capture the failing case, corrected case, and independent control in this leaf or its children before closure.
- Commit: pending.

### SIGNOFF-REPAIR.8.2 — Evaluation and routing evidence

- Status: `pending`.
- Sources / owned surfaces: `evaluation service, benchmark, routing`.
- Goal and acceptance: Bind corpora/digests/cases/arms/run references, validate seeds and duplicate assignments, reject missing/non-numeric gate measurements, derive calibration from eligible runs, and audit routing only in the intended authorized transaction.
- Verification: pending; capture the failing case, corrected case, and independent control in this leaf or its children before closure.
- Commit: pending.

### SIGNOFF-REPAIR.9.1 — Policy registration and authority

- Status: `pending`.
- Sources / owned surfaces: `policy registry, lifecycle, approvals, corrections`.
- Goal and acceptance: Tenant-scope all material records, bind claimed authorities to the authenticated caller and live action/scope/boundary, enforce immutable content digests and fail-closed selectors, and validate lifecycle/dependency/precedence/waiver semantics.
- Verification: pending; capture the failing case, corrected case, and independent control in this leaf or its children before closure.
- Commit: pending.

### SIGNOFF-REPAIR.9.2 — Atomic policy lifecycle and publication

- Status: `pending`.
- Sources / owned surfaces: `proposals/decisions/approvals, projections, publisher, reconciler`.
- Goal and acceptance: Serialize stage transitions, bind projection and manifest to approved policy, constrain filesystem targets, reject fabricated effective Git IDs, make CAS retries recoverable, verify both immutable and effective refs, and reconcile DB/Git failure points.
- Verification: pending; capture the failing case, corrected case, and independent control in this leaf or its children before closure.
- Commit: pending.

### SIGNOFF-REPAIR.9.3 — Deployment, correction and review lifecycle

- Status: `pending`.
- Sources / owned surfaces: `deployment assignments, drift, corrections, reviews`.
- Goal and acceptance: Bind desired digests/refs to publication, validate receipts and corrective authority, permit subsequent reviews after completed occurrences, enforce waiver constraints, and use relative test clocks with failure visibility.
- Verification: pending; capture the failing case, corrected case, and independent control in this leaf or its children before closure.
- Commit: pending.

### SIGNOFF-REPAIR.10.1 — Adapter subprocess supervision

- Status: `pending`.
- Sources / owned surfaces: `codex/claude adapters, process maps, fixtures`.
- Goal and acceptance: Bound UTF-8-safe stderr/stdout/chunk storage, drain concurrently, reap terminal children, prevent prompt-option injection, and preserve honest pre-dispatch versus unknown-outcome semantics.
- Verification: pending; capture the failing case, corrected case, and independent control in this leaf or its children before closure.
- Commit: pending.

### SIGNOFF-REPAIR.10.2 — Certification and release verification

- Status: `pending`.
- Sources / owned surfaces: `adapter certification, verification ladder, release-manifest tool`.
- Goal and acceptance: Bind adapter identity/capabilities/artifact to signed complete scenario evidence; require actual coverage of every invariant; secure key creation and manifest paths/hex parsing; implement load-side checks and document unsupported SDK execution.
- Verification: pending; capture the failing case, corrected case, and independent control in this leaf or its children before closure.
- Commit: pending.

### SIGNOFF-REPAIR.11.1 — Console behavior

- Status: `pending`.
- Sources / owned surfaces: `web/app.js, web-ui book chapter`.
- Goal and acceptance: Render numeric and structured values as inert text, prevent stale asynchronous views after navigation/identity change, and prove real browser timeline/audit behavior.
- Verification: pending; capture the failing case, corrected case, and independent control in this leaf or its children before closure.
- Commit: pending.

### SIGNOFF-REPAIR.11.2 — Repository safety and doctrine accuracy

- Status: `pending`.
- Sources / owned surfaces: `bootstrap/update_scaffold, check scripts, task acceptance probes`.
- Goal and acceptance: Protect populated repositories, preserve project indexes, enforce ownership before all changes, validate staged evidence for the actual leaf, reject failed process censuses, and correct table/path checks against primary specifications.
- Host execution follow-up from `.2.2.1`: a fresh `#!/bin/bash` stub under `target/pg-runner-baseline-controls` printed `ready` in 0.0 seconds through `/bin/bash <stub>` but direct execution timed out at 3 seconds; a baseline Python-shebang stub also stalled for 59 seconds. `BASH_ENV` was unset. An authorized one-second `sample` of the exact fresh stub returned 897 samples at `_dyld_start + 0`, before interpreter code; all probe groups were stopped/reaped and fixture census returned no residue. Cause is narrowed to host/loader execution startup, not the script body; the underlying host control is unproved. The current PG runner enters Python/Bash explicitly and its actual installed PostgreSQL processes pass. Follow-up: reproduce direct versus interpreter startup in the supported execution environment, identify the responsible loader/security/tooling control before changing any host policy, and retain explicit-interpreter entrypoints where sufficient. No global security setting has been changed.
- Additional `.2.2.2` lint evidence: after 8 minutes elapsed and about 2 seconds CPU, owned rustc PID 49982 was sampled for one second (`sample`, rc=0). All 800 samples of its compiler thread were in `rustc_metadata::host_dylib::load_dylib → dlopen → dyld4::Loader::mapSegments → fcntl → __fcntl`; `lsof` identified open descriptor 7 as `target/debug/deps/libasn1_rs_derive-5b39009ea2780f94.dylib`. The main thread waited for that compiler thread. This narrows the long Clippy delay to host dynamic-library loading for that process; it does not prove the responsible OS/security service. Raw local evidence: `target/doctrine_scratch/clippy-process-sample.txt`. Clippy continued advancing other dependency checks. Keep this follow-up open; no host policy or library signature was changed.
- Verification: pending; capture the failing case, corrected case, and independent control in this leaf or its children before closure.
- Commit: pending.

### SIGNOFF-REPAIR.11.3 — Operational scripts and evidence

- Status: `pending`.
- Sources / owned surfaces: `backup/restore/dev/demo/load scripts`, plus remaining artifact/error cleanup in `crates/reasonbraid-server/tests/backup_restore.rs`.
- Goal and acceptance: Protect restore targets and secrets, use atomic restrictive backups, validate identifiers and quoting, verify HTTP status and negative controls, avoid fixed-port/output collisions, reap jobs, and ensure requested load counts and honest demo evidence.
- Verification: pending; capture the failing case, corrected case, and independent control in this leaf or its children before closure.
- Commit: pending.

### SIGNOFF-REPAIR.11.4 — Documentation containment and historical claims

- Status: `pending`.
- Sources / owned surfaces: `live docs, book, task records, external ledger, CI`.
- Goal and acceptance: Partition oversized live status/history, review adopted containment requirements, reconcile all phase/gate claims with measured behavior, refresh dependency evidence, and make pre-push CI discover every required live suite without counting skips as passes.
- Verification: pending; capture the failing case, corrected case, and independent control in this leaf or its children before closure.
- Commit: pending.

### SIGNOFF-REPAIR.12 — Requalification and return to roadmap

- Status: `pending`.
- Sources / owned surfaces: `all corrective leaves, PHASE-8.5.3/.5.4/.6, PHASE-9`.
- Goal and acceptance: Close or refute every census finding with reproducible evidence and owning leaf, run broad required gates before push, update qualification limits, then resume store-and-forward, export/import, G8 and later executable roadmap work. External judgment gates remain explicit, never self-certified.
- Verification: pending; capture the failing case, corrected case, and independent control in this leaf or its children before closure.
- Commit: pending.

## Current commit acceptance — SIGNOFF-REPAIR.3.2.1

- [x] **ROOT CAUSE (WHY + WHERE)** — the owned legacy-server probe returned rc=0 after observing four shared writes return 200 from two tenant admins, including adapter/region writes after tenant-boundary revocation. SQL witness `1|2|2|1` is explained in the leaf. The helper queries only tenant grants and performs no boundary or audit check; no distinct site authority exists at baseline.
- [x] **ADDRESSED (verified)** — the owned `site_authority` suite passed all 10 tests, rc=0, including tenant-only refusal with unchanged state, actual-parent liveness, human/role receipts, audit rollback, grant/boundary revocation in both lock orders and expiry during a wait. The prepared operator-membership regression changed from an observed allowed second issuance after REVOKE (test rc=101) to an audited denial with one boundary, rc=0. CLI and HTTP integration remain `.3.2.2`/`.3.2.3`.
- [x] **NO REGRESSION** — the five-suite command recorded above passed 36 tests, rc=0. `python3 -B scripts/project_env.py cargo clippy --offline --locked -p reasonbraid-server --lib --test site_authority -- -D warnings` passed, rc=0 (28.02s); `cargo fmt --all --check`, `make book`, rendered site/authority/qualification content inspection and `git diff --check` passed, rc=0. The final staged doctrine gate runs in the commit hook.
- [x] **FIX / LOCKSTEP** — schema/service, protected issuance, immutable records, transactional audits and owned live controls are implemented; the corrected five-suite run stopped/removed its cluster, rc=0. Documentation distinguishes the implemented service from pending CLI/HTTP enforcement, with no production qualification advance. The stale-role-query lesson is promoted to its indexed decision; the core serde defect has concrete `.3.3` ownership.

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `SIGNOFF-REPAIR.3.2.2` | `pending` | expose the verified service through protected operator tooling |
| 2 | `SIGNOFF-REPAIR.3.2.3` | `pending` | enforce site authority on every shared registry HTTP route |
| 3 | `SIGNOFF-REPAIR.3.3` | `pending` | bind the actual boundary, select usable grants and serialize authority/effect audit |
| 4 | `SIGNOFF-REPAIR.3.4` | `pending` | delegation bounds and cached-decision freshness |

## Evidence routing

The census is partitioned under `docs/tasks/artifacts/signoff_review/`. Each record links to this tree's concrete repair leaves. Findings crossing several subsystems remain here until their shared mechanism is proved; no runtime reproduction is inferred from routing. Historical phase closures retain their original provenance and gain correction pointers.

## Blockers

None for the current documentation and repair work. G6/G7 external review, public-name clearance and license decisions remain their existing director/external-owned gates; they do not prevent local repairs.

## Verification Log

- `SIGNOFF-REPAIR.1`: startup full read completed; `git status --short --branch` returned only `## main...origin/main [ahead 269]` before creation. Documentation validation completed below.

## Leaf .1 closure evidence

- **REPRODUCE / ROOT CAUSE:** full source read at `9c2d2ba`; the registry helper selects an any-tenant admin grant without a boundary check. Runtime confirmation is explicitly pending `.3.2`. The live-document census measured 42,374 bytes / 18 lines and zero distinct Phase 5/6/7/9 rows (`python3 -B` over `Path.read_text().splitlines()` and row searches, rc=0).
- **FIX / ADDRESSED:** source notes preserved as 131 review records (including general caveats, not 131 confirmed defects), with references resolving to 35 repair leaves; no missing owner IDs. The new live snapshot has all ten phase rows, 33 lines / 3,414 bytes. In-memory negative controls removing a referenced owner and a phase row were detected (`doc controls: owner omission detected; phase omission detected; generated authority and qualification pages present; rc=0`).
- **NO REGRESSION:** `git diff --check` rc=0; `env TMPDIR="$PWD/target/doctrine_scratch/commit" mdbook build docs/book` rc=0, HTML generated; the generated authority and qualification pages were inspected for their expected content. `env TMPDIR="$PWD/target/doctrine_scratch/commit" bash scripts/check_doctrines.sh` printed `=== all doctrines green ===` (13 checks, rc=0). Commit hook rechecks the staged scope. Rust/runtime tests were not run: this leaf changes docs and navigation data, not product implementation.
- **LOCKSTEP:** roadmap security-correction pointer, programme and task indexes, Phase-8 correction, MEMORY, LIVE_STATUS, README navigation, book progress/authority/qualification, CHANGELOG, DEV_NOTES and decision index updated; Knowledge Map regenerated. README 49 lines / 1,914 bytes; MEMORY 22 lines / 2,230 bytes at validation. Source index holds the baseline digest and historical status reference.
- **Locality:** book output and doctrine scratch are under the repository, whose device ID and target device ID both measured `16777244`. Required installed mdBook executable `/opt/homebrew/bin/mdbook` was accessed read-only as a toolchain dependency; its output was repository-local. No Cargo/home cache access, provider calls or database runs occurred.
- **Policy review:** CLAIM_VERIFICATION matched the director-authorized donor at startup; README policy was already locally adopted and reviewed against its donor. Remaining containment/enforcement gaps are owned by `.11.4`; no automatic donor synchronization or cap increase occurred.

## Commit Log

- `SIGNOFF-REPAIR.3.2.1`: `REASONBRAID-REPAIR-0006 (leaf SIGNOFF-REPAIR.3.2.1): add explicit site authority and atomic registry auditing`.

- `SIGNOFF-REPAIR.3.1`: `REASONBRAID-REPAIR-0005 (leaf SIGNOFF-REPAIR.3.1): bind revocation to the authorized tenant before mutation`.

- `SIGNOFF-REPAIR.2.2.2`: `REASONBRAID-REPAIR-0004 (leaf SIGNOFF-REPAIR.2.2.2): require disposable ownership before database fixture writes`.

- `SIGNOFF-REPAIR.2.2.1`: `REASONBRAID-REPAIR-0003 (leaf SIGNOFF-REPAIR.2.2.1): supervise disposable PostgreSQL tests and focused suites`.

- `SIGNOFF-REPAIR.2.1`: `REASONBRAID-REPAIR-0002 (leaf SIGNOFF-REPAIR.2.1): localize command stores and verify Cargo cache seeding`.

- `SIGNOFF-REPAIR.1`: `REASONBRAID-REPAIR-0001 (leaf SIGNOFF-REPAIR.1): record corrective census and site authority decision`.
