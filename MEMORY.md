# MEMORY — resume pointer (layer A; overwrite-only, keep ≤ ~50 lines)

## How to resume

- Read `MEMORY_ARCHITECTURE.md` (the memory system) and `README.md` (the project).
- Work is tracked in task-trees under `docs/tasks/`; follow `COMMIT.md`.
- Durable facts/decisions live in `docs/decisions/` (+ its `INDEX.md`).
- Transferable methods: `TOOLBOX.md` and `docs/knowledge/` — consult, do not re-derive.

## Current state (OVERWRITE this block each update — do not append)

- latest_commit: `REASONBRAID-DOC-0043` (leaf `SIGNOFF-REPAIR.13.5`): the register asked a question about its reader. Re-derive the ahead-of-origin count with `git rev-list --count origin/main..HEAD`; push at ~300.
- active_work_unit: `SIGNOFF-REPAIR` → frontier leaf: `SIGNOFF-REPAIR.11.14.3.14` (`pending`) — **the director delegated this decision on 2026-09-17**; `.11.14.3.10` at row 1a.
- next_action: `.11.14.3.14` — take the §16.11 quota decision. ⭐ MEASURED: `POST /v1/resources/{id}/resolve` has **no quota, no storm control, no breaker** and performs a real network fetch per call — that is §16.11's "scraping"/"resolver abuse", and it is where `SCOPE_RESOLVER`/`SCOPE_DESTINATION` belong. ⛔ Fail-closed (`quota_unconfigured`) is right only for an ENUMERABLE scope space: the host space is open, so a fail-closed destination quota would refuse every acquisition. `usage_quotas` is keyed `(tenant_id, scope_kind, scope_id)` with free-form `scope_id`, so a `'*'` default row plus most-specific-wins is the shape.
- in_flight_uncommitted: none — working tree clean, no background job.
- standing lesson from 2026-09-17: **a claim derived by READING rather than measuring is this session's repeated failure mode** — six corrections now, the last being `.11.14.3.8`'s census, FALSE for seven commits in five documents with a live unbound write behind it, found only when the director asked whether the findings hold. ⭐ Three extensions earned: **a compatibility objection is itself a claim about the product**; **a field's name is not its contract — its consumer is**; and ⛔ **a rule earned from one instance is worth nothing until it is run over the instances that came before it** — `a-census-is-as-wide-as-its-key` was written from the second instance and never swept backwards to the first.
- blockers: the register's column is now **`Owed here?`**, set by ME from the owning leaf — `.13.5` (DOC-0043) replaced `Ack?`, which asked whether the DIRECTOR had engaged and was `no` on every open row for its whole life. ⭐ **B1, B2, B3 are `yes` (work owed HERE)** — all three share `SIGNOFF-REPAIR.14`'s frozen exposure candidate, which the blockers page already said; reporting them as "OUTSIDE" hid my own work from me. **B4 `no`** (professional name clearance — the only row with nothing owed here). **C2 `yes`** (four gate records remain). **C1 `no`** and relabelled: not a blocker, a limit on what may be CLAIMED. Rule: a `yes` row is WORK and rides the frontier; a `no` row is surfaced once per session with its trigger, not in every reply. Register: `SIGNOFF-REPAIR.13` + `docs/book/src/blockers.md`.
- ⚠️ standing corpus rule from `.13.4` (2026-09-17): **a correction is not complete until the LIVE-DOCUMENT corpus that restates it has been censused** — `.13.3` corrected C1 in the register, the tree and the blockers chapter and left it asserted in `LIVE_STATUS.md` (×3) and the book's qualification chapter. Live documents = `README.md`, `LIVE_STATUS.md`, `MEMORY.md`, `docs/book/src/*.md`; the dated ledgers are history and stay. Routed to `.11.16`.
