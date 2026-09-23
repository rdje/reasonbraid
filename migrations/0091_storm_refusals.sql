-- 0091_storm_refusals.sql — SIGNOFF-REPAIR.11.4.7.2.1.5.3.4 (ROADMAP §10.7): every
-- `storm_control` refusal the server answers is recorded, so the circuit
-- breakers' deferral trigger — *the first multi-tenant storm observed* — can be
-- observed at all.
--
-- `PHASE-3.4` deferred the per-origin and global circuit breakers behind that
-- trigger, and DOC-0134 found the trigger UNOBSERVABLE: `POST /v1/calls` answered
-- `429 storm_control` for the two fan-out caps and stored nothing, and REPAIR-0415
-- added two more producers (an autonomous cycle, an over-deep chain) with the
-- same silence. A refusal that is answered and recorded nowhere is the shape
-- `.11.4.7.2.1` names: a trigger nothing evaluates.
--
-- One row per refused request, written by the ONE helper every producer goes
-- through (`api.rs::refuse_storm`), in the same words the caller was given —
-- the `0088` shape for resolution refusals. `control` is the limit's own name
-- (`open_calls_per_tenant`, `open_calls_per_initiator`, `autonomous_cycle`,
-- `autonomous_depth`, and whatever a later control adds); it is deliberately NOT
-- constrained to a list, because a new storm control must be able to record
-- itself without a migration, and losing the record is the defect this table
-- removes. `limit_value` is the numeric limit when the control has one.
--
-- The breaker trigger is now a query anyone can run:
--   SELECT count(DISTINCT tenant_id) FROM storm_refusals
--    WHERE refused_at > now() - interval '1 hour';
-- ≥ 2 is *a multi-tenant storm observed*. The breakers themselves stay
-- deferred on it.

CREATE TABLE storm_refusals (
    refusal_id  TEXT        NOT NULL PRIMARY KEY,
    tenant_id   TEXT        NOT NULL REFERENCES tenants (tenant_id),
    initiator   TEXT        NOT NULL,
    control     TEXT        NOT NULL CHECK (control <> ''),
    limit_value BIGINT,
    target      TEXT,
    message     TEXT        NOT NULL,
    refused_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

COMMENT ON TABLE storm_refusals IS
    'Every 429 storm_control the server answered, one row per refused request, '
    'listed by GET /v1/admin/storm-refusals.';

CREATE INDEX storm_refusals_tenant_idx
    ON storm_refusals (tenant_id, refused_at DESC);
