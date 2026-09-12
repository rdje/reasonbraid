answers: how do I make a census someone can re-run; why did my reconciliation just re-read the same source; how do I know two censuses taken months apart are comparable; what should a census script do before it reports anything; how do I stop a lexical code census from over-approximating; how should I record that my measuring instrument was wrong

# A census is an instrument, not a table

- **Type:** `knowledge`
- **Date:** `2026-09-13`
- **Owner / source:** leaves `SIGNOFF-REPAIR.3.3.4.1` (which recorded the first table), `.11.4.3` (which first re-ran it), `.3.3.4.13` (which reconciled with two instruments, each wrong before it was right)

## The question

A leaf measures something across the codebase — how many call sites, which routes
are gated how — and writes the numbers into a document. A later leaf has to say
whether that is still true. What does it actually take to answer?

## The answer

> A table is a measurement someone took once. What a later reader needs is the
> **instrument that took it.**

Re-reading the source and writing new numbers produces a second table, and a
second table proves only that someone read the source twice. It cannot show that
the two readings asked the *same question*, which is the whole content of "is this
still true".

So a census belongs in a script, tracked beside the document, and the script has
one duty before it reports anything:

> **Reproduce the recorded baseline exactly.**

`scripts/census_authority_paths.py` reproduces `1ba6184`'s 101 files, 1,749,975
bytes, its corpus SHA-256 and its 42 call locations before it says a word about
today. A change to the predicate that breaks that reproduction makes every later
comparison incomparable — and that is the failure the self-check exists to
prevent, not a nicety.

Once it holds, divergence becomes readable. At `.3.3.4.13` the corpus GREW from
101 files to 118 while direct named-call locations FELL from 42 to 27. Two numbers
moving opposite ways is a signature: callers stopped naming an authority function
themselves and started entering a guarded service that names it once. Neither
number says that alone.

## A lexical census has a scope where it stops working, and you must find it

Lexical instruments over-approximate, and the over-approximation is not gradual —
it falls off a cliff at a particular scope.

`.3.3.4.13` needed to know which gate each HTTP route reaches. Three versions:

| Scope | Result |
| --- | --- |
| the handler's own body | **wrong**: 5 routes called unadmitted, because a handler that delegates carries its admission one level down |
| local calls to a fixed point, within the handler file | **right** |
| calls across the whole crate, to reach the SQL in the service modules | **wrong**: 53 of 118 routes classified as reaching the thread-command path, 41 as reaching a guarded transaction — once the closure leaves the handler file it reaches shared helpers that reach everything |

The lesson is not "go transitive" or "don't go transitive". It is that the useful
scope is an empirical property of the codebase, found by running it and looking at
whether the answer is credible. **A census that classifies most things into one
bucket is reporting its own method, not the code.**

When a question genuinely cannot be answered at the working scope, answer it from
a different source rather than stretching the instrument. That census gets
"does this route mutate" from the route's declared HTTP verb — a fact the router
states — instead of inferring it from code it cannot reach.

## Record the wrong answers in the instrument

Both of the above are in the script's docstring, with their numbers. So is a
third: a reachability check that reported two live generic functions as dead,
because `fn name<E>(` never matches a call-shaped pattern and the counter then
compared one real call against a declaration it had failed to count.

> ⚠️ Note which direction an instrument's errors run. One that misses a defect
> costs you a finding. One that **reports live code as dead** invites someone to
> delete it. The second kind deserves a permanent note, not a silent fix.

A script carrying its own history of wrong answers is more trustworthy than one
that appears to have been right first time, because the next person to change the
predicate can see which changes have already been tried and what they cost.

## Re-verify

Run the instrument at its recorded baseline commit and confirm it reproduces the
recorded fingerprint exactly. Then change one line of its predicate and confirm
the reproduction breaks — if it does not, the self-check is not checking anything.

Related: `docs/knowledge/proving-a-path-still-names-what-you-created.md`;
`docs/knowledge/writing-the-documentation-is-a-verification-pass.md`.
