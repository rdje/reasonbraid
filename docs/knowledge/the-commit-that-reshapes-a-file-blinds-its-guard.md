answers: my linter/census stopped working and nobody noticed; why did a passing self-test not catch a broken tool; what breaks when I rename a heading or restructure a document; how do I test a tool that reads a real file; a check has been green for months — is it actually running

# The commit that RESHAPES a file is the commit that blinds its guard

- **Type:** `knowledge`
- **Date:** `2026-09-18`
- **Owner / source:** leaf `SIGNOFF-REPAIR.11.20`

## The question

A tool reads a document's structure — a heading, a bullet name, a section marker.
Someone restructures that document, correctly. What happens to the tool, and who
finds out?

## The answer

> It stops working, silently, and **its self-test keeps passing** — because the
> fixtures carry the old shape. The blind spot was not introduced by the tool's
> author; it was introduced later, by a commit to a different file. No amount of
> care at authoring time covers that.

⭐ The pairing is the lesson, not the rename. A structural edit to a document is
*exactly* when its readers break, and *exactly* when nobody is looking at them:

- the edit's own acceptance is about the document, not about its readers,
- its diff does not mention the tool,
- the tool's self-test keeps passing, so every gate stays green.

## The measured instance

A census existed to stop a bounded file reaching its byte cap. It keyed on that
file's next-action bullet by literal name.

A later commit conformed the file to the template that governs it — a correct,
well-reviewed change — and the template spells that bullet differently. From that
commit the census **exited 1 on every run**. Nothing noticed for dozens of commits,
because nothing runs a census on a schedule.

Meanwhile:

- its `--self-test` reported **16 controls pass** throughout, every one a fixture
  carrying the old bullet name;
- the gate that exists to catch instruments which have silently stopped working was
  **green**, because it asks "does this script's self-test pass", never "does this
  census still run against the corpus it is about";
- and **the defect the census exists to prevent recurred** — the file went back to
  85% of its cap, which is how the dead instrument was finally found.

## The remedy: one arm, at the end of the self-test

```python
try:
    locate((ROOT / "THE_REAL_FILE.md").read_text())   # the real corpus, not a fixture
except SystemExit as exc:
    failures.append(f"live-corpus: the real corpus is unreadable to this census ({exc})")
```

⛔ **Assert almost nothing about the CONTENT.** Only that the instrument can still
find what it is about. An arm coupled to the live file's wording fails on every
honest edit and gets waived within a week — and a waived control is worse than no
control, because it still reads as coverage.

⚠️ A whole separate gate — *every census must still run against its corpus* — was
measured and **declined on cost**: 3.9 s against an 11.6 s enforcer, for a property
the one-line arm above proves for free inside a self-test that already runs. The
honest limit is that nothing then *requires* the arm to exist.

## The two questions that catch it at the rename

Both are cheap, and neither was asked:

1. **What reads this file's structure?** (`git grep` the heading you are changing.)
2. **Did I run it?** — not its self-test. It.

## Related

- [[a-census-is-an-instrument-not-a-table]] — why the census exists as a tracked
  producer at all, which is what makes "did I run it" answerable.
- [[an-instrument-must-explain-its-own-failure]] — this census did refuse loudly
  and with a good message; refusing well is not enough if nobody invokes it.
- [[a-restated-number-needs-a-producer]] — the same session's sibling: the file's
  regrowth was itself a mirror, restating what a durable layer already held.
- [[a-key-too-loose-returns-the-wrong-instance]] — the other way a literal key
  fails its reader.
