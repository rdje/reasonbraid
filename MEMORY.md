# MEMORY — resume pointer (layer A; overwrite-only, keep ≤ ~50 lines)

> The bounded layer-A resume pointer (see `MEMORY_ARCHITECTURE.md`). OVERWRITE the
> "Current state" block each update — never append history here.

## How to resume

1. Read `README.md`, `MEMORY_ARCHITECTURE.md`, `TOOLBOX.md`, `DOCTRINE_ENFORCEMENT.md`.
2. Open the active task-tree below → its Current Frontier → continue from the next action.
3. Scope/gates: `ROADMAP.md`. Phase 0 execution: companion `KICKOFF.md`.

## Current state

- **Project:** ReasonBraid (working name, not legally cleared).
- **Active tree:** `PHASE-0` → frontier leaf `PHASE-0.7` (`pending`); WP1–WP5 + **WP6 complete**
  (`.6.1` control API + CLI; `.6.2` node wiring + the two-host crash/reconnect demo —
  invite/challenge dispatch work items with reservations in the command transaction,
  node `work_result` events fold into the thread claim-first keyed on the inbox command
  id, `rb-node` worker never silently retries ambiguous work, `scripts/demo_two_host.sh`
  asserts every KICKOFF WP6 acceptance point with real kill points + an evidence bundle).
- **Next action:** WP7 small deliberation/routing benchmark (`PHASE-0.7`): versioned
  corpus vs single-agent / blind-independent / critique-revise / moderator-synthesis;
  results include cases + uncertainty, no independence score. Then WP8, then the
  queued maintenance leaf `PHASE-0-MAINT-1` (README_POLICY re-adoption — director's word).
- **Latest commit:** derive on read with `git log -1 --oneline`.
- **In-flight uncommitted work:** none after `PHASE-0.6.2`.
- **Push cadence:** every ~300 commits (director, 2026-09-06); run full CI before each push.
- **Local dev deps now installed (2026-09-06):** `postgresql@16`, `cargo-deny`, `gitleaks`, `mdbook` — plus `jq` for the demo script.
- **Blockers:** none.
