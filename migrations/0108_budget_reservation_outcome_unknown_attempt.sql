-- 0108_budget_reservation_outcome_unknown_attempt.sql — SIGNOFF-REPAIR.4.5.1.1:
-- WHICH attempt's unknown outcome keeps a hold counted.
--
-- 0107 stamped a reservation `outcome_unknown_at` when its node dead-lettered
-- the work item as `retry_requires_authorization`, and nothing ever un-stamped
-- it: an operator's verdict closed the ambiguous-attempt row and left the
-- allowance counted for good. Applying a verdict must find the hold the verdict
-- is ABOUT, and the register keys attempts by (node, attempt). The operation is
-- the wrong key: a node keeps one operation per command, so after a
-- possible-duplicate replay the original attempt and the re-run share it, each
-- with its own reservation. The dead letter now names its attempt, and the
-- stamp records both halves of that key.
ALTER TABLE budget_reservations
    ADD COLUMN outcome_unknown_node_id    TEXT,
    ADD COLUMN outcome_unknown_attempt_id TEXT;
CREATE INDEX budget_reservations_outcome_unknown_idx
    ON budget_reservations (outcome_unknown_node_id, outcome_unknown_attempt_id)
    WHERE outcome_unknown_attempt_id IS NOT NULL;
