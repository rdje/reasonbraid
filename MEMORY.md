# MEMORY — resume pointer (layer A; overwrite-only, keep ≤ ~50 lines)

## How to resume

- Read `MEMORY_ARCHITECTURE.md` (the memory system) and `README.md` (the project).
- Work is tracked in task-trees under `docs/tasks/`; follow `COMMIT.md`.
- Durable facts/decisions live in `docs/decisions/` (+ its `INDEX.md`).
- Transferable methods: `TOOLBOX.md` and `docs/knowledge/` — consult, do not re-derive.

## Current state (OVERWRITE this block each update — do not append)

- latest_commit: `REASONBRAID-REPAIR-0229` (leaf `SIGNOFF-REPAIR.11.14.3.7`): what a citation list costs. Re-derive the ahead-of-origin count with `git rev-list --count origin/main..HEAD`; push at ~300.
- active_work_unit: `SIGNOFF-REPAIR` → frontier leaf: `SIGNOFF-REPAIR.11.14.3.10` (`pending`), with `.11.14.3.14` at row 1a. The `.11.14.3` evidence-chain family is otherwise CLOSED.
- next_action: `.11.14.3.10` — `credential_binding_ref` is a caller-supplied field that SELECTS a credential and the broker's store carries no tenant at all. ⚠️ Reproduce it with the gate ON (`RB_ENABLE_R5R3RX`) and a registered binding, RED first — it is LATENT, not live, because no production path registers one. ⛔ Do NOT assume a tenant column on the broker: its deployment integration is named as the OS keychain, out of the dev profile's scope, so a dev-store change may not be the durable one.
- in_flight_uncommitted: none — working tree clean, no background job.
- standing lesson from 2026-09-17: **a claim derived by READING rather than measuring is this session's repeated failure mode** — five corrections now, the last being `.11.14.3.5`, where I read a test helper's comment as a missing check, wrote the repair, reproduced RED, went GREEN, and had it **refused by two of the suite's own controls**. ⭐ The two extensions earned: **a compatibility objection is itself a claim about the product** (enumerate what the "broken" caller can still do), and **a field's name is not its contract — its consumer is** (find every reader and say what it does with the value). ⛔ And run the AFFECTED suites broadly, not only the leaf's own control: the narrow run was green.
- blockers: **B1** external threat-model review (a threat model EXISTS at `spec/threat-model.md`; the leaf owes its fitness-for-review), **B2** penetration test (scope + environment owed), **B3** correctly deferred and gate-enforced by `ACTION-BOUNDARY` — ⭐ its stated ground is now sound after `.13.1.1`, **B4** name clearance — all four OUTSIDE, all `Ack? = no`; **C1** ✅ **CORRECTED 2026-09-17 (REPAIR-0220): the claim "remote CI has never run" was FALSE** — 41 runs, 29/12, all three workflows green at `c17841c` which IS `origin/main`; the real limit is the unpushed gap, inside cadence; **C2** four gate records remain to re-derive. Register: `SIGNOFF-REPAIR.13` + `docs/book/src/blockers.md`.
- ⚠️ standing corpus rule from `.13.4` (2026-09-17): **a correction is not complete until the LIVE-DOCUMENT corpus that restates it has been censused** — `.13.3` corrected C1 in the register, the tree and the blockers chapter and left it asserted in `LIVE_STATUS.md` (×3) and the book's qualification chapter. Live documents = `README.md`, `LIVE_STATUS.md`, `MEMORY.md`, `docs/book/src/*.md`; the dated ledgers are history and stay. Routed to `.11.16`.
