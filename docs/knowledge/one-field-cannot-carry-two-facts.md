answers: why does my test pass on CI and fail on my machine; why is a control host-dependent; what does it mean when a field is sometimes overwritten; how do I report two outcomes from one operation; should a failed cleanup replace the error I already have?

# One field cannot carry two facts

- **Type:** `knowledge`
- **Date:** `2026-09-12`
- **Owner / source:** leaf `SIGNOFF-REPAIR.11.4.3.1.2.27`

## The question

A control asserts `result.kind == X`. It passes on one host and fails on
another, and nothing in the code under test is host-specific. Where do you look?

## The answer

**Look at whether that one field is written from more than one independent
fact.** If it is, every assertion about it is implicitly a conjunction over all
of them — including conditions the control's author never meant to assert.

The measured instance: a worker returned one `kind` for two outcomes that are
established separately — what the RENDER did, and what CLEANUP observed. When
cleanup could not be confirmed, it overwrote `kind` and kept the render's kind
only as a prefix inside the human message. So a control asserting the render's
kind actually asserted *"the render refused this way **and** the host was fast
enough to confirm cleanup."* The second clause was true on the runner and false
on a loaded development machine, and the control looked flaky.

It was not flaky. It was correct, and it was measuring a **product** defect: the
consumer had the same problem the control did. A caller reading `kind` to learn
why its own request failed received an operator's fact about a stray process
instead.

### The three moves

1. **Separate the facts in the type.** One field per fact, both always present.
   Uncertainty is a value you carry, not a label that replaces something else.
2. **Ask what each fact's AUDIENCE does with it.** The caller acts on its own
   budget and its own selector; the operator acts on a process that may still be
   running. A field that serves two audiences serves neither when they disagree.
3. **Keep the one corner where the replacement was doing real work.** Here, a
   SUCCESSFUL result under unconfirmed cleanup must still refuse — there is no
   failure of its own to name, and letting it read as complete would retire a
   workspace whose process may be live. Separating facts is not an excuse to
   soften a refusal that was load-bearing. Find that corner before you change
   anything.

### Do not widen the assertion instead

The tempting repair is to accept either value in the control. It is the wrong
one twice over: the control stops proving the thing it exists to prove, and the
product defect it was reporting survives untouched. A control that is
host-dependent is telling you something; widening it is turning off the message.

### Reproduce it without the host

A load-dependent trigger is not a control — it did not reproduce on a quiet
machine even once. Inject the SHAPE instead of waiting for the condition. Here
the real-world cause was a helper process that escaped the browser's process
group and held its inherited stderr open, so no EOF arrived; a ten-line script
that forks, calls `setsid` and sleeps reproduces that exactly, in twelve seconds,
on any host and with no browser at all. Then the slow host and the fast host are
both controls you run every time, instead of one you wait for.

## Related

- [[proving-a-race-is-closed]] — the same shape for concurrency: assert the
  invariant, not the scenario.
- `docs/decisions/2026-09-12_browse-refusal-carries-two-facts.md` — the decision
  this lesson came from.
