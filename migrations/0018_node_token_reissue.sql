-- 0018_node_token_reissue.sql — the PHASE-2.7.2 replacement ritual: a lost node's
-- operator issues a SECOND enrollment token, but 0008's unconditional UNIQUE on
-- `node_id` made the FIRST token permanent — the table's own comment always said
-- "one UNUSED token per node"; the index did not say it. Drop the unconditional
-- constraint and pin the intent: at most one token with `used_at IS NULL` per node.

ALTER TABLE node_enrollment_tokens DROP CONSTRAINT node_enrollment_tokens_node_id_key;
CREATE UNIQUE INDEX node_enrollment_tokens_one_unused_idx
    ON node_enrollment_tokens (node_id) WHERE used_at IS NULL;
