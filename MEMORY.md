# MEMORY — resume pointer (layer A; overwrite-only, keep ≤ ~50 lines)

> The bounded layer-A resume pointer (see `MEMORY_ARCHITECTURE.md`). **OVERWRITE** the
> "Current state" block each update — never append history. History is git (layer D) + the
> task-tree logs (layer B); durable facts are `docs/decisions/`.

## Which mode are you in?

- **Maintaining bedrock itself?** → read `MAINTAINING.md`, then the active tree
  `docs/tasks/BEDROCK-MAINTENANCE.md` → its Current Frontier.
- **Starting a NEW project from bedrock?** → run `scripts/bootstrap.sh <name>`; it resets
  this file to a clean seed and removes the maintainer-only files (`MAINTAINING.md`, the
  maintenance tree, the provenance record).

## How to resume (maintainer)

1. Read `README.md`, `MAINTAINING.md`, `MEMORY_ARCHITECTURE.md`, `TOOLBOX.md`,
   `DOCTRINE_ENFORCEMENT.md`.
2. Open `docs/tasks/BEDROCK-MAINTENANCE.md` → Current Frontier → continue from the next action.

## Current state

- **Project:** bedrock — the project-neutral discipline spine extracted from PGEN
  (see `docs/decisions/reference_bedrock_provenance.md`; PGEN = `../pgen`).
- **Active tree:** `BEDROCK-MAINTENANCE` — `.1` **done**, `.2.1` (README policy + layer-A byte cap)
  **done**, `.2.2` (`WAIVER-ROUTING` + the neutrality bar) **done**, `.2.3` (admission test
  re-ordered) **done**, `.2.4` (`TASK-ACCEPTANCE` universal core) **done**, `.2.5` (the 2026-09 day-one
  batch: NO AGENT TRAILERS + hook, the handoff census, `LIVE-DOC-CURRENCY`) **done**, `.2.6` (part 2:
  `LESSON-PROMOTION`, `ROUTING-EVIDENCE`, `GAP-CLAIM-CENSUS`, a fresh `TABLE-ARITY-RATCHET`) **done**, `.2.7` (foolproof project creation:
  the bootstrap leaf owns the first commit) **done**; frontier
  `.2` = the transfer loop (`GATE-REACHABILITY` principle next; `DESTRUCTIVE-TARGET-GUARD` stays **rejected as-is**).
- **Next action:** a `.2.x` backlog item — `GATE-REACHABILITY` as a principle, or one of the input-bound
  principles noted in `.2.6` (`BASELINE-IDENTITY`, `IDENTITY-CARRIER-CURRENCY`) behind a project-declared
  seam; or a newly-landed **general** upstream improvement (process in `MAINTAINING.md`).
- **The bar for anything ported here (maintainer, 2026-07-30):** ask **Q1 first** — *does this
  objectively benefit any present and any future project?* — and only then Q2, *can it be said
  without domain nouns?* ⛔ A 0-noun score does NOT imply portability: neutral vocabulary can
  still encode one project's workflow. See `MAINTAINING.md`.
- **Latest commit:** `.2.7` (`BEDROCK-MAINTENANCE-0010`) — creating a project is now proven foolproof from a
  fresh clone through the FIRST COMMIT: `bootstrap.sh <name>` seeds `docs/tasks/BOOTSTRAP.md`, a done leaf
  that owns its own crate rename with the enforcer's evidence, and prints the commit command. Before it,
  the first commit was refused by two doctrines. `DOCTRINE_VERSION` → `0.6.1`.
- **Transfer runs BOTH WAYS** (maintainer-confirmed, now in `MAINTAINING.md`): this spine's
  layer-C check was stronger than upstream's; upstream adopted it, strengthened it (row-anchored
  + bidirectional) and sent it back. Owed upstream: the `WAIVER-ROUTING` fail-open fix.
- **In-flight uncommitted work:** none — `make gate` green (13/13).
