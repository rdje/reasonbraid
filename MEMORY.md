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
- **Active tree:** `PHASE-7` (the Internet-hardening lane) → frontier `.1.4.1`.
  `.1.1` done (ADR-034 accepted); `.1.2` done (the mTLS workload identity);
  the `.1.3` lane COMPLETE (the RLS layer, the quotas, the
  quarantine-evidence rule); `.1.4` done (the census: the dev secrets are
  the plaintext rows + the hashed node secret; the classification is
  RECORDED-ONLY — the "silent general"; no region/export machinery) →
  decomposed `.1.4.1` (the declared-profile contract) → `.1.4.2` (the
  secret-store profiles) → `.1.4.3` (the classification controls).
- **Next action:** execute `PHASE-7.1.4.1` — the declared-profile contract
  (the decision record): the secret-store profile vocabulary (the shipped
  `dev_database` profile, the external stores as the configuration choice,
  the undeclared-store typed refusal) + the classification-controls mapping
  (each control refuses at ITS decision point — ADR-034's "never a silent
  general") + the named deferrals with their triggers.
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
