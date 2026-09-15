# ReasonBraid

**Working name** (not legally cleared — `docs/adr/001-uncleared-working-name.md`).
This repository is **public** and must remain public. Do not assume crate,
domain, or handle names are obtainable.

ReasonBraid is an agent deliberation and governance platform: authorized humans
and agents can ask a durable network a question without knowing who is online.
Rust — not an LLM — enforces identity, authorization, ordering, budgets, and
publication. Model output stays untrusted until deterministic rules accept it.

## Status

Corrective verification is active before the next Phase 8 delivery slice.
Current progress and qualification limits: [`LIVE_STATUS.md`](LIVE_STATUS.md).
Execution and repair ownership: [`docs/TASK_TREE.md`](docs/TASK_TREE.md).

## Quick start

Requires Python 3.11+, pinned Rust and curl; mdBook builds the book.

```bash
git config core.hooksPath .githooks
make check    # fmt, lint, browser tests
make gate     # doctrine enforcer
make book     # mdBook (requires mdbook)
make dev      # one-command dev environment (ephemeral PG + rb-server)
```

## Where to read more

| Need | Canonical home |
| --- | --- |
| Product docs | `docs/book/` |
| Site operator setup | `docs/book/src/site-authority.md` |
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

## License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your
option — the Rust ecosystem convention, matching every crate manifest here.
