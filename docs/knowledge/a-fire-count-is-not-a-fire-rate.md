answers: how do I report what a gate would have caught; my new check would have fired N times, is that good; how do I compare a new gate against ones that were rejected; what denominator does a gate calibration need; why did my calibration number flatter my own work; is a standing red the same as a false positive

# A fire count is not a fire rate

- **Type:** `knowledge`
- **Date:** `2026-09-21`
- **Owner / source:** leaf `SIGNOFF-REPAIR.11.33.1`, which found the error in a
  calibration it had published three commits earlier, when the director asked
  whether the findings still held.

## The question

A new check is proposed. The responsible thing is to price it against history:
*how often would this have fired?* You run it over the past, get a number, and
compare it with the gates your project rejected for firing too much.

The number will be wrong unless you asked what the gate **scans**.

## The answer

> **An unconditional gate — one that reads the whole tree — is red for as long
> as the defect lives, not once per commit that touched it.**

Two different quantities get called "how often it fires":

| | what it counts | what it is a fact about |
| --- | --- | --- |
| commits that changed a file containing the defect | authorship | **who edited what** |
| commits whose tree contained the defect | the gate's behaviour | **how long you would have been blocked** |

For a check that only inspects the staged diff these coincide. For a check that
inspects the tree they can differ by a factor of forty.

*Measured instance:* a defect lived from one commit to another, **626 commits**
apart. Only **15** of those commits touched a shell file. The calibration
published "it fires on 15" for a gate that scans every shell file in the tree on
every commit — so the true answer was 626, and every one of those reds would
have been correct.

## ⛔ And the comparison is the part that misleads

The 15 was then set beside the fire rates of three gates the project had
rejected — 87%, 93%, 71% — to argue the new one was well behaved.

Those rejected rates were **false-positive** rates: proportions of *correct*
code the gate would have refused. The new gate's comparable number was **zero**.
Putting a red count next to a false-positive rate compares two different things,
and it happened to favour the author's own work.

> **Ask of every calibration number: is this counting refusals, or refusals that
> were WRONG?** They are the only two quantities worth publishing, and only the
> second belongs next to a rejected gate's rate.

⭐ The corrected statement is both more honest and *stronger*: zero false
positives over the whole history, and 626 true reds on a defect nobody had
noticed for two weeks.

## A standing red is not a bad gate

It is tempting to read "would have blocked 626 commits" as a reason to reject.
It is not, when the defect was genuinely present for all 626 — that is the gate
working, and the number is a measure of **how long the defect survived**, which
is an argument *for* the check rather than against it.

The failure mode a fire rate is meant to detect is different: a gate that
refuses work which is correct, so that people learn to bypass it. Only the
false-positive count sees that.

## Restated outside software

A smoke alarm that sounded for three hours while the kitchen was actually full
of smoke did not "go off 40 times" or "go off once" — it was *right for three
hours*. Counting how many people walked past it during that time tells you
about the household, not the alarm. The number that would condemn it is how
often it sounded with no smoke, and that number is usually zero or damning.

## Related

- [[calibrate-over-the-history-that-contains-the-instance]] — the prior step:
  price a rule against the history that contains the defect, not a sample.
- [[a-census-is-as-wide-as-its-key]] — the same class of error one layer down: a
  population counted on the wrong key answers a different question.
- [[a-restated-number-needs-a-producer]] — why the corrected figures now ship
  with the command that prints them and the commit they belong to.
- [[a-control-that-passes-for-an-unrelated-reason]] — the sibling caution about
  evidence that looks favourable for a reason you have not checked.
