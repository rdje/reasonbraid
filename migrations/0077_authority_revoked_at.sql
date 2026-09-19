-- 0077_authority_revoked_at.sql — SIGNOFF-REPAIR.11.24.1.1.2.1.1.1:
-- revoking an authority recorded THAT it happened and not WHEN.
--
-- ⛔ THE DEFECT. `authority/revocation.rs` mutates with
-- `UPDATE {table} SET status = 'revoked' WHERE {key} = $1 AND tenant_id = $2` —
-- the status and nothing else — and `migrations/0004_authority.sql` declares
-- both revocation targets with `valid_from`, `expires_at` and `status`, no
-- instant. So the GRANT could say it had been revoked and not when, and
-- anything deriving from the grant could not ask.
--
-- ⭐ WHAT THAT COST, concretely rather than in principle.
-- `SIGNOFF-REPAIR.11.24.1.1.2.1.1` had to leave §10.6's `revoked` inbox rows
-- permanently unprunable: a retention window measures time in the state being
-- retained, `expired` had the grant's own `expires_at` to measure, and `revoked`
-- had nothing. The one available substitute, `node_inbox.decided_at`, records
-- when the ROW was created — so a command queued thirty days ago under a grant
-- revoked this morning would have vanished under a seven-day window on the day
-- it entered the terminal.
--
-- ⚠️ WHAT IS NOT CLAIMED: that the audit trail was insufficient.
-- `administrative_effects` records every revocation with its actor, reason and
-- `effected_at`, and that is a complete audit record. The claim is narrower and
-- structural — the ROW the authorization path already loads does not carry the
-- instant, so `grant_is_live` and the delivery view read a fact they cannot
-- date.
--
-- ⭐ THE BACKFILL IS DERIVABLE, AND IT IS DERIVED RATHER THAN GUESSED. The
-- admission's effect record names the operation and its target
-- (`{"kind":"grant_revoke","grant_id":"…"}`) and carries the same `effected_at`
-- the transaction stamps everything else with, so an already-revoked row can be
-- dated from the act that revoked it.
--
-- ⛔ AND IT IS PARTIAL, WHICH IS STATED RATHER THAN HIDDEN.
-- `administrative_effects` arrived in `migrations/0058`; a revocation applied
-- before it left no such record, and nothing else in the schema dates one. Those
-- rows keep `revoked_at IS NULL`.
--
-- ⛔ SO THE COLUMN'S NULL MEANS ONE THING AND NOT THE OTHER: *this revocation's
-- instant is not recoverable*, NEVER *this authority is not revoked*. `status`
-- remains the sole answer to WHETHER, exactly as it is today; `revoked_at`
-- answers only WHEN. Every predicate that asks whether authority stands —
-- `authority::grant_is_live` and the `node_inbox_state` view — is unchanged by
-- this migration and keeps reading `status`.

ALTER TABLE authority_grants        ADD COLUMN revoked_at TIMESTAMPTZ;
ALTER TABLE enrollment_boundaries   ADD COLUMN revoked_at TIMESTAMPTZ;

-- The derived backfill. Bound to the tenant as well as the target id, because an
-- effect record cannot cite another tenant's admission and this join must not be
-- the one place that forgets it. Only an `applied` outcome dates a change: a
-- `no_op` records a repeated revocation whose instant belongs to the first one,
-- and a `refused` records no change at all.
UPDATE authority_grants g
   SET revoked_at = e.effected_at
  FROM administrative_effects e
 WHERE g.status = 'revoked'
   AND g.revoked_at IS NULL
   AND e.tenant_id = g.tenant_id
   AND e.operation->>'kind' = 'grant_revoke'
   AND e.operation->>'grant_id' = g.grant_id
   AND e.outcome->>'kind' = 'applied';

UPDATE enrollment_boundaries b
   SET revoked_at = e.effected_at
  FROM administrative_effects e
 WHERE b.status = 'revoked'
   AND b.revoked_at IS NULL
   AND e.tenant_id = b.tenant_id
   AND e.operation->>'kind' = 'boundary_revoke'
   AND e.operation->>'boundary_id' = b.boundary_id
   AND e.outcome->>'kind' = 'applied';
