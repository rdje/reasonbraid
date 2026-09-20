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
| [`SIGNOFF-REPAIR`](tasks/SIGNOFF-REPAIR.md) | `active` | `.9.3.4` — an administrative authority cannot name what it is authority OVER: `GrantAction` has ten variants and `TargetSelector` two, so a grant binds to its HOLDER but not to its SCOPE for the administrative surfaces (a correction, a deployment target, a policy approval). ⚠️ Both are closed wire vocabularies §9.8 publishes, so extending them is coordinated with `.11.7.1`. ✅ **`.6.8.2.1` (REPAIR-0335): a ledger row cited a version its own stated source no longer held** — `rustls 0.23.43 (Cargo.lock)` against a lock resolving 0.23.45; the drift is now a gate arm, falsified against the real historical defect, and `checked_at` was NOT bumped for a partial revalidation. ✅ **`.6.8.2` (REPAIR-0334): §7.4's dependency ledger advertised two mechanisms and NOTHING read the file** — no script, workflow or Makefile target opened it, while the MCP row sat 16 days stale on the day `.6.8` shipped its surface; revalidated from primary sources, and `census_external_ledger.py` now gates the shape, warns on age in CI, and gives `.6.6`'s upstream wait a mechanical trigger. 🔴 That trigger's first version announced the block had LIFTED while finding nothing. ✅ **`.6.8.1` (REPAIR-0333): eight of ten binaries are signed, and the list is DERIVED so it cannot drift again** — `make release` named FOUR while the line above built TEN, and two of the six omitted are the acquisition workers `rb-server` spawns from its own directory. ✅ **`.6.7` (DOC-0084): the listen gateway owes a DIALLER before an operator surface** — `ListenUpstream` has exactly one implementor and it is in the test suite, so a route would configure an upstream no code can reach; the dialler owes `ssrf::evaluate`, a named site authority, and a decided credential holder. ✅ **`.6.6` (DOC-0083): the MCP HTTP transports are blocked by ONE upstream split, and it is not ours to resolve** — every Streamable-HTTP profile brings `base64 0.23.1` against a `multiple-versions = "deny"` policy, and `base64 ^0.22` is required by 12 packages against 1 requiring `^0.23` (`rmcp`), so both the client and the deferred server half are **blocked on a third party** rather than *not yet*. ✅ **`.6.8` (REPAIR-0331): the MCP tools are REACHABLE** — `rb-mcp` serves them over stdio, a profile PRICED with `cargo metadata` (`transport-io` +0 packages against the HTTP server's +3, one of them a second `base64` major that `deny.toml` forbids outright). 🔎 The first `initialize` this project answered named the SDK, not the product. 🔎 `make release` signs 4 of 10 binaries → `.6.8.1`. 🔴 **`.11.24.1.6.1` (REPAIR-0330): a task tree has THREE relative-reference dialects, and not one of those references meant a lane** — the leaf's own premise refuted by its census; a lane is an organising number, not a node; and the ratchet re-priced one commit after registration, `internally-ambiguous` declined at 5 rises in 30, baseline 32 → 1. ✅ **`.11.24.1.4.1` (REPAIR-0329): a release identity must be PUBLISHABLE, or the signature is ceremonial** — `verify` took the PRIVATE key and derived the public one, so only the signer could verify and the check caught corruption rather than forgery; `pubkey` + `--public-key` ship, `make release` exports the identity, and the control DELETES the private key before verifying. ✅ **`.11.24.1.6` (REPAIR-0328): a relative leaf reference resolves against its own tree, and TWO DIALECTS disagree about what that means** — 5,634 references censused (home 3,089 · shared 2,178 · internally-ambiguous 126 · foreign 165 · dangling 76), `PHASE-1.md` writes 194 of its references in a phase-repeating dialect nobody wrote down, the convention is now in `docs/TASK_TREE_README.md`, and `RELATIVE-LEAF-REF` is registered as a ratchet priced at 1 rise in 30 commits — while `foreign` is DECLINED at 3 in 30 and the gate is stated NOT to catch the opening instance. ✅ **`.11.24.1.4` (REPAIR-0327): a re-key signs the MANIFEST, not the binaries** — the signing-key runbook prescribed *re-sign the SAME manifest content* and no command could do it, and the obvious workaround is measurably wrong: `generate` stamps `created_at: Utc::now()`, so two runs over one unchanged binary differ. `re-sign` signs the bytes verbatim, refuses the OLD key by name (deterministic Ed25519 would re-key nothing) and never overwrites a signature. ⚠️ `verify` still needs the PRIVATE key → new leaf `.11.24.1.4.1`. ✅ **`.11.24.1.5` (DOC-0082): there is NO in-process acquisition trait and there will not be one** — `PHASE-8.4.1` deferred it to `.4.4`, which closed having delivered the ADR-027 allowlist ladder instead, and the promise sat in `resolver.rs`'s module header the whole time; ADR-018's `sandbox_level` states what the CODE provides and the server filters on it, so in-process third-party code could advertise `vm_container` while structurally being `none`. 🔎 And the SDK's front door never mentioned resolvers at all. ✅ **`.11.24.1.3.2` (REPAIR-0325): a git snapshot is a REFERENCE, because the object database it acquires is not an identity** — one immutable commit yields 6857 bytes against 10071, two different digests, when the source repacks with `pack.window 0`, since the pack is built by `git-upload-pack` on the REMOTE; §12.9's second alternative taken with `resolved_commit` as the identity, and `storage_class` given four readers because `.11.24.1.3` had measured it as read by none. ✅ **`.11.25.1` (REPAIR-0323): the browser's stderr is held by two `chrome_crashpad_handler` outside the owned process group** — `ppid 1`, a group each — named by kernel pipe identity with a positive control in the same run, so `kill_process_group` cannot reach them by construction. ⛔ **And the hold does not reproduce: 22 runs, drain 0–1 ms, none censored**, while the control that opened `.11.25` now passes 18/0 with no predicate changed since it failed. No product change; seven launch flags suppress nothing and there is nothing to bound. ⚠️ **`.11.26` stays open and is NOT the frontier** — its remaining acceptance is to catch one occurrence of a failure that has not recurred in 42 runs, which is a wait rather than a slice. ✅ **`.11.34` (REPAIR-0321): two controls written to close a coverage gap had never once run in CI** — the `pg-tests` job never built the workspace bins, so both R2 joins aborted on an absent `reasonbraid-extract`; added 2026-09-12, last green run 2026-09-11, masked eight days by the runner breaking at the first failing suite. ✅ **`.11.30` (REPAIR-0319): the proof carried a nanosecond the ledger never stored** — the second push's one red test was a REAL product defect: the reservation reference kept a Rust nanosecond instant while the row holds microseconds and the ledger lends against the row. Reproduced on the macOS host whose microsecond clock made the original check unfalsifiable. ✅ **`.11.29` (REPAIR-0318): the demonstration read presence without saying who was calling** — and wrote a markdown table by executing it, silently emptying a cell in every evidence bundle. ✅ **`.11.28` (0317): two enrollment expectations counted quota rows**, plus a third site the broad run found. ✅ **`.11.27` (0316): a self-test that pins one host's collation is not ground truth.** ⭐ All three were pre-existing consumers left behind by a producer's change, surfaced by the first remote run | repo-local |
| [`PHASE-8`](tasks/PHASE-8.md) | `active` | corrective prerequisite `SIGNOFF-REPAIR`; then `.5.3` — the store-and-forward (opened at the Phase-7 close; **the `.1`–`.4` lanes COMPLETE**; `.5` the regional-routing census → decomposed `.5.1`–`.5.4`; `.5.1` ADR-035; `.5.2` the regional routing) | repo-local |
| [`PEER-COLLAB`](tasks/PEER-COLLAB.md) | `active` | `.1` — the census: walk the director's A↔B bug-report scenario step by step against the SHIPPED system and grade each step with an artefact. A director-requested acceptance track, off the frozen v0.4.1 roadmap; it MEASURES rather than promoting features, and its two expected gaps are parked in `docs/parking-lot.md` with this tree as their trigger | repo-local |
| [`PHASE-9`](tasks/PHASE-9.md) | `proposed` | `.1` — stable release | repo-local |
