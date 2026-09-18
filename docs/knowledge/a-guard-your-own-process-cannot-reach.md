answers: my self-test probes my own pid and passes — what have I missed; how do I test a permission-denied branch; why did flipping EPERM to "absent" leave my tests green; how do I build a fixture for a process I do not own; how do I test code whose subject is a relationship between me and something else; what is wrong with using os.getpid() in a liveness test; how do I cover a branch my test fixtures structurally cannot enter

# A guard your own process cannot reach

- **Type:** `knowledge`
- **Date:** `2026-09-18`
- **Owner / source:** leaf `SIGNOFF-REPAIR.7.3.2.1`, which shipped an instrument
  that deletes ~1.8 GB and found, by mutating its own source afterwards, that the
  single branch deciding *"is this live process someone else's?"* was exercised by
  no arm at all.

## The question

Your code asks a question about the relationship between your process and another
one — does that pid exist, may I signal it, do I own that file, can I read that
directory. You write a control. The natural fixture is the process you have:
`os.getpid()`, `os.getpgrp()`, a file you just created.

It passes. What have you not tested?

## The answer

> **The other side of the relationship — which is usually the side that matters.**
> A fixture built from yourself can only ever exercise the branch for *yourself*.
> The branch for *somebody else* is unreachable by construction, and no amount of
> running that control will ever enter it.

The measured instance. A retirement instrument must never touch a fixture whose
browser is still alive, so it probes each recorded id with signal 0:

```python
try:
    os.killpg(ident, 0)
except ProcessLookupError:
    return False          # ESRCH: no such group — absent
except PermissionError:
    return True           # EPERM: it exists and is SOMEBODY ELSE'S — present
return True
```

Nine refusal arms covered this function. Every one of them used `os.getpid()` or
`os.getpgrp()`, so `signal 0` **succeeded** and fell to the last line. Mutating
`except PermissionError: return True` into `return False` — a change that lets the
instrument delete a live fixture owned by another user — left the whole self-test
green.

And EPERM is not the obscure case. It is the case the guard exists for: a process
you cannot signal is precisely a process that is not yours to clean up.

## What to do instead

Two arms, because they answer different questions and the cheap one must always run:

1. **Inject the condition.** Replace the syscall with one that raises, and assert
   the branch's verdict. This is hermetic, runs on every machine, and cannot be
   skipped:

   ```python
   real_kill = os.kill
   try:
       os.kill = lambda pid, sig: (_ for _ in ()).throw(PermissionError())
       assert id_in_use("pid", 424242) is True
       os.kill = lambda pid, sig: (_ for _ in ()).throw(ProcessLookupError())
       assert id_in_use("pid", 424242) is False
   finally:
       os.kill = real_kill
   ```

2. **Then find a real one, and skip LOUDLY if you cannot.** An injected arm proves
   the branch's logic; it does not prove the syscall raises what you believe. On a
   Unix box, `ps -axo pid=,uid=` yields a root-owned pid an unprivileged caller
   cannot signal. Print the skip when the machine offers none — a silent skip is
   how an untested branch looks exactly like a tested one.

## How to notice it before a mutant does

Ask of each arm: **which branch does this enter?** Then compare that list against
the branches in the function. The gap is your answer. A function with an
`except PermissionError` and no arm that raises one has a hole, and you can see it
by reading, once you are looking for coverage of BRANCHES rather than of behaviour.

The generalisation past processes: whenever the subject is **ownership,
permission, identity or trust**, the fixture you reach for first is the one you
control, and it sits on the permissive side of every check. Uid, gid, file mode,
a lock another process holds, a port another process bound, a row another tenant
owns, a token issued to somebody else — all the same shape.

## Related

- [[a-control-that-passes-for-an-unrelated-reason]] — the parent rule: a green
  control is consistent with *the defect is absent* and with *this control cannot
  see the defect*. This note names one structural reason for the second.
- [[a-scoping-defect-errs-in-one-direction]] — which way the mistake costs you. Here,
  reading EPERM as presence errs toward keeping a fixture; reading it as absence
  errs toward deleting live evidence.
- [[a-falsification-you-can-leave-behind]] — mutation is what found this; the arms
  above are what it left behind.
