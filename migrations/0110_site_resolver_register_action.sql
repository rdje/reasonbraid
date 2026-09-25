-- 0110_site_resolver_register_action.sql — SIGNOFF-REPAIR.7.1.3.1: registering
-- a resolver becomes a named site-operator capability.
--
-- `POST /v1/resolvers` admitted any tenant administrator and wrote a row into
-- `resolver_capabilities`, which has no tenant column and which every tenant's
-- resolution ranks (`.7.1.3` measured one foreign row silencing every tenant's
-- acquisition). A resolver describes what this server can acquire through, so
-- registering one is site configuration, the shape `0071` gave the workflow
-- registry. `tests/site_authority.rs::every_site_action_the_code_authorizes_can_be_granted`
-- fails if this list and `site_authority::Action` disagree.
ALTER TABLE public.site_boundaries DROP CONSTRAINT site_boundaries_actions_check;
ALTER TABLE public.site_boundaries ADD CONSTRAINT site_boundaries_actions_check CHECK (
    array_ndims(actions) = 1 AND cardinality(actions) > 0
    AND array_position(actions, NULL) IS NULL
    AND actions <@ ARRAY['registry_inspect', 'adapter_allow', 'adapter_revoke',
                         'region_declare', 'region_pair', 'region_unpair',
                         'evidence_expire', 'workflow_register', 'policy_register',
                         'evaluation_record', 'gate_evaluate', 'charter_register',
                         'resolver_register']::TEXT[]
);

ALTER TABLE public.site_grants DROP CONSTRAINT site_grants_actions_check;
ALTER TABLE public.site_grants ADD CONSTRAINT site_grants_actions_check CHECK (
    array_ndims(actions) = 1 AND cardinality(actions) > 0
    AND array_position(actions, NULL) IS NULL
    AND actions <@ ARRAY['registry_inspect', 'adapter_allow', 'adapter_revoke',
                         'region_declare', 'region_pair', 'region_unpair',
                         'evidence_expire', 'workflow_register', 'policy_register',
                         'evaluation_record', 'gate_evaluate', 'charter_register',
                         'resolver_register']::TEXT[]
);
