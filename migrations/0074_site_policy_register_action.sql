-- 0074_site_policy_register_action.sql — SIGNOFF-REPAIR.6.1.5.4: registering a
-- policy version becomes a named site-operator capability.
--
-- `POST /v1/policies` admitted any enrolled principal into a FIRST-COME
-- identifier namespace: `policy_versions` is keyed `PRIMARY KEY (policy_id,
-- version)` with no tenant column (`migrations/0038_policy_registry.sql`), so
-- the first caller to name a coordinate owns it and every later caller is
-- refused as a duplicate. Reproduced at runtime before this migration was
-- written: an enrolled principal in tenant B took `red-org-baseline 1.0.0`
-- (200), tenant A's own registration at that coordinate was refused 400
-- `already exists`, tenant A then READ tenant B's clause as the organization
-- baseline, and tenant B appended a 2.0.0 that withdrew it — all four with no
-- site authority of any kind.
--
-- A policy version is SITE-WIDE GOVERNANCE, decided and recorded in
-- `docs/decisions/2026-09-19_the-policy-library-is-shared-the-lifecycle-is-its-tenants.md`
-- (DOC-0071): the library is shared BY DESIGN — a policy only its author can
-- read is not governance — and its ownership model is a GRANT, not a tenant. So
-- the repair is never a tenant column; it is the authority the write always
-- needed. That record also requires this registry and the workflow registry to
-- take the SAME shape, which `SIGNOFF-REPAIR.7.1.2.1` gave the second one in
-- `0071`, following `0063` before it. This is the third instance of one
-- template, not a third design.
--
-- ⛔ This does NOT decide whether `owning_authority` must be a grant the
-- REGISTRAR holds. `policy.rs` routes that question to `SIGNOFF-REPAIR.9.1` by
-- name, and a capability to register is a different question from a rule about
-- whose authority may be named.
--
-- ⛔ It also does not touch the READS. `GET /v1/policies`, `POST
-- /v1/policies/resolve`, the impact map and the MCP policy bundle stay on
-- enrolment, because DOC-0071 decided the library is readable by the tenants it
-- governs and `SIGNOFF-REPAIR.6.1.5.3`'s control asserts it.
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
                         'evidence_expire', 'workflow_register',
                         'policy_register']::TEXT[]
);

ALTER TABLE public.site_grants DROP CONSTRAINT site_grants_actions_check;
ALTER TABLE public.site_grants ADD CONSTRAINT site_grants_actions_check CHECK (
    array_ndims(actions) = 1 AND cardinality(actions) > 0
    AND array_position(actions, NULL) IS NULL
    AND actions <@ ARRAY['registry_inspect', 'adapter_allow', 'adapter_revoke',
                         'region_declare', 'region_pair', 'region_unpair',
                         'evidence_expire', 'workflow_register',
                         'policy_register']::TEXT[]
);
