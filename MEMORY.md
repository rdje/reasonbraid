# MEMORY — resume pointer

## How to resume

1. Read `CLAUDE.md`, `README.md`, `MEMORY_ARCHITECTURE.md`, `TOOLBOX.md`, `DOCTRINE_ENFORCEMENT.md`, `COMMIT.md`.
2. Open `docs/tasks/SIGNOFF-REPAIR.md` and its Current Frontier.
3. Source-review records: `docs/tasks/artifacts/signoff_review/INDEX.md`.

## Current state

- **Project:** ReasonBraid; working name uncleared, repository remains private.
- **Active tree:** `SIGNOFF-REPAIR`; `.3.3.4.3.2` complete in REPAIR-0023 after 85 selected controls (84 live / one pure), strict lint and book checks. Every result/shutdown is consumed; no verification job remains. Next enrollment child activates only after commit, cleared/untracked brief, clean tree and consumed handoff census.
- **Next action:** `.3.3.4.3.3`: integrate the complete development enrollment transaction with the tenant guard before replay, boundary/grant/identity/quota/enrollment work; use fresh database time and the same-context grant service. Preserve actual dev issuer semantics and successful response/replay shape. Reproduce both revocation orders, queued expiry, atomic failure/recovery and concurrent existing-tenant replay before closure. Card import remains `.3.3.4.11`; caller policy remains `.3.5`.
- **Architecture decision:** shared registries require explicit site-operator grants; tenant enrollment cannot mint them. Service, CLI and HTTP enforcement are implemented and verified (`docs/decisions/2026-09-09_site-operator-authority.md`).
- **Roadmap:** Phase 0–7 historical execution records exist; current qualification is under corrective review. Phase 8 remains incomplete. Resume `PHASE-8.5.3` after its corrective prerequisites, then `.5.4`, `.6`, and later executable work.
- **Defects:** open source-review findings owned by the repair tree; foreign-target revocation and repeated-revoke epoch defects reproduced; runner cleanup/spawn races reproduced and fixed. No zero-defect claim.
- **Latest commit:** derive with `git log -1 --oneline`; review baseline `9c2d2ba`.
- **Continuity:** five standalone services now use the guard; public errors preserve original SQL causes and commit uncertainty, malformed status preserves target/epoch, and issuance checks current parent time after guard/read. HTTP exposes commit_outcome_unconfirmed. Final 12 issuance + 14 primitive + 22 authority + 33 HTTP + one card + three upgrade controls pass; strict lint and book checks pass. Actual target→guard contention was observed and its old fixture corrected. All three owned clusters absent; evidence: docs/tasks/artifacts/signoff_review/tenant-authority-issuance.md. Complete enrollment/import and admission/effect coupling remain pending.
- **Reading complete:** roadmap, all 284 tracked non-Markdown files (3,276,496-character corpus), and all mdBook sources, before the first repository edit.
- **PNT:** continue through executable work; commit every bounded leaf; full CI before pushes. Push cadence approximately 300 commits; baseline ahead 269.
- **Storage:** all project output/cache/temp stores must be repository-derived on its volume. use `python3 -B scripts/project_env.py <command>`; Make targets use it. Shared cache seed verified; installed tools are read-only exceptions.
- **External gates:** G6/G7 Internet qualification, license choice and ADR-001 public-name clearance remain open; they do not block local repairs.
