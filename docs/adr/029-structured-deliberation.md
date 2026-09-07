# ADR-029 — Structured deliberation: typed disagreement over the same verbs — the records shape the wire, never a new capability

- **Status:** `accepted` (evidence-gated — the §13.4/§13.6 lane's
  contract: the typed records, the blind commitment point, the
  evidence requests, the adjudication, and the minority report are
  the shapes the `.2.2`–`.2.4` leaves implement)
- **Date:** `2026-09-07`
- **Leaf:** `PHASE-5.2.1`
- **Requirements:** `ROADMAP.md` §13.2 (the rigorous deliberation
  reference flow), §13.4 (convergence and honest failure), §13.6
  (anti-herding measures)

## Context

The `.2` census mapped §13.4/§13.6 against the shipped surface. The
challenge/revise pair is FREE TEXT against a target event id — no typed
claim, objection, or revision exists. Nothing defers visibility: a
contribution is readable at post time, so a blind initial position is
impossible (`git grep -c "blind" HEAD -- crates/reasonbraid-server/src/`
→ the 4 hits are the `blind_solicit` step NAME in `workflows.rs`).
`evidence_request` and `adjudicate` are step names only (1 and 5 hits,
the latter including the delivery-fate adjudication in
`node_channel.rs`). No minority report exists, and the close carries
two outcomes vs §13.4's twelve. The pieces the lane REUSES exist: the
contribution kind vocabulary (six kinds incl. `claim`/`assumption`)
+ the `evidence_refs` (Phase 1/4), the open_challenges register, the
unresolved register riding the close, and the Phase-4 evidence
pipeline (the claim assessments + the citation validation).

## Decision

- **The records are SHAPES over the existing verbs, never a new
  capability.** The typed claim rides the `thread.contribute` body
  (the kind is `claim` + the claim's content; the contribution event
  is the claim's identity — a claim is a contribution, not a new
  entity). The objection rides `thread.challenge` and targets a
  claim (the challenged contribution event + the claim index within
  it). The revision rides `thread.revise` and answers the objection.
  The authority/budget/lifecycle checks keep living in the verbs —
  the structure adds no transition path (ADR-016's boundary).
- **The claim is content-addressed at the wire's edge.** A claim
  record carries its digest; an objection names the digest it
  targets; a revision names the objection. The lineage is the event
  sequence — a revision never mutates a prior event (the Phase-1
  immutability stands).
- **The blind commitment point is a READ-SURFACE rule, not a store
  rewrite.** A blind contribution is written at post time and is
  visible pre-commitment only to its author and to the authority
  surface (the audit view); every other reader sees the digest + the
  `blind until <commitment>` marker. The commitment point is the
  round advance (the existing `thread.advance_round` transition) —
  no new verb. The §13.6 no-totals rule rides the same surface: no
  aggregation of blind content is served before the commitment
  point.
- **The evidence request rides the Phase-4 pipeline.** The
  `evidence_request` step executes as a contribution of kind
  `evidence_request` targeting a claim digest; the response is a
  contribution carrying `evidence_refs` that must pass the existing
  citation validation (Phase 4.6.3). A request is NOT an
  acquisition — it grants no budget, no resolver authority, no
  fetch (the acquire path stays the Phase-4 resolver's, under the
  envelope).
- **The adjudication is an attributable verdict record, never a
  silent rewrite.** The `adjudicate` step executes as a contribution
  of kind `summary`-adjacent verdict shape: the verdict names the
  proposal digest it judges, the rule it applies, and the outcome it
  declares; the event is attributable (the adjudicator's principal
  rides it, like every contribution). The thread's decided outcome
  references the verdict event — the system never rewrites
  disagreement as consensus (§13.4).
- **The minority report is derived content with a coverage
  report** (§13.5): the synthesizer identity/configuration, the
  input event range, the source links, and the list of objections/
  uncertainty INCLUDED or explicitly EXCLUDED (with the reason). It
  rides the close event (or the last contribution before it).
- **The close outcome speaks §13.4's twelve terminals.** The close
  body's `outcome` gains the twelve named terminals
  (`accepted_unanimously`, `accepted_with_recorded_objections`,
  `accepted_by_rule`, `advisory_answer_only`, `deadlocked`,
  `no_quorum`, `insufficient_evidence`, `budget_exhausted`,
  `expired`, `cancelled`, `human_decision_required`,
  `unsafe_to_continue`). The two legacy words remain accepted
  aliases (`decided` → `accepted_by_rule`, `inconclusive` → the
  unresolved register's honest failure terminal, defaulting to
  `deadlocked` when unnamed). A decided terminal that carries
  unresolved items is the typed refusal (the existing rule — §13.4
  forbids rewriting disagreement as consensus).
- **The registers stay honest.** `open_challenges` counts the
  STRUCTURED objections (the register's meaning is unchanged — the
  objection just gained a shape); the unresolved register keeps
  riding the close unchanged.

## Consequences

- `.2.2` implements the typed records (the wire shapes + the
  projection + the honest registers), `.2.3` the blind commitment
  point (the read-surface deferral + the round-advance commitment),
  `.2.4` the evidence requests + the adjudication + the minority
  report + the terminal vocabulary — each against this contract
  verbatim; a deviation is a contract change.
- The state machine stays the ONLY transition path — the records
  ride the existing verbs, so the Phase-1 lifecycle proofs continue
  to hold for every structured deliberation.
- The §13.6 anti-herding rules that remain OUT of scope here (the
  randomized order, the context partitioning, the independence
  badges' prohibition) belong to the `.5`/`.6` routing-evaluation
  lane, not to this contract — this lane ships the structure the
  evaluation programme will measure.

answers:

- **Typed disagreement is the cheap half of deliberation.** The
  objection/revision pair already exists as free text; the typed
  shape adds the digest-targeting the verification and the
  evaluation programme need — the verbs, the authority, the budget,
  and the lifecycle stay untouched, so the shape is safe by
  construction (the same argument ADR-016 makes for the profiles).
- **Blindness is a visibility property, not a storage property.**
  Deferring READ access at the projection keeps the event store
  immutable and the audit complete — the blind content is in the
  ledger from post time, and the commitment point is the existing
  round transition, so nothing new can leak or rewrite it.
- **The verdict and the minority report are evidence, not
  authority.** Both are attributable events over the same verbs —
  they can be wrong, but they can never be silent, and the
  coverage report makes the synthesizer's choices checkable.
