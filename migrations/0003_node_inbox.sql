-- 0003_node_inbox.sql — the WP3 (.3.2) node channel's server-side durable state.
--
-- The control plane owns desired commands and delivery attempts (§17.1). `node_inbox` is
-- the per-node delivery ledger: a monotonic per-node cursor, the command payload, and the
-- node's acknowledgement state. `node_events` records receipts of node-emitted events,
-- deduplicated by the NODE-assigned event id.
--
-- Replay is computed from the cursor the node REPORTS in its handshake (the node is
-- authoritative for what it durably holds — §17.4 step 2): the server replays every
-- inbox row with `cursor > reported`, and the node's journal deduplicates by command id.
-- A node that reports a cursor AHEAD of this server's ledger is refused (its journal saw
-- commands this server cannot reproduce — a journal-lost-class anomaly, §11.4).

CREATE TABLE node_inbox (
    node_id         TEXT        NOT NULL,
    cursor          BIGINT      NOT NULL,  -- per-node monotonic server sequence
    command_id      TEXT        NOT NULL,
    tenant_id       TEXT        NOT NULL,
    thread_id       TEXT        NOT NULL,
    payload         JSONB       NOT NULL,
    acknowledged_at TIMESTAMPTZ,           -- set when the node acknowledges this cursor
    PRIMARY KEY (node_id, cursor),
    UNIQUE (node_id, command_id)           -- one command appears once in a node's ledger
);

-- Receipts of node-emitted events. The event_id primary key is the dedupe key: the node
-- re-emits pending events with their ORIGINAL ids after a crash (§17.4 step 5), and the
-- server's insert-on-conflict turns a redelivery into a duplicate receipt, never a second
-- event.
CREATE TABLE node_events (
    event_id     TEXT        PRIMARY KEY,
    node_id      TEXT        NOT NULL,
    operation_id TEXT        NOT NULL,
    payload      JSONB       NOT NULL,
    received_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- The reconciliation lookups the handshake performs (per operation).
CREATE INDEX node_events_operation_idx ON node_events (operation_id);
