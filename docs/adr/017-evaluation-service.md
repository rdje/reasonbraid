# ADR-017 — The evaluation service: records and gates, never a second judge — the harness measures, the service persists and blocks

- **Status:** `accepted` (evidence-gated — the §13.7/§19.5 contract:
  the versioned registry, the experiment records, the shadow
  routing trials, the cohorts, the calibration, and the G5 gate
  are the shapes the `.4.2`–`.4.4` leaves implement)
- **Date:** `2026-09-07`
- **Leaf:** `PHASE-5.4.1`
- **Requirements:** `ROADMAP.md` §23 queue item 017 (evaluator
  hierarchy and release thresholds); §13.7 (the hypothesis and
  routing evaluation programme), §19.5 (deliberation evaluation),
  §19.6 (the G5 gate), §19.7 (the gate manifest + the declared
  seeds)

## Context

The `.4` census mapped §13.7/§19.5 against the shipped surface.
The WP7 bench harness (`crates/reasonbraid-adapter/bench/`, Phase
0) IS a substantial substrate: the versioned corpus (`Corpus {
version, cases }` — the nine cases with the ground truths, the
rubrics, the expected scores), the four deliberation workflows
over the real Adapter contract, the deterministic grading
(`Graded { score, confidence, normalized, checks_present,
citations_valid/invalid }` + the `Trap` honesty check + the
Brier), the spread-bearing reports, the ScriptedAgent self-test,
the `rb-bench` binary. The GREENFIELD (backlog 37): no evaluation
SERVICE — the harness is static files + a binary; nothing persists
a run, no registry, no experiments, no cohorts, no calibration
record, no regression gate. The `.1`–`.3` lanes just shipped the
deliberation machinery the evaluation will measure.

## Decision

- **The service RECORDS; the harness MEASURES.** The evaluation
  service is a recording + gating service — the WP7 harness
  remains the single runner (the workflows, the grading, the
  reports). The service persists the registry, the experiment
  records, the trial assignments, and the gate evaluations; it
  never re-implements a grader (one grading implementation, the
  harness's — a second judge would invite drift).
- **The case registry is versioned + content-addressed.** The
  registry stores corpus VERSIONS with the ADR-011 digests of
  `corpus.json`/`prompts.json`; every case row references its
  corpus version. The digest pins the measurement: a re-run with
  the same version + the declared seed must reproduce (§19.7 —
  non-deterministic evaluations use declared seeds + repeated
  trials).
- **The experiment record declares its seed.** Every run record
  carries the workflow arm, the corpus version, the seed (or
  `null` for a deterministic run), the trial count, and the
  result rows (the per-case `Graded` + the `Trap` + the cost).
  The seed is part of the record's identity — an undeclared
  randomness is a typed refusal, never a silent guess.
- **The randomized routing trial is a SHADOW experiment.** A
  routing trial assigns cases to arms by the seeded
  randomization and RECORDS the assignment + the per-arm
  results; it never changes the production routing (§13.8: the
  learned routing stays behind the rule-based baseline, shadow
  mode first). The trial's record is the evidence the `.5` lane's
  routing decision will consume.
- **The cohort is a recorded label, never a derived claim.** The
  cohort records attach the subject/case classes to runs; the
  service aggregates per-cohort ONLY over the recorded
  assignments. No independence badge: the cohort report states
  what was measured (§13.6's prohibition, applied to the
  reports).
- **The calibration record accumulates; the model grader never
  judges alone.** The Brier + the confidence accumulate across
  runs into the calibration record (the per-case + the
  aggregate). The record feeds the regression gate and the
  human review; it is never the sole authority for a
  high-impact correctness claim (§19.5).
- **The regression gate is the G5 threshold, and it only
  BLOCKS.** The gate compares each case's measured score
  against the recorded baseline (the corpus's expected scores
  or the established baseline row); a drop below the threshold
  is the typed failure the CI manifest's G5 row consumes. The
  gate never mutates a result — it evaluates.

## Consequences

- `.4.2` implements the service core (the registry + the
  experiment records + the result persistence), `.4.3` the
  shadow routing trials + the cohort tracking, `.4.4` the
  calibration + the regression gate — each against this
  contract verbatim; a deviation is a contract change.
- The harness's grading stays the ONLY grading: the service
  tables store its outputs, so the self-test's expected-score
  proof continues to hold for every recorded run.
- The `.5` routing lane consumes the trial records — the
  rule-based baseline's shadow evidence arrives before any
  learned routing exists.

answers:

- **A second judge is the risk the service must not take.** The
  harness already grades deterministically; a service-side
  grader would create two measurement implementations and
  correlated-error drift. Persisting the harness's outputs is
  the safe half of the evaluation programme — the service adds
  durability and gating, not opinion.
- **The seed is the experiment's honesty token.** §19.7 demands
  declared seeds and repeated trials; making the seed part of
  the record's identity turns an undeclared randomness into a
  typed refusal instead of an unverifiable run.
- **Shadow first is not a caution, it is the order.** A routing
  trial that could change production routing would be a policy
  change riding an experiment; the shadow rule keeps the trial
  evidence-only until the `.5` lane's gate explicitly flips it.
