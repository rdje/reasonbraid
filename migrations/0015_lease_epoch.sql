-- 0015_lease_epoch.sql — lease/fencing hardening (PHASE-2.2.2).
--
-- The renewal race: a heartbeat verifies the fencing token, then a concurrent
-- handshake rotates the lease, then the heartbeat's UPDATE lands — extending the
-- NEW session's lease on behalf of a process its own handshake just fenced. The
-- lease EPOCH closes it: every handshake bumps the epoch (with the token), and
-- every fenced write carries the epoch it SAW — a write from a fenced epoch
-- matches no row and is refused, so the last writer can never be a stale one.
-- The epoch also closes the check-vs-commit window on the events path: the
-- transaction re-verifies (token + epoch) FOR UPDATE, so a rotation that lands
-- between the admission check and the apply is observed, not lost.

ALTER TABLE node_leases ADD COLUMN lease_epoch BIGINT NOT NULL DEFAULT 0;
