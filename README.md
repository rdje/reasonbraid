# bedrock — a Rust project discipline-spine template

**bedrock** is a starting point for a new Rust project that ships with a battle-tested
*discipline spine* already wired in: durable memory, task-tree tracking, a strict commit
workflow, mechanical doctrine enforcement, a knowledge map, and an mdBook — all
project-neutral. Copy it, drop in your roadmap, and grow the project with that spine as its
backbone.

## Why a spine

Discipline that lives in an agent's head evaporates on session loss, a model switch, or a
new contributor. bedrock puts the discipline **in the repo** and enforces it at the **git
level** (pre-commit hook + CI), so it holds for any agent — Claude Code, Codex, Gemini,
Cursor, a custom runner — or a human, identically.

## What's inside (the spine)

| File / dir | What it gives you |
| --- | --- |
| `CLAUDE.md` / `AGENTS.md` | harness-neutral agent bootstrap (read this first) |
| `MEMORY_ARCHITECTURE.md` | the durable 4-layer memory model (A resume pointer · B task-trees · C decisions · D git) |
| `docs/TASK_TREE.md` + `docs/tasks/` | task-tree tracking — nothing changes without a leaf |
| `COMMIT.md` | the strict, repeatable commit workflow |
| `DOCTRINE_ENFORCEMENT.md` + `scripts/check_doctrines.sh` | the mechanical enforcer (registry + universal checks + a project slot) |
| `TOOLBOX.md` | the tools-first diagnostic doctrine |
| `KNOWLEDGE_MAP.md` + `knowledge-map/` | a derived, drift-proof orientation map |
| `docs/book/` | an mdBook skeleton — the public docs surface |
| `.githooks/` + `.github/workflows/` | the E3 (hook) + E4 (CI) enforcement layers |
| `MEMORY.md` · `CHANGELOG.md` · `DEV_NOTES.md` · `LIVE_STATUS.md` | the seeded live-docs |
| `ROADMAP.md` | **the one file you replace** — your project's roadmap |
| `Cargo.toml` · `crates/` · `Makefile` | a minimal Rust workspace + `make check`/`make gate` |

## Use it

**Option A — `cargo generate` (Rust-native):**

```bash
cargo generate --git <this-repo-url> --name <project>
```

**Option B — GitHub "Use this template"** (enable the *Template repository* setting).

Then, either way, `cd <project>` and finalize with one command:

1. `./scripts/bootstrap.sh <project>` — installs the git hooks, sets the crate + roadmap name,
   generates the Knowledge Map, verifies the enforcer, and seeds the leaf that owns this step —
   then **commit with the command it prints**. The canonical post-copy step for both paths
   (option A needs `cargo install cargo-generate`).
2. Replace `ROADMAP.md` with your project's roadmap, then create your first task-tree
   (`cp docs/tasks/TEMPLATE.md docs/tasks/<TREE-ID>.md`) and register it in `docs/TASK_TREE.md`.
3. Grow the project one task-tree leaf at a time, committed via `COMMIT.md`.

## Keep the spine current

bedrock improves over time. To pull the latest **project-neutral** spine (doctrine docs,
hooks, universal checks) into a project you already created — without touching your
roadmap, task-trees, decisions, or code — run:

```bash
./scripts/update_scaffold.sh <bedrock-repo-url>
```

The scaffold version is recorded in `DOCTRINE_VERSION`.

This README is deliberately a **landing page**, governed by [`README_POLICY.md`](README_POLICY.md)
and mechanically capped (line **and** byte) by the `README-STABILITY` doctrine. Route changing
detail to its canonical home rather than growing this file.

## The non-negotiables (full detail in `CLAUDE.md`)

- Nothing changes without a **task-tree leaf** first.
- Record durable facts/decisions in `docs/decisions/`.
- Commit per `COMMIT.md`; the hooks + CI enforce the doctrines.
- Keep **roadmap ↔ code ↔ docs** in lockstep, always.
