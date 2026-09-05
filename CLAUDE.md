# Agent bootstrap (read this first — whatever AI / harness you are: Claude Code, Codex, Gemini, Cursor, Aider, a custom runner, …)

This repository is built on a **portable discipline spine**: durable memory, task-tree
tracking, a strict commit workflow, and mechanical doctrine enforcement. The spine is
enforced at the **git level** (hooks + CI), so it holds regardless of which agent or
human is working. Follow it exactly.



1. Read `README.md` — project objective, layout, standard commands.
2. Read `MEMORY_ARCHITECTURE.md` — the durable 4-layer memory model (**MANDATORY**;
   defines how nothing important is lost across sessions, machines, or harness switches).
3. Read `TOOLBOX.md` — the tools-first doctrine. For ANY unknown / failure / surprising
   result: use or build a diagnostic tool FIRST; never guess a root cause.
4. Read `DOCTRINE_ENFORCEMENT.md` — how every mechanizable doctrine is enforced, and the
   task-acceptance checklist a change MUST pass.
5. Read `docs/CLAIM_VERIFICATION.md` — a published number is checked by re-derive,
   falsify, and durability (three different questions), not by repeating the same pass.
6. Resume from `MEMORY.md` (the bounded layer-A resume pointer) → the active task-tree's
   frontier under `docs/tasks/`.

## The non-negotiables

- **Nothing changes without a task-tree first.** Every code change is owned by a
  task-tree leaf under `docs/tasks/` (index: `docs/TASK_TREE.md`) BEFORE the change is
  made. Track every activity, task, slice, and lane so the project survives a lost session.
- **Record durable facts/decisions** as one-file-per-record notes under `docs/decisions/`
  (index there). Convert relative dates to absolute.
- **Commit per `COMMIT.md`** after each completed leaf, with the work-unit id in the
  subject. A code change must pass the `TOOLBOX.md` / `DOCTRINE_ENFORCEMENT.md` acceptance
  checklist (root cause + addressed + no regression) in its task leaf. A published
  measurement follows `docs/CLAIM_VERIFICATION.md` (re-derive · falsify · durability).
- **Activate the hooks once per clone:** `git config core.hooksPath .githooks`. The
  pre-commit hook runs `scripts/check_doctrines.sh` (the general enforcer); CI runs the
  same. These are git-level and harness-agnostic.
- **Keep the roadmap, the code, and the docs (README + mdBook) aligned** — locked
  together, no drift, for past, present, and future changes.
- **No background job at a handoff point.** Before you end a session (`/exit`, a pause, a
  handover), run `bash scripts/check_no_background_jobs.sh` and make it print `handoff: OK`:
  a job that outlives its session rewrites tracked files under the next one, with its log
  gone. Kill stragglers AND their children — a parent's death does not propagate.
- **A commit message ends with its own last line** — no agent/tool attribution trailers
  (`COMMIT.md`); the `commit-msg` hook refuses them.

> One rule above all: **information that exists only in the live conversation is not yet
> saved — route it to a layer and commit it before the turn ends.**

## First time in a fresh clone

Run `scripts/bootstrap.sh` once. It sets the project name, installs the git hooks, and
seeds the first task-tree from `ROADMAP.md`. Then drop your real roadmap into `ROADMAP.md`
and grow the project from there.
