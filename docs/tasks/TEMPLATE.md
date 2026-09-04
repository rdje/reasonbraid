# <TREE-ID>: <Task Title>

## Metadata

- Tree ID: `<TREE-ID>`
- Status: `proposed`
- Roadmap lane: `<roadmap lane name>`
- Created: `YYYY-MM-DD`
- Owner: repo-local workflow

## Goal

State the exact outcome this top-level task must deliver.

## Non-Goals

- State what this task deliberately does not solve.

## Acceptance Criteria

- The behavior, documentation, or infrastructure outcome is implemented.
- Focused validation passes.
- Broader validation runs when the blast radius warrants it.
- Live docs and roadmap status are updated where project state changed.
- Each completed leaf is committed through `COMMIT.md`.

## Task Tree

- ID: `<TREE-ID>`
  Status: `active`
  Goal: `<top-level goal>`
  Children: `<TREE-ID>.1`

- ID: `<TREE-ID>.1`
  Status: `pending`
  Goal: `<first executable leaf>`
  Acceptance: `<what proves this leaf is done>`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `<TREE-ID>.1` | `pending` | `<reason>` |

## Decisions

- `YYYY-MM-DD`: `<decision and rationale>`

## Open Questions

- `<question, owner, and why it does or does not block the frontier>`

## Blockers

- None.

## Acceptance Checklist (required for any leaf that lands a CODE change)

Enforced by the `TASK-ACCEPTANCE` doctrine (`scripts/check_task_acceptance.sh`). The three boxes
below are **hard-gated**: each must be ticked **and** carry output from a tool you actually ran,
**inside that box's own bullet**. A tick is a claim; the pasted output is the artifact someone
else can re-run.

- [ ] **ROOT CAUSE (WHY + WHERE)** — <the command you ran + its real output, naming the mechanism and the location>
- [ ] **ADDRESSED (verified)** — <measured before → after on the symptom>
- [ ] **NO REGRESSION** — <the suite/gate you re-ran + its result>
- [ ] **FIX** — <the minimal change made> *(not hard-gated)*
- [ ] **LOCKSTEP** — <docs / contracts / index updated, or N/A + reason> *(not hard-gated)*

⚠️ If none of the built-in evidence signatures fit your defect class, do **not** fake one and do
**not** quietly drop the box: declare your own tool's signature in `.doctrine/evidence_tokens.txt`
(see `.doctrine/README.md`). If you find yourself wanting to waive the gate instead, write the
waiver **and name the leaf that owns fixing the gate** — that is the `WAIVER-ROUTING` doctrine, and
an author hitting a gate's boundary is the highest-signal defect report the gate can receive.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `YYYY-MM-DD` | `<TREE-ID>.1` | `pending` | `pending` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `<TREE-ID>.1` | `pending` | `pending` |

## Changelog

- `YYYY-MM-DD`: Created task tree.
