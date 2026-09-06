# MEMORY — resume pointer (layer A; overwrite-only, keep ≤ ~50 lines)

> The bounded layer-A resume pointer (see `MEMORY_ARCHITECTURE.md`). OVERWRITE the
> "Current state" block each update — never append history here.

## How to resume

1. Read `README.md`, `MEMORY_ARCHITECTURE.md`, `TOOLBOX.md`, `DOCTRINE_ENFORCEMENT.md`.
2. Open the active task-tree below → its Current Frontier → continue from the next action.
3. Scope/gates: `ROADMAP.md`. Phase 0 execution: companion `KICKOFF.md`.

## Current state

- **Project:** ReasonBraid (working name, not legally cleared).
- **Active tree:** `PHASE-0` → frontier leaf `PHASE-0.6.2` (`pending`); WP1 + WP2 + WP3 +
  WP4 + **WP5 complete**; `.6.1` control API + CLI done — the `rb` binary drives
  enroll / create thread / invite / contribute / challenge / revise / close / inspect
  over `/v1/threads` with a one-transaction claim→authorize→validate→apply flow;
  inspection never touches the database.
- **Next action:** WP6 two-host crash/reconnect demo (`.6.2`): wire the node channel
  into the `.6.1` surface (inbox dispatch + node-result events) + the reproducible
  demo script.
- **Latest commit:** derive on read with `git log -1 --oneline`.
- **In-flight uncommitted work:** none after `PHASE-0.6.1`.
- **Push cadence:** every ~300 commits (director, 2026-09-06); run full CI before each push.
- **Local dev deps now installed (2026-09-06):** `postgresql@16`, `cargo-deny`, `gitleaks`, `mdbook` — the full `make check`/`gate`/`deny`/`secret-scan`/`book` stack runs locally.
- **Blockers:** none.
