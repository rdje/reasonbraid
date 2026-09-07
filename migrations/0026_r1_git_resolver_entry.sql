-- 0026_r1_git_resolver_entry.sql — PHASE-4.3.3: the R1 pack's install record.
-- The built-in public-Git acquirer registers itself as the resolver the
-- §12.3 R1 pack ships: the `git` scheme (the reference's original_locator is
-- the https transport URL with the fragment-carried ref), NO authentication
-- classes (public acquisition), the ADR-018 claims — egress `listed` (the
-- §12.4 public-only destination classes), sandbox `none` (the acquirer runs
-- in-process, executes NO repository code, and never materializes a worktree
-- — the honest ladder-bottom claim; a stricter requirement is the explicit
-- unresolvable-now, never a silent downgrade), the `follow-classified`
-- redirect policy (the belt re-classifies every dial), the ADR-011 digest
-- over the acquired odb bytes, and the security evidence the `.3.2` tests
-- measure.

INSERT INTO resolver_capabilities (
    resolver_id, schemes, locator_patterns, media_types, max_bytes, abilities,
    authentication_classes, egress_class, sandbox_level, redirect_policy,
    archive_policy, subresource_policy, javascript_policy, snapshot_formats,
    derivation_formats, latency_range_ms, version, security_evidence
) VALUES (
    'r1-git-fetcher',
    '["git"]'::jsonb,
    '["https://*"]'::jsonb,
    '[]'::jsonb,
    268435456,
    '["clone", "fetch"]'::jsonb,
    '["none"]'::jsonb,
    'listed',
    'none',
    'follow-classified',
    'deny',
    'deny',
    'deny',
    '["sha256:<hex>"]'::jsonb,
    '[]'::jsonb,
    '{"min": 1000, "max": 120000}'::jsonb,
    '0.1.0',
    '{"ssrf_policy": "public-only", "tls_roots": "system", "no_checkout": true, "refusals": ["submodules", "hooks", "filters", "alternates", "external-diff-clean-drivers", "git-lfs"], "budgets": ["depth", "files", "objects", "bytes"]}'::jsonb
);
