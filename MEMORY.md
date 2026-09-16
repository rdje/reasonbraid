# MEMORY — resume pointer (layer A; overwrite-only, keep ≤ ~50 lines)

## How to resume

- Read `MEMORY_ARCHITECTURE.md` (the memory system) and `README.md` (the project).
- Work is tracked in task-trees under `docs/tasks/`; follow `COMMIT.md`.
- Durable facts/decisions live in `docs/decisions/` (+ its `INDEX.md`).
- Transferable methods: `TOOLBOX.md` and `docs/knowledge/` — consult, do not re-derive.

## Current state (OVERWRITE this block each update — do not append)

- latest_commit: `69b6374` — "REASONBRAID-REPAIR-0217 (leaf SIGNOFF-REPAIR.11.14.3.1): the assess step records an assessment against the thread" (ahead of origin: 163; push at ~300)
- active_work_unit: `SIGNOFF-REPAIR` → frontier leaf: `SIGNOFF-REPAIR.11.14.3.3` (`pending`)
- next_action: measure the `claim_assessments` population written against invented identifiers, by command, and decide the disposition of the ROWS and of the standalone `POST /v1/assessments` writer. ⛔ "Leave it" is a legitimate answer (`.11.14.1`/`.11.14.2` both took fail-closed no-backfill) but must be published in the book, not assumed.
- in_flight_uncommitted: none — working tree clean, no background job.
- blockers: **B1** external threat-model review, **B2** penetration test, **B3** prompt-injection suite, **B4** name clearance — all OUTSIDE, all `Ack? = no`; **C1** remote CI has never run (cadence only), **C2** no gate record's shipped count re-derives. Register and the standing re-surfacing obligation: `SIGNOFF-REPAIR.13` + `docs/book/src/blockers.md`.
