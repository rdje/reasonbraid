-- 0084_governance_charters.sql — SIGNOFF-REPAIR.11.4.7.2.1.2.1: the charter
-- gets a home for the decision rules ROADMAP §4.1 says it defines.
--
-- §4.1: "Each tenant has a versioned `GovernanceCharter` defining ... allowed
-- decision rules and approval thresholds". Before this migration
-- `enrollment_boundaries.charter_digest` was a TEXT label nothing resolved —
-- the shipped values are `dev-charter-digest`, `dev-charter-000` and
-- `fixture-charter` — so the charter had a NAME and no CONTENT, and §13.3's
-- seven rule families existed only as prose.
--
-- ⭐ CONTENT-ADDRESSED, WHICH IS WHAT MAKES IT VERSIONED.
-- `docs/decisions/2026-09-22_a-decision-rule-is-a-charter-scoped-vocabulary-and-never-ships-without-its-tally.md`
-- requires that changing the allowed set must not rewrite what past decisions
-- were taken under. A row is keyed by the digest of its own content, so a
-- changed set is a DIFFERENT row and the old one stays readable forever. The
-- boundary keeps naming the digest it was issued under, and
-- `reasonbraid_core::authority`'s decision digest already folds
-- `charter_digest` into every authorization record — so a decision taken on
-- Monday remains interpretable after Tuesday's charter change, with no
-- history rewritten and no `valid_from`/`superseded_at` bookkeeping.
--
-- ⛔ `tenant_id` is INSIDE the digested content, not beside it. Two tenants
-- that allow the same rules hold two rows, because a charter is a tenant's
-- governance document and not a shared template. The `snapshot_objects`
-- precedent (identical bytes = one object) deliberately does NOT apply here.
--
-- ⚠️ Re-registering byte-identical content is IDEMPOTENT rather than a
-- duplicate refusal, for the same reason: the row's identity IS its content,
-- so a second write asserts nothing new. That differs from
-- `evaluation_corpora`, where the coordinate `(corpus_id, version)` is a
-- first-come namespace and a duplicate is a real conflict.

CREATE TABLE governance_charters (
    charter_digest         TEXT        NOT NULL PRIMARY KEY,
    tenant_id              TEXT        NOT NULL,
    allowed_decision_rules JSONB       NOT NULL,
    approval_thresholds    JSONB       NOT NULL,
    registered_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- The tenant's charters, newest first: the read path an authorization check
-- uses to ask "may this tenant decide under this rule" resolves the tenant's
-- ACTIVE boundary to its `charter_digest` and looks it up here.
CREATE INDEX governance_charters_tenant_idx
    ON governance_charters (tenant_id, registered_at DESC);
