-- 0070_citation_withdrawal.sql — SIGNOFF-REPAIR.7.4.4 (ROADMAP §12.9, §16.8,
-- `docs/decisions/2026-09-18_a-citation-is-withdrawn-a-row-is-tombstoned.md`):
-- separate the two acts that `DELETE /v1/snapshots/{id}` had conflated.
--
-- A snapshot two tenants cite is ONE row, by construction rather than by
-- accident — `resource_references` is UNIQUE on `(original_locator,
-- expected_digest)` and `snapshot_objects` is keyed by `digest` alone
-- (ADR-011), so the second citer REPLAYS the first's row and `0062` records it
-- in the citer SET. `.11.14.1` bound the delete verb to a CITING tenant, which
-- stopped a stranger reaching it and does not reach this: two citers are both
-- citers. So tenant A's deletion tombstoned the row for B as well and stamped
-- B's receipt with A's reason.
--
-- The two acts, and they are not the same authority:
--
--   * WITHDRAWING A CITATION says "this tenant no longer relies on this
--     evidence". It is a statement about one tenant's own relationship to a
--     row, it touches no other citer, and the tenant plainly owns it.
--   * TOMBSTONING THE ROW says "this evidence must not be relied upon by
--     anyone". That is a statement about shared bytes, which is the same
--     authority the retention sweep already needs: `snapshots::expire_due`
--     carries no tenant predicate and cannot, because `retention_class` is a
--     column on the shared row, so invoking it is a site-operator capability
--     (`SIGNOFF-REPAIR.7.4.3`).
--
-- ⛔ THE WITHDRAWAL IS RECORDED, NOT DELETED, and that is §12.9's "never a
-- silent disappearance" applied to the citation rather than to the row. A
-- `DELETE FROM evidence_citations` would have been simpler and would have left
-- no answer to *who stopped relying on this, when, and why* — the question an
-- audit of a deliberation's evidence trail actually asks. Three columns cost
-- less than that answer.
--
-- Re-citation restores rather than conflicts: the primary key is still
-- `(snapshot_id, tenant_id)`, so `record_citation`'s upsert clears the
-- withdrawal and deliberately leaves `cited_at`/`cited_by` alone — the
-- original citation time and actor are the durable fact, and the withdrawal
-- is an episode in that row's life rather than a new row.
--
-- NO BACKFILL IS NEEDED and none is possible to get wrong: every existing
-- citation is live, which is exactly what three NULL columns mean.

ALTER TABLE evidence_citations
    ADD COLUMN withdrawn_at     TIMESTAMPTZ,
    ADD COLUMN withdrawn_by     TEXT,
    ADD COLUMN withdrawal_reason TEXT;

-- The read predicate is `tenant_id = $ AND withdrawn_at IS NULL` on every
-- disclosure surface, so the partial index matches the live citer set the
-- reads actually scan.
CREATE INDEX evidence_citations_live_tenant_idx
    ON evidence_citations (tenant_id)
    WHERE withdrawn_at IS NULL;

COMMENT ON COLUMN evidence_citations.withdrawn_at IS
    'When this tenant stopped relying on the snapshot. NULL is a live citation. '
    'A withdrawal never touches the shared row — tombstoning it is a site-operator act.';
