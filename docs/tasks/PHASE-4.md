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
    Status: `done`
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
    Done (`2026-09-07`): ADR-011 + ADR-018 accepted
      (evidence-gated): the content-addressing format
      (`docs/adr/011-object-store-content-addressing.md`) pins
      `sha256:<hex>` over the ACQUIRED bytes — the references'
      `expected_digest` and the future snapshots speak one scheme;
      the store + the derivation graph ride the `.6` lane behind
      the first-snapshot-receipt trigger. The isolation vocabulary
      (`docs/adr/018-resolver-sandbox-isolation.md`) pins the
      sandbox-level ladder + the egress class (the claim is the
      maximum) — the `.1.3` registry's advertise shape carries the
      classes; a required-isolation miss is the explicit failure,
      never the silent downgrade; the runtimes ride the packs
      (`.2`–`.4`). No code changed. Frontier → `.1.2`.
    Acceptance: the two ADRs accepted (the format + the isolation
      classes named); no code changes.

  - ID: `PHASE-4.1.2`
    Status: `done`
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
    Done (`2026-09-07`): the typed reference landed — migration
      0023 (`resource_references` with the UNIQUE
      (original_locator, expected_digest)) + `crates/reasonbraid-
      server/src/resources.rs` (the §12.1 `ResourceReference` with
      the deny-unknown-fields boundary + the ADR-011 digest
      validation) + the verbs: `POST /v1/resources` (any enrolled
      principal; the same locator + digest is the REPLAY; the same
      locator with a DIFFERENT digest is the typed
      `locator_digest_conflict` — the immutability is the
      update-refusal + the conflict, not a convention) and `GET
      /v1/resources/{id}` (the inspection). Measured
      (`a_reference_submits_typed_and_the_locator_is_immutable`,
      profiles 13): the fresh submit, the replay, the conflict, the
      unknown-field 422, the malformed-digest 400 (the first run
      caught the digest validator's early-return bug), the
      read-back. Frontier → `.1.3`.
    Acceptance: the typed reference + the submission land, measured
      (the immutability + the unknown-field refusals); no
      regression.

  - ID: `PHASE-4.1.3`
    Status: `done`
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
    Done (`2026-09-07`): the resolver capability registry landed —
      migration 0024 (`resolver_capabilities`: the §12.2 advertise
      with the ADR-018 classes) + `crates/reasonbraid-server/src/
      resolvers.rs` (the typed `ResolverAdvertise` + the isolation
      validation + the `resolve` order: the scheme + the
      sandbox/egress filters FIRST — a resolver declaring LESS
      than the required class is ineligible — then the latency
      rank) + the verbs: `POST /v1/resolvers` (the tenant_admin
      registration — the future packs' install verb) and `POST
      /v1/resources/{id}/resolve` (any enrolled principal). The
      explicit `resource_unresolvable_now` result leaves the
      reference SUBMITTED (still readable — never fabricated).
      Measured (`the_resolver_registry_resolves_and_fails_explicitly`,
      profiles 14): the filter + the rank (the weaker sandbox is
      ineligible; the fast resolver ranks first), the off-ladder
      claim's 400, the unsupported scheme's explicit failure with
      the preserved reference. **`.1` COMPLETE** — frontier → `.2`.
    Acceptance: the registry + the resolution order land; the
      unresolvable-now result preserves the reference, measured; no
      regression.

- ID: `PHASE-4.2`
  Status: `done`
  Goal: pack R0 — safe HTTPS documents/pages (SSRF/DNS/redirect/size/content defenses, snapshot receipt)
  Backlog: 32
  Roadmap: §12.3–12.4
  Children: `.2.1`–`.2.3` (decomposed `2026-09-07` at the census
    seams): `.2.1` the destination classification + the SSRF policy
    (the pure §12.4 IP rules) → `.2.2` the safe HTTPS fetcher (the
    hardened URL parsing, the ceilings, the redirect policy at
    every hop, the TLS verification, the decompression-ratio
    limit) → `.2.3` the snapshot receipt + the pack wiring (the
    ADR-011 receipt + the R0 registry entry).
  Done (`2026-09-07`): the census mapped §12.4 against the shipped
    surface: NOTHING fetches (the reqwest dependency serves the
    wire tests only — no fetcher, no destination classification,
    no receipt machinery: `grep -rn "classify_destination\|fetch"
    crates/reasonbraid-server/src/` → only the SQL `fetch_*`
    calls). The R0 pack is a greenfield with the §12.4 rules as
    its spec. Children at those seams — frontier → `.2.1`.
  - ID: `PHASE-4.2.1`
    Status: `done`
    Goal: the destination classification + the SSRF policy — the
      PURE `classify_destination(ip)` over the §12.4 rules (the
      loopback, the link-local, the private ranges, the multicast,
      the reserved, the cloud-metadata special cases — the
      dev-profile egress is the `listed` class) + the policy
      (which classes the R0 fetcher may reach) — pure + tested
      (each range has a named class); the proxy configuration
      stays in the threat model (named, not built).
    Backlog: 32 (the SSRF half)
    Done (`2026-09-07`): the SSRF policy landed —
      `crates/reasonbraid-server/src/ssrf.rs`: the PURE
      `classify_destination(ip)` over the §12.4 rules (the
      loopback, the link-local, the private ranges, the multicast,
      the reserved, the cloud-metadata class — its own class
      INSIDE the link-local range; the IPv4-mapped IPv6 form
      re-classifies the embedded IPv4) + the policy (`evaluate`:
      ONLY the `Public` class is reachable; every refusal names
      its class). Four unit tests measure the 18-case refusal
      matrix (each reason names its class), the allowed publics,
      the mapped-form re-classification, and the metadata
      special case. The `.2.2` fetcher enforces this policy at
      every hop. Frontier → `.2.2`.
    Acceptance: the classification is pure + tested (the private/
      loopback/link-local/multicast/reserved refusals); no
      regression.

  - ID: `PHASE-4.2.2`
    Status: `done`
    Goal: the safe HTTPS fetcher — the hardened URL parsing (the
      ambiguous/userinfo/invalid-encoding refusals), the GET/HEAD
      with the byte + time ceilings, the redirect policy at EVERY
      hop (the re-classification + the hop cap), the TLS
      verification (the system roots), the response-type sniffing,
      the decompression-ratio limit, NO ambient credentials. The
      fetcher is the R0 resolver's engine; it reaches ONLY the
      classes the `.2.1` policy allows (the measured refusal of a
      loopback/private target is the SSRF proof).
    Backlog: 32 (the fetcher half)
    Done (`2026-09-07`): the safe HTTPS fetcher landed —
      `crates/reasonbraid-server/src/fetcher.rs`: the hardened URL
      parse (the length cap, the control-character/backslash
      refusals, the userinfo refusal, the alternative-numeric-
      literal refusal, the scheme/port allowlists), the GET/HEAD
      with the byte + time ceilings (the WHOLE acquisition is
      bounded; the body is read bounded), the manual redirect
      policy at EVERY hop (the full re-parse + re-classification +
      the hop cap — the escape dies at the classification, never at
      the socket), the destination policy at TWO layers (the
      pre-flight resolve+classify that names the refusing class
      BEFORE any socket opens, and the classified DNS belt inside
      reqwest's resolver hook — a dial can never touch a refused
      address even when the DNS answer changed between the
      pre-flight and the connect), the TLS verification (rustls,
      the ring provider, the SYSTEM roots), the manual
      content-encoding decode (gzip/deflate/br — reqwest's
      auto-decode would hide the encoded size the ratio brake
      measures), the decompression-ratio limit + the byte ceiling
      over the DECODED bytes, the response-type sniff (the header
      first, then the HTML magic, the JSON-shaped refusal, the
      UTF-8 fallback), NO ambient credentials (one static user
      agent, no cookie state, no proxy environment). Sixteen tests
      measure it all OFFLINE (the injected resolver pins test
      domains to a local origin — no real DNS): the loopback
      literal refuses with the class NAMED before any request
      reaches the origin (the SSRF proof), the mapped-form loopback
      refuses, the private redirect hop refuses after exactly one
      dial, the hop cap names itself, the byte ceiling, the REAL
      gzip bomb trips the ratio brake while a small gzip page
      passes, the JSON body refuses, the HEAD returns the empty
      document, the userinfo/dns-failure refusals, the https-only
      public-only defaults. The first runs caught three real bugs
      (the IPv6 bracket form of `host_str`, reqwest's auto-decode
      hiding `Content-Length` from the ratio brake, the WHATWG
      parser normalizing numeric hosts to IPv4 literals) — all
      fixed. A FOURTH, workspace-wide one surfaced at the offline
      sweep: the new rustls/ring feature joined cert-spike's
      default aws-lc-rs under `cargo test --all`'s documented
      cross-member feature unification into an AMBIGUOUS
      two-provider rustls (the spike's issuance test panicked;
      `b26f529` verified green before this leaf) — fixed by pinning
      the spike's rustls provider explicitly to ring (the
      workspace's crypto family; the Cargo.toml comment records
      why). Frontier → `.2.3`; the clippy evidence debt the sweep
      also measured is ROUTED to `PHASE-4-MAINT-1` below.
    Acceptance: the fetcher refuses the ambiguous URL + the
      private/loopback target + the redirect-chain escapes,
      measured; no regression.

  - ID: `PHASE-4-MAINT-1`
    Status: `proposed`
    Goal: the clippy evidence debt — the recorded
      `cargo clippy --all --all-targets -- -D warnings → clean`
      evidence of the Phase-3 leaves (the matching/recruitment/
      dependence/api modules) and of `.1.2` (resources) does NOT
      reproduce under the pinned clippy 0.1.98: NINE pre-existing
      findings fail the crate-wide run (7 lib: `useless_format`
      api.rs:2011, `type_complexity` dependence.rs:48, the
      `collapsible_if` + the `unnecessary if let` matching.rs:436/
      470, `too_many_arguments` recruitment.rs:108,
      `type_complexity` recruitment.rs:204 + resources.rs:138; 2
      test: the `field_reassign` pairs matching.rs:706/707 +
      878/879). ROUTED by `.2.2` (the ROUTING EVIDENCE section
      below). The fixes are the mechanical lint repairs + the one
      signature refactor (recruitment's 10-arg `open_call` takes a
      params struct); the gate's clippy run is green again at the
      close.
    Defect (tracked `2026-09-07`, discovered during the `.2.2`
      verification): the acceptance checklist is written when the
      leaf executes.

  - ID: `PHASE-4.2.3`
    Status: `proposed`
    Goal: the snapshot receipt + the pack wiring — the acquisition
      receipt (the ADR-011 `sha256:<hex>` over the ACQUIRED bytes,
      the resolved URL chain, the byte count, the content type,
      the acquisition time — the `.6` snapshot lane's input
      shape), the R0 resolver's registry entry (the `https`
      scheme + the egress/sandbox claims), and the resolution
      path's consumption (the `.1.3` resolve returns the R0
      resolver for the https references).
    Backlog: 32 (the receipt half)
    Acceptance: the receipt carries the digest + the chain; the R0
      registry entry resolves the https references, measured; no
      regression.

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
| 1 | `PHASE-4-MAINT-1` | `proposed` | `.2.2` done — its verification measured the pre-existing clippy evidence debt (9 findings fail the recorded `-D warnings` command under the pinned clippy; `b26f529` reproduces them); the mechanical repair + the one signature refactor execute next, then `.2.3` |
| 2 | `PHASE-4.2.3` | `proposed` | `.2.2` done — the safe HTTPS fetcher (the hardened parse, the ceilings, the per-hop redirect policy, the two-layer destination enforcement, the manual decode + the ratio brake); the snapshot receipt + the R0 pack wiring execute now |

## Changelog

- `2026-09-05`: Created from `ROADMAP.md` §20.6, §12, backlog 31–35.
- `2026-09-07`: Opened by the Phase-2/3 closes (the authz/budgets
  legs of the blocker; the object store is this phase's `.6`); `.1`
  decomposed at the census seams (the resource surface is a
  greenfield: the reason name + the typed `EvidenceRef`s exist, the
  contract + the registry + ADR-011/018 do not); children `.1.1`
  (the two ADRs) → `.1.2` (the typed reference) → `.1.3` (the
  registry); frontier → `.1.1`.
- `2026-09-07`: `.1.1` done — ADR-011 + ADR-018 accepted (the
  `sha256:<hex>` format + the isolation-class vocabulary); no code
  changed; frontier → `.1.2`.
- `2026-09-07`: `.1.2` done — the typed `ResourceReference` + the
  submission (migration 0023 + the verbs; the locator's
  immutability is the replay + the typed conflict); the profiles
  suite grew to 13; frontier → `.1.3`.
- `2026-09-07`: `.1.3` done — the resolver capability registry
  (migration 0024 + the register/resolve verbs; the filter-then-
  rank order + the explicit unresolvable-now); the profiles suite
  grew to 14; **`.1` COMPLETE** — frontier → `.2`.
- `2026-09-07`: `.2` decomposed at the census seams — nothing
  fetches (the reqwest dep serves the wire tests only); the R0
  pack is a greenfield with the §12.4 rules as its spec; children
  `.2.1` (the destination classification + the SSRF policy) →
  `.2.2` (the safe HTTPS fetcher) → `.2.3` (the snapshot receipt +
  the pack wiring); frontier → `.2.1`.
- `2026-09-07`: `.2.1` done — the destination classification + the
  SSRF policy (the pure §12.4 rules, the public-only policy, the
  mapped-form re-classification); four unit tests; frontier →
  `.2.2`.
- `2026-09-07`: `.2.2` done — the safe HTTPS fetcher (the hardened
  URL parse, the byte/time ceilings, the manual per-hop redirect
  policy, the two-layer destination enforcement — the pre-flight
  classification + the classified DNS belt, the manual
  content-encoding decode + the ratio brake, the response-type
  sniff, no ambient credentials); sixteen OFFLINE tests including
  the SSRF proof (the loopback literal refuses with the class named
  before any request reaches the origin); the offline sweep caught
  the workspace-wide rustls provider ambiguity (the new ring
  feature + cert-spike's default aws-lc-rs under `cargo test
  --all`'s cross-member unification) — fixed by pinning the spike
  to ring; the sweep ALSO measured the pre-existing clippy evidence
  debt (9 findings) — ROUTED to `PHASE-4-MAINT-1`; frontier →
  `PHASE-4-MAINT-1` → `.2.3`.

## Routing Evidence (PHASE-4.2.2 → PHASE-4-MAINT-1)

Finding: the recorded `cargo clippy --all --all-targets -- -D
warnings → clean` evidence of the Phase-3 leaves (the
matching/recruitment/dependence/api modules) and of `.1.2`
(resources) does NOT reproduce under the pinned clippy 0.1.98 —
NINE pre-existing findings fail the crate-wide run (7 lib:
`useless_format` api.rs:2011, `type_complexity` dependence.rs:48,
`collapsible_if` + `unnecessary if let` matching.rs:436/470,
`too_many_arguments` recruitment.rs:108, `type_complexity`
recruitment.rs:204 + resources.rs:138; 2 test: the
`field_reassign` pairs matching.rs:706/707 + 878/879). Measured:
`cargo clippy --all --all-targets -- -D warnings` → rc=101 at THIS
leaf's HEAD (its own files contribute zero findings), and the SAME
command at `b26f529` (the gate-verified `.2.1` commit) fails
IDENTICALLY (rc=101) — the findings pre-exist this leaf and
reproduce outside the family they are sent to. Routed to
`PHASE-4-MAINT-1` (opened above — the repair leaf; the Phase-3
modules' share rides it, and `docs/tasks/PHASE-3.md` references
the route).

## Acceptance Checklist (PHASE-4.2.2)

The CODE change owned by this leaf:
`crates/reasonbraid-server/src/fetcher.rs` (NEW — the hardened
parse, the two-layer destination enforcement, the manual redirect
policy, the bounded read + the ratio brake + the sniff, the
sixteen tests), `crates/reasonbraid-server/src/lib.rs` (the
module), `crates/reasonbraid-server/Cargo.toml` (reqwest with the
rustls/system-roots + stream features, url, futures-util, flate2,
brotli; tokio gains `time`), and
`crates/reasonbraid-cert-spike/Cargo.toml` (the rustls provider
pin the offline sweep's regression demanded) — `\.rs$` +
`Cargo.toml`.

- [x] **REPRODUCE / ISSUE** — the `.2` census: NOTHING fetches and
  no destination classification exists — the §12.4 acquisition
  rules have no machinery.
- [x] **ROOT CAUSE (WHY + WHERE)** — the R0 fetcher was a
  greenfield — `git grep -c "bytes_stream\|dns_resolver\|no_proxy\|harden_url" b26f529 -- crates/`
  → rc=1 (nothing before this leaf). The fix point is the
  hardened fetcher with the `.2.1` policy enforced at TWO layers
  (the pre-flight that names the class, the classified belt that
  guards the dial) — the measured refusal is the SSRF proof.
- [x] **ADDRESSED (verified)** — measured before→after. Before: the
  grep above. After:
  `cargo test -p reasonbraid-server --lib fetcher` → `test result:
  ok. 16 passed` — the pure refusals (the userinfo, the numeric
  literals, the scheme/port/control-char forms), the OFFLINE wire
  refusals: the loopback literal refuses with the class NAMED
  before any request reaches the origin (the counter stays 0 — the
  SSRF proof), the mapped-form loopback refuses, the private
  redirect hop dies at the re-classification after exactly one
  dial, the hop cap names itself, the byte ceiling, the REAL gzip
  bomb trips the ratio brake while a small gzip page passes, the
  JSON body refuses, the HEAD returns the empty document, the
  https-only public-only defaults. The first runs caught three
  real bugs (the IPv6 bracket form, the auto-decode hiding
  `Content-Length`, the WHATWG numeric-host normalization) — all
  fixed.
- [x] **NO REGRESSION** — `cargo test --all` → rc=0, 51 suites,
  327 tests (the sweep CAUGHT a real workspace-wide regression the
  leaf introduced: the new rustls/ring feature + cert-spike's
  default aws-lc-rs unified under `cargo test --all` into an
  ambiguous two-provider rustls — the spike's issuance test
  panicked at `b26f529` it was green — fixed by pinning the
  spike's rustls to ring, the workspace's crypto family);
  `bash scripts/run_pg_tests.sh` → rc=0, 18 live suites + the demo
  `ALL acceptance checks passed` (`target/pg422_guard.log`);
  `cargo fmt --all -- --check` → rc=0; clippy: this leaf's files
  are clean — the crate-wide `-D warnings` run fails on 9
  PRE-EXISTING findings (identical at `b26f529`, rc=101), ROUTED
  above to `PHASE-4-MAINT-1`; `make gate` → 13/13 at commit.
- [x] **FIX** — `src/fetcher.rs`, `src/lib.rs`,
  `crates/reasonbraid-server/Cargo.toml`,
  `crates/reasonbraid-cert-spike/Cargo.toml`.
- [x] **LOCKSTEP** — CHANGELOG, DEV_NOTES, MEMORY, LIVE_STATUS, this
  tree's logs above, `docs/TASK_TREE.md` frontier, KNOWLEDGE_MAP —
  same commit.

## Acceptance Checklist (PHASE-4.2.1)

The CODE change owned by this leaf:
`crates/reasonbraid-server/src/ssrf.rs` (NEW — the pure
classification + the policy + the four tests) and
`crates/reasonbraid-server/src/lib.rs` (the module) — `\.rs$` in
`.doctrine/code_paths.txt`.

- [x] **REPRODUCE / ISSUE** — the `.2` census: NOTHING fetches and
  no destination classification exists — the §12.4 SSRF rules
  have no machinery.
- [x] **ROOT CAUSE (WHY + WHERE)** — the R0 pack was a greenfield —
  `git grep -c "classify_destination\|ssrf" 7693623 -- crates/`
  → rc=1 (nothing before this leaf). The fix point is the PURE
  classification (the IP ranges the §12.4 rules name — testable
  without a socket) + the public-only policy the `.2.2` fetcher
  enforces at every hop.
- [x] **ADDRESSED (verified)** — measured before→after. Before: the
  grep above. After:
  `cargo test -p reasonbraid-server --lib ssrf` → `test result:
  ok. 4 passed` — the 18-case refusal matrix (every non-public
  class names itself in the reason: the loopback, the private
  ranges, the link-local, the cloud-metadata, the multicast, the
  reserved), the allowed publics, the IPv4-mapped form
  re-classifying the embedded IPv4 (the mapped metadata address
  refuses as `cloud_metadata`), and the metadata address's own
  class inside the link-local range.
- [x] **NO REGRESSION** — `bash scripts/run_pg_tests.sh` → 18 live
  suites + the demo `ALL acceptance checks passed` 34/34
  (`target/pg421_guard.log`); `cargo test --all` → 51 offline
  suites green; `cargo clippy --all --all-targets -- -D warnings` →
  clean; `cargo fmt --all -- --check` → rc=0; `make gate` → 13/13 at
  commit.
- [x] **FIX** — `src/ssrf.rs`, `src/lib.rs`.
- [x] **LOCKSTEP** — CHANGELOG, DEV_NOTES, MEMORY, LIVE_STATUS, this
  tree's logs below, `docs/TASK_TREE.md` frontier, KNOWLEDGE_MAP —
  same commit.

## Acceptance Checklist (PHASE-4.1.3)

The CODE change owned by this leaf:
`migrations/0024_resolver_capabilities.sql` (NEW),
`crates/reasonbraid-server/src/resolvers.rs` (NEW — the typed
advertise + the isolation validation + the filter-then-rank
resolution), `crates/reasonbraid-server/src/api.rs` + `src/lib.rs`
(the two verbs + the module), and
`crates/reasonbraid-server/tests/profiles.rs` (the measured
resolution) — `\.rs$` + `(^|/)migrations/`.

- [x] **REPRODUCE / ISSUE** — the `.1` census: no resolver registry
  exists — the §12.2 advertise + the resolution order + the
  explicit-failure result have no surface.
- [x] **ROOT CAUSE (WHY + WHERE)** — nothing registered the
  resolvers — `git grep -c "resolver_capabilities\|ResolverAdvertise"
  02d28ae -- crates/ migrations/` → rc=1 (nothing before this
  leaf). The fix point is the durable registry + the resolution
  order the §12.2 contract names (the authz + the risk filters
  FIRST — the scheme + the ADR-018 classes — then the rank), with
  the empty result as the explicit `resource_unresolvable_now`.
- [x] **ADDRESSED (verified)** — measured before→after. Before: the
  grep above. After:
  `DATABASE_URL=postgres://postgres@127.0.0.1:55432/reasonbraid_test
  cargo test -p reasonbraid-server --test profiles
  the_resolver_registry` → `test result: ok. 1 passed` — the
  constrained-process requirement filters the weaker resolver out;
  the latency rank orders the eligible; the off-ladder egress
  claim is the typed 400; the unsupported scheme is the explicit
  `resource_unresolvable_now` AND the reference stays submitted
  (the inspection still reads it — preserved, never fabricated).
  The first live run caught the SQL-continuation doubling (the
  third occurrence of the pattern — fixed).
- [x] **NO REGRESSION** — `bash scripts/run_pg_tests.sh` → 18 live
  suites + the demo `ALL acceptance checks passed` 34/34
  (`target/pg413_guard.log`); `cargo test --all` → 51 offline
  suites green; `cargo clippy --all --all-targets -- -D warnings` →
  clean; `cargo fmt --all -- --check` → rc=0; `make gate` → 13/13 at
  commit.
- [x] **FIX** — `0024_resolver_capabilities.sql`,
  `src/resolvers.rs`, `src/api.rs`, `src/lib.rs`,
  `tests/profiles.rs`.
- [x] **LOCKSTEP** — CHANGELOG, DEV_NOTES, MEMORY, LIVE_STATUS, this
  tree's logs below, `docs/TASK_TREE.md` frontier, KNOWLEDGE_MAP —
  same commit.

## Acceptance Checklist (PHASE-4.1.2)

The CODE change owned by this leaf:
`migrations/0023_resource_references.sql` (NEW),
`crates/reasonbraid-server/src/resources.rs` (NEW — the typed
reference + the digest validation + the submit/get functions),
`crates/reasonbraid-server/src/api.rs` + `src/lib.rs` (the two
verbs + the module), `crates/reasonbraid-server/tests/profiles.rs`
(the measured test), and the eleven purge lists (the 0023 ripple) —
`\.rs$` + `(^|/)migrations/`.

- [x] **REPRODUCE / ISSUE** — the `.1` census: the §12.1 contract
  has no typed shape (only the reason name + the `EvidenceRef`s
  exist) and no submission surface.
- [x] **ROOT CAUSE (WHY + WHERE)** — no reference machinery existed
  — `git grep -c "resource_references\|ResourceReference"
  34419a9 -- crates/ migrations/` → rc=1 (nothing before this
  leaf). The fix point is the typed contract + the durable table
  whose UNIQUE (locator, digest) makes the immutability mechanical
  (the replay + the conflict, never an update).
- [x] **ADDRESSED (verified)** — measured before→after. Before: the
  grep above. After:
  `DATABASE_URL=postgres://postgres@127.0.0.1:55432/reasonbraid_test
  cargo test -p reasonbraid-server --test profiles
  a_reference_submits` → `test result: ok. 1 passed` — the fresh
  submit; the same locator + digest is the REPLAY (the same id);
  the same locator with a different digest is the typed
  `locator_digest_conflict`; the unknown field is the 422; the
  malformed digest is the 400 naming the scheme; the inspection
  reads the submitted shape back. The first live run caught two
  real bugs (the SQL continuation doubling + the digest
  validator's early-return) — both fixed.
- [x] **NO REGRESSION** — `bash scripts/run_pg_tests.sh` → 18 live
  suites + the demo `ALL acceptance checks passed` 34/34
  (`target/pg412_guard.log`); `cargo test --all` → 51 offline
  suites green; `cargo clippy --all --all-targets -- -D warnings` →
  clean; `cargo fmt --all -- --check` → rc=0; `make gate` → 13/13 at
  commit.
- [x] **FIX** — `0023_resource_references.sql`, `src/resources.rs`,
  `src/api.rs`, `src/lib.rs`, `tests/profiles.rs`, the eleven
  purge lists.
- [x] **LOCKSTEP** — CHANGELOG, DEV_NOTES, MEMORY, LIVE_STATUS, this
  tree's logs below, `docs/TASK_TREE.md` frontier, KNOWLEDGE_MAP —
  same commit.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-09-07` | `PHASE-4.2.2` | `cargo test -p reasonbraid-server --lib fetcher` → `test result: ok. 16 passed` (the pure refusals + the OFFLINE wire refusals — the SSRF proof with the zero-request counter, the private hop, the hop cap, the ceilings, the REAL gzip bomb); `cargo test --all` → rc=0, 51 suites, 327 tests (the cert-spike rustls-provider ambiguity the sweep caught is fixed — the spike pinned to ring; `b26f529` verified green before the leaf); `bash scripts/run_pg_tests.sh` → rc=0, 18 live suites + the demo `ALL acceptance checks passed` (`target/pg422_guard.log`); clippy/fmt clean for the leaf's files (the crate-wide `-D warnings` run fails on 9 PRE-EXISTING findings — ROUTING EVIDENCE → `PHASE-4-MAINT-1`); `make gate` → 13/13 | the safe HTTPS fetcher; frontier → `PHASE-4-MAINT-1` → `.2.3` |
| `2026-09-07` | `PHASE-4.1` | docs-only (no code paths changed): `make gate` → 13/13 at commit | Phase 4 opened + the `.1` census + the contract-seam decomposition; frontier → `.1.1` |
| `2026-09-07` | `PHASE-4.2.1` | `cargo test -p reasonbraid-server --lib ssrf` → `test result: ok. 4 passed` (the 18-case refusal matrix, the allowed publics, the mapped-form re-classification, the metadata class); `bash scripts/run_pg_tests.sh` → 18 live suites + the demo 34/34 (`target/pg421_guard.log`); `cargo test --all` → 51 offline suites; clippy/fmt clean; `make gate` → 13/13 | the SSRF classification + policy; frontier → `.2.2` |
| `2026-09-07` | `PHASE-4.2` | docs-only (no code paths changed): `make gate` → 13/13 at commit | the R0 census + the contract-seam decomposition (`.2.1` the SSRF classification → `.2.2` the fetcher → `.2.3` the receipt); frontier → `.2.1` |
| `2026-09-07` | `PHASE-4.1.3` | `DATABASE_URL=… cargo test -p reasonbraid-server --test profiles the_resolver_registry` → `test result: ok. 1 passed` (the filter + the rank, the off-ladder 400, the explicit unresolvable-now with the preserved reference); `bash scripts/run_pg_tests.sh` → 18 live suites + the demo 34/34 (`target/pg413_guard.log`); `cargo test --all` → 51 offline suites; clippy/fmt clean; `make gate` → 13/13 | the resolver capability registry; **`.1` COMPLETE** — frontier → `.2` |
| `2026-09-07` | `PHASE-4.1.2` | `DATABASE_URL=… cargo test -p reasonbraid-server --test profiles a_reference_submits` → `test result: ok. 1 passed` (the submit, the replay, the conflict, the 422/400 refusals); `bash scripts/run_pg_tests.sh` → 18 live suites + the demo 34/34 (`target/pg412_guard.log`); `cargo test --all` → 51 offline suites; clippy/fmt clean; `make gate` → 13/13 | the typed reference + the submission; frontier → `.1.3` |
| `2026-09-07` | `PHASE-4.1.1` | docs-only (no code paths changed): `make gate` → 13/13 at commit | ADR-011 + ADR-018 accepted (the digest scheme + the isolation classes); frontier → `.1.2` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `PHASE-4.2.2` | `REASONBRAID-PHASE4-0007` | the safe HTTPS fetcher (the hardened parse + the two-layer destination enforcement + the manual decode + the ratio brake) — the SSRF proof measured offline |
| `PHASE-4.1` | `REASONBRAID-PHASE4-0001` | the resource-reference lane decomposed at the census seams (the greenfield contract + the registry + the two unopened ADRs) |
| `PHASE-4.2.1` | `REASONBRAID-PHASE4-0006` | the destination classification + the SSRF policy (the pure §12.4 rules + the public-only evaluation) |
| `PHASE-4.2` | `REASONBRAID-PHASE4-0005` | the R0 pack decomposed at the census seams (nothing fetches — the classification/fetcher/receipt are the greenfield) |
| `PHASE-4.1.3` | `REASONBRAID-PHASE4-0004` | the resolver capability registry (the §12.2 advertise + the filter-then-rank resolution + the explicit unresolvable-now) — **`.1` COMPLETE** |
| `PHASE-4.1.2` | `REASONBRAID-PHASE4-0003` | the typed `ResourceReference` + the submission (migration 0023 + the replay/conflict immutability) |
| `PHASE-4.1.1` | `REASONBRAID-PHASE4-0002` | ADR-011 + ADR-018 accepted (the `sha256:<hex>` format + the isolation-class vocabulary — no code) |
