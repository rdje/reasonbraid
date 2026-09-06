# MEMORY — resume pointer (layer A; overwrite-only, keep ≤ ~50 lines)

> The bounded layer-A resume pointer (see `MEMORY_ARCHITECTURE.md`). OVERWRITE the
> "Current state" block each update — never append history here.

## How to resume

1. Read `README.md`, `MEMORY_ARCHITECTURE.md`, `TOOLBOX.md`, `DOCTRINE_ENFORCEMENT.md`.
2. Open the active task-tree below → its Current Frontier → continue from the next action.
3. Scope/gates: `ROADMAP.md`. Phase 0 execution: companion `KICKOFF.md`.

## Current state

- **Project:** ReasonBraid (working name, not legally cleared).
- **Active tree:** `PHASE-0` → frontier leaf `PHASE-0.1.3` (`pending`)
- **Next action:** WP1 minimal thread (`open`/`closing`/`closed`/`cancelled`), participation (`invited`/`accepted`/`declined`/`expired`/`left`), provider-attempt (`prepared`/`dispatched`/`completed`/`failed_before_dispatch`/`outcome_unknown`/`reconciled`) state transitions — invalid transitions rejected deterministically.
- **Latest commit:** derive on read with `git log -1 --oneline`.
- **In-flight uncommitted work:** none after `PHASE-0.1.2`.
- **Push cadence:** every ~300 commits (director, 2026-09-06); run full CI before each push.
- **Blockers:** none.
