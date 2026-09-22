---
answers:
  - Why can the §15.8 reconciler not simply be given a caller?
  - What must a publication record before its Git write, and why?
  - Why must the publication commit be reproducible?
  - In what order are the pieces built?
---
# A publication records its Git operation before attempting it

- **Type:** decision
- **Status:** accepted
- **Owner:** `SIGNOFF-REPAIR.9.3.5.1`
- **Date:** 2026-09-22
- **Cites:** ROADMAP §15.7 step 4 (*in one database transaction, store `publication_staged`,
  manifest digest, **desired Git operation**, and outbox item*), §15.8 (the reconciliation
  matrix, *idempotent and regularly exercised under kill points*)

## Context

`.9.3.5.1` opened as *the reconciliation matrix is decided and never operated*: the pure
`reconciler::reconcile` has no production caller. Building that caller showed two
prerequisites that the leaf did not name.

🔴 **1. Nothing records which repository a publication was written to.** The publish verb
takes `repo_path` in its request and uses it. `policy_publications` has no column for it,
before or after the write. `mark_effective` checks the object ids against the repository and
does not store the path. So for a publication interrupted after its Git write and before
`mark_effective`, the database cannot say where to look. No reconciler can observe a state it
cannot locate.

🔴 **2. The publication commit cannot be reproduced.** `publisher::publish` stamps the commit's
author and committer with `Time::now_local_or_utc()`. The same manifest and bundle,
published twice, produce two different commit ids. That breaks two rows of the matrix:

- `RetryStagedWrite` says *the same content commits identically — the idempotent retry*.
  With a wall-clock timestamp it does not. A retry writes a different commit.
- `VerifyAndAdvance` compares the observed immutable ref with an *expected* object id, and
  nothing can compute that id after the fact.

The effective CAS has the same gap. The publish request supplies `expected_effective`, and
nothing records it, so a retry cannot redo the CAS that was attempted.

⭐ Both gaps are what §15.7 step 4 names: the *desired Git operation* is to be stored **in the
database before** the Git write. The code writes first and records after.

## Decision

1. **`.9.3.5.1.1` records the desired Git operation before the write.** The publish verb
   stores the repository (as requested, relative to the configured root, following §12's
   root-relative policy) and the expected effective id, committed **before** the first Git
   object is written. A publication that has begun publishing is therefore locatable
   whatever happens next.
2. **The commit becomes reproducible.** Its timestamp is the publication's own staging time
   (`created_at`, whole seconds, UTC) rather than the clock at write time. The commit id is
   then a pure function of the recorded publication and its content, so it can be recomputed.
   Git objects are content-addressed, so rewriting them to recompute an id is harmless.
3. **`.9.3.5.1.2` builds the driver on top.** It observes the three refs, recomputes the
   expected id, calls `reconcile`, and applies the action. Two actions are never automatic:
   `StopSecurityAlert`, and the `failed`-then-appearing quarantine (*never silently
   promoted*). The kill-point controls, the idempotent re-run, and ROADMAP §7.2's
   `reasonbraid-reconciler` binary all belong to this leaf.

## Consequences

- Rows staged before `.1` have no recorded operation. Once `.2` exists they answer *cannot be
  reconciled: no recorded Git operation*, and they are not guessed at.
- An effective publication's commit id changes meaning. It used to name "whenever it was
  written"; it now names "this publication's content at its staging time". Existing
  effective rows keep the ids they recorded, and nothing rewrites them.

## What would make this wrong

- ⛔ If two publications could share an id and a staging second, their commits would collide.
  The publication id is part of the commit message and the id is the primary key, so they
  cannot.
- ⛔ If a publication must be re-published to a *different* repository after a failure, the
  recorded repository is too strict. §15.7 does not describe that path.
