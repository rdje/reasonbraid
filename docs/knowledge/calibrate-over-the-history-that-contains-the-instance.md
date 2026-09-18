answers: how many commits should I calibrate a new rule against; my proposed gate catches nothing in the calibration, is it unnecessary; what number proves a lint rule is worth adding; how do I argue for a gate without arguing about a threshold; why did my backtest miss the bug I wrote the rule for; my backtest says this rule would never have fired — can I trust that; why does a mature gate always backtest at zero; how do I re-argue a rule that is already enforced

# Calibrate over the history that CONTAINS the instance, not a fixed window

- **Type:** `knowledge`
- **Date:** `2026-09-18`
- **Owner / source:** leaf `SIGNOFF-REPAIR.11.19.2`; the second failure mode below is leaf `SIGNOFF-REPAIR.11.18.2`

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

## The SECOND way a calibration's zero lies: the history was already policed

⭐ Absorbed here rather than given its own note (`SIGNOFF-REPAIR.11.18.2`), because
this note's `answers:` line already claims the reader's question — *my proposed
gate catches nothing in the calibration, is it unnecessary*. A reader with the
second failure mode lands here, is handed the window answer, widens the window,
and gets the same zero. ⛔ **A note that advertises a question owes every answer
to it**; splitting them makes the host a confidently wrong reply.

> Once a rule is **enforced**, every commit that LANDED had already been edited
> until it passed. A replay across that interval returns 0 **by construction**,
> and that 0 reads exactly like *"this rule would never have fired"*.

It measures the gate's **deterrence**, not the rule's **cost**, and the two are
indistinguishable from the number alone.

| | window too SHORT | population already POLICED |
| --- | --- | --- |
| what the 0 means | the defect predates the window | the defect was edited away before landing |
| the fix | widen the window | find history, or a corpus, the rule did not reach |
| does widening help? | ✅ yes | ⛔ **no** — more policed history is more zeros |

### The measured instance

Extending a doctrine gate to a directory it had never covered replayed at **0 of
559** commits blocked — a real zero, because nothing had ever policed those files.
The **identical harness** over the corpus the gate *had* policed since the initial
commit also returned **0 of 559**, and that one is an artefact.

⚠️ **The distortion is not static, which is the part that decides whether to care.**
It is the fraction of the window lying after registration, so it grows
monotonically — and it is **total** exactly when a mature rule is being re-argued,
which is when a calibration matters most. Measured across this project's three
calibration-bearing instruments on the day the trap was found: **2 of 3** replay
over a corpus their own `--check` gates, at 12 and 10 commits of a 200-commit
window — **5–6% today**, 100% eventually.

### The discriminator, and the disclosure

> Before believing a calibration's 0, ask: **was this population under
> enforcement during the replay?**

⭐ It is mechanizable for ~17 ms, and it is a **disclosure rather than a refusal**:
the post-registration number is the right one when the question is *does the gate
still hold*, and the wrong one when it is *what would this rule cost*. Only the
caller knows which, so the instrument names the span and lets them say. `git log
-S <script> -- <the enforcer>` finds the registration commit; `git rev-list
--count` gives the overlap.

### ⛔ And prove the harness before believing any zero

*Nothing to find* and *broken replay* produce the same output. A synthetic
positive through the same classifier invocation, plus its discharged twin,
separates them for the cost of two fixtures.

### Restated outside software

⭐ *"How many drivers would this speed limit catch?"*, measured on a road that
already has a camera. Almost none — because they slow down for the camera. The
number is real, it is reproducible, and it says nothing whatever about the limit.
Measure on a road without one, or measure before the camera went up.

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
- [[a-control-that-passes-for-an-unrelated-reason]] — the same suspicion aimed at a
  control rather than at a calibration: a green that cannot distinguish *working*
  from *blind*. Proving the replay harness above is that rule applied here.
