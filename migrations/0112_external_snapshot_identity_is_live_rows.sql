-- 0112_external_snapshot_identity_is_live_rows.sql — SIGNOFF-REPAIR.7.4.6
-- (ROADMAP §12.6, §12.9): a tombstoned acquisition is not what a new
-- acquisition replays onto.
--
-- A tombstone retires one ACQUISITION: the retention sweep writes it as well as
-- the operator's site act, and the row stays, with its reason. The inline class
-- replayed a re-acquisition of the same bytes onto a tombstoned row, re-citing
-- evidence the site had retired and reporting it as a replay; its lookup now
-- reads live rows only, and the same bytes acquired again are a new row.
--
-- The external-reference class carries that identity as a UNIQUE index
-- (`0080`), so the same repair has to be made here: without it the new row for
-- a commit whose earlier acquisition was tombstoned would conflict with the
-- dead row, and `DO NOTHING` would hand the writer the tombstoned id again.
--
-- ⭐ Uniqueness among LIVE rows is the whole of what `0080` needed: it exists so
-- two concurrent acquisitions of one commit cannot both insert, and a race
-- happens between live rows. Every existing row satisfies the narrower index,
-- because it satisfied the wider one.
DROP INDEX evidence_snapshots_external_identity_idx;
CREATE UNIQUE INDEX evidence_snapshots_external_identity_idx
    ON evidence_snapshots (reference_id, immutable_source_version)
    WHERE storage_class = 'external-reference' AND deleted_at IS NULL;
