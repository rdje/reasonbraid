answers: my new census found a huge population — is it real; why does one file account for most of my findings; my instrument reports a whole directory as broken, what should I check first; how do I sanity-check a census before publishing its number; the majority class of my measurement looks wrong, what now; how do I tell a corpus defect from a parser defect; my grep found N hits, is N the answer

# An instrument's first population describes its parser

- **Type:** `knowledge`
- **Date:** `2026-09-20`
- **Owner / source:** leaf `SIGNOFF-REPAIR.11.24.1.6.1`, promoted on the third
  instance in two commits — all three from one instrument, which is why the
  pattern was visible at all.

## The question

You have written a census. It runs, and it returns a large population with a
clear majority class. Before that number goes into a leaf, a changelog or a
decision record — what do you check?

## The answer

> **Look at the majority class first, and ask whether it is a property of the
> corpus or a property of the thing you just wrote.**
> A first run's dominant finding is far more often the parser's shape than the
> repository's.

This is not a general caution about bugs. It is a specific, repeating failure:
the instrument is new, the corpus is old, and the corpus has been under
continuous human attention while the parser has had none. When they disagree in
bulk, the young thing is usually wrong.

## Three instances, one instrument, two commits

`scripts/census_relative_leaf_refs.py` counts relative leaf references such as
`` `.2.3` `` in `docs/tasks/`. Its first three runs each produced a large,
confident, wrong majority class.

| Run | What it reported | What it was |
| --- | --- | --- |
| 1 | 267 `foreign` references, concentrated in one directory | `git ls-files docs/tasks/*.md` matches **recursively**; it had swept 24 artifact documents that define no leaves, so nothing in them could resolve |
| 2 | `PHASE-1.md` is 114 `foreign` + 80 `dangling` out of 252 | that tree writes references in a **second dialect**, repeating its own phase number — 194 correct references the parser knew nothing about |
| 3 | 76 `dangling`, 62 of them in one tree, "a lane with 195 children has no node" | 44 of them are a **third dialect**, relative to an ancestor of the leaf they sit in, and **not one** referred to a lane |

Each time the finding was stated in the corpus's voice — *this tree is broken* —
and each time the subject was the parser.

## The check, in one question

> **"If my instrument were right, would this corpus ever have worked?"**

A corpus in daily use does not contain 194 broken cross-references in one file,
or a whole directory of unresolvable ones. People would have tripped over them.
When a census says otherwise, the cheap move is to open six rows of the majority
class and read them in context — not to sample randomly, but to read the class
that is carrying the number.

⭐ **Read them in their own context, not in the instrument's.** Instance 3 was
caught only by asking *what leaf is this sentence inside?* — a question the
census was not asking, because it resolved every reference against the tree.
The rows looked identical either way until the enclosing heading was fetched.

## What this is not

⛔ **Not a reason to distrust a small or a negative population.** The same
instrument's `foreign` and `internally-ambiguous` counts were believable
immediately, because they were neither dominant nor concentrated. The signal is
*large AND concentrated AND surprising*, not *unwelcome*.

⛔ **Not satisfied by a self-test.** All three wrong runs came from an instrument
whose `--self-test` was green: the arms tested the predicate the author had in
mind, and the defect was that the author's model of the corpus was incomplete.
A self-test proves the parser does what you meant; it cannot tell you that what
you meant is the wrong shape. That gap is what reading the majority class closes.

## Related

- [[a-control-that-passes-for-an-unrelated-reason]] — the same asymmetry on a
  control rather than a census: green is consistent with *absent* and with
  *cannot see it*.
- [[calibrate-over-the-history-that-contains-the-instance]] — once the population
  is believed, this is how its gate gets priced.
- [[an-absence-claim-is-a-census-over-the-corpus]] — the neighbouring rule for
  the *nothing does X* sentence a census is often written to support.
