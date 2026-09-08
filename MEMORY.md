# MEMORY — resume pointer (layer A; overwrite-only, keep ≤ ~50 lines)

> The bounded layer-A resume pointer (see `MEMORY_ARCHITECTURE.md`). OVERWRITE the
> "Current state" block each update — never append history here.

## How to resume

1. Read `README.md`, `MEMORY_ARCHITECTURE.md`, `TOOLBOX.md`, `DOCTRINE_ENFORCEMENT.md`.
2. Open the active task-tree below → its Current Frontier → continue from the next action.
3. Scope/gates: `ROADMAP.md`. Phase 7 execution: the `PHASE-7` tree under `docs/tasks/`.

## Current state

- **Project:** ReasonBraid (working name, not legally cleared).
- **Phases 0–6 CLOSED** (Phase 0's exit gate — ADR-002 signed; Phase 1's
  G1–G2 Met + Demonstration A 30/30; Phase 2's exit line measured; Phase 3's
  six lanes shipped; Phase 4's G4 Met; Phase 5's G5 Met as a subtraction
  gate; Phase 6's G3 Met as machinery, blocked as binding use).
- **Active tree:** `PHASE-7` (the Internet-hardening lane) → frontier `.4.2`.
  The `.1` lane is COMPLETE; the `.2` lane is COMPLETE; `.3` done (the
  extraction criteria); `.4` done (the census) → decomposed; `.4.1` done
  (the load harness — the measured run: p50 3.3ms / p95 7.9ms / 189.2
  cmds/s at 8 workers, 0 failures).
- **Next action:** execute `PHASE-7.4.2` — the §18.6 runbook set: the
  remaining twelve families (the provider outage/ambiguous charge, the
  credential compromise, the notification storm, the runaway budget,
  the poisoned resource, the database failover, the object loss, the
  Git/DB publication mismatch, the signing-key incident, the
  cross-tenant exposure suspicion, the audit-chain break, the
  rollback/suspension, the full DR) as the runbook records over the
  shipped controls.
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
  (2026-09-07 → 2026-09-08); new records use the machine date — the historical
  anomaly stands (some prior-session records dated 2026-09-07 inside
  2026-09-06 commits).
