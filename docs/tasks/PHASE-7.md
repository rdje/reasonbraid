# PHASE-7: Internet-qualified operation

## Metadata

- Tree ID: `PHASE-7`
- Status: `proposed`
- Roadmap lane: Phase 7 (`ROADMAP.md` §20.9); Trust track (scoped)
- Created: `2026-09-05`
- Estimate: 18–32 engineer-weeks plus external review
- Depends on: Phase 1, applicable Phase 2 trust/recovery, and every feature-specific gate for the surface being exposed. Does **not** require Phases 3–6 for capabilities that remain disabled and unclaimed.
- Exit: G6–G7 for a named capability profile only.

## Goal

Internet capability is claimed only for a qualified feature set. Adding remote
discovery, arbitrary resources, governed policy, or another adapter later
reopens the applicable portions of G4–G7.

## Non-Goals

- Permissionless or anonymous public agent network.
- Exposing unfinished tracks behind an experimental default.

## Task Tree

- ID: `PHASE-7.1`
  Status: `proposed`
  Goal: hardened ingress/egress, mTLS workload identity, tenant isolation, quota/abuse, secret-manager integration, regional/data-class controls
  Roadmap: §16.2, §16.8, §16.11

- ID: `PHASE-7.2`
  Status: `proposed`
  Goal: public-node enrollment and quarantine, revocation propagation, signed software updates, SBOM/provenance, disclosure process
  Backlog: 40
  ADR: 027
  Roadmap: §16.10, §16.12

- ID: `PHASE-7.3`
  Status: `proposed`
  Goal: horizontally scalable coordinator workers only where measurements require them
  ADR: 002 (extraction criteria)

- ID: `PHASE-7.4`
  Status: `proposed`
  Goal: capacity/load tests, incident exercises, penetration-test remediation, production runbooks
  Roadmap: §16.12, §18.6

- ID: `PHASE-7.5`
  Status: `proposed`
  Goal: G6–G7 exit for a named capability profile; subtraction record; explicit unsupported matrix
  Gate: G6, G7
  Kill/pivot: do not expose remote enrollment if the qualification gate is incomplete (`ROADMAP.md` §25.1)
  ADR: 022

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| — | `PHASE-7.1` | `proposed` | blocked on Phase 1 + applicable Phase 2; open only for the surface being exposed |

## Changelog

- `2026-09-05`: Created from `ROADMAP.md` §20.9, §16.12, backlog 40.
