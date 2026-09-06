# 2026-09-06_budget-read-surface.md

## Context

`.1.6.1` (the `.1.6` census finding): the `.1.6` goal names "budgets" as an
inspection surface, but the gap census found budgets had NO read surface
anywhere — the ledger tables exist (`migrations/0005_budget.sql`: ceilings +
reservations with status/usage/reason), yet no GET endpoint and no CLI verb
touched them. The page the direction record describes could therefore not
render "budgets" without a new surface.

## Decision

- **The budget read surface is `GET /v1/threads/{thread_id}/budget`** — a
  READ-ONLY query over the existing ledger rows: the ceiling (dimensions,
  policy_version, created_at) plus every reservation row (status, held vs
  settled usage, denials with their reasons, expiry/settle times). Nothing is
  computed or invented; the view exposes the same rows the budget engine
  enforces against, so "spend and uncertainty are visible" (`ROADMAP.md`
  §26.1) follows the ledger itself.
- **Gated by the existing `thread_inspect` path** (the `get_thread` gate): a
  role without the grant is a typed 403 with the audit row — the UI inherits
  the authorization semantics unchanged, no new grant, no write path, no new
  table.
- **The CLI mirrors it**: `rb inspect budget --thread` (same resolve/print
  discipline as `rb inspect thread`).
- **Wire shape**: absent optional facts (usage, reason, expires_at, settled_at)
  are OMITTED, never `null` — the `.1.5.1` discipline applied to the new
  surface's own fields. Stored ledger JSONB is passed through verbatim
  (pre-existing shapes are not rewritten by a read surface).
- A thread whose ceiling row is missing reads `scope_hidden` (not an empty
  budget): BUDGET-003 says a thread exists with its ceiling, so a missing row
  is corruption, not an honest `{}`.

## Consequences

- The `.1.6` static page renders budgets from this endpoint; no other surface
  is needed for the backlog-18 "budgets" item.
- Denial reasons carry the budget engine's raw `detail` text (debug-shaped for
  the held-dimensions tail — a pre-existing engine wording, passed through
  faithfully; prettifying it is an engine task, not a read-surface one).

answers:

- **A read surface must not reimplement the ledger.** It exposes rows; the
  engine's own denial/usage facts are the authority. Any "summary" would be a
  second computation that can drift — the page can derive display from rows,
  the API does not.
- **Inherit the existing gate.** A new surface that reuses `thread_inspect`
  adds no authority; a surface with its own grant would reopen the
  deny-by-default review for zero new capability.
