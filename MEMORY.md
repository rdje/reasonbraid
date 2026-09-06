# MEMORY — resume pointer (layer A; overwrite-only, keep ≤ ~50 lines)

> The bounded layer-A resume pointer (see `MEMORY_ARCHITECTURE.md`). OVERWRITE the
> "Current state" block each update — never append history here.

## How to resume

1. Read `README.md`, `MEMORY_ARCHITECTURE.md`, `TOOLBOX.md`, `DOCTRINE_ENFORCEMENT.md`.
2. Open the active task-tree below → its Current Frontier → continue from the next action.
3. Scope/gates: `ROADMAP.md`. Phase 0 execution: companion `KICKOFF.md`.

## Current state

- **Project:** ReasonBraid (working name, not legally cleared).
- **Active tree:** `PHASE-0` → frontier leaf `PHASE-0-MAINT-1` (`pending`, director's word);
  **WP1–WP8 complete — the Phase 0 tree is exhausted.** The WP8 gate package is
  published (evidence manifest, ADR set, SubtractionRecord, ADR-002 with the Phase 1
  GO recommendation, refreshed risk register). **The Phase 0 exit gate's one remaining
  item is the director's signature on ADR-002** (`docs/adr/002-phase1-scope.md`).
- **Next action:** director signs ADR-002 (GO) → PHASE-1 tree opens (`.1` LAN slice);
  or the director words `PHASE-0-MAINT-1` (README_POLICY re-adoption).
- **Latest commit:** derive on read with `git log -1 --oneline`.
- **In-flight uncommitted work:** none after `PHASE-0.8.1`.
- **Push cadence:** every ~300 commits (director, 2026-09-06); run full CI before each push.
- **Local dev deps now installed (2026-09-06):** `postgresql@16`, `cargo-deny`, `gitleaks`, `mdbook` — plus `jq` for the demo script; `codex` (0.153.4) for env-gated real runs.
- **Blockers:** ADR-002 signature (director).
