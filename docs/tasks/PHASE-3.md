# PHASE-3: directory, unknown membership, and autonomous initiation

## Metadata

- Tree ID: `PHASE-3`
- Status: `active`
- Roadmap lane: Phase 3 (`ROADMAP.md` §20.5); Network track
- Created: `2026-09-05`
- Estimate: 12–19 engineer-weeks
- Depends on: stable identity, inbox, and grants
- Exit: an authorized agent on an unacquainted host can recruit appropriate available participants without enumerating the entire network; opt-out, visibility, capacity, and budget remain enforceable under storm tests

## Goal

Unknown-membership initiation: initiators do not need a roster. Deterministic
eligibility before ranking. Dependence indicators, never an independence score.

## Non-Goals

- Claiming statistical independence of agents.
- Learned ranking as an authorization control (shadow mode only).

## Task Tree

- ID: `PHASE-3.1`
  Status: `done`
  Goal: capability/interest/visibility profiles and versioned embeddings
  Backlog: 26
  ADR: 014
  Children: `.1.1`–`.1.3` (decomposed `2026-09-07` at the census
    seams): `.1.1` ADR-014 (the semantic-index engine + the embedding
    lifecycle, accepted with the dev profile's answer) → `.1.2` the
    profile schema + the write surface (the typed §10.1 fields, the
    versioned history, the owner-attested claims) → `.1.3` the
    visibility enforcement + the read surface (the per-field policy
    evaluated per reader; the privacy-filtered views the `.2` lane
    consumes).
  Done (`2026-09-07`): Phase 3 opens — the Phase-2 close delivered its
    blocker (stable identity, inbox, and grants). The `.1` census
    found NOTHING shipped for the directory profile: the §10.1
    registration fields have no table, no write verbs, no read
    surface, and no visibility policy — the incarnation writer
    (`.1.6.1`) carries only the lineage sliver (provider/model/
    harness/config); ADR-014 (the semantic-index engine + embedding
    lifecycle) is unopened. Children at those seams — frontier →
    `.1.1`.
    promotion: declined (the census is the leaf's recorded contract — the `.1.1`–`.1.3` children execute it).
  - ID: `PHASE-3.1.1`
    Status: `proposed`
    Goal: ADR-014 — the directory semantic-index engine + the
      embedding lifecycle, accepted with the dev profile's answer:
      the two-stage matching (§10.3) evaluates DETERMINISTIC
      structural eligibility first (the profile's typed fields), and
      the learned embedding/index machinery arrives with its trigger
      (the first ranked recruitment whose semantic search is measured
      to matter). The §10.1 rule pins the claims: a high self-declared
      score is never equivalent to verified competence — the
      provenance rides every capability claim. No code.
    Backlog: 26 (the ADR half)
    ADR: 014
    Acceptance: ADR-014 accepted (the dev answer named; the embedding
      trigger named); no code changes.

  - ID: `PHASE-3.1.2`
    Status: `proposed`
    Goal: the profile schema + the write surface — migration 0019:
      the `agent_profiles` table carrying the §10.1 registration
      fields (the stable role id/label, the tenant/owner/home node,
      the purpose + conversation modes, the capabilities with
      taxonomy/confidence/evidence/expiry, the interests +
      subscriptions, the languages/formats, the scopes +
      confidentiality classes, the incarnation lineage (the link to
      the `.1.6.1` rows), the availability/concurrency/wake policy,
      the resolver/tool capabilities, the cost/latency class + the
      local ceilings, the per-field visibility policy, and the
      grants-by-reference — never self-declared authority) + the
      CONTENT-ADDRESSED version history (every profile update is a
      new version; the old ones stay readable) + the write verbs
      (the role updates its own profile; the owner attests capability
      claims with the provenance).
    Backlog: 26 (the schema + writes half)
    Acceptance: the schema + the versioned writes land, measured; no
      regression.

  - ID: `PHASE-3.1.3`
    Status: `proposed`
    Goal: the visibility enforcement + the read surface — the
      per-field visibility policy (self/tenant/network/public)
      evaluated PER READER at read time (the reader's grants decide
      which fields exist in the response — a field hidden from a
      reader is absent, not nulled), the read verbs (the owner's own
      full view; the tenant-scoped view; the privacy-filtered
      network view the `.2` lane consumes), and the audit of every
      read-side visibility decision. The self-asserted vs attested
      provenance is shown, never flattened.
    Backlog: 26 (the visibility half)
    Acceptance: the per-reader filtering is measured (the adversarial
      reads: the same profile read by the owner, a tenant sibling,
      and a stranger yields exactly the allowed fields each); no
      regression.

- ID: `PHASE-3.2`
  Status: `proposed`
  Goal: lease-based presence; offline-known distinction; privacy-filtered views
  Backlog: 27
  Roadmap: §10.2

- ID: `PHASE-3.3`
  Status: `proposed`
  Goal: two-stage matching — deterministic eligibility then explainable ranking
  Backlog: 28, 29
  Roadmap: §10.3

- ID: `PHASE-3.4`
  Status: `proposed`
  Goal: recruitment protocol, capacity reservations, invitation fairness, anti-storm, privacy-safe explanations
  Backlog: 29, 30
  ADR: 015
  Roadmap: §10.5, §10.7

- ID: `PHASE-3.5`
  Status: `proposed`
  Goal: subscriptions, durable notifications, wake policies, node-initiated thread API
  Backlog: 30
  Roadmap: §10.6, §11.5

- ID: `PHASE-3.6`
  Status: `proposed`
  Goal: dependence indicators and controlled participant-selection strategies
  Roadmap: §10.4
  Acceptance: UI never labels these “independent probability”

- ID: `PHASE-3.7`
  Status: `proposed`
  Goal: churn/partition simulation and relevance/diversity evaluation; storm tests
  Gate: Network-track preview; subtraction record required

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `PHASE-3.1.1` | `proposed` | `.1` decomposed at the census seams (nothing shipped for the §10.1 profile; ADR-014 unopened); the ADR-014 record executes now |

## Changelog

- `2026-09-05`: Created from `ROADMAP.md` §20.5, §10, backlog 26–30.
- `2026-09-07`: Opened by the Phase-2 close (stable identity, inbox,
  and grants — the tree's blocker — shipped); `.1` decomposed at the
  census seams (the §10.1 profile is a greenfield: no table, no
  verbs, no visibility policy, ADR-014 unopened); children `.1.1`
  (ADR-014) → `.1.2` (the schema + writes) → `.1.3` (the visibility
  enforcement + reads); frontier → `.1.1`.
