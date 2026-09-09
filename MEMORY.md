# MEMORY — resume pointer

## How to resume

1. Read `CLAUDE.md`, `README.md`, `MEMORY_ARCHITECTURE.md`, `TOOLBOX.md`, `DOCTRINE_ENFORCEMENT.md`, `COMMIT.md`.
2. Open `docs/tasks/SIGNOFF-REPAIR.md` and its Current Frontier.
3. Source-review records: `docs/tasks/artifacts/signoff_review/INDEX.md`.

## Current state

- **Project:** ReasonBraid; working name uncleared, repository remains private.
- **Active tree:** `SIGNOFF-REPAIR`; `.3.3.4.3.1` complete in REPAIR-0022 after 56 live controls and strict lint. No verification job remains; next `.3.3.4.3.2` activates only after commit, clean tree, cleared/untracked brief and consumed handoff census.
- **Next action:** integrate standalone boundary/grant creation, active-boundary reads and tenant-bound revocation/epoch services with the qualified private transaction owner under `.3.3.4.3.2`. Exclusive guard before authority reads/target locks, actual-parent liveness at database issuance time after waits; preserve structural rules, scheduled grants and standalone namespaces. Enrollment/import bridges remain explicitly unordered until their owners migrate; HTTP admission/final-effect coupling remains `.8`.
- **Architecture decision:** shared registries require explicit site-operator grants; tenant enrollment cannot mint them. Service, CLI and HTTP enforcement are implemented and verified (`docs/decisions/2026-09-09_site-operator-authority.md`).
- **Roadmap:** Phase 0–7 historical execution records exist; current qualification is under corrective review. Phase 8 remains incomplete. Resume `PHASE-8.5.3` after its corrective prerequisites, then `.5.4`, `.6`, and later executable work.
- **Defects:** open source-review findings owned by the repair tree; foreign-target revocation and repeated-revoke epoch defects reproduced; runner cleanup/spawn races reproduced and fixed. No zero-defect claim.
- **Latest commit:** derive with `git log -1 --oneline`; review baseline `9c2d2ba`.
- **Continuity:** GrantCreateError distinguishes missing parent, structural refusal and original SQLx storage error. Enrollment/import storage faults use safe 500; corrupt active-parent data no longer panics. Final 22 authority + 33 HTTP + one card control pass; strict focused lint, formatting and book checks pass. Every result/shutdown consumed; all four owned clusters absent. Evidence: docs/tasks/artifacts/signoff_review/grant-error-qualification.md. Guard foundation and migration delivery remain qualified by the preceding artifact; application integration is next.
- **Reading complete:** roadmap, all 284 tracked non-Markdown files (3,276,496-character corpus), and all mdBook sources, before the first repository edit.
- **PNT:** continue through executable work; commit every bounded leaf; full CI before pushes. Push cadence approximately 300 commits; baseline ahead 269.
- **Storage:** all project output/cache/temp stores must be repository-derived on its volume. use `python3 -B scripts/project_env.py <command>`; Make targets use it. Shared cache seed verified; installed tools are read-only exceptions.
- **External gates:** G6/G7 Internet qualification, license choice and ADR-001 public-name clearance remain open; they do not block local repairs.
