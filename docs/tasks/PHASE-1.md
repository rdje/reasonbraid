# PHASE-1: trustworthy LAN vertical slice

## Metadata

- Tree ID: `PHASE-1`
- Status: `active`
- Roadmap lane: Phase 1 (`ROADMAP.md` §20.3)
- Created: `2026-09-05`
- Estimate: 14–22 engineer-weeks
- Depends on: Phase 0 contracts
- Exit: G1–G2; Demonstration A (`ROADMAP.md` §26.1)

## Goal

A killed node resumes without duplicated ReasonBraid effects; a provider
ambiguity is visible; all accepted messages appear once in domain state despite
transport redelivery. Invited users/agents on a trusted LAN can hold a durable
conversation without binding-governance claims.

## Non-Goals

- Automatic semantic discovery, arbitrary Web fetching, binding policy
  publication, public Internet exposure.

## Task Tree

- ID: `PHASE-1.1`
  Status: `in_progress`
  Goal: coordinator modular monolith, PostgreSQL migrations, aggregate/event/outbox patterns
  Backlog: 9, 10, 15
  ADR: 002, 004
  Children: `.1.1.1`–`.1.1.3` (decomposed `2026-09-06` so each child is one signoff-sized slice)

  - ID: `PHASE-1.1.1`
    Status: `pending`
    Goal: the aggregate/event/outbox library — extract the WP2 claim → authorize →
      validate → apply machinery (`reasonbraid-server/src/tx.rs`) into a typed,
      reusable aggregate module: revision-checked state transitions (the locked
      head), ordered event append, idempotency claim/replay/conflict, outbox
      enqueue in ONE transaction, plus in-tx test helpers. Every Phase 0 caller
      switches to it with zero behavior change.
    Backlog: 9
    ADR: 004
    Acceptance: all existing offline suites + the live-PG suites stay green; the
      library owns the claim-first and revision semantics (the transaction body is
      the single write path); a new helper proves fresh-apply vs replay against a
      test aggregate.

  - ID: `PHASE-1.1.2`
    Status: `pending`
    Goal: migration 0007 — first-class identity store: `tenants`, `hosts`, `nodes`,
      `agent_roles`, `incarnations`, `runs`, `human_principals` (the `.6.1`
      `enrollments` table is the dev stand-in). Enroll writes the enrollment row
      AND the identity row in one transaction; the existing surfaces keep working
      unchanged.
    Backlog: 10
    Acceptance: the new tables exist with UUIDv7 ids and the §17.2 conventions
      (tenant on every material record); enroll/re-enroll tests green; no existing
      suite regresses.

  - ID: `PHASE-1.1.3`
    Status: `pending`
    Goal: thread command API completion — `thread.cancel` (the `open → cancelled`
      edge), typed classification + workflow profile + participant rules on
      `thread.create` (default: single-agent routing, per ADR-002), and the
      existing create/read/list/idempotency re-verified against the `.1.1.1`
      library. Backlog 15's API-shape portion; the invitation accept/decline/
      timeout semantics stay with `.1.3`.
    Backlog: 15
    Acceptance: `thread.cancel` lands on the core machine and is inspected through
      the API only; create carries the three new fields with deny-unknown typing;
      the single-agent default is stated, not an empty profile.

- ID: `PHASE-1.2`
  Status: `proposed`
  Goal: Rust node with SQLite journal, enrollment, lease/presence, reconnect, durable inbox
  Backlog: 11–14

- ID: `PHASE-1.3`
  Status: `proposed`
  Goal: invitation/subscription semantics — explicit participants, invitations
    accept/decline/timeout, simple subscriptions (the create/read/list/cancel API
    shapes are owned by `.1.1.3`)
  Backlog: 15, 16

- ID: `PHASE-1.4`
  Status: `proposed`
  Goal: two genuinely distinct harness adapters where access permits, plus deterministic fakes for CI
  Backlog: 19–22
  Note: second real adapter may land here if Phase 0 deferred it

- ID: `PHASE-1.5`
  Status: `proposed`
  Goal: structured contributions, phases/rounds, evidence attachments, manual close, honest inconclusive outcome
  Backlog: 17

- ID: `PHASE-1.6`
  Status: `proposed`
  Goal: basic Web UI/CLI for threads, nodes, inbox, budgets, audit timeline
  Backlog: 18

- ID: `PHASE-1.7`
  Status: `proposed`
  Goal: local/LAN deployment packaging and one-command development environment

- ID: `PHASE-1.8`
  Status: `proposed`
  Goal: G1–G2 exit + Demonstration A (two hosts, blind contributions, kill-after-dispatch → ambiguous, duplicate delivery → one effect, inconclusive allowed)
  Acceptance: no manual relaying; restart/reconnect loses no accepted command; spend/uncertainty visible; inspectable via CLI/UI not database surgery
  Gate: G1, G2; subtraction record required

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `PHASE-1.1.1` | `pending` | the aggregate/event/outbox library is the substrate `.1.1.2` (identity store) and `.1.1.3` (thread API) build on; `.1` decomposed `2026-09-06` into three signoff-sized children |

## Changelog

- `2026-09-05`: Created from `ROADMAP.md` §20.3, §26.1, backlog 9–22.
- `2026-09-06`: Opened by the Phase 0 go — ADR-002 `accepted` (signed by the accountable owner, `PHASE-0.8.2`); `.1` unblocked.
- `2026-09-06`: `.1` decomposed into `.1.1.1` (aggregate/event/outbox library — backlog 9, ADR-004), `.1.1.2` (migration 0007 identity store — backlog 10), `.1.1.3` (thread command API completion — backlog 15's API-shape portion; the invitation semantics stay with `.1.3`); `.1.3`'s goal reworded to remove the double-claim of backlog 15; frontier → `.1.1.1`.
