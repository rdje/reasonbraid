# Phase 5 evidence manifest (`PHASE-5.6.2`)

- Date: 2026-09-07 · Repository revision: the `.6.2` close commit (this
  manifest ships in it)
- Environment: macOS (arm64), Rust pinned **1.98.0**
  (`rust-toolchain.toml`), PostgreSQL 16.15 (ephemeral, on-volume per
  §13)
- Gate: `ROADMAP.md` §20.6 exit gate — **G5** (benchmark thresholds and
  honest inconclusive behavior, blocking the "deliberation improves
  answers" claim)

## G5 evidence map

| G5 clause | Evidence | Where |
| --- | --- | --- |
| The honest inconclusive behavior | the twelve §13.4 terminals with the family rule (a decision terminal with an unresolved register is the typed refusal); the unresolved register riding the close; the budget-exhausted denial's honest `Inconclusive` terminal | `tests/profiles.rs` 31 tests (`.2.4.1`, `.2.4.2`, `.1.5.3`-family); `tests/command_api.rs` (the honest-close suite) |
| The benchmark thresholds exist as an instrument | the WP7 harness's scripted self-test (the corpus's expected scores re-derived — the measurement pipeline proof); the `.4` service's calibration + the baseline/threshold gates with the append-only evaluations | `crates/reasonbraid-adapter/tests/bench_harness.rs`; `tests/evaluation.rs` 3 tests (`.4.2`–`.4.4`) |
| The first controlled evaluation | the 4-case differential live run: H1 null (no structured workflow beat `single` at 2–4× the cost), the run-to-run variance finding, the Brier 0 sample | `docs/evidence/2026-09-07_benchmark-codex-run.md` |
| The claim the gate blocks | the product makes no "deliberation improves answers" claim — the census found the README carries only the mechanism claim; the subtraction record withdraws the lift claim | `docs/decisions/2026-09-07_phase5-subtraction-record.md` |
| The routing stays on the rule-based baseline | the seven §13.8 rules; the shadow recommendation recorded + never applied (the shadow proof); the audited resolutions | `tests/routing.rs` 2 tests (`.5.2`, `.5.3`) |
| The deliberation machinery is bounded | the profile composition (never a capability), the closed moderation vocabulary (the prohibitions by construction), the re-derivable synthesis, the blind read-surface rule | `tests/profiles.rs` 31 tests across `.1`–`.3`; ADRs 016, 029, 030, 031 |

## The guard re-run (the `.6.2` close)

- `cargo test --all` → rc=0, 57 suites (offline)
- `bash scripts/run_pg_tests.sh` → rc=0, 20 live suites + the demo
  `ALL acceptance checks passed`
- `cargo clippy --all --all-targets -- -D warnings` → rc=0;
  `cargo fmt --all -- --check` → rc=0; `make gate` → 13/13;
  `make book` builds; `make deny` + `make secret-scan` green (the
  prior-lane exceptions recorded in `deny.toml`)
