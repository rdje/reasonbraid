answers: my test run stopped early — what does the passing part prove; is a failure I saw once flaky; how do I read a runner's exit code honestly; why did "10 suites ok" mislead me; what does a fail-fast runner hide; how do I decide whether a failing control or the product is wrong?

# An aborted run is not a partial pass

- **Type:** `knowledge`
- **Date:** `2026-09-15`
- **Owner / source:** leaf `SIGNOFF-REPAIR.11.4.7.2` (the G1 unit-baseline re-derivation); the failure it found is owned by `.11.4.7.2.4`

## The question

A test run ends early with a failure. The output says ten suites passed and
eighty tests passed. What has been established?

## The answer

**About the crates the runner never reached: nothing.** `cargo test` — like most
runners in their default mode — stops at the first failing *test binary*. The
counts it printed describe the binaries that ran before the stop, not a sample of
the whole.

⛔ **"10 suites ok, 80 tests passed" and a failure is not 90% good news. It is one
failure and four crates in an UNKNOWN state**, and the difference matters because
an unknown reads exactly like a pass to anyone who scans the summary instead of
the exit code.

```bash
# what actually happened: rc=101, aborted at the 11th test binary
cargo test --all --locked; echo "rc=$?"

# what the run was silently NOT telling me
grep -c "^     Running " run.log     # binaries reached
ls crates/*/tests/*.rs | wc -l       # binaries that exist
```

⭐ **The instrument for this is the exit code, and it is the one thing a summary
line cannot fake.** A run that prints reassuring counts and returns non-zero has
told you it is incomplete; only the counts are quotable, and only for what ran.

⚠️ If the whole picture matters — and for any claim of the form "the suites are
green" it does — the run needs `--no-fail-fast`, priced against its cost. Fail-fast
is right for a tight edit loop and wrong for a claim about a workspace.

⭐ **Priced, on the run that produced this note** (`SIGNOFF-REPAIR.11.4.7.2.2`):

| | binaries reached | result | test execution |
| --- | --- | --- | --- |
| default (fail-fast) | **11** | 10 suites ok, 80 tests, 4 crates UNKNOWN | — |
| `--no-fail-fast` | **94** | 102 suites ok / 1 failed, **815 passed / 1 failed** | **319.3 s** |

⛔ The cost of knowing was **not** a pile of new failures — there was exactly ONE
failure in the whole workspace, the same one that had been hiding the other four
crates. The cost was run time, on a run that already took minutes.

## Was it flaky? Two load states before that word

A failure observed **once** is a joint claim about the code *and* the machine it
ran on. The first observation here came from a run that shared the machine with an
unrelated project's compile — a real confound, and enough to make "flaky browser
test" feel like the obvious reading.

⭐ **Re-run it isolated before you use the word.** Quiet machine, single-threaded,
same target:

| Run | Machine | Result |
| --- | --- | --- |
| 1 | shared with another project's build | FAILED, `elapsed_ms` 41170 |
| 2 | quiet, `--test-threads=1` | FAILED, `elapsed_ms` 40022 |

One run licenses either conclusion. Two load states with one result licenses
exactly one. ⛔ "Flaky" is a *measurement*, not an impression, and reaching for it
early is how a real defect gets a label that stops anyone looking again.

## When the control and the product disagree

The failing assertion was `cleanup_confirmed == true`. The assertion immediately
above it — the render's own outcome — **passed**, which was the tell: an earlier
repair had deliberately split those two facts apart so the product would stop
claiming a termination it had not observed. The product was now answering
honestly, and the control was failing *on the honest answer*.

⭐ **A control can assert exactly what a later repair stopped claiming, and the
result looks like a regression.** Check whether the neighbouring assertions pass:
if the ones about the subject hold and only the one about the *previously
overstated* fact fails, the disagreement is between the control and a repair, not
between the code and reality.

⛔ **Do not settle such a disagreement by loosening the assertion to accept either
value.** That converts a control into a decoration: it will pass whatever happens,
including the defect it was written to catch. Decide which side is wrong and say
so, keeping whatever proof the control was carrying.

## The same shape, one level up

⚠️ A gate nothing runs produces no exit code at all, which is the limiting case of
this whole note. Two supply-chain gates in this project were red for days because
they lived only in a CI workflow that had never executed — nobody was ignoring a
failure, because nothing was generating one.

Related: `a-falsification-you-can-leave-behind`,
`a-self-test-cannot-be-tidier-than-the-real-input`,
`a-census-is-an-instrument-not-a-table`.
