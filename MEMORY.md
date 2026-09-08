# MEMORY — resume pointer (layer A; overwrite-only, keep ≤ ~50 lines)

> The bounded layer-A resume pointer (see `MEMORY_ARCHITECTURE.md`). OVERWRITE the
> "Current state" block each update — never append history here.

## How to resume

1. Read `README.md`, `MEMORY_ARCHITECTURE.md`, `TOOLBOX.md`, `DOCTRINE_ENFORCEMENT.md`.
2. Open the active task-tree below → its Current Frontier → continue from the next action.
3. Scope/gates: `ROADMAP.md`. Phase 7 execution: the `PHASE-7` tree under `docs/tasks/`.

## Current state

- **Project:** ReasonBraid (working name, not legally cleared).
- **Phases 0–7 CLOSED.** Phase 7's exit: G6–G7 NOT MET for the
  Internet exposure, Met as the hardening-machinery exit for the LAN
  profile (the exposure stays UNCLAIMED per the §25.1 kill/pivot —
  the three external preconditions: the reviewed threat model, the
  prompt-injection suite, the pen-test). The gate record + the
  subtraction (S-1…S-12) + the unsupported matrix + the evidence
  manifest ship.
- **Active tree:** `PHASE-8` (the federation/interoperability lane) →
  frontier `.5.3`. **The `.1`–`.4` lanes are COMPLETE**; `.5` done (the
  regional-routing census → decomposed `.5.1`–`.5.4`); `.5.1` done
  (ADR-035); `.5.2` done (the regional routing).
- **Next action:** execute `PHASE-8.5.3` — the store-and-forward: the
  intermittent-site delivery over the shipped substrate (the outbox +
  the inbox), consuming the `.5.2` routing.
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
