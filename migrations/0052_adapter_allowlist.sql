-- 0052_adapter_allowlist.sql — the ADR-027 ladder's rung-1 ledger
-- (`PHASE-8.4.4`): the allowlist rows a downloaded adapter's verification
-- checks FIRST (the cheapest, the most stable rung). The dev three seed
-- BY CONSTRUCTION (the compiled-in adapters + their qualification); the
-- operator's allow/revoke verbs manage the rows. A row removal makes the
-- NEXT ladder run refuse at rung 1 — never a silent untrust.

CREATE TABLE adapter_allowlist (
    adapter_id TEXT        PRIMARY KEY,
    added_by   TEXT        NOT NULL,
    reason     TEXT        NOT NULL,
    added_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);

INSERT INTO adapter_allowlist (adapter_id, added_by, reason) VALUES
    ('fake',   'dev-seed', 'the deterministic conformance oracle (the §11.6 contract)'),
    ('codex',  'dev-seed', 'the .4.2 real harness (the qualified CLI adapter)'),
    ('claude', 'dev-seed', 'the .1.4.1 real harness (the qualified CLI adapter)');
