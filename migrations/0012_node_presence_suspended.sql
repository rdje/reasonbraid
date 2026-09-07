-- 0012_node_presence_suspended.sql — the PHASE-2.1.3.1 revocation observability:
-- the presence view gains `suspended` (a node with a revoked workload
-- certificate reads suspended, whatever its lease says). The revocation row
-- itself lives in `node_certificates` (0011) — this view only DERIVES it.

CREATE OR REPLACE VIEW node_presence AS
SELECT n.node_id,
       n.tenant_id,
       n.host_id,
       (l.lease_expires_at IS NOT NULL AND l.lease_expires_at > now()) AS online,
       l.last_seen_at,
       l.lease_expires_at,
       EXISTS (SELECT 1 FROM node_certificates c
               WHERE c.node_id = n.node_id AND c.revoked_at IS NOT NULL) AS suspended
FROM nodes n
LEFT JOIN node_leases l ON l.node_id = n.node_id;
