# MEMORY — resume pointer (layer A; overwrite-only, keep ≤ ~50 lines)

## How to resume

- Read `MEMORY_ARCHITECTURE.md` (the memory system) and `README.md` (the project).
- Work is tracked in task-trees under `docs/tasks/`; follow `COMMIT.md`.
- Durable facts/decisions live in `docs/decisions/` (+ its `INDEX.md`).
- Transferable methods: `TOOLBOX.md` and `docs/knowledge/` — consult, do not re-derive.

## Current state (OVERWRITE this block each update — do not append)
- latest_commit: `REASONBRAID-REPAIR-0250` (leaf `SIGNOFF-REPAIR.7.2.10`): a second refusal vocabulary, 41 strings, documented nowhere. Re-derive the ahead-of-origin count with `git rev-list --count origin/main..HEAD`; push at ~300.
- active_work_unit: `SIGNOFF-REPAIR` → frontier leaf: `SIGNOFF-REPAIR.7.2.9` (`pending`). ✅ CLOSED this session: `.11.17.2`, `.11.20.1`, `.11.20.2`, `.11.18.1`, `.11.18.2`, `.11.2.5`, `.3.4.3.1.1.1`, `.7.2.7`, `.7.2.10` (+ DOC-0046, the 22nd changelog rotation).
- next_action: `.7.2.9` — 🔴 **a pack can expand far beyond its wire size and only the clock stops it.** `gix-pack` has no ratio guard but allocates with `try_reserve`, so one absurd entry errors; the residual is aggregate expansion under the allocator's limit. ⚠️ Unreproduced — build the crafted pack or withdraw the concern. Full statement in the leaf.
- in_flight_uncommitted: none — working tree clean, no background job owned by this repo (`bash scripts/check_no_background_jobs.sh`). ⚠️ A foreign `cargo` is not this tree's: `docs/decisions/2026-09-18_a-cargo-process-is-not-evidence-of-this-repo.md` (`.11.20.1`).
- ⛔ **THIS FILE IS A POINTER. A lesson lives in `docs/knowledge/`; this block names it and stops.** Restating one here is the `SIGNOFF-REPAIR.11.16` defect one layer up, and it is what took the file to 6,123 of 7,168 bytes (`.11.20`). Run `python3 -B scripts/census_memory_warnings.py` before evicting anything — and note it can only see the next-action bullet until `.11.20.1` lands.
- ⭐ standing lessons, newest first — **read the note, not this line**: `trust-comes-from-the-check-not-the-shape` (now carries the classifier instance too) · `the-commit-that-reshapes-a-file-blinds-its-guard` · `a-restated-number-needs-a-producer` · `a-file-that-no-longer-exists-is-a-conclusion` · `calibrate-over-the-history-that-contains-the-instance` · `a-key-too-loose-returns-the-wrong-instance` · `a-census-is-as-wide-as-its-key` · `a-scoping-defect-errs-in-one-direction` · `an-injection-must-be-shown-to-land`. Methods: `TOOLBOX.md`'s *Ask the renderer, not the specification*, *A restated number needs a PRODUCER*, and *Give every census a LIVE-CORPUS arm*.
- ⭐ **A control that passes for an unrelated reason** — promoted by `.11.20.2`, read the note: `a-control-that-passes-for-an-unrelated-reason`. It carries the in-situ falsification, the labelling rule, and why a NEGATIVE arm needs a degenerate implementation rather than the old code.
- blockers: register `SIGNOFF-REPAIR.13` + `docs/book/src/blockers.md`. Column is `Owed here?`, set from the owning leaf. **B1, B2, B3 `yes`** (work owed HERE — all three share `SIGNOFF-REPAIR.14`'s frozen exposure candidate). **C2 `yes`** (four gate records remain). **B4 `no`** (professional name clearance). **C1 `no`** — not a blocker, a limit on what may be CLAIMED. A `yes` row is WORK and rides the frontier; a `no` row is surfaced once per session with its trigger.
- ⚠️ standing corpus rule from `.13.4`: **a correction is not complete until the LIVE-DOCUMENT corpus that restates it has been censused.** Live documents = `README.md`, `LIVE_STATUS.md`, `MEMORY.md`, `docs/book/src/*.md`; the dated ledgers are history and stay.
