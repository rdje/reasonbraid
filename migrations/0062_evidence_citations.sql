-- 0062_evidence_citations.sql — SIGNOFF-REPAIR.11.14.1 (ROADMAP §16.8,
-- `docs/decisions/2026-09-16_evidence-is-shared-the-read-is-tenant-bound.md`):
-- the tenant binding for a deliberately SHARED evidence chain.
--
-- The snapshot ROW is shared by construction, not by accident:
-- `resource_references` is UNIQUE on `(original_locator, expected_digest)` and
-- `snapshot_objects` is keyed by `digest` alone (ADR-011), so two tenants that
-- cite the same URL at the same digest hold the SAME row. A `tenant_id` COLUMN
-- on `evidence_snapshots` would therefore either break that uniqueness or
-- duplicate identical bytes per tenant — which is why the decision record
-- rejects it for all twelve evidence, policy, evaluation and deployment tables.
--
-- §16.8 asks for the tenant in every authorization DECISION, not in every
-- table, and the citation is that decision's input: a SET of tenants per
-- snapshot, written on every acquisition INCLUDING the replay. Recording the
-- citing tenant on the WRITE is what keeps the read binding from hiding a row
-- from its own author — `SIGNOFF-REPAIR.6.1.5`'s trap, which is the reason a
-- read predicate alone was rejected.
--
-- Both foreign keys CASCADE. The fixture plans delete `evidence_snapshots` and
-- `tenants`, and a citation is meaningless once either end is gone.
--
-- RLS: this table stays on the application layer with the rest of the
-- tenant-keyed tables. Migration 0046 enables row-level security on the WIRED
-- command core only (`aggregate_state`, `event_log`, `idempotency`) and names
-- the remainder as a deferral; the evidence reads do not carry the
-- transaction-local `app.tenant_id` claim.
--
-- NO BACKFILL IS POSSIBLE, and that is measured rather than assumed. A
-- snapshot written before this migration has no recoverable citer:
-- `evidence_snapshots` carries no principal column, its only link out is
-- `reference_id`, and `resource_references`'s only actor column is
-- `submitted_by` — which stores `actor_handle_for_subject(...)`, a UUIDv5 over
-- the subject's description that joins to no identity table, and which the
-- locator replay in `resources::submit` leaves naming the FIRST citer whatever
-- happens afterwards. A pre-existing row is therefore read by no tenant until
-- it is cited again, at which point the replay records the citation.

CREATE TABLE evidence_citations (
    snapshot_id TEXT        NOT NULL REFERENCES evidence_snapshots (snapshot_id) ON DELETE CASCADE,
    tenant_id   TEXT        NOT NULL REFERENCES tenants (tenant_id) ON DELETE CASCADE,
    cited_by    TEXT        NOT NULL,  -- the actor handle that asked for the acquisition
    cited_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (snapshot_id, tenant_id)
);

CREATE INDEX evidence_citations_tenant_idx ON evidence_citations (tenant_id);
