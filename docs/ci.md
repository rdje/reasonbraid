# CI and supply-chain checks

Phase 0 deliverable `PHASE-0.0.7` (roadmap backlog 8). This documents the automated
checks available beyond the discipline-spine baseline, and draws the line between a *skeleton*
and a *release claim*.

## What runs where

Three GitHub Actions workflows fire on every push and pull request:

| Workflow | Purpose | Local equivalent |
| --- | --- | --- |
| `rust` (job `check`) | local pinned compiler, required Chrome, strict format/lint, locked workspace worker build and tests; database suites skip here and run live in pg-tests | worker build + `make check` |
| `rust` (job `pg-tests`) | all Python controls, then the full owned PostgreSQL 16 collection and required crash/reconnect demonstration | Python unittest discovery + `bash scripts/run_pg_tests.sh --demo` |
| `rust` (job `book`) | pinned mdBook 0.5.4 installation in local stores and book build | `make book` with the matching installed renderer |
| `doctrines` | the 13-doctrine enforcer (same as the pre-commit hook) | `make gate` |
| `supply-chain` | `cargo deny` (advisories/bans/licenses/sources) + `gitleaks` secret scan | `make deny` / `make secret-scan` |

The first two are the discipline spine; `supply-chain` is what `.0.7` added.

## Commands

- `make check` — format, strict all-target/all-feature lint, then `make test`.
- `make test` — locked workspace binary build, then locked workspace tests through
  the pinned browser launcher below. Requires network access to the official archive.
- `make gate` — the doctrine enforcer (`scripts/check_doctrines.sh`).
- `make deny` — `cargo deny check` against `deny.toml`. **Requires** `cargo-deny`
  (`python3 -B scripts/project_env.py cargo install --locked cargo-deny --version 0.20.2`).
  Policy: advisories, duplicate/wildcard versions, source allow-list and reviewed licenses
  with the narrow exceptions explicitly recorded in deny.toml.
- `make secret-scan` — `gitleaks detect --source . --redact`. **Requires** `gitleaks`
  as an installed read-only tool. This command scans Git history; it does not establish
  a scan of uncommitted working-tree files. `--redact` redacts secret values from output;
  findings and their metadata can still be reported.
  `.gitleaksignore` contains two exact immutable fixture fingerprints, qualified
  under SIGNOFF-REPAIR.11.4.3.1.2.2. Both are metadata-only test literals, not issued
  credentials. The exceptions bind commit/file/rule/line; the file and rule remain
  scanned, including the same content in a new commit. Removal controls recover
  each finding. The pinned scanner also loads the source-root ignore file when
  another ignore path is supplied; use an isolated history without that file for
  omission checks. Requalify on scanner/exception changes. Evidence:
  `docs/tasks/artifacts/signoff_review/history-fixture-fingerprints.md`.
- `bash scripts/run_pg_tests.sh authority command_api` — focused suites in a new
  supervised PostgreSQL 16 cluster; caller DATABASE_URL is ignored. `--list`
  lists names. At the REPAIR-0033 census, no names runs 38 server suites, MCP and CLI
  tests (40 runner commands), then the
  demonstration (`RB_DEMO=0` omits it). Suites are serialized. Success removes
  only the stopped owned workspace; failure preserves diagnostic data under
  `target/pg-tests/run-*`. See `docs/book/src/deployment.md` for recovery and
  `--port`/`--demo` examples. Requires PostgreSQL 16; the demo also needs `jq`.
- The WP3 node journal tests and the WP4 adapter suites need no service at all:
  SQLite is a file and the fake/stub adapters are in-process, so the journal
  kill-point sweep, the `rb-journal` CLI tests, and the adapter/ supervisor tests run
  inside plain `cargo test --all` (`make check`) and the `rust` workflow above. The
  PG-backed suites require the owned `run_pg_tests.sh` environment. Live Codex and
  Claude qualification remain ignored and explicitly gated by RB_LIVE_CODEX and
  RB_LIVE_CLAUDE respectively. They dispatch to real providers and spend tokens;
  ordinary verification does not request them. The ignored core schema-golden writer
  is a deliberate regeneration tool, not a runtime gate.

The same dependency and history gates run in CI (`.github/workflows/supply-chain.yml`)
through the verified pinned scanner driver below, with no prior scanner installation
required. Receipts/logs and the redacted JSON report are retained as CI artifacts. The `pg-tests` job now invokes the supervised runner with the full local suite
list. It uses installed PostgreSQL 16 tools and provisions the pinned Rust
toolchain under `.project-data/installed-toolchains`. The workflow was reviewed
and syntax-checked locally; GitHub execution remains part of the next push.
Direct database-backed tests require the runner receipt; DATABASE_URL alone
refuses before fixture writes. See
`docs/decisions/2026-09-09_disposable-test-pool-ownership.md`.

## Scheduled pre-push checkpoint

Fourteen node-fixture cleanup callers now use the checked dependency-plan helper.
Actual MCP-listener→identity and spend-breaker→CLI sequences pass; the affected
caller census runs every cleanup plan successfully, but its feature assertions
are not all green. Participant-removal authorization and a dated retention fixture
each fail once and reproduce with the original fixtures. The server's participant
removal target repair now passes all six invitation, 22 authority and 33 command-API
tests plus ten evaluator controls and strict lint. CLI delegation companion
SIGNOFF-REPAIR.11.4.3.1.2.10 now passes the actual delegated/direct removal and
ordinary-thread controls in all five real CLI tests. Retention fixture .2.9 now
uses observed creation times and passes the focused control and all 31 profiles
tests. All six partial fixture plans now use checked explicit dependencies; each
passes after node-work residue, and their consecutive run passes. The original
25-plan census now has all 25 checked callers after the final five migrations
under .2.7.4. Exact source reconstruction and declared dependency closure pass;
the five-consumer sequence passes 22 tests and the consecutive affected collection
passes all 169 distinct tests (25 original suites plus the guard target), with
zero skips/ignores. Strict server/MCP/CLI lint and final verification pass; both
successful databases are removed and prior failures preserved. The complete
checkpoint still needs to pass before public push/remote CI. Exact results:
docs/tasks/artifacts/signoff_review/node-fixture-cleanup.md and
docs/tasks/artifacts/signoff_review/participant-removal-authority.md and
docs/tasks/artifacts/signoff_review/cli-removal-delegation.md and
docs/tasks/artifacts/signoff_review/retention-fixture-clock.md and
docs/tasks/artifacts/signoff_review/partial-fixture-cleanup.md and
docs/tasks/artifacts/signoff_review/fixture-plan-coverage.md.

The source census at 6bc76c6 identifies 12 workspace packages and 86 test-enabled
Cargo targets; these are targets, not test functions. All 38 registered server
suites exist. The three other server integration targets (mtls, publisher and
reconciler) require no PostgreSQL. The full checkpoint requires workspace checks,
the owned full PG collection with explicit `--demo`, all Python control
modules, doctrines, fresh dependency checks, redacted history scanning and the book.
Build workspace binaries before runtime checks so the extraction test cannot pass
by skipping a missing worker. Browser availability and actual execution must be
reported; an absent-browser early return is not browser qualification.

```bash
python3 -B scripts/project_env.py cargo build --workspace --bins --locked
env -u DATABASE_URL make check
python3 -B scripts/project_env.py python3 -B -m unittest discover -s scripts/tests -p 'test_*.py' -v
bash scripts/run_pg_tests.sh --demo
make gate
make deny
make secret-scan
make book
```

These are the required checkpoint commands, not a claim they have passed now.
All six project command jobs now use the local CI launcher. Python discovery,
worker builds, required browser presence, explicit full demo and pinned book build
are wired. Publisher/browser lifetime and compiler-artifact prerequisite repairs
are complete. Source-7e01097's full checkpoint stopped at two browser timing
witnesses; .2.4 repairs those witnesses and .2.5 binds the dedicated test runtime.
Source-b0cddfe passes nine gates including the complete workspace/browser run,
then fails on the fourteenth PostgreSQL suite: identity_store omitted certificate
children from its node cleanup. Thirteen live suites pass; twenty-six later
commands and the demo are unstarted. Repair .2.6 qualifies certificate cleanup on
fresh and real node-work residue. The remaining MCP-listener/CLI spend-breaker
cleanup candidates have concrete prerequisite .2.7 before checkpoint resumption.
Exact failed-gate disposition and focused evidence:
`docs/tasks/artifacts/signoff_review/identity-fixture-cleanup.md`.
The expanded explicit-plan census finds twenty affected plans. Three actual
producer/consumer baselines reproduce distinct FK failures. Prerequisite .2.7.1
qualifies a shared pre-deletion plan check with eight pg_guard tests (five new
live controls and three existing ownership controls). Caller migration remains
.2.7.2–.2.7.4; this primitive does not make the original failing suites pass yet.
Evidence: `docs/tasks/artifacts/signoff_review/fixture-cleanup-plan-check.md`.
Actual GitHub results remain to be consumed
after the authorized push. Local Make commands use project_env.py.

Exact census, tool versions, source hashes, skips and limits:
`docs/tasks/artifacts/signoff_review/ci-checkpoint-census.md`. Full CI runs before
pushes or selected important steps; ordinary slices use focused checks.

## Pinned browser test runtime

`make test`, `make check` and the Rust CI job use `scripts/ci_browser.py` to select
Chrome for Testing 153.0.8010.36. The source pins archive size and SHA-256 for Linux
and macOS on x86-64/ARM64. There is no floating latest lookup, desktop-browser
fallback or runtime checksum override. Unsupported platforms fail explicitly.

```bash
# Setup/version only; does not claim rendering or test coverage.
python3 -B scripts/project_env.py python3 -B scripts/ci_browser.py --verify-only
# Build workers before a selected live browser test.
python3 -B scripts/project_env.py cargo build --workspace --bins --locked
python3 -B scripts/project_env.py python3 -B scripts/ci_browser.py -- cargo test -p reasonbraid-browse --test browser_roundtrip --locked -- --nocapture
```

Each invocation downloads into its exclusive `target/ci-browser/<platform>-*`
directory, verifies the pinned bytes, validates the complete ZIP layout and
extracts a private runtime. Relative framework link chains must stay inside that
runtime; traversal, duplicate entries, missing targets, cycles and file/link
ancestors refuse. Archive/expanded/member limits are 300 MiB/1 GiB/512 MiB, with
at most 20,000 entries. HTTPS download and version phases have 300-second and
30-second limits. The command default is one hour; `--timeout 120` selects two
minutes (accepted range 1–7200 seconds). Consumed shutdown follows these deadlines.

The launcher overrides ambient browser selection and records the root-relative
executable path, archive/executable hashes, version and child phase identities in
`browser.json`; `version.log` preserves the version output. Successful calls remove
their own archive/runtime. Failures retain them for inspection. CI uploads only
the receipt/version log; command output remains in the job log. A setup-only pass
does not replace the actual tests. Direct Cargo calls bypass this prerequisite.

This dedicated runtime qualifies trusted test fixtures. The macOS artifact's
ad-hoc linker signature is not Developer ID authentication; no re-signing,
quarantine change or shared browser/updater mutation is performed. Production
untrusted-content and detached-process containment remain separately owned.
Exact source and qualification evidence:
`docs/tasks/artifacts/signoff_review/ci-browser-runtime.md`.

## CI environment launcher

`scripts/ci_env.py` now supplies the shared setup prerequisite. Without --rust it
sets the configured local stores and execs a command from the checkout root. With
--rust it reuses or provisions the exact numeric rust-toolchain.toml pin under
.project-data/installed-toolchains, with rustfmt/clippy and no rustup self-update.
Installed rustup/Python/OS tools are read-only inputs; downloads and scratch are
local before installation starts. An installer failure, timeout or terminal signal
is consumed before the launcher returns and prevents command dispatch.

```bash
python3 -B scripts/ci_env.py -- python3 -B -m unittest discover -s scripts/tests -p test_project_env.py -v
python3 -B scripts/ci_env.py --rust -- cargo fmt --all -- --check
```

CI clears inherited database/provider/demo/scanner and selected compiler overrides;
see docs/tasks/artifacts/signoff_review/ci-environment.md for the exact list and
executed controls. It is not a filesystem sandbox for explicit command paths.
The developer project_env launcher is unchanged. The shared launcher has eight
focused and eighteen adjacent passing controls. Scanner setup is now verified
below and all workflows now enter this launcher. Fifty Python controls pass
through its real command boundary; actual remote compiler installation and full
GitHub execution remain pending.

## Pinned scanner driver

`scripts/ci_scanners.py` downloads pinned cargo-deny 0.20.2 or Gitleaks 8.30.1
releases into an exclusive directory under target/ci-scanners. It verifies exact
archive size/SHA-256, bounds decoding and extracts only the expected regular
executable. Linux/macOS x86_64/aarch64 have explicit archive pins; unknown
platforms refuse. Curl default configuration and inherited TLS/QUIC log targets
cannot change the operation. Tool processes have bounded supervised lifetimes.

```bash
# Setup/version qualification only; these commands do not run security gates:
python3 -B scripts/project_env.py python3 -B scripts/ci_scanners.py cargo-deny --verify-only
python3 -B scripts/project_env.py python3 -B scripts/ci_scanners.py gitleaks --verify-only
# Actual gates, when the checkpoint reaches full execution:
python3 -B scripts/project_env.py python3 -B scripts/ci_scanners.py cargo-deny
python3 -B scripts/project_env.py python3 -B scripts/ci_scanners.py gitleaks
```

The gates preserve deny.toml policy and fresh online advisory behavior, and scan
Git history with full secret redaction. Scanner refusals remain nonzero. Read
scanner.json scope and exit_code: completed means execution ended, not necessarily
that a gate passed. Logs/receipts/redacted reports remain in the run directory;
normal consumed execution retires only its archive and executable. Failed setup
retains diagnostics and the last child identity; inspect actual process state
before cleanup. The driver uses the established project stores and explicit
selected compiler; installed OS tools remain read-only dependencies.

Thirteen focused controls and final affected configuration controls pass. All eight
archives pass checksum/layout checks; actual version probes pass on aarch64 macOS.
The first native Gitleaks check exposed a version-command format error, now fixed
with its failed log and successful retry retained. This is installation evidence;
no real security gate or remote workflow pass follows from it. Exact evidence and
limits: docs/tasks/artifacts/signoff_review/ci-scanners.md.

## Complete workflow wiring

Three parsed workflows contain six bounded project-command jobs, all entering
ci_env.py. Rust commands use a strict shell, two build jobs and no incremental
compilation in CI. The check job requires installed Chrome and built extraction/
browser workers before tests. pg-tests validates PostgreSQL 16, discovers all
Python modules and requests --demo explicitly. The book job installs mdBook 0.5.4
with --locked into .project-data/cargo and target/ci-mdbook-build, checks its exact
version and builds docs/book. Doctrine and secret jobs fetch complete history.

Scanner jobs execute actual gates and upload only scanner.json, version.log,
check.log and, for Gitleaks, gitleaks.json. Artifact temporary directories derive
from the checkout. GitHub-managed checkout/artifact metadata is an explicit
required platform boundary; installed OS tools are read-only dependencies. The
launcher does not sandbox arbitrary tool output paths.

Before upload wiring, a real Gitleaks synthetic-history probe returned one finding
and rc=1 with its value absent from both report and output. The initial alphabet
sample matched an example stopword; its zero-finding result was investigated and
retained before correcting the fixture. No actual credential was used. YAML/shell
checks, five omission controls and all fifty Python controls pass. These establish
local wiring and exercised redaction, not a full project security/remote CI pass.
Evidence: docs/tasks/artifacts/signoff_review/ci-workflows.md.

## Browser checkpoint follow-up

The source-7e01097 full checkpoint stops at two browser timing witnesses; its
PostgreSQL/demo gates are unstarted. A four-second render budget can expire during
startup, and an independent five-second prerequisite can prevent worker two from
launching. The repaired tests use observed gated navigation, preserve the delayed
second launch and require real simultaneous profiles plus consumed cleanup.

A longer real navigation exposes desktop Chrome's detached crash reporter/updater
writers holding stderr after browser-group exit. Do not qualify an installed
interactive browser merely from executable presence. Selected controls pass with
a verified repository-local Chrome for Testing 153.0.8010.36; its payload has an
ad-hoc linker signature, with official HTTPS/archive integrity and exact extracted
bytes checked separately. No Developer ID signature is claimed. Dedicated pinned
setup and local/workflow binding are owned by .11.4.3.1.2.5 before the next full run.
The current workflow still selects installed Chrome until that leaf is complete.
Evidence and remaining limits: docs/tasks/artifacts/signoff_review/browser-checkpoint-timing.md.

## Not a release claim

This skeleton deliberately stops short of the software-supply-chain *release* pipeline in
`ROADMAP.md` §16.10/§16.12:

- **No SBOM** generation and **no signed provenance** yet — those belong to the G9 release
  gate, once there are artifacts to attest to.
- **No release-signing**, reproducible builders, or protected release identities yet.
- **No release qualification claim:** the repository is public and must remain public.
  ADR-001 name clearance and the release gates remain separate outstanding requirements.

The pinned tool versions here (`gitleaks 8.30.1`) are a starting point and should be bumped
on a schedule. The workspace now has real dependencies; current deny.toml contains
reviewed duplicate/license exceptions. Every dependency checkpoint must evaluate
the actual locked graph and current advisory data without treating those
exceptions as blanket future approval.

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
