# Task-Tree Setup Guide

A task tree is a single markdown file under `docs/tasks/<TREE-ID>.md` that owns one
top-level task's recursive breakdown and its execution evidence.

## Create a tree

1. Copy [`tasks/TEMPLATE.md`](tasks/TEMPLATE.md) to `tasks/<TREE-ID>.md`.
2. Fill the metadata, goal, non-goals, and acceptance criteria.
3. Break the goal into leaves (`<TREE-ID>.1`, `<TREE-ID>.2`, …). A leaf is the smallest
   unit that can be finished, verified, and committed as one signoff-quality slice.
4. Set the **Current Frontier** table to the next executable leaf.
5. Register the tree in the Active Task Trees table in [`TASK_TREE.md`](TASK_TREE.md).

## Work a leaf

1. Move the leaf to `active`/`in_progress`.
2. Diagnose tools-first (`TOOLBOX.md`): pin WHY + WHERE before any code.
3. Implement only that leaf.
4. Record verification (before→after, the acceptance checklist from
   `DOCTRINE_ENFORCEMENT.md`) in the tree's Verification Log.
5. Commit via `COMMIT.md`; log the commit in the tree's Commit Log; move the leaf to `done`.
6. Update the frontier to the next leaf. One commit per leaf.

## Discover subtasks

When a leaf uncovers new work, add child leaves (`<TREE-ID>.2.1`, …) rather than expanding
the current leaf beyond a safe slice. The tree is meant to grow as understanding deepens.

## Referring to a leaf

A leaf may be cited **relatively** inside its own tree — `` `.2.3` `` in
`PHASE-8.md` means `PHASE-8.2.3`. That is the convention, and it has exactly one
rule attached:

> ⛔ **A reference to a leaf in ANOTHER tree is written in full.** `PHASE-7.2.3`,
> never `` `.2.3` ``.

The reason is that the trees share a numbering shape, so a relative reference
meant for another tree does not fail — it lands on a **real leaf with a real
status**. `PHASE-8.4.4` wrote *the `.2.3` distribution-channel deferral* meaning
`PHASE-7.2.3`, and `PHASE-8.2.3` exists and is the A2A facade. No resolver can
notice that; only a reader of the surrounding words can
(`SIGNOFF-REPAIR.11.24.1.6`).

A relative reference may also be **relative to an ancestor of the leaf it is
written inside**. A sentence in `SIGNOFF-REPAIR.3.3.4.3` writing `` `.11` ``
means its sibling `SIGNOFF-REPAIR.3.3.4.11`, which is more readable than the
five-component id — 44 references use this. ⚠️ It is a real dialect, not
sloppiness, and it is why a reference can be ambiguous **inside one tree**: the
same token may name a sibling and a tree-level leaf, and both exist.

⛔ **A LANE IS AN ORGANISING NUMBER, NOT A NODE.** `SIGNOFF-REPAIR.11` groups 195
leaves and has no heading of its own, and that is correct: a lane has no goal, no
acceptance and no terminal state, so declaring one would create a leaf that can
never be `done`. Measured with the rest: **nothing cites a lane** — every
reference that looked like one turned out to be a sibling reference
(`SIGNOFF-REPAIR.11.24.1.6.1`).

⚠️ **One tree is written in a different dialect, recorded here rather than
migrated.** `PHASE-1.md` writes `` `.1.6.1` `` for `PHASE-1.6.1`, repeating its
own phase number — 194 of its 252 references read that way, against 46 in the
dominant one. It is `done`; rewriting 194 references in a closed tree is a mass
edit with a real chance of introducing the errors it would be fixing, for no
reader who is not already there. New work uses the dominant dialect.

`scripts/census_relative_leaf_refs.py` measures the corpus, and its `--check`
arm is a registered ratchet: a reference that resolves under **no dialect at
all** may not become more numerous than it is at `HEAD`. ⚠️ It does **not**
catch a wrong cross-tree reference — that class is over two thousand strong and
only the words distinguish a right one from a wrong one — and it does **not**
gate ambiguity, which rose in 5 of 30 commits and is an honest property of a
corpus with three dialects. The rule above is what catches those, and it is a
rule for authors rather than a gate.

## When a tree completes

Mark it `done` in `TASK_TREE.md`, ensure every leaf's evidence and commit is recorded, and
confirm the repo is clean before pivoting to another tree (the pivot rule).
