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
| [`SIGNOFF-REPAIR`](tasks/SIGNOFF-REPAIR.md) | `active` | `.10.1` — ✅ `.9.1` CLOSED (`.9.1.7` REPAIR-0508: refusals say what happened, outages are 500s, versions are SemVer; `.9.1.8` deferred). `.9.1.6` REPAIR-0507: the resolver's dependency, precedence and set-shape steps refuse what the design refuses. `.9.1.5` REPAIR-0506: a selector is exactly a layer and a target, and a malformed one is refused, never widened. `.9.1.4` REPAIR-0505: the server writes the policy.lock from the registry. `.9.1.3` REPAIR-0504: the server derives a policy's digest, and reads say whether it still verifies. `.9.1.2` REPAIR-0503: a policy registrar must hold the grant it names as owner. ⭐ DOC-0177 censused `.9.1`: 6 blocking children (`.9.1.2` class 1, `.3`–`.6` class 2, `.7` class 3), `.8` deferred; the published `policy.lock` is caller-written (`.9.1.4`, new). Blocking leaves by exposure (`.11.4.3.1.10` REPAIR-0502: the §8 cleanup record; `.11.31.2` REPAIR-0501: every bound instant judged, the check wired; `.7.1` closed DOC-0174; `.7.1.6` REPAIR-0500: the registry censuses run on every commit; `.7.1.5` REPAIR-0499: the resolver rank is total; `.7.1.4.1` REPAIR-0498: a first citation no longer confirms another tenant's; `.7.1.4` REPAIR-0497: a tenant's citation is its own statement; it opened `.7.1.4.1`; `.7.1.3.1` REPAIR-0496: registering a resolver is a site act; it opened `.7.1.6`; the closing censuses are done: `.7.3.2` deferred DOC-0171; `.7.2` closed DOC-0170; `.11.4.5`, `.3.3.4.3.3.3.3.2.3`, `.11.4.3.1.7` closed DOC-0167–0169; `.9.2` DOC-0166: 1 blocking `.9.2.2`; `.8.2` DOC-0165: 2 blocking; `.7.4` DOC-0163: 4 blocking, 3 deferred); the tree now ends at a BUG BAR (DOC-0162: 25 blocking + `.12.2`, 12 deferred with triggers), then `.7.1.3.1` (`.7.1.3` done REPAIR-0494); ⭐ `.4.5` COMPLETE (REPAIR-0483…0490, DOC-0156/0157) (`.4.5.1`…`.4`, `.6` done REPAIR-0483…0487, `.5` deferred DOC-0157; then `.1.1` adjudicated holds); `.11.26` 0 in 299; `.4.4` COMPLETE (DOC-0153 censused `.4.4`: ✅ `.4.4.1` REPAIR-0459 → ✅ `.4.4.2` REPAIR-0460 (✅ `.4.4.2.1` REPAIR-0474; ✅ `.4.4.2.2` REPAIR-0475) → ✅ `.4.4.3` REPAIR-0461 → ✅ `.4.4.4` REPAIR-0462…0464 → ✅ `.4.4.5` REPAIR-0465…0469 → ✅ `.4.4.6` REPAIR-0470 (✅ `.4.4.6.1` REPAIR-0476) → `.4.4.7`; + ✅ `.4.4.8` REPAIR-0467 → ✅ `.4.4.9` REPAIR-0468 → ✅ `.4.4.10` DOC-0154, REPAIR-0471…0473). ✅ `.4.3` complete (REPAIR-0455…0458). ✅ `.5` complete. Nothing waits on the director. | repo-local |
| [`PHASE-8`](tasks/PHASE-8.md) | `active` | corrective prerequisite `SIGNOFF-REPAIR`; then `.5.3` — the store-and-forward (opened at the Phase-7 close; **the `.1`–`.4` lanes COMPLETE**; `.5` the regional-routing census → decomposed `.5.1`–`.5.4`; `.5.1` ADR-035; `.5.2` the regional routing) | repo-local |
| [`PEER-COLLAB`](tasks/PEER-COLLAB.md) | `active` | `.1` — the census: walk the director's A↔B bug-report scenario step by step against the SHIPPED system and grade each step with an artefact. A director-requested acceptance track, off the frozen v0.4.1 roadmap; it MEASURES rather than promoting features, and its two expected gaps are parked in `docs/parking-lot.md` with this tree as their trigger | repo-local |
| [`PARTICIPATION`](tasks/PARTICIPATION.md) | `active` | `.1` — the census: grade the director's 2026-09-25 requirements (email-like delivery to an offline agent; temporarily vs permanently offline; production CLI and web clients for humans; any chat app or agent — ChatGPT, Claude, Gemini, DeepSeek, Kimi, Qwen, GLM, MiniMax, MiMo — taking part through a connector or an adapter) against the SHIPPED system. Sequenced after the corrective exit bar (`REASONBRAID-DOC-0162`); measurement only if run earlier | repo-local |
| [`PHASE-9`](tasks/PHASE-9.md) | `proposed` | `.1` — stable release | repo-local |
