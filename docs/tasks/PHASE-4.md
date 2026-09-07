# PHASE-4: universal resource and evidence pipeline

## Metadata

- Tree ID: `PHASE-4`
- Status: `active`
- Roadmap lane: Phase 4 (`ROADMAP.md` §20.6); Evidence track
- Created: `2026-09-05`
- Estimate: 16–27 engineer-weeks
- Depends on: hardened authorization, budgets, object store, observability
- Exit: G4. Unsupported, denied, mutable, or non-reproducible resources fail explicitly rather than becoming fabricated evidence.

## Goal

Universal *reference* contract with staged built-in capability packs. Acceptance
of a URI is not a promise the core can resolve it.

## Non-Goals

- A complete Web crawler, search engine, browser farm, or media platform in the core.
- Interpreting “anything reachable” as unsafe arbitrary fetch.

## Task Tree

- ID: `PHASE-4.1`
  Status: `done`
  Goal: universal `ResourceReference` and resolver capability registry
  Backlog: 31
  Roadmap: §12.1–12.2
  ADR: 011, 018
  Children: `.1.1`–`.1.3` (decomposed `2026-09-07` at the census
    seams): `.1.1` ADR-011 + ADR-018 (the content-addressing format
    + the sandbox/isolation classes, accepted with the dev
    profile's answers) → `.1.2` the typed `ResourceReference` (the
    §12.1 contract, the immutable locator, the submission verb) →
    `.1.3` the resolver capability registry (the §12.2 advertises +
    the authz→risk→rank resolution order + the explicit
    `resource_unresolvable_now`).
  Done (`2026-09-07`): Phase 4 opens — the Phase-2/3 closes
    delivered the authz/budgets legs of the blocker (the object
    store is this phase's own `.6` lane). The `.1` census found the
    resource surface a GREENFIELD: the §9.8 registry carries the
    `resource_unresolvable` reason name and the contributions carry
    the typed `EvidenceRef`s (the Phase-1 §8.5 shape) but no
    `ResourceReference` type, no resolver registry, no submission
    verb exists (`grep -rn "ResourceReference\|unresolvable"
    crates/` → only the reason-code mapping); ADR-011 + ADR-018 are
    unopened. Children at those seams — frontier → `.1.1`.
    promotion: declined (the census is the leaf's recorded contract — the `.1.1`–`.1.3` children execute it).
  - ID: `PHASE-4.1.1`
    Status: `proposed`
    Goal: ADR-011 (the object store + the content-addressing
      format) + ADR-018 (the resolver sandbox/runtime + the
      network isolation), accepted with the dev profile's answers:
      the content-addressing format pins the digest scheme the
      snapshots (`.6`) and the references' `expected_digest` share;
      the isolation classes (the sandbox level + the egress class
      the §12.2 registry advertises) pin the shape the resolver
      packs (`.2`–`.4`) declare — the STORE itself rides the `.6`
      snapshots lane (the trigger named). No code.
    Backlog: 31 (the ADR half)
    ADR: 011, 018
    Acceptance: the two ADRs accepted (the format + the isolation
      classes named); no code changes.

  - ID: `PHASE-4.1.2`
    Status: `proposed`
    Goal: the typed `ResourceReference` — the §12.1 contract (the
      resource id, the IMMUTABLE original locator, the scheme, the
      media-type hint, the expected digest, the fragment/selector,
      the credential-binding ref (opaque — never a secret), the
      owning node/capability, the visibility scope, the purpose,
      the retention class, the risk class, the submitted-by) with
      the deny-unknown-fields boundary + the submission verb (the
      reference lands in a durable table; the locator's immutability
      is the update-refusal, not a convention).
    Backlog: 31 (the contract half)
    Acceptance: the typed reference + the submission land, measured
      (the immutability + the unknown-field refusals); no
      regression.

  - ID: `PHASE-4.1.3`
    Status: `proposed`
    Goal: the resolver capability registry — the §12.2 advertises
      (the schemes + the locator patterns, the media types + the
      max bytes, the abilities, the auth classes, the egress class,
      the sandbox level, the policies, the snapshot/derivation
      formats, the latency range, the version + the security
      evidence) as a durable registry + the resolution surface
      (the authz + the risk filters FIRST, then the rank of the
      eligible resolvers) + the explicit `resource_unresolvable_now`
      result (the reference is PRESERVED for later — never
      fabricated). The dev profile's resolvers are the future
      packs (`.2`–`.4`): the registry ships the SHAPE with the
      explicit-unsupported results measured.
    Backlog: 31 (the registry half)
    Acceptance: the registry + the resolution order land; the
      unresolvable-now result preserves the reference, measured; no
      regression.

- ID: `PHASE-4.2`
  Status: `proposed`
  Goal: pack R0 — safe HTTPS documents/pages (SSRF/DNS/redirect/size/content defenses, snapshot receipt)
  Backlog: 32
  Roadmap: §12.3–12.4

- ID: `PHASE-4.3`
  Status: `proposed`
  Goal: pack R1 — public Git with immutable commit resolution, limits, submodule/LFS policy
  Backlog: 33
  Roadmap: §12.5

- ID: `PHASE-4.4`
  Status: `proposed`
  Goal: pack R2 — PDFs/text/structured feeds/archives in sandboxed extraction workers
  Backlog: 34
  Roadmap: §12.3

- ID: `PHASE-4.5`
  Status: `proposed`
  Goal: opt-in private/authenticated connectors (R5) and sandboxed browser/agent-mediated acquisition (R3/RX)
  Roadmap: §12.3, §12.8
  Note: highest risk; do not enable by default

- ID: `PHASE-4.6`
  Status: `proposed`
  Goal: content-addressed snapshots, derivation graph, claim-evidence graph, citation validation, license/retention, freshness
  Backlog: 35
  Roadmap: §12.6–12.7, §12.9

- ID: `PHASE-4.7`
  Status: `proposed`
  Goal: G4 hostile-content suite; explicit failure for unsupported references
  Gate: G4; subtraction record required

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `PHASE-4.1.1` | `proposed` | `.1` decomposed at the census seams (the greenfield contract + the registry + the two unopened ADRs); the ADR-011 + ADR-018 records execute now |

## Changelog

- `2026-09-05`: Created from `ROADMAP.md` §20.6, §12, backlog 31–35.
- `2026-09-07`: Opened by the Phase-2/3 closes (the authz/budgets
  legs of the blocker; the object store is this phase's `.6`); `.1`
  decomposed at the census seams (the resource surface is a
  greenfield: the reason name + the typed `EvidenceRef`s exist, the
  contract + the registry + ADR-011/018 do not); children `.1.1`
  (the two ADRs) → `.1.2` (the typed reference) → `.1.3` (the
  registry); frontier → `.1.1`.
