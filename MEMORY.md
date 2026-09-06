# MEMORY — resume pointer (layer A; overwrite-only, keep ≤ ~50 lines)

> The bounded layer-A resume pointer (see `MEMORY_ARCHITECTURE.md`). OVERWRITE the
> "Current state" block each update — never append history here.

## How to resume

1. Read `README.md`, `MEMORY_ARCHITECTURE.md`, `TOOLBOX.md`, `DOCTRINE_ENFORCEMENT.md`.
2. Open the active task-tree below → its Current Frontier → continue from the next action.
3. Scope/gates: `ROADMAP.md`. Phase 0 execution: companion `KICKOFF.md`.

## Current state

- **Project:** ReasonBraid (working name, not legally cleared).
- **Phase 0 formally EXITED:** ADR-002 signed by the accountable owner (`docs/adr/002-phase1-scope.md`,
  GO, `accepted`); WP1–WP8 complete; the WP8 gate package is published.
- **Active tree:** `PHASE-0` → frontier leaf `PHASE-0-MAINT-1` (`pending`, director's word
  given — README_POLICY re-adoption, executing). After it, the PHASE-0 tree closes.
- **Opened:** `PHASE-1` (`active`) → frontier `.1` coordinator modular monolith, unblocked
  by the ADR-002 signature.
- **Next action:** finish `PHASE-0-MAINT-1`, then start `PHASE-1.1`.
- **Latest commit:** derive on read with `git log -1 --oneline`.
- **In-flight uncommitted work:** none after `PHASE-0.8.2`.
- **Push cadence:** every ~300 commits (director, 2026-09-06); run full CI before each push.
- **Local dev deps now installed (2026-09-06):** `postgresql@16`, `cargo-deny`, `gitleaks`,
  `mdbook` — plus `jq` for the demo script; `codex` (0.153.4) for env-gated real runs.
- **Blockers:** none. Director-owned open items: license choice (`Cargo.toml` says
  `MIT OR Apache-2.0`, no `LICENSE` file); ADR-001 name clearance (repo stays private);
  second real adapter (Phase 1).
- **Dating anomaly (flagged):** host/git clock = 2026-09-06; some records from the previous
  session are dated 2026-09-07 inside 2026-09-06 commits. New records use the machine date.
