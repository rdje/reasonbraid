# AGENTS.md — harness-neutral entrypoint

Different AI harnesses auto-read different bootstrap files (`CLAUDE.md`, `AGENTS.md`,
`.cursorrules`, `GEMINI.md`, …). They all point to the same place.

**→ Read [`CLAUDE.md`](CLAUDE.md) first.** It is the canonical agent bootstrap and is not
Claude-specific — the name is just the file one common harness auto-loads. Every rule in
it applies to any agent.

The discipline this repo enforces (durable memory, task-tree ownership, strict commits,
mechanical doctrine gates) is defined in `MEMORY_ARCHITECTURE.md`, `TOOLBOX.md`,
`DOCTRINE_ENFORCEMENT.md`, and `COMMIT.md`, and is enforced by git hooks + CI — so it
binds you regardless of which harness you are.
