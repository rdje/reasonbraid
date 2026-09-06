# PHASE-0: contracts, charter, and kill-risk experiments

## Metadata

- Tree ID: `PHASE-0`
- Status: `active`
- Roadmap lane: Phase 0 (`ROADMAP.md` §20.2)
- Created: `2026-09-05`
- Owner: repo-local workflow
- Companion: `KICKOFF.md` (day-to-day). Scope/gates: `ROADMAP.md`.
- Estimate: 8–14 engineer-weeks, then re-estimate from evidence

## Goal

Build the smallest executable foundation that can answer the six kill-risk
questions in `KICKOFF.md` §1 with code, measurements, failures, and decisions
that constrain Phase 1. Phase 0 does not implement the product.

## Non-Goals

- Semantic search, arbitrary Web acquisition, policy compilation, federation,
  browser workers, or a production UI.
- Exactly-once provider execution or billing.
- Reproducing the final `ROADMAP.md` §7.1 workspace on day one.
- Public namespaces, crates.io names, or domains (blocked until ADR-001).
- Roadmap v0.5.0.

## Operating rules (from `KICKOFF.md` §2)

- Keep the repository private until ADR-001 clears the name.
- Treat v0.4.1 as frozen; park non-blocking ideas in `docs/parking-lot.md`.
- Every experiment has a question, competing options, fixture, observable
  result, decision owner, and deletion plan.
- Build one end-to-end path early.
- Prefer the modular monolith and ordinary Tokio tasks.
- At exit, complete the subtraction record before proposing Phase 1 scope.

## Kill-risk questions

1. Can a Rust control plane and Rust node preserve accepted work across crashes and reconnects?
2. Can a real coding-agent harness be supervised through a narrow adapter without contaminating the core domain model?
3. Can two agents on different hosts participate in one durable thread without a human relaying messages?
4. Can the system report an indeterminate provider attempt honestly instead of blindly retrying it?
5. Are the identity, authority, budget, and event contracts small enough to evolve beside working code?
6. Does even a minimal structured exchange provide enough value to justify continuing toward semantic discovery and governance?

## Task Tree

- ID: `PHASE-0`
  Status: `active`
  Goal: answer the six kill-risk questions with executable evidence
  Children: `PHASE-0.0` … `PHASE-0.8`

### WP0 — Bootstrap and decision log (`KICKOFF` issue 1; backlog 1, 2, 4, 7, 8)

- ID: `PHASE-0.0`
  Status: `done`
  Goal: private-repo operating baseline, templates, CI skeleton, visible risks
  Children: `PHASE-0.0.1` … `PHASE-0.0.8`

- ID: `PHASE-0.0.1`
  Status: `done`
  Goal: ADR-001 — ReasonBraid as an uncleared working name; block public namespace commitments
  Acceptance: ADR recorded; no crates.io/domain/handle reservation assumed; rename remains mechanical
  Roadmap: §2.7, backlog 1, ADR 001
  Verification: recorded below
  Commit: `REASONBRAID-PHASE0-0001`

- ID: `PHASE-0.0.2`
  Status: `done`
  Goal: ADR and evidence-report templates under `docs/adr/` and `docs/evidence/`
  Acceptance: templates exist; every later experiment can fill them without inventing a format
  Verification: recorded below
  Commit: `REASONBRAID-PHASE0-0002`

- ID: `PHASE-0.0.3`
  Status: `done`
  Goal: create `docs/parking-lot.md` with revisit-trigger convention
  Acceptance: file exists; v0.4.1 freeze points here for non-blocking ideas
  Verification: recorded below
  Commit: `REASONBRAID-PHASE0-0003`

- ID: `PHASE-0.0.4`
  Status: `done`
  Goal: create `docs/risks.md` seeded from `ROADMAP.md` §25 (not a copy of the whole table)
  Acceptance: live risk register exists and names owners/triggers
  Verification: recorded below
  Commit: `REASONBRAID-PHASE0-0004`

- ID: `PHASE-0.0.5`
  Status: `done`
  Goal: `docs/dependencies/external-ledger.yaml` skeleton (`ROADMAP.md` §7.4 / §28)
  Acceptance: schema has name, owner, source_url, checked_at, versions, revalidation_trigger; MCP/A2A/Codex/Claude rows stubbed from the 2026-09-04 baseline
  Verification: recorded below
  Commit: `REASONBRAID-PHASE0-0005`
  promotion: declined (routine scaffold slice — ledger schema already normative in ROADMAP.md §7.4; no new durable cross-cutting fact)

- ID: `PHASE-0.0.6`
  Status: `done`
  Goal: record accountable architecture-decision owner and release/security gate owner
  Acceptance: named in a decision record even if one person fills several roles
  Verification: recorded below
  Commit: `REASONBRAID-PHASE0-0006`

- ID: `PHASE-0.0.7`
  Status: `done`
  Goal: CI/dependency/license/secret-scan skeleton beyond bedrock `make check`/`make gate`
  Acceptance: documented commands; `deny.toml` or equivalent policy; no public-release claim
  Roadmap: backlog 8
  Verification: recorded below
  Commit: `REASONBRAID-PHASE0-0007`
  promotion: declined (supply-chain policy is recorded authoritatively in deny.toml + docs/ci.md; no durable cross-cutting fact beyond the scaffold)

- ID: `PHASE-0.0.8`
  Status: `done`
  Goal: G0 contract drafts — glossary, requirement IDs, lifecycle tables, threat-model skeleton, governance-charter draft (not yet normative)
  Acceptance: stable IDs exist for identity/authority/thread/delivery/budget; threat-model skeleton lists trust boundaries; charter draft names bootstrap root authority as a human
  Roadmap: §20.2 deliverables; backlog 2, 4, 7
  Verification: recorded below
  Commit: `REASONBRAID-PHASE0-0008`

### WP1 — Minimal contracts (`KICKOFF` issues 2–3; backlog 3, 5, 6)

- ID: `PHASE-0.1`
  Status: `done`
  Goal: only the types the first slice needs
  Depends on: `PHASE-0.0`
  Children: `PHASE-0.1.1` … `PHASE-0.1.4`

- ID: `PHASE-0.1.1`
  Status: `done`
  Goal: strong IDs — tenant, human, host, node, agent role, incarnation, run, thread
  Acceptance: newtypes; role/incarnation/run/thread cannot be confused in types or wire; provider-attempt confusability deferred to `PHASE-0.1.3` (attempt state machine + `ProviderAttemptId`)
  Roadmap: §8.1, backlog 3, ADR 010
  Verification: recorded below
  Commit: `REASONBRAID-PHASE0-0009`

- ID: `PHASE-0.1.2`
  Status: `done`
  Goal: command submission envelope and committed event envelope; JSON Schema/golden fixtures
  Acceptance: fixtures for every wire payload the demo uses; client-supplied actor/tenant/sequence rejected
  Roadmap: §9.1–9.2, backlog 6
  Verification: recorded below
  Commit: `REASONBRAID-PHASE0-0010`

- ID: `PHASE-0.1.3`
  Status: `done`
  Goal: minimal thread (`open`/`closing`/`closed`/`cancelled`), participation (`invited`/`accepted`/`declined`/`expired`/`left`), provider-attempt (`prepared`/`dispatched`/`completed`/`failed_before_dispatch`/`outcome_unknown`/`reconciled`) transitions
  Acceptance: invalid transitions rejected deterministically; no policy/evidence/directory/federation entities
  Roadmap: §8.4, backlog 5
  Verification: recorded below
  Commit: `REASONBRAID-PHASE0-0011`

- ID: `PHASE-0.1.4`
  Status: `done`
  Goal: typed errors and stable reason-code registry
  Acceptance: codes from `ROADMAP.md` §9.8 that the demo needs; unknown codes remain preservable
  Verification: recorded below
  Commit: `REASONBRAID-PHASE0-0012`

### WP2 — PostgreSQL transaction and outbox (`KICKOFF` issues 4–5; backlog 9–10)

- ID: `PHASE-0.2`
  Status: `done`
  Goal: one transaction writes current state, ordered event, idempotency result, and outbox item; leased worker with fencing
  Depends on: `PHASE-0.1`
  Children: `PHASE-0.2.1`, `PHASE-0.2.2`
  Roadmap: ADR 004

- ID: `PHASE-0.2.1`
  Status: `done`
  Goal: prove PostgreSQL state/event/idempotency/outbox atomic transaction
  Acceptance: successful response ⇔ committed durable state; same key+hash returns original result; different hash is conflict; transport redelivery produces one domain effect
  Kill points: before commit; after commit before response
  Verification: recorded below
  Commit: `REASONBRAID-PHASE0-0013`

- ID: `PHASE-0.2.2`
  Status: `done`
  Goal: leased outbox worker with fencing and kill-point tests
  Acceptance: stale leased workers cannot commit after a newer fencing value; kill points after claim, after delivery, after ack
  Verification: recorded below
  Commit: `REASONBRAID-PHASE0-0014`

### WP3 — Node SQLite journal and reconnect (`KICKOFF` issues 6–7; backlog 12–13)

- ID: `PHASE-0.3`
  Status: `done`
  Goal: outbound node, WAL journal, cursor resume, reconciliation
  Depends on: `PHASE-0.1`; may proceed beside `PHASE-0.2`
  Children: `PHASE-0.3.1`, `PHASE-0.3.2`
  Roadmap: ADR 006, ADR 012

- ID: `PHASE-0.3.1`
  Status: `done`
  Goal: SQLite node journal (WAL, explicit durability) and journal inspection CLI
  Acceptance: operators inspect pending/ambiguous entries without opening SQLite by hand; crash after possible dispatch yields `outcome_unknown` unless the adapter can prove the result
  Verification: recorded below
  Commit: `REASONBRAID-PHASE0-0015`

- ID: `PHASE-0.3.2`
  Status: `done`
  Goal: outbound node channel with cursor resume and reconciliation handshake
  Acceptance: reconnect exchanges last acknowledged server cursor and pending local operation IDs; duplicate command never creates a second local operation; node not schedulable until reconciliation completes
  Verification: recorded below
  Commit: `REASONBRAID-PHASE0-0016`

### WP4 — Adapter boundary (`KICKOFF` issues 8–9; backlog 19–21)

- ID: `PHASE-0.4`
  Status: `done`
  Goal: narrow adapter contract, fake adapter, first real harness
  Depends on: `PHASE-0.1` and enough of `PHASE-0.3` to journal attempts
  Children: `PHASE-0.4.1`, `PHASE-0.4.2`

- ID: `PHASE-0.4.1`
  Status: `done`
  Goal: deterministic fake harness adapter and ambiguity fixtures
  Acceptance: can stream, fail, hang, report usage, ignore cancellation, lose a response after dispatch; vendor DTOs never enter core; credentials never enter events/fixtures
  Verification: recorded below
  Commit: `REASONBRAID-PHASE0-0018`

- ID: `PHASE-0.4.2`
  Status: `done`
  Goal: qualify the first real harness (Codex-family or Claude-family) behind the same contract
  Acceptance: dispatch ack ≠ completion; unsupported status lookup → `outcome_unknown` not a retry recommendation; evidence report says whether the second real adapter belongs in late Phase 0 or Phase 1
  Verification: recorded below
  Commit: `REASONBRAID-PHASE0-0019`

### WP5 — Identity, authority, budget (`KICKOFF` issues 10–11; backlog 11, 24)

- ID: `PHASE-0.5`
  Status: `done`
  Goal: development enrollment ceiling, scoped commands, reservation before dispatch
  Depends on: `PHASE-0.1`, `PHASE-0.2`
  Children: `PHASE-0.5.1`, `PHASE-0.5.2`

- ID: `PHASE-0.5.1`
  Status: `done`
  Goal: development `EnrollmentAuthorityBoundary`, scoped commands, authorization audit record
  Acceptance: tenant membership alone does not grant mandate; every command records actor, subject if delegated, grant/boundary reference, decision, policy digest/version; grant cannot exceed ceiling
  Roadmap: §4.4
  Verification: recorded below
  Commit: `REASONBRAID-PHASE0-0020`

- ID: `PHASE-0.5.2`
  Status: `done`
  Goal: call/token/time reservation and budget denial path
  Acceptance: no provider dispatch without an applicable reservation; denial tests cross server and node
  Verification: recorded below
  Commit: `REASONBRAID-PHASE0-0021`

### WP6 — Two-host vertical experiment (`KICKOFF` issues 12–13)

- ID: `PHASE-0.6`
  Status: `done`
  Goal: CLI flow and two-host crash/reconnect demonstration
  Depends on: `PHASE-0.2`–`PHASE-0.5`
  Children: `PHASE-0.6.1`, `PHASE-0.6.2`

- ID: `PHASE-0.6.1`
  Status: `done`
  Goal: CLI — enroll, create thread, invite, contribute, challenge, revise, close, inspect
  Acceptance: no database surgery required to inspect state
  Plan: server thread domain (`src/threads.rs` — typed operation bodies, projection in
    `aggregate_state.state`, core-machine validation) + control API (`src/api.rs` —
    enroll bootstrap, `/v1/threads` command/query surface, dev actor header, typed
    errors) + `rb-server` binary + `migrations/0006` (enrollments) + the new
    `reasonbraid-cli` crate (the eight verbs; state dir repo-local by default) + a
    live-PG command suite and a real-binary end-to-end CLI suite. Node-execution glue
    (server→inbox dispatch, node result→thread) belongs to `.6.2`.
  Verification: recorded below
  Commit: `REASONBRAID-PHASE0-0022`

- ID: `PHASE-0.6.2`
  Status: `done`
  Goal: script and run the two-host crash/reconnect demonstration
  Acceptance: no human copies messages; accepted commands survive restart; duplicate transport → one domain effect; no silent retry of indeterminate provider calls; closure preserves contributions and unresolved objections; reproducible evidence bundle
  Roadmap: Demonstration A subset
  Verification: recorded below
  Commit: `REASONBRAID-PHASE0-0023`

### WP7 — Small deliberation/routing benchmark (`KICKOFF` issue 14)

- ID: `PHASE-0.7`
  Status: `done`
  Goal: small versioned corpus vs single-agent, blind independent, critique/revise, moderator/synthesis
  Depends on: fake adapter and at least one real adapter; may use offline harness if vertical slice unfinished
  Acceptance: results include cases and uncertainty, not only an average; no independence score; null/negative result is acceptable and narrows the claim
  Roadmap: §13.7, ADR 017, hypothesis H1/H6
  Verification: recorded below
  Commit: `REASONBRAID-PHASE0-0024`

### WP8 — Phase 0 decision and subtraction package (`KICKOFF` issue 15)

- ID: `PHASE-0.8`
  Status: `done`
  Goal: evidence manifest, ADR set, subtraction record, Phase 1 go/rework/pivot/stop
  Depends on: completed experiments
  Children: `PHASE-0.8.1`

- ID: `PHASE-0.8.1`
  Status: `done`
  Goal: publish Phase 0 evidence manifest, ADR set, `SubtractionRecord`, and Phase 1 decision
  Acceptance: G0 evidence for identity/authority/thread/delivery/budget; named owner signs go/rework/pivot/stop; 2×-estimate review if total exceeds 28 engineer-weeks; v0.5.0 still forbidden
  Verification: recorded below (the ADR-002 signature is the ONE out-of-band item — see Blockers)
  Commit: `REASONBRAID-PHASE0-0025`

### Maintenance — spine/policy upkeep

- ID: `PHASE-0-MAINT-1`
  Status: `pending`
  Goal: review and adopt the revised `README_POLICY.md` (upstream fsmgen copy has been updated; the repo-local copy is the older revision)
  Blocked on: director decision on adoption timing
  Scope: deliberate local review of the upstream diff (fenced adoption note; "Authority and provenance"; "Routing pressure closure" with per-destination-class pressure controls; derived line/byte caps instead of example values; unconditional check rule; 9-step adoption checklist), then — if adopted — tighten `scripts/check_readme_stability.sh` from template defaults to derived caps and wire a routed-destination inventory; record the decision in `docs/decisions/`.
  Verification: pending
  Commit: pending

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `PHASE-0-MAINT-1` | `pending` | README_POLICY upstream revision found at session start (2026-09-07); owned, queued after the Phase 0 leaves, blocked on the director's word |

`RB-SEED` is `done`. This tree is executable.

## Decisions

- `2026-09-05`: KICKOFF work packages WP0–WP8 are this tree's children.
- `2026-09-05`: Phase 0 crate shape follows `KICKOFF.md` §3 (`reasonbraid-core`,
  `-server`, `-node`, `-adapter`, `-cli`), not the full §7.1 workspace.

## Open Questions

- Which real harness is first (Codex-family vs Claude-family) — decided in `PHASE-0.4.2` after the fake adapter.
- **Project license is unresolved — director-deferred (2026-09-06).** `Cargo.toml` declares `license = "MIT OR Apache-2.0"` (the bedrock default) but no `LICENSE` file exists; the director will resolve the choice later. Coupled to ADR-001 (no release until the name clears). Does not block the frontier; must be settled before any release or `cargo publish`.
- **`README_POLICY.md` upstream has been revised (found 2026-09-07 at session start, §14 check).** The fsmgen source now adds: a fenced local-adoption note; an "Authority and provenance" section; "Routing pressure closure" (inventory every routed destination through a controlled terminal, per-class pressure controls, the 1.5 MB status-file cautionary tale); derived line/byte caps instead of example values; the unconditional-check rule (no changed-path short-circuit); and a 9-step adoption checklist. The repo-local copy is the older revision, and `scripts/check_readme_stability.sh` still uses template defaults (300 lines / 16384 bytes) with no routing-closure inventory. `README.md` is 47 lines, so nothing is at hazard today. Owned by leaf `PHASE-0-MAINT-1`; executing it needs the director's word on timing.

## Blockers

- **ADR-002 signature (director-owned, the Phase 0 exit gate's one remaining item).** The WP8 package is published (evidence manifest, ADR set, SubtractionRecord, refreshed risk register); ADR-002 carries the GO recommendation with a pending signature line. Phase 0's formal exit (KICKOFF §7: "a named owner signs a go, rework, pivot, or stop record") closes when the director signs it.

## Acceptance Checklist (PHASE-0.0.7)

The Makefile edit is the CODE change owned by this leaf (per `.doctrine/code_paths.txt`);
`deny.toml`, `.github/workflows/supply-chain.yml`, and `docs/ci.md` are non-code. Enforced by
the `TASK-ACCEPTANCE` doctrine.

- [x] **ROOT CAUSE (WHY + WHERE)** — ROADMAP §16.10 / backlog 8 / KICKOFF §3 require
  "dependency/advisory/license checks" and "secret scanning", but bedrock's `make check`
  covers only fmt/clippy/test and `make gate` only doctrine — no `deny`/`secret-scan` target
  existed. `make -n check` resolves the whole Rust gate to:
  `cargo fmt --all -- --check` → `cargo clippy --all-targets --all-features -- -D warnings` → `cargo test --all`
  (nothing scans dependencies, licenses, or secrets)
- [x] **ADDRESSED (verified)** — added `deny.toml`, `.github/workflows/supply-chain.yml`,
  `docs/ci.md`, and Makefile `deny`/`secret-scan` targets. `make -n deny` → `cargo deny check`;
  `make -n secret-scan` → `gitleaks detect --source . --redact` (dry-runs confirm the recipes are wired)
- [x] **NO REGRESSION** — `make gate` → `=== all doctrines green ===` (13/13); `make check` →
  `test result: ok. 1 passed; 0 failed; 0 ignored` (fmt/clippy/test all pass)
- [x] **FIX** — Makefile gains `deny` (→ `cargo deny check`) and `secret-scan` (→ `gitleaks detect --source . --redact`)
  targets plus help text; `.PHONY` extended with both.
- [x] **LOCKSTEP** — `docs/ci.md` documents the commands and the no-release-claim boundary; the Makefile
  help text and `deny.toml` header point at it.

## Acceptance Checklist (PHASE-0.1.1)

The `crates/reasonbraid-core` crate (`.rs` + `Cargo.toml`) is the CODE change owned by
this leaf (per `.doctrine/code_paths.txt`). Enforced by the `TASK-ACCEPTANCE` doctrine.

- [x] **ROOT CAUSE (WHY + WHERE)** — `ROADMAP.md` §8.3 mandates "newtypes for every ID;
  never interchange plain UUID strings inside domain code" and §17.2 prefers a sortable
  UUIDv7, but the scaffold shipped only a placeholder binary with no domain types, so
  tenant/role/incarnation/run/thread could all collapse to bare UUID strings.
  `git ls-files 'crates/*'` (before) → `crates/app/Cargo.toml` / `crates/app/src/main.rs`
  — no `reasonbraid-core`, no ID type existed.
- [x] **ADDRESSED (verified)** — landed `crates/reasonbraid-core` with `Id<K>` branded
  newtypes over `uuid::Uuid` (v7) and eight families (Tenant, HumanPrincipal, Host, Node,
  AgentRole, AgentIncarnation, Run, Thread), each a distinct type AND a distinct
  prefix-checked wire form. `cargo test -p reasonbraid-core` →
  `test result: ok. 6 passed; 0 failed; 0 ignored` (distinct-type, v7, prefix, serde
  round-trip, wrong-prefix-rejected, parse tests all pass).
- [x] **NO REGRESSION** — `make check` → `cargo fmt --all -- --check` (clean),
  `cargo clippy --all-targets --all-features -- -D warnings` (no warnings),
  `cargo test --all` → `test result: ok. 6 passed; 0 failed`; `make gate` →
  `=== all doctrines green ===` (13/13).
- [x] **FIX** — new lib crate `crates/reasonbraid-core` (src/lib.rs + src/id.rs + Cargo.toml)
  with `serde` (derive) + `uuid` (v7) deps; placeholder `crates/app` binary removed (its
  package name `reasonbraid` is superseded by `reasonbraid-core` per `KICKOFF.md` §3).
- [x] **LOCKSTEP** — decision record `docs/decisions/2026-09-06_id-representation.md`
  (`answers:` present) + INDEX row; `knowledge-map/subsystems.md` gains the
  `reasonbraid-core` row; `CHANGELOG.md` / `DEV_NOTES.md` / `LIVE_STATUS.md` updated;
  README and mdBook unchanged (no user-facing surface change).

## Acceptance Checklist (PHASE-0.1.2)

The `crates/reasonbraid-core` crate (`.rs` + `Cargo.toml` + `fixtures/*.json` +
`schema/*.json`, all under `crates/`) is the CODE change owned by this leaf (per
`.doctrine/code_paths.txt`). Enforced by the `TASK-ACCEPTANCE` doctrine.

- [x] **ROOT CAUSE (WHY + WHERE)** — `ROADMAP.md` §9.1 sketches the command/event
  envelope split and the "client-supplied actor/timestamp/authority/sequence/tenant
  ignored or rejected" rule, but nothing enforced it: serde ignores unknown JSON fields
  by default, so a `CommandEnvelope` that merely *omitted* authoritative fields would
  still accept them from a client. `git ls-files 'crates/reasonbraid-core/*'` (before) →
  only `Cargo.toml` + `src/lib.rs` + `src/id.rs`; no envelope module, no fixtures, no
  schema goldens existed.
- [x] **ADDRESSED (verified)** — landed `src/envelope.rs` with `CommandEnvelope` /
  `ClientContext` / `CommittedEvent` + `PROTOCOL_VERSION = "reasonbraid/0.4"`, all
  `#[serde(deny_unknown_fields)]`; five new ID families (`evt`/`req`/`corr`/`agt`/`authz`)
  in `id.rs`; golden fixtures + `schemars`-derived golden schemas. `cargo test -p
  reasonbraid-core` → `test result: ok. 10 passed; 0 failed; 1 ignored` (round-trip
  canonical ×2, authoritative-field rejection, schema-drift, plus the six ID tests).
- [x] **NO REGRESSION** — `make check` → `cargo fmt --all -- --check` (clean),
  `cargo clippy --all-targets --all-features -- -D warnings` (no warnings),
  `cargo test --all` → `test result: ok. 10 passed; 0 failed; 1 ignored`; `make gate` →
  `=== all doctrines green ===` (13/13).
- [x] **FIX** — new `crates/reasonbraid-core/src/envelope.rs` (+ tests), `fixtures/`
  (3 wire payloads incl. a forged-authority fixture), `schema/` (2 golden schemas);
  `src/id.rs` gains 5 families + a manual `JsonSchema` impl; `src/lib.rs` re-exports the
  envelopes and new IDs; `Cargo.toml` adds `schemars` (derive) + promotes `serde_json`.
- [x] **LOCKSTEP** — decision record `docs/decisions/2026-09-06_envelope-representation.md`
  (`answers:` present) + INDEX row; `knowledge-map/subsystems.md` updated to "identifiers
  and envelopes landed"; `CHANGELOG.md` / `DEV_NOTES.md` / `LIVE_STATUS.md` updated; README
  and mdBook unchanged (no user-facing surface change).

## Acceptance Checklist (PHASE-0.1.3)

The `crates/reasonbraid-core` crate (`.rs` files under `crates/`) is the CODE change owned
by this leaf (per `.doctrine/code_paths.txt`). Enforced by the `TASK-ACCEPTANCE` doctrine.

- [x] **ROOT CAUSE (WHY + WHERE)** — `ROADMAP.md` §8.4 lists lifecycle *states* but not
  *edges*, and §8.6 requires "a deterministic aggregate may accept and translate to an
  event," yet no state type existed to reject an invalid move. `git ls-files
  'crates/reasonbraid-core/src/*'` (before) → `envelope.rs` / `id.rs` / `lib.rs` only; no
  `state.rs`, no provider-attempt ID.
- [x] **ADDRESSED (verified)** — landed `src/state.rs` with three minimal state machines
  (`ThreadState`, `ParticipationState`, `ProviderAttemptState`), each a total, fallible
  `apply` returning `TransitionError` on invalid moves; added `ProviderAttemptId` (`patt`)
  to `id.rs`. `cargo test -p reasonbraid-core` →
  `test result: ok. 17 passed; 0 failed; 1 ignored` (three exhaustive edge-table tests,
  terminal-rejection, deterministic-error, snake_case serde, distinct-type tests all pass).
- [x] **NO REGRESSION** — `make check` → `cargo fmt --all -- --check` (clean),
  `cargo clippy --all-targets --all-features -- -D warnings` (no warnings),
  `cargo test --all` → `test result: ok. 17 passed; 0 failed; 1 ignored`; `make gate` →
  `=== all doctrines green ===` (13/13).
- [x] **FIX** — new `crates/reasonbraid-core/src/state.rs` (3 state enums + 3 transition
  enums + `TransitionError` + tests); `id.rs` adds the `ProviderAttemptId` family and
  extends the distinct-type/prefix tests to 14; `lib.rs` adds `mod state` + re-exports.
- [x] **LOCKSTEP** — decision record `docs/decisions/2026-09-06_state-transitions.md`
  (`answers:` present) + INDEX row; `knowledge-map/subsystems.md` updated to "state
  machines"; `CHANGELOG.md` / `DEV_NOTES.md` / `LIVE_STATUS.md` updated; README and mdBook
  unchanged (no user-facing surface change).

## Acceptance Checklist (PHASE-0.1.4)

The `crates/reasonbraid-core` crate (`.rs` files under `crates/`) is the CODE change owned
by this leaf (per `.doctrine/code_paths.txt`). Enforced by the `TASK-ACCEPTANCE` doctrine.

- [x] **ROOT CAUSE (WHY + WHERE)** — `ROADMAP.md` §9.8 lists reason codes but does not say
  how a build treats a code it has never seen: a closed enum rejects it (deserialization
  error) and a bare `String` drops the typing of the known set. `git ls-files
  'crates/reasonbraid-core/src/*'` (before) → `envelope.rs` / `id.rs` / `lib.rs` /
  `state.rs` only; no `error.rs`, no reason-code type existed.
- [x] **ADDRESSED (verified)** — landed `src/error.rs` with `KnownReasonCode` (the complete
  20-code §9.8 registry, snake_case), `ReasonCode` (`Known` + `Unknown(String)` so an
  unknown code is preserved verbatim), `Retryability` (tri-state), and `DomainError` (code +
  retryability + message + optional correlation/details); `From<TransitionError>` maps to
  `invalid_transition`. `cargo test -p reasonbraid-core` →
  `test result: ok. 23 passed; 0 failed; 1 ignored` (known-code round-trip, unknown-code
  preservation, discrimination, retryability serde, error round-trip, transition mapping).
- [x] **NO REGRESSION** — `make check` → `cargo fmt --all -- --check` (clean),
  `cargo clippy --all-targets --all-features -- -D warnings` (no warnings),
  `cargo test --all` → `test result: ok. 23 passed; 0 failed; 1 ignored`; `make gate` →
  `=== all doctrines green ===` (13/13).
- [x] **FIX** — new `crates/reasonbraid-core/src/error.rs` (registry + typed error + tests);
  `lib.rs` adds `mod error` + re-exports.
- [x] **LOCKSTEP** — decision record `docs/decisions/2026-09-06_reason-codes.md`
  (`answers:` present) + INDEX row; `knowledge-map/subsystems.md` updated to "typed errors";
  `CHANGELOG.md` / `DEV_NOTES.md` / `LIVE_STATUS.md` updated; README and mdBook unchanged
  (no user-facing surface change).

## Acceptance Checklist (PHASE-0.2.1)

The `crates/reasonbraid-server` crate (`.rs` + `Cargo.toml` + `tests/*.rs`), the
repository-root `migrations/0001_atomic_transaction.sql`, `scripts/run_pg_tests.sh`, and the
`deny.toml` + `.github/workflows/rust.yml` edits are the CODE change owned by this leaf (per
`.doctrine/code_paths.txt`: `crates/`, `scripts/`, `.rs`, `.sh`; `Cargo.toml`/`Cargo.lock`
are code by the ownership check). Enforced by the `TASK-ACCEPTANCE` doctrine.

- [x] **ROOT CAUSE (WHY + WHERE)** — `ROADMAP.md` §8.6 / KICKOFF WP2 require "one transaction
  writes current state, ordered event, idempotency result, and outbox item," but no server
  crate or PostgreSQL driver existed to run it — `git ls-files 'crates/*'` (before) →
  `crates/reasonbraid-core` only, no `reasonbraid-server`, no `migrations/`, no Postgres.
  Four autocommit `INSERT`s would tear, and "check-then-insert" idempotency races under
  redelivery.
- [x] **ADDRESSED (verified)** — landed `crates/reasonbraid-server` (`apply_command` writes
  idempotency + event_log + aggregate_state + outbox in one transaction, claim-first
  idempotency via `ON CONFLICT DO NOTHING`), `migrations/0001_atomic_transaction.sql`
  (outbox FK → event_log), `scripts/run_pg_tests.sh` (ephemeral Postgres), and a `pg-tests`
  CI job. `bash scripts/run_pg_tests.sh` →
  `test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured` against live PostgreSQL 16.15
  (successful-response⇔durable, same-key+hash replay, different-hash conflict, redelivery→one
  effect, before-commit rollback).
- [x] **NO REGRESSION** — `make check` → `cargo fmt --all -- --check` clean,
  `cargo clippy --all-targets --all-features -- -D warnings` no warnings,
  `cargo test --all` → `test result: ok. 23 passed; 0 failed; 1 ignored` (core) + server
  `5 passed` (skip offline); `make gate` → `=== all doctrines green ===` (13/13);
  `make deny` → `advisories ok, bans ok, licenses ok, sources ok`; `make secret-scan` →
  `no leaks found`; `make book` → HTML written.
- [x] **FIX** — new `crates/reasonbraid-server` (src/lib.rs + src/tx.rs + tests/atomic_transaction.rs
  + Cargo.toml); `migrations/0001_atomic_transaction.sql`; `scripts/run_pg_tests.sh`;
  `.github/workflows/rust.yml` gains a `pg-tests` job; `deny.toml` corrected for cargo-deny
  0.20 (`unmaintained` scope, `BSD-3-Clause`, `skip` for getrandom/hashbrown/syn).
- [x] **LOCKSTEP** — decision record `docs/decisions/2026-09-06_atomic-transaction.md`
  (`answers:` present) + INDEX row; `knowledge-map/subsystems.md` gains the
  `reasonbraid-server` row; `docs/ci.md` documents `run_pg_tests.sh` + the `pg-tests` job;
  `CHANGELOG.md` / `DEV_NOTES.md` / `LIVE_STATUS.md` updated; README and mdBook unchanged
  (no user-facing surface change).

## Acceptance Checklist (PHASE-0.2.2)

The `crates/reasonbraid-server` crate (`.rs` + `Cargo.toml` + `tests/*.rs`), the
repository-root `migrations/0002_outbox_worker.sql`, and `scripts/run_pg_tests.sh` +
`.github/workflows/rust.yml` are the change owned by this leaf (per `.doctrine/code_paths.txt`:
`crates/`, `scripts/`, `.rs`, `.sh`; `Cargo.toml`/`Cargo.lock` are code by the ownership
check). Enforced by the `TASK-ACCEPTANCE` doctrine.

- [x] **ROOT CAUSE (WHY + WHERE)** — `ROADMAP.md` §17.3 and KICKOFF WP2 require a worker that
  "claims durable jobs with lease owner, lease expiry, attempt number" and "fencing tokens
  prevent a stale worker from committing after a newer lease," but the `.2.1` outbox was
  write-only: `git ls-files 'crates/reasonbraid-server/*'` (before) →
  `src/lib.rs` / `src/tx.rs` / `tests/atomic_transaction.rs` / `Cargo.toml` only, and
  `migrations/0001_atomic_transaction.sql` carries `dispatched BOOLEAN NOT NULL DEFAULT false`
  with no `lease_owner`/`lease_token`/`lease_until`/`attempt` columns — nothing could claim
  work, and nothing could stop a stale claim from acknowledging.
- [x] **ADDRESSED (verified)** — landed `src/outbox.rs` (`claim_ready`/`deliver`/`complete`,
  each phase its own commit), `migrations/0002_outbox_worker.sql` (lease+fencing columns with
  an all-or-nothing CHECK, claim index, deduplicated `outbox_delivery` sink), and
  `tests/outbox_worker.rs` (7 tests). `bash scripts/run_pg_tests.sh` →
  `test result: ok. 7 passed; 0 failed; 0 ignored` (`outbox_worker`) against live PostgreSQL
  16.15, plus the `.2.1` suite `5 passed` — exclusive concurrent claim, re-claim after expiry
  with a new token, **stale worker refused after a newer fencing value**, expired lease
  refused with a matching token, and kill points 3/4/5 recovering to exactly one delivery
  effect. First live run FAILED (7/7) and the failure was root-caused with a probe: the tests
  shared one queue and ran in parallel, so each test's global oldest-first claim took other
  tests' leftover rows; the suite now serializes under a module-level async mutex and purges
  the queue under the guard — all 12 PG tests green.
- [x] **NO REGRESSION** — `make check` → `cargo fmt --all -- --check` clean + `cargo clippy
  --all-targets --all-features -- -D warnings` no warnings + `cargo test --all` →
  `23 passed; 0 failed; 1 ignored` (core) + server `5 passed` + `7 passed` (skip offline);
  `make gate` → `=== all doctrines green ===` (13/13); `make deny` → `advisories ok, bans ok,
  licenses ok, sources ok`; `make secret-scan` → `no leaks found`; `make book` → HTML written.
- [x] **FIX** — new `crates/reasonbraid-server/src/outbox.rs`; `migrations/0002_outbox_worker.sql`;
  `crates/reasonbraid-server/tests/outbox_worker.rs`; server `Cargo.toml` gains `chrono` +
  sqlx `chrono` feature (caller-supplied lease clock; no sleeps in tests);
  `scripts/run_pg_tests.sh` + the CI `pg-tests` job now run both integration binaries.
- [x] **LOCKSTEP** — decision record `docs/decisions/2026-09-06_outbox-worker-fencing.md`
  (`answers:` present, measured behavior + rejected designs) + INDEX row;
  `knowledge-map/subsystems.md` updated ("leased outbox worker landed");
  `docs/ci.md` documents both test binaries; `CHANGELOG.md` / `DEV_NOTES.md` /
  `LIVE_STATUS.md` / `MEMORY.md` updated; `docs/TASK_TREE.md` frontier moved to `PHASE-0.3.1`;
  README and mdBook unchanged (internal server machinery — no user-facing surface change).

## Acceptance Checklist (PHASE-0.3.1)

The `crates/reasonbraid-node` crate (`.rs` + `Cargo.toml` + `migrations/`), the
`crates/reasonbraid-core` state-machine extension (`.rs`), and `Cargo.lock` are the CODE
change owned by this leaf (per `.doctrine/code_paths.txt`: `crates/`; `Cargo.toml`/
`Cargo.lock` are code by the ownership check). Enforced by the `TASK-ACCEPTANCE` doctrine.

- [x] **ROOT CAUSE (WHY + WHERE)** — `ROADMAP.md` §11.4/§17.4 + KICKOFF WP3 require "WAL mode
  and an explicit development durability setting" and "a crash after possible provider
  dispatch yields `outcome_unknown` unless the adapter can prove the result", but no node
  crate existed — `git ls-files 'crates/*'` (before) → `reasonbraid-core` +
  `reasonbraid-server` only, no SQLite journal, no inspection surface — and core's
  provider-attempt machine could not even EXPRESS a proven post-dispatch failure
  (`failed_known`) or the §11.3 provider-lookup recovery edges
  (`OutcomeUnknown → Completed|FailedKnown`): recording "the adapter proved failure"
  honestly was unrepresentable.
- [x] **ADDRESSED (verified)** — landed `crates/reasonbraid-node` (`src/journal.rs` +
  embedded `migrations/0001_node_journal.sql` + `src/bin/rb-journal.rs`) and extended the
  core machine (`FailedKnown` + `(Dispatched, FailKnown)` + `(OutcomeUnknown,
  Complete|FailKnown)` edges). `cargo test -p reasonbraid-node` →
  `test result: ok. 13 passed` (journal unit) + `test result: ok. 6 passed` (CLI) +
  `test result: ok. 10 passed` (kill points): the WAL/`synchronous=FULL` profile is applied
  AND recorded in `journal_meta`; the dispatch boundary record is durable and visible to a
  SECOND connection before the adapter runs; KP-1…KP-9 sweep every seam (before-command →
  nothing persisted, after-prepare → `safe_to_redeliver`, after-dispatch →
  `outcome_unknown`, proven status lookup → `completed`, after-result → terminal,
  emitted/acked events stable); command/operation dedupe; invalid moves rejected by the
  core machine; garbage files fail cleanly. `cargo test -p reasonbraid-core` →
  `test result: ok. 24 passed; 0 failed; 1 ignored`.
- [x] **NO REGRESSION** — `make check` → `cargo fmt --all -- --check` clean + `cargo clippy
  --all-targets --all-features -- -D warnings` no warnings + `cargo test --all` →
  `24 passed; 0 failed; 1 ignored` (core) + `13 passed` + `6 passed` + `10 passed` (node) +
  `5 passed` + `7 passed` (server, skip offline); `bash scripts/run_pg_tests.sh` →
  `test result: ok. 5 passed` (atomic) + `test result: ok. 7 passed` (outbox worker) against
  live PostgreSQL 16.15 (server untouched by this leaf, re-proven anyway); `make gate` →
  `=== all doctrines green ===` (13/13); `make deny` → `advisories ok, bans ok, licenses ok,
  sources ok` (clap + libsqlite3-sys tree; the path dependency is pinned
  `version = "0.1.0"` to satisfy the wildcard ban); `make secret-scan` → `no leaks found`;
  `make book` → HTML written.
- [x] **FIX** — new `crates/reasonbraid-node` (src/lib.rs + src/journal.rs +
  src/bin/rb-journal.rs + migrations/0001_node_journal.sql + tests/journal_kill_points.rs +
  tests/journal_cli.rs + Cargo.toml); `crates/reasonbraid-core/src/state.rs` gains
  `FailedKnown`, the three new edges, `from_wire_name` + `FromStr` with
  `UnknownProviderAttemptState`, and the exhaustive tables + parse tests; `Cargo.lock`
  updated (clap, libsqlite3-sys, etc.).
- [x] **LOCKSTEP** — decision record `docs/decisions/2026-09-06_node-journal.md`
  (`answers:` present, measured behavior + rejected designs) + INDEX row;
  `knowledge-map/subsystems.md` gains the `reasonbraid-node` row; the mdBook gains
  `docs/book/src/node-journal.md` + its SUMMARY entry (the inspection CLI is an operator
  surface); `TOOLBOX.md` now lists the real diagnostic tools; `docs/ci.md` notes the
  journal tests run in plain CI; `CHANGELOG.md` / `DEV_NOTES.md` / `LIVE_STATUS.md` /
  `MEMORY.md` updated; `docs/TASK_TREE.md` frontier moved to `PHASE-0.3.2`; README
  unchanged (no new standard command — the CLI is documented in the book).

## Acceptance Checklist (PHASE-0.3.2)

The `crates/reasonbraid-server` channel module (`.rs` + `Cargo.toml`), the
repository-root `migrations/0003_node_inbox.sql`, the `crates/reasonbraid-node` channel
client + node facade + journal additions (`.rs` + `Cargo.toml` + `migrations/`), the
`scripts/run_pg_tests.sh` + `.github/workflows/rust.yml` edits, and `Cargo.lock` are the
CODE change owned by this leaf (per `.doctrine/code_paths.txt`). Enforced by the
`TASK-ACCEPTANCE` doctrine.

- [x] **ROOT CAUSE (WHY + WHERE)** — `ROADMAP.md` §17.4/§9.3 + KICKOFF WP3 require an
  outbound node channel where "reconnect exchanges the last acknowledged server cursor
  and pending local operation IDs", "a duplicated command never creates a second local
  operation", and "the node does not become schedulable until reconciliation completes".
  Before this leaf, `git ls-files 'crates/*'` → core + server (`tx.rs`/`outbox.rs` only —
  no HTTP surface, no per-node delivery ledger) + node (journal only — no channel, no
  schedulability gate): a node could not receive a single command from the control plane,
  and nothing stopped a crashed node from silently double-processing redeliveries.
- [x] **ADDRESSED (verified)** — landed the server-side channel
  (`src/node_channel.rs` + `migrations/0003_node_inbox.sql`: durable per-node inbox with
  a monotonic cursor, replay from the node's reported cursor, deduplicated node-event
  receipts, handshake directives, axum routes) and the node side (`src/channel.rs` client,
  `src/node.rs` `Offline → Reconciling → Schedulable` facade, journal `channel_state`
  cursor + `known_events` acknowledgement). `bash scripts/run_pg_tests.sh` →
  `test result: ok. 13 passed` (`node_channel`) against live PostgreSQL 16.15 + real
  `127.0.0.1` sockets: fresh handshake plays the whole inbox; reconnect replays ONLY the
  tail after the reported cursor; **duplicate delivery leaves the same operation ids**;
  **the node refuses new work until reconciliation completes** (a failed reconcile keeps
  it unschedulable); both directive cases (no receipt → stays `outcome_unknown`; receipt
  → `reconciled`); pending events re-emitted with original ids, known events NOT
  re-sent; server restart resumes from the durable inbox; cursor-ahead refusal; poll
  tail; version mismatch (400) + forged field (422) rejected; double emission → one
  receipt.
- [x] **NO REGRESSION** — `make check` → `cargo fmt --all -- --check` clean + `cargo clippy
  --all-targets --all-features -- -D warnings` no warnings + `cargo test --all` →
  `24 passed; 0 failed; 1 ignored` (core) + `17 passed` + `6 passed` + `10 passed` (node)
  + `13 passed` + `5 passed` + `7 passed` (server, skip offline); `bash scripts/run_pg_tests.sh`
  → `test result: ok. 5 passed` (atomic) + `test result: ok. 7 passed` (outbox worker) +
  `test result: ok. 13 passed` (node channel) on live PostgreSQL 16.15; `make gate` →
  `=== all doctrines green ===` (13/13); `make deny` → `advisories ok, bans ok, licenses ok,
  sources ok` (axum + reqwest trees — no new allowances needed); `make secret-scan` →
  `no leaks found`; `make book` → HTML written.
- [x] **FIX** — new `crates/reasonbraid-server/src/node_channel.rs` (+ lib.rs exports +
  axum/serde deps + `reqwest` dev-dep); `migrations/0003_node_inbox.sql`;
  `crates/reasonbraid-node/src/channel.rs` + `src/node.rs` (+ lib.rs exports + reqwest
  dep); node journal migration `0002_node_channel.sql` + `last_acked_cursor` /
  `set_last_acked_cursor` / `pending_operations` / `operation_ids` /
  `acknowledge_known_event` APIs (+ `EventSummary.payload`); new
  `crates/reasonbraid-server/tests/node_channel.rs` (13 tests);
  `scripts/run_pg_tests.sh` + the CI `pg-tests` job now run all three suites; `Cargo.lock`
  updated.
- [x] **LOCKSTEP** — decision record `docs/decisions/2026-09-06_node-channel.md`
  (`answers:` present, measured behavior + rejected designs) + INDEX row;
  `knowledge-map/subsystems.md` rows updated (server gains the channel; node gains the
  channel + facade); the mdBook gains `docs/book/src/node-channel.md` + its SUMMARY entry;
  `docs/ci.md` notes the channel suite in the `pg-tests` job; `CHANGELOG.md` /
  `DEV_NOTES.md` / `LIVE_STATUS.md` / `MEMORY.md` updated; `docs/TASK_TREE.md` frontier
  moved to `PHASE-0.4.1`; README unchanged (no new standard command).

## Acceptance Checklist (PHASE-0.4.1)

The `crates/reasonbraid-adapter` crate (`.rs` + `Cargo.toml` + `fixtures/`), the
`crates/reasonbraid-node` supervisor + journal API (`.rs` + `Cargo.toml`), the
`crates/reasonbraid-core` machine edge (`.rs`), and `Cargo.lock` are the CODE change
owned by this leaf (per `.doctrine/code_paths.txt`: `crates/`; `Cargo.toml`/`Cargo.lock`
are code by the ownership check). Enforced by the `TASK-ACCEPTANCE` doctrine.

- [x] **ROOT CAUSE (WHY + WHERE)** — KICKOFF WP4 requires a fake adapter that "can
  stream, fail, hang, report usage, ignore cancellation, and simulate a lost response
  after dispatch", with "dispatch acknowledgement distinct from completion" and
  "unsupported status lookup produces `outcome_unknown`, not a retry recommendation".
  Before this leaf, `git ls-files 'crates/*'` → core/server/node only — no adapter
  crate, no contract, no supervisor existed (nothing could invoke an adapter behind the
  journal boundary), and the core machine could not express "conservative dispatch
  record, then the adapter CERTIFIES no dispatch ever began": the `.3.1` boundary rule
  records `dispatched` before `invoke`, but the `fail_before_dispatch` edge existed only
  from `prepared`.
- [x] **ADDRESSED (verified)** — landed `crates/reasonbraid-adapter` (the
  capability-declaring `Adapter` contract with ack ≠ completion, no credential field;
  the deterministic scripted `FakeAdapter` — no sleeps, hang = cancellation Notify;
  the ten-fixture sanitized corpus) and `crates/reasonbraid-node/src/supervisor.rs`
  (`execute_attempt` journals every boundary and lands the attempt on its honest
  terminal). `cargo test -p reasonbraid-adapter` →
  `test result: ok. 12 passed` (fake behaviors: refusal-without-handle, determinism,
  hang-until-confirmed-cancel, ignored/best-effort cancels, lost response, both lookup
  modes, usage confidence, malformed verbatim) + `test result: ok. 3 passed` (corpus
  integrity: every step/outcome covered, mechanical credential scan, well-formedness);
  `cargo test -p reasonbraid-node` → `test result: ok. 8 passed` (`supervisor_fake`:
  the corpus drives each outcome to its expected journal terminal; lost response
  without lookup → `outcome_unknown` whose error text contains no "retry"; proven
  lookup → `completed` with the provider handle attached; hang → confirmed cancel;
  ack ≠ completion proven at the journal boundary via a signaling adapter).
- [x] **NO REGRESSION** — `make check` → fmt clean + `cargo clippy --all-targets
  --all-features -- -D warnings` no warnings + `cargo test --all` → `24 passed; 0
  failed; 1 ignored` (core) + `3 passed` + `12 passed` (adapter) + `17 passed` +
  `8 passed` + `6 passed` + `10 passed` (node) + `13 passed` + `5 passed` + `7 passed`
  (server, skip offline); `bash scripts/run_pg_tests.sh` → `5 passed` + `7 passed` +
  `13 passed` on live PostgreSQL 16.15; `make gate` → `=== all doctrines green ===`
  (13/13); `make deny` → `advisories ok, bans ok, licenses ok, sources ok`;
  `make secret-scan` → `no leaks found`; `make book` → HTML written.
- [x] **FIX** — new `crates/reasonbraid-adapter` (src/lib.rs + src/contract.rs +
  src/fake.rs + src/fixtures.rs + 10 `fixtures/*.json` + tests/fake_adapter.rs +
  Cargo.toml); `crates/reasonbraid-node/src/supervisor.rs` +
  `Journal::attach_provider_request_id`; `crates/reasonbraid-core/src/state.rs` gains
  the proof-gated `(dispatched, fail_before_dispatch) → failed_before_dispatch` edge;
  `Cargo.lock` updated.
- [x] **LOCKSTEP** — decision record `docs/decisions/2026-09-06_fake-adapter.md`
  (`answers:` present, measured behavior + rejected designs, records the three bugs the
  corpus/probes caught) + INDEX row; `knowledge-map/subsystems.md` gains the
  `reasonbraid-adapter` row and the node row mentions the supervisor; the mdBook gains
  `docs/book/src/adapter-boundary.md` + its SUMMARY entry; `docs/ci.md` notes the
  adapter tests run in plain CI; `CHANGELOG.md` / `DEV_NOTES.md` / `LIVE_STATUS.md` /
  `MEMORY.md` updated; `docs/TASK_TREE.md` frontier moved to `PHASE-0.4.2`; README
  unchanged (no new standard command).

## Acceptance Checklist (PHASE-0.4.2)

The `crates/reasonbraid-adapter` Codex adapter (`.rs` + `Cargo.toml`), the contract's
`ProviderRequestId` event (`.rs`), the `crates/reasonbraid-node` supervisor handling +
`attempt_summary` API (`.rs`), the two new test suites, and `Cargo.lock` are the CODE
change owned by this leaf (per `.doctrine/code_paths.txt`). Enforced by the
`TASK-ACCEPTANCE` doctrine.

- [x] **ROOT CAUSE (WHY + WHERE)** — KICKOFF WP4 requires "implement one real adapter —
  Codex-family or Claude-family — using the narrowest supported CLI/SDK boundary
  available on the selected development host", with "dispatch acknowledgement distinct
  from completion" and "unsupported status lookup produces `outcome_unknown`, not a
  retry recommendation". Before this leaf, `git ls-files 'crates/reasonbraid-adapter/*'`
  → contract + fake + fixtures only — no real adapter existed, and the contract could
  not express a provider handle that arrives AFTER dispatch (Codex reveals its thread
  id in the stream, not the ack): `AttemptEvent` had no `ProviderRequestId` variant.
- [x] **ADDRESSED (verified)** — landed `src/codex.rs` (`CodexCliAdapter` supervising
  `codex exec --json`; streamed thread id → `ProviderRequestId`; non-zero exit →
  `failed_known` with the stderr tail; `query_status` → `Unsupported`; `BestEffort`
  cancel; token receipts → exact usage with cost `None`). The LIVE qualification passed:
  `RB_LIVE_CODEX=1 cargo test -p reasonbraid-node --test codex_live -- --ignored
  --nocapture` → `test result: ok. 1 passed` (a real dispatch through the real
  supervisor + journal; thread id attached; exact usage; honest unsupported lookup).
  Offline: `cargo test -p reasonbraid-adapter --test codex_adapter` →
  `test result: ok. 9 passed` (stub binary: spawn refusal, JSONL parsing, chunk order,
  exit-status verdicts, lost response, kill → BestEffort, receipt shapes) and
  `cargo test -p reasonbraid-node --test supervisor_codex_stub` →
  `test result: ok. 2 passed` (completion attaches the streamed handle; lost response →
  `outcome_unknown` with an error carrying no "retry").
- [x] **NO REGRESSION** — `make check` → fmt clean + clippy no warnings + `cargo test
  --all` → `24 passed; 0 failed; 1 ignored` (core) + `3 passed` + `12 passed` +
  `9 passed` (adapter) + `17 passed` + `8 passed` + `6 passed` + `10 passed` +
  `2 passed` + `1 ignored` (node) + `13 passed` + `5 passed` + `7 passed` (server,
  skip offline); `make gate` → `=== all doctrines green ===` (13/13); `make deny` →
  `advisories ok, bans ok, licenses ok, sources ok`; `make secret-scan` →
  `no leaks found`; `make book` → HTML written.
- [x] **FIX** — new `crates/reasonbraid-adapter/src/codex.rs` (+ lib.rs export + tokio
  `io-util`/`process`/`rt` features); `crates/reasonbraid-adapter/src/contract.rs` gains
  `AttemptEvent::ProviderRequestId`; `crates/reasonbraid-node/src/supervisor.rs` attaches
  streamed handles; `crates/reasonbraid-node/src/journal.rs` gains `attempt_summary`;
  new `tests/codex_adapter.rs`, `tests/supervisor_codex_stub.rs`, `tests/codex_live.rs`
  (env-gated live); `Cargo.lock` updated.
- [x] **LOCKSTEP** — decision record `docs/decisions/2026-09-06_real-adapter-codex.md`
  (`answers:` present, measured behavior + rejected designs + the director-owned open
  question) + INDEX row; evidence report
  `docs/evidence/2026-09-06_codex-adapter-qualification.md` (`reported`) +
  `docs/evidence/INDEX.md` row (the WP4 acceptance's evidence-report leg);
  `docs/dependencies/external-ledger.yaml` Codex row revalidated (its trigger fired:
  `checked_at 2026-09-06`, `tested_versions [0.153.4]`, `license Apache-2.0` verified
  from the primary source); the mdBook adapter chapter gains the real-adapter section;
  `docs/ci.md` notes the gated live test; `CHANGELOG.md` / `DEV_NOTES.md` /
  `LIVE_STATUS.md` / `MEMORY.md` updated; `docs/TASK_TREE.md` frontier moved to
  `PHASE-0.5.1`; README unchanged.

## Acceptance Checklist (PHASE-0.5.1)

The `crates/reasonbraid-core` authority module (`.rs` + `Cargo.toml`), the
`crates/reasonbraid-server` authority engine + tx refactor (`.rs` + `Cargo.toml`), the
repository-root `migrations/0004_authority.sql`, the new `tests/authority.rs`,
`scripts/run_pg_tests.sh` + `.github/workflows/rust.yml`, and `Cargo.lock` are the CODE
change owned by this leaf (per `.doctrine/code_paths.txt`). Enforced by the
`TASK-ACCEPTANCE` doctrine.

- [x] **ROOT CAUSE (WHY + WHERE)** — `ROADMAP.md` §4.4/§4.5 + KICKOFF WP5 require
  "tenant membership alone does not grant mandate or administrative authority", "every
  command records actor, subject if delegated, grant/boundary reference, decision, and
  policy digest/version", and "a grant cannot exceed the enrollment ceiling". Before
  this leaf, `git ls-files 'crates/*'` → core had NO authority types and the server had
  NO authorization path — `apply_command` (`.2.1`) accepted every command with no actor,
  no grant check, and no audit row, so any principal could write any aggregate.
- [x] **ADDRESSED (verified)** — landed the core authority model
  (`EnrollmentAuthorityBoundary`, `AuthorityGrant`, scoped actions/selectors, the
  deterministic `grant_exceeds_boundary` subset checker, liveness helpers, the
  SHA-256 `policy_digest`, `AuthorizationDecisionRecord`) and the server engine
  (`create_boundary`, `create_grant` — refused when overreaching, `authorize` writing
  the audit row for allowances AND denials, `apply_authorized_command` running the
  record + the `.2.1` writes in ONE transaction via the extracted
  `apply_command_in_tx`). `bash scripts/run_pg_tests.sh` →
  `test result: ok. 9 passed` (`authority`) against live PostgreSQL 16.15 —
  membership-without-grant denied AND audited with no domain effect; admin never
  implied; an accepted command's record carries actor + delegated subject + grant +
  boundary + allowed decision + a 64-hex digest that RE-DERIVES from the same inputs;
  overreaching grants refused at creation (nothing stored); scope denials; expired
  grants denied; boundary-less tenant denied; identical evaluations digest identically.
  `cargo test -p reasonbraid-core` → `test result: ok. 31 passed`.
- [x] **NO REGRESSION** — `make check` → fmt clean + `cargo clippy --all-targets
  --all-features -- -D warnings` no warnings + `cargo test --all` → all 20 suites green
  (`31 passed` core; adapter `3 + 12 + 9`; node `17 + 8 + 6 + 10 + 2 + 1 ignored`;
  server `9 + 13 + 5 + 7` skip offline); `bash scripts/run_pg_tests.sh` →
  `5 passed` + `7 passed` + `13 passed` + `9 passed` on live PostgreSQL 16.15;
  `make gate` → `=== all doctrines green ===` (13/13); `make deny` →
  `advisories ok, bans ok, licenses ok, sources ok` (sha2 as a direct core dep);
  `make secret-scan` → `no leaks found`; `make book` → HTML written.
- [x] **FIX** — new `crates/reasonbraid-core/src/authority.rs` (+ sha2/chrono deps +
  lib.rs exports); new `crates/reasonbraid-server/src/authority.rs` (+ the core dep +
  lib.rs exports); `crates/reasonbraid-server/src/tx.rs` extracts `apply_command_in_tx`
  (public `apply_command` unchanged — its 5 tests stay green); new
  `migrations/0004_authority.sql`; new `crates/reasonbraid-server/tests/authority.rs`
  (9 tests); `scripts/run_pg_tests.sh` + the CI `pg-tests` job run the fourth suite;
  `Cargo.lock` updated.
- [x] **LOCKSTEP** — decision record `docs/decisions/2026-09-06_authority-boundary.md`
  (`answers:` present, measured behavior + rejected designs) + INDEX row; the mdBook
  gains `docs/book/src/authority.md` + its SUMMARY entry; `knowledge-map/subsystems.md`
  rows updated (core gains the authority model; server gains the engine);
  `docs/ci.md` notes the fourth PG suite; `CHANGELOG.md` / `DEV_NOTES.md` /
  `LIVE_STATUS.md` / `MEMORY.md` updated; `docs/TASK_TREE.md` frontier moved to
  `PHASE-0.5.2`; README unchanged.

## Acceptance Checklist (PHASE-0.5.2)

The `crates/reasonbraid-core` budget module (`.rs`), the `crates/reasonbraid-server`
budget engine (`.rs`), the repository-root `migrations/0005_budget.sql`, the
`crates/reasonbraid-node` supervisor gate + `LocalBudget` (`.rs`), the new test suites,
`scripts/run_pg_tests.sh` + `.github/workflows/rust.yml`, and `Cargo.lock` are the CODE
change owned by this leaf (per `.doctrine/code_paths.txt`). Enforced by the
`TASK-ACCEPTANCE` doctrine.

- [x] **ROOT CAUSE (WHY + WHERE)** — `ROADMAP.md` §14.3/§14.6 + KICKOFF WP5 require "no
  provider dispatch begins without an applicable budget reservation" and "denial tests
  cross both server and node boundaries". Before this leaf,
  `git ls-files 'crates/*' | grep -i budget` → nothing: the `.4.1` supervisor
  dispatched the adapter with NO reservation check on either side — a ceiling could
  neither hold nor deny anything.
- [x] **ADDRESSED (verified)** — landed the core `BudgetDimensions` model (fail-closed
  `covers`, total fallible `add`/`subtract`, `ReservationReference.applicable()`), the
  server engine (`create_reservation` atomic against held, denial rows,
  `settle_reservation` with overrun reporting, `release_reservation`, expiry via the
  caller's clock), and the node gate (`execute_attempt` requires a reservation +
  `LocalBudget` headroom BEFORE the dispatch boundary; refusals journaled
  `failed_before_dispatch`; settlement with actual usage; indeterminate attempts keep
  their hold). `bash scripts/run_pg_tests.sh` → `test result: ok. 5 passed` (`budget`)
  against live PostgreSQL 16.15 — within-ceiling holds / beyond-ceiling denied AND
  recorded; settlement frees the unused remainder; release frees everything; expired
  reservations stop holding; overruns reported, never clamped, double-settlement a
  no-op. `cargo test -p reasonbraid-node --test supervisor_budget` →
  `test result: ok. 4 passed` — a reservation covering no dispatch is refused BEFORE
  the boundary record (a counting adapter proves it was never invoked); exhausted
  local headroom refused with the reason journaled; completed attempts settle ACTUAL
  usage (100 tokens, not the 5000 hold); an indeterminate attempt KEEPS its hold.
  `cargo test -p reasonbraid-core` → `test result: ok. 35 passed`.
- [x] **NO REGRESSION** — `make check` → fmt clean + `cargo clippy --all-targets
  --all-features -- -D warnings` no warnings + `cargo test --all` → all 22 suites green
  (core `35 passed`; adapter `3 + 12 + 9`; node `17 + 8 + 6 + 10 + 2 + 4 + 1 ignored`;
  server `9 + 5 + 13 + 5 + 7` skip offline); `bash scripts/run_pg_tests.sh` →
  `5 + 9 + 5 + 13 + 7 passed` on live PostgreSQL 16.15; `make gate` →
  `=== all doctrines green ===` (13/13); `make deny` → `advisories ok, bans ok,
  licenses ok, sources ok`; `make secret-scan` → `no leaks found`; `make book` →
  HTML written.
- [x] **FIX** — new `crates/reasonbraid-core/src/budget.rs` (+ lib.rs exports); new
  `crates/reasonbraid-server/src/budget.rs` (+ lib.rs exports); new
  `migrations/0005_budget.sql`; `crates/reasonbraid-node/src/supervisor.rs` gains the
  reservation gate + `LocalBudget` + `ExecutionReport.reservation_id`; all supervisor
  call sites updated (fake/codex-stub/live suites); new
  `crates/reasonbraid-server/tests/budget.rs` (5) +
  `crates/reasonbraid-node/tests/supervisor_budget.rs` (4);
  `scripts/run_pg_tests.sh` + the CI `pg-tests` job run the fifth suite; `Cargo.lock`
  updated.
- [x] **LOCKSTEP** — decision record `docs/decisions/2026-09-06_budget-reservation.md`
  (`answers:` present, measured behavior + rejected designs) + INDEX row; the mdBook
  gains `docs/book/src/budget.md` + its SUMMARY entry; `knowledge-map/subsystems.md`
  rows updated (core budget model; server budget engine; node supervisor gate);
  `docs/ci.md` notes the fifth PG suite; `CHANGELOG.md` / `DEV_NOTES.md` /
  `LIVE_STATUS.md` / `MEMORY.md` updated; `docs/TASK_TREE.md` frontier moved to
  `PHASE-0.6.1`; README unchanged.

## Acceptance Checklist (PHASE-0.6.1)

The `crates/reasonbraid-server` control API + thread domain + `rb-server` binary
(`.rs` + `Cargo.toml`), the repository-root `migrations/0006_control_api.sql`, the new
`crates/reasonbraid-cli` crate, the two new test suites, `scripts/run_pg_tests.sh` +
`.github/workflows/rust.yml`, and `Cargo.lock` are the CODE change owned by this leaf
(per `.doctrine/code_paths.txt`). Enforced by the `TASK-ACCEPTANCE` doctrine.

- [x] **ROOT CAUSE (WHY + WHERE)** — `KICKOFF.md` WP6 / issue 12 requires "implement CLI
  flow: enroll, create thread, invite, contribute, challenge, revise, close, inspect"
  with the acceptance "no database surgery required to inspect state". Before this leaf,
  `git ls-files 'crates/*'` → core/server/node/adapter only — no CLI crate; the server
  exposed ONLY the node channel (no control API, no `rb-server` binary); the `.2.1`
  transaction recorded generic rows but NO thread operation catalogue existed
  (`apply_command` took pre-built `Command`s; nothing validated a thread lifecycle), so
  a human could not create, drive, or inspect a thread except through psql.
- [x] **ADDRESSED (verified)** — landed the thread domain (`threads.rs`: the six
  operations with typed `deny_unknown_fields` bodies, the projection in
  `aggregate_state.state`, core-machine validation, dev rules — creator seated,
  auto-accept on first contribution, close fold, challenge/revise target checks), the
  control API (`api.rs`: enroll bootstrap in one transaction, `/v1/threads`
  command/query/audit surface, the trusted dev principal header + deterministic
  UUIDv5 actor handle, SHA-256 request hash, one-transaction
  claim→authorize→prepare→apply flow with idempotent REJECTIONS), `rb-server`, and
  the `rb` CLI (all eight verbs; repo-local state dir). `bash scripts/run_pg_tests.sh`
  → `test result: ok. 7 passed` (`command_api`: full flow with the ordered 6-event
  timeline + 8 digest-carrying audit records, denials recorded and effect-free,
  replay-vs-conflict, invalid transitions, forged-field rejection, header checks,
  challenge targets) + `test result: ok. 2 passed` (`cli_end_to_end`: the REAL binary
  drives the whole flow and inspects through its own stdout; typed denials + exit 1).
  All six live-PG suites: `5 + 7 + 13 + 9 + 5 + 7 passed` on PostgreSQL 16.15.
- [x] **NO REGRESSION** — `make check` → fmt clean + `cargo clippy --all-targets
  --all-features -- -D warnings` no warnings + `cargo test --all` → all 28 suites green
  offline (core `36 passed; 1 ignored`; adapter `3 + 12 + 9`; node `17 + 8 + 6 + 10 +
  4 + 2`; server offline-skips `5 + 9 + 5 + 7 + 13 + 7`; cli `3` unit + `2` e2e-skips);
  the `.2.1`/`.5.1`/`.5.2` refactors (claim/apply split, executor-generic authority +
  ceiling writers) left their public behavior untouched — their live suites stay green
  (`5 + 9 + 5 passed`); `make gate` → `=== all doctrines green ===` (13/13); `make deny`
  → `advisories ok, bans ok, licenses ok, sources ok`; `make secret-scan` →
  `no leaks found`; `make book` → HTML written.
- [x] **FIX** — new `crates/reasonbraid-server/src/threads.rs` + `src/api.rs` +
  `src/bin/rb-server.rs` + `tests/command_api.rs` (7 tests); server `tx.rs` splits
  `claim_idempotency_in_tx` / `apply_fresh_in_tx` (public `apply_command` unchanged);
  `authority.rs` + `budget.rs` gain executor-generic in-tx variants +
  `load_active_boundary_for_tenant`; new `migrations/0006_control_api.sql`; new
  `crates/reasonbraid-cli` (src/lib.rs + src/main.rs + tests/cli_end_to_end.rs);
  core gains `GrantAction::ThreadClose` + `actor_handle_for_subject` (uuid `v5`
  feature); `scripts/run_pg_tests.sh` + the CI `pg-tests` job run the two new suites;
  `Cargo.lock` updated.
- [x] **LOCKSTEP** — decision record `docs/decisions/2026-09-06_control-api-cli.md`
  (`answers:` present, measured behavior + rejected designs + the two falsified bugs)
  + INDEX row; the mdBook gains `docs/book/src/cli.md` + its SUMMARY entry;
  `knowledge-map/subsystems.md` rows updated (server gains the thread domain + control
  API + binary; the new CLI crate row); `docs/ci.md` documents the two new suites;
  `CHANGELOG.md` / `DEV_NOTES.md` / `LIVE_STATUS.md` / `MEMORY.md` updated;
  `docs/TASK_TREE.md` frontier moved to `PHASE-0.6.2`; README unchanged (no new
  standard command — the CLI is documented in the book).

## Acceptance Checklist (PHASE-0.6.2)

The `crates/` + `scripts/` + `Makefile` + `.github/workflows/rust.yml` changes are the
CODE change owned by this leaf (per `.doctrine/code_paths.txt`). Enforced by the
`TASK-ACCEPTANCE` doctrine.

- [x] **ROOT CAUSE (WHY + WHERE)** — KICKOFF WP6 (`issue 13`) and `ROADMAP.md` §26.1
  require the two-host demonstration: the `.6.1` surface had no path from an invitation
  to a node (no inbox dispatch — `node_inbox` rows were only written by tests), no path
  from a node result to a thread event (the channel's `events` handler stored receipts
  only), and no worker binary to execute work items (the node crate had only the
  read-only `rb-journal`). Each WP6 acceptance point was therefore unprovable.
- [x] **ADDRESSED (verified)** — `bash scripts/run_pg_tests.sh` →
  `test result: ok. 6 passed` (`node_work`) + `5 + 9 + 5 + 7 + 13 + 7` (all server
  suites) + `2 passed` (`cli_end_to_end`) on live PostgreSQL 16.15; then
  `scripts/demo_two_host.sh` → **every acceptance check PASS** (contribution via the
  node channel, duplicate delivery refused with one domain effect, server SIGKILL
  restart with nothing lost, node SIGKILL after dispatch → `outcome_unknown` bounded
  and never retried, budget denial at both boundaries, closure preserving the
  contribution + the unresolved challenge) with the evidence bundle under
  `target/demo/<run-id>/`.
- [x] **NO REGRESSION** — `make check` → all 29 offline suites green; `make gate` →
  `=== all doctrines green ===` (13/13); `make deny` → advisories/bans/licenses/sources
  ok; `make secret-scan` → no leaks; `make book` → HTML written. The WP2/WP3/WP5
  channel/authority/budget suites (5+7+13+9+5) still pass unchanged on live PG — the
  in-tx refactors (`create_reservation_in_tx`, `settle_reservation_in_tx`,
  `enqueue_in_tx`, `record_event_in_tx`, `load_command_in_tx`) changed no pool-level
  behavior.
- [x] **FIX** — the dispatch hook + `apply_node_result_in_tx` in `api.rs`; the in-tx
  channel/budget variants; `threads::work_payload`/`event_author_in_thread`;
  `worker.rs` + the `rb-node` binary; `journal::work_items`/`emitted_events` +
  `rb-journal events`; `tests/node_work.rs`; `scripts/demo_two_host.sh`; harness/CI/
  Makefile wiring (`make demo`).
- [x] **LOCKSTEP** — decision record `docs/decisions/2026-09-07_node-channel-wiring.md`
  + INDEX row; mdBook chapter `docs/book/src/two-host-demo.md` + SUMMARY; CHANGELOG,
  DEV_NOTES, MEMORY, LIVE_STATUS, `docs/ci.md`, `knowledge-map/subsystems.md` updated;
  this tree's log below.

## Acceptance Checklist (PHASE-0.7)

The `crates/reasonbraid-adapter` code changes (`.rs` + `Cargo.toml`) are the CODE
change owned by this leaf (per `.doctrine/code_paths.txt`). Enforced by the
`TASK-ACCEPTANCE` doctrine.

- [x] **ROOT CAUSE (WHY + WHERE)** — KICKOFF WP7 (`issue 14`) and `ROADMAP.md`
  §13.7/H1/H6 require the small versioned benchmark before the Phase 1 routing
  decision: no harness existed to compare single-agent vs blind-independent vs
  critique/revise vs moderator/synthesis with per-case scores, confidence, and
  cost. The WP8 memo (`PHASE-0.8`) cannot be written without it.
- [x] **ADDRESSED (verified)** — `cargo test -p reasonbraid-adapter` →
  `7 passed` (grader) + `5 passed` (`bench_harness` — the scripted run
  reproduces the corpus oracle EXACTLY over 8 cases × 4 workflows: computed ==
  expected scores, call accounting, structure validity, unresolved-register
  fidelity, the honesty trap, no-independence-score report hygiene, factual-only
  Brier, and the critique/revision prompt-split guard) + `9` + `12` (existing
  suites); `rb-bench --agent scripted` → full report; **real run**:
  `RB_LIVE_CODEX=1 rb-bench --agent codex --case fact-001,code-002,policy-002,
  insuff-001 --max-calls 36` → 16/16 rows structure-valid, all scores + token
  costs + per-case confidence recorded in
  `docs/evidence/2026-09-07_benchmark-codex-run.md` (result: structure did NOT
  beat single on the sample — the accepted null that narrows the claim).
- [x] **NO REGRESSION** — `make check` → all offline suites green (incl. the new
  bench tests); `make gate` → `=== all doctrines green ===` (13/13); `make deny`
  → advisories/bans/licenses/sources ok (regex/sha2/clap added to the adapter);
  `make secret-scan` → no leaks; `make book` → HTML written. Existing adapter
  suites (fake 12, codex stub 9, corpus integrity) unchanged and green.
- [x] **FIX** — `src/bench/` (corpus + grader + scripted agent + workflows +
  report), `src/bin/rb-bench.rs`, `bench/v1/{corpus,prompts}.json`,
  `tests/bench_harness.rs`; two REAL-RUN-CAUGHT defects fixed with regression
  tests (the critique/revision template conflation; the honesty trap's
  echoed-number false positive).
- [x] **LOCKSTEP** — decision record `docs/decisions/2026-09-07_deliberation-benchmark.md`
  + INDEX row; evidence report `docs/evidence/2026-09-07_benchmark-codex-run.md`
  + INDEX row; mdBook `benchmark.md` + SUMMARY; CHANGELOG, DEV_NOTES, MEMORY,
  LIVE_STATUS, `knowledge-map/subsystems.md` updated; this tree's log below.

## Acceptance Checklist (PHASE-0.8.1)

Documents + registers only — no `.rs`/`.sh`/`Makefile` code change in this leaf
(per `.doctrine/code_paths.txt`; the gate's evidence is the WP1–WP7 suites,
cited, not re-run into new tooling).

- [x] **ROOT CAUSE (WHY + WHERE)** — KICKOFF WP8 (`issue 15`) + `ROADMAP.md`
  §20.2/§19.8: Phase 0 cannot exit without the evidence manifest, the ADR
  decisions for the seven required topics, the SubtractionRecord, a refreshed
  risk register, and an explicit go/rework/pivot/stop — none of which existed
  as a gate package (they were scattered across leaves).
- [x] **ADDRESSED (verified)** — `docs/evidence/2026-09-07_phase0-evidence-manifest.md`
  maps each G0 boundary (identity/authority/thread/delivery/budget) to its
  suites and commands (all previously run and recorded: 5+9+5+7+13+7+6 live-PG,
  2 CLI e2e, 12 demo checks, 8×4 benchmark oracle, 36 real Codex calls);
  `docs/decisions/2026-09-07_phase0-adr-set.md` maps every required ADR topic to
  its accepted record; the SubtractionRecord carries non-empty lists throughout
  (§19.8); ADR-002 (proposed) recommends GO; `docs/risks.md` refreshed (two new
  rows from the `.7` run); the 2×-estimate review is NOT triggered (≈9.5
  engineer-weeks vs the 8–14 range — arithmetic recorded in the SubtractionRecord).
- [x] **NO REGRESSION** — `make check` → all offline suites green (no code
  changed); `make gate` → `=== all doctrines green ===` (13/13); `make deny` →
  advisories/bans/licenses/sources ok; `make secret-scan` → no leaks; `make
  book` → HTML written.
- [x] **FIX** — the four gate documents + ADR INDEX + evidence INDEX + decision
  INDEX + the risk-register refresh (R-AMB disposition, R-VALUE narrowing,
  R-VARIANCE/R-OVERHEAD additions).
- [x] **LOCKSTEP** — CHANGELOG, DEV_NOTES, MEMORY, LIVE_STATUS, this tree's log
  below, `docs/TASK_TREE.md` frontier. The one out-of-band item — the owner's
  signature on ADR-002 — is recorded in Blockers (an agent drafts the package;
  the accountable owner closes the gate).

## Verification Log



| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-09-05` | (tree created) | mapped from `KICKOFF.md` WP0–WP8 and issues 1–15 | pending execution |
| `2026-09-05` | `PHASE-0.0.1` | ADR-001 written; README landing page; `wc -lc README.md` under README-STABILITY caps | working name internal-only |
| `2026-09-05` | `PHASE-0.0.2` | `test -f docs/adr/TEMPLATE.md docs/evidence/TEMPLATE.md docs/adr/INDEX.md docs/evidence/INDEX.md` | templates present |
| `2026-09-05` | `PHASE-0.0.3` | `test -f docs/parking-lot.md`; table has Idea / Why not now / Revisit trigger / Date parked | file exists |
| `2026-09-05` | `PHASE-0.0.4` | `test -f docs/risks.md`; seven live rows with owner role + stop trigger | not a copy of §25 |
| `2026-09-06` | `PHASE-0.0.5` | `ruby -ryaml -e 'YAML.load_file(...)'` → `entries=4`, `SCHEMA OK`; MCP/A2A/Codex/Claude rows carry name, owner, source_url, checked_at, versions, revalidation_trigger | ledger skeleton created |
| `2026-09-06` | `PHASE-0.0.6` | `test -f docs/decisions/2026-09-06_accountable-owners.md`; names Richard DJE for both roles; INDEX row added; risks.md owner-roles note resolved | owners named |
| `2026-09-06` | `PHASE-0.0.7` | `make -n deny`→`cargo deny check`; `make -n secret-scan`→`gitleaks detect --source . --redact`; `make gate` 13/13; `make check` 1 test ok; `deny.toml`+`supply-chain.yml`+`docs/ci.md` present | supply-chain skeleton; no release claim |
| `2026-09-06` | `PHASE-0.0.8` | `test -f spec/{README,glossary,requirements,lifecycle,threat-model}.md spec/governance/charter.md`; five G0 ID prefixes (ID/AUTH/THREAD/DELIV/BUDGET) assigned in `spec/requirements.md`; threat-model lists 11 trust boundaries; charter names Richard DJE as bootstrap human root; decision record `2026-09-06_g0-contract-id-scheme.md` + INDEX row | G0 contract drafts, all "draft — not normative" |
| `2026-09-06` | `PHASE-0.1.1` | `cargo test -p reasonbraid-core` → `test result: ok. 6 passed; 0 failed`; `make check` → fmt clean + `cargo clippy --all-targets --all-features -- -D warnings` no warnings + `cargo test --all` 6 passed; `make gate` → `=== all doctrines green ===` (13/13); eight ID newtypes pairwise `TypeId`-distinct; decision record `2026-09-06_id-representation.md` + INDEX row | strong IDs landed; first real crate |
| `2026-09-06` | `PHASE-0.1.2` | `cargo test -p reasonbraid-core` → `test result: ok. 10 passed; 0 failed; 1 ignored`; `make check` → fmt clean + clippy no warnings + `cargo test --all` 10 passed; `make gate` → `=== all doctrines green ===` (13/13); golden fixtures round-trip, forged-authority fixture rejected, schema goldens in sync; decision record `2026-09-06_envelope-representation.md` + INDEX row | command/event envelopes landed; client forgery rejected |
| `2026-09-06` | `PHASE-0.1.3` | `cargo test -p reasonbraid-core` → `test result: ok. 17 passed; 0 failed; 1 ignored`; `make check` → fmt clean + clippy no warnings + `cargo test --all` 17 passed; `make gate` → `=== all doctrines green ===` (13/13); three state machines reject invalid transitions deterministically; decision record `2026-09-06_state-transitions.md` + INDEX row | minimal state machines landed; `ProviderAttemptId` (`patt`) added |
| `2026-09-06` | `PHASE-0.1.4` | `cargo test -p reasonbraid-core` → `test result: ok. 23 passed; 0 failed; 1 ignored`; `make check` → fmt clean + clippy no warnings + `cargo test --all` 23 passed; `make gate` → `=== all doctrines green ===` (13/13); §9.8 registry round-trips, unknown code preserved verbatim; decision record `2026-09-06_reason-codes.md` + INDEX row | typed errors + reason-code registry landed (WP1 complete) |
| `2026-09-06` | `PHASE-0.2.1` | `bash scripts/run_pg_tests.sh` → `test result: ok. 5 passed; 0 failed; 0 ignored` (live PostgreSQL 16.15); `make check` → fmt clean + clippy no warnings + `cargo test --all` 23 core + 5 server (skip offline); `make gate` → `=== all doctrines green ===` (13/13); `make deny` → advisories/bans/licenses/sources ok; `make secret-scan` → `no leaks found`; decision record `2026-09-06_atomic-transaction.md` + INDEX row | WP2 atomic transaction proven: 4 tables one transaction, claim-first idempotency, replay vs conflict |
| `2026-09-06` | `PHASE-0.2.2` | `bash scripts/run_pg_tests.sh` → `test result: ok. 7 passed; 0 failed` (`outbox_worker`) + `5 passed` (`atomic_transaction`) on live PostgreSQL 16.15 — exclusive claim, reclaim-after-expiry with new token, stale worker refused after newer fencing value, expired lease refused, kill points 3/4/5 to one effect; `make check` → fmt clean + clippy no warnings + `cargo test --all` 23 core + 5 + 7 server (skip offline); `make gate` → `=== all doctrines green ===` (13/13); `make deny` → advisories/bans/licenses/sources ok (chrono added); `make secret-scan` → `no leaks found`; `make book` → HTML written; decision record `2026-09-06_outbox-worker-fencing.md` + INDEX row | WP2 leased outbox worker proven: claim → deliver → complete with per-claim fencing tokens; **WP2 complete** |
| `2026-09-06` | `PHASE-0.3.1` | `cargo test -p reasonbraid-node` → `test result: ok. 13 passed` (journal) + `test result: ok. 6 passed` (CLI) + `test result: ok. 10 passed` (kill points KP-1…KP-9 + end-to-end); `cargo test -p reasonbraid-core` → `test result: ok. 24 passed; 0 failed; 1 ignored`; `make check` → fmt clean + clippy no warnings + `cargo test --all` 24 core + 13 + 6 + 10 node + 5 + 7 server (skip offline); `bash scripts/run_pg_tests.sh` → `5 passed` + `7 passed` on live PostgreSQL 16.15; `make gate` → `=== all doctrines green ===` (13/13); `make deny` → advisories/bans/licenses/sources ok (clap + libsqlite3-sys; path dep pinned); `make secret-scan` → `no leaks found`; `make book` → HTML written; decision record `2026-09-06_node-journal.md` + INDEX row | WP3 node journal proven: WAL + synchronous=FULL recorded, boundary record precedes dispatch, honest `outcome_unknown` recovery with prove/reconcile exits, read-only `rb-journal` CLI |
| `2026-09-06` | `PHASE-0.3.2` | `bash scripts/run_pg_tests.sh` → `test result: ok. 13 passed` (`node_channel`) + `5 passed` (`atomic_transaction`) + `7 passed` (`outbox_worker`) against live PostgreSQL 16.15 over real 127.0.0.1 sockets — fresh handshake plays the whole inbox, tail-only reconnect, duplicate delivery keeps the same operation ids, schedulability gate (emit refused before reconcile; failed reconcile stays unschedulable), both reconciliation directive cases, original-id re-emission + known-event skip, server restart resume, cursor-ahead refusal, poll tail, version-mismatch/forged-field rejection, double emission → one receipt; `make check` → fmt clean + clippy no warnings + `cargo test --all` 24 core + 17 + 6 + 10 node + 13 + 5 + 7 server (skip offline); `make gate` → `=== all doctrines green ===` (13/13); `make deny` → advisories/bans/licenses/sources ok (axum + reqwest); `make secret-scan` → `no leaks found`; `make book` → HTML written; decision record `2026-09-06_node-channel.md` + INDEX row | WP3 outbound node channel proven: cursor resume + reconciliation handshake + schedulability gate; **WP3 complete** |
| `2026-09-06` | `PHASE-0.5.2` | `bash scripts/run_pg_tests.sh` → `test result: ok. 5 passed` (`budget`) + `5 + 9 + 13 + 7 passed` against live PostgreSQL 16.15 — hold/deny-and-record, settle-frees-remainder, release, expiry, overrun-reported-never-clamped; `cargo test -p reasonbraid-node --test supervisor_budget` → `test result: ok. 4 passed` (refusal before the boundary with a never-invoked adapter, local headroom denial, actual-usage settlement, indeterminate keeps its hold); `cargo test -p reasonbraid-core` → `test result: ok. 35 passed`; `make check` → fmt clean + clippy no warnings + all 22 suites green; `make gate` → `=== all doctrines green ===` (13/13); `make deny` → advisories/bans/licenses/sources ok; `make secret-scan` → `no leaks found`; `make book` → HTML written; decision record `2026-09-06_budget-reservation.md` + INDEX row | WP5 budget engine proven: reserve-before-dispatch at both boundaries; **WP5 complete** |
| `2026-09-06` | `PHASE-0.5.1` | `bash scripts/run_pg_tests.sh` → `test result: ok. 9 passed` (`authority`) + `5 passed` + `7 passed` + `13 passed` against live PostgreSQL 16.15 — membership-without-grant denied and audited with no domain effect, admin never implied, accepted commands carry actor + delegated subject + grant/boundary + decision + a re-derivable 64-hex policy digest, overreaching grants refused at creation, scope/expiry/boundary-less denials, stable digests; `cargo test -p reasonbraid-core` → `test result: ok. 31 passed`; `make check` → fmt clean + clippy no warnings + all 20 suites green; `make gate` → `=== all doctrines green ===` (13/13); `make deny` → advisories/bans/licenses/sources ok (sha2); `make secret-scan` → `no leaks found`; `make book` → HTML written; decision record `2026-09-06_authority-boundary.md` + INDEX row | WP5 authority engine proven: boundary ceiling + scoped grants + audit records in the command transaction |
| `2026-09-06` | `PHASE-0.4.2` | `RB_LIVE_CODEX=1 cargo test -p reasonbraid-node --test codex_live -- --ignored --nocapture` → `test result: ok. 1 passed` (one bounded REAL Codex dispatch through the real supervisor + journal: completed, streamed thread id attached, exact usage, honest `Unsupported` lookup); `cargo test -p reasonbraid-adapter --test codex_adapter` → `test result: ok. 9 passed` (offline stub boundary); `cargo test -p reasonbraid-node --test supervisor_codex_stub` → `test result: ok. 2 passed`; `make check` → fmt clean + clippy no warnings + `cargo test --all` 24 core + 3 + 12 + 9 adapter + 17 + 8 + 6 + 10 + 2 + 1 ignored node + 13 + 5 + 7 server (skip offline); `make gate` → `=== all doctrines green ===` (13/13); `make deny` → advisories/bans/licenses/sources ok; `make secret-scan` → `no leaks found`; `make book` → HTML written; decision record `2026-09-06_real-adapter-codex.md` + evidence report + INDEX rows | WP4 first real harness qualified (Codex-family CLI, `exec --json`); ledger revalidated; **WP4 complete** |
| `2026-09-06` | `PHASE-0.4.1` | `cargo test -p reasonbraid-adapter` → `test result: ok. 12 passed` (fake) + `test result: ok. 3 passed` (corpus integrity incl. mechanical credential scan); `cargo test -p reasonbraid-node` → `test result: ok. 8 passed` (`supervisor_fake` — corpus drives every outcome to its journal terminal; lost response without lookup → `outcome_unknown` with no retry language; proven lookup → `completed`; ack ≠ completion at the journal boundary); `make check` → fmt clean + clippy no warnings + `cargo test --all` 24 core + 3 + 12 adapter + 17 + 8 + 6 + 10 node + 13 + 5 + 7 server (skip offline); `bash scripts/run_pg_tests.sh` → `5 passed` + `7 passed` + `13 passed` on live PostgreSQL 16.15; `make gate` → `=== all doctrines green ===` (13/13); `make deny` → advisories/bans/licenses/sources ok; `make secret-scan` → `no leaks found`; `make book` → HTML written; decision record `2026-09-06_fake-adapter.md` + INDEX row | WP4 fake harness adapter proven: scripted oracle + sanitized corpus + supervisor ambiguity path; three real bugs found by the corpus/probes (boundary-vs-refusal edge, Notify race, terminal-event loop) |
| `2026-09-06` | `PHASE-0.6.1` | `bash scripts/run_pg_tests.sh` → `test result: ok. 7 passed` (`command_api`) + `test result: ok. 2 passed` (`cli_end_to_end`, the REAL `rb` binary) against live PostgreSQL 16.15 — full flow (bootstrap → create → invite → auto-accepted contribution → challenge → revise → close) inspected through the API only (ordered 6-event timeline + 8 digest-carrying audit records); denials recorded and effect-free; replay returns the original result / conflicts typed; invalid transitions deterministic; forged fields + bad headers rejected; challenge targets checked; `make check` → fmt clean + clippy no warnings + all 28 suites green offline; `make gate` → `=== all doctrines green ===` (13/13); `make deny` → advisories/bans/licenses/sources ok; `make secret-scan` → `no leaks found`; `make book` → HTML written; decision record `docs/decisions/2026-09-06_control-api-cli.md` + INDEX row; mdBook cli chapter + SUMMARY entry | WP6 control-API + CLI landed: every thread command runs claim → authorize → validate (locked projection) → apply (+ ceiling) in ONE transaction, rejections are idempotent results, and inspection never touches the database |
| `2026-09-07` | `PHASE-0.6.2` | `bash scripts/run_pg_tests.sh` → `test result: ok. 6 passed` (`node_work`) + `5 + 9 + 5 + 7 + 13 + 7` (all server suites) + `2 passed` (`cli_end_to_end`) against live PostgreSQL 16.15 — invite dispatches work WITH a reservation (public handshake view), one contribution despite duplicates at BOTH layers (receipt dedupe + claim replay), challenge → revise → revision answers the challenge, budget-denied work enqueued without a reservation + a denial row, post-close results stored as idempotent rejections, ordinary channel events stay receipts; then `bash scripts/demo_two_host.sh` → ALL acceptance checks PASS (real SIGKILL kill points: server restart, node killed after the durable dispatch boundary → `outcome_unknown` bounded + never retried, duplicate delivery re-POSTed verbatim, budget exhaustion refused at both boundaries, closure preserving contribution + unresolved challenge) with the evidence bundle; `make check` → fmt clean + clippy no warnings + all 29 suites green offline; `make gate` → `=== all doctrines green ===` (13/13); `make deny` → advisories/bans/licenses/sources ok; `make secret-scan` → `no leaks found`; `make book` → HTML written; decision record `docs/decisions/2026-09-07_node-channel-wiring.md` + INDEX row; mdBook two-host-demo chapter + SUMMARY entry; `make demo` + CI pg-tests job wired | WP6 node wiring + two-host demo proven: dispatch rides the command transaction, node results fold in claim-first keyed on the inbox command id, no silent retry, and the demo script IS the acceptance test; **WP6 complete** |
| `2026-09-07` | `PHASE-0.7` | `cargo test -p reasonbraid-adapter` → `test result: ok. 7 passed` (grader) + `test result: ok. 5 passed` (`bench_harness`: the scripted agent reproduces the corpus oracle EXACTLY — computed == expected scores over 8 cases × 4 workflows, call accounting 1/2/3/3, structure validity, unresolved-register fidelity, the honesty trap, no-independence-score report hygiene + spread-bearing aggregates + factual-only Brier, and the critique/revision prompt-split guard) + `9 + 12` existing suites; `rb-bench --agent scripted` → full report with corpus/prompt digests; REAL run `RB_LIVE_CODEX=1 … --max-calls 36` → 16/16 rows structure-valid with scores, confidences, citations, and provider-reported tokens (`docs/evidence/2026-09-07_benchmark-codex-run.md`); the FIRST real run caught two harness defects the scripted oracle cannot see (revision leg re-rendering the CRITIQUE template — no confidence lines + one broken answer; the honesty trap flagging the question's own echoed year) — both fixed with regression tests; `make check` → fmt clean + clippy no warnings + all offline suites green; `make gate` → `=== all doctrines green ===` (13/13); `make deny` → advisories/bans/licenses/sources ok (regex, sha2, clap added); `make secret-scan` → `no leaks found`; `make book` → HTML written; decision record `docs/decisions/2026-09-07_deliberation-benchmark.md` + INDEX; evidence report + INDEX; mdBook benchmark chapter + SUMMARY | WP7 benchmark proven: deterministic graders (never an LLM judge), corpus-carried oracle, per-case confidence + spread, no independence score, env-gated call-budgeted real mode; real result on the sample is the accepted NULL (structure did not beat single at 2–4× cost) — the routing claim narrows honestly for WP8 |
| `2026-09-07` | `PHASE-0.8.1` | `test -f` over the four gate documents + INDEX rows; `make check` → all offline suites green (no code changed in this leaf); `make gate` → `=== all doctrines green ===` (13/13); `make deny` → advisories/bans/licenses/sources ok; `make secret-scan` → `no leaks found`; `make book` → HTML written; SubtractionRecord lists verified non-empty (grep) — removed 1 / deferred 9 / narrowed 4 / rejected 4 / avoided 7 / fallbacks 4 / eliminated 5; risk register `wc -l` grew by the two new rows; ADR-002 `proposed` with the signature line | WP8 gate package published: G0 evidence manifest + ADR set + SubtractionRecord + GO recommendation; **the Phase 0 tree is exhausted** — the formal exit awaits the director's signature on ADR-002 |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `PHASE-0.0.1` | `REASONBRAID-PHASE0-0001` | ADR-001 + README landing page |
| `PHASE-0.0.2` | `REASONBRAID-PHASE0-0002` | adr/evidence templates |
| `PHASE-0.0.3` | `REASONBRAID-PHASE0-0003` | parking-lot.md |
| `PHASE-0.0.4` | `REASONBRAID-PHASE0-0004` | docs/risks.md |
| `PHASE-0.0.5` | `REASONBRAID-PHASE0-0005` | external-ledger.yaml |
| `PHASE-0.0.6` | `REASONBRAID-PHASE0-0006` | accountable-owners decision record |
| `PHASE-0.0.7` | `REASONBRAID-PHASE0-0007` | deny.toml + supply-chain workflow + Makefile deny/secret-scan |
| `PHASE-0.0.8` | `REASONBRAID-PHASE0-0008` | G0 contract drafts under `spec/` + ID-scheme decision record |
| `PHASE-0.1.1` | `REASONBRAID-PHASE0-0009` | `crates/reasonbraid-core` strong ID newtypes + id-representation decision record |
| `PHASE-0.1.2` | `REASONBRAID-PHASE0-0010` | `crates/reasonbraid-core` envelopes + fixtures/schemas + envelope-representation decision record |
| `PHASE-0.1.3` | `REASONBRAID-PHASE0-0011` | `crates/reasonbraid-core` state machines + `ProviderAttemptId` + state-transitions decision record |
| `PHASE-0.1.4` | `REASONBRAID-PHASE0-0012` | `crates/reasonbraid-core` reason-code registry + typed errors + reason-codes decision record |
| `PHASE-0.2.1` | `REASONBRAID-PHASE0-0013` | `crates/reasonbraid-server` atomic transaction + migrations + `run_pg_tests.sh` + pg-tests CI + atomic-transaction decision record |
| `PHASE-0.2.2` | `REASONBRAID-PHASE0-0014` | `crates/reasonbraid-server` leased outbox worker (`outbox.rs`) + `migrations/0002_outbox_worker.sql` + kill-point/fencing tests + harness updates + outbox-worker-fencing decision record |
| `PHASE-0.3.1` | `REASONBRAID-PHASE0-0015` | `crates/reasonbraid-node` SQLite journal (WAL + synchronous=FULL, boundary-before-boundary) + read-only `rb-journal` CLI + kill-point tests; core gains `failed_known` + §11.3 provider-lookup edges; node-journal decision record; mdBook chapter |
| `PHASE-0.3.2` | `REASONBRAID-PHASE0-0016` | `crates/reasonbraid-server` node channel (durable inbox, replay, directives, axum routes) + `migrations/0003` + node-side client/facade (Offline/Reconciling/Schedulable) + journal channel state + 13 cross-crate channel tests + node-channel decision record; mdBook chapter |
| `PHASE-0.5.2` | `REASONBRAID-PHASE0-0021` | `crates/reasonbraid-core` budget model (fail-closed dimensions, reservation reference) + `crates/reasonbraid-server` budget engine (reserve/settle/release/deny, `migrations/0005`) + node supervisor reservation gate + `LocalBudget` + budget tests on both boundaries + budget-reservation decision record; mdBook budget chapter |
| `PHASE-0.5.1` | `REASONBRAID-PHASE0-0020` | `crates/reasonbraid-core` authority model (boundary ceiling, grants, subset checker, policy digest, decision record) + `crates/reasonbraid-server` authority engine (create/refuse, authorize + audit row, `apply_authorized_command`) + `migrations/0004` + 9 live-PG tests + authority-boundary decision record; mdBook authority chapter |
| `PHASE-0.4.2` | `REASONBRAID-PHASE0-0019` | `crates/reasonbraid-adapter` Codex CLI adapter (`exec --json` subprocess) + contract `ProviderRequestId` + supervisor/journal handling + offline stub suite + env-gated live test + real-adapter-codex decision record + qualification evidence report + ledger revalidation; mdBook real-adapter section |
| `PHASE-0.4.1` | `REASONBRAID-PHASE0-0018` | `crates/reasonbraid-adapter` (contract + scripted fake + sanitized 10-fixture corpus) + node supervisor (`execute_attempt`) + core proof-gated `(dispatched, fail_before_dispatch)` edge + fake-adapter decision record; mdBook chapter |
| `PHASE-0.6.1` | `REASONBRAID-PHASE0-0022` | `crates/reasonbraid-server` thread domain + control API + `rb-server` binary + `migrations/0006` (enrollments) + `crates/reasonbraid-cli` (the eight verbs; repo-local state dir) + command-API + real-binary e2e suites + control-api-cli decision record; mdBook cli chapter; core gains `thread_close` + the deterministic actor handle |
| `PHASE-0.6.2` | `REASONBRAID-PHASE0-0023` | node wiring: invite/challenge dispatch work items + best-effort reservations in the command transaction; `apply_node_result_in_tx` folds node results in claim-first keyed on the inbox command id; in-tx channel/budget variants; the `rb-node` worker + `journal::work_items`/`emitted_events` + `rb-journal events`; `tests/node_work.rs`; `scripts/demo_two_host.sh` (the acceptance-test demo + evidence bundle) + `make demo` + CI/harness wiring; node-channel-wiring decision record; mdBook two-host-demo chapter |
| `PHASE-0.7` | `REASONBRAID-PHASE0-0024` | the WP7 benchmark: `src/bench/` (corpus/grader/scripted/workflows/report) + the `rb-bench` binary + `bench/v1/` corpus/prompts + `tests/bench_harness.rs` (corpus-carried oracle self-test); two real-run-caught fixes with regression tests (critique/revision prompt split; honesty-trap echoed numbers); deliberation-benchmark decision record; evidence report + INDEX; mdBook benchmark chapter |
| `PHASE-0.8.1` | `REASONBRAID-PHASE0-0025` | the WP8 gate package: evidence manifest + ADR set + SubtractionRecord + ADR-002 (GO, proposed) + risk-register refresh (two new rows) + INDEX rows; no code change |

## Changelog

- `2026-09-05`: Created from `KICKOFF.md` + `ROADMAP.md` §20.2.
- `2026-09-05`: `PHASE-0.0.1` ADR-001. Frontier is `.0.2`.
- `2026-09-05`: `PHASE-0.0.2` templates. Frontier is `.0.3`.
- `2026-09-05`: `PHASE-0.0.3` parking lot. Frontier is `.0.4`.
- `2026-09-05`: `PHASE-0.0.4` risks. Frontier is `.0.5`.
- `2026-09-06`: `PHASE-0.0.5` external dependency ledger skeleton. Frontier is `.0.6`.
- `2026-09-06`: `PHASE-0.0.6` accountable owners named (Richard DJE, both roles). Frontier is `.0.7`.
- `2026-09-06`: `PHASE-0.0.7` supply-chain skeleton (deny.toml, supply-chain CI, `make deny`/`make secret-scan`). Frontier is `.0.8`.
- `2026-09-06`: `PHASE-0.0.8` G0 contract drafts under `spec/` (glossary, requirements, lifecycle, threat-model, governance/charter) + `docs/decisions/2026-09-06_g0-contract-id-scheme.md`. Frontier is `.1.1`.
- `2026-09-06`: `PHASE-0.1.1` strong IDs — `crates/reasonbraid-core` (branded newtypes over UUIDv7, eight families) + `docs/decisions/2026-09-06_id-representation.md`. Placeholder `crates/app` removed. Frontier is `.1.2`.
- `2026-09-06`: `PHASE-0.1.2` command/event envelopes — `CommandEnvelope`/`ClientContext`/`CommittedEvent` with `deny_unknown_fields`, five new ID families, JSON Schema goldens + wire fixtures + `docs/decisions/2026-09-06_envelope-representation.md`. Frontier is `.1.3`.
- `2026-09-06`: `PHASE-0.1.3` minimal state machines — thread/participation/provider-attempt lifecycles with deterministic fallible `apply`, `ProviderAttemptId` (`patt`), `docs/decisions/2026-09-06_state-transitions.md`. Frontier is `.1.4`.
- `2026-09-06`: `PHASE-0.1.4` typed errors + reason-code registry — complete §9.8 registry with unknown-code preservation, `Retryability`, `DomainError`, `docs/decisions/2026-09-06_reason-codes.md`. WP1 complete; frontier is `.2.1`.
- `2026-09-06`: `PHASE-0.2.1` WP2 atomic transaction — `crates/reasonbraid-server` (`apply_command` writes idempotency/event/state/outbox in one transaction), `migrations/0001_atomic_transaction.sql`, `scripts/run_pg_tests.sh` + `pg-tests` CI, `deny.toml` corrected for cargo-deny 0.20, `docs/decisions/2026-09-06_atomic-transaction.md`. Frontier is `.2.2`.
- `2026-09-06`: `PHASE-0.2.2` WP2 leased outbox worker — `outbox.rs` claim/deliver/complete (each phase its own commit, per-claim fencing tokens, caller-supplied clock), `migrations/0002_outbox_worker.sql` (lease+fencing columns, `outbox_delivery` dedupe sink), 7 kill-point/fencing tests, `docs/decisions/2026-09-06_outbox-worker-fencing.md`. **WP2 complete.** Frontier is `.3.1`.
- `2026-09-06`: `PHASE-0.3.1` WP3 node journal — `crates/reasonbraid-node` (WAL + `synchronous=FULL` recorded in `journal_meta`, `record_dispatch` commits before the adapter runs, `recover` → `outcome_unknown`, `prove_result`/`reconcile` exits, command/operation dedupe, boundary ledger, ack cursor), read-only `rb-journal` CLI (inspect/pending/ambiguous), KP-1…KP-9 kill-point sweep, core machine extended (`failed_known` + §11.3 provider-lookup edges), `docs/decisions/2026-09-06_node-journal.md`, mdBook node-journal chapter. Frontier is `.3.2`.
- `2026-09-06`: `PHASE-0.3.2` WP3 outbound node channel — server-side durable inbox (`migrations/0003_node_inbox.sql`, replay from the node's reported cursor, handshake directives, deduplicated event receipts, axum routes) + node-side client and `Offline → Reconciling → Schedulable` facade, journal channel state, 13 cross-crate channel tests over real localhost sockets + live PostgreSQL, `docs/decisions/2026-09-06_node-channel.md`, mdBook node-channel chapter. **WP3 complete.** Frontier is `.4.1`.
- `2026-09-06`: `PHASE-0.5.2` WP5 budget engine — core `BudgetDimensions` (fail-closed coverage, total fallible arithmetic), server reserve/settle/release/deny against the ceiling (`migrations/0005_budget.sql`, denial rows are the audit), and the node supervisor's dispatch gate (reservation + `LocalBudget` headroom before the boundary; refusals audited `failed_before_dispatch`; indeterminate attempts keep their hold), `docs/decisions/2026-09-06_budget-reservation.md`, mdBook budget chapter. **WP5 complete.** Frontier is `.6.1`.
- `2026-09-06`: `PHASE-0.5.1` WP5 authority engine — core boundary/grant/decision model (deterministic subset checker enforced at creation AND evaluation; SHA-256 policy digest) + server `apply_authorized_command` (audit record + `.2.1` writes in one transaction; denials audited and effect-free), `migrations/0004_authority.sql`, 9 live-PG tests, `docs/decisions/2026-09-06_authority-boundary.md`, mdBook authority chapter. Frontier is `.5.2`.
- `2026-09-06`: `PHASE-0.4.2` WP4 first real harness — `CodexCliAdapter` supervising `codex exec --json` (qualified live on codex-cli 0.153.4: one bounded real dispatch through the real supervisor + journal), streamed thread id attached as the provider handle, honest `Unsupported` status lookup, offline stub suite, `docs/decisions/2026-09-06_real-adapter-codex.md`, qualification evidence report (second adapter recommended for Phase 1 — director-owned), Codex ledger row revalidated. **WP4 complete.** Frontier is `.5.1`.
- `2026-09-06`: `PHASE-0.4.1` WP4 fake harness adapter — `crates/reasonbraid-adapter` (capability-declaring contract: ack ≠ completion, no credential field, unsupported lookup is never retry advice; deterministic scripted `FakeAdapter`; ten-fixture sanitized corpus with mechanical credential scan) + node supervisor (`execute_attempt`) + core proof-gated `(dispatched, fail_before_dispatch)` edge, `docs/decisions/2026-09-06_fake-adapter.md`, mdBook adapter-boundary chapter. Frontier is `.4.2`.
- `2026-09-06`: `PHASE-0.6.1` WP6 control API + CLI — `crates/reasonbraid-server` gains the thread domain (`threads.rs`: the six operations, the projection, core-machine validation, dev rules) + the control API (`api.rs`: enroll bootstrap, `/v1/threads` command/query/audit surface, trusted dev principal header + deterministic UUIDv5 actor handle, SHA-256 request hash, ONE transaction per command — claim → authorize → validate against the locked projection → apply (+ ceiling), with idempotent REJECTIONS) + the `rb-server` binary + `migrations/0006_control_api.sql`; the new `crates/reasonbraid-cli` lands the `rb` binary (the eight verbs, repo-local state dir); core gains `thread_close` + `actor_handle_for_subject`; `docs/decisions/2026-09-06_control-api-cli.md`, mdBook cli chapter. Frontier is `.6.2`.
- `2026-09-07`: `PHASE-0.6.2` WP6 node wiring + two-host demo — invite/challenge dispatch work items (with best-effort reservations; denials recorded and enqueued reservation-less) in the command transaction; `apply_node_result_in_tx` folds node `work_result` events into the thread claim-first (idempotency key = inbox command id) with in-tx settlement; in-tx channel/budget variants; `crates/reasonbraid-node` gains the `rb-node` worker (`worker.rs`: poll → journal → execute only absent/`prepared` attempts — never a silent retry) + `journal::work_items`/`emitted_events` + `rb-journal events`; `tests/node_work.rs` (6 live-PG tests); `scripts/demo_two_host.sh` (the acceptance-test demo with real SIGKILL kill points + evidence bundle) wired into `run_pg_tests.sh`/`make demo`/CI; `docs/decisions/2026-09-07_node-channel-wiring.md`, mdBook two-host-demo chapter. **WP6 complete.** Frontier is `.7`.
- `2026-09-07`: `PHASE-0.7` WP7 deliberation/routing benchmark — `crates/reasonbraid-adapter` gains `src/bench/` (versioned corpus + deterministic graders + scripted agent + four-workflow runner + spread-bearing report) + the `rb-bench` binary + `bench/v1/{corpus,prompts}.json` + `tests/bench_harness.rs` (the corpus carries its own scoring oracle; computed == expected proven over 8×4); the REAL Codex run (36 bounded calls, `RB_LIVE_CODEX=1`) caught two harness defects the scripted oracle cannot see (critique/revision template conflation; honesty-trap echoed numbers) — fixed with regression tests; the corrected run's NULL result (structure did not beat single at 2–4× cost) narrows the routing claim for WP8; `docs/decisions/2026-09-07_deliberation-benchmark.md`, `docs/evidence/2026-09-07_benchmark-codex-run.md` + INDEX rows, mdBook benchmark chapter. **WP7 complete.** Frontier is `.8`.
- `2026-09-07`: `PHASE-0.8.1` WP8 Phase 0 decision and subtraction package — evidence manifest (G0 map + fixtures + failures + commands), ADR-set audit map, the §19.8 SubtractionRecord (non-empty throughout), ADR-002 (Phase 1 GO recommendation, `proposed` — signature line pending the director), risk register refreshed (R-AMB mitigated, R-VALUE narrowed, R-VARIANCE + R-OVERHEAD added), INDEX rows; no code change; the 2×-estimate review is not triggered (≈9.5 vs 8–14 engineer-weeks, recorded). **WP8 complete; the PHASE-0 tree is exhausted — the formal exit awaits the director's signature on ADR-002.** Frontier is `PHASE-0-MAINT-1` (director's word).
