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
- **Active tree:** `PHASE-1` → frontier `.1.6` (Web UI/CLI, backlog 18; `proposed` —
  decompose or execute on pickup). `.1.5` COMPLETE (typed bodies + rounds + the
  honest `Inconclusive` close — backlog 17); **`PHASE-1-MAINT-2` done** — the
  stderr-drain race REPRODUCED and fixed (the EOF path awaits the drain,
  bounded, in `codex.rs` AND `claude.rs`); `.1.4` COMPLETE (Claude adapter
  live-qualified, backlogs 19–21); `.1`/`.1.2`/`.1.3` COMPLETE (backlogs 9–16);
  `PHASE-1-MAINT-1` done (§13).
- **Next action:** decompose `PHASE-1.6` (Web UI/CLI for threads, nodes, inbox,
  budgets, audit timeline — backlog 18) with a gap census.
- **Latest commit:** derive on read with `git log -1 --oneline`.
- **In-flight uncommitted work:** none after the `.1.4` decomposition commit (pending
  defect leaf `PHASE-1-MAINT-2`: one-off `codex_adapter` flake under parallel load —
  repro pending).
- **Push cadence:** every ~300 commits (director, 2026-09-06); run full CI before each push.
- **Local dev deps now installed (2026-09-06):** `postgresql@16`, `cargo-deny`, `gitleaks`,
  `mdbook` — plus `jq` for the demo script; `codex` (0.153.4) AND `claude` (2.1.263,
  Claude Code) for env-gated real runs.
- **Blockers:** none. Director-owned open items: license choice (`Cargo.toml` says
  `MIT OR Apache-2.0`, no `LICENSE` file); ADR-001 name clearance (repo stays private).
- **Dating anomaly (flagged):** host/git clock = 2026-09-06; some records from the previous
  session are dated 2026-09-07 inside 2026-09-06 commits. New records use the machine date.
