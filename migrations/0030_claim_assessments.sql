-- 0030_claim_assessments.sql — PHASE-4.6.3: the claim-evidence graph
-- (ROADMAP §12.7). Claims link to evidence with ONE of the five assessments;
-- the citation validation lives in the application layer (the excerpt must
-- appear in the snapshot's raw bytes) — this table records the edge: the
-- claim, the evidence, the kind, and the author's full rationale. Citation
-- existence alone never satisfies an evidence gate.

CREATE TABLE claim_assessments (
    assessment_id    TEXT        NOT NULL PRIMARY KEY,
    claim_id         TEXT        NOT NULL,
    snapshot_id      TEXT        NOT NULL REFERENCES evidence_snapshots (snapshot_id) ON DELETE CASCADE,
    assessment       TEXT        NOT NULL,
    author           TEXT        NOT NULL,
    verifier         TEXT,
    excerpt          TEXT        NOT NULL,
    selector         TEXT,
    rationale        TEXT        NOT NULL,
    source_authority TEXT        NOT NULL DEFAULT 'unassessed',
    freshness        TEXT        NOT NULL DEFAULT 'unassessed',
    independence     TEXT        NOT NULL DEFAULT 'unassessed',
    uncertainty      TEXT        NOT NULL DEFAULT 'unassessed',
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT claim_assessments_kind CHECK (
        assessment IN ('supports', 'contradicts', 'contextualizes', 'source_only', 'unverifiable')
    )
);

CREATE INDEX claim_assessments_snapshot_idx ON claim_assessments (snapshot_id);
CREATE INDEX claim_assessments_claim_idx ON claim_assessments (claim_id);
CREATE UNIQUE INDEX claim_assessments_replay_idx
    ON claim_assessments (claim_id, snapshot_id, assessment, author);
