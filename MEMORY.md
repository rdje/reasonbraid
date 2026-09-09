# MEMORY — resume pointer

## How to resume

1. Read `CLAUDE.md`, `README.md`, `MEMORY_ARCHITECTURE.md`, `TOOLBOX.md`, `DOCTRINE_ENFORCEMENT.md`, `COMMIT.md`.
2. Open `docs/tasks/SIGNOFF-REPAIR.md` and its Current Frontier.
3. Source-review records: `docs/tasks/artifacts/signoff_review/INDEX.md`.

## Current state

- **Project:** ReasonBraid; working name uncleared, repository remains private.
- **Active tree:** `SIGNOFF-REPAIR`; `.3.1` closes tenant-bound revocation in `REASONBRAID-REPAIR-0005`; next leaf `.3.2` explicit site authority.
- **Next action:** activate `.3.2`, decompose if needed, then implement explicit site-operator grants and protected issuance/revocation tooling for adapter and region registries. Follow with `.3.3` bound-boundary authorization and atomic effect auditing.
- **Architecture decision:** shared mutations require explicit site-operator grants issued through protected deployment tooling; tenant enrollment cannot mint them. Design accepted, implementation pending (`docs/decisions/2026-09-09_site-operator-authority.md`).
- **Roadmap:** Phase 0–7 historical execution records exist; current qualification is under corrective review. Phase 8 remains incomplete. Resume `PHASE-8.5.3` after its corrective prerequisites, then `.5.4`, `.6`, and later executable work.
- **Defects:** open source-review findings owned by the repair tree; foreign-target revocation and repeated-revoke epoch defects reproduced; runner cleanup/spawn races reproduced and fixed. No zero-defect claim.
- **Latest commit:** derive with `git log -1 --oneline`; review baseline `9c2d2ba`.
- **Commit boundary:** `.3.1` verified by 34 focused tests, strict server/API-test Clippy and format; all owned PG clusters removed. Verify this commit, cleared message file and clean state before starting `.3.2`. Admission/effect audit remains explicitly distinguished in the book.
- **Reading complete:** roadmap, all 284 tracked non-Markdown files (3,276,496-character corpus), and all mdBook sources, before the first repository edit.
- **PNT:** continue through executable work; commit every bounded leaf; full CI before pushes. Push cadence approximately 300 commits; baseline ahead 269.
- **Storage:** all project output/cache/temp stores must be repository-derived on its volume. use `python3 -B scripts/project_env.py <command>`; Make targets use it. Shared cache seed verified; installed tools are read-only exceptions.
- **External gates:** G6/G7 Internet qualification, license choice and ADR-001 public-name clearance remain open; they do not block local repairs.
