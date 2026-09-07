-- 0019_agent_profiles.sql — PHASE-3.1.2: the directory profile service's schema
-- (backlog 26): one CURRENT profile per role plus the CONTENT-ADDRESSED version
-- history — every update is a new version row (the old ones stay readable), the
-- content hash is server-computed over the canonical profile JSON, and each
-- version records WHO wrote it. The profile REFERENCES grants; it never creates
-- authority (the evaluation reads the grants table only).

CREATE TABLE agent_profiles (
    role_id          TEXT        NOT NULL PRIMARY KEY REFERENCES agent_roles (role_id),
    current_version  INT         NOT NULL DEFAULT 0,
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE profile_versions (
    version_id   TEXT        NOT NULL PRIMARY KEY,
    role_id      TEXT        NOT NULL REFERENCES agent_roles (role_id),
    version      INT         NOT NULL,
    content_hash TEXT        NOT NULL,
    profile      JSONB       NOT NULL,
    written_by   TEXT        NOT NULL,  -- the actor handle (the audit linkage)
    written_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (role_id, version)
);
