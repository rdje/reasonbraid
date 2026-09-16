# MEMORY — resume pointer (layer A; overwrite-only, keep ≤ ~50 lines)

## How to resume

- Read `MEMORY_ARCHITECTURE.md` (the memory system) and `README.md` (the project).
- Work is tracked in task-trees under `docs/tasks/`; follow `COMMIT.md`.
- Durable facts/decisions live in `docs/decisions/` (+ its `INDEX.md`).
- Transferable methods: `TOOLBOX.md` and `docs/knowledge/` — consult, do not re-derive.

## Current state (OVERWRITE this block each update — do not append)

- latest_commit: `ba6e77e` — "REASONBRAID-DOC-0037 (leaf SIGNOFF-REPAIR.11.14.3): the deliberation flow owns the evidence chain" (ahead of origin: 162; push at ~300)
- active_work_unit: `SIGNOFF-REPAIR` → frontier leaf: `SIGNOFF-REPAIR.11.14.3.2` (`pending`)
- next_action: §13.2 step 2 — observe a contribution accepted with an `EvidenceRef` matching no `resource_references` row, RED first. ⛔ The disposition is NOT assumed to be refusal: §13.2 says *register* at step 2 and acquires at step 6.
- in_flight_uncommitted: none — working tree clean, no background job.
- blockers: **B1** external threat-model review, **B2** penetration test, **B3** prompt-injection suite, **B4** name clearance — all OUTSIDE, all `Ack? = no`; **C1** remote CI has never run (cadence only), **C2** no gate record's shipped count re-derives. Register and the standing re-surfacing obligation: `SIGNOFF-REPAIR.13` + `docs/book/src/blockers.md`.
