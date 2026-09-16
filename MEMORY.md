# MEMORY — resume pointer (layer A; overwrite-only, keep ≤ ~50 lines)

## How to resume

- Read `MEMORY_ARCHITECTURE.md` (the memory system) and `README.md` (the project).
- Work is tracked in task-trees under `docs/tasks/`; follow `COMMIT.md`.
- Durable facts/decisions live in `docs/decisions/` (+ its `INDEX.md`).
- Transferable methods: `TOOLBOX.md` and `docs/knowledge/` — consult, do not re-derive.

## Current state (OVERWRITE this block each update — do not append)

- latest_commit: `3b94c9b` — "REASONBRAID-DOC-0034 (leaf SIGNOFF-REPAIR.7.3.2.1): the retained-fixture accumulation has a second instance" (ahead of origin: 157; push at ~300)
- active_work_unit: `SIGNOFF-REPAIR` → frontier leaf: `SIGNOFF-REPAIR.11.14.3` (`pending`)
- next_action: ⛔ frontier row 1 is HELD for a director decision (the deliberation/evidence seam, censused 0-in-both-directions over nine modules). Work the next unheld row: `SIGNOFF-REPAIR.7.4.5`, extracting the site authorization skeleton `.7.4.3` deliberately duplicated.
- in_flight_uncommitted: none — working tree clean, no background job.
- blockers: **B1** external threat-model review, **B2** penetration test, **B3** prompt-injection suite, **B4** name clearance — all OUTSIDE, all `Ack? = no`; **C1** remote CI has never run (cadence only), **C2** no gate record's shipped count re-derives. Register and the standing re-surfacing obligation: `SIGNOFF-REPAIR.13` + `docs/book/src/blockers.md`.
