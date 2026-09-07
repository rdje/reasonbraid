# Phase 1 evidence manifest (`PHASE-1.8.2`)

- Date: 2026-09-07 · Repository revision: `ecceb8d` at packaging time (this
  manifest ships in the `.1.8.2` close commit)
- Environment: macOS (arm64), Rust pinned **1.98.0** (`rust-toolchain.toml`),
  PostgreSQL 16.15 (ephemeral, on-volume per §13), SQLite 3 (WAL,
  `synchronous=FULL`), claude 2.1.263 + codex 0.153.4 for env-gated real runs
- Gate: `ROADMAP.md` §20.3 exit gate — **G1** (unit/property baseline;
  dependency + license checks) and **G2** (real durable stores, node journal,
  an adapter, a recovery demonstration); Demonstration A (`ROADMAP.md` §26.1)

## G1 evidence map

| G1 clause | Evidence | Where |
| --- | --- | --- |
| Unit baseline | 39 offline suites green (`cargo test --all`, rc=0) + 12 live-PG server suites + the real-binary CLI e2e | `target/gate82_offline.log`; `target/gate82_guard.log` (4+5+9+5+13+3+4+17+3+3+6+7 + 2) |
| Property-flavored baseline | exhaustive state-machine transition tables (`.1.5.3` extended), the grant-registry canaries, the WP2/WP3 kill-point sweeps (KP-1…KP-9, `journal_kill_points.rs` 11 tests) | `crates/reasonbraid-core/src/state.rs`, `authority.rs`; `tests/atomic_transaction.rs` (5), `tests/outbox_worker.rs` (7) |
| Dependency + license checks | `make deny` → rc=0: `advisories ok, bans ok, licenses ok, sources ok` | `target/gate82_deny.log`; `deny.toml`; CI `.github/workflows/supply-chain.yml` |
| Secret scan | `make secret-scan` → rc=0, `no leaks found` (66 commits scanned) | `target/gate82_gitleaks.log`; CI `supply-chain.yml` (pinned gitleaks 8.30.1) |
| Fuzz | **not yet applicable** — no untrusted-parser surface exists before Phase 4's resource packs; recorded as a named deferral (trigger: the first resolver parser) | gate record `docs/decisions/2026-09-07_phase1-gate-record.md` |

## G2 evidence map

| G2 clause | Evidence | Where |
| --- | --- | --- |
| Real durable stores | PostgreSQL (migrations 0001–0010, aggregate/event/outbox/idempotency in one transaction) + per-node SQLite journals (WAL + `synchronous=FULL`) | `tests/atomic_transaction.rs`, `tests/outbox_worker.rs`; WP3 kill points; `rb-journal` |
| Node journal | durable boundary records, honest `outcome_unknown`, read-only inspection CLI | `crates/reasonbraid-node`; `docs/decisions/2026-09-06_node-journal.md` |
| An adapter (two real + the deterministic fake) | Codex CLI (`.4.2`), Claude CLI (`.1.4.1`/`.1.4.2`, live-qualified), the scripted fake as the CI oracle | `crates/reasonbraid-adapter`; `target/claude_live.log`; `tests/codex_adapter.rs` (9), `tests/claude_adapter.rs` (10) |
| Recovery demonstration | the two-host demo with REAL kill points — server SIGKILL + restart (no accepted command lost), node SIGKILL after dispatch (exactly one `outcome_unknown`, no silent retry, no effect), duplicate transport (one domain effect) | `scripts/demo_two_host.sh` sections 4–7; bundle `target/demo/20260907-021558/` |

## Demonstration A — the §26.1 acceptance map

| §26.1 acceptance | Evidence (all re-runnable) |
| --- | --- |
| no manual message relaying between agents | demo check: "the agent content is the adapter's scripted chunks (no human relay)" — the contribution content comes from the fake adapter's script, never the operator's keyboard |
| restart/reconnect loses no accepted command and creates no duplicate domain effect | demo sections 4–7: the duplicate re-POST answers `accepted:false` and exactly ONE contribution exists; the server SIGKILL + restart preserves every accepted command; the node SIGKILL after dispatch recovers exactly one `outcome_unknown` with no revision entered |
| offline inbox and resume cursor work | demo sections 6–7: node A is stopped so the revise work queues in its inbox; the restart re-handshakes (cursor + pending ops) and picks it up; `tests/node_inbox.rs` (3), `tests/node_channel.rs` (17) |
| agent role, incarnation, harness, model/provider route, and attempt remain distinguishable | branded id families (`ten`/`hpr`/`hst`/`nod`/`rol`/`inc`/`run`/`patt` — `docs/decisions/2026-09-06_id-representation.md`), the 0007 identity hierarchy, attempt records with provider handles; the incarnation/run row WRITERS are a named deferral → Phase 2 identity (the schema + id space + attempt records exist today) |
| spend and uncertainty are visible | `GET /v1/threads/{id}/budget` (`.1.6.1`) — ceiling + every reservation row (held vs settled usage, denials with the engine's reasons); demo sections 10–11 fetch the live ledger; THREAD_B's denial row is asserted with its reason |
| the system can conclude `inconclusive` with minority/unresolved items | `.1.5.3` core `Inconclusive` terminal + the close body's `outcome`/`unresolved`; demo THREAD_B closes inconclusively with the register asserted |
| all state is inspectable through supported CLI/UI, not database surgery | every demo state assertion runs through the `rb` CLI, the control API (curl), the console's read surfaces, and `rb-journal`; the two psql reads obtain the fencing token to forge the duplicate transport (a credential oracle — documented in the script; exposing a live credential through the API would violate least privilege) |

Scenario-step deferrals (named, with triggers — full table in the gate
record): capability advertisement → Phase 3 directory; expected-artifact +
manual-decision-rule create fields → Phase 5 workflow/decision rules; LLM
synthesis → Phase 5 (the demo's "synthesis + unresolved register" is the
register today, honestly labelled); incarnation/run row writers → Phase 2
identity.

## Reproducible commands

```bash
make check                        # fmt + clippy + all offline suites (pinned 1.98.0)
bash scripts/run_pg_tests.sh      # 12 live server suites + CLI e2e + the two-host demo (ephemeral PG)
make demo                         # same, one target
make release                      # the four self-contained binaries
# the release-built demo (the packaging proof — the demo itself boots nothing):
#   ephemeral PG on-volume (§13), then:
bash scripts/demo_two_host.sh --database-url postgres://… --release
make gate && make deny && make secret-scan && make book
bash scripts/dev.sh --check       # the one-command dev environment self-verification beat
RB_LIVE_CLAUDE=1 cargo test -p reasonbraid-node --test claude_live -- --ignored
```

## Failures and their dispositions (all fixed with regression coverage)

1. The `.1.8.1` beat mis-read (thread-scoped audit view starts at the invite —
   `thread.create` authorizes at tenant scope): fixed, the beat asserts the
   `command_api` suite's adjudicated shape. `target/demo81_guard.log`.
2. The `.1.7.1` dev-beat authoring slips (verb shape, `--as` requirement,
   census ordering): fixed, `dev.sh --check` green.
3. The stderr-drain race (`.1.5.3` verification, fixed in both adapters):
   `docs/decisions/2026-09-06_stderr-drain-race.md`.
4. The toolchain drift (unpinned `stable` moved rustfmt): pinned 1.98.0,
   `docs/decisions/2026-09-06_pinned-toolchain.md`.
5. Full leaf-level failure history: the tree's Verification Log
   (`docs/tasks/PHASE-1.md`) — every leaf's first-run findings are recorded
   there with their fixes.
