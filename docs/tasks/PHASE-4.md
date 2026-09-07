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
    Status: `done`
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
    Done (`2026-09-07`): the debt is repaired — `cargo clippy --all
      --all-targets -- -D warnings` → rc=0 (the recorded evidence
      reproduces again). The census the `.2.2` routing named was
      NINE findings, but it was a SHADOWED census: the lib
      failure stopped the downstream targets from compiling, so
      THREE more pre-existing findings in the profiles suite
      (`unused_mut` ×2 + `let_and_return`, tests/profiles.rs
      1308/1636/1866) only surfaced after the nine were fixed —
      twelve total, all repaired: the `useless_format` (api), the
      three `type_complexity` sites (the `AttributePicker`, the
      `CallTuple`, the `ResourceRow` aliases — dependence,
      recruitment, resources), the `collapsible_if` (the match
      guard), the `unnecessary if let` (the `.flatten()` form),
      the four `field_reassign` test sites (the struct-update
      form), the `too_many_arguments` (the `OpenCallParams` struct
      + the call site), and the three profiles findings. The
      re-run discipline the shadowing demanded: after EVERY fix
      round the FULL clippy run is the census, never the previous
      error list. Frontier → `.2.3`.

  - ID: `PHASE-4.2.3`
    Status: `done`
    Goal: the snapshot receipt + the pack wiring — the acquisition
      receipt (the ADR-011 `sha256:<hex>` over the ACQUIRED bytes,
      the resolved URL chain, the byte count, the content type,
      the acquisition time — the `.6` snapshot lane's input
      shape), the R0 resolver's registry entry (the `https`
      scheme + the egress/sandbox claims), and the resolution
      path's consumption (the `.1.3` resolve returns the R0
      resolver for the https references).
    Backlog: 32 (the receipt half)
    Done (`2026-09-07`): the R0 pack is WIRED — migration 0025
      seeds the built-in's install record (resolver
      `r0-https-fetcher`: the https scheme, the text/HTML media
      types, the GET/HEAD abilities, the `none` authentication
      class, the ADR-018 claims — egress `listed` (the §12.4
      public-only destination classes are the list), sandbox
      `none` (the fetcher executes NO content — the honest
      ladder-bottom claim, and a stricter requirement is the
      explicit unresolvable-now, never a silent downgrade), the
      `follow-classified` redirect policy, the ADR-011 snapshot
      digest format, the security evidence the `.2.2` tests
      measure); `fetcher.rs` gains the `AcquisitionReceipt` (the
      digest over the ACQUIRED bytes, the byte count, the sniffed
      type, the chain — the raw requested locator + every hop —
      the RFC3339 acquisition time) + `digest_sha256_hex` +
      `FetchError::kind`; `resolvers.rs` gains `R0_RESOLVER_ID` +
      the outcome's optional `acquisition`/`acquisition_error`;
      the resolve handler carries the built-in fetcher in
      `ApiState` and EXECUTES it when the R0 entry ranks first —
      the receipt on success, the NAMED refusal on failure (the
      reference stays submitted either way). Measured (profiles
      15): the https reference resolves to the built-in under its
      own classes; the loopback AND the private literals refuse
      with the class named THROUGH the resolution path (the SSRF
      proof end-to-end); the reference stays readable; the
      stricter requirement is the explicit unresolvable-now. The
      receipt shape is pure-tested (17 fetcher tests now: the
      digest matches the §12.1 validator's scheme). **`.2`
      COMPLETE** — frontier → `.3`.
    Acceptance: the receipt carries the digest + the chain; the R0
      registry entry resolves the https references, measured; no
      regression.

- ID: `PHASE-4.3`
  Status: `done`
  Goal: pack R1 — public Git with immutable commit resolution, limits, submodule/LFS policy
  Backlog: 33
  Roadmap: §12.5
  Children: `.3.1`–`.3.3` (decomposed `2026-09-07` at the census
    seams): `.3.1` the R1 contract + the library census (the §12.5
    rules as the typed contract: the transport + the immutable-
    commit pin, the budget vocabulary, the refusal list, the no-
    checkout-execution rule, the manifest shape — plus the
    measured gitoxide-vs-git2 choice) → `.3.2` the acquisition
    (the clone/fetch with the budgets + the mechanical refusals +
    the resolved-commit recording) → `.3.3` the receipt + the
    pack wiring (the R1 receipt + the `git` registry entry + the
    resolve-path execution).
  Done (`2026-09-07`): the census mapped §12.5 against the shipped
    surface: NOTHING fetches Git — no git library in the lock or
    the registry cache (`git grep -c "gix\|git2\|gitoxide"
    efc9ba8 -- crates/` → rc=1; the "clone" hits are all
    `.cloned()`/`Arc::clone`), and the library choice is OPEN
    (gitoxide = the Rust-native family, git2 = the libgit2 C
    build — the `.3.1` census measures both against the lean
    supply-chain doctrine). The pieces R1 reuses exist: the `.2.1`
    destination policy (the git transport dials under the same
    public-only rule), the `.1.3` registry's `git` scheme slot
    (the test's `rsv-git` reserved it), and the `.2.3` receipt
    (the R1 receipt extends it with the resolved immutable commit
    + the included/excluded manifest). Children at those seams —
    frontier → `.3.1`.

  - ID: `PHASE-4.3.1`
    Status: `done`
    Goal: the R1 contract + the library census — the §12.5 rules
      as the TYPED contract: the URL/ref grammar (the https
      transport, the branch/tag/pinned-commit refs, the host
      allowlist), the transport policy (the `.2.1` destination
      classification at every connection — the git dial is the
      same public-only rule), the budget vocabulary (the object/
      file/path/depth/decompressed-size ceilings), the refusal
      list (submodules, hooks, filters, alternates, external
      diff/clean drivers, Git LFS — ALL default-deny, named), the
      no-checkout-execution rule (the working tree is never
      materialized), the license/retention metadata + the
      included/excluded manifest shape — PLUS the measured
      gitoxide-vs-git2 census (the pure-Rust family vs the
      libgit2 C build, against the lean supply-chain doctrine; the
      crate footprint each pulls). No code.
    Backlog: 33 (the contract half)
    Done (`2026-09-07`): the contract is decided + durable —
      `docs/decisions/2026-09-07_r1-git-acquisition-contract.md`
      (top-level `answers:`): the library is **gix** (measured:
      `cargo add --dry-run gix` → v0.87.1, a pure-Rust family, 32
      features, no C; `cargo add --dry-run git2` → v0.21.0 whose
      features name `openssl-sys`/`vendored-libgit2`/`vendored-
      openssl` — the C surface the lean doctrine rejects); the
      transport MUST ride the classified reqwest stack (the gix
      `http-client-reqwest` backend — every git dial passes the
      `.2.1` policy at both layers, and `.3.2` verifies the
      client-injection seam mechanically); the ref grammar is the
      fragment-carried selector (`https://<host>/<path>[#<ref>]`,
      the bare URL resolves the default tip, and the receipt
      records BOTH the requested ref AND the resolved immutable
      commit); the budget vocabulary (object/file/path/depth/
      decompressed/total-byte ceilings, each refusal named); the
      default-deny refusal list (submodules, hooks, filters,
      alternates, external diff/clean drivers, Git LFS — named,
      never a prompt, never a silent skip); NO checkout execution
      (the worktree is never materialized); the `.2.2` test seams
      (the injectable resolver + policy) carry over so the R1
      wire tests stay OFFLINE. No code changed. Frontier → `.3.2`.
    Acceptance: the contract + the library census land durably (a
      decision record with top-level answers); no code changes.

  - ID: `PHASE-4.3.2`
    Status: `done`
    Goal: the acquisition — the clone/fetch machinery under the
      `.3.1` contract: the transport dials through the `.2.1`
      classification, the shallow/partial fetch where adequate,
      the budgets enforced MECHANICALLY (the object/file/path/
      depth/decompressed-size ceilings trip with their names),
      the refusal list enforced at the configuration level (never
      a prompt, never a silent skip), NO checkout execution (no
      working tree, no hooks, no filters), the resolved immutable
      commit recorded, the archive/symlink/path-traversal checks.
    Backlog: 33 (the acquisition half)
    Done (`2026-09-07`): the acquisition landed —
      `crates/reasonbraid-server/src/git.rs` (gix 0.87.1, the
      pure-Rust engine, driven through an API spike that proved
      the blocking fetch flow end-to-end first): the hardened URL
      grammar (https-only, the userinfo + numeric-literal
      refusals, the fragment-carried ref selector — a branch, a
      tag, a full ref name, or a 40-hex commit, anything else the
      typed refusal), the pre-flight classification (the
      loopback/private literals refuse with the class NAMED
      before any socket opens — the R1 SSRF proof), the
      CLASSIFIED transport (the `.3.1` seam verified mechanically:
      gix's generic `new_http` + the `Http` trait implemented on
      a wrapper around a blocking reqwest client whose DNS rides
      the `.2.2` belt — every dial, redirect hops included, passes
      the destination policy; no proxy env; the unbounded upload
      kind is refused — R1 never pushes), the budgets enforced
      mechanically (the depth ceiling trips DURING the tree walk;
      the file/object/byte ceilings trip with their names), the
      default-deny refusal list (a gitlink entry refuses as a
      SUBMODULE, an LFS pointer blob refuses as Git LFS — named,
      never skipped), NO checkout execution (the target is a BARE
      repository — no worktree, no hooks, no filters), the
      shallow depth (the `DepthAtRemote` mapping), and the
      resolved immutable commit recorded (the requested ref
      resolved through the advertised refs, verified present).
      Five tests: the grammar table, the pre-flight refusals, the
      acquisition over the injected file transport (the offline
      wire path — the resolved commit + the counts measured), the
      submodule/LFS refusals, the budget trips. The first runs
      caught three real bugs (the WHATWG numeric-host
      normalization, the second commit's missing parent, the
      depth ceiling not counting leaf entries) — all fixed. The
      offline sweep ALSO caught the THIRD occurrence of the
      workspace rustls-provider rule: gix's
      `blocking-http-transport-reqwest-rust-tls` feature pulled
      reqwest's plain `rustls` (the aws-lc-rs default provider)
      back into the union — fixed by the backend-less
      gix-transport `http-client` feature (the classified wrapper
      IS the backend); the rule is sharpened in the
      `2026-09-07_workspace-single-rustls-provider.md` record.
      Frontier → `.3.3`.

  - ID: `PHASE-4.3.3`
    Status: `done`
    Goal: the receipt + the pack wiring — the R1 receipt (the
      resolved immutable commit + the requested URL/ref + the
      included/excluded manifest + the `.2.3` receipt's digest/
      chain fields), the R1 resolver's registry entry (the `git`
      scheme the `.1.3` test reserved + the egress/sandbox
      claims), and the resolution path's consumption (the `.1.3`
      resolve returns the R1 resolver for the git references, and
      the resolve handler executes it when it ranks first —
      mirroring the `.2.3` R0 wiring).
    Backlog: 33 (the receipt half)
    Done (`2026-09-07`): the R1 pack is WIRED — migration 0026
      seeds the install record (resolver `r1-git-fetcher`: the
      `git` scheme, the https transport patterns, the
      clone/fetch abilities, the `none` auth class, egress
      `listed` + sandbox `none` (the honest claims — no worktree
      is ever materialized, no repository code executes), the
      `follow-classified` redirect policy, the ADR-011 digest
      format, the refusal/budget evidence); `git.rs` gains the
      `GitReceipt` (the resolved immutable commit, the requested
      URL/ref, the ADR-011 digest over the ACQUIRED odb bytes —
      every object file sorted + hashed, the chain, the counts,
      the included/excluded manifest — the walk now collects the
      paths) + the pure receipt test (6 tests now);
      `resolvers.rs` gains `R1_RESOLVER_ID` + the untagged
      `Acquisition` enum (Web/Git receipts); the resolve handler
      carries the `GitFetcher` in `ApiState` and EXECUTES the R1
      pack when it ranks first — the receipt on success, the
      NAMED refusal on failure (the reference preserved either
      way). Measured (profiles 16): the git reference resolves to
      `r1-git-fetcher` under its own classes; the loopback
      literal refuses with the class named THROUGH the resolution
      path (the R1 SSRF proof end-to-end); the reference stays
      readable; the stricter requirement is the explicit
      unresolvable-now. **`.3` COMPLETE (pack R1)** — frontier →
      `.4`.

- ID: `PHASE-4.4`
  Status: `done`
  Goal: pack R2 — PDFs/text/structured feeds/archives in sandboxed extraction workers
  Backlog: 34
  Roadmap: §12.3
  Children: `.4.1`–`.4.3` (decomposed `2026-09-07` at the census
    seams): `.4.1` the R2 contract + the parser census (the format
    set, the §12.6 Derivation shape, the WORKER sandbox class —
    the honest ladder-up from the packs' `none` — the measured
    parser-library census) → `.4.2` the extraction workers (the
    worker process + the stdio protocol + the per-format parsers +
    the mechanical refusals) → `.4.3` the receipt + the pack
    wiring (the Derivation receipt + the media-type routing + the
    resolve-path execution).
  Done (`2026-09-07`): the census mapped §12.3's R2 row against
    the shipped surface: NOTHING extracts — no PDF/zip/tar/feed
    crate in the lock (only flate2, the fetcher's gzip), and the
    src's `extract` hits are axum's `extract` module + the CA's
    EC-point helper (`git grep -c "extract\|pdf\|archive"
    a18a25e -- crates/reasonbraid-server/src/` → the hits are
    unrelated). The R2 lane is a greenfield with the §12.3 row as
    its spec. The pieces R2 reuses exist: the receipt shapes (the
    Web/Git acquisitions), the ADR-018 vocabulary (R2's parsers
    claim `process` at least — the honest ladder-up: untrusted
    CONTENT executes, unlike R0/R1's byte-only `none`), the
    adapter lane's subprocess pattern (the worker-quarantine
    boundary), and the §12.6 Derivation shape (the extraction/
    normalization version + the derived chunk digests + the
    parent links). Children at those seams — frontier → `.4.1`.

  - ID: `PHASE-4.4.1`
    Status: `done`
    Goal: the R2 contract + the parser census — the §12.3 R2 row
      as the TYPED contract: the format set (PDF; the open
      formats — UTF-8 text, the structured feeds (Atom/RSS), the
      archives (zip/tar)); the extraction contract (the §12.6
      Derivation edge: the extraction/normalization version, the
      derived text + chunk digests, the parent link to the
      acquired bytes — the extraction is ALWAYS a derivation,
      never the original); the sandbox claim (the parsers run in
      DEDICATED WORKER PROCESSES — the ADR-018 `process` class,
      the honest ladder-up from the packs' `none`: untrusted
      content EXECUTES here, so the boundary is the worker
      process + the stdio protocol, mirroring the adapter lane's
      subprocess pattern); the refusal vocabulary (encrypted
      PDFs, JS-bearing PDFs, recursive/nested archives, path
      traversal, the archive bomb — the decompression-ratio rule
      again); the media-type routing design (the R2 pack
      PIPELINES the acquisition packs: it advertises the https/
      git schemes with its media types and consumes the R0/R1
      acquisition internally when it ranks) — PLUS the measured
      parser-library census (lopdf vs pdf; zip/tar/atom_syndication
      — the pure-Rust families against the lean supply-chain
      doctrine). No code.
    Backlog: 34 (the contract half)
    Done (`2026-09-07`): the contract is decided + durable —
      `docs/decisions/2026-09-07_r2-extraction-contract.md`
      (top-level `answers:`): the format set (PDF text only, the
      one-level archives, the Atom/RSS feeds — plain text is NOT
      an extraction); the extraction is ALWAYS a Derivation (the
      parent digest + the extractor version + the derived chunks,
      each with its own ADR-011 digest); the sandbox claim is
      `process` — the FIRST ladder-up (the parsers EXECUTE
      untrusted content in a dedicated worker process per
      extraction: the stdio JSON protocol + the killing budgets =
      the quarantine); the refusal vocabulary decided before the
      parsers (encrypted/JS PDFs, nested archives, traversal, the
      bomb); the media-type routing (the R2 pack advertises the
      acquisition schemes with its media types; a hinted
      reference pipelines acquire→extract, a hintless one stays
      acquisition-only — the `.4.3` resolve gains the filter);
      the parser census MEASURED (`cargo add --dry-run`: lopdf
      0.44.0 chosen, pdf 0.10.0 rejected as the lower-level API,
      zip 8.6.0 + tar 0.4.46, atom_syndication 0.12.10 — all
      pure Rust, no C). No code changed. Frontier → `.4.2`.

  - ID: `PHASE-4.4.2`
    Status: `done`
    Goal: the extraction workers — the worker process under the
      `.4.1` contract: the stdio JSON protocol (the acquired bytes
      or a temp path in, the derived text + the chunk digests +
      the refusal out), the per-format parsers, the mechanical
      refusals (the encrypted/JS PDFs, the nested archives, the
      traversal, the bomb — each named), the per-extraction
      limits (the byte/time ceilings on the WORKER side), the
      quarantine boundary (the worker is a fresh process per
      extraction — nothing persists).
    Backlog: 34 (the worker half)
    Done (`2026-09-07`): the extraction worker landed —
      `crates/reasonbraid-extract` (the new workspace crate, the
      `.4.1` census's pure-Rust set: lopdf/zip(deflate)/tar/
      atom_syndication): the stdio JSON protocol (ONE request —
      the temp path + the media type + the ceilings — in, ONE
      response out, exit; the refusal envelope is ALWAYS
      `{error: {kind, message}}`), the per-format parsers (the
      PDF text layer per page, the one-level archives with the
      text-entry chunks + the excluded list, the feed titles +
      entries), the mechanical refusals each named (the
      encrypted + JS-bearing PDFs — the catalog walk, the nested
      archives, the path traversal, the decompression-ratio brake
      over the COMPRESSED envelope vs the decoded bytes, the
      entry/input/output/chunk ceilings), the Derivation shape
      (the parent digest over the input + every chunk's own
      ADR-011 digest + the extractor version), and the
      quarantine boundary (a fresh process per extraction —
      nothing persists). Nine tests: the per-format extractions +
      refusals (the PDF fixture built with lopdf's own writer),
      the zip/tar/feed fixtures, the output ceiling, and the two
      stdio roundtrips spawning the BUILT binary (the Derivation
      response + the named refusal). The first runs caught three
      real bugs (the PDF fixture's missing xref, the page-number
      vs page-id argument, the ratio brake comparing the declared
      size instead of the compressed envelope) — all fixed.
      Frontier → `.4.3`.

  - ID: `PHASE-4.4.3`
    Status: `done`
    Goal: the receipt + the pack wiring — the R2 receipt (the
      Derivation edge: the derived chunk digests + the parent
      digest + the extractor version), the R2 registry entry (the
      media-type routing per the `.4.1` design — the advertised
      media types + the sandbox `process` claim), and the
      resolution path's consumption (the resolve returns the R2
      pack for the media-typed references, and the handler
      executes the pipeline — the acquisition then the extraction
      — when it ranks first, mirroring the `.2.3`/`.3.3` wiring).
    Backlog: 34 (the receipt half)
    Done (`2026-09-07`): the R2 pack is WIRED — migration 0027
      seeds the install record (resolver `r2-extract-worker`: the
      https scheme, the extraction media types, the `extract`
      ability, egress `listed` + sandbox `process` — the FIRST
      ladder-up, with the worker-quarantine evidence, the
      kill-on-budget-trip marker); `src/extraction.rs` (NEW): the
      `ExtractionReceipt` (the Derivation edge — the parent
      digest, the derived chunks each with their own ADR-011
      digest, the extractor version, the excluded list) + the
      spawner (`run_extraction`: the temp file + ONE request
      line, the ONE response line, the time budget KILLS the
      worker on the trip — the quarantine's enforcement; the
      worker path = the `R2_WORKER_BIN` override or the
      server-binary-adjacent default) + the spawner test (the
      real binary, the skip-if-absent pattern); `resolvers.rs`:
      `R2_RESOLVER_ID` + the resolve's MEDIA-TYPE filter (a
      hinted reference ranks only the resolvers whose advertised
      types include the hint; a hintless one keeps the
      acquisition-only path) + the `Extract` acquisition variant;
      the resolve handler EXECUTES the pipeline when the R2 pack
      ranks first — the R0 fetcher acquires the bytes under the
      `.2.1` policy (the refusal names the class), then the
      worker derives the chunks (the receipt on success, the
      NAMED refusal on failure, the reference preserved either
      way). Measured (profiles 17): the hinted reference ranks
      `r2-extract-worker` under its own classes; the pipeline's
      acquisition leg refuses the loopback with the class named
      THROUGH the resolution path (the SSRF proof end-to-end);
      the hintless reference keeps the R0 acquisition-only path;
      the stricter requirement is the explicit unresolvable-now.
      **`.4` COMPLETE (pack R2)** — frontier → `.5`.

- ID: `PHASE-4.5`
  Status: `done`
  Goal: opt-in private/authenticated connectors (R5) and sandboxed browser/agent-mediated acquisition (R3/RX)
  Roadmap: §12.3, §12.8
  Note: highest risk; do not enable by default
  Children: `.5.1`–`.5.3` (decomposed `2026-09-07` at the census
    seams): `.5.1` the three contracts + the OPT-IN gate (the R5
    credential broker, the R3 browser worker, the RX §12.8
    vocabulary — one decision record, plus the measured browser/
    broker censuses) → `.5.2` the machinery (the broker + the
    browser worker + the agent-mediated client — ALL behind the
    gate) → `.5.3` the receipt + the wiring (the disclosure
    receipts + the gated registry entries + the resolve path).
  Done (`2026-09-07`): the census mapped §12.3's R3/R5/RX rows +
    §12.8 against the shipped surface: NOTHING exists for the
    browser, the credential broker, or the agent-mediated
    acquisition — no browser/MCP crate in the lock (`git grep -c
    "credential_broker\|browser\|mcp\|a2a" e2b43ea -- crates/`
    → the only hits are the WEB UI's app.js/index.html), and the
    credential surface is the `.1.2` opaque
    `credential_binding_ref` (never a secret) + the fetcher's
    no-ambient-credentials baseline. The lane is a greenfield
    with the R3/R5/RX rows + §12.8 as its spec. The pieces it
    reuses exist: the worker-subprocess pattern (R2's), the
    ADR-018 ladder's top (the browser's isolation claim), the
    registry's `authentication_classes` column (all `none` so
    far), and the §12.9 budget vocabulary (the browser steps +
    the model calls join it). Children at those seams — frontier
    → `.5.1`.

  - ID: `PHASE-4.5.1`
    Status: `done`
    Goal: the three contracts + the OPT-IN gate — the R5
      credential-broker contract (the LOCAL broker only — the
      credential never enters the reference, the
      `credential_binding_ref` is the opaque handle, the
      delegated-session model, the explicit-disclosure rule), the
      R3 browser contract (the bounded interaction — the step
      budget, the network log, the rendering policy; the
      isolation claim — the ADR-018 ladder's TOP; the measured
      browser-runtime census — the CDP chromium reality vs the
      pure-Rust doctrine, the binary's provenance named), the RX
      §12.8 contract (the acquisition-call response vocabulary —
      the snapshot/excerpt/structured-fact/redacted-derivative/
      test-receipt/refusal; the not-inspected-original record;
      the second-verifier rule), and the ENABLEMENT gate (the
      packs ship compiled but DISABLED — the open state is a
      named configuration change, never a default, never a
      registry row) — one decision record with top-level
      `answers:`. No code.
    Backlog: 35 (the contracts half)
    Done (`2026-09-07`): the contracts are decided + durable —
      `docs/decisions/2026-09-07_r5r3rx-contracts-opt-in.md`
      (top-level `answers:`): R5 — the LOCAL broker (the opaque
      binding ref, the per-request delegated session, the
      explicit-disclosure receipt — a credential is a disclosure,
      not a permission); R3 — the bounded interaction (the step +
      network-log budgets, the killing worker, the rendering
      denials) + the DEPLOYMENT-CHECKED isolation (the
      `vm_container` requirement the gate refuses to open
      without) + the measured browser census (chromiumoxide
      0.9.1 chosen over headless_chrome 1.0.22; the engine binary
      = a pinned chromium, not vendored — the provenance named +
      the startup version check; the pure-Rust doctrine ends at
      the browser engine); RX — the typed §12.8 vocabulary (the
      six response shapes + the not-inspected-original record +
      the second-verifier rule); the OPT-IN gate — the packs ship
      COMPILED but DISABLED, the enablement is a named recorded
      configuration change, the resolve never returns a disabled
      pack, the default is OFF everywhere. No code changed.
      Frontier → `.5.2`.

  - ID: `PHASE-4.5.2`
    Status: `done`
    Goal: the machinery — the credential broker, the browser
      worker, and the agent-mediated client, ALL behind the `.5.1`
      gate: the broker resolves the binding ref to the session
      credential at the request boundary (the credential never
      logs, never persists in the reference), the browser worker
      mirrors the R2 worker's stdio quarantine with the step +
      network-log budgets, the §12.8 response shapes are the
      typed vocabulary the enrolled-agent surface answers with.
    Backlog: 35 (the machinery half)
    Done (`2026-09-07`): the machinery landed, all COMPILED but
      UNWIRED (the gate is the `.5.3` wiring's) —
      `crates/reasonbraid-browse` (the new workspace crate): the
      R3 browser worker — the stdio protocol (the already-
      classified URL + the step list + the budgets in, the
      rendered chunks + the NETWORK LOG + the browser version
      out; the refusal envelope is `{error: {kind, message}}`),
      the bounded interaction (navigate/click/scroll/type with
      the step budget + the wall-clock ceiling — the trip
      refuses), the network-log disclosure (every request the
      page makes, recorded), the startup check (the
      provenance-named browser binary — `R3_BROWSER_BIN` or the
      platform defaults — the worker refuses to run without it),
      and the Derivation shape (the parent digest + the chunk
      digest). TWO tests pass against the REAL Chrome (the local
      origin render + the network log; the step-budget refusal
      before any navigation — the skip pattern for machines
      without a browser); `src/broker.rs` (the R5 broker: the
      local store, the opaque binding ref, the per-request
      Authorization attach, the REDACTED Debug (the value never
      logs), the `DisclosureRecord`) + `src/mediated.rs` (the
      typed §12.8 vocabulary: the six response shapes, the
      not-inspected-original record, the second-verifier rule) —
      four unit tests. The credential-broker's keychain
      integration stays the deployment's (named, out of the dev
      profile). Frontier → `.5.3`.

  - ID: `PHASE-4.5.3`
    Status: `done`
    Goal: the receipt + the wiring — the disclosure receipts (the
      explicit-disclosure record: what credential class, what
      was disclosed), the gated registry entries (the R3/R5/RX
      rows exist ONLY when the gate is open), and the resolution
      path's consumption (the resolve returns the gated packs
      only when enabled; the handler executes them behind the
      same ranked-first rule, mirroring the `.2.3`–`.4.3`
      wiring).
    Backlog: 35 (the receipt half)
    Done (`2026-09-07`): the gate is WIRED — `resolvers.rs`: the
      three gated ids + the STARTUP SYNC (`sync_gated_entries`:
      opening registers the R3/R5/RX rows, closing REMOVES them —
      the resolve never returns a disabled pack because the
      disabled pack has no row) + the auth filter (a
      credential-carrying reference ranks only the `credential`
      class; a binding-less one only the `none` class) + the
      `acquisition_call` outcome field; `fetcher.rs`: the
      per-request `fetch_authenticated` (the credential attaches
      for THIS acquisition only — never ambient) + the
      `preflight` (the R3 spawn's classification gate);
      `browse.rs` (NEW): the R3 spawner (ONE request line, ONE
      response line, the time budget KILLS the worker) + the
      `BrowserReceipt` (the network-log disclosure); `broker.rs`:
      the `AuthenticatedReceipt` (the disclosure + the web
      receipt); the resolve handler: the R5 branch (the broker
      resolve → the authenticated fetch → the disclosure
      receipt), the R3 branch (the preflight → the spawner), the
      RX branch (the §12.8 capability-call publication) — ALL
      behind the belt `state.r5r3rx_enabled`; the binary's
      startup syncs the gate (the `RB_ENABLE_R5R3RX` env, OFF by
      default); the `api_router_gated` seam. Measured (profiles
      18): the gate CLOSED — the binding-carrying reference is
      the explicit unresolvable-now; the gate OPEN — the R5 pack
      ranks + the authenticated acquisition refuses the loopback
      with the class NAMED (the SSRF proof through the
      authenticated path), the R3 pack ranks + the pre-flight
      refuses before any worker spawn, the RX pack ranks + the
      capability call publishes the locator; the gate CLOSED
      again — the rows are gone, the unresolvable-now returns.
      **`.5` COMPLETE (the gated lane)** — frontier → `.6`.
      (The RX delivery — the actual agent round-trip — rides the
      capability-call lane; the vocabulary + the publication
      shape ship here.)

- ID: `PHASE-4.6`
  Status: `done`
  Goal: content-addressed snapshots, derivation graph, claim-evidence graph, citation validation, license/retention, freshness
  Backlog: 35
  Roadmap: §12.6–12.7, §12.9
  Children: `.6.1`–`.6.4` (decomposed `2026-09-07` at the census
    seams): `.6.1` the snapshot store + the tombstone (the §12.6
    `EvidenceSnapshot` table + the content-addressed object store
    — the `.1.1`'s named trigger — + the retention classes + the
    tombstone rule) → `.6.2` the derivation graph (the
    `Derivation` edges + the traversal) → `.6.3` the
    claim-evidence graph + the citation validation (the five
    assessments + the digest/selector checks) → `.6.4` the
    license/retention + the freshness (the §12.9 re-fetch
    policy).
  Done (`2026-09-07`): the census mapped §12.6–12.7/§12.9 against
    the shipped surface: the RECEIPTS exist (the `.2`–`.5` packs
    produce the Web/Git/Extract/Authenticated/Browse shapes, all
    with the ADR-011 digests + the parent links) but NOTHING
    persists them — no `EvidenceSnapshot` machinery, no
    `Derivation` edges, no claim-evidence assessments, no
    tombstone (`git grep -c "EvidenceSnapshot\|tombstone"
    f4696a6 -- crates/ migrations/` → the hits are the
    node-inbox's Phase-2 retention + the references'
    `retention_class` field — unrelated). The lane is a
    greenfield with §12.6–12.7/§12.9 as its spec; it is the
    Phase-4 blocker's LAST leg (the object store the `.1` census
    named). Children at those seams — frontier → `.6.1`.

  - ID: `PHASE-4.6.1`
    Status: `done`
    Goal: the snapshot store + the tombstone — the §12.6
      `EvidenceSnapshot` (the original reference + the resolved
      final locator; the retrieval time, the resolver
      identity/version, the network + auth class; the provider
      receipts + the immutable source version; the raw-byte
      digest, length, media type, storage/retention class; the
      quarantine/quality status), the content-addressed object
      store (the acquired bytes persisted under their digest —
      the `.1.1`'s named trigger), the submission surface (the
      `.2`–`.5` receipts land here), the retention classes + the
      tombstone rule (the deletion creates the tombstone + the
      reason — never a silent disappearance).
    Backlog: 35 (the store half)
    Done (`2026-09-07`): the snapshot store landed — migration
      0028 (`snapshot_objects`: the content-addressed store — the
      raw bytes persist UNDER their ADR-011 digest, shared:
      identical bytes = one row; `evidence_snapshots`: the §12.6
      shape — the reference FK rides ON DELETE CASCADE, the
      tombstone state rides the row's `deleted_at`/
      `deletion_reason`); `crates/reasonbraid-server/src/
      snapshots.rs` (NEW): the typed `SnapshotSubmission` (the
      deny-unknown-fields boundary + the ADR-011 digest
      validation), the `submit` (the bytes MUST hash to the
      declared digest — the content-addressing is VERIFIED, not
      trusted; the same reference + digest is the REPLAY), the
      `get` (the `FromRow` struct — the 22-column row exceeds
      sqlx's tuple impls), and the `tombstone` (the deletion
      records the reason + the time; idempotent — the first
      reason wins); the verbs: `POST /v1/snapshots` (the base64
      bytes — the base64 0.22 dep rides the lock's single
      version), `GET /v1/snapshots/{id}`, `DELETE
      /v1/snapshots/{id}` (the reason required); the resolve
      handler's R0/R2/R5 SUCCESS paths auto-submit the acquired
      bytes (the provider receipts + the disclosure policies ride
      the rows; a persistence failure leaves the receipt returned
      — the acquisition succeeded either way; the GIT/BROWSE
      auto-submissions ride `.6.2`'s derivation layer). Measured
      (profiles 19): the submit → the read-back, the replay (the
      same id), the digest-mismatch 400 (the verification), the
      tombstone (the row stays readable with the reason + the
      time). The first live run hit the SQL-continuation
      doubling bug (the FOURTH occurrence of the pattern — the
      `\\\n` heredoc artifact) inside the object-upsert query —
      fixed with the `s.replace` sweep; the live debug also
      proved the exists-check's false-positive reading was the
      masked DB error, not the check. Frontier → `.6.2`.

  - ID: `PHASE-4.6.2`
    Status: `done`
    Goal: the derivation graph — the `Derivation` edges (every
      transformation: the snapshot → the derived chunks, the
      extract/browse receipts' parent links, the
      extraction/normalization version), the persistence + the
      read surface (the parent/derived traversal — a quote or a
      summary is NEVER the original, and the graph says so).
    Backlog: 35 (the graph half)
    Done (`2026-09-07`): the derivation graph landed — migration
      0029 (`derivations`: the `Derivation` edge — the parent
      snapshot FK (CASCADE), the derived kind (the §12.6 named
      set), the derived content's OWN ADR-011 digest, the
      extraction version, the source selector, the replay unique
      index); `crates/reasonbraid-server/src/derivations.rs`
      (NEW): the typed `DerivationSubmission` (the
      deny-unknown-fields boundary), the `submit` (the content
      MUST hash to the declared digest — verified, never
      trusted; the parent must exist; the same parent + kind +
      digest is the REPLAY), the `children_of` traversal (the
      parent/derived read surface — a quote or a summary is
      NEVER the original, and the graph says so); the verbs:
      `POST /v1/derivations` + `GET
      /v1/snapshots/{id}/derivations`; the resolve handler's R2
      branch auto-submits the extract chunks as the snapshot's
      Derivation edges (the R3 render's derivations ride the
      `.6.4` lane — its source snapshot is defined with the
      freshness). Measured (profiles 20): the edge roundtrip,
      the replay, the traversal, the digest-mismatch 400, the
      missing-parent refusal. The write-tool's verbatim
      continuations re-introduced the SQL-doubling (the FIFTH
      occurrence) — caught at the pre-compile sweep this time.
      Frontier → `.6.3`.

  - ID: `PHASE-4.6.3`
    Status: `done`
    Goal: the claim-evidence graph + the citation validation —
      the five assessments (supports/contradicts/contextualizes/
      source_only/unverifiable) with the author/verifier, the
      excerpt/selector, the entailment rationale, the source
      authority, the freshness, the independence/dependence
      indicators, the uncertainty; the citation validation (the
      claim's digest re-verification + the excerpt/selector check
      — the citation must point at a REAL snapshot; citation
      existence alone never satisfies an evidence gate).
    Backlog: 35 (the claim half)
    Done (`2026-09-07`): the claim-evidence graph landed —
      migration 0030 (`claim_assessments`: the claim → snapshot
      edge with the FIVE assessment kinds (the CHECK constraint),
      the author/verifier, the excerpt + the selector, the
      rationale, the authority/freshness/independence/
      uncertainty, the replay unique index);
      `crates/reasonbraid-server/src/claims.rs` (NEW): the typed
      `AssessmentSubmission` + the CITATION VALIDATION (the
      excerpt MUST appear in the snapshot's raw bytes — the
      object store is consulted; a fake excerpt is the typed
      refusal: citation existence alone never satisfies an
      evidence gate) + the replay + the two read surfaces (the
      snapshot's + the claim's assessments); the verbs: `POST
      /v1/assessments` + `GET /v1/snapshots/{id}/assessments` +
      `GET /v1/claims/{claim_id}/assessments`. Measured (profiles
      21): the true excerpt accepts + the replay, the FAKE
      excerpt refuses (the validation), the unknown kind refuses
      with its name, the two read surfaces. The SQL-doubling
      struck a SIXTH time (the write-tool continuations — rustc
      accepts the doubled form silently, embedding the backslash
      in the SQL) — swept at the pre-compile check. Frontier →
      `.6.4`.

  - ID: `PHASE-4.6.4`
    Status: `proposed`
    Goal: the license/retention + the freshness — the retention
      classes' enforcement (the tombstone from `.6.1` rides the
      class's expiry), the license metadata, the freshness (the
      §12.9 re-fetch policy + the staleness surface the
      assessments read).
    Backlog: 35 (the retention half)

- ID: `PHASE-4.7`
  Status: `proposed`
  Goal: G4 hostile-content suite; explicit failure for unsupported references
  Gate: G4; subtraction record required

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `PHASE-4.6.4` | `proposed` | `.6.3` done — the claim-evidence graph + the citation validation (migration 0030, the five assessments, the excerpt-in-the-bytes check, the two read surfaces — profiles 21); the license/retention + the freshness execute now |

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
- `2026-09-07`: `.6.3` done — the claim-evidence graph + the
  citation validation (migration 0030: the five assessments +
  the excerpt-in-the-bytes check — the fake excerpt refuses;
  profiles 21); frontier → `.6.4`.
- `2026-09-07`: `.6.2` done — the derivation graph (migration
  0029: the `Derivation` edges with the verified digests + the
  replay + the children traversal; the R2 chunks auto-derive);
  profiles 20; frontier → `.6.3`.
- `2026-09-07`: `.6.1` done — the snapshot store + the
  tombstone (migration 0028: the content-addressed objects + the
  §12.6 shape; the VERIFIED digest, the replay, the tombstone
  rule; the R0/R2/R5 auto-submits); profiles 19; frontier →
  `.6.2`.
- `2026-09-07`: `.6` decomposed at the census seams — the
  receipts exist (the `.2`–`.5` packs), NOTHING persists them
  (no snapshot/derivation/claim/tombstone machinery — the object
  store is the Phase-4 blocker's last leg); children `.6.1` (the
  snapshot store + the tombstone) → `.6.2` (the derivation
  graph) → `.6.3` (the claim-evidence graph + the citation
  validation) → `.6.4` (the license/retention + the freshness);
  frontier → `.6.1`.
- `2026-09-07`: `.5.3` done — the gated receipt + the wiring
  (the startup sync — the rows exist ONLY while the gate is
  open; the auth filter; the per-request authenticated fetch +
  the preflight + the render spawner + the capability-call
  publication — all behind the enabled belt); profiles 18;
  **`.5` COMPLETE (the gated lane)** — frontier → `.6`.
- `2026-09-07`: `.5.2` done — the machinery, compiled but
  unwired: the browser worker (`crates/reasonbraid-browse` — the
  stdio protocol, the step + network-log budgets, the startup
  browser check; two tests against the REAL Chrome), the
  credential broker (`src/broker.rs` — the redacted value, the
  disclosure record), the §12.8 vocabulary (`src/mediated.rs`);
  frontier → `.5.3`.
- `2026-09-07`: `.5.1` done — the three contracts + the OPT-IN
  gate: the local disclosed credential broker, the bounded
  browser with the deployment-checked `vm_container` requirement
  (chromiumoxide measured), the typed §12.8 vocabulary, and the
  off-by-default enablement — durable in `docs/decisions/2026-
  09-07_r5r3rx-contracts-opt-in.md`; no code; frontier → `.5.2`.
- `2026-09-07`: `.5` decomposed at the census seams — NOTHING
  exists for the browser/credential-broker/agent-mediated
  acquisition (no crate in the lock; only the `.1.2` opaque
  binding-ref hook + the worker-subprocess pattern exist to
  reuse); children `.5.1` (the three contracts + the OPT-IN
  gate) → `.5.2` (the machinery, gated) → `.5.3` (the receipt +
  the wiring); frontier → `.5.1`.
- `2026-09-07`: `.4.3` done — the R2 receipt + the pack wiring
  (migration 0027's install record: the extraction media types +
  the sandbox `process` claim — the first ladder-up; the
  `ExtractionReceipt` Derivation edge; the resolve's media-type
  filter; the handler's pipeline — the acquisition under the
  `.2.1` policy, then the killing-budget worker); profiles 17;
  **`.4` COMPLETE (pack R2)** — frontier → `.5`.
- `2026-09-07`: `.4.2` done — the extraction worker
  (`crates/reasonbraid-extract`: the stdio JSON protocol, the
  per-format parsers, the named refusals — the encrypted/JS
  PDFs, the nested archives, the traversal, the ratio brake over
  the compressed envelope — the Derivation chunks); nine tests
  (the fixtures + the stdio roundtrips); frontier → `.4.3`.
- `2026-09-07`: `.4.1` done — the R2 contract + the parser
  census: the Derivation-only extraction, the `process`-class
  worker quarantine (the stdio protocol + the killing budgets),
  the named refusal vocabulary, the media-type routing, and the
  measured crate set (lopdf/zip/tar/atom_syndication, all pure
  Rust) — durable in `docs/decisions/2026-09-07_r2-extraction-
  contract.md`; no code; frontier → `.4.2`.
- `2026-09-07`: `.4` decomposed at the census seams — NOTHING
  extracts (no PDF/zip/tar/feed crate in the lock; the R0/R1
  pieces exist to reuse: the receipts, the ADR-018 ladder, the
  subprocess pattern, the Derivation shape); children `.4.1` (the
  R2 contract + the parser census) → `.4.2` (the extraction
  workers) → `.4.3` (the receipt + the pack wiring); frontier →
  `.4.1`.
- `2026-09-07`: `.3.3` done — the R1 receipt + the pack wiring
  (migration 0026's install record: the `git` scheme, the honest
  listed/none claims, the no-worktree evidence; the `GitReceipt`
  with the ADR-011 digest over the acquired odb bytes + the
  included/excluded manifest; the untagged Web/Git acquisition
  outcome; the resolve handler executes the R1 pack when it ranks
  first); profiles 16 (the git reference resolves; the loopback
  refusal names the class through the resolution path; the
  reference preserved); **`.3` COMPLETE (pack R1)** — frontier →
  `.4`.
- `2026-09-07`: `.3.2` done — the R1 acquisition (`src/git.rs`:
  gix over the classified transport — the `Http` trait wrapped
  around the belt-riding blocking client (the seam verified
  mechanically), the hardened URL grammar + the fragment-carried
  ref, the pre-flight class-named refusals, the mechanical budget
  trips, the submodule/LFS named refusals, the bare-repo
  no-checkout property, the resolved immutable commit); five
  tests (the offline file-transport wire path); frontier → `.3.3`.
- `2026-09-07`: `.3.1` done — the R1 contract + the library
  census: gix v0.87.1 (the pure-Rust family, measured against
  git2's `openssl-sys`/`vendored-libgit2` C surface); the
  classified reqwest transport requirement, the fragment-carried
  ref, the budget + default-deny vocabularies, no checkout
  execution — durable in `docs/decisions/2026-09-07_r1-git-
  acquisition-contract.md`; no code; frontier → `.3.2`.
- `2026-09-07`: `.3` decomposed at the census seams — NOTHING
  fetches Git (no git library in the lock or the registry cache;
  the R0 pieces exist to reuse: the destination policy, the `git`
  registry slot, the receipt shape); children `.3.1` (the R1
  contract + the library census) → `.3.2` (the acquisition) →
  `.3.3` (the receipt + the pack wiring); frontier → `.3.1`.
- `2026-09-07`: `.2.3` done — the snapshot receipt + the R0 pack
  wiring (migration 0025's install record: the https scheme, the
  ADR-018 honest claims — egress `listed`, sandbox `none` — the
  `follow-classified` redirect policy, the ADR-011 digest format;
  the `AcquisitionReceipt` + the digest-over-acquired-bytes + the
  `FetchError::kind`; the resolve handler executes the built-in
  when it ranks first — the receipt on success, the NAMED refusal
  on failure, the reference preserved); profiles 15 (the https
  reference resolves; the loopback + private literals refuse with
  the class named through the resolution path; the stricter
  requirement is the explicit unresolvable-now); **`.2` COMPLETE
  (pack R0)** — frontier → `.3`.
- `2026-09-07`: `PHASE-4-MAINT-1` done — the clippy evidence debt
  repaired: `cargo clippy --all --all-targets -- -D warnings` →
  rc=0 — TWELVE pre-existing findings fixed (the nine the `.2.2`
  routing named + three profiles findings the lib failure had
  shadowed — the census is the full run after every fix round);
  the `OpenCallParams` refactor, the three row/attribute type
  aliases, the match-guard + flatten + struct-update forms;
  frontier → `.2.3`.
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

## Acceptance Checklist (PHASE-4.6.3)

The CODE change owned by this leaf:
`migrations/0030_claim_assessments.sql` (NEW — the assessment
edges), `crates/reasonbraid-server/src/claims.rs` (NEW — the
typed submission + the citation validation + the read surfaces),
`src/api.rs` (the three verbs), `src/lib.rs` (the module), and
`tests/profiles.rs` (profiles 21).

- [x] **REPRODUCE / ISSUE** — the `.6.2` close: the snapshots +
  the derivations exist, but no claim links to the evidence with
  an assessment, and nothing VALIDATES the citation.
- [x] **ROOT CAUSE (WHY + WHERE)** — the claim layer was the
  lane's third third — `git grep -c "claim_assessments\|AssessmentSubmission"
  4856aff -- crates/ migrations/` → rc=1 (nothing before this
  leaf). The fix point is the §12.7 edge + the validation: the
  excerpt must appear in the snapshot's raw bytes.
- [x] **ADDRESSED (verified)** — measured before→after. Before: the
  grep above. After:
  `DATABASE_URL=… cargo test -p reasonbraid-server --test
  profiles the_claim_assessments` → `test result: ok. 1 passed`
  — the TRUE excerpt accepts (the object store's bytes contain
  it) + the REPLAY returns the same id; the FAKE excerpt is the
  typed 400 (the citation validation); the unknown assessment
  kind names itself; the snapshot's + the claim's read surfaces
  list the edge.
- [x] **NO REGRESSION** — `cargo test --all` → rc=0, 55 suites;
  `bash scripts/run_pg_tests.sh` → rc=0, 18 live suites + the
  demo `ALL acceptance checks passed` (`target/pg463_guard.log`);
  `cargo clippy --all --all-targets -- -D warnings` → rc=0;
  `cargo fmt --all -- --check` → rc=0; `make gate` → 13/13 at
  commit.
- [x] **FIX** — `0030_claim_assessments.sql`, `src/claims.rs`,
  `src/api.rs`, `src/lib.rs`, `tests/profiles.rs`.
- [x] **LOCKSTEP** — CHANGELOG, DEV_NOTES, MEMORY, LIVE_STATUS, this
  tree's logs above, `docs/TASK_TREE.md` frontier, KNOWLEDGE_MAP —
  same commit.

## Acceptance Checklist (PHASE-4.6.2)

The CODE change owned by this leaf:
`migrations/0029_derivations.sql` (NEW — the `Derivation` edges),
`crates/reasonbraid-server/src/derivations.rs` (NEW — the typed
submission + the verified submit + the traversal), `src/api.rs`
(the two verbs + the R2 chunk auto-derivations), `src/lib.rs`
(the module), and `tests/profiles.rs` (profiles 20).

- [x] **REPRODUCE / ISSUE** — the `.6.1` close: the snapshots
  store the originals, but no transformation is an EDGE — the
  derived chunks float parentless.
- [x] **ROOT CAUSE (WHY + WHERE)** — the graph was the lane's
  second third — `git grep -c "derivations\|DerivationSubmission"
  62a7f0b -- crates/ migrations/` → rc=1 (nothing before this
  leaf). The fix point is the §12.6 `Derivation` edge: the own
  digest verified, the parent link, the traversal.
- [x] **ADDRESSED (verified)** — measured before→after. Before: the
  grep above. After:
  `DATABASE_URL=… cargo test -p reasonbraid-server --test
  profiles the_derivation_graph` → `test result: ok. 1 passed`
  — the edge submits (the content hashes to the declared
  digest), the REPLAY returns the same id, the children
  traversal lists the edge (the parent stays addressable), the
  digest MISMATCH is the 400, the missing parent refuses with
  its name.
- [x] **NO REGRESSION** — `cargo test --all` → rc=0, 55 suites;
  `bash scripts/run_pg_tests.sh` → rc=0, 18 live suites + the
  demo `ALL acceptance checks passed` (`target/pg462_guard.log`);
  `cargo clippy --all --all-targets -- -D warnings` → rc=0;
  `cargo fmt --all -- --check` → rc=0; `make gate` → 13/13 at
  commit.
- [x] **FIX** — `0029_derivations.sql`, `src/derivations.rs`,
  `src/api.rs`, `src/lib.rs`, `tests/profiles.rs`.
- [x] **LOCKSTEP** — CHANGELOG, DEV_NOTES, MEMORY, LIVE_STATUS, this
  tree's logs above, `docs/TASK_TREE.md` frontier, KNOWLEDGE_MAP —
  same commit.

## Acceptance Checklist (PHASE-4.6.1)

The CODE change owned by this leaf:
`migrations/0028_evidence_snapshots.sql` (NEW — the object store +
the snapshot table), `crates/reasonbraid-server/src/snapshots.rs`
(NEW — the typed submission + the verified submit + the get + the
tombstone), `src/api.rs` (the three verbs + the R0/R2/R5
auto-submits), `src/lib.rs` (the module), `Cargo.toml` (the base64
0.22 dep), and `tests/profiles.rs` (profiles 19).

- [x] **REPRODUCE / ISSUE** — the `.6` census: the receipts exist,
  NOTHING persists them (no snapshot/object store, no tombstone).
- [x] **ROOT CAUSE (WHY + WHERE)** — the store was the Phase-4
  blocker's last leg — `git grep -c "evidence_snapshots\|SnapshotSubmission"
  d7cfbc6 -- crates/ migrations/` → rc=1 (nothing before this
  leaf). The fix point is the §12.6 shape + the content-addressed
  store (the bytes under their digest) + the tombstone rule.
- [x] **ADDRESSED (verified)** — measured before→after. Before: the
  grep above. After:
  `DATABASE_URL=… cargo test -p reasonbraid-server --test
  profiles the_snapshot_store` → `test result: ok. 1 passed` —
  the submit (the base64 bytes) → the read-back (the digest, the
  byte length, the null tombstone); the REPLAY (the same
  reference + digest → the same id); the digest MISMATCH is the
  400 (the bytes hash differently — the content-addressing is
  verified, not trusted); the DELETE records the reason + the
  time and the row stays readable (the tombstone — never a
  silent disappearance). The first live run caught the FOURTH
  occurrence of the SQL-continuation doubling (the `\\\n`
  artifact in the object upsert) — fixed; the debug cluster
  (the ephemeral 55433) proved the masked-error reading.
- [x] **NO REGRESSION** — `cargo test --all` → rc=0, 55 suites;
  `bash scripts/run_pg_tests.sh` → rc=0, 18 live suites + the
  demo `ALL acceptance checks passed` (`target/pg461_guard.log`);
  `cargo clippy --all --all-targets -- -D warnings` → rc=0;
  `cargo fmt --all -- --check` → rc=0; `make gate` → 13/13 at
  commit.
- [x] **FIX** — `0028_evidence_snapshots.sql`, `src/snapshots.rs`,
  `src/api.rs`, `src/lib.rs`, `Cargo.toml`, `tests/profiles.rs`.
- [x] **LOCKSTEP** — CHANGELOG, DEV_NOTES, MEMORY, LIVE_STATUS, this
  tree's logs above, `docs/TASK_TREE.md` frontier, KNOWLEDGE_MAP —
  same commit.

## Acceptance Checklist (PHASE-4.5.3)

The CODE change owned by this leaf:
`crates/reasonbraid-server/src/resolvers.rs` (the gated ids + the
sync + the auth filter + the call field), `src/fetcher.rs` (the
authenticated fetch + the preflight), `src/browse.rs` (NEW — the
spawner + the receipt), `src/broker.rs` (the authenticated
receipt), `src/api.rs` (the gate + the branches + the seam),
`src/bin/rb-server.rs` (the startup sync), `src/lib.rs` (the
re-exports), and `tests/profiles.rs` (profiles 18).

- [x] **REPRODUCE / ISSUE** — the `.5.2` close: the machinery
  exists, compiled but UNWIRED — the gate has no rows, no
  receipts, no resolution consumption.
- [x] **ROOT CAUSE (WHY + WHERE)** — the wiring was the lane's
  last third — `git grep -c "sync_gated_entries\|AuthenticatedReceipt\|BrowserReceipt"
  929aae2 -- crates/` → rc=1 (nothing before this leaf). The fix
  point is the `.5.1` gate made structural: the startup sync
  (rows exist ONLY while open), the auth filter, the per-request
  credential, the receipts, the resolve-path branches.
- [x] **ADDRESSED (verified)** — measured before→after. Before: the
  grep above. After:
  `DATABASE_URL=… cargo test -p reasonbraid-server --test
  profiles the_gated_packs` → `test result: ok. 1 passed` — the
  gate CLOSED: the binding-carrying reference is the explicit
  unresolvable-now (the disabled pack has no row); the gate
  OPEN: the R5 pack ranks + the authenticated acquisition
  refuses the loopback with the class NAMED (the SSRF proof
  through the authenticated path), the R3 pack ranks + the
  pre-flight refuses BEFORE any worker spawn, the RX pack ranks
  + the §12.8 capability call publishes the locator; the gate
  CLOSED again: the sync REMOVES the rows and the
  unresolvable-now returns.
- [x] **NO REGRESSION** — `cargo test --all` → rc=0, 55 suites;
  `bash scripts/run_pg_tests.sh` → rc=0, 18 live suites + the
  demo `ALL acceptance checks passed` (`target/pg453_guard.log`);
  `cargo clippy --all --all-targets -- -D warnings` → rc=0;
  `cargo fmt --all -- --check` → rc=0; `make gate` → 13/13 at
  commit.
- [x] **FIX** — `src/resolvers.rs`, `src/fetcher.rs`,
  `src/browse.rs`, `src/broker.rs`, `src/api.rs`,
  `src/bin/rb-server.rs`, `src/lib.rs`, `tests/profiles.rs`.
- [x] **LOCKSTEP** — CHANGELOG, DEV_NOTES, MEMORY, LIVE_STATUS, this
  tree's logs above, `docs/TASK_TREE.md` frontier, KNOWLEDGE_MAP —
  same commit.

## Acceptance Checklist (PHASE-4.5.2)

The CODE change owned by this leaf:
`crates/reasonbraid-browse/` (NEW — the browser worker crate:
the stdio protocol, the bounded interaction, the network log, the
two wire tests), `crates/reasonbraid-server/src/broker.rs` (NEW —
the credential broker), `src/mediated.rs` (NEW — the §12.8
vocabulary), and `src/lib.rs` (the modules) — `\.rs$` +
`Cargo.toml`.

- [x] **REPRODUCE / ISSUE** — the `.5` census: NOTHING exists for
  the browser/broker/agent-mediated acquisition.
- [x] **ROOT CAUSE (WHY + WHERE)** — the lane was a greenfield —
  `git grep -c "chromiumoxide\|AcquisitionAnswer\|DisclosureRecord"
  a9cd4c6 -- crates/` → rc=1 (nothing before this leaf). The fix
  point is the `.5.1` contract's machinery: the browser worker,
  the broker, the vocabulary — all compiled but UNWIRED (the
  gate is the `.5.3` wiring's).
- [x] **ADDRESSED (verified)** — measured before→after. Before: the
  grep above. After:
  `cargo test -p reasonbraid-browse` → `test result: ok. 2
  passed` — the REAL-Chrome render of the local origin (the
  heading + the body text derived, the network log records the
  page request, the parent + chunk digests present) and the
  step-budget refusal BEFORE any navigation (the origin's
  counter stays 0); `cargo test -p reasonbraid-server --lib
  broker` → `test result: ok. 2 passed` (the redacted Debug, the
  unknown-binding name, the disclosure record) + `--lib
  mediated` → `test result: ok. 2 passed` (the six-shape
  roundtrip, the second-verifier carry).
- [x] **NO REGRESSION** — `cargo test --all` → rc=0, 55 suites;
  `bash scripts/run_pg_tests.sh` → rc=0, 18 live suites + the
  demo `ALL acceptance checks passed` (`target/pg452_guard.log`);
  `cargo clippy --all --all-targets -- -D warnings` → rc=0;
  `cargo fmt --all -- --check` → rc=0; `make gate` → 13/13 at
  commit.
- [x] **FIX** — `crates/reasonbraid-browse/`, `src/broker.rs`,
  `src/mediated.rs`, `src/lib.rs`.
- [x] **LOCKSTEP** — CHANGELOG, DEV_NOTES, MEMORY, LIVE_STATUS, this
  tree's logs above, `docs/TASK_TREE.md` frontier, KNOWLEDGE_MAP —
  same commit.

## Acceptance Checklist (PHASE-4.4.3)

The CODE change owned by this leaf:
`migrations/0027_r2_extract_worker_entry.sql` (NEW — the R2
install record), `crates/reasonbraid-server/src/extraction.rs`
(NEW — the receipt + the spawner + the test),
`src/resolvers.rs` (the R2 id + the media-type filter + the
`Extract` variant), `src/api.rs` (the pipeline branch),
`src/lib.rs` (the module), and `tests/profiles.rs` (profiles 17).

- [x] **REPRODUCE / ISSUE** — the `.4.2` close: the worker exists
  but the receipt, the install record, and the pipeline do not.
- [x] **ROOT CAUSE (WHY + WHERE)** — the pack was built in halves
  — `git grep -c "ExtractionReceipt\|r2-extract-worker"
  ede2e2a -- crates/ migrations/` → rc=1 (nothing before this
  leaf). The fix point is the Derivation receipt + the seeded
  install record + the pipeline in the resolve path (the
  `.2.3`/`.3.3` pattern).
- [x] **ADDRESSED (verified)** — measured before→after. Before: the
  grep above. After:
  `cargo test -p reasonbraid-server --lib extraction` → `test
  result: ok. 1 passed` (the spawner roundtrip against the REAL
  worker binary — the Derivation response + the surfaced
  `{error}` refusal); the live `DATABASE_URL=… cargo test -p
  reasonbraid-server --test profiles the_r2_resolver` → `test
  result: ok. 1 passed` — the hinted reference ranks
  `r2-extract-worker` under its own classes (`process`/`listed`);
  the pipeline's acquisition leg refuses the loopback with the
  class NAMED through the resolution path; the hintless reference
  keeps the R0 acquisition-only path; the stricter requirement is
  the explicit unresolvable-now.
- [x] **NO REGRESSION** — `cargo test --all` → rc=0, 53 suites;
  `bash scripts/run_pg_tests.sh` → rc=0, 18 live suites + the
  demo `ALL acceptance checks passed` (`target/pg443_guard.log`);
  `cargo clippy --all --all-targets -- -D warnings` → rc=0;
  `cargo fmt --all -- --check` → rc=0; `make gate` → 13/13 at
  commit.
- [x] **FIX** — `0027_r2_extract_worker_entry.sql`,
  `src/extraction.rs`, `src/resolvers.rs`, `src/api.rs`,
  `src/lib.rs`, `tests/profiles.rs`.
- [x] **LOCKSTEP** — CHANGELOG, DEV_NOTES, MEMORY, LIVE_STATUS, this
  tree's logs above, `docs/TASK_TREE.md` frontier, KNOWLEDGE_MAP —
  same commit.

## Acceptance Checklist (PHASE-4.4.2)

The CODE change owned by this leaf:
`crates/reasonbraid-extract/Cargo.toml` + `src/main.rs` (NEW — the
worker crate: the stdio protocol, the parsers, the refusals, the
nine unit tests) + `tests/worker_roundtrip.rs` (the stdio
roundtrips) — `\.rs$` + `Cargo.toml`.

- [x] **REPRODUCE / ISSUE** — the `.4` census: NOTHING extracts
  (no parser crate, no worker).
- [x] **ROOT CAUSE (WHY + WHERE)** — the R2 worker was a
  greenfield — `git grep -c "lopdf\|atom_syndication\|ZipArchive"
  53c62a4 -- crates/` → rc=1 (nothing before this leaf). The fix
  point is the `.4.1` contract's worker: the stdio protocol, the
  four parsers, the named refusals, the fresh-process quarantine.
- [x] **ADDRESSED (verified)** — measured before→after. Before: the
  grep above. After:
  `cargo test -p reasonbraid-extract` → `test result: ok. 9
  passed` — the PDF text extraction (the fixture built with
  lopdf's writer), the JS-bearing + unsupported-type refusals,
  the zip text extraction + the excluded binaries, the nested/
  traversal/bomb refusals (the ratio brake over the COMPRESSED
  envelope), the tar + feed extractions, the output ceiling, and
  the two stdio roundtrips SPAWNING the built binary (the
  Derivation response + the `{error: {kind, message}}` refusal).
  The first runs caught three real bugs (the fixture's missing
  xref, the page-number vs page-id argument, the ratio brake's
  wrong size) — all fixed.
- [x] **NO REGRESSION** — `cargo test --all` → rc=0, 53 suites;
  `bash scripts/run_pg_tests.sh` → rc=0, 18 live suites + the
  demo `ALL acceptance checks passed` (`target/pg442_guard.log`);
  `cargo clippy --all --all-targets -- -D warnings` → rc=0;
  `cargo fmt --all -- --check` → rc=0; `make gate` → 13/13 at
  commit.
- [x] **FIX** — `crates/reasonbraid-extract/` (the whole crate).
- [x] **LOCKSTEP** — CHANGELOG, DEV_NOTES, MEMORY, LIVE_STATUS, this
  tree's logs above, `docs/TASK_TREE.md` frontier, KNOWLEDGE_MAP —
  same commit.

## Acceptance Checklist (PHASE-4.3.3)

The CODE change owned by this leaf:
`migrations/0026_r1_git_resolver_entry.sql` (NEW — the R1 install
record), `crates/reasonbraid-server/src/git.rs` (the `GitReceipt` +
the `GitManifest` + the `git_digest` + the path collection + the
pure receipt test), `src/resolvers.rs` (the R1 id + the untagged
`Acquisition`), `src/api.rs` (the `GitFetcher` in `ApiState` + the
execution branch), and `tests/profiles.rs` (profiles 16).

- [x] **REPRODUCE / ISSUE** — the `.3.2` close: the acquisition
  exists but the receipt shape, the install record, and the
  resolution consumption do not.
- [x] **ROOT CAUSE (WHY + WHERE)** — the pack was built in halves
  — `git grep -c "GitReceipt\|r1-git-fetcher" b5a9de9 -- crates/
  migrations/` → rc=1 (nothing before this leaf). The fix point is
  the receipt over the acquired odb + the seeded install record +
  the execution in the resolve path (the `.2.3` R0 pattern).
- [x] **ADDRESSED (verified)** — measured before→after. Before: the
  grep above. After:
  `cargo test -p reasonbraid-server --lib git` → `test result:
  ok. 6 passed` (the new receipt test: the resolved commit, the
  ADR-011 `sha256:` + 64-hex digest over the acquired odb bytes,
  the chain, the included manifest paths); the live
  `DATABASE_URL=… cargo test -p reasonbraid-server --test profiles
  the_r1_resolver` → `test result: ok. 1 passed` — the git
  reference resolves to `r1-git-fetcher` under its own classes;
  the loopback literal refuses with the class NAMED through the
  resolution path; the reference stays submitted; the stricter
  requirement is the explicit unresolvable-now.
- [x] **NO REGRESSION** — `cargo test --all` → rc=0, 51 suites;
  `bash scripts/run_pg_tests.sh` → rc=0, 18 live suites + the
  demo `ALL acceptance checks passed` (`target/pg433_guard.log`);
  `cargo clippy --all --all-targets -- -D warnings` → rc=0;
  `cargo fmt --all -- --check` → rc=0; `make gate` → 13/13 at
  commit.
- [x] **FIX** — `0026_r1_git_resolver_entry.sql`, `src/git.rs`,
  `src/resolvers.rs`, `src/api.rs`, `tests/profiles.rs`.
- [x] **LOCKSTEP** — CHANGELOG, DEV_NOTES, MEMORY, LIVE_STATUS, this
  tree's logs above, `docs/TASK_TREE.md` frontier, KNOWLEDGE_MAP —
  same commit.

## Acceptance Checklist (PHASE-4.3.2)

The CODE change owned by this leaf:
`crates/reasonbraid-server/src/git.rs` (NEW — the hardened grammar,
the pre-flight, the classified transport, the acquisition, the five
tests), `crates/reasonbraid-server/src/lib.rs` (the module),
`crates/reasonbraid-server/Cargo.toml` (gix + gix-transport; reqwest
gains `blocking`), and `crates/reasonbraid-server/src/fetcher.rs`
(the belt's `pub(crate)` exposure) — `\.rs$` + `Cargo.toml`.

- [x] **REPRODUCE / ISSUE** — the `.3` census: NOTHING fetches Git
  (no git library in the lock or the cache; the §12.5 rules have
  no machinery).
- [x] **ROOT CAUSE (WHY + WHERE)** — the R1 pack was a greenfield
  — `git grep -c "gix\|git2\|gitoxide" efc9ba8 -- crates/` →
  rc=1 (nothing before this leaf). The fix point is the
  acquisition under the `.3.1` contract: the classified transport
  (the seam verified mechanically — gix's `new_http` + the `Http`
  trait), the mechanical budgets, the named refusals, the
  bare-repo no-checkout property, the resolved commit.
- [x] **ADDRESSED (verified)** — measured before→after. Before: the
  grep above. After:
  `cargo test -p reasonbraid-server --lib git` → `test result:
  ok. 5 passed` — the grammar table (the https-only + userinfo +
  numeric + selector refusals), the pre-flight loopback/private
  refusals (the class NAMED before any socket — the R1 SSRF
  proof), the acquisition over the injected file transport (the
  OFFLINE wire path: the resolved commit matches the source's,
  the file/depth/object counts measured), the submodule gitlink +
  the LFS pointer refusals with their names, the budget trips
  (file/depth/object ceilings). An API spike proved the gix
  blocking fetch flow end-to-end before the module was written
  (deleted after). The first runs caught three real bugs (the
  WHATWG numeric-host normalization, the second commit's missing
  parent, the depth ceiling skipping leaf entries) — all fixed.
- [x] **NO REGRESSION** — `cargo test --all` → rc=0, 51 suites (the sweep CAUGHT the
  third rustls-provider occurrence — the gix reqwest backend's
  plain-`rustls` aws-lc-rs default — fixed by the backend-less
  `http-client` seam; the ring-only union verified via
  `cargo tree -e features -i rustls`);
  `bash scripts/run_pg_tests.sh` → rc=0, 18 live suites + the
  demo `ALL acceptance checks passed` (`target/pg432b_guard.log`);
  `cargo clippy --all --all-targets -- -D warnings` → rc=0;
  `cargo fmt --all -- --check` → rc=0; `make gate` → 13/13 at
  commit.
- [x] **FIX** — `src/git.rs`, `src/lib.rs`, `src/fetcher.rs`,
  `crates/reasonbraid-server/Cargo.toml`.
- [x] **LOCKSTEP** — CHANGELOG, DEV_NOTES, MEMORY, LIVE_STATUS, this
  tree's logs above, `docs/TASK_TREE.md` frontier, KNOWLEDGE_MAP —
  same commit.

## Acceptance Checklist (PHASE-4.2.3)

The CODE change owned by this leaf:
`migrations/0025_r0_resolver_entry.sql` (NEW — the built-in R0
install record), `crates/reasonbraid-server/src/fetcher.rs` (the
receipt + the digest + `kind` + the pure receipt test),
`src/resolvers.rs` (the R0 id + the outcome extension),
`src/api.rs` (the built-in fetcher in `ApiState` + the execution),
`crates/reasonbraid-server/tests/profiles.rs` (the measured
resolution — profiles 15), and `Cargo.toml` (chrono gains serde).

- [x] **REPRODUCE / ISSUE** — the `.2` census: the R0 pack ships
  nothing — no receipt machinery, no registry entry, the resolve
  path returns no built-in.
- [x] **ROOT CAUSE (WHY + WHERE)** — the pack was built in halves:
  the classification (`.2.1`) + the fetcher (`.2.2`) exist, but
  the receipt shape, the install record, and the resolution
  consumption do not — `git grep -c "AcquisitionReceipt\|r0-https-fetcher"
  4b15310 -- crates/ migrations/` → rc=1 (nothing before this
  leaf). The fix point is the receipt over the ACQUIRED bytes +
  the seeded install record + the execution in the resolve path.
- [x] **ADDRESSED (verified)** — measured before→after. Before: the
  grep above. After:
  `cargo test -p reasonbraid-server --lib fetcher` → `test result:
  ok. 17 passed` (the new receipt test: the digest is the
  `sha256:` + 64-hex over the acquired bytes AND passes the §12.1
  `digest_error` validator; the chain carries the raw locator +
  every hop); the live
  `DATABASE_URL=… cargo test -p reasonbraid-server --test profiles
  the_r0_resolver` → `test result: ok. 1 passed` — the https
  reference resolves to `r0-https-fetcher` under its own classes
  (`none`/`listed`); the loopback + private literals refuse with
  the class NAMED through the resolution path (the SSRF proof
  end-to-end); the reference stays submitted; the stricter
  requirement is the explicit unresolvable-now (never a silent
  downgrade).
- [x] **NO REGRESSION** — `cargo test --all` → rc=0, 51 suites;
  `bash scripts/run_pg_tests.sh` → rc=0, 18 live suites + the
  demo `ALL acceptance checks passed` (`target/pg423b_guard.log`);
  `cargo clippy --all --all-targets -- -D warnings` → rc=0;
  `cargo fmt --all -- --check` → rc=0; `make gate` → 13/13 at
  commit.
- [x] **FIX** — `0025_r0_resolver_entry.sql`, `src/fetcher.rs`,
  `src/resolvers.rs`, `src/api.rs`, `tests/profiles.rs`,
  `Cargo.toml`.
- [x] **LOCKSTEP** — CHANGELOG, DEV_NOTES, MEMORY, LIVE_STATUS, this
  tree's logs above, `docs/TASK_TREE.md` frontier, KNOWLEDGE_MAP —
  same commit.

## Acceptance Checklist (PHASE-4-MAINT-1)

The CODE change owned by this leaf: the twelve lint repairs across
`crates/reasonbraid-server/src/api.rs`,
`src/dependence.rs`, `src/matching.rs` (+ its tests),
`src/recruitment.rs` (the `OpenCallParams` refactor + the
`CallTuple` alias), `src/resources.rs` (the `ResourceRow` alias),
and `tests/profiles.rs` — `\.rs$`.

- [x] **REPRODUCE / ISSUE** — the `.2.2` routing: `cargo clippy
  --all --all-targets -- -D warnings` → rc=101 (nine findings),
  identical at `b26f529`.
- [x] **ROOT CAUSE (WHY + WHERE)** — the recorded clippy evidence
  of the Phase-3 leaves + `.1.2` does not reproduce under the
  pinned clippy 0.1.98; the census the routing named was SHADOWED
  — the lib failure stopped the downstream targets (profiles)
  from compiling, so three more pre-existing findings surfaced
  only after the nine were fixed (twelve total).
- [x] **ADDRESSED (verified)** — measured before→after. Before:
  rc=101 at this HEAD AND at `b26f529` (the `.2.1` commit). After:
  `cargo clippy --all --all-targets -- -D warnings` → rc=0 — the
  `useless_format` to `.to_string()`, the three `type_complexity`
  sites to the `AttributePicker`/`CallTuple`/`ResourceRow`
  aliases, the `collapsible_if` to the match guard, the
  `unnecessary if let` to the `.flatten()` form, the four
  `field_reassign` test sites to the struct-update form, the
  `too_many_arguments` to the `OpenCallParams` struct, and the
  three profiles findings (`unused_mut` ×2 + `let_and_return`).
- [x] **NO REGRESSION** — `cargo test --all` → rc=0, 51 suites;
  `bash scripts/run_pg_tests.sh` → rc=0, 18 live suites + the
  demo `ALL acceptance checks passed` (`target/pgm1_guard.log`);
  `cargo fmt --all -- --check` → rc=0; `make gate` → 13/13 at
  commit.
- [x] **FIX** — `src/api.rs`, `src/dependence.rs`, `src/matching.rs`,
  `src/recruitment.rs`, `src/resources.rs`, `tests/profiles.rs`.
- [x] **LOCKSTEP** — CHANGELOG, MEMORY, LIVE_STATUS, this tree's
  logs above, `docs/TASK_TREE.md` frontier, KNOWLEDGE_MAP — same
  commit.

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
| `2026-09-07` | `PHASE-4.6.3` | `DATABASE_URL=… cargo test -p reasonbraid-server --test profiles the_claim_assessments` → `test result: ok. 1 passed` (the true excerpt accepts + the replay, the FAKE excerpt refuses, the unknown kind names itself, the two read surfaces); `cargo test --all` → rc=0, 55 suites; `bash scripts/run_pg_tests.sh` → rc=0, 18 live suites + the demo (`target/pg463_guard.log`); clippy/fmt clean; `make gate` → 13/13 | the claim-evidence graph + the citation validation; frontier → `.6.4` |
| `2026-09-07` | `PHASE-4.6.2` | `DATABASE_URL=… cargo test -p reasonbraid-server --test profiles the_derivation_graph` → `test result: ok. 1 passed` (the edge roundtrip, the replay, the traversal, the mismatch 400, the missing-parent refusal); `cargo test --all` → rc=0, 55 suites; `bash scripts/run_pg_tests.sh` → rc=0, 18 live suites + the demo (`target/pg462_guard.log`); clippy/fmt clean; `make gate` → 13/13 | the derivation graph; frontier → `.6.3` |
| `2026-09-07` | `PHASE-4.6.1` | `DATABASE_URL=… cargo test -p reasonbraid-server --test profiles the_snapshot_store` → `test result: ok. 1 passed` (the submit → the read-back, the replay, the digest-mismatch 400, the tombstone); `cargo test --all` → rc=0, 55 suites; `bash scripts/run_pg_tests.sh` → rc=0, 18 live suites + the demo (`target/pg461_guard.log`); clippy/fmt clean; `make gate` → 13/13 | the snapshot store + the tombstone; frontier → `.6.2` |
| `2026-09-07` | `PHASE-4.6` | docs-only (no code paths changed): `make gate` → 13/13 at commit | the snapshot/derivation/claim census + the contract-seam decomposition (`.6.1` the store + the tombstone → `.6.2` the graph → `.6.3` the claims + the validation → `.6.4` the retention + the freshness); frontier → `.6.1` |
| `2026-09-07` | `PHASE-4.5.3` | `DATABASE_URL=… cargo test -p reasonbraid-server --test profiles the_gated_packs` → `test result: ok. 1 passed` (the gate closed → the unresolvable-now; open → the R5 rank + the authenticated loopback refusal, the R3 rank + the pre-flight refusal, the RX rank + the capability call; closed again → the rows gone); `cargo test --all` → rc=0, 55 suites; `bash scripts/run_pg_tests.sh` → rc=0, 18 live suites + the demo (`target/pg453_guard.log`); clippy/fmt clean; `make gate` → 13/13 | the gated wiring; **`.5` COMPLETE** — frontier → `.6` |
| `2026-09-07` | `PHASE-4.5.2` | `cargo test -p reasonbraid-browse` → `test result: ok. 2 passed` (the REAL-Chrome render + the network log against the local origin; the step-budget refusal before any navigation); `cargo test -p reasonbraid-server --lib broker/mediated` → 4 passed; `cargo test --all` → rc=0, 55 suites; `bash scripts/run_pg_tests.sh` → rc=0, 18 live suites + the demo (`target/pg452_guard.log`); clippy/fmt clean; `make gate` → 13/13 | the R3/R5/RX machinery (compiled, unwired); frontier → `.5.3` |
| `2026-09-07` | `PHASE-4.5.1` | docs-only (no code paths changed): the browser census measured (`cargo add --dry-run chromiumoxide/headless_chrome` → 0.9.1/1.0.22); `make gate` → 13/13 at commit | the three contracts + the OPT-IN gate; frontier → `.5.2` |
| `2026-09-07` | `PHASE-4.5` | docs-only (no code paths changed): `make gate` → 13/13 at commit | the R3/R5/RX census + the contract-seam decomposition (`.5.1` the contracts + the opt-in gate → `.5.2` the machinery → `.5.3` the receipt + the wiring); frontier → `.5.1` |
| `2026-09-07` | `PHASE-4.4.3` | `cargo test -p reasonbraid-server --lib extraction` → `test result: ok. 1 passed` (the spawner roundtrip against the REAL worker binary: the Derivation response + the surfaced refusal); `DATABASE_URL=… cargo test -p reasonbraid-server --test profiles the_r2_resolver` → `test result: ok. 1 passed` (the hinted reference ranks the R2 built-in; the pipeline's acquisition leg names the loopback class; the hintless reference keeps the R0 path; the stricter requirement explicit); `cargo test --all` → rc=0, 53 suites; `bash scripts/run_pg_tests.sh` → rc=0, 18 live suites + the demo (`target/pg443_guard.log`); clippy/fmt clean; `make gate` → 13/13 | the R2 pack wired; **`.4` COMPLETE** — frontier → `.5` |
| `2026-09-07` | `PHASE-4.4.2` | `cargo test -p reasonbraid-extract` → `test result: ok. 9 passed` (the per-format extractions + refusals + the two stdio roundtrips spawning the built binary); `cargo test --all` → rc=0, 53 suites; `bash scripts/run_pg_tests.sh` → rc=0, 18 live suites + the demo (`target/pg442_guard.log`); `cargo clippy --all --all-targets -- -D warnings` → rc=0; `cargo fmt --all -- --check` → rc=0; `make gate` → 13/13 | the extraction worker; frontier → `.4.3` |
| `2026-09-07` | `PHASE-4.4.1` | docs-only (no code paths changed): the parser census measured (`cargo add --dry-run lopdf/pdf/zip/tar/atom_syndication` → the versions above, all pure Rust); `make gate` → 13/13 at commit | the R2 contract + the parser census; frontier → `.4.2` |
| `2026-09-07` | `PHASE-4.4` | docs-only (no code paths changed): `make gate` → 13/13 at commit | the R2 census + the contract-seam decomposition (`.4.1` the contract + the parser census → `.4.2` the workers → `.4.3` the receipt + the wiring); frontier → `.4.1` |
| `2026-09-07` | `PHASE-4.3.3` | `cargo test -p reasonbraid-server --lib git` → `test result: ok. 6 passed` (the new receipt test: the resolved commit + the ADR-011 digest over the odb + the included manifest); `DATABASE_URL=… cargo test -p reasonbraid-server --test profiles the_r1_resolver` → `test result: ok. 1 passed` (the git reference resolves to the built-in; the loopback refusal names the class through the resolution path; the reference preserved; the stricter requirement explicit); `cargo test --all` → rc=0, 51 suites; `bash scripts/run_pg_tests.sh` → rc=0, 18 live suites + the demo (`target/pg433_guard.log`); clippy/fmt clean; `make gate` → 13/13 | the R1 pack wired; **`.3` COMPLETE** — frontier → `.4` |
| `2026-09-07` | `PHASE-4.3.2` | `cargo test -p reasonbraid-server --lib git` → `test result: ok. 5 passed` (the grammar table, the pre-flight loopback/private refusals, the offline file-transport acquisition with the resolved commit + the counts, the submodule/LFS refusals, the budget trips); `cargo test --all` → rc=0, 51 suites; `bash scripts/run_pg_tests.sh` → rc=0, 18 live suites + the demo (`target/pg432_guard.log`); `cargo clippy --all --all-targets -- -D warnings` → rc=0; `cargo fmt --all -- --check` → rc=0; `make gate` → 13/13 | the R1 acquisition; frontier → `.3.3` |
| `2026-09-07` | `PHASE-4.3.1` | docs-only (no code paths changed): the library census measured (`cargo add --dry-run gix` → v0.87.1 pure Rust; `cargo add --dry-run git2` → v0.21.0 with the `openssl-sys`/`vendored-libgit2` C features); `make gate` → 13/13 at commit | the R1 contract + the library census; frontier → `.3.2` |
| `2026-09-07` | `PHASE-4.3` | docs-only (no code paths changed): `make gate` → 13/13 at commit | the R1 census + the contract-seam decomposition (`.3.1` the contract + the library census → `.3.2` the acquisition → `.3.3` the receipt + the wiring); frontier → `.3.1` |
| `2026-09-07` | `PHASE-4.2.3` | `cargo test -p reasonbraid-server --lib fetcher` → `test result: ok. 17 passed` (the receipt's digest + chain, pure); `DATABASE_URL=… cargo test -p reasonbraid-server --test profiles the_r0_resolver` → `test result: ok. 1 passed` (the https reference resolves to the built-in; the loopback + private refusals name their classes through the resolution path; the reference preserved; the stricter requirement is the explicit unresolvable-now); `cargo test --all` → rc=0, 51 suites; `bash scripts/run_pg_tests.sh` → rc=0, 18 live suites + the demo `ALL acceptance checks passed` (`target/pg423b_guard.log`); clippy/fmt clean; `make gate` → 13/13 | the R0 pack wired; **`.2` COMPLETE** — frontier → `.3` |
| `2026-09-07` | `PHASE-4-MAINT-1` | `cargo clippy --all --all-targets -- -D warnings` → rc=0 (twelve findings fixed: the routed nine + the three profiles findings the lib failure shadowed); `cargo test --all` → rc=0, 51 suites; `bash scripts/run_pg_tests.sh` → rc=0, 18 live suites + the demo (`target/pgm1_guard.log`); `cargo fmt --all -- --check` → rc=0; `make gate` → 13/13 | the clippy evidence debt repaired; frontier → `.2.3` |
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
| `PHASE-4.6.3` | `REASONBRAID-PHASE4-0025` | the claim-evidence graph + the citation validation (the excerpt must be in the bytes — citation existence alone never satisfies the gate) |
| `PHASE-4.6.2` | `REASONBRAID-PHASE4-0024` | the derivation graph (the verified edges + the replay + the traversal — a quote is never the original) |
| `PHASE-4.6.1` | `REASONBRAID-PHASE4-0023` | the snapshot store + the tombstone (the content-addressing verified, the replay, the R0/R2/R5 auto-submits) |
| `PHASE-4.6` | `REASONBRAID-PHASE4-0022` | the snapshot/derivation/claim lane decomposed at the census seams (the receipts exist, nothing persists them) |
| `PHASE-4.5.3` | `REASONBRAID-PHASE4-0021` | the gated receipt + the wiring (the startup sync, the authenticated/render/agent paths — the disabled pack has no row) — **`.5` COMPLETE** |
| `PHASE-4.5.2` | `REASONBRAID-PHASE4-0020` | the R3/R5/RX machinery (the browser worker with the real render, the broker, the §12.8 vocabulary — compiled, unwired) |
| `PHASE-4.5.1` | `REASONBRAID-PHASE4-0019` | the R3/R5/RX contracts + the OPT-IN gate (the disclosed broker, the deployment-checked browser, the §12.8 vocabulary — the decision record) |
| `PHASE-4.5` | `REASONBRAID-PHASE4-0018` | the R3/R5/RX lane decomposed at the census seams (nothing exists — the contracts/machinery/wiring are the greenfield) |
| `PHASE-4.4.3` | `REASONBRAID-PHASE4-0017` | the R2 receipt + the pack wiring (the hinted references pipeline acquire→extract — the Derivation receipt) — **`.4` COMPLETE** |
| `PHASE-4.4.2` | `REASONBRAID-PHASE4-0016` | the extraction worker (the stdio protocol + the four parsers + the named refusals — the fresh-process quarantine) |
| `PHASE-4.4.1` | `REASONBRAID-PHASE4-0015` | the R2 contract + the parser census (the Derivation-only extraction + the worker quarantine — the decision record) |
| `PHASE-4.4` | `REASONBRAID-PHASE4-0014` | the R2 lane decomposed at the census seams (nothing extracts — the contract/parser-census/worker/receipt are the greenfield) |
| `PHASE-4.3.3` | `REASONBRAID-PHASE4-0013` | the R1 receipt + the pack wiring (the git references resolve to the built-in — the refusal names the class) — **`.3` COMPLETE** |
| `PHASE-4.3.2` | `REASONBRAID-PHASE4-0012` | the R1 acquisition (gix over the classified transport — the budgets + the named refusals + the resolved commit) |
| `PHASE-4.3.1` | `REASONBRAID-PHASE4-0011` | the R1 contract + the library census (gix over the classified transport — the decision record) |
| `PHASE-4.3` | `REASONBRAID-PHASE4-0010` | the R1 lane decomposed at the census seams (nothing fetches Git — the contract/library-census/acquisition/receipt are the greenfield) |
| `PHASE-4.2.3` | `REASONBRAID-PHASE4-0009` | the snapshot receipt + the R0 pack wiring (the built-in executes through the resolve path — the refusal names the class) — **`.2` COMPLETE** |
| `PHASE-4-MAINT-1` | `REASONBRAID-PHASE4-0008` | the clippy evidence debt repaired (twelve pre-existing findings — the `-D warnings` run is green again) |
| `PHASE-4.2.2` | `REASONBRAID-PHASE4-0007` | the safe HTTPS fetcher (the hardened parse + the two-layer destination enforcement + the manual decode + the ratio brake) — the SSRF proof measured offline |
| `PHASE-4.1` | `REASONBRAID-PHASE4-0001` | the resource-reference lane decomposed at the census seams (the greenfield contract + the registry + the two unopened ADRs) |
| `PHASE-4.2.1` | `REASONBRAID-PHASE4-0006` | the destination classification + the SSRF policy (the pure §12.4 rules + the public-only evaluation) |
| `PHASE-4.2` | `REASONBRAID-PHASE4-0005` | the R0 pack decomposed at the census seams (nothing fetches — the classification/fetcher/receipt are the greenfield) |
| `PHASE-4.1.3` | `REASONBRAID-PHASE4-0004` | the resolver capability registry (the §12.2 advertise + the filter-then-rank resolution + the explicit unresolvable-now) — **`.1` COMPLETE** |
| `PHASE-4.1.2` | `REASONBRAID-PHASE4-0003` | the typed `ResourceReference` + the submission (migration 0023 + the replay/conflict immutability) |
| `PHASE-4.1.1` | `REASONBRAID-PHASE4-0002` | ADR-011 + ADR-018 accepted (the `sha256:<hex>` format + the isolation-class vocabulary — no code) |
