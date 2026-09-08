-- 0046_rls_tenant_isolation — the RLS defense-in-depth (`PHASE-7.1.3.1`, ADR-034,
-- `docs/decisions/2026-09-08_rls-tenant-claim.md`): the SECOND layer over the
-- tenant-scoped keys. The claim is the transaction-local `app.tenant_id` GUC
-- (set with `set_config(..., is_local := true)` as the transaction's first
-- statement); fail-closed — the unset claim reads NULL and matches no row.
--
-- The enablement set matches the WIRED set: the COMMAND CORE (the aggregate
-- head, the audit trail, the replay protection). The outbox stays exempt (the
-- worker's cross-tenant queue — a claim would blind it); the remaining
-- tenant-keyed tables stay on the application layer (the named deferral).
-- FORCE: the future non-superuser app role OWNS these tables, and owners
-- bypass RLS unless forced. The dev profile's superuser bypasses regardless —
-- the probe-role proof (`tests/rls.rs`) measures the layer as it will bind.

ALTER TABLE aggregate_state ENABLE ROW LEVEL SECURITY;
ALTER TABLE aggregate_state FORCE ROW LEVEL SECURITY;
CREATE POLICY aggregate_state_tenant_claim ON aggregate_state
    USING (tenant_id = current_setting('app.tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.tenant_id', true));

ALTER TABLE event_log ENABLE ROW LEVEL SECURITY;
ALTER TABLE event_log FORCE ROW LEVEL SECURITY;
CREATE POLICY event_log_tenant_claim ON event_log
    USING (tenant_id = current_setting('app.tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.tenant_id', true));

ALTER TABLE idempotency ENABLE ROW LEVEL SECURITY;
ALTER TABLE idempotency FORCE ROW LEVEL SECURITY;
CREATE POLICY idempotency_tenant_claim ON idempotency
    USING (tenant_id = current_setting('app.tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.tenant_id', true));
