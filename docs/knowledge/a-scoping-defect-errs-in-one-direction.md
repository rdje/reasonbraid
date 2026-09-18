answers: my parser mis-reads a format — which way does it fail; is this bug a false positive or a false negative; how do I know a fix closed the failure I care about; why did my regression fixture pass before and after; how do I test that a repair changed anything; should I trust a bug report's claim about its own direction

# A scoping defect errs in ONE direction, and which one is a measurement

- **Type:** `knowledge`
- **Date:** `2026-09-17`
- **Owner / source:** leaf `SIGNOFF-REPAIR.11.18`

## The question

Something that computes a SCOPE — a parser's notion of a section, a query's
notion of a tenant, a sweep's notion of a population — computes it wrongly. Before
you fix it, which way was it wrong? And after you fix it, how do you know?

## The answer

> A scope is either too NARROW or too WIDE, and the two have opposite symptoms.
> Work out which, then build a fixture that classifies **differently** before and
> after. A fixture that behaves the same either way has not tested the fix.

- **Too narrow** → the thing under scope loses evidence that was really its own →
  **false positives**, and they are loud: a blocked commit, a failing check,
  somebody complaining within minutes.
- **Too wide** → the thing under scope collects evidence belonging to something
  else → **false negatives**, and they are silent for ever.

⭐ The direction is usually decidable from the structure alone, in one sentence.
Ask whether the wrong scope is a SUBSET or a SUPERSET of the right one.

## The measured instance

A doctrine gate required a "nothing checks X" claim to have a census **in its own
heading section**. Its section scanner called every `^#{1,6} ` line a heading —
including shell comments inside fenced census blocks. So a fenced `# PINNED at …`
opened a pseudo-section, and a claim after it lost the census three lines below.

The leaf that opened this wrote, plausibly: *"the dangerous direction is the
mirror of the one that fired, and it is silent"* — a claim discharging against
evidence that is not its own.

🔴 **That was wrong, and one sentence settles it.** A pseudo-section starts
*later* than the real section and ends *no later*, so it is a strict SUBSET.
A subset can only withhold a discharge, never invent one. The defect could produce
**false positives only** — which is exactly the symptom that had been observed and
was the entire visible history of the bug.

🔎 **And it was settled by fixture, not by the argument above.** A case built to
exhibit the asserted silent direction classified *identically before and after the
fix*. It is now pinned as a self-test arm labelled as the gate's DECLARED limit,
so nobody reads the repair as having closed it.

## Why the false claim was tempting

⚠️ "There must be a silent version of this" is a good instinct and a bad
conclusion. Every loud failure feels like it should have a quiet twin, and
sometimes it does. ⛔ But a bug's direction is a **claim about the code**, and
`CLAIM_VERIFICATION.md` grades it like any other: it needs a reproduction, not a
symmetry argument.

⭐ The cost of getting it wrong is concrete rather than philosophical: an
acceptance criterion demanding "an arm for the silent direction" cannot be met,
and the honest options are to write an arm that pins the *limit* — or to write one
that pretends. The second is how a self-test grows an arm that asserts nothing.

## The check that makes this mechanical

> Run the NEW control against the OLD code, in situ.

⭐ **The general rule now lives in its own note** —
[[a-control-that-passes-for-an-unrelated-reason]] — because it is a tool this note
*uses* rather than this note's thesis, and by the time it had fired in three
consecutive leaves it was reachable only by readers who arrived here thinking about
scope (`SIGNOFF-REPAIR.11.20.2`). Go there for the labelling rule, for why a
NEGATIVE arm needs a degenerate implementation rather than the old code, and for
the restatement outside software.

🔴 **The instance stays here, because it is this defect's.** That run earned its
keep immediately: it caught a defect in one of the *new arms*. The fixture for "an
indented fence is still a fence" had put its `#` at indent 2, and an indented `#`
was not a heading to the old scanner either — so the arm passed before and after,
discriminating on the **indentation** instead of on the fence.
[[an-injection-must-be-shown-to-land]] is the same failure one layer out.

## Related

- [[an-injection-must-be-shown-to-land]] — the stimulus version: a control whose
  defect was never actually applied.
- [[an-instrument-must-explain-its-own-failure]] — what to do about the input a
  scope cannot read at all: refuse, rather than silently narrow.
- [[a-census-is-as-wide-as-its-key]] — the same question asked of an enumeration
  rather than of a parser.
- [[a-falsification-you-can-leave-behind]] — why the RED belongs in the repository
  and not only in the session.
