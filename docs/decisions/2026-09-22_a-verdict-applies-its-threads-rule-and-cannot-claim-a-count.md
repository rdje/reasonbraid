---
answers:
  - Does an adjudication verdict state its own decision rule?
  - May a verdict declare accepted_unanimously, accepted_with_recorded_objections or no_quorum?
  - What does a verdict event record as its rule?
  - How were the existing verdict fixtures re-derived rather than edited to pass?
---
# A verdict applies its thread's rule, and cannot claim a count

- **Type:** decision
- **Status:** accepted
- **Owner:** `SIGNOFF-REPAIR.8.1.1.3`
- **Date:** 2026-09-22
- **Cites:** `docs/adr/029-structured-deliberation.md` (*the adjudication is an
  attributable verdict record, never a silent rewrite … the verdict names the proposal
  digest it judges, the rule it applies, and the outcome it declares*; *the verdict …
  [is] evidence, not authority*), ROADMAP §13.3–§13.4;
  `docs/decisions/2026-09-22_a-counted-rule-derives-its-outcome-and-a-rule-it-cannot-count-is-refused.md`
  (its §5 table)

## Context

`VerdictInput` was `{ target_digest, rule: String, outcome: CloseOutcome }`. The rule was
free text, and the outcome could be any of §13.4's words. So one adjudicator could
record `rule: "unanimity"` with `accepted_unanimously` on a thread that declared
`owner_decides`, and `tests/policy.rs` did exactly that. A verdict never reaches the
thread's close outcome (ADR-029: evidence, not authority), but its words still claimed
a count that one person cannot make.

## Decision

1. **The rule leaves the input.** ADR-029's verdict *names the rule it applies*, and the
   rule a verdict applies is the thread's. The server writes the thread's declared
   `decision_rule` into the verdict event, or `null` when the thread declared none. A
   request carrying `rule` is refused as an unknown field. ⭐ ADR-029's sentence is
   honoured more strictly than before: the rule is now DERIVED, where it used to be
   asserted.
2. **Three outcomes are refused on a verdict:** `accepted_unanimously`,
   `accepted_with_recorded_objections` and `no_quorum`. These are exactly the outcomes
   `.8.1.1`'s §5 table derives from a tally. A verdict keeps every other §13.4 word,
   because each of those is a judgement one person can make.
3. **A `verdict`-kind contribution must carry its `verdict`.** It used to be optional, so
   a verdict that judged nothing could be recorded.
4. **Order:** the `adjudicate` step gate runs first. A verdict out of turn keeps that
   gate's refusal, even when its outcome is also a count word.

## How the fixtures were re-derived

Sixteen fixtures across two suites sent a verdict, and every one sent `rule`. Each was
re-derived for what it proves, not edited to pass:

- **`tests/policy.rs`**: 12 fixtures. Eleven supply the `verdict_event_id` a policy
  decision cites as supporting evidence, on threads that declare `owner_decides`; their
  `rule: "majority"` claimed a rule the thread does not use, and it is removed. The
  twelfth recorded `unanimity` with `accepted_unanimously`. It now asserts that claim is
  REFUSED, that a request still sending `rule` is refused, and that the valid verdict's
  event records the derived `"owner_decides"`.
- **`tests/profiles.rs`**: 4 fixtures. The assertion that the recorded rule is
  `"majority"` now asserts `null`, because that thread declares no rule. The early-verdict
  arm keeps its count-word outcome deliberately: it shows the step gate still answers
  first. New arms show a `no_quorum` verdict refused on its own step, and a verdict-kind
  contribution with no verdict refused.

Census of producers: `git grep -n '"kind": "verdict"' -- crates` finds no product caller
(CLI, MCP, node or console) that constructs a verdict. The two suites are the whole
population.

## Consequences

- ⚠️ **A wire change on one contribution kind:** `verdict.rule` is gone. Nothing in the
  product sent it, and the old verdict events in the log keep their free-text rule as
  historical fact.
- ✅ A verdict can no longer state a count or a rule its thread does not have. ADR-029's
  *never a silent rewrite of disagreement as consensus* is now enforced at the verdict as
  well as at the close.
