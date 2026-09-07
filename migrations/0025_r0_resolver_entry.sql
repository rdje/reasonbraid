-- 0025_r0_resolver_entry.sql — PHASE-4.2.3: the R0 pack's install record.
-- The built-in safe HTTPS fetcher registers itself as the resolver the
-- §12.3 R0 pack ships: the https scheme, the text/HTML media types, the
-- GET/HEAD abilities, NO authentication classes (public acquisition), the
-- ADR-018 claims — egress `listed` (the §12.4 public-only destination
-- classes ARE the list the `.2.1` classifier enforces), sandbox `none` (the
-- fetcher runs in-process and executes NO fetched content — the bytes go to
-- storage; the honest claim is the ladder's bottom, and a caller requiring
-- more isolation gets the explicit unresolvable-now, never a silent
-- downgrade), the redirect policy `follow-classified` (the manual per-hop
-- re-classification), the ADR-011 snapshot digest format, and the
-- security-evidence the `.2.2` fetcher measures.

INSERT INTO resolver_capabilities (
    resolver_id, schemes, locator_patterns, media_types, max_bytes, abilities,
    authentication_classes, egress_class, sandbox_level, redirect_policy,
    archive_policy, subresource_policy, javascript_policy, snapshot_formats,
    derivation_formats, latency_range_ms, version, security_evidence
) VALUES (
    'r0-https-fetcher',
    '["https"]'::jsonb,
    '["https://*"]'::jsonb,
    '["text/html", "text/plain"]'::jsonb,
    4194304,
    '["GET", "HEAD"]'::jsonb,
    '["none"]'::jsonb,
    'listed',
    'none',
    'follow-classified',
    'deny',
    'deny',
    'deny',
    '["sha256:<hex>"]'::jsonb,
    '[]'::jsonb,
    '{"min": 100, "max": 1000}'::jsonb,
    '0.1.0',
    '{"ssrf_policy": "public-only", "tls_roots": "system", "decompression_ratio_limit": 10, "byte_ceiling": 4194304}'::jsonb
);
