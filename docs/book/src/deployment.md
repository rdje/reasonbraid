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
pass, using an instrumented installer; workflow wiring and actual remote compiler
installation remain pending. The exact contract and evidence are in
`docs/tasks/artifacts/signoff_review/ci-environment.md`.

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
deployed `rb-server` needs no runtime path back to the checkout. The `rb-site`
operator tool runs from within the checkout to verify storage locality. The node's
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
