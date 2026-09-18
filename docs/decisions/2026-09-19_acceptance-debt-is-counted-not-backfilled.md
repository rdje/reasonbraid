# Acceptance debt is counted, never backfilled

- Date: 2026-09-19
- Status: accepted
- Owner: `SIGNOFF-REPAIR.11.2.6` (REPAIR-0262)
- Related: `scripts/check_task_acceptance.sh` (`--debt`), `.doctrine/acceptance_labels.txt`,
  `DOCTRINE_ENFORCEMENT.md`'s `TASK-ACCEPTANCE` row, `SIGNOFF-REPAIR.11.2.5` (calibrating a
  predicate before shipping it), `docs/CLAIM_VERIFICATION.md`.

## The situation

`TASK-ACCEPTANCE` was vacuous for roughly 200 commits: its extractor stopped at the first
matching box in the staged tree file, so every code commit was validated against one
historical bullet at line 388. Repairing the scope revealed that its vocabulary had drifted
too — it blocked on the corpus's two rarest spellings.

With both repaired, `scripts/check_task_acceptance.sh --debt` reports **73 of 281 closed
leaves** answering fewer than every question. The dominant gap is `NO REGRESSION`.

## The decision

> **Count it, publish the command that counts it, and do not backfill it.**

The gate is staged-scope-aware: it only ever examines the leaf a commit is closing now. So the
debt neither blocks a future commit nor is cleared by one. It is a measurement of how far the
practice drifted while nothing was asking, and it belongs in a number, not in a sentence.

## Why backfilling is rejected, and it is not a matter of effort

Writing `NO REGRESSION: …` into a leaf that closed weeks ago asserts that a suite was run at a
time when nobody ran one. **That is manufacturing evidence after the fact — precisely the thing
this gate exists to prevent.** Satisfying a gate by committing the offence it guards against is
self-defeating, and it would make every one of those 67 leaves *less* truthful than it is now.

⛔ The same argument rules out the softer version — "re-run the suites today and record the
result in the old leaf". That produces a true statement about today attached to a claim about
a past change, which is the shape of a false record rather than a repair. If a past leaf's
result genuinely needs establishing, it needs a leaf of its own, dated now.

## What is done instead

1. `--debt` prints the population, per leaf, with the questions each one leaves unanswered. A
   number nothing derives is a number that drifts (`a-restated-number-needs-a-producer`), so the
   figure above is not restated anywhere as a live fact — ask the command.
2. The gate holds the line from here: a leaf closing today answers all three or the commit is
   refused.
3. ⚠️ **A closed leaf is not re-opened by this.** If one of the 67 is later found to have
   shipped a regression, that is a finding with its own leaf — which is the ordinary route and
   needs no special provision.

## What this does NOT claim

⛔ Not that the 67 leaves are wrong, or that their changes regressed anything. Most were
verified; what is missing is the *record* of that verification in the leaf. The distinction
matters: this is a documentation gap measured precisely, not a quality claim about the code.
