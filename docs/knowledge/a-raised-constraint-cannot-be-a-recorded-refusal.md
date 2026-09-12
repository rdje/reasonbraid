answers: why can I not catch the unique violation and still commit my audit row; why does everything after a constraint error fail until rollback; how do I turn a database refusal into an outcome I can record; when should an INSERT use ON CONFLICT DO NOTHING RETURNING; what should I check before putting a route onto one transaction; why did my new atomic transaction start returning 500 for a case that used to work?

# A raised constraint cannot be a recorded refusal

- **Type:** `knowledge`
- **Date:** `2026-09-12`
- **Owner / source:** leaf `SIGNOFF-REPAIR.3.3.4.10.1`

## The question

A route lets a unique index refuse the request: the `INSERT` raises, the handler
catches the violation and returns a typed `409`. That has worked for as long as
the route has existed. Now the route must also record what the request did, in
the same transaction as the thing it did. Why does the established shape stop
working?

## The answer

**Because in PostgreSQL a constraint violation aborts the transaction.** Every
statement after it fails with `current transaction is aborted` until a rollback,
so there is nothing left to commit — and the refusal you wanted to record is
exactly the case where you have nothing to record it with.

This is not an error-handling inconvenience. The two designs are incompatible:

- "let the constraint raise, catch it, translate it" needs the transaction to be
  expendable, because by the time you have the error it already is;
- "the admission, the mutation and the outcome are one commit" needs a refusal to
  be a VALUE the code can act on, because a refusal is one of the outcomes that
  has to commit.

A refusal that must be durable cannot be delivered by a mechanism that destroys
the transaction carrying it.

## The resolution

Convert the refusal to a value before it can abort anything:

```sql
INSERT INTO t (…) VALUES (…)
ON CONFLICT DO NOTHING
RETURNING …
```

Zero rows returned IS the refusal. No exception, no abort, the same wire answer,
and the transaction survives to commit the outcome beside it. `ON CONFLICT DO
NOTHING` with no conflict target catches any unique violation, including one from
a partial index.

`SAVEPOINT` is the other route and is worse here: it costs a round trip per
attempt and leaves the abort-handling logic in place, when the question was never
"how do I recover from the abort" but "how do I not cause one".

## What to look for

Any administrative route whose refusal is currently produced by a database
constraint raising is a route that cannot record that refusal yet. Finding them
is mechanical — search for `is_unique_violation`, `is_foreign_key_violation`,
and SQLSTATE-matching arms — and each one needs this conversion before its family
can be put on a one-transaction shape.

The same reasoning applies to any check that fails by raising: a deferred
constraint, an `EXCLUDE`, a trigger that calls `RAISE`. If the failure is a
domain answer rather than a fault, the query has to hand it back as data.

### Searching for the catch is not enough — enumerate the constraints

The search above finds routes that *already knew* about a constraint. It cannot
find the ones that never handled it, and those are the dangerous ones: before the
route was made atomic, an unhandled raise was an ugly `500` that someone might
eventually notice; afterwards it is an **unrecordable** refusal, because the
transaction it aborts is now the one carrying the admission and the effect
record.

So the check that belongs in the work itself is not textual:

> For every `INSERT` and `UPDATE` the newly-atomic transaction contains, list the
> constraints on the columns it writes, and ask which of them **a caller can
> trip**. Not "is this statement correct" — "can a request reach this and get a
> raise, and is that a refusal someone is supposed to be told about?"

Constraints are declared in migrations, not in the handler, so they are exactly
the thing not in front of you while writing the transaction. `SIGNOFF-REPAIR.3.3.4.11.5`
is the worked example: the card import wrote a caller-supplied display label into
two tables carrying `UNIQUE (tenant_id, name)` and `UNIQUE (tenant_id, kind,
name)`, nothing caught either, and re-importing the same card answered `500` and
recorded nothing — one leaf after this very record was cited by the leaf that
made the route atomic.

⚠️ Note the direction of the damage carefully when writing it up. The collision
always raised; the repair did not introduce it. What the repair changed is that
the refusal became unrecordable. A correct repair can move a neighbouring defect
to a worse place, and the honest account says which half is which.

## Re-verify

Restore the catch-the-violation shape, run the control that asserts the refusal's
own evidence committed, and confirm it fails — the record is absent because the
transaction that would have written it was aborted by the violation.

Related: `docs/knowledge/where-an-invariant-lives.md` (the constraint is still the
right place for the invariant — only the delivery changes).
