answers: is this compatibility break worth taking; how do I check whether removing a permission breaks a real caller; a repair is held because it would break someone — how do I decide; how do I tell a real cost from a plausible one; why did a leaf sit unrepaired for days

# A compatibility objection is a claim, and claims get measured

- **Type:** `knowledge`
- **Date:** `2026-09-17`
- **Owner / source:** leaf `SIGNOFF-REPAIR.11.14.3.8`

## The question

A repair is obvious and a shipped surface changes behaviour if you take it. The
leaf records the reason to hesitate in the form users would state it:

> *"A principal legitimately assessing evidence another team acquired would
> start being refused."*

That sentence is the whole cost side of the decision. How do you price it?

## The answer

> An objection phrased in terms of the product is **a claim about the product**,
> and it is checkable against the product. Until someone checks it, it is doing
> the work of evidence while being an intuition.

The measured instance: the objection protected a caller assessing a snapshot its
tenant had never cited. One census — every surface in the codebase that names a
`snapshot_id`, and the gate each reaches — showed that **every read of that
snapshot already refused such a caller**: the row, its derivations, its
assessments, the staleness list, and the delete, all `404`.

So the protected workflow could not run. The un-gated write bought exactly one
thing: the ability to assert about bytes the caller cannot read, and to probe
them one substring at a time. The "cost" was zero and the leaf had been held on
it.

## The rule

> Before paying for a compatibility break, **enumerate what the caller you would
> be breaking can currently do other than the thing you are removing.** If the
> answer is nothing, the break is a correction, not a cost.

The enumeration is the same census the finding already needs — every surface
that reaches the same object, and the gate each one applies — so it costs one
command, not a new investigation.

## The mirror, which is why the rule is stated this way

⚠️ The census could have come out the other way. If one read had been open, the
objection would have been **sound**, and the disposition would have had to move
to something narrower than a gate.

⛔ So the rule is not *"objections are excuses"*. It is that an objection stated
in product terms is checkable, and leaving it unchecked is how a leaf sits: a
plausible cost with nobody assigned to price it reads exactly like a settled
one. The same shape appears wherever a decision is *held* rather than *open* —
a held question looks like a state, not like work still owed.

## A second thing the reproduction produced

The leaf described its finding as two distinguishable refusals — an existence
oracle. Driving it found **three** answers, and the third was a `200` confirming
that a chosen substring appears in bytes the caller had never read, plus a stored
row. That is a *content* probe rather than an *existence* one: a different
severity class from the one the leaf was filed under.

> ⭐ **A defect's class is a measurement, not an inheritance from the leaf that
> opened it.** Reproduce before you classify, even when the leaf that opened it
> already classified it — especially then, because the inherited label is what
> stops anyone looking.

## Related

- [[where-an-invariant-lives]] — having decided to gate, the gate belongs at the
  function both writers reach, not at each caller.
- [[trust-comes-from-the-check-not-the-shape]] — the same distinction on the
  other side: what made one identifier trustworthy was its membership check.
- [[a-re-export-is-not-a-caller]] — the sibling failure, where the *absence* of a
  caller was the thing nobody measured.
