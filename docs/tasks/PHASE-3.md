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
    Children at those seams — frontier → `.2.1`.
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
    Status: `proposed`
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
    Acceptance: the offline-known vs unknown distinction is measured
      (the expired-lease node reads offline-with-expiry; the unknown
      id is the typed 404); no regression.

  - ID: `PHASE-3.2.3`
    Status: `proposed`
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
    Acceptance: the three scopes are measured (the same directory
      read by a tenant member, a stranger, and the owner yields the
      allowed shapes); no regression.

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
| 1 | `PHASE-3.2.2` | `proposed` | `.2.1` done — the presence state machine (the six states derived; the response names the state); the offline-known distinction + the stale handling executes now |

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
| `2026-09-07` | `PHASE-3.2.1` | `cargo test -p reasonbraid-server --lib presence` → `test result: ok. 5 passed` (the derivation precedence); `bash scripts/run_pg_tests.sh` → 18 live suites + the demo 34/34 (`target/pg321_guard.log`, the node_channel presence legs assert `offline`/`available`); `cargo test --all` → 51 offline suites; clippy/fmt clean; `make gate` → 13/13 | the presence state machine (the six states derived; presence reads, never writes); frontier → `.2.2` |
| `2026-09-07` | `PHASE-3.2` | docs-only (no code paths changed): `make gate` → 13/13 at commit | the presence-lane census + the contract-seam decomposition (`.2.1` state machine → `.2.2` offline-known → `.2.3` filtered views); frontier → `.2.1` |
| `2026-09-07` | `PHASE-3.1.3` | `DATABASE_URL=… cargo test -p reasonbraid-server --test profiles` → `test result: ok. 5 passed` (the four-reader measurement + the full-only history); `bash scripts/run_pg_tests.sh` → 18 live suites + the demo 34/34 (`target/pg313_guard.log`); `cargo test --all` → 51 offline suites; clippy/fmt clean; `make gate` → 13/13 | the per-reader visibility enforcement (the absent-not-nulled filter + the classification + the self-describing response); **`.1` COMPLETE** — frontier → `.2` |
| `2026-09-07` | `PHASE-3.1.2` | `DATABASE_URL=… cargo test -p reasonbraid-server --test profiles` → `test result: ok. 3 passed` (the write + history, the gates, the attestation); `bash scripts/run_pg_tests.sh` → 18 live suites + the demo 34/34 (`target/pg312_guard.log`); `cargo test --all` → 51 offline suites; clippy/fmt clean; `make gate` → 13/13 | the profile schema + the write surface (migration 0019 + the verbs + the measured suite); frontier → `.1.3` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `PHASE-3.1` | `REASONBRAID-PHASE3-0001` | the directory-profile lane decomposed at the census seams (the §10.1 greenfield; ADR-014 unopened) |
| `PHASE-3.2.1` | `REASONBRAID-PHASE3-0006` | the presence state machine (the six-state derivation + the response's `state` field) |
| `PHASE-3.2` | `REASONBRAID-PHASE3-0005` | the presence lane decomposed at the census seams (the shipped lease/presence forms vs the three gaps) |
| `PHASE-3.1.3` | `REASONBRAID-PHASE3-0004` | the per-reader visibility enforcement (the four-reader measurement, the absent-not-nulled filter, the full-only history) — **`.1` COMPLETE** |
| `PHASE-3.1.2` | `REASONBRAID-PHASE3-0003` | the profile schema + the write surface (migration 0019 + the typed §10.1 fields + the content-addressed history + the write/attest verbs + the measured suite) |
| `PHASE-3.1.1` | `REASONBRAID-PHASE3-0002` | ADR-014 accepted (deterministic eligibility first; the embedding engine behind its trigger; no embedding columns) |
