-- 0067_reference_registrations.sql — SIGNOFF-REPAIR.11.14.3.4 (ROADMAP §16.8,
-- §12.1, §12.6): the tenant binding for the §12.1 reference DETAIL READ.
--
-- `.11.14` marked `resource_references` *site-wide by design*, and that verdict
-- answered the COLUMN question — can the row carry an owner? — to which
-- `migrations/0023`'s `UNIQUE (original_locator, expected_digest)` says no: two
-- tenants citing one URL at one digest hold the SAME row, by construction. It
-- did not answer the READ one. The three sibling tables in that same record got
-- *"site-wide row, TENANT-BOUND READ"* for a reason that applies here too — the
-- record's own words for `evidence_snapshots` are "which documents they
-- acquired", and `resource_references` holds `original_locator`.
--
-- So this is `0062`'s shape, applied to the other content-addressed table: a SET
-- of registering tenants per reference, written on every registration INCLUDING
-- the replay. Recording it on the WRITE is what keeps the read binding from
-- hiding a row from its own author (`SIGNOFF-REPAIR.6.1.5`'s trap, the reason a
-- read predicate alone was rejected there and is rejected here).
--
-- ⛔ WHAT THIS DELIBERATELY DOES NOT CLOSE, stated in the migration because a
-- later reader will ask. `POST /v1/resources` answers `replayed: true` for a
-- pair that exists, and that IS an existence confirmation. It cannot be closed
-- without breaking the pair key: §12.1 forbids erasing security-relevant
-- distinctions and §12.6 requires a changed page to be a second reference, so
-- the same pair MUST return the same id. ⭐ The difference from the snapshot
-- case is nameable and is why the two halves get different answers: confirming a
-- SNAPSHOT requires presenting its bytes, while confirming a REFERENCE requires
-- presenting a locator, which anyone can type.
--
-- ⭐ What the binding buys is therefore precise rather than total: the read stops
-- disclosing anything the caller did not already hold. A bare opaque `res_…`
-- handle used to be the whole predicate; the locator is now required, and the
-- locator is the thing the row would have disclosed.
--
-- Both foreign keys CASCADE, for `0062`'s reason: the fixture plans delete
-- `resource_references` and `tenants`, and a registration is meaningless once
-- either end is gone.
--
-- RLS: this table stays on the application layer with the rest of the
-- tenant-keyed tables, exactly as `0062` records — migration 0046 enables
-- row-level security on the WIRED command core only.
--
-- NO BACKFILL, and it is the same impossibility `0062` measured rather than a
-- convenience. `resource_references`'s only actor column is `submitted_by`,
-- which stores `actor_handle_for_subject(...)` — a one-way `Uuid::new_v5` over
-- the subject's description that joins to no identity table — and the pair
-- replay leaves it naming the FIRST registrant whatever happens afterwards.
-- A pre-existing reference is therefore read by no tenant until it is
-- registered again, at which point the replay records the registration and
-- restores the read. That recovery is the same one a snapshot has.

CREATE TABLE reference_registrations (
    resource_id   TEXT        NOT NULL REFERENCES resource_references (resource_id) ON DELETE CASCADE,
    tenant_id     TEXT        NOT NULL REFERENCES tenants (tenant_id) ON DELETE CASCADE,
    registered_by TEXT        NOT NULL,  -- the actor handle that registered it
    registered_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (resource_id, tenant_id)
);

CREATE INDEX reference_registrations_tenant_idx ON reference_registrations (tenant_id);
