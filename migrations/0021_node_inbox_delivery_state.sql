-- 0021_node_inbox_delivery_state.sql — PHASE-3.5.1: the §10.6 delivery ladder
-- as ONE DERIVED truth — a view, never a parallel column. The shipped
-- columns (the cursor, `acknowledged_at`, `quarantined_at`, the work-result
-- receipts) already carry the transitions; the view names them:
--
--   queued → acknowledged → consumed
--                ↘ dead_lettered (the quarantine IS the dead letter)
--
-- The `expired`/`revoked` terminals ride the retention/prune machinery and
-- the revocation re-delivery semantics (named in the leaf, not derivable
-- from the shipped columns). Transport receipt ≠ read: the ack is the
-- explicit per-event-type contract, now VISIBLE per row.

CREATE OR REPLACE VIEW node_inbox_state AS
SELECT i.*,
       CASE
         WHEN i.quarantined_at IS NOT NULL THEN 'dead_lettered'
         WHEN i.acknowledged_at IS NOT NULL
              AND EXISTS (SELECT 1 FROM node_events e
                          WHERE e.operation_id = i.command_id
                            AND e.payload->>'kind' = 'work_result') THEN 'consumed'
         WHEN i.acknowledged_at IS NOT NULL THEN 'acknowledged'
         ELSE 'queued'
       END AS delivery_state
FROM node_inbox i;
