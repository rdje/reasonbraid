answers: my check's self-test is green but the check is wrong — how; what should a self-test's fixtures be made of; why did my gate fail a perfectly valid input; why did a truncated file pass a gate that greps for the right phrases; how do I choose sentinel phrases for a document; how do I keep a hole I just fixed from reopening?

# A self-test cannot be tidier than the real input

- **Type:** `knowledge`
- **Date:** `2026-09-15`
- **Owner / source:** leaf `SIGNOFF-REPAIR.13.2` (`scripts/check_licence_grant.sh`); the same shape as `SIGNOFF-REPAIR.11.4.3.1.7.1`, where a census asserted a probe name appeared in zero tracked files and wrote that name into its own source

## The question

A check ships with a two-sided self-test. The self-test is green. Is the check
working?

## The answer

**Not necessarily, and the self-test cannot tell you — if its fixtures were
written by the same hand, at the same sitting, as the check.** A hand-written
fixture is the author's *mental model* of the input rendered as a string. The
check is the same mental model rendered as code. Testing one against the other
tests the model against itself, and it passes exactly when the model is
self-consistent, which is not the property anyone wanted.

⭐ **The fix is one line of policy: fixtures are the real artifacts, mutated.**
Read the file the gate will really see, then apply each mutation to *that*.

```python
# ⛔ fixtures invented to match the check
MIT_OK = 'Permission is hereby granted... THE SOFTWARE IS PROVIDED "AS IS"...'

# ✅ the shipped file, mutated — the gate cannot be right about a document
#    tidier than the one it will meet
real = pathlib.Path("LICENSE-MIT").read_text()
cases = [("healthy", real, False),
         ("truncated", head_lines(real, 3), True)]
```

## What this cost, measured

`LICENCE-GRANT` was built to assert that the licence texts a manifest declares
actually exist and actually read like those licences. Its self-test passed on the
first run. Two defects were nonetheless live, and **the self-test was green
through both**:

| Defect | Why the fixtures hid it |
| --- | --- |
| Sentinels were literal substrings, and the real MIT text is hard-wrapped at ~55 columns, so `WITHOUT WARRANTY OF ANY KIND` spans a line break — the gate **failed a perfectly valid licence** | The fixture prose was written on single long lines. Nobody hand-wraps a fixture at 55 columns |
| All four Apache sentinels sat in the **first five lines**, so `head -5 LICENSE-APACHE` passed the gate while granting nothing | The fixture *was* five lines. A five-line fixture cannot express the difference between a document and its title block |

⚠️ Both were found by **falsifying the gate against the working tree** — mutate
the real file, run the gate, restore, re-compare — which is the practice
`a-falsification-you-can-leave-behind` describes. The self-test is a unit check on
the classifier; falsification is the only thing that asks whether the classifier
is pointed at reality.

## Two rules that fall out of it

**1. Sentinels must span the document.** A phrase check on a structured text
document is really a sampling problem, and samples drawn from one region measure
one region. Take them from the opening, the numbered body, *and* the closing
line, and back them with a length floor. ⛔ The title block of a document is not
the document — and the title block is exactly where an author's eye lands when
picking "distinctive" phrases.

**2. A hole you fixed becomes a pinned case, named as such.** Both failures above
are now self-test cases carrying a comment that says deleting them restores the
hole. A regression test that does not say what it is protecting gets deleted by
the next person tidying up, because from the inside it looks like a duplicate.

```python
# ⛔ PINNED: every Apache sentinel of the first draft lived in these five
# lines, so this stub PASSED. Deleting this case restores that hole.
("Apache text truncated to its five-line title block",
 DUAL, {**BOTH, "LICENSE-APACHE": head_lines(APACHE_OK, 5)}, True),
```

## The general shape

⭐ **A control's fixtures are part of its threat model.** Ask of every fixture:
*which properties of the real input did I simplify away, and does the control
depend on any of them?* Wrapping, length, encoding, ordering, duplication and
whitespace are the usual casualties, and each one is a place where a green
control and a broken control look identical.

Related: `a-falsification-you-can-leave-behind`,
`a-control-is-calibrated-against-the-renderer`, `a-census-is-an-instrument-not-a-table`.
