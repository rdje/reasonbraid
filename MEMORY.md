# MEMORY — resume pointer

## How to resume

1. Read `CLAUDE.md`, `README.md`, `MEMORY_ARCHITECTURE.md`, `TOOLBOX.md`, `DOCTRINE_ENFORCEMENT.md`, `COMMIT.md`.
2. Open `docs/tasks/SIGNOFF-REPAIR.md` and its Current Frontier.
3. Source-review records: `docs/tasks/artifacts/signoff_review/INDEX.md`.

## Current state

- **Project:** ReasonBraid; public repository rdje/reasonbraid must remain public. Director correction 2026-09-09 supersedes the wrong README/ADR private instruction. Working-name clearance remains open and does not block public source visibility.
- **Active tree:** `SIGNOFF-REPAIR`; REPAIR-0053 .11.4.3.1.2.7.3 completes six partial fixture plans after clean/census-verified afbf7c4. Every unchanged node_work→consumer baseline reproduces the incarnation FK failure; all six corrected pairs and the consecutive consumer run pass (14 distinct tests, 60 final executions). Explicit minimum FK closure preserves original table order, feature assertions and observed CA rows. Strict server lint/format/book/source verification pass; all 46 groups absent, seven successful databases removed and six failed baselines preserved. Production/schema and categories unchanged.
- **Next action:** commit REPAIR-0053, clear/verify brief and clean state, consume native census; execute remaining adoption/coverage .11.4.3.1.2.7.4. The original 25-plan census now has twenty checked callers; five remain: regions, allowlist, rls, mcp_write and the MCP internal fixture. Inspect explicit cascade dependencies too, then qualify affected regressions before the full checkpoint .2, authorized public push/remote CI and CLI transport .3.3.4.3.3.3.3.2.3.2. Raw: target/partial-fixture-controls. Preserve all failed fixtures and .11.2 host diagnostics; use pinned browser wrapper for workspace tests.
- **Architecture decision:** shared registries require explicit site-operator grants; tenant enrollment cannot mint them. Service, CLI and HTTP enforcement are implemented and verified (`docs/decisions/2026-09-09_site-operator-authority.md`).
- **Roadmap:** Phase 0–7 historical execution records exist; current qualification is under corrective review. Phase 8 remains incomplete. Resume `PHASE-8.5.3` after its corrective prerequisites, then `.5.4`, `.6`, and later executable work.
- **Defects:** open source-review findings owned by the repair tree; foreign-target revocation and repeated-revoke epoch defects reproduced; runner cleanup/spawn races reproduced and fixed. No zero-defect claim.
- **Latest commit:** derive with `git log -1 --oneline`; review baseline `9c2d2ba`.
- **Continuity:** server canonical RequestId/outcome recovery is qualified after actual commit uncertainty/disconnect. CLI now saves that key before HTTP, preserves matching requests through failure, strictly validates original replies, publishes principal/receipt and clears pending under one guard. Explicit --resume-bootstrap recovers pending/latest completion; completed receipts restore locally with labelled historical source, normal no-pending invocation stays fresh. Real unread-output interruption and server restoration/fresh intent pass. Prospective completion capacity is now checked before dispatch; later filesystem errors, live authority and endpoint/database continuity retain their limits. Semantic introspection/MCP remains tracked .6.4 proposal, no pivot.
- **Reading complete:** roadmap, all 284 tracked non-Markdown files (3,276,496-character corpus), and all mdBook sources, before the first repository edit.
- **PNT:** resumed after director correction; continue executable work, committing each bounded leaf and running full CI before push. Source f0265e2 was 310 commits ahead of confirmed remote main b932c054023ea127520e74cfaf95b9bdf1ea47fe.
- **Storage:** all project output/cache/temp stores must be repository-derived on its volume. use `python3 -B scripts/project_env.py <command>`; Make targets use it. Shared cache seed verified; installed tools are read-only exceptions.
- **External gates:** G6/G7 Internet qualification, license choice and ADR-001 public-name clearance remain open; they do not block local repairs.
