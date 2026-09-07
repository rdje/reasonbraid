-- 0024_resolver_capabilities.sql — PHASE-4.1.3: the §12.2 resolver capability
-- registry — the advertise shape the future packs (`.2`–`.4`) declare. The
-- egress class + the sandbox level are the ADR-018 vocabulary; the registry
-- stores the CLAIMS; the `.7` hostile suite will test them.

CREATE TABLE resolver_capabilities (
    resolver_id             TEXT        NOT NULL PRIMARY KEY,
    schemes                 JSONB       NOT NULL,
    locator_patterns        JSONB       NOT NULL DEFAULT '[]'::jsonb,
    media_types             JSONB       NOT NULL DEFAULT '[]'::jsonb,
    max_bytes               BIGINT      NOT NULL DEFAULT 10485760,
    abilities               JSONB       NOT NULL DEFAULT '[]'::jsonb,
    authentication_classes  JSONB       NOT NULL DEFAULT '[]'::jsonb,
    egress_class            TEXT        NOT NULL,
    sandbox_level           TEXT        NOT NULL,
    redirect_policy         TEXT        NOT NULL DEFAULT 'deny',
    archive_policy          TEXT        NOT NULL DEFAULT 'deny',
    subresource_policy      TEXT        NOT NULL DEFAULT 'deny',
    javascript_policy       TEXT        NOT NULL DEFAULT 'deny',
    snapshot_formats        JSONB       NOT NULL DEFAULT '[]'::jsonb,
    derivation_formats      JSONB       NOT NULL DEFAULT '[]'::jsonb,
    latency_range_ms        JSONB       NOT NULL DEFAULT '{"min": 1000, "max": 60000}'::jsonb,
    version                 TEXT        NOT NULL,
    security_evidence       JSONB       NOT NULL DEFAULT '{}'::jsonb,
    registered_at           TIMESTAMPTZ NOT NULL DEFAULT now()
);
