-- 0050_mcp_listen_state.sql — the MCP listen-stream DURABLE state
-- (`PHASE-8.3.4`, ADR-024, §9.6): the listen stream is the EPHEMERAL
-- transport state; the DURABLE state — the subscription, the last
-- accepted ReasonBraid cursor, the delivery ids, the deduplication —
-- stays in REASONBRAID. The reconnect resumes from the OWN cursor and
-- surfaces the possible-gap when the upstream offers no replay — the
-- MCP continuation is never advertised as stronger than the upstream
-- can prove.

CREATE TABLE mcp_listen_state (
    tenant_id       TEXT   NOT NULL REFERENCES tenants (tenant_id),
    subscription_id TEXT   NOT NULL,
    last_cursor     BIGINT NOT NULL DEFAULT 0,
    last_delivery   TEXT,
    dedup_window    JSONB  NOT NULL DEFAULT '[]'::jsonb,
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (tenant_id, subscription_id)
);
