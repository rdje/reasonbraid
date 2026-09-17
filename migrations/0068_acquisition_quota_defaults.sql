-- 0068_acquisition_quota_defaults.sql — SIGNOFF-REPAIR.11.14.3.14 (ROADMAP
-- §16.11, ADR-034): the two quota scope kinds that shipped with a vocabulary,
-- a schema and no producer.
--
-- `0047` wired the TENANT scope to `thread.invite` and named the principal,
-- resolver and destination bindings as deferrals. `.3.5.1` wired the PRINCIPAL
-- scope to the MCP write gate. `resolver` and `destination` remained: declared
-- in `SCOPE_KINDS`, permitted by this table's CHECK constraint, and written by
-- nothing — while §16.11 names "resolver abuse" and "scraping" explicitly and
-- `POST /v1/resources/{id}/resolve` performs a real network acquisition per
-- call with no quota, no storm control and no breaker.
--
-- ⛔ WHY THIS NEEDS A DEFAULT ROW RATHER THAN A SEEDED ONE PER MEMBER, which is
-- the load-bearing finding and the reason the two scopes sat unwired.
--
-- `check_in_tx` is FAIL-CLOSED: a scope with no configured row is the typed
-- `quota_unconfigured` refusal. That is correct — and it is correct because of a
-- property the two WIRED scopes happen to have and these two do not: their
-- members are created by a path the server controls, so the bound can be seeded
-- at creation. A tenant gets its row from the enroll transaction; a principal
-- gets its row from enrolment and from card import.
--
-- Neither unwired scope has such a path:
--
--   * the RESOLVER space grows at runtime — `POST /v1/resolvers` registers new
--     packs — so a per-resolver seeding rule closes every newly registered pack
--     for every tenant until someone notices;
--   * the DESTINATION space is the open internet, and cannot be enumerated in
--     advance at all. A fail-closed per-host bound would refuse every
--     acquisition to every host nobody had pre-configured, turning a bound into
--     an outage.
--
-- ⭐ So the fail-closed CONTRACT is kept and the fail-closed MECHANISM changes:
-- each tenant gets a DEFAULT row at the wildcard scope id `*`, and a
-- host-specific or resolver-specific row overrides it (most specific wins). The
-- property fail-closed exists for — there is always a bound — holds exactly as
-- before, because the absence of BOTH rows is still the typed refusal.
--
-- The ceilings are the dev-profile shape `0047` already established: generous
-- for the demo, still a bound, and explicitly NOT a measured production figure.
-- `SIGNOFF-REPAIR.11.6` forbids proposing a threshold before its population is
-- measured, and no acquisition volume has ever been measured; what is decided
-- here is the SHAPE of the bound, not its number.

INSERT INTO usage_quotas (quota_id, tenant_id, scope_kind, scope_id, ceiling, window_seconds)
SELECT 'quo_' || t.tenant_id || '_resolver_default', t.tenant_id, 'resolver', '*', 1000, 3600
FROM tenants t
ON CONFLICT (tenant_id, scope_kind, scope_id) DO NOTHING;

INSERT INTO usage_quotas (quota_id, tenant_id, scope_kind, scope_id, ceiling, window_seconds)
SELECT 'quo_' || t.tenant_id || '_destination_default', t.tenant_id, 'destination', '*', 1000, 3600
FROM tenants t
ON CONFLICT (tenant_id, scope_kind, scope_id) DO NOTHING;
