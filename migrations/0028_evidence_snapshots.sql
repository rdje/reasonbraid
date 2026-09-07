-- 0028_evidence_snapshots.sql — PHASE-4.6.1: the §12.6 EvidenceSnapshot +
-- the content-addressed object store (the `.1.1` census's named trigger —
-- the Phase-4 blocker's last leg). A snapshot records the ACQUISITION facts
-- (the receipts the `.2`–`.5` packs produce) + points at the raw bytes,
-- which the object store persists UNDER their ADR-011 digest (shared:
-- identical bytes = one row). The tombstone rule (§12.9): the deletion is
-- the row's `deleted_at` + `deletion_reason` — never a silent disappearance.

CREATE TABLE snapshot_objects (
    digest      TEXT        NOT NULL PRIMARY KEY,
    bytes       BYTEA       NOT NULL,
    stored_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE evidence_snapshots (
    snapshot_id             TEXT        NOT NULL PRIMARY KEY,
    reference_id            TEXT        NOT NULL REFERENCES resource_references (resource_id) ON DELETE CASCADE,
    original_locator        TEXT        NOT NULL,
    final_locator           TEXT        NOT NULL,
    retrieved_at            TIMESTAMPTZ NOT NULL,
    resolver_id             TEXT        NOT NULL,
    resolver_version        TEXT        NOT NULL,
    network_class           TEXT        NOT NULL DEFAULT 'none',
    auth_class              TEXT        NOT NULL DEFAULT 'none',
    provider_receipt        JSONB       NOT NULL DEFAULT '{}'::jsonb,
    immutable_source_version TEXT      ,
    raw_digest              TEXT        NOT NULL REFERENCES snapshot_objects (digest),
    byte_length             BIGINT      NOT NULL,
    media_type              TEXT        NOT NULL,
    storage_class           TEXT        NOT NULL DEFAULT 'standard',
    retention_class         TEXT        NOT NULL DEFAULT 'standard',
    extraction_version      TEXT,
    quarantine_status       TEXT        NOT NULL DEFAULT 'none',
    redactions              JSONB       NOT NULL DEFAULT '[]'::jsonb,
    disclosure_policy       JSONB       NOT NULL DEFAULT '{}'::jsonb,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at              TIMESTAMPTZ,
    deletion_reason         TEXT
);

CREATE INDEX evidence_snapshots_reference_idx ON evidence_snapshots (reference_id);
CREATE INDEX evidence_snapshots_raw_digest_idx ON evidence_snapshots (raw_digest);
