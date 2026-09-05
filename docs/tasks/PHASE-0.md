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
  Status: `pending`
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
  Status: `pending`
  Goal: create `docs/parking-lot.md` with revisit-trigger convention
  Acceptance: file exists; v0.4.1 freeze points here for non-blocking ideas
  Verification: pending
  Commit: pending

- ID: `PHASE-0.0.4`
  Status: `pending`
  Goal: create `docs/risks.md` seeded from `ROADMAP.md` §25 (not a copy of the whole table)
  Acceptance: live risk register exists and names owners/triggers
  Verification: pending
  Commit: pending

- ID: `PHASE-0.0.5`
  Status: `pending`
  Goal: `docs/dependencies/external-ledger.yaml` skeleton (`ROADMAP.md` §7.4 / §28)
  Acceptance: schema has name, owner, source_url, checked_at, versions, revalidation_trigger; MCP/A2A/Codex/Claude rows stubbed from the 2026-09-04 baseline
  Verification: pending
  Commit: pending

- ID: `PHASE-0.0.6`
  Status: `pending`
  Goal: record accountable architecture-decision owner and release/security gate owner
  Acceptance: named in a decision record even if one person fills several roles
  Verification: pending
  Commit: pending

- ID: `PHASE-0.0.7`
  Status: `pending`
  Goal: CI/dependency/license/secret-scan skeleton beyond bedrock `make check`/`make gate`
  Acceptance: documented commands; `deny.toml` or equivalent policy; no public-release claim
  Roadmap: backlog 8
  Verification: pending
  Commit: pending

- ID: `PHASE-0.0.8`
  Status: `pending`
  Goal: G0 contract drafts — glossary, requirement IDs, lifecycle tables, threat-model skeleton, governance-charter draft (not yet normative)
  Acceptance: stable IDs exist for identity/authority/thread/delivery/budget; threat-model skeleton lists trust boundaries; charter draft names bootstrap root authority as a human
  Roadmap: §20.2 deliverables; backlog 2, 4, 7
  Verification: pending
  Commit: pending

### WP1 — Minimal contracts (`KICKOFF` issues 2–3; backlog 3, 5, 6)

- ID: `PHASE-0.1`
  Status: `pending`
  Goal: only the types the first slice needs
  Depends on: `PHASE-0.0`
  Children: `PHASE-0.1.1` … `PHASE-0.1.4`

- ID: `PHASE-0.1.1`
  Status: `pending`
  Goal: strong IDs — tenant, human, host, node, agent role, incarnation, run, thread
  Acceptance: newtypes; role/incarnation/run/attempt cannot be confused in types or wire
  Roadmap: §8.1, backlog 3, ADR 010
  Verification: pending
  Commit: pending

- ID: `PHASE-0.1.2`
  Status: `pending`
  Goal: command submission envelope and committed event envelope; JSON Schema/golden fixtures
  Acceptance: fixtures for every wire payload the demo uses; client-supplied actor/tenant/sequence rejected
  Roadmap: §9.1–9.2, backlog 6
  Verification: pending
  Commit: pending

- ID: `PHASE-0.1.3`
  Status: `pending`
  Goal: minimal thread (`open`/`closing`/`closed`/`cancelled`), participation (`invited`/`accepted`/`declined`/`expired`/`left`), provider-attempt (`prepared`/`dispatched`/`completed`/`failed_before_dispatch`/`outcome_unknown`/`reconciled`) transitions
  Acceptance: invalid transitions rejected deterministically; no policy/evidence/directory/federation entities
  Roadmap: §8.4, backlog 5
  Verification: pending
  Commit: pending

- ID: `PHASE-0.1.4`
  Status: `pending`
  Goal: typed errors and stable reason-code registry
  Acceptance: codes from `ROADMAP.md` §9.8 that the demo needs; unknown codes remain preservable
  Verification: pending
  Commit: pending

### WP2 — PostgreSQL transaction and outbox (`KICKOFF` issues 4–5; backlog 9–10)

- ID: `PHASE-0.2`
  Status: `pending`
  Goal: one transaction writes current state, ordered event, idempotency result, and outbox item; leased worker with fencing
  Depends on: `PHASE-0.1`
  Children: `PHASE-0.2.1`, `PHASE-0.2.2`
  Roadmap: ADR 004

- ID: `PHASE-0.2.1`
  Status: `pending`
  Goal: prove PostgreSQL state/event/idempotency/outbox atomic transaction
  Acceptance: successful response ⇔ committed durable state; same key+hash returns original result; different hash is conflict; transport redelivery produces one domain effect
  Kill points: before commit; after commit before response
  Verification: pending
  Commit: pending

- ID: `PHASE-0.2.2`
  Status: `pending`
  Goal: leased outbox worker with fencing and kill-point tests
  Acceptance: stale leased workers cannot commit after a newer fencing value; kill points after claim, after delivery, after ack
  Verification: pending
  Commit: pending

### WP3 — Node SQLite journal and reconnect (`KICKOFF` issues 6–7; backlog 12–13)

- ID: `PHASE-0.3`
  Status: `pending`
  Goal: outbound node, WAL journal, cursor resume, reconciliation
  Depends on: `PHASE-0.1`; may proceed beside `PHASE-0.2`
  Children: `PHASE-0.3.1`, `PHASE-0.3.2`
  Roadmap: ADR 006, ADR 012

- ID: `PHASE-0.3.1`
  Status: `pending`
  Goal: SQLite node journal (WAL, explicit durability) and journal inspection CLI
  Acceptance: operators inspect pending/ambiguous entries without opening SQLite by hand; crash after possible dispatch yields `outcome_unknown` unless the adapter can prove the result
  Verification: pending
  Commit: pending

- ID: `PHASE-0.3.2`
  Status: `pending`
  Goal: outbound node channel with cursor resume and reconciliation handshake
  Acceptance: reconnect exchanges last acknowledged server cursor and pending local operation IDs; duplicate command never creates a second local operation; node not schedulable until reconciliation completes
  Verification: pending
  Commit: pending

### WP4 — Adapter boundary (`KICKOFF` issues 8–9; backlog 19–21)

- ID: `PHASE-0.4`
  Status: `pending`
  Goal: narrow adapter contract, fake adapter, first real harness
  Depends on: `PHASE-0.1` and enough of `PHASE-0.3` to journal attempts
  Children: `PHASE-0.4.1`, `PHASE-0.4.2`

- ID: `PHASE-0.4.1`
  Status: `pending`
  Goal: deterministic fake harness adapter and ambiguity fixtures
  Acceptance: can stream, fail, hang, report usage, ignore cancellation, lose a response after dispatch; vendor DTOs never enter core; credentials never enter events/fixtures
  Verification: pending
  Commit: pending

- ID: `PHASE-0.4.2`
  Status: `pending`
  Goal: qualify the first real harness (Codex-family or Claude-family) behind the same contract
  Acceptance: dispatch ack ≠ completion; unsupported status lookup → `outcome_unknown` not a retry recommendation; evidence report says whether the second real adapter belongs in late Phase 0 or Phase 1
  Verification: pending
  Commit: pending

### WP5 — Identity, authority, budget (`KICKOFF` issues 10–11; backlog 11, 24)

- ID: `PHASE-0.5`
  Status: `pending`
  Goal: development enrollment ceiling, scoped commands, reservation before dispatch
  Depends on: `PHASE-0.1`, `PHASE-0.2`
  Children: `PHASE-0.5.1`, `PHASE-0.5.2`

- ID: `PHASE-0.5.1`
  Status: `pending`
  Goal: development `EnrollmentAuthorityBoundary`, scoped commands, authorization audit record
  Acceptance: tenant membership alone does not grant mandate; every command records actor, subject if delegated, grant/boundary reference, decision, policy digest/version; grant cannot exceed ceiling
  Roadmap: §4.4
  Verification: pending
  Commit: pending

- ID: `PHASE-0.5.2`
  Status: `pending`
  Goal: call/token/time reservation and budget denial path
  Acceptance: no provider dispatch without an applicable reservation; denial tests cross server and node
  Verification: pending
  Commit: pending

### WP6 — Two-host vertical experiment (`KICKOFF` issues 12–13)

- ID: `PHASE-0.6`
  Status: `pending`
  Goal: CLI flow and two-host crash/reconnect demonstration
  Depends on: `PHASE-0.2`–`PHASE-0.5`
  Children: `PHASE-0.6.1`, `PHASE-0.6.2`

- ID: `PHASE-0.6.1`
  Status: `pending`
  Goal: CLI — enroll, create thread, invite, contribute, challenge, revise, close, inspect
  Acceptance: no database surgery required to inspect state
  Verification: pending
  Commit: pending

- ID: `PHASE-0.6.2`
  Status: `pending`
  Goal: script and run the two-host crash/reconnect demonstration
  Acceptance: no human copies messages; accepted commands survive restart; duplicate transport → one domain effect; no silent retry of indeterminate provider calls; closure preserves contributions and unresolved objections; reproducible evidence bundle
  Roadmap: Demonstration A subset
  Verification: pending
  Commit: pending

### WP7 — Small deliberation/routing benchmark (`KICKOFF` issue 14)

- ID: `PHASE-0.7`
  Status: `pending`
  Goal: small versioned corpus vs single-agent, blind independent, critique/revise, moderator/synthesis
  Depends on: fake adapter and at least one real adapter; may use offline harness if vertical slice unfinished
  Acceptance: results include cases and uncertainty, not only an average; no independence score; null/negative result is acceptable and narrows the claim
  Roadmap: §13.7, ADR 017, hypothesis H1/H6
  Verification: pending
  Commit: pending

### WP8 — Phase 0 decision and subtraction package (`KICKOFF` issue 15)

- ID: `PHASE-0.8`
  Status: `pending`
  Goal: evidence manifest, ADR set, subtraction record, Phase 1 go/rework/pivot/stop
  Depends on: completed experiments
  Children: `PHASE-0.8.1`

- ID: `PHASE-0.8.1`
  Status: `pending`
  Goal: publish Phase 0 evidence manifest, ADR set, `SubtractionRecord`, and Phase 1 decision
  Acceptance: G0 evidence for identity/authority/thread/delivery/budget; named owner signs go/rework/pivot/stop; 2×-estimate review if total exceeds 28 engineer-weeks; v0.5.0 still forbidden
  Verification: pending
  Commit: pending

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `PHASE-0.0.3` | `pending` | parking lot before freeze-violating ideas have nowhere to go |
| 2 | `PHASE-0.0.8` | `pending` | G0 drafts ride with WP0, not after experiments |

`RB-SEED` is `done`. This tree is executable.

## Decisions

- `2026-09-05`: KICKOFF work packages WP0–WP8 are this tree's children.
- `2026-09-05`: Phase 0 crate shape follows `KICKOFF.md` §3 (`reasonbraid-core`,
  `-server`, `-node`, `-adapter`, `-cli`), not the full §7.1 workspace.

## Open Questions

- Which real harness is first (Codex-family vs Claude-family) — decided in `PHASE-0.4.2` after the fake adapter.
- Accountable human owners for architecture vs gates — `PHASE-0.0.6`.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-09-05` | (tree created) | mapped from `KICKOFF.md` WP0–WP8 and issues 1–15 | pending execution |
| `2026-09-05` | `PHASE-0.0.1` | ADR-001 written; README landing page; `wc -lc README.md` under README-STABILITY caps | working name internal-only |
| `2026-09-05` | `PHASE-0.0.2` | `test -f docs/adr/TEMPLATE.md docs/evidence/TEMPLATE.md docs/adr/INDEX.md docs/evidence/INDEX.md` | templates present |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `PHASE-0.0.1` | `REASONBRAID-PHASE0-0001` | ADR-001 + README landing page |
| `PHASE-0.0.2` | `REASONBRAID-PHASE0-0002` | adr/evidence templates |

## Changelog

- `2026-09-05`: Created from `KICKOFF.md` + `ROADMAP.md` §20.2.
- `2026-09-05`: `PHASE-0.0.1` ADR-001. Frontier is `.0.2`.
- `2026-09-05`: `PHASE-0.0.2` templates. Frontier is `.0.3`.
