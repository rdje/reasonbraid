# PHASE-5: deliberation quality and routing

## Metadata

- Tree ID: `PHASE-5`
- Status: `done`
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
  Status: `done`
  Goal: moderator and synthesizer constraints with auditable transformations
  Roadmap: §13.5
  Acceptance: moderator cannot vote, suppress dissent silently, fabricate evidence, change electorate, authorize spend, or publish policy
  Children: `.3.1`–`.3.3` (decomposed `2026-09-07` at the census
    seams): `.3.1` ADR-030 + the census (the moderator/synthesizer
    contract: the moderation actions are CONTRIBUTIONS — the
    closed vocabulary IS the structural prohibition; the
    appealable action = the existing challenge; the synthesis
    record = the derived-content shape) → `.3.2` the moderation
    kinds (the typed moderation contributions + the closed-
    vocabulary refusals + the `moderate` step) → `.3.3` the
    synthesis record (the `synthesize` step executes as the
    derived-content contribution with the coverage report).
  Done (`2026-09-07`): the census mapped §13.5 against the
    shipped surface. The moderator is ZERO machinery: no role,
    no action, no kind, no step (`git grep -n "moderator" HEAD
    -- crates/reasonbraid-server/` → the hits are comments; the
    STEP vocabulary has no `moderate`). The synthesizer is
    ONE-half shipped: the `synthesize` STEP name + the `Summary`
    contribution kind exist, and the derived-content shape (the
    identity/input-range/sources/coverage) exists ONLY as the
    `.2.4.1` minority report. The prohibitions have no
    enforcement surface because nothing to enforce them on
    exists. The reusable pieces: the contribute verb's
    authority/budget checks (a moderation action riding it is
    bounded by construction), the challenge verb (the appeal —
    a moderation contribution is challengeable like any other),
    the `.2.4.1` coverage shapes. The §23 queue has no entry
    for this contract — the lane opens ADR-030. Frontier →
    `.3.1`.

  - ID: `PHASE-5.3.1`
    Status: `done`
    Goal: ADR-030 + the census — the moderator/synthesizer
      contract: the moderation actions are CONTRIBUTIONS (the
      closed kind vocabulary IS the structural prohibition — a
      moderation kind carries no vote/verdict/evidence/
      electorate/spend/policy by construction); the appealable
      moderation action is the existing challenge over the
      moderation contribution; the `moderate` step enters the
      step vocabulary; the synthesis record generalizes the
      `.2.4.1` derived-content shape. No code.
    ADR: 030
    Roadmap: §13.5
    Done (`2026-09-07`): ADR-030 accepted (evidence-gated) —
      `docs/adr/030-moderation-and-synthesis.md` (top-level
      `answers:`): the moderation action is a CONTRIBUTION
      (never a new role/authority — the verbs' checks bound
      it); the closed kind set (`classify`,
      `request_clarification`, `propose_close`, `draft_summary`,
      `identify_unanswered`) REFUSES the capability-shaped
      fields (the verdict/claims/evidence_refs/target) — the
      §13.5 prohibitions hold by construction; the action may
      reference its target via `ref_event_id` and can never
      erase (the moderated content stays in the ledger); the
      appealable action IS the existing challenge; the
      `moderate` step joins the vocabulary (the built-ins
      unchanged); the synthesis record is re-derivable derived
      content (the identity + the event-log input range + the
      sources + the coverage). No code changed. Frontier →
      `.3.2`.

  - ID: `PHASE-5.3.2`
    Status: `done`
    Goal: the moderation kinds — the typed moderation
      contributions (the classify/request-clarification/
      propose-close/draft-summary vocabulary), the closed-
      vocabulary refusals (a moderation kind refuses the
      verdict/claims/evidence_refs/target fields), the
      `moderate` step gate.
    Roadmap: §13.5
    Done (`2026-09-07`): the moderation kinds landed per
      ADR-030 — the kind vocabulary gains the closed set
      (`classify`, `request_clarification`, `propose_close`,
      `draft_summary`, `identify_unanswered`;
      `is_moderation_kind`); a moderation-kind contribution
      REFUSES the capability-shaped fields (the verdict, the
      claims, the evidence_refs, the target — the §13.5
      prohibitions hold by construction); the action may
      reference its target via `ref_event_id` (the ref must
      exist in the thread; it rides the moderation kinds
      only); the `moderate` step gate (the moderation kinds
      execute on it); the `moderate` step joined the STEP
      vocabulary (twelve → thirteen; the built-ins unchanged);
      the action can never erase (it is a contribution — an
      event). Measured (profiles 30): the custom
      `moderated_panel` profile composes the step; the
      solicit-step refusal; the classify + the clarification
      with the reference riding the event; the forged-ref +
      the capability-field + the misplaced-ref refusals; the
      challenge over the moderation action (the appeal IS the
      challenge). Frontier → `.3.3`.

  - ID: `PHASE-5.3.3`
    Status: `done`
    Goal: the synthesis record — the `synthesize` step executes
      as the derived-content contribution (the synthesizer
      identity, the input event range, the source links, the
      coverage report — reusing the `.2.4.1` shapes), the step
      gate, the auditable transformation (the input range is
      the event log's, never a rewrite).
    Roadmap: §13.5
    Done (`2026-09-07`): the synthesis record landed per
      ADR-030 — the contribute body gains `synthesis`
      (`SynthesisInput`: the synthesizer, the `input_from`/
      `input_to` event-log version range, the sources, the
      coverage — the `.2.4.1` shapes generalized); it rides a
      `summary`-kind contribution on the `synthesize` step
      only; the input range is VALIDATED against the event log
      (`1 <= from <= to` AND `to <=` the thread's max version
      — the transformation names events that exist, never a
      claim over nothing); the record rides the event (the
      synthesis is itself an event — it never mutates a prior
      one). The first live pass caught the `.1.3`-lane gap the
      bare-thread default carried: the create handlers resolved
      the profile ONLY when one was named, so a bare thread's
      steps were EMPTY (the step gates read `none`) — both
      handlers now resolve ALWAYS (`None` → `quick_advice`).
      Measured (profiles 31): the solicit-step + the
      wrong-kind refusals, the advance → the synthesize-step
      record riding the event, the overreaching + the inverted
      range refusals. **`.3` COMPLETE** — frontier → `.4`.

- ID: `PHASE-5.4`
  Status: `done`
  Goal: versioned evaluation service, randomized routing experiments, cohort tracking, calibration, regression gates
  Backlog: 37
  ADR: 017
  Roadmap: §13.7, §19.5
  Children: `.4.1`–`.4.4` (decomposed `2026-09-07` at the census
    seams): `.4.1` ADR-017 + the census (the evaluation-service
    contract: the versioned case registry, the experiment
    records with the declared seeds, the randomized routing
    trials, the cohort tracking, the calibration record, the
    regression gates — the G5 thresholds) → `.4.2` the service
    core (the case registry + the experiment records + the
    results persistence) → `.4.3` the randomized routing
    experiments + the cohort tracking → `.4.4` the calibration +
    the regression gates.
  Done (`2026-09-07`): the census mapped §13.7/§19.5 against the
    shipped surface. The WP7 bench harness (Phase 0,
    `crates/reasonbraid-adapter/bench/`) IS a substantial
    substrate: the versioned corpus (the 9 cases + the ground
    truths + the rubrics + the expected scores, digest-carrying
    `corpus.json`/`prompts.json`), the four deliberation
    workflows over the real Adapter contract (single / blind+
    adjudication / critique-revise / moderator-synthesis), the
    deterministic grading (`grade`/`brier`/`normalize`/`Trap`),
    the per-case confidence + the cost accounting, the
    spread-bearing report (`build`/`write`), the ScriptedAgent
    self-test asserting the expected scores, the `rb-bench`
    binary. The GREENFIELD (backlog 37): no evaluation SERVICE
    (the harness is static files + a binary — no case registry,
    no run records, no persistence); no randomized routing
    experiments; no cohort tracking; no calibration RECORD
    (the Brier computes per-run, nothing accumulates); no
    regression gates (nothing fails when a score drops below a
    baseline); the §19.5 axes the harness misses (the citation
    entailment, the unresolved visibility, the robustness
    under correlated agents) stay out of scope for the `.4`
    service (the harness's axes grow later). The `.1`–`.3`
    lanes just shipped the deliberation machinery the
    evaluation will measure. The queue's 017 (the evaluator
    hierarchy + the release thresholds) is THIS lane's ADR.
    Frontier → `.4.1`.

  - ID: `PHASE-5.4.1`
    Status: `done`
    Goal: ADR-017 + the census — the evaluation-service
      contract: the versioned case registry (the corpus
      versions ride the service, the digests pin them), the
      experiment records (the workflow + the corpus version +
      the DECLARED SEED + the repeated trials — §19.7), the
      randomized routing trials (the seeded assignment, the
      shadow mode first — §13.8), the cohort tracking (the
      subject/case cohorts), the calibration record (the
      Brier + the confidence accumulated across runs, the
      model graders never the sole authority — §19.5), the
      regression gates (the G5 thresholds — a drop below the
      baseline blocks the claim). No code.
    ADR: 017
    Roadmap: §13.7, §19.5, §19.7
    Done (`2026-09-07`): ADR-017 accepted (evidence-gated) —
      `docs/adr/017-evaluation-service.md` (top-level
      `answers:`): the service RECORDS, the harness MEASURES
      (one grading implementation — a second judge would
      invite drift); the registry is versioned +
      content-addressed (the ADR-011 digests pin the corpus);
      the experiment record DECLARES its seed (an undeclared
      randomness is the typed refusal); the randomized routing
      trial is SHADOW-ONLY (the evidence the `.5` lane's
      decision consumes — it never changes production
      routing); the cohort is a recorded label (the reports
      aggregate only the recorded assignments); the
      calibration record accumulates (the model grader never
      judges alone); the regression gate is the G5 threshold
      and only BLOCKS. No code changed. Frontier → `.4.2`.

  - ID: `PHASE-5.4.2`
    Status: `done`
    Goal: the evaluation-service core — the case registry (the
      versioned cases + the digests) + the experiment records
      (the run record: the workflow, the corpus version, the
      seed, the trial count, the result rows) + the results
      persistence (the per-case grades + the confidence + the
      cost).
    Roadmap: §13.7, §19.7
    Done (`2026-09-07`): the service core landed per ADR-017 —
      migration 0033 (`evaluation_corpora`: the corpus id +
      the version + the declared 64-hex digests + the cases
      JSONB; `evaluation_runs`: the workflow arm + the corpus
      reference + the seed + the determinism flag + the trial
      count + the harness's results JSONB);
      `crates/reasonbraid-server/src/evaluation.rs` (NEW):
      the `register_corpus` (the digest shape check, the
      duplicate = the typed refusal — never an overwrite),
      the `record_run` (the positive trial count; the
      UNDECLARED-seed refusal — a non-deterministic run must
      declare its seed; the registered-corpus check — never a
      phantom; the duplicate run id refuses), the list verbs;
      the api: `POST`/`GET /v1/evaluations/corpora` +
      `POST`/`GET /v1/evaluations/runs` (the enrolled gate —
      the dev-trusted operator surface, like the profile
      register); the pg script gained the `evaluation` suite.
      Measured (evaluation 1): the corpus registers with the
      digests; the duplicate + the malformed-digest + the
      seedless-run + the phantom-corpus + the duplicate-run
      refusals; the seeded + the deterministic runs record;
      the lists (the runs newest first); the unenrolled read
      refuses. Frontier → `.4.3`.

  - ID: `PHASE-5.4.3`
    Status: `done`
    Goal: the randomized routing experiments + the cohort
      tracking — the trial assignment (the seeded
      randomization over the workflow arms), the cohort
      records (the case cohorts + the subject cohorts), the
      per-arm results.
    Roadmap: §13.7, §13.8
    Done (`2026-09-07`): the shadow trials landed per ADR-017 —
      migration 0034 (`evaluation_trials`: the declared seed +
      the arms + the cohorts + the case ids + the SERVER-
      computed assignment; `evaluation_trial_results`:
      APPEND-ONLY per-arm rows — never an overwrite);
      `evaluation.rs` gains the `create_trial` (the seeded
      assignment via a dependency-free splitmix64 — the std
      hasher is NOT stable across releases, the draw must be;
      the empty-arms/cases + the unknown-cohort-kind + the
      phantom-corpus + the duplicate refusals), the
      `record_trial_results` (the append to a REGISTERED trial
      only), the list verbs; the api: `POST`/`GET
      /v1/evaluations/trials` + `POST`/`GET
      /v1/evaluations/trials/{id}/results`; the trial never
      changes production routing (the shadow rule — the `.5`
      lane's decision consumes the records). Measured
      (evaluation 2): the assignment covers every case with a
      declared arm, the SAME seed + cases re-draw the SAME
      assignment (the reproducibility proof), the different
      seed differs, the five refusals, the append-only results
      (both rows survive), the newest-first list. Frontier →
      `.4.4`.

  - ID: `PHASE-5.4.4`
    Status: `done`
    Goal: the calibration + the regression gates — the
      calibration record (the Brier + the confidence
      accumulated across the runs), the regression gate (the
      per-case threshold — a score below the baseline is the
      typed failure, the CI manifest's G5 row).
    Roadmap: §13.7, §19.5, §19.6 (G5)
    Done (`2026-09-07`): the calibration + the gates landed per
      ADR-017 — migration 0035 (`evaluation_calibrations`: the
      NAMED runs + the brier + the confidence;
      `evaluation_gates`: the baseline + the threshold;
      `evaluation_gate_results`: APPEND-ONLY evaluations);
      `evaluation.rs` gains the `record_calibration` (the run
      ids must be REGISTERED runs — the accumulation is over
      real measurements; the brier in [0, 1]), the
      `record_gate` (the threshold + the baseline scores in
      [0, 1], the non-empty case→score object), the
      `evaluate_gate` (each measured case against the baseline
      − the threshold; a drop below it is the typed FAILURE
      with the delta; the result APPENDS — the gate never
      rewrites a result); the api: `POST`/`GET
      /v1/evaluations/calibrations`, `POST`/`GET
      /v1/evaluations/gates`, `POST`/`GET
      /v1/evaluations/gates/{id}/evaluations`. Measured
      (evaluation 3): the calibration over the two runs, the
      ghost-run + the out-of-range refusals, the gate's
      failing evaluation (the failing case + the baseline +
      the measured + the delta), the second all-passing
      evaluation appending (both rows survive), the threshold/
      baseline/ghost refusals, the lists. **`.4` COMPLETE** —
      frontier → `.5`.

- ID: `PHASE-5.5`
  Status: `done`
  Goal: deterministic/constrained routing policy; learned routing only after a rule-based baseline and sufficient data, shadow mode first
  Roadmap: §13.8
  Children: `.5.1`–`.5.3` (decomposed `2026-09-07` at the census
    seams): `.5.1` ADR-031 + the census (the routing-policy
    contract: the case-class vocabulary, the rule-based policy
    as the deterministic class→arm resolution, the learned
    routing as the SHADOW RECOMMENDATION constrained to the
    existing arms) → `.5.2` the rule-based policy (the §13.8
    rows as the built-in rules + the deterministic resolution)
    → `.5.3` the shadow recommendation (the recorded,
    never-applied learned routing over the `.4` evidence).
  Done (`2026-09-07`): the census mapped §13.8 against the
    shipped surface. The routing decision today is the
    CLIENT's choice: the create carries the explicit
    `workflow_profile`, the bare thread defaults to the
    hardcoded `quick_advice` — no rule, no case-class
    vocabulary, no policy (`git grep -n "routing" HEAD --
    crates/reasonbraid-server/src/` → the hits are the
    HTTP router + the node channel's route comments; the
    `Classification` enum is the SECURITY classification,
    not a routing class). The reusable pieces: the §13.1
    built-in profiles map §13.8's left side (quick_advice =
    low-risk/simple, evidence_review = factual/current,
    independent_panel = uncertain/high-value, critique +
    architecture_decision = design/policy, policy_proposal =
    binding/high-impact), the `.4` evaluation service's
    trials/gates (the evidence the recommendation consumes),
    and the create boundary's validated-profile resolution.
    The §23 queue has no routing entry — the lane opens
    ADR-031. Frontier → `.5.1`.

  - ID: `PHASE-5.5.1`
    Status: `done`
    Goal: ADR-031 + the census — the routing-policy contract:
      the case-class vocabulary (the §13.8 rows), the
      rule-based policy (the deterministic class→arm
      resolution — the client's EXPLICIT profile stays
      authoritative, the human authority outranks the rule),
      the learned routing as the SHADOW RECOMMENDATION only
      (it names an EXISTING arm — it cannot raise authority,
      spend, data access, or side-effect scope), the policy
      selection recorded + auditable. No code.
    ADR: 031
    Roadmap: §13.8
    Done (`2026-09-07`): ADR-031 accepted (evidence-gated) —
      `docs/adr/031-routing-policy.md` (top-level `answers:`):
      the case class is a SUBMITTED input (the §13.8 rows —
      never a derived judgment); the rule-based policy is a
      deterministic table (the built-in class→arm rules); the
      human authority outranks the rule (the policy applies
      at the create boundary ONLY when no explicit profile is
      named — the explicit choice always wins); the learned
      routing is the SHADOW RECOMMENDATION (an arm from the
      EXISTING registered set — never a raise of
      authority/spend/access/side-effects — recorded with its
      `.4` evidence reference, never applied); every selection
      is recorded. No code changed. Frontier → `.5.2`.

  - ID: `PHASE-5.5.2`
    Status: `done`
    Goal: the rule-based policy — the §13.8 rows as the
      built-in rules (the class → the profile arm), the
      deterministic resolution (the class → the arm + the
      rule id + the audit record), the create-boundary
      application (the class submitted + no explicit profile
      → the policy's arm; the explicit profile outranks it).
    Roadmap: §13.8
    Done (`2026-09-07`): the rule-based policy landed per
      ADR-031 — migration 0036 (`routing_rules`: the seven
      §13.8 rows as the built-in rules — one arm per class;
      `routing_resolutions`: the append-only audit);
      `crates/reasonbraid-server/src/routing.rs` (NEW): the
      `CASE_CLASSES` vocabulary, the `resolve` (the
      deterministic lookup; the arm must be a REGISTERED
      profile — a phantom arm fails closed), the
      `record_resolution` (the class + the arm + the rule id
      + the caller + the surface), the list verbs; the
      create body gains `routing_class` and BOTH create
      handlers apply the policy ONLY when no explicit profile
      is named (the explicit profile always wins; the bare
      thread stays the `quick_advice` default); the api:
      `GET /v1/routing/rules`, `POST /v1/routing/resolve`,
      `GET /v1/routing/resolutions`; the pg script gained the
      `routing` suite. Measured (routing 1): the seven rules,
      the deterministic repeat, the unknown-class refusal,
      the routed create (the class → the independent_panel
      projection with its steps), the explicit profile
      outranking the rule, the bare default unchanged, the
      create-boundary unknown-class refusal, the three audit
      rows with both surfaces. Frontier → `.5.3`.

  - ID: `PHASE-5.5.3`
    Status: `done`
    Goal: the shadow recommendation — the learned-routing
      surface: the recommendation records (the class → the
      arm, the evidence reference — the `.4` trial/gate
      results), the constraint (the arm must be one of the
      EXISTING registered profiles — never a raise), the
      recommendation is recorded, never applied.
    Roadmap: §13.8
    Done (`2026-09-07`): the shadow recommendation landed per
      ADR-031 — migration 0037
      (`routing_recommendations`: the class + the arm + the
      evidence reference); `routing.rs` gains the
      `record_recommendation` (the class in the vocabulary;
      the arm must be a REGISTERED profile — the
      never-a-raise constraint; the evidence ref must be a
      `.4` trial or gate — the recommendation names what it
      rests on; the duplicate id refuses) + the list; the api:
      `POST`/`GET /v1/routing/recommendations` — the record
      carries `applied: false` ON ITS FACE; the create
      boundary keeps resolving the RULE table (the shadow
      proof: the class `uncertain` still routes to the rule's
      `independent_panel`, not the recommended `critique`).
      Measured (routing 2): the recorded recommendation with
      the stated non-application, the phantom-arm + the
      ghost-evidence + the unknown-class refusals, the
      shadow-create still on the rule's arm, the list.
      **`.5` COMPLETE** — frontier → `.6`.

- ID: `PHASE-5.6`
  Status: `done`
  Goal: G5 exit on declared domains; honest inconclusive behavior; subtract unsupported quality claims
  Gate: G5; subtraction record required
  Kill/pivot: `ROADMAP.md` §25.1 after Phase 5
  Children: `.6.1`–`.6.2` (decomposed `2026-09-07` at the census
    seams): `.6.1` the G5 evidence census (the claim census —
    what the product claims vs what the evidence supports; the
    honest-inconclusive machinery shipped across `.1`–`.3`;
    the benchmark evidence the WP7 harness + the `.4` service
    hold) → `.6.2` the G5 gate package (the gate record, the
    §19.8 subtraction record, the evidence manifest, the
    narrowed product claims — the README/book edits, the
    Phase-5 close).
  Done (`2026-09-07`): the census at the seams. The G5 gate
    (`.4.4`'s machinery) exists as a SERVICE: the calibration
    records + the baseline/threshold gates + the append-only
    evaluations — the gate MEASURES; what the exit needs is
    the PACKAGE: the claim census (the docs' quality claims vs
    the shipped evidence), the gate record (the G5 outcome +
    the evidence), and the §19.8 subtraction record (the
    claims that must be narrowed). The precedents exist: the
    Phase-1 and Phase-4 gate records + the subtraction records
    + the evidence manifests
    (`docs/decisions/2026-09-07_phase{1,4}-gate-record.md`,
    `phase{1,2}-subtraction-record.md`, `docs/evidence/`).
    The honest-inconclusive machinery is SHIPPED and tested
    (the twelve terminals, the unresolved register, the
    budget-exhausted denial, the blind commitment — profiles
    31). Frontier → `.6.1`.

  - ID: `PHASE-5.6.1`
    Status: `done`
    Goal: the G5 evidence census — the claim census (the
      quality claims across the README/book/ROADMAP vs the
      evidence: the WP7 harness's expected scores + the `.4`
      service's registry/runs/gates, the honest-inconclusive
      behavior shipped across `.1`–`.3`), the subtraction
      candidates (the claims the evidence does NOT support),
      the declared domains (the case classes the exit names).
      No code.
    Roadmap: §19.5, §19.6 (G5), §19.8
    Done (`2026-09-07`): the G5 evidence census landed. THE
      CLAIMS: the README carries NO quality claim to retract —
      its claim is the MECHANISM one ("Rust — not an LLM —
      enforces identity, authorization, ordering, budgets,
      and publication"; the model output stays untrusted
      until the deterministic rules accept it — shipped +
      tested across Phases 1–5). THE EVIDENCE: (a) the WP7
      harness's scripted self-test proves the measurement
      pipeline (the corpus's expected scores, the four
      workflows, the deterministic grading — the
      `tests/bench_harness.rs` proof); (b) the FIRST live run
      (`docs/evidence/2026-09-07_benchmark-codex-run.md`): on
      the 4-case differential feasibility sample NO structured
      workflow beat `single` (H1 read: negative), the burden
      measured (H6: 2–3× calls, 1.5–4× tokens — structure
      costs more with no measured gain HERE), the calibration
      too small to claim (Brier 0 on the one factual case),
      the run-to-run variance itself a finding (code-002
      single: 1.000 → 0.667 across runs); (c) the `.4`
      service's registry/runs/trials/gates are the instrument
      for the ENDURING accumulation. THE HONEST-INCONCLUSIVE
      MACHINERY ships + tests (the twelve terminals, the
      unresolved register, the budget-exhausted denial, the
      blind commitment, the honesty trap — profiles 31,
      evaluation 3, routing 2). THE SUBTRACTION CANDIDATES
      (for `.6.2`'s record): S-1 the "structured deliberation
      improves answers" claim (never shipped; the bench reads
      null on the sample — the narrowed claim: the machinery
      is BUILT and MEASURED, the default stays the cheapest
      baseline per §13.8/§25.1); S-2 the calibration claim
      (deferred until the `.4` accumulation); S-3 the
      learned-routing claim (already shadow-only — `.5.3`);
      S-4 the cost claim is SUPPORTED (structure costs 2–4×
      on the sample); S-5 the README's STALE status line
      ("Phase 0 — contracts and kill-risk experiments" —
      the tree is at Phase 5; the `.6.2` edits fix it). THE
      DECLARED DOMAINS: the seven §13.8 case classes + the
      honest-inconclusive terminal vocabulary. No code.
      Frontier → `.6.2`.

  - ID: `PHASE-5.6.2`
    Status: `done`
    Goal: the G5 gate package — the gate record (the G5
      outcome on the declared domains + the evidence
      manifest), the §19.8 subtraction record (the narrowed
      product claims), the doc edits (the README/book claim
      alignment), the Phase-5 tree close.
    Gate: G5; subtraction record required
    Roadmap: §19.6 (G5), §19.8, §25.1
    Done (`2026-09-07`): the G5 gate package landed — the
      gate record (`docs/decisions/2026-09-07_phase5-gate-
      record.md`): **G5 Met as a subtraction gate** (the
      blocking function discharged — the product makes no
      "deliberation improves answers" claim; the `.6.1`
      census + the first controlled evaluation's H1-null read
      per §25.1 narrow the claims; the honest-inconclusive
      half ships + tests; the benchmark-threshold half ships
      as the instrument; four named deferrals); the §19.8
      subtraction record (`2026-09-07_phase5-subtraction-
      record.md`: the features_removed, the deferred with the
      triggers, the narrowed claims, the rejected
      abstractions, the avoided dependencies, the manual
      fallbacks, the eliminated entities, the effort removed,
      the parking lot); the evidence manifest
      (`docs/evidence/2026-09-07_phase5-evidence-manifest.md`:
      the G5 clause map); the README status line fixed (the
      S-5 candidate — the landing page reads the current
      phase). **PHASE 5 IS CLOSED** — frontier → `PHASE-6.1`.

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `PHASE-6.1` | `proposed` | `.6.2` done — the G5 gate package (**G5 Met as a subtraction gate**: the claim withdrawn per §25.1, the subtraction record, the evidence manifest, the README fixed) — **PHASE 5 IS CLOSED**; the semantic-policy lane (`PHASE-6`) executes next |

## Changelog

- `2026-09-07`: `.6.2` done — the G5 gate package (the gate
  record: **G5 Met as a subtraction gate** — the claim
  withdrawn per §25.1; the §19.8 subtraction record; the
  evidence manifest; the README status fix). **PHASE 5 IS
  CLOSED** — frontier → `PHASE-6.1`.
- `2026-09-07`: `.6.1` done — the G5 evidence census (the
  claim census: the README's mechanism claim stands, no
  quality claim exists to retract; the bench's live run reads
  H1 null on the 4-case sample; five subtraction candidates;
  the declared domains = the seven case classes); no code;
  frontier → `.6.2`.
- `2026-09-07`: `.6` decomposed at the census seams — the G5
  gate ships as the `.4.4` service; the exit needs the
  package (the claim census → the gate record + the §19.8
  subtraction record + the narrowed claims); children `.6.1`
  (the evidence census) → `.6.2` (the gate package);
  frontier → `.6.1`.
- `2026-09-07`: `.5.3` done — the shadow recommendation
  (migration 0037: the registered-arm constraint, the named
  evidence, the stated `applied: false`; the shadow proof —
  the create keeps the rule's arm); routing 2; **`.5`
  COMPLETE** — frontier → `.6`.
- `2026-09-07`: `.5.2` done — the rule-based policy (migration
  0036: the seven §13.8 rules, the deterministic resolution,
  the append-only audit; the create-boundary application with
  the explicit profile outranking the rule); routing 1;
  frontier → `.5.3`.
- `2026-09-07`: `.5.1` done — ADR-031 accepted (the
  routing-policy contract: the submitted class, the
  deterministic table, the explicit profile outranks the
  rule, the shadow-only recommendation); no code; frontier →
  `.5.2`.
- `2026-09-07`: `.5` decomposed at the census seams — the
  routing decision is the client's choice today (no rule, no
  class vocabulary, no policy; the §13.1 built-ins map §13.8's
  rows); children `.5.1` (ADR-031 + the census) → `.5.2` (the
  rule-based policy) → `.5.3` (the shadow recommendation);
  frontier → `.5.1`.
- `2026-09-07`: `.4.4` done — the calibration + the regression
  gates (migration 0035: the named-run accumulation, the
  baseline/threshold gate, the append-only evaluations);
  evaluation 3; **`.4` COMPLETE** — frontier → `.5`.
- `2026-09-07`: `.4.3` done — the shadow trials (migration
  0034: the server-computed seeded assignment via the
  dependency-free splitmix64, the recorded cohorts, the
  append-only per-arm results); evaluation 2; frontier →
  `.4.4`.
- `2026-09-07`: `.4.2` done — the evaluation-service core
  (migration 0033: the registry + the run records; the
  declared-seed rule, the phantom-corpus + duplicate
  refusals, the four verbs; the pg script gained the
  evaluation suite); evaluation 1; frontier → `.4.3`.
- `2026-09-07`: `.4.1` done — ADR-017 accepted (the
  evaluation-service contract: the records-and-gates service,
  the digest-pinned registry, the declared seeds, the shadow
  trials, the recorded cohorts, the G5 threshold); no code;
  frontier → `.4.2`.
- `2026-09-07`: `.4` decomposed at the census seams — the WP7
  bench harness is the substrate (the versioned corpus, the
  four workflows, the deterministic grading, the reports); the
  service/experiments/cohorts/calibration/gates are the
  greenfield; children `.4.1` (ADR-017 + the census) → `.4.2`
  (the service core) → `.4.3` (the routing experiments + the
  cohorts) → `.4.4` (the calibration + the regression gates);
  frontier → `.4.1`.
- `2026-09-07`: `.3.3` done — the synthesis record (the
  validated event-log input range, the coverage report, the
  step gate; the bare-thread steps gap fixed); profiles 31;
  **`.3` COMPLETE** — frontier → `.4`.
- `2026-09-07`: `.3.2` done — the moderation kinds (the closed
  vocabulary's negative space: the capability-shaped fields
  refuse, the `ref_event_id` check, the `moderate` step gate,
  the appealable action = the challenge); profiles 30;
  frontier → `.3.3`.
- `2026-09-07`: `.3.1` done — ADR-030 accepted (the
  moderator/synthesizer contract: the prohibition is the
  vocabulary's negative space, the appeal is the challenge, the
  synthesis is re-derivable); no code; frontier → `.3.2`.
- `2026-09-07`: `.3` decomposed at the census seams — the
  moderator is zero machinery (no role/kind/step), the
  synthesizer half-shipped (the `synthesize` step + the
  `.2.4.1` coverage shapes); children `.3.1` (ADR-030 + the
  census) → `.3.2` (the moderation kinds) → `.3.3` (the
  synthesis record); frontier → `.3.1`.
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


## Acceptance Checklist (PHASE-5.3.2)

The CODE change owned by this leaf:
`crates/reasonbraid-server/src/workflows.rs` (the `moderate`
step joining the vocabulary), `crates/reasonbraid-server/src/
threads.rs` (the five moderation kinds + `is_moderation_kind`,
the `ref_event_id` field, the capability-field + step-gate +
ref-exists validations, the event's `ref_event_id`),
`crates/reasonbraid-server/tests/profiles.rs` (the new test) —
`\.rs$`.

- [x] **REPRODUCE / ISSUE** — the pre-leaf surface: no
  moderation kind/step/ref existed (the `.3` census: the
  moderator was zero machinery); nothing enforced the §13.5
  prohibitions.
- [x] **ROOT CAUSE (WHY + WHERE)** — `git grep -c
  "is_moderation_kind\|ref_event_id\|RequestClarification"
  297e2e6 -- crates/` → rc=1 (nothing before this leaf). The
  fix point is the ADR-030 contract: the closed kind set rides
  the contribute verb; the prohibition is the vocabulary's
  negative space (the capability-shaped fields refuse).
- [x] **ADDRESSED (verified)** — measured before→after. Before:
  the grep above. After: `DATABASE_URL=… cargo test -p
  reasonbraid-server --test profiles
  the_moderation_actions_are_bounded_contributions` →
  `test result: ok. 1 passed` (also inside the full live
  suite: `running 30 tests … ok`) — the custom moderated_panel
  profile composes the step; the solicit-step refusal; the
  classify + the clarification (the reference rides the
  event); the forged-ref + the capability-field (the verdict +
  the evidence) + the misplaced-ref refusals; the challenge
  over the moderation action. The first live pass caught the
  SHARED-registry pollution (the custom profile row broke the
  `.1.2` built-in count) + the validation ORDER (the
  kind-field check pre-empted the moderation refusal) — both
  fixed.
- [x] **NO REGRESSION** — `cargo test --all` → rc=0, 55 suites;
  `bash scripts/run_pg_tests.sh` → rc=0, 18 live suites + the
  demo `ALL acceptance checks passed`
  (`target/pg518_guard.log`);
  `cargo clippy --all --all-targets -- -D warnings` → rc=0;
  `cargo fmt --all -- --check` → rc=0; `make gate` → 13/13 at
  commit.
- [x] **FIX** — `src/workflows.rs`, `src/threads.rs`,
  `tests/profiles.rs`.
- [x] **LOCKSTEP** — CHANGELOG, MEMORY, LIVE_STATUS, this tree's
  logs above, `docs/TASK_TREE.md` frontier — same commit (the
  KNOWLEDGE_MAP regen produced no diff; no new DEV_NOTES
  heading).


## Acceptance Checklist (PHASE-5.3.3)

The CODE change owned by this leaf:
`crates/reasonbraid-server/src/threads.rs` (the `SynthesisInput`
shape, the synthesis kind/step/range validations, the
`max_event_version_in_thread` helper, the event's `synthesis`
field), `crates/reasonbraid-server/src/api.rs` (both create
handlers resolve the profile ALWAYS — the bare thread's steps
were empty), `crates/reasonbraid-server/tests/profiles.rs` (the
new test) — `\.rs$`.

- [x] **REPRODUCE / ISSUE** — the pre-leaf surface: the
  `synthesize` step + the `Summary` kind existed as names, but
  nothing carried the derived-content record; the bare-thread
  steps were EMPTY (the create resolved only a NAMED profile —
  the step gates read `none`).
- [x] **ROOT CAUSE (WHY + WHERE)** — `git grep -c
  "SynthesisInput\|max_event_version_in_thread" e8d11a6 --
  crates/` → rc=1 (nothing before this leaf); the empty-steps
  gap is the `.1.3` Some-only resolve in both create handlers.
  The fix point is the ADR-030 synthesis contract + the
  always-resolve default.
- [x] **ADDRESSED (verified)** — measured before→after. Before:
  the grep above + the live `none`-step refusal. After:
  `DATABASE_URL=… cargo test -p reasonbraid-server --test
  profiles the_synthesis_record_is_derived_content` →
  `test result: ok. 1 passed` (also inside the full live
  suite: `running 31 tests … ok`) — the solicit-step + the
  wrong-kind refusals, the advance → the synthesize-step
  record riding the event (the synthesizer/range/sources/
  coverage), the overreaching + the inverted range refusals;
  the bare thread now carries the quick_advice steps.
- [x] **NO REGRESSION** — `cargo test --all` → rc=0, 55 suites;
  `bash scripts/run_pg_tests.sh` → rc=0, 18 live suites + the
  demo `ALL acceptance checks passed`
  (`target/pg519_guard.log`);
  `cargo clippy --all --all-targets -- -D warnings` → rc=0;
  `cargo fmt --all -- --check` → rc=0; `make gate` → 13/13 at
  commit.
- [x] **FIX** — `src/threads.rs`, `src/api.rs`,
  `tests/profiles.rs`.
- [x] **LOCKSTEP** — CHANGELOG, MEMORY, LIVE_STATUS, this tree's
  logs above, `docs/TASK_TREE.md` frontier — same commit (the
  KNOWLEDGE_MAP regen produced no diff; no new DEV_NOTES
  heading).


## Acceptance Checklist (PHASE-5.4.2)

The CODE change owned by this leaf:
`migrations/0033_evaluation_service.sql` (NEW — the registry +
the run tables), `crates/reasonbraid-server/src/evaluation.rs`
(NEW — the register/record/list + the typed refusals),
`crates/reasonbraid-server/src/lib.rs` (the module),
`crates/reasonbraid-server/src/api.rs` (the four verbs),
`crates/reasonbraid-server/tests/evaluation.rs` (NEW — the
suite), `scripts/run_pg_tests.sh` (the evaluation suite joins
the guard) — `\.rs$` + `(^|/)migrations/` + `scripts/`.

- [x] **REPRODUCE / ISSUE** — the pre-leaf surface: the WP7
  harness ran from static files + a binary; nothing persisted
  a registry or a run; no seed rule, no phantom-corpus check
  (the `.4` census).
- [x] **ROOT CAUSE (WHY + WHERE)** — `git grep -c
  "evaluation_corpora\|evaluation_runs" a2f96a5 -- crates/
  migrations/` → rc=1 (nothing before this leaf). The fix
  point is the ADR-017 service core: the versioned registry +
  the seed-declaring run records persist the harness's
  outputs.
- [x] **ADDRESSED (verified)** — measured before→after. Before:
  the grep above. After: `DATABASE_URL=… cargo test -p
  reasonbraid-server --test evaluation` → `test result: ok. 1
  passed` — the corpus registers (the digests echo), the
  duplicate + the malformed-digest + the seedless-run (the
  undeclared randomness) + the phantom-corpus + the
  duplicate-run refusals, the seeded + the deterministic runs
  record, the lists (the runs newest first), the unenrolled
  read refuses.
- [x] **NO REGRESSION** — `cargo test --all` → rc=0, 56 suites;
  `bash scripts/run_pg_tests.sh` → rc=0, 19 live suites + the
  demo `ALL acceptance checks passed`
  (`target/pg520_guard.log`);
  `cargo clippy --all --all-targets -- -D warnings` → rc=0;
  `cargo fmt --all -- --check` → rc=0; `make gate` → 13/13 at
  commit.
- [x] **FIX** — `0033_evaluation_service.sql`, `src/evaluation.rs`,
  `src/lib.rs`, `src/api.rs`, `tests/evaluation.rs`,
  `scripts/run_pg_tests.sh`.
- [x] **LOCKSTEP** — CHANGELOG, MEMORY, LIVE_STATUS, this tree's
  logs above, `docs/TASK_TREE.md` frontier — same commit (the
  KNOWLEDGE_MAP regen produced no diff; no new DEV_NOTES
  heading).


## Acceptance Checklist (PHASE-5.4.3)

The CODE change owned by this leaf:
`migrations/0034_evaluation_trials.sql` (NEW — the trial +
the append-only results tables), `crates/reasonbraid-server/
src/evaluation.rs` (the `TrialSubmission`/`CohortRecord`/
`StoredTrial` shapes, the `splitmix64` stable draw, the
`create_trial` + `record_trial_results` + the list verbs),
`crates/reasonbraid-server/src/api.rs` (the four trial verbs),
`crates/reasonbraid-server/tests/evaluation.rs` (the new test)
— `\.rs$` + `(^|/)migrations/`.

- [x] **REPRODUCE / ISSUE** — the pre-leaf surface: no trial,
  no assignment, no cohort records, no per-arm results (the
  `.4` census — the harness ran one workflow at a time).
- [x] **ROOT CAUSE (WHY + WHERE)** — `git grep -c
  "evaluation_trials\|splitmix64\|TrialSubmission" 5579701 --
  crates/ migrations/` → rc=1 (nothing before this leaf). The
  fix point is the ADR-017 shadow-trial contract: the SERVER
  computes the seeded assignment (reproducible — never a
  client-supplied draw), the cohorts are recorded labels, the
  results append.
- [x] **ADDRESSED (verified)** — measured before→after. Before:
  the grep above. After: `DATABASE_URL=… cargo test -p
  reasonbraid-server --test evaluation
  the_shadow_trials_record_the_seeded_assignment_and_the_cohorts`
  → `test result: ok. 1 passed` (also inside the full live
  suite: `running 2 tests … ok`) — the assignment covers every
  case with a declared arm, the SAME seed + cases re-draw the
  SAME assignment (the reproducibility proof), the different
  seed differs, the five refusals (the empty arms / the empty
  cases / the unknown cohort kind / the phantom corpus / the
  duplicate), the append-only results (both rows survive), the
  newest-first list.
- [x] **NO REGRESSION** — `cargo test --all` → rc=0, 56 suites;
  `bash scripts/run_pg_tests.sh` → rc=0, 19 live suites + the
  demo `ALL acceptance checks passed`
  (`target/pg521_guard.log`);
  `cargo clippy --all --all-targets -- -D warnings` → rc=0;
  `cargo fmt --all -- --check` → rc=0; `make gate` → 13/13 at
  commit.
- [x] **FIX** — `0034_evaluation_trials.sql`, `src/evaluation.rs`,
  `src/api.rs`, `tests/evaluation.rs`.
- [x] **LOCKSTEP** — CHANGELOG, MEMORY, LIVE_STATUS, this tree's
  logs above, `docs/TASK_TREE.md` frontier — same commit (the
  KNOWLEDGE_MAP regen produced no diff; no new DEV_NOTES
  heading).


## Acceptance Checklist (PHASE-5.4.4)

The CODE change owned by this leaf:
`migrations/0035_evaluation_gates.sql` (NEW — the calibration
+ the gate + the append-only results tables),
`crates/reasonbraid-server/src/evaluation.rs` (the
`CalibrationSubmission`/`GateSubmission` shapes, the
`record_calibration`/`record_gate`/`evaluate_gate` + the
lists), `crates/reasonbraid-server/src/api.rs` (the six verbs),
`crates/reasonbraid-server/tests/evaluation.rs` (the new test)
— `\.rs$` + `(^|/)migrations/`.

- [x] **REPRODUCE / ISSUE** — the pre-leaf surface: no
  calibration record (the harness's Brier computed per-run,
  nothing accumulated), no gate (nothing failed when a score
  dropped — the G5 threshold had no home).
- [x] **ROOT CAUSE (WHY + WHERE)** — `git grep -c
  "evaluation_calibrations\|evaluation_gates\|evaluate_gate"
  fa64621 -- crates/ migrations/` → rc=1 (nothing before this
  leaf). The fix point is the ADR-017 gate contract: the
  calibration accumulates the NAMED runs; the gate records the
  baseline + the threshold and only BLOCKS.
- [x] **ADDRESSED (verified)** — measured before→after. Before:
  the grep above. After: `DATABASE_URL=… cargo test -p
  reasonbraid-server --test evaluation
  the_calibration_accumulates_and_the_gate_only_blocks` →
  `test result: ok. 1 passed` (also inside the full live
  suite: `running 3 tests … ok`) — the calibration over the
  two runs, the ghost-run + the out-of-range refusals, the
  failing evaluation (the case + the baseline + the measured +
  the delta), the all-passing second evaluation appending
  (both rows survive), the threshold/baseline/ghost refusals,
  the lists. The first live pass caught the f64 delta epsilon
  (0.8 − 0.5 ≠ 0.3 exactly) — fixed.
- [x] **NO REGRESSION** — `cargo test --all` → rc=0, 56 suites;
  `bash scripts/run_pg_tests.sh` → rc=0, 19 live suites + the
  demo `ALL acceptance checks passed`
  (`target/pg522_guard.log`);
  `cargo clippy --all --all-targets -- -D warnings` → rc=0;
  `cargo fmt --all -- --check` → rc=0; `make gate` → 13/13 at
  commit.
- [x] **FIX** — `0035_evaluation_gates.sql`, `src/evaluation.rs`,
  `src/api.rs`, `tests/evaluation.rs`.
- [x] **LOCKSTEP** — CHANGELOG, MEMORY, LIVE_STATUS, this tree's
  logs above, `docs/TASK_TREE.md` frontier — same commit (the
  KNOWLEDGE_MAP regen produced no diff; no new DEV_NOTES
  heading).


## Acceptance Checklist (PHASE-5.5.2)

The CODE change owned by this leaf:
`migrations/0036_routing_policy.sql` (NEW — the rule table +
the seven built-ins + the audit table),
`crates/reasonbraid-server/src/routing.rs` (NEW — the
`CASE_CLASSES` vocabulary, the `resolve`, the
`record_resolution`, the lists), `crates/reasonbraid-server/
src/threads.rs` (the create body's `routing_class`),
`crates/reasonbraid-server/src/api.rs` (both create handlers'
policy application + the three verbs),
`crates/reasonbraid-server/tests/routing.rs` (NEW — the
suite), `scripts/run_pg_tests.sh` (the routing suite joins
the guard) — `\.rs$` + `(^|/)migrations/` + `scripts/`.

- [x] **REPRODUCE / ISSUE** — the pre-leaf surface: the
  routing decision was the CLIENT's choice (the explicit
  profile or the hardcoded `quick_advice`); no rule table, no
  class vocabulary, no audit (the `.5` census).
- [x] **ROOT CAUSE (WHY + WHERE)** — `git grep -c
  "routing_rules\|routing_class\|ResolvedRoute" 5d667d1 --
  crates/ migrations/` → rc=1 (nothing before this leaf). The
  fix point is the ADR-031 policy: the deterministic table +
  the create-boundary application with the explicit profile
  outranking the rule.
- [x] **ADDRESSED (verified)** — measured before→after. Before:
  the grep above. After: `DATABASE_URL=… cargo test -p
  reasonbraid-server --test routing
  the_rule_based_policy_routes_deterministically` →
  `test result: ok. 1 passed` — the seven rules, the
  deterministic repeat, the unknown-class refusal, the routed
  create (the class → the independent_panel projection with
  its steps), the explicit profile outranking the rule, the
  bare default unchanged, the create-boundary unknown-class
  refusal, the three audit rows with both surfaces.
- [x] **NO REGRESSION** — `cargo test --all` → rc=0, 57 suites;
  `bash scripts/run_pg_tests.sh` → rc=0, 20 live suites + the
  demo `ALL acceptance checks passed`
  (`target/pg523_guard.log`);
  `cargo clippy --all --all-targets -- -D warnings` → rc=0;
  `cargo fmt --all -- --check` → rc=0; `make gate` → 13/13 at
  commit.
- [x] **FIX** — `0036_routing_policy.sql`, `src/routing.rs`,
  `src/threads.rs`, `src/api.rs`, `tests/routing.rs`,
  `scripts/run_pg_tests.sh`.
- [x] **LOCKSTEP** — CHANGELOG, MEMORY, LIVE_STATUS, this tree's
  logs above, `docs/TASK_TREE.md` frontier — same commit (the
  KNOWLEDGE_MAP regen produced no diff; no new DEV_NOTES
  heading).


## Acceptance Checklist (PHASE-5.5.3)

The CODE change owned by this leaf:
`migrations/0037_routing_recommendations.sql` (NEW — the
recommendation table), `crates/reasonbraid-server/src/
routing.rs` (the `RecommendationSubmission` shape, the
`record_recommendation` + the list),
`crates/reasonbraid-server/src/api.rs` (the two verbs),
`crates/reasonbraid-server/tests/routing.rs` (the new test) —
`\.rs$` + `(^|/)migrations/`.

- [x] **REPRODUCE / ISSUE** — the pre-leaf surface: no
  learned-routing surface existed (the `.5` census — the
  routing was the rule table + the client's choice only).
- [x] **ROOT CAUSE (WHY + WHERE)** — `git grep -c
  "routing_recommendations\|RecommendationSubmission" dc02a50
  -- crates/ migrations/` → rc=1 (nothing before this leaf).
  The fix point is the ADR-031 shadow contract: the record
  with the registered-arm constraint + the named evidence +
  the stated non-application.
- [x] **ADDRESSED (verified)** — measured before→after. Before:
  the grep above. After: `DATABASE_URL=… cargo test -p
  reasonbraid-server --test routing
  the_shadow_recommendation_records_and_never_applies` →
  `test result: ok. 1 passed` (also inside the full live
  suite: `running 2 tests … ok`) — the recorded recommendation
  with `applied: false`, the phantom-arm + the ghost-evidence
  + the unknown-class refusals, the shadow-create still on
  the RULE's arm (the never-applied proof), the list.
- [x] **NO REGRESSION** — `cargo test --all` → rc=0, 57 suites;
  `bash scripts/run_pg_tests.sh` → rc=0, 20 live suites + the
  demo `ALL acceptance checks passed`
  (`target/pg524_guard.log`);
  `cargo clippy --all --all-targets -- -D warnings` → rc=0;
  `cargo fmt --all -- --check` → rc=0; `make gate` → 13/13 at
  commit.
- [x] **FIX** — `0037_routing_recommendations.sql`,
  `src/routing.rs`, `src/api.rs`, `tests/routing.rs`.
- [x] **LOCKSTEP** — CHANGELOG, MEMORY, LIVE_STATUS, this tree's
  logs above, `docs/TASK_TREE.md` frontier — same commit (the
  KNOWLEDGE_MAP regen produced no diff; no new DEV_NOTES
  heading).


## Acceptance Checklist (PHASE-5.6.2)

The PACKAGE change owned by this leaf (no code — the records + the
landing-page fix):
`docs/decisions/2026-09-07_phase5-gate-record.md` (NEW),
`docs/decisions/2026-09-07_phase5-subtraction-record.md` (NEW),
`docs/evidence/2026-09-07_phase5-evidence-manifest.md` (NEW),
`README.md` (the stale status line — the S-5 candidate),
`docs/decisions/INDEX.md` + `docs/evidence/INDEX.md` (the rows) —
`\.md$`.

- [x] **REPRODUCE / ISSUE** — the pre-leaf state: the G5 gate
  had no record, no subtraction record, no manifest; the
  README's status line still read "Phase 0 — contracts and
  kill-risk experiments" (the `.6.1` census's S-5).
- [x] **ROOT CAUSE (WHY + WHERE)** — the exit gate's package
  was unwritten (the machinery shipped across `.1`–`.5`, the
  gate's evidence unassembled); the README's status line was
  a hand-kept fact nobody refreshed. The fix point is the
  gate-package pattern (the Phase-1/Phase-4 precedents): the
  record + the subtraction + the manifest + the landing-page
  alignment.
- [x] **ADDRESSED (verified)** — the gate record states **G5
  Met as a subtraction gate** (the blocking function
  discharged: no "deliberation improves answers" claim exists
  — the `.6.1` census + the H1-null sample per §25.1); the
  subtraction record carries every §19.8 list (none empty);
  the manifest maps every G5 clause to a re-runnable
  artifact; `git grep -c "Phase 0 — contracts" HEAD --
  README.md` → rc=1 after the edit.
- [x] **NO REGRESSION** — `cargo test --all` → rc=0, 57 suites;
  `bash scripts/run_pg_tests.sh` → rc=0, 20 live suites + the
  demo `ALL acceptance checks passed` (the `.6.1` guard —
  no code changed since); `cargo clippy --all --all-targets
  -- -D warnings` → rc=0; `cargo fmt --all -- --check` →
  rc=0; `make gate` → 13/13 at commit; `make book` builds.
- [x] **FIX** — the three records, `README.md`, the two
  INDEX rows.
- [x] **LOCKSTEP** — CHANGELOG, MEMORY, LIVE_STATUS, this
  tree's logs above, `docs/TASK_TREE.md` (the PHASE-5 row →
  done, the PHASE-6 row → active) — same commit.
