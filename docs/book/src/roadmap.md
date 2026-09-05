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
evidence package and a working Phase 1 LAN vertical slice exist. New features
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

## Phase 0 (now)

Phase 0 does not implement the product. It must answer six kill-risk questions
with code, measurements, failures, and decisions that constrain Phase 1. The
work packages are WP0–WP8 in `KICKOFF.md`. Execution is tracked in `docs/tasks/PHASE-0.md`.
The programme map is `docs/tasks/PROGRAM.md`.

The first credible product milestone after Phase 0 is Demonstration A: a
trustworthy LAN conversation between two nodes, with crash/reconnect, duplicate
delivery, and an honest `outcome_unknown` provider attempt.
