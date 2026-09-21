answers: I replaced a hand-written list with a derivation, am I done; my census derives everything correctly and still reports the wrong total; where does the hand-written part of an automated measurement hide; why is a derived number sometimes worse than a typed one; how do I choose what corpus an instrument reads; my instrument only reads one language, does that matter

# A derivation is only as wide as the CORPUS you read

- **Type:** `knowledge`
- **Date:** `2026-09-21`
- **Owner / source:** leaf `SIGNOFF-REPAIR.11.2.1.3.2.1.1`, which found it three
  commits after shipping the instrument, while pricing an unrelated task.

## The question

A hand-written list was found short and replaced with a derivation: membership is
now read out of the code that produces the thing, exactly as
`docs/CLAIM_VERIFICATION.md` §2 requires. The instrument is tracked, self-tested,
and pins its figures to a commit.

Is the population right now?

## The answer

> **Not necessarily — because the derivation runs over a CORPUS, and the corpus
> is the new hand-written thing.** Replacing a list with a rule does not remove
> the typed decision. It moves it up one level, where it is harder to see and
> carries the instrument's authority.

*Measured instance:* a census derived fixture families from `.join("target/…")`
literals in tracked **Rust**. That derivation was correct and caught six families
a hand list had missed. It reported **359,820 KiB**. The real figure was
**2,264,752 KiB** — it was publishing **15.9% of the bytes**, because the largest
generated directories in the repository are created by **Python**, and a single
one of them was **3.64×** the entire population the census believed in.

Nothing in the instrument was wrong. The sentence *the families are read out of
the literals in tracked Rust* was in its own docstring, stated plainly, and was
the defect.

## ⭐ Why this is worse than the list it replaced

A hand-written list looks like a hand-written list. Everybody reading it knows to
distrust it, and the standard already says to.

A derived number does not. It is reproducible, it has a producer, it pins its
commit, and it re-derives to the same value every time — which is exactly what
makes it convincing. **Re-running it can never find the corpus, because the
corpus is what decides what running means.** So the check that catches a list
cannot catch this.

## ⛔ The question to ask instead

Not *where does this list come from* but:

> **What could create one of these that I am not looking at?**

Answer it by naming the CREATION idiom in each place that has one, not by
widening a glob. In the measured case every Python producer went through one
helper, `local_directory(root, "target/<family>")`, which is the volume-safety
gate every script already had to call — so the scope extended by one pattern per
corpus, and nothing was listed.

⚠️ And that also settles what to exclude. `target/debug` was 64 GiB sitting in
the same parent, and it needed no deny-list: a family is something a producer
**creates**, and nothing creates `target/debug` — the scripts only read paths
beneath it. A rule built on creation does not have to enumerate what somebody
else owns.

## ⚠️ Widening the corpus wakes the self-reference trap

The instrument's own self-test text is source code in one of the corpora. The
moment this census began reading Python it read **itself**, and published two
family names out of its own test fixtures — the count went 37 → 39 with no
producer anywhere.

Two defences, and prefer the derived one:

- match each corpus with **its own language's patterns only**, so Rust-shaped
  literals living in Python source cannot become families;
- skip a file that **imports the patterns**, because importing them is what
  makes a file an instrument rather than a producer. That test maintains itself;
  a list of filenames would be the same second copy all over again.

Then assert the defence against the **real tree**, not a model of it. The
corpus-isolation check passed while the census was reading its own text.

## Restated outside software

A census that has stopped guessing how many households there are, and now counts
every door on every street in its street list, is a better census. It still
reports nothing at all about the next town — and the more rigorous its
door-counting becomes, the more confident everyone is in the number.

## Related

- [[a-re-derivation-must-re-derive-the-population]] — the same batch, one level
  down: there the membership list was wrong, here the rule that builds it is.
- [[deriving-from-the-producer-goes-blind-when-the-producer-moves]] — the third
  face of it: right corpus, right rule, and the producer changes shape.
- [[an-absence-claim-is-a-census-over-the-corpus]] — why *nothing does X* is the
  claim this failure damages most.
- [[a-second-copy-is-only-a-risk-if-it-has-a-second-source]] — why the exclusion
  is derived from the import graph rather than written down as filenames.
