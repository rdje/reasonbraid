# MEMORY — resume pointer

## How to resume

1. Read `CLAUDE.md`, `README.md`, `MEMORY_ARCHITECTURE.md`, `TOOLBOX.md`, `DOCTRINE_ENFORCEMENT.md`, `COMMIT.md`.
2. Open `docs/tasks/SIGNOFF-REPAIR.md` and its Current Frontier.
3. Source-review records: `docs/tasks/artifacts/signoff_review/INDEX.md`.

## Current state

- **Project:** ReasonBraid; working name uncleared, repository remains private.
- **Active tree:** `SIGNOFF-REPAIR`; `.3.2.3` active: HTTP registry enforcement. Protected CLI `.3.2.2` committed as `b98aa36`, with live controls, strict lint and all 13 doctrines passed.
- **Next action:** consume the final `.3.2.3` escalation/region results and runner shutdowns, then commit closure evidence before another leaf. Follow with `.3.3` actual-boundary authorization, atomic tenant effect auditing and the tracked core subject serde repair.
- **Architecture decision:** shared registries require explicit site-operator grants; tenant enrollment cannot mint them. Service and protected CLI implemented; HTTP integration remains (`docs/decisions/2026-09-09_site-operator-authority.md`).
- **Roadmap:** Phase 0–7 historical execution records exist; current qualification is under corrective review. Phase 8 remains incomplete. Resume `PHASE-8.5.3` after its corrective prerequisites, then `.5.4`, `.6`, and later executable work.
- **Defects:** open source-review findings owned by the repair tree; foreign-target revocation and repeated-revoke epoch defects reproduced; runner cleanup/spawn races reproduced and fixed. No zero-defect claim.
- **Latest commit:** derive with `git log -1 --oneline`; review baseline `9c2d2ba`.
- **Continuity:** HTTP implementation is committed in REPAIR-0008 with verification-pending. All eight corrected HTTP controls and strict lint passed; final adjacent checks remain. Failed `run-79xtxgad` evidence is durable and its stopped cluster removed. Consume adjacent-security runner `target/pg-tests/run-7swjcb5s` and corrected wire-input/registry runner `target/pg-tests/run-ouyhe95e`, then finish gates/docs and commit before another leaf. Keep fresh text operator-role queries. Core serde, delivery-reader error typing and other passfile entrypoints remain owned by `.3.3`/`.5.3`/`.11.2`.
- **Reading complete:** roadmap, all 284 tracked non-Markdown files (3,276,496-character corpus), and all mdBook sources, before the first repository edit.
- **PNT:** continue through executable work; commit every bounded leaf; full CI before pushes. Push cadence approximately 300 commits; baseline ahead 269.
- **Storage:** all project output/cache/temp stores must be repository-derived on its volume. use `python3 -B scripts/project_env.py <command>`; Make targets use it. Shared cache seed verified; installed tools are read-only exceptions.
- **External gates:** G6/G7 Internet qualification, license choice and ADR-001 public-name clearance remain open; they do not block local repairs.
