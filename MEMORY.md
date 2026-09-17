# MEMORY — resume pointer (layer A; overwrite-only, keep ≤ ~50 lines)

## How to resume

- Read `MEMORY_ARCHITECTURE.md` (the memory system) and `README.md` (the project).
- Work is tracked in task-trees under `docs/tasks/`; follow `COMMIT.md`.
- Durable facts/decisions live in `docs/decisions/` (+ its `INDEX.md`).
- Transferable methods: `TOOLBOX.md` and `docs/knowledge/` — consult, do not re-derive.

## Current state (OVERWRITE this block each update — do not append)

- latest_commit: `REASONBRAID-DOC-0042` (leaf `SIGNOFF-REPAIR.13.4`): a corrected claim was corrected in the register and left standing in the corpus. Re-derive the ahead-of-origin count with `git rev-list --count origin/main..HEAD`; push at ~300.
- active_work_unit: `SIGNOFF-REPAIR` → frontier leaf: `SIGNOFF-REPAIR.11.14.3.4` (`pending`) — promoted after `.11.14.3.8` closed; the `.13.x` blocker work below was director-directed and is not frontier row 1.
- next_action: `.11.14.3.4` — measure the `resource_references` read oracle by command (a second tenant confirming a first's registration through `GET /v1/resources/{id}` and `POST /v1/resources`'s `replayed: true`), then decide whether the read is bound. ⛔ Do NOT assume a citation table: a locator a contributor CHOSE to cite is weaker than an acquisition receipt, and a per-tenant set breaks the dedupe the pair key exists for. ⚠️ `.11.14.3.8` ruled the snapshot case the other way (REPAIR-0222) — this leaf must NAME the difference or that ruling wins (`docs/CLAIM_VERIFICATION.md` §3 leg 2).
- in_flight_uncommitted: none — working tree clean, no background job.
- standing lesson from 2026-09-17: **a claim derived by READING rather than measuring is this session's repeated failure mode** — four corrections now (a published width, a durability census whose exclusion removed the answer, blocker C1 quoted from a page that contradicted it, and `.11.14.3.8`'s compatibility objection, which named a workflow that could not function). ⭐ Extension earned by the fourth: **a compatibility objection is itself a claim about the product — enumerate what the "broken" caller can currently do OTHER than the thing being removed.**
- blockers: **B1** external threat-model review (a threat model EXISTS at `spec/threat-model.md`; the leaf owes its fitness-for-review), **B2** penetration test (scope + environment owed), **B3** correctly deferred and gate-enforced by `ACTION-BOUNDARY` — ⭐ its stated ground is now sound after `.13.1.1`, **B4** name clearance — all four OUTSIDE, all `Ack? = no`; **C1** ✅ **CORRECTED 2026-09-17 (REPAIR-0220): the claim "remote CI has never run" was FALSE** — 41 runs, 29/12, all three workflows green at `c17841c` which IS `origin/main`; the real limit is the unpushed gap, inside cadence; **C2** four gate records remain to re-derive. Register: `SIGNOFF-REPAIR.13` + `docs/book/src/blockers.md`.
- ⚠️ standing corpus rule from `.13.4` (2026-09-17): **a correction is not complete until the LIVE-DOCUMENT corpus that restates it has been censused** — `.13.3` corrected C1 in the register, the tree and the blockers chapter and left it asserted in `LIVE_STATUS.md` (×3) and the book's qualification chapter. Live documents = `README.md`, `LIVE_STATUS.md`, `MEMORY.md`, `docs/book/src/*.md`; the dated ledgers are history and stay. Routed to `.11.16`.
