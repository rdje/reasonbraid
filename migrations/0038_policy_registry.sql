-- The typed policy schema + the versioned registry (`.1.2`, ADR-019): the
-- policy is a versioned digest-pinned DOCUMENT — the stable clause ids, the
-- applicability + the explicit non-applicability, the exception schema, and
-- the OWNERSHIP metadata (the owning_authority is a GRANT reference — the
-- label grants nothing; an unresolvable owning authority is invalid at
-- registration). The same registry pattern as the workflow profiles
-- (ADR-016): the unknown version is the typed refusal, never a stored guess.
CREATE TABLE policy_versions (
    policy_id        TEXT        NOT NULL,
    version          TEXT        NOT NULL,
    digest           TEXT        NOT NULL,
    lifecycle        TEXT        NOT NULL,
    title            TEXT        NOT NULL,
    intent           TEXT        NOT NULL DEFAULT '',
    rationale        TEXT        NOT NULL DEFAULT '',
    domain           TEXT        NOT NULL DEFAULT '',
    risk_class       TEXT        NOT NULL DEFAULT 'general',
    owning_authority TEXT        NOT NULL,
    clauses          JSONB       NOT NULL,
    applicability    JSONB       NOT NULL DEFAULT '[]'::jsonb,
    non_applicability JSONB      NOT NULL DEFAULT '[]'::jsonb,
    dependencies     JSONB       NOT NULL DEFAULT '[]'::jsonb,
    conflicts        JSONB       NOT NULL DEFAULT '[]'::jsonb,
    precedence_hints JSONB       NOT NULL DEFAULT '[]'::jsonb,
    exceptions       JSONB       NOT NULL DEFAULT '[]'::jsonb,
    provenance       JSONB       NOT NULL DEFAULT '[]'::jsonb,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (policy_id, version)
);
