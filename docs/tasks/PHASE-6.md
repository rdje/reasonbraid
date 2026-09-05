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
  Status: `proposed`
  Goal: semantic policy schema, layer/precedence/exception rules, impact maps, ownership metadata
  Backlog: 38
  ADR: 019
  Roadmap: §15.1–15.3

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
| — | `PHASE-6.1` | `proposed` | blocked on authority, deliberation, Git/object, correction model |

## Changelog

- `2026-09-05`: Created from `ROADMAP.md` §20.8, §15, §26.2, backlog 38–39.
