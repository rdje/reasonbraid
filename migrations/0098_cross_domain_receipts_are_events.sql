-- 0098_cross_domain_receipts_are_events.sql — SIGNOFF-REPAIR.5.3.4 (found by its
-- control): a cross-domain receipt is a record of ONE acceptance, and two
-- acceptances against the same remote terms are two receipts.
--
-- 0049 declared `UNIQUE (tenant_id, remote_tenant_id, kind, remote_ref)`. For an
-- agreement acceptance that key is the counterparty's record — the tenant id
-- until 0096, the terms digest since — so a SECOND acceptance in the same
-- direction against an unchanged counterparty record (this side re-proposed
-- its lifetime, or its own terms, and accepted again) RAISED the key into a
-- `500` inside the transaction carrying the admission and the effect record:
-- the shape `docs/knowledge/a-raised-constraint-cannot-be-a-recorded-refusal.md`
-- forbids, latent since the verbs were put on one transaction and reachable
-- from the wire the whole time. For a card import the key is the card's digest,
-- and a repeat of the same origin is answered as a replay before any receipt is
-- written (0095), so the key guarded nothing there either.
ALTER TABLE cross_domain_receipts
    DROP CONSTRAINT cross_domain_receipts_tenant_id_remote_tenant_id_kind_remot_key;
