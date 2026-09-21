answers: I re-checked my finding and it held, why was it still wrong; what exactly does re-derive by a different route mean; my number recomputed to the same value, is that verification; how do I verify a percentage or a total; why did a second pass miss what the first pass missed; what part of a measurement is the easiest to leave un-verified

# A re-derivation must re-derive the POPULATION, not just the number

- **Type:** `knowledge`
- **Date:** `2026-09-21`
- **Owner / source:** leaf `SIGNOFF-REPAIR.11.2.1.3.2.1`, which found the error
  in the verification pass that had been written one commit earlier specifically
  to catch errors of this kind — and which had itself reported this very figure
  as holding.

## The question

You published *221,496 KiB across thirteen directories*. The director asks
whether your findings still hold. You go back, you re-measure — with a different
tool, honestly, at a later commit — and you get 221,496 KiB. You record:
**re-measured, unchanged.**

Is the finding verified?

## The answer

> **No. You verified the measurement and left the population untouched.** Every
> measurement is a function applied to a set, and *which set* is the input you
> are most likely to carry forward unexamined, because it does not look like a
> number. It looks like the subject of the sentence.

The thirteen directories were real and their total was exactly right. There were
**twenty-six** of them, and the six largest omissions held 129,504 KiB. The
published figure was 63.1% of the population. The second pass could not have
caught that, because it re-measured the same thirteen names.

*Measured instance:* the same batch also published *98.5% of the total*. That
was not an arithmetic error either — it is exactly 98.5% of the thirteen-family
subtotal. Against the real population it is 62.2%. **A ratio inherits its
denominator's blind spot silently, and unlike a total it gives no hint that
anything is missing**, because a percentage always looks complete.

## ⭐ How to tell which one you actually checked

Write the finding as `f(P)` and say what each half was.

- If your second route computed `f` differently over the same `P` — a different
  tool, a different command, a later commit — you verified `f`.
- You verified `P` only if the second route **derived membership independently**:
  from the code that creates the members, from the schema, from a traversal of
  the real container.

Re-running a count with `du` instead of Python is a different `f`. Re-running it
over a list you typed is the same `P`. The first is cheap and feels like
diligence; only the second can find an absent member, and an absent member is
the error a total cannot show you.

## ⛔ The tell: where did the list come from?

In the measured instance the answer was *from what I had noticed*. That is the
signature, and it is worth being blunt about it, because the same batch had
already condemned it once:

> A population you can recite is a population nothing derived.

The repair is `docs/CLAIM_VERIFICATION.md` §2 — derive membership from the
producer. Here the producer is the code that creates each directory, and the
six missing families were all built the same way (a base joined with `"target"`
in one statement and the family name in another), so **one shape was invisible
to a human reading for names and obvious to a parser reading for joins**. The
list was not short by accident; it was short by a rule nobody had stated.

## ⚠️ The direction this errs in

Stated because a scoping defect always errs in one direction: a population
derived from what somebody noticed is **too small**, never too large. So every
total built on it understates, every ratio built on it overstates the share of
what it did see, and every *nothing does X* claim over it is weaker than it
sounds. None of those failures announces itself. A too-large population, by
contrast, produces members you can inspect and reject.

## Restated outside software

An auditor recounts the cash in thirteen tills and confirms the figure to the
penny. The shop has twenty-six. Nothing in the recount can reveal the other
thirteen, and the more carefully it is performed the more confident everyone
becomes. The question that finds them is not *did I count right* but **how did I
decide what to count** — and the only good answer is that something other than
memory decided it.

## Related

- [[a-sample-is-not-a-traversal]] — the near neighbour, and the difference is
  worth holding: there the population is known and only part of it is checked;
  here the population itself is wrong, so even a full traversal of it is.
- [[an-absence-claim-is-a-census-over-the-corpus]] — the same defect in its most
  dangerous form: *nothing does X* over a population nothing derived.
- [[a-restated-number-needs-a-producer]] — the other half of this leaf's finding,
  and why the corrected figure ships with a command instead of a value.
- [[a-census-is-an-instrument-not-a-table]] — what the repair has to become
  before the next reader can check it rather than quote it.
- [[a-second-copy-is-only-a-risk-if-it-has-a-second-source]] — why a hand-kept
  membership list is a second copy of the producer, and drifts as one.
