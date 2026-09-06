# MEMORY — resume pointer (layer A; overwrite-only, keep ≤ ~50 lines)

> The bounded layer-A resume pointer (see `MEMORY_ARCHITECTURE.md`). OVERWRITE the
> "Current state" block each update — never append history here.

## How to resume

1. Read `README.md`, `MEMORY_ARCHITECTURE.md`, `TOOLBOX.md`, `DOCTRINE_ENFORCEMENT.md`.
2. Open the active task-tree below → its Current Frontier → continue from the next action.
3. Scope/gates: `ROADMAP.md`. Phase 1 execution: the `PHASE-1` tree under `docs/tasks/`.

## Current state

- **Project:** ReasonBraid (working name, not legally cleared).
- **Phase 0 COMPLETE:** the exit gate is closed (ADR-002 signed by the accountable owner —
  `docs/adr/002-phase1-scope.md`, GO) and the PHASE-0 tree is `done`: WP1–WP8 +
  `MAINT-1` (README_POLICY re-adopted: derived caps + routing-pressure closure) +
  `MAINT-2` (ReasonBraid-only naming — zero scaffold-name tokens remain).
- **Active tree:** `PHASE-1` → frontier `.1.5.1` — `.1.5` decomposed (`2026-09-06`) at the
  body-vs-rounds-vs-close seams: `.1.5.1` typed contribution kinds (§8.5 enum) +
  `evidence_refs` → `.1.5.2` rounds → `.1.5.3` honest inconclusive close (core
  `Inconclusive` terminal). `.1.4` is COMPLETE: `ClaudeCliAdapter` live-qualified
  (`RB_LIVE_CLAUDE=1`, first-run pass on 2.1.263) — backlogs 19–21 done. `.1`/`.1.2`/`.1.3`
  COMPLETE (backlogs 9–16); `PHASE-1-MAINT-1` done (§13 same-volume PG data).
- **Next action:** execute `PHASE-1.5.1` — the structured contribution body
  (`kind` enum + evidence references, additive projection growth).
- **Latest commit:** derive on read with `git log -1 --oneline`.
- **In-flight uncommitted work:** none after the `.1.4` decomposition commit (pending
  defect leaf `PHASE-1-MAINT-2`: one-off `codex_adapter` flake under parallel load —
  repro pending).
- **Push cadence:** every ~300 commits (director, 2026-09-06); run full CI before each push.
- **Local dev deps now installed (2026-09-06):** `postgresql@16`, `cargo-deny`, `gitleaks`,
  `mdbook` — plus `jq` for the demo script; `codex` (0.153.4) AND `claude` (2.1.263,
  Claude Code) for env-gated real runs.
- **Blockers:** none. Director-owned open items: license choice (`Cargo.toml` says
  `MIT OR Apache-2.0`, no `LICENSE` file); ADR-001 name clearance (repo stays private).
- **Dating anomaly (flagged):** host/git clock = 2026-09-06; some records from the previous
  session are dated 2026-09-07 inside 2026-09-06 commits. New records use the machine date.
