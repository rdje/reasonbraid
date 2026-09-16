-- 0063_site_evidence_expire_action.sql — SIGNOFF-REPAIR.7.4.3: the retention
-- sweep becomes a named site-operator capability.
--
-- `POST /v1/snapshots/expire-due` admitted any enrolled principal and read its
-- cutoff from the request body, while `snapshots::expire_due` carries no tenant
-- predicate — so one request naming a far-future instant tombstoned every
-- tenant's live `standard` and `temporary` evidence, irreversibly.
--
-- Which rows are DUE is a property of `evidence_snapshots.retention_class` — a
-- column on the SHARED row, not on a citation (`0062`) — so it is not a
-- per-tenant fact and the sweep cannot be a tenant verb. It takes the same
-- shape `docs/decisions/2026-09-09_site-operator-authority.md` defines for every
-- other site-wide act: an explicitly issued grant naming the action, evaluated
-- against its actual boundary, with the effect and its audit committing
-- together.
--
-- The action set is enumerated in two CHECK constraints rather than a lookup
-- table, so widening it is a migration by construction. Both are replaced in
-- one statement each; no row can already hold the new name, so neither
-- replacement can fail validation against existing data.

ALTER TABLE public.site_boundaries DROP CONSTRAINT site_boundaries_actions_check;
ALTER TABLE public.site_boundaries ADD CONSTRAINT site_boundaries_actions_check CHECK (
    array_ndims(actions) = 1 AND cardinality(actions) > 0
    AND array_position(actions, NULL) IS NULL
    AND actions <@ ARRAY['registry_inspect', 'adapter_allow', 'adapter_revoke',
                         'region_declare', 'region_pair', 'region_unpair',
                         'evidence_expire']::TEXT[]
);

ALTER TABLE public.site_grants DROP CONSTRAINT site_grants_actions_check;
ALTER TABLE public.site_grants ADD CONSTRAINT site_grants_actions_check CHECK (
    array_ndims(actions) = 1 AND cardinality(actions) > 0
    AND array_position(actions, NULL) IS NULL
    AND actions <@ ARRAY['registry_inspect', 'adapter_allow', 'adapter_revoke',
                         'region_declare', 'region_pair', 'region_unpair',
                         'evidence_expire']::TEXT[]
);
