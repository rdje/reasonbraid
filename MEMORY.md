# MEMORY — resume pointer (layer A; overwrite-only, keep ≤ ~50 lines)

## How to resume

- Read `MEMORY_ARCHITECTURE.md` (the memory system) and `README.md` (the project).
- Work is tracked in task-trees under `docs/tasks/`; follow `COMMIT.md`.
- Durable facts/decisions live in `docs/decisions/` (+ its `INDEX.md`).
- Transferable methods: `TOOLBOX.md` and `docs/knowledge/` — consult, do not re-derive.

## Current state (OVERWRITE this block each update — do not append)

- latest_commit: `b4d6201` — "REASONBRAID-REPAIR-0213 (leaf SIGNOFF-REPAIR.11.14.1): bind every evidence read to the tenant that cited it" (ahead of origin: 155; push at ~300)
- active_work_unit: `SIGNOFF-REPAIR` → frontier leaf: `SIGNOFF-REPAIR.11.14.2` (`pending`)
- next_action: Reproduce the claim-side assessment oracle — `GET /v1/claims/{claim_id}/assessments` is enrolment-gated and `claim_id` is caller-supplied `TEXT` the server never mints (no `claims` table exists) — observed RED before any repair.
- in_flight_uncommitted: none — working tree clean, no background job.
- blockers: **B1** external threat-model review, **B2** penetration test, **B3** prompt-injection suite, **B4** name clearance — all OUTSIDE, all `Ack? = no`; **C1** remote CI has never run (cadence only), **C2** no gate record's shipped count re-derives. Register and the standing re-surfacing obligation: `SIGNOFF-REPAIR.13` + `docs/book/src/blockers.md`.
