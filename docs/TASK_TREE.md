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
| [`SIGNOFF-REPAIR`](tasks/SIGNOFF-REPAIR.md) | `active` | `.11.9.1.3.2` — tranche 4b of the record reconciliation, the largest child and the one carrying the declared deviation. ⭐ `.9.3.4`'s DECISION is taken (DOC-0020) and measuring it REVERSED the pre-announced answer: the leaf's premise that `GrantAction`/`TargetSelector` are frozen under §9.8 is false — §9.8 is the reason-code registry and `git grep -c` over `ROADMAP.md` returns **0** for both. `GrantAction` EXTENDS with administrative verbs; `TargetSelector` does NOT, because `Threads { threads }` enumerates objects at grant time and the ordinary flow publishes a publication whose id does not yet exist. 🔴 The extension is a **migration**: boundaries store wire names, so no existing row can contain a new action and the verb stops working for every enrolled tenant unless a disposition is chosen. Decomposed into `.9.3.4.1`/`.9.3.4.2` with acceptance; not implemented (207 references, 19 files). 🔴 `.9.2.1` is CLOSED, all three children: REPAIR-0192 closes `.9.2.1.2`, where both publish verbs admitted **any enrolled principal** — enrolment in any tenant was the whole predicate for writing a publication into a Git repository and for declaring it effective. One definition both verbs call now requires an `owning_authority` the caller HOLDS (`authority::grant_held_by`, `.9.3.1`'s predicate), checked before the path is resolved and before the publication is loaded. ⭐ Grant ids are derivable from a principal id, so the load-bearing leg is naming SOMEONE ELSE'S grant — refused — beside the matched pair where only the grant's holder differs and the request is admitted. ⚠️ **The limit is recorded, not papered over**: no grant action and no target selector can NAME a publication, so a held grant is effectively TENANT-WIDE for these verbs, and the book says so rather than implying the verb is scoped; `.9.3.4` owns the narrowing and is promoted to frontier row 1b. 🔴 `.9.2.1.3` is closed by REPAIR-0191: `mark_effective` recorded whatever Git object ids the caller declared and nothing opened a repository, and **three of this project's own fixtures drove the transition with `abc123`** — a suite qualifying the transition with ids that do not exist. `publisher::missing_objects` now answers per id, in the CORE both publish verbs go through rather than in either handler, and `resolve_repository` returns a `PublicationRepository` newtype with no other constructor, so containment became a type rather than a convention. The three fixtures are RE-SEEDED with ids read back from a real repository, not relaxed. ⚠️ `repo_path` is now required on the effective verb — a wire-contract change, because a transition that cannot say which repository it means cannot check anything. ⚠️ Limit stated: an id is proved to EXIST, not to be the publication's own; binding it to the publication's refs needs the repository recorded with the row. 🔴 `.9.2.1.1` is closed by REPAIR-0190: the publish verb took its repository location from the REQUEST BODY and handed it to `gix::open`, so any enrolled principal named any path on the server's filesystem. The deployment now declares one root, the caller names a location inside it, and both sides are CANONICALIZED before they are compared — which is why `..` and a symlink out of the root are refused by the same test, the symlink case being the one a string comparison admits. An undeclared root closes the verb with a new `503 publication_repository_unconfigured`, and an unusable declared one refuses the boot before the migrations run. ⚠️ The repair exposed a pre-existing fixture defect it was told to preserve: the "non-repository refuses" leg ran after the publication was already `effective`, so the STAGE check answered it and `gix::open` was never reached — green for the wrong reason for as long as it has existed. 🔴 `.9.2.1` was DECOMPOSED by REPAIR-0189 into three children: its three acceptance clauses are three different repairs — one invents server configuration and a containment predicate, one changes the wire contract of two shipped verbs, one adds a repository read to a database transition — and each needs its own control observed RED before its fix. 🔴 `.11.2.3` is closed by REPAIR-0187: the arity gate modelled the OPPOSITE of the renderer — it treated a pipe inside a code span as part of the cell, and its own `--self-test` asserted that same false answer, so the corpus read **0** defective rows in 323 files while mdbook was dropping cells. The deliberately-retained row was the proof: it published **516 of its 1,027** rationale characters and a bare `—` where its enforcer name belongs. All nine arms are now the renderer’s verdict, rendered before being asserted. 🔴 `.6.1.2` is closed by REPAIR-0185: the leaf named the MCP seam and the defect was in the SHARED CORE, where the HTTP verb takes no tenant at all and was equally open — a seam-level repair would have left it open and the seam's own suite green. 🔴 `.9.3.1` is closed by REPAIR-0184: citing an authority was the same as holding one on THREE surfaces, not the two its leaf named, and no site in the family had ever consulted `valid_from`. 🔴 `.6.1.1` is closed by REPAIR-0182, the severest defect this reconciliation produced: the three MCP read tools ran a PRIVATE re-implementation of an authorization their own module header said they shared, and a live control measured a caller in one tenant receiving another tenant's entire thread projection. The three reads now CALL the HTTP handlers' own gates through the new `mcp_read_internal` seam; the copy is deleted. ⚠️ **One part of the finding was corrected by measurement rather than confirmed**: the policy registry has no tenant column and no site filters by one, so the bundle was never crossing a tenant boundary — it is site-wide on both surfaces, and the tool's real debts were the enrolment gate and a false tenant label. The schema question is opened as `.6.1.5`. Tranche 4b is row 2 | repo-local |
| [`PHASE-8`](tasks/PHASE-8.md) | `active` | corrective prerequisite `SIGNOFF-REPAIR`; then `.5.3` — the store-and-forward (opened at the Phase-7 close; **the `.1`–`.4` lanes COMPLETE**; `.5` the regional-routing census → decomposed `.5.1`–`.5.4`; `.5.1` ADR-035; `.5.2` the regional routing) | repo-local |
| [`PHASE-9`](tasks/PHASE-9.md) | `proposed` | `.1` — stable release | repo-local |
