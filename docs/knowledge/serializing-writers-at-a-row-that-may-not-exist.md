answers: how do I stop two concurrent writers colliding on a version number; why does my SELECT FOR UPDATE not serialize the first write; why do two concurrent writes raise a unique violation instead of queueing; how do I serialize writers at an anchor row that may not exist yet; why is a unique constraint the wrong place to serialize a write

# Serializing writers at a row that may not exist yet

- **Type:** `knowledge`
- **Date:** `2026-09-12`
- **Owner / source:** leaf `SIGNOFF-REPAIR.3.3.4.11` (the profile/card census), applying the construction already present in `authority::transaction::acquire_in_tx`

## The question

Two concurrent writers each need the next number in a per-entity sequence — a
profile version, a revision, an attempt counter. The obvious shape is to read the
current value, add one, and insert under a `UNIQUE (entity, number)` constraint.
Why does that collide, and why does the obvious fix not work either?

## The answer

Reading and then writing is a check-then-act across two statements: both writers
read N, both write N+1, and the constraint turns the second one into a **raised
error**, not a queue. The caller receives a failure where they should have
received N+2.

The obvious fix — take a row lock on the anchor the counter lives on — is right
in shape and wrong in detail:

> **`SELECT … FOR UPDATE` over a row that does not exist locks nothing.**

It is not an error and it does not block. It matches zero rows, returns
immediately, and both writers proceed exactly as they did before. So the fix
protects every write except the one that is hardest to reason about — the
**first** one, where the anchor has not been created yet and the code path
usually defaults the counter to zero.

The construction that works is two statements, in this order, inside the same
transaction:

```sql
INSERT INTO anchor (key) VALUES ($1) ON CONFLICT (key) DO NOTHING;
SELECT counter FROM anchor WHERE key = $1 FOR UPDATE;
```

First-use creation is *inside* the acquisition rather than in a gap before it, so
there is no ordering in which a writer reaches the lock before the row exists.
Every writer then holds the same row and the increments serialize.

This project already had the construction before it needed it here:
`authority::transaction::acquire_in_tx` inserts the tenant's guard row
`ON CONFLICT DO NOTHING` and only then selects it `FOR SHARE`/`FOR NO KEY UPDATE`,
for exactly this reason. Look for an existing acquisition in the codebase before
writing a second one.

## Why the constraint cannot be the serialization

Leaving the `UNIQUE` constraint to raise looks like a cheaper way to reach the
same outcome — catch the violation, retry, move on. It is not, once the write has
to record anything about itself:

> A constraint violation **aborts the PostgreSQL transaction**, so every
> statement after it fails until rollback.

An admission record, an administrative effect, an audit row — none of them can
commit alongside a refusal that was delivered by a raise. See
`docs/knowledge/a-raised-constraint-cannot-be-a-recorded-refusal.md`. The anchor
lock is therefore not a nicer route to the same place; it is the only shape in
which the outcome can be recorded at all.

Keep the constraint. It is the backstop that proves the lock is working, and it
should never fire in normal operation.

## Re-verify

Run two concurrent writers against the same entity and assert both succeed with
consecutive numbers and both payloads readable — then run the same pair against
an entity with **no anchor row yet**, which is the case a lock-only fix silently
fails. Restoring the read-then-write shape must turn the second control red.

Related: `docs/knowledge/a-raised-constraint-cannot-be-a-recorded-refusal.md`;
`docs/knowledge/proving-a-race-is-closed.md`, "Deriving the mode is part of the
repair".
