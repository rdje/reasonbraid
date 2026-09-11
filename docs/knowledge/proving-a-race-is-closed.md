answers: how do I show a race is actually closed; why did a passing test suite still ship a race; what should a concurrency test assert; why is a green local run silent about a Linux-only race?

# Proving a race is closed

- **Type:** `knowledge`
- **Date:** `2026-09-12`
- **Owner / source:** leaves `SIGNOFF-REPAIR.11.4.3.1.2.17`, `.11.4.3.1.2.25`, `.11.4.3.1.2.26`

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

## Re-verify

Restore the superseded implementation, run the control, and confirm it fails
while the ordinary scenarios still pass. Then restore the repair and confirm all
pass.

Related: `TOOLBOX.md`, "Remote-only is a hypothesis, not a category";
`docs/knowledge/proving-a-path-still-names-what-you-created.md`.
