answers: is it dangerous to keep a copy of a stored value; when do two copies of a value drift; I found 43 places that duplicate a value, how many are defects; how do I scope a search for a precision or rounding bug; why did the same duplication turn out to be a bug in one place and fine in another; what should I measure before auditing duplicated state

# A second copy is only a risk if it has a second source

- **Type:** `knowledge`
- **Date:** `2026-09-21`
- **Owner / source:** leaf `SIGNOFF-REPAIR.11.31.1`, which inherited a search
  scoped on the wrong axis and found the right one while adjudicating it.

## The question

A real defect is found: a value was held in two places and the two disagreed,
because one of them had been through a lossy store. The obvious follow-up is to
find every other place that holds a second copy — and there are forty of them.

How many are defects?

## The answer

> **Ask where each copy CAME FROM, not how many copies there are.** Two copies
> that were produced independently can disagree. Two copies where one was *read
> from the other* cannot.

A duplicate is a risk when it has an independent **source**. If the second copy
was derived from the first — read back, echoed, projected — then whatever the
store did to the value, it did before the copy existed, and there is nothing for
the copies to differ about.

So the search is two-dimensional, and only one cell is dangerous:

| | the copy stays local | the copy is handed out |
| --- | --- | --- |
| derived from the store | safe | safe |
| **produced independently** | safe | ⚠️ **the defect** |

⭐ The cell that is empty is worth as much as the cell that is full. *No site both
invents a value and hands it out* is a strong, checkable statement about a
corpus, and it is invisible to a one-dimensional count.

## The measured instance

A timestamp truncated by the database to microseconds had been kept in memory at
full precision and handed to a verifier — a real defect. A census of the same
shape returned **43** further sites.

| origin of the bound value | stays local | handed out |
| --- | --- | --- |
| read back from the store | 2 | 2 |
| normalized to the store's precision first | 0 | 4 |
| supplied by the caller (followed by hand) | 19 | 5 |
| other (second-granular input, echoed request field) | 1 | 3 |
| **produced by the local clock** | **7** | **0** |

Every site that handed a value out had obtained it from the store, normalized it
first, or was handing back something second-granular that could not carry the
lost precision. **Zero defects in 43 candidates**, and the reason is structural
rather than lucky: the codebase had a convention of sampling the instant from
the store.

## ⛔ The trap: the wrong axis makes a clean corpus look alarming

Scoped as *how many places hold a copy*, the answer was 43 and every one needed
reading. Scoped as *how many places hold an independently produced copy that
escapes*, the answer was 0 and the reading was eight sites.

> A search scoped on the wrong axis is not merely slow. It produces a number
> that sounds like a finding — "43 sites with the same shape as the bug" — and
> that number will be quoted.

## ⭐ The by-product: an unwritten convention is the real gap

If most of the population is safe because of a convention, and the convention is
not written where the next author will meet it, then the safety is an accident
waiting to be un-made. That is usually the repair a clean audit should ship:
not a gate, and not a code change, but the sentence that says **why** the
existing code is right.

Ask of a convention: *if someone did the other thing tomorrow, what would tell
them?* If the answer is "nothing", write it down at the place they will be
standing.

## Restated outside software

A recipe copied from the tin and a recipe remembered from a friend will
eventually disagree with the tin; a recipe photographed from the tin will not,
however many times it is passed on. Counting how many copies of the recipe are
in circulation tells you nothing. Asking which of them were *written from
memory* tells you everything, and there are usually far fewer of those.

## Related

- [[a-delta-is-a-second-measurement]] — the same caution about derived versus
  independently obtained numbers, one layer up.
- [[a-census-is-as-wide-as-its-key]] — a population counted on the wrong key
  answers a different question from the one asked.
- [[a-prefix-closed-rule-costs-terminals-not-members]] — the sibling pricing
  error: counting members when the answer is in terminals.
- [[trust-comes-from-the-check-not-the-shape]] — why "it looks like the bug we
  fixed" is not evidence that it is.
