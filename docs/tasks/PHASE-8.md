# PHASE-8: federation, interoperability, and ecosystem

## Metadata

- Tree ID: `PHASE-8`
- Status: `active` (opened `2026-09-08` — the `.1` block LIFTS: the
  trust contracts (the workload identity, the enrollment policy, the
  revocation ladder, the audit groundwork) and the compatibility
  contracts (the wire envelopes, the ADR-027 adapter ladder) ship +
  are measured by the Phase-7 guard of record)
- Roadmap lane: Phase 8 (`ROADMAP.md` §20.10)
- Created: `2026-09-05`
- Estimate: 16–30 engineer-weeks
- Depends on: stable trust and compatibility contracts
- Exit: G8. External implementations interoperate without sharing ReasonBraid’s database or trusting its internal types.

## Goal

MCP and A2A are interoperability boundaries, not ReasonBraid’s governance
protocol. Record semantic losses. Federation is explicit, not a transitive
default.

## Non-Goals

- Fully decentralized federation in the first Internet-capable release (already a v1 non-goal; this phase is post-v1 unless a narrower interop slice is pulled forward).
- Letting an A2A Agent Card confer ReasonBraid enrollment or governance authority.

## Task Tree

- ID: `PHASE-8.1`
  Status: `done`
  Goal: explicit federation trust agreements, tenant-to-tenant visibility, remote recruitment, portable agent cards/profiles, cross-domain audit receipts
  ADR: 026
  Roadmap: §6.6 Federation profile
  Kill: remote domain must not authorize local effects without a local grant (`ROADMAP.md` §25)
  Done (`2026-09-08`): the census at the seams — the
    block LIFTS and the lane decomposes. Measured: the
    TRUST contracts ship + are measured (the `.1.2`
    mTLS workload identity + the cert proof, the `.2.4`
    enrollment policy's staged ladder, the `.2.7`
    revocation drill, the audit linkage groundwork —
    the Phase-7 guard of record); the COMPATIBILITY
    contracts ship (the versioned wire envelopes, the
    ADR-027 adapter verification ladder, the pinned
    interfaces); the lane's five pieces split at the
    seams: the visibility scopes ship in their
    intra-tenant form (the network-pseudonym class +
    the ADR-034 explicit opt-in — the FEDERATION form
    is the contract's), the remote recruitment, the
    portable cards, and the cross-domain receipts are
    the greenfield (ADR-026 is reserved — no record).
    Children: `.1.1` the census + ADR-026 → `.1.2` the
    visibility + the remote recruitment → `.1.3` the
    portable cards/profiles → `.1.4` the receipts.
  Children: `.1.1`–`.1.4`

  - ID: `PHASE-8.1.1`
    Status: `done`
    Goal: the census + ADR-026 — the federation
      trust-agreement contract: the explicit agreement
      shape (a federated domain is a NAMED trust, never
      a transitive default), the local-grant rule (the
      kill line: the remote domain never authorizes
      local effects — the local grant is the only
      authority), the visibility/recruitment/receipt
      vocabulary over the shipped machinery.
    ADR: 026
    Roadmap: §6.6
    Done (`2026-09-08`): ADR-026 accepted
      (evidence-gated) — `docs/adr/026-federation-trust-agreement.md`
      (top-level `answers:`): the federation is
      EXPLICIT — the agreement record is the single
      capability source (no record, no cross-domain
      effect; never a transitive default); the remote
      domain NEVER authorizes local effects — the
      remote agreement vouches for the remote half,
      the local grant acts locally (the §25 kill line
      as the invariant); the visibility rides the
      shipped scopes + the opt-in (no agreement → the
      network pseudonym only); the remote recruitment
      is the agreement-scoped opt-in; the receipts
      CROSS-REFERENCE (the remote domain's own
      digest-pinned records), never merge (the local
      chain stays the local truth — the ADR-022
      groundwork's federation form); the portable cards
      verify through the ADR-027 five-rung ladder
      before they confer anything. No code changed.
      Frontier → `.1.2`.

  - ID: `PHASE-8.1.2`
    Status: `done`
    Goal: the tenant-to-tenant visibility + the remote
      recruitment — the explicit opt-in machinery over
      the shipped visibility scopes + the recruitment
      (the federation form of the network-pseudonym
      class; the cross-tenant recruitment rides the
      explicit agreement, never the default).
    Roadmap: §6.6
    Done (`2026-09-08`): migration 0048 —
      `federation_agreements` (the NAMED pairing: the
      agreement_id, the two tenants, the
      `directory_visibility` + the `recruitment` scope
      columns, the proposed/accepted/revoked state);
      `federation.rs` — the propose/accept/revoke verbs
      + the EFFECTIVE check (BOTH directions accepted
      AND both carry the scope); the API gains the
      three tenant_admin-gated routes; the visibility
      widening binds at `classify_reader`: a network
      reader whose tenant holds the effective
      directory-visibility agreement reads the TENANT
      view (exactly what the agreement names — never
      beyond); no agreement (or a one-sided/revoked
      one) stays the network pseudonym; a THIRD tenant
      never inherits (the transitive-default refusal).
      The recruitment scope column ships as the
      vocabulary — the cross-tenant CALL-panel widening
      is the named deferral (the trigger: the call
      machinery's remote-panel surface). The measured
      suite (`tests/federation.rs`, LIVE — the guard's
      26th): the no-agreement network view, the
      one-sided no-widening, the typed 409 (the accept
      without a proposal), the both-sides TENANT view,
      the revocation fallback, the third-tenant
      refusal. The guard's first runs caught the
      FK-purge ripple (the federation table joined all
      15 purge lists) + the migration-boundary move
      (the migration_upgrade test's quota-backfill
      assertion dropped — the boundary is the moving
      prefix, the survival assertions are the
      boundary-independent truth).
    Acceptance:
    - [x] **ROOT CAUSE (WHY + WHERE)** — the ADR-026
      contract (the explicit agreement as the single
      capability source) had no machinery; the
      widening binds at the reader-classification seam
      (the single choke point for the profile reads).
      Evidence: `cargo test -p reasonbraid-server
      --test federation` → `test result: ok. 1 passed;
      0 failed`.
    - [x] **ADDRESSED** — the migration + the verbs +
      the effective-pair check + the widening + the
      measured legs. Evidence: `bash scripts/run_pg_tests.sh`
      → rc=0, 26 suites (`target/pg_federation_guard6.log`).
    - [x] **NO REGRESSION** — `bash scripts/run_pg_tests.sh`
      → rc=0, 26 suites + the demo `ALL acceptance
      checks passed` (`target/pg_federation_guard6.log`);
      `cargo test --all` → rc=0, 71 suites
      (`target/federation_offline.log`); clippy/fmt
      clean; `make gate` → 13/13.
    - [x] **LESSON PROMOTED** — none new: the
      FK-purge ripple + the migration-boundary move
      are the known recurring lessons (the purge-list
      doctrine + the `.1.3.2` boundary note).

  - ID: `PHASE-8.1.3`
    Status: `done`
    Goal: the portable agent cards/profiles — the
      export/import shape (the digest-pinned portable
      form of the §10.1 profile + the capability
      declaration), the ADR-027 verification ladder
      applied to the imported card.
    Roadmap: §6.6
    Done (`2026-09-08`): the portable card ships —
      `cards.rs` (the `AgentCard` — the origin identity
      + the §10.1 profile + the canonical field order +
      the `sha256:<hex>` digest) + the two routes: the
      EXPORT (`GET /v1/profiles/{role_id}/card` — the
      role itself or the tenant admin mints the
      portable form) and the IMPORT (`POST
      /v1/profiles/cards/import` — the ADR-027 ladder:
      the digest rung (the re-derivation of the
      canonical bytes), the compatibility rung (the
      `agent-card/1` schema), the allowlist rung (the
      EFFECTIVE recruitment agreement with the origin —
      the `.1.2` machinery's `has_effective_recruitment_agreement`),
      the capability rung (the fresh local role + the
      boundary-checked DEFAULT grant — the card's
      self-asserted capabilities NEVER confer authority,
      the ADR-026 invariant — + the imported profile
      through the content-addressed write path). The
      measured suite (`tests/cards.rs`, LIVE — the
      guard's 27th): the export, the no-agreement
      403, the tampered-card digest refusal, the
      unknown-schema refusal, the clean import (the
      fresh local role + the imported profile), the
      default-grant-only invariant. The guard's first
      runs caught the profile-FK purge gaps (the
      profile tables joined three more lists).
    Acceptance:
    - [x] **ROOT CAUSE (WHY + WHERE)** — the ADR-026
      card clause had no machinery (the portable form +
      the ladder); the digest-pinned canonical card +
      the four rungs land. Evidence: `cargo test -p
      reasonbraid-server --test cards` → `test result:
      ok. 1 passed; 0 failed`.
    - [x] **ADDRESSED** — the export + the import
      ladder + the measured legs. Evidence: `bash
      scripts/run_pg_tests.sh` → rc=0, 27 suites
      (`target/pg_cards_guard5.log`).
    - [x] **NO REGRESSION** — the guard → rc=0, 27
      suites + the demo `ALL acceptance checks passed`
      (`target/pg_cards_guard5.log`); `cargo test --all`
      → rc=0, 72 suites (`target/cards_offline.log`);
      clippy/fmt clean; `make gate` → 13/13.
    - [x] **LESSON PROMOTED** — none new: the FK-purge
      ripple is the known recurring lesson.

  - ID: `PHASE-8.1.4`
    Status: `done`
    Goal: the cross-domain audit receipts — the
      receipt shape over the shipped audit linkage (the
      ADR-022 groundwork's federation form: the remote
      domain's receipt references its OWN records; the
      local chain stays the local truth).
    Roadmap: §6.6
    Done (`2026-09-08`): migration 0049 —
      `cross_domain_receipts` (the receipt_id, the two
      tenants, the kind (`card_import` | `agreement`),
      the `remote_ref` — the remote domain's
      digest-pinned reference, the `local_ref` — the
      local record it attached to); `receipts.rs` — the
      in-transaction record (the receipt commits WITH
      the cross-domain action, never a follow-up) + the
      read surface; the card IMPORT records the receipt
      in its transaction (the remote_ref = the card's
      digest, the local_ref = the fresh role); the read
      surface (`GET /v1/audit/receipts?tenant_id=…`,
      the tenant_admin gate) lists the tenant's
      receipts. The receipts CROSS-REFERENCE, never
      merge: the remote reference is verifiable against
      the REMOTE domain's records; the local chain
      stays the local truth (the ADR-022 groundwork's
      federation form). The measured legs ride the
      cards suite (the guard's 27th — the receipt row
      after the import + the read surface). The
      guard's first runs caught the receipts-FK purge
      ripple (the table joined the 18 tenant-purging
      lists). **The `.1` lane is COMPLETE.** Frontier →
      `.2`.
    Acceptance:
    - [x] **ROOT CAUSE (WHY + WHERE)** — the ADR-026
      receipt clause (the cross-reference, never the
      merge) had no machinery; the receipt rides the
      cross-domain actions' transactions. Evidence:
      `cargo test -p reasonbraid-server --test cards`
      → `test result: ok. 1 passed; 0 failed` (the
      extended receipt legs).
    - [x] **ADDRESSED** — the migration + the record +
      the read + the import wiring. Evidence: `bash
      scripts/run_pg_tests.sh` → rc=0, 27 suites
      (`target/pg_receipts_guard3.log`).
    - [x] **NO REGRESSION** — the guard → rc=0, 27
      suites + the demo `ALL acceptance checks passed`
      (`target/pg_receipts_guard3.log`); `cargo test
      --all` → rc=0, 72 suites (`target/receipts_offline.log`);
      clippy/fmt clean; `make gate` → 13/13.
    - [x] **LESSON PROMOTED** — none new: the FK-purge
      ripple is the known recurring lesson.

- ID: `PHASE-8.2`
  Status: `done`
  Goal: current A2A interoperability for compatible task/message exchange
  ADR: 025
  Roadmap: §9.7
  Baseline: official `a2a-rs` crates published as of 2026-09-04; pin and revalidate
  Done (`2026-09-08`): the census at the seams —
    decomposed. Measured: NOTHING ships for A2A (the
    §9.7 facade is the greenfield); the contract is
    precise (the facade boundary — A2A is the
    interoperability facade, never the governance
    protocol; the semantic-losses map — the
    authority/budget/evidence/decision-rule/lifecycle
    each record their loss; the Agent Card confers
    nothing; the official `a2a-lf`/`a2a-client-lf`/
    `a2a-server-lf` crates + the `a2a-cli` exist on
    crates.io (the 2026-09-04 baseline, re-checked —
    the pre-1.0 compatibility must be DEMONSTRATED,
    not inferred from SemVer; the exact versions pin at
    the spike + the tested spec/conformance revision
    records)); ADR-025 is reserved (no record).
    Children: `.2.1` the census + ADR-025 → `.2.2` the
    dependency census + the pin → `.2.3` the facade →
    `.2.4` the compatibility demonstration + the
    qualification.
  Children: `.2.1`–`.2.4`

  - ID: `PHASE-8.2.1`
    Status: `proposed`
    Goal: the census + ADR-025 — the A2A surface +
      the version-profile contract: the facade
      boundary (the message mapping where the
      semantics align; the local governance stays),
      the semantic-losses vocabulary (each lost
      dimension names its loss), the version-pin +
      the revalidation stance (the pre-1.0
      compatibility demonstrated).
    ADR: 025
    Roadmap: §9.7

  - ID: `PHASE-8.2.2`
    Status: `proposed`
    Goal: the dependency census + the pin — the
      `a2a-lf` crate family's exact versions + the
      supply-chain gate (the deny + the license
      checks), the JSON-RPC/REST profile selection
      (test only what the slice needs), the
      `Cargo.lock` pin + the tested revision record.
    Roadmap: §9.7

  - ID: `PHASE-8.2.3`
    Status: `proposed`
    Goal: the A2A facade — the compatible task/message
      exchange over the JSON-RPC/REST profile: the
      incoming A2A messages map to the local commands
      (the local grants/authorization ride every
      effect), the external IDs + the signatures
      preserved, the semantic losses recorded per
      message.
    Roadmap: §9.7

  - ID: `PHASE-8.2.4`
    Status: `proposed`
    Goal: the compatibility demonstration + the
      qualification — the a2a-cli (or the SDK's test
      peer) roundtrip against the facade (the measured
      exchange + the recorded semantic losses), the
      tested spec/conformance revision, the
      qualification record (the broad claims stay
      gated; the gateway is deployable-off).
    Roadmap: §9.7

- ID: `PHASE-8.3`
  Status: `proposed`
  Goal: MCP servers/clients for tool/resource exposure; ReasonBraid owns durable continuation of listen streams
  ADR: 024
  Roadmap: §9.6
  Baseline: MCP 2026-07-28; `subscriptions/listen` does not auto-resume

- ID: `PHASE-8.4`
  Status: `proposed`
  Goal: adapter/resolver SDK, compatibility matrix, certification suite, signed plugin registry or allowlist
  ADR: 027

- ID: `PHASE-8.5`
  Status: `proposed`
  Goal: regional routing, store-and-forward for intermittent sites, export/import, documented exit path

- ID: `PHASE-8.6`
  Status: `proposed`
  Goal: protocol extension process and independent implementation exercise; G8 exit
  Gate: G8; subtraction record required

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `PHASE-8.2.1` | `proposed` | `.2` decomposed at the census seams (nothing ships for A2A; the §9.7 contract + the 2026-09-04 baseline re-checked; ADR-025 reserved) — the census + ADR-025 execute first |

## Changelog

- `2026-09-08`: `.2` done — the A2A census at the
  seams (the §9.7 facade is the greenfield; the
  `a2a-lf` family re-checked on crates.io; ADR-025
  reserved) → decomposed `.2.1` (the census +
  ADR-025) → `.2.2` (the dependency pin) → `.2.3`
  (the facade) → `.2.4` (the compatibility
  demonstration); frontier → `.2.1`.
- `2026-09-08`: `.1.4` done — the cross-domain audit
  receipts (migration 0049 + the in-transaction
  record + the read surface + the import wiring + the
  measured legs); **the `.1` lane is COMPLETE**;
  frontier → `.2`.
- `2026-09-08`: `.1.3` done — the portable agent
  cards (the canonical digest-pinned export + the
  four-rung import ladder — the digest, the schema,
  the agreement-allowlist, the local-grant capability
  — + the measured suite); frontier → `.1.4`.
- `2026-09-08`: `.1.2` done — the federation
  machinery (migration 0048's agreement pairing + the
  propose/accept/revoke verbs + the effective-pair
  visibility widening at the reader-classification
  seam + the measured suite; the recruitment
  widening is the named deferral); frontier → `.1.3`.
- `2026-09-08`: `.1.1` done — ADR-026 accepted
  (evidence-gated): the federation is explicit + the
  local grant is the only authority that acts locally
  + the receipts cross-reference + the cards verify
  through the ADR-027 ladder; no code; frontier →
  `.1.2`.
- `2026-09-08`: the tree opens — `.1` done (the census
  at the seams: the block LIFTS — the trust +
  compatibility contracts ship + are measured; the
  visibility ships intra-tenant, the recruitment/
  cards/receipts are the greenfield; ADR-026
  reserved) → decomposed `.1.1` (the census + ADR-026)
  → `.1.2` (the visibility + the recruitment) → `.1.3`
  (the portable cards) → `.1.4` (the receipts);
  frontier → `.1.1`.
