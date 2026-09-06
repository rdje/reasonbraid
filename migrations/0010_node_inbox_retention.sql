-- 0010_node_inbox_retention.sql — durable inbox hardening (PHASE-1.2.3; backlog 14).
--
-- Quarantine is a database fact on the row, not a handler branch: a quarantined
-- command is never re-delivered (the replay/poll queries filter it), and the
-- reason is stored WITH the row so an operator can see why. Retention needs no
-- schema: delivered rows are deleted by an explicit, measured operator action
-- (`POST /v1/nodes/inbox/prune`), never by a background sweeper.
ALTER TABLE node_inbox
    ADD COLUMN quarantined_at     TIMESTAMPTZ,
    ADD COLUMN quarantine_reason  TEXT;
