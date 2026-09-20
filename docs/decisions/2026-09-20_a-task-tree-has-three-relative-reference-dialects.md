---
answers:
  - How is a leaf referred to relatively inside a task tree?
  - Is a lane like SIGNOFF-REPAIR.11 a node?
  - Why does the RELATIVE-LEAF-REF ratchet watch only one class?
  - What do the dangling / shared / internally-ambiguous classes mean?
---
# A task tree has three relative-reference dialects, and a lane is not a node

- **Type:** decision
- **Status:** accepted; the ratchet re-priced, the corpus repaired to one row
- **Owner:** `SIGNOFF-REPAIR.11.24.1.6.1`
- **Date:** 2026-09-20
- **Related:**
  `docs/decisions/2026-09-20_resolution-is-not-third-party-implementable-in-process.md`
  is unrelated; the immediate predecessor is `SIGNOFF-REPAIR.11.24.1.6`, which
  wrote the convention and registered the gate this record re-prices.

## The leaf's premise was refuted by its own census

`SIGNOFF-REPAIR.11.24.1.6.1` opened on a number its parent had produced: **62 of
76 `dangling` relative references live in `SIGNOFF-REPAIR.md`, and the top three
are lane 9 (20), lane 11 (16) and lane 10 (6) — lanes with children and no node
of their own.** The stated conclusion was that a lane with 195 children is
unaddressable and every reference to it resolves to nothing.

🔴 **Classified against their enclosing leaf rather than against the tree, none
of them is a lane reference.** Every one resolves:

| Reading | Rows | Example |
| --- | --- | --- |
| `ancestor` — relative to an ancestor of the leaf it sits in | 44 | `.11` inside `SIGNOFF-REPAIR.3.3.4.3` is its sibling `SIGNOFF-REPAIR.3.3.4.11` |
| cross-tree in PHASE-1's dialect | 21 | `.1.2.3` in five different trees is `PHASE-1.2.3` |
| elided-prefix compound | 4 | `` `.3.3.4.10`/`.11` `` — the second reference reuses the first's prefix |
| a deliberate NON-reference | 1 | a control asserting that a citation to a heading that does not exist must not read as anchored |

**Zero referred to a lane.** The lane question the leaf was opened on is moot:
nothing cites a lane, so lanes having no nodes costs no reader anything.

## The decisions

**1. There are three dialects, and the third is legitimate.** A relative
reference resolves against (a) its own tree, (b) an ancestor of the leaf it is
written inside, or (c) in `PHASE-1.md`, that tree's own phase number. The
`ancestor` dialect is not sloppiness: inside `SIGNOFF-REPAIR.3.3.4.3`, writing
`.11` for its sibling is more readable than the five-component id, and 44 rows
use it. `scripts/census_relative_leaf_refs.py` now knows it.

⭐ **That is the instrument's THIRD self-description.** Its first glob matched
recursively and swept 24 artifact documents; its first dialect knew only the
suffix reading and reported `PHASE-1` as 194 defects; and its `dangling` class
called 44 correct sentences broken. Each time the majority class of the first
run was a property of the parser. `docs/CLAIM_VERIFICATION.md` leg 2, three
times over.

**2. A lane is an organising number, not a node.** Only 4 of the 14 top-level
numbers in `SIGNOFF-REPAIR` are headings, and those 4 are leaves that own work
directly with no children. Declaring the other ten would create nodes with no
goal, no acceptance and no terminal state — which the lifecycle in
`docs/TASK_TREE.md` has no room for. Nothing cites a lane, so nothing is owed.

**3. The ratchet is re-priced and narrowed, one commit after it was
registered.** `SIGNOFF-REPAIR.11.24.1.6` calibrated it over `dangling` AND
`internally-ambiguous` at 1 and 0 rises in 30 commits — with the classifier that
did not know the `ancestor` dialect. With it:

| Class | Was | Now | Rises in 30 commits |
| --- | --- | --- | --- |
| `dangling` | 76 | 32 → **1** after repair | **0** |
| `internally-ambiguous` | 126 | 785 | **5** |
| `foreign` | 165 | 54 | **2** |

⛔ **Five in thirty is more than the three-in-thirty that got `foreign`
declined**, so `internally-ambiguous` is declined on the same rule rather than
kept because it was in the first draft. The gate watches `dangling` alone, and a
reference resolving under no dialect is never ordinary work.

⚠️ **The re-pricing is not a re-base**, and the design is what makes that true:
`--check` recomputes the `HEAD` baseline with the CURRENT classifier, so an
instrument change moves both sides identically. The baseline moved **down**,
from 32 to 1, because the corpus was repaired.

## The repair

31 references rewritten, each resolved by subject rather than by "the only id
that exists": the 21 cross-tree ones to `PHASE-N.x.y`, the 4 compounds to their
full prefix, and 6 quoted inside `SIGNOFF-REPAIR.13.4.1` to the leaves whose
checklists they quote. ⚠️ One is graded weaker than the rest and said so:
`PHASE-8.md`'s *the `.2.7` revocation drill* was resolved to `PHASE-2.7` by
subject adjacency — that lane's children are the node-replacement drill and the
audit-linkage groundwork, both named in the same sentence — rather than by an
exact title match, because `PHASE-7.2.7` does not exist and PHASE-7's own lane
stops at `.2.4`.

⭐ **One `dangling` row remains, and it is correct.** A control in
`SIGNOFF-REPAIR.11.4.2.1` asserts that *a warning citing a heading that does not
exist must NOT read as anchored*, and cites a deliberately absent id to say so.
The census cannot tell an example from a use, and teaching it to would be worse
than one honest permanent row.

## The instrument census this rests on

**22 instruments read `docs/tasks/`.** Six locate a leaf by its heading or `ID:`
line — `check_task_status.sh`, `check_frontier_status.py`,
`check_tree_index_frontier.sh`, `check_book_frontier.sh`,
`check_task_acceptance.sh`, `census_goal_receipt_gap.py` — and
`check_leaf_id_unique.sh` `exec`s the last of those, so seven by effect. The
other fifteen read the tree files for staged-file checks, prose predicates or
link resolution and never resolve an id to a node. ⚠️ Hand-checked by grepping
each for its locator, after an automated predicate over the same 22 disagreed
with two of them — the count is small enough to check and was.
