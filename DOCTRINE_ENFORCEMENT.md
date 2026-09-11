# DOCTRINE_ENFORCEMENT.md — how every mechanizable doctrine is enforced

Discipline holds because it is **mechanical**, not remembered. A rule that lives only in a
doc is a suggestion; a rule wired into a git hook + CI is enforced for every agent and
every human, identically.

## Defense in depth (four layers)

- **E1 — discovery.** The doctrine docs: this file, `README.md`, `MEMORY_ARCHITECTURE.md`,
  `TOOLBOX.md`, `COMMIT.md`, and `docs/decisions/`. Where an agent learns the rules.
- **E2 — self-check.** `scripts/check_doctrines.sh` (the driver) + each registered
  `scripts/check_*.sh`. The single source of truth for "which doctrine is enforced by
  what". Runnable by hand anytime.
- **E3 — git hook.** `.githooks/pre-commit` calls the enforcer; `.githooks/commit-msg`
  checks the subject shape. Activate once per clone: `git config core.hooksPath .githooks`.
- **E4 — CI.** The same enforcer runs in CI (`.github/workflows/doctrines.yml`), so a
  locally `--no-verify`'d hook still fails the build. This is the "no matter what" backstop.

## The enforcer registry

`scripts/check_doctrines.sh` carries the **universal** registry:

| ID | Proves | Check |
| --- | --- | --- |
| `MEMORY-ARCH` | the durable 4-layer memory invariants hold | `scripts/check_memory_architecture.sh` |
| `DOCPATH` | tracked `.md` carry no checkout-specific absolute paths | `scripts/check_docpaths.sh` |
| `TASK-TREE-OWNERSHIP` | every staged code change is owned by a task-tree leaf | `scripts/check_task_tree_ownership.sh` |
| `TASK-ACCEPTANCE` | a staged **code** change is owned by a task-tree leaf whose acceptance checklist has ROOT CAUSE / ADDRESSED / NO REGRESSION **ticked**, each backed by tool output **inside that box's own bullet**. ⭐ Box-scoping is the soundness property, not a nicety: it closes two measured leakage holes — a co-staged unrelated leaf supplying the evidence, and a token matched anywhere in the file. ⚠️ Honest limit: it proves the author cited something re-runnable, never that the output is true — the un-fakeable leg is re-running the cited command in CI. Project seams in `.doctrine/` keep it neutral | `scripts/check_task_acceptance.sh` |
| `WAIVER-ROUTING` | a task leaf saying a gate **does not apply** names the leaf that owns fixing it — ⭐ *an author writing a waiver IS the gate reporting a missing capability*, the highest-signal defect report a gate can receive. Deliberately does **not** punish honesty: the waiver stays legal, it just has to name an owner | `scripts/check_waiver_routing.sh` |
| `README-STABILITY` | `README.md` stays a stable landing page — **derived** line AND byte caps (a line cap alone is measurably bypassable: a real project running this spine passed its 60-line layer-A cap while carrying 138,403 bytes), plus **routing-pressure closure** (every destination the README, the policy, or the guard's hint name ends at a governed terminal in `.doctrine/readme_routes.txt`) and the CHANGELOG append-only-history rotation threshold | `scripts/check_readme_stability.sh` |
| `LIVE-DOC-CURRENCY` | no tracked document reports its own currency (`Last updated:` and kin) — git already carries it, and a hand-kept date is right the day it is typed and false the day after; the upstream instrument that scores distinct dates per live surface against a declared charter is a backlog item | `scripts/check_live_doc_currency.sh` |
| `LESSON-PROMOTION` | a NEW dated lesson heading staged in `DEV_NOTES.md` must be either **promoted** (a `docs/knowledge/` change, or a `docs/decisions/` record gaining `answers:`) or **explicitly declined** (`promotion: declined (<reason>)` in the owning leaf) — never silently dropped. Founding measurement upstream: 1 592 lesson entries, none reachable by question, because no gate asked. Evidence archetype: it verifies a decision was RECORDED, not that it was right | `scripts/check_lesson_promotion.sh` |
| `ROUTING-EVIDENCE` | a task leaf that routes a finding **out to another tree** carries a `ROUTING EVIDENCE` section: does the finding reproduce OUTSIDE the family it is sent to, what was measured, what would make the routing wrong. Keyed on the semantics of leaving the tree (the first cut upstream, keyed on a tree-ID spelling, missed its own founding incident); intra-tree routing is not flagged | `scripts/check_routing_evidence.sh` |
| `GAP-CLAIM-CENSUS` | a task leaf that **ADDS** a *"nothing checks X"* claim records the CENSUS it rests on in the same heading section (a command that enumerates a population, or `census: not run (<why>)`). Such a sentence is a universally quantified claim over the whole tree, false the moment one reader exists; staged-diff-scoped (81 pre-existing claims upstream would otherwise teach bypass); `--all` reports the backlog, advisory | `scripts/check_gap_claims.sh` |
| `TABLE-ARITY-RATCHET` | a staged `.md` may not RAISE the number of table rows whose cell count disagrees with their header — GFM silently DROPS extra cells and PADS missing ones, so the page looks fine and the reader loses the rightmost column (26 of 197 rows of a shipped contract upstream, every enforcer green). Per-file ratchet against HEAD; code spans and escaped pipes respected; a fresh minimal implementation with an 8-arm `--self-test` | `scripts/check_table_arity.sh` |
| `KNOWLEDGE-MAP` | the derived Knowledge Map is in sync (if the subsystem exists) | `knowledge-map/scripts/check_knowledge_map.sh` |
| `VISIBILITY-POLICY` | every tracked-Markdown sentence stating a **private** visibility for this repository is reviewed verbatim in `.doctrine/visibility_exceptions.txt`. ⭐ The director's public-repository correction leaked past two hand-run censuses; a census is a one-shot measurement of a moving corpus. A NEW or REWORDED sentence breaches, and so does a STALE exception — the allowlist must describe what is actually there. Matching is deliberately precise: an allowlist thirty entries long teaches bypass | `scripts/check_visibility_policy.sh` (`--self-test`) |
| `BOOK-FRONTIER` | the book may not hold a SECOND, unchecked copy of the task tree's frontier — its roadmap page named a leaf seven commits after that leaf closed. A page may name the frontier only if it matches row 1 of the owning tree's table; naming none passes, because routing to the per-leaf maintained page is the intended shape | `scripts/check_book_frontier.sh` (`--self-test`) |
| `FILE-TERMINATION` | every tracked text file ends with **exactly one** newline — no missing terminator, no blank line at EOF. ⭐ Deliberately NOT a `git diff --check` wrapper: `ROADMAP.md` uses trailing double-spaces as Markdown hard line breaks, so that check's whitespace family contains a legitimate use here and a blanket rule would teach bypass. A blank line at EOF has no legitimate use, and is the exact defect that reached a commit and forced REPAIR-0062. Whole-tree, so the CI backstop is real; 642 files scan in well under a second. Exceptions are verbatim in `.doctrine/file_termination_exceptions.txt` — generated or digest-bound bytes only — and a stale entry is also a breach | `scripts/check_file_termination.sh` (`--self-test`) |
| `BOOK-LINKS` | every intra-book Markdown link resolves to a file that exists. ⭐ `mdbook build` does not validate links, so three dead ones shipped to the rendered book. Their cause is the instructive part: `DOCPATH` requires repo-root-relative references, an author applied that to INTRA-BOOK navigation, and mdbook resolves a link relative to its page — so `cli.md` rendered `href="docs/book/src/cli-state.html"` while the page is `cli-state.html`. The doctrine's intent was satisfied and the navigation broke. External URLs are deliberately out of scope: a network call in a commit hook is a flake generator | `scripts/check_book_links.sh` (`--self-test`) |
| `PROJECT-SPECIFIC` | this project's own doctrines | `scripts/check_doctrines.project.sh` |

**Project-specific doctrines go in `scripts/check_doctrines.project.sh`** (the pluggable
slot) — never in the universal driver. That is where a project adds the equivalent of its
own build gates, format checks, invariant proofs, etc.

## Adding a doctrine

1. Write `scripts/check_<name>.sh` — cheap, deterministic, self-describing; exit nonzero
   with a one-line stderr message on breach. Keep it fast (heavy proofs belong in CI).
2. Register it — universal → the `DOCTRINES` array in the driver; project → append it to
   `scripts/check_doctrines.project.sh`.
3. Mirror it in the table above (this file is the human-readable mirror of the registry).

## The task-acceptance checklist (every code-change leaf must pass)

A task-tree owner is a direct `docs/tasks/<TREE-ID>.md` file, as defined in
`docs/TASK_TREE_README.md`. Nested evidence files may accompany that owner; they
neither need ownership checklists of their own nor substitute for the real tree.
The blank `TEMPLATE.md` remains excluded.

A code change cannot commit until its owning task-tree leaf records all six:

- [ ] **REPRODUCE / ISSUE** — the problem, shown (not asserted).
- [ ] **ROOT CAUSE (WHY + WHERE)** — tool-backed and pinpointed (`TOOLBOX.md`).
- [ ] **FIX** — the change, made at the lowest-risk level that actually works.
- [ ] **ADDRESSED (verified)** — measured before→after (the global metric where one exists).
- [ ] **NO REGRESSION** — the guard set stays green; state how you proved it.
- [ ] **LOCKSTEP** — live docs (`MEMORY.md`, `CHANGELOG.md`, `DEV_NOTES.md`,
  `LIVE_STATUS.md`), the book, and any trackers updated in the SAME commit.
