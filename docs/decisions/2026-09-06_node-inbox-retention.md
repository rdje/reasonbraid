# 2026-09-06_node-inbox-retention.md

## Context

`PHASE-1.2.3` (backlog 14's remainder) landed durable inbox hardening: a
quarantine status (with reason) that the replay/poll paths skip, and a
retention window for delivered rows cleaned by an explicit, measured operator
action. Filtered delivery by eligibility stays with Phase 3's directory.

## Decision

- **Quarantine is a database fact on the row, not a handler branch.** Two
  nullable columns on `node_inbox` (`quarantined_at`, `quarantine_reason`):
  the replay/poll queries filter `quarantined_at IS NULL`, so a quarantined
  command is never re-delivered — whatever cursor the node reports, whatever
  path (handshake replay or live poll) serves it. The reason is stored WITH
  the row: the skip is explainable to an operator, never silent.
- **Quarantine controls DELIVERY, not result application.** A node that
  received a command before it was quarantined may still emit a result; the
  domain applies it (`load_command` stays unfiltered). Quarantine is a
  delivery-control fact, not a result-suppression fact.
- **Retention cleanup is an explicit, measured operator action** —
  `POST /v1/nodes/inbox/prune` with a `min_age_seconds` window deletes only
  DELIVERED rows (`acknowledged_at IS NOT NULL AND acknowledged_at <= cutoff`),
  and the before-count, delete, and after-count ride ONE transaction so the
  response is the operator's receipt for exactly what was removed. No
  background sweeper; no dry-run toggle (the measured before/after IS the
  safety).
- **The operator surface is the control API, `tenant_admin`-audited** — the
  same gate as token issuance (the authorization record is the audit; allowed
  and denied both leave one). The node channel surface only behaves
  differently (skips quarantined rows). Inspection is `GET /v1/nodes/inbox`
  (delivery + quarantine facts per row, WITH the payload — judging a
  quarantine needs the content).
- **Typed refusals:** unknown command 400, re-quarantine 409 (like the token
  re-issue — one quarantine per row, ever), empty reason 400 (a quarantine
  without a reason is a silent skip), negative window 400, non-admin 403.

## Consequences

- `node_inbox` grew two nullable columns (migration 0010) — additive; no purge
  list changed, no existing query shape changed except the replay filter.
- Cursor semantics keep holes honest: a node that never saw a quarantined row
  simply has a gap in its ledger; acknowledging `max(cursor)` marks the
  quarantined row terminal too, so the hole never reopens.
- The CLI gains `rb node quarantine|inbox|prune` (the `.1.2.1` verb family).
- New `tests/node_inbox.rs` (3 live-PG tests) registered in
  `scripts/run_pg_tests.sh`.

answers:

- **A hole in a cursor ledger is fine when acknowledgement marks it terminal.**
  Quarantine creates cursor gaps; the ack path (`cursor <= ack_cursor`)
  deliberately covers quarantined rows, so a node that skipped one converges
  instead of re-requesting the gap forever.
- **Test closures that build async requests must own their captures (again).**
  The `.1.2.2` record's lesson re-applied immediately: closures returning
  `async move` futures must take OWNED parameters (the borrowed `&str` broke
  the borrow checker with an unnameable lifetime). Recorded twice, applied
  once, now a habit — no new machinery.
- **Reuse the denial-row AUTHORIZATION surface instead of a new audit table.**
  Quarantine and prune ride `authorize()` (tenant_admin): the audit record
  comes from the authority engine, so these operator actions are auditable
  without schema — the `.1.2.1` enrollment needed its own table only because
  the node-side enroll carries no principal header.
