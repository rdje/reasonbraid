-- 0016_spend_breakers.sql — spend circuit breakers (PHASE-2.3.2; backlog 23).
--
-- A per-tenant SPEND LATCH, independent of the per-thread budget ceilings: the
-- operator ARMS a breaker with a spend threshold (BudgetDimensions). The
-- reservation path checks it before issuing anything: while tripped (or when
-- the tenant's recorded spend — settled usage + active holds — crosses the
-- threshold at the next attempt) NEW dispatch reservations are refused with a
-- typed reason, riding the existing budget-denial flow. The operator RESETS
-- the latch (tripped_at cleared); the threshold persists until re-armed.

CREATE TABLE spend_breakers (
    tenant_id      TEXT PRIMARY KEY REFERENCES tenants (tenant_id),
    threshold      JSONB NOT NULL,             -- BudgetDimensions (the declared ceiling)
    tripped_at     TIMESTAMPTZ,                -- NULL = armed but not tripped
    tripped_reason TEXT,
    armed_at       TIMESTAMPTZ NOT NULL DEFAULT now()
);
