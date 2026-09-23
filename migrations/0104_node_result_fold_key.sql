-- 0104_node_result_fold_key.sql — SIGNOFF-REPAIR.4.3.3: a result's idempotency
-- and the `consumed` rung are bound to the node.
--
-- Command ids are unique PER NODE (`UNIQUE (node_id, command_id)`, 0003), so
-- two nodes in one tenant can hold one id — two work items that share a name.
-- Two surfaces treated the name as the identity:
--
--   * the result fold claimed idempotency on (tenant_id, command_id), so the
--     second node's result met the first node's claim (a different request
--     hash: a stored rejection) and was lost;
--   * `node_inbox_state`'s `consumed` rung joined the work-result receipt on the
--     operation id alone, so another node's result read this node's row consumed.
--
-- (1) The rung joins the receipt of THIS node. Same columns, so the dependent
--     `node_presence` (0079) is untouched.
CREATE OR REPLACE VIEW node_inbox_state AS
SELECT i.*,
       CASE
         WHEN i.quarantined_at IS NOT NULL THEN 'dead_lettered'
         WHEN i.acknowledged_at IS NOT NULL
              AND EXISTS (SELECT 1 FROM node_events e
                          WHERE e.node_id = i.node_id
                            AND e.operation_id = i.command_id
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

-- (2) The fold's key becomes `<node_id>:<command_id>` (`api::node_result_fold_key`).
--     Stored fold rows are re-keyed where the work item is unambiguous: the key
--     equals the command id of a thread-work inbox row held by exactly ONE node
--     in the tenant. Left as they are, and stated: a key whose id two nodes hold
--     (nothing says whose fold it was), or that no row holds any more (pruned);
--     a later re-emission of such a result under a NEW event id would fold
--     again. A caller-chosen CLI key equal to a work command id would be
--     re-keyed too — a shape no caller here produces.
UPDATE idempotency i
   SET idempotency_key = n.node_id || ':' || i.idempotency_key
  FROM node_inbox n
 WHERE n.tenant_id = i.tenant_id
   AND n.command_id = i.idempotency_key
   AND n.payload->>'kind' IN ('contribute', 'revise')
   AND NOT EXISTS (SELECT 1 FROM node_inbox m
                    WHERE m.tenant_id = n.tenant_id
                      AND m.command_id = n.command_id
                      AND m.node_id <> n.node_id);
