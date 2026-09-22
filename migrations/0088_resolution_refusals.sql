-- 0088_resolution_refusals.sql — SIGNOFF-REPAIR.4.6.1.2: the server keeps the
-- refusals the resolve path answers, so an operator can list them.
--
-- ROADMAP §18.5 asks the admin surface to expose *evidence acquisitions and
-- resolver denials*. `POST /v1/resources/{id}/resolve` explained every refusal
-- in its response — a named `acquisition_error`, the explicit
-- `unresolvable_now`, a quota refusal — and stored none of them except the
-- quota denial, which lands in `quota_events` without saying which resource was
-- asked for or by whom
-- (`docs/decisions/2026-09-22_three-of-the-five-missing-operator-surfaces-have-no-stored-fact.md`).
--
-- One row per refused resolution. `kind` is the refusal's own name, exactly as
-- the response carried it: an acquisition error kind (`destination_refused`,
-- `scheme_not_allowed`, …), `unresolvable_now`, `quota_exceeded` or
-- `quota_unconfigured`. It is deliberately NOT constrained to a list: the
-- acquisition packs name their own refusals, and a CHECK here would turn a new
-- refusal kind into a failed write — losing the very record this table exists
-- to keep.
--
-- ⛔ A quota refusal ALSO writes its `quota_events` denial. The two are
-- different facts: `quota_events` is the counting ledger the window arithmetic
-- reads, keyed by quota; this row is the operator's record of which request was
-- refused. Both commit in one transaction.

CREATE TABLE resolution_refusals (
    refusal_id    TEXT        NOT NULL PRIMARY KEY,
    -- The CALLER's tenant: the resolve path is bound to it before anything is
    -- refused, so a refusal is always the asking tenant's own fact.
    tenant_id     TEXT        NOT NULL REFERENCES tenants (tenant_id),
    resource_id   TEXT        NOT NULL,
    requested_by  TEXT        NOT NULL,
    -- The resolver that ranked first and then refused. NULL for
    -- `unresolvable_now`, where no resolver was eligible at all.
    resolver_id   TEXT,
    kind          TEXT        NOT NULL CHECK (kind <> ''),
    message       TEXT        NOT NULL,
    refused_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

COMMENT ON TABLE resolution_refusals IS
    'Refusals answered by POST /v1/resources/{id}/resolve, one row per refused '
    'request, listed by GET /v1/admin/resolution-refusals.';

CREATE INDEX resolution_refusals_tenant_idx
    ON resolution_refusals (tenant_id, refused_at DESC);
