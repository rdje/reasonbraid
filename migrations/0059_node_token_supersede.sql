-- SIGNOFF-REPAIR.4.1.1: a token that LAPSED unused locked its node out for good.
--
-- 0018 replaced 0008's unconditional UNIQUE with a partial one — "at most one
-- token with `used_at IS NULL` per node" — and that is right for a LIVE token and
-- wrong for a dead one. An EXPIRED token still has `used_at IS NULL`, so it stayed
-- in the index permanently; issuance (`ON CONFLICT DO NOTHING RETURNING`) answered
-- `409 … consume or expire it before issuing another` for every later attempt at
-- that node id, while the lapsed token itself enrolled nothing. Expiring it was the
-- recovery the message advertised and the one thing that could not work.
--
-- ⛔ The index predicate CANNOT simply become `AND expires_at > now()`: a partial
-- index predicate must be IMMUTABLE and `now()` is not. So the liveness that the
-- index cannot evaluate is written down instead — the issuance transaction stamps
-- a lapsed row `superseded_at` and the index keys on the stamp.
--
-- Additive, and nothing is backfilled. Every existing row keeps `superseded_at
-- IS NULL`, so the new index keys exactly the rows the old one keyed and the
-- one-live-token invariant is carried over unchanged at the instant of migration.
-- A lapsed row already sitting in the index is not swept here; it is superseded by
-- the next issuance for its node, which is the moment its liveness is being asked
-- about and the only moment the answer matters.

ALTER TABLE node_enrollment_tokens ADD COLUMN superseded_at TIMESTAMPTZ;

COMMENT ON COLUMN node_enrollment_tokens.superseded_at IS
    'Set when a later issuance for this node replaced this token because it had '
    'expired unused. Distinct from used_at, which means redeemed: a superseded '
    'token was never redeemed and must never be recorded as though it had been.';

DROP INDEX node_enrollment_tokens_one_unused_idx;

-- At most one token per node that is both unredeemed AND not superseded. Two LIVE
-- unused tokens still collide, which is what 0018 bought.
CREATE UNIQUE INDEX node_enrollment_tokens_one_live_idx
    ON node_enrollment_tokens (node_id) WHERE used_at IS NULL AND superseded_at IS NULL;
