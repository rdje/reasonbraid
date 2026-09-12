answers: how do I show a race is actually closed; why did a passing test suite still ship a race; what should a concurrency test assert; why is a green local run silent about a Linux-only race; why does my new ordering control pass against the unrepaired code; which lock mode should this operation take?

# Proving a race is closed

- **Type:** `knowledge`
- **Date:** `2026-09-12`
- **Owner / source:** leaves `SIGNOFF-REPAIR.11.4.3.1.2.17`, `.11.4.3.1.2.25`, `.11.4.3.1.2.26`, `.3.3.4.9`, `.3.3.4.10.1`

## The question

A repair claims a race is closed and the suite is green. What has that actually
established?

## The answer

**Nothing, on its own.** A race is a claim about what CANNOT happen. A suite of
passing scenarios samples what DID happen on one run, on one platform, at one
timing. The two are not the same kind of statement, and the gap is where races
ship.

Assert the INVARIANT that makes the race impossible, as its own control:

- Not "the spawn succeeded" but "holding either stub path implies every stub is
  already written" — the property that leaves no window for a `fork` to inherit
  a write descriptor.
- Not "the fixture ran" but "an occupied name is refused rather than adopted".

A good invariant control has a specific signature: it **fails against the
superseded design while every scenario still passes.** If it passes against the
old code too, it is not testing the repair.

## Why the platform makes this worse

The development platform is routinely silent about the rule being violated:
macOS does not enforce `ETXTBSY` at any timing, so a local run cannot produce
the failure however many times it is repeated. A check-then-act race went
further and was *invisible* locally, because `target/` stays warm between runs
and the racing branch is dead code on the second run onward.

In both cases the invariant is checkable locally even though the symptom is not.
Write the test against the CAUSE, not the effect.

## The fence has to be one the old code did not already have

There is a specific way to get this wrong when the repair is "put the operation
under a lock", and `SIGNOFF-REPAIR.3.3.4.9` walked into it: **the fixture held
the wrong lock mode.**

The control held the tenant's guard EXCLUSIVELY, issued the request, and
asserted it waited. It was green. It was also green against the unrepaired code
— because the superseded shape admitted through a SHARED acquisition, and shared
waits behind exclusive too. The fixture fenced both designs equally and could
not tell them apart, while looking exactly like a passing race control.

The discriminating fence was the SHARED holder: the repair's exclusive
acquisition must wait behind it, and the old shape's shared acquisition walks
straight through. Swapping one word in the fixture turned a green control into
`left: 200, right: 403` — the operation applying after the caller's authority had
already ended.

The rule that generalises: when a repair changes WHICH lock an operation takes,
the fixture must hold a lock the old mode was NOT excluded by. Ask what the
superseded code acquired before choosing the holder's mode, rather than copying
the holder from the previous leaf — the previous leaf was repairing a path whose
old shape may have differed.

## Sometimes NO fixture can discriminate, and that has to be said

Take the rule above one step further. If the repair puts an operation under the
SAME lock mode the superseded code already took, then neither holder mode
distinguishes them: exclusive fences both, shared fences neither. No concurrency
fixture can falsify that repair, because concurrency is not what changed.

When that happens, find what did change — usually atomicity: the mutation and its
evidence now share one commit, so making the evidence unwritable must roll the
mutation back, and on the old shape it does not. That control discriminates, and
the ordering control beside it is a REGRESSION control. Label it as one. A suite
that presents a control which cannot fail as proof of a repair is worse than one
that omits it.

## Deriving the mode is part of the repair

The same leaf sequence produced three different correct answers — exclusive for
one family, shared for the next — so the mode is not a house style to inherit.
Derive it from two questions:

1. Is there a classify-then-write over a row that **may not exist**? A row lock
   cannot cover an absent row, so only tenant-level exclusion makes that exact.
2. Does anything this operation must be ordered against hold the SHARED guard?
   If so, exclusion is what buys the ordering.

If both answers are no, shared is correct and exclusive is a cost with no
invariant behind it — it blocks every concurrent operation in the tenant to buy
nothing.

## Re-verify

Restore the superseded implementation, run the control, and confirm it fails
while the ordinary scenarios still pass. Then restore the repair and confirm all
pass.

Related: `TOOLBOX.md`, "Remote-only is a hypothesis, not a category";
`docs/knowledge/proving-a-path-still-names-what-you-created.md`.
