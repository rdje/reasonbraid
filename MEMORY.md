# MEMORY — resume pointer (layer A; overwrite-only, keep ≤ ~50 lines)

> The bounded layer-A resume pointer (see `MEMORY_ARCHITECTURE.md`). OVERWRITE the
> "Current state" block each update — never append history here.

## How to resume

1. Read `README.md`, `MEMORY_ARCHITECTURE.md`, `TOOLBOX.md`, `DOCTRINE_ENFORCEMENT.md`.
2. Open the active task-tree below → its Current Frontier → continue from the next action.
3. Scope/gates: `ROADMAP.md`. Phase 1 execution: the `PHASE-1` tree under `docs/tasks/`.

## Current state

- **Project:** ReasonBraid (working name, not legally cleared).
- **Phase 0 COMPLETE:** the exit gate is closed (ADR-002 signed by the accountable owner —
  `docs/adr/002-phase1-scope.md`, GO) and the PHASE-0 tree is `done`: WP1–WP8 +
  `MAINT-1` (README_POLICY re-adopted: derived caps + routing-pressure closure) +
  `MAINT-2` (ReasonBraid-only naming — zero scaffold-name tokens remain).
- **Active tree:** `PHASE-1` → frontier `.1.3.1` (the `.1` coordinator leaf is **done**;
  `.1.2` is **complete**: enrollment · authenticated channel + lease/presence ·
  inbox hardening — backlogs 11–14 closed; `.1.3` is **decomposed** into `.1.3.1`
  (the explicit-participants contract: invitation lifecycle + dispatch-on-accept,
  one contract) → `.1.3.2` (join/subscriptions); the dev node-id space is `nod_…`
  OR the `rol_…` role wire id the dev wiring serves).
- **Next action:** implement `PHASE-1.3.1` — the explicit-participants contract:
  `thread.invite` records a PENDING invitation (typed optional expiry) and
  enqueues NO work; `thread.accept_invitation` (invited role, new
  `thread_invitation_respond` grant) transitions to `accepted` AND dispatches the
  work item with its reservation; `thread.decline_invitation`,
  `thread.remove_participant` (tenant_admin; core `revoked` state); expiry
  derived from `expires_at`; invited roles may not act until accepted.
- **Latest commit:** derive on read with `git log -1 --oneline`.
- **In-flight uncommitted work:** none after the `.1.2.2` commit (pending defect leaf
  `PHASE-1-MAINT-1`: `run_pg_tests.sh`'s ephemeral PG data dir defaults to `/tmp` — §13).
- **Push cadence:** every ~300 commits (director, 2026-09-06); run full CI before each push.
- **Local dev deps now installed (2026-09-06):** `postgresql@16`, `cargo-deny`, `gitleaks`,
  `mdbook` — plus `jq` for the demo script; `codex` (0.153.4) for env-gated real runs.
- **Blockers:** none. Director-owned open items: license choice (`Cargo.toml` says
  `MIT OR Apache-2.0`, no `LICENSE` file); ADR-001 name clearance (repo stays private);
  second real adapter (Phase 1, `.4`).
- **Dating anomaly (flagged):** host/git clock = 2026-09-06; some records from the previous
  session are dated 2026-09-07 inside 2026-09-06 commits. New records use the machine date.
