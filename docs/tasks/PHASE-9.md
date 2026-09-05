# PHASE-9: stable product release

## Metadata

- Tree ID: `PHASE-9`
- Status: `proposed`
- Roadmap lane: Phase 9 (`ROADMAP.md` §20.11, §21)
- Created: `2026-09-05`
- Estimate: 12–24 engineer-weeks after beta evidence
- Depends on: sustained operational and user evidence
- Exit: G9 plus product release criteria in §21. Stability is earned by evidence, not elapsed time.

## Goal

A named release authority accepts an immutable gate manifest for declared
profiles. “All planned features implemented” is neither necessary nor
sufficient (`ROADMAP.md` §24.5).

## Non-Goals

- Claiming Stable 1.0 for unclaimed adapters, resolvers, or federation modes.
- Treating roadmap document version 0.4.1 as a software version.

## Task Tree

- ID: `PHASE-9.1`
  Status: `proposed`
  Goal: usability/accessibility (agreed WCAG profile) and human-review interface audit
  Roadmap: §5 accessibility objective

- ID: `PHASE-9.2`
  Status: `proposed`
  Goal: performance and cost optimization from measured SLOs, not premature microservices

- ID: `PHASE-9.3`
  Status: `proposed`
  Goal: retention/privacy administration, legal-hold, redaction, export

- ID: `PHASE-9.4`
  Status: `proposed`
  Goal: installation/upgrades, support policy, documentation (this book)
  ADR: 028

- ID: `PHASE-9.5`
  Status: `proposed`
  Goal: long-duration soak, repeated recovery exercises, evaluation stability, closure of beta findings

- ID: `PHASE-9.6`
  Status: `proposed`
  Goal: G9 exit — signed artifacts/SBOM/provenance, runbooks, capability/limitation matrices, known-risk disposition, named release authority
  Gate: G9; subtraction record required
  Roadmap: §21 Stable 1.0, §24.5

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| — | `PHASE-9.1` | `proposed` | blocked on sustained beta evidence |

## Changelog

- `2026-09-05`: Created from `ROADMAP.md` §20.11, §21, §24.5, ADR 028.
