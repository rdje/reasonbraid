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
  Status: `done`
  Goal: blind-first contributions, structured claims/objections/revisions, evidence requests, adjudication, minority reports, unresolved registers
  Roadmap: §13.4, §13.6
  Children: `.2.1`–`.2.4` (decomposed `2026-09-07` at the census
    seams): `.2.1` ADR-029 + the census (the structured-
    deliberation contract: the typed claim/objection/revision
    records, the blind commitment point, the evidence-request
    shape, the adjudication record, the minority report, the
    §13.4 terminal vocabulary) → `.2.2` the structured records
    (the typed claims/objections/revisions over the
    contribute/challenge/revise wire) → `.2.3` the blind-first
    lane (the deferred visibility + the commitment point — the
    `blind_solicit` step executes) → `.2.4` the evidence requests
    + the adjudication + the minority reports (the
    `evidence_request` verb riding the Phase-4 pipeline, the
    `adjudicate` execution, the minority report on the close, the
    §13.4 terminals).
  Done (`2026-09-07`): the census mapped §13.4/§13.6 against the
    shipped surface. The lane is a greenfield at four seams: (1)
    BLIND-FIRST — nothing defers visibility; a contribution is
    visible at post time, no commitment point exists
    (`git grep -c "blind" HEAD -- crates/reasonbraid-server/src/`
    → the 4 hits are the `blind_solicit` step NAME in
    `workflows.rs`); (2) STRUCTURED CLAIMS/OBJECTIONS/REVISIONS —
    the challenge/revise pair is free text against a target
    event id (no typed claim, objection, or revision records);
    (3) EVIDENCE REQUESTS — `evidence_request` is a step name
    only (1 hit), no verb; (4) ADJUDICATION + MINORITY REPORTS +
    TERMINALS — `adjudicate` is a step name only (the
    node_channel hits are the delivery-fate adjudication, not
    deliberation); no minority report exists; the close carries
    two outcomes (`decided`/`inconclusive`) vs §13.4's twelve.
    What the lane REUSES: the contribute kind vocabulary (six
    kinds incl. `claim`/`assumption`) + the `evidence_refs`
    (Phase 1/4), the open_challenges register, the unresolved
    register riding the close, and the Phase-4 evidence pipeline
    (the claim assessments + the citation validation). The §23
    queue's 017 (the evaluator hierarchy) is the `.4` leaf's and
    019 (the canonical policy schema) is the policy lane's, so
    `.2.1` opens ADR-029 (the first free number past the queue).
    Frontier → `.2.1`.

  - ID: `PHASE-5.2.1`
    Status: `done`
    Goal: ADR-029 + the census — the structured-deliberation
      contract: the typed claim/objection/revision records (the
      digest-targeted structure over the existing
      contribute/challenge/revise verbs), the blind commitment
      point (the visibility deferral + the §13.6 no-totals rule),
      the evidence-request shape (riding the Phase-4 pipeline),
      the adjudication record (the verdict is attributable, never
      a silent rewrite), the minority report (the synthesis's
      coverage report), and the §13.4 terminal vocabulary. No
      code.
    ADR: 029
    Roadmap: §13.4, §13.6
    Done (`2026-09-07`): ADR-029 accepted (evidence-gated) —
      `docs/adr/029-structured-deliberation.md` (top-level
      `answers:`): the records are SHAPES over the existing verbs
      (the claim rides `contribute`, the objection rides
      `challenge` and targets the claim digest, the revision
      rides `revise` and answers the objection — the
      authority/budget/lifecycle checks keep living in the
      verbs); the blind commitment point is a READ-SURFACE rule
      (the blind content is in the ledger from post time; the
      pre-commitment readers see the digest + the marker; the
      commitment is the round advance — no new verb); the
      evidence request is a contribution, NOT an acquisition (no
      budget/resolver authority); the adjudication is an
      attributable verdict event (never a silent rewrite); the
      minority report is derived content with the coverage
      report; the close speaks §13.4's twelve terminals (the
      legacy words stay aliases). No code changed. Frontier →
      `.2.2`.

  - ID: `PHASE-5.2.2`
    Status: `done`
    Goal: the structured records — the typed claim/objection/
      revision over the contribute/challenge/revise wire: the
      contribution carries structured claims (the claim digest +
      the kind), the objection targets a claim (not free text),
      the revision answers the objection; the projection carries
      the structure; the registers stay honest.
    Roadmap: §13.4
    Done (`2026-09-07`): the structured records landed per
      ADR-029 — the contribute body gains `claims`
      (`ClaimInput` content-only; the server computes the
      ADR-011 `sha256:<hex>` digest into `ClaimRecord` — the
      wire never supplies one, so an objection can only name a
      server-derived digest); the claims ride a `claim`-kind
      contribution only (the typed refusal otherwise); the
      challenge body gains `claim_digest` (the structured
      objection names ONE claim of the target contribution —
      the digest is checked against the target event's
      server-computed records via the new `event_body_in_thread`
      helper; a foreign/absent digest is the typed refusal); the
      revision answers the objection (the register decrements —
      unchanged); the projection carries the structure
      (`structured_claims` counter, `#[serde(default)]`-ed);
      the free-text wire stays valid (the fields default).
      Measured (profiles 26): the structured contribute seats
      two claims with the re-derived digests; the objection
      rides its claim digest; the foreign digest + the claimless
      target + the position-with-claims refusals; the revision
      closes the register. Frontier → `.2.3`.

  - ID: `PHASE-5.2.3`
    Status: `done`
    Goal: the blind-first lane — the `blind_solicit` step
      executes: the deferred visibility (a blind contribution is
      NOT readable until the commitment point), the commitment
      point (the round advance), the §13.6 no-totals-before-
      commitment rule.
    Roadmap: §13.6
    Done (`2026-09-07`): the blind-first lane landed per
      ADR-029 — a contribution posted while the CURRENT step is
      `blind_solicit` carries `blind: true` (the marker rides
      the event; the ledger keeps the full body — a read rule,
      never a store rewrite); the commitment point is the
      EXISTING round advance: advancing during the blind phase
      moves the step past `blind_solicit` and the event records
      `blind_committed` (no new verb); the read surface
      (`get_events`) serves a blind, uncommitted contribution as
      `blind_until: round_advance` + the server-computed content
      digest to every reader who is NOT the author (the author
      and the post-commitment readers see the full body); a
      non-author challenge of a still-blind contribution is the
      typed refusal (the challenger cannot name content they
      cannot read); the §13.6 no-totals rule holds at the
      surfaces that serve blind content (the events surface
      aggregates nothing). Measured (profiles 27): the blind
      marker rides the contribute; the author sees the full body
      while the non-author role sees the digest + the marker;
      the blind-target challenge refuses; the round advance
      commits (the step → 1, `blind_committed: true`); the
      non-author then reads the full body and challenges.
      Frontier → `.2.4`.

  - ID: `PHASE-5.2.4`
    Status: `done`
    Goal: the evidence requests + the adjudication + the
      minority reports — the `evidence_request` verb (riding the
      Phase-4 claim-evidence pipeline), the `adjudicate`
      execution (the attributable verdict record), the minority
      report on the close, the §13.4 terminal vocabulary.
    Roadmap: §13.4, §13.6
    Children: `.2.4.1`–`.2.4.2` (decomposed `2026-09-07` at the
      close/contribute seam): `.2.4.1` the close vocabulary —
      the §13.4 twelve terminals (the legacy `decided`/
      `inconclusive` words stay accepted aliases but never
      persist; the decision-family/failure-family rule — a
      decision terminal with a non-empty unresolved register is
      the typed refusal) + the minority report riding the close
      (the coverage report) → `.2.4.2` the contribution-side
      execution — the `evidence_request` kind (targets ONE claim
      digest of this thread; the step gate) and the `verdict`
      kind (the attributable record: the judged digest + the
      rule + the §13.4 outcome; the `adjudicate` step gate; the
      `evidence_reference` kind's non-empty refs).
    Done (`2026-09-07`): the census found the two halves of the
      leaf sit on different surfaces: the CLOSE side (the
      outcome is a two-variant enum today — `decided`/
      `inconclusive` — vs §13.4's twelve; no minority report
      exists; the `.1.5.3` decided-with-unresolved refusal is
      the two-valued ancestor of the family rule) and the
      CONTRIBUTION side (the kind vocabulary has no
      `evidence_request` or `verdict`; the `evidence_reference`
      kind accepts empty refs). Children at that seam — frontier
      → `.2.4.1`.

  - ID: `PHASE-5.2.4.1`
    Status: `done`
    Goal: the close vocabulary — the §13.4 twelve terminals on
      the close outcome (the legacy words stay accepted aliases
      mapping to `accepted_by_rule`/`deadlocked`; the canonical
      name persists on the event + the projection; the
      decision-family/failure-family rule replaces the
      `.1.5.3` two-valued check — a decision terminal with a
      non-empty unresolved register is the typed refusal) + the
      minority report riding the close (the synthesizer
      identity, the input event range, the source links, the
      coverage report of included/excluded items).
    Roadmap: §13.4, §13.5
    Done (`2026-09-07`): the close vocabulary landed per
      ADR-029 — the `CloseOutcome` gains the §13.4 twelve; the
      legacy `decided`/`inconclusive` wire words stay accepted
      aliases (→ `accepted_by_rule`/`deadlocked`) and NEVER
      persist (the event body + the projection's new
      `close_outcome` carry the canonical name); the
      `.1.5.3` refusal generalized to the FAMILY rule
      (`is_decision_family`: the four decision terminals +
      `decided` refuse a non-empty unresolved register; the
      eight failure terminals accept and carry it); the
      terminal STATE follows the family (decision → closed,
      failure → the honest `Inconclusive` state); the close
      body gains the `minority_report` (the synthesizer, the
      input event range, the sources, the coverage items with
      the included/`reason` shape) riding the close event; the
      CLI's close help documents the twelve. Measured (profiles
      28): all twelve terminals persist canonically with the
      family-correct thread state; the aliases accept but never
      persist; the family refusal + the failure-family
      acceptance with the register; the minority report rides
      the close verbatim. Frontier → `.2.4.2`.

  - ID: `PHASE-5.2.4.2`
    Status: `done`
    Goal: the contribution-side execution — the
      `evidence_request` kind (targets ONE claim digest of this
      thread; the step gate) and the `verdict` kind (the
      attributable record: the judged digest + the rule + the
      §13.4 outcome; the `adjudicate` step gate); the
      `evidence_reference` kind requires non-empty refs (the
      response carries evidence, never an empty claim of it).
    Roadmap: §13.4
    Done (`2026-09-07`): the contribution-side execution landed
      per ADR-029 — the kind vocabulary gains
      `evidence_request` + `verdict`; the contribute body gains
      `target_claim_digest` (legal only on the request kind,
      REQUIRED there, and it must be a claim of THIS thread —
      the JSONB-containment scan over the server-computed
      `claims[].digest` records; the request is NOT an
      acquisition) and `verdict` (legal only on the verdict
      kind; the judged digest + the rule + the §13.4 outcome —
      the canonical outcome persists, the aliases never do);
      the STEP gates execute the ADR-016 composition (the
      request requires the current step `evidence_request`, the
      verdict requires `adjudicate`); the round advance
      GENERALIZES to the step advance (one step per round,
      clamped at the terminal — the blind commitment flag is
      its special case); the `evidence_reference` kind refuses
      empty refs. Measured (profiles 29): the claim→advance→
      request roundtrip with the target riding the event, the
      foreign-digest + the early-step + the empty-refs + the
      misplaced-field refusals, the panel's blind→advance→
      verdict roundtrip with the canonical outcome. **`.2`
      COMPLETE** — frontier → `.3`.

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
| 1 | `PHASE-5.3` | `proposed` | `.2.4.2` done — the contribution-side execution (the request + verdict kinds with their step gates, the generalized step advance; profiles 29) — **the `.2` lane (the blind-first deliberation) is COMPLETE**; the moderator/synthesizer lane executes next |

## Changelog

- `2026-09-07`: `.2.4.2` done — the contribution-side execution
  (the `evidence_request` + `verdict` kinds with the step
  gates, the generalized step advance, the non-empty
  evidence_reference rule); profiles 29; **`.2` COMPLETE** —
  frontier → `.3`.
- `2026-09-07`: `.2.4.1` done — the close vocabulary (the §13.4
  twelve terminals, the legacy aliases never persisting, the
  family rule, the minority report riding the close); profiles
  28; frontier → `.2.4.2`.
- `2026-09-07`: `.2.4` decomposed at the close/contribute seam
  — `.2.4.1` the close vocabulary (the twelve terminals + the
  minority report) → `.2.4.2` the contribution-side execution
  (the `evidence_request` + `verdict` kinds); frontier →
  `.2.4.1`.
- `2026-09-07`: `.2.3` done — the blind-first lane (the
  `blind_solicit` marker on the contribute, the round advance as
  the commitment point with `blind_committed`, the read-surface
  digest+marker for non-authors, the blind-target challenge
  refusal); profiles 27; frontier → `.2.4`.
- `2026-09-07`: `.2.2` done — the structured records
  (the server-computed claim digests riding the contribute
  event, the digest-targeted objection, the claim-kind rule,
  the honest registers); profiles 26; frontier → `.2.3`.
- `2026-09-07`: `.2.1` done — ADR-029 accepted (the
  structured-deliberation contract: the typed records, the
  read-surface blind commitment, the request-is-not-acquisition
  rule, the attributable verdict, the coverage report, the
  twelve terminals); no code; frontier → `.2.2`.
- `2026-09-07`: `.2` decomposed at the census seams — the four
  greenfields (the blind-first visibility, the structured
  records, the evidence requests, the adjudication + the
  minority reports + the terminals) against the reusable pieces
  (the kind vocabulary, the registers, the Phase-4 pipeline);
  children `.2.1` (ADR-029 + the census) → `.2.2` (the
  structured records) → `.2.3` (the blind-first lane) → `.2.4`
  (the requests + the adjudication + the reports); frontier →
  `.2.1`.
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


## Acceptance Checklist (PHASE-5.2.2)

The CODE change owned by this leaf:
`crates/reasonbraid-server/src/threads.rs` (the `ClaimInput`/
`ClaimRecord` shapes, the contribute body's `claims`, the
challenge body's `claim_digest`, the claim-kind rule, the
server-computed digests, the `event_body_in_thread` helper, the
projection's `structured_claims` counter),
`crates/reasonbraid-server/tests/profiles.rs` (the new test) —
`\.rs$`.

- [x] **REPRODUCE / ISSUE** — the pre-leaf wire: the contribute
  carried free content only; the challenge was free text against
  an event; no claim record, no digest, no targeted objection
  existed (the `.2` census).
- [x] **ROOT CAUSE (WHY + WHERE)** — `git grep -c
  "ClaimRecord\|claim_digest\|structured_claims" 7bbdb4a --
  crates/` → rc=1 (nothing before this leaf). The fix point is
  the ADR-029 wire shapes: the claim rides the contribute body
  with a SERVER-computed digest (the wire never supplies one —
  so an objection can only name a digest the server derived),
  and the objection names one claim of the target.
- [x] **ADDRESSED (verified)** — measured before→after. Before:
  the grep above. After: `DATABASE_URL=… cargo test -p
  reasonbraid-server --test profiles
  the_structured_claims_and_objections_ride_the_wire` →
  `test result: ok. 1 passed` (also inside the full live suite:
  `running 26 tests … ok`) — the two server-computed digests
  re-derived from the claim content, the objection riding its
  claim digest, the foreign-digest refusal, the claimless-target
  refusal, the position-with-claims refusal, the revision
  closing the register. The first live attempts caught the
  relative-`-k` socket-dir trap (postgres resolves a relative
  `-k` against the DATA dir — the ephemeral cluster needs
  ABSOLUTE paths; `run_pg_tests.sh` already does).
- [x] **NO REGRESSION** — `cargo test --all` → rc=0, 55 suites;
  `bash scripts/run_pg_tests.sh` → rc=0, 18 live suites + the
  demo `ALL acceptance checks passed`
  (`target/pg514_guard.log`);
  `cargo clippy --all --all-targets -- -D warnings` → rc=0;
  `cargo fmt --all -- --check` → rc=0; `make gate` → 13/13 at
  commit.
- [x] **FIX** — `src/threads.rs`, `tests/profiles.rs`.
- [x] **LOCKSTEP** — CHANGELOG, MEMORY, LIVE_STATUS, this tree's
  logs above, `docs/TASK_TREE.md` frontier — same commit (the
  KNOWLEDGE_MAP regen produced no diff; no new DEV_NOTES
  heading).


## Acceptance Checklist (PHASE-5.2.3)

The CODE change owned by this leaf:
`crates/reasonbraid-server/src/threads.rs` (the blind marker on
the contribute event, the round advance as the commitment point
with `blind_committed` + the step advance, the blind-target
challenge refusal via `event_body_and_version_in_thread` +
`blind_phase_committed`), `crates/reasonbraid-server/src/api.rs`
(the `get_events` read-surface redaction — digest + marker for
non-authors), `crates/reasonbraid-server/tests/profiles.rs` (the
new test) — `\.rs$`.

- [x] **REPRODUCE / ISSUE** — the pre-leaf surface: every
  contribution was readable at post time; nothing deferred
  visibility, no commitment point existed (the `.2` census: the
  `blind` hits were the step NAME only).
- [x] **ROOT CAUSE (WHY + WHERE)** — `git grep -c
  "blind_committed\|blind_until\|content_digest" be2fd35 --
  crates/` → rc=1 (no blind machinery before this leaf). The
  fix point is the ADR-029 read-surface rule: the marker rides
  the event (the store stays complete), the round advance is
  the commitment (marked on the event), and the read surface
  defers only for non-authors.
- [x] **ADDRESSED (verified)** — measured before→after. Before:
  the grep above. After: `DATABASE_URL=… cargo test -p
  reasonbraid-server --test profiles
  the_blind_contributions_commit_at_the_round_advance` →
  `test result: ok. 1 passed` (also inside the full live suite:
  `running 27 tests … ok`) — the blind marker, the author's
  full read vs the non-author role's digest+marker read (the
  content and the claims absent, the digest re-derived), the
  blind-target challenge refusal, the round-advance commitment
  (`blind_committed: true`, the step → 1), the post-commitment
  reveal + the accepted challenge. The first live pass caught
  the participant gate (a challenge needs PARTICIPANT status —
  the non-author reader is an INVITED role, not a bystander) —
  fixed.
- [x] **NO REGRESSION** — `cargo test --all` → rc=0, 55 suites;
  `bash scripts/run_pg_tests.sh` → rc=0, 18 live suites + the
  demo `ALL acceptance checks passed`
  (`target/pg515_guard.log`);
  `cargo clippy --all --all-targets -- -D warnings` → rc=0;
  `cargo fmt --all -- --check` → rc=0; `make gate` → 13/13 at
  commit.
- [x] **FIX** — `src/threads.rs`, `src/api.rs`,
  `tests/profiles.rs`.
- [x] **LOCKSTEP** — CHANGELOG, MEMORY, LIVE_STATUS, this tree's
  logs above, `docs/TASK_TREE.md` frontier — same commit (the
  KNOWLEDGE_MAP regen produced no diff; no new DEV_NOTES
  heading).


## Acceptance Checklist (PHASE-5.2.4.1)

The CODE change owned by this leaf:
`crates/reasonbraid-server/src/threads.rs` (the twelve-terminal
`CloseOutcome` + `is_decision_family` + `canonical`, the
`MinorityReportInput`/`CoverageItem` shapes, the close arm's
family rule + the canonical event outcome + the projection's
`close_outcome`), `crates/reasonbraid-server/tests/profiles.rs`
(the new test), `crates/reasonbraid-server/tests/command_api.rs`
(the legacy-event assertion → the canonical name),
`crates/reasonbraid-cli/src/main.rs` (the close help documents
the twelve) — `\.rs$`.

- [x] **REPRODUCE / ISSUE** — the pre-leaf close: a two-valued
  outcome (`decided`/`inconclusive` — §13.4 lists twelve); no
  minority report; the `.1.5.3` refusal was the two-valued
  ancestor of the family rule.
- [x] **ROOT CAUSE (WHY + WHERE)** — `git grep -c
  "close_outcome\|MinorityReportInput\|is_decision_family"
  d6fc732 -- crates/` → rc=1 (nothing before this leaf). The
  fix point is the ADR-029 close contract: the twelve terminals
  on the outcome, the aliases accepted-but-never-persisted, the
  family rule, the report riding the event.
- [x] **ADDRESSED (verified)** — measured before→after. Before:
  the grep above. After: `DATABASE_URL=… cargo test -p
  reasonbraid-server --test profiles
  the_twelve_terminals_and_the_minority_report_ride_the_close`
  → `test result: ok. 1 passed` (also inside the full live
  suite: `running 28 tests … ok`) — the twelve terminals
  persist canonically with the family-correct state, the
  aliases accept but persist the canonical name, the family
  refusal + the failure-family acceptance with the register,
  the minority report rides the close verbatim (the
  synthesizer/range/sources/coverage with the included +
  reason shape).
- [x] **NO REGRESSION** — `cargo test --all` → rc=0, 55 suites;
  `bash scripts/run_pg_tests.sh` → rc=0, 18 live suites + the
  demo `ALL acceptance checks passed`
  (`target/pg516_guard.log`);
  `cargo clippy --all --all-targets -- -D warnings` → rc=0;
  `cargo fmt --all -- --check` → rc=0; `make gate` → 13/13 at
  commit.
- [x] **FIX** — `src/threads.rs`, `tests/profiles.rs`,
  `tests/command_api.rs`, `crates/reasonbraid-cli/src/main.rs`.
- [x] **LOCKSTEP** — CHANGELOG, MEMORY, LIVE_STATUS, this tree's
  logs above, `docs/TASK_TREE.md` frontier — same commit (the
  KNOWLEDGE_MAP regen produced no diff; no new DEV_NOTES
  heading).


## Acceptance Checklist (PHASE-5.2.4.2)

The CODE change owned by this leaf:
`crates/reasonbraid-server/src/threads.rs` (the
`EvidenceRequest`/`Verdict` kinds, the `VerdictInput` shape, the
contribute body's `target_claim_digest`/`verdict`, the
kind-specific-field + empty-refs + step-gate validations, the
`claim_exists_in_thread` scan, the generalized step advance, the
event's new fields), `crates/reasonbraid-server/tests/profiles.rs`
(the new test) — `\.rs$`.

- [x] **REPRODUCE / ISSUE** — the pre-leaf wire: the kind
  vocabulary had no `evidence_request`/`verdict`; the
  `evidence_reference` kind accepted empty refs; the steps
  advanced only past `blind_solicit` (the other profiles'
  steps were unreachable).
- [x] **ROOT CAUSE (WHY + WHERE)** — `git grep -c
  "EvidenceRequest\|VerdictInput\|claim_exists_in_thread"
  d6fc732 -- crates/` → rc=1 (nothing before this leaf). The
  fix point is the ADR-029 contribution contract: the kinds
  ride the contribute verb with the step gates + the
  thread-scoped claim-target membership.
- [x] **ADDRESSED (verified)** — measured before→after. Before:
  the grep above. After: `DATABASE_URL=… cargo test -p
  reasonbraid-server --test profiles
  the_evidence_requests_and_verdicts_execute_on_their_steps` →
  `test result: ok. 1 passed` (also inside the full live
  suite: `running 29 tests … ok`) — the claim→advance→request
  roundtrip (the target rides the event), the foreign-digest
  + the early-step + the empty-refs + the misplaced-field
  refusals, the panel's blind→advance→verdict roundtrip with
  the canonical outcome (the alias never persists).
- [x] **NO REGRESSION** — `cargo test --all` → rc=0, 55 suites;
  `bash scripts/run_pg_tests.sh` → rc=0, 18 live suites + the
  demo `ALL acceptance checks passed`
  (`target/pg517_guard.log`);
  `cargo clippy --all --all-targets -- -D warnings` → rc=0;
  `cargo fmt --all -- --check` → rc=0; `make gate` → 13/13 at
  commit.
- [x] **FIX** — `src/threads.rs`, `tests/profiles.rs`.
- [x] **LOCKSTEP** — CHANGELOG, MEMORY, LIVE_STATUS, this tree's
  logs above, `docs/TASK_TREE.md` frontier — same commit (the
  KNOWLEDGE_MAP regen produced no diff; no new DEV_NOTES
  heading).
