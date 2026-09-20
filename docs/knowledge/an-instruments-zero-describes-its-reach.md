answers: my gate is green — does that mean the codebase is clean; the census reports zero undocumented, can I trust it; why did a passing check turn out to be measuring nothing; how do I tell "no findings" from "no reach"; what should a gate assert about its own parse; my grep returned a count, is it counting what I think

# An instrument's zero describes its reach

- **Type:** `knowledge`
- **Date:** `2026-09-20`
- **Owner / source:** leaf `SIGNOFF-REPAIR.13.4.6` (the verification pass that found three of these in one session), and the two repairs it opened, `.13.4.6.1` and `.13.4.6.2`

## The question

An instrument reports **nothing**: a gate exits 0, a census prints `0`, a grep
returns no lines. That is the answer you were hoping for. When is it evidence
about the codebase, and when is it evidence about the instrument?

## The answer

> **A negative result is a statement about the instrument's REACH until
> something proves it is a statement about the world.**
> A positive finding carries its own evidence — here is the thing, at this
> line. A zero carries none. It is equally produced by *there is nothing
> there* and by *I cannot see anything*, and those two are indistinguishable
> from the output.

⛔ This is the sibling of
[`an-instruments-first-population-describes-its-parser`](an-instruments-first-population-describes-its-parser.md)
and the harder half. There, the instrument found **too much**, and the majority
class was inspectable — you can look at the findings and recognise your own
parser in them. Here the instrument found **nothing**, so there is no
population to inspect, and the failure is silent for as long as nobody asks a
different question.

## Three instances, one session

**1. A gate reported full coverage over a set it could not see.**
`census_reason_codes.py` matched `code:\s*"…"` — a Rust struct field — and
printed *"… not documented in the book: 0"*. One code, `undeclared_region`, is
emitted as `json!({"code": "…"})`, a quoted JSON key. It is returned as a real
`400`, asserted by two test suites, and documented nowhere. The gate existed to
prevent exactly that and reported success for as long as it had existed.

**2. A gate passed over a table it had not read.**
`check_frontier_status.py` parses the frontier table and applies three rules. A
blank line ends a GFM table, so one inside it makes the parser correctly stop —
and the three rules then have nothing to object to. Measured: one blank line,
**exit 0**. Ten historical commits carried that shape.

**3. A count answered a different question than its label.**
`git grep -c '()'` returned 733 and was published as *733 occurrences*. It
counts **lines**. The occurrences are 948. Nothing failed; the tool answered
exactly what it was asked, and the label claimed something else.

⚠️ A fourth, from the verification pass's own probe: subtracting a definition
that the pattern never matched turned 9 call sites into 8. Caught only because
a second route disagreed.

## How to apply

**Ask an instrument to prove its reach in the same run.** Three cheap forms,
in ascending strength:

1. **A positive control.** Before believing a zero, feed the instrument
   something it MUST find. `census_reason_codes.py`'s self-test now neutralizes
   its own parser and asserts the self-test goes red — without that, every
   other case is satisfied by a parser that returns nothing.
2. **Grade an empty parse as a breach.** A gate that reads a structure must
   assert it read one. *No table* and *a table I could not parse* are different
   facts and only the second is a defect — so distinguish them rather than
   passing on both.
3. **Re-derive by a route with a different failure mode.** A second grep shares
   the first's blind spots; a compiler does not. Renaming a function and reading
   `E0425` enumerates its callers in a way no text search can.

⛔ **And read the label against the tool.** `git grep -c` counts lines,
`grep -o | wc -l` counts occurrences, `git ls-files 'a/*.md'` crosses `/`
because it is a pathspec. Each is correct; each answers a question slightly
different from the one the sentence around it claims.

## What it is not

⚠️ **Not an argument for distrusting gates.** Every instance above was repaired
by making the instrument say more, not by abandoning it. A gate whose reach is
asserted is worth more than one that is merely believed.

⚠️ **Not the same as a stale figure.** A number that was true at its commit and
published without one is
[`a-metric-scoped-to-one-record-ages-silently`](a-metric-scoped-to-one-record-ages-silently.md)
— the instrument was right and the scope went unstated. Here the instrument's
answer was never about what the sentence claimed. The two failed together in one
session and were nearly written up as one rule; they are not one rule.

See also:
[`a-control-that-passes-for-an-unrelated-reason`](a-control-that-passes-for-an-unrelated-reason.md),
[`an-absence-claim-is-a-census-over-the-corpus`](an-absence-claim-is-a-census-over-the-corpus.md)
— that one is a HUMAN's absence claim after a targeted search; this is the same
claim delegated to a tool whose reach nobody examined.
