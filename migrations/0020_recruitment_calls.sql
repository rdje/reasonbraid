-- 0020_recruitment_calls.sql — PHASE-3.4.2: the open-call artifact (backlog 29/30):
-- the §10.5 call spec (the eligibility expression + the audience + the
-- min/max participants + the role slots + the advertisement window + the join
-- deadline + the expiry + the recommendations flag) + the typed responses
-- (the §10.5 vocabulary) + the panel snapshot with its selection explanation.
-- ADR-015's baseline: the call rides the SAME thread invitation machinery —
-- the panel snapshot feeds the thread's participant flow, never a parallel
-- system.

CREATE TABLE recruitment_calls (
    call_id                TEXT        NOT NULL PRIMARY KEY,
    tenant_id              TEXT        NOT NULL REFERENCES tenants (tenant_id),
    thread_id              TEXT        NOT NULL,
    initiator              TEXT        NOT NULL,  -- the actor handle (the audit linkage)
    expression             JSONB       NOT NULL,  -- the EligibilityExpression
    min_participants       INT         NOT NULL DEFAULT 1,
    max_participants       INT         NOT NULL DEFAULT 4,
    role_slots             JSONB       NOT NULL DEFAULT '[]'::jsonb,
    recommendations_allowed BOOL       NOT NULL DEFAULT false,
    advertises_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    join_deadline          TIMESTAMPTZ NOT NULL,
    expires_at             TIMESTAMPTZ NOT NULL,
    status                 TEXT        NOT NULL DEFAULT 'open',  -- open | closed
    created_at             TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE recruitment_responses (
    response_id   TEXT        NOT NULL PRIMARY KEY,
    call_id       TEXT        NOT NULL REFERENCES recruitment_calls (call_id),
    respondent    TEXT        NOT NULL,  -- the role wire id
    response_kind TEXT        NOT NULL,  -- join | observe | decline | defer |
                                         -- conditional_join | recommend |
                                         -- request_context | recuse
    payload       JSONB       NOT NULL DEFAULT '{}'::jsonb,  -- the reason/until/
                                                             -- requirements/target
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (call_id, respondent)
);

CREATE TABLE recruitment_panels (
    call_id     TEXT        NOT NULL PRIMARY KEY REFERENCES recruitment_calls (call_id),
    panel       JSONB       NOT NULL,  -- the chosen role ids, ranked
    explanation JSONB       NOT NULL,  -- per-panelist: the stage-1 reasons + the
                                       -- stage-2 features (the visibility-safe
                                       -- explanations)
    snapshotted_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
