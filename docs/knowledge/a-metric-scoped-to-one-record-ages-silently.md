answers: my published figure kept moving plausibly and turned out to mean the wrong thing — why; how do I define a residue or backlog count so it does not go stale; why did a number stay correct while its meaning drifted; a figure I cannot reproduce — is it wrong; how do I scope "already decided" without making the key so wide it always says yes

# A metric scoped to one record ages the moment a second record is written

- **Type:** `knowledge`
- **Date:** `2026-09-19`
- **Owner / source:** leaf `SIGNOFF-REPAIR.7.1.2.2.3`, re-deriving `SIGNOFF-REPAIR.7.1.2.2`'s published figures

## The question

You publish a *residue* — the part of a measured population that nobody has
adjudicated yet. It is defined as *the members not named in the decision record*.
The number moves sensibly for several commits. What goes wrong, and how do you
notice?

## The answer

> **"The decision record" is a snapshot of the durable layer, not a definition.**
> The moment a second record decides part of the same population, a metric that
> names only the first counts adjudicated members as outstanding — and it keeps
> reporting a plausible number while doing it. Scope the metric to *the records
> that adjudicate this population*, name them, and check that each one exists.

The measured instance. A census counted the site-global tables written on
enrolment alone and reported the residue against one decision record:

| commit | tables | residue | correct at the time? |
| --- | --- | --- | --- |
| `e4604ad` | 30 | 17 | yes |
| `4c36840` | 29 | 16 | yes |
| `d1ba384` | 27 | 14 | yes |
| today | 17 | **5** → really **0** | no |

Every published value re-derives exactly. Nothing was ever wrong. But two more
records had since adjudicated the population, and all five tables the metric
still called outstanding were decided in full by one of them. The figure was
right, its meaning was not, and no drift check could see it — because the number
it printed was still a number that looked like the last one.

## How it is found

Two questions, in this order:

1. **Can the figure be reproduced by a command?** If not, that is
   `docs/CLAIM_VERIFICATION.md` Leg 3 answered `no`, whatever the figure's truth.
   A correct number with no producer is the case this note is about: it will
   still be correct next week, and you will have no way to know.
2. **What does its key name?** If the key names a *document*, ask what happens
   when a second document of the same kind is written. If nothing happens, the
   metric has an expiry date nobody set.

## The trap on the other side

Widening the key to *any* record in the durable layer is the obvious fix and it
is worse. Scanning all 170 decision records returned residue 0 too — but a bare
table name is an ordinary word, so `derivations` matched **nine** records and
`resource_references` **twelve**, most about something else entirely. That is
[`a-census-is-as-wide-as-its-key`](a-census-is-as-wide-as-its-key.md) in the
direction that MANUFACTURES a clean answer, which is the more dangerous one: a
residue of 0 that nothing could have made non-zero.

The scope that survives both traps is the *declared* list of records that
adjudicate this population, with an arm asserting each one is tracked — a
missing record inflates the residue, which is the safe direction.

## The arm that makes a zero mean something

A residue of `0` and a function that returns nothing are indistinguishable from
the outside, so the negative arm must run against a **degenerate input**, not
against the old code: feed the census a row writing a table no record could name
and assert it comes back. Written first as list equality, that arm failed for an
unrelated reason the moment the live residue moved; as a membership test it is
independent of the population. See
[`a-control-that-passes-for-an-unrelated-reason`](a-control-that-passes-for-an-unrelated-reason.md).

## Where this has bitten

- `SIGNOFF-REPAIR.7.1.2.2.3` — the instance above.
- `SIGNOFF-REPAIR.6.1.5.3.1` — the sibling failure one step earlier: a figure
  *carried* between documents rather than derived. The rule there is
  **subtract, and if the arithmetic does not close, the starting figure is the
  stale one.** Both are the same family: a number whose producer is not tracked.
