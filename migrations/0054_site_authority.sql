-- SIGNOFF-REPAIR.3.2.1: site authority is independent of tenant enrollment.
-- No existing tenant grant is upgraded and no operator role is created here.

CREATE TABLE public.site_authority_guard (
    guard_id SMALLINT PRIMARY KEY CHECK (guard_id = 1)
);
INSERT INTO public.site_authority_guard VALUES (1);

CREATE TABLE public.site_boundaries (
    boundary_id TEXT PRIMARY KEY,
    issued_by TEXT NOT NULL,
    actions TEXT[] NOT NULL CHECK (
        array_ndims(actions) = 1 AND cardinality(actions) > 0
        AND array_position(actions, NULL) IS NULL
        AND actions <@ ARRAY['registry_inspect', 'adapter_allow', 'adapter_revoke',
                            'region_declare', 'region_pair', 'region_unpair']::TEXT[]
    ),
    valid_from TIMESTAMPTZ NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL CHECK (expires_at > valid_from),
    status TEXT NOT NULL CHECK (status IN ('active', 'suspended', 'revoked')),
    reason TEXT NOT NULL CHECK (octet_length(reason) BETWEEN 1 AND 1024),
    issued_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp()
);

CREATE TABLE public.site_grants (
    grant_id TEXT PRIMARY KEY,
    boundary_id TEXT NOT NULL REFERENCES public.site_boundaries (boundary_id),
    issued_by TEXT NOT NULL,
    subject_kind TEXT NOT NULL CHECK (subject_kind IN ('human', 'role')),
    subject_id TEXT NOT NULL CHECK (octet_length(subject_id) BETWEEN 1 AND 128),
    actions TEXT[] NOT NULL CHECK (
        array_ndims(actions) = 1 AND cardinality(actions) > 0
        AND array_position(actions, NULL) IS NULL
        AND actions <@ ARRAY['registry_inspect', 'adapter_allow', 'adapter_revoke',
                            'region_declare', 'region_pair', 'region_unpair']::TEXT[]
    ),
    valid_from TIMESTAMPTZ NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL CHECK (expires_at > valid_from),
    status TEXT NOT NULL CHECK (status IN ('active', 'suspended', 'revoked')),
    reason TEXT NOT NULL CHECK (octet_length(reason) BETWEEN 1 AND 1024),
    issued_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp()
);
CREATE INDEX site_grants_subject_idx ON public.site_grants (subject_kind, subject_id);

-- Issuance facts, including the parent binding, cannot be rewritten. Suspension
-- and revocation only remove authority; replacement requires explicit issuance.
CREATE FUNCTION public.site_authority_monotonic_update() RETURNS trigger
LANGUAGE plpgsql SET search_path = pg_catalog AS $$
BEGIN
    IF (to_jsonb(NEW) - 'status') IS DISTINCT FROM (to_jsonb(OLD) - 'status') THEN
        RAISE EXCEPTION 'site authority issuance facts are immutable';
    END IF;
    IF (OLD.status = 'revoked' AND NEW.status <> 'revoked')
       OR (OLD.status = 'suspended' AND NEW.status = 'active') THEN
        RAISE EXCEPTION 'site authority cannot be reactivated';
    END IF;
    RETURN NEW;
END;
$$;
CREATE TRIGGER site_boundary_immutable BEFORE UPDATE ON public.site_boundaries
FOR EACH ROW EXECUTE FUNCTION public.site_authority_monotonic_update();
CREATE TRIGGER site_grant_immutable BEFORE UPDATE ON public.site_grants
FOR EACH ROW EXECUTE FUNCTION public.site_authority_monotonic_update();

CREATE TABLE public.site_audit (
    audit_id TEXT PRIMARY KEY,
    actor_kind TEXT NOT NULL CHECK (actor_kind IN ('database', 'human', 'role')),
    actor TEXT NOT NULL,
    action TEXT NOT NULL,
    target JSONB NOT NULL,
    grant_id TEXT,
    boundary_id TEXT,
    requested_reason TEXT NOT NULL CHECK (octet_length(requested_reason) BETWEEN 1 AND 1024),
    decision TEXT NOT NULL CHECK (decision IN ('allowed', 'denied')),
    outcome TEXT NOT NULL CHECK (outcome IN ('applied', 'noop', 'inspected', 'denied')),
    reason TEXT NOT NULL,
    evaluation JSONB NOT NULL,
    decided_at TIMESTAMPTZ NOT NULL,
    CHECK ((decision = 'denied') = (outcome = 'denied'))
);
CREATE INDEX site_audit_time_idx ON public.site_audit (decided_at, audit_id);

-- Audit references deliberately outlive authority rows. The supported tooling
-- has no delete/history-rewrite operation; database administration is a trust root.
