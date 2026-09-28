-- 0117_a_retention_class_is_one_the_sweep_knows.sql — SIGNOFF-REPAIR.11.53.
--
-- `retention_class` was any string. The expiry sweep matched `standard` and
-- `temporary`, so a snapshot submitted with any other class was kept for ever,
-- while `evidence.md` says a snapshot expires by its class. The submit now
-- refuses any class but these three, and this constraint holds every row to
-- the same set, whoever writes it.
--
-- ⛔ A row written before the check with a class nobody knows becomes
-- `standard`: its author's intent is not recoverable, and `standard` is the
-- class the store defaults to and the sweep treats as the ordinary lifetime.
-- It is converted rather than left, because a NOT VALID constraint is still
-- enforced on UPDATE: the sweep's own tombstone write on such a row would
-- violate it, and one such row would fail the whole sweep. Measured before
-- this migration was written that way.
UPDATE evidence_snapshots
   SET retention_class = 'standard'
 WHERE retention_class NOT IN ('standard', 'temporary', 'audit');

ALTER TABLE evidence_snapshots
    ADD CONSTRAINT evidence_snapshots_retention_class_known
    CHECK (retention_class IN ('standard', 'temporary', 'audit'));
