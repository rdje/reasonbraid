-- 0002_node_channel.sql — WP3 outbound channel state (PHASE-0.3.2).
--
-- The node's own acknowledgement cursor: the highest server command cursor the node has
-- durably recorded (commands land in `commands.server_cursor`). It is reported in the
-- reconnect handshake (§17.4 step 2); the server replays everything after it and the
-- journal deduplicates by command id — so a crash between journaling the replay and
-- recording this cursor re-delivers and dedupes, never doubles.

CREATE TABLE channel_state (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

PRAGMA user_version = 2;
