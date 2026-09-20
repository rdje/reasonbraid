answers: how do I tell a convention from a deviation; I found two surfaces answering differently which one is wrong; is this inconsistency a defect or a documented exception; how should I check a rule before writing a repair against it; why did my repair make the codebase less consistent

# A convention is what the corpus asserts, not what one surface does

- **Type:** `knowledge`
- **Date:** `2026-09-20`
- **Owner / source:** leaf `SIGNOFF-REPAIR.9.2.1.2.3` (which refuted its own premise), the divergence it corrected in `.9.2.1.2.2`

## The question

Two surfaces answer the same question differently. One of them is wrong. How do
you tell which, before you write a repair that makes the codebase *less*
consistent than you found it?

## The answer

> Count the population, and read what the tests assert. A surface that
> **documents itself as an exception** is the strongest available evidence that
> you are looking at the exception.

## The instance

A typed handler refused a malformed body with `422` and a plain-text rejection.
A neighbouring surface, `api::site_request`, mapped the same rejection into the
API's `{code, message}` shape with `400`. I generalised from that one surface —
*the API refuses in the typed shape, so the `422` is off-contract* — and shipped
a repair: an `Option` field whose absence a handler graded by hand, "so all four
verbs answer one question with one refusal."

The corpus said the opposite, and it was not subtle:

```text
46   typed Json<T> extractors in api.rs, all answering 422
 9   site routes mapping the rejection to 400
 4   test suites asserting the 422 BY NAME
 1   repair (SIGNOFF-REPAIR.4.2.2) that DEPENDS on it
```

⚠️ **Those counts are pinned to `8a3e120`, the commit they were taken at, and
the repair that followed moved one of them: typing three verbs took the
typed count to 49 and the untyped count from 7 to 4** (`SIGNOFF-REPAIR.13.4.6`
re-derived both). The note keeps the original figures because they are what the
decision was taken on — and it says so, because *the majority is not the
anomaly* is an argument about a ratio, and a ratio quoted without its commit is
the very thing `a-metric-scoped-to-one-record-ages-silently` is about. Re-derive
before quoting; never quote this block.

The assertions are not incidental. `command_api` asserts the rejection body
`contains("unknown field")`, because naming the forged field is the point.
`profiles` writes *"An unknown field is the typed 422"* two lines above *"A
malformed digest is the typed 400"* — the two levels named as two levels, in one
test. `node_channel` calls it *"the strict wire boundary"* and `.4.2.2` relies on
a missing credential field being refused there rather than at the ladder.

So the contract has two levels and both are deliberate: **`422` = the body is
not this verb's shape and the handler never ran; `400 invalid_command` = the
handler ran and the request is semantically wrong.**

## What would have caught it

`site_request` explains itself, in its own comment, one line above the code:

> Do not echo malformed caller input or driver diagnostics. Keep body-size and
> media-type refusal statuses; normalize JSON syntax/schema errors to typed 400.

That is an argument about *the site routes* — an operator surface that must not
reflect caller bytes. It is a reason to differ, which is what an exception has
and a convention does not need. Reading it would have ended the question before
the repair was written.

## How to apply

Before calling an inconsistency a defect:

1. **Count both sides.** `git grep` the shape. A 46-to-9 split is an answer.
2. **Read what the tests assert**, not only what the code does. A convention
   with four suites asserting it by name is a contract; a behaviour nothing
   asserts is an accident.
3. **Look for a self-documenting exception.** If one side explains *why it
   differs*, that side is the exception — and its reason tells you whether your
   surface shares it.
4. Only then decide, and if the minority is right, say what makes it right.

⛔ The failure here is not "a control passed for an unrelated reason" — the
controls were correct and I had not read them. It is generalising a rule from a
single surface while the corpus that would have refuted it was one `grep` away.

## What it cost

One shipped commit had to be partly reverted: an `Option` field made required
again, an error variant deleted, and a control's assertion moved from `400` to
`422` with a comment explaining the reversal. The finding UNDERNEATH the false
premise was real and survived — three verbs took an untyped body, so an unknown
field was silently ignored on all three — but the direction of the repair was
backwards, and the census is what turned it around.

See also: [`a-control-that-passes-for-an-unrelated-reason`](a-control-that-passes-for-an-unrelated-reason.md),
[`an-absence-claim-is-a-census-over-the-corpus`](an-absence-claim-is-a-census-over-the-corpus.md).
