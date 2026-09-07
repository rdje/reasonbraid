# ADR-011 — Object store and content-addressing format: the digest scheme pins the snapshots; the store rides the `.6` lane

- **Status:** `accepted` (evidence-gated — the references' `expected_digest`
  field (`.1.2`'s contract) and the snapshot receipts (`.2`'s pack) need ONE
  digest scheme BEFORE either lands; the §9.8 registry already names the
  failure vocabulary)
- **Date:** `2026-09-07`
- **Leaf:** `PHASE-4.1.1`
- **Requirements:** `ROADMAP.md` §23 queue item 011 (object store and
  content-addressing format); §12.1 (the `expected_digest`), §12.6 (the
  content-addressed snapshots)

## Context

The roadmap queued "object store and content-addressing format" as ADR-011.
The `.1` census found the resource surface a greenfield: no store, no
snapshots, no digest convention — but the §12.1 reference contract carries
`expected_digest` and the `.6` lane will land the content-addressed snapshots.
The ADR must pin the FORMAT before either lands, so the reference's expected
digest and the snapshot's computed digest speak the same language.

## Options

1. **Pin the digest scheme now; the store rides the `.6` snapshots lane** —
   `sha256:<hex>` over the canonical bytes (the bytes as acquired, before any
   transformation; the canonicalization is scheme-specific and separate —
   §12.1's rule). The store's layout + the derivation graph are the `.6`
   lane's (the trigger: the first resolver pack's snapshot receipt, `.2`).
2. Defer the format until the store exists — the `.1.2` reference's
   `expected_digest` would then be an untyped string, and the `.2` pack's
   snapshot receipts would invent the scheme ad hoc (two surfaces drifting
   apart from day one).

## Evidence

- **Two future consumers already name the digest**: the §12.1 reference
  (`expected_digest?`) and the §12.6 snapshots (the content-addressed store).
- **The §12.1 immutability rule implies a canonical byte string** — the
  original locator is immutable; the digest binds the ACQUIRED bytes, so the
  scheme must name the byte source (as acquired), not a transformed form.

## Decision

Accept option 1. The content-addressing format is `sha256:<hex>` over the
acquired bytes (as acquired, pre-transformation; the scheme-specific
canonicalization for cache/deduplication stays separate and must not erase
the security-relevant distinctions — §12.1). The object store's layout,
the derivation graph, and the retention machinery land with the `.6`
snapshots lane behind the trigger (the first resolver pack's snapshot
receipt).

## Consequences

- The `.1.2` `expected_digest` is a TYPED `sha256:<hex>` (the parse refuses
  anything else).
- The `.2`–`.4` packs' snapshot receipts speak the same scheme from the
  first pack.
- No store tables, no derivation-graph columns, no retention machinery in
  the `.1` lane (the `.6` lane owns them).

## Revisit trigger

The first resolver pack's snapshot receipt (`.2`) — the store + the
derivation graph land as their own lane with this ADR's format.
