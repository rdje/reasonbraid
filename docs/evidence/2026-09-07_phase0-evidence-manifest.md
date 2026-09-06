# Phase 0 evidence manifest (`PHASE-0.8.1`)

- Date: 2026-09-07 · Repository revision: `657be6e` (WP1–WP7 complete; this
  manifest ships in the WP8 commit)
- Environment: macOS (arm64), Rust stable, PostgreSQL 16.15 (ephemeral, port
  55432/55433), SQLite 3 (WAL), codex-cli 0.153.4 for env-gated real runs
- Gate: `ROADMAP.md` §20.2 exit gate — G0 passes for **identity, authority,
  thread, delivery, budget**; no unresolved experiment threatens the Phase 1
  vertical slice.

## G0 evidence map

| G0 boundary | Evidence | Where |
| --- | --- | --- |
| Identity | strong branded IDs (14 families), enrollment with durable name→principal rows, deterministic actor handles, roles outlive models | `.1.1`/`.5.1`/`.6.1`; `crates/reasonbraid-core/src/id.rs`; `tests/command_api.rs` |
| Authority | enrollment boundary = ceiling, subset-checked grants, membership grants nothing, every decision (allow AND deny) audited with a re-derivable policy digest, budget-denied dispatches refused at both boundaries | `.5.1`/`.5.2`; `tests/authority.rs` (9 passed); `tests/budget.rs` (5 passed) |
| Thread | core state machines (`open→closing→closed`, participation, provider-attempt) validate every transition; one transaction per command (claim → authorize → validate locked projection → apply); closure preserves contributions + the unresolved register; inspection through the API only | `.1.3`/`.6.1`/`.6.2`; `tests/command_api.rs` (7) + `tests/node_work.rs` (6) |
| Delivery | atomic state/event/idempotency/outbox transaction; leased outbox worker with fencing + kill points 3–5; node journal (WAL + synchronous=FULL) with boundary-before-boundary ordering and honest `outcome_unknown`; cursor-resume channel with reconciliation; server→inbox dispatch in the command transaction; duplicate transport → one domain effect at both layers | `.2.1`/`.2.2`/`.3.1`/`.3.2`/`.6.2`; `tests/atomic_transaction.rs` (5), `tests/outbox_worker.rs` (7), `tests/node_channel.rs` (13), journal kill points (KP-1…KP-9), `scripts/demo_two_host.sh` (12 acceptance checks) |
| Budget | multi-dimensional ceilings, atomic reservations against held amount, denial rows, settlement with actual usage (overruns reported, never clamped), node-side dispatch gate (`failed_before_dispatch` before any provider contact), exhausted ceilings prevent new dispatches (demo leg 8) | `.5.2`; `tests/budget.rs` + `supervisor_budget` (4) + `node_work.rs` (budget-denial test) + demo |

## Reproducible commands

```bash
make check                        # fmt + clippy + all offline suites
bash scripts/run_pg_tests.sh      # 7 server suites + CLI e2e + the two-host demo (ephemeral PG)
make demo                         # same, one target
make gate && make deny && make secret-scan && make book
RB_LIVE_CODEX=1 target/debug/rb-bench --agent codex \
    --case fact-001,code-002,policy-002,insuff-001 --max-calls 36   # real benchmark run
```

## Fixtures and measurements

- WP4 sanitized outcome corpus: `crates/reasonbraid-adapter/fixtures/*.json`
  (10 fixtures, mechanically credential-scanned; `corpus integrity` suite).
- WP7 benchmark corpus v1: `crates/reasonbraid-adapter/bench/v1/` (8 cases,
  4 classes; SHA-256 digests ride every report — the scripted self-test
  reproduces the corpus oracle exactly, 8×4).
- Real measurements: two-host demo ≈ 5 s end-to-end (fake adapter); real Codex
  calls 7–42 s wall each with ~16k ambient input tokens per call (user Codex
  config) — recorded in `docs/evidence/2026-09-07_benchmark-codex-run.md`.

## Failures and their dispositions (all fixed with regression coverage)

1. WP2–WP6 fixture/harness races and the subset-checker wall-clock class
   (`.5.1` fixtures; `.6.1` dev-grant window) — fixed, evidence in the leaves.
2. `.6.2`: revise-target inversion, `node_work` purge race, demo `$$`-pid bug —
   fixed (`2026-09-07_node-channel-wiring.md`).
3. `.7`: the FIRST real run falsified the harness — revision leg re-rendered the
   CRITIQUE template (all revisions structure-invalid, one answer broken), and
   the honesty trap flagged echoed statement numbers — fixed with regression
   tests (`2026-09-07_deliberation-benchmark.md`).
4. Open by design: provider ambiguity stays `outcome_unknown` until proof or
   adjudication (no silent retry — the acceptance, not a defect).

## Remaining open items (none threaten the Phase 1 slice)

- Working-name clearance (ADR-001) and the license choice (director-owned; see
  the tree's Open Questions) — pre-release gates, not slice blockers.
- The second real adapter (Claude-family) — recommended for Phase 1.
- The README_POLICY upstream revision review — owned by `PHASE-0-MAINT-1`.
