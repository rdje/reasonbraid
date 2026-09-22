---
answers:
  - Can a thread with no declared decision rule close as accepted_unanimously, accepted_with_recorded_objections or no_quorum?
  - Why was refusing those words rejected before, and why is it right now?
  - When can declaring a decision rule become mandatory?
---
# A thread that declared no rule cannot close on a count

- **Type:** decision
- **Status:** accepted
- **Owner:** `SIGNOFF-REPAIR.8.1.1.5`
- **Date:** 2026-09-22
- **Cites:** ROADMAP §13.3–§13.4;
  `docs/decisions/2026-09-22_a-counted-rule-derives-its-outcome-and-a-rule-it-cannot-count-is-refused.md`
  (§5, §6, alternative 3, which this record reverses);
  `docs/decisions/2026-09-22_a-verdict-applies-its-threads-rule-and-cannot-claim-a-count.md`

## Context

A thread with no `decision_rule` closed on its closer's word, including
`accepted_unanimously`, `accepted_with_recorded_objections` and `no_quorum`. Each of
those describes a count of ballots, and on such a thread nothing was counted. The `.8.1.1`
record made this visible (`outcome_provenance: "caller_asserted"`) and rejected refusing
the words (alternative 3). It gave two reasons:
- that change would edit the twelve-terminal walk's assertions, which `.8.1.1`'s own
  acceptance forbade;
- a rule cannot be made mandatory while charters fail closed.

## Decision

1. **A rule-less close stating one of the three count words is refused**, with
   `CloseRefusal::NoRuleDeclared`. The message names the word and says to declare a
   counted rule at creation. Every other outcome is still accepted as stated, marked
   `caller_asserted`.
2. **The three words are one list**, `threads::names_a_count`, shared with the verdict
   refusal from `.8.1.1.3`. The §5 table and every refusal of a count word read the same
   definition.

### Why alternative 3's two reasons no longer hold

- **The walk was `.8.1.1`'s constraint, not this leaf's.** This leaf owns the change. The
  walk is **re-derived**, not relaxed: it still visits all twelve words, asserts the
  refusal for the three, and names where they are derived when a count exists.
- **Refusing a word does not make a rule mandatory.** A rule-less thread still creates and
  still closes. It can say `accepted_by_rule`, `deadlocked`, `advisory_answer_only` or any
  process terminal. It simply cannot claim a tally. ⭐ **And this is exactly §5's existing
  row for `owner_decides` and `advisory_synthesis`**, which already refuse the three as
  "nothing counted". A thread that declared LESS than those rules could claim MORE than
  they can. That inversion was the defect.

## What stays open — owned, with a condition anyone can check

Making `decision_rule` **mandatory** is still wrong today. Every enrollment boundary names
a label charter, which fails closed, so a mandatory rule would refuse every create. It
becomes possible exactly when **no active enrollment boundary names a `charter_digest` with
no registered `governance_charters` row**. That is one SQL query, and `.8.1.1.5.1` owns it.

## Re-derivation of the fixtures

- `profiles.rs` `the_twelve_terminals_and_the_minority_report_ride_the_close`: the three
  words now assert the refusal; the other nine are unchanged.
- The same file's dishonest-close control (decision word plus unresolved register) used
  `accepted_unanimously` incidentally; it uses `accepted_by_rule` so it keeps testing that
  rule.
- `a_declared_rule_is_checked_against_the_charter_after_authorization`'s rule-less arm now
  asserts the refusal first, then closes with `accepted_by_rule` for its provenance checks.
- `policy.rs`'s `pdc-ruleless` control needed a caller-asserted close and nothing else, so
  it uses `accepted_by_rule`.
- ⚠️ The first census of these fixtures MISSED the third one: a line was read as a
  declared-rule test without checking its thread. The suite caught it, and each remaining
  use was then classified against its own thread's `decision_rule`.
