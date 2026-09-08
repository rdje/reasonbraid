-- 0049_cross_domain_receipts.sql — the cross-domain audit receipts
-- (`PHASE-8.1.4`, ADR-026): the cross-domain actions record a RECEIPT —
-- the remote domain's own digest-pinned reference + the local record it
-- attached to. The receipts CROSS-REFERENCE, never merge: the remote
-- reference is verifiable against the REMOTE domain's records; the local
-- chain stays the local truth (the ADR-022 groundwork's federation form).

CREATE TABLE cross_domain_receipts (
    receipt_id       TEXT        PRIMARY KEY,
    tenant_id        TEXT        NOT NULL REFERENCES tenants (tenant_id),
    remote_tenant_id TEXT        NOT NULL REFERENCES tenants (tenant_id),
    kind             TEXT        NOT NULL CHECK (kind IN ('card_import', 'agreement')),
    remote_ref       TEXT        NOT NULL, -- the remote record's digest-pinned reference
    local_ref        TEXT        NOT NULL, -- the local record it attached to
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (tenant_id, remote_tenant_id, kind, remote_ref)
);
