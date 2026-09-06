-- 0008_node_enrollment.sql — the dev-profile node enrollment store (PHASE-1.2.1; backlog 11).
--
-- ROADMAP.md §16.2: the one-time enrollment token is bound to tenant, host claim,
-- expected node id, expiry, and nonce. Certificate issuance (X.509/mTLS) is
-- EXPLICITLY deferred to Phase 2 (ADR-007); the Phase 1 credential is the token plus
-- a dev shared secret whose fingerprint the server records — the server IS the dev
-- trust store (the same documented stance as the .6.1 dev principal header). The
-- HMAC key-proof over the channel handshake arrives with .1.2.2.

-- The host get-or-create key for enrollment (0007 left hosts unconstrained).
CREATE UNIQUE INDEX hosts_tenant_name_idx ON hosts (tenant_id, name) WHERE name IS NOT NULL;

CREATE TABLE node_enrollment_tokens (
    token_id   TEXT        NOT NULL PRIMARY KEY,
    tenant_id  TEXT        NOT NULL REFERENCES tenants (tenant_id),
    node_id    TEXT        NOT NULL UNIQUE,  -- bound to the expected node at issuance
    host_claim TEXT        NOT NULL,         -- the host the token may enroll from
    nonce      TEXT        NOT NULL,         -- echoed by the node at use
    expires_at TIMESTAMPTZ NOT NULL,
    used_at    TIMESTAMPTZ,                  -- NULL = unused; one use per token, ever
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- The node's dev signing key (one active key per node in the dev profile; rotation
-- and revocation are Phase 2's — they extend this table, not a guessed shape now).
CREATE TABLE node_keys (
    node_id         TEXT        NOT NULL PRIMARY KEY REFERENCES nodes (node_id),
    key_fingerprint TEXT        NOT NULL,    -- sha256 hex of the dev secret
    key_secret      TEXT        NOT NULL,    -- dev profile: the server stores the secret
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- The enrollment audit: every attempt — enrolled or refused — leaves a row.
CREATE TABLE node_enroll_audit (
    record_id TEXT        NOT NULL PRIMARY KEY,
    tenant_id TEXT        NOT NULL,
    node_id   TEXT        NOT NULL,
    token_id  TEXT        NOT NULL,
    decision  TEXT        NOT NULL,          -- 'enrolled' | 'refused'
    reason    TEXT,
    at        TIMESTAMPTZ NOT NULL DEFAULT now()
);
