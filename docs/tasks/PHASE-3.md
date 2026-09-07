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
| 1 | `PHASE-3.1.3` | `proposed` | `.1.2` done — the profile schema + the write surface (migration 0019, the typed §10.1 fields, the content-addressed history, the write + attest verbs, measured); the visibility enforcement + the read surface executes now |

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
| `2026-09-07` | `PHASE-3.1.2` | `DATABASE_URL=… cargo test -p reasonbraid-server --test profiles` → `test result: ok. 3 passed` (the write + history, the gates, the attestation); `bash scripts/run_pg_tests.sh` → 18 live suites + the demo 34/34 (`target/pg312_guard.log`); `cargo test --all` → 51 offline suites; clippy/fmt clean; `make gate` → 13/13 | the profile schema + the write surface (migration 0019 + the verbs + the measured suite); frontier → `.1.3` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `PHASE-3.1` | `REASONBRAID-PHASE3-0001` | the directory-profile lane decomposed at the census seams (the §10.1 greenfield; ADR-014 unopened) |
| `PHASE-3.1.2` | `REASONBRAID-PHASE3-0003` | the profile schema + the write surface (migration 0019 + the typed §10.1 fields + the content-addressed history + the write/attest verbs + the measured suite) |
| `PHASE-3.1.1` | `REASONBRAID-PHASE3-0002` | ADR-014 accepted (deterministic eligibility first; the embedding engine behind its trigger; no embedding columns) |
