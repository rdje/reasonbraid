-- The randomized routing trials + the cohort tracking (`.4.3`, ADR-017):
-- the SHADOW experiment records. The trial declares its seed, the arms, the
-- cohorts (the recorded labels), and the cases; the SERVER computes the
-- seeded assignment (reproducible — the same seed + cases re-draw the same
-- assignment, never a silent guess). The trial never changes production
-- routing (the `.5` lane's decision consumes these records).
CREATE TABLE evaluation_trials (
    trial_id        TEXT        NOT NULL,
    corpus_id       TEXT        NOT NULL,
    corpus_version  BIGINT      NOT NULL,
    seed            BIGINT      NOT NULL,
    arms            JSONB       NOT NULL,
    cohorts         JSONB       NOT NULL,
    case_ids        JSONB       NOT NULL,
    assignment      JSONB       NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (trial_id)
);

-- The per-arm results, APPEND-ONLY (each submission is a new row — the
-- record's identity is its content, never an overwrite).
CREATE TABLE evaluation_trial_results (
    trial_id    TEXT        NOT NULL,
    results     JSONB       NOT NULL,
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
