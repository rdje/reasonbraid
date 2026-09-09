# MEMORY — resume pointer

## How to resume

1. Read `CLAUDE.md`, `README.md`, `MEMORY_ARCHITECTURE.md`, `TOOLBOX.md`, `DOCTRINE_ENFORCEMENT.md`, `COMMIT.md`.
2. Open `docs/tasks/SIGNOFF-REPAIR.md` and its Current Frontier.
3. Source-review records: `docs/tasks/artifacts/signoff_review/INDEX.md`.

## Current state

- **Project:** ReasonBraid; working name uncleared, repository remains private.
- **Active tree:** `SIGNOFF-REPAIR`; `.3.3.4.1` census/design complete in REPAIR-0020; `.3.3.4.2` is next. Prior actual-parent selection and inspection provenance/readback `.3.3.3` are complete.
- **Next action:** after clean-commit postconditions, activate `.3.3.4.2` and implement/qualify a dedicated tenant guard with shared/exclusive transaction ownership. Read docs/decisions/2026-09-09_tenant-authority-transaction-order.md and docs/tasks/artifacts/signoff_review/tenant-authority-paths.md first. Do not use tenant identity rows as required anchors; standalone authority permits absent identity rows. No integrated guard behavior is claimed yet.
- **Architecture decision:** shared registries require explicit site-operator grants; tenant enrollment cannot mint them. Service, CLI and HTTP enforcement are implemented and verified (`docs/decisions/2026-09-09_site-operator-authority.md`).
- **Roadmap:** Phase 0–7 historical execution records exist; current qualification is under corrective review. Phase 8 remains incomplete. Resume `PHASE-8.5.3` after its corrective prerequisites, then `.5.4`, `.6`, and later executable work.
- **Defects:** open source-review findings owned by the repair tree; foreign-target revocation and repeated-revoke epoch defects reproduced; runner cleanup/spawn races reproduced and fixed. No zero-defect claim.
- **Latest commit:** derive with `git log -1 --oneline`; review baseline `9c2d2ba`.
- **Continuity:** no runtime job in the census/design child. Its 101-file / 1,749,975-byte Rust corpus hash and exact 42-location re-derivation/negative controls are durable in the artifact; thirteen children own guard and effect integration. Earlier receipt readback passes 18 authority + 30 HTTP tests and strict lint, all results/shutdown consumed. Metrics remains `.3.5`, source evidence only.
- **Reading complete:** roadmap, all 284 tracked non-Markdown files (3,276,496-character corpus), and all mdBook sources, before the first repository edit.
- **PNT:** continue through executable work; commit every bounded leaf; full CI before pushes. Push cadence approximately 300 commits; baseline ahead 269.
- **Storage:** all project output/cache/temp stores must be repository-derived on its volume. use `python3 -B scripts/project_env.py <command>`; Make targets use it. Shared cache seed verified; installed tools are read-only exceptions.
- **External gates:** G6/G7 Internet qualification, license choice and ADR-001 public-name clearance remain open; they do not block local repairs.
