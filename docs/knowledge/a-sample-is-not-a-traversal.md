answers: I checked several cases and they all agreed, can I say all of them do; how should I word a finding when I only sampled; when is a universal claim worth the cost of verifying it; my conclusion is right but my evidence was generalised, does it matter; how do I stop an audit's prose from overclaiming

# A sample is not a traversal

- **Type:** `knowledge`
- **Date:** `2026-09-21`
- **Owner / source:** leaf `SIGNOFF-REPAIR.11.31.1.1`, which found the sentence
  in its own parent leaf when the director asked whether the findings reflected
  reality rather than whether the numbers recomputed.

## The question

An audit enumerates a population — 24 call sites, say. You follow eight of them,
they all say the same thing, the pattern is obvious, and you write:

> *"All 24 were followed to their call sites; every one passes X."*

The conclusion is right. Is the sentence?

## The answer

> **No, and the tell is that you can name the number you checked.** If the
> sentence says *every one of N* and you can only produce evidence for *k*, the
> sentence is a different claim from the work.

This matters even when the conclusion survives, because the sentence is what the
next reader inherits. They will not re-run the audit; they will quote it. A
universal claim is a promise that someone looked, and a promise nobody kept
decays silently into a fact.

*Measured instance:* eight functions were followed, the result was written as
all twenty-four sites, and enumerating the twenty-four showed several callers
doing the opposite of what the sentence asserted. The audit's **conclusion** was
still correct — but for a reason the sentence had not given.

## ⭐ The repair is usually to need the claim less

Before verifying a universal, ask whether the argument requires it.

In the measured case it did not. Provenance only mattered for sites whose value
**escaped**; nineteen of the twenty-four were contained, so their callers were
irrelevant to the conclusion. The honest sentence was shorter, weaker-sounding,
easier to check and **completely sufficient**:

> *Nineteen are contained, so their provenance does not bear on this. The five
> that escape were each followed individually, and here is what each one does.*

> **A claim scoped to what the argument needs is almost always a claim you can
> actually verify.** Overclaiming is often a symptom of not having noticed which
> part of the population the conclusion rests on.

## ⛔ When you do need the universal, make it data

If the argument genuinely requires a statement about every member, prose will
not hold it. Carry the judgement as a table keyed to each member, and guard it
so that it refuses when:

- a member exists that nothing judged;
- a judgement exists for a member that is gone;
- **a member's inputs have changed since it was judged** — the reason was
  reached about facts that no longer hold.

The third is the one usually missing, and it is the one that makes a stale
judgement look like a current one.

## Restated outside software

"I checked every window in the building" is a different statement from "I
checked the eight I could reach from the ground, and they were all shut." The
second is less impressive, entirely verifiable, and — if the ones you could not
reach are on a sealed floor — exactly as reassuring. The trouble starts when the
first sentence is the one that goes in the report.

## Related

- [[an-absence-claim-is-a-census-over-the-corpus]] — the sibling: *nothing does
  X* is a quantifier too, and needs the same treatment.
- [[a-second-copy-is-only-a-risk-if-it-has-a-second-source]] — the audit this
  error was found inside; its conclusion held, its evidence did not.
- [[an-adjudication-is-keyed-to-the-words-it-judged]] — how to hold a hand
  judgement so it refuses when what it judged has moved.
- [[a-fire-count-is-not-a-fire-rate]] — the previous round's finding, and the
  same underlying habit: a sentence that sounds like the measurement but is not.
