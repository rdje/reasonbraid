answers: why did dead_code not warn about my unused function; is a pub item with no callers a defect; how do I find controls that are never invoked; why does my usage census over-count; what does a pub use re-export hide

# A re-export is not a caller

- **Type:** `knowledge`
- **Date:** `2026-09-17`
- **Owner / source:** leaf `SIGNOFF-REPAIR.13.1.1`

## The question

A security control is written, tested, `pub`, and re-exported from the crate
root. The compiler is silent. Is it running?

## The answer

> `dead_code` proves reachability from the crate's **public surface**, not from
> its **behaviour**. Making an item `pub` and re-exporting it satisfies the lint
> permanently, whether or not any code ever calls it.

So for anything whose value is that it *runs* — an admission check, a validator,
a guard — the compiler's silence carries no information. The measured instance: a
five-rung adapter-verification ladder, fully implemented and tested, whose only
mentions outside its own file were **one `pub use` line**. Six call sites existed
and all six were inside the file's own `#[cfg(test)]` module.

⭐ **And the re-export is not incidental — it is the mechanism.** Without it the
item would be crate-private, unused, and `dead_code` would have said so on the
first build.

## The rule

> To ask whether a control runs, count **call sites in non-test code, excluding
> its own defining file and excluding re-exports**. A `pub use` naming an item is
> a visibility statement, not a use of it.

⚠️ **A census that counts mentions will over-count**, and by exactly one per
re-export — which is the count most likely to read as "it has a caller". In the
measured instance three separate items each showed exactly one non-test mention
outside their own file, and it was the *same* `pub use` line for all three. One
is the most dangerous possible answer here, because it looks like a caller and a
zero would have prompted a second look.

## Classify before you conclude

A `pub` item with no caller is a **population**, not a defect list. The same
census over that crate returned 17 of 102, and they were three different things:

| Class | What it is | Is it a problem? |
| --- | --- | --- |
| a control at an admission point | the finding | **yes** — it is inert where its whole value is running |
| test-support surface | reached only by integration tests | no |
| over-exposed helpers | `pub` but used only inside their own file | tidiness, not risk |

⛔ Publishing "17 items have no caller" as a defect count would have been false in
14 cases out of 17. What made the three matter was **where they sit**, not that
the census found them.

## Related

- [[a-census-is-an-instrument-not-a-table]] — the census belongs in a tracked
  script that reproduces its own baseline.
- [[an-instrument-must-explain-its-own-failure]] — the same doubt aimed at the
  measuring command rather than at the code.
- [[a-pin-a-second-path-can-bypass-is-not-a-pin]] — a control that can be routed
  around is the runtime twin of a control nothing calls.
