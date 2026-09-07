# PHASE-5: deliberation quality and routing

## Metadata

- Tree ID: `PHASE-5`
- Status: `active`
- Roadmap lane: Phase 5 (`ROADMAP.md` §20.7); Quality track
- Created: `2026-09-05`
- Estimate: 15–26 engineer-weeks plus domain-evaluator effort
- Depends on: evidence provenance and stable workflow data
- Exit: G5 on declared domains. If deliberation improves some outcomes but not others, routing and product language reflect that evidence.

## Goal

Treat deliberation quality as an evaluated product hypothesis (H1–H6), not a
consequence of adding agents. Preserve dissent, lineage, evidence, uncertainty,
and honest inconclusive outcomes.

## Non-Goals

- An independence score or rewarding textual disagreement for its own sake.
- Learned routing that can raise authority, spend, data access, or side effects.

## Task Tree

- ID: `PHASE-5.1`
  Status: `done`
  Goal: workflow profile DSL/state machines for consult, parallel review, rigorous deliberation, incident, and policy modes
  Backlog: 36
  ADR: 016
  Roadmap: §13.1–13.2
  Children: `.1.1`–`.1.3` (decomposed `2026-09-07` at the census
    seams): `.1.1` ADR-016 + the census (the workflow-profile
    contract: the profile = versioned configuration over the
    thread aggregates; the composition invariants — no step
    bypasses authorization/budget/lifecycle; the eight §13.1
    built-ins) → `.1.2` the profile registry + the validation
    (the typed shape + the built-ins + the custom-profile
    validation) → `.1.3` the profile-driven execution (the step
    composition over the state machine).
  Done (`2026-09-07`): the census mapped §13.1 against the
    shipped surface: the profile is an UNVALIDATED STRING today
    — the CLI passes `workflow_profile` through to the thread
    body (the Phase-1 wire's opaque field), and `threads.rs`
    stores it verbatim; no DSL, no state machines, no
    validation, no step composition exists
    (`git grep -c "workflow" 01da603 -- crates/reasonbraid-server/src/`
    → the 8 hits are the stored string). ADR-016 is UNOPENED
    (`docs/adr/` has no 016). The pieces the lane reuses exist:
    the thread lifecycle's deterministic state machines (Phase
    1), the typed contributions (§8.5), the budget envelopes
    (Phase 2), the evidence pipeline (Phase 4 — the
    `evidence_review` profile's claims/assessments). Children at
    those seams — frontier → `.1.1`.

  - ID: `PHASE-5.1.1`
    Status: `done`
    Goal: ADR-016 + the census — the workflow-profile contract:
      the profile is VERSIONED CONFIGURATION over the same
      thread aggregates (the §13.1 table's eight built-ins as
      the initial set), the step vocabulary (the composition
      over the existing verbs), and the INVARIANTS (a profile
      cannot bypass authorization, budget, or the lifecycle
      invariants — the composition is the configuration, never a
      new capability). No code.
    Backlog: 36 (the ADR half)
    ADR: 016
    Done (`2026-09-07`): ADR-016 accepted (evidence-gated) —
      `docs/adr/016-workflow-profiles.md` (top-level `answers:`):
      the profile is VERSIONED CONFIGURATION over the same
      thread aggregates (the composition over the existing
      verbs — never a new capability), the invariants are the
      composition's boundary (no profile bypasses
      authorization/budget/lifecycle — an invalid profile is
      INVALID at validation time, not at run time), the initial
      registry is the §13.1 table's eight built-ins, the custom
      profiles validate against the same invariants, and the
      thread's `workflow_profile` becomes a reference to a
      profile VERSION (the unknown profile is the typed refusal,
      never a stored string). No code changed. Frontier →
      `.1.2`.

  - ID: `PHASE-5.1.2`
    Status: `proposed`
    Goal: the profile registry + the validation — the typed
      profile shape (the steps + the version), the eight §13.1
      built-ins shipped as the versioned registry entries, the
      custom-profile validation (the composition rules + the
      invariant checks), and the thread's `workflow_profile`
      becoming a VALIDATED reference (the unknown profile is the
      typed refusal, never a stored string).
    Backlog: 36 (the registry half)

  - ID: `PHASE-5.1.3`
    Status: `proposed`
    Goal: the profile-driven execution — the step composition
      over the state machine: the profile selects the
      contribution/terminal sequence, the invariants enforced at
      the transition level (the authorization, the budget, the
      lifecycle checks ride every step), the explicit per-step
      failures (the profile never fabricates progress).
    Backlog: 36 (the execution half)

- ID: `PHASE-5.2`
  Status: `proposed`
  Goal: blind-first contributions, structured claims/objections/revisions, evidence requests, adjudication, minority reports, unresolved registers
  Roadmap: §13.4, §13.6

- ID: `PHASE-5.3`
  Status: `proposed`
  Goal: moderator and synthesizer constraints with auditable transformations
  Roadmap: §13.5
  Acceptance: moderator cannot vote, suppress dissent silently, fabricate evidence, change electorate, authorize spend, or publish policy

- ID: `PHASE-5.4`
  Status: `proposed`
  Goal: versioned evaluation service, randomized routing experiments, cohort tracking, calibration, regression gates
  Backlog: 37
  ADR: 017
  Roadmap: §13.7, §19.5

- ID: `PHASE-5.5`
  Status: `proposed`
  Goal: deterministic/constrained routing policy; learned routing only after a rule-based baseline and sufficient data, shadow mode first
  Roadmap: §13.8

- ID: `PHASE-5.6`
  Status: `proposed`
  Goal: G5 exit on declared domains; honest inconclusive behavior; subtract unsupported quality claims
  Gate: G5; subtraction record required
  Kill/pivot: `ROADMAP.md` §25.1 after Phase 5

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `PHASE-5.1.2` | `proposed` | `.1.1` done — ADR-016 accepted (`docs/adr/016-workflow-profiles.md`: the versioned configuration + the composition invariants + the eight built-ins); the profile registry + the validation execute now |

## Changelog

- `2026-09-07`: `.1.1` done — ADR-016 accepted (the versioned
  configuration + the composition invariants + the eight
  built-ins — durable in `docs/adr/016-workflow-profiles.md`);
  no code; frontier → `.1.2`.
- `2026-09-07`: `.1` decomposed at the census seams — the profile
  is an unvalidated string today (the CLI passes it through, no
  DSL/validation/execution exists, ADR-016 unopened); children
  `.1.1` (ADR-016 + the census) → `.1.2` (the registry + the
  validation) → `.1.3` (the execution); frontier → `.1.1`.


- `2026-09-05`: Created from `ROADMAP.md` §20.7, §13, backlog 36–37.
