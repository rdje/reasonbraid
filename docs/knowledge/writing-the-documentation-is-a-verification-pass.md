answers: is it worth documenting a surface I already tested; why did writing the docs find bugs the tests missed; how should I check a claim before publishing it; what is a cheap way to find the behaviours I never exercised; when is a documentation task not just a documentation task

# Writing the documentation is a verification pass

- **Type:** `knowledge`
- **Date:** `2026-09-13`
- **Owner / source:** leaf `SIGNOFF-REPAIR.3.3.4.11.4`, which produced two defects and three corrections from four claims it checked this way

## The question

A surface has tests, and they pass. Writing its chapter looks like transcription
— work that can only restate what is already known. Why does it keep finding
things?

## The answer

Because implementing and documenting ask different questions.

> Implementing asks **"does my path work?"**
> Documenting asks **"what happens for each thing a reader might do?"**

The first question is answered by the cases you thought of, which are the cases
you built and then tested. The second forces an enumeration: every route, every
gate, every refusal, and — hardest — every sentence in the "what this does not do
yet" section, which cannot be written without knowing what actually happens at
each edge.

That enumeration is a test-design activity wearing different clothes, and it runs
over the surface's *whole* contract rather than over the diff you just wrote.

## The rule

> **A sentence you are about to publish is a hypothesis. Run it.**

Not "check the code says so" — run the request and read the answer. The failure
mode this catches is specifically the confident claim: you wrote the handler, you
remember what it does, and the memory is of the path you exercised.

At `SIGNOFF-REPAIR.3.3.4.11.4`, four drafted claims were checked by running them
and three were wrong:

| Drafted claim | What running it returned |
| --- | --- |
| "importing the same card twice creates a second local role" | `200`, then **`500`**, recording nothing — a real defect, repaired under its own leaf |
| "an unknown field is a typed `400`" | **`422`** — the body never deserialized, so the handler never ran |
| "`expires_at` … no route filters on it" | true but far too soft: the eligibility check never reads it, so an **expired claim still satisfies a requirement** |

## The other half: a documented claim needs a control

A chapter is not a control. If the only place a behaviour is asserted is prose,
it drifts the first time someone edits the code, silently, and the chapter starts
lying. So while enumerating, keep a second list: **which claim is pinned by which
test.**

Most will already be covered — trace them rather than duplicating them. The ones
that are not are the valuable output of the exercise: at `.11.4`, exactly one
documented claim (that only the owner class exports the portable card) had no
control anywhere, and the chapter would have been its only assertion.

⛔ One exception, and it matters: when the enumeration turns up a **defect**, the
control asserts the repaired behaviour, not the measured one. A test written
around a defect enshrines it. Document the gap, open the leaf, repair it, and let
the control assert what should happen.

## Re-verify

Take any surface with a chapter, pick three sentences that make a specific claim
about a status code or an answer, and run them. If all three hold, the chapter is
carrying its weight; if one does not, it was carrying a belief.

Related: `docs/knowledge/proving-a-path-still-names-what-you-created.md`;
`docs/knowledge/a-raised-constraint-cannot-be-a-recorded-refusal.md` (which the
`.11.4` enumeration led straight back to).
