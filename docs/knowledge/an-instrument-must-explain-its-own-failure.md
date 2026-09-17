answers: why did my check pass when the thing it checks was broken; how do I choose exclusions for a find or grep census; should I pipe a gate through tail or head; my CI step failed and I cannot tell why; what makes a negative result trustworthy; how do I know my measurement command is the right one

# An instrument must explain its own failure

- **Type:** `knowledge`
- **Date:** `2026-09-17`
- **Owner / source:** leaf `SIGNOFF-REPAIR.11.14.3.3`, where both instances below happened in one session

## The question

You run a command to establish something — a census that returns nothing, a gate
that returns non-zero. How do you know the command was capable of telling you
what you needed?

## The answer

> Two failure modes, one cause. A check can be **unable to go red** when the
> answer is bad, or **unable to say why** when it does go red. Both are the same
> defect: the instrument discards the evidence its own conclusion depends on.

### Shape 1 — the exclusion that removes the answer

A census over a tree almost always carries an exclusion: skip the build
directory, skip vendored code, skip generated output. ⛔ **Ask what lives in the
path you excluded before you trust the empty result.**

Measured instance: `find . -name PG_VERSION -not -path "./target/*"` returned
nothing, and was published as "no database cluster exists anywhere in the
repository". The test runner destroys its cluster on success and **retains it
under `target/` as failure evidence** — so the exclusion pointed exactly at the
only place the answer could live. Without it: 32 hits, three retained clusters,
one of them the same session's own failed run.

⭐ The general test: **for a census that returned nothing, name the place a
positive result would have been, and confirm your command looked there.** If the
exclusion and the expected location coincide, the empty result carries no
information.

### Shape 2 — the truncation that removes the diagnosis

Piping a gate through `tail -N`, `head -N` or a narrow `grep` is convenient while
it passes and useless the moment it fails: compilers and test runners interleave
parallel progress lines with diagnostics, so the last N lines are frequently
progress and the error is in the middle.

Measured instance: a strict lint run reported exit 1 through `| tail -8`, and the
eight captured lines were all dependency progress. The cause was unrecoverable
from the log — the run had to be repeated in full to learn that it reported
**zero** diagnostics the second time. An hour of wall-clock to re-derive
something the first run already knew and threw away.

## The rule

> Capture a gate's output in full to a file, then filter the file. Filter the
> **artifact**, never the **stream**.

```bash
cmd > run.log 2>&1; echo "exit=$?"     # keep everything
grep -nE '^(error|warning)' -A15 run.log   # then narrow, repeatably
```

The file costs nothing, survives the process, and can be re-questioned with a
different filter when the first one turns out to be the wrong question — which is
the whole point, because you do not know which filter you need until it fails.

⚠️ **And report an unexplained failure as unexplained.** A gate that failed once,
passed on re-run, and left no evidence is not "flaky" and not "fixed" — those are
both causes, and neither was established. Write down what was observed, what was
ruled out, and that no cause is claimed. Inventing a cause is worse than the
missing log, because the next reader stops looking.

## Related

- [[a-census-is-an-instrument-not-a-table]] — the positive form: a census belongs
  in a tracked script that reproduces its own baseline.
- [[an-absence-claim-is-a-census-over-the-corpus]] — "nothing matches" is a claim
  about a corpus, and it carries the command that enumerated it.
- [[a-falsification-you-can-leave-behind]] — a control never seen red is not known
  to work; this is the same doubt aimed at measurement rather than at tests.
