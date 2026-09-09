# MEMORY — resume pointer

## How to resume

1. Read `CLAUDE.md`, `README.md`, `MEMORY_ARCHITECTURE.md`, `TOOLBOX.md`, `DOCTRINE_ENFORCEMENT.md`, `COMMIT.md`.
2. Open `docs/tasks/SIGNOFF-REPAIR.md` and its Current Frontier.
3. Source-review records: `docs/tasks/artifacts/signoff_review/INDEX.md`.

## Current state

- **Project:** ReasonBraid; working name uncleared. Remote rdje/reasonbraid is confirmed PUBLIC by authenticated and unauthenticated GitHub APIs, conflicting with README/ADR-001. No push or visibility change; director decision required.
- **Active tree:** `SIGNOFF-REPAIR`; publication-precondition audit .11.4.3.1.2.1 complete through REPAIR-0042. Evidence: docs/tasks/artifacts/signoff_review/publication-precondition.md. All started gate/probe/cleanup results are consumed; native census is clear.
- **Next action:** ask director whether to restore repository privacy under .11.4.3.1.2.3. Then classify/repair two redacted historical fixture-token findings .2.2 and resume full checkpoint .2 on a fresh named source/receipt directory. Source-f0265e2 format/cargo-deny passed; history scan failed (2 matches), Clippy deliberately interrupted/consumed at policy conflict; worker/workspace/Python/PG/demo gates not started, no push. Preserve target/checkpoint-ci/full-f0265e2 and both named scanner runs. Return to CLI .3.3.4.3.3.3.3.2.3.2 after checkpoint.
- **Architecture decision:** shared registries require explicit site-operator grants; tenant enrollment cannot mint them. Service, CLI and HTTP enforcement are implemented and verified (`docs/decisions/2026-09-09_site-operator-authority.md`).
- **Roadmap:** Phase 0–7 historical execution records exist; current qualification is under corrective review. Phase 8 remains incomplete. Resume `PHASE-8.5.3` after its corrective prerequisites, then `.5.4`, `.6`, and later executable work.
- **Defects:** open source-review findings owned by the repair tree; foreign-target revocation and repeated-revoke epoch defects reproduced; runner cleanup/spawn races reproduced and fixed. No zero-defect claim.
- **Latest commit:** derive with `git log -1 --oneline`; review baseline `9c2d2ba`.
- **Continuity:** server canonical RequestId/outcome recovery is qualified after actual commit uncertainty/disconnect. CLI now saves that key before HTTP, preserves matching requests through failure, strictly validates original replies, publishes principal/receipt and clears pending under one guard. Explicit --resume-bootstrap recovers pending/latest completion; completed receipts restore locally with labelled historical source, normal no-pending invocation stays fresh. Real unread-output interruption and server restoration/fresh intent pass. Prospective completion capacity is now checked before dispatch; later filesystem errors, live authority and endpoint/database continuity retain their limits. Semantic introspection/MCP remains tracked .6.4 proposal, no pivot.
- **Reading complete:** roadmap, all 284 tracked non-Markdown files (3,276,496-character corpus), and all mdBook sources, before the first repository edit.
- **PNT:** stopped only at the confirmed publication-policy conflict; resume after director decision. Commit every bounded leaf, full CI before push. Source f0265e2 was 310 commits ahead of confirmed remote main b932c054023ea127520e74cfaf95b9bdf1ea47fe.
- **Storage:** all project output/cache/temp stores must be repository-derived on its volume. use `python3 -B scripts/project_env.py <command>`; Make targets use it. Shared cache seed verified; installed tools are read-only exceptions.
- **External gates:** G6/G7 Internet qualification, license choice and ADR-001 public-name clearance remain open; they do not block local repairs.
