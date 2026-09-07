-- The evaluation-service core (`.4.2`, ADR-017): the versioned corpus
-- registry + the experiment run records. The service RECORDS; the WP7
-- harness MEASURES (one grading implementation — the harness's outputs
-- persist here; a second judge would invite drift).
--
-- The registry is content-addressed: the row carries the declared 64-hex
-- digests of the corpus/prompts files (the harness re-derives them at run
-- time — the service stores what was declared, never recomputes an opinion).
CREATE TABLE evaluation_corpora (
    corpus_id       TEXT        NOT NULL,
    version         BIGINT      NOT NULL,
    cases_digest    TEXT        NOT NULL,
    prompts_digest  TEXT        NOT NULL,
    cases           JSONB       NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (corpus_id, version)
);

-- The experiment record (ADR-017): the workflow arm, the corpus version,
-- the DECLARED seed (NULL only for a deterministic run — an undeclared
-- randomness is the typed refusal), the trial count, and the harness's
-- result rows (the per-case grades + the confidence + the cost).
CREATE TABLE evaluation_runs (
    run_id          TEXT        NOT NULL,
    workflow        TEXT        NOT NULL,
    corpus_id       TEXT        NOT NULL,
    corpus_version  BIGINT      NOT NULL,
    seed            BIGINT,
    deterministic   BOOLEAN     NOT NULL DEFAULT false,
    trial_count     BIGINT      NOT NULL,
    results         JSONB       NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (run_id)
);
