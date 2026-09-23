-- 0105_node_inbox_consumed_by_command.sql — SIGNOFF-REPAIR.4.4.9: a real node's
-- delivered result reads `consumed`.
--
-- The `consumed` rung has joined `node_events.operation_id = node_inbox.command_id`
-- since `0021`, and every redefinition (`0075`, `0076`, `0078`, `0104`) carried
-- the join over. But the operation id is the NODE's own (`op_<uuid>`, minted by
-- its journal) and never the command id, so no real node's result ever matched:
-- a delivered row read `transport_received` for good. The capacity count reads
-- the same rung, so every finished item stayed IN FLIGHT and a node that had
-- completed its declared concurrency was handed nothing more until the retention
-- prune removed the rows. The controls that asserted `consumed` inserted
-- synthetic events whose operation id WAS the command id, a shape no node sends.
--
-- The result names its command in its payload, which is what the fold itself
-- reads (`api::node_result_command`) and what the dead-letter guard reads
-- (`SIGNOFF-REPAIR.4.4.8`). The rung reads it too: THIS node's `work_result`
-- naming THIS command. Same columns, so the dependent `node_presence` (0079)
-- is untouched.
CREATE OR REPLACE VIEW node_inbox_state AS
SELECT i.*,
       CASE
         WHEN i.quarantined_at IS NOT NULL THEN 'dead_lettered'
         WHEN i.acknowledged_at IS NOT NULL
              AND EXISTS (SELECT 1 FROM node_events e
                          WHERE e.node_id = i.node_id
                            AND e.payload->>'kind' = 'work_result'
                            AND e.payload->>'command_id' = i.command_id) THEN 'consumed'
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
