-- 0069_registration_credential_binding.sql — SIGNOFF-REPAIR.11.14.3.10
-- (ROADMAP §16.3 invariant 5, §12.1): a credential selector does not belong on
-- a content-addressed row.
--
-- `resource_references` is keyed `UNIQUE (original_locator, expected_digest)` —
-- its identity is the CONTENT. `credential_binding_ref` is not content: it is
-- the caller's means of ACCESS, and the R5 arm feeds it straight to
-- `broker.resolve`. Putting an access decision on a content row made it shared,
-- and `0067`'s registrations are what made the sharing observable:
--
--   * a second tenant registering the same pair REPLAYS the first tenant's row,
--     so it inherited a binding it never named and its own value was discarded;
--   * `POST /v1/resources/{id}/resolve` then attached the FIRST tenant's
--     credential to the second tenant's acquisition — the confused-deputy shape
--     §16.3 exists to forbid: *"target credentials are selected only after
--     authorization for the concrete target and action."*
--   * and the pair key made the honest case impossible too: two tenants citing
--     one URL could not hold two different bindings, which nobody had stated.
--
-- ⭐ The fix is the one this family has applied four times: the SHARED row keeps
-- the content, and the TENANT-BOUND row keeps the decision. `0067` already
-- created exactly that row, so the selector moves onto it and both problems
-- close at once — no inheritance, and two tenants may now differ.
--
-- ⛔ NOT a tenant column on the broker. The broker's deployment integration is
-- the OS keychain (`broker.rs`), a machine-local operator store with no tenant
-- dimension at all; keying it by tenant would push a server-side authorization
-- fact into a component that cannot hold one. Tenant facts live here.
--
-- ⚠️ The backfill is EXACT rather than best-effort, and that is the only reason
-- it is safe to move a credential selector. `reference_registrations.registered_by`
-- and `resource_references.submitted_by` are literally the same value — both are
-- `actor_handle_for_subject(principal)`, written from one binding in
-- `api::submit_resource` — so the row that SUBMITTED a reference can be joined
-- to its own registration and to no other. The binding lands on the submitter's
-- registration; every other tenant's registration gets NULL, which is the
-- fail-closed direction §16.4 requires for secret access ("publication, secret
-- access, grant changes, and irreversible writes fail closed").
--
-- ⛔ And then the old column is DROPPED, which no migration in this repository
-- has done before. It is deliberate: after this change nothing writes it and
-- nothing reads it, and a dead credential selector sitting on the shared row is
-- not inert — it is the next author's mistake, already in the schema. This
-- repository has repaired a dead column once already (`SIGNOFF-REPAIR.11.14.3.5`,
-- the `NOT NULL DEFAULT` clauses that no writer could reach), and that one only
-- stored a risk class. Dropping it makes the schema itself the gate: the read
-- that caused this defect can no longer be written.

ALTER TABLE reference_registrations
    ADD COLUMN credential_binding_ref TEXT;

COMMENT ON COLUMN reference_registrations.credential_binding_ref IS
    'The credential binding THIS tenant declared for this shared reference (§12.1). '
    'Opaque, never a secret, and never inherited from another tenant''s registration.';

UPDATE reference_registrations g
SET credential_binding_ref = r.credential_binding_ref
FROM resource_references r
WHERE r.resource_id = g.resource_id
  AND r.credential_binding_ref IS NOT NULL
  AND g.registered_by = r.submitted_by;

ALTER TABLE resource_references
    DROP COLUMN credential_binding_ref;
