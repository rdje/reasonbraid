-- 0064_assessment_authoring_tenant.sql — SIGNOFF-REPAIR.11.14.2: the tenant
-- that AUTHORED an assessment, recorded by the server.
--
-- `GET /v1/claims/{claim_id}/assessments` admitted any enrolled principal over
-- a namespace the server never mints: no `claims` table exists, `claim_id` is
-- caller-supplied TEXT with no key, and the shipped control's own identifier is
-- `clm_budget`. That made it an ORACLE over guessable ids.
--
-- ⭐ This table takes a COLUMN where `evidence_snapshots` took a citation set
-- (`0062`), and the difference is in the replay keys rather than in taste.
-- `derivations_replay_idx` is `(parent_snapshot_id, derived_kind,
-- derived_digest)` — content-addressed, so one row genuinely serves every
-- tenant. `claim_assessments_replay_idx` is `(claim_id, snapshot_id,
-- assessment, author)` and carries the AUTHOR, so two tenants asserting the
-- same thing about the same evidence already hold two separate rows. An
-- assessment is an authored opinion, not a shared receipt, and the
-- content-addressing argument that forbids a column on the other two evidence
-- tables does not reach it. `docs/decisions/2026-09-16_evidence-is-shared-the-read-is-tenant-bound.md`
-- grouped all three together; that record is corrected rather than followed.
--
-- ⛔ NULLABLE, and deliberately. A row written before this migration has no
-- recoverable author tenant: `author` is a caller-supplied label, not a
-- principal, so there is nothing to derive one from. NULL therefore means
-- "unattributed", and an unattributed row is read by NO tenant — the same
-- fail-closed disposition `0062` takes for an uncited snapshot. New rows are
-- always attributed; `claims::submit` requires the tenant to build one.
--
-- ⛔ This does NOT make `author` trustworthy. `author` and `verifier` remain
-- unauthenticated caller strings, which is `SIGNOFF-REPAIR.7.4`'s attached
-- clause and is deferred to it by name. This column sits beside them and is
-- server-derived; the authorization reads this one and never `author`.

ALTER TABLE claim_assessments ADD COLUMN authored_by_tenant TEXT;

CREATE INDEX claim_assessments_authoring_tenant_idx
    ON claim_assessments (authored_by_tenant);
