-- 0116_a_freshness_horizon_is_the_citing_tenants.sql — SIGNOFF-REPAIR.7.4.9.
--
-- A freshness horizon is a tenant's own decision about when to re-acquire, and
-- the snapshot row it sat on is SHARED by every tenant that cites the same
-- bytes (docs/decisions/2026-09-16_evidence-is-shared-the-read-is-tenant-bound.md).
-- On the shared row, the first tenant to acquire decided every later tenant's
-- stale list, and a re-acquisition's horizon was discarded. It moves to the
-- tenant's citation: every existing citation takes the row's horizon, the only
-- one ever recorded, and the shared column goes, so no reader can take the
-- first acquirer's horizon for its own.

ALTER TABLE evidence_citations ADD COLUMN fresh_until TIMESTAMPTZ;

UPDATE evidence_citations c
   SET fresh_until = s.fresh_until
  FROM evidence_snapshots s
 WHERE s.snapshot_id = c.snapshot_id;

ALTER TABLE evidence_snapshots DROP COLUMN fresh_until;
