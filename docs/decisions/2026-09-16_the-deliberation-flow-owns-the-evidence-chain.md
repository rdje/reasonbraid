---
answers:
  - Should the deliberation subsystem and the evidence chain be joined, or are they separately intended?
  - What does ROADMAP §13.2 require of a thread that reaches an `assess` step?
  - Why is `claim_assessments.claim_id` free text, and what should it be?
  - Should a contribution's EvidenceRef resolve to a registered resource reference?
  - Is the claim-evidence graph reachable from a deliberation?
---
# The deliberation flow owns the evidence chain — §13.2 already specifies the join

- **Type:** decision
- **Status:** accepted
- **Owner:** `SIGNOFF-REPAIR.11.14.3`
- **Date:** 2026-09-16

## Context

`SIGNOFF-REPAIR.11.14.2` measured, while deciding what a claim is scoped to,
that the two subsystems share nothing. The census runs in both directions:

```bash
# evidence-store symbols inside the deliberation modules
for f in threads.rs agg.rs projections.rs workflows.rs matching.rs; do
  git grep -c 'snapshots::\|derivations::\|claims::\|resource_references\|evidence_snapshots' \
    -- crates/reasonbraid-server/src/$f; done      # -> 0 0 0 0 0
# deliberation symbols inside the evidence modules
for f in claims.rs snapshots.rs derivations.rs resources.rs; do
  git grep -c 'threads::\|ClaimRecord\|contribution\|thread_id' \
    -- crates/reasonbraid-server/src/$f; done      # -> 0 0 0 0
```

Nine modules, zero references. No call, no shared key, no shared type.

## The correction this record has to make first

⛔ **`SIGNOFF-REPAIR.11.14.3` was opened saying the specification "settles it
neither way", and that is wrong.** It was written from §12.7 (which lives in the
evidence chapter and names no thread) and §5.2.2 (which defines claims without
naming the evidence store). **§13.2 was not read.** It is the section that
settles it, and it is in the same frozen document:

> ### 13.2 Rigorous deliberation reference flow
> 2. **Register context and resource references**; identify unavailable or sensitive context.
> 5. **Normalize claims**, assumptions, disagreements, and requested evidence.
> 6. **Acquire/assess evidence** within the allowed plan.

The canonical deliberation flow *contains* reference registration and evidence
assessment as its own steps. The two subsystems were never two designs; they are
two halves of one specified flow, built separately, with the flow between them
unimplemented.

## The measurement that makes it unambiguous

The product already encodes §13.2's steps, and ships profiles that use them.

`workflows.rs:17` — `STEP_KINDS` carries thirteen steps including
`evidence_request` **and** `assess`. `migrations/0032_workflow_profiles.sql`
seeds eight built-in profiles, and **two of them declare `assess`**:

| built-in profile | steps |
| --- | --- |
| `evidence_review` | `solicit`, `evidence_request`, **`assess`**, `decide` |
| `policy_proposal` | `solicit`, `revise`, **`assess`**, `vote`, `approve` |

Now the other half of the measurement:

```bash
git grep -n '"assess"' -- crates/reasonbraid-server/src   # -> 1 hit
```

**One hit — the vocabulary constant itself.** No contribution kind maps to
`assess`, nothing gates on it, nothing writes through it. Compare
`evidence_request`, which *is* wired: `threads.rs:1436` refuses the contribution
unless the current step is `evidence_request`, and `:1430` requires a
`target_claim_digest`.

⛔ So a tenant running the shipped `evidence_review` profile — the built-in whose
entire purpose is reviewing evidence — advances onto an `assess` step at which
no assessment can be recorded against the thread, while a complete assessment
store sits beside it holding five assessment kinds, excerpt-validated against
the snapshot's own bytes.

## The decision

⭐ **The deliberation flow owns the evidence chain. The join is implemented at
the two points §13.2 already names, and neither namespace stays free.**

1. **`assess` becomes a real step.** A contribution on it records a
   `claim_assessments` row. Its `claim_id` is a **claim digest of this thread** —
   the server-computed ADR-011 digest from `ClaimRecord`, membership-checked the
   way `threads.rs:1605` already checks a targeted claim digest — and its
   snapshot must be one the thread's tenant cited (`evidence_citations`, from
   `SIGNOFF-REPAIR.11.14.1`). This is §13.2 step 6.
2. **A contribution's `EvidenceRef` resolves to a registered reference.**
   `EvidenceRef { uri, digest }` and `resource_references
   UNIQUE (original_locator, expected_digest)` are **the same key**. A
   contributor naming a URL at a digest is naming exactly the row the evidence
   store would hold. This is §13.2 step 2.

⭐ **Every part the join needs already exists**, which is itself evidence that
this is wiring rather than design: the claim digest and its membership check
(`threads.rs`), the citation gate (`.11.14.1`), the authoring-tenant binding
(`.11.14.2`), the excerpt validation against raw bytes (`claims::submit`), and
the reference replay (`resources::submit`).

## The rejected alternatives

1. **Leave both namespaces free and document it.** Rejected on §13.2: the flow
   is specified, and two shipped built-in profiles declare a step that would
   stay inert. Documenting an inert built-in as intended is how a gap becomes a
   feature.
2. **Bind the claim end only** (`.11.14.2`'s original framing). Rejected: it
   would make `claim_id` a minted digest while a contribution's own citations
   stayed unresolvable strings, so the graph would be checked at one end and not
   the other — the exact asymmetry `.7.4`'s attached context already flags for
   `author`.
3. **Make the evidence chain thread-scoped outright** — a `thread_id` column on
   the evidence tables. Rejected on the same schema argument as
   `2026-09-16_evidence-is-shared-the-read-is-tenant-bound.md`: the chain is
   content-addressed and shared by construction. The join belongs on the
   citation and the assessment, which are per-tenant and per-author
   respectively, not on the shared receipt.
4. **Treat `assess` as a generic discussion step** and delete it from the
   vocabulary. Rejected: `STEP_KINDS`'s own contract is that each step "names an
   existing contribution/terminal/budget surface", and the surface exists. The
   defect is that it is unreachable, not that it is absent.

## What this decision does NOT say

It does not say the standalone `POST /v1/assessments` route disappears; whether
an assessment may exist outside a deliberation is `.11.14.3.1`'s to state. It
does not decide what becomes of rows already written against invented
identifiers — that is `.11.14.3.3`. And it does not widen the frozen roadmap:
every clause above is §13.2 as written.
