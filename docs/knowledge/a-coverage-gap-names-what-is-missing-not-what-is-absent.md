answers: my census says this surface is undocumented but I am sure we wrote about it; should I write a new section for every route a coverage gate flags; why did documenting sixteen endpoints take four edits; how do I tell an explanation gap from an addressing gap; what does a coverage census actually measure

# A coverage gap names what is missing, not what is absent

- **Type:** `knowledge`
- **Date:** `2026-09-21`
- **Owner / source:** leaf `SIGNOFF-REPAIR.11.4.6.6`, which was scoped to write up
  sixteen undocumented operator routes and found fourteen of them already
  explained.

## The question

A coverage census reports that sixteen endpoints are undocumented. The obvious
reading is *sixteen things nobody has written about*, and the obvious plan is
sixteen new sections.

Is that what the census said?

## The answer

> **No. It said the book does not NAME them.** Whether it explains them is a
> different question, and the census cannot ask it.

A coverage instrument matches an address — a path, a symbol, a flag. So it
answers *is this addressable in the corpus?* A reader who already knows the
concept and wants the endpoint is asking the same question, which is why the
measure is a good one. But it is silent on whether the concept is explained, and
those two failures need opposite repairs:

- **an explanation gap** — nothing tells the reader what this does. The repair is
  to write it.
- **an addressing gap** — the prose is there and correct, and never writes the
  address. The repair is one line.

*Measured instance:* sixteen operator routes read as gaps. Fourteen already had
sections — grant and boundary revocation, the spend breaker, node enrolment
tokens, certificate revocation, the three inbox verbs, the federation
directions — each explaining the verb in full and none writing its path beside a
method. One table listed eight routes as bare suffixes (`grants`, `usage`)
rather than paths. One section elided two of three verbs as `…/accept` and
`…/revoke`. Adding contract lines to seven existing sections moved the census
from **81 described to 95**; only two routes in the whole set needed prose.

## ⭐ Check the subject before writing the section

Before treating a flagged item as unwritten, search the corpus for its
**concept**, not its address: the verb's name, the noun it acts on, the section
title someone would have given it. The instrument already told you the address is
absent; that tells you nothing about the concept.

> **The cost of getting this wrong is not wasted effort — it is a second,
> divergent explanation of the same thing**, which is worse than the gap, because
> now two passages describe one verb and only one of them will be maintained.

## ⛔ This is not a reason to weaken the measure

The tempting conclusion is that the census over-reports and should also count a
passing mention, or a nearby heading. It should not. Those fourteen routes were
genuinely unfindable: a reader searching the book for `/v1/admin/usage` got
nothing, and so did every tool. **The census was right about what it measured.**
The error was in the reader of the census, who took *not named* to mean *not
explained* and sized the work from that.

## Restated outside software

A library that holds a book but has never catalogued it has a real problem, and
"we own a copy" is not the answer to "I cannot find it". But the fix is a
catalogue card, not buying the book again — and buying it again leaves two copies
with different bindings, only one of which anyone updates.

## Related

- [[a-census-is-as-wide-as-its-key]] — the instrument-side sibling: what a key
  matches bounds what its count can mean.
- [[writing-the-documentation-is-a-verification-pass]] — the opposite direction:
  writing a section that really is missing keeps finding defects.
- [[a-sample-is-not-a-traversal]] — the same discipline applied to the
  conclusion rather than the corpus: say what you measured, not what it suggests.
- [[a-restated-number-needs-a-producer]] — why the census must be re-run rather
  than quoted once the repair lands.
