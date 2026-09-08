# MEMORY — resume pointer (layer A; overwrite-only, keep ≤ ~50 lines)

> The bounded layer-A resume pointer (see `MEMORY_ARCHITECTURE.md`). OVERWRITE the
> "Current state" block each update — never append history here.

## How to resume

1. Read `README.md`, `MEMORY_ARCHITECTURE.md`, `TOOLBOX.md`, `DOCTRINE_ENFORCEMENT.md`.
2. Open the active task-tree below → its Current Frontier → continue from the next action.
3. Scope/gates: `ROADMAP.md`. Phase 3 execution: the `PHASE-3` tree under `docs/tasks/`.

## Current state

- **Project:** ReasonBraid (working name, not legally cleared).
- **Phases 0–4 COMPLETE** (Phase 0's exit gate closed — ADR-002 signed;
  Phase 1's G1–G2 Met + Demonstration A 30/30; Phase 2's exit line measured
  + the §19.8 subtraction record; Phase 3's six lanes shipped; Phase 4's
  seven lanes shipped — the resource packs, the evidence pipeline, G4 Met).
- **Active tree:** `PHASE-6` → frontier `.7` (**PHASE 5 CLOSED**; the `.6` reviews ship — the seven triggers + the idempotent schedule; policy 11).. The `.1` lane (the workflow
  profiles), the `.2` lane (the blind-first deliberation), the `.3` lane
  (the moderator/synthesizer constraints), and the `.4` lane (the
  evaluation service) are COMPLETE — the `.5` lane (the routing policy)
  runs: `.5.1` done (ADR-031), `.5.2` next, then `.5.3`, then `.6`.
- **Next action:** execute `PHASE-6.7` — the G3 exit + the
  Demonstration B (the gate record + the §19.8 subtraction record +
  the G7 publication portion); §25.1's pre-Phase-6 gate applies (no
  binding policy governance unless the real owners accept the
  authority/correction model). Decompose at the census seams first.
- **Latest commit:** derive on read with `git log -1 --oneline`.
- **Defects:** 0 tracked leaves outstanding (`MAINT-1` §13 locality, `MAINT-2`
  drain race, `MAINT-3` toolchain pin — all closed).
- **In-flight uncommitted work:** none.
- **Push cadence:** every ~300 commits (director, 2026-09-06); run full CI before each push.
- **Local dev deps (2026-09-06):** `postgresql@16`, `cargo-deny`, `gitleaks`,
  `mdbook` — plus `jq` for the demo script; `codex` (0.153.4) AND `claude`
  (2.1.263, Claude Code) for env-gated real runs.
- **Blockers:** none. Director-owned open items: license choice (`Cargo.toml` says
  `MIT OR Apache-2.0`, no `LICENSE` file); ADR-001 name clearance (repo stays private).
- **Dating anomaly (flagged):** the machine clock crossed midnight mid-session
  (2026-09-06 → 2026-09-07); new records use the machine date — the historical
  anomaly stands (some prior-session records dated 2026-09-07 inside
  2026-09-06 commits).
