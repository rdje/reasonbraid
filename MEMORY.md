# MEMORY — resume pointer (layer A; overwrite-only, keep ≤ ~50 lines)

> The bounded layer-A resume pointer (see `MEMORY_ARCHITECTURE.md`). OVERWRITE the
> "Current state" block each update — never append history here.

## How to resume

1. Read `README.md`, `MEMORY_ARCHITECTURE.md`, `TOOLBOX.md`, `DOCTRINE_ENFORCEMENT.md`.
2. Open the active task-tree below → its Current Frontier → continue from the next action.
3. Scope/gates: `ROADMAP.md`. Phase 0 execution: companion `KICKOFF.md`.

## Current state

- **Project:** ReasonBraid (working name, not legally cleared).
- **Active tree:** `PHASE-0` → frontier leaf `PHASE-0.6.1` (`pending`); WP1 + WP2 + WP3 +
  WP4 + **WP5 complete** — `.5.1` authority engine (boundary ceiling, scoped grants,
  audited decisions), `.5.2` budget engine (reserve-before-dispatch at BOTH
  boundaries; overruns recorded; indeterminate attempts keep their hold).
- **Next action:** WP6 CLI — enroll, create thread, invite, contribute, challenge,
  revise, close, inspect (`.6.1`).
- **Latest commit:** derive on read with `git log -1 --oneline`.
- **In-flight uncommitted work:** none after `PHASE-0.5.2`.
- **Push cadence:** every ~300 commits (director, 2026-09-06); run full CI before each push.
- **Local dev deps now installed (2026-09-06):** `postgresql@16`, `cargo-deny`, `gitleaks`, `mdbook` — the full `make check`/`gate`/`deny`/`secret-scan`/`book` stack runs locally.
- **Blockers:** none.
