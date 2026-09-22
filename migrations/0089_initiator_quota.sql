-- 0089_initiator_quota.sql — SIGNOFF-REPAIR.11.4.7.2.1.5.3.2.1 (ROADMAP §11.5's
-- *rate* bound on autonomous initiation; §10.7): the fifth quota scope — how
-- often a ROLE may start a thread on its own (`POST /v1/threads/auto`).
--
-- `0047` made a quota a WINDOWED CEILING over recorded events, with a refusal
-- that is itself a recorded event. Four scopes exist: `tenant` (invitations),
-- `principal` (MCP write calls), `resolver` and `destination` (acquisitions).
-- None of them bounds autonomous initiation, and a quota row's scope has ONE
-- meaning per surface — `principal` already counts MCP write calls — so the
-- initiation bound is a scope of its own rather than a second reader of an
-- existing row.
--
-- The member space is the agent roles, which the server creates (enrolment and
-- card import), so the row is seeded PER MEMBER at creation exactly like the
-- `principal` scope, and fail-closed applies as `0047` shipped it: a role with
-- no `initiator` row is refused with `quota_unconfigured`. The backfill below
-- gives every EXISTING role the dev default so older deployments keep working.
--
-- ⚠️ The ceiling is the dev-profile SHAPE `0047` established: generous for the
-- demo, still a bound, and NOT a measured production figure.
-- `SIGNOFF-REPAIR.11.6` forbids proposing a threshold before its population is
-- measured, and no initiation volume has been measured. What is decided here is
-- that the bound exists and what it counts, not its number.
--
-- Until this migration the only thing bounding autonomous initiation was a
-- DEFECT: the idempotency key `auto_{role}_{tenant}` made every second
-- initiation replay the first (`SIGNOFF-REPAIR.5.2`, observed on the running
-- server in DOC-0136). That key becomes per-initiation in the same change, so
-- this bound is what replaces the defect, and the two land together.

ALTER TABLE usage_quotas DROP CONSTRAINT usage_quotas_scope_kind_check;
ALTER TABLE usage_quotas ADD CONSTRAINT usage_quotas_scope_kind_check
    CHECK (scope_kind IN ('tenant', 'principal', 'resolver', 'destination', 'initiator'));

INSERT INTO usage_quotas (quota_id, tenant_id, scope_kind, scope_id, ceiling, window_seconds)
SELECT 'quo_' || r.role_id || '_initiations', r.tenant_id, 'initiator', r.role_id, 1000, 3600
FROM agent_roles r
ON CONFLICT (tenant_id, scope_kind, scope_id) DO NOTHING;
