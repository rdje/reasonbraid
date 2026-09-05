# PHASE-1: trustworthy LAN vertical slice

## Metadata

- Tree ID: `PHASE-1`
- Status: `proposed`
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
  Status: `proposed`
  Goal: coordinator modular monolith, PostgreSQL migrations, aggregate/event/outbox patterns
  Backlog: 9, 10, 15
  ADR: 002, 004

- ID: `PHASE-1.2`
  Status: `proposed`
  Goal: Rust node with SQLite journal, enrollment, lease/presence, reconnect, durable inbox
  Backlog: 11–14

- ID: `PHASE-1.3`
  Status: `proposed`
  Goal: thread create/read/list/cancel; explicit participants; invitations accept/decline/timeout; simple subscriptions
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
| — | `PHASE-1.1` | `proposed` | blocked on `PHASE-0.8.1` go decision |

## Changelog

- `2026-09-05`: Created from `ROADMAP.md` §20.3, §26.1, backlog 9–22.
