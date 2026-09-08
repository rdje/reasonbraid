-- The scheduled review records (`.6`, §15.11): the trigger vocabulary
-- (the seven review triggers), the DUE evaluation (one due review per
-- (publication, trigger) — the dedupe), and the done transition. The
-- reviews consume the `.5.3` outcomes/corrections/drift records.
CREATE TABLE policy_reviews (
    review_id      TEXT        NOT NULL,
    publication_id TEXT        NOT NULL,
    trigger        TEXT        NOT NULL,
    status         TEXT        NOT NULL DEFAULT 'due',
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    done_at        TIMESTAMPTZ,
    PRIMARY KEY (review_id)
);
