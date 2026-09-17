answers: should two code paths mint identifiers the same way; is this inconsistency worth fixing; why does making two formats match make things worse; how do I decide between unifying and labelling two namespaces; a reviewer says my two paths are inconsistent — are they wrong

# Trust comes from the check, not the shape

- **Type:** `knowledge`
- **Date:** `2026-09-17`
- **Owner / source:** leaf `SIGNOFF-REPAIR.11.14.3.3`; the decision it came from is `docs/decisions/2026-09-17_the-assessment-namespace-is-part-of-the-row.md`

## The question

Two paths write the same column with the same *kind* of value, but they produce
it differently — one derives it, one takes it from the caller. It reads as an
inconsistency, and the obvious repair is to make both paths derive it. Is that
right?

## The answer

> Ask what makes the trustworthy one trustworthy. It is almost never the
> **format**; it is the **check** performed before the value is accepted.

The measured instance: a deliberation's `claim_id` was a SHA-256 digest the
server computed, and a standalone route's `claim_id` was a caller's label. Making
the standalone route hash its input too was the symmetric-looking repair. It is
the wrong one, and the reason generalizes:

- The digest was not what made the deliberation's identifier trustworthy —
  `claim_exists_in_thread` was. The server accepted the digest only after
  confirming it named a claim **of that thread**.
- Hashing is the shape. The membership check is the property.
- A standalone route has no thread to check membership against. A digest there
  would be a hash of the caller's own string: exactly as invented as the label it
  replaced, but now **indistinguishable from a trustworthy one by construction**.

⭐ So the "consistency" repair would have deleted the last signal a reader had
and called the result uniform.

## The rule

> When two paths produce the same kind of identifier, name the act that makes one
> of them trustworthy, then ask whether the other path can perform that same act.
> If it cannot, the asymmetry is **real** — record it, do not erase it.

Recording it means a server-set discriminant on the row saying which path minted
the value. That is a smaller change than unification and it preserves the
information unification destroys.

### Why the wrong answer is attractive

⚠️ Worth naming, because a reviewer will ask for it. "Why does one route mint and
the other not?" sounds like a defect report. The honest reply is that one path
runs inside a deliberation and the other does not, and that difference is the
thing the column should carry.

### The third option, also wrong, also reasonable-looking

⛔ Having labelled the rows, it is tempting to *filter* the read down to the
trustworthy namespace. That yields a route which writes rows nothing can read —
a shape nobody would choose deliberately, reached by a sequence of individually
sensible steps. If a route's rows should not be readable, remove the route and
say so; do not arrive there by narrowing a read.

## Related

- [[a-replay-key-is-an-addressing-scheme]] — the other half of this leaf: why the
  discriminant belongs inside the key rather than beside it.
- [[a-grouping-is-not-an-argument]] — treating two things as one kind because they
  are adjacent is the same error in a different dress.
- [[a-repair-owns-every-sentence-that-states-its-limit]] — the asymmetry you keep
  has to be published where readers meet it.
