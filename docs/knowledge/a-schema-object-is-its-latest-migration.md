answers: which migration defines this table or view; why did my column mean something different from what the migration said; how do I read a schema fact out of a migrations directory; why did a control asserting a documented schema behaviour fail; is CREATE TABLE the definition of a table; how do I check a view before I depend on it?

# A schema object is its latest migration, not its first

- **Type:** `knowledge`
- **Date:** `2026-09-13`
- **Owner / source:** leaf `SIGNOFF-REPAIR.4.1.3.1`

## The question

You need to know what a column, view or constraint means before you build on it.
You grep the migrations directory, find the `CREATE` that introduced it, read the
comment its author wrote, and quote that. Is what you just read still true?

## The answer

**Only if nothing has redefined it since — and a redefinition leaves the original
`CREATE` untouched and still findable.** A migrations directory is an append-only
log, not a schema. The schema is the *fold* of that log. `grep -n "CREATE TABLE
x"` and `grep -n "CREATE OR REPLACE VIEW x"` answer different questions, and only
the second one tells you what the database will do today.

The trap is sharper than an ordinary stale-comment problem, because the first
migration is the one that *explains itself*. It carries the rationale, the design
note, the worked example. The migration that supersedes it is usually terse. So
the most quotable text in the directory is the one most likely to be wrong.

### What it looked like here

`node_presence.suspended` was introduced by migration 0012 as:

```sql
EXISTS (SELECT 1 FROM node_certificates c
        WHERE c.node_id = n.node_id AND c.revoked_at IS NOT NULL)  AS suspended
```

— *ever revoked*. A leaf read that, correctly concluded the column was the wrong
predicate to gate a lease renewal on ("it would starve every replaced node"), and
wrote the reason into its record, into `MEMORY.md` and into `CHANGELOG.md`.

Migration 0017 had already replaced the view:

```sql
(EXISTS (… revoked_at IS NOT NULL)
 AND NOT EXISTS (… revoked_at IS NULL))  AS suspended
```

— written for exactly that hazard, five migrations earlier than the leaf that
warned about it. The *conclusion* survived; the stated reason for it was false,
and it was false in three durable documents, propagating into the next leaf.

### How it was caught, and how to catch it

It was caught by a control that asserted the claim, not by a review that read it.
The test asserted `suspended == true` for a replaced node — the documented
behaviour — and failed with `left: Bool(false)`. **An assertion on a schema fact
you are about to depend on is cheap and is the only thing that dates itself.**

Before quoting a schema object:

```bash
# every migration that touches it, in order — not just the one that created it
grep -rln '\bnode_presence\b' migrations/ | sort
```

If that returns more than one file, the last one wins. Read it before the first.

## The rule

- The definition of a schema object is the **last** migration that writes it.
  `CREATE OR REPLACE`, `ALTER`, and a later `DROP`/`CREATE` pair all supersede.
- When a record states a schema fact as its *reason* for a decision, assert it in
  a control. A reason nothing tests decays independently of the decision it
  supports, and takes longer to notice because the decision still looks right.
- When the reason turns out to be stale, correct it where it was written **and**
  in every live document that copied it — and say that the conclusion stands, so
  the correction is not read as a reversal.

## See also

- [`a-census-is-an-instrument-not-a-table.md`](a-census-is-an-instrument-not-a-table.md) — the same failure at the level of a measurement: a number recorded once and never re-derived.
- [`where-an-invariant-lives.md`](where-an-invariant-lives.md) — choosing the layer a rule is enforced at, which is the decision a schema fact usually feeds.
