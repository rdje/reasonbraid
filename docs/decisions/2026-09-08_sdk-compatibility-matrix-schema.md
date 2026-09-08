# The SDK compatibility-matrix schema (`PHASE-8.4.1`)

- **Status:** accepted (the `.4.2` leaf fills the matrix against this schema)
- **Date:** 2026-09-08
- **Leaf:** `PHASE-8.4.1`
- **Requirements:** `ROADMAP.md` §9.8 (the ecosystem/compatibility lane), ADR-027

## Context

The `.4` census measured the greenfield: no document maps the adapter/resolver
surface against the profiles, the platforms, and the qualification status.
This record fixes the matrix's SCHEMA — the columns, the fill rules, and the
re-derivation contract — so the `.4.2` fill is a mechanical exercise, never a
prose claim.

## Decision

- **The matrix's columns (the row vocabulary):**
  1. `sdk_version` — the adapter contract's version token (`SDK_VERSION`, bumped
     on any contract change);
  2. `surface_id` — the adapter id (the dev three + any third-party) or the
     resolver id (the built-in packs);
  3. `protocol_profile` — the protocol/transport the surface speaks (the A2A
     JSON-RPC/REST profile, the MCP 2026-07-28 baseline, the thread command
     protocol, the §12.2 resolver scheme);
  4. `platform` — the toolchain/OS cell (the measured matrix rows carry the
     measured platform; the unmeasured cells are NAMED untested);
  5. `qualification` — the demonstrated status: `conformance-suite`,
     `live-roundtrip`, `fixture-corpus`, `manual-only`, or `untested`;
  6. `evidence` — the re-derivable artifact (the suite run, the fixture
     manifest digest, the live demonstration log).
- **The fill rules:**
  - a cell is filled ONLY from a measured run (the §19.4 harness, the live
    roundtrip, the fixture corpus) — a qualified cell cites its evidence
    artifact;
  - an unmeasured cell is the explicit `untested`, never blank and never
    inferred from a sibling cell (the A2A/MCP discipline: the compatibility
    is demonstrated, not inferred from the SemVer);
  - a contract bump invalidates the `sdk_version` column's rows (the
    re-qualification rides the bump, the ADR-027 ladder's digest rung).
- **The re-derivation contract:** the matrix is a GENERATED artifact where
  possible — the `.6.2` failure-fixture corpus is the replay oracle; a cell
  that disagrees with the corpus's manifest is a drift error, not a note.

## answers:

- **The matrix is evidence-bound, not prose-bound**: every qualified cell
  names the run that produced it; every unqualified cell says `untested`.
- **The version token is the matrix's first axis**: the bump invalidates,
  the re-derivation follows — the same re-derive principle as the claim
  verification.
