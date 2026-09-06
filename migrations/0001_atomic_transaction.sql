-- 0001_atomic_transaction.sql — the WP2 (.2.1) durability tables.
--
-- One transaction writes all four of these (or none). The schema is deliberately minimal:
-- the leased-worker columns (lease token, lease_until, fencing) belong to .2.2, which will
-- add them in a later migration rather than guessing their shape now.

-- Current state of one aggregate (a thread, today). Upserted on every accepted command.
CREATE TABLE aggregate_state (
    tenant_id         TEXT   NOT NULL,
    aggregate_id      TEXT   NOT NULL,
    aggregate_type    TEXT   NOT NULL,
    aggregate_version BIGINT NOT NULL,
    state             JSONB  NOT NULL,
    PRIMARY KEY (tenant_id, aggregate_id)
);

-- Append-only ordered event log. The (tenant, aggregate, version) triple is unique, so a
-- version can never be silently reused for the same aggregate.
CREATE TABLE event_log (
    event_id          TEXT        NOT NULL,
    tenant_id         TEXT        NOT NULL,
    aggregate_id      TEXT        NOT NULL,
    aggregate_version BIGINT      NOT NULL,
    event_type        TEXT        NOT NULL,
    body              JSONB       NOT NULL,
    committed_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (event_id),
    UNIQUE (tenant_id, aggregate_id, aggregate_version)
);

-- Idempotency record: one row per (tenant, key). The PK is the serialization point that
-- turns a transport redelivery into a replay (same hash) or a conflict (different hash).
CREATE TABLE idempotency (
    tenant_id       TEXT        NOT NULL,
    idempotency_key TEXT        NOT NULL,
    request_hash    TEXT        NOT NULL,
    response_result JSONB       NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (tenant_id, idempotency_key)
);

-- Transactional outbox. The FK to event_log means an outbox item can only exist if its
-- event is already durable — an outbox row *implies* its event committed.
CREATE TABLE outbox (
    outbox_id  BIGSERIAL   PRIMARY KEY,
    tenant_id  TEXT        NOT NULL,
    event_id   TEXT        NOT NULL REFERENCES event_log (event_id),
    dispatched BOOLEAN     NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
