-- 0087_node_ambiguous_attempts.sql — SIGNOFF-REPAIR.4.6.1.1: the server keeps
-- the ambiguous attempts a node reports, so an operator can list them.
--
-- ROADMAP §18.5 asks the admin surface to expose *ambiguous attempts and safe
-- resolution actions*. The handshake already decided each reported attempt —
-- `adjudicated` when the server holds a receipt for its operation,
-- `needs_adjudication` otherwise — but returned the directive to the node and
-- stored nothing, so no route could list what is still open
-- (`docs/decisions/2026-09-22_three-of-the-five-missing-operator-surfaces-have-no-stored-fact.md`).
--
-- One row per (node, attempt), written by every handshake that reports it:
--   * reported with no server receipt → the row is OPEN (inserted, or re-opened
--     if an earlier closure was followed by a new report);
--   * reported and the server holds a receipt → closed `adjudicated`, with the
--     receipt as the evidence;
--   * an open row the node no longer reports → closed `resolved_by_node`: the
--     node's journal left `outcome_unknown` on its own (a proven status lookup,
--     or a local reconciliation), and the server did not see how.
--
-- ⛔ The node's journal remains the authority on the attempt itself (§11.4:
-- *the journal owns locally observed facts*). This table records what the
-- server was TOLD and what it answered, which is exactly what an operator who
-- cannot reach the node needs to see.
--
-- The tenant is the node's (`nodes.tenant_id`) and is read by join rather than
-- copied, so it cannot disagree with the node row.

CREATE TABLE node_ambiguous_attempts (
    node_id            TEXT        NOT NULL REFERENCES nodes (node_id),
    attempt_id         TEXT        NOT NULL,
    operation_id       TEXT        NOT NULL,
    -- The `needs_adjudication` reason the handshake last returned for it.
    reason             TEXT        NOT NULL,
    first_reported_at  TIMESTAMPTZ NOT NULL,
    last_reported_at   TIMESTAMPTZ NOT NULL,
    -- Handshakes that reported it while it was open.
    report_count       BIGINT      NOT NULL CHECK (report_count >= 1),
    closed_at          TIMESTAMPTZ,
    closure            TEXT        CHECK (closure IN ('adjudicated', 'resolved_by_node')),
    -- For `adjudicated`: the receipt the server holds. NULL otherwise.
    closure_evidence   TEXT,
    PRIMARY KEY (node_id, attempt_id),
    CHECK (last_reported_at >= first_reported_at),
    -- A row is open or closed as a whole: never a closure without its instant.
    CHECK ((closed_at IS NULL) = (closure IS NULL)),
    CHECK (closure_evidence IS NULL OR closure = 'adjudicated')
);

COMMENT ON TABLE node_ambiguous_attempts IS
    'Ambiguous attempts reported by nodes at the handshake, with the server''s '
    'answer. Open rows (closed_at IS NULL) are what GET '
    '/v1/admin/nodes/ambiguous-attempts lists; closed rows keep how each ended.';

CREATE INDEX node_ambiguous_attempts_open_idx
    ON node_ambiguous_attempts (node_id) WHERE closed_at IS NULL;
