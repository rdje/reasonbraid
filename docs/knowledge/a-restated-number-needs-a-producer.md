answers: a number in my docs went stale — should I add a gate; how do I stop documentation drifting from the code; what is the difference between a historical measurement and a live claim; my two copies of a fact disagree, which one do I fix; when is a doc-consistency rule worth mechanizing; how do I write a count into prose safely

# A restated number needs a PRODUCER, not a rule

- **Type:** `knowledge`
- **Date:** `2026-09-18`
- **Owner / source:** leaf `SIGNOFF-REPAIR.11.16`

## The question

A document restates a number that something else computes — a population size, a
count of instruments, a percentage. It drifted. The obvious response is a gate:
*"a number in the mirror must match its census."* Is that the right instrument?

## The answer

> Usually not. Ask a smaller question first: **can the thing that owns this number
> print it?** If it can, delete the number and cite the command. If it cannot, the
> number is history — anchor it to the moment it was measured.

A gate has to tell a **frozen historical measurement** ("it would have fired on all
nine") from a **live claim about the current tree** ("the product emits 18"). Only
the second can be wrong. Separating them is a judgement over prose, and every
mechanical approximation of it is a growing list of special cases.

⭐ The two classes have different remedies, and neither remedy is a rule:

| the number is… | what makes it safe |
| --- | --- |
| a live population size | a `--census` flag on the instrument that enumerates it; the prose cites the command |
| a measurement of a past tree | a citation *in the same sentence* — a commit, a leaf, a named upstream — so a reader can see it is not a claim about now |

## The measured instance

A doctrine registry described itself as "the human-readable mirror of the
registry". Two sibling gates already existed because a second copy of a fact,
derived by nothing, had drifted from the first — one of them for 39 of 60 commits.
This was the third instance of that shape, **inside the file that documents the
gates**.

The population, by command: **103 numerals** across 19 rows. Three candidate rules
were priced against it before any was proposed:

| candidate | what it would fire on | verdict |
| --- | ---: | --- |
| a numeral must sit in a sentence carrying a freezing citation | 68 of 103 (66%) | ⛔ rejected |
| the same, minus four mechanical "structural" exclusions | 51 of 103 (50%) | ⛔ rejected |
| the same, scoped to the staged diff | 10 of the 14 commits in 200 that add one (71%) | ⛔ rejected |

Against the same project's thresholds: two earlier gates were rejected at 87% and
93% for teaching bypass, and the one that shipped fired on 9.5%. ⛔ A per-numeral
allowlist was rejected too, by the project's own measured sentence in a
neighbouring row — *"an allowlist thirty entries long teaches bypass"* — against a
population of 103.

🔴 **Then the interesting part: verifying the numbers one at a time found five
rows in trouble, in four different ways, and only one way needed a rule.**

| row | state | remedy |
| --- | --- | --- |
| 4 numerals across 2 rows | **stale** — "17 self-tests of 28 instruments" was 29 of 38; "642 tracked files" was 773 | a `--census` flag; the prose cites the command |
| 1 row | **drifted, then true again BY ACCIDENT** — a later change retired a code and restored the count | the numbers deleted, not corrected: a mirror that happens to agree teaches a reader it never drifted |
| 1 row | **prose, not the number** — a founding measurement reading as a present-tense claim, false today | a citation added in the same sentence |
| 1 row | **correct and latent** — a self-test banner reading `9/9` beside nine arms, hardcoded in two places | derive the count |

⭐ The four stale numerals share one property the other 99 do not: each is a
POPULATION SIZE that the named instrument enumerates on every run and simply never
printed. Each was *also* stale in the instrument's own header comment — the mirror
was a mirror of a mirror. Two `--census` flags fixed all four, permanently, with no
rule at all.

⭐ **A census is a claim about a MOMENT, and the moment passes.** A second
instance, from `SIGNOFF-REPAIR.4.2.3.1`: the leaf inherited *"both writers of
`node_leases.lease_expires_at`"* from the census `.4.2.3` ran at its own closure.
That census was correct when run — and `.4.1.5` (REPAIR-0167) added a **third**
writer three leaves later. Nothing related the number to the code it counted, so
the leaf opened, sat and was worked on a population that had moved.

⚠️ The tell is specific to inherited numbers: it was not restated in a live
document, so no currency check could see it; it was restated in a TASK LEAF,
which reads as a historical record and is therefore trusted like one. ⛔ A leaf's
census is history; a leaf's *Owns* line is a plan, and a plan written from a
stale census is a plan for the wrong work. Re-derive at the moment you act.

## Why the gate was tempting

⚠️ Because the defect is real and the rule sounds obviously correct. The trap is
that "obviously correct" was never priced. The measurement is what shows the rule
would fire mostly on *legitimately frozen* census results whose anchoring citation
happens to sit one sentence away — dated measurements that are correct forever and
that a gate would demand be rewritten.

⭐ And the measurement redirects the work rather than merely blocking it. Asking
*which* numbers actually drifted — instead of *how do I gate all of them* — is what
surfaced that the drifting ones share a property, and that the property has a
cheaper remedy than any rule.

## The check that makes this mechanical

> A number that is derived on every run cannot go stale. So the rule is not over
> the prose — it is over the instrument: **anything that enumerates a population
> must be able to print its size.**

Two consequences worth applying without waiting for drift:

1. When you write a count into prose, ask which command produces it. If the answer
   is "none", you have just created a mirror.
2. An instrument's own banner counts as prose. One self-test printed a hardcoded
   `9/9 arms` beside nine arms — correct, and one added arm away from publishing a
   false total. Deriving the count is a two-line change and is falsifiable: add an
   arm, watch it read `10/10`, restore, watch it read `9/9`.

## Related

- [[a-census-is-an-instrument-not-a-table]] — the same argument for the census
  itself: the numbers belong in a tracked producer, not in a document.
- [[a-census-is-as-wide-as-its-key]] — the measurement only counts what its key
  can see, which is how the first pass here found 6 numerals where there were 103.
- [[an-absence-claim-is-a-census-over-the-corpus]] — the neighbouring shape: a
  claim quantified over a corpus, which IS gated, because its population is
  mechanically identifiable and this one is not.
- [[a-repair-owns-every-sentence-that-states-its-limit]] — why the discharged rows
  say what they used to claim instead of being quietly corrected.
