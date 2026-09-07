-- 0011_workload_certificates.sql — the PHASE-2.1.2.1 identity issuance (ADR-007):
-- the control plane's project-local CA and the short-lived workload leaves it issues
-- at enrollment. The CA is ONE row (id 1) per deployment; the leaf's CN is the durable
-- node id and its SAN is the enrollment token's host claim, so the certificate rides
-- the identity (§16.2) instead of replacing it.

CREATE TABLE server_ca (
    ca_id      INTEGER     NOT NULL PRIMARY KEY,
    ca_der     BYTEA       NOT NULL,
    -- Dev/Trusted-LAN profile (ADR-007 consequences): the CA key lives with the
    -- control plane. Its compromise = re-issue, a documented recovery operation.
    key_der    BYTEA       NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE node_certificates (
    -- sha256 of the leaf DER — the node-id -> current-fingerprint binding the
    -- handshake gates on (ADR-007). Rotation adds rows additively; revocation
    -- (PHASE-2.1.3) sets revoked_at.
    cert_fingerprint TEXT        NOT NULL PRIMARY KEY,
    node_id          TEXT        NOT NULL REFERENCES nodes (node_id),
    cert_der         BYTEA       NOT NULL,
    -- Dev escrow: the server generates the node's keypair (the `.1.2.1` dev
    -- trust-store stance; the Internet profile re-evaluates — ADR-007's trigger).
    key_der          BYTEA       NOT NULL,
    issued_at        TIMESTAMPTZ NOT NULL,
    expires_at       TIMESTAMPTZ NOT NULL,
    revoked_at       TIMESTAMPTZ
);

CREATE INDEX node_certificates_node_idx ON node_certificates (node_id);
