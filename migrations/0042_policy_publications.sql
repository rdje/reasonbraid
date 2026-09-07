-- The publication records + the staging (`.4.2`, ADR-020): the publication
-- is the nine-step §15.7 machine's AGGREGATE — its own row (never folded
-- into the approval) carrying the decision/approval/projection references,
-- the manifest digest, the typed state (staged → effective | failed), the
-- Git object ids, and the failure reason (a step that cannot complete is
-- the typed failure, never a skip).
CREATE TABLE policy_publications (
    publication_id  TEXT        NOT NULL,
    proposal_id     TEXT        NOT NULL,
    decision_id     TEXT        NOT NULL,
    approval_id     TEXT        NOT NULL,
    projection_id   TEXT        NOT NULL,
    state           TEXT        NOT NULL DEFAULT 'staged',
    manifest_digest TEXT        NOT NULL,
    git_object_ids  JSONB       NOT NULL DEFAULT '[]'::jsonb,
    failed_reason   TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (publication_id)
);
