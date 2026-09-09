# MEMORY — resume pointer

## How to resume

1. Read `CLAUDE.md`, `README.md`, `MEMORY_ARCHITECTURE.md`, `TOOLBOX.md`, `DOCTRINE_ENFORCEMENT.md`, `COMMIT.md`.
2. Open `docs/tasks/SIGNOFF-REPAIR.md` and its Current Frontier.
3. Source-review records: `docs/tasks/artifacts/signoff_review/INDEX.md`.

## Current state

- **Project:** ReasonBraid; working name uncleared, repository remains private.
- **Active tree:** `SIGNOFF-REPAIR`; `.3.3.4.3.3.1` complete in REPAIR-0024. Typed rollback support is qualified; complete enrollment `.3.3.4.3.3.2` is next. All verification results and shutdown are consumed; both owned clusters are absent.
- **Next action:** finish the REPAIR-0024 commit/brief/clean handoff checks, then activate `.3.3.4.3.3.2`: put replay and all development enrollment writes under one exclusive guard, using typed rollback errors and same-context authority helpers. Preserve replay before action parsing, successful response shape and the documented dev issuer policy; qualify contention, expiry, exact rollback and concurrent replay.
- **Architecture decision:** shared registries require explicit site-operator grants; tenant enrollment cannot mint them. Service, CLI and HTTP enforcement are implemented and verified (`docs/decisions/2026-09-09_site-operator-authority.md`).
- **Roadmap:** Phase 0–7 historical execution records exist; current qualification is under corrective review. Phase 8 remains incomplete. Resume `PHASE-8.5.3` after its corrective prerequisites, then `.5.4`, `.6`, and later executable work.
- **Defects:** open source-review findings owned by the repair tree; foreign-target revocation and repeated-revoke epoch defects reproduced; runner cleanup/spawn races reproduced and fixed. No zero-defect claim.
- **Latest commit:** derive with `git log -1 --oneline`; review baseline `9c2d2ba`.
- **Continuity:** private transact_with_error takes bounded Limits and shares the qualified connection/guard/commit implementation. create_grant now returns typed errors directly, aborting new anchors on missing/structural/time refusals; pre-existing anchors remain. Fixed callback success values still commit intentional refusal evidence. All 89 selected controls (88 live / one pure), focused strict lint and book checks pass; original SQL causes, whole deadlines and actual commit uncertainty survive. Evidence: docs/tasks/artifacts/signoff_review/typed-rollback-errors.md. Complete enrollment/import and admission/effect coupling remain pending.
- **Reading complete:** roadmap, all 284 tracked non-Markdown files (3,276,496-character corpus), and all mdBook sources, before the first repository edit.
- **PNT:** continue through executable work; commit every bounded leaf; full CI before pushes. Push cadence approximately 300 commits; baseline ahead 269.
- **Storage:** all project output/cache/temp stores must be repository-derived on its volume. use `python3 -B scripts/project_env.py <command>`; Make targets use it. Shared cache seed verified; installed tools are read-only exceptions.
- **External gates:** G6/G7 Internet qualification, license choice and ADR-001 public-name clearance remain open; they do not block local repairs.
