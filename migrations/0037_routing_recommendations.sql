-- The shadow routing recommendations (`.5.3`, ADR-031): the learned-routing
-- surface — a recommendation maps a class to an arm drawn from the EXISTING
-- registered profiles (never a raise of authority/spend/access/side-effects),
-- names its evidence reference (the `.4` trial/gate it rests on), and is
-- RECORDED, never applied (the create boundary keeps resolving the RULE
-- table — the recommendation is evidence the future policy gate weighs).
CREATE TABLE routing_recommendations (
    recommendation_id TEXT        NOT NULL,
    case_class       TEXT        NOT NULL,
    arm              TEXT        NOT NULL,
    evidence_ref     TEXT        NOT NULL,
    recorded_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (recommendation_id)
);
