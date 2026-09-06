-- 0005_budget.sql — the WP5 (.5.2) development budget store.
--
-- ROADMAP.md §14: a ceiling is the hard limit; a reservation holds a multi-dimensional
-- amount against it; settlement records ACTUAL usage (which may overrun the hold —
-- an estimate error is recorded, never silently dropped); release returns the unused
-- remainder; a denial is itself a recorded row. Held amount at reserve time =
-- active (unexpired) reservations + settled usage — so settling less than the
-- reservation frees the difference implicitly.

CREATE TABLE budget_ceilings (
    ceiling_id     TEXT PRIMARY KEY,
    tenant_id      TEXT NOT NULL,
    thread_id      TEXT NOT NULL,
    dimensions     JSONB NOT NULL,  -- BudgetDimensions (None = not metered)
    policy_version TEXT NOT NULL,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE budget_reservations (
    reservation_id TEXT PRIMARY KEY,
    ceiling_id     TEXT NOT NULL REFERENCES budget_ceilings (ceiling_id),
    tenant_id      TEXT NOT NULL,
    thread_id      TEXT NOT NULL,
    dimensions     JSONB NOT NULL,  -- reserved dims (denied rows: requested dims)
    usage          JSONB,           -- settled actuals (overrun may exceed `dimensions`)
    status         TEXT NOT NULL,   -- active | settled | released | denied
    reason         TEXT,
    expires_at     TIMESTAMPTZ,     -- active rows stop holding after this
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    settled_at     TIMESTAMPTZ
);

CREATE INDEX budget_reservations_ceiling_idx
    ON budget_reservations (ceiling_id, status);
