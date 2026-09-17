answers: how many commits should I calibrate a new rule against; my proposed gate catches nothing in the calibration, is it unnecessary; what number proves a lint rule is worth adding; how do I argue for a gate without arguing about a threshold; why did my backtest miss the bug I wrote the rule for

# Calibrate over the history that CONTAINS the instance, not a fixed window

- **Type:** `knowledge`
- **Date:** `2026-09-18`
- **Owner / source:** leaf `SIGNOFF-REPAIR.11.19.2`

## The question

You have a candidate rule and a habit of pricing it across the last N commits.
The backtest comes back almost empty. Does that mean the rule is unnecessary, or
that you asked the wrong window?

## The answer

> A fixed window answers *"how often would this have fired on how people write
> lately"*. If the defect you wrote the rule for is **older than the window**, the
> backtest cannot see the rule working at all, and it reports as evidence of
> harmlessness what is actually evidence of nothing.

⭐ The rule of thumb:

- **Window** — for a rule about how people write *today*: a style constraint, a
  convention, anything whose population is generated continuously.
- **Full history** — for a rule about a defect you found in something *old*. The
  instance is the reason the rule exists; a calibration that excludes it is not a
  calibration of that rule.

## The measured instance

A gate was being extended with a second arm, for a table defect found in a file
written during the project's third phase.

| window | commits blocked | what it showed |
| --- | ---: | --- |
| last 200 commits | 1 (0.5%) | ⛔ and that 1 was the OTHER arm — the new arm appeared to catch nothing |
| all 551 commits | **19 (3.4%)** | every one of the nineteen introduced a real defect |

The instance the arm was written for entered at a commit far outside any window
short enough to be cheap. The 200-commit number was not wrong; it was answering a
narrower question than the one being asked.

## Report the OVERLAP, not the percentage

⛔ "3.4% of commits would have been blocked" invites an argument about whether 3.4
is an acceptable number. It is the wrong frame, and it is the frame in which good
rules get rejected and bad ones get waved through.

> **"The blocked set and the defect set are the same set."**

That sentence ends the argument. A rule with **zero false positives over a whole
history** is a different object from a rule that merely fires rarely, and the two
can have identical percentages. Report:

1. how many commits it would have blocked,
2. how many of those introduced a real defect,
3. and what the standing population is **today** — because a rule that ships with a
   backlog is a backlog wearing a gate's clothes, whatever its backtest said.

## The cost, stated rather than hidden

⚠️ The full sweep took 73 s against 45 s for the window, because it re-scans the
changed Markdown of every commit. That is a **per-proposal** cost paid once, not a
per-commit one — the gate itself runs against the working tree and costs
milliseconds. Confusing the two is how a cheap gate gets rejected as expensive.

## Related

- [[a-census-is-an-instrument-not-a-table]] — the calibration belongs in a tracked
  producer with a flag, so a later reader can re-run it rather than believe it.
- [[a-restated-number-needs-a-producer]] — the sibling failure: publishing the
  calibration's number into prose where nothing re-derives it.
- [[a-census-is-as-wide-as-its-key]] — the same question about the population's
  *width*; this note is about its *depth*.
