# BOOTSTRAP: this project's bootstrap from the ReasonBraid spine template

## Metadata

- Tree ID: `BOOTSTRAP`
- Status: `done`
- Roadmap lane: project setup
- Created: `2026-09-04`
- Owner: repo-local workflow

## Goal

Record the one-time de-templating of this copy of the ReasonBraid spine (reasonbraid-scaffold 0.6.1) into
project `reasonbraid`, performed by `scripts/bootstrap.sh reasonbraid`, with the evidence that run produced —
so the first commit of this project passes the same gates every later commit will.

## Non-Goals

- This tree does not describe the project's roadmap. Seed that tree from `ROADMAP.md`.

## Task Tree

- ID: `BOOTSTRAP.1`
  Status: `done`
  Goal: de-template, rename the crate, install the hooks, regenerate the Knowledge Map, verify the enforcer.

  ### Acceptance Checklist (enforced by `TASK-ACCEPTANCE`)

  - [x] **ROOT CAUSE (WHY + WHERE)** — a spine copy carries the template's crate name and
    maintainer files: `grep -c '^name = "app"' crates/app/Cargo.toml` → 1 before the run,
    0 after (`rc=0`); `MAINTAINING.md` and the maintainer tree are removed by the de-template step.
  - [x] **ADDRESSED (verified)** — crate renamed to `reasonbraid`; hooks installed: `git config core.hooksPath`
    → `.githooks` (`rc=0`); the Knowledge Map regenerated; the enforcer run inside this
    bootstrap: `=== doctrine enforcement (13 checks) ===` … `=== all doctrines green ===` (`rc=0`).
  - [x] **NO REGRESSION** — the same enforcer is the pre-commit hook: `scripts/check_doctrines.sh` →
    `=== all doctrines green ===`, `rc=0`; `make gate` is that command.

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| — | `BOOTSTRAP.1` | `done` | the bootstrap itself; nothing further belongs here |

## Commit Log

- `2026-09-04` — `BOOTSTRAP.1` — `REASONBRAID-BOOTSTRAP-0001`: bootstrapped from the ReasonBraid spine.
