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
| 1 | `PHASE-4.1.3` | `proposed` | `.1.2` done — the typed `ResourceReference` + the submission (the replay + the immutability conflict, measured); the resolver capability registry executes now |

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
| `2026-09-07` | `PHASE-4.1` | docs-only (no code paths changed): `make gate` → 13/13 at commit | Phase 4 opened + the `.1` census + the contract-seam decomposition; frontier → `.1.1` |
| `2026-09-07` | `PHASE-4.1.2` | `DATABASE_URL=… cargo test -p reasonbraid-server --test profiles a_reference_submits` → `test result: ok. 1 passed` (the submit, the replay, the conflict, the 422/400 refusals); `bash scripts/run_pg_tests.sh` → 18 live suites + the demo 34/34 (`target/pg412_guard.log`); `cargo test --all` → 51 offline suites; clippy/fmt clean; `make gate` → 13/13 | the typed reference + the submission; frontier → `.1.3` |
| `2026-09-07` | `PHASE-4.1.1` | docs-only (no code paths changed): `make gate` → 13/13 at commit | ADR-011 + ADR-018 accepted (the digest scheme + the isolation classes); frontier → `.1.2` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `PHASE-4.1` | `REASONBRAID-PHASE4-0001` | the resource-reference lane decomposed at the census seams (the greenfield contract + the registry + the two unopened ADRs) |
| `PHASE-4.1.2` | `REASONBRAID-PHASE4-0003` | the typed `ResourceReference` + the submission (migration 0023 + the replay/conflict immutability) |
| `PHASE-4.1.1` | `REASONBRAID-PHASE4-0002` | ADR-011 + ADR-018 accepted (the `sha256:<hex>` format + the isolation-class vocabulary — no code) |
