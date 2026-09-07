-- 0032_workflow_profiles.sql — PHASE-5.1.2: the workflow-profile registry
-- (ADR-016 + ROADMAP §13.1). A profile is VERSIONED CONFIGURATION over the
-- thread aggregates: the steps compose the EXISTING verbs — never a new
-- capability. The eight §13.1 built-ins ship as the versioned entries; the
-- custom profiles validate against the same invariants.

CREATE TABLE workflow_profiles (
    profile_id TEXT        NOT NULL,
    version    INTEGER     NOT NULL,
    steps      JSONB       NOT NULL,
    built_in   BOOLEAN     NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (profile_id, version)
);

INSERT INTO workflow_profiles (profile_id, version, steps, built_in) VALUES
    ('quick_advice', 1, '["solicit", "synthesize", "decide"]', true),
    ('independent_panel', 1, '["blind_solicit", "adjudicate", "decide"]', true),
    ('critique', 1, '["solicit", "critique", "revise", "decide"]', true),
    ('evidence_review', 1, '["solicit", "evidence_request", "assess", "decide"]', true),
    ('architecture_decision', 1, '["solicit", "critique", "adjudicate", "decide"]', true),
    ('incident_review', 1, '["solicit", "evidence_request", "synthesize", "decide"]', true),
    ('policy_proposal', 1, '["solicit", "revise", "assess", "vote", "approve"]', true),
    ('retrospective', 1, '["solicit", "synthesize", "retrospect", "decide"]', true);
