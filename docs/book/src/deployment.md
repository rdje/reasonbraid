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

The test uses the existing HTTP at override only to drive its owned fixture.
Production expiry authority/scope and caller-clock restrictions, actual freshness-
horizon refresh and object retirement remain open under SIGNOFF-REPAIR.7.4. This
fixture repair changes no production policy. Evidence:
`docs/tasks/artifacts/signoff_review/retention-fixture-clock.md`.

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

A render result cannot bypass failed cleanup. The worker instead emits
`browser_cleanup_unconfirmed`, preserving the original render error in the message
when present. A consumed successful invocation removes only its verified original
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
  `results_rejected`). The gate: the caller HOLDS `tenant_admin` in any active
  grant. Every counter is incremented at the same boundary that writes its
  record, and the live test asserts the denial delta against the denied row —
  the surface cannot drift from the ledger. Structured `log_event!` JSON lines
  carry the correlation fields under ADR-023's redaction rules.
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

One gap is stated rather than implied: no live test yet drives a SUCCESSFUL R2
acquisition through to a snapshot and a derivation, because the live R2 test
refuses at the loopback destination gate and an outbound Internet fetch is not
an acceptable test dependency. `.7.3.3.4` owns closing that join with a
policy-allowed local origin, not with a weakened production rule.


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
