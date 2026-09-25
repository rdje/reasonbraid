-- Every deployment receipt is kept (`SIGNOFF-REPAIR.9.3.3.3`, §12.9's *never a
-- silent disappearance*). A receipt used to overwrite the assignment's single
-- observed pair, so each one destroyed the one before it and nothing recorded
-- who filed which; the drift comparison's input had no history.
--
-- ⭐ Each receipt is a row naming its reporter (`.9.3.3.2`) and the database's
-- time; the assignment's `observed_digest`/`observed_state` is the LATEST row,
-- written in the same transaction under a lock on the assignment, so the two
-- cannot disagree and "latest" is the commit order.
--
-- ⭐ A receipt carries its TENANT, copied from its assignment's publication when
-- it is written (the parent is the anchor, never the caller), so its read is
-- predicated on the table itself and the table joins no site-global population:
-- the lifecycle tables' shape (`.6.1.5.2`), not `deployment_assignments`'.
--
-- ⚠️ An assignment whose observed pair was reported before this migration has
-- no row here: that history was already gone, and a row invented for it would
-- name a reporter nobody recorded.
CREATE TABLE deployment_receipts (
    receipt_id      TEXT        NOT NULL,
    target_id       TEXT        NOT NULL,
    publication_id  TEXT        NOT NULL,
    tenant_id       TEXT        NOT NULL,
    reporter        TEXT        NOT NULL,
    observed_digest TEXT        NOT NULL,
    observed_state  TEXT        NOT NULL,
    recorded_at     TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    PRIMARY KEY (receipt_id),
    FOREIGN KEY (target_id, publication_id)
        REFERENCES deployment_assignments (target_id, publication_id)
);
CREATE INDEX deployment_receipts_by_assignment
    ON deployment_receipts (target_id, publication_id, recorded_at, receipt_id);

-- ⛔ A receipt is never rewritten. DELETE is left to the table's owner (the
-- test harness empties tables between runs); no product path deletes a row.
CREATE FUNCTION deployment_receipts_immutable() RETURNS trigger
LANGUAGE plpgsql SET search_path = pg_catalog AS $$
BEGIN
    RAISE EXCEPTION 'a deployment receipt is never rewritten';
END;
$$;
CREATE TRIGGER deployment_receipts_immutable BEFORE UPDATE ON deployment_receipts
FOR EACH ROW EXECUTE FUNCTION deployment_receipts_immutable();
