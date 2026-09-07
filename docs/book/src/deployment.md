# Deployment — local and LAN

The Phase 1 deployment package (`.1.7.2`) is **four self-contained binaries**
and a runbook. This chapter is the product view; the operator surface is the
`deploy/README.md` runbook in the repository.

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
