---
answers:
  - Would integrating Jev or another System One model fix an existing ReasonBraid defect?
  - Is the caller-declared case_class a privilege escalation?
  - Where could a calibrated closed-set classifier sit without breaking the architecture?
  - Why can a classifier never sit in the enforcement path here?
  - What blocks a hosted classifier today, and what would unblock it?
  - What is the connection to the small-swarm orchestration objective?
---
# A System One classifier is an enhancement here, not a repair — and the LAN bar decides when

- **Type:** decision
- **Status:** accepted as a DIRECTION; **unscheduled**; no leaf open
- **Owner:** none — recorded, not scheduled
- **Date:** 2026-09-19
- **Raised by:** the director, 2026-09-19, naming Jev from typesafe.ai
- **Related:** `docs/decisions/2026-09-18_multi-agent-orchestration-is-the-objective-not-a-pivot.md`,
  `docs/decisions/2026-09-18_lan-completeness-precedes-internet-exposure.md`

## The question asked

Would Jev — a "System One model" returning a value from a closed set plus a
calibrated probability, at 70–500 ms against a frontier model's seconds — help
ReasonBraid reach its roadmap? And, on a follow-up: if it can be shown to fix an
existing defect, integrate it.

## The answer, and the measurement that produced it

⛔ **No existing defect was found that it repairs.** The direction is sound; the
condition attached to the greenlight is not met, and this record exists so the
next reader does not have to re-derive that.

The one candidate was **the caller-declared case class**. `POST /v1/routing/resolve`
reads `case_class` straight from the request body (`api.rs`, the
`invalid_command("the case_class is required")` site) and validates it against a
closed set of seven; `routing.rs`'s own test comment says *"The case class is a
SUBMITTED input; the resolution is a lookup."* That looked like the defect family
this repair programme keeps finding — an unbound, caller-supplied identifier — the
shape `.3.5.3` and `.3.5.5` both closed, whose rule is *derive it from the
authenticated caller, never take it from the wire*.

⭐ **It is not that shape, and the code says so in one line.** Thread creation
reads the caller's profile FIRST:

```rust
// resolves through the rule table ONLY when no profile is named (the
// human authority outranks the rule).
let profile_id = match body.workflow_profile.as_deref() {
    Some(explicit) => Some(explicit.to_owned()),
    None => match body.routing_class.as_deref() { … }
```

A caller that declares a favourable class gains **nothing it cannot already
obtain by naming the workflow profile outright**, and naming it outright is the
documented, intended path. So the declared class is a characterisation for the
audit trail and the routing statistics, not a gate — and deriving it with a model
would improve the honesty of a record, not close a hole.

⚠️ **Recorded because it was published wrongly first.** This was stated to the
director as a live defect before it was measured. It is not one. The correction is
kept here rather than deleted, per `.11.9.1.2.3`'s rule that a claim keeps its
superseded form.

## Where it would fit, if scheduled

The architecture's load-bearing sentence is `README.md`'s: *"Rust — not an LLM —
enforces identity, authorization, ordering, budgets, and publication. Model output
stays untrusted until deterministic rules accept it."* A closed-set value with a
calibrated probability is an **input to** a deterministic rule, which is the one
AI-shaped thing that sentence permits.

| Surface | Fit | Why it is safe there |
| --- | --- | --- |
| `routing_recommendations` (ADR-031, `.5.3`) | **best first use** | Already a SHADOW lane, already bounded in its own words — *"can re-order what exists, never raise authority/spend/access/side-effect scope"*. A classifier can be measured against the deterministic table here for as long as it takes, binding nothing |
| the derived case class, as EVIDENCE | good | `routing_recommendations.evidence_ref` exists for exactly this. A derived class recorded beside a declared one is a measurement of how often callers mis-characterise their own questions — which nothing currently knows |
| the swarm ACCEPTOR's pre-filter | the interesting one | see below |
| eligibility ranking (`matching`, `directory_match`) | plausible | already produces stage-1 reasons and stage-2 explanations; a score is additive |

⛔ **Never:** `authority::grant_is_live`, admission, `quota::check_in_tx`,
`policy::resolve`'s seven fail-closed steps, `ssrf::classify`, the site-authority
gate. A calibrated 0.97 is still a probability, and this system's refusals are
facts.

### The acceptor connection, which is the real one

`docs/decisions/2026-09-18_multi-agent-orchestration-is-the-objective-not-a-pivot.md`
records that small-swarm orchestration is the existing objective and that **the
binding constraint is the ACCEPTOR** — the deterministic rule that accepts or
rejects agent output. A System One model cannot be that acceptor. It can be a
cheap calibrated pre-screen in front of one, and the economics are the whole
point: at 70–500 ms and free output tokens, screening *every* contribution is
affordable where a frontier model is not. That lands directly on the named
binding constraint rather than beside it.

## What blocks it, and what would unblock it

1. 🔴 **The LAN bar, which is decisive today.**
   `docs/decisions/2026-09-18_lan-completeness-precedes-internet-exposure.md`
   records the director's instruction that the LAN must fully work first. A
   HOSTED classifier in the deliberation path makes the LAN **incomplete by
   design** — a deliberation could not complete without reaching the internet.
   ⛔ Until it is known whether Jev can be self-hosted or run on-device, this is
   post-LAN by the project's own standing rule. **The published documentation
   does not say**, and that is the question to put to typesafe.ai.
2. ⚠️ **Data locality (CLAUDE.md §13).** A hosted call also sends deliberation
   content off-volume and off-site. That needs its own decision about what may
   leave, independent of whether the LAN bar is met.
3. ⚠️ **The abstention mismatch.** Their own material states **"No
   Refusals/Abstention"** — a probabilistic answer is always returned. A
   governance platform whose value is fail-closed refusal must therefore build
   abstention itself, as a confidence threshold in Rust. ⛔ A threshold is a
   POLICY decision with a decision record and a leaf, not a constant someone
   picks — and `.11.6`'s rule applies: no rule before its population.

## What is NOT decided

- It is not rejected. The routing-recommendation lane is a genuinely good first
  use and costs the architecture nothing.
- No leaf is open and none should be until the LAN bar is met or self-hosting is
  confirmed, whichever comes first.
- Nothing here is a commitment to Jev specifically. What is recorded is the
  SHAPE — a closed-set answer with a calibrated probability, feeding a
  deterministic rule — and any model of that shape fits the same three slots and
  hits the same three blockers.
