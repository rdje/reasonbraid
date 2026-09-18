# MEMORY — resume pointer (layer A; overwrite-only, keep ≤ ~50 lines)

## How to resume

- Read `MEMORY_ARCHITECTURE.md` (the memory system) and `README.md` (the project).
- Work is tracked in task-trees under `docs/tasks/`; follow `COMMIT.md`.
- Durable facts/decisions live in `docs/decisions/` (+ its `INDEX.md`).
- Transferable methods: `TOOLBOX.md` and `docs/knowledge/` — consult, do not re-derive.

## Current state (OVERWRITE this block each update — do not append)
- latest_commit: `REASONBRAID-REPAIR-0243` (leaf `SIGNOFF-REPAIR.11.17.2`): a slash is not a resolution — `POSITIONAL-REF` read resolvability off the SHAPE of a string. Re-derive the ahead-of-origin count with `git rev-list --count origin/main..HEAD`; push at ~300.
- active_work_unit: `SIGNOFF-REPAIR` → frontier leaf: `SIGNOFF-REPAIR.11.20.1` (`pending`). ✅ CLOSED this session: `.11.17.2`.
- next_action: `.11.20.1` — 🔴 **the memory census reports 3 standing warnings while 3,983 bytes, 65% of this file, sit in bullets it cannot see.** `.11.20` fixed WHERE it looks; nothing fixed WHAT it counts. ⛔ Check the new shape against `MEMORY_ARCHITECTURE.md` §6's template, not against this file's current contents, or the instrument gets fitted to one snapshot. Full statement in the leaf.
- in_flight_uncommitted: none — working tree clean, no background job owned by this repo. ⚠️ A `cargo` build belonging to `github/pgen` may be running; check `pgrep -fl cargo` and read the cwd before blaming this tree for a build lock.
- ⛔ **THIS FILE IS A POINTER. A lesson lives in `docs/knowledge/`; this block names it and stops.** Restating one here is the `SIGNOFF-REPAIR.11.16` defect one layer up, and it is what took the file to 6,123 of 7,168 bytes (`.11.20`). Run `python3 -B scripts/census_memory_warnings.py` before evicting anything — and note it can only see the next-action bullet until `.11.20.1` lands.
- ⭐ standing lessons, newest first — **read the note, not this line**: `trust-comes-from-the-check-not-the-shape` (now carries the classifier instance too) · `the-commit-that-reshapes-a-file-blinds-its-guard` · `a-restated-number-needs-a-producer` · `a-file-that-no-longer-exists-is-a-conclusion` · `calibrate-over-the-history-that-contains-the-instance` · `a-key-too-loose-returns-the-wrong-instance` · `a-census-is-as-wide-as-its-key` · `a-scoping-defect-errs-in-one-direction` · `an-injection-must-be-shown-to-land`. Methods: `TOOLBOX.md`'s *Ask the renderer, not the specification*, *A restated number needs a PRODUCER*, and *Give every census a LIVE-CORPUS arm*.
- ⚠️ standing lesson, not yet a note: **a control that passes for an UNRELATED reason.** The question that catches it: *what would this have done if the defect were present?* The mechanical form: run the NEW control against the OLD code, in situ — every arm claiming to cover the defect must FAIL by name, and any arm passing both ways must be LABELLED in the source. ⭐ `.11.17.2` is the latest application: 11 of 12 new arms went red by name, 2 were labelled.
- blockers: register `SIGNOFF-REPAIR.13` + `docs/book/src/blockers.md`. Column is `Owed here?`, set from the owning leaf. **B1, B2, B3 `yes`** (work owed HERE — all three share `SIGNOFF-REPAIR.14`'s frozen exposure candidate). **C2 `yes`** (four gate records remain). **B4 `no`** (professional name clearance). **C1 `no`** — not a blocker, a limit on what may be CLAIMED. A `yes` row is WORK and rides the frontier; a `no` row is surfaced once per session with its trigger.
- ⚠️ standing corpus rule from `.13.4`: **a correction is not complete until the LIVE-DOCUMENT corpus that restates it has been censused.** Live documents = `README.md`, `LIVE_STATUS.md`, `MEMORY.md`, `docs/book/src/*.md`; the dated ledgers are history and stay.
