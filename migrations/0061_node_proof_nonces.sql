-- SIGNOFF-REPAIR.4.2.2: a captured certificate proof replays until the
-- certificate expires.
--
-- Measured, not read. The handshake coverage is
-- {channel_version, node_id, last_acked_cursor, pending_operations,
-- ambiguous_attempts} and the rotate coverage is
-- {channel_version, node_id, cert_der} — the second entirely STATIC for a given
-- node and certificate. Neither carries a nonce, a timestamp or a server
-- challenge, so a captured request is byte-identical to a valid one. Driven:
-- the replayed handshake answered 200, took a new lease, and the legitimate
-- node's next heartbeat answered 401 — it was fenced out of its own session;
-- the replayed rotate answered 200 twice and returned TWO DISTINCT PRIVATE KEYS
-- from one captured request.
--
-- The decision is a per-request nonce, consumed once, rather than a server
-- challenge. Reasons, in order:
--
--  * it costs NO extra round trip, where a challenge costs one on every
--    handshake and every rotation;
--  * it needs no clock. A timestamp-and-skew-window design would put a clock
--    dependency on the authentication path, and this repository has just
--    repaired three separate cross-clock defects (`.3.4.3`, `.3.4.3.1.2`,
--    `.3.4.3.1.3`) — that is evidence, not taste;
--  * the pattern is already this codebase's: `node_enrollment_tokens.nonce` is
--    issued once and echoed at use.
--
-- ⛔ Consuming the CERTIFICATE instead — one rotation per certificate, no new
-- table — was considered and rejected: a node whose rotate response was lost
-- retries with the same certificate, and that retry is indistinguishable from a
-- replay, so the rule would break the §17.4 recovery path it must not touch. A
-- retry carries a FRESH nonce, so this design admits it and simply issues
-- another certificate, which rotation's additive contract already allows.

-- ⛔ `node_id` carries NO foreign key to `nodes`, and that is deliberate rather
-- than an omission. The first draft had one, and it DEADLOCKED the rotation
-- against itself: the rotate transaction holds `nodes … FOR UPDATE`
-- (`SIGNOFF-REPAIR.4.2.1`), and the foreign-key check on this insert needs a
-- KEY SHARE lock on that same row. Worse than the deadlock was what the key
-- implied even when it did not deadlock — every handshake would have had to
-- queue behind any in-flight revocation of that node, which is a coupling on
-- the authentication path that nothing asked for.
--
-- The integrity a key would buy is close to nothing here: these rows are
-- transient bookkeeping, nothing joins them, and `node_id` exists only to scope
-- the prune. A nonce for a node that no longer exists is pruned like any other.
CREATE TABLE node_proof_nonces (
    -- The node-chosen nonce, inside the SIGNED coverage — a replayer cannot
    -- change it without invalidating the signature, which is what makes
    -- consuming it a replay defence rather than a formality.
    nonce    TEXT        NOT NULL PRIMARY KEY,
    node_id  TEXT        NOT NULL,
    seen_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

COMMENT ON TABLE node_proof_nonces IS
    'Consumed handshake/rotate proof nonces. A proof is admitted only if its '
    'nonce inserts here, so a captured request is refused the second time it is '
    'presented. Retention is bounded per node at first use: a replay is useless '
    'once the certificate it presents has expired, so rows outlive that by a '
    'wide margin and are pruned on the next proof from the same node.';

CREATE INDEX node_proof_nonces_prune_idx ON node_proof_nonces (node_id, seen_at);
