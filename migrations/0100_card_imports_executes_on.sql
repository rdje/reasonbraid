-- 0100_card_imports_executes_on.sql — SIGNOFF-REPAIR.5.3.5.3.1 (DOC-0148,
-- ROADMAP §20.10 *remote recruitment*): the ORIGIN execution binding of an
-- imported identity is a ledger fact, and every role-to-node resolution reads
-- it through ONE view.
--
-- `executes_on` is NULL for the `local` binding (the importing tenant enrols a
-- machine for the imported role, the dev rule's node == role) and names the
-- origin role's node for the `origin` binding, where the origin operator's
-- machine executes what the importing tenant's grant admitted. It is written
-- once, by the import, and only after the import verified the origin role HAS
-- an enrolled node in this deployment.
--
-- ⛔ No foreign key to `nodes`, deliberately. The node belongs to the ORIGIN
-- tenant and this row to the IMPORTING one; a reference would let one tenant's
-- binding hold the other's node row in place, coupling the origin's lifecycle
-- to a decision it did not take. A binding whose node row is gone resolves to a
-- node with no presence, which every reader already refuses — fail-closed
-- without the coupling.
ALTER TABLE card_imports ADD COLUMN executes_on TEXT;

-- The resolution every reader uses (`role_execution`): the node a role's work
-- runs on, or NULL when it runs nowhere.
--   * a role with no origin binding runs on the node of its own id (the dev
--     rule, unchanged for every existing row);
--   * an origin-bound identity runs on the bound node ONLY while BOTH
--     directions of the recruitment agreement are effective — the import's own
--     allowlist rung, re-asked at every resolution (ADR-026: no agreement, no
--     cross-domain answer) — and on NO node otherwise. ⛔ Never a fallback to a
--     local node of the same id: the binding is single-valued (DOC-0148).
-- The agreement test is `federation::has_effective_recruitment_agreement_in_tx`'s
-- SQL, both directions accepted, recruiting and unexpired.
CREATE VIEW role_execution AS
SELECT r.role_id,
       r.tenant_id,
       CASE
           WHEN ci.executes_on IS NULL THEN r.role_id
           WHEN EXISTS (SELECT 1 FROM federation_agreements fa
                        WHERE fa.tenant_id = ci.tenant_id
                          AND fa.remote_tenant_id = ci.origin_tenant_id
                          AND fa.status = 'accepted' AND fa.recruitment
                          AND (fa.expires_at IS NULL OR fa.expires_at > now()))
            AND EXISTS (SELECT 1 FROM federation_agreements fa
                        WHERE fa.tenant_id = ci.origin_tenant_id
                          AND fa.remote_tenant_id = ci.tenant_id
                          AND fa.status = 'accepted' AND fa.recruitment
                          AND (fa.expires_at IS NULL OR fa.expires_at > now()))
           THEN ci.executes_on
       END AS node_id
FROM agent_roles r
LEFT JOIN card_imports ci ON ci.role_id = r.role_id;
