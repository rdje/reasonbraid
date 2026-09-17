# MEMORY — resume pointer (layer A; overwrite-only, keep ≤ ~50 lines)

## How to resume

- Read `MEMORY_ARCHITECTURE.md` (the memory system) and `README.md` (the project).
- Work is tracked in task-trees under `docs/tasks/`; follow `COMMIT.md`.
- Durable facts/decisions live in `docs/decisions/` (+ its `INDEX.md`).
- Transferable methods: `TOOLBOX.md` and `docs/knowledge/` — consult, do not re-derive.

## Current state (OVERWRITE this block each update — do not append)

- latest_commit: `REASONBRAID-REPAIR-0233` (leaf `SIGNOFF-REPAIR.11.14.1.2`): the plan checker now has a commit-time trigger. Re-derive the ahead-of-origin count with `git rev-list --count origin/main..HEAD`; push at ~300.
- active_work_unit: `SIGNOFF-REPAIR` → frontier leaf: `SIGNOFF-REPAIR.11.14.3.10` (`pending`). The `.11.14.3` evidence-chain family is otherwise CLOSED.
- next_action: `.11.14.3.10` — `credential_binding_ref` is a caller-supplied field that SELECTS a credential; the broker's store carries no tenant at all and the pair key makes the field shared. Reproduce with `RB_ENABLE_R5R3RX=1` and a registered binding, RED first. ⚠️ LATENT, not live: no production path registers a binding. ⛔ Do NOT assume a tenant column on the broker — its deployment integration is the OS keychain, out of the dev profile's scope, so a dev-store change may not be the durable one.
- in_flight_uncommitted: none — working tree clean, no background job.
- standing lesson from 2026-09-17: ⛔ **KNOWING A RULE IS NOT APPLYING IT** — I promoted `a-census-is-as-wide-as-its-key`, then failed to sweep it backwards (a FALSE census hiding a live write) and repeated it in `migrations/0067`. Now mechanical: `FIXTURE-PLAN-CHILDREN`. ⭐ The family, all met today: a **scope** that silently skips what it cannot read; an **instrument** that reports the same value either way; a **stimulus** that never landed. All three pass for a reason unrelated to what they test, and all three are caught by one question — *what would this have done if the defect were present?* Also earned: a compatibility objection is a claim about the product; a field's name is not its contract, its consumer is.
- blockers: the register's column is now **`Owed here?`**, set by ME from the owning leaf — `.13.5` (DOC-0043) replaced `Ack?`, which asked whether the DIRECTOR had engaged and was `no` on every open row for its whole life. ⭐ **B1, B2, B3 are `yes` (work owed HERE)** — all three share `SIGNOFF-REPAIR.14`'s frozen exposure candidate, which the blockers page already said; reporting them as "OUTSIDE" hid my own work from me. **B4 `no`** (professional name clearance — the only row with nothing owed here). **C2 `yes`** (four gate records remain). **C1 `no`** and relabelled: not a blocker, a limit on what may be CLAIMED. Rule: a `yes` row is WORK and rides the frontier; a `no` row is surfaced once per session with its trigger, not in every reply. Register: `SIGNOFF-REPAIR.13` + `docs/book/src/blockers.md`.
- ⚠️ standing corpus rule from `.13.4` (2026-09-17): **a correction is not complete until the LIVE-DOCUMENT corpus that restates it has been censused** — `.13.3` corrected C1 in the register, the tree and the blockers chapter and left it asserted in `LIVE_STATUS.md` (×3) and the book's qualification chapter. Live documents = `README.md`, `LIVE_STATUS.md`, `MEMORY.md`, `docs/book/src/*.md`; the dated ledgers are history and stay. Routed to `.11.16`.
