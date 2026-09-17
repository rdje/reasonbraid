# MEMORY — resume pointer (layer A; overwrite-only, keep ≤ ~50 lines)

## How to resume

- Read `MEMORY_ARCHITECTURE.md` (the memory system) and `README.md` (the project).
- Work is tracked in task-trees under `docs/tasks/`; follow `COMMIT.md`.
- Durable facts/decisions live in `docs/decisions/` (+ its `INDEX.md`).
- Transferable methods: `TOOLBOX.md` and `docs/knowledge/` — consult, do not re-derive.

## Current state (OVERWRITE this block each update — do not append)

- latest_commit: `REPAIR-0219` — "REASONBRAID-REPAIR-0219 (leaf SIGNOFF-REPAIR.11.14.3.3): a replay key is an addressing scheme" (ahead of origin: **166**, measured by `git rev-list --count origin/main..HEAD` rather than incremented; push at ~300). Preceded by `665c8f4` DOC-0038, the twentieth changelog rotation.
- active_work_unit: `SIGNOFF-REPAIR` → frontier leaf: `SIGNOFF-REPAIR.11.14.3.8` (`pending`)
- next_action: reproduce the `POST /v1/assessments` citation-gate oracle by command — a second tenant distinguishing `SnapshotMissing` from `ExcerptAbsent` over a snapshot it never cited — then decide: the citation gate, a uniform refusal, or neither. ⛔ The obvious repair (`is_cited_by` on the route) is a COMPATIBILITY break on a shipped route and must be decided, not assumed. ⭐ `.11.14.3.3`'s cross-tenant control arm already reaches the excerpt check through this gap, so the reproduction is half-written.
- in_flight_uncommitted: none — working tree clean, no background job.
- standing lesson from `.11.14.3.3`: **a published width derived by READING rather than measuring is the failure mode to watch here.** "Bounded to one tenant" was published from the SQL and was wrong; measuring it found a second, wider defect and a false premise in `migrations/0064`. Write one control arm per caller-set column.
- blockers: **B1** external threat-model review, **B2** penetration test, **B3** prompt-injection suite, **B4** name clearance — all OUTSIDE, all `Ack? = no`; **C1** ✅ CORRECTED 2026-09-17 — the remote gate is GREEN at `origin/main` (41 runs, 29/12; all three workflows green at `c17841c`); the real limit is 167 commits beyond it, inside the ~300 cadence, **C2** no gate record's shipped count re-derives. Register and the standing re-surfacing obligation: `SIGNOFF-REPAIR.13` + `docs/book/src/blockers.md`.
