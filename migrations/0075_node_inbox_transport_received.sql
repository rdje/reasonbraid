-- 0075_node_inbox_transport_received.sql — SIGNOFF-REPAIR.11.24.1.1:
-- the derived delivery state stops calling a TRANSPORT RECEIPT an
-- ACKNOWLEDGEMENT, because ROADMAP.md §10.6 distinguishes them by name and
-- this view used one of its words for the other one's fact.
--
-- §10.6, in full:
--
--     queued → offered → transport_received → acknowledged → consumed
--                       ↘ expired / revoked / dead_lettered
--
--     Transport receipt does not mean an agent read or acted.
--     Acknowledgement semantics are explicit per event type.
--
-- What `acknowledged_at` actually records, read from the producer rather than
-- from its name: the node's reconcile loop journals every inbound command
-- (`record_inbound_command` + `ensure_operation`, step 4) and only then calls
-- `channel.acknowledge(max_cursor)` (step 7). So the column means *the node
-- process durably holds this command* — a durable transport receipt. The agent
-- has not read it and has not acted; acting is what writes the `work_result`
-- event this view already derives `consumed` from.
--
-- ⛔ AND THE CURSOR ACK IS NOT PER-EVENT-TYPE. It covers every row up to a
-- cursor, whatever the event types are, so it cannot be §10.6's
-- `acknowledged`, whose whole definition is that its semantics are explicit
-- per event type. Nothing in the shipped channel produces that state.
--
-- The derived ladder is therefore a true SUBSET of §10.6 with every name
-- meaning what §10.6 says:
--
--   queued → transport_received → consumed
--                       ↘ dead_lettered (the quarantine IS the dead letter)
--
-- ⚠️ THIS IS A WIRE-VISIBLE VOCABULARY CHANGE. `delivery_state` is returned by
-- `GET /v1/nodes/inbox` and by the MCP `list_inbox` tool. A client matching the
-- string `acknowledged` sees `transport_received` after this migration. The
-- break is taken deliberately: a published vocabulary that reuses a
-- specification's word for a different fact is precisely what the §10.6
-- sentence above exists to prevent, and this project is pre-1.0 with the
-- surface on the development profile.
--
-- ⭐ `offered`, `acknowledged`, `expired` and `revoked` remain underived, each
-- with its disposition recorded at `SIGNOFF-REPAIR.11.24.1.1` and a leaf where
-- work is owed. A state with no producer is not added to this CASE.

CREATE OR REPLACE VIEW node_inbox_state AS
SELECT i.*,
       CASE
         WHEN i.quarantined_at IS NOT NULL THEN 'dead_lettered'
         WHEN i.acknowledged_at IS NOT NULL
              AND EXISTS (SELECT 1 FROM node_events e
                          WHERE e.operation_id = i.command_id
                            AND e.payload->>'kind' = 'work_result') THEN 'consumed'
         WHEN i.acknowledged_at IS NOT NULL THEN 'transport_received'
         ELSE 'queued'
       END AS delivery_state
FROM node_inbox i;
