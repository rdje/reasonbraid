# MEMORY — resume pointer

## How to resume

1. Read `CLAUDE.md`, `README.md`, `MEMORY_ARCHITECTURE.md`, `TOOLBOX.md`, `DOCTRINE_ENFORCEMENT.md`, `COMMIT.md`.
2. Open `docs/tasks/SIGNOFF-REPAIR.md` and its Current Frontier.
3. Source-review records: `docs/tasks/artifacts/signoff_review/INDEX.md`.

## Current state

- **Project:** ReasonBraid; working name uncleared, repository remains private.
- **Active tree:** `SIGNOFF-REPAIR`; `.3.3.4.2` complete; finish the REPAIR-0021 commit workflow. Actual-parent selection and inspection provenance/readback `.3.3.3` are complete.
- **Next action:** finish COMMIT.md for REPAIR-0021 if still dirty; otherwise activate `.3.3.4.3` and refine bounded authority-writer/issuance/enrollment children before implementation. Read docs/decisions/2026-09-09_tenant-authority-transaction-order.md and the tenant-authority-paths/tenant-guard-qualification artifacts. Guard primitives are tested through the private-source integration seam; no application path is wired yet. Coordinate status writers before claiming issuance/revocation ordering.
- **Architecture decision:** shared registries require explicit site-operator grants; tenant enrollment cannot mint them. Service, CLI and HTTP enforcement are implemented and verified (`docs/decisions/2026-09-09_site-operator-authority.md`).
- **Roadmap:** Phase 0–7 historical execution records exist; current qualification is under corrective review. Phase 8 remains incomplete. Resume `PHASE-8.5.3` after its corrective prerequisites, then `.5.4`, `.6`, and later executable work.
- **Defects:** open source-review findings owned by the repair tree; foreign-target revocation and repeated-revoke epoch defects reproduced; runner cleanup/spawn races reproduced and fixed. No zero-defect claim.
- **Latest commit:** derive with `git log -1 --oneline`; review baseline `9c2d2ba`.
- **Continuity:** final guard/upgrade/authority run passes 35 controls (34 live / one pure); 28 migration-directory rebuild/cache checks and all four focused strict lint commands pass. Results and shutdown consumed; all seven owned clusters and both temporary probes are absent. The cancelled-BEGIN connection-ownership repair and cached-migration build fix are durable in the leaf/artifact. The commit-checker path prerequisite is also corrected: seven focused controls and actual staged acceptance pass, with local Git fixtures removed. No verification job remains. Application/effect integration and remaining SQLx entrypoints retain explicit owners.
- **Reading complete:** roadmap, all 284 tracked non-Markdown files (3,276,496-character corpus), and all mdBook sources, before the first repository edit.
- **PNT:** continue through executable work; commit every bounded leaf; full CI before pushes. Push cadence approximately 300 commits; baseline ahead 269.
- **Storage:** all project output/cache/temp stores must be repository-derived on its volume. use `python3 -B scripts/project_env.py <command>`; Make targets use it. Shared cache seed verified; installed tools are read-only exceptions.
- **External gates:** G6/G7 Internet qualification, license choice and ADR-001 public-name clearance remain open; they do not block local repairs.
