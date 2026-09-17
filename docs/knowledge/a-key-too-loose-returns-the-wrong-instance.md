answers: my search found the wrong occurrence; why did grep/find give me a misleading answer; how do I locate one specific line in a large file reliably; my before-and-after metric did not move, does that mean nothing changed; how do I pick a metric that cannot match the wrong thing; I published a conclusion from a probe and it was wrong

# A key too LOOSE returns the wrong instance, at full confidence

- **Type:** `knowledge`
- **Date:** `2026-09-18`
- **Owner / source:** leaf `SIGNOFF-REPAIR.11.19.1`

## The question

You have a probe that locates something in a large file — `str.find(phrase)`, a
`grep | head -1`, a regex on rendered output — and you draw a conclusion from what
it returns. How do you know it found the thing you meant?

## The answer

> Ask the key how many times it matches **before** you use its first hit. A count
> above 1 means the probe is a coin toss. Then state the conclusion in terms of a
> property **no other candidate shares**.

This is the twin of [[a-census-is-as-wide-as-its-key]], and it fails the *opposite*
way:

| the key is… | what you get | how it announces itself |
| --- | --- | --- |
| too NARROW | fewer instances than exist | loudly, the moment anyone widens it — the count jumps |
| too LOOSE | the WRONG instance, at full confidence | ⛔ never. The output looks complete and is about something else |

## The measured instance

A leaf needed to know how the renderer treated twelve damaged table rows. The probe
was `html.find("inventory-groundwork deferral record")`, then "what element
encloses this?"

It reported the text inside a `<p>` — a paragraph — so the leaf published *"the
damage is not uniform: one row renders outside the table entirely"*. That sentence
reached five documents.

🔴 **The phrase occurs 7 times in the file.** `find` returned the first, which is
ordinary prose **1,889 lines above** the table row it was meant to locate. All
twelve rows were in fact rows; what was wrong was which COLUMN each value landed
in. ⚠️ And the near miss is instructive on its own: with a leading *"the"* the
same key matches 5 times instead of 7 — a wrong key sits very close to a right one.

## What replaced it

Two metrics that cannot match the wrong instance, because prose 1,889 lines away is
not a table row at all:

- rows whose **first cell is a date**: 21 → 33 (+12, one per repaired row)
- rows ending in **two empty padding cells**: 12 → 0

⭐ And the same run showed why the obvious metric was worthless here: the table's
**total row count is 76 before and after**. The rows were always rows. A check
counting rows would have reported success against the unrepaired file.

## The check that makes this mechanical

Before drawing a conclusion from a located instance:

1. **Count the key's matches.** `grep -c` costs nothing. If it is not 1, either
   narrow the key or say which occurrence you mean by position.
2. **Prefer a structural anchor to a textual one.** A line number, an enclosing
   element, a table column — something the wrong candidate cannot possess.
3. **State the before/after on a discriminating property.** If your metric is
   unchanged by the repair (like a row count here), it was never measuring the
   defect — that is information, not a null result.

⛔ None of this is a substitute for the renderer or the tool being the authority
(see [[a-control-is-calibrated-against-the-renderer]]). It is about making sure you
asked the authority about the right object.

## Related

- [[a-census-is-as-wide-as-its-key]] — the same failure from the other side: a key
  that misses instances rather than picking the wrong one.
- [[a-control-is-calibrated-against-the-renderer]] — ask the tool that publishes
  the page; this note is about asking it the right question.
- [[an-instrument-must-explain-its-own-failure]] — printing the detail rather than
  the count is what makes a wrong instance visible at all.
- [[a-restated-number-needs-a-producer]] — the sibling published in the same
  session: a number asserted from the shape of a corpus rather than from a command.
