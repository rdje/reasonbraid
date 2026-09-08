-- The drift + the corrections + the outcomes (`.5.3`, ADR-021): the drift
-- record names the §15.10 category over the desired/observed pair; the
-- correction is its OWN row with the §4.7 operation + the AUTHORITY PROOF
-- (the grant re-check — the reversal is fast, the authority is NOT
-- universally lower) — the retraction NEVER deletes the original; the
-- outcome record links the publication to what happened after (§15.11).
CREATE TABLE policy_drift (
    drift_id       TEXT        NOT NULL,
    target_id      TEXT        NOT NULL,
    publication_id TEXT        NOT NULL,
    category       TEXT        NOT NULL,
    desired_digest TEXT        NOT NULL,
    observed_digest TEXT,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (drift_id)
);

CREATE TABLE policy_corrections (
    correction_id  TEXT        NOT NULL,
    publication_id TEXT        NOT NULL,
    operation      TEXT        NOT NULL,
    authority_grant TEXT       NOT NULL,
    supersedes     TEXT,
    expires_at     TIMESTAMPTZ,
    reason         TEXT        NOT NULL,
    evidence       JSONB       NOT NULL DEFAULT '[]'::jsonb,
    remediation    TEXT,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (correction_id)
);

CREATE TABLE policy_outcomes (
    outcome_id     TEXT        NOT NULL,
    publication_id TEXT        NOT NULL,
    kind           TEXT        NOT NULL,
    review_trigger TEXT,
    note           TEXT        NOT NULL,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (outcome_id)
);
