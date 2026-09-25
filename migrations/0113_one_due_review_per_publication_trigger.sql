-- 0113_one_due_review_per_publication_trigger.sql — SIGNOFF-REPAIR.9.3.2
-- (ROADMAP §15.11): a publication's review lifecycle RECURS.
--
-- A review's id was `rev_{publication}_{trigger}` and the id is the primary
-- key, so once a pair's first review was done, every later review for the pair
-- collided with it — and the scheduler discarded the insert error. A pair could
-- be reviewed once, for ever. Reviews now carry their own ids, and the rule the
-- deterministic id was standing in for — at most one DUE review per
-- (publication, trigger) — is stated where concurrent schedules cannot race
-- past it: here, as a partial unique index the scheduler's insert names in its
-- `ON CONFLICT`.
--
-- ⭐ Every existing row satisfies it: the scheduler skipped a pair with a due
-- review, and the old primary key allowed only one row per pair at all.
CREATE UNIQUE INDEX policy_reviews_one_due
    ON policy_reviews (publication_id, trigger)
    WHERE status = 'due';
