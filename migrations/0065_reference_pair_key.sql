-- 0065_reference_pair_key.sql — SIGNOFF-REPAIR.11.14.3.2
-- (`docs/decisions/2026-09-16_a-citation-registers-the-reference-it-names.md`):
-- a reference's identity is the (locator, digest) PAIR, and an UNPINNED
-- citation is ONE row rather than one per citation.
--
-- `migrations/0023:21` already declares `UNIQUE (original_locator,
-- expected_digest)`. The code never used that key: `resources::submit`
-- pre-checked on the LOCATOR ALONE and refused a second digest outright, so the
-- constraint could not fire for a differing digest and the pair was never the
-- identity in practice. With that refusal retired — it was a §9.8 cross-tenant
-- existence leak, and it let the first principal to pin a locator make that
-- locator uncitable by every other tenant in any form — the declared constraint
-- becomes the live key, and its treatment of NULL starts to matter.
--
-- ⛔ PostgreSQL's default is NULLS DISTINCT: two rows with the same locator and
-- NO digest do not violate `UNIQUE (original_locator, expected_digest)`. A
-- digest-less citation is legitimate — §12.1 marks `expected_digest` optional,
-- and a contributor who has not acquired the bytes cannot know the digest,
-- which is what §13.2 step 6 is for — so under the default every unpinned
-- citation of one locator would insert another row. `NULLS NOT DISTINCT`
-- (PostgreSQL 15+; this project pins 16) makes the unpinned row replay like
-- every other, and makes `ON CONFLICT (original_locator, expected_digest)`
-- cover it.
--
-- NO BACKFILL, and the one way this could fail is named rather than waved away.
-- The stricter constraint collides only with a pre-existing duplicate
-- `(locator, NULL)` pair, which required two concurrent submissions of the same
-- unpinned locator to BOTH pass the old locator-alone pre-check before either
-- inserted. ⛔ That race was reachable in principle — the pre-check and the
-- insert were never atomic — so this does not claim impossibility. It does not
-- need to: `ADD CONSTRAINT` REFUSES on a duplicate rather than dropping a row,
-- so the failure mode is a loud migration, not silent data loss.
--
-- ⛔ The OLD constraint's name is PostgreSQL-GENERATED (`0023` declares the
-- constraint inline and never names it), so this discovers it instead of
-- assuming the derivation. The generated form happens to be
-- `resource_references_original_locator_expected_digest_key` — 56 bytes, under
-- the 63-byte identifier limit, so no truncation — but a migration that hard-codes
-- a name it did not write is a guess, and a guess in a migration fails in every
-- suite at once. The NEW name is stated explicitly, so from here it is a fact
-- rather than a derivation, and it stays the one every
-- `ON CONFLICT (original_locator, expected_digest)` infers.
--
-- ⚠️ Exactly one unique constraint may cover those columns when this finishes:
-- `ON CONFLICT` inference needs an unambiguous arbiter index, so adding the new
-- one BESIDE the old would break the very statement this migration exists to
-- enable.

DO $$
DECLARE
    doomed  text;
    dropped int := 0;
BEGIN
    -- `contype = 'u'` selects UNIQUE constraints only; the table's primary key
    -- is `contype = 'p'` and is untouched. `0023` declares exactly one UNIQUE,
    -- so the loop runs once and the assertion below makes any other number a
    -- loud failure rather than a silent partial migration.
    FOR doomed IN
        SELECT conname
          FROM pg_constraint
         WHERE conrelid = 'public.resource_references'::regclass
           AND contype  = 'u'
    LOOP
        EXECUTE format('ALTER TABLE public.resource_references DROP CONSTRAINT %I', doomed);
        dropped := dropped + 1;
    END LOOP;
    IF dropped <> 1 THEN
        RAISE EXCEPTION
            'resource_references: expected exactly one UNIQUE constraint to replace, dropped %',
            dropped;
    END IF;
END $$;

ALTER TABLE resource_references
    ADD CONSTRAINT resource_references_original_locator_expected_digest_key
        UNIQUE NULLS NOT DISTINCT (original_locator, expected_digest);
