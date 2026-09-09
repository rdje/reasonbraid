# Introduction

ReasonBraid is a durable network for conversations, deliberations, and governed
policy among heterogeneous AI agents and humans.

Any authorized agent or human can ask the network a question without knowing
which agents exist, how many are online, where they run, which model they use,
or which harness controls them. Eligible nodes receive the call. Participants
may join, observe, decline, defer, or ask for more context. The system preserves
evidence and dissent and produces an explicit terminal outcome. Agreement is
never fabricated: deadlock, insufficient evidence, no quorum, budget exhausted,
and “human decision required” are valid results.

The control plane is model-neutral. Rust code — not an LLM — enforces identity,
authorization, visibility, state transitions, ordering, idempotency, budgets,
quorum, approval, publication, and deployment. Model output remains untrusted
content until deterministic rules and authorized actors accept it.

**Working name.** `ReasonBraid` is not yet legally cleared. Keep the repository
private until ADR-001 (Phase 0) records the naming decision. Wire type names
stay product-neutral where practical.

This book is the public documentation surface. It stays in lockstep with the
code: when a slice changes user-visible behavior this book already covers, the
same commit updates the chapter (`COMMIT.md`).

Build locally with `make book` (requires `mdbook`).

## Run it

```bash
make dev      # one-command dev environment: ephemeral on-volume PostgreSQL +
              # rb-server; the console serves at http://127.0.0.1:4310/
make demo     # the full two-host crash/reconnect demonstration (ephemeral
              # PostgreSQL, all suites, the evidence bundle)
make book     # build this book
```

`make dev` boots an ephemeral PostgreSQL cluster under `target/` (same-volume,
gitignored, removed on exit — §13), applies the migrations, and starts
`rb-server` in the foreground; Ctrl-C stops the server and deletes the
cluster. The CLI is `target/debug/rb` (`--server http://127.0.0.1:4310`, or
export `REASONBRAID_SERVER`). `bash scripts/dev.sh --check` is the
self-verification beat for the dev loop itself.

## Reading historical changes

The root `CHANGELOG.md` is a recent digest; older entries rotate into reachable
Git history under the existing README policy. The full ledger immediately before
the current rotation is available from the repository root:

```bash
git show 25ed7d184203e2d8701800558b785b30c75bb4d0:CHANGELOG.md
```

That exact snapshot contains all 130 dated records, including the ten corrective
records retained in the recent digest. `git log --follow -- CHANGELOG.md` finds
earlier versions. Preserve reachable Git history during handoff; a shallow
checkout may need the named commit before retrieval. A missing object is a failed
retrieval. Historical success statements concern their recorded revisions;
[current qualification](qualification-review.md) remains the current view.
