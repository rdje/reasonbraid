# ADR-016 — Workflow profiles: versioned configuration over the same aggregates — composition never creates capability

- **Status:** `accepted` (evidence-gated — the §13.1 profiles are the lane's
  contract, and the thread's `workflow_profile` string needs the VALIDATED
  reference shape before the `.1.2` registry lands)
- **Date:** `2026-09-07`
- **Leaf:** `PHASE-5.1.1`
- **Requirements:** `ROADMAP.md` §23 queue item 016 (workflow profile
  DSL/state machines); §13.1–13.2 (the profile table + the reference flow)

## Context

The census found the profile is an UNVALIDATED STRING: the CLI passes
`workflow_profile` through to the thread body, and `threads.rs` stores it
verbatim — no DSL, no state machines, no validation, no step composition.
The §13.1 table's eight profiles must become configuration over the
aggregates the earlier phases shipped: the thread lifecycle's deterministic
state machines (Phase 1), the typed contributions (§8.5), the budget
envelopes (Phase 2), and the evidence pipeline (Phase 4 — the
`evidence_review` profile's claims/assessments substrate).

## Decision

- **A profile is VERSIONED CONFIGURATION, never a capability.** The step
  vocabulary composes the EXISTING verbs — the contribution kinds, the
  terminal outcomes (decided/inconclusive + the unresolved register), the
  budget checks — and sequences them. A profile cannot introduce a new
  operation, a new permission, or a new side effect.
- **The invariants are the composition's boundary.** No profile bypasses
  authorization (every step re-runs the authority evaluation), budget
  (every step draws from the thread's envelope), or the lifecycle
  invariants (the state machine's `apply` remains the only transition
  path). A profile that would violate any of them is INVALID at
  validation time, not at run time.
- **The initial registry is the §13.1 table.** The eight built-ins ship as
  versioned entries: `quick_advice` (one/few responses + optional
  synthesis), `independent_panel` (blind responses then adjudication),
  `critique` (draft → critics → revision → owner decision),
  `evidence_review` (claims → acquisition → verifier assessment),
  `architecture_decision` (options → constraints → trade-offs → adversarial
  review), `incident_review` (timeline → hypotheses → evidence → actions),
  `policy_proposal` (revisions → impact → authority → vote/approval),
  `retrospective` (predicted vs observed → lessons → corrections).
- **Custom profiles are validated against the same invariants** and carry
  a version; the thread's `workflow_profile` becomes a reference to a
  profile VERSION — the unknown profile is the typed refusal, never a
  stored string.
- **The blind-first property rides the profile, not the transport:**
  `independent_panel`'s blind phase is the composition's first step — the
  .1.2 registry records it; the transport-level enforcement is the `.5.2`
  blind-first lane's.

## Consequences

- The `.1.2` registry and the `.1.3` execution implement this contract
  verbatim; a deviation is a contract change.
- The state machine stays the ONLY transition path — the profiles are
  sequences over it, so the Phase-1 lifecycle proofs continue to hold for
  every profile.

answers:

- **Configuration is the safe half of a DSL.** The profiles compose the
  verbs that already exist — the authorization/budget/lifecycle checks ride
  every step because they live in the verbs, not in the profile. A profile
  that needs a new capability is out of scope by construction.
- **The unknown profile is a refusal, not a default.** The current stored
  string means nothing; the validated reference means the refusal is typed
  and the default is explicit (`quick_advice` for a bare thread, named in
  the `.1.2` registry).
