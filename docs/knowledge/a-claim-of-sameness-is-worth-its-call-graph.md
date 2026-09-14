answers: my module says it uses the same authorization as another surface — how do I check; should I add the missing checks to the copy or delete the copy; why did a second implementation drift from the first; how do I stop a doc comment from being the only thing holding an invariant; where should a second surface's authorization live; how do I repair a duplicated read path safely

# A claim of sameness is worth exactly the call graph that enforces it

- **Type:** `knowledge`
- **Date:** `2026-09-14`
- **Owner / source:** leaf `SIGNOFF-REPAIR.6.1.1`

## The question

A module's documentation says it runs another surface's checks — *"the same
queries and the same authorization as the HTTP handlers"*, *"this mirrors the
validation in X"*, *"the same gate the CLI applies"*. The code beneath it is a
separate implementation. What has that sentence established?

## The answer

**Nothing that a reader can check, and nothing the compiler is holding.** A
sentence asserting sameness across two implementations is a claim about a
relationship no tool is maintaining. It is right on the day it is typed and free
to be wrong from the next commit — and worse than silence, because the next
reader trusts it *instead of* reading the query.

Measured instance. An MCP tools crate opened with *"the READ tools over the
inspection verbs — the SAME queries + the SAME authorization as the HTTP
handlers"* and *"every read runs the reader classification"*, over a private
re-implementation of three reads. The copy ran neither:

- one tool declared a `principal` argument its body never read;
- one took its tenant as an underscore-prefixed **unused** parameter, the
  compiler having been told in the signature that the argument does nothing;
- one computed the correct foreign-reader class, then returned the full
  projection anyway and printed the class beside it.

Measured live, a principal in one tenant received another tenant's entire thread
projection — subject, objective, participants, creator, budget — labelled
`"visibility":"network"`.

## The repair is the call, not the checks

The obvious fix is to add the three missing checks next to the copy. It leaves
the mechanism entirely in place: a second implementation of an authorization is
free to omit it again, and now the sentence is true, so the next omission is
even harder to see.

Make the second surface **call** the first. Extract the shared halves — the
authorization, the query — so each exists once, and let the seam consume them.
Then *"the same authorization"* is a fact about the call graph, and the only way
to break it is a change the compiler sees.

⭐ The by-product is worth naming: once the seam calls the real read, it
inherits everything that read had accreted and the copy had not. In the measured
instance the tool gained a derived view (an expired invitation reads `expired`
rather than `invited`) and a view-backed select carrying the delivery state —
two divergences nobody had found, fixed by deletion rather than by discovery.

## Delete the copy; prove it dead with the compiler

⛔ A `git grep` returning no callers is a claim about a search pattern. Removing
the item and rebuilding every target is the compiler's verdict, and it has no
loyalty to the grep. (This is the same instrument `a-signature-is-a-promise-the-body-must-keep`
reaches for.) The deletion is also what converts the header from aspiration to
description: there is no longer a second path for the sentence to be wrong about.

## ⚠️ Two traps found while proving this one

**A fail-fast control is the wrong shape for a claim about a SET.** The claim
was *"all three read tools"*. The control asserted immediately and stopped at
the first breach, proving one tool red and saying nothing about the other two.
Accumulating each leg's verdict and asserting once at the end turned one red leg
into eight — the picture the claim actually required. A control should have the
arity of the sentence it defends.

**`git diff` cannot prove a change landed in a NEW file.** A falsification
harness neutralized one gate at a time and printed `git diff` as its evidence
that the neutralization had taken. The file was new and untracked, so the diff
was empty and the harness reported success while proving nothing. Use an
instrument that does not depend on the file's tracking state — a grep census of
the call sites, for instance — and check the run's own result against it.

## ⭐ Where the check goes, when two surfaces share a core

The same question arrives a second way: not "does this surface run the other's
check" but "which of them should hold it". When two surfaces already share a
core, the binding belongs in the **core**, and there is a clean giveaway for
when you have put it in the wrong place.

*Instance (reference deployment): a tool seam gated its caller against a tenant
the caller supplied, then handed a core a target id; the core fetched by that id
alone. The obvious repair is at the seam, where the finding was written. But the
HTTP verb over the same core took **no tenant at all** — so a seam-level repair
would have left that verb fully open **and the seam's own suite green**.*

⛔ **That is the test: if fixing it at the seam would leave a sibling caller
broken while every test you can see passes, the check is in the wrong place.**
Put it where the target's own identity is in scope — which is the same place
`inspect_call`-shaped code already looks — and both callers get it at once.

⚠️ A corollary about the finding you inherited: it names the surface where
someone happened to look. Before repairing at that surface, enumerate the
core's callers. Here the leaf said "the MCP write seam"; the census said "two
surfaces, one core", and the second one was worse.

Related: `docs/CLAIM_VERIFICATION.md` leg 2 (prefer an oracle you did not
build); `where-an-invariant-lives`; `a-signature-is-a-promise-the-body-must-keep`.
