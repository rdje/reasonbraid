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
| [`SIGNOFF-REPAIR`](tasks/SIGNOFF-REPAIR.md) | `active` | `.11.4.7.2.1.2` — wave B item 4 of the `.15` sequence: the decision-rule and expected-artifact contract, which GATES item 6 (`.8.1.1`). ✅ **`.4.6` CLOSED (DOC-0124)**: four nouns, four verdicts — SLO baselines and game days ALREADY DISCHARGED by records dated 2026-09-07/08, *before* the 2026-09-22 audit that graded the row open; the OpenTelemetry sink DEFERRED on a readable fact (`rb-server --host` not loopback — leaving §6.6's Developer profile — or a second control-plane process); §18.5 PARTIAL at **4 covered · 3 partial · 2 absent**, scoped to `.4.6.1`. ✅ **`.9.3.5` CLOSED (REPAIR-0394)**: the publication store is BUILT and the leaf retracted its own premise — both halves shipped 2026-09-08, fourteen days before the audit that called them absent; three real gaps opened instead (`.9.3.5.1`–`.3`). ⛔ **Second consecutive leaf graded open over shipped work** — one by a grep keyed on the wrong module, one by a verdict over four nouns at once. ⏳ `.11.26` (row 2) stands: a pre-push gate failing unreproducibly, localized to the `download` phase, 8 of 8; standing sample **0 failures in 179 runs**; `.11.2.7` consolidates there | repo-local |
| [`PHASE-8`](tasks/PHASE-8.md) | `active` | corrective prerequisite `SIGNOFF-REPAIR`; then `.5.3` — the store-and-forward (opened at the Phase-7 close; **the `.1`–`.4` lanes COMPLETE**; `.5` the regional-routing census → decomposed `.5.1`–`.5.4`; `.5.1` ADR-035; `.5.2` the regional routing) | repo-local |
| [`PEER-COLLAB`](tasks/PEER-COLLAB.md) | `active` | `.1` — the census: walk the director's A↔B bug-report scenario step by step against the SHIPPED system and grade each step with an artefact. A director-requested acceptance track, off the frozen v0.4.1 roadmap; it MEASURES rather than promoting features, and its two expected gaps are parked in `docs/parking-lot.md` with this tree as their trigger | repo-local |
| [`PHASE-9`](tasks/PHASE-9.md) | `proposed` | `.1` — stable release | repo-local |
