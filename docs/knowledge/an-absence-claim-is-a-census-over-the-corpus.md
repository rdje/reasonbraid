answers: how do I know the specification really is silent on something; why did my targeted search miss the section that answers my question; when may I say "nothing in the docs covers X"; how should I escalate a decision without escalating one that is already answered; what does an unimplemented vocabulary entry tell me
# An absence claim is a census over the corpus

- **Type:** `knowledge`
- **Date:** `2026-09-16`
- **Owner / source:** leaf `SIGNOFF-REPAIR.11.14.3`, which escalated a decision the frozen specification had already made

## The question

You are about to write *"the specification does not settle this"*, *"nothing in
the documentation covers X"*, or *"this is an open design question"*. What has to
be true for that to be a measurement rather than an impression?

## The answer

> **"X is absent" quantifies over the whole corpus.** It is the same shape as
> "nothing checks X" over a codebase — and it is false the moment one section
> you did not read says otherwise.

The failure is not carelessness. It is that **a targeted search can only find
sections that share your vocabulary.** You search for the terms the question gave
you, and a section describing the same thing in other words is invisible to that
search *by construction*.

An instance: a decision about joining two subsystems was escalated as undecidable
after reading the two sections that named them. The section that settled it
described the same join as *"register context and resource references"* and
*"acquire/assess evidence within the allowed plan"* — naming neither subsystem,
and therefore unreachable from either's vocabulary. Two sections had been read.
The document had twenty-five.

**So state the population.** Before asserting absence, enumerate the corpus and
say how much of it the claim rests on:

```bash
grep -c '^### ' SPEC.md          # the population the claim quantifies over
```

A silence claim that names its denominator is falsifiable; one that does not is
an impression with a confident voice. ⭐ This is the same discipline a codebase
gap claim already gets — *"nothing checks X"* backed by the `git grep` that
returns zero — applied to prose, which usually has no gate watching it.

⚠️ **Search by MECHANISM, not only by name.** If the question is "are these two
things joined?", search for the verbs a join would use, not the nouns the two
things are called. The section you need may name neither.

## The companion diagnostic, for code

For any closed vocabulary — an enum, a `const` array, a step list, an action set
— count each entry's occurrences **outside its own definition**:

```bash
for v in "${VOCAB[@]}"; do
  n=$(git grep -h "\"$v\"" -- src | grep -v '^\s*"'"$v"'",$' | wc -l)
  printf '%-18s %s\n' "$v" "$n"; done
```

⛔ **Then classify the zeroes rather than reporting them.** A zero is either a
genuine entry that needs no code, or a capability nothing can reach, and the
question that separates them is: *is there a store, route or table sitting
behind it?* In the instance that produced this card, thirteen entries yielded two
zeroes and **neither was a defect** — while the entry that WAS a defect had a
complete, unreachable store behind it and a specification naming the step that
should reach it.

## A zero that was wrong because of one hyphen

`SIGNOFF-REPAIR.11.4.7.4` had to show that a withdrawn quality claim is still
absent. The first census returned **zero hits** — and it was wrong:

```bash
git grep -niE "quality lift|deliberation improves" -- README.md LIVE_STATUS.md docs/book/src   # 0
git grep -niE "quality.?lift|deliberation improves" -- README.md LIVE_STATUS.md docs/book/src  # 3
```

The corpus writes **`quality-lift`**, hyphenated. A zero from the first pattern
is indistinguishable from a true absence, and it is the answer the author wants,
which is exactly what makes it dangerous.

⛔ **The rule this adds: an absence census must first be shown to FIND the
places that legitimately discuss the thing.** Here the corrected pattern returns
three hits and all three are *withdrawals* — which is both the positive control
and the real evidence. A pattern that finds nothing anywhere has not measured an
absence; it has measured itself.

## The tell that is easy to misread

⛔ **An unimplemented vocabulary entry is not evidence of an open question. It is
evidence of an implementation gap.** A constant, enum variant, config key or
state name that exists and is wired to nothing means someone already decided it
should exist — which is an answer, sitting in the code, pointing at the
specification that asked for it. Treating it as part of the puzzle inverts the
evidence.

## Why this one is worth a card

Escalating a decision already made costs a reviewer a judgement they should not
have had to make, and it arrives **dressed as rigour**: *"I held this rather than
deciding unilaterally"* reads as restraint. Restraint that is actually an unread
section is the failure shape in
[[a-grouping-is-not-an-argument]] — the answer that looks careful escapes the
scrutiny a bolder one would have drawn. See also
[[a-census-is-an-instrument-not-a-table]] for the positive form: a claim ships
with the instrument that produced it.
