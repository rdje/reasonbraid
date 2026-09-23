-- 0099_authority_grants_conditions.sql — SIGNOFF-REPAIR.11.4.7.2.1.5.4.3 (ROADMAP
-- §4.2): a grant's `conditions[]` have a vocabulary and a column.
--
-- §4.2 named the field and nothing defined a condition, so the field was never
-- declared — the one honest state for a vocabulary that does not exist. The
-- vocabulary is now a CLOSED, typed set (`reasonbraid_core::GrantCondition`),
-- validated at issuance and evaluated at every admission; its first member is
-- `within_hours`. NULL is unconditional, which is what every existing grant
-- was.
ALTER TABLE authority_grants ADD COLUMN conditions JSONB;
