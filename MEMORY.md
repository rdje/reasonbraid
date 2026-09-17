# MEMORY — resume pointer (layer A; overwrite-only, keep ≤ ~50 lines)

## How to resume

- Read `MEMORY_ARCHITECTURE.md` (the memory system) and `README.md` (the project).
- Work is tracked in task-trees under `docs/tasks/`; follow `COMMIT.md`.
- Durable facts/decisions live in `docs/decisions/` (+ its `INDEX.md`).
- Transferable methods: `TOOLBOX.md` and `docs/knowledge/` — consult, do not re-derive.

## Current state (OVERWRITE this block each update — do not append)

- latest_commit: `REASONBRAID-REPAIR-0242` (leaf `SIGNOFF-REPAIR.11.20`): the instrument guarding this file had been dead since the commit that reshaped it. Re-derive the ahead-of-origin count with `git rev-list --count origin/main..HEAD`; push at ~300.
- active_work_unit: `SIGNOFF-REPAIR` → frontier leaf: `SIGNOFF-REPAIR.11.17.2` (`pending`). ✅ CLOSED this session: the `.11.14.3` family, `.11.14.1.1`, `.11.14.1.2`, `.11.18`, `.11.17`, `.11.16`, `.11.19`, `.11.19.1`, `.11.19.2`, `.11.17.1`, `.11.20`.
- next_action: `.11.17.2` — 🔴 **`POSITIONAL-REF` calls a reference `pathed` on the presence of a slash ALONE and never checks the path exists.** Of **251** `pathed` occurrences **39 do not resolve**, and 🔴 **one is a suffix of THREE tracked files** — an ambiguous reference waved through by the gate whose purpose is refusing one. ⛔ Discharge that one FIRST. ⚠️ 5 of the 39 are legitimate dependency citations; `Cargo.lock` makes that class checkable. Full classification in the leaf.
- in_flight_uncommitted: none — working tree clean, no background job.
- ⛔ **THIS FILE IS A POINTER. A lesson lives in `docs/knowledge/`; this block names it and stops.** Restating one here is the `SIGNOFF-REPAIR.11.16` defect one layer up, and it is what took the file to 6,123 of 7,168 bytes — **65% of it was six standing-lesson bullets** restating notes that were already durable (`.11.20`). Run `python3 -B scripts/census_memory_warnings.py` before evicting anything.
- ⭐ standing lessons, newest first — **read the note, not this line**: `the-commit-that-reshapes-a-file-blinds-its-guard` · `a-restated-number-needs-a-producer` · `a-file-that-no-longer-exists-is-a-conclusion` · `calibrate-over-the-history-that-contains-the-instance` · `a-key-too-loose-returns-the-wrong-instance` · `a-census-is-as-wide-as-its-key` (fired FOUR times in one session; the worst reported **1** where the answer was **51**) · `a-scoping-defect-errs-in-one-direction` · `an-injection-must-be-shown-to-land`. Methods: `TOOLBOX.md`'s *Ask the renderer, not the specification* and *A restated number needs a PRODUCER, not a rule*.
- ⚠️ standing lesson from 2026-09-17, not yet a note: **a control that passes for an UNRELATED reason.** The question that catches it: *what would this have done if the defect were present?* The mechanical form: run the NEW control against the OLD code, in situ — every arm claiming to cover the defect must FAIL by name, and any arm passing both ways must be LABELLED. ⭐ `.11.20` is its sharpest instance: a census dead for dozens of commits while its own `--self-test` reported 16 controls passing.
- blockers: register `SIGNOFF-REPAIR.13` + `docs/book/src/blockers.md`. Column is `Owed here?`, set from the owning leaf. **B1, B2, B3 `yes`** (work owed HERE — all three share `SIGNOFF-REPAIR.14`'s frozen exposure candidate). **C2 `yes`** (four gate records remain). **B4 `no`** (professional name clearance). **C1 `no`** — not a blocker, a limit on what may be CLAIMED. A `yes` row is WORK and rides the frontier; a `no` row is surfaced once per session with its trigger.
- ⚠️ standing corpus rule from `.13.4`: **a correction is not complete until the LIVE-DOCUMENT corpus that restates it has been censused.** Live documents = `README.md`, `LIVE_STATUS.md`, `MEMORY.md`, `docs/book/src/*.md`; the dated ledgers are history and stay.
