# PHASE-2: delivery, identity, and recovery hardening

## Metadata

- Tree ID: `PHASE-2`
- Status: `active`
- Roadmap lane: Phase 2 (`ROADMAP.md` §20.4); Trust track
- Created: `2026-09-05`
- Estimate: 12–20 engineer-weeks
- Depends on: Phase 1 slice
- Exit: authority non-escalation; full restore and node replacement; no known path that reports an ambiguous provider attempt as safely retryable

## Goal

Harden identity, delivery, recovery, and observability so a later Internet
slice can reuse the same control plane without rewriting it.

## Task Tree

- ID: `PHASE-2.1`
  Status: `proposed`
  Goal: workload certificate lifecycle, scoped grants, delegated authority context, revocation, cached-decision rules
  Backlog: 11
  ADR: 007, 008, 009

- ID: `PHASE-2.2`
  Status: `proposed`
  Goal: production-grade leases/fencing, retry policy, dead-letter/quarantine/replay
  ADR: 005 (transport choice if Phase 0 left it open)

- ID: `PHASE-2.3`
  Status: `proposed`
  Goal: provider-attempt state machine, usage reconciliation, spend circuit breakers, ambiguous-outcome workflows
  Backlog: 23, 25
  ADR: 012, 013

- ID: `PHASE-2.4`
  Status: `proposed`
  Goal: backup, PITR, object/Git inventory groundwork, migrations, upgrade/rollback testing
  Roadmap: §17.5–17.6

- ID: `PHASE-2.5`
  Status: `proposed`
  Goal: OpenTelemetry, operator dashboards, initial SLO baselines, game days
  Backlog: —
  ADR: 023
  Roadmap: §18

- ID: `PHASE-2.6`
  Status: `proposed`
  Goal: adapter conformance kit and permanent failure fixture corpus
  Roadmap: §19.4

- ID: `PHASE-2.7`
  Status: `proposed`
  Goal: exit — non-escalation properties; restore + node replacement; no false safe-retry of unknown attempts
  Gate: feeds G6–G7; subtraction record required
  ADR: 022 (audit hash-chain groundwork)

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `PHASE-2.1` | `proposed` | Phase 1 is CLOSED (G1–G2 Met, Demonstration A 30/30) — the identity/recovery lane executes; the pickup gap census + decomposition come first (the `.1`-pattern) |

## Changelog

- `2026-09-05`: Created from `ROADMAP.md` §20.4.
- `2026-09-07`: Unblocked — the Phase-1 G2 close (`PHASE-1.8.2`, gate record
  **Met**) releases the frontier; `.1` (workload certificate lifecycle,
  scoped grants, delegated authority context, revocation, cached-decision
  rules — backlog 11, ADR 007/008/009) executes after its pickup census.
