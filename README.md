# ReasonBraid

**Working name** (not legally cleared — `docs/adr/001-uncleared-working-name.md`).
Keep this repository **private** until that ADR’s clearance gate passes. Do not
assume crate, domain, or handle names are obtainable.

ReasonBraid is an agent deliberation and governance platform: authorized humans
and agents can ask a durable network a question without knowing who is online.
Rust — not an LLM — enforces identity, authorization, ordering, budgets, and
publication. Model output stays untrusted until deterministic rules accept it.

## Status

Phases 0–6 closed (the G3 gate recorded as machinery-blocked-binding-use);
Phase 7 — the Internet-qualified operation — runs: the `.1` hardening lane
is complete (the mTLS identity, the RLS layer, the quotas, the quarantine
evidence rule, the secret-store profiles, the classification controls),
the `.2` supply-chain lane is open (frontier `PHASE-7.2.2`). Work is
owned by task-trees under `docs/tasks/`.

## Quick start

```bash
git config core.hooksPath .githooks
make check    # fmt, clippy -D warnings, tests
make gate     # doctrine enforcer
make book     # mdBook (requires mdbook)
make dev      # one-command dev environment (ephemeral PG + rb-server)
```

## Where to read more

| Need | Canonical home |
| --- | --- |
| Product docs | `docs/book/` |
| Architecture / gates | `ROADMAP.md` |
| Phase 0 execution | `KICKOFF.md` |
| Agent bootstrap | `CLAUDE.md` / `AGENTS.md` |
| Claim checks | `docs/CLAIM_VERIFICATION.md` |
| Security reporting / support | `SECURITY.md` |

This landing page is governed by [`README_POLICY.md`](README_POLICY.md) and
capped (line and byte) by the `README-STABILITY` doctrine. The repo keeps the
ReasonBraid discipline spine (task-trees, memory architecture, `COMMIT.md`, git
hooks). Pull spine updates with `./scripts/update_scaffold.sh <reasonbraid-url>`.

## Non-negotiables

- Nothing changes without a **task-tree leaf** first.
- Record durable facts in `docs/decisions/`.
- Commit per `COMMIT.md`; hooks + CI enforce doctrines.
- Keep **roadmap ↔ code ↔ docs** in lockstep.
