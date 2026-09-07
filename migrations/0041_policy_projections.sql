-- The projection records (`.3.2`, ADR-033): the compiled artifact — the
-- target, the ADR-011 digest over the rendered bytes, the bytes, and the
-- DECLARED unrepresentable list (never a silent omission). The compiler
-- renders; the server resolves + records.
CREATE TABLE policy_projections (
    projection_id   TEXT        NOT NULL,
    target          TEXT        NOT NULL,
    digest          TEXT        NOT NULL,
    bytes           TEXT        NOT NULL,
    unrepresentable JSONB       NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (projection_id)
);
