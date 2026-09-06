-- 0004_authority.sql — the WP5 (.5.1) development authority store.
--
-- ROADMAP.md §4.4: the enrollment boundary is the root/parent-granted CEILING; a
-- grant is a scoped mandate under it; every command's authorization decision is an
-- audit record bound to the policy digest + version it was evaluated against.
-- Development profile: the control plane's own database is the authority store (no
-- certificate issuer — WP5 uses development credentials), one ACTIVE boundary per
-- tenant, and grants are keyed by (tenant, subject).

CREATE TABLE enrollment_boundaries (
    boundary_id              TEXT PRIMARY KEY,
    tenant_id                TEXT NOT NULL,
    parent_or_root_authority TEXT NOT NULL,
    target_owner             TEXT NOT NULL,
    permitted_actions        JSONB NOT NULL,  -- array of GrantAction wire names
    permitted_domains        JSONB NOT NULL,
    risk_ceiling             TEXT NOT NULL,
    spend_ceiling            JSONB,
    delegable                BOOLEAN NOT NULL,
    max_delegation_depth     INTEGER NOT NULL,
    valid_from               TIMESTAMPTZ NOT NULL,
    expires_at               TIMESTAMPTZ NOT NULL,
    charter_digest           TEXT NOT NULL,
    policy_version           TEXT NOT NULL,
    status                   TEXT NOT NULL
);

-- The dev profile has ONE active boundary per tenant (the unique partial index is the
-- serialization point — a second active boundary is refused).
CREATE UNIQUE INDEX enrollment_boundaries_tenant_active_idx
    ON enrollment_boundaries (tenant_id) WHERE status = 'active';

CREATE TABLE authority_grants (
    grant_id     TEXT PRIMARY KEY,
    boundary_id  TEXT NOT NULL REFERENCES enrollment_boundaries (boundary_id),
    tenant_id    TEXT NOT NULL,
    issuer       TEXT NOT NULL,
    subject_kind TEXT NOT NULL,  -- 'human' | 'role'
    subject_id   TEXT NOT NULL,
    actions      JSONB NOT NULL, -- array of GrantAction wire names
    selector     JSONB NOT NULL, -- TargetSelector wire form (tagged)
    risk_ceiling TEXT NOT NULL,
    spend_limits JSONB,
    delegable    BOOLEAN NOT NULL,
    valid_from   TIMESTAMPTZ NOT NULL,
    expires_at   TIMESTAMPTZ NOT NULL,
    status       TEXT NOT NULL
);

CREATE INDEX authority_grants_subject_idx
    ON authority_grants (tenant_id, subject_kind, subject_id);

-- The audit record every command leaves (§4.5): actor, subject if delegated, the
-- grant + boundary referenced, the decision (with the denial reason), and the policy
-- digest + version the decision was evaluated against. Denials are recorded too —
-- a refused command is still an audited event.
CREATE TABLE authorization_records (
    record_id      TEXT PRIMARY KEY,
    tenant_id      TEXT NOT NULL,
    actor          TEXT NOT NULL,
    subject_kind   TEXT,
    subject_id     TEXT,
    boundary_id    TEXT,
    grant_id       TEXT,
    action         TEXT NOT NULL,
    target_kind    TEXT NOT NULL,
    target_tenant  TEXT NOT NULL,
    target_thread  TEXT,
    decision       TEXT NOT NULL,  -- 'allowed' | 'denied'
    reason         TEXT,
    policy_digest  TEXT NOT NULL,
    policy_version TEXT NOT NULL,
    decided_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX authorization_records_tenant_idx ON authorization_records (tenant_id);
