-- 0083_site_evaluation_actions.sql — SIGNOFF-REPAIR.8.2.5: the evaluation
-- harness's writes become named site-operator capabilities.
--
-- All seven `evaluation_*` tables carry no tenant column, and every write
-- admitted on bare enrolment — `reader_tenant(...).is_some()` and nothing else,
-- with no principal reaching the service. So any enrolled principal in the
-- deployment registered a corpus, recorded a run, created a trial, recorded a
-- calibration, recorded a gate and evaluated any gate, over the whole site.
--
-- ⛔ The absent tenant column is the DESIGN, not the defect.
-- `docs/decisions/2026-09-19_the-policy-registry-is-a-shared-control-surface.md`
-- records the verdict for this exact family: site-wide by design, gated by
-- SITE-OPERATOR grants. ROADMAP §19 is release engineering — gate records,
-- corpus versions, CI manifests — not a tenant product surface. What was never
-- applied is the ruled gate, and this is that application, in the shape
-- `docs/decisions/2026-09-09_site-operator-authority.md` defines and `0063` and
-- `0071` already took.
--
-- ⭐ TWO ACTIONS, NOT ONE, AND THE SPLIT IS DERIVED RATHER THAN PREFERRED.
-- ROADMAP §4.1's `GovernanceCharter` names `separation-of-duties and
-- conflict-of-interest constraints` as a first-class charter property, and §19's
-- release flow has two distinct parties: one maintains the corpus, the runs and
-- the gate baselines — the STANDARD — while CI evaluates gates against it
-- continuously. A party that measures against a standard must not be able to
-- move the standard, so:
--
--   evaluation_record : register_corpus, record_run, create_trial,
--                       record_trial_results, record_calibration, record_gate
--   gate_evaluate     : evaluate_gate
--
-- ⚠️ The READ side is deliberately unchanged and stays on enrolment. DOC-0029
-- rules these tables site-wide, and §19.7's gate manifest is meant to be
-- auditable; nothing in the roadmap makes a corpus version or a gate result
-- confidential. Gating reads would be a separate decision about confidentiality
-- and is not taken here by omission.
--
-- The action set is enumerated in two CHECK constraints rather than a lookup
-- table, so widening it is a migration by construction. Both are replaced in one
-- statement each; no row can already hold either new name, so neither
-- replacement can fail validation against existing data.

ALTER TABLE public.site_boundaries DROP CONSTRAINT site_boundaries_actions_check;
ALTER TABLE public.site_boundaries ADD CONSTRAINT site_boundaries_actions_check CHECK (
    array_ndims(actions) = 1 AND cardinality(actions) > 0
    AND array_position(actions, NULL) IS NULL
    AND actions <@ ARRAY['registry_inspect', 'adapter_allow', 'adapter_revoke',
                         'region_declare', 'region_pair', 'region_unpair',
                         'evidence_expire', 'workflow_register', 'policy_register',
                         'evaluation_record', 'gate_evaluate']::TEXT[]
);

ALTER TABLE public.site_grants DROP CONSTRAINT site_grants_actions_check;
ALTER TABLE public.site_grants ADD CONSTRAINT site_grants_actions_check CHECK (
    array_ndims(actions) = 1 AND cardinality(actions) > 0
    AND array_position(actions, NULL) IS NULL
    AND actions <@ ARRAY['registry_inspect', 'adapter_allow', 'adapter_revoke',
                         'region_declare', 'region_pair', 'region_unpair',
                         'evidence_expire', 'workflow_register', 'policy_register',
                         'evaluation_record', 'gate_evaluate']::TEXT[]
);
