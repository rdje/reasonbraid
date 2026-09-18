# Task-Tree Workflow

This document defines the repo-local task-tree workflow. A step-by-step setup guide is in
[TASK_TREE_README.md](TASK_TREE_README.md). Individual trees live under
[`tasks/`](tasks/); the leaf template is [`tasks/TEMPLATE.md`](tasks/TEMPLATE.md).

## Purpose

Use a task tree when a top-level task is too broad to finish safely as one signoff-quality
slice, or when it is expected to discover subtasks over time. The tree owns the recursive
breakdown, current frontier, acceptance criteria, blockers, decisions, validation, and
completion evidence for one top-level task — so the project survives a lost session and
continuity holds across sessions, machines, and harness switches.

The tree is not a second roadmap. `ROADMAP.md` states the high-level direction; a task tree
owns the disciplined execution of one lane of it.

## Code-change doctrine (binding, non-negotiable)

**It is strictly forbidden to make any code change unless it is first tracked/owned by a
task-tree leaf.** "Code change" = any edit to Rust sources, `Cargo.toml`/build scripts,
generated artifacts, or anything altering behavior. Before touching code, a leaf must exist
that owns the change (create/extend a tree, or add a leaf). The leaf — its goal, acceptance,
verification, and commit — is the unit of traceability. Enforced by
`scripts/check_task_tree_ownership.sh`.

## Leaf lifecycle (statuses)

`proposed` → `pending` → `active`/`in_progress` → `done`. A leaf is `done` only when its
acceptance criteria are met, verification is recorded, and it is committed via `COMMIT.md`.
Mark `blocked` (with the blocker named) rather than leaving a stalled leaf `active`.

## The pivot rule

**Do not pivot to a different task-tree while the repo is dirty.** The repo is
handoff-ready only when the tree is clean (no modified/untracked work except the task-tree
file itself). Finish the current leaf and get the repo clean before switching — even if
asked to pivot immediately. The guarantor of repo integrity holds this line.

## Commit traceability

Each slice uses a work-unit id in the commit subject (e.g. `MYPROJ-AREA-0007`). When the
slice belongs to a leaf, the subject or first body line also names the leaf ID
(e.g. `MYPROJ-AREA-0007 (leaf FEATURE-X.2): …`), so the slice id and the tree node coexist
on the same commit. One commit per completed leaf.

## Active Task Trees

| Tree | Status | Frontier (next leaf) | Owner |
| --- | --- | --- | --- |
| [`BOOTSTRAP`](tasks/BOOTSTRAP.md) | `done` | `.1` — bootstrapped from the ReasonBraid spine | repo-local |
| [`RB-SEED`](tasks/RB-SEED.md) | `done` | `.3` — CLAIM_VERIFICATION adopted | repo-local |
| [`PROGRAM`](tasks/PROGRAM.md) | `active` | index only — current executable tree is `SIGNOFF-REPAIR` | repo-local |
| [`PHASE-0`](tasks/PHASE-0.md) | `done` | tree complete — WP1–WP8 + `MAINT-1`/`MAINT-2`; next executable work is `PHASE-1.1` | repo-local |
| [`PHASE-1`](tasks/PHASE-1.md) | `done` | tree complete — G1–G2 **Met** + Demonstration A passed 30/30 (debug + release-built); next executable work is `PHASE-2.1` | repo-local |
| [`PHASE-2`](tasks/PHASE-2.md) | `done` | tree complete — the exit line's properties measured (non-escalation, restore + replacement, no false safe-retry) + the subtraction record; next executable work is `PHASE-3.1` | repo-local |
| [`PHASE-3`](tasks/PHASE-3.md) | `done` | tree complete — the directory, the presence, the matching, the recruitment, the subscriptions, the dependence indicators; next executable work is `PHASE-4.1` | repo-local |
| [`PHASE-4`](tasks/PHASE-4.md) | `done` | tree complete — the five pack lanes + the evidence pipeline; the G4 gate Met (the record + the subtraction + the manifest) | repo-local |
| [`PHASE-5`](tasks/PHASE-5.md) | `done` | tree complete — the five lanes + the G5 gate **Met as a subtraction gate** (the lift claim withdrawn per §25.1; the record + the subtraction + the manifest); next executable work is `PHASE-6.1` | repo-local |
| [`PHASE-6`](tasks/PHASE-6.md) | `done` | tree complete — the seven lanes + the G3 gate **Met as machinery, blocked as binding use** (the record + the subtraction + the Demo-B walk + the manifest); next executable work is `PHASE-7.1` | repo-local |
| [`PHASE-7`](tasks/PHASE-7.md) | `done` | tree complete — the five lanes + the G6–G7 gate package (**NOT MET for the Internet exposure, Met as the hardening-machinery exit** — the gate record + the subtraction + the unsupported matrix + the manifest); next executable work is `PHASE-8.1` | repo-local |
| [`SIGNOFF-REPAIR`](tasks/SIGNOFF-REPAIR.md) | `active` | `.11.20.1` — 🔴 **The memory census reports 3 standing warnings while 3,983 bytes, 65% of `MEMORY.md`, sit in bullets it cannot see.** `.11.20` repaired WHERE it looks for the bullet; nothing repaired WHAT it counts as one, so its headline answers a question the template stopped asking — worse than wrong, because a reader at the byte cap sees 3 and concludes the file is mostly next-action. ✅ **`.11.17.2` IS DONE (REPAIR-0243): a slash is not a resolution.** `POSITIONAL-REF` read resolvability off the SHAPE of a string — the branch for a reference without a slash consulted the tracked tree, the branch with one did not. Of **251** such occurrences at `6f91897`, **39** named no tracked file and **one was a suffix of THREE**, the ambiguity the gate exists to refuse. ⭐ The repair is the OLD question asked of both branches: `partial` (a path-suffix of exactly one) accepted as `unique` already is, several `ambiguous`, none `unresolved`, and a `<crate>-<version>` segment a `dependency` with `Cargo.lock` as its oracle — which earns `dependency-stale`, a failure mode nothing had before. ⛔ A suffix must fall on a SEGMENT boundary or the founding 93-false-positive hyphen bug returns as a false NEGATIVE. **Calibrated at 21 of 200 commits (10.5%)** against the 9.5% that argued this gate in; refusing `partial` too was priced at **4.5%** and DECLINED on SHAPE rather than cost. Standing population **8 → 0**; 11 of 12 new arms falsified by name in situ and the two passing both ways are LABELLED. ✅ `.11.20` (0242), `.11.17.1` (0241), `.11.19.2` (0240), `.11.19.1` (0239). | repo-local |
| [`PHASE-8`](tasks/PHASE-8.md) | `active` | corrective prerequisite `SIGNOFF-REPAIR`; then `.5.3` — the store-and-forward (opened at the Phase-7 close; **the `.1`–`.4` lanes COMPLETE**; `.5` the regional-routing census → decomposed `.5.1`–`.5.4`; `.5.1` ADR-035; `.5.2` the regional routing) | repo-local |
| [`PHASE-9`](tasks/PHASE-9.md) | `proposed` | `.1` — stable release | repo-local |
