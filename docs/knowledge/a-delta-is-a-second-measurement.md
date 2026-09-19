answers: my before/after numbers look wrong — where did the "before" come from; is it safe to infer the baseline from a previous slice; how do I catch a number I reasoned to instead of measured; why did my test-count delta not reproduce; what makes a sentence a measurement

# A delta is a second measurement

- **Type:** `knowledge`
- **Date:** `2026-09-19`
- **Owner / source:** leaf `SIGNOFF-REPAIR.13.4.2`

## The question

You publish *"40 passed (39 before)"*. You ran the suite once — afterwards. Where
did the 39 come from?

## The answer

> From nowhere a command produced. A **before/after pair is two measurements**,
> and a baseline you did not take is an inference wearing a measurement's
> clothes. The reader cannot tell the two apart, which is the whole problem:
> your sentence promises a number that was read off a run.

The measured instance: three consecutive slices reported a suite growing —
`4 → 6`, `122 → 127` — both true and both measured. The fourth reported
`39 → 40` and the suite had been **40 on both sides**. That change EXTENDED an
existing control rather than adding one; the static test-attribute count was
identical at both commits. The *39* came from the shape of the three sentences
before it.

## The rule

> Publish a delta only when you have **run the thing twice**, or say where the
> baseline came from. *"N after"* is a measurement; *"N after (M before)"* is
> two, and the second one is the one that goes unmeasured.

The cheapest baselines, when re-running is expensive:

| baseline source | cost | good for |
| --- | --- | --- |
| run it at `HEAD~1` in a worktree | minutes | anything |
| a **static** count at both commits (`git show <rev>:file \| grep -c`) | seconds | test counts, route counts, call sites |
| the previous slice's recorded output | free | only if it is the *same* command on the *same* corpus |

## The tell, and it costs nothing to apply

⭐ **The sentence names a quantity the command you ran could not have
produced.** That is checkable while writing, with no re-run:

- *"40 passed (39 before)"* — one run cannot report two.
- *"`rb-server`, `rb-node`, … are the workspace's binaries"* — a `[[bin]]` grep
  cannot see a crate that uses `src/main.rs`; the sentence claims a complete
  enumeration and the command answered a narrower question. (Nine, not six.)
- *"`respond` (5 hits)"* — `grep -c … | wc -l` counts matching **files**. The
  number was right; its unit was not.

All three shipped in one session, all three graded exact under
`docs/CLAIM_VERIFICATION.md` §4.1, and all three were catchable by reading the
sentence against the command rather than by re-running anything.

## Related

- [[a-metric-scoped-to-one-record-ages-silently]] — the other failure mode: a
  number that WAS measured and stopped being true. This one was never taken.
- [[a-census-is-an-instrument-not-a-table]] — the baseline belongs in the
  instrument, so the "before" is a run rather than a recollection.
- [[a-control-that-passes-for-an-unrelated-reason]] — the same suspicion, aimed
  at a control rather than at a number.
