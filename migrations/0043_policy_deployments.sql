-- The target-deployment records + the waves (`.5.2`, ADR-021): the
-- deployment is PER-TARGET, never globally atomic — each target carries
-- its DESIRED/OBSERVED pair; the canary wave assignment rides the
-- effective publication's ref + the projection digest; the RECEIPT attests
-- the OBSERVED digest (the drift's comparison input, never the hope).
CREATE TABLE deployment_targets (
    target_id        TEXT        NOT NULL,
    target_type      TEXT        NOT NULL,
    owning_authority TEXT        NOT NULL,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (target_id)
);

CREATE TABLE deployment_assignments (
    target_id       TEXT        NOT NULL,
    publication_id  TEXT        NOT NULL,
    wave            BIGINT      NOT NULL,
    desired_ref     TEXT        NOT NULL,
    desired_digest  TEXT        NOT NULL,
    observed_digest TEXT,
    observed_state  TEXT        NOT NULL DEFAULT 'pending',
    PRIMARY KEY (target_id, publication_id)
);
