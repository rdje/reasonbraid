# MEMORY — resume pointer (layer A; overwrite-only, keep ≤ ~50 lines)

> The bounded layer-A resume pointer (see `MEMORY_ARCHITECTURE.md`). OVERWRITE the
> "Current state" block each update — never append history here.

## How to resume

1. Read `README.md`, `MEMORY_ARCHITECTURE.md`, `TOOLBOX.md`, `DOCTRINE_ENFORCEMENT.md`.
2. Open the active task-tree below → its Current Frontier → continue from the next action.
3. Scope/gates: `ROADMAP.md`. Phase 0 execution: companion `KICKOFF.md`.

## Current state

- **Project:** ReasonBraid (working name, not legally cleared).
- **Active tree:** `PHASE-0` → frontier leaf `PHASE-0.3.2` (`pending`); WP1 + WP2 (atomic
  transaction `.2.1`, leased outbox worker with fencing `.2.2`) complete; `.3.1` done —
  WAL node journal with honest `outcome_unknown` recovery + read-only `rb-journal` CLI.
- **Next action:** WP3 outbound node channel — cursor resume + reconciliation handshake
  (`.3.2`), built on the `.3.1` journal's dedupe primitives and ack cursor.
- **Latest commit:** derive on read with `git log -1 --oneline`.
- **In-flight uncommitted work:** none after `PHASE-0.3.1`.
- **Push cadence:** every ~300 commits (director, 2026-09-06); run full CI before each push.
- **Local dev deps now installed (2026-09-06):** `postgresql@16`, `cargo-deny`, `gitleaks`, `mdbook` — the full `make check`/`gate`/`deny`/`secret-scan`/`book` stack runs locally.
- **Blockers:** none.
