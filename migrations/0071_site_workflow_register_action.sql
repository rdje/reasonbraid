-- 0071_site_workflow_register_action.sql — SIGNOFF-REPAIR.7.1.2.1: registering a
-- workflow profile becomes a named site-operator capability.
--
-- `POST /v1/workflow-profiles` admitted any enrolled principal and appended
-- `MAX(version) + 1` under ANY `profile_id`, the eight §13.1 built-ins included,
-- while `workflows::resolve` selects `ORDER BY version DESC LIMIT 1` site-wide
-- with no `built_in` filter and no tenant predicate — and thread creation
-- resolves `DEFAULT_PROFILE_ID = 'quick_advice'` through it. So one enrolled
-- principal chose the steps every other tenant's next bare thread executed.
--
-- A workflow profile is SITE-WIDE CONFIGURATION: `workflow_profiles` is keyed
-- `(profile_id, version)` with no tenant column, and its rows are the arms the
-- routing table names (`migrations/0036_routing_policy.sql`). Registering one is
-- therefore not a tenant verb, which the module already said in its own words —
-- `workflows::register`'s doc comment reads "the operator's verb" — and never
-- enforced. It takes the shape
-- `docs/decisions/2026-09-09_site-operator-authority.md` defines for every
-- site-wide act, exactly as `0063` did for the retention sweep.
--
-- ⛔ This does NOT decide whether a workflow profile could instead belong to a
-- tenant. That is the same question `SIGNOFF-REPAIR.6.1.5` owns for the policy
-- registry, and a second registry must not answer it unilaterally
-- (`docs/decisions/2026-09-19_the-policy-registry-is-a-shared-control-surface.md`).
-- One site-wide library is what this code already claims to be; it is now what
-- it is.
--
-- The action set is enumerated in two CHECK constraints rather than a lookup
-- table, so widening it is a migration by construction. Both are replaced in one
-- statement each; no row can already hold the new name, so neither replacement
-- can fail validation against existing data.

ALTER TABLE public.site_boundaries DROP CONSTRAINT site_boundaries_actions_check;
ALTER TABLE public.site_boundaries ADD CONSTRAINT site_boundaries_actions_check CHECK (
    array_ndims(actions) = 1 AND cardinality(actions) > 0
    AND array_position(actions, NULL) IS NULL
    AND actions <@ ARRAY['registry_inspect', 'adapter_allow', 'adapter_revoke',
                         'region_declare', 'region_pair', 'region_unpair',
                         'evidence_expire', 'workflow_register']::TEXT[]
);

ALTER TABLE public.site_grants DROP CONSTRAINT site_grants_actions_check;
ALTER TABLE public.site_grants ADD CONSTRAINT site_grants_actions_check CHECK (
    array_ndims(actions) = 1 AND cardinality(actions) > 0
    AND array_position(actions, NULL) IS NULL
    AND actions <@ ARRAY['registry_inspect', 'adapter_allow', 'adapter_revoke',
                         'region_declare', 'region_pair', 'region_unpair',
                         'evidence_expire', 'workflow_register']::TEXT[]
);
