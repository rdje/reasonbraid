answers: how do I know my test actually covers the bug it was written for; my new control passes — is that evidence; how do I falsify a control; why did my regression test pass before and after the fix; how do I test a NEGATIVE control, the one that stops a rule over-reaching; my self-test is green, is the instrument working; what should I do before trusting a check I just wrote; how do I prove a guard is driven by the thing it guards

# A control that passes for an unrelated reason

- **Type:** `knowledge`
- **Date:** `2026-09-18`
- **Owner / source:** leaf `SIGNOFF-REPAIR.11.20.2`, which extracted it from
  `docs/knowledge/a-scoping-defect-errs-in-one-direction.md` after it had fired in
  three consecutive leaves (`.11.20`, `.11.17.2`, `.11.20.1`). It was a tool that
  note used rather than that note's thesis, and a rule stated only inside another
  rule's example is reachable only by readers of that example.

## The question

You have written a control — a test, a gate, a self-test arm, a monitor — for a
defect you just repaired. It passes. What have you learned?

## The answer

> Almost nothing, until you have seen it FAIL for the reason you wrote it.
> A green control is consistent with *"the defect is absent"* and with *"this
> control cannot see the defect"*, and those are not the same claim.

The question that separates them is one sentence, and it is worth asking out loud
before the control is committed:

> **What would this have done if the defect were present?**

If you cannot answer from the control's own text, the answer is a measurement, not
an argument.

## The mechanical form

> Put the OLD, broken logic back — **in situ**, inside the current file, arms and
> all — and run the control suite.

- Every arm that claims to cover the defect must **FAIL, by name**.
- Any arm that passes **both** ways must be **LABELLED in the source** as to why:
  a no-regression arm, a boundary arm, or an arm exercising machinery the old code
  did not have all legitimately pass both ways. An **unlabelled** pass-both-ways
  arm is indistinguishable from a broken one.
- Restore, and prove the restore: `cmp -s` against a copy taken before, so the
  falsification cannot leave a residue.

⭐ **Do it in situ rather than by checking out the old commit.** The old commit has
the old controls; you need the *new* controls meeting the *old* behaviour, which
exists at no commit and has to be constructed.

## ⛔ A negative arm cannot be falsified this way, and this is the part most often missed

A **positive** arm asserts the rule catches what it should. A **negative** arm
asserts the rule does not catch what it should not — it is what stops *"read the
whole block"* collapsing into *"treat everything as a hit"*.

> A positive arm is falsified against **the past**. A negative arm is falsified
> against **the future you rejected**.

The old code did not over-reach; it under-reached. So a negative arm passes
against the old code for the same reason it passes against the fix, and the
in-situ falsification cannot fire it at all. Its red comes from somewhere else:

> Write the **degenerate** version of the new rule — the over-reaching one you
> deliberately did not ship — and run the suite against that.

*Instance (`SIGNOFF-REPAIR.11.20.1`):* a census was widened from reading one
bullet of a document to reading a whole section. The pre-fix model reddened three
arms by name and left the negative arms green. A segmenter that returned **every**
bullet as a warning — the rule's degenerate form — fired exactly the two arms that
exist to prevent it. Nothing else would have.

## ⭐ The by-product: it is also the only time your controls are run against broken code

A suite that has only ever run green has never demonstrated that it can **report**.

*Instance (same leaf):* one new arm was written `rows[1]["bullet"]`. Against the
pre-fix model the list was shorter, the arm raised `IndexError`, and it took the
whole suite down — a red that names nothing, produced by the exercise whose entire
purpose is a red that names something. Indexing it safely cost one line, and no
amount of green would have revealed it.

## Restated outside software, because the rule is not about tests

⭐ *A fire alarm that has never sounded and a fire alarm with a flat battery are
the same observation.* The building is not on fire either way, so the quiet tells
you nothing; you press the test button, and pressing it is also how you find out
whether anyone can hear it from the stairwell. A backup you have never restored,
an insurance policy never claimed on, a failover never exercised — each is a
control whose only evidence is the absence of the thing it guards against, which
is precisely the evidence that cannot distinguish working from broken.

## The three measured instances

| leaf | the control | what the green meant |
| --- | --- | --- |
| `.11.20` | a census's `--self-test`, **16 controls passing** | the census had REFUSED on every real run for dozens of commits; every fixture used the pre-rename key |
| `.11.17.2` | 12 new arms over a widened classifier | **11 failed by name** against the old code; the 2 that passed both ways were boundary arms and are labelled |
| `.11.20.1` | 13 new arms over a widened census model | 3 red by name; the negative arms needed a **degenerate** implementation to fire at all |

## Related

- [[a-scoping-defect-errs-in-one-direction]] — where this rule was first written
  down, as the check that settles a defect's direction. That note keeps the
  instance; the general rule lives here.
- [[an-injection-must-be-shown-to-land]] — the stimulus half: a control whose
  defect was never actually applied to the thing under test.
- [[a-falsification-you-can-leave-behind]] — what to do when the neutralization is
  impossible or not re-runnable: a matched pair inside the control.
- [[a-self-test-cannot-be-tidier-than-the-real-input]] — why fixtures written
  beside the code share its blind spots, which is what makes the live-corpus arm
  and this falsification complementary rather than redundant.
- [[an-instrument-must-explain-its-own-failure]] — the same standard applied to
  the instrument's refusals rather than to its controls.
- [[the-commit-that-reshapes-a-file-blinds-its-guard]] — the mechanism behind the
  `.11.20` row above.
