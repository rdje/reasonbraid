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
    Status: `done`
    Goal: the profile registry + the validation — the typed
      profile shape (the steps + the version), the eight §13.1
      built-ins shipped as the versioned registry entries, the
      custom-profile validation (the composition rules + the
      invariant checks), and the thread's `workflow_profile`
      becoming a VALIDATED reference (the unknown profile is the
      typed refusal, never a stored string).
    Backlog: 36 (the registry half)
    Done (`2026-09-07`): the registry landed — migration 0032
      (`workflow_profiles`: the profile_id + the version + the
      steps + the built_in flag; the eight §13.1 built-ins
      seeded as version 1); `crates/reasonbraid-server/src/
      workflows.rs` (NEW): the step vocabulary (the twelve kinds
      — the composition over the existing verbs, nothing else
      expressible), the `validate_steps` invariants (the known
      kinds only, the terminal last, the adjudicate-after-blind
      rule — the authorization/budget/lifecycle invariants live
      in the VERBS), the `resolve` (the latest version; `None` →
      the `quick_advice` default; the unknown id is the typed
      refusal), the `register` (the custom profiles — validated,
      the version increments), the `list`; the thread's
      `workflow_profile` became a VALIDATED REFERENCE: the
      CreateBody carries the id string (the old Phase-1
      `WorkflowProfile` enum is gone), the create boundary
      resolves it (the unknown = the 400 naming the id; the
      canonical id rides the projection), the bare thread
      defaults to `quick_advice`; the verbs: `POST`/`GET
      /v1/workflow-profiles`. Measured (profiles 25): the
      built-ins list, the custom registration, the three
      invalid-composition refusals (the unknown step / the
      non-terminal last / the adjudicate-without-blind), the
      unknown-id create refusal, the known-id projection
      roundtrip. Frontier → `.1.3`.

  - ID: `PHASE-5.1.3`
    Status: `done`
    Goal: the profile-driven execution — the step composition
      over the state machine: the profile selects the
      contribution/terminal sequence, the invariants enforced at
      the transition level (the authorization, the budget, the
      lifecycle checks ride every step), the explicit per-step
      failures (the profile never fabricates progress).
    Backlog: 36 (the execution half)
    Done (`2026-09-07`): the profile-driven execution landed —
      the projection carries the RESOLVED step sequence + the
      current index (`workflow_steps` + `workflow_step`: the
      create seats step 0 with the registry's steps, the close
      advances to the terminal step — the lifecycle's own
      transitions are the only step transitions, so the
      authorization/budget/lifecycle invariants ride EVERY step
      by construction); the create boundary resolves the FULL
      profile (the steps ride the create event — the event
      replay carries them; the auto-initiation path resolves
      too); the inspection shows the plan (the steps + the
      index). Measured (profiles 25 — the suite grew 24→25): the independent_panel
      create seats `["blind_solicit", "adjudicate", "decide"]`
      at index 0; the close advances to index 2 (`decide`).
      **`.1` COMPLETE** — frontier → `.2`.

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
| 1 | `PHASE-5.2` | `proposed` | `.1.3` done — **the `.1` lane (the workflow profiles) is COMPLETE**: ADR-016 + the registry + the validation + the execution (the steps ride the projection, the close advances to the terminal — profiles 25); the blind-first contributions lane executes next |

## Changelog

- `2026-09-07`: `.1.3` done — the profile-driven execution (the
  projection's steps + the index: the create seats step 0, the
  close advances to the terminal); profiles 25; **`.1`
  COMPLETE** — frontier → `.2`.
- `2026-09-07`: `.1.2` done — the profile registry + the
  validation (migration 0032: the eight built-ins; the twelve-
  kind vocabulary; the three composition rules; the thread's
  profile is a validated reference — the unknown id refuses at
  the create boundary); profiles 25; frontier → `.1.3`.
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


## Acceptance Checklist (PHASE-5.1.2)

The CODE change owned by this leaf:
`migrations/0032_workflow_profiles.sql` (NEW — the registry +
the eight built-ins), `crates/reasonbraid-server/src/workflows.rs`
(NEW — the vocabulary + the validation + the resolve/register/
list), `src/threads.rs` (the CreateBody's string reference, the
projection's id, the default), `src/api.rs` (the create-boundary
resolution + the two verbs), `src/lib.rs` (the module),
`tests/profiles.rs` (profiles 25), `tests/command_api.rs` + the
CLI e2e (the old wire names → the ADR-016 names) — `\.rs$` +
`(^|/)migrations/`.

- [x] **REPRODUCE / ISSUE** — the `.1` census: the profile is an
  unvalidated string (the Phase-1 enum's four variants; no
  registry, no validation).
- [x] **ROOT CAUSE (WHY + WHERE)** — the wire predates ADR-016 —
  `git grep -c "workflow_profiles\|validate_steps" 7dc77a2 --
  crates/ migrations/` → rc=1 (nothing before this leaf). The
  fix point is the ADR-016 registry: the versioned entries, the
  composition validation, the create-boundary resolution.
- [x] **ADDRESSED (verified)** — measured before→after. Before: the
  grep above. After:
  `DATABASE_URL=… cargo test -p reasonbraid-server --test
  profiles the_workflow_profile_registry` → `test result: ok. 1
  passed` — the eight built-ins list, the valid custom
  registration (version 1), the three invalid-composition
  refusals (the unknown step / the non-terminal last / the
  adjudicate-without-blind), the unknown-id create refusal, the
  known-id projection roundtrip. The first live runs caught the
  SEVENTH SQL-continuation doubling (in the resolve query) +
  the two old-wire test suites (the command_api + the CLI e2e
  asserted the Phase-1 enum names) — all fixed.
- [x] **NO REGRESSION** — `cargo test --all` → rc=0, 55 suites;
  `bash scripts/run_pg_tests.sh` → rc=0, 18 live suites + the
  demo `ALL acceptance checks passed` (`target/pg512e_guard.log`);
  `cargo clippy --all --all-targets -- -D warnings` → rc=0;
  `cargo fmt --all -- --check` → rc=0; `make gate` → 13/13 at
  commit.
- [x] **FIX** — `0032_workflow_profiles.sql`, `src/workflows.rs`,
  `src/threads.rs`, `src/api.rs`, `src/lib.rs`,
  `tests/profiles.rs`, `tests/command_api.rs`,
  `crates/reasonbraid-cli/tests/cli_end_to_end.rs`.
- [x] **LOCKSTEP** — CHANGELOG, DEV_NOTES, MEMORY, LIVE_STATUS, this
  tree's logs above, `docs/TASK_TREE.md` frontier, KNOWLEDGE_MAP —
  same commit.


## Acceptance Checklist (PHASE-5.1.3)

The CODE change owned by this leaf:
`crates/reasonbraid-server/src/threads.rs` (the projection's
`workflow_steps` + `workflow_step`, the `default_workflow_steps()`,
the `prepare_create` fifth parameter, the close's terminal-index
advance), `crates/reasonbraid-server/src/api.rs` (the create
boundary resolves the FULL `ResolvedProfile` into the steps — both
create handlers; `CommandTarget::Create` carries them; the
match-by-ref), `crates/reasonbraid-server/tests/profiles.rs` (the
new projection test) — `\.rs$`.

- [x] **REPRODUCE / ISSUE** — the `.1.2`-era projection: the
  profile was a validated REFERENCE (the id + the registry) but
  the thread carried NO step sequence and NO index — the profile
  could not steer anything (the execution half was missing).
- [x] **ROOT CAUSE (WHY + WHERE)** — `git grep -c "workflow_step"
  c1b8cdd -- crates/ migrations/` → rc=1 (the projection had no
  step state before this leaf). The fix point is the projection +
  the create boundary: the steps resolve ONCE, ride the create
  event (replay carries them), and the close seats the terminal
  index.
- [x] **ADDRESSED (verified)** — measured before→after. Before:
  the grep above. After: `DATABASE_URL=… cargo test -p
  reasonbraid-server --test profiles
  the_profile_steps_ride_the_projection_through_the_close` →
  `test result: ok. 1 passed` (also inside the full live suite:
  `running 25 tests … ok`, `target/pg513_guard.log`) — the
  `independent_panel` create seats
  `["blind_solicit", "adjudicate", "decide"]` at index 0, the
  close advances to index 2 (`decide`); the auto-initiation path
  resolves the steps too. The first offline pass caught the
  match-by-ref partial move + the `prepare_create` arity — fixed.
- [x] **NO REGRESSION** — `cargo test --all` → rc=0, 55 suites;
  `bash scripts/run_pg_tests.sh` → rc=0, 18 live suites + the
  demo `ALL acceptance checks passed`
  (`target/pg513_guard.log`; evidence
  `target/demo/20260907-171316/evidence`);
  `cargo clippy --all --all-targets -- -D warnings` → rc=0;
  `cargo fmt --all -- --check` → rc=0; `make gate` → 13/13 at
  commit.
- [x] **FIX** — `src/threads.rs`, `src/api.rs`,
  `tests/profiles.rs`.
- [x] **LOCKSTEP** — CHANGELOG, MEMORY, LIVE_STATUS, this tree's
  logs above, `docs/TASK_TREE.md` frontier — same commit (the
  KNOWLEDGE_MAP regen produced no diff; no new DEV_NOTES heading,
  so no promotion gate).
