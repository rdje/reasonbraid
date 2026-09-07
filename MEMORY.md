# MEMORY — resume pointer (layer A; overwrite-only, keep ≤ ~50 lines)

> The bounded layer-A resume pointer (see `MEMORY_ARCHITECTURE.md`). OVERWRITE the
> "Current state" block each update — never append history here.

## How to resume

1. Read `README.md`, `MEMORY_ARCHITECTURE.md`, `TOOLBOX.md`, `DOCTRINE_ENFORCEMENT.md`.
2. Open the active task-tree below → its Current Frontier → continue from the next action.
3. Scope/gates: `ROADMAP.md`. Phase 3 execution: the `PHASE-3` tree under `docs/tasks/`.

## Current state

- **Project:** ReasonBraid (working name, not legally cleared).
- **Phases 0–3 COMPLETE:** Phase 0's exit gate closed (ADR-002 signed);
  Phase 1's G1–G2 **Met** + Demonstration A 30/30; Phase 2's exit line
  measured (the adversarial suite, the replacement drill, the six-leg
  retry inventory) + the §19.8 subtraction record + the G6–G7 feed; Phase 3's
  six lanes shipped (the directory profiles, the presence, the two-stage
  matching, the recruitment protocol, the subscriptions + the node-initiated
  API, the dependence indicators) — all four trees `done`.
- **Active tree:** `PHASE-4` → frontier `.1.3` (`.1.2` done: the
  typed `ResourceReference` + the submission — migration 0023, the
  replay/conflict immutability). Then `.2`–`.7`.
- **Next action:** execute `PHASE-4.1.3` — the resolver capability
  registry: the §12.2 advertises (the schemes/patterns, the media
  types + the max bytes, the abilities, the auth classes, the
  egress class + the sandbox level per ADR-018, the policies, the
  snapshot formats, the latency range, the version + the security
  evidence) + the resolution order (the authz + the risk filters
  FIRST, then the rank) + the explicit `resource_unresolvable_now`
  (the reference is PRESERVED — never fabricated).
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
