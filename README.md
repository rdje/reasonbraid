# ReasonBraid

**Working name** (not legally cleared — `docs/adr/001-uncleared-working-name.md`).
Keep this repository **private** until that ADR’s clearance gate passes. Do not
assume crate, domain, or handle names are obtainable.

ReasonBraid is an agent deliberation and governance platform: authorized humans
and agents can ask a durable network a question without knowing who is online.
Rust — not an LLM — enforces identity, authorization, ordering, budgets, and
publication. Model output stays untrusted until deterministic rules accept it.

## Status

Phase 0 — contracts and kill-risk experiments. Scope and gates:
`ROADMAP.md` (v0.4.1, frozen). Day-to-day execution: companion `KICKOFF.md`.
Work is owned by task-trees under `docs/tasks/` (start at `PHASE-0`).

## Quick start

```bash
git config core.hooksPath .githooks
make check    # fmt, clippy -D warnings, tests
make gate     # doctrine enforcer
make book     # mdBook (requires mdbook)
```

## Where to read more

| Need | Canonical home |
| --- | --- |
| Product docs | `docs/book/` |
| Architecture / gates | `ROADMAP.md` |
| Phase 0 execution | `KICKOFF.md` |
| Agent bootstrap | `CLAUDE.md` / `AGENTS.md` |
| Claim checks | `docs/CLAIM_VERIFICATION.md` |

This landing page is governed by [`README_POLICY.md`](README_POLICY.md) and
capped (line and byte) by the `README-STABILITY` doctrine. The repo keeps the
bedrock discipline spine (task-trees, memory architecture, `COMMIT.md`, git
hooks). Pull spine updates with `./scripts/update_scaffold.sh <bedrock-url>`.

## Non-negotiables

- Nothing changes without a **task-tree leaf** first.
- Record durable facts in `docs/decisions/`.
- Commit per `COMMIT.md`; hooks + CI enforce doctrines.
- Keep **roadmap ↔ code ↔ docs** in lockstep.
