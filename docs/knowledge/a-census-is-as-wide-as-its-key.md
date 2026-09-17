answers: my census looked complete and missed a surface — why; how do I know an enumeration covered everything that can reach this object; why did a route census miss an endpoint; how wide is a grep over route definitions; how do I scope a security census so it cannot be silently incomplete

# A census is only as wide as the key it enumerates on

- **Type:** `knowledge`
- **Date:** `2026-09-17`
- **Owner / source:** leaf `SIGNOFF-REPAIR.11.14.3.6`, correcting `SIGNOFF-REPAIR.11.14.3.4`

## The question

You have bound every surface that reaches a protected object, and you published
the command that enumerated them. How do you know the enumeration was complete?

## The answer

> An enumeration is complete **with respect to its key**, and no wider. Write the
> key down, then ask whether the thing you are protecting can be named some other
> way.

The measured instance. A leaf bound the routes that read a §12.1 reference, on
this census:

```bash
grep -n '"/v1/resources' crates/reasonbraid-server/src/api.rs   # -> 3 routes
```

Three routes, two unbound, both bound. One commit later a different leaf walked
past `POST /v1/snapshots`, which names a `reference_id` **in its request body**.
A route-prefix census cannot see it — the surface is not under that prefix.

⭐ **The census was not wrong. It was silent.** It was correct about every route
it enumerated and quiet about a surface that names the same object under a
different key. That is the more dangerous failure: a wrong census invites a
second look, and a silent one reads as complete.

## The rule

> Enumerate on the **identifier**, not on the address shape.

An object with an identifier is reachable by anything that accepts that
identifier: a path segment, a body field, a query parameter, a column some other
writer fills. So the census that bounds a protected object is the one over its
id, and the route census is a refinement of it:

```bash
grep -rn "resource_id\|reference_id" crates/…/src   # the object — the census
grep -n  '"/v1/resources'            crates/…/src   # one of its addresses
```

⛔ Publish the key beside the count. *"Three routes"* is a true statement about
routes and not about reachability, and a reader cannot tell which claim was meant
unless the command is there.

## Three instances, and the third is the one that hurts

⚠️ This shape has now appeared three times in this repository.

| | The census | What it could not see |
| --- | --- | --- |
| `.3.5.3` | the node-administration **mutations** | the inbox **read**, which selected by node id alone |
| `.11.14.3.8` | the **`snapshot_id` surfaces**, counted from `Path(…)` extractors and gate call sites | `POST /v1/derivations`, which names its parent in the **body** — and which was **unbound** |
| `.11.14.3.4` | the **routes under a prefix** | `POST /v1/snapshots`, which names the object in its **body** |

Each was correct about its own scope and said so. None was careless. What they
share is that the scoping key was chosen for convenience — it is what the grep
could express — and then not re-examined against the object being protected.

⛔ **The third row is the expensive one, and it is the reason this section is
written in this order.** `.11.14.3.8` came FIRST chronologically and published
*"seven surfaces, six bound, exactly one not"*. This note was written from the
second instance — and **never applied backwards to the first**. The false number
stood for seven commits, in five documents, with a live unbound write behind it,
and was found only when the maintainer asked whether the findings held.

> ⭐ **A rule earned from one instance is worth almost nothing until it is run
> over the instances that came before it.** Write the note, then immediately
> re-run its check against every census the project has already published. That
> sweep is minutes; the alternative is a number that is false for as long as
> nobody asks.

## What makes this checkable rather than a resolution to be careful

🔎 The scoping key is already a **written artefact**: it is the command in the
leaf. Reading it back and asking *"what else could name this object?"* costs one
sentence — and it is the sentence neither census contained.

## Related

- [[an-absence-claim-is-a-census-over-the-corpus]] — the same discipline aimed at
  a specification rather than at code.
- [[a-census-is-an-instrument-not-a-table]] — once the key is right, the census
  belongs in something that reproduces its own baseline.
- [[a-pin-a-second-path-can-bypass-is-not-a-pin]] — the runtime twin: a control
  that can be routed around, found by the same question.
