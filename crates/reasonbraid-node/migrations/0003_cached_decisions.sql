-- 0003_cached_decisions.sql — cached-decision semantics (PHASE-2.1.5.2; ADR-008).
--
-- The ADMISSION decision a delivered command carries (the server's inbox metadata):
-- the admitting authorization record id (the pre-shaped `authz_ref` finally gains a
-- value), the policy digest, the decision time (the freshness TTL runs from it), and
-- the tenant's revocation epoch AT DECISION TIME. A command recorded without these
-- (pre-0003 rows) has no cached decision — the dispatch boundary refuses it
-- (fail-closed, §16.4).

ALTER TABLE commands ADD COLUMN policy_digest TEXT;
ALTER TABLE commands ADD COLUMN decided_at TEXT;
ALTER TABLE commands ADD COLUMN revocation_epoch INTEGER;

PRAGMA user_version = 3;
