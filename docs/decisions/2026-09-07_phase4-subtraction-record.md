# 2026-09-07_phase4-subtraction-record.md

## Context

`PHASE-4.7.2` (the §19.8 subtraction record, the Phase-1 `.1.8.2` pattern):
what Phase 4 SHIPPED versus the `ROADMAP.md` §12 backlog the gate package
must not silently narrow.

## Shipped (the §20.6 backlog rows 31–35)

| Row | Shipped | The evidence |
| --- | --- | --- |
| 31 — the universal reference + the resolver registry | the typed `ResourceReference` (migration 0023, the immutable locator + the replay/conflict), the `resolver_capabilities` registry (0024) with the filter-then-rank resolution + the explicit `resource_unresolvable_now`, the media-type + auth filters | profiles 13–18 |
| 32 — the safe HTTPS resolver | the SSRF classifier (the 18-case matrix), the safe fetcher (the two-layer destination enforcement, the manual redirects, the ratio brake), the receipt + the pack wiring | `src/ssrf.rs`, `src/fetcher.rs`; profiles 14/15 |
| 33 — the public Git resolver | the R1 contract + the gix acquisition (the classified transport, the budgets, the refusals, the no-checkout, the resolved commit) + the receipt + the wiring | `src/git.rs`; profiles 16 |
| 34 — the extraction worker | the R2 contract + the worker crate (the four formats, the named refusals, the Derivation chunks) + the receipt + the media-type-routed pipeline | `crates/reasonbraid-extract`; profiles 17 |
| 35 — the evidence store/graph | the snapshot store + the tombstone (0028), the derivation graph (0029), the claim-evidence graph + the citation validation (0030), the retention + the freshness (0031) | `src/snapshots.rs`, `src/derivations.rs`, `src/claims.rs`; profiles 19–22 |

Plus the gated lane: the R3/R5/RX contracts + the machinery + the gate
(the startup sync, the disabled-pack-has-no-row rule) — the `.5.1` decision
records the off-by-default doctrine.

## Subtracted (deferred, named — not silent)

1. **The R3 enablement**: the browser pack stays gated without a real
   `vm_container` boundary — no current deployment profile can open it.
2. **The RX agent round-trip**: the §12.8 vocabulary + the publication
   shape ship; the delivery rides the capability-call lane.
3. **The media formats beyond the four**: PDF/zip/tar/feeds only; the rest
   are the named `media_type_unsupported` refusal.
4. **The browser-engine vendoring**: the pinned chromium is provenance-
   named, not vendored (the pure-Rust doctrine's honest boundary).
5. **The git/browse snapshot auto-submission**: the R0/R2/R5 acquisitions
   auto-submit; the git + browse landing rides the `.6.4` freshness
   lane's source-snapshot shape.

## Not shipped at all (the roadmap never promised them here)

- The §12.8 second-verifier ENFORCEMENT (the vocabulary carries the rule;
  the enforcement is the capability-call lane's policy).
- The `.6.4` retention EXPIRY scheduler (the enforcement is the verb; the
  scheduled trigger rides the operations lane).
