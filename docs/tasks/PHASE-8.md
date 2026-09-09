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

## Current qualification correction

`SIGNOFF-REPAIR` owns the source-review findings discovered after the historical
`.5.2` closure. Shared adapter/region HTTP operations now use explicit site
operator authority (`docs/decisions/2026-09-09_site-operator-authority.md`);
`SIGNOFF-REPAIR.3.2.3` owns the current live HTTP verification. The earlier
any-tenant-admin behavior and its positive fixtures below are historical records.
MCP authorization/continuation, A2A transport, federation and adapter certification
also have source-review records. Their previous test results retain provenance;
they do not close the untested guarantees. Repairs precede extending delivery.
The shared-write escalation is runtime-confirmed in the corrective tree; remaining
findings require their own runtime disposition.

## Current correction synchronization — SIGNOFF-REPAIR.3.2.3

Implementation ownership and remaining verification are in `docs/tasks/SIGNOFF-REPAIR.md`.

- [x] **ROOT CAUSE (WHY + WHERE)** — the legacy registry probe reproduced shared tenant-admin writes after boundary revocation, HTTP 200 and SQL witness 1|2|2|1, rc=0. Historical success fixtures below did not prove site isolation.
- [x] **ADDRESSED (verified)** — the corrected HTTP suite passed all 8 tests, rc=0, including every route, tenant/freeze refusals, actual-parent liveness, audit rollback, wire errors and both revocation orders. This index records the correction while preserving the old execution evidence.
- [x] **NO REGRESSION** — Focused verification passed; confirmation is pending. strict focused Clippy passed, rc=0; `make book` and rendered contract inspection passed, rc=0. Existing regional routing and adapter controls passed; final repeated region/escalation verification remains tracked in the owner. No Phase-8 or Internet qualification is advanced.

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
    Follow-up (`2026-09-08`): the `.2.3` verification
      chain's `cargo fmt --all` reformatted the receipt
      assertions this leaf's suite landed — the fmt-only
      normalization commits under this leaf (the
      post-commit fmt sweep).

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
    Status: `done`
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
    Done (`2026-09-08`): ADR-025 accepted
      (evidence-gated) — `docs/adr/025-a2a-interoperability-surface.md`
      (top-level `answers:`): the FACADE boundary (A2A
      maps the compatible semantics; the message is an
      INPUT the local machinery evaluates — the local
      grants authorize, the local budgets bound, the
      local policy digests decide; the Agent Card
      confers nothing); the semantic losses are
      RECORDED per exchange (the authority, the
      budget, the evidence, the decision rule, the
      policy lifecycle — each names its loss, never a
      silent merge); the version profile (the
      JSON-RPC/REST first, the exact crate versions
      pin at `.2.2`, the tested conformance revision
      records, the pre-1.0 compatibility DEMONSTRATED
      by the `.2.4` roundtrip — never inferred); the
      qualification precedes the broad claims, the
      gateway is deployable-off. No code changed.
      Frontier → `.2.2`.

  - ID: `PHASE-8.2.2`
    Status: `done`
    Goal: the dependency census + the pin — the
      `a2a-lf` crate family's exact versions + the
      supply-chain gate (the deny + the license
      checks), the JSON-RPC/REST profile selection
      (test only what the slice needs), the
      `Cargo.lock` pin + the tested revision record.
    Roadmap: §9.7
    Done (`2026-09-08`): the census (crates.io, the
      2026-09-08 check) + the PIN DECISION recorded:
      `a2a-lf` → **0.3.0** (the 2026-05-12 release,
      Apache-2.0, Rust 1.85); `a2a-server-lf` →
      **0.4.3** (2026-08-28, Apache-2.0);
      `a2a-client-lf` → **0.2.3** (2026-08-28,
      Apache-2.0); the profile: the JSON-RPC/REST
      (the protocol binding factory's default — the
      ONLY profile the slice needs). THE SUPPLY-CHAIN
      FINDING (the workspace rule, the `.4.3.2`
      lesson, third occurrence): the crates' DEFAULT
      features enable `rustls-tls` (= `reqwest/rustls`
      → the aws-lc-rs provider) — a provider vote; the
      `.2.3` facade MUST pin `default-features = false`
      + the `rustls-no-provider` feature + the
      workspace's ring-pinned `rustls` explicitly (the
      two-provider union never enters the graph). The
      `Cargo.toml`/`Cargo.lock` additions ride the
      `.2.3` facade (the pin lands WITH the use — no
      phantom dependencies); the tested
      conformance revision records at the `.2.4`
      roundtrip. No code changed. Frontier → `.2.3`.
    Acceptance:
    - [x] **ROOT CAUSE (WHY + WHERE)** — the §9.7
      baseline demanded the exact versions + the
      revalidation; the census fixed them AND caught
      the provider-vote trap in the default features.
      Evidence: the crates.io API checks (the
      version/feature/license records).
    - [x] **ADDRESSED** — the three exact pins + the
      profile choice + the no-provider feature
      decision. Evidence: the recorded versions +
      the feature-map analysis above.
    - [x] **NO REGRESSION** — docs-only: `make gate` →
      13/13; no dependency graph changed (the pin
      lands with the `.2.3` facade).

  - ID: `PHASE-8.2.3`
    Status: `done`
    Goal: the A2A facade — the compatible task/message
      exchange over the JSON-RPC/REST profile: the
      incoming A2A messages map to the local commands
      (the local grants/authorization ride every
      effect), the external IDs + the signatures
      preserved, the semantic losses recorded per
      message.
    Roadmap: §9.7
    Done (`2026-09-08`): the facade's CORE ships — the
      new `crates/reasonbraid-a2a` (the `a2a-lf`
      **0.3.0** exact pin as `a2a`, the core types
      only — the crate carries NO features, so no
      transport, no provider vote; the server/client
      crates ride the `.2.4` demonstration WITH the
      no-provider pins per the `.2.2` decision): the
      `SemanticLosses` record (the five §9.7
      dimensions — the authority, the budget, the
      evidence, the decision rule, the policy
      lifecycle — each recorded; a bare A2A message
      carries NONE of the local machinery, so all
      five record lost), `map_message` (the text
      survives, the external role preserves as the
      wire value — an external id, never a local
      principal), `map_task_request` (the external
      task id preserves verbatim — the local
      aggregate gets its OWN id), `response_message`
      (the response round-trips the A2A shape). The
      offline suite (3 tests) runs over the REAL
      a2a-lf 0.3.0 types (the `Message::new`/
      `Part::text`/`Role` wire shapes — the first
      demonstrated compatibility). The local-command
      wiring (the mapped message → the thread
      command under the local grants) + the transport
      + the roundtrip ride the `.2.4` demonstration.
    Acceptance:
    - [x] **ROOT CAUSE (WHY + WHERE)** — the ADR-025
      facade boundary had no machinery; the core
      mapper + the loss record land over the pinned
      real types. Evidence: `cargo test -p
      reasonbraid-a2a` → `test result: ok. 3 passed;
      0 failed`.
    - [x] **ADDRESSED** — the crate (the exact pin) +
      the mapper + the loss record + the external-id
      preservation. Evidence: `cargo test -p
      reasonbraid-a2a` → 3 passed; `make deny` →
      advisories/bans/licenses/sources ok (the
      Apache-2.0 family passes the gate).
    - [x] **NO REGRESSION** — `cargo test --all` →
      rc=0 (the new crate joins the offline count);
      clippy/fmt clean; `make deny` green; `make gate`
      → 13/13; `make book` builds.
    - [x] **LESSON PROMOTED** — none new: the
      provider-vote rule is the workspace decision's
      known application.

  - ID: `PHASE-8.2.4`
    Status: `done`
    Goal: the compatibility demonstration + the
      qualification — the a2a-cli (or the SDK's test
      peer) roundtrip against the facade (the measured
      exchange + the recorded semantic losses), the
      tested spec/conformance revision, the
      qualification record (the broad claims stay
      gated; the gateway is deployable-off).
    Roadmap: §9.7
    Done (`2026-09-08`): the WIRE-level demonstration
      ships — the facade's suite gains the JSON-RPC
      2.0 roundtrip over the REAL a2a-lf 0.3.0 wire
      shapes (the `JsonRpcRequest` (`SendMessage`) →
      the serialize/deserialize → the facade maps →
      the `JsonRpcResponse::success` → the roundtrip;
      the unknown-method refusal via the typed
      `JsonRpcError` — 5 tests). The tested revision:
      the a2a-lf 0.3.0 types + the JSON-RPC 2.0
      envelope (the recorded conformance point). THE
      QUALIFICATION RECORD: the WIRE-level
      compatibility is DEMONSTRATED; the TRANSPORT
      profile (the a2a-server-lf/a2a-client-lf
      harness) stays the named follow-on — the §9.7
      "test only the profiles actually needed" rule:
      no external A2A peer exists in the dev profile,
      so the transport profile is NOT needed yet (its
      pins are the `.2.2` decision's). The broad
      Internet agent interoperability claim stays
      GATED; the gateway remains deployable-off.
      **The `.2` lane is COMPLETE.** Frontier → `.3`.
    Acceptance:
    - [x] **ROOT CAUSE (WHY + WHERE)** — the §9.7
      baseline demanded the demonstrated compatibility
      (the pre-1.0 reality, never inferred); the wire
      roundtrip demonstrates it. Evidence: `cargo test
      -p reasonbraid-a2a` → `test result: ok. 5
      passed; 0 failed`.
    - [x] **ADDRESSED** — the JSON-RPC roundtrip + the
      typed refusal + the tested revision + the
      qualification stance. Evidence: the 5-test suite
      (`cargo test -p reasonbraid-a2a` → 5 passed).
    - [x] **NO REGRESSION** — `cargo test --all` →
      rc=0, 74 ok-lines (`target/a2a24_offline.log`);
      clippy/fmt clean; `make deny` green; `make gate`
      → 13/13; `make book` builds.
    - [x] **LESSON PROMOTED** — none new.

- ID: `PHASE-8.3`
  Status: `done`
  Goal: MCP servers/clients for tool/resource exposure; ReasonBraid owns durable continuation of listen streams
  ADR: 024
  Roadmap: §9.6
  Baseline: MCP 2026-07-28; `subscriptions/listen` does not auto-resume
  Done (`2026-09-08`): the census at the seams —
    decomposed. Measured: NOTHING ships for MCP (the
    §9.6 surface is the greenfield); the contract is
    precise (the TOOL vocabulary — `ask_network`/
    `get_thread`/`respond`/`join_call`/`list_inbox`/
    `propose_policy_change`/`get_policy_bundle` — maps
    to the SAME command handlers as HTTP; the
    resources — the thread timelines, the evidence,
    the authorized policy sets; the client role; the
    remote metadata never grants authority; the
    listen-stream contract: the stream is the
    EPHEMERAL transport state, the durable
    subscription/cursor/delivery-ids/dedup stay in
    REASONBRAID, the reconnect reauthorizes +
    recreates + reconciles + resumes from the own
    cursor + surfaces the possible-gap — never
    stronger than the upstream proves); ADR-024 is
    reserved (no record). THE PARKING-LOT ENTRY (the
    director's 2026-09-08 brainstorm) is this lane's
    trigger: the READ half first (the inspection verbs
    as the read-only tools over the cross-store
    corpus), the WRITE half as the qualified
    capability profile (the grant-scoped principal +
    the per-principal quota — the `.1.3.2` named
    deferral re-opens). Children: `.3.1` the census +
    ADR-024 → `.3.2` the SDK census + the pin → `.3.3`
    the read-half tools + the resources → `.3.4` the
    listen-stream durability → `.3.5` the write-half
    profile.
  Children: `.3.1`–`.3.5`

  - ID: `PHASE-8.3.1`
    Status: `done`
    Goal: the census + ADR-024 — the MCP surface +
      the version-profile contract: the tool
      vocabulary + the same-handlers mapping, the
      resource vocabulary, the listen-stream
      durability contract (the ephemeral transport
      state + the ReasonBraid-owned durable state),
      the version-pin + the conformance-fixture
      stance.
    ADR: 024
    Roadmap: §9.6
    Done (`2026-09-08`): ADR-024 accepted
      (evidence-gated) — `docs/adr/024-mcp-interoperability-surface.md`
      (top-level `answers:`): the tools ARE the same
      command handlers (the MCP surface is the HTTP
      verbs' re-expression — never a new authority
      path; a tool no handler backs is NOT exposed);
      the READ/WRITE split (the read tools first —
      the inspection verbs over the cross-store
      corpus; the write tools as the QUALIFIED
      profile — the enrolled principal, the per-verb
      local grants, the per-principal quota (the
      `.1.3.2` machinery's `principal` scope
      re-opens), the audit; the remote metadata never
      grants authority; the tokens never enter the
      thread content); the listen stream is the
      EPHEMERAL transport state (the durable
      subscription/cursor/delivery-ids/dedup stay in
      REASONBRAID; the reconnect reauthorizes →
      recreates → reconciles → resumes from the OWN
      cursor → surfaces the possible-gap — never
      stronger than the upstream proves); the version
      profile (the official SDK behind
      `reasonbraid-mcp`, the tested release pin, the
      independent conformance fixtures — the A2A
      lane's discipline). No code changed. Frontier →
      `.3.2`.

  - ID: `PHASE-8.3.2`
    Status: `done`
    Goal: the SDK census + the pin — the official
      Rust MCP SDK's exact versions + the supply-chain
      gate, the protocol profile selection (the MCP
      2026-07-28 baseline), the `Cargo.lock` pin + the
      tested release record.
    Roadmap: §9.6
    Done (`2026-09-08`): the census (the crates.io
      checks) + the PIN DECISION recorded: the OFFICIAL
      `mcp-server`/`mcp-client` crates are the STALE
      0.1.0 (2025-02-27 — the abandoned early SDK); the
      official SDK's ACTIVE home is **`rmcp` 3.2.0**
      (2026-08-31, Apache-2.0, Rust 1.88, the
      modelcontextprotocol/rust-sdk repository — the
      full feature map: the
      transport-streamable-http-server/client, the
      auth family, and the `reqwest-tls-no-provider`
      feature for the workspace's provider rule). The
      pin: `rmcp = "=3.2.0"`, `default-features =
      false`, the explicit feature list lands WITH the
      `.3.3` read-half code (the macros + the server +
      the transport the slice needs; the
      `reqwest-tls-no-provider` where the reqwest
      transport enters — the workspace single-provider
      rule, the fourth occurrence of the trap). The
      protocol profile: the MCP 2026-07-28 baseline +
      the Streamable HTTP transport. No code changed.
      Frontier → `.3.3`.
    Acceptance:
    - [x] **ROOT CAUSE (WHY + WHERE)** — the §9.6
      baseline demanded the official SDK + the exact
      pin; the census fixed both AND caught the
      official crates' staleness (the 0.1.0 trap —
      the "official" name no longer means the active
      line). Evidence: the crates.io API checks (the
      version/feature/license records).
    - [x] **ADDRESSED** — the rmcp 3.2.0 pin + the
      no-default-features rule + the protocol profile.
      Evidence: the recorded versions + the
      feature-map analysis.
    - [x] **NO REGRESSION** — docs-only: `make gate` →
      13/13; no dependency graph changed (the pin
      lands with the `.3.3` read-half).

  - ID: `PHASE-8.3.3`
    Status: `done`
    Goal: the read-half tools + the resources — the
      inspection verbs as the read-only MCP tools over
      the cross-store corpus (the `get_thread`/
      `list_inbox`/`get_policy_bundle` family + the
      timelines/evidence/policy-set resources), the
      same command handlers as HTTP, the independent
      conformance fixtures.
    Roadmap: §9.6
    Done (`2026-09-08`): the read-half ships — the new
      `crates/reasonbraid-mcp` (the §9.6 name!) over
      the **rmcp 3.2.0** exact pin (`default-features
      = false` + the explicit `macros`/`server`/
      `transport-async-rw` — the `.3.2` decision; the
      MCP 2026-07-28 baseline is the SDK's native
      target). The three READ tools (`get_thread`,
      `list_inbox`, `get_policy_bundle`) ride the
      SAME queries + the SAME authorization as the
      HTTP handlers (the principal rides the tool's
      argument — the dev-profile trust shape; the
      thread read runs the per-reader classification;
      the inbox + the policy reads run the same
      queries). The write tools stay OFF (the
      qualified profile — `.3.5`). The conformance
      fixtures: the offline tests pin the tool
      ROUTER (exactly the three read tools — the
      write names refuse) + the input SCHEMAS (the
      principal + the target fields — the recorded
      goldens) + the principal wire-space bound. The
      supply-chain gate caught the darling split (the
      rmcp-macros' darling 0.24 vs the derive_builder
      family's 0.20 — the reviewed skip rows, the
      documented proc-macro-family coexistence). The
      Streamable-HTTP transport + the live roundtrip
      ride `.3.4` (the listen-stream durability).
    Acceptance:
    - [x] **ROOT CAUSE (WHY + WHERE)** — the ADR-024
      read-half (the inspection verbs as the read
      tools) had no machinery; the rmcp-pinned tools
      land over the same queries/authorization.
      Evidence: `cargo test -p reasonbraid-mcp` →
      `test result: ok. 3 passed; 0 failed`.
    - [x] **ADDRESSED** — the crate + the three tools
      + the conformance fixtures. Evidence:
      `cargo test -p reasonbraid-mcp` → 3 passed;
      `make deny` → advisories/bans/licenses/sources
      ok (the darling split reviewed + skipped).
    - [x] **NO REGRESSION** — `cargo test --all` →
      rc=0; clippy/fmt clean; `make deny` green;
      `make gate` → 13/13; `make book` builds.
    - [x] **LESSON PROMOTED** — none new: the
      duplicate-split review is the deny doctrine's
      known path.

  - ID: `PHASE-8.3.4`
    Status: `done`
    Goal: the listen-stream durability — the
      ReasonBraid-owned subscription + the cursor +
      the delivery ids + the dedup state; the
      reconnect ritual (the reauthorize → the recreate
      → the reconcile → the resume from the own
      cursor → the possible-gap surface), never
      stronger than the upstream proves.
    Roadmap: §9.6
    Done:
    - migration `0050_mcp_listen_state.sql` — the
      durable state table (the tenant + the
      subscription + the last cursor + the last
      delivery id + the 64-id dedup window + the
      updated stamp).
    - `crates/reasonbraid-server/src/mcp_listen.rs`
      — the state machine: `record_delivery_in_tx`
      (the FOR UPDATE dedup check → the replay SKIP
      with the cursor unchanged; the first delivery
      registers the state; the accepted delivery
      advances the cursor + trims the window — the
      caller's transaction commits the state WITH
      the delivery's effects), `listen_state` (the
      reconnect's input), the pure `resume_plan`
      (the resume is ALWAYS the OWN cursor; the
      possible-gap flag names the no-replay
      condition — never stronger than the upstream
      proves) + its unit test.
    - The lib seam `mcp_listen_internal` carries the
      four names (the `.3.5` transport consumes the
      same path); the re-export also resolves the
      guard's build warnings (the pub-in-private-mod
      items used only by tests → dead-code) —
      `cargo clippy -p reasonbraid-server
      --all-targets` → 0 warnings.
    - The live suite `tests/mcp_listen.rs` (the
      guard's 28th): the first delivery registers →
      the duplicate id is the replay SKIP → the next
      delivery advances the cursor → the state reads
      back; the FK-purge ripple handled (the new
      table in the purge list before `tenants`).
    Acceptance:
    - [x] **ROOT CAUSE (WHY + WHERE)** — the ADR-024
      listen-stream contract had no durable
      machinery: the transport state was ephemeral,
      so a reconnect lost the cursor + the dedup (the
      replay re-delivered; the resume could only
      guess). Evidence: `cargo test -p
      reasonbraid-server --lib` → `test result: ok.
      66 passed; 0 failed`; the guard's mcp_listen
      suite → `test result: ok. 1 passed; 0 failed`.
    - [x] **ADDRESSED** — the durable table + the
      in-transaction machine + the pure resume plan
      + the live suite. Evidence: `bash
      scripts/run_pg_tests.sh` → rc=0, `grep -c
      "test result: ok."` → 28 suites; the demo →
      `ALL acceptance checks passed`.
    - [x] **NO REGRESSION** — `cargo test --all` →
      rc=0; clippy/fmt clean (the two build warnings
      fixed via the seam re-export); `make deny`
      green; `make gate` → 13/13; `make book`
      builds.
    - [x] **LESSON PROMOTED** — none new: the
      dead-code-on-the-unwired-plan lesson is the
      known re-export path. Recorded here, not
      promoted: the demo's fixed 20s first-boot wait
      flaked ONCE under the peak machine load (the
      failed run's own server.log shows both listens
      + every downstream beat; the re-run passed) —
      the pre-existing load sensitivity, not a
      regression.

  - ID: `PHASE-8.3.5`
    Status: `done`
    Goal: the write-half profile — the write tools
      (`respond`/`join_call`/`propose_policy_change`)
      as the QUALIFIED capability profile: the
      grant-scoped principal + the per-verb grants +
      the per-principal quota binding (the `.1.3.2`
      named deferral re-opens) + the audit — never
      the ambient authority.
    Roadmap: §9.6
    Done (`2026-09-08`): the census at the seams —
      decomposed. Measured: the three write handlers
      SHIP — `respond` rides `thread.contribute` over
      the thread-command pipeline (the idempotency →
      the `thread_contribute` authz → the domain →
      the audit), `join_call` rides the call-respond
      verb (the enrolled-ROLE-only gate + the
      eligibility re-resolution),
      `propose_policy_change` rides the
      policy-proposal verb (the enrolled-principal
      gate + `register_proposal`); the per-verb
      LOCAL grants ship (the GrantAction vocabulary
      + the enrolled-presence gates); the audit rides
      the same pipelines. THE GAPS: (1) no MCP write
      seam — the qualified profile's re-expression
      (the enrolled principal → the per-verb local
      grant → the per-principal quota → the SAME
      domain handler, never the ambient authority) is
      the greenfield; (2) the per-principal quota is
      UNBOUND — the machinery ships scope-generic
      (`SCOPE_PRINCIPAL` in the vocabulary,
      `check_in_tx` fail-closed) but no row creator +
      no caller: the `.1.3.2` named deferral's
      trigger FIRES; (3) the write tools are OFF —
      the router refuses the three write names (the
      `.3.3` fixtures pin exactly three tools).
      Children: `.3.5.1` the qualified gate + the
      quota binding → `.3.5.2` the three write tools
      → `.3.5.3` the fixtures + the live
      demonstration.
  Children: `.3.5.1`–`.3.5.3`

  - ID: `PHASE-8.3.5.1`
    Status: `done`
    Goal: the qualified write gate + the quota
      binding (the `.1.3.2` re-open): the server-side
      seam (the `mcp_write` module — the enrolled
      principal → the per-verb LOCAL grant → the
      per-principal quota in the caller's transaction
      → the SAME domain handler), the principal-scope
      row creator at the principal-creation paths +
      migration 0051's backfill (the 0047 pattern),
      the live suite (the guard's 29th: the granted
      write lands + the audit + the quota use; the
      ungranted → the typed refusal; the unconfigured
      → the fail-closed).
    Roadmap: §9.6
    Done:
    - `crates/reasonbraid-server/src/mcp_write.rs`
      — the qualified gate (`gate`: the enrollment
      binding via the SAME `reader_tenant` the HTTP
      handlers run → the per-principal quota via the
      SAME `check_in_tx` fail-closed machinery — the
      exhaustion denial COMMITS its row; the quota
      counts the ADMITTED CALLS, never the domain
      effects) + the three same-handler delegates
      (`respond` over the thread-command pipeline —
      the idempotency → the `thread_contribute` grant
      → the domain → the audit, with the deterministic
      replay key + the tenant injection; `join_call`
      over the extracted `respond_to_call_core`;
      `propose_policy_change` over
      `register_proposal`). The per-verb LOCAL grants
      + the audit ride the handlers — never a new
      authority path.
    - The `.1.3.2` re-open: migration 0051's
      principal-scope backfill (the 0047 pattern) +
      the row creator at the enroll + the
      card-import paths (the identity row implies
      its quota row — the fail-closed check never
      meets a new principal unbound).
    - The api.rs seam exposures: `run_thread_command`
      returns `(StatusCode, Value)` (the three HTTP
      call sites wrap); `respond_to_call_core`
      extracted; `request_hash`/`CommandTarget`/
      `reader_tenant` pub(crate). The lib seam
      `mcp_write_internal` carries the gate + the
      three delegates for the `.3.5.2` tools.
    - The live suite `tests/mcp_write.rs` (the
      guard's 29th): the four measured legs; the
      FK-purge ripple handled (`mcp_listen_state`
      added to the purge list — the `.3.4` table).
    Acceptance:
    - [x] **ROOT CAUSE (WHY + WHERE)** — the ADR-024
      write-half had no qualified seam: the write
      tools were OFF (the router refused the names)
      and the per-principal quota was UNBOUND (the
      `.1.3.2` deferral — the machinery shipped
      scope-generic, no row creator + no caller).
      Evidence: the guard's mcp_write suite →
      `test result: ok. 4 passed; 0 failed`; `cargo
      test --all` → rc=0.
    - [x] **ADDRESSED** — the gate + the three
      same-handler delegates + the 0051 backfill +
      the row creators. Evidence: `bash
      scripts/run_pg_tests.sh` → rc=0, `grep -c
      "test result: ok."` → 29 suites; the demo →
      `ALL acceptance checks passed`.
    - [x] **NO REGRESSION** — `cargo test --all` →
      rc=0; clippy/fmt clean; `make deny` green;
      `make gate` → 13/13; `make book` builds.
    - [x] **LESSON PROMOTED** — none new: the
      FK-purge ripple (the new suite needed the
      `.3.4` table in its purge list) is the known
      recurring doctrine; the audit-record
      actor-handle (not the subject id) and the
      enroll `actions` override (the explicit list
      REPLACES the default grant set) are recorded
      here, not promoted.

  - ID: `PHASE-8.3.5.2`
    Status: `done`
    Goal: the three write tools — `respond`/
      `join_call`/`propose_policy_change` in
      `reasonbraid-mcp` over the `.3.5.1` gate: the
      schemas (the principal + the tenant + the
      targets + the payloads; the token fields
      EXCLUDED — the tokens never enter the thread
      content), the tools delegate to the SAME domain
      handlers through the qualified gate.
    Roadmap: §9.6
    Done:
    - The `McpTools` handle (the `ReadTools` rename —
      the handle now carries the write half): the
      three write tools with the typed schemas —
      `respond` (the principal + the tenant + the
      thread + the minimal `ContributePayload`
      WITHOUT the tenant — the seam injects it),
      `join_call` (the kind + the optional decline
      reason), `propose_policy_change` (the flat
      ProposalInput fields). NO schema names a token
      field — the tokens never enter the thread
      content; the remote metadata never grants
      authority.
    - The handlers call the `.3.5.1` seam
      (`mcp_write_internal`) + surface the gate's
      typed refusals as the tool errors (the family +
      the message); the per-verb LOCAL grants + the
      audit ride the handlers.
    - The conformance fixtures: the six-tool router
      (exactly the three reads + the three writes —
      the `.3.3` write-names-refuse fixture is
      superseded), the write-schema goldens, the
      token-exclusion check. The advanced
      contribution fields + the extended response
      vocabulary are the named follow-on (the minimal
      demonstration profile).
    Acceptance:
    - [x] **ROOT CAUSE (WHY + WHERE)** — the ADR-024
      write tools were OFF (the `.3.3` fixtures
      pinned exactly three tools) while the `.3.5.1`
      gate awaited its consumer. Evidence: `cargo
      test -p reasonbraid-mcp` → `test result: ok. 4
      passed; 0 failed`.
    - [x] **ADDRESSED** — the three tools + the
      typed schemas + the seam delegation + the
      fixtures (the six-tool router + the write
      goldens + the token exclusion). Evidence:
      `cargo test -p reasonbraid-mcp` → 4 passed;
      `cargo clippy -p reasonbraid-mcp
      --all-targets` → 0 warnings.
    - [x] **NO REGRESSION** — `cargo test --all` →
      rc=0; clippy/fmt clean; `make deny` green;
      `make gate` → 13/13; `make book` builds. (The
      server crate is untouched since the `.3.5.1`
      guard — the 29-suite baseline stands; the
      crate's offline tests + the sweep carry this
      change.)
    - [x] **LESSON PROMOTED** — none new: the
      schema-payload boundary (the tool's minimal
      typed payload vs the handler's full body) is
      the read-half's known pattern, applied to the
      writes.

  - ID: `PHASE-8.3.5.3`
    Status: `done`
    Goal: the live demonstration — the measured
      write roundtrip through the TOOL path (the
      six-tool router + the write-schema goldens +
      the token exclusion shipped with `.3.5.2`):
      the granted write lands through the tool
      handler → the effect + the audit + the quota
      use; the qualified-gate refusals pinned (the
      ungranted, the unconfigured, the exhausted);
      the same-handler demonstration, never the
      inferred compatibility.
    Roadmap: §9.6
    Done:
    - The live tool-path roundtrip
      (`the_write_tools_roundtrip_the_qualified_gate_live`
      in the crate's tests, DATABASE_URL-gated): the
      granted `respond` through the TOOL HANDLER (the
      `McpTools` method over the rmcp tool shape)
      lands the effect + the quota use; the
      ungranted role's refusal surfaces as the typed
      tool error (`handler:unauthorized`); the
      unconfigured quota surfaces the fail-closed
      (`quota_unconfigured`); the `join_call`
      decline + the `propose_policy_change` ride the
      same handlers through the tools.
    - The guard now runs `cargo test -p
      reasonbraid-mcp` (the live leg rides the
      guard's DATABASE_URL; the offline sweep skips
      it) — the run_pg_tests.sh append.
    Acceptance:
    - [x] **ROOT CAUSE (WHY + WHERE)** — the tools +
      the gate shipped demonstrated only at the seam
      (the `.3.5.1` suite); the tool layer's
      delegation was unmeasured. Evidence: `cargo
      test -p reasonbraid-mcp` → `test result: ok. 5
      passed; 0 failed` (the live leg gated
      offline).
    - [x] **ADDRESSED** — the live roundtrip (the
      tool handler → the qualified gate → the SAME
      handler → the effect + the recorded use; the
      typed refusals through the tool errors).
      Evidence: `bash scripts/run_pg_tests.sh` →
      rc=0, `grep -c "test result: ok."` → 31
      (the 29 suites + the crate's live + the
      doc-tests); the demo → `ALL acceptance checks
      passed`.
    - [x] **NO REGRESSION** — `cargo test --all` →
      rc=0; clippy/fmt clean; `make deny` green;
      `make gate` → 13/13; `make book` builds.
    - [x] **LESSON PROMOTED** — none new: the
      tool-path demonstration is the A2A lane's
      wire-demonstration discipline applied to MCP
      (the compatibility demonstrated, never
      inferred). **The `.3` lane is COMPLETE.**

- ID: `PHASE-8.4`
  Status: `done`
  Goal: adapter/resolver SDK, compatibility matrix, certification suite, signed plugin registry or allowlist
  ADR: 027
  Done (`2026-09-08`): the census at the seams —
    decomposed. Measured — SHIPPED: the adapter
    CONTRACT (`crates/reasonbraid-adapter/src/
    contract.rs` — the `Adapter` trait + the
    RunRequest/AdapterCapabilities/DispatchAck/
    NormalizedUsage/AttemptStream types), the three
    dev adapters (the fake + the codex + the
    claude), the conformance harness + the
    failure-fixture corpus (the §19.4
    six-invariant suite), the qualification
    checklist (the book's six-box gate), the
    resolver REGISTRY (`resolvers.rs` — the
    server-internal §12.2 filter-then-rank, NO
    public third-party trait), the ADR-027 signing
    machinery (the `rb-release-manifest` tool + the
    five-rung ladder vocabulary). GREENFIELD: the
    VERSIONED SDK surface (the adapter contract is
    crate-internal; the resolver side has no public
    contract), the compatibility MATRIX (no
    document maps the SDK version × the profiles ×
    the platforms), the third-party CERTIFICATION
    gate (the harness qualifies the DEV adapters
    only), the ADR-027 ladder's LOAD-side
    verification (the allowlist rungs exist as the
    vocabulary + the dev-by-construction
    satisfaction; the downloaded-adapter check
    machinery does not).
  Children: `.4.1`–`.4.4`

  - ID: `PHASE-8.4.1`
    Status: `done`
    Goal: the SDK contract — the versioned
      adapter/resolver surface: the `Adapter`
      contract promoted as the VERSIONED SDK (the
      pinned interface + the version token), the
      resolver SURFACE extracted from the internal
      registry (the advertise type as the
      third-party shape — the acquisition trait
      rides the `.4.4` load side), the
      compatibility-matrix SCHEMA (the SDK version
      × the protocol profile × the platform × the
      qualification status).
    ADR: 027
    Done:
    - `SDK_VERSION = "1"` + the
      `Adapter::sdk_version()` provided method (the
      default keeps the three adapters
      source-compatible; the harness refuses a
      mismatch instead of guessing the semantics) —
      the token is the matrix's first axis; the
      bump invalidates the qualifications.
    - The resolver surface moved to the SDK home:
      `reasonbraid-adapter::resolver` (the
      `ResolverAdvertise` type + the ADR-018
      vocabulary consts + the `isolation_error`
      validation) — the server's registry re-imports
      the single shape (the dep promoted from
      dev-dependencies). The acquisition-execution
      trait is the named `.4.4` follow-on.
    - The matrix schema accepted:
      `docs/decisions/2026-09-08_sdk-compatibility-matrix-schema.md`
      (top-level `answers:`): the six columns, the
      measured-only fill rules (the explicit
      `untested`, never blank), the fixture corpus
      as the replay oracle — the `.4.2` fill rides
      it.
    Acceptance:
    - [x] **ROOT CAUSE (WHY + WHERE)** — the `.4`
      census: the adapter contract shipped
      unversioned (no token for the matrix to hang
      on) and the resolver shape lived
      server-internal (no third-party dependency
      home). Evidence: `cargo test -p
      reasonbraid-adapter --lib` → `test result: ok.
      11 passed; 0 failed`.
    - [x] **ADDRESSED** — the version token + the
      provided method + the moved resolver surface
      + the schema record. Evidence: `cargo test -p
      reasonbraid-adapter --lib` → 11 passed;
      `cargo build -p reasonbraid-server` → rc=0.
    - [x] **NO REGRESSION** — `cargo test --all` →
      rc=0; clippy/fmt clean; `make deny` green;
      `make gate` → 13/13; `make book` builds;
      the guard → 29 suites + the demo (the move is
      type-only; the resolver suites re-derive).
    - [x] **LESSON PROMOTED** — none new: the
      dev-deps-vs-deps linkage lesson (the lib
      cannot see a dev-only dependency — the
      adapter dep moved to the main table) is the
      known cargo rule, recorded here.

  - ID: `PHASE-8.4.2`
    Status: `done`
    Goal: the compatibility matrix — the measured
      rows (the three dev adapters + the built-in
      resolvers × the protocol profiles × the
      toolchains), the fixture-backed
      re-derivation (the `.6.2` corpus as the
      replay oracle), the unsupported cells named.
    ADR: 027
    Done:
    - `docs/compatibility-matrix.md` — the filled
      matrix over the `.4.1` schema: the 12 rows
      (the three dev adapters × the conformance
      suite, the real-provider live runs NAMED
      untested, the corpus row, the R0–R2 packs'
      live-roundtrip rows, the gated R3/R5/RX
      named untested).
    - `scripts/check_compatibility_matrix.sh` — the
      mechanical re-derivation: the matrix's
      sdk_version column carries the contract's
      CURRENT token; the cited evidence artifacts
      exist (the conformance suite's three test
      names + the corpus manifest); wired into the
      doctrine gate's project slot
      (`check_doctrines.project.sh`) — the matrix
      is evidence-bound, never prose-bound.
    Acceptance:
    - [x] **ROOT CAUSE (WHY + WHERE)** — the `.4.1`
      schema fixed the matrix's shape but nothing
      filled it; the compatibility claims had no
      evidence-bound artifact. Evidence: `bash
      scripts/check_compatibility_matrix.sh` →
      `compatibility-matrix: OK`; rc=0.
    - [x] **ADDRESSED** — the 12 measured rows +
      the named untested cells + the checker + the
      gate wiring. Evidence: `make gate` → `===
      all doctrines green ===` (the new check runs
      inside).
    - [x] **NO REGRESSION** — `cargo test --all` →
      rc=0 (the doc/script-only leaf — the Rust
      tree is untouched since the `.4.1` guard);
      `make deny` green; `make book` builds.
    - [x] **LESSON PROMOTED** — none new: the
      evidence-bound-artifact pattern (the corpus
      oracle's permanence rules, applied to the
      matrix) is the `.6.2` doctrine's known path.

  - ID: `PHASE-8.4.3`
    Status: `done`
    Goal: the certification suite — the
      third-party certification run: the harness's
      six §19.4 invariants + the book's six-box
      gate applied to a THIRD-PARTY adapter, the
      evidence bundle (the manifest-signed
      qualification record), the fail-closed
      refusal vocabulary.
    ADR: 027
    Done:
    - `crates/reasonbraid-adapter/src/certification.rs`
      — the REPORT-producing certification: the
      six §19.4 invariants as the typed
      pass/refusal results, the fail-closed
      verdict (ANY refusal refuses the whole
      certification — never a partial trust; the
      first refusal is named), the six-box gate
      (the conformance box mirrors the verdict —
      the caller cannot self-attest it; the rest
      ride the submitted evidence), the
      digest-pinned record (`sha256:<hex>` over
      the canonical JSON — the self-digest
      re-derives + the SDK token checks).
    - The conformance harness re-expresses through
      the certification (the tests assert the
      verdict + the digest + the token); the
      THIRD-PARTY demonstration
      (`tests/third_party_certification.rs`): the
      honest vendor adapter certifies (the record
      round-trips); the lying vendor adapter gets
      the typed `usage_accounting_honesty`
      refusal.
    - The release tool's `certify` verb
      (sign/verify): the record's self-digest
      re-derives BEFORE any signature (a lying
      record refuses), the canonical bytes are the
      signed + stored form, the verify re-checks
      the signature + the digest + the SDK token.
      (The adapter crate's tokio `time` feature
      gap surfaced by the tool's build — added.)
    Acceptance:
    - [x] **ROOT CAUSE (WHY + WHERE)** — the
      conformance harness was panic-based (the
      dev-three qualification only); a
      third-party adapter had no report-producing
      certification surface. Evidence: `cargo test
      -p reasonbraid-adapter --test
      third_party_certification` → `test result:
      ok. 2 passed; 0 failed`.
    - [x] **ADDRESSED** — the certification
      module + the harness re-expression + the
      third-party demonstration + the tool's
      certify verb. Evidence: `cargo test -p
      reasonbraid-adapter --lib --test
      adapter_conformance --test
      third_party_certification` → 11 + 3 + 2
      passed; `cargo build -p
      reasonbraid-release-tool` → rc=0.
    - [x] **NO REGRESSION** — `cargo test --all` →
      rc=0; clippy/fmt clean; `make deny` green;
      `make gate` → 13/13; `make book` builds.
    - [x] **LESSON PROMOTED** — none new: the
      tokio feature-gap lesson (the standalone
      consumer exposes the missing `time` feature
      the unified build hid) is the known feature
      unification rule, recorded here.

  - ID: `PHASE-8.4.4`
    Status: `done`
    Goal: the signed plugin registry/allowlist —
      the ADR-027 ladder's LOAD side: the allowlist
      ledger rows + the five-rung verification at
      the adapter-load path (the allowlist → the
      digest → the signature → the API
      compatibility → the capability manifest), the
      typed per-rung refusals, the registry
      surface.
    ADR: 027
    Done:
    - `crates/reasonbraid-adapter/src/allowlist.rs`
      — the five-rung ladder (`verify_ladder`):
      the ordered, fail-closed verification (the
      allowlist membership → the record's
      self-digest → the release identity's Ed25519
      over the canonical bytes (the ring provider)
      → the SDK token → the capability ceilings) +
      the typed `RungRefusal` (each refusal names
      its rung) + the unit tests (the REAL signed
      pass + the per-rung refusals).
    - Migration `0052_adapter_allowlist.sql` — the
      rung-1 ledger (the adapter_id + the added_by
      + the recorded reason + the stamp) with the
      dev three seeded BY CONSTRUCTION.
    - The registry surface — the tenant_admin-gated
      verbs: `GET /v1/admin/adapters` (the list),
      `POST /v1/admin/adapters` (the allow with the
      recorded reason, the idempotent no-op),
      `POST /v1/admin/adapters/{id}/revoke` (the
      next ladder run refuses at rung 1) + the live
      suite (the guard's 30th: the seeded list, the
      allow/re-allow/revoke, the non-admin 403).
    - The load-path WIRING is the named follow-on:
      no download mechanism exists (the `.2.3`
      distribution-channel deferral) — the ladder +
      the ledger + the verbs ship measured; the
      first downloaded adapter rides them.
    Acceptance:
    - [x] **ROOT CAUSE (WHY + WHERE)** — the
      ADR-027 ladder existed as the vocabulary; the
      verification machinery + the durable ledger +
      the operator's surface were the greenfield
      (the `.4` census's fourth gap). Evidence:
      `cargo test -p reasonbraid-adapter --lib` →
      `test result: ok. 13 passed; 0 failed`.
    - [x] **ADDRESSED** — the ladder + the typed
      refusals + the ledger + the verbs. Evidence:
      `cargo test -p reasonbraid-server --test
      allowlist` → `test result: ok. 2 passed; 0
      failed` (the live legs).
    - [x] **NO REGRESSION** — `cargo test --all` →
      rc=0; clippy/fmt clean; `make deny` green;
      `make gate` → 13/13; `make book` builds;
      the guard → the 30 suites + the demo.
    - [x] **LESSON PROMOTED** — none new: the
      per-rung typed-refusal pattern is the
      ADR-027 ladder's own vocabulary, made
      mechanical. **The `.4` lane is COMPLETE.**

- ID: `PHASE-8.5`
  Status: `done`
  Goal: regional routing, store-and-forward for intermittent sites, export/import, documented exit path
  Done (`2026-09-08`): the census at the seams —
    decomposed. Measured — SHIPPED: the node-level
    store-and-forward substrate (the outbox worker
    + the durable per-node inbox with the cursor
    resume + the lease/presence states + the
    replay — the demo's crash/reconnect scenario),
    the export seeds (the backup/restore scripts +
    the guard's restore exercise; the portable
    agent cards' digest-pinned export + the
    four-rung import ladder), the exit-path seeds
    (the §25.1 kill/pivot discipline + the
    subtraction records + the unsupported matrix +
    the rollback/failover runbooks). GREENFIELD:
    the REGION machinery (the ADR-034 stance — the
    regions are DECLARED; no region/export
    machinery exists — the named deferral), the
    SITE-level store-and-forward (the intermittent
    cross-site delivery over the shipped substrate),
    the tenant/site DATA export/import (the
    backup/restore is the operator's tooling, not
    the tenant surface), and the documented EXIT
    PATH (the leave-ReasonBraid story as one
    document). The Phase-5 routing policy is the
    workflow-profile routing — a different domain,
    not this lane's regional routing.
  Children: `.5.1`–`.5.4`

  - ID: `PHASE-8.5.1`
    Status: `done`
    Goal: the census + ADR-035 — the site/region
      contract: the DECLARED region vocabulary (the
      ADR-034 seeds), the site identity + the
      region scope, the store-and-forward contract
      (the intermittent-site delivery over the
      shipped outbox/inbox), the export/import
      vocabulary (the tenant-data shape), the
      exit-path shape.
    ADR: 035
    Done:
    - ADR-035 accepted (`docs/adr/
      035-site-region-contract.md`, top-level
      `answers:`): the regions are DECLARED (the
      declaration is the fail-closed seam — an
      undeclared region refuses at the routing
      boundary); the store-and-forward rides the
      SHIPPED substrate (the site-level pairing
      over the outbox + the inbox — the buffered
      rows persist for a disconnected site, the
      reconnect flushes in order, the possible-gap
      surfaces); the export is the exit path's
      machinery (the digest-pinned signed bundle +
      the ordered import ladder — the portable
      cards' four-rung pattern applied to the
      tenant data); the exit path is the documented
      runbook, never an emergency invention.
    Acceptance:
    - [x] **ROOT CAUSE (WHY + WHERE)** — the `.5`
      census: the region/export vocabulary was the
      ADR-034 named deferral (no contract fixed the
      shape). Evidence: `make gate` → `=== all
      doctrines green ===`; `git grep -c
      "regions are DECLARED"` → rc=0.
    - [x] **ADDRESSED** — the ADR-035 record + the
      index row. Evidence: `make gate` → `=== all
      doctrines green ===`.
    - [x] **NO REGRESSION** — no code changed (the
      decisions-only leaf); `cargo test --all` →
      rc=0 (the `.4.4` baseline).
    - [x] **LESSON PROMOTED** — the ADR IS the
      promotion (the top-level `answers:` form).

  - ID: `PHASE-8.5.2`
    Status: `done`
    Goal: the regional routing — the routing rules
      over the declared regions (the region-scoped
      delivery + the region-scoped visibility), the
      measured refusal vocabulary (the undeclared
      region, the cross-region policy).
    ADR: 035
    Done:
    - Migration `0053_site_regions.sql` — the
      `site_regions` declarations + the
      `region_pairs` allowlist (the cross-region
      delivery rides the explicit pair row — the
      ADR-027 pattern applied to the region
      vocabulary); the dev profile declares
      `dev-local`.
    - `crates/reasonbraid-server/src/regions.rs` —
      the routing machinery: `route` (the
      same-region routes; the undeclared region
      refuses with its OWN name; the unpaired
      cross-region refuses until the pair lands) +
      the `RegionRefusal` vocabulary + the
      declare/pair/unpair helpers. The `.5.3`
      store-and-forward consumes `route` through
      the `regions_internal` seam.
    - The registry surface — the tenant_admin-gated
      verbs: `GET /v1/admin/regions` (the declared
      + the pairs), `POST /v1/admin/regions` (the
      declare), `POST /v1/admin/regions/{from}/
      pair/{to}` + `/unpair/{to}` (the typed 400 on
      the undeclared pair) + the live suite (the
      guard's 31st: the routed same-region, the
      per-refusal names, the pair/unpair cycle, the
      non-admin 403).
    Acceptance:
    - [x] **ROOT CAUSE (WHY + WHERE)** — the ADR-035
      vocabulary (the declared regions + the pair
      allowlist) had no machinery. Evidence: `cargo
      test -p reasonbraid-server --test regions` →
      `test result: ok. 2 passed; 0 failed`.
    - [x] **ADDRESSED** — the migration + the
      routing module + the verbs + the suite.
      Evidence: the guard → rc=0, 32 ok-lines (the
      30 suites + the crate lines + the regions
      suite), the demo → `ALL acceptance checks
      passed`.
    - [x] **NO REGRESSION** — `cargo test --all` →
      rc=0; clippy/fmt clean; `make deny` green;
      `make gate` → 13/13; `make book` builds.
    - [x] **LESSON PROMOTED** — none new: the
      pair-allowlist pattern is the ADR-027
      ladder's known shape applied to the region
      vocabulary.

  - ID: `PHASE-8.5.3`
    Status: `proposed`
    Goal: the store-and-forward — the
      intermittent-site delivery over the shipped
      substrate (the outbox + the inbox): the
      site-level buffer + the reconnect flush + the
      measured legs (the disconnected site
      accumulates, the reconnect delivers).
    ADR: 035

  - ID: `PHASE-8.5.4`
    Status: `proposed`
    Goal: the export/import + the documented exit
      path — the tenant-data export (the
      digest-pinned bundle over the backup/restore
      pattern) + the import ladder + the exit-path
      runbook (the leave-ReasonBraid story: the
      export → the verification → the local
      continuation, the §25.1 wiring).
    ADR: 035

- ID: `PHASE-8.6`
  Status: `proposed`
  Goal: protocol extension process and independent implementation exercise; G8 exit
  Gate: G8; subtraction record required

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `SIGNOFF-REPAIR.3.2.3` | `active` | corrective prerequisite; HTTP site-authority verification, then remaining authority repairs |
| 2 | `PHASE-8.5.3` | `proposed` | `.5.2` done — the regional routing ships (the declarations + the pair allowlist + the typed refusals + the 31st suite); the store-and-forward executes next |

## Changelog

- `2026-09-08`: `.5.2` done — the regional routing
  (migration 0053's declarations + the pair
  allowlist, the `regions` module's `route` + the
  typed refusals, the admin-gated declare/pair/
  unpair/list verbs + the 31st suite); frontier →
  `.5.3`.

- `2026-09-08`: `.5.1` done — ADR-035 accepted
  (the site/region contract: the declared regions,
  the store-and-forward over the shipped substrate,
  the export-is-the-exit-path ladder); frontier →
  `.5.2`.

- `2026-09-08`: `.5` done — the regional-routing
  lane's census at the seams (the node-level
  store-and-forward substrate + the export seeds +
  the exit-path seeds ship; the region machinery +
  the site-level store-and-forward + the tenant
  export + the exit-path document are the
  greenfield) → decomposed `.5.1` the census +
  ADR-035 → `.5.2` the regional routing → `.5.3`
  the store-and-forward → `.5.4` the export/import
  + the exit path; frontier → `.5.1`.

- `2026-09-08`: `.4.4` done — the signed
  allowlist (the five-rung ladder with the typed
  per-rung refusals + the 0052 ledger + the
  admin-gated allow/revoke/list verbs + the 30th
  suite) — **the `.4` lane is COMPLETE**;
  frontier → `.5`.

- `2026-09-08`: `.4.3` done — the certification
  suite (the report-producing certification with
  the fail-closed verdict + the six-box gate, the
  third-party demonstration — the honest vendor
  certifies, the lying one gets the typed refusal
  — and the release tool's `certify` sign/verify
  verb); frontier → `.4.4`.

- `2026-09-08`: `.4.2` done — the compatibility
  matrix (the 12 rows over the `.4.1` schema, the
  named untested cells, the gate-wired checker —
  the evidence-bound artifact); frontier →
  `.4.3`.

- `2026-09-08`: `.4.1` done — the SDK contract
  (the `SDK_VERSION` token + the
  `Adapter::sdk_version` method, the resolver
  advertise moved to `reasonbraid-adapter::resolver`
  — the SDK home — and the matrix schema record);
  frontier → `.4.2`.

- `2026-09-08`: `.4` done — the adapter/resolver
  SDK census at the seams (the adapter contract +
  the harness + the fixtures + the resolver
  registry + the ADR-027 signing vocabulary ship;
  the versioned SDK surface, the compatibility
  matrix, the certification gate, and the ladder's
  load side are the greenfield) → decomposed
  `.4.1` the SDK contract → `.4.2` the matrix →
  `.4.3` the certification → `.4.4` the allowlist;
  frontier → `.4.1`.

- `2026-09-08`: `.3.5.3` done — the live
  tool-path roundtrip (the granted write through
  the TOOL handler + the typed refusals + the
  join_call/propose rides; the guard now runs the
  crate's DATABASE_URL-gated live leg) — **the
  `.3` lane (the MCP surface) is COMPLETE**;
  frontier → `.4`.

- `2026-09-08`: `.3.5.2` done — the three MCP
  write tools (the `McpTools` handle — the
  `respond`/`join_call`/`propose_policy_change`
  schemas with the token fields EXCLUDED — over
  the `.3.5.1` seam; the six-tool router fixtures
  supersede the `.3.3` write-names-refuse pin);
  frontier → `.3.5.3`.

- `2026-09-08`: `.3.5.1` done — the qualified
  write gate + the quota binding (the `mcp_write`
  seam — the enrollment binding + the
  per-principal quota + the three same-handler
  delegates; migration 0051's principal backfill +
  the enroll/card-import row creators; the
  `run_thread_command` seam refactor; the 29th
  suite); frontier → `.3.5.2`.

- `2026-09-08`: `.3.5` done — the MCP write-half
  census at the seams (the three write handlers
  ship — `respond` over the thread-command pipeline,
  `join_call` over the call-respond verb,
  `propose_policy_change` over the policy-proposal
  verb; the gaps: the qualified gate seam, the
  unbound `principal` quota — the `.1.3.2` trigger
  FIRES — and the OFF write tools) → decomposed
  `.3.5.1` the qualified gate + the quota binding →
  `.3.5.2` the three write tools → `.3.5.3` the
  fixtures + the live demonstration; frontier →
  `.3.5.1`.

- `2026-09-08`: `.3.4` done — the MCP
  listen-stream durability (migration 0050's
  durable state + the in-transaction dedup/cursor
  machine + the pure resume plan + the live suite
  (the guard's 28th) + the seam re-export fix);
  frontier → `.3.5`.

- `2026-09-08`: `.3.3` done — the MCP read-half
  (the `reasonbraid-mcp` crate over the rmcp 3.2.0
  pin + the three read tools over the same
  queries/authorization + the conformance fixtures +
  the darling-split review); the transport + the
  live roundtrip ride `.3.4`; frontier → `.3.4`.
- `2026-09-08`: `.3.2` done — the MCP SDK census +
  the pin decision (the official `mcp-server`/
  `mcp-client` are the stale 0.1.0; **`rmcp` 3.2.0**
  is the official SDK's active line — the exact pin +
  the no-default-features rule + the Streamable HTTP
  profile); the pin lands with the `.3.3` read-half;
  no code; frontier → `.3.3`.
- `2026-09-08`: `.3.1` done — ADR-024 accepted
  (evidence-gated): the tools are the same handlers
  + the read/write split + the listen-stream
  durability contract + the version-profile
  discipline; no code; frontier → `.3.2`.
- `2026-09-08`: `.3` done — the MCP census at the
  seams (the §9.6 tool/resource vocabulary + the
  listen-stream contract; ADR-024 reserved; the
  parking-lot trigger fires) → decomposed `.3.1` (the
  census + ADR-024) → `.3.2` (the SDK pin) → `.3.3`
  (the read-half) → `.3.4` (the listen-stream
  durability) → `.3.5` (the write-half profile);
  frontier → `.3.1`.
- `2026-09-08`: `.2.4` done — the A2A wire-level
  demonstration (the JSON-RPC 2.0 roundtrip over the
  real 0.3.0 shapes + the typed refusal + the
  qualification record: the wire compatibility
  demonstrated, the transport profile stays the named
  follow-on, the broad claims stay gated);
  **the `.2` lane is COMPLETE**; frontier → `.3`.
- `2026-09-08`: `.2.3` done — the A2A facade core
  (the `reasonbraid-a2a` crate over the exact 0.3.0
  pin + the semantic-loss record + the mapper +
  the external-id preservation + the 3-test suite);
  the local-command wiring + the transport ride the
  `.2.4` demonstration; frontier → `.2.4`.
- `2026-09-08`: `.2.2` done — the A2A dependency
  census + the pin decision (a2a-lf 0.3.0,
  a2a-server-lf 0.4.3, a2a-client-lf 0.2.3,
  Apache-2.0; the JSON-RPC/REST profile; the
  no-provider feature rule — the default features
  vote aws-lc-rs, the workspace rule's third
  occurrence); the pin lands with the `.2.3` facade;
  no code; frontier → `.2.3`.
- `2026-09-08`: `.2.1` done — ADR-025 accepted
  (evidence-gated): the facade boundary + the
  recorded semantic losses + the version-pin/
  demonstrated-compatibility stance; no code;
  frontier → `.2.2`.
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
