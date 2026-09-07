-- 0029_derivations.sql — PHASE-4.6.2: the Derivation edges (ROADMAP §12.6).
-- Every transformation is a derivation: the snapshot's derived chunks, the
-- extraction/normalization outputs, the excerpts — each carries its own
-- ADR-011 digest + the parent link. A quote, a summary, an OCR result, a
-- model-generated caption, or a repository analysis is NEVER the original,
-- and this graph says so (the parent snapshot stays addressable).

CREATE TABLE derivations (
    derivation_id      TEXT        NOT NULL PRIMARY KEY,
    parent_snapshot_id TEXT        NOT NULL REFERENCES evidence_snapshots (snapshot_id) ON DELETE CASCADE,
    derived_kind       TEXT        NOT NULL,
    derived_digest     TEXT        NOT NULL,
    content            TEXT        NOT NULL,
    extraction_version TEXT,
    source_selector    TEXT,
    derived_at         TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX derivations_parent_idx ON derivations (parent_snapshot_id);
CREATE UNIQUE INDEX derivations_replay_idx ON derivations (parent_snapshot_id, derived_kind, derived_digest);
