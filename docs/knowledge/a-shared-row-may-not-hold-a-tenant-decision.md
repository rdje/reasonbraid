answers: where should a per-tenant field live when the row is content-addressed; why did a second tenant inherit the first tenant's setting; what is wrong with a caller field on a deduplicated row; how do I tell content from a decision in a schema; why does a replay discard my value; how do I stop a confused-deputy credential reach

# A shared row may not hold a tenant's decision

- **Type:** `knowledge`
- **Date:** `2026-09-17`
- **Owner / source:** leaf `SIGNOFF-REPAIR.11.14.3.10`; the decision it came from is `docs/decisions/2026-09-17_a-credential-binding-is-tenant-bound.md`

## The question

A table is content-addressed: its key is a digest, a `(locator, digest)` pair, a
canonical form — something derived from *what the row is*, so that two callers
naming the same thing get the same row. Then a column on it is filled from the
request body. Which columns may live there?

## The answer

> A row keyed on CONTENT may hold only content. Anything that encodes what a
> particular tenant *decided* — who may see it, what it costs, which credential
> reaches it — belongs on a row keyed by tenant.

The test is one question about the key, and it is mechanical:

> **Two tenants name the same content and disagree about this column. Which one
> wins?**

If the honest answer is *"whichever wrote first, and the other is silently
given that value"*, the column is on the wrong row. Note both halves of the
damage — the second tenant **inherits** a decision it never made, *and* it
**cannot express** its own. They are the same defect seen from two sides, and a
repair that fixes only one has not moved the column.

## The measured instance

`resource_references` is keyed `UNIQUE (original_locator, expected_digest)`.
`credential_binding_ref` sat on it, arrived as free text on `POST /v1/resources`,
and was handed to `broker.resolve` — it **selects a credential**.

So a second tenant registering the same URL at the same digest replayed the
first tenant's row and its resolve attached the **first tenant's credential** to
its own acquisition. ROADMAP §16.3 invariant 5 forbids exactly that: *target
credentials are selected only after authorization for the concrete target and
action.*

⭐ And the honest case was broken too, invisibly: two tenants citing one URL
could not hold two different bindings, because the replay discarded the second
value. Nobody had written that down — the sharing was only ever discussed as a
leak, never as a limit.

## Why it keeps happening

⚠️ This was the **fourth** instance in one family, and they read identically:

| | The shared row | The decision that had been put on it |
| --- | --- | --- |
| `.11.14.1` | `evidence_snapshots` (content digest) | which tenant may READ the bytes |
| `.11.14.2` | `claim_assessments` | which tenant AUTHORED the assessment |
| `.11.14.3.4` | `resource_references` (pair key) | which tenant REGISTERED the reference |
| `.11.14.3.10` | `resource_references` (pair key) | which credential REACHES the content |

🔎 The pull is real, not carelessness: the content row is *where the thing is*,
so every field about the thing feels at home there. The distinction is not about
the subject of the field — all four are about the reference — it is about
**whose answer it is**. Content has one answer for everyone. A decision has one
answer per tenant.

## The repair shape, and the one it is not

> Add the tenant-bound row, move the column, and **drop the original**.

⛔ Stopping at *"stop reading the old column"* is not the repair. A dead column
that still names a credential, a visibility or a price is the next author's
mistake already sitting in the schema — and this repository has had to repair a
dead column before ([[where-an-invariant-lives]]'s neighbours: the `NOT NULL
DEFAULT` clauses no writer could reach, `.11.14.3.5`). Dropping it makes the
schema itself the gate: the read that caused the defect can no longer be
written.

⭐ The strongest form of the fix is structural rather than conventional. After
the move, the UNBOUND read has **no column to read the decision from**, so it
cannot leak it even by mistake; only the tenant-bound read can supply one, and
only the asking tenant's own.

## The backfill is the part that decides whether you may move a secret at all

⚠️ Moving a decision off a shared row means choosing who keeps it. Guessing is
not available for a credential: §16.4 requires secret access to fail closed.

🔎 So look for an **exact join** before deciding the move is safe. Here there was
one — `reference_registrations.registered_by` and `resource_references.submitted_by`
are literally the same value, written from one binding — so the submitter's
registration could be identified precisely and everyone else's left `NULL`. ⛔ If
no such join exists, the honest backfill is *nobody*, and the leaf says so out
loud rather than spreading the value to every registration.

## Related

- [[a-replay-key-is-an-addressing-scheme]] — the same key, asked about aliasing
  rather than about what the row may carry.
- [[one-field-cannot-carry-two-facts]] — the single-row version: one column, two
  meanings.
- [[a-census-is-as-wide-as-its-key]] — how to find the remaining instances once
  the shape is named.
- [[a-repair-owns-every-sentence-that-states-its-limit]] — the published limit
  the fourth instance had to go back and rewrite.
