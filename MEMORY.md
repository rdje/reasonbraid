# MEMORY — resume pointer (layer A; overwrite-only, keep ≤ ~50 lines)

## How to resume

- Read `MEMORY_ARCHITECTURE.md` (the memory system) and `README.md` (the project).
- Work is tracked in task-trees under `docs/tasks/`; follow `COMMIT.md`.
- Durable facts/decisions live in `docs/decisions/` (+ its `INDEX.md`).
- Transferable methods: `TOOLBOX.md` and `docs/knowledge/` — consult, do not re-derive.

## Current state (OVERWRITE this block each update — do not append)

- latest_commit: `afc4c9e` — "REASONBRAID-REPAIR-0221 (leaf SIGNOFF-REPAIR.13.1.1): a re-export is not a caller". Re-derive the ahead-of-origin count with `git rev-list --count origin/main..HEAD`; push at ~300.
- active_work_unit: `SIGNOFF-REPAIR` → frontier leaf: `SIGNOFF-REPAIR.11.14.3.8` (`pending`) — unchanged; the `.13.x` blocker work below was director-directed and is not frontier row 1.
- next_action: reproduce the `POST /v1/assessments` citation-gate oracle by command — a second tenant distinguishing `SnapshotMissing` from `ExcerptAbsent` over a snapshot it never cited — then decide: the citation gate, a uniform refusal, or neither. ⛔ The obvious repair (`is_cited_by` on the route) is a COMPATIBILITY break on a shipped route and must be decided, not assumed. ⭐ `.11.14.3.3`'s cross-tenant control arm already reaches the excerpt check through this gap, so the reproduction is half-written.
- in_flight_uncommitted: none — working tree clean, no background job.
- standing lesson from 2026-09-17: **a claim derived by READING rather than measuring is this session's repeated failure mode**, and it produced three separate corrections — a published width ("bounded to one tenant"), a durability census whose exclusion removed the answer, and blocker C1, which was quoted from `COMMIT.md` while that page contradicted it four lines below. Measure the source, not the project's own prose.
- blockers: **B1** external threat-model review (a threat model EXISTS at `spec/threat-model.md`; the leaf owes its fitness-for-review), **B2** penetration test (scope + environment owed), **B3** correctly deferred and gate-enforced by `ACTION-BOUNDARY` — ⭐ its stated ground is now sound after `.13.1.1`, **B4** name clearance — all four OUTSIDE, all `Ack? = no`; **C1** ✅ **CORRECTED 2026-09-17 (REPAIR-0220): the claim "remote CI has never run" was FALSE** — 41 runs, 29/12, all three workflows green at `c17841c` which IS `origin/main`; the real limit is the unpushed gap, inside cadence; **C2** four gate records remain to re-derive. Register: `SIGNOFF-REPAIR.13` + `docs/book/src/blockers.md`.
