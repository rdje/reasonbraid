---
answers:
  - Why can the policy approval's quorum not be repaired on its own?
  - Where does a policy decision's rule, electorate and result come from now?
  - What does an approval's quorum snapshot record, and who supplies it?
  - What happens to the decision's `verdict_event_id` and its request fields?
  - What stays open after this?
---
# A policy decision is its thread's counted close, and an approval copies it

- **Type:** decision
- **Status:** accepted
- **Owner:** `SIGNOFF-REPAIR.11.4.7.2.1.2.3`
- **Date:** 2026-09-22
- **Cites:** ROADMAP §4.5 (decision rules and authority proofs), §15.6 (*deliberation and
  revisions → verification/evaluation → **deterministic decision** → authority approval*),
  §15.7 step 1 (*verify decision, approvals, authority proof*), Phase 6's build line
  (*approval records with authority proofs and **quorum snapshots***);
  `docs/decisions/2026-09-22_a-counted-rule-derives-its-outcome-and-a-rule-it-cannot-count-is-refused.md`
  (the thread's counted close this reads)

## Context

The leaf opened on one field. `ApprovalInput.quorum` is caller JSON, and the only
check on it is that `participants` is a non-empty array. Its acceptance asks that
*an approval whose declared participants are not the recorded electorate is refused*.

🔴 **The measurement moved the scope.** The *recorded electorate* is
`policy_decisions.electorate`, and `record_decision` stores that exactly as the caller
sends it. The decision's `rule` is also free text (`"supermajority"`, `"unanimity"`),
and so is the decision itself: it stores whatever it is told. It never reads its
thread's result. It only checks that a `verdict` contribution exists in the thread. So
checking the approval against the decision would check one caller's claim against
another caller's claim.

🔴 **The book documents a request that could never succeed.** `policy-lifecycle.md`
sends `"electorate": {"members": [...]}`. The code counts only `participants`, so the
documented call is refused as an empty electorate.

⭐ **What makes the repair possible now is `.8.1.1`.** A thread can declare a
charter-checked rule, fix its electorate when voting opens, count ballots, and close
with a *derived* outcome. §15.6 names that step *deterministic decision*. The policy
decision is therefore not a second vote. It is the record of the thread's counted
close.

## Decision

### 1. The decision derives its rule, electorate and result from its proposal's thread

`record_decision` reads the proposal's thread projection and requires all of the
following:

- the thread is **closed**;
- the close's provenance is **`derived`**, which means the thread declared a rule and
  the server worked out the outcome;
- the outcome is a **binding acceptance**: `accepted_unanimously`,
  `accepted_with_recorded_objections` or `accepted_by_rule`.

It then stores the thread's `decision_rule`, its approval threshold, its charter
digest, its electorate and its tally as the decision's record. Nothing about the result
is taken from the request.

- A thread with **no declared rule** cannot yield a policy decision. Its close is the
  closer's claim, and a binding policy decision may not rest on a claim.
- A thread that ended `deadlocked`, `no_quorum` or any other non-acceptance cannot yield
  one either. There is nothing to approve.
- `advisory_synthesis` ends `advisory_answer_only`, which is not binding, so it is
  refused too.

### 2. The request's `rule` and `electorate` become optional assertions

They stay on the wire so existing callers get a precise refusal instead of an
unknown-field error. When present, each must equal the derived value, and a mismatch is
refused with both values named. The stored record is always the derived one. This is
the same shape as `thread.close`'s optional `outcome`.

### 3. `verdict_event_id` becomes optional

The deterministic decision is the counted close, not an adjudicator's verdict. A verdict
is one attributable judgement over a claim (ADR-029), and `.8.1.1.3` owns its free-text
rule. When one is named, it is still checked to exist in the thread and is recorded as
supporting evidence. It no longer stands in for the decision.

### 4. The approval's quorum is a copy of the decision's, and the approver supplies nothing

The Phase 6 build line asks for *approval records with authority proofs and quorum
snapshots*. The snapshot an approval rests on is the decision's derived rule, threshold,
electorate and tally. `record_approval` copies these from the stored decision.
`ApprovalInput.quorum` becomes an optional assertion on the same terms as §2: an
approver who inflates the electorate, or claims a different tally, is refused.

⭐ The threshold therefore comes from the charter by one route only: charter → thread
at creation → decision → approval. The request never supplies it.

### 5. What stays open

- **Recusal.** The leaf's *a recused participant does not count toward quorum* has no
  mechanism yet. No member is recused anywhere (`.8.1.1.6`), so the clause holds
  vacuously, and this record says that rather than claiming it.
- **Several decisions from one thread.** §15.6 says *one thread may yield multiple
  decisions*. A thread has one close, so it yields one derived result. A second
  decision against the same close would record the same result twice. This is not
  built, and the proposal's one-decision stage machine already refuses it.
- **Rows written before this.** Existing `policy_decisions` rows keep their caller-given
  `rule` and `electorate`. They are not rewritten, because a record is never silently
  changed. The book says their provenance is the caller's.

## Consequences

- The leaf splits. `.1` covers the decision and the book passage that could not
  succeed. `.2` covers the approval, and it depends on `.1`'s derived electorate.
- The policy live suite drives decisions through rule-less threads with verdicts. Those
  tests assert the defect, so they are re-derived through a counted thread (a charter,
  a `policy_proposal` thread with a declared rule, ballots, a derived close). This is
  a repair, not editing assertions to pass: each changed assertion is the old
  acceptance of an underived claim.

## What would make this wrong

- ⛔ If a policy decision may legitimately be taken outside a deliberation thread, for
  example as a direct authority act, then §1 is too strict. §15.6 places the decision
  after deliberation, and the proposal already requires a thread.
- ⛔ If `owner_decides` should not be able to carry a binding policy decision, then §1's
  acceptance set is too wide. §13.3 lists it as a supported family, and the charter
  decides whether a tenant allows it.
