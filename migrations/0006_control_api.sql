-- 0006_control_api.sql — the WP6 (.6.1) control-API tables.
--
-- The enroll surface persists the development principals the CLI resolves by NAME
-- into their wire ids (the CLI keeps the name -> id mapping in its local state dir;
-- the server keeps the authoritative rows). (tenant, kind, name) is unique so a
-- re-run of the same enroll returns the ORIGINAL principal id (idempotent bootstrap).
CREATE TABLE enrollments (
    principal_id TEXT        NOT NULL,
    tenant_id    TEXT        NOT NULL,
    kind         TEXT        NOT NULL CHECK (kind IN ('human', 'role')),
    name         TEXT        NOT NULL,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (principal_id),
    UNIQUE (tenant_id, kind, name)
);
