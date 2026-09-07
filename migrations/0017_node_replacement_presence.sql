-- 0017_node_replacement_presence.sql — the PHASE-2.7.2 replacement ritual: the
-- presence derivation distinguishes a DEAD node (every certificate revoked) from
-- a REPLACED node (the old certs stay revoked — still fenced — while a fresh
-- active cert serves the new incarnation). 0012's "any revoked cert" reading
-- would keep the replacement suspended forever, so the view now reads:
-- suspended = a revoked cert exists AND no active cert exists.

CREATE OR REPLACE VIEW node_presence AS
SELECT n.node_id,
       n.tenant_id,
       n.host_id,
       (l.lease_expires_at IS NOT NULL AND l.lease_expires_at > now()) AS online,
       l.last_seen_at,
       l.lease_expires_at,
       (EXISTS (SELECT 1 FROM node_certificates c
                WHERE c.node_id = n.node_id AND c.revoked_at IS NOT NULL)
        AND NOT EXISTS (SELECT 1 FROM node_certificates c
                        WHERE c.node_id = n.node_id AND c.revoked_at IS NULL))
       AS suspended
FROM nodes n
LEFT JOIN node_leases l ON l.node_id = n.node_id;
