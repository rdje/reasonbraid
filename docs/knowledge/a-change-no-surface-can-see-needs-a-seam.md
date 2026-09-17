answers: how do I test an optimisation that changes no output; my control passes with and without the repair — what now; where should a control live when the product cannot observe the change; is a database statistics counter a good instrument; how do I falsify an in-handler optimisation

# A change no surface can see needs a seam you can test

- **Type:** `knowledge`
- **Date:** `2026-09-17`
- **Owner / source:** leaf `SIGNOFF-REPAIR.11.14.3.7`

## The question

A repair removes redundant work inside a request handler. Every byte the product
returns is identical afterwards, and so is every row it writes. How do you show
it works?

## The answer, in the order it has to happen

**First, notice.** The tell is a control that passes against the *unrepaired*
code. The measured instance: a handler registered one database row per cited
`(uri, digest)` pair, and the repair skipped the repeats. The row count was
**1 either way** — the store's own replay already returned the existing row — so
an assertion on rows could never discriminate.

⛔ **Second, do not reach for a cleverer external instrument first.** A
`pg_stat_user_tables` scan-counter probe was written for exactly this, read
either side of the request. It reported **the same value with and without the
repair**, so its assertion could not fail. A control that passes identically
either way measures nothing, and shipping it is worse than shipping none —
it converts an unverified change into one that looks verified.

⭐ **Third, move the control to a seam.** Extract the decision into a small pure
function the handler calls, and falsify *that* directly:

```rust
fn first_citation_of_each_pair(citations: &[EvidenceRef]) -> Vec<usize>
```

One answer per input, so the caller's record stays one-for-one; the function is
total, deterministic and testable in microseconds. Injecting a wrong
implementation — keyed on the locator instead of the pair — turned the test red
with `left: [0, 0, 0, 0]` against `right: [0, 1, 0, 3]`.

## The rule

> When a repair changes no observable output, **the control belongs at a seam you
> create for it**, not at the product's edge. Extracting the decision into a pure
> function is the cheapest seam, and the extraction is a refactor with its own
> compiler-checked correctness.

And say so where the control would have been. The live control keeps the arms
the product *can* show — the O(n) that is observable, the record, the semantics —
and a comment naming the unit test and the discarded instrument, so the next
reader does not re-derive the probe that failed.

## The second rule, which is the one that costs

> ⛔ **Any instrument must be shown to discriminate before its verdict is used.**
> Run it against the unrepaired code. If it reports the same value, it has told
> you nothing, whatever it reported.

⚠️ This is the sharp edge of `docs/CLAIM_VERIFICATION.md` §3 leg 2. The scan
counter was not obviously broken: it returned a plausible number, from a real
system view, about the right table. It simply did not move. **A number that does
not move is indistinguishable from a number that moved by zero**, and only the
red run separates them.

## Related

- [[a-falsification-you-can-leave-behind]] — how to falsify when the predicate
  cannot safely be removed; this note is what to do when there is nothing to
  observe in the first place.
- [[an-instrument-must-explain-its-own-failure]] — the sibling failure: there the
  instrument discarded its evidence, here it could not discriminate.
- [[a-census-is-an-instrument-not-a-table]] — an instrument is a tracked artefact,
  and a discarded one is worth recording too.
