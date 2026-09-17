answers: my falsification passed — does that prove the control works; how do I know my injected defect was actually applied; why did neutralising the fix change nothing; how should I restore a file after a falsification; what is the safest way to revert an injected change

# An injection must be shown to land

- **Type:** `knowledge`
- **Date:** `2026-09-17`
- **Owner / source:** leaf `SIGNOFF-REPAIR.11.14.1.2`

## The question

You falsify a control by breaking the thing it guards and re-running. The run
comes back the way you hoped. Is the control working?

## The answer

> Only if the break actually happened. **A green run after an injection is
> evidence only when the injection landed**, and a scripted edit that silently
> matches nothing is the commonest way for it not to.

The measured instance: a `.replace()` whose indentation did not match the target
file. Nothing was injected, the gate stayed green, and for a moment that read as
*the registration works*. It proved nothing at all.

## The rule

> Before reading a falsification's result, assert the stimulus:
>
> ```bash
> git diff --stat -- <path>   # no change reported = the falsification has not started
> ```

One command, and it converts "I think I broke it" into a fact. Anchor-based edits
are the usual culprit — indentation, a reflowed line, a formatter that ran since
you last looked.

⭐ The same discipline applies to the other direction: after restoring, assert
the restore. `git status --short -- <path>` printing nothing is the proof that
the tree is byte-identical again.

## Restore with the tool that cannot fail silently

⚠️ Copy a file aside and copy it back, and you have two chances to fail quietly:
the backup can fail to write, and the restore can fail to run because the backup
is missing. In the measured instance the backup went to a temporary directory the
environment refuses, so the restore never ran and the file was left injected.

> **`git checkout -- <path>` is the restore that needs no backup.** It cannot
> half-succeed, and its result is checkable with one `git status`.

## The same shape, one layer in and one layer out

This is a family, and recognising it is worth more than any one rule:

| Layer | The failure | What it looks like |
| --- | --- | --- |
| the **stimulus** | the injection never landed | a control that passes because nothing was broken |
| the **instrument** | the measurement cannot discriminate | a counter that reads the same with and without the repair |
| the **scope** | the census silently skipped what it could not read | a clean report over a subset nobody declared |

⛔ All three produce a control that passes for a reason unrelated to the thing it
claims to test — and all three are invisible unless you ask the one question
that separates them: *what would this have done if the defect were present?*

## Related

- [[a-falsification-you-can-leave-behind]] — making the falsification permanent
  once you have shown it discriminates.
- [[a-change-no-surface-can-see-needs-a-seam]] — the instrument-layer sibling: a
  measurement that reports the same value either way.
- [[a-census-is-as-wide-as-its-key]] — the scope-layer sibling.
- [[an-instrument-must-explain-its-own-failure]] — why an instrument should
  refuse rather than narrow itself in silence.
