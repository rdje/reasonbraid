# MEMORY — resume pointer (layer A; overwrite-only, keep ≤ ~50 lines)

> The bounded layer-A resume pointer (see `MEMORY_ARCHITECTURE.md`). OVERWRITE the
> "Current state" block each update — never append history here.

## How to resume

1. Read `README.md`, `MEMORY_ARCHITECTURE.md`, `TOOLBOX.md`, `DOCTRINE_ENFORCEMENT.md`.
2. Open the active task-tree below → its Current Frontier → continue from the next action.
3. Scope/gates: `ROADMAP.md`. Phase 0 execution: companion `KICKOFF.md`.

## Current state

- **Project:** ReasonBraid (working name, not legally cleared).
- **Active tree:** `PHASE-0` → frontier leaf `PHASE-0.8` (`pending`); WP1–WP6 + **WP7 complete**
  (`.6.2` two-host demo; `.7` deliberation benchmark — `rb-bench` over a versioned
  8-case corpus, four workflows, deterministic graders, real Codex run recorded in
  `docs/evidence/2026-09-07_benchmark-codex-run.md`: NULL result, structure didn't
  beat single at 2–4× cost).
- **Next action:** WP8 (`.8.1`) Phase 0 decision and subtraction package: evidence
  manifest, ADR set, subtraction record, Phase 1 go/rework/pivot/stop — then the
  queued maintenance leaf `PHASE-0-MAINT-1` (README_POLICY re-adoption — director's word).
- **Latest commit:** derive on read with `git log -1 --oneline`.
- **In-flight uncommitted work:** none after `PHASE-0.7`.
- **Push cadence:** every ~300 commits (director, 2026-09-06); run full CI before each push.
- **Local dev deps now installed (2026-09-06):** `postgresql@16`, `cargo-deny`, `gitleaks`, `mdbook` — plus `jq` for the demo script; `codex` (0.153.4) for env-gated real runs.
- **Blockers:** none.
