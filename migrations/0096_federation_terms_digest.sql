-- 0096_federation_terms_digest.sql — SIGNOFF-REPAIR.5.3.1 (ADR-026): a direction
-- carries a digest of its terms, and an acceptance records the counterparty's.
--
-- `cross_domain_receipts.remote_ref` is declared *the remote record's
-- digest-pinned reference* (0049), and the acceptance wrote the counterparty's
-- TENANT ID into it — an id, of no record, and no digest — because a direction
-- row had nothing to digest. Now every row carries `terms_digest`, computed by
-- the server over the canonical terms on every proposal, and the accepting row
-- records `accepted_against`: the counterparty's `terms_digest` as read at
-- acceptance, which is also what the receipt's `remote_ref` names.
--
-- The canonical form is ONE recipe in two places — here for the backfill and in
-- `crates/reasonbraid-server/src/federation.rs::terms_digest` for every later
-- write — and a control derives the same value by both routes:
--     tenant_id || '\n' || remote_tenant_id || '\n' || directory_visibility || '\n' || recruitment
-- with the booleans spelled `true` / `false`, SHA-256, hex, `sha256:` prefixed.
-- A re-proposal on different terms changes the digest and clears
-- `accepted_against` (the acceptance it clears was against the old terms).
ALTER TABLE federation_agreements
    ADD COLUMN terms_digest     TEXT,
    ADD COLUMN accepted_against TEXT;

UPDATE federation_agreements
   SET terms_digest = 'sha256:' || encode(sha256(convert_to(
           tenant_id || E'\n' || remote_tenant_id || E'\n'
           || directory_visibility::text || E'\n' || recruitment::text, 'UTF8')), 'hex');

ALTER TABLE federation_agreements ALTER COLUMN terms_digest SET NOT NULL;
