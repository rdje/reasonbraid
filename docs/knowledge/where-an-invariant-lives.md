answers: should this rule be a type, a schema constraint or a decoder check; why did my validation not stop the bad row; where do I enforce that two records belong together; why is a check that runs on read weaker than one that runs on write; how do I stop an invalid combination instead of detecting it?

# Where an invariant lives decides what it can promise

- **Type:** `knowledge`
- **Date:** `2026-09-12`
- **Owner / source:** leaf `SIGNOFF-REPAIR.3.3.4.7.2`

## The question

You have a rule — "this operation's target must be one of these", "this record
must belong to that record's tenant", "this text is at most N bytes". You can
enforce it in the type, in the database, or in the codec that reads the row back.
They are not interchangeable. Which one does the job?

## The answer

Rank them by **what they make impossible**, not by where they are easiest to
write.

| Where | What it promises | What it cannot do |
| --- | --- | --- |
| The type | The invalid state cannot be *constructed* in this language. | Nothing about rows another writer put there — a migration, a repair script, a DBA, an older build. |
| The schema | The invalid row cannot be *stored*, by anyone, ever. | Only what SQL can express cheaply, and only what you thought of when you wrote it. |
| The decoder | The invalid row is *refused when read*. | It has already been written. Everything between the write and the read saw it. |

A decoder check is the weakest of the three because **it runs on the way out**.
The row existed. Anything that read it with different code, counted it, exported
it, or replicated it, saw the invalid value. For evidence — the kind of record an
operator later trusts — that gap is the whole problem.

So the order to reach for is: make it unrepresentable in the type; if the type
cannot carry it, constrain it in the schema; use the decoder only for what
neither can express — and then treat a decoder refusal as a *storage failure*,
not as a value.

### Three worked shapes

**"These two fields must belong together."** Do not check it. Put both in the
same variant so the invalid pair has no spelling. An operation enum whose
variants carry their own targets makes "breaker reset, targeting a grant"
unrepresentable; two parallel enums make it merely wrong, and something has to
keep checking it forever.

**"This row must belong to that row's owner."** A single-column foreign key plus
a check is a convention. A **composite** foreign key — `(child_id, owner)`
referencing `(parent_id, owner)` — is the invariant. It costs one extra unique
index on the parent, and afterwards no writer, including a future one nobody has
written yet, can bind a record across the boundary.

**"This value is well-formed."** Split it. Let the schema constrain the coarse,
cheap discriminant — a `kind` that must be in a closed list — and let the codec
enforce the structure beneath it. That split is deliberate and worth stating,
because it predicts something useful: **a row can pass the column constraint and
still be refused on the way out**, and a control should assert exactly that
rather than assuming one layer covers the other.

### The cost is real and should be paid on purpose

A schema-level invariant is not free. A new child table means every fixture plan
that deletes its parent must now name the child, or a cleanup silently leaves
rows behind — measured here as 27 files. That is the enforcement working, not the
enforcement being inconvenient: the alternative is 27 plans that quietly stop
cleaning up. Census the blast radius **before** writing the constraint, so the
cost is a decision rather than a red gate.

## Related

- `docs/knowledge/a-missing-field-is-not-an-absent-value.md` — the same layering
  question for one field's presence.
- `docs/knowledge/one-field-cannot-carry-two-facts.md` — why the type is where
  separate facts should be separated.
