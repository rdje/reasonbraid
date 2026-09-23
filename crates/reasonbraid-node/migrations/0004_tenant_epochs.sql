-- 0004_tenant_epochs.sql — SIGNOFF-REPAIR.5.3.5.3.2 (ADR-008): the node keeps
-- the revocation epoch of EVERY tenant whose work it holds, and the dispatch
-- gate evaluates each command's cached admission decision against ITS tenant's.
--
-- Until this table the journal kept ONE epoch (`channel_state` key
-- `revocation_epoch`), the one the server sent for the node's own tenant, and
-- the gate compared every command against it — so a node holding two tenants'
-- work would have judged one tenant's admission by the other's revocations.
-- The single key is removed with the reader that used it: an epoch nothing
-- reads is a second answer to the same question waiting to be trusted.

CREATE TABLE tenant_epochs (
    tenant_id TEXT PRIMARY KEY,
    epoch     INTEGER NOT NULL
);

DELETE FROM channel_state WHERE key = 'revocation_epoch';

PRAGMA user_version = 4;
