-- 0102_node_events_keyed_per_node.sql — SIGNOFF-REPAIR.4.3.1: a node's event
-- receipts are deduplicated PER NODE, and the reconciliation lookup is bound to
-- the node.
--
-- Event ids are node-chosen strings the server never namespaces (§17.4 step 5:
-- the node re-emits pending events with their ORIGINAL ids), so two nodes can
-- legitimately emit the same id for two different events. With `event_id` as
-- the whole key (0003), a node recording an id first made another node's own
-- submission of it a duplicate — `accepted = false`, no receipt, and the events
-- handler skipped that node's fold. The dedup the key serves is per node by
-- definition, so the key becomes (node_id, event_id).
--
-- ADDITIVE: stored receipts are not rewritten. Every row already carries its
-- node_id, and the old key is a strict subset of the new one, so the promotion
-- cannot conflict.
ALTER TABLE node_events DROP CONSTRAINT node_events_pkey;
ALTER TABLE node_events ADD PRIMARY KEY (node_id, event_id);

-- The handshake's reconciliation lookups (directives, known_events) now ask
-- for THIS node's receipt of an operation: `WHERE node_id = $1 AND
-- operation_id = $2`. The operation-only index from 0003 stays for the
-- `node_inbox_state` view's `consumed` rung, which still joins on the
-- operation alone until SIGNOFF-REPAIR.4.3.3 binds it to the node.
CREATE INDEX node_events_node_operation_idx ON node_events (node_id, operation_id);
