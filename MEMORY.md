# MEMORY — resume pointer (layer A; overwrite-only, keep ≤ ~50 lines)

> The bounded layer-A resume pointer (see `MEMORY_ARCHITECTURE.md`). OVERWRITE the
> "Current state" block each update — never append history here.

## How to resume

1. Read `README.md`, `MEMORY_ARCHITECTURE.md`, `TOOLBOX.md`, `DOCTRINE_ENFORCEMENT.md`.
2. Open the active task-tree below → its Current Frontier → continue from the next action.
3. Scope/gates: `ROADMAP.md`. Phase 3 execution: the `PHASE-3` tree under `docs/tasks/`.

## Current state

- **Project:** ReasonBraid (working name, not legally cleared).
- **Phases 0–2 COMPLETE:** Phase 0's exit gate closed (ADR-002 signed, the
  WP1–WP8 + MAINT leaves `done`); Phase 1's G1–G2 **Met** + Demonstration A
  30/30 (the gate + subtraction records ship); Phase 2's exit line measured —
  authority non-escalation (the `.7.1` adversarial suite), restore + node
  replacement (the `.4.1` exercise + the `.7.2` drill), no false safe-retry
  of unknown attempts (the `.7.3` six-leg inventory) — the §19.8 subtraction
  record (S-1…S-13) + the G6–G7 feed ship; all three trees `done`.
- **Active tree:** `PHASE-3` → frontier `.6.3` (`.6.2` done: the
  diversity feature + the panel wiring). Phase 3 closes after
  `.6.3`.
- **Next action:** execute `PHASE-3.6.3` — the label discipline:
  the §10.4 acceptance as a mechanical sweep (the term "independent
  probability" appears nowhere; the indicator's wire name is the
  approved "diversity and dependence indicators" label). No code
  beyond the labels. **Phase 3 closes after this leaf.**
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
