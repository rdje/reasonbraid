-- Coordination anchors, not tenant identities or permission grants. Authority
-- namespaces predate the identity store and intentionally have no tenant FK.
CREATE TABLE tenant_authority_guards (
    tenant_id TEXT COLLATE "C" PRIMARY KEY
);

INSERT INTO tenant_authority_guards (tenant_id)
SELECT tenant_id COLLATE "C" FROM tenants
UNION SELECT tenant_id COLLATE "C" FROM enrollment_boundaries
UNION SELECT tenant_id COLLATE "C" FROM authority_grants
UNION SELECT tenant_id COLLATE "C" FROM authorization_records;

COMMENT ON TABLE tenant_authority_guards IS
    'Persistent full-tenant coordination keys. No identity or authority is conferred; do not delete during tenant authority use.';
