-- 0007_identity_store.sql — the first-class identity store (PHASE-1.1.2; backlog 10).
--
-- ROADMAP.md §8.1 names the identity hierarchy: Tenant; HumanPrincipal; Host ->
-- NodeInstance -> HarnessInstallation; AgentRole -> AgentIncarnation -> AgentSession/Run.
-- §17.2 requires tenant_id on every material record and sortable opaque ids (the core
-- Id<K> newtypes are UUIDv7; their wire forms are stored here as TEXT).
--
-- The .6.1 `enrollments` table stays what it was — the dev bootstrap's
-- (tenant, kind, name) -> principal-id map — and these tables become the identity
-- records themselves: an enrollment implies its identity row (written in the same
-- transaction, PHASE-1.1.2).
--
-- Deliberately minimal: columns that belong to later features (a node's workload
-- certificate state, a run's budget/provider receipts, an incarnation's live
-- attestation) arrive with those features, not here — the 0002 precedent: never
-- guess a shape early. The incarnation carries the §8.1 facts that DEFINE it
-- (provider/model/harness/config and its validity interval); hosts/nodes/runs
-- carry identity and lineage only.

CREATE TABLE tenants (
    tenant_id  TEXT        NOT NULL PRIMARY KEY,
    name       TEXT,           -- the dev bootstrap does not name tenants yet
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE human_principals (
    principal_id TEXT        NOT NULL PRIMARY KEY,
    tenant_id    TEXT        NOT NULL REFERENCES tenants (tenant_id),
    name         TEXT        NOT NULL,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (tenant_id, name)
);

CREATE TABLE agent_roles (
    role_id    TEXT        NOT NULL PRIMARY KEY,
    tenant_id  TEXT        NOT NULL REFERENCES tenants (tenant_id),
    name       TEXT        NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (tenant_id, name)
);

CREATE TABLE hosts (
    host_id    TEXT        NOT NULL PRIMARY KEY,
    tenant_id  TEXT        NOT NULL REFERENCES tenants (tenant_id),
    name       TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE nodes (
    node_id    TEXT        NOT NULL PRIMARY KEY,
    host_id    TEXT        NOT NULL REFERENCES hosts (host_id),
    tenant_id  TEXT        NOT NULL REFERENCES tenants (tenant_id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE incarnations (
    incarnation_id TEXT        NOT NULL PRIMARY KEY,
    role_id        TEXT        NOT NULL REFERENCES agent_roles (role_id),
    tenant_id      TEXT        NOT NULL REFERENCES tenants (tenant_id),
    provider       TEXT,       -- §8.1: the actual provider/model/harness/configuration
    model          TEXT,
    harness        TEXT,
    config         JSONB,
    valid_from     TIMESTAMPTZ,
    valid_to       TIMESTAMPTZ,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE runs (
    run_id         TEXT        NOT NULL PRIMARY KEY,
    incarnation_id TEXT        NOT NULL REFERENCES incarnations (incarnation_id),
    tenant_id      TEXT        NOT NULL REFERENCES tenants (tenant_id),
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);
