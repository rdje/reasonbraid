# Deployment — local and LAN

The Phase 1 deployment package (`.1.7.2`) is **four self-contained binaries**
and a runbook. This chapter is the product view; the operator surface is the
`deploy/README.md` runbook in the repository.

## Repository-local development and verification

Development now requires Python 3.11+ and an installed toolchain matching
`rust-toolchain.toml`. Makefile build/check/dev/book commands use
`scripts/project_env.py` to derive writable stores from the current repository.
Build output stays in `target/`; Cargo packages, temporary files, XDG data and CLI
state live under ignored `.project-data/`. A symlink or volume escape in those
store paths is refused. Moving the checkout does not require editing saved paths.

Inspect the selected paths or run a focused command:

```bash
python3 -B scripts/project_env.py --print
python3 -B scripts/project_env.py cargo test --locked --offline -p reasonbraid-core
make book
```

To initialize a local Cargo cache from a complete existing cache:

```bash
python3 -B scripts/project_env.py --seed-cargo-cache "$HOME/.cargo"
```

That explicit source is read-only. Locked archives are hash-checked before use;
credentials and global Cargo configuration are not copied. Shared source data is
retained. If the source is incomplete, use the launcher with `cargo fetch --locked`
when network access is available. Installed compiler, Python, database and mdBook
binaries remain documented read-only toolchain inputs.

Invoke direct diagnostic scripts through the launcher too. The PostgreSQL
runner enters it automatically; see the focused verification commands below.
The launcher controls default stores; explicit output paths and worker-specific
storage remain subject to the same-volume policy. It is not a filesystem sandbox.
Details and verification: `docs/decisions/2026-09-09_repository-local-command-environment.md`.

## Focused PostgreSQL verification

```bash
bash scripts/run_pg_tests.sh --list
bash scripts/run_pg_tests.sh authority command_api
bash scripts/run_pg_tests.sh site_registry_http allowlist regions
# Broad collection, without the demonstration:
RB_DEMO=0 bash scripts/run_pg_tests.sh
# Add the demonstration to a focused run:
bash scripts/run_pg_tests.sh authority --demo
```

Install PostgreSQL 16 first. The runner finds its binaries through PATH or
Homebrew; PG_BIN can select an installed directory. It ignores caller
DATABASE_URL and connection overrides, creates a unique database under
`target/pg-tests/run-*`, and verifies the server belongs to that run before
creating fixtures. Database creation rechecks that identity on its own connection,
so a changed server cannot receive the creation command. An available loopback port is selected automatically;
`--port 55432` requests a particular free port. The old PG_PORT variable is no
longer used. Suites run sequentially with one test thread to protect shared
fixtures.

Success stops the server, reaps its process group and removes the workspace.
Command output is saved to numbered logs and printed when that command exits.
Failure retains those logs, `postgres.log` and `runner.json` for diagnosis; `stopped` means
shutdown was verified, while `shutdown-unverified` requires process inspection
before cleanup. Ctrl-C also stops owned processes. Forced termination cannot
run cleanup, so inspect the receipt and server metadata before removing residue.
This is a trusted-host test service using loopback trust authentication and
synthetic data.

Database-backed tests now require the runner's live ownership receipt and exact
endpoint. Merely setting DATABASE_URL makes them refuse before migration or
cleanup. They also check the server's data directory, database and owner marker
on each new pooled connection, including replacement connections. The `pg_guard`
suite exercises these refusals. With DATABASE_URL unset, the existing offline
skips remain available; clear an inherited DATABASE_URL before an offline run.

The runner supplies a private matching synthetic passfile under its owned cluster
and explicit loopback defaults. SQLx can fall back to the home passfile after a
missing or unmatched custom file, so the former nonexistent placeholder did not
establish locality. The operator CLI uses explicit options with passfile lookup
disabled; the live CLI control verifies the runner's selected fixture credential.

```bash
bash scripts/run_pg_tests.sh pg_guard authority
# Offline checks, with no database target inherited from a development shell:
env -u DATABASE_URL make check
```

The PG CI job now uses this runner and the same suite list. Compiler installation,
packages and scratch stay under the repository; installed PostgreSQL and rustup
are read-only tool inputs. GitHub execution is verified on the next push.
Detailed contracts: `docs/decisions/2026-09-09_disposable-postgresql-runner.md` and
`docs/decisions/2026-09-09_disposable-test-pool-ownership.md`.

## Full checkpoint before pushing

Ordinary slices run focused checks and commit. Before a push, the scheduled
checkpoint also runs workspace format/strict lint/tests, the full owned PostgreSQL
collection with explicit `--demo`, Python runner/environment controls, doctrine
checks, fresh dependency checks, redacted Git-history scanning and this book build.
Build the worker binaries first and inspect test output: a missing browser or
extraction worker can cause an early return, and offline database skips do not
qualify live database behavior. Ignored Codex/Claude provider runs require separate
live-provider qualification; the schema-golden writer is deliberate regeneration.

The current source census finds 38 server suites plus MCP and CLI in the owned
runner (40 commands), and 86 Cargo test-enabled targets across 12 packages. These
are inventory counts, not passing-test counts. At the census commit, workflow
locality/Python coverage and publisher/browser fixture lifetimes still need their
tracked checkpoint repairs before broad execution. No fresh full-CI or release
claim follows from the inventory. See `docs/ci.md` for exact commands and
`docs/tasks/artifacts/signoff_review/ci-checkpoint-census.md` for source evidence,
tool boundaries and the concrete repair sequence.

CI now has a shared setup launcher:

```bash
python3 -B scripts/ci_env.py --rust -- cargo fmt --all -- --check
```

It establishes repository stores before optional pinned compiler installation and
execs the command from the checkout root. Installed rustup is read-only; new
compiler files stay under .project-data/installed-toolchains. Installer failure,
timeout or terminal cancellation prevents dispatch and consumes child cleanup.
CI clears inherited database/provider/scanner and selected compiler overrides;
ordinary developer project_env behavior is unchanged. Explicit output paths still
need their command's storage checks. Eight focused and eighteen adjacent controls
pass, using an instrumented installer. Workflows now use the launcher; actual
remote compiler installation and full GitHub execution remain pending. The exact contract and evidence are in
`docs/tasks/artifacts/signoff_review/ci-environment.md`.

Scanner installation now has an explicit verified path:

```bash
python3 -B scripts/project_env.py python3 -B scripts/ci_scanners.py cargo-deny --verify-only
python3 -B scripts/project_env.py python3 -B scripts/ci_scanners.py gitleaks --verify-only
```

These commands verify pinned release bytes, bounded executable extraction and the
actual version; they do not run security gates. Omit --verify-only when the full
checkpoint reaches actual dependency/history scanning. Each invocation owns a
directory under target/ci-scanners. It retains logs, redacted reports and a
scanner.json receipt; read both scope and exit_code. A completed operation can
still have a nonzero scanner result. Successful consumed setup/execution retires
only its archive and executable; failed setup retains diagnostic files and its
last recorded child identity for inspection.

Thirteen focused controls and final affected controls pass. All eight pinned
Linux/macOS architecture archives pass identity/layout checks; actual version
execution is verified on aarch64 macOS. The native probe caught and corrected a
Gitleaks version-format mismatch that instrumented tests missed, with the original
failure retained. Workflow wiring is complete; full local/remote gates remain pending.
See `docs/tasks/artifacts/signoff_review/ci-scanners.md` for exact versions,
limits and evidence. Installation integrity does not complete release qualification.

## CI requires its runtime prerequisites

All six project command jobs enter the local launcher. Rust checks pin Chrome for Testing,
build the workspace binaries and verify executable extraction/browser workers
before tests. Missing prerequisites fail instead of supplying apparent coverage.
The PostgreSQL job validates installed version-16 tools, discovers every Python
control module and runs the full owned collection with explicit `--demo`. The book
job installs pinned mdBook 0.5.4 into .project-data/cargo using a local build tree,
checks its version and builds this book. Strict shell errors and bounded jobs
preserve failures. Provider tests remain separately authorized qualification.

The supply-chain jobs run the actual pinned scanner gates. CI retains their
receipts, version/check logs and redacted Gitleaks JSON report; binaries and archives
are excluded. A real synthetic-history probe detected one deliberately unissued
value and fully redacted it from report and output. The initial alphabet example
was excluded by the scanner's example filter; its result was investigated before
correcting the fixture. No repository-wide secret-free claim follows.

Three YAML workflows, six shell command blocks, five deliberate omission controls
and fifty Python tests pass locally, including actual PostgreSQL ownership checks.
GitHub's managed checkout/artifact transport and installed OS tools are explicit
platform dependencies; project stores and artifact temporary paths derive from the
checkout. Workflow, publisher/browser and compiler-artifact prerequisites are now
complete; full Rust/security gates and actual remote outcomes remain pending. See
`docs/tasks/artifacts/signoff_review/ci-workflows.md` for commands and exact evidence.

## Historical scanner exceptions

The history scanner retains its normal rules. Two exact fingerprints in
`.gitleaksignore` identify predictable literals used only by historical local
metadata tests. Source and data-flow inspection establish that they were never
issued credentials or used for connections. Each exception names the immutable
commit, file, rule and line; it does not exclude the file or future changes.

Five native controls verify that removing either entry restores that finding,
using both gives a clean configured history result, and committing identical
content again still produces both findings. An alternate ignore path does not
replace the source-root policy in the pinned scanner, so omission checks use an
isolated copy of the exact history. Requalify these boundaries when changing the
scanner version or exceptions. The full configured history scan still runs before
push; it does not scan uncommitted files or prove that every possible secret is
absent. See `docs/tasks/artifacts/signoff_review/history-fixture-fingerprints.md`.

## Publisher verification owns its directories

The offline publisher tests create private, exclusive directories under
`target/publisher-tests`, after checking the parent components remain directories
on the repository volume. They never remove a preexisting directory to start a
test. A directory collision refuses. Repository readers and handles close before
explicit cleanup, which checks the original directory identity and confirms its
removal. An assertion failure or unfinished fixture retains its data and prints
its location for diagnosis. Inspect the owning test/process before removing it;
old `pub-N` directories may belong to historical work.

The old counter-based helper was reproduced in two isolated processes: the second
removed the first process's witness while that owner was still running. The new
helper's two-process control preserves both witnesses; finishing one owner removes
only its own directory. Fixture isolation improves verification reliability; it
does not change production publication or qualify its broader governance gates.
Exact checks and their results are tracked in
`docs/tasks/artifacts/signoff_review/publisher-fixtures.md`.

## Browser verification bounds its workers and origins

The browser integration harness gives each command a private on-volume fixture
under `target/browser-lifetime-controls`, including temporary, profile/cache and
bounded diagnostic data. Each worker starts in its own process group. Input,
stdout, stderr and exit share a deadline; each output stream has a 2 MiB ceiling
plus one byte to detect overflow. After any result the harness consumes the worker
and confirms group absence, escalating bounded shutdown if needed. Failed tests
or unconfirmed cleanup retain their directory and receipt for inspection.

A native macOS control shows that a group containing an exited, unreaped child
can transiently refuse inspection with `EPERM`. The harness waits only within a fixed bound and requires
an actual absence observation; persistent denial stays an error. Refused signal
requests never qualify cleanup. The initial eight-control harness qualification passed with strict focused lint;
the original failed fixture is retained with its unconfirmed receipt.

Local HTTP origins have explicit graceful shutdown; the serving task is consumed
before successful fixture removal. Budget admission always runs, even without a
browser. The render test reports an absent browser as unqualified; an explicitly
configured invalid browser is an error. The CI workflow requires browser presence.

The original worker left a real renderer running after it returned. The production
worker now owns Chrome before its first asynchronous launch wait, uses private
per-invocation storage and consumes process/task shutdown before returning a
successful result. The harness independently verifies browser-group absence; its
own emergency cleanup still cannot count as production-owned shutdown. Exact
baseline and controls are in
`docs/tasks/artifacts/signoff_review/browser-production-lifetimes.md`.

The earlier combined qualification passed fifteen integration controls with unchanged
production worker bytes, alongside the separately recorded five production unit
controls. Two real renders succeed across a same-volume runtime-root rename; the
original directory identity and a witness survive, while completed invocation
storage is removed. A linked parent refuses before Chrome startup and preserves
its target. Overlap is held by an explicit origin gate until both live groups and
profiles are observed, including a deliberately delayed second launch.

Origin shutdown is checked by identity. A native control demonstrates that a new
listener can reuse a closed listener's port and even its numeric file descriptor;
a later successful connection does not prove the original listener survived. The
harness consumes the original socket's close receipt as well as its serving task.
The original failed connection's peer was not captured, so its exact cause remains
unstated. All twenty-six final worker/browser groups are independently absent;
the nine earlier failed fixtures remain preserved. Exact results and remaining
boundaries are in `docs/tasks/artifacts/signoff_review/browser-combined-qualification.md`.

### Browser timing and the selected test runtime

The later full checkpoint found that two timing assumptions were too short for
real startup: the navigation deadline could fire before any request reached the
origin, and the overlap test could abandon its second launch. Navigation now has
an explicit arrival witness and a response that remains gated until the worker
returns. For example, a six-second startup delay must still reach the origin before
a thirty-second render deadline; cleanup must finish before the refusal returns.
Overlap keeps worker one's response gated, delays worker two four seconds after
its arrival, and proves both distinct profiles/groups are live before release.
Missing requests, early responses and unconfirmed cleanup still fail qualification.

That stronger deadline witness exposed a separate runtime boundary. Desktop Chrome
started crash reporting/update helpers outside its process group; native pipe
observations showed them retaining stderr after that group stopped. The worker
returned cleanup-unconfirmed rather than claiming success. The observed helpers
later exited naturally. A dedicated repository-local Chrome for Testing 153.0.8010.36
passes the delayed navigation/overlap witnesses and all sixteen integration controls
with the unchanged production worker. The archive and extracted bytes are verified;
this testing build has an ad-hoc linker signature, not verified Developer ID signing.

The result qualifies these trusted loopback fixtures. The pinned launcher below
now supplies the local/CI browser prerequisite under .11.4.3.1.2.5.
Untrusted-content isolation, detached
process containment and aggregate resource limits remain .7.3.2. The source-7e01097
attempt did not pass workspace tests or start PostgreSQL/demo. Exact
results, retained failures and scope: `docs/tasks/artifacts/signoff_review/browser-checkpoint-timing.md`.

### Reproduce tests with the selected browser

`make test` builds the workspace binaries with the lockfile and runs the workspace
tests using Chrome for Testing **153.0.8010.36**. `make check` runs format and strict
lint first. The Rust CI job uses the same browser launcher. Linux and macOS on
x86-64/ARM64 have exact archive size/SHA-256 pins; other platforms refuse.

Both workspace runs pass `--no-fail-fast`. Cargo's default stops at the first
failing test binary, which reached 11 of 94 here and left four crates unmeasured;
`--no-fail-fast` reaches all 94. A command whose job is to report the workspace's
state cannot report it from 11 binaries. A crate-scoped run during development
keeps the default early stop.

Both also build the test targets with `cargo test --all --locked --no-run`
*before* entering the browser launcher, and give the launcher `--timeout 5400`.
The launcher's deadline should bound test **execution**, not a compile: a cold
tree once spent so long compiling inside it that the run was cut off at 78 of 94
binaries, while the same work on a warm tree executes in **1,816.78 seconds**.
The launcher's own default stays 3600 seconds, which is the right bound for the
crate-scoped invocations below — those finish in about 31 seconds.

A completed workspace run under the pinned runtime measures **103 suites, 816
passed, 0 failed, 3 ignored** across 94 test binaries and 9 doc-test targets, in
`real 30m15.757s`. Acquiring the browser is 55.27 seconds of that — download
51.54 s for 191,016,009 bytes, extraction and the binary's SHA-256 0.63 s, the
version check 1.86 s — about 3% of the run, which is why each call keeps its own
verified download rather than a cache.

```bash
# Verify download, installation and version only.
python3 -B scripts/project_env.py python3 -B scripts/ci_browser.py --verify-only
# Exercise only the real browser integration tests after building workers.
python3 -B scripts/project_env.py cargo build --workspace --bins --locked
python3 -B scripts/project_env.py python3 -B scripts/ci_browser.py -- cargo test -p reasonbraid-browse --test browser_roundtrip --locked -- --nocapture
```

Each call requires network access to the official pinned archive. The launcher
creates a private `target/ci-browser/<platform>-*` directory on the repository
volume, verifies the download before extraction, validates internal framework
links and checks the executable's exact version before dispatch. It overrides
ambient `R3_BROWSER_BIN`; there is no desktop fallback. Direct Cargo commands
bypass this setup and do not establish pinned-runtime coverage by themselves.

That last sentence is now enforced rather than advised. The browse test support
reads `R3_BROWSER_BIN` and nothing else, so a Cargo command that names no runtime
**skips** the six real-browser controls instead of qualifying them against
whatever browser the host happens to have installed. Each skip prints one line
naming what went unqualified and the command that qualifies it, through a channel
the test harness does not capture, so a skipped control cannot be mistaken for a
passing one. The remaining eleven controls in that suite — including the injected
escaped-writer refusal and the stalled-launch deadline — need no browser and
always run.

The production worker's own discovery is unchanged: a deployment runs the browser
its host provides. Only the test harness refuses to guess.

Download/version waits are bounded to 300/30 seconds; the command default is one
hour. For a shorter selected run, insert `--timeout 120` before `--` for a two-minute
command deadline. Shutdown is consumed afterward. The receipt and version log
remain; successful invocations retire their own archive/runtime, while failed
invocations preserve them. Inspect `browser.json` for the selected executable's
root-relative path, archive/executable hashes and exact child phases. A `started`
phase without a consumed result requires inspection, never an assumption of success.

Setup verification is distinct from real rendering. Archive integrity is distinct
from vendor signing, and this test runtime does not establish production isolation
for hostile pages. Full bounds, refusal behavior and qualification evidence:
`docs/ci.md` and `docs/tasks/artifacts/signoff_review/ci-browser-runtime.md`.

### Current checkpoint and database fixture ordering

The later **b0cddfe** checkpoint passes nine local gates, including workspace tests
with the pinned browser, Python controls and both scanners. Its live PostgreSQL
run passes thirteen suites, then fails during `identity_store` fixture cleanup:
a certificate left by node-work tests still references the node being deleted.
The following twenty-six commands and the crash/reconnect demo do not run.
The stopped failed database and its logs are retained; no full checkpoint or
remote-CI success is claimed.

The identity fixture now removes `node_certificates` before `nodes`. A regression
first proves that the real foreign key refuses deleting a referenced node, then
checks that fixture cleanup removes the complete identity hierarchy. All four
identity tests pass both on a fresh database and after all eight node-work tests.
The deployment CA remains intact. Production constraints and API behavior are
unchanged.

These two commands exercise different preconditions:

```bash
# Fresh database: all four identity tests.
python3 -B scripts/project_env.py bash scripts/run_pg_tests.sh identity_store
# Same owned database: real node-work residue, then the identity tests.
python3 -B scripts/project_env.py bash scripts/run_pg_tests.sh node_work identity_store
```

A fresh-only pass missed the original defect. The initial node-plan census found
MCP-listener gaps in fourteen fixtures and a CLI spend-breaker gap. The expanded
census finds dependency candidates in twenty of twenty-five explicit cleanup
plans. Actual producer/consumer tests reproduce listener-state, spend-breaker and
incarnation failures. These are concrete fixture failures; the source/catalog
edge count is not a count of independently reproduced product defects.

A shared test-only checker now validates the entire declared table plan before
any deletion. For example, an otherwise complete tenant cleanup must name its
listener-state dependency even when that table is empty. Dependent tables must
precede parents, including existing cascading/null/default effects. Unsupported
table shapes, cross-schema dependencies and cycles refuse; unrelated rows stay
outside the declared plan. Callers must use the existing ownership-verified pool
and exclusive fixture access. A later SQL error retains its cause and may leave
earlier deletes committed; this helper does not promise atomic rollback.

```bash
# Five live plan controls plus the three existing ownership controls.
python3 -B scripts/project_env.py bash scripts/run_pg_tests.sh pg_guard
```

All eight checks pass. Fourteen node-fixture plans now use the checker, including
the identity and CLI fixtures. They name `mcp_listen_state` before `tenants`; the
CLI also names `spend_breakers`. Existing resource cascade effects are now explicit:
`claim_assessments` and `derivations` precede `evidence_snapshots`, which precedes
`resource_references`. Each fixture keeps its prior table order and unrelated
state policy, including the identity fixture's preserved deployment CA.

```bash
# Real listener residue followed by the repaired identity fixture.
python3 -B scripts/project_env.py bash scripts/run_pg_tests.sh mcp_listen identity_store
```

The real listener→identity and selected spend-breaker→CLI sequences pass. The
broader caller run exposed a separate application defect: participant removal
returned 403 for an authorized administrator because the handler supplied a thread
target to the tenant-administration action. The exact original fixture reproduced
that refusal on a fresh database. `SIGNOFF-REPAIR.11.4.3.1.2.8` now corrects the
server target; all six invitation tests and adjacent authority/command tests pass.
The companion CLI scope correction is `.2.10`: removal requests administrative
scope while ordinary thread delegation stays narrow. The historical fixture census
is preserved: all fourteen cleanup callers completed and twelve whole suites passed;
no complete affected-suite or full-checkpoint pass is claimed. Profiles exposed
a separate fixture-clock failure: its fixed expiry date preceded newly created
snapshots, so the expected tombstone was not due. Original-fixture reproduction
and database-time predicates confirmed that mistake. The `.2.9` repair derives
cutoffs from recorded creation times, as described below. The six partial plans
under `.2.7.3` now declare their required dependency tables explicitly. Remaining
adoption/coverage `.2.7.4` precedes the full checkpoint.
The MCP producer above is an internal durable-state test, not MCP-wire or agent
qualification. Native executable startup delays have separate diagnostic ownership
under `.11.2`. Evidence: `docs/tasks/artifacts/signoff_review/identity-fixture-cleanup.md`,
`docs/tasks/artifacts/signoff_review/fixture-cleanup-plan-check.md` and
`docs/tasks/artifacts/signoff_review/node-fixture-cleanup.md` and
`docs/tasks/artifacts/signoff_review/participant-removal-authority.md`.

The six formerly partial fixtures are cards, quota, classification, mcp_listen,
federation and quarantine. For example, removing a role requires first removing
incarnations that reference it; removing a node requires its certificate, key and
lease children first. Each fixture now names those required dependencies, keeps
its original table order and leaves the deployment CA outside its deletion list.
The checker verifies the whole declared plan before the first deletion; it does
not silently add tables to the requested scope.

The source census covers the original 25 literal-array DELETE plans. All 25 now
use checked cleanup, including regions, allowlist, rls, mcp_write and the MCP
crate's internal fixture. Four of these final five plans already deleted resource
evidence through CASCADE; they now explicitly name claim_assessments, derivations
and evidence_snapshots before resource_references. Their existing deployment-CA
deletion policy stays intact. RLS keeps its five-table event/outbox scope; it does
not gain tenant or node deletion. Every original feature assertion stays intact.
Other direct SQL shapes and relationships without FKs are outside this census.
The helper is test-only, including its MCP import; this does not qualify MCP wire
transport or an AI agent workflow. Reproduction and coverage evidence:
`docs/tasks/artifacts/signoff_review/fixture-plan-coverage.md`.

For example, exercise the final consumers against actual predecessor residue:

```bash
python3 -B scripts/project_env.py bash scripts/run_pg_tests.sh node_work rls regions allowlist mcp_write mcp
```

The runner uses an owned disposable database. Its cleanup validates each fixture's
explicit scope even when dependent tables are empty. Keep a failing database and
its receipts for diagnosis; a passing selected run does not replace the complete
pre-push checkpoint. Earlier partial-plan failure baselines remain recorded in
`docs/tasks/artifacts/signoff_review/partial-fixture-cleanup.md`.

### Retention fixture time

The retention/freshness test uses each snapshot's stored creation time. It no
longer assumes a fixed calendar date lies beyond a newly created row's lifetime.
The existing retention behavior exercised in the disposable database is:

| Class | Automatic expiry threshold measured from creation |
| --- | --- |
| temporary | More than one day; exactly one day remains live. |
| standard | More than thirty days; exactly thirty days remains live. |
| audit | No automatic TTL in the current implementation. |

For example, a temporary snapshot created at instant T remains live when the
fixture submits T + one day. Submitting one microsecond later tombstones that row;
repeating the same expiry request changes no rows. The standard snapshot remains
live until its own thirty-day threshold is passed. Audit state remains unchanged
through both finite expiries and the later observation.

A tombstone retains the snapshot metadata and records its deletion reason.
The existing same-content replay updates refreshed_at while retaining created_at;
it does not reset retention age. Freshness horizons are a separate field. The
focused fixture preserves its freshness-list, license and replay assertions and
checks exact rows/counts at expiry boundaries.

Run these controls in the owned disposable PostgreSQL environment:

```bash
python3 -B scripts/project_env.py bash scripts/run_pg_tests.sh profiles
```

The boundary controls above drive the sweep directly rather than over HTTP,
because the caller's clock is no longer a wire field: expiry authority, scope and
the caller clock were repaired under SIGNOFF-REPAIR.7.4.3 and are described in
"Who may run the retention sweep" below. Every assertion these controls make is
the one they made through the route. Actual freshness-horizon refresh and object
retirement remain open under SIGNOFF-REPAIR.7.4. Evidence:
`docs/tasks/artifacts/signoff_review/retention-fixture-clock.md`.

### Who may read an evidence snapshot

An evidence snapshot is one row shared by every tenant that acquired the same
bytes. `resource_references` is unique on `(original_locator, expected_digest)`
and `snapshot_objects` is keyed by digest alone, so two tenants citing the same
URL at the same digest hold the same snapshot: that is the content-addressed
design, not an accident, and it is why the row carries no owning tenant.

What names the tenant is the decision to disclose the row. `evidence_citations`
records which tenants cited which snapshot, and the server writes a citation
every time a snapshot is acquired through `POST /v1/snapshots` or through a
resolver pack — on a replay as well as on a first acquisition. Five surfaces
read that citation:

| Surface | A tenant that cited the snapshot | Any other enrolled tenant |
| --- | --- | --- |
| `GET /v1/snapshots/stale` | the tenant's own stale rows | those rows are absent from the list |
| `GET /v1/snapshots/{id}` | 200 with the row | 404 |
| `GET /v1/snapshots/{id}/derivations` | 200 with the children | 404 |
| `GET /v1/snapshots/{id}/assessments` | 200 with the tenant's OWN assessments | 404 |
| `DELETE /v1/snapshots/{id}` | tombstones the row | 404 |

A tenant that did not cite a snapshot receives 404 rather than 403. The two
answers are deliberately indistinguishable: a refusal that separated them would
confirm that an identifier exists, which is the enumeration the binding closes.

A principal that is enrolled nowhere is refused earlier and differently: a
well-formed but unknown principal receives **403** `unauthorized` from the
enrolment gate, and a principal whose identifier is not of the right shape
receives **401** `unauthenticated` from the header parser, before any gate
runs.

Recording the citation on the write is what makes the read safe to narrow. If
the reads had been filtered without it, a tenant that submitted a snapshot
someone else had already acquired would have been refused its own evidence.
Instead the replay records the second citation, both tenants read the shared
row, and a count of `evidence_citations` for that snapshot returns 2.

### Who may read an assessment

An assessment is an authored opinion rather than a shared receipt, and the
schema already says so: `claim_assessments` replays on
`(claim_id, snapshot_id, assessment, author)`, so two tenants asserting the same
thing about the same evidence hold two separate rows. It therefore carries a
server-recorded `authored_by_tenant`, and both assessment reads return only the
rows the reading tenant wrote:

| Surface | Returns |
| --- | --- |
| `GET /v1/snapshots/{id}/assessments` | the reader's own assessments of a snapshot it cited; 404 if it did not cite the snapshot |
| `GET /v1/claims/{claim_id}/assessments` | the reader's own assessments of that claim, each labelled with its `claim_namespace` |

The claim-keyed read has no citation gate and cannot have one — a claim spans
snapshots and the route names none. That matters because `claim_id` is
caller-supplied text the server never mints: there is no claims table, and the
identifier is whatever the submitter typed. Before this binding the route
answered any enrolled principal for any identifier it could guess. The authoring
gate is what makes a guessed identifier useless; which writer minted a given
identifier is recorded on the row itself, and is described under
[the two assessment namespaces](#the-two-assessment-namespaces).

Two tenants that cite the *same* snapshot each read their own assessments of it
and not the other's. Their `derivations` of it remain shared, because a
derivation is content-addressed in the way a snapshot is — that half is open
under the same leaf family.

⛔ `author` and `verifier` on an assessment are still unauthenticated caller
labels. The authorization never reads them; it reads the server-recorded tenant.
Making those fields trustworthy is open under `SIGNOFF-REPAIR.7.4`.

Two limits are published rather than implied:

- **Snapshots written before this binding have no recorded citer, so no tenant
  reads them.** The attribution cannot be recovered: the only actor column
  reachable from a snapshot is a reference's `submitted_by`, which stores a
  one-way `Uuid::new_v5` of the submitting subject that joins to no identity
  table, and which the locator replay leaves naming the first citer regardless.
  Re-acquiring such a snapshot records the citation and restores the read.
- **A shared row can still be tombstoned by any one of its citers**, which
  removes it from the others' staleness surface. Who may delete shared evidence
  is open under `SIGNOFF-REPAIR.7.4.4`. Who may run the site-wide retention
  sweep is settled below.
- **An assessment written before its binding has no recorded author tenant** and
  is likewise read by no one. There is nothing to recover it from: `author` is a
  caller label, not a principal.
- **An assessment written before the namespace column has no recorded
  namespace**, and nothing backfills one. Which writer produced such a row is not
  recoverable even in principle: the only evidence would be the *shape* of its
  `claim_id`, and a caller could always type the minted shape — which is the
  collision the column exists to record. The row keeps its `null` namespace and
  is read as unattributed.

Run the control in the owned disposable PostgreSQL environment:

```bash
RB_DEMO=0 bash scripts/run_pg_tests.sh profiles
```

`the_evidence_reads_are_bound_to_the_citing_tenant` drives two enrolled tenants
through all five surfaces, including the shared row both of them cite.

### Who may run the retention sweep

Retention is enforced by `POST /v1/snapshots/expire-due`, which tombstones every
live snapshot whose class TTL has passed. It is a **site-operator** act, not a
tenant one, and it requires the `evidence_expire` capability on an explicitly
issued site grant — the same machinery the shared adapter and region registries
use. Tenant enrolment, and tenant-administrator authority, convey none of it.

The reason is in the schema rather than in a policy preference: `retention_class`
is a column on the shared snapshot row, not on a citation, so *which rows are
due* is a site-wide fact that no single tenant owns.

The request carries a reason and no time:

```http
POST /v1/snapshots/expire-due
{"reason": "the scheduled retention sweep"}
```

| Caller | Answer |
| --- | --- |
| holds a live `evidence_expire` grant on its actual boundary | 200 `{"tombstoned": n, "swept_at": …}` |
| any other principal, enrolled or not | 403 `site_authority_required`, with the audit id |
| supplies an `at` field, or no reason | 400 |

The cutoff is the database's own clock, read inside the transaction that writes
the tombstones and the audit record together. It is not a caller input, and that
is the repair: the route previously accepted an unbounded `at`, so one enrolled
principal naming a far-future instant tombstoned every tenant's live `standard`
and `temporary` evidence — stamping each row `"the retention expired"` when it
had not, with nothing in the product able to clear `deleted_at` again.

Allowed and refused sweeps both leave a `public.site_audit` row naming the actor,
the action, the grant and boundary, the outcome and the time. A sweep that finds
nothing due records `noop` rather than nothing, so an operator can show the sweep
ran.

Issue the capability through the deployment-controlled operator tool, exactly as
any other site action — `--action evidence_expire` on the boundary and on the
grant bound to it. The full invocation, its environment and its prerequisites
are in [Site authority](site-authority.md).

### How a deliberation registers the evidence it cites

ROADMAP §13.2 step 2 is *"register context and resource references"*. A
contribution's `evidence_refs` is where that happens: each citation registers the
§12.1 resource reference it names, **in the contribution's own transaction**, and
the committed event carries the `resource_id` it resolved to.

```json
{
  "tenant_id": "...",
  "content": "the position this citation supports",
  "kind": "evidence_reference",
  "evidence_refs": [
    {
      "uri": "https://example.org/cited-report",
      "digest": "sha256:<64 hex>",
      "note": "the acquired report"
    }
  ]
}
```

The timeline then shows the citation *resolved*:

```json
"evidence_refs": [
  {
    "uri": "https://example.org/cited-report",
    "digest": "sha256:<64 hex>",
    "note": "the acquired report",
    "resource_id": "res_…"
  }
]
```

`resource_id` is the server's — it is not accepted on the wire, and the same row
is what `POST /v1/resources` returns for that locator and digest. Citing is not
acquiring: §13.2 registers at step 2 and acquires at step 6, so a citation of
something the network has not yet fetched is the flow working, not an error.

#### The key is the pair, not the locator

A reference's identity is `(original_locator, expected_digest)` — the key
`resource_references` has declared since it was created. Two consequences a
client can rely on:

| The request | The result |
| --- | --- |
| the same locator **and** the same digest, again | the **same** `resource_id`; `POST /v1/resources` reports `"replayed": true` |
| the same locator at a **different** digest | a **second** reference. §12.6's live page changed, and §12.1 forbids erasing that distinction |
| the same locator with **no** digest, twice | one unpinned reference, replayed the second time |

⛔ `locator_digest_conflict` **no longer exists.** Until
`SIGNOFF-REPAIR.11.14.3.2` the store refused a second digest for a locator, and
that refusal was wrong twice over: §9.8 requires that cross-tenant existence is
not leaked, and a 409 told the caller that somebody else had pinned that locator
to a digest they were never shown; and it meant the first principal to pin a
locator made that locator **uncitable by every other tenant**, in any form,
including without a digest. A client that branched on the code should treat the
citation as accepted.

#### What `scheme` is, and what it is not

`scheme` on a §12.1 reference is the **resolver-selection key** §12.2 ranks on —
not the locator's URI scheme. Two shipped packs make the difference concrete:

| Pack | advertises `schemes` | for locators matching |
| --- | --- | --- |
| `r1-git-fetcher` | `git` | `https://*` |
| the R3 browser pack | `web+render` | `https://*` |

A Git repository and a rendered page are both reached over HTTPS. The field is
how you ask for a **capability**, so `{"original_locator":
"https://…/repo.git", "scheme": "git"}` is correct, not a contradiction.

⚠️ **It is validated against nothing, deliberately.** §3.7: accepting a reference
is not a promise the core can resolve it, and unsupported references stay durable
for later. So a scheme no installed pack advertises registers fine and answers
`resource_unresolvable_now` at resolution — an explicit failure at the point where
resolution is actually attempted.

⛔ `SIGNOFF-REPAIR.11.14.3.5` tried to validate this field against its locator and
was **refused by the two packs' own controls**. The attempt is recorded because
the wrong reading is a natural one: a caller-supplied routing field that nothing
checks describes a defect and this design equally well, and only the consumer —
one `WHERE schemes @> …` lookup — tells them apart.

#### Omitted fields take the schema's declared defaults

`visibility_scope` defaults to `network` and `risk_class` to `low` — the values
`migrations/0023` declares.

⛔ They used to default to the **empty string**, because the typed field's
`#[serde(default)]` is `String::default()` and the store binds it explicitly, so
the column default never applied. A contribution's citation had always written
the declared values, so the two writers produced different rows for the same
omission. They now agree.

⚠️ `low` is the permissive direction, and it is adopted rather than chosen — it is
what the schema already declared. Nothing in the product reads either column to
make a decision yet; §12.2's risk filter, when it is built, owns whether `low` may
be a default at all.

#### A fragment stays in the locator

`…/page`, `…/page#a` and `…/page#b` are **three** references. §12.1 lists
`fragment_or_selector` beside `original_locator`, which invites the opposite
reading — but splitting the fragment out *is* canonicalization, and §12.1 says
canonicalization is scheme-specific and must not erase security-relevant
distinctions. Merging three locators onto one row is exactly such an erasure, so
the distinction is kept.

#### What a citation is refused for

| The citation | Refused because |
| --- | --- |
| `{"uri": "see the internal wiki"}` | a §12.1 reference names a URI **scheme**; the citation's scheme is parsed from its own locator (RFC 3986 §3.1), never claimed beside it |
| `{"uri": "https://…", "digest": "md5:…"}` | the digest is the ADR-011 `sha256:<64 hex>` scheme — the same rule `POST /v1/resources` applies |

Both are new: before this change a citation's `uri` and `digest` were free text
that nothing read. A contribution whose citation is refused commits nothing —
the event and every reference it registers share one transaction.

#### What a pin does

`expected_digest` is not a note. A reference that declares one **accepts only the
bytes it names**, and a snapshot of anything else is refused before it is written:

```text
400 the bytes do not match the digest this reference is pinned to — a page that
    changed is a SECOND reference (§12.6), so register the locator at the new
    digest and acquire against that
```

⛔ Until `SIGNOFF-REPAIR.11.14.3.6` the pin was recorded and read by **nothing**.
The snapshot store asked its reference table one question — does this
`resource_id` exist? — so a reference pinned to one digest accepted a snapshot of
entirely different bytes, and the field a caller supplied to say *"these are the
bytes I expect"* constrained nothing.

⚠️ **An unpinned reference is unchanged, and that is the shape of the rule rather
than an exemption.** A snapshot replays on `(reference_id, raw_digest)`, so one
reference holds many versions — which is what a living page needs. A pin says the
opposite about its own reference: these bytes, this row. The two compose because
the same locator at a different digest is a **second reference**, which is where
the changed page's snapshot belongs:

| The reference | What it holds |
| --- | --- |
| no `expected_digest` | every version acquired against it, over time |
| pinned to `sha256:…` | that one version; anything else is refused |
| the same locator at a different digest | a separate reference, holding the changed page |

⛔ **The refusal quotes neither digest.** You already have the one you sent, and
the pinned one belongs to a reference this route does not check you may read.

#### A resolution that could not store its evidence says so

`POST /v1/resources/{id}/resolve` answers with a receipt **only when the evidence
is there**. If the acquisition succeeded and the snapshot did not persist — the
commonest reason now being a pin the served bytes do not match — the resolution
carries the refusal instead:

```json
{"resolvers":["r0-https-fetcher"],"unresolvable_now":false,
 "acquisition_error":{"kind":"evidence_unstored","message":"…"}}
```

There is **no `acquisition` field** in that answer. A receipt describes a
document you can go and read; one returned beside an empty evidence store would
say the document was acquired while nothing was stored.

⛔ **Two of the three acquisition arms used to do exactly that.** The R2 arm has
reported `evidence_unstored` since `SIGNOFF-REPAIR.7.4.2`; the R0 and R5 arms
discarded the store's answer and set the receipt regardless.
`SIGNOFF-REPAIR.11.14.3.12` put all three on the same contract — the same `kind`,
so a client already handling it from R2 handles it from R0 and R5 unchanged.

⚠️ The acquisition itself still happened, and the server logs it
(`acquisition_evidence_unstored`, with the resource id and the reason). What the
caller no longer receives is a receipt implying evidence that is not there.

#### A snapshot names its reference's locator

A snapshot is an acquisition **of** the reference it is filed against, so its
`original_locator` must be the one the reference carries — byte for byte. A
submission that names a different document is refused:

```text
400 this snapshot names `https://example.org/other` and its reference names
    `https://example.org/report` — a snapshot is an acquisition OF its
    reference, so the two cannot disagree (`final_locator` is where a redirect
    ended and is free)
```

⛔ Until `SIGNOFF-REPAIR.11.14.3.13` the two were never compared: the field went
straight into the row, so a snapshot could say it was an acquisition of one
document while its reference named another, and §12.6's *"original reference and
resolved final locator"* were two facts that need not agree.

⚠️ **`final_locator` is deliberately free.** It is where the acquisition *ended*,
and a redirect legitimately moves it — that is why the snapshot records both.

⚠️ **The comparison is strict, not normalising.** §12.1 keeps the original
locator immutable and canonicalization separate and scheme-specific, so deciding
that two spellings mean the same document is a decision this product has not
taken. Refusing a disagreement takes none of it.

#### Who may file a snapshot against a reference

`POST /v1/snapshots` names a `reference_id`, and the reference must be one **this
tenant registered**. A reference it did not register answers exactly what an
absent one answers:

```json
{"code":"invalid_command",
 "message":"the reference does not exist, or this tenant did not register it"}
```

One sentence for two cases, deliberately — separating them would confirm that a
`res_…` id exists.

⛔ **Until `SIGNOFF-REPAIR.11.14.3.11` this route admitted any enrolled
principal, and the consequence was a write rather than a read.** Measured with a
second tenant against a reference it had never registered, the submission
**succeeded**: a snapshot was attached to another tenant's reference. Nothing
could list it — no route returns a reference's snapshots — so the attachment was
invisible to the reference's own registrants.

⚠️ **The supported path is the one the submission already carries.** A snapshot
submission names its `original_locator`, so a caller that can make one can
register that locator, receive the **same** reference id back, and file. The
registration is recorded on the replay, exactly as a snapshot re-acquisition
records the second citation.

#### Who may read a reference

The reference ROW is shared, exactly as a snapshot's is:
`UNIQUE (original_locator, expected_digest)` means two tenants citing one URL at
one digest hold the same row. What names the tenant is the decision to disclose
it — `reference_registrations` records which tenants registered which reference,
written on the replay as well as on the first registration.

| Surface | A tenant that registered the reference | Any other enrolled tenant |
| --- | --- | --- |
| `GET /v1/resources/{resource_id}` | 200 with the §12.1 row | **404**, the same answer an absent id gets |
| `POST /v1/resources/{resource_id}/resolve` | resolves and acquires | **404** |
| `POST /v1/resources` with the same locator and digest | `replayed: true`, the same `resource_id` | **the same** `replayed: true` — and it records the second registration |

⛔ **Until `SIGNOFF-REPAIR.11.14.3.4` the first two admitted any enrolled
principal.** Measured with a second tenant holding a `res_…` id it had never
registered, the read returned the whole row — `original_locator`,
`expected_digest`, `credential_binding_ref`, the free-text `purpose` one tenant
wrote about its own research, `risk_class`, `submitted_by`. ⭐ Including
`visibility_scope`, which that row declared as `"tenant"` while the read ignored
it.

⚠️ **The last row of that table is a limit and it is deliberate.** The pair
replay IS an existence confirmation, and it cannot be closed: §12.6 requires a
changed page to be a second reference and §12.1 forbids erasing that
distinction, so the same pair must return the same id. ⭐ The difference from a
snapshot is nameable — confirming a snapshot means presenting its **bytes**,
while confirming a reference means presenting a **locator**, which anyone can
type. What the binding buys is therefore precise rather than total: **the read
stops disclosing anything the caller did not already hold.** A bare opaque
handle used to be the whole predicate; the locator is now required, and the
locator is the thing the row would have disclosed.

⚠️ **A reference registered before this binding has no recorded registrant, so
no tenant reads it** — and registering the same pair again restores the read,
exactly as re-acquiring a snapshot restores its. The attribution cannot be
recovered: `submitted_by` is a one-way hash that joins to no identity table and
the pair replay leaves it naming the first registrant regardless.

⛔ **What this does not close.** `credential_binding_ref` is still an
unauthenticated caller field that SELECTS a credential, and a second tenant that
registers the same pair inherits the shared row's binding. ⚠️ Its reach today is
measured rather than assumed: the R5 pack is off by default and no production
code path registers a broker binding, so every shipped deployment runs an empty
broker store. `SIGNOFF-REPAIR.11.14.3.10` owns it.

### How a deliberation records an assessment

ROADMAP §13.2's deliberation flow registers resource references (step 2) and
acquires or assesses evidence (step 6). The `assess` workflow step is where that
happens, and two shipped built-in profiles declare it — `evidence_review`
(`solicit`, `evidence_request`, **`assess`**, `decide`) and `policy_proposal`
(`solicit`, `revise`, **`assess`**, `vote`, `approve`).

A contribution of kind `assessment`, on the `assess` step, records one §12.7
assessment:

```json
{
  "tenant_id": "...",
  "content": "the evidence supports the claim",
  "kind": "assessment",
  "assessment": {
    "claim_digest": "<a claim digest of this thread>",
    "snapshot_id": "<a snapshot this tenant cited>",
    "assessment": "supports",
    "excerpt": "preserved every row",
    "rationale": "the report states it in the acquired bytes"
  }
}
```

The claim is named by the digest **the server computed** when the claim was
contributed — the client never supplies it — so the identifier cannot be
invented. Six refusals, each naming the gate that produced it:

| The request | Refused because |
| --- | --- |
| the step is not `assess` | the contribution names the step it requires and the step it found |
| the claim digest is not a claim of this thread | the membership check an evidence request already applies |
| the snapshot is not one this tenant cited | assess evidence this deliberation acquired |
| the excerpt is not in the snapshot's bytes | citation existence alone never satisfies an evidence gate (§12.7) |
| the assessment payload rides another kind | it rides an `assessment`-kind contribution only |
| the assessment word is outside the five | `supports`, `contradicts`, `contextualizes`, `source_only`, `unverifiable` |

The contribution event and the assessment row commit in **one transaction**, so
a deliberation never records an assessment the evidence store did not accept,
and never accepts one the timeline does not show. The resulting row is readable
through `GET /v1/claims/{claim_digest}/assessments` by the tenant that authored
it — ⚠️ alongside any assessment the same tenant asserted on that claim through
the non-deliberation route, which the next section describes. That read is not a
list of what the deliberation recorded; each row says which it is.

### The two assessment namespaces

`POST /v1/assessments` still exists and still takes a free-text `claim_id`. It is
the **non-deliberation path**: an assessment asserted outside any thread, whose
identifier is a caller's label rather than a digest the server minted and
membership-checked. Both writers reach one table, so every row records **which
namespace its identifier belongs to**:

| `claim_namespace` | Written by | What `claim_id` is |
| --- | --- | --- |
| `thread` | a contribution of kind `assessment`, on the `assess` step | a claim digest the server computed and checked against that thread |
| `external` | `POST /v1/assessments` | a caller label, bound only by the authoring tenant |

Both columns are recorded by the server. A row written before them carries
`null` in each and is read as unattributed; nothing backfills them, because
which writer minted an older row is not recoverable — the only evidence would be
the *shape* of its `claim_id`, and a caller can always type the minted shape.

The namespace is **part of the row's identity**, not a label beside it: it joins
the replay key, so the same claim, snapshot, kind and author in two namespaces
are two assertions and two rows.

It is one of **two** server-set columns in that key, and the rule behind both is
worth stating once:

> Every column of a replay key must be a value the submitter is entitled to
> assert. A caller-set column is safe only where the key space is already
> partitioned by something the server sets.

`author` on the standalone route is a caller-supplied string — the authorization
never reads it — so before those two columns it could be aimed, twice over:

| What a caller supplied | What it was handed back | Closed by |
| --- | --- | --- |
| a real thread's claim digest, its snapshot, its kind and its author | **the deliberation's own `assessment_id`**, for an assessment it never contributed | `claim_namespace` |
| another tenant's claim, snapshot, kind and `author` label | **that tenant's `assessment_id`** | `authored_by_tenant` |

Both now write their own row and read their own id back.

⚠️ One case is left open and stated rather than implied: **two principals inside
one tenant can still alias each other** on the standalone route. That is not a
disclosure — the authoring gate already admits both of them to that row — so it
is a deduplication question. Making `author` itself trustworthy is open under
`SIGNOFF-REPAIR.7.4`.

Both namespaces ride `GET /v1/claims/{claim_id}/assessments`, labelled. Nothing
is filtered out: a read that returned only `thread` rows would make the
standalone route write-only, which is a worse outcome than removing it. What a
reader gets instead is the ability to tell an assessment a deliberation's gates
admitted from one asserted beside it.

### Both writers assess evidence their tenant acquired

⛔ **`POST /v1/assessments` used to apply no citation gate**, while the `assess`
step refused a snapshot the tenant did not cite. Both writers call the same
store function, and that function selected the snapshot's bytes on
`snapshot_id` alone. Measured against the unrepaired route, a second tenant
holding an `snp_` id it had never cited received three distinguishable answers:

| The request | The answer, before `SIGNOFF-REPAIR.11.14.3.8` |
| --- | --- |
| an identifier that names nothing | `400` `the cited snapshot does not exist` |
| a real snapshot, excerpt NOT in its bytes | `400` `the excerpt does not appear in the snapshot's bytes` |
| a real snapshot, excerpt IS in its bytes | **`200`**, and the row was stored |

The first two separate *exists* from *does not exist*. ⭐ The third is stronger:
a `200` says a **chosen substring appears in bytes the caller was never allowed
to read**.

Both writers now ask the same question, in the store rather than at each route,
so no future writer can be built around it:

| Caller | Answer |
| --- | --- |
| a tenant that cited the snapshot | the §12.7 checks as before — the five kinds, the excerpt against the acquired bytes, the replay |
| a tenant that did not | `400`, **one message**, naming neither the snapshot nor any fact about it: `the snapshot is not cited by this tenant — assess evidence this tenant acquired` |

⚠️ **This is a change to a shipped route** — a caller that assessed evidence its
tenant never cited used to receive `200`. The objection that it breaks a team
legitimately assessing another team's evidence was *measured* rather than
accepted: every read of that snapshot — [the five surfaces
above](#who-may-read-an-evidence-snapshot) — already answers a non-citing tenant
`404`, so that workflow could not function. The excerpt check was running over
bytes the caller could not see, list or delete.

The supported cross-team path is the one this chapter already describes:
**re-acquiring the snapshot records the second citation on the replay**, and the
second tenant then assesses the shared row normally, holding its own separate
assessment. The rejected alternatives — a uniform refusal that leaves the
content probe, and leaving the oracle published — are in
`docs/decisions/2026-09-17_the-standalone-assessment-is-citation-bound.md`.

⚠️ Two limits stay open and are stated rather than implied. A caller that HAS
cited the snapshot still gets `the cited snapshot does not exist` and `the
excerpt does not appear …` as separate answers, which is deliberate: the
diagnosis is owed to a caller entitled to the bytes. And **two principals inside
one tenant can still alias each other** on this route, which the citation gate
does not touch — `SIGNOFF-REPAIR.7.4` owns making `author` trustworthy.

Run the control in the owned disposable PostgreSQL environment:

```bash
RB_DEMO=0 bash scripts/run_pg_tests.sh profiles
```

`the_two_assessment_writers_are_two_namespaces` drives both writers onto one
claim digest, then drives a **second tenant** onto the first tenant's row by
presenting its `author` label. Three labelled rows where two aliased ones used to
be.

## Public repository and publication checks

This project is public and must remain public. The director confirmed that the
earlier private-repository instruction was wrong; README, ADR-001 and their
companion guidance now reflect the correction. No visibility change is needed.
Name/package/domain and release qualification remain separate requirements.

Repository commits, branches, issues and pull requests are public. Confidential
security reports and pre-disclosure work require a separate private channel or
workspace arranged with the accountable owner; see `SECURITY.md`. Do not rely on
public Git to provide an embargo or make this repository private for that purpose.

The visibility question is resolved. The earlier checkpoint's format/dependency
passes, two redacted history-scan findings and deliberately interrupted Clippy
result retain their exact historical status. History-scan and browser prerequisites
are repaired; complete the remaining fixture repairs and full checkpoint, then perform the
authorized normal push and consume triggered CI results. Details:
`docs/decisions/2026-09-09_public-repository-policy.md` and
`docs/tasks/artifacts/signoff_review/publication-precondition.md`.

## Periodic compiler artifact cleanup

Review generated artifacts about once per day. Keep diagnostics and failed-run
fixtures until their owning task has established a safe disposition. An old
`.bin` file alone is not disposable evidence: incremental compiler sessions contain
related cache components and may be in use or shared by hard links.

The checkpoint cleanup qualified the exact installed macOS compiler, preserved
every newest finalized session and excluded young, partial, working, hard-linked
and evidence-dependent data. Native controls verified both shared and exclusive
cache locks against actual compiler collection and successful later execution.
Only a frozen, identity-checked set of whole obsolete sessions was removed under
exclusive locks, after a native inactivity census.

That operation removed 645 sessions / 1,984 files / 5,116,558,334 logical bytes.
Immediately after removal, the remaining cache metadata and 3,545 inspected
source/diagnostic hashes were unchanged. Zero-byte session locks remain for the compiler to collect. Logical
bytes do not establish physical disk space recovered on a cloning filesystem.
This was a qualified maintenance operation, not a general automatic deletion
command. Recheck compiler/platform locking before repeating or broadening it.
See `docs/tasks/artifacts/signoff_review/compiler-artifact-disposition.md` for
selection, refusal conditions, native controls, exact residue and workflow evidence.

## Browser invocation storage and shutdown

Run an R3-enabled server or `reasonbraid-browse` from within the repository. The worker
finds the root from current-directory ancestors containing Cargo.toml and migrations.
It creates `.project-data/browser/run-<UUID>` exclusively, mode 0700, on the repository
volume. Profile, cache, scratch, configuration and data paths are private children;
Chrome receives explicit paths, including its log and crash-dump environment
overrides; ambient TLS/QUIC diagnostic output overrides are removed. Moving the
checkout changes those paths at runtime.
A missing root, linked storage parent, foreign volume or unsafe writable parent
refuses with `browser_storage_failed`; there is no OS/home temporary fallback.

For example, from the repository root with the installed browser configured:

```bash
python3 -B scripts/project_env.py target/debug/reasonbraid-browse <<'JSON'
{"url":"http://127.0.0.1:8080/page","steps":[{"action":"navigate","url":"http://127.0.0.1:8080/page"}],"limits":{"max_steps":4,"max_output_bytes":1048576,"time_budget_secs":5}}
JSON
```

The caller must first classify and authorize the destination; this example assumes
an owned local origin is running. Existing successful JSON keeps derived chunks,
network log, title and browser/worker versions. `click_failed`, `output_too_large`
and `time_budget_exceeded` remain named refusals. Rendering includes startup under
the requested budget, minimum one second; startup also has a twenty-second ceiling.
Every cooperative result then receives up to ten additional seconds for process
and task shutdown. Chrome is closed, its direct child reaped, its group observed
absent and the CDP/network/stderr tasks consumed. Bounded TERM/KILL requests may be
needed; a refused inspection or signal is never evidence of absence.

A render result cannot bypass failed cleanup, and a refusal carries **two**
facts rather than one. Every error object states `cleanup_confirmed`, and adds
`cleanup_error` when cleanup could not be confirmed. `kind` names the render's
own outcome — the budget you set, the selector you supplied — because that is
the fact the caller acts on; the cleanup fields are the fact an operator acts on.
Only a render that SUCCEEDED under unconfirmed cleanup is named for the cleanup
itself, as `browser_cleanup_unconfirmed`: it has no failure of its own to report
and must not read as complete while a browser may still be running. A refusal
raised before any browser was spawned reports `cleanup_confirmed: true` with no
`cleanup_error`, which means what it says — nothing was started, so nothing can
remain. The cleanup detail is also appended to the message, so a consumer that
reads only `kind` and `message` still receives it.

For example, a tripped 30-second budget on a host where a detached browser
helper outlived the cleanup budget answers:

```json
{"error":{"kind":"time_budget_exceeded",
  "message":"the render exceeded the 30-second budget; browser stderr completion unconfirmed",
  "cleanup_confirmed":false,
  "cleanup_error":"browser stderr completion unconfirmed"}}
```

A consumed successful invocation removes only its verified original
directory. A failed or unconfirmed invocation retains private `owner.json`,
`completion.json` and at most 64 KiB of `browser.stderr` when those files can be
written. Stderr also carries ownership/completion receipts with root-relative
workspace paths. Inspect these records before cleanup; an old numeric PID alone
must never authorize signalling a current process.

The shutdown budget covers asynchronous process/task operations, not a hard bound
on filesystem calls, stdin or response delivery. Storage assumes an operator-owned
repository. Aggregate retained-data quotas are still open. The current server-side
spawner can kill the worker at the render deadline, interrupting this shutdown;
`.7.3.1` owns that integration repair. Container enforcement for hard termination or
detached descendants remains `.7.3.2`. Broader network, sandbox, total-output and
evidence policies retain their existing qualification gaps. Linux/macOS process
support is explicit; the current real-browser evidence is native macOS evidence.

## Project binaries

```bash
make release
```

| Binary | Role |
| --- | --- |
| `rb` | the CLI — the primary surface (threads, enrollment, node ops, budget/audit inspection) |
| `rb-server` | the control plane — API + node channel + the embedded console at `/` (one listener, one binary) |
| `rb-site` | deployment-local site authority administration and audited inventory; see [site authority](site-authority.md) for required database permissions and volume checks |
| `rb-bench` | adapter benchmark runner |
| `reasonbraid-browse` | browser acquisition worker |
| `reasonbraid-extract` | resource extraction worker |
| `rb-release-manifest` | release manifest tooling |
| `rb-node` | the node worker — outbound channel, SQLite journal, the adapter supervisor |
| `rb-journal` | the node-journal inspection tool |

The control-plane binary embeds its database migrations and console assets, so a
deployed `rb-server` needs no runtime path back to the checkout for those embedded
assets. Enabling the R3 browser worker currently requires a working directory within
the checkout for private storage discovery. The `rb-site` operator tool also runs
from within the checkout to verify storage locality. The node's
journal is a local SQLite file that stays on the node's own volume.

## The two profiles

`ROADMAP.md` §6.6 names the initial profiles; Phase 1 ships two of them:

- **Developer** — loopback, one host. `make dev` boots an ephemeral on-volume
  PostgreSQL and `rb-server`; Ctrl-C cleans everything up (the `.1.7.1`
  one-command development environment).
- **Trusted LAN** — enrolled nodes on a private network. The control plane
  binds the LAN (`rb-server --host 0.0.0.0 …`); each node enrolls with a
  one-time token and connects outbound (`rb-node --server http://<cp-host>:4310 …`).

The `deploy/` runbook walks both hosts step by step: PostgreSQL provision,
server start (migrations apply on startup), enrollment, node start, and the
adapter choice (the deterministic fake for practice, the env-gated real
harness suites for live runs).

## The packaging proof

The two-host demonstration runs against the **release binaries** with one
flag — the packaging claim is verified by running the package:

```bash
bash scripts/demo_two_host.sh --database-url <url> --release
# two real hosts:
bash scripts/demo_two_host.sh --database-url <url> --release \
    --node-host <other-host> --remote-workdir '~/rb-demo'
```

The demo's ssh mode copies `rb-node` + `rb-journal` to the remote host and
keeps the journals there; the bundle's `env.txt` records the mode and the
build.

## Honest boundaries (Phase 1)

- **Trusted LAN only:** the channel credential is the dev shared secret (the
  HMAC key-proof handshake); X.509/mTLS workload identity is ADR-006/ADR-007
  (Phase 2), and the wire is plain HTTP. Public exposure waits for the G6
  Internet-qualification gate.
- **One node, one role:** the dev profile has no directory — a node id is the
  agent-role wire id it serves.
- **Flags, not config files:** everything is flags/env; process supervision
  units and container images arrive with Phase 2 operations — recorded in the
  runbook's subtraction record.

## Backup and restore (`.4.1`)

The recovery control is the RESTORE, not the dump (§17.5: a backup that has
never been restored is not a recovery control):

```bash
bash scripts/backup.sh                  # DATABASE_URL -> target/backups/<date>.dump
createdb reasonbraid_restored           # an ISOLATED target database
BACKUP_FILE=… RESTORE_DATABASE_URL=… bash scripts/restore.sh
```

The dev-profile dump is plaintext (the encryption + key story deferral is
named in the `.4.3` record), and the guard runs the restore EXERCISE on every
pass: the `backup_restore` suite seeds rows, takes a real `pg_dump`, mutates
the live database, restores into an isolated database, and asserts the
pre-mutation state came back.

## Observability, SLOs, and the runbook (`.5`)

- **Metrics:** `GET /v1/admin/metrics` exposes the seven process-wide
  counters (`authorization_denials`, `idempotency_replays`,
  `handshake_refusals`, `lease_refusals`, `dead_letters`, `results_folded`,
  `results_rejected`). The gate: the caller holds `tenant_admin` in an active
  grant **whose boundary is also live**. Every counter is incremented at the
  same boundary that writes its record, and the live test asserts the denial
  delta against the denied row — the surface cannot drift from the ledger.
  Structured `log_event!` JSON lines carry the correlation fields under
  ADR-023's redaction rules.

  ⚠️ **The boundary half of that gate was missing** (`SIGNOFF-REPAIR.3.5.2`).
  Revoking a boundary updates only the boundary row; it does not cascade to the
  grants issued under it, so those grants keep `status = 'active'`. The check
  read the grant alone, and an administrator whose authority had been withdrawn
  kept this surface. Reproduced against the live route, then repaired and
  falsified against the unjoined query.

  ⛔ This surface deliberately does **not** take the frozen-boundary exception
  the nine `/v1/admin/*` inspection reads take. That exception exists so an
  administrator can inspect *authority* state while a revocation is in flight;
  process counters are not authority state.

  ⭐ **The read is now audited, and it needed no wire change to become so**
  (`SIGNOFF-REPAIR.3.5.2.1`). Every call commits an authorization record —
  allowed or denied — and a successful one returns
  `x-reasonbraid-authorization` naming it, so `curl -i` retains the receipt
  exactly as it does on the other administrative routes.

  The route still takes **no `tenant_id`**. It does not need one: a principal
  belongs to exactly one tenant structurally, so the record binds to the tenant
  the caller already has. That is why closing this gap broke no caller — the
  three shapes originally considered all assumed the route had to start naming a
  tenant, and none of them was necessary.

  ⚠️ **One thing narrowed, and it is worth knowing.** The gate used to admit a
  holder of `tenant_admin` in **any** tenant; it now requires you to administer
  **your own**. Every principal this system can create holds its grant in its own
  tenant, so no reachable caller loses access — but the rule is strictly
  stronger than it was.

  ⚠️ **The width of what you SEE is unchanged and still deliberate**: the
  payload is process-wide, so an administrator's counters include other tenants'
  volume. That is aggregate counts with no identities and no per-tenant
  breakdown, and narrowing it would be a different decision from auditing it.
- **SLOs:** the dev profile's hypotheses are the guard's own measurements
  (`docs/decisions/2026-09-07_phase2-slo-hypotheses.md`): acceptance, the
  demo's 34 checks, the restore exercise, and the reconcile-after-kill beats
  all target 100 % with a ZERO error budget — a red pass halts the frontier.
  Unmeasured latency families are named with their triggers, not numbers.
- **Runbook:** node lost/replaced (`docs/runbooks/node-lost-replaced.md`)
  covers detection through closure tests; its closure tests are the demo's
  SIGKILL beat, the revoke beat, the replay suites, and the restore exercise.

The resumed **8d1504d** checkpoint passes eight gates but workspace testing stops
at one state-writer lock failure; PostgreSQL and the demo never start. All sixteen
browser controls pass. Six controlled actual-CLI scenarios then prove that an
inherited child descriptor can retain exclusion after parent success, error or
cancellation. Exact attribution of the original deleted fixture is unavailable.
All 341 source hashes match and fifty recorded groups are absent. That diagnostic commit left production unchanged; the explicit-release repair
and permanent controls follow below, before checkpoint resumption/public push. Broader inherited-descriptor process-loss
qualification remains owned separately. See
`docs/tasks/artifacts/signoff_review/state-writer-lock-lifetime.md`.

Inherited state-lock release is now repaired under .11.4.3.1.2.11.2. A guard
explicitly unlocks before closing its File, including post-acquisition failures;
two successful test probes do likewise. The permanent five-path child regression
fails on unchanged production and passes after repair. All 32 selected CLI tests,
the final twelve-writer rerun, six raw-fork actual-API scenarios and strict CLI
lint/format pass. Original assertions and 339 other non-Markdown sources remain
unchanged. The original holder remains uncaptured; inherited references after
abrupt owner death retain separate restart ownership. Resume the full checkpoint
from this committed repair before public push/remote CI. Evidence:
`docs/tasks/artifacts/signoff_review/state-writer-lock-release.md`.


### Extraction test input ownership


The **ec8df08** checkpoint next stops at two PDF tests: zero chunks instead of
one, and a JavaScript-bearing fixture unexpectedly accepted. Eight other gates
pass; PostgreSQL/demo never start. An unchanged extractor rerun receives another
test's outer.zip. The exact old helper then reproduces three native-clock filename
collisions among 32 simultaneous writers. Original checkpoint paths are missing;
passing isolated/instrumented reruns do not erase the failures. Exclusive
repository-local unit/stdio inputs are repaired under .11.4.3.1.2.12, with all
original parser assertions preserved. All twelve selected tests, strict lint/format
and three independent locality cases pass. The analogous server input risk has concrete
next repair owner .7.3.3 before checkpoint resumption. See
`docs/tasks/artifacts/signoff_review/extraction-fixture-ownership.md`.

From the repository root, run the extraction controls:

```bash
python3 -B scripts/project_env.py cargo test --locked -p reasonbraid-extract --all-targets -- --nocapture
```

Each fixture owns an exclusively created 0600 file under `target/extract-tests`.
The helper derives that path at runtime, checks its parent directories stay on the
repository volume, and never falls back to a home or system temporary directory.
An existing candidate is skipped without changing its bytes. Tests keep inputs
through their assertions, remove successful inputs only after checking identity,
and print a repository-relative retained-input path on panic. If an input path
has been replaced, cleanup refuses to remove the successor. Investigate retained
failures before any exact, ownership-proven artifact retirement.

For example, two concurrent PDF fixtures must retain separate paths and exact
source bytes until both owners finish. A pre-existing file or symlink must survive
a candidate collision. The permanent controls also cover 32 simultaneous owners,
assertion failure and a replaced path. These are fixture guarantees in a controlled
repository workspace. They do not qualify hostile concurrent directory changes or
the server's still-pending production R2 input/worker isolation boundary.


### Where an extraction input lives, and when it is deleted

An extraction request's acquired bytes reach the worker through a file. A name
built from a process id and a nanosecond field is a guess about uniqueness, and
a write that truncates whatever is already there turns that guess into another
request's loss — which is exactly what `.7.3.3.1` reproduced.

The server now creates one private file per request, mode 0600, under
`.project-data/extraction` below the repository root it discovers at runtime
from the current directory. Nothing persists an absolute path, so moving the
checkout moves the store; there is no temporary-directory or home fallback, and
an unusable location refuses rather than writing somewhere quieter. Every parent
is proved to be a directory this process owns, on the same volume, and not
writable by group or other. An occupied candidate name is skipped whole — never
opened, truncated or adopted.

Deleting that input needs two facts, and a successful response is neither: no
worker may still be able to read it (none was started, or the spawner observed
its exit), and the file must still be the exact one created — same device, same
inode, one link. A file failing either test is retained, with its
repository-relative path printed. For example, an unconfirmed worker leaves its
input in place and says so, and a file that something else replaced is never
deleted. The store removes one file it created, or nothing.

The owner also exposes the digest of the bytes it wrote, in the worker's own
`sha256:<hex>` form, and the R2 handler now binds every response to it. A
receipt whose `parent_digest` is not the digest of the bytes this request
supplied is refused with the kind `extraction_source_mismatch`, before any
snapshot, derivation or receipt is written — an acquisition is not accepted
merely because a response arrived. The handler's whole extraction leg is one
call, so the same controls that cover the boundary cover the handler's use of
it: a preserved `feed_unreadable` refusal, a worker failure keeping its own
classification, the mismatch refusal naming both digests, and eight concurrent
callers each receiving a receipt for their own document. Evidence:
`docs/tasks/artifacts/signoff_review/extraction-owned-input.md`.

That join is now driven live. A control serves one Atom document from a local
origin and resolves the SAME reference through three deployments that differ
only in the R0 fetcher their state was built with, so each production gate is
visible on its own: the shipped state refuses at the scheme, the origin's scheme
under the shipped destination policy refuses at the `loopback` class by name,
and the origin's scheme under a loopback-admitting policy acquires. The
persisted evidence is then read back and asserted against the bytes the origin
served — the snapshot's `raw_digest` and `byte_length`, and the `derivations`
rows' content and digests — rather than against a status code.

The seam this uses is `ApiState::with_acquisition`, which takes the R0 fetcher a
deployment's acquisition legs use. It relaxes nothing on its own: `new`,
`with_gate`, `api_router` and `api_router_gated` all still build
`Fetcher::new`, so the shipped https-only public-destination policy is what the
server binary runs. A deployment that wants another destination policy has to
write it in its own source and owns what it admitted.

Three deliberate injections, each reverted, establish that the control can go
red: serving one extra byte fails the digest assertions (its chunk digests were
unchanged, so the raw-byte leg is the one that caught it), discarding the
supplied fetcher fails at the destination gate, and persisting a derivation that
is not the worker's chunk fails at the derivation assertion and nowhere earlier.
Evidence: `docs/tasks/artifacts/signoff_review/r2-acquisition-join.md`.

The mismatch refusal has its own live control. A dishonest worker, injected
through the `R2_WORKER_BIN` override the spawner already reads, returns a
well-formed reply describing bytes the request never supplied; the handler
refuses it `extraction_source_mismatch` and the control asserts an ABSENCE —
zero snapshots for that reference, zero derivations joined to it, and the whole
store's snapshot and derivation counts unchanged across the request. Writing a
snapshot before reporting the same refusal leaves the refusal's own name intact
and still fails that control, which is why the counts are there and the error
kind alone is not enough.

One limit stays stated rather than implied. The R2 pack advertises
five media types its own acquisition leg refuses: the R0 sniff accepts a
declared content type only when it is `text/html`, `application/xhtml+xml` or
`text/*`, so the same feed that succeeds served as `text/xml` is refused
`media_type_refused` under its own `application/atom+xml` — measured by that
control, on the same bytes. What an untyped response of each format sniffs to
from its bytes is not yet measured. `SIGNOFF-REPAIR.7.3.3.5` owns the per-format
census and the decision it leads to; neither the registry row nor the sniff
changes before that census exists.

That census is now run, and it changed the finding's shape. A declared content
type is accepted only when it is `text/html`, `application/xhtml+xml` or any
`text/*` — three arms, enumerated in both directions by a control in the
fetcher's own module. With NO declared type the verdict is a property of the
BYTES rather than of the format: an all-printable body is accepted whatever
format it belongs to, and the same body with one non-text byte is refused. ZIP
and tar cannot reach that branch at all, structurally — a ZIP local file header
is `PK\x03\x04` followed by nine little-endian integer fields, and a tar header
is a 512-byte block with a NUL-padded 100-byte name.

So "the five advertised formats are unacquirable" is true but the wrong shape:
the leg's rule is not about formats, and no subset of the advertisement
satisfies it. Narrowing the registry row is therefore rejected — it cannot
express the truth — and the accepted repair is that the acquisition leg admits
the RANKED resolver's own advertised media types, read at resolution time. The
destination policy, the scheme list, the byte ceiling, the ratio brake, the
redirect policy and the time ceiling are all outside that change, and an
R0-ranked acquisition keeps the text/HTML accept set exactly as shipped. The
decision, its rejected option and the risk it accepts are recorded in
`docs/decisions/2026-09-12_r2-acquisition-accept-set.md`.

That repair now ships. `Fetcher::fetch_admitting` takes the ranked pack's
advertised types, read from its registry row per resolution, and a declared type
in that set sniffs to the additive `SniffedKind::DeclaredType`. `fetch`,
`fetch_head` and `fetch_authenticated` pass an empty set, so R0's shipped accept
set is unchanged — and `fetch_admitting` has exactly one caller, the R2 arm. A
missing or malformed registry row yields an EMPTY set, because a bad
advertisement must never widen a gate. The live control now acquires the same
feed served as `application/atom+xml`, records `application/atom+xml` as the
snapshot's media type rather than a sniffed stand-in, and still refuses a type
the pack does not advertise — a bound proved by adding one such type to the
admitted set and watching the negative control fail.


### Where a Git acquisition works, and when it is removed

An R1 acquisition clones into a working directory before its objects are
measured and digested. That directory used to be named from the process id and
a nanosecond field inside the ambient temporary directory, and created with a
call that adopts an existing path instead of refusing it. Two concurrent
acquisitions in one server share the process id by construction, so they could
clone into one directory — and either one's error cleanup then deleted the
other's objects. A probe measured that naming at 501 distinct values in 2000
calls, with every collision falling between adjacent calls, and confirmed the
ambient temporary directory was on a different volume from the checkout.

Acquisitions now use the same owned storage the extraction inputs use, under
`.project-data/git` below the repository root discovered at runtime. The
directory is created exclusively — an occupied name is skipped, never adopted —
mode 0700, with every parent proved to be a directory this process owns on the
same volume. There is no temporary-directory or home fallback.

The acquisition OWNS that directory for its whole life. The superseded contract
said the caller owned the cleanup and no caller ever did, so every successful
acquisition left a bare repository behind; the directory now goes away when the
last holder of the acquisition drops it, after the receipt has taken its digest.
Removal first proves the path still names the directory that was created — the
open descriptor held on it pins the inode, so a replacement cannot present the
same identity and be deleted in its place. A directory that fails that test is
retained with its repository-relative path named. Unlike a file, a directory's
link count is not part of the proof: its links grow as subdirectories appear
inside it, and macOS was measured still reporting two links on a held descriptor
after the directory was removed.

### What `rb-server` checks before it touches the database

Every configuration refusal now happens **before** anything mutates. The
declared secret-store profile and the bind address are both pure checks — a
match on a name and a parse — and both used to run *after*
`sqlx::migrate!` had already moved the schema. A typo'd profile pointed at the
wrong database left that database migrated and no service running, which is a
refusal that has already acted.

A control proves the order without needing PostgreSQL at all: it points the
server at a port nothing serves, so a boot that reaches the connection reports
`PoolTimedOut` and a boot that refuses first reports the configuration it
refused. One knob — which argument is wrong — decides which message appears. A
third control drives a valid configuration and asserts it still reaches the
connection, so a repair that refused every boot would fail it.

The bind-address refusal also names itself. `main` returns `Box<dyn Error>`,
whose termination prints the *debug* form, so a typo'd `--host` used to report
`Error: AddrParseError(Socket)` — no argument, no value. It now reads
``the bind address `not a host:4310` is not a host and port``.

### How far a boot reaches, and where that limit is published

`rb-server --host 0.0.0.0` is **not** refused. It is the supported trusted-LAN
profile this chapter documents, the one `deploy/`'s runbook and the two-host
demonstration both walk, and gating it would refuse a shipped capability. What
bounds Internet exposure is the G6/G7 gate and the blockers on the
[Blockers](blockers.md) page — an externally reviewed threat model, a
penetration test and a prompt-injection action-boundary suite — none of which is
a property of a command-line argument.

What was wrong is that the startup line could not tell the two apart: it printed
`(Phase 0 dev profile)` whether the server was reachable from one loopback or
from every host on the network. It now names its reach:

```text
rb-server listening on http://127.0.0.1:4310 (Phase 0 dev profile; reachable from: loopback)
rb-server listening on http://0.0.0.0:4310 (Phase 0 dev profile; reachable from: every interface)
```

The classification is a pure function with its own control over six addresses,
both IPv6 forms included. The decision, its rejected alternatives — a refusal, a
confirmation flag — and what it deliberately does **not** claim are recorded in
`docs/decisions/2026-09-16_rb-server-bind-exposure.md`.

### What a rendered page is allowed to fetch

The R3 browser pack advertises two deny-policies to every caller that reads the
resolver registry — `redirect_policy: "deny"` and `subresource_policy: "deny"` —
and it now enforces both.

The server's pre-flight classifies exactly **one** URL: the locator the caller
named. Everything after that is the page's own doing, and until this repair the
worker enforced nothing at all. It subscribed to Chrome's request events purely
to write the network log, so an image, a stylesheet, a script or a redirect went
wherever the page pointed it. A control drove a page whose `<img>` named
`0.0.0.0` — a reserved address the destination policy refuses, which the kernel
routes at the local host — and the origin recorded the request arriving.

The gate is Chrome's `Fetch` domain, which pauses every request before it leaves
the browser. A **document** request for a URL the worker was asked to navigate
to continues; everything else fails with `BlockedByClient`. That is exactly the
two advertised lines: a non-document request is a subresource, and a document
request for a URL nobody asked for is a redirect.

Each refusal is named on the render receipt, in `refused_requests`, with the URL,
the policy that refused it and Chrome's own resource type. The network log still
records the attempt, so the disclosure and the refusal are two separate facts and
a reader can see both. A denial nobody can see is indistinguishable from a page
that never asked.

**The consequence, stated rather than discovered.** A page that assembles its
visible text from an external stylesheet or script renders less text here than it
would in a desktop browser. That is what "deny" means, and it is the posture this
pack advertises for untrusted content: the rendered evidence is what the document
itself carries. Denying every request including the navigation is *not* the
policy — a control asserts that the requested page still loads, so a repair that
blacked the browser out would fail it.

### Where a credential goes, and where it stops

The R5 pack acquires a resource with a credential the broker resolves for one
binding. That credential is bound to **the origin the caller named** — scheme,
host and port — and it is dropped on any redirect hop that leaves it.

The hop itself is still followed. It has already been re-hardened and
re-classified by the same destination policy as the first dial, so it is a
legitimate public address; what must not cross an origin boundary is the secret,
not the request. An acquisition that redirects to a signed URL on a CDN
therefore still succeeds, carrying no credential to the CDN — which is how
browsers and `curl --location` have behaved for years. A redirect that stays on
the caller's origin keeps the credential, so this is a rule rather than a
prohibition, and a control asserts each half.

Before this repair the credential was re-attached on **every** hop of the
fetcher's manual redirect loop, and the origin — not the caller and not the
server — chose where those hops went. A control drove an authenticated fetch
through a redirect from `fetch.test` to `second.test` and the origin recorded
receiving the `Authorization` header at both.

The acquisition receipt now says where the credential actually went. A fetched
document carries `credential_hosts`, the distinct hosts the header was attached
to, recorded at the moment of attachment; the snapshot's disclosure policy names
that list and the disclosure record names its first entry. It used to name the
**final** URL's host, so a credential sent to one host was disclosed against
whichever host the redirect chain happened to end on.

### What the operator's git configuration can reach during an acquisition

Nothing. An R1 acquisition fetches a **caller-supplied** URL, so the repository
it fetches into is opened with `gix::open::Options::isolated()` — in gix's own
words, permissions that "prevent accessing anything else than the repository
configuration file, prohibiting accessing the environment or spreading beyond
the git repository location".

The acquisition used to call `gix::init_bare`, which is
`ThreadSafeRepository::init(…, open::Options::default_for_level(Trust::Full))`
— `Permissions::all()`. A census of gix 0.87.1's own source names every source
that admitted, and what isolating it turns off:

| Source | `gix::init_bare` | The acquisition now |
| --- | --- | --- |
| The repository's own `config` | loaded | loaded — gix always loads it, and this process just created it |
| System config, `$(prefix)/etc/gitconfig` | loaded | refused |
| Application config, `$XDG_CONFIG_HOME/git/config`, else `$HOME/.config/git/config` | loaded | refused |
| User config, `~/.gitconfig` | loaded | refused |
| Environment config, `GIT_CONFIG_COUNT` / `GIT_CONFIG_KEY_n` | loaded | refused |
| `include` and `includeIf` directives | followed | not followed |
| System and application `gitattributes` | loaded | refused |
| `GIT_*` and `SSH_*` environment categories (`home`, `xdg_config_home`, `http_transport`, `identity`, `objects`, `git_prefix`, `ssh_prefix`) | all allowed | all denied |
| The `git` binary's own configuration | not loaded | not loaded — off in both |

This closes the last of ROADMAP §12.5's "default refusal of submodules, hooks,
filters, alternates, and external diff/clean drivers" that a directory boundary
could not reach: owning *where* gix writes says nothing about *what* gix reads.

A control proves it rather than asserting it, and it proves both halves in one
run. A child process is started with `GIT_CONFIG_COUNT=1`,
`GIT_CONFIG_KEY_0=init.defaultBranch`, `GIT_CONFIG_VALUE_0=smuggled`; it opens
one repository the superseded way and one the way the acquisition does, and
prints each repository's `HEAD`. The first says `ref: refs/heads/smuggled` — so
the setting really does arrive on this host — and the second says
`ref: refs/heads/main`. Before the repair, both said `smuggled`.

The same control covers the suite's own inputs. The acquisition fixtures had
kept the default open after production lost it, so the repositories the tests
build were themselves a function of whoever ran them; the probe now builds a
real fixture under the same setting and its `HEAD` must read
`ref: refs/heads/main` too. The probe's first open stays deliberately
un-isolated, and its source says so: a control whose every open refuses the
setting cannot tell a refusal from a setting that never arrived.

### The Git LFS policy, and what a pointer file is

An acquisition **refuses** a Git LFS pointer, by name, and never treats it as
the file it stands for. A pointer is roughly 130 bytes of text naming an object
on a separate LFS endpoint: its bytes were never transferred by this clone, so
they were never classified by the destination policy and never counted against
the object, file or decompressed-size budgets. Accepting one would put a
stand-in into the snapshot manifest and report it as content. The policy is
therefore refusal rather than a declared passthrough, and this paragraph is
where ROADMAP §12.5's "explicit Git LFS policy" can be checked against the
code.

A pointer is identified by the specification's own grammar: the version line

```text
version https://git-lfs.github.com/spec/v1
```

is the pointer's **first** line, so the gate is a 42-byte prefix test at offset
0 that must end at the newline — or at the end of a blob carrying nothing else.
The `oid` and `size` lines are deliberately not required: a truncated pointer is
still not the content, and the refusal is the same either way.

The superseded gate searched a blob's first 64 bytes for the ten-byte run
`version ht` instead, so **any** file that merely quoted the spec URL near its
start was refused as if it were the content it describes — a repository whose
README explains LFS could not be acquired at all. That defect ran in the
over-refusing direction: it was an availability and correctness fault in the
acquisition, never a way to smuggle a pointer past the gate. `SIGNOFF-REPAIR.7.2.3`
repaired it, and the page you are reading quotes the version line itself, which
is exactly the shape the old gate rejected.

### The extraction worker's completion contract

The server spawns one direct worker process per extraction and owns exactly that
process. Every return now says which of three things is true about it: no child
was started (the worker binary was absent, or the spawn failed), this process
observed the child's exit and reaped it, or a bounded stop left that unresolved.
These are separate facts from the request's own outcome — a named refusal is
still a finished worker, and a tripped time budget says nothing by itself about
whether the stop was observed.

Every exit path passes through one bounded stop and reap. A worker whose stdin
is already closed gets 500 ms to finish on its own before the stop escalates to
a signal; a tripped time budget skips that grace, because the killing budget is
the quarantine's enforcement. The whole cleanup is capped at five seconds, so a
worker that ignores the signal cannot hold its caller open. A stop request that
fails is recorded in the evidence and the wait continues; it never becomes a
claimed termination.

For example, a request write that fails after the worker has closed its stdin
used to return while that worker was still running with nobody waiting for it.
A permanent control reproduces exactly that and asks the operating system
whether the recorded pid is still in the process table — a reaped child has no
entry, and a killed-but-unreaped child is still a zombie entry, so one
observation covers stop and reap together.

Callers read the completion through `run_extraction_reporting`; the established
`run_extraction` keeps its call shape and error classification and is what the
R2 API still uses. An input whose reader has not been confirmed finished must be
retained, so `.7.3.3.3` migrates that caller before it gates input cleanup on
this evidence. The error vocabulary is unchanged; only the timeout message
dropped a kill it had not confirmed.

This is evidence about one process. Descendants that worker may start, pipe
pressure, total deadlines and aggregate retained storage remain `.7.3.4`; the
spawner still drains stdout only after the child exits. Sixteen process and
evidence controls plus six spawner controls pass, four of them deliberately
synthetic injections — no cooperative worker reproduces an unkillable process on
demand. See
`docs/tasks/artifacts/signoff_review/extraction-worker-completion.md`.


The subsequent production-boundary diagnosis under .7.3.3.1 uses the exact
server input-name/write span and unchanged extraction module/worker. With 32
simultaneous input owners, six paths collide and eight worker responses describe
another owner's bytes. Two positive controls return their own bytes correctly.
This reproduces input interference before the worker; it does not execute the
HTTP handler or prove an incorrect database write. The API currently lacks a
source/response parent-digest equality check. Production remains unrepaired at
this diagnosis: .7.3.3.2 establishes confirmed direct-worker completion, then
.7.3.3.3 adds exclusive same-volume inputs, exact digest binding and checked
cleanup/retention. For example, two acquired documents must never share a worker
input, and a response for different bytes must be refused before persistence.
Unconfirmed reader completion must retain its input. Pipe/output bounds,
descendant containment and aggregate retained-storage limits remain .7.3.4.
See `docs/tasks/artifacts/signoff_review/extraction-input-boundary.md`.
