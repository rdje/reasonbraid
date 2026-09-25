-- 0109_site_charter_register_action.sql — SIGNOFF-REPAIR.9.1.1: the charter
-- registration action becomes GRANTABLE.
--
-- `site_authority::Action::CharterRegister` ("charter_register") is the action
-- `POST /v1/governance-charters` authorizes, and it was added to the Rust enum
-- without the migration every earlier site action came with (0063, 0071, 0074,
-- 0083). The two CHECK constraints below enumerated eleven actions and not this
-- one, so no boundary or grant carrying it could be stored and the verb refused
-- every caller. The charter suite registered through `charters::register`
-- directly and never met the gate.
--
-- `tests/site_authority.rs::every_site_action_the_code_authorizes_can_be_granted`
-- now derives the action list from the enum and issues a boundary for each, so
-- the enum and these constraints cannot drift apart silently again.
ALTER TABLE public.site_boundaries DROP CONSTRAINT site_boundaries_actions_check;
ALTER TABLE public.site_boundaries ADD CONSTRAINT site_boundaries_actions_check CHECK (
    array_ndims(actions) = 1 AND cardinality(actions) > 0
    AND array_position(actions, NULL) IS NULL
    AND actions <@ ARRAY['registry_inspect', 'adapter_allow', 'adapter_revoke',
                         'region_declare', 'region_pair', 'region_unpair',
                         'evidence_expire', 'workflow_register', 'policy_register',
                         'evaluation_record', 'gate_evaluate', 'charter_register']::TEXT[]
);

ALTER TABLE public.site_grants DROP CONSTRAINT site_grants_actions_check;
ALTER TABLE public.site_grants ADD CONSTRAINT site_grants_actions_check CHECK (
    array_ndims(actions) = 1 AND cardinality(actions) > 0
    AND array_position(actions, NULL) IS NULL
    AND actions <@ ARRAY['registry_inspect', 'adapter_allow', 'adapter_revoke',
                         'region_declare', 'region_pair', 'region_unpair',
                         'evidence_expire', 'workflow_register', 'policy_register',
                         'evaluation_record', 'gate_evaluate', 'charter_register']::TEXT[]
);
