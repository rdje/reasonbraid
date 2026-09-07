-- The calibration records + the regression gates (`.4.4`, ADR-017): the
-- calibration ACCUMULATES the Brier + the confidence across the recorded
-- runs (the model grader never judges alone — the record feeds the gate);
-- the gate records the baseline + the threshold and only BLOCKS (the
-- evaluation appends a result row — it never mutates a measurement).
CREATE TABLE evaluation_calibrations (
    calibration_id  TEXT        NOT NULL,
    corpus_id       TEXT        NOT NULL,
    corpus_version  BIGINT      NOT NULL,
    workflow        TEXT        NOT NULL,
    run_ids         JSONB       NOT NULL,
    brier           DOUBLE PRECISION,
    confidence      JSONB       NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (calibration_id)
);

CREATE TABLE evaluation_gates (
    gate_id         TEXT             NOT NULL,
    corpus_id       TEXT             NOT NULL,
    corpus_version  BIGINT           NOT NULL,
    workflow        TEXT             NOT NULL,
    baseline        JSONB            NOT NULL,
    threshold       DOUBLE PRECISION NOT NULL,
    created_at      TIMESTAMPTZ      NOT NULL DEFAULT now(),
    PRIMARY KEY (gate_id)
);

-- The gate evaluations, APPEND-ONLY (each evaluation records a row — the
-- latest is the current; the gate never rewrites a result).
CREATE TABLE evaluation_gate_results (
    gate_id      TEXT        NOT NULL,
    passed       BOOLEAN     NOT NULL,
    failures     JSONB       NOT NULL,
    evaluated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
