# Phase 5 SubtractionRecord (`PHASE-5.6.2`; ROADMAP §19.8)

- gate_id: `PHASE-5-G5`
- evidence_revision: the `.6.2` close commit (this record ships in it)
- Date: 2026-09-07

The architecture ratchet counter: what Phase 5 actively did NOT build, and
why that is a decision rather than an omission. The census (`.6.1`) named
five candidates; each list here carries them with their trigger. No list is
empty.

## features_removed

- **The "structured deliberation improves answers" claim** — never shipped,
  and the first controlled evaluation reads H1 null (no structured
  workflow beat `single` on the 4-case differential sample at 2–4× the
  cost); the claim is WITHDRAWN, not softened: the product claims the
  machinery, not the lift.
- **The implicit-`quick_advice` routing shortcut** — replaced by the
  explicit `.5.2` rule table: the bare-thread default is now the RULE's
  artifact (the audited resolution), not a hardcoded string.

## features_deferred (with revisit triggers)

| Feature | Revisit trigger |
| --- | --- |
| The enduring multi-run accumulation (S-2: the calibration claim) | the `.4` service accumulates enough seeded runs; the Brier sample today is one factual case |
| The learned-routing policy (S-3) | a future gate over the `.5.3` shadow records — the recommendation stays recorded, never applied |
| The §19.5 axes the harness does not measure (the discovery-relevance, the correlated-agent robustness, the human-review demand) | the harness's corpus grows; nothing claims them until they measure |
| A new-quality-capability profile (any step outside the thirteen-kind vocabulary) | a contract change — the ADR-016 composition rule |

## product_claims_narrowed

- **"Deliberation quality" → "deliberation machinery, measured."** The
  shipped claim: the deterministic machinery (the profiles, the structured
  records, the blind commitment, the moderation bounds, the synthesis
  coverage, the honest terminals) exists and is tested; the QUALITY LIFT
  over the single-agent baseline is unclaimed (null on the sample).
- **"The routing policy improves outcomes" → "the routing policy is
  deterministic and auditable."** The rule table is a lookup over the
  submitted class; the evidence for learned routing is shadow-only.

## abstractions_or_generalizations_rejected

- **A server-side grader** (ADR-017): the service RECORDS, the harness
  MEASURES — a second grading implementation was rejected as drift risk.
- **A routing class derived from content** (ADR-031): the class is a
  SUBMITTED input — the derived-classifier was rejected as an untested
  judgment.

## dependencies_or_services_avoided

- **A randomness dependency for the trial assignment**: the splitmix64
  draw is dependency-free (the `std` hasher is not stable across
  releases — the assignment must be).
- **A learned-routing engine**: no model serves routing today; the
  recommendation is a recorded row, not a service.

## manual_fallbacks_accepted (with limits)

- **The corpus digest verification**: the registry stores the declared
  digests; the harness re-derives them at run time (the file-byte check
  stays in the runner — the service never re-implements it).
- **The human authority over the rule**: the explicit profile outranks
  the policy — a deliberate, bounded manual override, audited.

## operations_and_persistent_entities_eliminated

- **No per-trial production routing state**: the trials write the
  `evaluation_trials` rows only; the production routing consults the
  `routing_rules` table — the two never mix.

## estimated effort and risk removed

- The withdrawn claim removes the evidence debt a false claim would
  carry (the re-run programme, the rebuttal surface); the shadow-only
  recommendation removes the policy-rollout risk surface (no learned
  routing can raise authority/spend/access/side-effects).

## proposals retained in parking_lot

- The learned-routing policy flip (the `.5.3` records are the evidence
  a future gate weighs).
- The derived case-class classifier (ADR-031 named it out — parked, not
  lost).

## owner, reviewers, rationale, signatures

- Owner: the Phase-5 lane owner (director-attested records). Reviewers:
  the gate package rides the doctrine gates (13/13) + the full guard.
  Rationale: §25.1's post-Phase-5 instruction executed verbatim — the
  controlled evaluation did not beat the cheaper baseline, so the claims
  narrow.
