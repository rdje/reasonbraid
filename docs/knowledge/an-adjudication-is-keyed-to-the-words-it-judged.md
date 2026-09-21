answers: I hand-classified some prose and stored the verdict — how do I stop it going stale; my registry row carries a human sentence and a machine field, how do I keep them together; the text changed but my adjudication still says the old thing; how do I key a verdict that was earned for a specific wording; is a hand judgement over prose allowed in a mechanical gate; what makes a curated classification durable rather than folklore

# An adjudication is keyed to the words it judged

- **Type:** `knowledge`
- **Date:** `2026-09-21`
- **Owner / source:** leaf `SIGNOFF-REPAIR.11.4.2.6.7.1`, the census of the twenty
  declared pressure controls, promoted on its **second** measured instance. The
  first shipped the defect it was built to find
  (`SIGNOFF-REPAIR.7.3.6.1`, `scripts/census_advertised_policies.py`).

## The question

Some judgements cannot be derived. *Is this control sentence a claim a machine
could evaluate?* *Is this advertised policy line actually enforced?* Both are
judgements over prose, and a matcher that tried to answer them would be guessing
at paraphrase — the failure mode `SIGNOFF-REPAIR.11.6` is about.

So you hand-classify, and store the verdict beside the text. Now: what stops the
verdict outliving the text it was earned for?

## The answer

> **Put the judged words IN THE KEY, verbatim, and refuse when they detach.**

Not the row's id. Not the file's path. Not the field's name. The exact substring
that was read and adjudicated. A verdict keyed by anything more stable than the
words it judged survives a rewrite of those words — which is precisely the event
that should have invalidated it.

The test is one question:

> **If someone rewrote this sentence tonight, would my stored verdict still be
> attached to it — and would anything say so?**

If the answer is *yes* and *no*, the key is too loose.

## The mechanical form

1. Store `(subject, quoted_clause, verdict)` — never `(subject, verdict)`.
2. On every run, assert `quoted_clause` is a **verbatim substring** of the
   subject's current text.
3. On failure, **REFUSE** — do not re-derive, do not fuzzy-match, do not fall
   back to the id. The correct response is a human re-reading the new sentence,
   because the adjudication that existed was about words that are gone.
4. Assert coverage in **both directions**: every subject has an adjudication and
   every adjudication has a subject. A subset test lets a new row arrive
   unclassified, which is the state the instrument exists to end.

⭐ **The refusal is the feature.** A detached quote is not a bug in the census; it
is the census reporting that a judgement is owed. An instrument that silently
re-attached would be converting a stale verdict into a fresh-looking one.

## ⛔ Why "close enough" is the wrong repair

When a quote detaches by a comma, the tempting fix is to loosen the comparison —
normalize whitespace, strip punctuation, match a prefix. That is the same move
`SIGNOFF-REPAIR.11.27` refused for a different instrument: *pin the instrument,
do not loosen the comparison*. A looser key does not make the verdict more true;
it makes the next real rewrite invisible.

## The two measured instances

| leaf | what was adjudicated | what the loose key did |
| --- | --- | --- |
| `.7.3.6.1` | six resolver-pack policy lines, each with an *enforced / not enforced* verdict in `.doctrine/advertised_policy_verdicts.tsv` | keyed by pack + field, flipping `subresource_policy` from `deny` to `allow` in the producer left the census **GREEN**, still reporting `enforced — .7.3.5` for a line advertising the opposite of what that leaf repaired. The value went into the key. |
| `.11.4.2.6.7.1` | twenty free-text pressure controls in `.doctrine/readme_routes.txt`, classified by whether they make a machine-evaluable claim | built with the quoted clause in the key from the start, on the strength of the row above; a paraphrased clause is refused by name, and the self-test proves both directions |

## ⚠️ What this does NOT license

It does not make the hand judgement itself checkable. Nothing here says the
adjudication was *correct* — only that it is still about the words it was made
from. The correctness half is a different discipline: evaluate every claim you
called evaluable, in the same run, so *expressible* is demonstrated rather than
asserted.

## Restated outside software

A margin note in a contract that says *"clause 7 — acceptable"* is worthless once
clause 7 has been renumbered or rewritten; a note that quotes the sentence it
approved is self-invalidating, and that is what makes it worth keeping. The
quotation is not decoration. It is the only part that knows when it has expired.

## Related

- [[a-key-too-loose-returns-the-wrong-instance]] — the same failure in addressing
  rather than adjudication: a key that resolves to something, just not the thing
  the claim was about.
- [[a-restated-number-needs-a-producer]] — the numeric sibling: a figure copied
  out of its derivation drifts in exactly this way, and the repair is also to
  re-attach it to its source rather than to re-check it by eye.
- [[a-census-is-an-instrument-not-a-table]] — why the classification ships as a
  runnable instrument with a `--self-test` rather than as a Markdown table.
- [[a-convention-is-what-the-corpus-asserts-not-what-one-surface-does]] — the
  reason a hand adjudication is sometimes the only honest instrument: the corpus
  is prose, and a matcher over it measures the matcher.
- [[an-instruments-first-population-describes-its-parser]] — the control this
  census runs against itself: the naive extractor's output describes the regex,
  not the registry.
