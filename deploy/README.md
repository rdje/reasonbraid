# deploy — ReasonBraid local/LAN deployment package

The Phase 1 deployment story (`.1.7.2`): four self-contained binaries and the
runbook that stands a control plane + nodes up on a trusted LAN. This is the
packaging deliverable named by `ROADMAP.md` §20.3; the product documentation
lives in the mdBook (`deployment` chapter), this file is the operator surface.

## The package

```bash
make release
```

produces four binaries under `target/release/`:

| Binary | Role |
| --- | --- |
| `rb` | the CLI — the primary surface (threads, enrollment, node ops, budget/audit inspection) |
| `rb-server` | the control plane — API + node channel + the embedded console at `/` (one listener, one binary) |
| `rb-node` | the node worker — outbound channel, SQLite journal, the adapter supervisor |
| `rb-journal` | the node-journal inspection tool (`inspect`/`pending`/`ambiguous`) |

The binaries are **self-contained**: the database migrations and the console
assets embed at compile time (`include_str!`/`sqlx::migrate!`), so a deployed
`rb-server` needs no runtime path back to this checkout (§12). The node's
journal is a local SQLite file (WAL, `synchronous=FULL`) — device-local state
that stays on the node's own volume.

## Install

- **Build on one host, copy:** `make release`, then `scp` the four binaries.
- **Per-crate install:** `cargo install --path crates/reasonbraid-cli` (and the
  `-server`/`-node` crates) — same artifacts, Cargo-managed.

## Profiles (ROADMAP §6.6)

- **Developer** (loopback, one host): `make dev` — the `.1.7.1` one-command
  development environment (ephemeral on-volume PostgreSQL + `rb-server`).
- **Trusted LAN** (this runbook): enrolled nodes on a private network, dev
  trust store — no binding governance claims, no public exposure.

## Control-plane host

1. Provide PostgreSQL 16 (`postgresql@16`) and a database:
   `createdb -U postgres reasonbraid`.
2. Run (migrations apply on startup):
   `target/release/rb-server --host 0.0.0.0 --port 4310 --database-url postgres://postgres@<cp-host>:5432/reasonbraid?sslmode=disable`
   - `--host 0.0.0.0` binds the LAN; the default is loopback (the dev profile).
   - the console serves at `http://<cp-host>:4310/` from the same listener.
3. Enroll principals: `target/release/rb --server http://<cp-host>:4310 enroll human <name>`.

## Node hosts

1. Copy `rb-node` + `rb-journal` to the host (or `cargo install --path
   crates/reasonbraid-node`).
2. On the control-plane host, issue a one-time enrollment token:
   `rb node issue-token --node-id <node-id> --host-claim <claim>` (prints the
   token + nonce).
3. On the node host, enroll then run:
   `rb-node --server http://<cp-host>:4310 --enroll-token <token> --enroll-nonce <nonce> --node-id <node-id> --node-secret <secret> --journal <node-local-dir>/node.db`
   (subsequent runs omit the enrollment flags; the journal + secret are the
   node's durable identity).
4. The adapter: the deterministic fake (`--fake-script '[{"step":…}]') for
   ops practice and CI; the real harness legs are the env-gated
   `RB_LIVE_CODEX=1`/`RB_LIVE_CLAUDE=1` adapter suites (Phase 0).

## The reference exercise

The demo script carries a real two-host mode — the packaging proof is the demo
on release binaries, locally or across hosts:

```bash
# local, release-built (PHASE-1.7.2)
bash scripts/demo_two_host.sh --database-url <url> --release
# two real hosts
bash scripts/demo_two_host.sh --database-url <url> --release \
    --node-host <h2> --remote-workdir '~/rb-demo'
```

The ssh mode copies `rb-node` + `rb-journal` to the remote scratch directory
(caller-authorized) and keeps the journals there; `psql` must be reachable
where the script runs. The bundle's `env.txt` records the mode and the build.

## Honest limits (Phase 1 — trusted LAN only)

- **Dev trust store:** the channel credential is the shared secret registered
  at enrollment, proved by the HMAC key-proof handshake. X.509/mTLS workload
  identity is ADR-006/ADR-007 (Phase 2).
- **Plain HTTP:** no TLS on the wire — trusted LAN only. Public exposure is
  the G6 Internet-qualification gate, not a flag flip.
- **No directory:** a node id is the agent-role wire id it serves (one node,
  one role).
- **No config files, no process supervision, no containers, no PG install
  automation:** everything is flags/env; launchd/systemd units and packaged
  images arrive with Phase 2 operations (§20.4). See the subtraction record.

## Subtraction record (`PHASE-1.7.2`)

| Deferred | Trigger to revisit |
| --- | --- |
| Config files (server/node) | when the flag surface exceeds the operator's copy-paste tolerance (Phase 2 ops) |
| launchd/systemd supervision units | with Phase 2's restart/upgrade work (§20.4) |
| TLS/mTLS on the wire | ADR-006/ADR-007 workload identity (Phase 2) |
| Container images | measured need; the modular-monolith single binary is the Phase 1 package |
| PostgreSQL installation automation | the control-plane host operator owns the database service |
| Public Internet exposure | the G6 gate (ROADMAP §16.12) |
