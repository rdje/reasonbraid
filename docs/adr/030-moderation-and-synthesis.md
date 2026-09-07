# ADR-030 — Moderation and synthesis: the prohibition is the vocabulary's negative space — the moderator gets no new verbs

- **Status:** `accepted` (evidence-gated — the §13.5 contract: the
  moderation kinds, the structural prohibitions, the appealable
  action, and the synthesis record are the shapes the `.3.2`/`.3.3`
  leaves implement)
- **Date:** `2026-09-07`
- **Leaf:** `PHASE-5.3.1`
- **Requirements:** `ROADMAP.md` §13.5 (moderator and synthesizer
  boundaries)

## Context

The `.3` census mapped §13.5 against the shipped surface. The
moderator is ZERO machinery: no role, no action, no kind, no step —
`git grep -n "moderator" HEAD -- crates/reasonbraid-server/src/`
finds only comments, and the step vocabulary has no `moderate`. The
synthesizer is half-shipped: the `synthesize` STEP name and the
`Summary` contribution kind exist, and the derived-content shape
(the identity, the input event range, the sources, the coverage
report) exists ONLY as the `.2.4.1` minority report. The §13.5
prohibitions (no vote, no silent dissent suppression, no evidence
fabrication, no electorate change, no spend, no policy publication)
have no enforcement surface because nothing exists to enforce them
on. The reusable pieces: the contribute verb's authority/budget
checks (ADR-016's boundary — anything riding the verb is bounded by
them), the challenge verb (the appeal), and the `.2.4.1` coverage
shapes.

## Decision

- **The moderation action is a CONTRIBUTION, never a new role or
  authority.** The moderator is a function, not a principal class:
  a moderation action rides `thread.contribute` with a moderation
  kind. The existing authority/budget/lifecycle checks ride it
  unchanged — the moderator's powers are exactly the verbs' powers,
  bounded by the same envelopes.
- **The closed kind vocabulary IS the structural prohibition.** The
  moderation kinds are a closed set: `classify`, `request_clarification`,
  `propose_close`, `draft_summary`, `identify_unanswered`. A
  moderation-kind contribution REFUSES the capability-shaped fields
  (the `verdict`, the `claims`, the `evidence_refs`, the
  `target_claim_digest`) — the §13.5 acceptance criteria hold BY
  CONSTRUCTION: no vote/verdict, no evidence fabrication, no spend
  (the contribute verb's own budget draw is the only effect), no
  electorate change (the invite/remove verbs are separate and
  authority-checked), no policy publication (no such verb exists).
  A moderation action MAY reference the event it acts on via
  `ref_event_id` (the classify names the message it classifies) —
  the reference must exist in the thread.
- **A moderation action can never erase.** The action is a NEW
  event; the moderated content stays in the ledger. The only
  "suppression" is a visible, attributable, challengeable action —
  the reader sees both the original and the action.
- **The appealable moderation action is the existing challenge.**
  A moderation contribution is challengeable exactly like any
  contribution (the challenge targets its event id) — "appealable"
  is not a new verb, the challenge IS the appeal.
- **The `moderate` step enters the step vocabulary** (the
  twelve kinds become thirteen; the moderation kinds execute on
  the `moderate` step — the same gate the `.2.4.2` kinds use).
  The existing built-in profiles stay unchanged; custom profiles
  may compose the step.
- **The synthesis record is derived content, auditable by
  construction.** The `synthesize` step executes as a `Summary`-kind
  contribution carrying the synthesis record: the synthesizer
  identity/configuration, the INPUT EVENT RANGE (the event log's
  version range — the transformation is re-derivable), the source
  links, and the coverage report (the included/excluded items with
  reasons — the `.2.4.1` shapes, generalized). The synthesis never
  mutates a prior event: it is itself an event.

## Consequences

- `.3.2` implements the moderation kinds (the typed shapes, the
  capability-field refusals, the `ref_event_id` check, the
  `moderate` step gate) and `.3.3` the synthesis record — each
  against this contract verbatim; a deviation is a contract change.
- The moderator's ceiling is the contribute verb's ceiling: no new
  authority table, no new grant, no new budget lane — the Phase-1/
  Phase-2 proofs continue to hold for every moderation action.
- The synthesis's coverage report makes the synthesizer's choices
  checkable; the input range makes its transformation re-derivable.

answers:

- **The prohibition is the vocabulary's negative space.** §13.5's
  bans enumerate capabilities; the implementation bans the
  capability-shaped FIELDS on the moderation kinds — structural
  enforcement instead of a behavioral promise, the same argument
  ADR-016 makes for the profiles.
- **Appealability is challengeability.** Making the moderation
  action a contribution gives the appeal for free: the challenge
  verb already targets any contribution, and the action can never
  hide what it challenges.
- **The synthesis is evidence, not authority.** The record is an
  attributable event naming its inputs and its coverage — it can be
  wrong, but never silent, and its transformation is re-derivable
  from the named event range.
