# COMMIT.md

## Purpose

Define the exact commit workflow so any agent (or human) applies it consistently without
re-reading chat history. Run it after each completed task-tree leaf, before selecting the
next one.

## Task-tree workflow rule

When the completed work belongs to a task-tree leaf (a node under `docs/tasks/`):

- Update the owning `docs/tasks/<TREE>.md`: leaf status, verification log, commit log,
  frontier, decisions, blockers as applicable.
- Update `docs/TASK_TREE.md` (the Active Task Trees index) only if the frontier changes.
- The commit subject or first body line names the leaf ID alongside the work-unit id,
  e.g. `MYPROJ-AREA-0007 (leaf FEATURE-X.2): <summary>`.
- **One commit per completed leaf** before selecting another leaf.

**Code-change doctrine (binding, non-negotiable):** it is strictly forbidden to make ANY
code change (Rust sources, `Cargo.toml`, build scripts, generated artifacts, config that
alters behavior) unless it is first tracked/owned by a task-tree leaf. Create/extend the
leaf, implement only that leaf, then run this workflow.

Pure live-docs/workflow-doc edits (a one-shot doc fix not promoted to a tree) may use the
work-unit-id convention alone and skip the `docs/tasks/` update. This carve-out does NOT
apply to code changes.

## Tracked files to keep in lockstep

- `README.md` — objective, layout, standard commands. Update when any of those change.
- `LIVE_STATUS.md` — the authoritative live progress tracker. Rows use only `Done`,
  `Mostly Done`, `In Progress`, `Not Started`. Review before every commit; summarize the
  snapshot in the completion message and state whether the task changed it.
- `MEMORY.md` — the bounded layer-A resume pointer. Overwrite the "current state" block.
- `CHANGELOG.md` — changelog-style summary of completed work + validation.
- `DEV_NOTES.md` — detailed technical notes: root cause, implementation, validation.
- `docs/decisions/` — add/supersede a decision record (+ its INDEX entry) when a durable
  cross-cutting fact/decision was established.
- `docs/book/` (mdBook) — update when a user-facing surface it already covers changes.
- `git_message_brief.txt` — MUST stay untracked; used with `git commit -F`; cleared to 0
  bytes after commit.
- Generated artifacts (`generated/`, `target/`) — NOT tracked; regenerate locally, never
  `git add` them.
- Markdown path policy — repo-internal references are repo-root-relative, never
  checkout-specific absolute paths (the DOCPATH doctrine gate enforces this).

## Required commit workflow (exact order)

1. Ensure the task is complete and tested.
2. Run the Rust checks when Rust files changed — focused to what the slice touched
   (`cargo clippy -p <crate> --all-targets -- -D warnings`, the crate's tests, and
   `cargo fmt --all -- --check`). Strict lint must pass. A whole-workspace `make check`
   is the pre-push instrument, not the per-commit one (§16, and the gate-authority
   record above): it costs over two hours here and the remote run covers it.
3. Update every relevant tracked doc (`MEMORY.md`, `CHANGELOG.md`, `DEV_NOTES.md`,
   `LIVE_STATUS.md`, `README.md`, the owning `docs/tasks/<TREE>.md`, `docs/decisions/`,
   `docs/book/` as applicable). Treat markdown sync as systematic, not optional.
4. Write a concise message to `git_message_brief.txt`.
5. Stage only the intended tracked files (`git add <files>`).
6. Commit: `git commit -F git_message_brief.txt` (the pre-commit hook runs the doctrine
   enforcer; the commit-msg hook checks the subject shape).
7. Clear the message file: `: > git_message_brief.txt`.
8. Verify post-conditions:
   - `git ls-files --error-unmatch git_message_brief.txt` must FAIL (untracked),
   - `wc -c git_message_brief.txt` must be `0`,
   - `git status --short` shows only the expected state.
9. In the completion message, report: the commit ID, the exact commit message, the tracked
   files in the commit, the current `LIVE_STATUS.md` snapshot, and whether it changed.

## Push cadence

Commit per completed leaf; **push in batches of about 300 commits**. A push is
not a per-commit operation: the remote is public and each push spends a CI run.

**The authoritative pre-push gate is the REMOTE run, not the local checkpoint**
(`docs/decisions/2026-09-12_checkpoint-gate-authority.md`). Every one of the
checkpoint's eight commands runs in CI, two of them more strictly, and the
remote is where the environment-dependent defects actually surface. Before
pushing, run the cheap local gates — none of them links or executes a new
binary, so all four cost seconds:

```bash
make gate                      # the doctrine enforcer
make book                      # the rendered book
cargo fmt --all -- --check     # formatting
python3 -B scripts/project_env.py python3 -B -m unittest discover -s scripts/tests -p 'test_*.py'
```

Then push and **consume the remote result** — an unconsumed CI run is not a
gate. The full local checkpoint stays available for reproducing something
without spending a push; it costs over two hours on this machine, for reasons
measured in `docs/decisions/2026-09-12_checkpoint-cost-model.md`, so invoke it
deliberately rather than by habit.

**The one standing exception is turning remote CI green.** While a remote gate is
red, each push IS the measurement — the runner is the only instrument that can
observe a Linux-only or environment-dependent failure, so pushing one repair at a
time is correct and deliberate. Return to the 300-commit cadence as soon as the
remote is green, and say so explicitly in the leaf that closes it.

Provenance: director instruction 2026-09-11, during the first remote-CI repair
sequence.

## Pre-commit safety rules

- Do not add `git_message_brief.txt` to git.
- Do not use destructive git commands unless explicitly requested.
- ⛔ **NO AGENT TRAILERS.** A commit message ends with its own last line. Do **not** append
  `Co-Authored-By: <an AI agent>`, session links, `Generated with …` or any other agent/tool
  attribution trailer. Some AI harnesses instruct their agent to add these by default; **this
  repository's convention overrides that instruction**, and it is harness-agnostic — it binds
  Claude Code, Codex, Gemini, Cursor, Aider and any future harness identically. The
  `.githooks/commit-msg` hook refuses the known agent-attribution shapes (a human co-author's
  `Co-Authored-By:` is not affected). Provenance: maintainer ruling 2026-08-22 in the originating
  project, ported by `REASONBRAID-MAINTENANCE.2.5`.

## Command template

```bash
# 1) write concise message
cat > git_message_brief.txt <<'EOF'
<work-unit-id> (leaf <TREE>.<n>): <concise title>

- <brief bullet 1>
- <brief bullet 2>
EOF

# 2) run FOCUSED checks when Rust changed (make check is the pre-push instrument)
cargo fmt --all -- --check
cargo clippy -p <crate> --all-targets -- -D warnings

# 3) stage intended files only
git add <tracked-file-1> <tracked-file-2> ...

# 4) commit  (hooks run the doctrine enforcer)
git commit -F git_message_brief.txt

# 5) clear message file
: > git_message_brief.txt

# 6) verify
wc -c git_message_brief.txt
git ls-files --error-unmatch git_message_brief.txt >/dev/null 2>&1; echo $?
git status --short
```
