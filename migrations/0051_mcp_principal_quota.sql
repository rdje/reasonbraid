-- 0051_mcp_principal_quota.sql — the MCP write-half's per-principal quota
-- binding (`PHASE-8.3.5.1`, ADR-024 §9.6, the `.1.3.2` named deferral's
-- trigger — the MCP write gate FIRES it): the `principal` scope of the 0047
-- machinery was shipped unbound (the vocabulary + the fail-closed check, no
-- row creator, no caller). The MCP write gate (crates/reasonbraid-server/
-- src/mcp_write.rs) checks the caller's principal scope BEFORE admitting a
-- write call — the call-volume bound ADR-024 names.
--
-- The 0047 pattern: the backfill gives every EXISTING principal the dev
-- default (1000 write calls/hour) so older deployments keep working; the
-- enroll + the card-import paths create the row for NEW principals in the
-- same transaction as the identity row (an enrollment implies its quota).

INSERT INTO usage_quotas (quota_id, tenant_id, scope_kind, scope_id, ceiling, window_seconds)
SELECT 'quo_' || p.principal_id || '_writes',
       p.tenant_id,
       'principal',
       p.principal_id,
       1000,
       3600
FROM (
    SELECT principal_id, tenant_id FROM human_principals
    UNION
    SELECT role_id, tenant_id FROM agent_roles
) AS p;
