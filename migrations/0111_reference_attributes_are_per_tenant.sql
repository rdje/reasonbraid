-- 0111_reference_attributes_are_per_tenant.sql — SIGNOFF-REPAIR.7.1.4 (ROADMAP
-- §12.1, §12.2): what a tenant DECLARES about a reference is that tenant's
-- statement, not the shared row's.
--
-- `resource_references` is content-addressed, `UNIQUE (original_locator,
-- expected_digest)`, so two tenants citing one URL at one digest hold ONE row.
-- The row also carried the eight attributes a citer DECLARES about the URL, and
-- `resources::submit` wrote them once, from the FIRST citer; every later
-- tenant's replay discarded its own. Resolution selects on `scheme`, so the
-- first tenant to cite a URL chose how every tenant's citation of it resolves,
-- and the detail read showed each later tenant the first citer's purpose, hints
-- and risk class. Measured live before this migration: tenant A cites a URL as
-- `ftp`, tenant B cites it as `https`, and B's read answers `ftp` with A's
-- purpose and B's resolution finds nothing to serve it.
--
-- ⭐ The shape `0069` gave the credential selector, for the same reason: the
-- SHARED row keeps the CONTENT (the locator, the digest, and the row's
-- provenance), and the TENANT-BOUND row keeps the DECLARATION. `0067` created
-- that row; the declared attributes move onto it, beside the binding.
--
-- ⛔ Making `scheme` part of the row's identity was the rejected alternative: it
-- would split one URL into one row per scheme and still leave the other seven
-- attributes first-writer.
--
-- ⚠️ The backfill copies the shared values onto EVERY existing registration,
-- because that is exactly what each registered tenant has been reading. A later
-- citer's own values were discarded at its replay and were never stored, so
-- they cannot be recovered; its next submission is its complete statement and
-- replaces them. A reference with no registration (one written before `0067`)
-- loses its declared attributes, and no read could reach them: `0067` made the
-- registration the read's gate and wrote none for older rows.
--
-- ⛔ And the old columns are DROPPED, as `0069` dropped the binding: after this
-- change nothing writes them and nothing reads them, and a first-writer
-- attribute left on the shared row is the inheriting read already in the
-- schema.

ALTER TABLE reference_registrations
    ADD COLUMN scheme                    TEXT,
    ADD COLUMN media_type_hint           TEXT,
    ADD COLUMN fragment_or_selector      TEXT,
    ADD COLUMN owning_node_or_capability TEXT,
    ADD COLUMN visibility_scope          TEXT,
    ADD COLUMN purpose                   TEXT,
    ADD COLUMN retention_class           TEXT,
    ADD COLUMN risk_class                TEXT;

UPDATE reference_registrations g
SET scheme                    = r.scheme,
    media_type_hint           = r.media_type_hint,
    fragment_or_selector      = r.fragment_or_selector,
    owning_node_or_capability = r.owning_node_or_capability,
    visibility_scope          = r.visibility_scope,
    purpose                   = r.purpose,
    retention_class           = r.retention_class,
    risk_class                = r.risk_class
FROM resource_references r
WHERE r.resource_id = g.resource_id;

-- No DEFAULT: every writer binds all eight explicitly, and `.11.14.3.5` measured
-- what a schema default no writer reaches is worth. The foreign key guarantees
-- every registration had a row to copy from, so the three required columns are
-- filled by the backfill.
ALTER TABLE reference_registrations
    ALTER COLUMN scheme           SET NOT NULL,
    ALTER COLUMN visibility_scope SET NOT NULL,
    ALTER COLUMN risk_class       SET NOT NULL;

COMMENT ON COLUMN reference_registrations.scheme IS
    'The resolver-selection key THIS tenant declared for the shared reference (§12.2). '
    'Never inherited from another tenant''s registration.';

ALTER TABLE resource_references
    DROP COLUMN scheme,
    DROP COLUMN media_type_hint,
    DROP COLUMN fragment_or_selector,
    DROP COLUMN owning_node_or_capability,
    DROP COLUMN visibility_scope,
    DROP COLUMN purpose,
    DROP COLUMN retention_class,
    DROP COLUMN risk_class;
