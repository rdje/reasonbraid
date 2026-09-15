answers: my repair closed a limitation — what else do I owe; why did the documentation still describe a limit I removed; why did no gate catch two sentences that contradict each other; how do I find every place that states a limitation; why did the superseded service survive when its siblings were deleted; does the compiler tell me a public function is dead; why did my leaf's split miss a mechanism its own goal line names?

# A repair owns every sentence that states its limit

- **Type:** `knowledge`
- **Date:** `2026-09-15`
- **Owner / source:** leaves `SIGNOFF-REPAIR.3.3.4.12.2`, `.11.9.1.3.2`; the falsified limits belong to `.3.3.4.12`, `.3.3.4.12.1`, `.3.3.4.11.3`

## The question

A repair removes a limitation. The code is right, the controls are green, the
leaf records the measurement. What is still owed?

## The answer

**Every sentence that states the limitation, in the same commit.** A limitation
is not only a property of the code; it is a claim that gets written down — in a
doc comment, in a neighbouring module's comment, in the book, in a leaf. Those
copies do not move when the code does, and each of them keeps reading as true
because it was true when written.

The instrument is cheap and specific: **grep for the limitation's own words, not
for the leaf id.** A sentence that becomes false almost always quotes the thing
it is about, so the words survive the copy while the leaf reference often does
not.

```bash
# the limit was "these verbs take no guard" — search the CLAIM, not the ticket
git grep -n "take no guard\|takes no guard\|no guard at all" -- crates docs
```

Measured instance: `.3.3.4.12` put three federation direction verbs under a
tenant guard and `.3.3.4.12.1` made a card import declare both tenants' keys.
Neither searched. The superseded sentence — *these verbs take no guard, so
nothing here can fence them* — survived in two source files and in the published
book for two leaves, in the present tense, naming the very leaf that had already
landed.

## Why no gate catches it

Because each sentence is individually well-formed and was individually true.
The book ended up saying, in ONE chapter, that an import "is now fenced by a
revocation from either side" and, forty lines later, that nothing there "can
fence them". Only the CONJUNCTION is wrong, and a mechanical check would have to
understand what the sentences mean.

That is the argument for the grep rather than for a new gate: the failure is
semantic, so the defence has to be a habit attached to the repair.

## Three things that make this worse than it sounds

- **A conservative error still costs.** The stale sentence UNDERSTATED what was
  protected, so nothing unsafe followed. What followed is that the project's
  primary review surface disagreed with itself about a property two leaves spent
  commits proving — and a reader cannot tell a conservative staleness from a real
  limitation without re-deriving both.
- **Record the superseded sentence; do not silently replace it.** The same claim
  had propagated to three files. Saying *this used to be true and here is what
  changed it* is what stops the next reader re-deriving the old answer from a
  copy you did not find.
- **Distinguish the live claim from the historical record.** A dated leaf or
  status entry saying "at the time, X held" is correct and must not be "fixed".
  Only present-tense claims about current behaviour are in scope. The test that
  already wrote it in the past tense needed no change.

## The sibling rule: superseded code is a sentence too

When a repair replaces a path, the superseded path is the most durable statement
of the old limit. Delete it, or make its own doc say it is superseded and name
its replacement.

⛔ **Do not rely on the compiler to notice.** `dead_code` does not apply to a
`pub` item in a `pub mod` — a public item is assumed to have callers outside the
crate. Two sibling leaves deleted their superseded bridges because the compiler
flagged them; three `pub` functions with no caller anywhere survived for exactly
that reason. The instrument there is an explicit caller census, confirmed by an
all-targets build after the deletion:

```bash
git grep -c "mod_name::fn_a\|mod_name::fn_b" -- crates      # rc=1 means no caller
cargo check -p <crate> --all-targets --locked                # quantifies over every target
```

## The same shape one level up: a leaf's own split

A goal line names the mechanisms a leaf owns. A split draws children. Nothing
checks that the second covers the first, and the sentence recording the split
frequently contains its own arithmetic.

Measured instance: `SIGNOFF-REPAIR.3.4`'s goal line names five mechanisms; its
split bullet says "five children along the **four** mechanisms the goal line
names plus the two follow-ups". Two mechanisms — a delegated subject's consent,
and binding the replay hash to the command's target — got no child, and the leaf
read as complete with all five children `done`.

⭐ **At every split, count the goal line's mechanisms against the children and
require the split's own sentence to reconcile.** It is arithmetic rather than
judgement, it takes one minute, and no census driven by the goal line can find
the gap afterwards — because the text a census reads is not missing anything.

## Related

- [[a-census-is-an-instrument-not-a-table]] — a count written while reading is
  not a count; this is that rule applied to a leaf's account of itself.
- [[writing-the-documentation-is-a-verification-pass]] — the converse habit:
  writing the docs finds the defect. This is what happens when the docs are
  written once and the code moves underneath them.
- [[a-claim-of-sameness-is-worth-its-call-graph]] — a module header claiming it
  shares another surface's behaviour, checked against what it calls.
