-- 0009_node_leases.sql — the authenticated node channel's lease/presence store
-- (PHASE-1.2.2; ROADMAP backlog 13's heartbeat/lease remainder).
--
-- One live lease per node: the fencing token the handshake issued, the expiry
-- clock, and the last contact time. Presence is DERIVED from the expiry clock —
-- no background updater flips an `offline` flag — so a crashed process cannot
-- leave a stale `online` row behind (backlog 27's stale handling, Phase 1 scope).
CREATE TABLE node_leases (
    node_id          TEXT        NOT NULL PRIMARY KEY REFERENCES nodes (node_id),
    fencing_token    TEXT        NOT NULL,
    lease_expires_at TIMESTAMPTZ NOT NULL,
    last_seen_at     TIMESTAMPTZ NOT NULL,
    issued_at        TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- The presence view: `online` is a function of time. The fencing token is
-- deliberately NOT exposed here — presence is an observability fact, the token
-- is a credential.
CREATE VIEW node_presence AS
SELECT n.node_id,
       n.tenant_id,
       n.host_id,
       (l.lease_expires_at IS NOT NULL AND l.lease_expires_at > now()) AS online,
       l.last_seen_at,
       l.lease_expires_at
FROM nodes n
LEFT JOIN node_leases l ON l.node_id = n.node_id;
