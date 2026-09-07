-- 0027_r2_extract_worker_entry.sql — PHASE-4.4.3: the R2 pack's install record.
-- The built-in extraction worker registers itself as the resolver the §12.3
-- R2 row ships: the https scheme (the PIPELINE's acquisition input — the R0
-- fetcher acquires the bytes, the worker derives the text), the extraction
-- media types (a reference CARRYING one of these hints ranks the R2 pack; a
-- hintless reference stays the acquisition-only path), NO authentication
-- classes, the ADR-018 claims — egress `listed` (the §12.4 public-only
-- destination classes), sandbox `process` (the FIRST ladder-up: the parsers
-- EXECUTE untrusted content in a dedicated worker process per extraction —
-- the quarantine boundary; a stricter requirement is the explicit
-- unresolvable-now, never a silent downgrade), the `follow-classified`
-- redirect policy, the ADR-011 digest format (the Derivation chunks), and
-- the security evidence the `.4.2` worker tests measure.

INSERT INTO resolver_capabilities (
    resolver_id, schemes, locator_patterns, media_types, max_bytes, abilities,
    authentication_classes, egress_class, sandbox_level, redirect_policy,
    archive_policy, subresource_policy, javascript_policy, snapshot_formats,
    derivation_formats, latency_range_ms, version, security_evidence
) VALUES (
    'r2-extract-worker',
    '["https"]'::jsonb,
    '["https://*"]'::jsonb,
    '["application/pdf", "application/zip", "application/x-tar", "application/atom+xml", "application/rss+xml"]'::jsonb,
    16777216,
    '["extract"]'::jsonb,
    '["none"]'::jsonb,
    'listed',
    'process',
    'follow-classified',
    'deny',
    'deny',
    'deny',
    '["sha256:<hex>"]'::jsonb,
    '["text/chunks"]'::jsonb,
    '{"min": 2000, "max": 60000}'::jsonb,
    '0.1.0',
    '{"worker": "process-per-extraction", "derivation": true, "refusals": ["pdf-encrypted", "pdf-javascript", "nested-archives", "path-traversal", "decompression-ratio"], "kill_on_budget_trip": true}'::jsonb
);
