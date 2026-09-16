# MEMORY — the layer-A resume pointer

⛔ **This file is OVERWRITTEN, never appended.** It carries the next action, the active tree, and the traps that would make a fresh session act wrongly in its first hour. It is not a log and not a lesson store. **A fact worth keeping goes to a durable layer** — the owning leaf under `docs/tasks/`, `docs/decisions/`, `docs/knowledge/`, `TOOLBOX.md` or the book — and the pointer names the layer, not the fact. (Director instruction 2026-09-16; `SIGNOFF-REPAIR.11.4.2.2`.)

## How to resume

1. Read `CLAUDE.md`, `README.md`, `MEMORY_ARCHITECTURE.md`, `TOOLBOX.md`, `DOCTRINE_ENFORCEMENT.md`, `COMMIT.md`.
2. Open `docs/tasks/SIGNOFF-REPAIR.md` → **Current Frontier**. Row 1 is the next leaf; the caption's `awk` command re-derives the pending count.
3. Method statements: `TOOLBOX.md` and `docs/knowledge/` — **consult, do not re-derive**. Source-review records: `docs/tasks/artifacts/signoff_review/INDEX.md`.

## Current state

- **Project:** ReasonBraid. The public repository `rdje/reasonbraid` must remain public (director, 2026-09-09). Working-name clearance is open and blocks package/domain/marketing release only — never repairs (`docs/adr/001`).
- **Active tree:** `SIGNOFF-REPAIR`. Re-derive both numbers rather than reading them here: `git rev-list --count origin/main..HEAD`, and the frontier caption's `awk` (it must read the `- Opened:`-only spelling as well as `- Status:`).
- **Next action:** frontier row 1 — `SIGNOFF-REPAIR.11.14.1`, the cross-tenant evidence enumeration. Reproduce before repairing.
- **Latest commit:** `git log -1 --oneline`. Source-review baseline: `9c2d2ba`.
- **Roadmap:** Phase 0–7 records exist; qualification is under corrective review and Phase 8 is incomplete. After `SIGNOFF-REPAIR`, resume `PHASE-8.5.3`.

## Standing obligations

- **Blockers.** The register is `SIGNOFF-REPAIR.13` (`## Blockers` in the tree) and `docs/book/src/blockers.md`. **Re-surface every `Ack? = no` row in EVERY stopping-point reply**, as a labelled list. Clear a row only when the director engages with *that* row.
- **PNT mode**, resumed by the director 2026-09-11. Commit each bounded leaf per `COMMIT.md`; no background job may outlive a handoff.
- **Push cadence: batches of ~300 commits.** ⛔ Never per commit. The rule and its ended red-remote exception are in `COMMIT.md`.
- ⛔ **No zero-defect claim.** Findings are owned in the tree, never merely reported.

## Traps — what a fresh session gets wrong

- ⚠️ **Always `python3 -B scripts/project_env.py cargo …`.** A bare `cargo` uses a different `CARGO_HOME` and re-downloads the world. **One `cargo` at a time** — check `pgrep -fl cargo` first.
- ⛔ **Never raise a cap to fit content** (`MEMORY_ARCHITECTURE.md`): demote it.
- ⛔ **Every published number comes from a command**, re-run at HEAD — and check an instrument's first number a *different* way (`docs/CLAIM_VERIFICATION.md`, `TOOLBOX.md`). That habit has caught a wrong number in this file's own history.
- **Storage:** every output, cache, scratch and temporary store is repository-derived and on the repository's volume (`CLAUDE.md` §13).

## Where the rest lives

A leaf's findings and evidence are in its own section of `docs/tasks/SIGNOFF-REPAIR.md`; durable cross-cutting decisions in `docs/decisions/` and its `INDEX.md`; transferable methods in `TOOLBOX.md` and `docs/knowledge/`; what each gate enforces and why in `DOCTRINE_ENFORCEMENT.md`; current progress and qualification limits in `LIVE_STATUS.md`; and what a user or operator sees in `docs/book/`.
