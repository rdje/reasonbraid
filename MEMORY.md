# MEMORY — resume pointer (layer A; overwrite-only, keep ≤ ~50 lines)

## How to resume

- Read `MEMORY_ARCHITECTURE.md` (the memory system) and `README.md` (the project).
- Work is tracked in task-trees under `docs/tasks/`; follow `COMMIT.md`.
- Durable facts/decisions live in `docs/decisions/` (+ its `INDEX.md`).
- Transferable methods: `TOOLBOX.md` and `docs/knowledge/` — consult, do not re-derive.

## Current state (OVERWRITE this block each update — do not append)

- latest_commit: `84fa043` — "REASONBRAID-DOC-0036 (leaf SIGNOFF-REPAIR.11.2.5): the census gate is discharged by any command in the section" (ahead of origin: 161; push at ~300)
- active_work_unit: `SIGNOFF-REPAIR` → frontier leaf: `SIGNOFF-REPAIR.11.14.3.1` (`pending`)
- next_action: Drive the shipped `evidence_review` profile to its `assess` step and observe that NO assessment can be attached to the thread — `git grep -n '"assess"' -- crates/reasonbraid-server/src` returns 1 hit, the vocabulary constant. RED before any repair.
- in_flight_uncommitted: none — working tree clean, no background job.
- blockers: **B1** external threat-model review, **B2** penetration test, **B3** prompt-injection suite, **B4** name clearance — all OUTSIDE, all `Ack? = no`; **C1** remote CI has never run (cadence only), **C2** no gate record's shipped count re-derives. Register and the standing re-surfacing obligation: `SIGNOFF-REPAIR.13` + `docs/book/src/blockers.md`.
