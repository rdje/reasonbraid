# MEMORY — resume pointer (layer A; overwrite-only, keep ≤ ~50 lines)

> The bounded layer-A resume pointer (see `MEMORY_ARCHITECTURE.md`). OVERWRITE the
> "Current state" block each update — never append history here.

## How to resume

1. Read `README.md`, `MEMORY_ARCHITECTURE.md`, `TOOLBOX.md`, `DOCTRINE_ENFORCEMENT.md`.
2. Open the active task-tree below → its Current Frontier → continue from the next action.
3. Scope/gates: `ROADMAP.md`. Phase 1 execution: the `PHASE-1` tree under `docs/tasks/`.

## Current state

- **Project:** ReasonBraid (working name, not legally cleared).
- **Phase 0 COMPLETE:** the exit gate is closed (ADR-002 signed by the accountable owner —
  `docs/adr/002-phase1-scope.md`, GO) and the PHASE-0 tree is `done`: WP1–WP8 +
  `MAINT-1` (README_POLICY re-adopted: derived caps + routing-pressure closure) +
  `MAINT-2` (ReasonBraid-only naming — zero scaffold-name tokens remain).
- **Active tree:** `PHASE-1` → frontier `.1.8.2` (`.1.8.1` done — the
  audit-reconstruction leg; `.1.8.2` closes Phase 1). **`.1.7` COMPLETE** — `.1.7.1` `make dev` (the
  one-command dev loop, `--check` beat) + `.1.7.2` the packaged LAN story
  (`make release` → four self-contained binaries, `deploy/` runbook, the
  book's `deployment` chapter, the release-built demo proof 24/24;
  `docs/decisions/2026-09-07_deployment-packaging.md`).
  **`.1.6` COMPLETE** (backlog 18) —
  `.1.6.1` budget read surface (`docs/decisions/2026-09-06_budget-read-surface.md`),
  `.1.6.2` the embedded static shell at `/` (read-only, text-safe;
  `docs/decisions/2026-09-06_ui-embedding.md`), `.1.6.3` the demo's console
  beat. Direction: a vanilla static page served by `rb-server`, no build
  pipeline (`docs/decisions/2026-09-06_ui-direction.md`). Toolchain PINNED to
  `1.98.0` (`PHASE-1-MAINT-3`). `.1.5` COMPLETE (backlogs 9–17 done); all
  three Phase-1 defect leaves closed (`MAINT-1` §13, `MAINT-2` drain race,
  `MAINT-3` toolchain pin); `.1.4` live-qualified.
- **Next action:** execute `.1.8.2` (the G1–G2 gate package: `make deny` +
  `make secret-scan`, the evidence manifest + gate record + subtraction
  record, the full guard re-run, the Phase 1 close), then `PHASE-2.1`.
- **Latest commit:** derive on read with `git log -1 --oneline`.
- **Defects:** 0 tracked leaves outstanding (`MAINT-1` §13 locality, `MAINT-2`
  drain race, `MAINT-3` toolchain pin — all closed).
- **In-flight uncommitted work:** none.
- **Push cadence:** every ~300 commits (director, 2026-09-06); run full CI before each push.
- **Local dev deps now installed (2026-09-06):** `postgresql@16`, `cargo-deny`, `gitleaks`,
  `mdbook` — plus `jq` for the demo script; `codex` (0.153.4) AND `claude` (2.1.263,
  Claude Code) for env-gated real runs.
- **Blockers:** none. Director-owned open items: license choice (`Cargo.toml` says
  `MIT OR Apache-2.0`, no `LICENSE` file); ADR-001 name clearance (repo stays private).
- **Dating anomaly (flagged):** the machine clock crossed midnight mid-session
  (`2026-09-06` → `2026-09-07`); new records use the machine date. The earlier
  anomaly stands historically: some previous-session records are dated
  2026-09-07 inside 2026-09-06 commits.
