-- 0031_snapshot_license_freshness.sql — PHASE-4.6.4: the license metadata +
-- the freshness horizon (ROADMAP §12.6's license/retention + §12.9's
-- freshness). The retention ENFORCEMENT rides the application layer (the
-- tombstone + the class's TTL — the audit class never expires: binding
-- decisions stay addressable for the charter's audit period).

ALTER TABLE evidence_snapshots
    ADD COLUMN license TEXT,
    ADD COLUMN fresh_until TIMESTAMPTZ,
    ADD COLUMN refreshed_at TIMESTAMPTZ;
