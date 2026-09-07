# Roadmap and Phase 0

ReasonBraid is planned from two companion files at the repository root:

| File | Role |
| --- | --- |
| `ROADMAP.md` | Frozen v0.4.1 master: architecture, scope, and gates |
| `KICKOFF.md` | Phase 0 day-to-day execution plan |

They are one pair, not two competing roadmaps. A change to one must stay
consistent with the other.

## What is frozen

Roadmap v0.4.1 is the execution baseline. v0.5.0 is forbidden until the Phase 0
evidence package and a working Phase 1 LAN vertical slice exist — both now do
(ADR-002's package and the Phase-1 G1–G2 close), so the precondition is met and
opening v0.5.0 is the director's call. New features
and speculative refinements go to a parking lot, not back into the master.

## Delivery shape

Phases 0 and 1 are sequential. After the LAN foundation, later phase numbers
are work packages on parallel tracks (trust, network, evidence, quality and
governance), not a mandatory single-file sequence.

| Phase | Purpose | First evidence |
| --- | --- | --- |
| 0 | Contracts and kill-risk experiments | G0 for identity, authority, thread, delivery, budget |
| 1 | Trustworthy LAN vertical slice | G1–G2; Demonstration A |
| 2 | Delivery, identity, recovery | Authority non-escalation; restore; no silent retry of unknown provider outcomes |
| 3 | Directory, unknown membership | Recruit without enumerating the network |
| 4 | Resources and evidence | G4; explicit failure for unsupported references |
| 5 | Deliberation quality and routing | G5 on declared domains |
| 6 | Policy and doctrine governance | G3; reconstructable policy lifecycle |
| 7 | Internet-qualified operation | G6–G7 for a named capability profile |
| 8 | Federation and interoperability | G8 |
| 9 | Stable product release | G9 |

## Phase 1 — complete (G1–G2 Met)

The trustworthy LAN vertical slice (`ROADMAP.md` §20.3) is **complete**:
Demonstration A passed **30/30** acceptance checks with real kill points
(server SIGKILL + restart, node SIGKILL after dispatch → exactly one visible
`outcome_unknown`, duplicate transport → one domain effect, budget denial,
`inconclusive` + the unresolved register, the audit reconstruction through the
supported read surfaces only) — on the debug AND the release-built binaries.
The G1–G2 gate record is **Met** with five named deferrals
(`docs/decisions/2026-09-07_phase1-gate-record.md`), the subtraction record
ships (`docs/decisions/2026-09-07_phase1-subtraction-record.md`), and the
evidence manifest is `docs/evidence/2026-09-07_phase1-evidence-manifest.md`.
Execution is recorded in `docs/tasks/PHASE-1.md` (`done`).

## Phase 2 (current)

Phase 2 hardens delivery, identity, and recovery (`ROADMAP.md` §20.4) so a
later Internet slice can reuse the same control plane. The frontier is
`PHASE-2.1` — workload certificate lifecycle, scoped grants, delegated
authority context, revocation, and cached-decision rules (tracked in
`docs/tasks/PHASE-2.md`).
