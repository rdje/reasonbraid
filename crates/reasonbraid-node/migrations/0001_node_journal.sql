-- 0001_node_journal.sql — the WP3 node journal schema (PHASE-0.3.1).
--
-- Node-local durable state (ROADMAP.md §17.1 / §17.4): received commands, local
-- operations, provider attempts with a before/after transition ledger, and outgoing
-- events with acknowledgement state. The node persists a fact BEFORE advancing the
-- corresponding boundary (§17.4): `record_dispatch` commits before the adapter is
-- invoked, so a crash after a possible dispatch is recoverable as `outcome_unknown`
-- and a crash before it is `safe_to_redeliver`.
--
-- WAL mode and synchronous=FULL are applied by the journal at open and recorded in
-- `journal_meta` (pragmas are per-connection; the meta row makes the durability
-- profile inspectable by the read-only CLI). This migration is embedded in the node
-- crate and ships with the node binary — it never runs against the control-plane
-- PostgreSQL schema in the repository-root `migrations/`.
--
-- All timestamps are RFC 3339 TEXT written by the caller's clock (deterministic
-- tests; no dependence on host/database clock agreement).

CREATE TABLE journal_meta (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

INSERT INTO journal_meta (key, value) VALUES
    ('durability_journal_mode', 'wal'),
    ('durability_synchronous', 'FULL'),
    ('created_by', 'reasonbraid-node 0.1.0');

-- A command received from the control plane. The primary key deduplicates transport
-- redelivery: the same command id is recorded exactly once (`record_command`).
CREATE TABLE commands (
    command_id    TEXT PRIMARY KEY,
    tenant_id     TEXT NOT NULL,
    thread_id     TEXT NOT NULL,
    payload       TEXT NOT NULL,  -- the server command body (JSON)
    authz_ref     TEXT,           -- §17.4 authorization snapshot/reference; WP5 fills it
    server_cursor TEXT NOT NULL,  -- the server sequence the command came from (reconnect evidence, .3.2)
    received_at   TEXT NOT NULL
);

-- The local execution unit. `command_id` is UNIQUE, so a duplicated command never
-- creates a second local operation (`ensure_operation`).
CREATE TABLE operations (
    operation_id TEXT PRIMARY KEY,
    command_id   TEXT NOT NULL UNIQUE REFERENCES commands (command_id),
    created_at   TEXT NOT NULL
);

-- One supervised provider attempt. `status` is a ProviderAttemptState wire name and
-- every change is validated against reasonbraid-core's deterministic machine.
-- `provider_request_id` is recorded at the dispatch boundary (the proof handle a
-- status lookup needs); `evidence` carries JSON proof of a terminal result or the
-- reason an attempt became ambiguous.
CREATE TABLE attempts (
    attempt_id          TEXT PRIMARY KEY,  -- ProviderAttemptId wire form (`patt_…`)
    operation_id        TEXT NOT NULL REFERENCES operations (operation_id),
    status              TEXT NOT NULL,
    provider_request_id TEXT,
    evidence            TEXT,
    created_at          TEXT NOT NULL,
    updated_at          TEXT NOT NULL
);

CREATE INDEX attempts_status_idx ON attempts (status);

-- The before/after boundary ledger (§11.4: "records state before and after every
-- irreversible boundary"): one row per status change, in the same transaction.
CREATE TABLE attempt_transitions (
    attempt_id  TEXT NOT NULL REFERENCES attempts (attempt_id),
    seq         INTEGER NOT NULL,  -- per-attempt monotonic boundary sequence
    from_status TEXT NOT NULL,
    to_status   TEXT NOT NULL,
    at          TEXT NOT NULL,
    PRIMARY KEY (attempt_id, seq)
);

-- An event the node emitted toward the control plane, with its acknowledgement
-- state: `acked_at` stays NULL until the server acknowledges, and `ack_cursor`
-- records the server cursor at acknowledgement (reconnect evidence, .3.2).
CREATE TABLE outgoing_events (
    event_id     TEXT PRIMARY KEY,
    operation_id TEXT NOT NULL REFERENCES operations (operation_id),
    payload      TEXT NOT NULL,  -- the event body (JSON)
    emitted_at   TEXT NOT NULL,
    acked_at     TEXT,
    ack_cursor   TEXT
);

-- Schema version, mirrored in `PRAGMA user_version` so the CLI can show it without
-- knowing sqlx's migration bookkeeping. New migrations must bump both.
PRAGMA user_version = 1;
