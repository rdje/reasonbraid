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

## When a tree completes

Mark it `done` in `TASK_TREE.md`, ensure every leaf's evidence and commit is recorded, and
confirm the repo is clean before pivoting to another tree (the pivot rule).
