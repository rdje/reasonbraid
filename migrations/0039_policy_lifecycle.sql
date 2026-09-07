-- The policy-lifecycle records (`.2.2`, ADR-032): the five records NEVER
-- fold. The proposal is a REFERENCE (the policy version + the deliberation
-- thread — never a content copy); the decision freezes the ELECTORATE
-- snapshot (the §4.5 "at action time" facts) + the verdict reference; the
-- proposal's status machine is the typed stage vocabulary (the `.2.3`
-- approval, the `.4` publication, and the `.5` deployment advance it).
CREATE TABLE policy_proposals (
    proposal_id    TEXT        NOT NULL,
    policy_id      TEXT        NOT NULL,
    policy_version TEXT        NOT NULL,
    thread_id      TEXT        NOT NULL,
    status         TEXT        NOT NULL DEFAULT 'draft',
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (proposal_id)
);

CREATE TABLE policy_decisions (
    decision_id      TEXT        NOT NULL,
    proposal_id      TEXT        NOT NULL,
    rule             TEXT        NOT NULL,
    electorate       JSONB       NOT NULL,
    verdict_event_id TEXT        NOT NULL,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (decision_id)
);
