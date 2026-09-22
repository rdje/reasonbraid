-- 0086_publication_git_operation.sql — SIGNOFF-REPAIR.9.3.5.1.1: a publication
-- records its Git operation BEFORE attempting it.
--
-- ROADMAP §15.7 step 4 stores the *desired Git operation* in the database before
-- the Git write. Before this migration the publish verb wrote first and recorded
-- after: the repository rode the REQUEST and was never stored, and neither was
-- the effective id the compare-and-swap expected. A publication interrupted
-- after its Git write could not be located, so the §15.8 reconciler had nothing
-- to observe
-- (`docs/decisions/2026-09-22_a-publication-records-its-git-operation-before-attempting-it.md`).
--
-- `repository` is ROOT-RELATIVE (§12): the path inside the deployment's
-- configured publication root, never an absolute path that would break when
-- the root moves. `expected_effective` is the object id the effective channel's
-- compare-and-swap expected; NULL means it expected NO effective ref.
--
-- ⛔ Both are NULL on every row published or staged before this migration, and
-- that is the honest reading: nothing recorded where those went. They are not
-- backfilled from guesses. Adding nullable columns cannot fail against
-- existing rows.

ALTER TABLE policy_publications ADD COLUMN repository TEXT;
ALTER TABLE policy_publications ADD COLUMN expected_effective TEXT;
