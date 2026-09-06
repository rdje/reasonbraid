# 2026-09-06_pinned-toolchain.md

## Context

`PHASE-1-MAINT-3` (discovered during the `.1.6.1` verification): `cargo fmt
--all -- --check` failed on hunks in files the leaf never touched (`claude.rs`,
`claude_adapter.rs`, `state.rs`, `invitations.rs`) — under BOTH installed
rustfmt builds (rustc 1.95.0's `1.9.0-stable (59807616e1 2026-04-14)` and
1.98.0's `1.9.0-stable (88d9e12ae1 2026-08-18)`). HEAD was simply not
fmt-clean under the current stable channel.

## Decision

- **Pin the toolchain version.** `rust-toolchain.toml` moves from
  `channel = "stable"` to `channel = "1.98.0"` (the newest locally installed),
  and the CI workflow's `dtolnay/rust-toolchain` steps name
  `toolchain: 1.98.0` explicitly. Formatting and linting become reproducible:
  local and CI use the same rustfmt/clippy, and future channel moves are an
  explicit, reviewed bump — not a silent re-format.
- **Normalize once under the pin** (`cargo fmt --all`) rather than pinning
  older formatting.
- Observed but out of scope: rust.yml's PG job omits the post-`.2.1` suites
  (`aggregate_library`, `identity_store`, `node_enrollment`, `node_inbox`,
  `invitations`) — the §16 full-CI-before-push step reconciles the CI suite
  lists at the first push.

## Consequences

- Every future `make check`/CI fmt run is deterministic against the pin.
- Toolchain bumps become a decision: update the pin + record the version that
  was validated.

answers:

- **`stable` is a moving pointer, not a toolchain.** Two rustfmt builds with
  the SAME version string disagree across rustc releases; only a pinned
  channel converts "formatted once" into "formatted reproducibly".
- **A toolchain move is a change.** It deserves the same leaf/checklist
  treatment as a dependency bump, because it can re-format the whole tree.
