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
- **Active tree:** `PHASE-1` → frontier `.1.6.1` (the budget read surface).
  `.1.6` DECOMPOSED `2026-09-06` after the gap census (direction: a vanilla
  static page served by `rb-server`, no build pipeline —
  `docs/decisions/2026-09-06_ui-direction.md`): `.1.6.1` budget read (the
  census-found gap — budgets had NO read surface anywhere), `.1.6.2` the
  embedded static shell (`web/{index.html,app.js,style.css}`, `/`), `.1.6.3`
  the demo/evidence leg. `.1.5` COMPLETE (backlogs 9–17 done across
  `.1`–`.1.5`); both Phase-1 defect leaves closed (`MAINT-1` §13,
  `MAINT-2` drain race); `.1.4` live-qualified (backlogs 19–21).
- **Next action:** execute `PHASE-1.6.1` — `GET /v1/threads/{id}/budget` +
  `rb inspect budget` — then `.1.6.2`/`.1.6.3`.
- **Latest commit:** derive on read with `git log -1 --oneline`.
- **In-flight uncommitted work:** none (MAINT-2 closed with the fix; 0 pending defect leaves).
- **Push cadence:** every ~300 commits (director, 2026-09-06); run full CI before each push.
- **Local dev deps now installed (2026-09-06):** `postgresql@16`, `cargo-deny`, `gitleaks`,
  `mdbook` — plus `jq` for the demo script; `codex` (0.153.4) AND `claude` (2.1.263,
  Claude Code) for env-gated real runs.
- **Blockers:** none. Director-owned open items: license choice (`Cargo.toml` says
  `MIT OR Apache-2.0`, no `LICENSE` file); ADR-001 name clearance (repo stays private).
- **Dating anomaly (flagged):** host/git clock = 2026-09-06; some records from the previous
  session are dated 2026-09-07 inside 2026-09-06 commits. New records use the machine date.
