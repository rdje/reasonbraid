-- 0014_run_attempt_link.sql — run writer support (PHASE-2.1.6.2; deferral #4's second half).
--
-- A `runs` row links an attempt to the incarnation that ran it. The attempt id
-- rides the node-emitted `work_result` payload (the node's local journal fact);
-- the server records it when the result receipt lands (the idempotency claim
-- dedupes redelivery BEFORE this write, so one result = one run, ever).

ALTER TABLE runs ADD COLUMN attempt_id TEXT;
