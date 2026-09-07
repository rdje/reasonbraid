# MEMORY — resume pointer (layer A; overwrite-only, keep ≤ ~50 lines)

> The bounded layer-A resume pointer (see `MEMORY_ARCHITECTURE.md`). OVERWRITE the
> "Current state" block each update — never append history here.

## How to resume

1. Read `README.md`, `MEMORY_ARCHITECTURE.md`, `TOOLBOX.md`, `DOCTRINE_ENFORCEMENT.md`.
2. Open the active task-tree below → its Current Frontier → continue from the next action.
3. Scope/gates: `ROADMAP.md`. Phase 2 execution: the `PHASE-2` tree under `docs/tasks/`.

## Current state

- **Project:** ReasonBraid (working name, not legally cleared).
- **Phase 0 COMPLETE:** the exit gate is closed (ADR-002 signed by the accountable owner —
  `docs/adr/002-phase1-scope.md`, GO) and the PHASE-0 tree is `done`: WP1–WP8 +
  `MAINT-1` (README_POLICY re-adopted: derived caps + routing-pressure closure) +
  `MAINT-2` (ReasonBraid-only naming — zero scaffold-name tokens remain).
- **Phase 1 COMPLETE:** G1–G2 **Met** + Demonstration A passed 30/30 (debug
  AND release-built) — the gate record, the §19.8 subtraction record, and
  the evidence manifest ship (`docs/decisions/2026-09-07_phase1-*.md`,
  `docs/evidence/2026-09-07_phase1-evidence-manifest.md`). All leaves `.1`–
  `.1.8` done (identity store → node/channel/inbox → participants →
  adapters → contributions/rounds/close → UI/budget → dev + packaging →
  gate) + the three defect leaves closed.
- **Active tree:** `PHASE-2` → frontier `.7.3` (`.7.2` done: the
  node-replacement drill — the measured ritual, the epoch fence held,
  migrations 0017/0018, the guard grew to 17 live suites). Then
  `.7.4`; Phase 2 closes after `.7.4`.
- **Next action:** execute `PHASE-2.7.3` — ADR-022 (the audit
  hash-chain groundwork: the shipped actor/subject/grant/digest
  bindings accepted-with-evidence; the chain itself deferred with its
  trigger) + the no-false-safe-retry evidence inventory (the four
  retry-policy legs + the quarantine replay gate + the `.7.2` drill's
  measured epoch fence), as the §16.12 line's dev-profile proof.
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
