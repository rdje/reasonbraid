answers: how do I know my linter agrees with the tool that actually consumes the file; why did my whole-corpus scan report zero defects while the output was visibly broken; what is wrong with writing a checker and its self-test from the same reading of a spec; how should I calibrate a control over a format I did not implement; why is my self-test passing on the exact shape that is broken

# A control is calibrated against the renderer, not the spec

- **Type:** `knowledge`
- **Date:** `2026-09-14`
- **Owner / source:** leaf `SIGNOFF-REPAIR.11.2.3` (which found it), REPAIR-0187 (which repaired it); the blind-spot rule it instantiates is `TOOLBOX.md`'s

## The question

You write a control over a file format you did not implement — a Markdown table
linter, a YAML policy checker, a config validator. You read the specification,
you implement the rule, you write a self-test asserting what you just implemented.
Every arm passes, the whole-corpus scan reports zero defects, and the gate is green
for months. What has actually been proved?

## The answer

> Nothing about the format. The self-test proves the implementation matches **the
> author's reading of the spec** — and the implementation came from that same
> reading, so the two cannot disagree.

A control over a format has exactly one authority, and it is not the document:

> **The consumer that actually processes the file in production.**

For a book, that is the renderer that publishes it. For a config, the daemon that
loads it. Ask it. Feed it the shape and read what comes out.

## The instance, and it is a sharp one

`scripts/check_table_arity.sh` flags Markdown table rows whose cell count disagrees
with their header — the defect matters because GFM **silently** drops the extra
cells and pads the missing ones, so the page looks fine and the reader loses a
column. Its cell splitter modelled a pipe inside an inline code span as part of the
cell. Its `--self-test` carried an arm named *"a pipe inside a code span is not a
separator"*, asserting zero defects for that shape.

GFM does the opposite. A table row is split into cells **before** inline parsing,
so only a backslash escape protects a pipe; a code span does not. Asked of
mdbook 0.5.2, the renderer that project publishes with:

| Shape written in the source | What the renderer emits |
| --- | --- |
| a raw pipe inside a code span | **2 cells**, backticks rendered literally, the third cell **discarded** |
| a backslash-escaped pipe | 2 cells, the pipe literal |
| a backslash-escaped pipe inside a code span | 2 cells, the code span intact and the pipe literal |
| an unpaired backtick run | 2 cells — **not an arity defect at all** |

Three consequences, and each is a distinct lesson:

1. **The corpus number was false, not merely stale.** The scan reported **0**
   defective rows across 323 tracked files. Under the renderer's rule there were
   **2**. A green whole-corpus scan is the most convincing artifact a control
   produces, and it was measuring the wrong predicate.
2. **The self-test was the *reason* it survived**, not the thing that would have
   caught it. It was written from the same wrong model, so it locked the defect in
   and made every later reader confident.
3. **The control also carried a false positive.** It scored an unpaired backtick
   run as a defect; the renderer does not. A wrong model is wrong in both
   directions, and only the consumer can tell you which.

The damage was visible on the project's own doctrine page: one registry row used a
code span containing pipes, so the renderer cut it at **516 of its 1,027**
characters and published a bare em dash where the enforcer's name belonged. The
page had looked correct to everyone who read it.

## The rule, stated portably

> **Every arm of a format control's self-test is the consumer's verdict, rendered
> before it is asserted — never the author's reading of the spec.**

Cheap to obey. Standing up a throwaway book and reading cell counts out of the
emitted HTML took minutes, and it settled nine shapes that no amount of
spec-reading would have settled, including two the author had backwards.

⚠️ And record the verdict *next to the assertion*. The repaired arms name the
renderer and its answer in the arm's own title, so the next reader can see what
the number came from rather than re-deriving it.

## Re-verify

Take any control you own over a format you did not implement. Pick the three arms
whose answers you are most sure of. Feed those exact shapes to the real consumer
and compare. If all three agree, the control is calibrated; if one does not, the
self-test has been agreeing with the implementation rather than with the format.

Related: `docs/knowledge/a-census-is-an-instrument-not-a-table.md` (the corpus
number here was exactly the table an instrument should have owned);
`docs/knowledge/writing-the-documentation-is-a-verification-pass.md` (the same
move — run the sentence instead of trusting it).
