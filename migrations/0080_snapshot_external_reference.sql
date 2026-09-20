-- 0080_snapshot_external_reference.sql — SIGNOFF-REPAIR.11.24.1.3.2:
-- §12.9's SECOND alternative, taken deliberately and for a measured reason.
--
-- ⛔ THE MEASUREMENT THAT DECIDES IT. §12.9 permits a snapshot to *remain
-- addressable for the charter's audit period OR retain a verifiable external
-- archival reference*. For a git acquisition the first alternative is not
-- merely expensive — it is UNSOUND, and `git::tests::the_acquired_object_
-- database_is_not_a_stable_identity` measures it on real acquisitions: the
-- SAME immutable commit, acquired twice from the same source across a
-- server-side `git repack`, yields two different object databases (6857 and
-- 10071 bytes, two different digests). The pack is chosen by the REMOTE's
-- packing configuration, which nobody here controls. So keying an evidence
-- snapshot on the acquired odb bytes would file a SECOND snapshot every time
-- an upstream forge repacks, for evidence that did not change.
--
-- ⭐ `resolved_commit` IS the stable identity, and §12.6 already has the column
-- for it: *immutable source version where available*. Git is itself a
-- content-addressed archive — the commit id commits to the whole tree — so the
-- reference is verifiable by re-acquisition, which is precisely what §12.9's
-- second alternative asks for.
--
-- ⛔ AND THE DISTINCTION IS STRUCTURAL, NOT A LABEL. Before this migration
-- `storage_class` was written by three callers as the literal `'standard'` and
-- READ BY NO PREDICATE (`SIGNOFF-REPAIR.11.24.1.3` measured that and declined
-- to grade it). Shipping a second value that nothing consults would have made
-- it a defect, so the class is given two readers here, both in the database and
-- therefore binding on every writer including a future one: the CHECK below,
-- and the partial unique index that carries the class's identity.
--
-- ⚠️ `byte_length = 0` for the external class is not a claim that the artefact
-- is empty — it is the honest statement that THIS STORE HOLDS NO BYTES for it,
-- and the CHECK pins the pairing so no reader can take it for the artefact's
-- size. The acquisition's own measured size rides `provider_receipt`, where
-- §12.6 puts *HTTP/Git/provider receipts*.

ALTER TABLE evidence_snapshots
    ALTER COLUMN raw_digest DROP NOT NULL,
    ADD COLUMN external_reference JSONB;

-- Reader 1 of `storage_class`: the two shapes are mutually exclusive and
-- complete. An inline snapshot holds bytes under a digest and names no external
-- reference; an external-reference snapshot holds no bytes, names a reference,
-- and MUST carry the immutable source version its identity rests on.
ALTER TABLE evidence_snapshots
    ADD CONSTRAINT evidence_snapshots_storage_class_shape CHECK (
        (storage_class = 'external-reference'
            AND raw_digest IS NULL
            AND byte_length = 0
            AND external_reference IS NOT NULL
            AND immutable_source_version IS NOT NULL)
        OR
        (storage_class <> 'external-reference'
            AND raw_digest IS NOT NULL
            AND external_reference IS NULL)
    );

-- Reader 2 of `storage_class`: the identity of an external-reference snapshot.
-- The inline class replays on `(reference_id, raw_digest)`; that key is `NULL`
-- for this class and `= NULL` never matches, so without this index every
-- re-acquisition would insert a new row. `(reference_id,
-- immutable_source_version)` is the replacement, and it is an INDEX rather than
-- only a SELECT in the writer so two concurrent acquisitions of one commit
-- cannot both insert.
CREATE UNIQUE INDEX evidence_snapshots_external_identity_idx
    ON evidence_snapshots (reference_id, immutable_source_version)
    WHERE storage_class = 'external-reference';
