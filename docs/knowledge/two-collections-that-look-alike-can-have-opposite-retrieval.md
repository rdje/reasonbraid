answers: should I rotate this growing collection into history; when is it safe to retire old records; my log and my task tree both keep growing, do they need the same remedy; how do I know whether archiving will break anything; what measurement decides between a ceiling and a rotation; why did rotation work for one file and not another

# Two collections that look alike can have opposite retrieval

- **Type:** `knowledge`
- **Date:** `2026-09-21`
- **Owner / source:** leaf `SIGNOFF-REPAIR.11.4.2.9`, which was about to apply a
  rotation remedy to the largest file in the repository and measured first.

## The question

Two collections in the same project both accumulate, both are large, both are
governed by the same registry, and both are getting uncomfortable to carry. One
has already been rotated into history successfully. Does the other get the same
remedy?

The resemblance is strong and it is the wrong thing to reason from.

## The answer

> **Measure whether anything CITES an individual record. That single number
> decides it, and it is cheap to get.**

A retirement remedy — rotation, archiving, tiering, cold storage — moves a record
out of the live surface and leaves a retrieval path behind it. That is lossless
only if nothing was *pointing at the record where it used to be*. If the records
are addresses that other content resolves against, retirement does not archive
them; it breaks every reference at once.

| | the ledger | the work-memory tree |
| --- | --- | --- |
| shape | accumulates, dated entries | accumulates, id-addressed leaves |
| size | 905,695 bytes at its peak | 3,964,829 bytes |
| **references to an individual record** | **none** | **4,092** |
| correct remedy | rotate into history | leave it alone |

Both measurements were taken with tracked instruments in the same repository, and
they point in opposite directions. The first authorized a rotation that retired
430 records; the second refused one.

## ⛔ The trap is that the resemblance is real

These are not superficially similar things. They accumulate for the same reason,
they are written by the same people in the same session, they hit the same
reviewer discomfort at the same size. Everything a glance can see says *same
problem, same fix*.

> The discriminator is not visible in the file. It is in **what else in the
> project points into it**, which is a property of the corpus and not of the
> collection.

⭐ So the check is not "is this like the thing I fixed before?" but "does the
remedy that worked there depend on a property this one has?" Name the property —
here, *no consumer cites an individual record* — and go and measure it.

## What to do when the answer is "it is cited"

Not a ceiling either, and for a separate reason: a ceiling's remedy *is*
retirement, so a ceiling on a collection that cannot retire is a rule whose only
compliant response is to delete something irreplaceable.

What is left is the honest answer, and it should be recorded rather than left as
an open worry:

- state that the collection is **deliberately unbounded**, with the measurement;
- name the **cost**, so nobody re-opens it from the size alone (here: 2.04 s of a
  47 s gate, 4.3%);
- name the **lifecycle** that does bound it, if one exists (here: the tree closes
  when its work is exhausted, and the closed file stays, so references keep
  resolving).

## Restated outside software

A library's newspaper stacks and its card catalogue both fill up, and both are
just paper in boxes. Move the newspapers to off-site storage and a reader waits a
day for one. Move the catalogue cards and every shelf reference in the building
stops meaning anything. The cards are not bigger or more precious than the
newspapers — they are *pointed at*, and that is a fact about the library rather
than about the cards.

## Related

- [[a-prefix-closed-rule-costs-terminals-not-members]] — the sibling pricing
  question: measure the cost of the remedy before declining or accepting it.
- [[an-overwrite-only-file-cannot-accumulate]] — the other half of the family:
  when a bounded file grows, the contract is not wrong, something else is in it.
- [[a-metric-scoped-to-one-record-ages-silently]] — why a figure from one
  collection must not be carried to another without re-measuring.
- [[a-file-that-no-longer-exists-is-a-conclusion]] — what a retirement is allowed
  to mean once it has been shown to be safe.
