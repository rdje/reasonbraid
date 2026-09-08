-- 0048_federation_agreements.sql — the explicit federation trust agreements
-- (`PHASE-8.1.2`, ADR-026): the NAMED tenant-to-tenant pairing — the single
-- capability source. No agreement row, no cross-domain effect: the
-- visibility stays the network pseudonym and the cross-tenant recruitment
-- refuses (the ADR-026 invariant).
--
-- The pairing is BOTH-SIDES: each side records its own row; the EFFECTIVE
-- agreement is the pair of `accepted` rows. A one-sided proposal widens
-- nothing.

CREATE TABLE federation_agreements (
    agreement_id         TEXT      PRIMARY KEY,
    tenant_id            TEXT      NOT NULL REFERENCES tenants (tenant_id),
    remote_tenant_id     TEXT      NOT NULL REFERENCES tenants (tenant_id),
    directory_visibility BOOLEAN   NOT NULL DEFAULT false,
    recruitment          BOOLEAN   NOT NULL DEFAULT false,
    status               TEXT      NOT NULL CHECK (status IN ('proposed', 'accepted', 'revoked')),
    proposed_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    accepted_at          TIMESTAMPTZ,
    UNIQUE (tenant_id, remote_tenant_id)
);
