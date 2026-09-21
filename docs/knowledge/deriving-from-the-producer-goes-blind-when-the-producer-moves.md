answers: my census suddenly reports zero, what broke; I refactored a literal into a shared helper and a check stopped seeing anything; how do I keep an instrument that derives from code working across refactors; is deriving from the producer actually safer than a hand-kept list; what should I check in the same commit as an extraction

# Deriving from the producer goes blind when the producer MOVES

- **Type:** `knowledge`
- **Date:** `2026-09-21`
- **Owner / source:** leaf `SIGNOFF-REPAIR.11.2.1.3.2.2`, which caught it in the
  same commit that caused it — but only because it went looking.

## The question

The rule that keeps an instrument honest is: **derive membership from the code
that produces the thing, never from a description of it**
(`docs/CLAIM_VERIFICATION.md` §2). A census built that way found six families a
hand-written list had missed.

Then a refactor moves the producing expression into a shared helper. What happens
to the census?

## The answer

> **It goes blind, silently, and the failure is worse than the one derivation was
> protecting against.** A hand-kept list drifts by one entry at a time as reality
> moves. A derived list can lose an entire population in a single commit, because
> the shape it matches stopped existing everywhere at once.

*Measured instance:* fifty call sites each joined `"target/journal-tests"` and
created a directory. One commit replaced all fifty with
`Fixture::create("journal-tests", name)`, moving the join into a shared guard.
The census that derives families from `.join("target/…")` literals immediately
reported the family with **12** producing call sites instead of 62 — and had the
last twelve been converted in the same commit it would have reported the family
as **gone**, while 2,375 of its fixtures sat on disk.

## ⭐ The rule this gives you

> **An extraction is a change to the producer, so every instrument that derives
> from that producer is part of the same commit.**

Not the next commit, and not "when someone notices" — the window in between is
exactly when the instrument reports a confident, wrong, green answer. In the
measured case the new shape and its self-test case went in beside the refactor,
and the count came back to 62.

## ⛔ Do not respond by going back to a list

The correct conclusion is not *derivation is fragile, keep a list*. Both fail;
they fail differently, and the difference is what to design around:

| | how it fails | how you find out |
| --- | --- | --- |
| hand-kept list | drifts by ONE, quietly, forever | only if somebody re-derives |
| derived pattern | loses MANY at once, at a known commit | the number moves, sharply |

A sharp move at a commit you made is a signal. A slow drift is not. So derivation
stays — **and what it needs is a lower bound it can fail against**, not a list.

## ⚠️ The tell to watch for

Any commit whose diff removes the same textual shape from many files at once is
this commit. Extractions, "introduce a helper", "stop repeating the base path",
codemods. Before writing one, ask what reads that shape — gates, censuses,
lint rules, documentation generators — and bring them along.

## Restated outside software

A wildlife survey counts nests by looking for a particular kind of twig
structure. The birds start building in boxes the warden installed. The survey
does not report "fewer nests"; it reports **none**, with the same confidence it
always had, on the day the last nest moved. Nothing about the method was wrong
except that it never asked whether the thing it recognised was still the thing
being built.

## Related

- [[a-re-derivation-must-re-derive-the-population]] — the other half of this
  batch, and its mirror: there the population was wrong from the start, here a
  correct one is lost in one step.
- [[a-second-copy-is-only-a-risk-if-it-has-a-second-source]] — why the answer is
  not to keep a list beside the pattern.
- [[a-census-is-an-instrument-not-a-table]] — an instrument can be asked again
  after a refactor; a table cannot, which is what makes this recoverable.
- [[an-instrument-must-explain-its-own-failure]] — a census reporting zero should
  be able to say whether zero means *none exist* or *I no longer recognise them*.
