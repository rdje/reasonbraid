-- The policy-approval records (`.2.3`, ADR-032): the approval NEVER folds
-- into the decision — its own row carries the AUTHORITY PROOF (the grant id
-- RE-CHECKED at the approval boundary: the status, the expiry, and the
-- subject match — the §4.5 identity/authority at the action time) and the
-- QUORUM snapshot. The approval advances the proposal (`decided` →
-- `approved`).
CREATE TABLE policy_approvals (
    approval_id TEXT        NOT NULL,
    proposal_id TEXT        NOT NULL,
    decision_id TEXT        NOT NULL,
    approver    TEXT        NOT NULL,
    grant_id    TEXT        NOT NULL,
    quorum      JSONB       NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (approval_id)
);
