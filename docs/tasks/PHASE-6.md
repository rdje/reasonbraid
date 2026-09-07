# PHASE-6: policy and doctrine governance

## Metadata

- Tree ID: `PHASE-6`
- Status: `proposed`
- Roadmap lane: Phase 6 (`ROADMAP.md` §20.8); Governance track
- Created: `2026-09-05`
- Estimate: 18–30 engineer-weeks plus governance review
- Depends on: authority model, rigorous deliberation, Git/object consistency, correction model
- Exit: G3 and publication portions of G7. A full policy lifecycle — including failed publication recovery and later correction — can be reconstructed without relying on chat prose.

## Goal

Canonical semantic policy, deterministic projections, signed publication,
per-target deployment (not globally atomic), drift, and designed correction.
Consensus does not grant authority. Demonstration B (`ROADMAP.md` §26.2).

## Non-Goals

- Silent overwrite or history deletion.
- Globally atomic multi-repository rollout.
- Universally weaker retraction authority than adoption.

## Task Tree

- ID: `PHASE-6.1`
  Status: `done`
  Goal: semantic policy schema, layer/precedence/exception rules, impact maps, ownership metadata
  Backlog: 38
  ADR: 019
  Roadmap: §15.1–15.3
  Children: `.1.1`–`.1.3` (decomposed `2026-09-07` at the census
    seams): `.1.1` ADR-019 + the census (the semantic-policy
    contract: the `PolicyVersion`/`PolicySetVersion` shapes,
    the seven-step layered resolution with the fail-closed
    rule, the ownership = the authority binding) → `.1.2` the
    policy schema (the typed `PolicyVersion` + the validation
    + the versioned registry + the ownership metadata) →
    `.1.3` the layering + the precedence (the seven-step
    resolution, the exceptions/waivers, the explanation tree,
    the impact maps).
  Done (`2026-09-07`): the census at the seams — the block is
    LIFTED (the frontier row's "blocked on authority,
    deliberation, Git/object, correction model" is resolved:
    the authority ships (Phase 2's boundaries/grants/audit),
    the deliberation ships (Phase 5 closed), the Git/object
    consistency ships (Phase 4's git pack + the snapshots),
    the correction model ships (the revision lineage + the
    supersession/retraction vocabulary rides `.5`). The
    GREENFIELD: no policy schema exists (no `PolicyVersion`,
    no layer/precedence rules, no impact maps, no
    policy-ownership binding — `git grep -c "PolicyVersion"
    HEAD -- crates/ migrations/` → rc=1); the REUSABLE
    pieces: the versioned-config precedent (the Phase-5.1
    profile registry — the versioned entry + the digest
    pinning), the content-addressing (ADR-011), the authority
    model (the grant checks — the §15.3 step-1 substrate),
    the evidence pipeline. The §23 queue's 019 (the canonical
    policy schema + the merge/precedence rules) is THIS
    leaf's ADR. Frontier → `.1.1`.

  - ID: `PHASE-6.1.1`
    Status: `proposed`
    Goal: ADR-019 + the census — the semantic-policy
      contract: the `PolicyVersion` shape (the §15.1 fields),
      the `PolicySetVersion` (the compatible authorized
      collection + the lock manifest), the seven-step layered
      resolution (the issuer authority, the applicability, the
      dependencies/conflicts, the charter precedence, the
      exceptions, the FAIL-CLOSED binding conflict, the
      explanation tree), the ownership = the authority binding
      (the policy's owning authority rides the grants — the
      label grants nothing, §15.3). No code.
    ADR: 019
    Roadmap: §15.1–15.3

  - ID: `PHASE-6.1.2`
    Status: `proposed`
    Goal: the policy schema — the typed `PolicyVersion` (the
      §15.1 fields: the stable id + the semantic version +
      the digest + the lifecycle status, the normative
      statements with the stable clause ids, the applicability
      selectors, the dependencies/conflicts/precedence hints,
      the exception schema, the ownership metadata) + the
      validation (the clause ids, the digest, the ownership
      reference) + the versioned registry.
    Roadmap: §15.1, §15.2

  - ID: `PHASE-6.1.3`
    Status: `proposed`
    Goal: the layering + the precedence — the seven-step
      resolution (the issuer authority check, the
      applicability filter, the dependency/conflict
      resolution, the charter precedence/specificity, the
      exceptions/waivers, the fail-closed on the unresolved
      binding conflict, the explanation tree), the impact
      maps (the clause → the target/domain/action coverage).
    Roadmap: §15.3

- ID: `PHASE-6.2`
  Status: `proposed`
  Goal: proposal/review/approval records with authority proofs and quorum snapshots
  Roadmap: §4.5, §15.6
  Acceptance: discussion, decision, approval, publication, and deployment remain separate records

- ID: `PHASE-6.3`
  Status: `proposed`
  Goal: deterministic compiler plus initial Codex and Claude-family projections; unrepresentable clauses fail closed
  Roadmap: §15.5

- ID: `PHASE-6.4`
  Status: `proposed`
  Goal: signed canonical publication protocol; kill-point-tested Git/PostgreSQL reconciliation
  Backlog: 39
  ADR: 020
  Roadmap: §15.7–15.8

- ID: `PHASE-6.5`
  Status: `proposed`
  Goal: canary target deployment, PR/apply adapters, receipts, drift, waivers, suspension, supersession, retraction
  ADR: 021
  Roadmap: §15.9–15.11, §4.7

- ID: `PHASE-6.6`
  Status: `proposed`
  Goal: outcome monitoring and scheduled review triggers
  Roadmap: §15.11

- ID: `PHASE-6.7`
  Status: `proposed`
  Goal: G3 exit + Demonstration B; publication portion of G7; subtraction record
  Gate: G3; G7 publication portion
  Kill/pivot: do not ship binding policy governance unless real owners accept the authority/correction model (`ROADMAP.md` §25.1)

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `PHASE-6.1.1` | `proposed` | `.1` decomposed at the census seams (the block is LIFTED — Phase 5 closed the deliberation dependency; the policy schema is the greenfield) — ADR-019 opens the lane |

## Changelog

- `2026-09-07`: `.1` decomposed at the census seams — the block
  is LIFTED (the authority/deliberation/Git-object/correction
  dependencies all ship; the policy schema is the greenfield);
  children `.1.1` (ADR-019 + the census) → `.1.2` (the policy
  schema) → `.1.3` (the layering + the precedence); frontier
  → `.1.1`.
- `2026-09-05`: Created from `ROADMAP.md` §20.8, §15, §26.2, backlog 38–39.
