-- 0072_routing_journal_tenant.sql — SIGNOFF-REPAIR.7.1.2.2: the routing journal
-- is read by its own tenant.
--
-- `GET /v1/routing/resolutions` and `GET /v1/routing/recommendations` admitted
-- any enrolled principal and selected with NO predicate of any kind, so every
-- tenant read every tenant's routing trail: which case classes it submitted,
-- which arm each resolved to, which principal asked, and — for the shadow
-- recommendations — which evaluation trial or gate the recommendation rested on.
--
-- ⚠️ A DISCLOSURE path, not a control one, and the migration should not be read
-- as claiming otherwise. `routing::resolve` reads NEITHER table — it reads
-- `routing_rules` and `workflow_profiles` — so no row here binds anybody's
-- outcome. This is the shape
-- `docs/decisions/2026-09-16_evidence-is-shared-the-read-is-tenant-bound.md`
-- names: a site-wide row whose READ must name the tenant.
--
-- ⛔ The tenant is DERIVED from the authenticated caller and never accepted on
-- the wire, per
-- `docs/decisions/2026-09-18_node-presence-is-read-by-its-own-tenant.md`. A
-- principal belongs to exactly one tenant structurally, so there is no second
-- identifier to name and none to bind.
--
-- THE ROWS ALREADY STORED, decided rather than defaulted into — and the two
-- tables get DIFFERENT answers, because they recorded different things:
--
--   * `routing_resolutions.caller` holds `GrantSubject::id_string()`, which is
--     the primary key of `human_principals` or `agent_roles`, and each carries
--     exactly one `tenant_id`. So the historical rows ARE attributable, and the
--     backfill below is a derivation rather than a guess.
--   * `routing_recommendations` has no actor column at all — it never recorded
--     who submitted a recommendation. Nothing in the schema can attribute those
--     rows, so they keep `tenant_id IS NULL` and are read by NOBODY, exactly as
--     DOC-0029's snapshots written before their citation table are.
--
-- ⚠️ A row whose caller resolves to neither table (a principal since deleted)
-- also stays NULL and is read by nobody. Left NULL deliberately: inventing an
-- owner for an unattributable audit row would be worse than losing its
-- visibility, because the row asserts who did something.

ALTER TABLE routing_resolutions ADD COLUMN tenant_id TEXT REFERENCES tenants (tenant_id);
ALTER TABLE routing_recommendations ADD COLUMN tenant_id TEXT REFERENCES tenants (tenant_id);

-- The derivation, both subject kinds. `caller` is the principal's own id.
UPDATE routing_resolutions r
   SET tenant_id = h.tenant_id
  FROM human_principals h
 WHERE r.tenant_id IS NULL AND r.caller = h.principal_id;

UPDATE routing_resolutions r
   SET tenant_id = a.tenant_id
  FROM agent_roles a
 WHERE r.tenant_id IS NULL AND r.caller = a.role_id;

-- The read predicate is `tenant_id = $1`, which never matches NULL, so the
-- unattributable rows need no further treatment: they are already invisible.
CREATE INDEX routing_resolutions_tenant_idx ON routing_resolutions (tenant_id, resolved_at DESC);
CREATE INDEX routing_recommendations_tenant_idx ON routing_recommendations (tenant_id, recorded_at DESC);
