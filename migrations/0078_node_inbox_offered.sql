-- 0078_node_inbox_offered.sql — SIGNOFF-REPAIR.11.24.1.1.1:
-- §10.6's `offered` had no producer, so a row the server had handed to a
-- transport was indistinguishable from one it had never tried to deliver.
--
-- ⛔ THE GAP. `ROADMAP.md` §10.6 states the ladder as
--     queued → offered → transport_received → acknowledged → consumed
--                       ↘ expired / revoked / dead_lettered
-- and `migrations/0076` derives every rung of it except two. `offered` is the
-- difference between a QUIET node and one LOSING ITS RESPONSES: rows that never
-- leave `queued` say the node is not polling, and rows that reach `offered` and
-- stop there say it is polling and the responses are not arriving. Without the
-- rung both read `queued`, and the two diagnoses are the same row.
--
-- ⭐ ONE COLUMN, AND THE COUNT IS REFUSED. `SIGNOFF-REPAIR.11.24.1.1.1` asked
-- for "`offered_at`, or a count". The count is NOT added: the question the state
-- exists to answer — quiet versus losing responses — is answered by the STATE,
-- and `a column is not missing until something needs it` is this tree's own
-- promoted rule (`.11.24.1.1.2.1.1.1.1`). ⚠️ Trigger, written down so the
-- refusal is safe to act on later: a reader that must distinguish ONE offer from
-- many — §10.7's storm controls are the likely one.
--
-- ⭐ WRITE-ONCE, AND THAT IS DERIVED RATHER THAN PREFERRED. `offered_at` records
-- when the row ENTERED the state, not when it was last re-offered.
-- `SIGNOFF-REPAIR.11.24.1.1.2.1.1` established that a window measures time in
-- the state being retained; an instant that a re-offer bumps silently resets
-- every age measured from it, so a node polling in a loop would keep its oldest
-- outstanding work looking new. The `offered_at IS NULL` guard in the writing
-- statement is what makes it write-once.
--
-- ⛔ NO BACKFILL, AND THE REFUSAL IS DERIVED, NOT AN OMISSION.
--   * A row already at `transport_received` or above was certainly offered, and
--     those states OUTRANK `offered` in the view below — so dating it would
--     change nothing observable, while dating it from `acknowledged_at` (the
--     only instant available) would record an offer at an instant that is
--     provably later than the real one.
--   * A row still at `queued` is exactly the case where the answer would matter,
--     and nothing in the schema records whether it was ever offered.
-- So the backfill is unnecessary where it is derivable and impossible where it
-- would be informative. A NULL `offered_at` means *not offered since this
-- migration*, never *never offered*.

ALTER TABLE node_inbox ADD COLUMN offered_at TIMESTAMPTZ;

-- ⛔ DROP AND CREATE, NOT `CREATE OR REPLACE`, AND THE SUITE PROVED IT RATHER
-- THAN A REVIEWER GUESSING IT. This view is `SELECT i.*, … AS delivery_state`,
-- and `*` expanded positionally when the view was created, so `delivery_state`
-- is the LAST column. The `ALTER TABLE` above inserts `offered_at` into that
-- expansion ahead of it, and `CREATE OR REPLACE VIEW` may only APPEND columns:
-- PostgreSQL refuses with `42P16 cannot change name of view column
-- "delivery_state" to "offered_at"`, and every suite that applies migrations
-- fails at once. Nothing else in the schema references this view, so dropping
-- it is local.
--
-- ⚠️ THE STANDING HAZARD, written here because this is the file a person edits
-- next: while `node_inbox_state` selects `i.*`, EVERY future
-- `ALTER TABLE node_inbox ADD COLUMN` must drop and recreate it. Measured
-- population, not a guess: the schema has two views, and this is the only one
-- that expands a star (`node_presence` names its columns). One instance is not
-- a rule (`.11.6`), so no gate is proposed; the trigger for one is a second
-- table gaining a star-expanding view.
DROP VIEW node_inbox_state;

-- The rung sits between `transport_received` and `queued`, which is §10.6's own
-- order. ⚠️ The terminals keep outranking it, unchanged from `0076`: a row that
-- was offered and then had its authority withdrawn reads `revoked`, because the
-- act is the operative fact and the more informative answer to an operator
-- asking why a command was never delivered.
CREATE VIEW node_inbox_state AS
SELECT i.*,
       CASE
         WHEN i.quarantined_at IS NOT NULL THEN 'dead_lettered'
         WHEN i.acknowledged_at IS NOT NULL
              AND EXISTS (SELECT 1 FROM node_events e
                          WHERE e.operation_id = i.command_id
                            AND e.payload->>'kind' = 'work_result') THEN 'consumed'
         WHEN EXISTS (SELECT 1 FROM authorization_records r
                        JOIN authority_grants g ON g.grant_id = r.grant_id
                       WHERE r.record_id = i.authz_ref
                         AND g.status <> 'active') THEN 'revoked'
         WHEN EXISTS (SELECT 1 FROM authorization_records r
                        JOIN authority_grants g ON g.grant_id = r.grant_id
                       WHERE r.record_id = i.authz_ref
                         AND g.expires_at <= now()) THEN 'expired'
         WHEN i.acknowledged_at IS NOT NULL THEN 'transport_received'
         WHEN i.offered_at IS NOT NULL THEN 'offered'
         ELSE 'queued'
       END AS delivery_state
FROM node_inbox i;
