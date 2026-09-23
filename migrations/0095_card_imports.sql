-- 0095_card_imports.sql — SIGNOFF-REPAIR.5.3.2 (ADR-026, ROADMAP §20.10): an import
-- is identified by its ORIGIN, and the imported role's provenance is on the
-- ledger.
--
-- Until this table the import's only identity was the card's `display_label`,
-- through `agent_roles (tenant_id, name)`: the same card twice was refused as a
-- taken label, the same origin role under a new label imported AGAIN as a
-- second local identity, and an unrelated card carrying a taken label was
-- refused as if it were a repeat. The origin ROLE id was stored nowhere — the
-- receipt carries the origin tenant and the card digest — so which origin a
-- local role came from was derivable only by whoever still held the card.
--
-- One row per imported role, written in the import's transaction: the origin
-- pair, the digest of the card that landed, the admission it ran under, and
-- when. The UNIQUE key is what makes a repeat a REPLAY (the enrolment route's
-- shape) rather than a second identity; it is read under the importing
-- tenant's exclusive guard, which serializes that tenant's imports, so the key
-- is the backstop and never the detector. The origin tenant references
-- `tenants` because within one deployment the allowlist rung already requires
-- the origin to be a tenant here; federation between deployments is post-v1.
CREATE TABLE card_imports (
    role_id          TEXT        NOT NULL PRIMARY KEY REFERENCES agent_roles (role_id),
    tenant_id        TEXT        NOT NULL REFERENCES tenants (tenant_id),
    origin_tenant_id TEXT        NOT NULL REFERENCES tenants (tenant_id),
    origin_role_id   TEXT        NOT NULL,
    card_digest      TEXT        NOT NULL,  -- the presented digest, verified against the card's bytes
    record_id        TEXT        NOT NULL,  -- the admission the import ran under
    imported_at      TIMESTAMPTZ NOT NULL,
    UNIQUE (tenant_id, origin_tenant_id, origin_role_id)
);
