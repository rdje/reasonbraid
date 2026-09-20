answers: should §9.8 gain the reason codes this build emits beyond it; what is undeclared_region and why was it missing from the count; is not_found a duplicate of scope_hidden; do not_found and scope_hidden leak existence; what is the difference between quota_exceeded, rate_limited and budget_unavailable; why does the registry contain codes nothing emits; should every unconfigured surface get its own reason code; is commit_outcome_unconfirmed the same thing as provider_outcome_unknown

# The nine unregistered reason codes, recommended one by one (`SIGNOFF-REPAIR.11.7.1`)

🔴 **CORRECTED 2026-09-20 BY `SIGNOFF-REPAIR.13.4.6.1`: THE POPULATION IS TEN.**
The tenth is `undeclared_region`, and it was missing for a reason worth more
than the code itself — `census_reason_codes.py` matched only `code: "…"`, a Rust
struct field, while this one is emitted through `json!({"code": "…"})`. It was
therefore invisible to the census, absent from `docs/book/src/errors.md`, and
the `REASON-CODE-DOC` gate reported *not documented in the book: 0* over a set
one short. The matcher now reads both forms and the book documents it.
⚠️ **The title's number is deliberately not chased**, for the reason `.11.7.1`
gives: it names the population at writing, and every move is a recorded
correction rather than a silent edit. Re-derive with
`python3 -B scripts/census_reason_codes.py`, never from this document.

- Date: 2026-09-20 · Leaf: `SIGNOFF-REPAIR.11.7.1` · Decision record (a RECOMMENDATION; it changes no code and no frozen document)

## Context

`ROADMAP.md` §9.8 publishes a stable error vocabulary. This build emits nine
codes that are not in it, each documented in `docs/book/src/errors.md` and
marked `ext`. `.11.7` measured that and deliberately did not spend the evidence:
§9.8 is frozen at v0.4.1 to factual errata, security corrections and Phase 0
blockers, and a v0.5.0 change "must cite measurements … produced by working
code". This record is that citation.

⛔ **Nothing here edits `ROADMAP.md`.** The freeze holds until the director lifts
it or decides otherwise. What follows is the per-code recommendation the leaf's
acceptance asks for.

## The measurements

Every number is a command, and the population is derived rather than quoted —
it has moved twice already, both times by this tree's own hand.

| question | command | answer |
| --- | --- | --- |
| the registry | `python3 -B scripts/census_reason_codes.py` | **20** codes |
| what the server emits | same | **18** |
| emitted, unregistered | same | **9** |
| **registered, never emitted** | same | **11** |
| `not_found` producers | `git grep -c "ControlApiError::not_found(" -- crates/reasonbraid-server/src` | **23** |
| `scope_hidden` producers | `git grep -c "ControlApiError::scope_hidden()" -- crates/reasonbraid-server/src` | **3** |

⭐ **The most surprising number is 11, and it reframes the whole question.** The
registry is not a subset of what this build uses — over half of it has no
producer at all, including `rate_limited`, `budget_unavailable` and
`provider_outcome_unknown`, each of which a code below is arguably a rename of.
⇒ *Should §9.8 gain these nine* is the wrong shape of question on its own. The
registry and the emitted set differ in **both** directions, and three of the
nine are better read as *the product implemented a registered concept under
another name* than as *the product invented a concept*.

## Decision — the recommendation per code

### A. Add as published (4)

These name concepts §9.8 has no word for, they are deployment- or
workflow-shaped rather than security-shaped, and nothing in the registry is a
candidate rename.

| Code | HTTP | Why it stands alone |
| --- | --- | --- |
| `quota_unconfigured` | 503 | *the scope declares no bound and the surface fails closed*. §9.8 has no vocabulary for a **deployment gap** at all; every registered 5xx is a runtime fault. |
| `classification_unqualified` | 409 | the thread's classification needs a qualified evaluator and the deployment registers none. No analogue. |
| `storm_control` | 429 | a fan-out or invitation-rate **breaker** tripped. See §C on why this is not `rate_limited`. |
| `idempotency_conflict` | 409 | a bootstrap **request id** bound to a different request, against `idempotency_mismatch`'s replayed **command payload**. ⚠️ Two idempotency codes is a genuine cost; the distinction is real (two different keys at two different layers) and should be written into §9.8 rather than left to the book. |

### B. Reconcile before adding — the registry already has the concept (3)

⛔ These should **not** simply be appended. Appending them would publish two
words for one idea and leave the registered one with no producer for ever.

**`not_found` (404) against the registered `scope_hidden` (404).**
🔴 **The product voted with its feet and the vote was 23 to 3.** Both mean *no
such thing, within your scope*; both deliberately collapse **foreign** into
**absent**. ⇒ Recommend: **one 404 word.** Either `not_found` enters §9.8 and
the three `scope_hidden` sites move to it, or the twenty-three move the other
way. The asymmetry is the evidence for which.

**`unknown_node` (404) against the same pair.**
Its book entry justifies it as *distinct from `not_found` so a channel client
can tell "re-enrol" from "wrong id"*. ⇒ Recommend: **add, but re-justify it on
the honest ground.** The stated ground does not survive its own code —
`node_channel.rs` answers `unknown_node` for a node in ANOTHER tenant exactly as
for one that does not exist, so the client cannot in fact tell those apart. What
it genuinely buys is a **channel-specific** 404 on a surface whose clients are
nodes rather than operators.

**`commit_outcome_unconfirmed` (500) against the registered, unemitted
`provider_outcome_unknown` (500).**
Both are *the outcome is genuinely unknown — do not treat it as a failure*, one
about this server's own transaction and one about a provider call. ⇒ Recommend:
**decide whether §9.8 wants one such code or two**, and if two, say in §9.8 what
distinguishes them. A client's handling is identical either way: inspect before
retrying.

### B2. The tenth code, added by the correction above

**`undeclared_region` (400).** Both regions must be **declared** before they can
be paired — a DOMAIN refusal from the site registry, raised after the authority
check has passed. ⇒ Recommend: **add as published, in §A's group.** It names a
condition §9.8 has no word for; `invalid_command` is the nearest registered code
and would collapse *your request is malformed* into *the registry cannot express
this pairing yet*, which are different facts and lead to different client
behaviour.

⚠️ **The interesting part is not the verdict, it is what the omission says about
the method.** This record's own population came from an instrument, and the
instrument had a blind spot that made the set look complete. ⛔ A recommendation
is only ever as complete as the census under it, and *the census printed a zero*
is not evidence of coverage — it is evidence the census found nothing, which is
a different claim.

### C. Generalise rather than add (1) — and it is the recommendation with the longest reach

**`publication_repository_unconfigured` (503).**
⛔ Recommend **against** adding it as published. It is the **second** instance of
a family — `quota_unconfigured` is the first — and the family is open: every
future surface that requires deployment configuration will want its own code by
the same argument. A registry that grows one code per surface is a list, not a
vocabulary.

⇒ Recommend: **one code for *this deployment has not configured what this verb
needs*, with the missing thing in the structured details.** §A therefore
recommends `quota_unconfigured` as published only if this generalisation is
declined; if it is accepted, both collapse into the general code and §A's count
drops to three.

### The three-way distinction the leaf asked for, stated

`quota_exceeded` is in neither list above, because the leaf asked for this
specific ruling and it is the answer:

| Concept | Question it answers | Registry status |
| --- | --- | --- |
| **quota** (`quota_exceeded`, 429) | a **declared allowance** over a sliding window is exhausted | emitted, unregistered |
| **rate** (`rate_limited`, 429) | the caller is going too **fast** right now | registered, **no producer** |
| **breaker** (`storm_control`, 429) | a **protective** circuit tripped, independent of any declared allowance | emitted, unregistered |
| **budget** (`budget_unavailable`, 429/503) | the **spend** ceiling cannot fund this work | registered, **no producer** |

⇒ **Recommend keeping all four and writing that table into §9.8.** They are four
different facts and a client acts differently on each: wait for the window
(quota), back off (rate), stop and escalate (breaker), obtain funding (budget).
⛔ Collapsing them was the alternative and it is refused: a caller told only
`rate_limited` cannot tell *slow down* from *your allowance is spent for the
month*, and only one of those is fixed by waiting.

## What was refuted

- **"`not_found` beside `scope_hidden` is an existence oracle."** The leaf
  flagged the overlap as possibly a §9.8 *breach*. It is not: every `not_found`
  site sampled binds the caller's tenant and answers identically for foreign and
  absent — `api.rs` says so in its own words, *"Missing and foreign are ONE
  answer, so a caller learns nothing about another tenant's nodes"*. The pair is
  a **vocabulary duplication**, not a leak, and the recommendation is
  correspondingly a tidy-up rather than a security correction.
- **"`unknown_node` lets a channel client tell re-enrol from wrong id."** Its own
  book entry claims this and its own code refutes it: a foreign node and an
  absent one give the same answer. The code should stay; the justification must
  change.
- **"The question is which of the nine to add."** Half the registry has no
  producer, so the two sets differ in both directions and three of the nine are
  candidate renames rather than candidate additions.

## Consequences

- ⛔ No edit to `ROADMAP.md` §9.8. This is evidence for a v0.5.0 decision, and
  the director's is the decision.
- The `.11.7` finding is now spendable: a per-code recommendation exists, with
  the measurement behind each — **ten codes**, after `.13.4.6.1`'s correction.
- ⚠️ **Three follow-on questions this record deliberately does not answer**,
  because each is a change to shipped behaviour rather than to a published list:
  whether the twenty-three `not_found` sites or the three `scope_hidden` ones
  move; whether `rate_limited`, `budget_unavailable` and
  `provider_outcome_unknown` should acquire producers or be retired; and whether
  the two `*_unconfigured` codes collapse. Each needs its own leaf, and none may
  precede the freeze decision.
- ⭐ The retirement of `locator_digest_conflict` (`.11.14.3.2`) stands as the
  precedent this record leans on: a code can be a §9.8 **breach** rather than an
  omission, so a recommendation that only ever appends cannot express what the
  evidence actually says.
