-- 0103_node_inbox_cursors.sql — SIGNOFF-REPAIR.4.3.2: a node's cursor
-- high-water mark is DURABLE, so it survives the node's inbox being pruned.
--
-- Every cursor writer computed `COALESCE(MAX(cursor), 0) + 1` over the rows
-- that happened to remain in `node_inbox`, and the three refusals (handshake,
-- poll, ack) compared the node's reported cursor with that same maximum. The
-- retention prune deletes delivered rows, the top of the ledger included, so
-- after a node acknowledged up to N and its delivered rows were pruned the
-- ledger forgot N: the node was refused `cursor_ahead` on every crossing, and
-- every row written afterwards took a cursor <= N that the node's `cursor > N`
-- replay never offered — hidden work, held against the backlog cap for ever.
--
-- One row per node, bumped in the statement that reads it (the allocator is
-- `node_channel::next_cursor_in_tx`; every writer goes through it). The row is
-- also the natural per-node lock: two concurrent allocations serialize on it.
--
-- Seeded from today's maxima so no live node moves. A node whose WHOLE ledger
-- was already pruned has no maximum to seed from and keeps the cursor it lost:
-- that loss predates this migration and nothing here can recover it.
CREATE TABLE node_inbox_cursors (
    node_id    TEXT   PRIMARY KEY,
    high_water BIGINT NOT NULL DEFAULT 0
);

INSERT INTO node_inbox_cursors (node_id, high_water)
SELECT node_id, MAX(cursor) FROM node_inbox GROUP BY node_id;
