answers: my neutralization went red and the restore went green — what does that actually prove; how do I know my controls test the repair's precision and not just its presence; how do I tell whether two controls are bounding two different things or the same thing twice; I fixed the headline of the finding, how do I check I fixed the finding; how do I pick a second neutralization that is worth running

# Neutralize into the fix you almost wrote

- **Type:** `knowledge`
- **Date:** `2026-09-16`
- **Owner / source:** leaf `SIGNOFF-REPAIR.7.2.3`, REPAIR-0206 (where the second neutralization was the only one that said anything new)

## The question

The standard falsification here is: take the repair out, watch the right legs go
red, put it back, prove the file is byte-identical, re-run green. It is a good
move and `docs/CLAIM_VERIFICATION.md` leg 2 asks for it.

But notice what it compares. It compares the repair against **its own absence**,
and the absence is the state everybody already agrees is broken. A control that
distinguishes "the fix is there" from "there is no fix at all" is a low bar —
almost any change to the right line clears it. So after the red-restore-green
cycle you know the control is wired to the code you touched. You do **not** know
the control can tell your repair from a worse one.

## The answer

> Neutralize a **second** time — into the fix you almost wrote.

Not the absence of the repair: the *plausible wrong repair*. The one that
satisfies the finding's headline sentence, that a hurried author would have
written, that reviews fine. Run the controls against that.

- The first neutralization tests the controls against the repair's **absence**.
- The second tests them against the repair's **neighbours**, which is where a
  real regression will come from — nobody deletes a gate, they simplify it.

If every control that was red in the first run is red again in the second, the
second run taught you nothing and your controls are all bounding the same thing.
If a **subset** goes red, that subset is the part of the repair the rest of your
suite cannot see, and you have just learned which control is load-bearing for
which half.

## The instance

`SIGNOFF-REPAIR.7.2.3` repaired a Git LFS gate that refused any blob containing
the ten-byte run `version ht` anywhere in its first 64 bytes — so a file that
merely *quoted* the LFS spec URL was rejected as if it were the content it
described. The repair anchors the test at offset 0 and matches the whole version
line.

The headline of the finding is **"the predicate is unanchored"**. So the fix
almost written is:

```rust
fn is_lfs_pointer(data: &[u8]) -> bool {
    data.starts_with(b"version ht")   // anchored! headline satisfied.
}
```

Two controls had gone red in the first neutralization: a documentation file
quoting the line, and a manifest whose first line is `version https://example.org/…`
for an unrelated specification. Against the almost-fix:

| Neutralization | Result |
| --- | --- |
| the shipped repair | `11 passed; 0 failed` |
| the original 64-byte search (absence) | `9 passed; 2 failed` |
| `starts_with(b"version ht")` (the almost-fix) | `10 passed; 1 failed` |

The prose control went **green** and the other-specification control stayed red.
That one row is the whole lesson: anchoring and spec-matching are two separate
properties of the repair, each held up by exactly one control, and the first
neutralization could not have told them apart because it broke both at once.

## How to choose the almost-fix

⭐ It is usually the fix you can remember considering. Failing that:

- **Satisfy the finding's headline sentence and nothing else.** A finding is
  written as a slogan — "unanchored", "no tenant check", "not in a transaction".
  The almost-fix is the minimum edit that makes the slogan false.
- **Take the first clause of the acceptance and ignore the rest.** Acceptance
  clauses are usually written one per property; honouring one is the shape of a
  partial repair.
- **Prefer an edit a reviewer would approve.** If it looks obviously wrong on the
  page, a future author will not write it, and neutralizing into it proves
  nothing about a risk anyone runs.

⛔ It is not a random mutation. Mutation testing asks whether the suite notices
arbitrary damage; this asks whether the suite notices the **specific** plausible
mistake the finding invites, which is a much smaller and much more likely set.

## When it does not apply

- A repair with **one** property has no neighbours worth visiting: absence and
  almost-fix are the same edit, and one neutralization is the honest answer.
- A repair that **changed a type signature** cannot host either neutralization;
  see [[a-falsification-you-can-leave-behind]] for the matched-pair instrument
  that replaces both.
- ⚠️ Neither neutralization is durable. Both are transcript entries, proved once,
  to whoever was present. The second one costs a compile and buys a real fact,
  but if the property deserves to be checked for ever, it belongs in a control.

## Related

- [[a-falsification-you-can-leave-behind]] — what to do when you cannot
  neutralize at all, and why a matched pair inside the control outlives either
  neutralization.
- [[a-control-is-calibrated-against-the-renderer]] — the adjacent worry: a
  control written from the same reading as the code cannot disagree with it.
  This one is about a control that agrees for too wide a range of code.
- [[a-self-test-cannot-be-tidier-than-the-real-input]] — the almost-fix's
  counterpart on the input side: a fixture that is neater than production hides
  the same class of imprecision.
