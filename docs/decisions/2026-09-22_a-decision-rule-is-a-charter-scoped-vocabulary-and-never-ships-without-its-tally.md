---
answers:
  - What does the roadmap actually require of a thread's decision rule and expected artifact?
  - Are the two fields one commitment or two, and does `objective` already stand in for the artifact?
  - Why must the decision rule NOT ship as a create field on its own?
  - What is the prerequisite nobody owned before this record?
  - Where else does the system declare a decision result it never computed?
  - Which of these is owed before G9?
---
# A decision rule is a charter-scoped vocabulary, and it never ships without the tally that reads it

- **Type:** decision
- **Status:** accepted
- **Owner:** `SIGNOFF-REPAIR.11.4.7.2.1.2`
- **Date:** 2026-09-22
- **Cites:** ROADMAP §3.2 (thread creation), §4.1 (the governance charter), §13.2 (the
  deliberation walk, steps 10–11), §13.3 (decision rules), §26 (the LAN walk, step 2),
  §19.6 (the release gate matrix);
  `docs/decisions/2026-09-22_the-remaining-roadmap-gaps-are-sequenced-by-exposure-then-deletion.md`
  (this is that record's item 4, and it gates item 6)

## Context

The Phase-5 deferral *the create fields Phase 5's engine was supposed to bring* was graded
`fired and open`, and `.15` made settling its contract item 4 — **before** `.8.1.1` builds the
ballot it would be counted under, because a tally against an undecided rule is the rework the
wave order exists to prevent.

⭐ **Unlike the two leaves before it, this leaf's premise HOLDS — and is understated.**
`.9.3.5` and `.4.6` were both graded open over work already shipped. This one is not: the
gap is real, and measurement made it larger rather than smaller.

## The contract, quoted rather than paraphrased

| source | the roadmap's own words |
| --- | --- |
| **§3.2**, thread creation | *decision rule, electorate policy, quorum, veto scope, and human role when applicable* — one of eight creation bullets |
| **§13.3**, decision rules | **seven** supported rule families: *owner decides after consultation; simple or supermajority of a defined electorate; unanimity of all non-recused electorate members, with explicit abstention semantics; consensus with no unresolved blocking objection; role-weighted or chambered approval defined by a governance charter; human committee approval; advisory synthesis with no binding decision* |
| **§13.2**, steps 10–11 | *Snapshot electorate and collect vote/approval actions **under the declared rule***, then ***Compute the deterministic decision result*** |
| **§4.1**, the charter | the versioned `GovernanceCharter` defines *allowed decision rules and approval thresholds* |
| **§26**, the walk, step 2 | *A human creates a thread with objective, **expected artifact**, participant constraints, budget, and manual decision rule* |

⭐ **§26's sentence settles a question the leaf carried open.** The deferral recorded that
`objective` stands in for the expected artifact. §26 names **`objective` and `expected
artifact` in one list, as two items** — so they are not the same field, and the stand-in
reading is refuted by the roadmap's own sentence rather than by preference.

## The measurement

| what the contract requires | what is shipped |
| --- | --- |
| `decision_rule` at creation | 🔴 absent. `threads::CreateBody` carries `tenant_id`, `subject`, `objective`, `budget`, `classification`, `workflow_profile`, `routing_class`, `participant_rules` — and neither field |
| `expected_artifact` at creation | 🔴 absent, and not covered by `objective` (above) |
| a typed §13.3 vocabulary | 🔴 absent. `git grep -n "expected_artifact\|decision_rule" -- crates migrations` returns **one** hit and it is `reasonbraid-a2a`'s `SemanticLosses.decision_rule` — a **bool** recording that a remote decision rule is ALWAYS LOST |
| the charter's *allowed decision rules and approval thresholds* | 🔴 absent. `enrollment_boundaries.charter_digest` is a `TEXT` digest, not a structure; no table holds an allowed-rule set |
| §13.2 step 11's *deterministic decision result* | 🔴 **declared, never computed** — at BOTH decision surfaces, below |

### The declared-not-computed defect, at two surfaces

🔴 **Thread close.** `threads::VerdictInput` is `{ target_digest, rule: String, outcome:
CloseOutcome }` — the rule is a **free-form string** the caller supplies to name *the decision
rule applied*, beside an outcome the same caller declares. Nothing interprets the string, and
`CloseOutcome::AcceptedByRule` is assertable although no rule was ever declared at creation.

🔴 **Policy approval.** `lifecycle::ApprovalInput.quorum` is a `serde_json::Value`, and its
only validation is that `quorum.participants` is a non-empty array (`lifecycle.rs:503`–`510`).
The caller declares who was in the quorum; nothing checks that those participants exist, are
eligible, are non-recused, or that any threshold was met.

⛔ **So §13.2 step 11 is unimplemented at every decision surface the product has**, and that
is a wider statement than the deferral row made. ⭐ It is `.8.2.4`'s and `.16`'s shape at the
governance layer: **a stored record asserting a provenance nothing derived** — and here the
asserted word is the product's entire output.

## Decision

**1. `decision_rule` is IN SCOPE, and it is a TYPED, CHARTER-SCOPED vocabulary — never a
string.** The seven §13.3 families are the vocabulary; §4.1 makes the ALLOWED subset a charter
property, so the field is validated against the tenant's charter and not against a global
list. An unknown or malformed value is a typed refusal at the create boundary, on the rule
`workflow_profile` already follows (ADR-016: the unknown id is refused where it is named).

**2. ⛔ It does NOT ship as a create field on its own, and this is the load-bearing half of
the decision.** A declared rule that nothing evaluates is exactly the defect being repaired,
moved one step earlier: today the falsehood is asserted at close, and a create field alone
would let it be asserted at creation too. **`decision_rule` lands in the same commit as the
tally that reads it — `.8.1.1`.** That leaf's acceptance already names this record, so the
constraint is enforced by the leaf rather than by this record being remembered.

**3. `expected_artifact` is IN SCOPE and IS separable — the one thing here that can ship
alone.** It is a declaration of what the thread should produce, read by the human who closes
it; no roadmap section asks anything to EVALUATE it, so it cannot become an unread field the
way a decision rule would. Owned by `.11.4.7.2.1.2.2`.

**4. 🔴 A PREREQUISITE nobody owned: the charter must be able to hold its allowed rules.**
§4.1 names *allowed decision rules and approval thresholds* as charter content, and no such
structure exists. Without it, decision 1 has nothing to validate against and would degrade
into a global enum — which §4.1 explicitly does not say. Owned by `.11.4.7.2.1.2.1`, and it
is a prerequisite of `.8.1.1` as much as this contract is.

**5. The policy-approval surface carries the same defect and is now owned** rather than noted:
`.11.4.7.2.1.2.3`.

### Is this owed before G9?

✅ **Yes, and not on a preference.** §19.6's release gate matrix gives G3 the line *authority/consent/**quorum**/
publication/correction tests | binding policy use* — quorum is named in the gate itself, and
G3 was exited as MACHINERY with binding policy use still gated. A quorum the caller declares
is not a quorum test. ⛔ This is therefore not deferrable behind a trigger the way `.4.6`'s
OpenTelemetry sink was: that one had no reader today, this one has a reachable surface
publishing a governance claim it never derived.

## Consequences

- ⭐ **The leaf shrinks to a CONTRACT and grows three children**, one of which (`.2.1`) was
  not visible at all before this measurement.
- ⛔ **`.8.1.1` is unblocked and its scope is now larger**: it builds the thread tally AND
  receives `decision_rule` in the same commit. The policy-approval half is `.2.3`, separately.
- ⚠️ **Abstention is in the vocabulary by §13.3's own words** — *unanimity … with explicit
  abstention semantics* — so `.8.1.1` may not leave it implicit.
- ⚠️ **`reasonbraid-a2a`'s `SemanticLosses.decision_rule` becomes meaningful only after this
  lands.** Today it records the loss of a field ReasonBraid does not have, which is true but
  vacuous; §9.7's *record semantic losses for … decision rule* is satisfied in form only.

## What would make this wrong

- ⛔ If §4.1's *allowed decision rules* is read as prose guidance rather than charter CONTENT,
  decision 4 is not a prerequisite and decision 1 reduces to a global enum. The reading taken
  here is §4.1's own frame — it is a list of what the charter *defines*.
- ⛔ If a tenant may legitimately close a thread under a rule its charter does not allow (an
  emergency or a waiver), then the create-boundary refusal in decision 1 is too strict and the
  rule must be recorded-with-waiver instead. §4.1 names *waiver authority and maximum duration*
  separately, so the mechanism exists; `.8.1.1` must not assume its absence.
- ⛔ If `.8.1.1` turns out to need the charter structure before it can start, the order of
  `.2.1` and `.8.1.1` inverts. `.2.1` is written to be startable now for that reason.

## Alternatives considered

1. **Add both fields now as free-form strings, matching `VerdictInput.rule`.** Rejected: it
   propagates the defect rather than repairing it, and a second uninterpreted string is harder
   to remove than the first.
2. **Ship `decision_rule` at creation now and the tally later.** Rejected on the argument in
   decision 2 — it is the defect one step earlier, and the interval during which the field is
   declared-but-unread is exactly when a caller learns to rely on it.
3. **Defer the whole thing behind a trigger, as `.4.6` deferred the sink.** Rejected: the
   comparison fails on reachability. The sink had no reader; this surface publishes
   `accepted_by_rule` to whoever asks, today.
4. **Treat `objective` as the expected artifact and delete that half.** Rejected by §26's own
   sentence, which lists both in one enumeration.
