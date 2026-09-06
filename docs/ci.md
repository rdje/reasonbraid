# CI and supply-chain skeleton

Phase 0 deliverable `PHASE-0.0.7` (roadmap backlog 8). This documents the automated
checks available beyond the bedrock baseline, and draws the line between a *skeleton*
and a *release claim*.

## What runs where

Three GitHub Actions workflows fire on every push and pull request:

| Workflow | Purpose | Local equivalent |
| --- | --- | --- |
| `rust` | format, clippy (deny warnings), test — `cargo test --all` covers core, the WP3 SQLite node journal + kill points + CLI, the WP4 fake-adapter behaviors + corpus integrity + supervisor flow (all file-based/in-process, no service), the server suites (skip offline), and the CLI's unit tests | `make check` |
| `rust` (job `pg-tests`) | the PostgreSQL integration tests — atomic transaction (`.2.1`), leased outbox worker with fencing + kill points (`.2.2`), node channel with cursor resume + reconciliation handshake (`.3.2`), the authority engine with the enrollment-boundary ceiling + audit records (`.5.1`), the budget engine with reservations + denials (`.5.2`), the WP6 command API (`.6.1`), the node wiring (`.6.2`), the real-binary CLI end-to-end suite, and the two-host crash/reconnect demonstration (`scripts/demo_two_host.sh`) — against a PostgreSQL 16 service (`DATABASE_URL`) | `scripts/run_pg_tests.sh` |
| `doctrines` | the 13-doctrine enforcer (same as the pre-commit hook) | `make gate` |
| `supply-chain` | `cargo deny` (advisories/bans/licenses/sources) + `gitleaks` secret scan | `make deny` / `make secret-scan` |

The first two are the bedrock spine; `supply-chain` is what `.0.7` added.

## Commands

- `make check` — `cargo fmt --check` + `cargo clippy --all-targets --all-features -- -D warnings` + `cargo test --all`.
- `make gate` — the doctrine enforcer (`scripts/check_doctrines.sh`).
- `make deny` — `cargo deny check` against `deny.toml`. **Requires** `cargo-deny`
  (`cargo install cargo-deny`). Policy: advisories, duplicate/wildcard versions, source
  allow-list, permissive-only licenses.
- `make secret-scan` — `gitleaks detect --source . --redact`. **Requires** `gitleaks`
  (`brew install gitleaks`). Scans working tree and history for secrets; `--redact` keeps
  any finding out of the log.
- `bash scripts/run_pg_tests.sh` — the PostgreSQL proofs: `initdb` into a temp dir,
  start an ephemeral server on a throwaway port, run the seven server integration
  suites (atomic transaction, outbox-worker fencing/kill points, node channel
  reconnect/reconciliation, authority, budget, command API, node wiring) plus the
  real-binary CLI end-to-end suite, then the two-host demonstration
  (`scripts/demo_two_host.sh`, `RB_DEMO=0` to skip) — and tear everything down
  (no background service left running). **Requires** `postgresql@16`
  (`brew install postgresql@16`) and `jq`.
- The WP3 node journal tests and the WP4 adapter suites need no service at all:
  SQLite is a file and the fake/stub adapters are in-process, so the journal
  kill-point sweep, the `rb-journal` CLI tests, and the adapter/ supervisor tests run
  inside plain `cargo test --all` (`make check`) and the `rust` workflow above. The
  PG-backed suites require `DATABASE_URL` / `run_pg_tests.sh`; the REAL Codex
  qualification test stays `#[ignore]`-gated and is run deliberately with
  `RB_LIVE_CODEX=1` (it dispatches to the live harness and spends tokens).

Both `make deny` and `make secret-scan` are also wired into CI (`.github/workflows/supply-chain.yml`),
which installs the tooling itself, so they gate every push even on a machine that has not
installed them locally. The `pg-tests` job runs the same integration tests the local script
does, against a `postgres:16` service container.

## Not a release claim

This skeleton deliberately stops short of the software-supply-chain *release* pipeline in
`ROADMAP.md` §16.10/§16.12:

- **No SBOM** generation and **no signed provenance** yet — those belong to the G9 release
  gate, once there are artifacts to attest to.
- **No release-signing**, reproducible builders, or protected release identities yet.
- **No public-release claim** of any kind: the repository remains private pending ADR-001
  (name clearance), and this file documents a working baseline, not a shipped capability.

The pinned tool versions here (`gitleaks 8.30.1`) are a starting point and should be bumped
on a schedule; the `deny.toml` allow-lists are a conservative starting policy to be reviewed
the moment the first real dependency lands (WP1).
