-- 0085_policy_decision_derivation.sql — SIGNOFF-REPAIR.11.4.7.2.1.2.3.1: a
-- policy decision records its thread's COUNTED close instead of whatever it
-- was told.
--
-- Before this migration `record_decision` stored the caller's `rule` (free
-- text) and `electorate` (JSON) verbatim, and checked only that a `verdict`
-- contribution existed in the proposal's thread. ROADMAP §15.6 places a
-- *deterministic decision* between deliberation and approval, and since
-- `SIGNOFF-REPAIR.8.1.1` the thread's close IS that decision: derived from a
-- charter-checked rule, a fixed electorate and counted ballots.
--
-- `derivation` holds what the decision was derived FROM — the thread's close
-- outcome, the rule's charter threshold and the charter's digest, and the
-- tally. ⛔ It is NULL on every row written before this migration, and that
-- is the honest reading of those rows: their `rule` and `electorate` are the
-- caller's, and nothing rewrites them.
--
-- `verdict_event_id` becomes optional: an adjudicator's verdict is supporting
-- evidence (ADR-029), no longer a stand-in for the decision. It cannot fail
-- against existing rows — relaxing NOT NULL admits every value it admitted.

ALTER TABLE policy_decisions ADD COLUMN derivation JSONB;
ALTER TABLE policy_decisions ALTER COLUMN verdict_event_id DROP NOT NULL;
