-- 0047_usage_quotas.sql — the quota/abuse machinery (`PHASE-7.1.3.2`, ADR-034
-- §16.11): the per-key bounds — the tenant, the principal, the resolver, the
-- destination — riding the Phase-2 budget pattern: a quota is a WINDOWED
-- CEILING over recorded events, and a refusal is a RECORDED event (the
-- denial row), never silent.
--
-- The shipped binding: the per-tenant INVITE bound (the invitation-storm
-- surface — `thread.invite` checks the `tenant` scope before dispatching).
-- The principal/resolver/destination bindings are the named deferrals (each
-- with its trigger in the `.1.3.2` leaf).
--
-- The check is FAIL-CLOSED: a tenant with NO configured quota is the typed
-- `quota_unconfigured` refusal — the bound must exist before the surface is
-- usable (ADR-034's "a classification without the controls is the typed
-- refusal", applied to the abuse vocabulary). The backfill below gives every
-- EXISTING tenant the dev default, so older deployments keep working.

CREATE TABLE usage_quotas (
    quota_id       TEXT   PRIMARY KEY,
    tenant_id      TEXT   NOT NULL REFERENCES tenants (tenant_id),
    scope_kind     TEXT   NOT NULL CHECK (scope_kind IN ('tenant', 'principal', 'resolver', 'destination')),
    scope_id       TEXT   NOT NULL,
    ceiling        BIGINT NOT NULL CHECK (ceiling >= 0),
    window_seconds BIGINT NOT NULL CHECK (window_seconds > 0),
    UNIQUE (tenant_id, scope_kind, scope_id)
);

-- The recorded uses + denials (the audit — a quota denial is a recorded
-- event, exactly like the budget's denial rows).
CREATE TABLE quota_events (
    event_id BIGSERIAL   PRIMARY KEY,
    quota_id TEXT        NOT NULL REFERENCES usage_quotas (quota_id),
    kind     TEXT        NOT NULL CHECK (kind IN ('use', 'denial')),
    at       TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX quota_events_window_idx ON quota_events (quota_id, kind, at);

-- The dev-profile default for every existing tenant: the invite-storm bound
-- (1000 invites/hour — generous for the demo, still a bound).
INSERT INTO usage_quotas (quota_id, tenant_id, scope_kind, scope_id, ceiling, window_seconds)
SELECT 'quo_' || tenant_id || '_invites', tenant_id, 'tenant', tenant_id, 1000, 3600
FROM tenants;
