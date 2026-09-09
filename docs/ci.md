# CI and supply-chain checks

Phase 0 deliverable `PHASE-0.0.7` (roadmap backlog 8). This documents the automated
checks available beyond the discipline-spine baseline, and draws the line between a *skeleton*
and a *release claim*.

## What runs where

Three GitHub Actions workflows fire on every push and pull request:

| Workflow | Purpose | Local equivalent |
| --- | --- | --- |
| `rust` | format, clippy (deny warnings), test — `cargo test --all` covers core, the WP3 SQLite node journal + kill points + CLI, the WP4 fake-adapter behaviors + corpus integrity + supervisor flow (all file-based/in-process, no service), the server suites (skip offline), and the CLI's unit tests | `make check` |
| `rust` (job `pg-tests`) | the full local PostgreSQL collection in an owned Ubuntu 24.04 cluster: test-side ownership guard, server integration suites, MCP, CLI and crash/reconnect demonstration; repository-local compiler/cache stores | `bash scripts/run_pg_tests.sh` |
| `doctrines` | the 13-doctrine enforcer (same as the pre-commit hook) | `make gate` |
| `supply-chain` | `cargo deny` (advisories/bans/licenses/sources) + `gitleaks` secret scan | `make deny` / `make secret-scan` |

The first two are the discipline spine; `supply-chain` is what `.0.7` added.

## Commands

- `make check` — `cargo fmt --check` + `cargo clippy --all-targets --all-features -- -D warnings` + `cargo test --all`.
- `make gate` — the doctrine enforcer (`scripts/check_doctrines.sh`).
- `make deny` — `cargo deny check` against `deny.toml`. **Requires** `cargo-deny`
  (`python3 -B scripts/project_env.py cargo install --locked cargo-deny --version 0.20.2`).
  Policy: advisories, duplicate/wildcard versions, source allow-list and reviewed licenses
  with the narrow exceptions explicitly recorded in deny.toml.
- `make secret-scan` — `gitleaks detect --source . --redact`. **Requires** `gitleaks`
  as an installed read-only tool. This command scans Git history; it does not establish
  a scan of uncommitted working-tree files. `--redact` redacts secret values from output;
  findings and their metadata can still be reported.
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

Both `make deny` and `make secret-scan` are also wired into CI (`.github/workflows/supply-chain.yml`),
which installs the tooling itself, so they gate every push even on a machine that has not
installed them locally. The `pg-tests` job now invokes the supervised runner with the full local suite
list. It uses installed PostgreSQL 16 tools and provisions the pinned Rust
toolchain under `.project-data/installed-toolchains`. The workflow was reviewed
and syntax-checked locally; GitHub execution remains part of the next push.
Direct database-backed tests require the runner receipt; DATABASE_URL alone
refuses before fixture writes. See
`docs/decisions/2026-09-09_disposable-test-pool-ownership.md`.

## Scheduled pre-push checkpoint

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
The Rust check workflow still uses ambient writable stores, the supply-chain
container's stores require review, and none of the workflows invokes the Python
controls. Repairs are owned by SIGNOFF-REPAIR.11.4.3.1.3. Publisher fixture ownership
and browser profile/child lifetimes are checkpoint prerequisites .4/.5; safe
compiler-artifact cleanup is .6. Complete these before broad execution under .2.
The PG workflow's source has local stores; actual GitHub results remain to be
consumed after the authorized push. Local Make commands use project_env.py.

Exact census, tool versions, source hashes, skips and limits:
`docs/tasks/artifacts/signoff_review/ci-checkpoint-census.md`. Full CI runs before
pushes or selected important steps; ordinary slices use focused checks.

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
below; actual workflow wiring remains .11.4.3.1.3.3, so this is not yet a
corrected remote-CI claim.

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
limits: docs/tasks/artifacts/signoff_review/ci-scanners.md. Workflow wiring is next.

## Not a release claim

This skeleton deliberately stops short of the software-supply-chain *release* pipeline in
`ROADMAP.md` §16.10/§16.12:

- **No SBOM** generation and **no signed provenance** yet — those belong to the G9 release
  gate, once there are artifacts to attest to.
- **No release-signing**, reproducible builders, or protected release identities yet.
- **No public-release claim** of any kind: the repository remains private pending ADR-001
  (name clearance), and this file documents a working baseline, not a shipped capability.

The pinned tool versions here (`gitleaks 8.30.1`) are a starting point and should be bumped
on a schedule. The workspace now has real dependencies; current deny.toml contains
reviewed duplicate/license exceptions. Every dependency checkpoint must evaluate
the actual locked graph and current advisory data without treating those
exceptions as blanket future approval.
