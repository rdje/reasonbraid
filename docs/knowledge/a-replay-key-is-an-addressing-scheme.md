answers: is my idempotency key safe; what happens when two writers share a dedupe key; why did a caller get back an id it should not know; how do I audit a UNIQUE index for abuse; is a uniqueness constraint enough to stop aliasing; what makes a replay pre-check different from the index behind it; can a caller-supplied column be part of a key

# A replay key is an addressing scheme

- **Type:** `knowledge`
- **Date:** `2026-09-17`
- **Owner / source:** leaf `SIGNOFF-REPAIR.11.14.3.3`; the decision it came from is `docs/decisions/2026-09-17_the-assessment-namespace-is-part-of-the-row.md`

## The question

A table has a replay / dedupe / idempotency key — a `UNIQUE` index over a few
columns, and a pre-check that returns the existing row's id when a submission
matches. It has worked for months. What makes it a security question rather than
a storage detail?

## The answer

> A replay key is not only a rule about which rows may coexist. It is the
> **address** by which a caller names an existing row — and the pre-check hands
> that row's identifier back.

So the key is an authorization surface, and the test is:

> **Every column of a replay key must be a value the submitter is entitled to
> assert.** A column the caller chooses is an address it can aim; a column the
> server sets is a partition it cannot leave.

⛔ **"One writer, therefore safe" is the wrong safety condition**, and this note
originally stated it that way — the correction is the whole lesson. What bounds
the reachable rows is not how many code paths write the table; it is whether the
key space is already partitioned by something **server-set**. One writer and two
tenants is enough: the caller fills in another tenant's values and the pre-check
returns that tenant's row id.

The measured instance, and it failed on **both** dimensions at once:

- `claim_assessments` replayed on `(claim_id, snapshot_id, assessment, author)`.
- `author` is an unauthenticated caller label. A repair had already added a
  server-recorded `authored_by_tenant` **because** `author` could not be trusted
  for authorization — and nobody asked why a field too weak to authorize was
  still carrying a quarter of the row's identity.
- ⭐ The migration that added the trustworthy column reasoned, in its own header,
  that the key "carries the AUTHOR, so two tenants asserting the same thing about
  the same evidence already hold two separate rows" — and stated **eight lines
  later, in the same file**, that `author` is an unauthenticated caller string.
  Both cannot be true. Two tenants hold two rows only while they happen to type
  different labels.

⭐ The general form: **a field you have already decided not to trust for
authorization must not be load-bearing for identity either.** Those are the same
decision arriving twice, and the second arrival is easy to miss because it looks
like schema rather than policy.

## The rule

> Enumerate a replay key's columns and mark each one **caller-set** or
> **server-set**. For every caller-set column, ask which rows a caller can reach
> by choosing it. If that set contains rows belonging to another principal,
> another tenant, or another writer, the key needs the corresponding **server-set
> column** — the tenant, the authenticated principal, the writer's identity —
> added to it, not merely recorded beside it.

A discriminant a reader can ignore does not stop collisions. A column *in* the
unique index makes the same caller-set values in two partitions two rows, which
is the property you actually wanted.

### The index is not the fix — the pre-check is, too

⛔ Worth separating, because fixing only the schema leaves the defect exactly
where it was. A `UNIQUE` constraint constrains the **write**. A pre-check that
`SELECT`s on the key and returns early stands **in front of** the write, so a row
the new constraint would have separated is never inserted — the pre-check matched
first and returned. Both have to move in the same commit, and a control that only
inserts will not notice.

### Reproducing it

Drive two submitters at one key and assert the returned ids **differ**:

```rust
assert_ne!(forged["assessment_id"].as_str().unwrap(), external_id,
    "a SECOND TENANT was handed the first tenant's assessment_id");
```

Two lines, and it fails loudly with both ids printed — which is also the clearest
possible statement of the finding. ⚠️ **Write one arm per caller-set column.** The
first version of this control used a single tenant, proved the two-writer case,
and left the two-tenant case to be inferred by reading the SQL. The reading was
wrong in the safe-sounding direction: it reported the defect as *narrower* than it
is.

## Related

- [[trust-comes-from-the-check-not-the-shape]] — the other half of this leaf: why
  making the two identifiers *look* alike would have been the wrong repair.
- [[one-field-cannot-carry-two-facts]] — the same disease one level down: a single
  column written from two independent facts.
- [[an-absence-claim-is-a-census-over-the-corpus]] — "no caller can reach that row"
  is a set claim, and it carries its enumeration.
