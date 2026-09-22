---
answers:
  - Which of the four ballot-result close outcomes does the server derive, and which may a closer still assert?
  - What does a ballot record, who may cast one, and when is the electorate fixed?
  - What does an abstention mean under each counted rule?
  - Why are `role_weighted` and `human_committee` refused at thread creation?
  - How does each shipped rule answer ROADMAP §13.3's thirteen per-rule obligations?
  - What happens to a thread that declares no rule?
---
# A counted rule derives its outcome, and a rule it cannot count is refused

- **Type:** decision
- **Status:** accepted
- **Owner:** `SIGNOFF-REPAIR.8.1.1`
- **Date:** 2026-09-22
- **Cites:** ROADMAP §13.2 (steps 10–11), §13.3 (decision rules and their thirteen
  per-rule obligations), §13.4 (the twelve terminals and *never rewrites disagreement as
  consensus*);
  `docs/decisions/2026-09-22_a-decision-rule-is-a-charter-scoped-vocabulary-and-never-ships-without-its-tally.md`
  (the contract this builds against);
  `docs/decisions/2026-09-22_a-charter-is-content-addressed-and-registering-one-is-a-site-act.md`
  (the charter it validates against)

## Context

`CloseOutcome` has fourteen variants, and four are ballot results —
`accepted_unanimously`, `accepted_with_recorded_objections`, `accepted_by_rule`,
`no_quorum`. Before this record the closer asserted all four, and nothing counted
anything. `.11.4.7.2.1.2` decided the fix's contract: `decision_rule` is a typed,
charter-scoped vocabulary, and it lands in the same commit as the count that reads it.

🔴 **That contract, and `.8.1.1`, both under-quoted §13.3.** Its second paragraph reads:
*Every rule defines electorate creation, quorum, denominator, abstentions, recusals,
timeouts, unreachable members, role replacement, changed incarnations, veto scope,
amendments after voting starts, tie handling, and terminal outcomes.* That is thirteen
obligations for each rule. Every one is answered below, either decided or owned by a
named child. None is left out.

## Decision

### 1. A ballot is a contribution, and the electorate is fixed when voting opens

- A ballot is a `ballot`-kind contribution with a `choice` — `approve`, `reject` or
  `abstain`. It needs the `thread_contribute` grant like every other content verb, so it
  is not a new authority. It is accepted only while the current step is `vote`, only from
  an electorate member, and only once per member.
- **The electorate is the thread's `accepted` participants at the moment the thread
  enters its `vote` step.** That happens on `thread.advance_round`, or at creation when
  `vote` is the profile's first step. A participant who joins later is not in it (§13.2
  step 10: *snapshot electorate*).
- A thread that declares a counted rule must run a profile with a `vote` step. Otherwise
  every close would count an empty ballot box, so creation refuses the thread instead.

### 2. Three families are counted, two are derived without a ballot, two are refused

| family | how its outcome is reached |
| --- | --- |
| `majority_of_electorate` | **counted** against the charter's threshold |
| `unanimity` | **counted** |
| `consensus` | **counted**; a `reject` is the blocking objection |
| `owner_decides` | **derived from who closes**: only the thread's creator decides |
| `advisory_synthesis` | **derived from the rule**: its only decision terminal is `advisory_answer_only` |
| `role_weighted` | ⛔ **refused at creation** — its weights have no schema (`.11.4.7.2.1.2.1` decision 4); owned by `.8.1.1.4` |
| `human_committee` | ⛔ **refused at creation** — §13.3 names it and gives it no bar; owned by `.8.1.1.4` |

⛔ Accepting either refused family and then closing on the closer's word would break
`.11.4.7.2.1.2` decision 2 in the exact case it was written for: *a declared rule that
nothing evaluates.*

### 3. The count

Let `n` be the electorate's size, and let `a`, `r` and `s` be its approve, reject and
abstain ballots. Then `o = n − a − r − s` is the outstanding count. An empty electorate
(`n = 0`, or a vote that never opened) is always `no_quorum`.

| family | outcome |
| --- | --- |
| `majority_of_electorate`, threshold `t` | `need` is the smallest whole number with `need ≥ t·n`. If `a ≥ need`, the thread is accepted: `accepted_unanimously` when `a = n`, `accepted_with_recorded_objections` when `r > 0`, and `accepted_by_rule` otherwise. If `a < need`: `no_quorum` when `a + o ≥ need`, because the outstanding ballots could still carry it; otherwise `deadlocked` |
| `unanimity` | `deadlocked` when `r > 0` or `s > 0`; otherwise `no_quorum` when `o > 0`; otherwise `accepted_unanimously` |
| `consensus` | `deadlocked` when `r > 0`; otherwise `no_quorum` when `o > 0`; otherwise `deadlocked` when `a = 0`, since nobody supports it; otherwise `accepted_unanimously` when `s = 0`, and `accepted_by_rule` when members stood aside |

⚠️ `need` is computed as `ceil(t·n − 1e-9)`. Thresholds are human-entered decimals, and
without the epsilon `0.56 × 25` comes out as `14.000000000000002` in `f64`, which needs
15 approvals where 14 are exactly 56%. Measured, not assumed: 13 of the 10,000
(threshold, electorate) pairs over thresholds `0.51`–`1.00` and electorates 1–200 land
one ulp above a whole number. (The first draft cited `0.51 × 100`, which is exactly
`51.0` — a mutation that removed the epsilon survived it, and that is how the wrong
witness was found.) The epsilon is many orders of magnitude below `1/n` for any real
electorate.

### 4. Abstention is in the vocabulary, and its meaning depends on the family

An abstention is a CAST ballot: it is never outstanding and never an approval.

- **Under `unanimity` it withholds, so the result is `deadlocked`.** §13.4 decides this:
  *An initiator who requests unanimity receives failure/deadlock if any applicable member
  withholds it.* §13.3 takes a member out of the unanimity denominator only by *recusal*.
  An abstainer has not recused, so they are still an applicable member, and they did not
  approve.
- **Under `consensus` it stands aside.** It is not a blocking objection, so it does not
  block. It does move an all-approve `accepted_unanimously` down to `accepted_by_rule`.
- **Under `majority_of_electorate` it stays in the denominator** and does not count toward
  `need`.

### 5. The close classification — the four ballot words, classified

A close may name no outcome. In that case the server supplies the derived one, and this is
the normal path.

| asserted outcome | no rule declared | `owner_decides` | counted family | `advisory_synthesis` |
| --- | --- | --- | --- | --- |
| `accepted_unanimously` | caller-asserted (`.8.1.1.5`) | refused — nothing counted | **derived** | refused |
| `accepted_with_recorded_objections` | caller-asserted (`.8.1.1.5`) | refused — nothing counted | **derived** | refused |
| `accepted_by_rule` (and `decided`) | caller-asserted | **derived** — the closer is the creator, else refused | **derived** | refused |
| `no_quorum` | caller-asserted (`.8.1.1.5`) | refused — nothing counted | **derived** | refused |
| `deadlocked` (and `inconclusive`) | caller-asserted | caller-asserted | **derived** | caller-asserted |
| `advisory_answer_only` | caller-asserted | refused — the rule binds | refused — the rule binds | **derived** |
| the six process terminals | caller-asserted | caller-asserted | caller-asserted | caller-asserted |

- **Derived** means the asserted word must equal the server's result, and a disagreement is
  refused with both words named. The six process terminals are `insufficient_evidence`,
  `budget_exhausted`, `expired`, `cancelled`, `human_decision_required` and
  `unsafe_to_continue`. They state facts about the process rather than about the ballot,
  so they stay caller-asserted under every rule.
- ⚠️ A process terminal can therefore override a count that carried: a thread whose vote
  passed may still close `unsafe_to_continue`. That is the safe direction. §13.4 forbids
  the other one — *rewriting disagreement as consensus* — and that direction is exactly
  what derivation closes.
- ⭐ **The close event records `decision_rule`, the `tally` and `outcome_provenance`**
  (`derived` or `caller_asserted`). Before this record the provenance was implicit, and
  implicit provenance is the defect itself (`.8.2.4`'s shape at the governance layer).

### 6. A thread that declares no rule keeps today's behaviour, and now says so

`decision_rule` stays optional. Every shipped enrollment boundary names a label charter
that fails closed, so a mandatory, charter-validated rule would refuse every thread
creation in every existing deployment. A rule-less thread therefore closes exactly as it
did before, and the twelve-terminal walk keeps every assertion it has. Its close event now
carries `decision_rule: null` and `outcome_provenance: "caller_asserted"`. ⛔ This makes
the defect visible for those threads. It does not end it, and `.8.1.1.5` owns the path to
ending it.

### 7. §13.3's thirteen obligations, per shipped rule

| obligation | answer |
| --- | --- |
| electorate creation | the `accepted` participants when the `vote` step is entered (§1) |
| quorum | no separate parameter. `majority_of_electorate` needs its share of the WHOLE electorate; `unanimity` and `consensus` need every member. A shortfall the outstanding ballots could still change is `no_quorum` |
| denominator | the whole electorate snapshot, never the ballots cast |
| abstentions | §4 |
| recusals | ⛔ **not implemented**: no member is recused. Owned by `.8.1.1.6` |
| timeouts | there is no timer. The close is the cutoff, and outstanding ballots count as not cast |
| unreachable members | the same as outstanding. There is no separate state |
| role replacement | ⛔ **not implemented**: a ballot belongs to the principal wire id that cast it. Owned by `.8.1.1.6` |
| changed incarnations | ⛔ **not implemented**. §13.3: *a role's vote remains attributable to its incarnation*, and a wire id is not an incarnation. Owned by `.8.1.1.6` |
| veto scope | under `consensus`, any member's `reject` is a block. `unanimity` needs every member's approval. `majority_of_electorate` has no veto |
| amendments after voting starts | ⛔ **not modelled**: a ballot is final, and a second one is refused. Owned by `.8.1.1.6` |
| tie handling | a threshold lies in `(0.5, 1.0]`, so a tie never carries. It lands at `no_quorum` or `deadlocked` by §3 |
| terminal outcomes | §3 and §5 |

## Consequences

- `.8.1.1.1` lands `crate::decisions` — the vocabulary, the count and the classification —
  as pure functions nothing calls yet. `.8.1.1.2` wires them in one commit together with
  `decision_rule`.
- The charter check at creation runs INSIDE the command transaction, after authorization.
  Before authorization, it would tell a caller who may not create in a tenant whether that
  tenant's charter allows a rule.
- The adjudication verdict's free-text `rule` is a separate surface and is not settled
  here (`.8.1.1.3`).

## What would make this wrong

- ⛔ If *withholds* in §13.4 means only a `reject`, then abstention under `unanimity`
  should be a stand-aside and not a deadlock. The reading taken here rests on §13.3
  naming recusal as the only exit from the unanimity denominator.
- ⛔ If `no_quorum` must mean a participation floor distinct from the threshold, then
  `majority_of_electorate` needs a second charter number, and `.11.4.7.2.1.2.1` decided
  that thresholds parameterize families rather than multiply them.
- ⛔ If a deployment must close a thread under a rule its charter no longer allows, §4.1's
  *waiver authority* is owed. `.11.4.7.2.1.2` decided waivers are out of scope, and this
  record does not read that silence as *waivers are impossible*.

## Alternatives considered

1. **A new `thread.vote` operation.** Rejected: a new operation would need a new grant
   action, and ADR-016 says a profile's steps compose EXISTING verbs. The verdict and the
   assessment are the precedents for a step-bound contribution kind.
2. **Letting a member change their ballot, where the latest ballot counts.** Rejected for
   now: it is the *amendments after voting starts* obligation, and a final ballot is the
   narrowest answer that can be made honest without a mechanism.
3. **Refusing the three count words on rule-less threads today.** Rejected: that edits
   the twelve-terminal walk's assertions, which `.8.1.1`'s acceptance forbids, and it
   cannot be made mandatory while charters fail closed (§6).
