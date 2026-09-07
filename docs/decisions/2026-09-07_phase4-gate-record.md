# 2026-09-07_phase4-gate-record.md

## Context

`PHASE-4.7.2` (the G4 exit gate package, the Phase-1 `.1.8.2` pattern):
Phase 4 ships the universal resource + evidence pipeline — the packs R0–R2
live + the gated R3/R5/RX lane, the snapshot/derivation/claim store, and
the retention/freshness enforcement. The gate: `ROADMAP.md` §20.6's **G4** —
unsupported, denied, mutable, or non-reproducible resources fail explicitly
rather than becoming fabricated evidence.

## Decision

- **G4 outcome: Met.** The evidence manifest
  (`docs/evidence/2026-09-07_phase4-evidence-manifest.md`) maps every G4
  clause to a re-runnable artifact; the consolidated hostile suite
  (profiles 23) is the gate's one-test citation.
- **The named deferrals** (each recorded, none silent):
  1. **The R3 container gate**: the browser pack's `vm_container`
     requirement stays gated in every current deployment profile — the
     hostile-JS render is refused by default; the enablement is the `.5.1`
     decision's recorded change.
  2. **The RX delivery**: the §12.8 vocabulary + the capability-call
     publication ship; the actual agent round-trip rides the
     capability-call lane.
  3. **The media formats beyond the four**: PDF/zip/tar/feeds ship; the
     remaining formats are the named `media_type_unsupported` refusal.
  4. **The browser-engine provenance**: the pinned chromium is not vendored
     (the pure-Rust doctrine ends at the browser engine) — the startup
     version check + the `R3_BROWSER_BIN` override carry the provenance.
  5. **The git/browse snapshot auto-submission**: the R0/R2/R5 acquisitions
     auto-submit; the git + browse receipts' snapshot landing rides the
     `.6.4` freshness lane's defined source-snapshot shape.
- **The subtraction record** (`2026-09-07_phase4-subtraction-record.md`)
  lists what shipped vs the §12 backlog — no empty lists.

## Consequences

- The tree closes (`PHASE-4` `done`), the frontier moves to `PHASE-5.1`,
  and the book's roadmap chapter reflects the completion.

answers:

- **The explicit failure is the pipeline's load-bearing wall.** Every
  unsupported, denied, mutable, or non-reproducible path has a TYPED
  refusal with a measured test — the G4 property is a suite, not a
  promise.
- **The supply-chain gate grew with the packs.** The R2/R3 crates added
  the digest/rand/nom/reqwest duplicate families + the uluru MPL-2.0
  license — each reviewed, the narrowest exceptions recorded in
  `deny.toml`, and the re-run green at the close.
