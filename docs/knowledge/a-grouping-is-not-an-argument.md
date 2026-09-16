answers: how do I check a per-table or per-item verdict record; why did my decision apply the right reason to the wrong item; what goes wrong when I answer N items in one table; how should I correct one row of an accepted decision record without discrediting the rest; how do I stop a conservative-sounding verdict from escaping review

# A grouping is not an argument

- **Type:** `knowledge`
- **Date:** `2026-09-16`
- **Owner / source:** leaf `SIGNOFF-REPAIR.11.14.2`, correcting `SIGNOFF-REPAIR.11.14`'s record six commits after it was accepted

## The question

A decision record answers many items at once — twelve tables, twenty routes,
eight crates — and presents the answer as a table: item, verdict, why. What is
the failure mode, and what checks it?

## The answer

> Rows that share a cell share an argument. **Grouping is a presentation
> choice that silently becomes a justification**, and the reason is then
> verified against whichever member of the group prompted it.

`SIGNOFF-REPAIR.11.14` decided twelve tables. Its reason was correct and
carefully derived: the evidence chain is content-addressed, so a row several
tenants share cannot carry an owner, and a `tenant_id` column would break the
deduplication the digest exists for. Three tables were listed under it —
`evidence_snapshots`, `derivations`, `claim_assessments` — with one "Why" cell
reading *"the row is a shared receipt"*.

That was true of two of them. The schema said so, in the line nobody read:

```sql
CREATE UNIQUE INDEX derivations_replay_idx
    ON derivations (parent_snapshot_id, derived_kind, derived_digest);
CREATE UNIQUE INDEX claim_assessments_replay_idx
    ON claim_assessments (claim_id, snapshot_id, assessment, author);
```

The second key carries the **author**. Two tenants asserting the same thing
about the same evidence already held two separate rows. That table was never
shared, the argument never reached it, and the verdict it inherited — "gains no
column" — was wrong for six commits.

**The check is one question per row, with a command that answers it.** Take the
group's reason, invert it into a property the item must have, and enumerate:

> the reason is *"the row is shared"* → for each item, **is this row ever
> shared?** → read its uniqueness constraint.

Twelve items, one question, twelve cheap answers. A reason stated once for a
group must be *re-derived* per member, or it is an assertion about one member
and a hope about the rest.

⚠️ **Watch the conservative-sounding verdict hardest.** The wrong answer here
was "do not add a column". Restraint reads as rigour, so it draws less review
than a change would — and it shipped inside a record whose own argument was that
the obvious reading of a specification would have produced twelve unnecessary
migrations. A record about not over-applying a rule over-applied its own.

## Correcting one row without discrediting the record

Amend, do not rewrite. Add a dated **Correction** section; put the evidence that
refutes the row next to the evidence that supported it, in the same shape (here,
the two index definitions side by side); mark exactly one verdict superseded;
and state explicitly that the others stand. ⭐ The last clause is not politeness
— a correction that does not bound itself invites the next reader to distrust
the whole record, which costs more than the original error did.

See also [[a-census-is-an-instrument-not-a-table]]: the same defect one level up
— a table is a measurement someone took once, and what a later reader needs is
the question that produced it.
