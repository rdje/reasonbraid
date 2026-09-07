-- 0013_cached_decisions.sql — cached-decision semantics (PHASE-2.1.5.2; backlog 11, ADR-008).
--
-- §16.4: nodes may cache ONLY explicitly cacheable decisions and must honor expiry +
-- revocation freshness. ADR-008's dev shape: the server's ADMISSION decision rides the
-- delivery (the node caches ONLY that — §11.1's minimum state), the node evaluates it
-- at the dispatch boundary against a 60s freshness TTL and the tenant's revocation
-- epoch.
--
-- `tenants.revocation_epoch` is the per-tenant invalidation counter: every revocation
-- write (node/grant/boundary — the `.1.3` paths) bumps it in the SAME transaction as
-- the status change, so a cached decision recording the epoch it was decided under is
-- invalidated by any later revocation (however fresh it looks).
ALTER TABLE tenants ADD COLUMN revocation_epoch BIGINT NOT NULL DEFAULT 0;

-- The decision metadata a delivered work item carries: the admitting authorization
-- record, the policy digest it bound, when it was decided (the freshness TTL runs from
-- here), and the tenant's epoch AT DECISION TIME. Rows enqueued before this migration
-- have none — the node treats a missing decision as fail-closed (refuse the dispatch).
ALTER TABLE node_inbox ADD COLUMN authz_ref TEXT,
                      ADD COLUMN policy_digest TEXT,
                      ADD COLUMN decided_at TIMESTAMPTZ,
                      ADD COLUMN revocation_epoch BIGINT;
