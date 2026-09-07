-- 0023_resource_references.sql — PHASE-4.1.2: the §12.1 universal reference
-- contract as a durable table. The original locator is IMMUTABLE (the
-- update-refusal is the absent verb + the same-locator/different-digest
-- conflict); the credential-binding ref is opaque (never a secret).

CREATE TABLE resource_references (
    resource_id                TEXT        NOT NULL PRIMARY KEY,
    original_locator           TEXT        NOT NULL,
    scheme                     TEXT        NOT NULL,
    media_type_hint            TEXT,
    expected_digest            TEXT,  -- the ADR-011 `sha256:<hex>` scheme
    fragment_or_selector       TEXT,
    credential_binding_ref     TEXT,  -- opaque; never a secret
    owning_node_or_capability  TEXT,
    visibility_scope           TEXT        NOT NULL DEFAULT 'network',
    purpose                    TEXT,
    retention_class            TEXT,
    risk_class                 TEXT        NOT NULL DEFAULT 'low',
    submitted_by               TEXT        NOT NULL,  -- the actor handle
    created_at                 TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (original_locator, expected_digest)
);
