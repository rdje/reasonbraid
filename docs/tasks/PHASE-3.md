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
    `.1.1`. **`.1` is COMPLETE** — ADR-014 (the structural answer),
    the typed + content-addressed write surface, and the measured
    per-reader visibility enforcement ship; the profile lane feeds
    the `.2` presence lane and the `.3` two-stage matching.
    promotion: declined (the census is the leaf's recorded contract — the `.1.1`–`.1.3` children execute it).
  - ID: `PHASE-3.1.1`
    Status: `done`
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
    Done (`2026-09-07`): ADR-014 accepted (evidence-gated) —
      `docs/adr/014-directory-semantic-index.md`: the §10.3 stage-1
      eligibility is STRUCTURAL by the roadmap's own table and the
      shipped authority machinery evaluates it deterministically; the
      embedding engine arrives behind its trigger (the first measured
      under-selection on a non-loopback corpus) in shadow mode —
      never an authorization control; the `.1.2` profile schema
      therefore carries NO embedding columns. No code changed.
      Frontier → `.1.2`.
    Acceptance: ADR-014 accepted (the dev answer named; the embedding
      trigger named); no code changes.

  - ID: `PHASE-3.1.2`
    Status: `done`
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
    Done (`2026-09-07`): the write surface landed — migration 0019
      (`agent_profiles` + `profile_versions`: one CURRENT pointer per
      role, the content-addressed history — every write is a new
      version, the hash is server-computed over the typed profile,
      the writer is the actor handle) + `crates/reasonbraid-server/
      src/profiles.rs` (the typed §10.1 `AgentProfile`: the
      capabilities with the four provenance classes, the
      interests/scopes/availability/ceilings, the per-field
      visibility policy, the grants-by-reference, the incarnation
      lineage) + the verbs: `PUT /v1/profiles/{role_id}` (ONLY the
      role itself writes — a self-declaration), `POST
      /v1/profiles/{role_id}/attest` (tenant_admin-audited: the
      claim's provenance upgrades to `owner_attested` with the
      evidence), the self/owner reads + the version history. The
      measured suite (`tests/profiles.rs`, in the guard — 18 live
      suites now) proves: identical content hashes identically,
      changed content versions anew, the old versions stay readable,
      a stranger (the owner AND a sibling role) cannot write or read,
      a dangling lineage reference fails closed, an unknown field is
      a typed 422, and the attestation is audited. The migration's FK
      ripple updated every tenant-purging suite's purge list (the
      spend_breakers lesson, again). Frontier → `.1.3`.
    Acceptance: the schema + the versioned writes land, measured; no
      regression.

  - ID: `PHASE-3.1.3`
    Status: `done`
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
    Done (`2026-09-07`): the visibility enforcement landed —
      `filter_profile` in `profiles.rs` (the per-field policy
      evaluated against the reader's class; a hidden field is ABSENT,
      never nulled; the claim provenance rides every visible
      capability) + the per-reader classification in `api.rs` (the
      role itself → FULL; the tenant owner → FULL via the AUDITED
      tenant-admin check; a same-tenant principal → TENANT; any other
      enrolled principal → NETWORK; an unenrolled principal reads
      nothing) + the response names the applied class
      (`visibility: full|tenant|network` — the read decision is
      deterministic + self-describing). The version history stays
      FULL-only (the past versions may carry fields later
      reclassified). Measured (`tests/profiles.rs` grew to 5): the
      SAME profile read by four readers yields exactly the allowed
      field sets each — the tenant view absents the self-only fields,
      the network view absents the tenant+self fields, and the
      history refuses the stranger. Frontier → `.2`.
    Acceptance: the per-reader filtering is measured (the adversarial
      reads: the same profile read by the owner, a tenant sibling,
      and a stranger yields exactly the allowed fields each); no
      regression.

- ID: `PHASE-3.2`
  Status: `done`
  Goal: lease-based presence; offline-known distinction; privacy-filtered views
  Backlog: 27
  Roadmap: §10.2
  Children: `.2.1`–`.2.3` (decomposed `2026-09-07` at the census
    seams): `.2.1` the presence state machine (the six states DERIVED
    from the lease clock + the suspension + the profile's availability
    class) → `.2.2` the offline-known distinction + the stale
    handling (the directory's offline-known row, the honest unknown)
    → `.2.3` the privacy-filtered directory views (counts /
    pseudonyms / nothing per the initiator's scope).
  Done (`2026-09-07`): the census mapped §10.2 against the shipped
    surface: the lease store (0009) + the presence view (0012/0017)
    + the channel's one-node presence endpoint exist — but the
    response carries only `online` + `suspended` + the clock fields
    (`grep -rn "draining\|busy\|offline_known" crates/` → no state
    machine anywhere), the offline-KNOWN row (an enrolled node whose
    lease expired) is indistinguishable from an unknown node at the
    directory level, and the privacy-filtered views (counts /
    pseudonyms / no roster per the initiator's scope) do not exist.
    Children at those seams — frontier → `.2.1`. **`.2` is COMPLETE** —
    the six-state derivation, the offline-known distinction + the
    operator's enumeration, and the per-scope privacy-filtered
    directory views ship; the `.3` two-stage matching consumes the
    presence + the filtered profile views.
  - ID: `PHASE-3.2.1`
    Status: `done`
    Goal: the presence state machine — the six §10.2 states as a
      DETERMINISTIC derivation: `available` (a live lease + the
      profile's availability class admits work), `offline` (the
      lease expired), `suspended` (a revoked cert — the 0017 view),
      `unknown` (no enrollment), `busy`/`draining` (the concurrency
      accounting — NAMED with their trigger: the `.4` capacity
      reservations lane). Presence never changes enrollment (the
      derivation reads, it never writes). The pure
      `presence_state(lease, suspension, availability)` function +
      the offline tests; the channel's presence response gains the
      derived state.
    Backlog: 27 (the state machine half)
    Done (`2026-09-07`): the state machine landed —
      `crates/reasonbraid-server/src/presence.rs`: the six §10.2
      states as the deterministic `presence_state(enrolled,
      suspended, lease_live, concurrency)` with the honesty
      precedence (unknown is never fabricated; suspension outranks
      the lease; the expired lease reads offline; zero declared
      concurrency drains; `busy` is the named `.4` trigger — no
      input feeds it yet). The channel's presence response gains the
      derived `state` (the profile's declared concurrency feeds the
      derivation through the current profile version); presence reads
      only — it never changes enrollment. Measured: the five pure
      derivation tests + the live offline/available assertions in the
      node_channel suite; the demo's presence checks unchanged.
      Frontier → `.2.2`.
    Acceptance: the derivation is pure + tested; the response names
      the state; presence does not change enrollment; no regression.

  - ID: `PHASE-3.2.2`
    Status: `done`
    Goal: the offline-known distinction + the stale handling — the
      directory's OFFLINE-KNOWN row (an enrolled node whose lease
      expired reads `offline` WITH its expiry visible — the
      operator's "this node is known, just quiet"), the honest
      `unknown` (an unenrolled node id is the typed 404, never a
      fabricated offline), the stale-lease handling (the heartbeat's
      expiry window + the re-handshake path re-leases). The
      offline-delivery expiry + max age are the `.5` lane's (named,
      not built here).
    Backlog: 27 (the distinction half)
    Done (`2026-09-07`): the distinction landed + is measured —
      `GET /v1/admin/nodes/presence?tenant_id=` (tenant_admin-gated,
      the `.3.2.2` enumeration): every enrolled node with its derived
      state + the lease clock — the operator's offline-KNOWN rows
      (an expired-lease node reads `offline` WITH its past expiry
      visible; a never-leased node reads `offline` with null clocks);
      the unknown id stays the typed `unknown_node` 404, never a
      fabricated offline. The stale handling was already measured by
      the `.2.2`-era fencing test (the expired lease's heartbeat is
      refused; only a fresh handshake re-leases) — the leaf pins it
      as the distinction's third leg. The test
      (`the_offline_known_distinction_and_the_operator_enumeration`,
      node_channel 23) runs on every guard pass. The
      offline-delivery expiry + max age stay the `.5` lane's (named).
      Frontier → `.2.3`.
    Acceptance: the offline-known vs unknown distinction is measured
      (the expired-lease node reads offline-with-expiry; the unknown
      id is the typed 404); no regression.

  - ID: `PHASE-3.2.3`
    Status: `done`
    Goal: the privacy-filtered directory views — the §10.2 rule
      "counts, pseudonyms, or no roster at all according to the
      initiator's scope": `GET /v1/directory/presence` returns per
      the initiator's own scope + each profile's visibility policy —
      the tenant view lists the tenant's nodes with their
      tenant-visible fields, the network view lists only the
      network-visible pseudonyms (or the count alone when the policy
      says so), and a zero-visibility profile contributes nothing
      (not even a count). The `.1.3` filter is the field-level
      engine.
    Backlog: 27 (the views half)
    Done (`2026-09-07`): the privacy-filtered directory views landed
      — `GET /v1/directory/presence` (the reader's identity decides
      everything, no params): the OWNER reads their tenant's nodes
      with the FULL fields (the audited tenant-admin path), a tenant
      member the TENANT-filtered fields, every enrolled principal
      the network pseudonyms of the other tenants (each with the
      network-visible fields + the derived state), and a
      zero-visibility profile contributes NOTHING — not even a
      count. The `.1.3` filter is the field-level engine. Measured
      (`the_directory_reads_yield_the_allowed_shapes_per_reader`,
      profiles 6): the three scopes yield exactly the allowed shapes
      — the owner's own view carries the self-only fields, the
      member's view absents them, the stranger sees only the
      network pseudonyms, and the zero-visibility profile is absent
      everywhere. **`.2` COMPLETE** — frontier → `.3`.
    Acceptance: the three scopes are measured (the same directory
      read by a tenant member, a stranger, and the owner yields the
      allowed shapes); no regression.

- ID: `PHASE-3.3`
  Status: `done`
  Goal: two-stage matching — deterministic eligibility then explainable ranking
  Backlog: 28, 29
  Roadmap: §10.3
  Children: `.3.1`–`.3.3` (decomposed `2026-09-07` at the census
    seams): `.3.1` the eligibility expression + the stage-1
    evaluation (the typed §10.3 stage-1 fields, resolved
    server-side) → `.3.2` the stage-2 explainable ranking (the
    deterministic features, each with its source + contribution +
    a visibility-safe explanation; the semantic slot stays EMPTY per
    ADR-014) → `.3.3` the matching query surface (the initiator
    submits the expression; the response is the eligible + ranked
    list filtered per the reader's scope).
  Done (`2026-09-07`): the census mapped §10.3 against the shipped
    surface: the matching INPUTS exist (the `.1` profiles with the
    provenance + the interests/scopes/ceilings, the `.2` derived
    presence, the grants/budget facts from Phase 2) but NO matching
    machinery — `grep -rn "eligible\|ranking" crates/…/src/` → only
    an unrelated state-machine comment (no expression, no evaluator,
    no scoring, no query surface). Children at those seams —
    frontier → `.3.1`.
  - ID: `PHASE-3.3.1`
    Status: `done`
    Goal: the eligibility expression + the stage-1 evaluation — the
      typed expression over the §10.3 stage-1 fields (the visibility
      scope, the capability requirements, the policy restrictions,
      the confidentiality classes, the concurrency + budget
      availability, the explicit exclusions) + the PURE evaluator
      over the shipped facts (the `.1` profile, the `.2` derived
      presence state, the grants + the budget ceilings) —
      `eligible(expression, profile, presence, grants, budget) ->
      verdict + reasons`. Resolved SERVER-side (§10.2: the caller
      need not know the membership size); an ineligible role is
      never restored by ranking (ADR-014's ordering).
    Backlog: 29 (the policy-filter half)
    Done (`2026-09-07`): the stage-1 evaluation landed —
      `crates/reasonbraid-server/src/matching.rs`: the typed
      `EligibilityExpression` (the §10.3 stage-1 fields with the
      least-restrictive defaults; unknown fields are typed
      rejections) + the pure `eligible(expression, candidate)` with
      the honesty order — the explicit exclusion, the presence gate,
      the VISIBILITY-SCOPED checks (every capability/interest/
      confidentiality requirement reads the profile filtered at the
      expression's scope: a tenant-hidden capability cannot satisfy
      a network-scope requirement), the declared-concurrency gate,
      the hard-budget gate — and every decision (either way) carries
      the named reasons. Six unit tests measure the gates (the
      provenance refusal, the visibility refusal, the presence
      refusal, the exclusion, the concurrency + budget shortfalls,
      the happy path with the reasons). Frontier → `.3.2`.
    Acceptance: the expression parses typed; the evaluator is pure +
      tested (each stage-1 field has a named reason); no regression.

  - ID: `PHASE-3.3.2`
    Status: `done`
    Goal: the stage-2 explainable ranking — the deterministic feature
      scores (the exact capability match, the subscription/interest
      match, the domain affinity, the latency class, the workload
      balance) each with its source + version + contribution + a
      visibility-safe explanation (no raw profile fields leak into
      the explanation); the dependence-indicator slot is the `.6`
      lane's (named, not computed); the semantic slot stays EMPTY
      per ADR-014. The ranking never controls authorization (the
      stage-1 verdict does).
    Backlog: 28 (the metadata-prefilter half), 29 (the scoring half)
    Done (`2026-09-07`): the stage-2 ranking landed in `matching.rs`
      — `RankingPreferences` (the five weights, all 1.0 by default),
      the five deterministic features each as a `FeatureScore` with
      its score + contribution + a VISIBILITY-SAFE explanation (the
      explanations name the counts + the initiator's own inputs +
      the matched facts visible at the expression's scope — never a
      hidden profile field), and the pure `rank(...)`: the weighted
      total over the ELIGIBLE set only (the ranking never restores
      an ineligible role — the stage-1 verdict does), ties broken by
      the role id. The expression gained the stage-2 inputs (the
      domains + the preferred latency class). Four new unit tests
      measure the ordering, the zero-weight contribution, the
      ineligible exclusion, and the determinism (10 in the module).
      Frontier → `.3.3`.
    Acceptance: each feature's explanation is source-tagged; the
      ranking is a pure function of the eligible set; no regression.

  - ID: `PHASE-3.3.3`
    Status: `done`
    Goal: the matching query surface — `POST /v1/directory/match`
      (the initiator submits the eligibility expression + the
      preferences; the response is the eligible + ranked candidate
      list, each with the stage-1 reasons + the stage-2
      explanations, filtered per the reader's scope — the `.2.3`
      directory rules: the zero-visibility profiles never appear,
      the hidden fields never ride the explanation). The `.4` lane
      owns the recruitment + capacity reservations that CONSUME
      this list.
    Backlog: 28, 29 (the surface half)
    Done (`2026-09-07`): the query surface landed — `POST
      /v1/directory/match` (the expression + the preferences; the
      server resolves the eligibility + the ranking over the shipped
      facts — the caller never enumerates the network): the
      expression's scope is CLAMPED to the reader's classification
      (a non-owner demanding the full scope is the typed 403), the
      candidates' profiles ride the response filtered at the
      reader's class, the zero-visibility profiles never appear, and
      the stage-1 reasons + the stage-2 explanations ride every
      candidate. The leaf also closed a provenance gap the test
      exposed: a role's own write may declare only SELF-ASSERTED
      claims (the upgrades ride the audited attest verb — the forged
      upgrade is the typed 400). Measured (`the_match_query_…`,
      profiles 7): the ranked candidates, the affinity separation,
      the clamp, the gate. **`.3` COMPLETE** — frontier → `.4`.
    Acceptance: the measured query (the initiator sees only the
      allowed candidates with the allowed fields); no regression.

- ID: `PHASE-3.4`
  Status: `done`
  Goal: recruitment protocol, capacity reservations, invitation fairness, anti-storm, privacy-safe explanations
  Backlog: 29, 30
  ADR: 015
  Roadmap: §10.5, §10.7
  Children: `.4.1`–`.4.3` (decomposed `2026-09-07` at the census
    seams): `.4.1` ADR-015 (the recruitment policy baseline — the
    shipped explicit-invitation contract promotes) → `.4.2` the call
    artifact + the typed recruitment responses (the §10.5
    vocabulary + the panel snapshot + the selection explanation) →
    `.4.3` the storm controls (the §10.7 items, built or named).
  Done (`2026-09-07`): the census mapped §10.5/§10.7 against the
    shipped surface: the `.1.3` invitation flow is the EXPLICIT
    baseline (invite → accept/decline/remove + dispatch-on-accept),
    the `.3` match surface supplies the candidates — but the §10.5
    response vocabulary (`join`/`observe`/`defer`/`conditional_join`/
    `recommend`/`request_context`/`recuse`) has NO typed shape
    (`grep -rn "conditional_join\|recuse" crates/` → only the
    state-machine comment), no call artifact exists (the spec's
    window/deadline/min-max/slots), the panel snapshot + the
    selection explanation do not exist, and the §10.7 storm controls
    have nothing (no fan-out limits, no call expiry, no depth/cycle
    machinery); ADR-015 is unopened. Children at those seams —
    frontier → `.4.1`.
  - ID: `PHASE-3.4.1`
    Status: `proposed`
    Goal: ADR-015 — the recruitment policy baseline: the shipped
      explicit-invitation contract (`.1.3`: the invite →
      accept/decline/remove + dispatch-on-accept) PROMOTES as the
      recruitment baseline (the human names the participants; the
      matching lane's candidates feed the OPEN calls); the
      dependence indicators are the `.6` lane's input (the ADR names
      the trigger); the §10.5 responses + the §10.7 storm rules pin
      the vocabulary the `.4.2`/`.4.3` children build. No code.
    Backlog: 29, 30 (the ADR half)
    ADR: 015
    Acceptance: ADR-015 accepted (the baseline named; the
      dependence-indicator trigger named); no code changes.

  - ID: `PHASE-3.4.2`
    Status: `proposed`
    Goal: the call artifact + the typed recruitment responses — the
      §10.5 call spec (the eligibility expression + the audience +
      the min/max participants + the role slots + the advertisement
      window + the join deadline + the expiry + the
      recommendations-allowed flag) as a durable artifact
      (migration), the typed response vocabulary (`join` | `observe`
      | `decline(reason)` | `defer(until)` | `conditional_join`
      | `recommend` | `request_context` | `recuse`), and the panel
      snapshot + the selection explanation (the server records what
      it chose + why — the `.3` explanations ride).
    Backlog: 29, 30 (the protocol half)
    Acceptance: the call artifact + the responses land typed; the
      panel snapshot carries the explanation; measured; no
      regression.

  - ID: `PHASE-3.4.3`
    Status: `proposed`
    Goal: the storm controls — the §10.7 items at the dev profile's
      scale: the per-tenant/initiator/node/role/topic/thread fan-out
      limits, the call expiry + the max offline backlog, the
      duplicate-thread suggestions (without information leakage),
      the parent/causation chains + the max autonomous depth, the
      cycle detection, the per-origin + global call circuit
      breakers, and the quiet hours — each BUILT at the dev scale or
      NAMED with its trigger (the emergency broadcast authority is
      a named deferral: no emergency class exists yet).
    Backlog: 30 (the storm half)
    Acceptance: each §10.7 item is built-and-measured or named with
      its trigger; no regression.

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
| 1 | `PHASE-3.4.1` | `proposed` | `.4` decomposed at the census seams (the explicit-invitation baseline + the match candidates exist; the responses vocabulary, the call artifact, and the storm controls do not); ADR-015 executes now |

## Changelog

- `2026-09-05`: Created from `ROADMAP.md` §20.5, §10, backlog 26–30.
- `2026-09-07`: Opened by the Phase-2 close (stable identity, inbox,
  and grants — the tree's blocker — shipped); `.1` decomposed at the
  census seams (the §10.1 profile is a greenfield: no table, no
  verbs, no visibility policy, ADR-014 unopened); children `.1.1`
  (ADR-014) → `.1.2` (the schema + writes) → `.1.3` (the visibility
  enforcement + reads); frontier → `.1.1`.
- `2026-09-07`: `.1.1` done — ADR-014 accepted (the deterministic
  eligibility stage is structural and shipped; the embedding engine
  waits for the first measured under-selection, in shadow mode); no
  code changed; frontier → `.1.2`.
- `2026-09-07`: `.1.2` done — the profile schema + the write
  surface: migration 0019 (the content-addressed version history) +
  the typed §10.1 `AgentProfile` + the write/attest/read verbs +
  the measured suite (3 tests; the guard grew to 18 live suites);
  the FK ripple updated every tenant-purging purge list; frontier →
  `.1.3`.
- `2026-09-07`: `.1.3` done — the per-reader visibility enforcement
  (the four-reader measurement, the absent-not-nulled rule, the
  self-describing class, the full-only history); the profile suite
  grew to 5; **`.1` is COMPLETE** — frontier → `.2`.
- `2026-09-07`: `.2` decomposed at the census seams — the lease
  store + the presence view + the one-node endpoint exist (the
  Phase-1/2 forms); the six-state machine, the offline-known row,
  and the privacy-filtered views are the gaps; children `.2.1` (the
  state machine) → `.2.2` (the offline-known + stale handling) →
  `.2.3` (the filtered views); frontier → `.2.1`.
- `2026-09-07`: `.2.1` done — the presence state machine (the six
  states derived with the honesty precedence; the response names the
  state; `busy` is the named `.4` trigger); five pure tests + the
  live offline/available legs; frontier → `.2.2`.
- `2026-09-07`: `.2.2` done — the offline-known distinction + the
  operator's enumeration (`GET /v1/admin/nodes/presence`: the
  offline-KNOWN rows with their expiries; the unknown id stays the
  typed 404; the stale handling pinned as the third leg); the
  node_channel suite grew to 23; frontier → `.2.3`.
- `2026-09-07`: `.2.3` done — the privacy-filtered directory views
  (`GET /v1/directory/presence`: the owner's full own-tenant view,
  the member's tenant-filtered view, the network pseudonyms, the
  zero-visibility rule); the profiles suite grew to 6;
  **`.2` COMPLETE** — frontier → `.3`.
- `2026-09-07`: `.3` decomposed at the census seams — the matching
  INPUTS exist (the profiles, the presence, the grants/budget) but
  no expression/evaluator/scoring/surface (the grep found only an
  unrelated comment); children `.3.1` (the eligibility expression +
  the stage-1 evaluation) → `.3.2` (the stage-2 explainable
  ranking) → `.3.3` (the matching query surface); frontier →
  `.3.1`.
- `2026-09-07`: `.3.1` done — the eligibility expression + the
  stage-1 evaluation (the typed expression, the pure evaluator with
  the visibility-scoped checks, the named reasons); six unit tests;
  frontier → `.3.2`.
- `2026-09-07`: `.3.2` done — the stage-2 explainable ranking (the
  five weighted features with the visibility-safe explanations, the
  eligible-only ordering, the deterministic tie-break); the module's
  tests grew to 10; frontier → `.3.3`.
- `2026-09-07`: `.3.3` done — the matching query surface (`POST
  /v1/directory/match`: the scope clamp, the reader-filtered
  candidates, the reasons + explanations) + the provenance gate (a
  role's own write is self-asserted-only — the upgrades ride the
  attest verb); the profiles suite grew to 7; **`.3` COMPLETE** —
  frontier → `.4`.
- `2026-09-07`: `.4` decomposed at the census seams — the `.1.3`
  invitation flow + the `.3` match surface are the inputs; the
  §10.5 response vocabulary, the call artifact, the panel snapshot,
  and the §10.7 storm controls are the gaps (ADR-015 unopened);
  children `.4.1` (ADR-015) → `.4.2` (the call + the responses) →
  `.4.3` (the storm controls); frontier → `.4.1`.

## Acceptance Checklist (PHASE-3.3.3)

The CODE change owned by this leaf: `crates/reasonbraid-server/src/
api.rs` (the `POST /v1/directory/match` handler + the route + the
scope clamp + the self-asserted-only write gate in `put_profile`),
`crates/reasonbraid-server/src/matching.rs` (the unknown-budget
honesty + the `RankingPreferences` serde), and
`crates/reasonbraid-server/tests/profiles.rs` (the measured match
query) — `\.rs$` in `.doctrine/code_paths.txt`.

- [x] **REPRODUCE / ISSUE** — the `.3` census: the evaluator + the
  ranking exist as pure functions but no SURFACE resolves them
  server-side, and the test design exposed a provenance gap: a
  role's own write could self-declare `owner_attested`/`certified`
  claims (the §10.1 provenance is shown, never self-granted).
- [x] **ROOT CAUSE (WHY + WHERE)** — no route wired the evaluator —
  `git grep -c "directory/match" 56a38d3 -- crates/` → rc=1 (no
  surface before this leaf); and the `.1.2` write accepted any
  confidence value (the gate lived only in the attest verb). The
  fix points: the POST surface with the scope clamp (an
  expression's scope may not exceed the reader's classification),
  and the write gate (a role's own write is self-asserted-only —
  the upgrades ride the audited attest).
- [x] **ADDRESSED (verified)** — measured before→after. Before: the
  grep above + the un-gated write. After:
  `DATABASE_URL=postgres://postgres@127.0.0.1:55432/reasonbraid_test
  cargo test -p reasonbraid-server --test profiles the_match` →
  `test result: ok. 1 passed` — the owner's tenant-scope expression
  returns the two eligible candidates ranked (the affinity
  separates them), the zero-visibility profile never appears, the
  stage-1 reasons ride, the stranger's `full`-scope demand is the
  typed 403, and the forged `certified` self-declaration is the
  typed 400 naming the gate. The first run caught the offline
  default (the drill's lease-less nodes needed the explicit
  presence widening) — the test now exercises the gate's widening.
- [x] **NO REGRESSION** — `bash scripts/run_pg_tests.sh` → 18 live
  suites + the demo `ALL acceptance checks passed` 34/34
  (`target/pg333_guard.log`); `cargo test --all` → 51 offline
  suites green; `cargo clippy --all --all-targets -- -D warnings` →
  clean; `cargo fmt --all -- --check` → rc=0; `make gate` → 13/13 at
  commit.
- [x] **FIX** — `src/api.rs` (the handler + the clamp + the write
  gate), `src/matching.rs` (the unknown-budget honesty + the serde),
  `tests/profiles.rs` (the measurement).
- [x] **LOCKSTEP** — CHANGELOG, DEV_NOTES, MEMORY, LIVE_STATUS, this
  tree's logs below, `docs/TASK_TREE.md` frontier, KNOWLEDGE_MAP —
  same commit.

## Acceptance Checklist (PHASE-3.3.2)

The CODE change owned by this leaf: `crates/reasonbraid-server/src/
matching.rs` (the `RankingPreferences`, the five `FeatureScore`
features, the pure `rank`, the expression's stage-2 inputs, the four
new unit tests) — `\.rs$` in `.doctrine/code_paths.txt`.

- [x] **REPRODUCE / ISSUE** — the `.3` census: the matching inputs
  exist but no scoring — §10.3's stage-2 (the exact match, the
  affinity, the latency, the balance, the explanations) has no
  typed shape.
- [x] **ROOT CAUSE (WHY + WHERE)** — the stage-1 evaluator
  (`.3.1`) had no stage-2 counterpart — `git grep -c
  "RankingPreferences\|FeatureScore" 752af74 -- crates/` → rc=1
  (nothing before this leaf). The fix point is a pure weighted
  ranking over the ELIGIBLE set only, every feature reading the
  profile filtered at the expression's scope, with the
  explanations naming only the initiator's inputs + the matched
  VISIBLE facts (a hidden field never leaks into an explanation).
- [x] **ADDRESSED (verified)** — measured before→after. Before: the
  grep above. After:
  `cargo test -p reasonbraid-server --lib matching` → `test result:
  ok. 10 passed` — the full match ranks first (the affinity
  separates two eligible candidates); a zero-weight feature
  contributes exactly nothing; an INELIGIBLE role never appears in
  the ranking; the tie-break is the deterministic role-id order;
  and the explanations carry no hidden field (the leak assertion).
- [x] **NO REGRESSION** — `bash scripts/run_pg_tests.sh` → 18 live
  suites + the demo `ALL acceptance checks passed` 34/34
  (`target/pg332_guard.log`); `cargo test --all` → 51 offline
  suites green; `cargo clippy --all --all-targets -- -D warnings` →
  clean; `cargo fmt --all -- --check` → rc=0; `make gate` → 13/13 at
  commit.
- [x] **FIX** — `src/matching.rs` (the ranking machinery + the
  expression's `domains`/`preferred_latency` + the four tests).
- [x] **LOCKSTEP** — CHANGELOG, DEV_NOTES, MEMORY, LIVE_STATUS, this
  tree's logs below, `docs/TASK_TREE.md` frontier, KNOWLEDGE_MAP —
  same commit.

## Acceptance Checklist (PHASE-3.3.1)

The CODE change owned by this leaf:
`crates/reasonbraid-server/src/matching.rs` (NEW — the typed
`EligibilityExpression` + the pure `eligible` evaluator + the six
unit tests), `crates/reasonbraid-server/src/profiles.rs` (the
`ClaimConfidence::rank_name` + the serde on `ReaderClass`),
`crates/reasonbraid-server/src/lib.rs` (the module) — `\.rs$` in
`.doctrine/code_paths.txt`.

- [x] **REPRODUCE / ISSUE** — the `.3` census: the matching inputs
  exist but no expression/evaluator/scoring/surface — §10.3's
  stage-1 gate has no typed shape an initiator can express.
- [x] **ROOT CAUSE (WHY + WHERE)** — no matching machinery existed
  at all — `git grep -c "eligible\|EligibilityExpression"
  c6c49c9 -- crates/` → 1 match, the core state machine's §8.4
  comment (no machinery, rc=0). The fix point is the PURE stage-1
  evaluation: the typed expression +
  the deterministic evaluator whose every check reads the profile
  AS VISIBLE AT THE EXPRESSION'S SCOPE (a hidden field satisfies
  nothing) — an ineligible role is never restored by ranking.
- [x] **ADDRESSED (verified)** — measured before→after. Before: the
  grep above. After:
  `cargo test -p reasonbraid-server --lib matching` → `test result:
  ok. 6 passed` — a self-asserted claim fails a benchmarked
  requirement (the provenance gate); a tenant-hidden capability
  cannot satisfy a network-scope requirement (the visibility gate);
  an offline candidate refuses the default expression; an excluded
  role refuses regardless; the concurrency + budget gates refuse
  shortfalls; the happy path carries the named positive reasons.
- [x] **NO REGRESSION** — `bash scripts/run_pg_tests.sh` → 18 live
  suites + the demo `ALL acceptance checks passed` 34/34
  (`target/pg331_guard.log`); `cargo test --all` → 51 offline
  suites green; `cargo clippy --all --all-targets -- -D warnings` →
  clean; `cargo fmt --all -- --check` → rc=0; `make gate` → 13/13 at
  commit.
- [x] **FIX** — `src/matching.rs` (the expression + the evaluator +
  the tests), `src/profiles.rs` (the rank names + the serde),
  `src/lib.rs`.
- [x] **LOCKSTEP** — CHANGELOG, DEV_NOTES, MEMORY, LIVE_STATUS, this
  tree's logs below, `docs/TASK_TREE.md` frontier, KNOWLEDGE_MAP —
  same commit.

## Acceptance Checklist (PHASE-3.2.3)

The CODE change owned by this leaf: `crates/reasonbraid-server/src/
api.rs` (the `GET /v1/directory/presence` handler + the route — the
per-reader scopes over the presence view + the `.1.3` filter) and
`crates/reasonbraid-server/tests/profiles.rs` (the three-scope
measurement + the node-enrollment helper; the suite's TestServer
merges the node router) — `\.rs$` in `.doctrine/code_paths.txt`.

- [x] **REPRODUCE / ISSUE** — the §10.2 rule ("counts, pseudonyms,
  or no roster at all according to the initiator's scope") has no
  surface: the one-node presence + the admin enumeration exist, but
  no per-scope directory view consumes the `.1.3` visibility
  policies.
- [x] **ROOT CAUSE (WHY + WHERE)** — the visibility filter (`.1.3`)
  evaluated single profiles only — `git grep -c
  "directory/presence" 463566e -- crates/` → rc=1 (no directory
  surface existed before this leaf). The fix point is the READER's
  classification applied to the whole directory: the owner's scope
  (FULL own-tenant fields via the audited admin path), the member's
  scope (TENANT-filtered), the network pseudonyms (NETWORK-filtered,
  and a zero-visibility profile contributes NOTHING).
- [x] **ADDRESSED (verified)** — measured before→after. Before: the
  grep above. After:
  `DATABASE_URL=postgres://postgres@127.0.0.1:55432/reasonbraid_test
  cargo test -p reasonbraid-server --test profiles the_directory`
  → `test result: ok. 1 passed` — the owner's own view carries the
  self-only fields; the member's view absents them while keeping
  the tenant-scoped capabilities; the stranger sees only the
  network pseudonyms (the tenant-scoped fields absent); and the
  zero-visibility profile is absent from EVERY network view (not
  even a count). The first run caught the suite's missing node
  router (the enroll route lives there) — merged.
- [x] **NO REGRESSION** — `bash scripts/run_pg_tests.sh` → 18 live
  suites + the demo `ALL acceptance checks passed` 34/34
  (`target/pg323_guard.log`); `cargo test --all` → 51 offline
  suites green; `cargo clippy --all --all-targets -- -D warnings` →
  clean; `cargo fmt --all -- --check` → rc=0; `make gate` → 13/13 at
  commit.
- [x] **FIX** — `src/api.rs` (the handler + the route),
  `tests/profiles.rs` (the measurement + the helper + the merged
  router).
- [x] **LOCKSTEP** — CHANGELOG, DEV_NOTES, MEMORY, LIVE_STATUS, this
  tree's logs below, `docs/TASK_TREE.md` frontier, KNOWLEDGE_MAP —
  same commit.

## Acceptance Checklist (PHASE-3.2.2)

The CODE change owned by this leaf: `crates/reasonbraid-server/src/
api.rs` (the `GET /v1/admin/nodes/presence` enumeration — the
tenant_admin-gated offline-known rows with the derived state + the
lease clock) and `crates/reasonbraid-server/tests/node_channel.rs`
(the measured three-way distinction test) — `\.rs$` in
`.doctrine/code_paths.txt`.

- [x] **REPRODUCE / ISSUE** — the `.2` census: the one-node presence
  endpoint carries the raw clock but the operator has NO tenant-wide
  enumeration — the offline-KNOWN rows ("this node is known, just
  quiet") cannot be listed, and the unknown-vs-offline distinction
  is asserted nowhere as one measured surface.
- [x] **ROOT CAUSE (WHY + WHERE)** — presence was one-node-only and
  clock-raw — `git grep -c "admin/nodes/presence" 8db3bd3 --
  crates/` → rc=1 (no enumeration existed before this leaf). The
  fix point is the operator's tenant-scoped enumeration over the
  same `node_presence` view + the `.2.1` derivation, with the
  distinction pinned by one measured test (the never-leased node,
  the expired-lease node, the typed unknown).
- [x] **ADDRESSED (verified)** — measured before→after. Before: the
  grep above. After:
  `DATABASE_URL=postgres://postgres@127.0.0.1:55432/reasonbraid_test
  cargo test -p reasonbraid-server --test node_channel
  the_offline_known` → `test result: ok. 1 passed` — the
  never-leased node reads `offline` with null clocks; the
  expired-lease node reads `offline` WITH its past expiry visible
  (the timestamp comparison asserts the past); the unknown id is
  the typed `unknown_node` 404; the enumeration lists both
  offline-KNOWN rows with their states + expiries; the non-admin is
  403.
- [x] **NO REGRESSION** — `bash scripts/run_pg_tests.sh` → 18 live
  suites + the demo `ALL acceptance checks passed` 34/34
  (`target/pg322_guard.log`); `cargo test --all` → 51 offline
  suites green; `cargo clippy --all --all-targets -- -D warnings` →
  clean; `cargo fmt --all -- --check` → rc=0; `make gate` → 13/13 at
  commit.
- [x] **FIX** — `src/api.rs` (the enumeration handler + the route),
  `tests/node_channel.rs` (the three-way distinction test — the
  suite grew to 23).
- [x] **LOCKSTEP** — CHANGELOG, DEV_NOTES, MEMORY, LIVE_STATUS, this
  tree's logs below, `docs/TASK_TREE.md` frontier, KNOWLEDGE_MAP —
  same commit.

## Acceptance Checklist (PHASE-3.2.1)

The CODE change owned by this leaf:
`crates/reasonbraid-server/src/presence.rs` (NEW — the six-state
derivation + the five tests), `crates/reasonbraid-server/src/lib.rs`
(the module), `crates/reasonbraid-server/src/node_channel.rs` (the
presence query gains the profile's declared concurrency; the response
gains the derived `state`), and
`crates/reasonbraid-server/tests/node_channel.rs` (the offline/
available legs assert the derived state) — `\.rs$` in
`.doctrine/code_paths.txt`.

- [x] **REPRODUCE / ISSUE** — the `.2` census: the presence surface
  carries a clock + a flag but no state machine (`grep -rn
  "draining\|busy\|offline_known" crates/` → nothing); §10.2 names
  six states the response cannot express.
- [x] **ROOT CAUSE (WHY + WHERE)** — the states were never DERIVED:
  `online` and `suspended` are raw view columns, and the profile's
  availability class (the `.1` lane) feeds nothing —
  `git grep -c "draining\|busy\|offline_known" 5dd367b -- crates/`
  → 2 files, both the SQLite `busy_timeout` settings (rc=0), and
  `git grep -c "offline_known\|draining\|PresenceState" 5dd367b
  -- crates/` → rc=1 (no presence-state machinery existed). The fix
  point is a pure
  function with an honesty precedence — an unknown id is never
  fabricated, suspension outranks the lease, an expired lease reads
  offline, zero declared concurrency drains — wired into the
  channel's presence response (reads only: presence never changes
  enrollment).
- [x] **ADDRESSED (verified)** — measured before→after. Before:
  `PresenceResponse { node_id, online, suspended, last_seen_at,
  lease_expires_at }` (no state). After:
  `cargo test -p reasonbraid-server --lib presence` → `test result:
  ok. 5 passed` (the unenrolled-is-unknown rule, the suspension
  precedence, the offline-known, the draining, the available);
  `bash scripts/run_pg_tests.sh` → the node_channel presence legs
  assert `state: "offline"` (the seeded lease-less node) and
  `state: "available"` (the heartbeat-renewed node) — `test result:
  ok.` 18 live suites + the demo 34/34 (`target/pg321_guard.log`).
- [x] **NO REGRESSION** — `cargo test --all` → 51 offline suites
  green; `cargo clippy --all --all-targets -- -D warnings` → clean;
  `cargo fmt --all -- --check` → rc=0; `make gate` → 13/13 at commit.
- [x] **FIX** — `src/presence.rs` (the enum + the derivation + the
  five tests), `src/lib.rs`, `src/node_channel.rs` (the concurrency
  join + the `state` field), `tests/node_channel.rs` (the two state
  legs).
- [x] **LOCKSTEP** — CHANGELOG, DEV_NOTES, MEMORY, LIVE_STATUS, this
  tree's logs below, `docs/TASK_TREE.md` frontier, KNOWLEDGE_MAP —
  same commit.

## Acceptance Checklist (PHASE-3.1.3)

The CODE change owned by this leaf:
`crates/reasonbraid-server/src/profiles.rs` (the `ReaderClass` +
`filter_profile` + the visibility ladder) and
`crates/reasonbraid-server/src/api.rs` (the per-reader classification
+ the filtered `get_profile` response) and
`crates/reasonbraid-server/tests/profiles.rs` (the two visibility
tests + the `.1.2` stranger-read assertion updated to the classified
semantics) — `\.rs$` in `.doctrine/code_paths.txt`.

- [x] **REPRODUCE / ISSUE** — the `.1.2` read gate was binary
  (self/owner or 403): the §10.1 per-field visibility policy is
  STORED but nothing enforces it, so the `.2` lane would have no
  privacy-filtered view to consume.
- [x] **ROOT CAUSE (WHY + WHERE)** — the policy was data without an
  evaluator — `git grep -c "filter_profile\|ReaderClass" 0eade27 --
  crates/` → rc=1 (no filter existed). The fix point is the READ
  boundary: classify the reader (self/owner → FULL, same-tenant →
  TENANT, any other enrolled principal → NETWORK, unenrolled →
  nothing), evaluate each field's class against the reader, and OMIT
  the hidden fields (absent, never nulled).
- [x] **ADDRESSED (verified)** — measured before→after. Before: the
  grep above + the 403-for-everyone-else gate. After:
  `DATABASE_URL=postgres://postgres@127.0.0.1:55432/reasonbraid_test
  cargo test -p reasonbraid-server --test profiles` → `test result:
  ok. 5 passed` — the SAME profile read by the role (full), the
  owner (full, the audited admin read), a tenant sibling (the
  tenant view: the self-only fields ABSENT), and a stranger (the
  network view: the tenant+self fields ABSENT) yields exactly the
  allowed field sets each; the response names the applied class;
  the version history refuses the stranger (full-only).
- [x] **NO REGRESSION** — `bash scripts/run_pg_tests.sh` → 18 live
  suites + the demo `ALL acceptance checks passed` 34/34
  (`target/pg313_guard.log`); `cargo test --all` → 51 offline
  suites green; `cargo clippy --all --all-targets -- -D warnings` →
  clean; `cargo fmt --all -- --check` → rc=0; `make gate` → 13/13 at
  commit.
- [x] **FIX** — `src/profiles.rs` (`ReaderClass`, the visibility
  ladder, `filter_profile` — the provenance rides every visible
  claim), `src/api.rs` (`reader_tenant`, `classify_reader`, the
  filtered `get_profile` with the `visibility` marker; the history
  reads keep the full-only gate), `tests/profiles.rs` (the two
  visibility tests + the updated stranger-read assertion).
- [x] **LOCKSTEP** — CHANGELOG, DEV_NOTES, MEMORY, LIVE_STATUS, this
  tree's logs below, `docs/TASK_TREE.md` frontier, KNOWLEDGE_MAP —
  same commit.

## Acceptance Checklist (PHASE-3.1.2)

The CODE change owned by this leaf: `migrations/0019_agent_profiles.sql`
(NEW), `crates/reasonbraid-server/src/profiles.rs` (NEW — the typed
§10.1 profile + the content addressing + the write path),
`crates/reasonbraid-server/src/api.rs` + `src/lib.rs` (the verbs +
the routes), `crates/reasonbraid-server/tests/profiles.rs` (NEW — the
measured suite), `scripts/run_pg_tests.sh` (the guard gained the
suite), and the ten tenant-purging suites' purge lists (the 0019 FK
ripple — `profile_versions`/`agent_profiles` purge before
`agent_roles`) — `\.rs$` + `\.sh$` + `(^|/)migrations/`.

- [x] **REPRODUCE / ISSUE** — the `.1` census: the §10.1
  registration profile has NO table, NO write verbs, NO read
  surface, and ADR-014's answer requires the typed fields the
  deterministic eligibility stage consumes.
- [x] **ROOT CAUSE (WHY + WHERE)** — nothing shipped for backlog
  26 — `git grep -c "agent_profiles" 3ed96dd -- crates/ migrations/`
  → rc=1 (zero matches); the profile is a greenfield the `.1.2`/`.1.3`
  children split at the schema-vs-visibility seam. The fix point is
  the TYPED boundary (deny-unknown-fields — a forged field is a 422,
  never silently dropped) + the content-addressed history (every
  write is a new version; the old ones stay readable).
- [x] **ADDRESSED (verified)** — measured before→after. Before:
  the grep above. After:
  `DATABASE_URL=postgres://postgres@127.0.0.1:55432/reasonbraid_test
  cargo test -p reasonbraid-server --test profiles` → `test result:
  ok. 3 passed` — the role's write lands version 1 with a 64-hex
  content hash, identical content re-hashes identically, changed
  content versions anew, the old versions stay readable; the owner
  AND a sibling role cannot write (403) or read (403); a dangling
  incarnation reference is 400; a forged field is 422 naming it; the
  owner's attestation upgrades the claim's provenance (a new version,
  `owner_attested` + the evidence) and is audited (the tenant_admin
  authorization record).
- [x] **NO REGRESSION** — `bash scripts/run_pg_tests.sh` → 18 live
  suites + the demo `ALL acceptance checks passed` 34/34
  (`target/pg312_guard.log` — the FIRST guard run caught the 0019 FK
  ripple: every tenant-purging purge list now clears the two new
  tables before `agent_roles`); `cargo test --all` → 51 offline
  suites green; `cargo clippy --all --all-targets -- -D warnings` →
  clean; `cargo fmt --all -- --check` → rc=0; `make gate` → 13/13 at
  commit.
- [x] **FIX** — `0019_agent_profiles.sql` (the two tables + the
  content-addressed history), `src/profiles.rs` (the typed profile,
  `content_hash`, `write_profile`, `current_profile`, `version_at`,
  `attest_capability`), `src/api.rs` (the five routes + the self/
  owner gates + the lineage validation), `src/lib.rs` (the module),
  `tests/profiles.rs`, the guard script, the ten purge lists.
- [x] **LOCKSTEP** — CHANGELOG, DEV_NOTES, MEMORY, LIVE_STATUS, this
  tree's logs below, `docs/TASK_TREE.md` frontier, KNOWLEDGE_MAP —
  same commit.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-09-07` | `PHASE-3.1` | docs-only (no code paths changed): `make gate` → 13/13 at commit | Phase 3 opened + the `.1` census + the contract-seam decomposition; frontier → `.1.1` |
| `2026-09-07` | `PHASE-3.1.1` | docs-only (no code paths changed): `make gate` → 13/13 at commit | ADR-014 accepted (the structural-eligibility answer + the embedding trigger); frontier → `.1.2` |
| `2026-09-07` | `PHASE-3.4` | docs-only (no code paths changed): `make gate` → 13/13 at commit | the recruitment-lane census + the contract-seam decomposition (`.4.1` ADR-015 → `.4.2` the call + responses → `.4.3` the storm controls); frontier → `.4.1` |
| `2026-09-07` | `PHASE-3.3.3` | `DATABASE_URL=… cargo test -p reasonbraid-server --test profiles the_match` → `test result: ok. 1 passed` (the ranked resolution + the clamp + the gate); `bash scripts/run_pg_tests.sh` → 18 live suites + the demo 34/34 (`target/pg333_guard.log`); `cargo test --all` → 51 offline suites; clippy/fmt clean; `make gate` → 13/13 | the matching query surface; **`.3` COMPLETE** — frontier → `.4` |
| `2026-09-07` | `PHASE-3.3.2` | `cargo test -p reasonbraid-server --lib matching` → `test result: ok. 10 passed` (the stage-1 gates + the ordering, the zero weight, the eligible-only rule, the deterministic tie); `bash scripts/run_pg_tests.sh` → 18 live suites + the demo 34/34 (`target/pg332_guard.log`); `cargo test --all` → 51 offline suites; clippy/fmt clean; `make gate` → 13/13 | the stage-2 explainable ranking; frontier → `.3.3` |
| `2026-09-07` | `PHASE-3.3.1` | `cargo test -p reasonbraid-server --lib matching` → `test result: ok. 6 passed` (the provenance/visibility/presence/exclusion/concurrency/budget gates + the happy path); `bash scripts/run_pg_tests.sh` → 18 live suites + the demo 34/34 (`target/pg331_guard.log`); `cargo test --all` → 51 offline suites; clippy/fmt clean; `make gate` → 13/13 | the stage-1 evaluation (pure, visibility-scoped, named reasons); frontier → `.3.2` |
| `2026-09-07` | `PHASE-3.3` | docs-only (no code paths changed): `make gate` → 13/13 at commit | the matching-lane census + the contract-seam decomposition (`.3.1` expression + stage-1 → `.3.2` ranking → `.3.3` surface); frontier → `.3.1` |
| `2026-09-07` | `PHASE-3.2.3` | `DATABASE_URL=… cargo test -p reasonbraid-server --test profiles the_directory` → `test result: ok. 1 passed` (the three scopes + the zero-visibility rule); `bash scripts/run_pg_tests.sh` → 18 live suites + the demo 34/34 (`target/pg323_guard.log`); `cargo test --all` → 51 offline suites; clippy/fmt clean; `make gate` → 13/13 | the privacy-filtered directory views; **`.2` COMPLETE** — frontier → `.3` |
| `2026-09-07` | `PHASE-3.2.2` | `DATABASE_URL=… cargo test -p reasonbraid-server --test node_channel the_offline_known` → `test result: ok. 1 passed` (the three-way distinction + the enumeration); `bash scripts/run_pg_tests.sh` → 18 live suites + the demo 34/34 (`target/pg322_guard.log`); `cargo test --all` → 51 offline suites; clippy/fmt clean; `make gate` → 13/13 | the offline-known distinction + the operator's enumeration; frontier → `.2.3` |
| `2026-09-07` | `PHASE-3.2.1` | `cargo test -p reasonbraid-server --lib presence` → `test result: ok. 5 passed` (the derivation precedence); `bash scripts/run_pg_tests.sh` → 18 live suites + the demo 34/34 (`target/pg321_guard.log`, the node_channel presence legs assert `offline`/`available`); `cargo test --all` → 51 offline suites; clippy/fmt clean; `make gate` → 13/13 | the presence state machine (the six states derived; presence reads, never writes); frontier → `.2.2` |
| `2026-09-07` | `PHASE-3.2` | docs-only (no code paths changed): `make gate` → 13/13 at commit | the presence-lane census + the contract-seam decomposition (`.2.1` state machine → `.2.2` offline-known → `.2.3` filtered views); frontier → `.2.1` |
| `2026-09-07` | `PHASE-3.1.3` | `DATABASE_URL=… cargo test -p reasonbraid-server --test profiles` → `test result: ok. 5 passed` (the four-reader measurement + the full-only history); `bash scripts/run_pg_tests.sh` → 18 live suites + the demo 34/34 (`target/pg313_guard.log`); `cargo test --all` → 51 offline suites; clippy/fmt clean; `make gate` → 13/13 | the per-reader visibility enforcement (the absent-not-nulled filter + the classification + the self-describing response); **`.1` COMPLETE** — frontier → `.2` |
| `2026-09-07` | `PHASE-3.1.2` | `DATABASE_URL=… cargo test -p reasonbraid-server --test profiles` → `test result: ok. 3 passed` (the write + history, the gates, the attestation); `bash scripts/run_pg_tests.sh` → 18 live suites + the demo 34/34 (`target/pg312_guard.log`); `cargo test --all` → 51 offline suites; clippy/fmt clean; `make gate` → 13/13 | the profile schema + the write surface (migration 0019 + the verbs + the measured suite); frontier → `.1.3` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `PHASE-3.1` | `REASONBRAID-PHASE3-0001` | the directory-profile lane decomposed at the census seams (the §10.1 greenfield; ADR-014 unopened) |
| `PHASE-3.4` | `REASONBRAID-PHASE3-0013` | the recruitment lane decomposed at the census seams (the baseline + the candidates exist; the vocabulary/call/storm machinery are the gaps) |
| `PHASE-3.3.3` | `REASONBRAID-PHASE3-0012` | the matching query surface (the scope clamp + the filtered candidates + the provenance gate) — **`.3` COMPLETE** |
| `PHASE-3.3.2` | `REASONBRAID-PHASE3-0011` | the stage-2 explainable ranking (the five weighted features, the visibility-safe explanations, the eligible-only ordering) |
| `PHASE-3.3.1` | `REASONBRAID-PHASE3-0010` | the eligibility expression + the stage-1 evaluation (the typed expression, the pure visibility-scoped evaluator, six measured gates) |
| `PHASE-3.3` | `REASONBRAID-PHASE3-0009` | the matching lane decomposed at the census seams (the inputs exist; the expression/evaluator/scoring/surface are the gaps) |
| `PHASE-3.2.3` | `REASONBRAID-PHASE3-0008` | the privacy-filtered directory views (the three measured scopes + the zero-visibility rule) — **`.2` COMPLETE** |
| `PHASE-3.2.2` | `REASONBRAID-PHASE3-0007` | the offline-known distinction + the operator's presence enumeration (the measured three-way distinction) |
| `PHASE-3.2.1` | `REASONBRAID-PHASE3-0006` | the presence state machine (the six-state derivation + the response's `state` field) |
| `PHASE-3.2` | `REASONBRAID-PHASE3-0005` | the presence lane decomposed at the census seams (the shipped lease/presence forms vs the three gaps) |
| `PHASE-3.1.3` | `REASONBRAID-PHASE3-0004` | the per-reader visibility enforcement (the four-reader measurement, the absent-not-nulled filter, the full-only history) — **`.1` COMPLETE** |
| `PHASE-3.1.2` | `REASONBRAID-PHASE3-0003` | the profile schema + the write surface (migration 0019 + the typed §10.1 fields + the content-addressed history + the write/attest verbs + the measured suite) |
| `PHASE-3.1.1` | `REASONBRAID-PHASE3-0002` | ADR-014 accepted (deterministic eligibility first; the embedding engine behind its trigger; no embedding columns) |
