answers: my bounded file keeps hitting its cap even though it is overwritten each time; why does a resume pointer grow; should I add an early warning before a cap; my file's contract says it cannot grow but it does; how do I tell whether a size limit is too small or the file is holding the wrong thing; two kinds of content ended up in one document, how do I notice

# An overwrite-only file cannot accumulate

- **Type:** `knowledge`
- **Date:** `2026-09-21`
- **Owner / source:** leaf `SIGNOFF-REPAIR.11.4.2.7.1`, promoted on its second
  instance. The first was `LIVE_STATUS.md` at `.11.4.2.6.5`; this one is
  `MEMORY.md`, and the director found it by asking a one-sentence question the
  author had not asked.

## The question

A file has a contract that says it cannot grow: *overwritten, not appended*.
*Regenerated from source.* *A pointer, not a log.* And it keeps hitting its size
cap anyway — repeatedly, over months, each time resolved by trimming something
to make room.

The instinct is to treat the cap as too tight, or to add an earlier warning so
the trim is less rushed. Both are wrong, and the second is worse than the first.

## The answer

> **The contract is not wrong. Something else is in the file.**

An overwrite-only file has no mechanism by which to accumulate. If it grows,
the growing part is not the thing the contract describes — it is a **second
information role** that has moved in beside the first, and it is that role,
not the cap, that needs an owner.

The diagnostic is one measurement, and it is worth taking before any remedy:

> **What fraction of this file is the thing its contract names?**

If the answer is most of it, the cap may genuinely be tight. If the answer is a
minority, stop looking at the cap.

## ⛔ Why an early-warning threshold is the wrong remedy

A health target that fires before the ceiling is the right tool for a file whose
growth is *legitimate and bounded by rotation* — a log, a changelog, an
append-only ledger. Its remedy is mechanical: retire the oldest records.

For a file that should not be growing at all, the same threshold announces a
defect the architecture already says to remove. It converts *"this content is in
the wrong layer"* into *"you have 4 KB left"*, and it will be green for as long
as somebody keeps trimming. A warning whose correct response is "delete the
thing that made me fire" is a warning that institutionalises what it measures.

⭐ The test: **name the remedy before you set the threshold.** If the remedy is a
command, a health target fits. If the remedy is a judgement about where content
belongs, the content belongs somewhere else and the threshold is a distraction.

## The measured instances

| file | its contract | what had moved in | share |
| --- | --- | --- | --- |
| `LIVE_STATUS.md` | a current-status snapshot | an accreted correction log under one heading | the table was **1.58%** of the file |
| `MEMORY.md` | an overwrite-only resume pointer | 26 standing warnings, accumulating one per session | the pointer was **19%** of the file |

Both were repaired the same way: the second role was given its own home and the
file returned to the one thing it names. Neither cap was raised. `MEMORY.md`
went 6,412 → 1,682 bytes with its warning count at zero, and nothing was lost —
every evicted item was shown to exist in a durable layer first.

## ⚠️ Before you evict, prove the content is elsewhere — per item

The eviction is only safe because the content is redundant, and *redundant* is a
claim about each item, not about the pile. This project's own census says so in
its output: an "uncited" item is a population to classify, not a count of things
safe to delete, because one of them once turned out to be an environment fact
that existed in that file and nowhere else.

> Classify each item against the durable layers by hand. The class that costs a
> **fact** rather than a **pointer** is rare, real, and invisible to a tool that
> only counts.

## Restated outside software

A noticeboard by the door is for *today's* notes. When it fills up, the answer is
not a bigger board or a note saying it is nearly full — it is that somebody has
been pinning the house rules to it, and the house rules belong in the folder
where they can be found on purpose rather than seen on the way past.

## Related

- [[a-metric-scoped-to-one-record-ages-silently]] — the numeric sibling: content
  that is correct where it sits and wrong to carry forward.
- [[an-adjudication-is-keyed-to-the-words-it-judged]] — why the eviction has to
  be checked per item against the words actually written.
- [[a-census-is-an-instrument-not-a-table]] — why the share above is measured by
  a tracked instrument rather than counted by eye.
- [[the-commit-that-reshapes-a-file-blinds-its-guard]] — the adjacent failure: a
  guard that keeps passing while the file it watches changes shape underneath it.
