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

Invoke direct diagnostic scripts through the launcher too, for example
`python3 -B scripts/project_env.py bash scripts/run_pg_tests.sh`. That existing
script runs the broad suite; focused PostgreSQL selection is the next repair.
The launcher controls default stores; explicit output paths and worker-specific
storage remain subject to the same-volume policy. It is not a filesystem sandbox.
Details and verification: `docs/decisions/2026-09-09_repository-local-command-environment.md`.

## The four binaries

```bash
make release
```

| Binary | Role |
| --- | --- |
| `rb` | the CLI — the primary surface (threads, enrollment, node ops, budget/audit inspection) |
| `rb-server` | the control plane — API + node channel + the embedded console at `/` (one listener, one binary) |
| `rb-node` | the node worker — outbound channel, SQLite journal, the adapter supervisor |
| `rb-journal` | the node-journal inspection tool |

The binaries are self-contained: the database migrations and the console
assets embed at compile time, so a deployed `rb-server` needs no runtime path
back to the checkout. The node's journal is a local SQLite file that stays on
the node's own volume.

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

