# MEMORY — resume pointer

## How to resume

1. Read `CLAUDE.md`, `README.md`, `MEMORY_ARCHITECTURE.md`, `TOOLBOX.md`, `DOCTRINE_ENFORCEMENT.md`, `COMMIT.md`.
2. Open `docs/tasks/SIGNOFF-REPAIR.md` and its Current Frontier.
3. Source-review records: `docs/tasks/artifacts/signoff_review/INDEX.md`.

## Current state

- **Project:** ReasonBraid; working name uncleared, repository remains private.
- **Active tree:** `SIGNOFF-REPAIR`; `.3.2.1` complete: separate site authority/service, 36 focused tests and strict lint pass. `.3.1` committed as `0bf6fda`.
- **Next action:** `.3.2.2` protected operator CLI, commit; then `.3.2.3` HTTP registry enforcement. Follow with `.3.3` actual-boundary authorization, atomic tenant effect auditing and the tracked core subject serde repair.
- **Architecture decision:** shared registries require explicit site-operator grants; tenant enrollment cannot mint them. Service implemented; CLI/HTTP integration remains (`docs/decisions/2026-09-09_site-operator-authority.md`).
- **Roadmap:** Phase 0–7 historical execution records exist; current qualification is under corrective review. Phase 8 remains incomplete. Resume `PHASE-8.5.3` after its corrective prerequisites, then `.5.4`, `.6`, and later executable work.
- **Defects:** open source-review findings owned by the repair tree; foreign-target revocation and repeated-revoke epoch defects reproduced; runner cleanup/spawn races reproduced and fixed. No zero-defect claim.
- **Latest commit:** derive with `git log -1 --oneline`; review baseline `9c2d2ba`.
- **Continuity:** `.3.2.1` commit is `REASONBRAID-REPAIR-0006`; resolve its hash with git. All live checks consumed, successful cluster removed; failed clusters removed after recorded evidence and absent-process proof. Native libpq reproduced stale prepared role membership; fresh text-query gate passes queued-revocation control. Core subject serialization remains owned by `.3.3`. No implementation pivot before the current commit and clean-tree verification.
- **Reading complete:** roadmap, all 284 tracked non-Markdown files (3,276,496-character corpus), and all mdBook sources, before the first repository edit.
- **PNT:** continue through executable work; commit every bounded leaf; full CI before pushes. Push cadence approximately 300 commits; baseline ahead 269.
- **Storage:** all project output/cache/temp stores must be repository-derived on its volume. use `python3 -B scripts/project_env.py <command>`; Make targets use it. Shared cache seed verified; installed tools are read-only exceptions.
- **External gates:** G6/G7 Internet qualification, license choice and ADR-001 public-name clearance remain open; they do not block local repairs.
