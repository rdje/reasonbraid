answers: what does this field actually mean; why did validating a field break two shipped controls; how do I find a field's real contract; a comment describes a field as unvalidated — is that a defect; how do I check my understanding of a routing key before repairing it

# A field's name is not its contract — its consumer is

- **Type:** `knowledge`
- **Date:** `2026-09-17`
- **Owner / source:** leaf `SIGNOFF-REPAIR.11.14.3.5`

## The question

A struct field is called `scheme`. A test helper's comment says it "is
caller-supplied and is NOT validated against the locator, so a control that wrote
`https` here would be routed on data that is simply false." The check that would
validate it already exists a few lines away. Is this a missing check?

## The answer

> **No — and the way to know is to read the CONSUMER, not the field.** A field's
> meaning is whatever the code that acts on it does with it. The name, the
> doc comment and even a colleague's note about it are descriptions; the
> consumer is the contract.

Measured in the reference deployment: the consumer was one SQL predicate,

```sql
SELECT … FROM resolver_capabilities WHERE schemes @> $1::jsonb
```

and two **shipped** registry rows paired a non-URI scheme with an `https://*`
locator pattern — a Git pack advertising `["git"]` and a browser pack advertising
`["web+render"]`. Both are reached over HTTPS. So the field was never a claim
about the locator at all: it is the key by which a caller asks for a
**capability**.

The repair that treated it as the locator's scheme was written, taken through a
RED/GREEN cycle, and then **refused by two of the product's own controls** —
which is where the error surfaced, one step later than it should have.

## The rule

> Before repairing a field, find every site that READS it and state what each one
> does with the value. If the answer is "one lookup keyed on it", that lookup's
> key space *is* the field's vocabulary — go and read that vocabulary.

```bash
grep -rn "\.scheme\b"            src/   # who reads it
grep -rn "schemes"               src/ migrations/   # and what the key space holds
```

⛔ The trap is that the wrong reading **also explains the evidence**. "A
caller-supplied routing field that nothing validates" is a perfectly good
description of both the defect and the design; only the consumer separates them.
That is `docs/CLAIM_VERIFICATION.md` §3 leg 2 in a new costume — *if your
evidence is consistent with both hypotheses, you have not tested, you have
illustrated.*

## Where the wrong premise came from, and what to do about it

⭐ The premise came from a **comment in a test helper**, which was accurate about
its own situation (that control really does need to name the right scheme) and
misleading as a general statement. A finding that rests on someone else's prose
inherits its scope.

So when a repair is refuted, correct the sentence **at its source** as part of
the same change. A comment that produced one wrong repair will produce another;
tidying the repair and leaving the comment fixes the instance and not the cause.

## What survived

⚠️ Refuting the check did not make the field harmless: it is still unvalidated
against **anything**. That turned out to be deliberate too — the specification
says accepting a reference is not a promise the core can resolve it, so an
unknown scheme is an explicit *unresolvable* at resolution time rather than a
refusal at registration. ⭐ The deliverable became **a stated contract at the
type and in the manual**, plus a control arm that pins the refutation so the next
reader does not re-derive the repair.

## Related

- [[a-prohibition-names-a-mechanism]] — the mirror: there, the warning was too
  broad; here, the finding was too confident. Both are fixed by naming the
  mechanism precisely.
- [[a-census-is-as-wide-as-its-key]] — the enumeration version of the same error.
- [[trust-comes-from-the-check-not-the-shape]] — what makes an identifier mean
  something is the check, not its spelling.
