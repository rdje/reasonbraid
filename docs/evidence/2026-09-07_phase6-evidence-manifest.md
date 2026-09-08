# Phase 6 evidence manifest (`PHASE-6.7.2`)

- Date: 2026-09-07 · Repository revision: the `.7.2` close commit (this
  manifest ships in it)
- Environment: macOS (arm64), Rust pinned **1.98.0**
  (`rust-toolchain.toml`), PostgreSQL 16.15 (ephemeral, on-volume per
  §13)
- Gate: `ROADMAP.md` §20.6 exit gate — **G3** (the authority/consent/
  quorum/publication/correction tests, blocking the binding policy use)

## G3 evidence map

| G3 clause | Evidence | Where |
| --- | --- | --- |
| The authority | the grant re-checks at the decision/approval/correction boundaries (the status + the expiry + the subject match — the §4.5 action-time facts); the policy ownership's active-grant check | `tests/policy.rs` (policy 4/10/1); the `.2`/`.5.3` lanes |
| The consent | the invitation-response machinery (the invite → the accept/dispel dispatch; the capability is the invitation) | the Phase-1 suites |
| The quorum | the electorate snapshots frozen at the action time (the participants + the denominator + the abstentions) | `tests/policy.rs` (policy 3) |
| The publication | the nine-step §15.7 machine, the compare-and-swap idempotency, the six-row §15.8 reconciliation (the never-silent-promote), the byte-identical projections | `tests/policy.rs` (policy 7/8); `tests/publisher.rs` 2; `tests/reconciler.rs` 3; `reasonbraid-policy-compiler` 8 |
| The correction | the §4.7 operations with the authority proofs (the expiring suspension/waiver, the linked supersession, the preserving retraction) | `tests/policy.rs` (policy 10) |
| The blocker (the binding use) | the claim is WITHDRAWN per §25.1 — the exit claims the machinery; the owners' acceptance is the open condition | `docs/decisions/2026-09-07_phase6-gate-record.md` + the subtraction record |

## The guard re-run (the `.7.2` close)

- `cargo test --all` → rc=0, 63 suites (offline)
- `bash scripts/run_pg_tests.sh` → rc=0, 21 live suites + the demo
  `ALL acceptance checks passed`
- `cargo clippy --all --all-targets -- -D warnings` → rc=0;
  `cargo fmt --all -- --check` → rc=0; `make gate` → 13/13;
  `make book` builds; `make deny` + `make secret-scan` green (the
  prior-lane exceptions recorded in `deny.toml`)
