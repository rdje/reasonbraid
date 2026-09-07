-- 0022_recruitment_offers.sql — PHASE-3.5.2: the call's advertisement record —
-- the subscribers whose DECLARED interests match the call's topic tags, the
-- server-recorded offer (the §10.5 advertisement window's durable trace; the
-- role's profile interests become ACTIONABLE subscriptions).

CREATE TABLE recruitment_offers (
    offer_id    TEXT        NOT NULL PRIMARY KEY,
    call_id     TEXT        NOT NULL REFERENCES recruitment_calls (call_id),
    role_id     TEXT        NOT NULL REFERENCES agent_roles (role_id),
    offered_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (call_id, role_id)
);
