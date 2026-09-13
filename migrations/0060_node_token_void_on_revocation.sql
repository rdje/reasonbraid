-- SIGNOFF-REPAIR.4.1.2: a token does not outlive the authority that issued it.
--
-- Redemption validated only the token's own fields, so a token issued by an
-- administrator whose grant or boundary was revoked before it was redeemed still
-- enrolled the node. The principle this project has already applied twice —
-- `.3.5.2` (a revoked boundary left the metrics surface open) and `.4.1.3` (a
-- revoked node kept running and kept being fed) — is that withdrawn authority
-- stops producing effects. A redeemed token produces an enrolled node.
--
-- ⛔ The check does NOT go into redemption. `enroll` takes no tenant authority
-- guard, so re-reading `authority_grants.status` there would read authority state
-- outside the guard a revocation holds exclusively — the class `.3.3.4.5`
-- repaired. Revocation already holds that exclusive guard, and the token row is
-- already the declared serialization point, so voiding there is ordered by the
-- row lock alone: a redemption that arrives first commits (and the resulting node
-- is separately revocable), one that arrives second reads a voided row.
--
-- `issued_under` is the link, and it is exact: `authorization_records` already
-- stores the `grant_id` and `boundary_id` that allowed each decision, and the
-- issuance writes that record in the SAME transaction as the token. So a
-- revocation voids the tokens THAT authority issued, never a whole tenant's.

ALTER TABLE node_enrollment_tokens
    ADD COLUMN issued_under TEXT REFERENCES authorization_records (record_id),
    ADD COLUMN voided_at    TIMESTAMPTZ;

COMMENT ON COLUMN node_enrollment_tokens.issued_under IS
    'The authorization record that admitted this issuance. Resolves to the grant '
    'and boundary that allowed it, so revoking either can void exactly the tokens '
    'that authority issued. NULL for rows issued before this migration.';

COMMENT ON COLUMN node_enrollment_tokens.voided_at IS
    'Set when the grant or boundary that issued this token was revoked. Distinct '
    'from used_at (redeemed) and from superseded_at (expired unused and replaced '
    'by a later issuance): a voided token was never redeemed, never expired, and '
    'was not replaced — the authority behind it was withdrawn.';

-- 🔴 The one-live-token index MUST key on the new column. It reads
-- `used_at IS NULL AND superseded_at IS NULL`, and a voided token satisfies both
-- — so without this it would hold its node's index slot for ever, and an operator
-- who revoked a compromised administrator could never issue a replacement token
-- for that node. That is exactly `SIGNOFF-REPAIR.4.1.1`'s lockout, re-entered
-- through a different door.
DROP INDEX node_enrollment_tokens_one_live_idx;

CREATE UNIQUE INDEX node_enrollment_tokens_one_live_idx
    ON node_enrollment_tokens (node_id)
    WHERE used_at IS NULL AND superseded_at IS NULL AND voided_at IS NULL;

-- Selecting the tokens one revoked authority issued.
CREATE INDEX node_enrollment_tokens_issued_under_idx
    ON node_enrollment_tokens (issued_under) WHERE issued_under IS NOT NULL;
