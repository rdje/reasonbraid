# Phase 2's G6–G7 feed: what the phase proved against the Internet-security and operations gates (`PHASE-2.7.4`)

- Date: 2026-09-07 · Leaf: `PHASE-2.7.4` · Decision record

## Context

Phase 2 exits feeding G6 (Internet security — §16.12 evidence + external
review) and G7 (operations — load, backup restore, chaos/game day, SLO
instrumentation). This record maps each gate line to the Phase-2 evidence
that feeds it, or names it OPEN with the profile that re-opens it. The
mapping is the hand-off: the next deployment profile starts from this list,
not from a re-census.

## The §16.12 feed (G6)

| Line | Phase-2 status |
| --- | --- |
| Externally reviewed threat model and abuse cases | **OPEN** — no external review (the Internet profile's own gate) |
| Authenticated enrollment, rotation, revocation, and tenant-isolation tests | **EVIDENCE** — the enroll bootstrap + the workload certs (`.1.2.1`), the channel v3 proof handshake + rotation (`.1.2.2`), the node/grant/boundary revocation (`.1.3`), the cross-tenant adversarial suite (`.7.1` — create 403 audited, read 404, replay 409) |
| Authorization non-escalation properties and confused-deputy tests | **EVIDENCE** — the `.7.1` suite (the delegation scope is the ceiling even for a deputy whose own grant covers the target; the re-arm fence; the revocation fence around history) |
| SSRF / DNS-rebinding / redirect / archive-bomb suite | **OPEN** — no arbitrary-reference surface (S-2) |
| Prompt-injection action-boundary suite | **OPEN** — no policy projection (S-3) |
| Dependency / SBOM / provenance / release-signing pipeline | **PARTIAL** — the dependency ledger + `cargo deny` + the pinned toolchain exist; SBOM/provenance/signing wait for the signing key (S-4, G9) |
| Backup restore and compromised-key recovery exercise | **EVIDENCE** — the `.4.1` restore exercise (every guard pass) + the `.7.2` replacement drill (the destroyed credential's ritual: revoke → replacement enroll → fence → replay) |
| Rate-limit, cost-circuit-breaker, and notification-storm tests | **PARTIAL** — the spend circuit breakers (`.3.2`, measured); rate-limit machinery is the `.6.3` deferral; notification-storm tests need the storm injector (Phase 7) |
| Penetration test with critical/high findings resolved or release cancelled | **OPEN** — external (the Internet profile's own gate) |
| Incident runbooks, contacts, evidence preservation, and disclosure process | **PARTIAL** — the node lost/replaced runbook (`.5.3` + the `.7.2` ritual) + the evidence-bundle pattern; contacts/disclosure are the production profile's |

## The G7 feed

| Line | Phase-2 status |
| --- | --- |
| Load | **OPEN** — no workload measurement (S-8; SLO-5's trigger) |
| Backup restore | **EVIDENCE** — the `.4.1` exercise on every guard pass + the migration-upgrade path (`.4.2`) |
| Chaos / game day | **PARTIAL** — the demo's real kill points + the `.7.2` replacement drill are the dev-profile game-day set; at-scale churn is Phase 7's |
| SLO instrumentation | **EVIDENCE** — the `.5.2` metrics surface (the seven counters, record-anchored) + the `.5.3` SLO record (SLO-1…SLO-5, zero error budget) |

## answers:

- **Every gate line is either fed by a named measured artifact or OPEN with its re-opening profile** — the G6–G7 hand-off is a checklist, not a narrative.
- **The dev profile's own SLOs run the same guard the evidence rides** — SLO-1…SLO-4 measure the 17 live suites + the demo + the restore + the kill beats on every pass; a red pass halts the frontier (the zero error budget).
- **Phase 2's exit does not claim any gate is MET** — G6 and G7 stay open until their named deployment profiles exist; this record is the evidence inventory they will consume.
