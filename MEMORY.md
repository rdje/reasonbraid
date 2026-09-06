# MEMORY — resume pointer (layer A; overwrite-only, keep ≤ ~50 lines)

> The bounded layer-A resume pointer (see `MEMORY_ARCHITECTURE.md`). OVERWRITE the
> "Current state" block each update — never append history here.

## How to resume

1. Read `README.md`, `MEMORY_ARCHITECTURE.md`, `TOOLBOX.md`, `DOCTRINE_ENFORCEMENT.md`.
2. Open the active task-tree below → its Current Frontier → continue from the next action.
3. Scope/gates: `ROADMAP.md`. Phase 0 execution: companion `KICKOFF.md`.

## Current state

- **Project:** ReasonBraid (working name, not legally cleared).
- **Active tree:** `PHASE-0` → frontier leaf `PHASE-0.5.1` (`pending`); WP1 + WP2 + WP3 +
  **WP4 complete** — `.4.1` adapter contract + deterministic fake + sanitized corpus +
  supervisor, `.4.2` the first REAL harness (Codex-family CLI, `codex exec --json`,
  qualified live 2026-09-06; second adapter recommendation: Phase 1, director-owned).
- **Next action:** WP5 development `EnrollmentAuthorityBoundary` + scoped commands +
  authorization audit record (`.5.1`).
- **Latest commit:** derive on read with `git log -1 --oneline`.
- **In-flight uncommitted work:** none after `PHASE-0.4.2`.
- **Push cadence:** every ~300 commits (director, 2026-09-06); run full CI before each push.
- **Local dev deps now installed (2026-09-06):** `postgresql@16`, `cargo-deny`, `gitleaks`, `mdbook` — the full `make check`/`gate`/`deny`/`secret-scan`/`book` stack runs locally.
- **Blockers:** none.
