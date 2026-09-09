-- Explicit evaluation provenance. Old rows and old writers remain unspecified;
-- their action alone cannot distinguish ordinary evaluation from a read exception.
ALTER TABLE authorization_records
    ADD COLUMN evaluation JSONB NOT NULL DEFAULT '{"kind":"legacy_unspecified"}'::jsonb;

ALTER TABLE authorization_records
    ADD CONSTRAINT authorization_records_evaluation_kind CHECK (
        COALESCE(
            jsonb_typeof(evaluation) = 'object'
            AND evaluation->>'kind' IN (
                'legacy_unspecified', 'boundary_checked', 'tenant_admin_inspection'
            ),
            false
        )
    );

COMMENT ON COLUMN authorization_records.evaluation IS
    'Evaluation provenance, not proof of effect or response delivery. Missing historical intent is legacy_unspecified.';
