# PROGRAM: ReasonBraid v0.4.1 programme map

## Metadata

- Tree ID: `PROGRAM`
- Status: `active`
- Roadmap lane: whole programme (index, not an implementation lane)
- Created: `2026-09-05`
- Owner: repo-local workflow
- Sources: `ROADMAP.md` v0.4.1, companion `KICKOFF.md`

## Goal

Keep a complete, session-survivable map from the frozen roadmap to task-trees:
every phase, work package, backlog item, gate, ADR, demonstration, and
post-LAN track has a named owner. This tree does not implement product code.

## Non-Goals

- Executing product work (owned by `PHASE-*` and `SIGNOFF-REPAIR`).
- Un-freezing v0.4.1 or writing v0.5.0.
- Hosting day-to-day Phase 0 issues (that is `KICKOFF.md` + `PHASE-0`).

## Companion rule

| File | Authority |
| --- | --- |
| `ROADMAP.md` | architecture, scope, gates, tracks, non-goals |
| `KICKOFF.md` | Phase 0 day-to-day execution |
| `docs/tasks/PHASE-*.md` | owned implementation |
| `docs/parking-lot.md` | non-blocking ideas (created in `PHASE-0.0.3`) |

## Phase trees

| Phase | Tree | Status | Estimate (eng-wk) | Depends on | Exit |
| --- | --- | --- | --- | --- | --- |
| 0 | `PHASE-0` | `done` | 8–14 | `RB-SEED` done | G0 for identity/authority/thread/delivery/budget |
| 1 | `PHASE-1` | `done` | 14–22 | Phase 0 contracts | G1–G2; Demonstration A |
| 2 | `PHASE-2` | `done` | 12–20 | Phase 1 | authority non-escalation (the adversarial suite); restore + node replacement (the exercises); no silent unknown-retry (the six-leg inventory) — CLOSED 2026-09-07 |
| 3 | `PHASE-3` | `done` | 12–19 | stable identity, inbox, grants | recruit without enumerating the network — CLOSED 2026-09-07 |
| 4 | `PHASE-4` | `done` | 16–27 | authz, budgets, object store, observability | G4 |
| 5 | `PHASE-5` | `done` | 15–26 + evaluators | evidence provenance, workflows | G5 on declared domains |
| 6 | `PHASE-6` | `done` | 18–30 + governance review | authority, deliberation, Git/object, correction | G3; reconstructable policy lifecycle |
| 7 | `PHASE-7` | `done` | 18–32 + external review | Phase 1 + applicable Phase 2; not Phases 3–6 if disabled | G6–G7 for a named profile |
| 8 | `PHASE-8` | `active` | 16–30 | stable trust and compatibility | G8 |
| 9 | `PHASE-9` | `proposed` | 12–24 after beta | sustained operational evidence | G9 |

The phase states above record historical execution closure. Current qualification
is under `SIGNOFF-REPAIR`; see `LIVE_STATUS.md`. Source findings reopen affected
guarantees without erasing historical evidence. The repair tree owns the full-read
census and is the current executable prerequisite for `PHASE-8.5.3`.

After Phase 1, phase numbers are work packages on parallel tracks, not a
mandatory single-file sequence (`ROADMAP.md` §20.1.1).

## Post-LAN tracks

| Track | Phases | Capability release it unblocks |
| --- | --- | --- |
| Trust | 2 + scoped 7 | Direct remote conversation preview |
| Network | 3 | Remote network discovery preview |
| Evidence | 4 | Evidence-enabled deliberation preview |
| Quality and governance | 5–6 | Governance alpha |

## Gates

| Gate | Blocks | Owning tree (exit leaf) |
| --- | --- | --- |
| G0 Contract | implementation of affected boundary | `PHASE-0.8.1` |
| G1 Component | merge/release artifact | `PHASE-1` exit |
| G2 Vertical slice | LAN preview | `PHASE-1` exit |
| G3 Governance | binding policy use | `PHASE-6` exit |
| G4 Resource safety | arbitrary-reference feature | `PHASE-4` exit |
| G5 Quality | “deliberation improves answers” claim | `PHASE-5` exit |
| G6 Internet security | Internet exposure | `PHASE-7` exit |
| G7 Operations | production beta | `PHASE-7` / `PHASE-6` publication portion |
| G8 Compatibility | stable public protocol | `PHASE-8` exit |
| G9 Release | declared product maturity | `PHASE-9` exit |

Every gate produces a `SubtractionRecord` (`ROADMAP.md` §19.8).

## Backlog → tree map (`ROADMAP.md` §22)

| Items | Theme | Tree |
| --- | --- | --- |
| 1–8 | Foundation and contracts | `PHASE-0` (1–7, 8) and `PHASE-0` WP0/WP1 |
| 9–18 | Durable control plane and node | `PHASE-0` WP2–WP3; remainder `PHASE-1` |
| 19–25 | Harnesses, attempts, economics | `PHASE-0` WP4–WP5; remainder `PHASE-1`/`PHASE-2` |
| 26–30 | Discovery and autonomy | `PHASE-3` |
| 31–35 | Resources and evidence | `PHASE-4` |
| 36–40 | Deliberation, policy, qualification | `PHASE-5`, `PHASE-6`, `PHASE-7`/`PHASE-8` |

Exact leaf IDs live in the phase trees. Do not execute a backlog item from this
index; open the owning phase tree.

## ADR queue → tree map (`ROADMAP.md` §23)

| ADR | Decision | First owning tree |
| --- | --- | --- |
| 001 | Public name and namespace clearance | `PHASE-0.0.1` |
| 002 | Modular monolith boundary | `PHASE-0` / `PHASE-1` |
| 003 | Async/concurrency / actor framework | `PHASE-0` (experiment, not preference) |
| 004 | PostgreSQL aggregate/event/outbox | `PHASE-0.2` |
| 005 | Event transport (PG queue vs NATS) | `PHASE-0` / `PHASE-2` |
| 006 | Node transport and reconnect | `PHASE-0.3` |
| 007 | Workload identity issuer | `PHASE-2` |
| 008 | Authorization language/engine | `PHASE-0.5` then `PHASE-2` |
| 009 | Delegated authority tokens | `PHASE-2` |
| 010 | ID, time, causal ordering | `PHASE-0.1` |
| 011 | Object store and content-addressing | `PHASE-1` / `PHASE-4` |
| 012 | Provider-attempt ambiguity | `PHASE-0.3` / `PHASE-0.4` |
| 013 | Budget units and settlement | `PHASE-0.5` / `PHASE-2` |
| 014 | Directory semantic-index engine | `PHASE-3` |
| 015 | Dependence indicators / recruitment | `PHASE-3` |
| 016 | Workflow definition representation | `PHASE-5` |
| 017 | Evaluator hierarchy | `PHASE-0.7` / `PHASE-5` |
| 018 | Resolver sandbox/runtime | `PHASE-4` |
| 019 | Canonical policy schema | `PHASE-6` |
| 020 | Git publication refs/signatures | `PHASE-6` |
| 021 | Target deployment authority | `PHASE-6` |
| 022 | Audit hash-chain/checkpoint | `PHASE-2` / `PHASE-7` |
| 023 | Telemetry storage/redaction | `PHASE-2` |
| 024 | MCP interoperability profile | `PHASE-0` spike; `PHASE-8` |
| 025 | A2A interoperability profile | `PHASE-0` spike; `PHASE-8` |
| 026 | Federation trust agreement | `PHASE-8` |
| 027 | Plugin/adapter signing | `PHASE-7` / `PHASE-8` |
| 028 | Product versioning and support window | `PHASE-9` |

No ADR is approved merely because the roadmap names a candidate technology.

## Demonstrations

| Demo | Roadmap | Owning tree |
| --- | --- | --- |
| A — trustworthy LAN conversation | §26.1 | `PHASE-1` (prep in `PHASE-0.6`) |
| B — governed doctrine change | §26.2 | `PHASE-6` (needs Phases 4–6) |

## Task Tree

- ID: `PROGRAM`
  Status: `active`
  Goal: keep the v0.4.1 map complete
  Children: `PHASE-0` … `PHASE-9`, `SIGNOFF-REPAIR` (separate files)

- ID: `PROGRAM.1`
  Status: `done`
  Goal: create this index and the `PHASE-*` trees from `ROADMAP.md` + `KICKOFF.md`
  Acceptance: every phase, gate, backlog item, ADR, and demo maps to a named leaf
  Verification: census in `RB-SEED.2`
  Commit: `RB-SEED.2`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| — | (none in this file) | — | next executable work is `SIGNOFF-REPAIR.2.1`; return to `PHASE-8.5.3` after corrective prerequisites |

## Decisions

- `2026-09-05`: this index is derived from v0.4.1; it is not a third roadmap.
- `2026-09-05`: later phases stay `proposed` until their predecessor exit gate
  is met or a track is explicitly opened.

## Open Questions

- None that block converting the map. Naming clearance is `PHASE-0.0.1`.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-09-05` | `PROGRAM.1` | created with `RB-SEED.2` | pending census |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `PROGRAM.1` | `RB-SEED.2` | programme conversion |

## Changelog

- `2026-09-05`: Created as the v0.4.1 programme map.
- `2026-09-06`: Phase 1 row → `active`; frontier pointer refreshed (stale `proposed`/PHASE-0 wording retired after the `.1` decomposition).
