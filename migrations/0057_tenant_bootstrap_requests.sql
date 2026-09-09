-- One client request identity -> one server-allocated tenant and creation outcome.
-- The service only inserts and reads these bindings: no update, expiry, deletion
-- or reassignment path. Maintenance must preserve them while the original tenant
-- is recoverable. Do not backfill legacy name-based enrollment with invented keys.
CREATE TABLE tenant_bootstrap_requests (
    request_id TEXT COLLATE "C" PRIMARY KEY
        CHECK (request_id ~ '^req_[0-9a-f]{8}-[0-9a-f]{4}-7[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$'),
    tenant_id TEXT COLLATE "C" NOT NULL UNIQUE REFERENCES tenants (tenant_id),
    request_name TEXT COLLATE "C" NOT NULL,
    outcome_version SMALLINT NOT NULL CHECK (outcome_version > 0),
    outcome JSONB NOT NULL CHECK (jsonb_typeof(outcome) = 'object'),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
