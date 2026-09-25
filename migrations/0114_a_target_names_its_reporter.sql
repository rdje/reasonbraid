-- A target names the one principal that files its receipts
-- (`SIGNOFF-REPAIR.9.3.3.2`, `docs/decisions/2026-09-25_a-target-names-its-reporter.md`).
-- Until here any principal of the owning tenant wrote the observed half of a
-- deployment, which is the drift comparison's input.
--
-- ⚠️ NULL means *registered before this migration*: such a target names no
-- reporter and takes no receipt. Nothing is inferred for it, because a guessed
-- reporter is a principal nobody named, which is the defect this column closes.
ALTER TABLE deployment_targets ADD COLUMN reporter TEXT;
