# Revoking an authority records when

- Date: 2026-09-20
- Status: accepted
- Leaf: `SIGNOFF-REPAIR.11.24.1.1.2.1.1.1`
- Closes the deferral in
  `2026-09-20_a-retention-window-measures-time-in-the-state-it-retains.md`.

## The defect

`authority/revocation.rs` mutated with `UPDATE {table} SET status = 'revoked'
WHERE {key} = $1 AND tenant_id = $2` — the status and nothing else — and
`migrations/0004_authority.sql` declares both revocation targets with
`valid_from`, `expires_at` and `status`, no instant. **The row said that it had
been revoked and not when.**

⚠️ What is *not* claimed: that the audit trail was insufficient.
`administrative_effects` records every revocation with its actor, reason and
`effected_at`, and that is a complete audit record. The claim is narrower and
structural — the row the authorization path already loads does not carry the
instant, so everything deriving from the grant reads a fact it cannot date.

## What it cost, concretely

`SIGNOFF-REPAIR.11.24.1.1.2.1.1` had to leave §10.6's `revoked` inbox rows
permanently unprunable. A retention window measures time in the state being
retained; `expired` had the grant's own `expires_at` to measure and `revoked` had
nothing. The one available substitute, `node_inbox.decided_at`, records when the
*row* was created, so a command queued thirty days ago under a grant revoked this
morning would have vanished under a seven-day window on the day it entered the
terminal.

## The decision

`revoked_at TIMESTAMPTZ` on `authority_grants` and `enrollment_boundaries`,
written by **the same statement that changes the status**. A second statement
would be a second source of truth about one act and could leave a row revoked
with no instant if anything between them failed.

The value is `at` — the database time the transaction samples once after the
guard wait — which is also what stamps the effect record, so **the row and the
audit trail agree by construction rather than by two clocks happening to match**.
A control asserts that equality rather than assuming it.

## The column answers *when* and never *whether*

`status` remains the sole answer to whether authority stands. `grant_is_live` and
the `node_inbox_state` view are untouched by this migration and keep reading
`status`. A NULL `revoked_at` on a revoked row means **the instant is not
recoverable**, never that the row is live.

## The backfill is derived, and partial, and both are stated

The admission's effect record names the operation and its target
(`{"kind":"grant_revoke","grant_id":"…"}`) and carries the same `effected_at`, so
an already-revoked row can be dated from the act that revoked it. The join is
bound to the tenant as well as the id, because an effect cannot cite another
tenant's admission and this must not be the one place that forgets it, and only
an `applied` outcome dates a change — a `no_op` records a repeated revocation
whose instant belongs to the first one.

It is partial: `administrative_effects` arrived in `migrations/0058`, and a
revocation applied before it left no such record. Those rows keep
`revoked_at IS NULL` and stay retained.

## What the operator is told

`deleted_revoked` joins `deleted_delivered` and `deleted_expired` beside the
total, and the CLI summary names all three. Each class is aged by its own clock:
the acknowledgement, the grant's expiry, the revocation's instant.

## A clause kept although it is redundant, and the reason recorded

The prune's new statement carries `AND g.revoked_at IS NOT NULL` beside
`AND g.revoked_at <= $3`. It is **behaviourally redundant** — `NULL <= $3` is
already UNKNOWN, so three-valued logic alone retains an undatable row — and that
was measured rather than assumed: deleting the clause left all twelve arms green.

It stays because the guarantee is a **retention** one. A retention guarantee that
depends on the next editor remembering NULL semantics is one clause away from
deleting work nobody meant to delete.

## Evidence

- `node_work` **12 passed, 0 failed**; `node_inbox` 9/9, `node_channel` 40/40,
  `authority` 22/22, `authority_transaction` 16/16, `migration_upgrade` 5/5,
  `administrative_effects` 25/25, `cli_end_to_end` 5/5.
- The control runs three arms in order: the instant is recorded **and equals the
  effect record's**; a revocation from a moment ago is *not* removed by a one-day
  window; a row whose grant's instant is NULLed survives even a zero-second
  window and stays inspectable — then, with the instant restored and aged, it is
  removed and attributed to `deleted_revoked` with the other two classes at zero.
- Falsified in two places, each alone, restored byte-identical (`sha256`
  `d9252e6e…` and `070012b9…` matched): dropping `revoked_at = $3` from the status
  UPDATE fails *the revocation records WHEN it happened*; `g.revoked_at <= $3` →
  `<= now()` fails the window arm with
  `{"deleted":1,"deleted_revoked":1,"deleted_expired":0}`.

## What is still owed

`federation_agreements` has the same shape and is **not** repaired here:
`authority/federation_admin.rs` writes `SET status = 'revoked'` on a table that
already records `proposed_at` and `accepted_at` — two of its three lifecycle
instants — and leaves the third undated. It is a different verb in a different
transaction, so it carries its own leaf,
`SIGNOFF-REPAIR.11.24.1.1.2.1.1.1.1`.
