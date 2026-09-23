-- 0093_authority_grants_decision_rule_constraints.sql —
-- SIGNOFF-REPAIR.11.4.7.2.1.5.4.1 (ROADMAP §4.2 `decision_rule_constraints`): the
-- decision rules a thread this grant's subject creates may declare, narrower
-- than the tenant's charter.
--
-- The charter is the tenant's law (`governance_charters.allowed_decision_rules`,
-- checked by `charters::allows_on` in the create transaction). This column is
-- the ISSUER's narrowing of it for one subject: a JSON array of charter wire
-- names, validated at the one grant-creation path so a row can only carry
-- names the vocabulary knows, and read from the ADMITTING grant beside the
-- charter check. NULL is what every existing grant reads as — the charter
-- alone, exactly as before. Added with its producer (the enrolment body) and
-- its reader in the same change, per DOC-0137's rule: never a column with no
-- producer (DOC-0140).

ALTER TABLE authority_grants ADD COLUMN decision_rule_constraints JSONB;
