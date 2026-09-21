# MEMORY — resume pointer (layer A; overwrite-only)

## Current state (OVERWRITE this block each update — do not append)

- next_action: `SIGNOFF-REPAIR.11.4.6.7` — the deliberation verbs: `calls` (3 of 4 — the collection `POST /v1/calls`, `{}/close`, `{}/respond`) and `threads` (`/v1/threads/auto`). 🔎 **Check whether the prose already exists before writing any**, per `docs/knowledge/a-coverage-gap-names-what-is-missing-not-what-is-absent.md`: `.3.4.4` documented `respond`'s eight-response vocabulary, so at least part of this is notation too. Re-derive with `python3 -B scripts/census_surface_judgement.py`.
- active_work_unit: `SIGNOFF-REPAIR` → frontier leaf: `.11.4.6` (`pending`); `.11.4.6.1`–`.11.4.6.6` are `done`, and its open children are `.11.4.6.7` and `.8` (both `pending`), which hold the remaining three gaps.
- latest_commit: `REASONBRAID-DOC-0115`.
- in_flight_uncommitted: none.
- blockers: `SIGNOFF-REPAIR.13`.
- ⚠️ Standing note from `.11.4.6.2`/`.3`: the surface adjudication is **37 covered · 3 internal · 3 gap** over **43 members** — and the coarse mention test in `census_book_coverage.py` is wrong in BOTH directions, calling 21 of 30 families covered where only 14 are, while reading `/` as a bare mention when `web-ui.md` is a whole chapter about it. ⛔ Never judge a surface from that census; the ledger is `.doctrine/book_surface_verdicts.tsv` and the gate is `SURFACE-JUDGEMENT`.
- ⚠️ Standing note from `.11.8.2` (REPAIR-0378): the route census under-reported documentation because it demanded ONE space between a method and its path, and the book writes ALIGNED contract lines. The corrected figures are **104 routes — 62 described, 2 mentioned, 40 absent**; `.11.8.1`'s published `49 / 3 / 52` is **50 / 2 / 52**. The `absent` column never moved, so no published gap was ever inflated by it. ⛔ `.11.4.6.2`'s judgement must be keyed to the CORRECTED output.
- ⚠️ Standing note from the last verification: the fixture census answers TWO questions and both are needed — `python3 -B scripts/census_fixture_population.py` reports families (what WILL accumulate) AND orphans (what HAS). At `50a9712` the total is 2,619,172 KiB; 11 orphans hold 197,268 KiB, the largest `target/claude-stubs` at 270 directories with no producer. Nothing has been removed; removal has never been this lane's act.
