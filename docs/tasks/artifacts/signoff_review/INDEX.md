# Full-read source census

Owner: `SIGNOFF-REPAIR.1`. Baseline: `9c2d2ba`. Review spans 2026-09-08–09.

These notes preserve source-level mechanisms and test limitations from the full startup read. They are **not runtime reproduction results**. The repair tree defines execution and closure criteria. Each record carries candidate owning leaves; a record mentioning several mechanisms may require several of those leaves. Resolve every mechanism before marking its record closed. The full source corpus was read; notes record findings, not a replacement for source code.

- [Part 1](docs/tasks/artifacts/signoff_review/census-1.md): pages 6_27, 31_32, 33_35, 36_39, 40_42, 43, 44_45, 46.
- [Part 2](docs/tasks/artifacts/signoff_review/census-2.md): pages 47, 48_49, 50, 51, 52, 53, 54, 55.
- [Part 3](docs/tasks/artifacts/signoff_review/census-3.md): pages 56_57, 58, 59, 61_62, 63, 65, 66, 67_68.
- [Part 4](docs/tasks/artifacts/signoff_review/census-4.md): pages 69, 70, 71_72, 73_74, 75, 76_77, 78, 80_82.
- [Part 5](docs/tasks/artifacts/signoff_review/census-5.md): pages 83, 84, 85, 86, 87, 88, 89, 90.

## Coverage and continuity

Measured baseline: 284 tracked non-Markdown files; corpus SHA-256
`f474835f3d35a3191907b00f120e3b4afc42ac3840b9b17c37f218654092f57a`.
The digest covers UTF-8 bytes of the full corpus including path separators.
The baseline commit is `9c2d2baa2a5d0661536fb5432b41f230a5c7216f`.

The progress-file census found `LIVE_STATUS.md` at 42,374 bytes in 18 lines,
with no separate rows for Phases 5, 6, 7 or 9. Most later phase history had
accumulated inside the Phase 4 row. Leaf `.1` replaces the current snapshot;
the exact former content remains at `9c2d2ba:LIVE_STATUS.md`. This is a
documentation defect confirmed by parsing the file, not a runtime product finding.

The corpus was generated from sorted `git ls-files`, including every tracked non-`.md` file, and read without omitted pages (0 through 91). Roadmap and mdBook Markdown were read independently. No code changes, tests, or builds occurred before reading completed. The first changed file was the owning `docs/tasks/SIGNOFF-REPAIR.md`.

The historical phase trees remain the provenance of earlier implementations. This census requires their current qualification claims to be reconciled by the owning repair leaves. `SIGNOFF-REPAIR.12` may close only after every source-review record has a fixed or refuted disposition supported by tools.

## Focused corrective follow-ups

- [Tenant authority/effect paths](docs/tasks/artifacts/signoff_review/tenant-authority-paths.md), owner `SIGNOFF-REPAIR.3.3.4.1`, source baseline `1ba6184`: 42 direct named-call locations plus transitive/mutation and alternate-gate coverage. The selected guard/effect contract has bounded implementation owners; this source census is not runtime ordering qualification.
