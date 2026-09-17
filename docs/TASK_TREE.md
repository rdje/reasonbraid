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
| [`SIGNOFF-REPAIR`](tasks/SIGNOFF-REPAIR.md) | `active` | `.11.17.2` — 🔴 **`POSITIONAL-REF` calls a reference `pathed` on the presence of a slash alone and never checks that the path exists.** Of **251** `pathed` occurrences **39 do not resolve**, and one of them is a suffix of **three** tracked files — an AMBIGUOUS reference waved through by the gate whose entire purpose is refusing one, which is `BOOK-LINKS`' founding shape inside a gate that cites `BOOK-LINKS`. ⚠️ 5 of the 39 are legitimate dependency citations that must not be swept up, and `Cargo.lock` makes that class mechanically checkable. ✅ **`.11.17.1` IS DONE (REPAIR-0241): the leaf's own premise was wrong — `init.rs` was never renamed or deleted, because it was never in this repository.** It is `gix-0.87.1/src/config/cache/init.rs`, a DEPENDENCY's source, and every one of the seven cited lines is exact at the version `Cargo.lock` pins. ⭐ **DECIDED: a dependency citation is written crate-and-version qualified** — it resolves for a reader AND dates itself, which a bare basename never could. ⭐ **The example-versus-citation obstacle is answered rather than waived, and not by telling them apart**: no instrument can, so an illustrative example may not be WRITTEN in the positional form, and the gate's own registry row was reworded rather than excluded — the fourth time this doctrine has policed its own description. **17 citations qualified** (not the 8 the leaf counted — its key missed the partially-pathed ones), `unresolved` **9 → 0**, arm calibrated at **2 of 200 commits (1.0%)** and both are the instances discharged. 🔴 It then flagged this leaf's own closing prose. ✅ **`.11.20` (REPAIR-0242): the instrument that exists to stop `MEMORY.md` hitting its cap had been REFUSING on every run since `6199f43`, the commit that conformed `MEMORY.md` to its template and renamed the bullet the census keys on** — while its own `--self-test` reported 16 fixture controls passing. The defect it prevents had recurred (**6,123 of 7,168 bytes**), and **65% of the file was six standing-lesson bullets restating notes already durable in `docs/knowledge/`**, which is `.11.16`'s mirror shape one layer up. Repaired to **3,825 bytes with nothing lost** — every lesson is now a named pointer, all nine targets verified tracked BEFORE eviction. A live-corpus arm now reads the real file and was falsified by name. ⛔ The general gate (*every census still runs against its corpus*) is DECLINED on COST rather than on shape: it would have fired on 1 of 12 then and 0 of 12 now — `REASON-CODE-DOC`'s shape — but costs 3.9 s on an 11.6 s enforcer, against the 1.01 s on 3.15 s that argued `SELF-TEST` in. ✅ `.11.17.1` (0241), `.11.19.2` (0240), `.11.19.1` (0239), `.11.19` (0238), `.11.16` (0237). | repo-local |
| [`PHASE-8`](tasks/PHASE-8.md) | `active` | corrective prerequisite `SIGNOFF-REPAIR`; then `.5.3` — the store-and-forward (opened at the Phase-7 close; **the `.1`–`.4` lanes COMPLETE**; `.5` the regional-routing census → decomposed `.5.1`–`.5.4`; `.5.1` ADR-035; `.5.2` the regional routing) | repo-local |
| [`PHASE-9`](tasks/PHASE-9.md) | `proposed` | `.1` — stable release | repo-local |
