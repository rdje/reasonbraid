-- 0066_assessment_claim_namespace.sql — SIGNOFF-REPAIR.11.14.3.3
-- (`docs/decisions/2026-09-17_the-assessment-namespace-is-part-of-the-row.md`):
-- `claim_assessments.claim_id` holds two kinds of identifier, and the row now
-- says which.
--
-- `SIGNOFF-REPAIR.11.14.3.1` made a deliberation's `claim_id` the SERVER-MINTED
-- claim digest, membership-checked against the thread. `POST /v1/assessments`
-- was left standing deliberately — an assessment asserted outside any
-- deliberation may be legitimate, and removing a shipped route is a breaking
-- change ROADMAP §13.2 does not require — and still takes a free-text
-- `claim_id`. Two writers, two namespaces, one unlabelled column.
--
-- ⭐ The namespace joins the REPLAY KEY, and that is the repair rather than a
-- tidying. `claim_assessments_replay_idx` was `(claim_id, snapshot_id,
-- assessment, author)`; `author` is a caller-supplied label on the standalone
-- route, so a caller who typed a real thread's claim digest and that thread's
-- author matched all four columns and the pre-check in `claims::submit`
-- returned the DELIBERATION's `assessment_id`. Measured, not inferred: the
-- control `the_two_assessment_writers_are_two_namespaces` failed
-- `assert_ne!` with the same `asn_…` id on both sides before this migration.
-- An identifier two writers mint differently is not one identifier — the same
-- conclusion `0065` reached for a reference's `(locator, digest)` pair.
--
-- ⛔ NULLABLE, and for `0064`'s reason on this same table. A row written before
-- this migration has no recoverable namespace: the only evidence of which
-- writer produced it would be the shape of `claim_id`, and a caller could
-- always type the minted shape — which is exactly the collision this column
-- exists to record. NULL therefore means "unattributed namespace"; nothing
-- writes it from here.
--
-- ⛔ `NULLS NOT DISTINCT` (PostgreSQL 15+; this project pins 16) so the new key
-- is not WEAKER than the one it replaces. Under the default, two NULL-namespace
-- rows agreeing on the other four columns would stop colliding — rows the old
-- index forbade and no writer can now create, so the weakening is unreachable;
-- preserving the old guarantee exactly costs nothing and is not a judgement
-- call. `0065` takes the same clause for the same reason.
--
-- ⛔ NO `ON CONFLICT` infers this index — `git grep -n 'ON CONFLICT' --
-- crates/*/src` finds none naming these columns — so dropping and recreating it
-- breaks no statement. `claims::submit` pre-checks and inserts; the pre-check
-- gains the namespace in the same commit, because an index alone would not stop
-- the aliasing: the pre-check short-circuits before the insert ever runs.
--
-- ⛔ The index name is `0030`'s own, declared there explicitly rather than
-- generated, so this migration names a fact rather than a guess — the
-- distinction `0065` had to discover for a PostgreSQL-generated constraint name.
--
-- 🔴 `authored_by_tenant` JOINS THE KEY TOO, and that corrects a false premise
-- in `0064` rather than adding a refinement. `0064` justified taking a column
-- instead of a citation table by arguing that the replay key "carries the
-- AUTHOR, so two tenants asserting the same thing about the same evidence
-- already hold two separate rows" — and stated, eight lines further down the
-- SAME file, that "`author` and `verifier` remain unauthenticated caller
-- strings". Both cannot be true. Two tenants hold two rows only while they
-- happen to type different labels; a caller that presents ANOTHER tenant's
-- label matches the whole key, and `claims::submit`'s pre-check returns that
-- tenant's `assessment_id`. Measured, not inferred: with the namespace column
-- in place but the tenant absent, the control
-- `the_two_assessment_writers_are_two_namespaces` failed `assert_ne!` with
-- `asn_01a0ace7727c7b31bea32938bf807721` on both sides — a second tenant
-- holding the first tenant's row id.
--
-- ⭐ `0064`'s INTENT is what this implements: the column that makes "two tenants
-- hold two rows" true is the SERVER-set one, not the caller-set one. `0064`
-- itself is left byte-unchanged — a shipped migration's checksum is
-- load-bearing, and `docs/decisions/` supersedes rather than mutates.
--
-- ⛔ Making `author` itself trustworthy is NOT done here. `0064` defers that to
-- `SIGNOFF-REPAIR.7.4` by name, and it is a wire-contract change; this migration
-- stops `author` from being load-bearing for IDENTITY, which is a different and
-- smaller claim.
--
-- ⚠️ One case is deliberately left open and stated rather than implied: two
-- principals WITHIN one tenant can still alias each other on the standalone
-- route. That is not a disclosure — `0064`'s authoring gate already admits both
-- of them to that row — so it is a deduplication question, not a security one.

ALTER TABLE claim_assessments ADD COLUMN claim_namespace TEXT;

ALTER TABLE claim_assessments
    ADD CONSTRAINT claim_assessments_namespace CHECK (
        claim_namespace IS NULL OR claim_namespace IN ('thread', 'external')
    );

DROP INDEX claim_assessments_replay_idx;

CREATE UNIQUE INDEX claim_assessments_replay_idx
    ON claim_assessments (claim_id, snapshot_id, assessment, author,
                          claim_namespace, authored_by_tenant)
    NULLS NOT DISTINCT;
