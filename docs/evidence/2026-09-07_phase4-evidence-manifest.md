# Phase 4 evidence manifest (`PHASE-4.7.2`)

- Date: 2026-09-07 · Repository revision: `e0edfb9` at packaging time (this
  manifest ships in the `.7.2` close commit)
- Environment: macOS (arm64), Rust pinned **1.98.0** (`rust-toolchain.toml`),
  PostgreSQL 16.15 (ephemeral, on-volume per §13), Google Chrome (the R3
  worker's real-browser tests, the `R3_BROWSER_BIN`-overridable provenance)
- Gate: `ROADMAP.md` §20.6 exit gate — **G4** (unsupported, denied, mutable,
  or non-reproducible resources fail explicitly rather than becoming
  fabricated evidence)

## G4 evidence map

| G4 clause | Evidence | Where |
| --- | --- | --- |
| Unsupported resources fail explicitly | the resolve's `resource_unresolvable_now` (the unsupported scheme/type/kind refusals) | `tests/profiles.rs` 23 tests (the `.1.3`/`.4.3`/`.7.1` suites); the registry's filter-then-rank order |
| Denied destinations fail explicitly | the SSRF refusal matrix (the loopback/private/link-local/multicast/reserved/cloud-metadata classes each name themselves; the mapped-form re-classification) through EVERY pack's path | `src/ssrf.rs` 4 tests; `src/fetcher.rs` 17; `src/git.rs` 5; `src/extraction.rs` 1; the G4 suite (profiles 23) |
| Mutable resources are pinned, not quoted | the ADR-011 digests over the ACQUIRED bytes; the git resolved-commit recording; the replay semantics | `src/fetcher.rs`, `src/git.rs`, `src/snapshots.rs`; profiles 15/16/19 |
| Non-reproducible submissions are refused | the digest-mismatch 400s (the snapshots + the derivations), the fake-excerpt citation refusal | `src/snapshots.rs`, `src/derivations.rs`, `src/claims.rs`; profiles 19–21 |
| Hostile content is refused, named | the worker refusals (the zip bomb via the ratio brake, the traversal, the nested archives, the encrypted/JS PDFs, the LFS pointers, the budget trips) + the G4 suite's eight scenarios | `crates/reasonbraid-extract` 9 tests; `crates/reasonbraid-browse` 2 (the step-budget refusal before any navigation); profiles 23 |
| The fabrication boundary holds | the tombstone rule (the deletion is never silent), the derivation edges (a quote is never the original), the citation validation (the excerpt must be in the bytes) | `src/snapshots.rs`, `src/derivations.rs`, `src/claims.rs`; profiles 19/20/21/22 |

## Supply-chain re-run (the `.7.2` close)

| Check | Result | Where |
| --- | --- | --- |
| `make deny` | rc=0 — `advisories ok, bans ok, licenses ok, sources ok` (the R2/R3 pack crates' duplicates reviewed + skipped with the rationale; the uluru MPL-2.0 exception narrowed to the one crate) | `deny.toml`; CI `supply-chain.yml` |
| `make secret-scan` | rc=0 — `no leaks found` (163 commits scanned) | CI `supply-chain.yml` |
| Offline baseline | `cargo test --all` → rc=0, 55 suites | `target/offline471.log` |
| Live baseline | `bash scripts/run_pg_tests.sh` → rc=0, 18 live suites + the demo `ALL acceptance checks passed` | `target/pg471_guard.log` |
| The G4 suite | `DATABASE_URL=… cargo test -p reasonbraid-server --test profiles the_g4_hostile_suite` → `test result: ok. 1 passed` | the `.7.1` leaf's checklist |
