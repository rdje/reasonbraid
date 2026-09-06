-- 0002_outbox_worker.sql — the WP2 (.2.2) leased outbox worker and its fencing columns.
--
-- 0001 made the outbox a durable fact (an item implies its event committed). This migration
-- turns an outbox item into CLAIMABLE WORK (ROADMAP.md §17.3): a worker claims an item with
-- a lease owner, an unguessable per-claim lease token (the fencing handle), and an expiry;
-- completion requires presenting the CURRENT token while the lease is still live. A stale
-- worker whose claim was superseded by a newer one therefore cannot commit — its token no
-- longer matches — and an expired lease cannot complete even with a matching token.
--
-- This is a NEW migration on purpose: 0001 is immutable once landed (see
-- docs/decisions/2026-09-06_atomic-transaction.md), so the worker columns arrive here rather
-- than by editing it.

ALTER TABLE outbox
    ADD COLUMN lease_owner TEXT        NULL,  -- identity of the worker holding the lease
    ADD COLUMN lease_token TEXT        NULL,  -- per-claim fencing handle (gen_random_uuid()::text)
    ADD COLUMN lease_until TIMESTAMPTZ NULL,  -- claim expiry; NULL when the item is unclaimed
    ADD COLUMN attempt     BIGINT      NOT NULL DEFAULT 0,  -- claim count (§17.3 "attempt number")
    ADD CONSTRAINT outbox_lease_all_or_nothing CHECK (
        (lease_owner IS NULL AND lease_token IS NULL AND lease_until IS NULL)
        OR (lease_owner IS NOT NULL AND lease_token IS NOT NULL AND lease_until IS NOT NULL)
    );

-- Claim-query shape: undispatched and (never leased or lease expired), oldest item first.
-- A retry/backoff "next eligible time" is deliberately absent: nothing in .2.2 fails a
-- delivery, so there is no backoff schedule yet — that arrives with the Phase 2 retry policy
-- (§20.4), which will add the column rather than guess its shape now.
CREATE INDEX outbox_claim_idx ON outbox (dispatched, lease_until, outbox_id);

-- The worker's delivery sink: one row per delivered event. The event_id primary key is the
-- dedupe key, so a redelivery after kill-point 4 (delivery committed, acknowledgement lost)
-- inserts nothing new — one domain effect per event. The FK chain
-- outbox_delivery → outbox → event_log makes "a delivery effect exists" imply "its event is
-- durable", mirroring 0001's outbox → event_log fact.
CREATE TABLE outbox_delivery (
    event_id     TEXT        PRIMARY KEY REFERENCES event_log (event_id),
    outbox_id    BIGINT      NOT NULL REFERENCES outbox (outbox_id),
    delivered_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
