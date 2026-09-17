# MEMORY — resume pointer (layer A; overwrite-only, keep ≤ ~50 lines)

## How to resume

- Read `MEMORY_ARCHITECTURE.md` (the memory system) and `README.md` (the project).
- Work is tracked in task-trees under `docs/tasks/`; follow `COMMIT.md`.
- Durable facts/decisions live in `docs/decisions/` (+ its `INDEX.md`).
- Transferable methods: `TOOLBOX.md` and `docs/knowledge/` — consult, do not re-derive.

## Current state (OVERWRITE this block each update — do not append)

- latest_commit: `REASONBRAID-REPAIR-0228` (leaf `SIGNOFF-REPAIR.11.14.3.5`): a reference's declared fields are checked or defaulted. Re-derive the ahead-of-origin count with `git rev-list --count origin/main..HEAD`; push at ~300.
- active_work_unit: `SIGNOFF-REPAIR` → frontier leaf: `SIGNOFF-REPAIR.11.14.3.7` (`pending`); `.11.14.3.10` is the last other leaf open in this family.
- next_action: `.11.14.3.7` — `thread.contribute` carries no quota (`quota::check_in_tx`'s one call is `OP_INVITE`'s) and no length bound, so `.11.14.3.2`'s citation registration turned unbounded input into O(n) statements inside the thread's `FOR UPDATE` transaction. ⛔ Do NOT invent a cap: §16.11's machinery exists and reaches one verb, and WHICH verbs and WHICH dimension is the decision. Measure the before→after transaction duration rather than asserting the amplification.
- in_flight_uncommitted: none — working tree clean, no background job.
- standing lesson from 2026-09-17: **a claim derived by READING rather than measuring is this session's repeated failure mode** — five corrections now, the last being `.11.14.3.5`, where I read a test helper's comment as a missing check, wrote the repair, reproduced RED, went GREEN, and had it **refused by two of the suite's own controls**. ⭐ The two extensions earned: **a compatibility objection is itself a claim about the product** (enumerate what the "broken" caller can still do), and **a field's name is not its contract — its consumer is** (find every reader and say what it does with the value). ⛔ And run the AFFECTED suites broadly, not only the leaf's own control: the narrow run was green.
- blockers: **B1** external threat-model review (a threat model EXISTS at `spec/threat-model.md`; the leaf owes its fitness-for-review), **B2** penetration test (scope + environment owed), **B3** correctly deferred and gate-enforced by `ACTION-BOUNDARY` — ⭐ its stated ground is now sound after `.13.1.1`, **B4** name clearance — all four OUTSIDE, all `Ack? = no`; **C1** ✅ **CORRECTED 2026-09-17 (REPAIR-0220): the claim "remote CI has never run" was FALSE** — 41 runs, 29/12, all three workflows green at `c17841c` which IS `origin/main`; the real limit is the unpushed gap, inside cadence; **C2** four gate records remain to re-derive. Register: `SIGNOFF-REPAIR.13` + `docs/book/src/blockers.md`.
- ⚠️ standing corpus rule from `.13.4` (2026-09-17): **a correction is not complete until the LIVE-DOCUMENT corpus that restates it has been censused** — `.13.3` corrected C1 in the register, the tree and the blockers chapter and left it asserted in `LIVE_STATUS.md` (×3) and the book's qualification chapter. Live documents = `README.md`, `LIVE_STATUS.md`, `MEMORY.md`, `docs/book/src/*.md`; the dated ledgers are history and stay. Routed to `.11.16`.
