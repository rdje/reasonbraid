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

- [Bootstrap completion capacity](docs/tasks/artifacts/signoff_review/bootstrap-capacity.md): exact-boundary real CLI reproduction and pre-dispatch capacity repair under .3.3.4.3.3.3.3.2.3.1.

- [Keyed CLI bootstrap flow](docs/tasks/artifacts/signoff_review/cli-bootstrap-flow.md): matched missing-key baseline, implementation and explicit recovery qualification under .3.3.4.3.3.3.3.2.2.

- [Tenant authority/effect paths](docs/tasks/artifacts/signoff_review/tenant-authority-paths.md), owner `SIGNOFF-REPAIR.3.3.4.1`, source baseline `1ba6184`: 42 direct named-call locations plus transitive/mutation and alternate-gate coverage. The selected guard/effect contract has bounded implementation owners; this source census is not runtime ordering qualification.
- [Tenant guard qualification](docs/tasks/artifacts/signoff_review/tenant-guard-qualification.md), owner `SIGNOFF-REPAIR.3.3.4.2`: migration, guard/connection ownership, observed waits, cancelled-BEGIN reproduction/repair and commit uncertainty; application integration remains subsequent children.
- [Grant creation errors](docs/tasks/artifacts/signoff_review/grant-error-qualification.md), owner `SIGNOFF-REPAIR.3.3.4.3.1`: repository error-loss, enrollment panic and enrollment/import HTTP baselines, typed errors, checked decoding and matched failure/recovery controls. All 56 live controls and focused strict lint pass; results/shutdown consumed and four owned clusters absent.
- [Standalone authority writers](docs/tasks/artifacts/signoff_review/tenant-authority-issuance.md), owner `SIGNOFF-REPAIR.3.3.4.3.2`: guarded issuance/status services, actual wait dependencies, current parent liveness, checked status and public commit uncertainty. All 85 selected controls and focused strict lint pass; every result/shutdown consumed and three owned clusters absent.

- [Typed rollback errors](docs/tasks/artifacts/signoff_review/typed-rollback-errors.md), owner `SIGNOFF-REPAIR.3.3.4.3.3.1`: matched first-use anchor/deferred refusal failures; typed error rollback, SQL cause/deadline/commit-phase preservation and recovery. All 89 selected controls (88 live / one pure), focused strict lint and book checks pass; every result/shutdown consumed and both owned clusters absent.

- [Complete enrollment transaction](docs/tasks/artifacts/signoff_review/enrollment-transaction.md), owner `SIGNOFF-REPAIR.3.3.4.3.3.2`: replay-before-validation compatibility, complete exclusive-guard ordering, fresh database time, exact eight-table rollback and concurrent replay. All 97 selected controls (96 live / one pure), final focused strict lint and book checks pass; all results/shutdown consumed and three owned clusters absent. Client bootstrap recovery is tracked separately in the next child.

- [Bootstrap recovery qualification](docs/tasks/artifacts/signoff_review/bootstrap-recovery.md), owner `SIGNOFF-REPAIR.3.3.4.3.3.3.1`: actual commit uncertainty followed by original committed readback and distinct no-key repetition; selected RequestId/guarded outcome/durable CLI contract. All 25 selected controls and focused strict lint pass; all results/shutdown consumed and cluster absent. Server/CLI recovery implementation remains pending.


- [Keyed bootstrap server](docs/tasks/artifacts/signoff_review/bootstrap-server.md), owner `SIGNOFF-REPAIR.3.3.4.3.3.3.2`: matched absent-field baseline, optional canonical RequestId, guarded outcome binding/recovery and migration; 73 selected controls, final eleven-control fixture rerun and strict lint pass. Every result/shutdown consumed, four owned clusters absent.
- [Semantic introspection proposal](docs/tasks/artifacts/signoff_review/semantic-introspection-proposal.md), capture owner `.3.3.4.3.3.3.2`, assessment owner `SIGNOFF-REPAIR.6.4`: director discussion on a typed API/MCP for evidence-based autonomous diagnosis. Proposed assessment only; no implementation pivot or full automation claim.

- [CLI state publication](docs/tasks/artifacts/signoff_review/cli-state-publication.md), owner `SIGNOFF-REPAIR.3.3.4.3.3.3.3.1.1`: three matched in-place/link/exclusion defects repaired with bounded strict snapshots, same-volume paths and synchronized atomic replacement. Twelve selected controls, strict lint and book checks pass; results consumed, unique fixtures absent. Whole HTTP writer locking and pending bootstrap recovery remain staged.

- [Whole CLI writer exclusion](docs/tasks/artifacts/signoff_review/cli-writer-locking.md), owner `SIGNOFF-REPAIR.3.3.4.3.3.3.3.1.2`: matched busy-error-after-dispatch baseline; held store across HTTP/fresh actor/merge/publication, overlap/crash controls and real-server CLI compatibility. Nineteen selected controls, final strict lint and book checks pass; results/shutdown consumed, fixtures/cluster absent. Pending request/deadline recovery remains next.

- [Bootstrap recovery state](docs/tasks/artifacts/signoff_review/bootstrap-state-schema.md), owner `SIGNOFF-REPAIR.3.3.4.3.3.3.3.2.1`: strict version-two pending/completed records, legacy wire compatibility and guarded publication continuity. Twenty-four selected controls, strict lint and book checks pass; results consumed and fixtures absent. The interrupted pre-main launch and unchanged-hash retry remain explicit. Keyed CLI/explicit resume and deadlines are still staged.

- [Scheduled CI checkpoint census](docs/tasks/artifacts/signoff_review/ci-checkpoint-census.md): source/target/skip/tool/locality/pressure inventory and concrete pre-push repair owners under .11.4.3.1.

- [CI environment launcher](docs/tasks/artifacts/signoff_review/ci-environment.md): instrumented raw-workflow baseline and eight real-child locality/lifetime controls under .11.4.3.1.3.1.

- [Pinned CI scanners](docs/tasks/artifacts/signoff_review/ci-scanners.md): eight-archive identity/layout verification, thirteen controls and native version-contract correction under .11.4.3.1.3.2.
