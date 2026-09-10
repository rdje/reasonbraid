# MEMORY — resume pointer

## How to resume

1. Read `CLAUDE.md`, `README.md`, `MEMORY_ARCHITECTURE.md`, `TOOLBOX.md`, `DOCTRINE_ENFORCEMENT.md`, `COMMIT.md`.
2. Open `docs/tasks/SIGNOFF-REPAIR.md` and its Current Frontier.
3. Source-review records: `docs/tasks/artifacts/signoff_review/INDEX.md`.

## Current state

- **Project:** ReasonBraid; public repository rdje/reasonbraid must remain public. Director correction 2026-09-09 supersedes the wrong README/ADR private instruction. Working-name clearance remains open and does not block public source visibility.
- **Active tree:** `SIGNOFF-REPAIR`; REPAIR-0056 .11.4.3.1.2.11.2 explicitly releases the state lock before File close, including post-acquisition failures. Five-path permanent child regression fails on unchanged production and passes after repair; all 32 selected CLI tests, final twelve-writer rerun, six actual-API raw-fork scenarios and strict CLI lint/format pass. Two fixture probes also explicitly release. Original assertions/339 other non-Markdown sources unchanged; original evidence retained. Final verification passes: 62 groups absent, preserved evidence and eight rendered markers; parent .2.11 closes; broader inherited-descriptor owner-death qualification remains the existing restart leaf.
- **Next action:** commit REPAIR-0056, clear/verify brief and clean tree, consume native handoff census; resume full checkpoint .11.4.3.1.2 on that committed source, then authorized public push/actual remote CI and CLI transport .3.3.4.3.3.3.3.2.3.2. Raw repair: target/state-writer-lock-controls/repair. Both source-b0cddfe and source-8d1504d checkpoints remain failed/incomplete; never relabel them. Use the pinned browser wrapper and all forty PostgreSQL commands plus explicit local demo. Preserve host diagnostics, prior failure databases/payloads and original lock diagnosis.
- **Architecture decision:** shared registries require explicit site-operator grants; tenant enrollment cannot mint them. Service, CLI and HTTP enforcement are implemented and verified (`docs/decisions/2026-09-09_site-operator-authority.md`).
- **Roadmap:** Phase 0–7 historical execution records exist; current qualification is under corrective review. Phase 8 remains incomplete. Resume `PHASE-8.5.3` after its corrective prerequisites, then `.5.4`, `.6`, and later executable work.
- **Defects:** open source-review findings owned by the repair tree; foreign-target revocation and repeated-revoke epoch defects reproduced; runner cleanup/spawn races reproduced and fixed. No zero-defect claim.
- **Latest commit:** derive with `git log -1 --oneline`; review baseline `9c2d2ba`.
- **Continuity:** server canonical RequestId/outcome recovery is qualified after actual commit uncertainty/disconnect. CLI now saves that key before HTTP, preserves matching requests through failure, strictly validates original replies, publishes principal/receipt and clears pending under one guard. Explicit --resume-bootstrap recovers pending/latest completion; completed receipts restore locally with labelled historical source, normal no-pending invocation stays fresh. Real unread-output interruption and server restoration/fresh intent pass. Prospective completion capacity is now checked before dispatch; later filesystem errors, live authority and endpoint/database continuity retain their limits. Semantic introspection/MCP remains tracked .6.4 proposal, no pivot.
- **Reading complete:** roadmap, all 284 tracked non-Markdown files (3,276,496-character corpus), and all mdBook sources, before the first repository edit.
- **PNT:** resumed after director correction; continue executable work, committing each bounded leaf and running full CI before push. Source f0265e2 was 310 commits ahead of confirmed remote main b932c054023ea127520e74cfaf95b9bdf1ea47fe.
- **Storage:** all project output/cache/temp stores must be repository-derived on its volume. use `python3 -B scripts/project_env.py <command>`; Make targets use it. Shared cache seed verified; installed tools are read-only exceptions.
- **External gates:** G6/G7 Internet qualification, license choice and ADR-001 public-name clearance remain open; they do not block local repairs.
