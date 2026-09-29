# Deciding a thread: rules, ballots and the counted close

A thread can declare the rule it will be decided under. When it does, the server
works out the result: it counts the ballots, or it checks who closed the thread. The
person closing the thread no longer states the outcome. If they state one anyway and
it does not match, the close is refused.

This chapter covers declaring a rule, casting a ballot, and how the close works out
its outcome. ROADMAP §13.2 steps 10–11 and §13.3 define it. The binding record is
`docs/decisions/2026-09-22_a-counted-rule-derives-its-outcome-and-a-rule-it-cannot-count-is-refused.md`.

## Declaring the rule at creation

`thread.create` takes an optional `decision_rule`. Its value is one of the seven
[charter](governance-charter.md) wire names:

```json
{
  "tenant_id": "ten_…",
  "subject": "Adopt the retention policy",
  "objective": "decide whether to adopt v3",
  "workflow_profile": "policy_proposal",
  "decision_rule": "unanimity"
}
```

The create is refused, with `400 invalid_command`, when:

| Situation | Why |
| --- | --- |
| the name is not one of the seven | the field is typed, so an unknown name is not stored |
| `role_weighted` or `human_committee` | nothing can evaluate these yet. `role_weighted`'s weights have no schema, and §13.3 gives `human_committee` no bar. Accepting either would declare a rule that nothing checks |
| a counted rule (below) on a profile with no `vote` step | there would be nowhere to cast a ballot, so every close would count an empty box |
| the tenant's charter does not allow the rule | the refusal names the rule, never the charter's full set |
| the creator's grant constrains its rules and this is not among them | the issuer narrowed the charter for this subject — `decision_rule_constraints` on the admitting grant ([authority](authority.md#enrollment-boundaries-and-grants)); the refusal names the rule and says the grant refused, never the charter |
| the tenant's boundary names no registered charter | fail closed: the question cannot be answered, and no rule is ever allowed by default |

The charter check runs **after** authorization, inside the command's transaction.
A caller who may not create threads in the tenant gets an authority refusal and
learns nothing about the tenant's charter. The grant's constraint is read at the
same point, from the grant the authorization record names — the one that
actually admitted this create — so two layers apply: the charter (the tenant's
law) and the grant (the issuer's narrowing of it for one subject). A grant with
no constraint leaves the charter alone to decide.

The thread records the rule, the charter's threshold for it, and the charter's
digest. Charters are content-addressed, so a thread keeps pointing at the exact
charter it was created under, even after the tenant's charter changes.

⚠️ **`decision_rule` is optional.** Every boundary issued before charters existed
names a label rather than a registered charter. A mandatory rule would therefore
refuse every create in an existing deployment. A thread without a rule closes
exactly as before (see [the last section](#a-thread-that-declares-no-rule)).

## How each rule reaches its outcome

| Rule | How the outcome is reached |
| --- | --- |
| `majority_of_electorate` | **counted** against the charter's threshold |
| `unanimity` | **counted** |
| `consensus` | **counted**; a `reject` ballot is a blocking objection |
| `owner_decides` | **no ballot**: only the thread's creator can close it with a decision, and that decision is `accepted_by_rule` |
| `advisory_synthesis` | **no ballot**: the only decision it can end in is `advisory_answer_only` |

## The electorate and the ballot

The **electorate** is fixed when the thread first enters its `vote` step. It is the
set of participants who have accepted at that moment. A participant who joins later
is not in it. The `thread.round_advanced` event that opens the vote lists the
electorate, so the count can be checked again from the event log.

A **ballot** is a contribution of kind `ballot`:

```json
{
  "tenant_id": "ten_…",
  "content": "I can live with v3",
  "kind": "ballot",
  "ballot": { "choice": "approve" }
}
```

`choice` is `approve`, `reject` or `abstain`. A ballot is refused when:

- the current step is not `vote`;
- the thread's rule does not count ballots, or the thread declares no rule;
- the caller is not in the electorate;
- the caller has already voted. A ballot is final.

## The count

Let `n` be the size of the electorate. Count the approvals, rejections and
abstentions. Any member who has not voted is **outstanding**. There is no timer: the
close is the cut-off point, and an outstanding ballot is simply not cast.

| Rule | Outcome |
| --- | --- |
| `majority_of_electorate`, threshold `t` | The motion needs at least `t × n` approvals, rounded up. If it has them, it is `accepted_unanimously` when everyone approved, `accepted_with_recorded_objections` when anyone rejected, and `accepted_by_rule` otherwise. If it does not, it is `no_quorum` when the outstanding ballots could still carry it, and `deadlocked` when they could not |
| `unanimity` | `deadlocked` if anyone rejects **or abstains**, otherwise `no_quorum` while anyone is outstanding, otherwise `accepted_unanimously` |
| `consensus` | `deadlocked` if anyone rejects, otherwise `no_quorum` while anyone is outstanding, otherwise `deadlocked` if nobody approved, otherwise `accepted_unanimously` when everyone approved, and `accepted_by_rule` when some stood aside |

An empty electorate, or a vote that never opened, is `no_quorum`.

**What an abstention means depends on the rule.** An abstention is always a ballot
that was cast, so it is never outstanding, and it never counts as an approval.

- Under `unanimity` an abstention **withholds**. §13.4 says an initiator who
  requests unanimity *receives failure/deadlock if any applicable member withholds
  it*. The only way to leave the unanimity denominator is recusal, and recusal is
  not implemented yet.
- Under `consensus` an abstention **stands aside**. It does not block.
- Under `majority_of_electorate` an abstention stays in the denominator.

**A tie never carries**, because the threshold is always above one half.

### A worked example

Two members vote under `unanimity`: Alice approves, and Bob rejects.

```json
{ "tenant_id": "ten_…", "reason": "we agreed", "outcome": "accepted_unanimously" }
```

That close is **refused**:

```text
400 invalid_command — the close asserts `accepted_unanimously` and the counted
ballot is `deadlocked` — under this thread's decision rule the outcome is
derived; omit it to close with the count
```

Now close without an outcome:

```json
{ "tenant_id": "ten_…", "reason": "the vote is in" }
```

The thread ends `inconclusive`. Its `thread.closed` event records the result, where
it came from, and the count:

```json
{
  "outcome": "deadlocked",
  "outcome_provenance": "derived",
  "decision_rule": "unanimity",
  "tally": { "electorate": 2, "approve": 1, "reject": 1, "abstain": 0 }
}
```

## Which outcomes a closer may still state

`outcome` on `thread.close` is optional. Under a declared rule, leave it out: the
server fills in the outcome it works out.

| The close states | no rule | `owner_decides` | counted rule | `advisory_synthesis` |
| --- | --- | --- | --- | --- |
| `accepted_unanimously` | **refused** — nothing was counted | refused | must match the count | refused |
| `accepted_with_recorded_objections` | **refused** — nothing was counted | refused | must match the count | refused |
| `accepted_by_rule` (or `decided`) | accepted as stated | only the creator may state it | must match the count | refused |
| `no_quorum` | **refused** — nothing was counted | refused | must match the count | refused |
| `deadlocked` (or `inconclusive`) | accepted as stated | accepted as stated | must match the count | accepted as stated |
| `advisory_answer_only` | accepted as stated | refused | refused | worked out by the rule |
| `insufficient_evidence`, `budget_exhausted`, `expired`, `cancelled`, `human_decision_required`, `unsafe_to_continue` | accepted as stated | accepted as stated | accepted as stated | accepted as stated |

The last row covers what happened to the *process*, not to the ballot, so a
closer may always state these. That includes overriding a vote that passed: a
thread whose vote carried can still close `unsafe_to_continue`. §13.4 forbids the
other direction — *rewriting disagreement as consensus* — and a counted rule now
blocks that direction.

## Policy decisions read this close

A [policy decision](policy-lifecycle.md#decisions) is the record of its proposal
thread's counted close. It can only be recorded once the thread has closed with
a **derived** binding acceptance, and it stores the thread's rule, electorate,
tally and charter digest rather than anything the request says.

## A thread that declares no rule

A thread without a `decision_rule` closes on the closer's word, and the record
says so:

```json
{ "outcome": "accepted_by_rule", "outcome_provenance": "caller_asserted",
  "decision_rule": null, "tally": null }
```

**It cannot claim a count** (`SIGNOFF-REPAIR.8.1.1.5`). `accepted_unanimously`,
`accepted_with_recorded_objections` and `no_quorum` each describe a vote count,
and nothing was counted, so a rule-less close stating one is refused:

```text
400 invalid_command — `accepted_unanimously` names a ballot result, and this thread
    declared no decision rule, so nothing was counted — declare a counted rule at
    creation (`decision_rule`) to close on a count
```

This matches what `owner_decides` and `advisory_synthesis` already did: a thread
that declared less than those may not claim more. Every other outcome is still
accepted as stated, and marked `caller_asserted`.

⚠️ Declaring a rule is still **optional**. Every existing enrollment boundary
names a label charter, which refuses every rule, so a mandatory rule would refuse
every thread in every existing deployment. That step waits until deployments
have registered charters (tracked in [Blockers and known gaps](blockers.md)).

## What is not implemented yet

§13.3 says every rule must define thirteen things. These four are not built yet,
and each rule currently takes the narrowest honest reading:

- **Recusals** — no member is recused.
- **Role replacement** and **changed incarnations** — a ballot belongs to the
  principal ID that cast it, which under delegation is the caster, not the
  principal it acts for. §13.3 asks for the role's incarnation, and a principal
  ID is not one.
- **Amendments after voting starts** — a ballot is final.

`role_weighted` and `human_committee` cannot be declared. They stay refused until
they are designed: a schema for the charter's weights, and a §13.3 bar for the
committee. The refusal names that deferred design work,
`SIGNOFF-REPAIR.8.1.1.4.1`.

## An adjudication verdict applies the thread's rule, and cannot claim a count

A `verdict` contribution on the `adjudicate` step is one adjudicator's
attributable judgement (ADR-029). Since `SIGNOFF-REPAIR.8.1.1.3` it carries
only what it judges and what it concludes:

```json
{ "kind": "verdict",
  "verdict": { "target_digest": "sha256:…", "outcome": "accepted_by_rule" } }
```

⚠️ **`target_digest` is recorded as given** (`SIGNOFF-REPAIR.11.63`). The server
does not check that it names anything in the thread, and nothing reads it
after it is recorded. This chapter used to call the verdict a judgement *of a
claim*, and ADR-029 says it names *the proposal digest it judges*. The two
readings are not settled, and a policy decision's verdict judges a proposal,
not a claim. So the check waits for that decision (`SIGNOFF-REPAIR.11.63.1`).
A reader should treat the digest as the adjudicator's statement of what they
judged, not as a link the server verified.

- **The rule is not an input.** The rule a verdict applies is the thread's
  own, so the server writes the thread's declared `decision_rule` into the
  verdict event, or `null` for a thread that declared none. A request that
  still sends `"rule"` is refused as an unknown field. It used to be free text,
  so one adjudicator could record `"rule": "unanimity"` on an `owner_decides`
  thread.
- **Three outcomes are refused on a verdict**: `accepted_unanimously`,
  `accepted_with_recorded_objections` and `no_quorum`. Each names a count of
  ballots, and one adjudicator cannot count:

  ```text
  400 `accepted_unanimously` names a count of ballots, and a verdict is one
      adjudicator's judgement (ADR-029) — a count is cast as ballots under a
      counted rule
  ```

  Under a counted rule the thread's close works that outcome out from the
  ballots ([which outcomes a closer may still state](#which-outcomes-a-closer-may-still-state)).
- A `verdict`-kind contribution must carry its `verdict`. One without it used to
  be accepted, recording a verdict that judged nothing.

A verdict out of turn is still refused by the step check first, with that
check's own message.
