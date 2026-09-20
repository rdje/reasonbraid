-- 0079_node_presence_in_flight.sql — SIGNOFF-REPAIR.11.24.1.2:
-- §10.2 requires six presence states and the deployment could report five.
--
-- ⛔ THE DEFECT. `presence.rs` declares `PresenceState::Busy`, renders it as
-- the wire string `"busy"`, and CONSTRUCTS IT NOWHERE — the module says so
-- itself, deferring to a capacity lane whose leaf (`PHASE-3.4`) has closed.
-- §10.2 is not a menu: *a role can be `available`, `busy`, `draining`,
-- `offline`, `suspended`, or `unknown`*. Six states, `busy` among them. So the
-- defect is not an over-declared variant to delete — deleting it would take the
-- code OUT of §10.2 conformance to make a vocabulary honest — it is a REQUIRED
-- state the deployment cannot report.
--
-- ⭐ AND `migrations/0075` MADE IT DERIVABLE, which is why it can close now.
-- §10.2's `busy` means AT CAPACITY, and both inputs are shipped facts: the
-- profile's declared `concurrency`, and the number of commands the node HOLDS
-- and has not finished. Before `0075` the state meaning *the node process
-- durably holds this* was called `acknowledged`, and counting
-- acknowledged-not-consumed rows as *in flight* would have been an inference
-- from a name that did not mean that.
--
-- ⭐ THE IN-FLIGHT SET IS DEFINED BY THE LADDER, NOT BY A LIST OF EXCLUSIONS.
-- It is exactly `node_inbox_state.delivery_state = 'transport_received'`, and
-- every exclusion the definition needs comes free from that view's own
-- precedence rather than from a list somebody has to maintain:
--
--   * `consumed` OUTRANKS it, so finished work is excluded — a `work_result`
--     landed;
--   * `dead_lettered` outranks it, so a quarantined row is excluded — it will
--     never be worked, and counting it would hold a node at capacity forever;
--   * `revoked` and `expired` outrank it, so a row whose authority ended is
--     excluded — it is withheld from delivery and will not be worked;
--   * `queued` and `offered` rank BELOW it and are excluded by not reaching it.
--
-- ⛔ `offered` is the one worth arguing rather than assuming. A row the server
-- put on the wire and the node has not confirmed holding is NOT work the node
-- holds — that is exactly what the rung means — and counting it would leave a
-- node with a lossy connection permanently `busy` over rows it never received.
--
-- ⭐ IT IS A VIEW COLUMN RATHER THAN A FIFTH ARGUMENT AT FIVE CALL SITES.
-- All five readers of `presence_state` already select from `node_presence`, and
-- each already pays a correlated subquery for `concurrency`; one definition
-- here is one place to be wrong instead of five.
--
-- ⚠️ `CREATE OR REPLACE VIEW` is legal here and it was checked rather than
-- assumed: this view NAMES its columns, so the new one APPENDS, which is the
-- one shape a replacement may take. `node_inbox_state` selects `i.*` and had to
-- be dropped and recreated for exactly that reason (`migrations/0078`).

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
       AS suspended,
       -- The commands this node HOLDS and has not finished. See the header: the
       -- ladder's precedence is the exclusion list.
       (SELECT count(*) FROM node_inbox_state i
         WHERE i.node_id = n.node_id
           AND i.delivery_state = 'transport_received') AS in_flight
FROM nodes n
LEFT JOIN node_leases l ON l.node_id = n.node_id;
