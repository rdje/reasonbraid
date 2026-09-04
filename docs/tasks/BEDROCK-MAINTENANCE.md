# BEDROCK-MAINTENANCE: keep the discipline spine SOTA, neutral, and self-enforcing

## Metadata

- Tree ID: `BEDROCK-MAINTENANCE`
- Status: `active`
- Roadmap lane: maintain & evolve the spine (this IS bedrock's own roadmap; `ROADMAP.md` is
  the consumer placeholder — see `MAINTAINING.md`)
- Created: `2026-07-24`
- Owner: repo-local workflow

## Goal

Keep bedrock the best-available project-neutral discipline spine: transfer the **general**
improvements that land in PGEN (`../pgen`), keep every spine
file project-agnostic, and keep the whole thing self-enforcing (`make gate` green on a fresh
clone). See `MAINTAINING.md` for provenance, the neutral/specific boundary, and the transfer
process.

## Non-Goals

- Porting PGEN-specific content (grammars, parsers, domain gates) — those stay in PGEN.
- Turning bedrock into a framework/library — it is a *starting structure*, not a dependency.

## Task Tree

- ID: `BEDROCK-MAINTENANCE`
  Status: `active`
  Goal: SOTA neutral self-enforcing spine
  Children: `.1`, `.2`

- ID: `BEDROCK-MAINTENANCE.1`
  Status: `done`
  Goal: initial extraction of the spine from PGEN (memory arch, task-trees, commit workflow,
  doctrine enforcer + universal checks + project slot, tools-first, derived knowledge map,
  mdBook skeleton, Rust workspace, CI, bootstrap + update_scaffold).
  Acceptance: the enforcer runs green on the fresh clone; manifest valid; hooks work.
  Verification: `scripts/check_doctrines.sh` → all checks ✅; `cargo metadata` valid;
  `commit-msg` hook rejects blank / accepts id-shaped subjects; KM generates deterministically.
  Commit: the genesis commit — bedrock spine extraction (see `git log`)

- ID: `BEDROCK-MAINTENANCE.2`
  Status: `active`
  Goal: the standing PGEN→bedrock transfer loop + the improvement backlog below.
  Acceptance: each transferred improvement is neutralized, landed under its own leaf, added
  to `update_scaffold.sh` NEUTRAL if re-syncable, and `DOCTRINE_VERSION` bumped.
  Verification: `.2.1` done (see below)
  Commit: see `.2.1`

- ID: `BEDROCK-MAINTENANCE.2.1`
  Status: `done`
  Goal: adopt the README Stability Policy **and** close the layer-A size-cap bypass the spine
  was itself shipping. Transferred from the reference deployment by direct maintainer order.

  **Why this is one leaf and not two.** The upstream work adopted a README growth guard and,
  in doing so, measured *why* a growth guard needs two caps. The evidence came from the
  spine's own layer-A check: the reference deployment's `MEMORY.md` sat at **60 lines —
  PASSING, exactly at its line cap — and 138,403 bytes** (2,306 bytes per line; one line of
  18,816 bytes). A file the standard calls a *"bounded resume pointer"* was a 138 KB document
  with a green guard.

  ⛔ **And the defect was HERE, in the spine, not only downstream.** `MEMORY_ARCHITECTURE.md`
  §9's reference check prescribed a LINE-only cap, so **every project adopting bedrock
  inherits a bound that does not bind.** Shipping the README policy while leaving that in
  place would have published the *rule* and kept the *counter-example*.

  Measured in this repository before the change:
  - `scripts/check_memory_architecture.sh:14` — `lines=$(wc -l < MEMORY.md)`, cap **120**, no
    byte bound at all (looser even than the deployment that failed).
  - `MEMORY_ARCHITECTURE.md:129/249/315` — §6 rule, §9 reference script and §9.1 knob list all
    line-only.
  - no README size guard of any kind; `README_POLICY.md` absent.

  **Neutralization** (per MAINTAINING.md step 3): the policy text was already project-neutral
  and is copied verbatim. The guard names no domain noun and no upstream project; its routing
  hint points at this spine's own generic homes (`docs/`, `CHANGELOG.md`, `ROADMAP.md`,
  `docs/decisions/`, `docs/tasks/`). The measured evidence is cited as *"a real project
  running this spine"* — the number is what carries the argument, not whose file it was.

  **Cap choice differs from the upstream deployment ON PURPOSE.** Upstream chose caps fitted
  to its own trimmed files. bedrock is a **template**: its caps ship to a consumer whose README
  is not this one. So the README caps default to the policy's own published example
  (**300 lines / 16384 bytes**), generous enough not to false-fail a new project on day one,
  with the policy telling the consumer to tighten them after their own review-and-trim. Both
  are env-overridable. ⛔ The layer-A cap is *tightened* rather than loosened — 120 → **50
  lines** — because the standard itself says *"≤ ~50 lines"* in §6 and in the pointer template,
  so 120 was more than twice as loose as the rule it policed. Byte cap **7168** ≈ 143 B/line at
  50 lines: the shape of a real pointer file, so the two caps bind on the same document rather
  than one shadowing the other. This repo's own layer A measures 30 lines / 1,654 bytes, so it
  has ample headroom under both.

  Acceptance: the new guard REJECTS an over-cap fixture and PASSES this repo's files; the
  retired layer-A guard PASSES a file the new one rejects (proving the change was necessary,
  not cosmetic); `make gate` green; both new files in the `update_scaffold.sh` NEUTRAL
  allow-list; `DOCTRINE_VERSION` bumped; `CHANGELOG.md` noted.
  Verification: see the Verification Log entry for `.2.1`.
  Commit: see the Commit Log entry for `.2.1`.

  ⭐ **REVERSE-FLOW FINDING — the spine is AHEAD of the reference deployment on one check, and
  the documented flow does not anticipate this direction.** MAINTAINING.md states the flow as
  *"PGEN → (generalize) → bedrock"*. But this repo's layer-C check
  (`scripts/check_memory_architecture.sh:23-29`) already **reconciles every decision record
  against `INDEX.md`**, one record at a time — while the upstream deployment asserts only that
  the index has *more than zero* rows. Upstream measured **135 records / 133 index rows** with
  its doctrine green: two records invisible to their own index. ⇒ the fix upstream needs is
  already implemented here. **Reported back rather than silently duplicated.** ⚠️ Honest bound:
  this check is one-directional — it catches a record with no row, not a row with no record.

- ID: `BEDROCK-MAINTENANCE.2.2`
  Status: `done`
  Goal: port `WAIVER-ROUTING` — the highest-value doctrine that passes the neutrality bar
  unchanged — and write that bar down as an explicit admission test for every future port.

  **Why this one, chosen by measurement rather than taste.** The reference deployment enforces
  **15** doctrines; this spine had **4**. Classifying all 15 by domain-dependence *of the logic*
  (comments stripped, since comments legitimately cite the originating evidence):

  | verdict | doctrines |
  |---|---|
  | ⭐⭐ neutral as-is (0 domain nouns in logic) | `MEMORY-ARCH` ✅here · `KNOWLEDGE-MAP` ✅here · **`WAIVER-ROUTING` ← this leaf** |
  | ⭐ portable with 2–4 edits | `ROUTING-EVIDENCE` (3) · `DESTRUCTIVE-TARGET-GUARD` (4) · `GATE-REACHABILITY` (4) · `README-STABILITY` ✅`.2.1` |
  | ⛔ domain-bound by construction | source-of-truth / self-hosting / oracle-anchor / version-currency / flow-integrity checks, plus `TASK-ACCEPTANCE` (10) and `DESIGN-PRIOR-ART` (7) |

  ⚠️ **The first cut of that instrument was WRONG and was thrown away, not tuned.** Counting
  domain nouns across the whole file scored `README-STABILITY` as domain-bound — a doctrine
  already ported cleanly here — because its *routing-hint prose* names domain homes. Comments
  were being read as logic. Re-measured on non-comment lines only, the ranking matched
  independent judgement. ⚠️ Remaining honest bound: strings and heredocs still count as logic,
  so the number ranks candidates; it does not rule.

  ⛔ **`TASK-ACCEPTANCE` is the single highest-value discipline upstream and is deliberately NOT
  ported**: its evidence-signature families name that project's own tools, so a neutral version
  needs a project-declared token list — a design, not a copy. Recorded in the backlog rather
  than half-ported.

  ⭐⭐ **The port FIXED a defect instead of inheriting one.** The origin's version contains
  `printf '%s\n' "$added" | grep -qE "$RE" || continue`. Under `pipefail`, `grep -q` exits at the
  first match, the producer takes SIGPIPE (141), and 141 becomes the pipeline status — so the
  `|| continue` **skips the file** and an unrouted waiver passes silently. It **fails OPEN**,
  the worst direction. Both sites here are written to a file instead. ⚠️ The threshold is NOT a
  flat 64 KiB: measured **65,606 B → PIPESTATUS=(0 0)** but **131,139 B → (141 0)**, i.e. pipe
  capacity plus the consumer's read-ahead. The first CTRL fixture was built on the flat-64 KiB
  assumption and **failed to reproduce** — the probe caught the over-claim, not review.

  Acceptance: 0 domain nouns in the ported logic AND 0 project references anywhere in the file;
  registered in the driver + mirrored; probes RED/GREEN/CONTROL green; `make gate` green; in the
  `update_scaffold.sh` NEUTRAL allow-list; `DOCTRINE_VERSION` bumped.
  Verification: see the Verification Log entry for `.2.2`.
  Commit: see the Commit Log entry for `.2.2`.

  🔜 **Owed back upstream**: the fail-open fix (that project tracks it, latent, as its own leaf).

- ID: `BEDROCK-MAINTENANCE.2.3`
  Status: `done`
  Goal: put the **applicability** question ahead of the **neutralizability** question in the
  admission test, and re-rank the port backlog by benefit rather than by cost.

  **Maintainer directive, 2026-07-30, verbatim:** *"The thing you need to ask yourself before
  porting to bedrock is, is this PGEN specific or project neutral. Is this thing I want to port
  can benefit any present and any future projects objectively."*

  ⛔ **What `.2.2` got wrong.** It wrote the bar down as a *mechanical* test — count domain nouns
  in the logic, require 0 — and ranked the backlog by how cheap each port would be. That measures
  whether a check **can** be neutralized. The prior question is whether it **should** be: does it
  objectively benefit any present and any future project? A check can score 0 domain nouns and
  still encode a workflow only one project needs — **neutral vocabulary, project-shaped
  substance** — and a noun-count waves it straight through.

  ⭐ **The re-ranking is not cosmetic; it changed a verdict, measured.**
  `DESTRUCTIVE-TARGET-GUARD` was ranked a cheap win at 4 domain sites. Its logic hardcodes a
  `Makefile` path and extracts a `clean:` recipe ⇒ what it actually offers is *"benefits any
  project **that builds with make and has a clean target**"*. That is a **conditional**, so it
  fails Q1 and is now **rejected as-is** — the principle is universal, that implementation is not.
  Conversely `ROUTING-EVIDENCE` measures **0** build-system references and presumes only the
  task-tree system this template ships ⇒ it passes Q1 outright and moves to the top.

  ✅ **Retroactive audit of everything already ported — all four PASS Q1**, each stated with no
  project nouns and each presuming only what bedrock itself ships:
  README stability (a landing page becoming a changelog) · layer-A both caps (a "bounded pointer"
  growing unbounded while green) · layer-C reconcile (a record its own index cannot see) ·
  `WAIVER-ROUTING` (an author's report of a gate's blind spot left inert). Nothing to retract.

  ⚠️ No `DOCTRINE_VERSION` bump: `MAINTAINING.md` and this tree are maintainer-only files, not in
  the `update_scaffold.sh` NEUTRAL allow-list, so no re-syncable spine file changed. Stated rather
  than bumped reflexively.
  Verification: see the Verification Log entry for `.2.3`.
  Commit: see the Commit Log entry for `.2.3`.

- ID: `BEDROCK-MAINTENANCE.2.4`
  Status: `done`
  Goal: port the **universal core** of the acceptance-evidence discipline — the highest-value
  doctrine outstanding — after `.2.3` established that the question is *"does this objectively
  benefit any project?"* rather than *"can the nouns be stripped?"*

  **Maintainer's question, verbatim:** *"is there anything there that can benefit any projects
  after stripping out any [project] specificities? Genuine question. If not, then drop it."*

  **Answered by decomposing it rather than by judgement.** There is a substantial universal core:

  | part | verdict |
  |---|---|
  | the checklist itself — ROOT CAUSE / ADDRESSED / NO REGRESSION present **and ticked** | ⭐ universal, zero project nouns |
  | **box-scoped** evidence — the signature must sit in *that box's own bullet* | ⭐⭐ universal, and it is the **soundness** property |
  | default evidence signatures | ⭐ universal *for this template*: standard Rust/Cargo/clippy/test output (this scaffold ships `Cargo.toml`, `rust-toolchain.toml`, `crates/`) + build-flow forensics available in ANY project (`git log -S`, `shellcheck`, `bash -n`, `make -n`, `ENOSPC`…) |
  | the originating project's ~14 tool tokens | ⛔ must not cross — replaced by a **project-declared seam** |
  | the code-change path globs | ⛔ must not cross — seam, with a Rust-workspace default |

  ⇒ the earlier call (*"needs a design, not a copy"*) was right that it is not a copy, and
  **understated** how much is portable. `.doctrine/code_paths.txt` and
  `.doctrine/evidence_tokens.txt` are the seams; both optional, both defaulted.

  ⭐⭐ **The seam is proven load-bearing, not decorative**: `CTRL-4` shows a leaf evidenced ONLY
  by a project-declared token PASSING, and `CTRL-4b` shows the byte-identical leaf FAILING in a
  repo without the declaration. Neutrality that cannot be demonstrated is a claim.

  ⛔ **A PORTABILITY DEFECT WAS FOUND BY THE PROBES, and it would have been invisible in review.**
  The box extractor used `IGNORECASE=1` — a **gawk extension**. BSD awk (the default on several
  platforms) *silently ignores* it, so `root.?cause` never matched `ROOT CAUSE` and **every leaf
  was reported as having no checklist at all**. Rewritten with POSIX `tolower()`. A template must
  run on whatever `awk` the consumer has; this is exactly the class of bug a spine must not ship.

  ⭐⭐⭐ **THE DOCTRINE BLOCKED ITS OWN COMMIT, TWICE, AND BOTH REFUSALS WERE CORRECT.**
  1. **The port was incomplete and the check said so.** It demanded a checklist shape
     `docs/tasks/TEMPLATE.md` never taught — **0** checklist boxes in the template — so a consumer
     would have hit the same wall on their first code change. Fixed by shipping the checklist in
     the template (which is in the NEUTRAL allow-list, so consumers re-sync it).
  2. **The default signature set was too narrow, measured against a real corpus of one — this
     leaf.** The boxes cited `awk version 20200816`, `probes: 3 pass / 6 fail` and `exit=0`: all
     genuinely tool-emitted, none matched. That is precisely the *"a signature family that does not
     fit the real corpus is a gate that teaches authors to waive it"* failure. Generic result
     shapes (`exit=N`, `rc=N`, `N pass / N fail`, version banners) were added **because they are
     what tools print**, not to make the gate easier — and the probes were re-run to prove the
     widening did **not** make it vacuous: `RED-3`, `CTRL-1`, `CTRL-2` and `CTRL-4b` all still
     REJECT (9/0).
  3. ⚠️ **And the third refusal was the check being WRONG — a false positive, stated as such.**
     It treated `docs/tasks/TEMPLATE.md` as a leaf; a template's boxes are *deliberately*
     unticked, so every commit touching the blank form would have been blocked. Excluded (the
     layer-C check in this repo already excludes `INDEX`/`TEMPLATE` — the precedent was one file
     away), and pinned by `CTRL-5`.
  ⇒ Two correct refusals and one false positive, all found by *using* the doctrine on itself
  rather than reviewing it. ⛔ Worth stating plainly rather than presenting three self-blocks as
  three successes: a gate that is only ever exercised on friendly input has not been tested.

  ⭐ The seam is also dogfooded: this repository declares its own probe-driver and driver summary
  lines in `.doctrine/evidence_tokens.txt`, so the template ships a worked example rather than an
  empty extension point.

  ⚠️ **Known overlap, recorded rather than resolved**: `TASK-TREE-OWNERSHIP` already requires a
  staged code change to have an owning leaf, which is `TASK-ACCEPTANCE`'s first leg. The new check
  is strictly stronger, so the older one may be redundant — but removing an existing green check
  is its own decision with its own failure modes, and is not made in passing here.

  Acceptance: 0 domain nouns in logic and 0 foreign tool tokens anywhere; RED/GREEN/CONTROL probes
  green incl. both leakage arms and both seam arms; `make gate` green; in the NEUTRAL allow-list;
  `DOCTRINE_VERSION` bumped.
  Verification: see the Verification Log entry for `.2.4`.
  Commit: see the Commit Log entry for `.2.4`.

  ### Acceptance Checklist (enforced by `TASK-ACCEPTANCE`)

  - [x] **ROOT CAUSE (WHY + WHERE)** — the box extractor silently matched nothing on this
    platform: `awk --version` → `awk version 20200816` (BSD awk), which ignores the gawk-only
    `IGNORECASE`, so `root.?cause` never matched `ROOT CAUSE` and the probe run reported
    `has no 'ROOT CAUSE' box` for every leaf (9 probes, 6 failing identically).
  - [x] **ADDRESSED (verified)** — rewritten with POSIX `tolower()`; before → after on the same
    fixtures: `probes: 3 pass / 6 fail` → `probes: 9 pass / 0 fail`, `exit=0`.
  - [x] **NO REGRESSION** — `bash -n` clean on all edited scripts; the full driver re-run reports
    all doctrines green (8 checks) with `git ls-files` confirming the two new files staged; the
    neutrality counts stayed at 0 domain nouns / 0 foreign tool tokens.
  - [x] **FIX** — replace the gawk extension with `tolower()`, and ship the checklist shape in
    `docs/tasks/TEMPLATE.md` so the doctrine enforces a form the template actually teaches.
  - [x] **LOCKSTEP** — `DOCTRINE_ENFORCEMENT.md` registry mirror, `.doctrine/README.md`,
    `update_scaffold.sh` NEUTRAL allow-list, `CHANGELOG.md`, `MEMORY.md`, `DOCTRINE_VERSION`.

- ID: `BEDROCK-MAINTENANCE.2.5`
  Status: `done`
  Goal: the **day-one batch** of the 2026-09 upstream transfer — the rules a freshly generated
  project needs before its first commit and its first handoff, ported while the maintainer is
  about to create the first project from this template. Three items, each admitted by **Q1**
  (objective benefit to any project) before **Q2** (neutralizable), per `.2.3`:

  | item | Q1 | Q2 | what landed |
  |---|---|---|---|
  | **NO AGENT TRAILERS** (upstream maintainer ruling 2026-08-22) | ⭐ any project: a rule a harness overrides by default is not a rule unless a hook holds it | 0 nouns | `COMMIT.md` pre-commit safety rule rewritten (it had said the OPPOSITE — *"end commit messages with the project's co-authorship trailer"* — so every generated project inherited the wrong default); `.githooks/commit-msg` refuses the known agent/tool attribution shapes and leaves a human co-author's `Co-Authored-By:` alone |
  | **handoff background-job census** (upstream standing rule 2026-08-30) | ⭐ any project: a job that outlives its session rewrites tracked files under the next one | 0 nouns (one message word generalized) | `scripts/check_no_background_jobs.sh` — pattern-free (`lsof` over this uid: an open handle under the repo, or a command line naming the checkout); deliberately NOT a doctrine, a handoff check named in `CLAUDE.md` |
  | **LIVE-DOC-CURRENCY**, the principle (upstream `LIVE-MEANS-LIVE.4a`, 2026-07-31) | ⭐ any project: a hand-kept `Last updated:` is right the day it is typed and false the day after; git carries it | 0 nouns | the field deleted from `docs/tasks/TEMPLATE.md` and this tree; `scripts/check_live_doc_currency.sh` (structural, over `git ls-files '*.md'`, `--self-test` 3 arms) registered as doctrine #7; the upstream instrument that scores distinct dates per surface against a declared charter needs a per-project charter — backlog |

  ⏸ **PAUSED HERE by the maintainer** (2026-09-04, *"pause BEDROCK as soon as you can … I do not
  want you to work on both projects at the same time"*). Part 2 of the same transfer is queued as
  `.2.6`, read and classified but NOT written: `LESSON-PROMOTION` (Q1 ⭐; needs `DEV_NOTES.md`'s
  heading shape and a decline token), `ROUTING-EVIDENCE` (frontier #1, Q1 ⭐), `GAP-CLAIM-CENSUS`
  (Q1 ⭐; 8-arm self-test; fixtures must be re-worded, not just the comments), a fresh minimal
  `TABLE-ARITY-RATCHET` (Q1 ⭐; the upstream self-test is bound to a shipped contract file, so it is
  a rewrite, not a copy), and backlog notes for `BASELINE-IDENTITY` / `IDENTITY-CARRIER-CURRENCY`
  (principle universal, inputs project-bound) and `SCRATCH-SLOT-HEADER`.

  Acceptance: 0 domain nouns in every ported logic path; hook arms observed both ways; the new
  check's self-test observed; `make gate` green with the doctrine registered; both new scripts in
  the NEUTRAL allow-list; `DOCTRINE_VERSION` bumped.
  Verification: see the Verification Log entry for `.2.5`.
  Commit: see the Commit Log entry for `.2.5`.

  ### Acceptance Checklist (enforced by `TASK-ACCEPTANCE`)

  - [x] **ROOT CAUSE (WHY + WHERE)** — the template carried the inverse of the trailer ruling at
    `COMMIT.md` *Pre-commit safety rules* (`grep -n 'co-authorship trailer' COMMIT.md` → 1 hit, the
    line now replaced), a hand-kept currency field in two tracked files (`grep -rn 'Last updated'
    --include='*.md' .` → 2 hits: `docs/tasks/TEMPLATE.md`, this tree), and no handoff census at
    all (`ls scripts/` → 10 scripts, none a census). Each is a rule the originating project holds
    mechanically and this template did not; `probes: 3 pass / 0 fail` on the three gaps.
  - [x] **ADDRESSED (verified)** — `bash .githooks/commit-msg <msg>`: agent `Co-Authored-By` →
    `rc=1`; human `Co-Authored-By: Jane Doe <jane@example.org>` → `rc=0`; `🤖 Generated with …` →
    `rc=1`. `bash scripts/check_live_doc_currency.sh --self-test` → `LIVE-DOC-CURRENCY --self-test:
    3/3 arms`, `rc=0`; the ordinary run → `ok (26 tracked .md files, none self-reports a currency
    date)`. `bash scripts/check_no_background_jobs.sh` → `handoff: OK — no project-owned background
    job is running`, `rc=0`. `grep -ci 'pgen\|grammar\|parser\|probe\|corpus'
    scripts/check_no_background_jobs.sh scripts/check_live_doc_currency.sh` → 0 / 0.
  - [x] **NO REGRESSION** — `bash scripts/check_doctrines.sh` → `=== doctrine enforcement (9
    checks) ===` … `=== all doctrines green ===`, `rc=0` (was 8 checks; `LIVE-DOC-CURRENCY` added,
    every prior check still ✅). `make gate` is that command.

- ID: `BEDROCK-MAINTENANCE.2.6`
  Status: `done`
  Goal: part 2 of the 2026-09 upstream transfer, resumed after the maintainer's pause — the four
  evidence / ratchet doctrines classified in `.2.5`, each admitted by **Q1** before **Q2**:

  | doctrine | Q1 | Q2 | what landed |
  |---|---|---|---|
  | `LESSON-PROMOTION` | ⭐ any project with a notes file and a retrievable layer: a lesson written down but unreachable by question is the measured failure (1 592 upstream) | 0 nouns; `DEV_NOTES.md` heading shape (dated `##`), promotion = `docs/knowledge/` or a decisions record gaining `answers:`, decline token in the leaf | `scripts/check_lesson_promotion.sh` — pure verdict + 9 controls at import |
  | `ROUTING-EVIDENCE` | ⭐ presumes only the task-tree system this template ships (frontier item 1 since `.2.3`) | 0 nouns; messages say component / family | `scripts/check_routing_evidence.sh` — semantic route-out predicate, 5-arm `--self-test` pinning the founding phrasing and the intra-tree / routed-in negatives |
  | `GAP-CLAIM-CENSUS` | ⭐ any project whose leaves make "nothing checks X" claims — a universally quantified sentence over the tree | 0 nouns in logic; the 10 self-test fixtures re-worded (the founding active and passive sentences keep their SHAPE) | `scripts/check_gap_claims.sh` — staged-diff-scoped, section-scoped discharge, `--all` advisory backlog, 10-arm `--self-test` |
  | `TABLE-ARITY-RATCHET` | ⭐ any project with markdown tables: GFM drops extra cells and pads missing ones silently | a REWRITE, not a copy — the upstream self-test is bound to a shipped contract file | `scripts/check_table_arity.sh` — per-file ratchet against HEAD, code spans and escaped pipes respected, 8-arm `--self-test` |

  ⛔ **Two defects found by the ports' own controls, both invisible in review.** (1) The table-arity
  detector was written as `python3 - <<'PY'` and read its markdown from stdin — which the heredoc
  had already consumed — so every RED arm reported 0 and only the GREEN arms "passed"; caught
  because the self-test has RED arms, rewritten as `python3 -c "$PY_SRC"`. (2) A lesson-promotion
  control used `grep -c … | grep -qx 0` under `pipefail`: `grep -c` prints `0` and exits 1, so the
  pipeline failed while the predicate was right; rewritten to capture the count. ⇒ Both are the
  class a template must not ship: a green gate that judges nothing.

  Census of the reserved and domain vocabulary in the four scripts: `grep -ciE 'grammar|parser|ebnf|systemverilog|corpus|regex|pgen' scripts/check_lesson_promotion.sh scripts/check_routing_evidence.sh scripts/check_gap_claims.sh scripts/check_table_arity.sh` → 0 / 0 / 0 / 0 **as corrected by `BEDROCK-MAINTENANCE-0009`**: at `-0008` the same census read 0 / 0 / **1** / 0 — one occurrence of a domain word on a comment continuation line (`# grammar and cannot mis-parse one.`) escaped a re-wording keyed on the one-line phrase, and the leaf published the 0 before re-running the census on the final bytes. ⛔ A census is quoted from its last run on the bytes being committed, not from the run before the last edit. Backlog census for the new claim doctrine on this tree: `bash scripts/check_gap_claims.sh --all` → 1 claim line across 1 file, 0 unbacked. Table backlog: `bash scripts/check_table_arity.sh --all` → 0 defective rows.

  promotion: declined (the lessons of this batch ARE the four doctrine texts and their controls; the notes entry names them and the enforcer carries them)

  Backlog notes (principles admitted by Q1, implementations NOT ported — each needs project-bound
  inputs): `BASELINE-IDENTITY` (a tracked baseline that holds a value derived from the tree says
  WHICH tree — it re-hashes its declared inputs on every run; upstream 1 148 lines bound to that
  project's contracts and build products), `IDENTITY-CARRIER-CURRENCY` (an artifact that tells its
  reader to re-hash its inputs IS re-hashed by a gate over a derived population), `SCRATCH-SLOT-HEADER`
  (a scratch directory carries a header naming its producer and its leaf), the full `LIVE-DOC-CURRENCY`
  instrument (distinct dates per live surface against a declared charter). `GATE-REACHABILITY` stays
  frontier item 2 as a principle. `DESTRUCTIVE-TARGET-GUARD` stays rejected as-is (`.2.3`).

  Acceptance: 0 domain nouns in every ported logic path; every self-test observed green with its
  RED arms; `make gate` green with the four doctrines registered (13 checks); all four scripts in
  the NEUTRAL allow-list; `DOCTRINE_VERSION` bumped.
  Verification: see the Verification Log entry for `.2.6`.
  Commit: see the Commit Log entry for `.2.6`.

  ### Acceptance Checklist (enforced by `TASK-ACCEPTANCE`)

  - [x] **ROOT CAUSE (WHY + WHERE)** — four universal doctrines hardened upstream after the port
    (`.2.5`'s classification) and absent here: `grep -c '^  "' scripts/check_doctrines.sh` → 7
    registered before this leaf, 11 after; the two implementation defects above were located by the
    scripts' own RED arms (`TABLE-ARITY self-test: MISS … want=1 got=0` ×3, then `rc=0`;
    `lesson-promotion: CONTROL MISSED: an undated heading matched`, then `rc=0`).
  - [x] **ADDRESSED (verified)** — `bash scripts/check_lesson_promotion.sh --self-test` →
    `LESSON-PROMOTION --self-test: 9/9 controls`, `rc=0`; `bash scripts/check_routing_evidence.sh
    --self-test` → `ROUTING-EVIDENCE --self-test: 5/5 arms`, `rc=0`; `bash scripts/check_gap_claims.sh
    --self-test` → `GAP-CLAIM-CENSUS: --self-test: arms=10/10`, `rc=0`; `bash scripts/check_table_arity.sh
    --self-test` → `TABLE-ARITY-RATCHET --self-test: 8/8 arms`, `rc=0`; on the staged tree of this
    commit each ordinary run → `ok` / `NOT EVALUATED`, `rc=0`; `probes: 32 pass / 0 fail` across the
    four self-tests.
  - [x] **NO REGRESSION** — `bash scripts/check_doctrines.sh` → `=== doctrine enforcement (13
    checks) ===` … `=== all doctrines green ===`, `rc=0` (was 9 checks; every prior check still ✅).
    `make gate` is that command.

- ID: `BEDROCK-MAINTENANCE.2.7`
  Status: `done`
  Goal: make creating a project from bedrock **dead simple and foolproof** — proven by generating one
  from the PUSHED template and driving it through its FIRST COMMIT, which no earlier trial had done.

  **What the trial found (maintainer question 2026-09-04, *"creating a brand new project from bedrock
  shall be dead simple, fool proof. do you confirm that?"* — answered by doing it):** clone, `bootstrap`,
  `make gate` (13/13) and `make check` all passed on the fresh project, and then the user's very first
  commit — the one that records the bootstrap — was REFUSED by `TASK-TREE-OWNERSHIP` and
  `TASK-ACCEPTANCE`: the crate rename bootstrap made is a CODE change with no owning leaf. A new user's
  first contact with the discipline would have been a wall of doctrine text about a rename the tool
  itself performed. The discipline is right; the template had to ship the leaf.

  **The fix (`scripts/bootstrap.sh`):** on a fresh de-template it seeds `docs/tasks/BOOTSTRAP.md` — a
  done leaf that OWNS the bootstrap, whose ticked checklist carries the evidence that run produced (the
  crate-name count before/after, the hooks path, the enforcer's own summary and verdict lines with
  `rc=0`) — registers it in `docs/TASK_TREE.md`, points `MEMORY.md` at it, and prints the exact
  first-commit command as step 0 of "Next:". The enforcer lines are placeholders filled in step 5,
  because running the enforcer before the Knowledge Map is regenerated fails on `KNOWLEDGE-MAP`
  (measured). Idempotent: the leaf is seeded once.

  ⛔ **Two defects in the fix itself, found by the trial, not by review:** (1) the first cut ran the
  enforcer inside the seeding step (before the map) and its empty `grep | tail` under `pipefail` ended
  the script silently at `rc=1`; (2) `grep -c` prints `0` AND exits 1, so `$(grep -c … || echo 0)`
  yielded `0⏎0`, which split the ROOT CAUSE bullet at a flush-left line and hid its `rc=0` from the
  box-scoped extractor — exactly the `.2.4` soundness property doing its job on the template's own leaf.

  **Follow-up (`-0011`, maintainer question *"where should I be to run make gate and make check?"*):**
  everything after the clone runs INSIDE `<name>/` — and bootstrap itself no longer cares: it acted on
  `git rev-parse --show-toplevel` of the CALLER's directory, so `<name>/scripts/bootstrap.sh <name>` run from
  the parent directory would have failed outside a repository or, worse, de-templated the parent's
  repository. It now resolves its own location and refuses if that is not a clone root. Proven by running it
  from the parent directory: the project, not the parent, was bootstrapped.
  Acceptance: from a fresh clone, `bootstrap → first commit → make gate → make check` all green with no
  hand edits; the bootstrap re-run is idempotent (only `Cargo.lock`, created by `cargo test`, appears);
  `DOCTRINE_VERSION` bumped.
  Verification: see the Verification Log entry for `.2.7`.
  Commit: see the Commit Log entry for `.2.7`.

  ### Acceptance Checklist (enforced by `TASK-ACCEPTANCE`)

  - [x] **ROOT CAUSE (WHY + WHERE)** — on a fresh clone of `c769113` + `bootstrap.sh myproj`, the
    first commit's hook run: `❌ TASK-TREE-OWNERSHIP` / `❌ TASK-ACCEPTANCE: a CODE change is staged
    but NO owning task-tree leaf (docs/tasks/*.md) is. staged code: crates/app/Cargo.toml`, commit
    `rc=1`; `git diff --cached --name-only | grep -E '(crates|src|scripts)/'` → 1 path.
  - [x] **ADDRESSED (verified)** — fresh clone + patched `bootstrap.sh newproj` → `rc=0`, `13 ✅`,
    `✓ docs/tasks/BOOTSTRAP.md seeded with this run's evidence`; the printed step-0 commit →
    `=== all doctrines green ===`, commit `rc=0`, `f3ae296 NEWPROJ-BOOTSTRAP-0001 (leaf BOOTSTRAP.1)`,
    `dirty=0`; `make gate` → `=== all doctrines green ===`; `make check` → `rc=0`
    (`test result: ok. 1 passed; 0 failed`); re-run of `bootstrap.sh newproj` → `rc=0`.
  - [x] **NO REGRESSION** — in this repository `bash scripts/check_doctrines.sh` → `=== all doctrines
    green ===`, `rc=0` (13 checks); `bash scripts/bootstrap.sh` without a name (the source-repo mode)
    does not seed the leaf (guarded on the de-template branch) — `ls docs/tasks/BOOTSTRAP.md` → absent,
    `rc=2`.

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `BEDROCK-MAINTENANCE.2` | `active` | the ongoing transfer loop; pick a backlog item below |
| — | `BEDROCK-MAINTENANCE.2.1` | `done` | README Stability Policy + the layer-A byte cap (0.2.0) |
| — | `BEDROCK-MAINTENANCE.2.2` | `done` | `WAIVER-ROUTING` ported + the neutrality bar written down (0.3.0) |
| — | `BEDROCK-MAINTENANCE.2.3` | `done` | applicability (Q1) put AHEAD of neutralizability (Q2); backlog re-ranked by benefit |
| — | `BEDROCK-MAINTENANCE.2.4` | `done` | `TASK-ACCEPTANCE` universal core ported behind `.doctrine/` seams (0.4.0) |
| — | `BEDROCK-MAINTENANCE.2.5` | `done` | day-one batch: NO AGENT TRAILERS + hook, the handoff census, `LIVE-DOC-CURRENCY` principle (0.5.0) |
| — | `BEDROCK-MAINTENANCE.2.6` | `done` | part 2 of the 2026-09 transfer: `LESSON-PROMOTION`, `ROUTING-EVIDENCE`, `GAP-CLAIM-CENSUS`, a fresh `TABLE-ARITY-RATCHET` (0.6.0) |
| — | `BEDROCK-MAINTENANCE.2.7` | `done` | foolproof project creation: `bootstrap.sh` seeds the leaf that owns its own crate rename, so the FIRST commit passes the hooks (0.6.1) |
| — | `.2.x` — port `ROUTING-EVIDENCE` | `done` (as `.2.6`) | ⭐ passes Q1 outright: presumes **only** the task-tree system this template ships (measured: 0 build-system references). Universal discipline — a finding routed elsewhere must record what was measured, above all whether it reproduces outside the area it is being sent to |
| 2 | `.2.x` — `GATE-REACHABILITY`, principle only | `todo` | universal principle (*a check nothing invokes is indistinguishable from one that does not exist*) but the implementation is **37 lines bound to make + workflow files**. Needs an enumeration seam before it can pass Q1. ⚠️ Its instrument produced six different confident answers upstream before it was right — port the ground-truth controls with it or not at all |
| — | ⛔ `DESTRUCTIVE-TARGET-GUARD` — **do NOT port as-is** | `rejected` | fails **Q1**: hardcodes a `Makefile` path and a `clean:` recipe, so it benefits *any project that builds with make* — a conditional, not an objective benefit. Scored well on Q2, which is exactly why Q2 must not run first. Revisit only with a project-declared target list |

## Improvement backlog (seed — for a future session to pick up)

Concrete candidates, each to become a `.2.x` leaf when worked:

- **Generalize PGEN's `run_with_memory_guard.sh`** into a neutral `scripts/run_with_resource_guard.sh`
  (RAM/disk/timeout guard for heavy jobs) — a broadly useful utility.
- **Add more universal doctrine checks** as they prove general in PGEN: e.g. a commit-message
  co-authorship-trailer check, a "no unguarded destructive make targets" check, a
  "generated artifacts not staged" check.
- **Wire cargo-generate placeholders properly** (`{{project-name}}` substitution + a rename
  hook) if the cargo-generate path is used often — currently naming is done by `bootstrap.sh`.
- **A `docs/decisions/` linter** (every record has the required frontmatter fields).
- **Port memory-architecture refinements** (§ any new enforcement or layer discipline).
  ⭐ Partly discharged by `.2.1` (layer-A byte cap) and `.2.2` (layer-C reconcile was already
  stronger here, and went the other way). Still open: the rest of the upstream enforcement
  surface — see the ranked frontier rows above, which are now evidence-backed rather than a
  wish list.
- **Build the behavioural cross-repo differential** — where both repos implement the same
  invariant, run both against one fixture and compare verdicts. This is the trigger that would
  have surfaced the layer-C gap deliberately instead of by accident. ⛔ A raw text `diff` is NOT
  the answer: 11 of the 12 shared files differ by 15–559 lines, by design.
- **An example filled-in slice** (a tiny worked task-tree + commit) as living documentation.

## Decisions

- `2026-07-24`: bedrock extracted from PGEN as a separate repo; kept out of PGEN's git; its
  own memory layers describe its maintenance (this tree + `reference_bedrock_provenance`),
  so a cold future session picks up with no lost context.

## Blockers

- None.

## Verification Log

- `2026-09-04` — `.2.7`: trial from a fresh clone — `bootstrap.sh newproj` **13/13**, first commit through
  the hooks **green** (`f3ae296`), `make gate` green, `make check` green, bootstrap re-run idempotent. Before
  the fix the same first commit was refused by two doctrines (measured on `c769113`).
- `2026-09-04` — `.2.6`: `make gate` → **13/13 green** (adds `LESSON-PROMOTION`, `ROUTING-EVIDENCE`,
  `GAP-CLAIM-CENSUS`, `TABLE-ARITY-RATCHET`). Self-tests 9/9 · 5/5 · 10/10 · 8/8, every RED arm
  observed; two implementation defects caught by those arms (a heredoc that ate the detector's
  stdin; a `pipefail` control) before the gate ever ran green on friendly input. Neutrality: 0 domain
  nouns in all four scripts. `--all` backlogs: 1 claim line / 0 unbacked; 0 defective table rows.
- `2026-09-04` — `.2.5`: `make gate` → **9/9 green** (adds `LIVE-DOC-CURRENCY`). Hook arms:
  agent trailer `rc=1` · human co-author `rc=0` · `🤖 Generated with` `rc=1`. `check_live_doc_currency.sh
  --self-test` **3/3**; ordinary run `ok (26 tracked .md files …)`. `check_no_background_jobs.sh` →
  `handoff: OK`. Neutrality: 0 domain nouns in both new scripts.
- `2026-07-30` — `.2.4`: `make gate` → **8/8 green** (adds `TASK-ACCEPTANCE`).
  Probes **9 pass / 0 fail** (`docs/tasks/artifacts/task_acceptance/run_task_acceptance_probes.sh`):
  GREEN-1 compliant leaf · RED-1 code with no owning leaf · RED-2 unticked box · RED-3 ticked but
  unevidenced · ⭐⭐ **CTRL-1 evidence present in the FILE but outside the box ⇒ a whole-file grep
  PASSES while the box-scoped check REJECTS** · ⭐ CTRL-2 evidence in a co-staged unrelated leaf ⇒
  rejected · CTRL-3 a pure-docs change is not governed · ⭐ **CTRL-4/4b the same leaf PASSES with a
  project-declared token and FAILS without it** ⇒ the seam, not a hardcoded vocabulary, does the
  work. Neutrality: 0 domain nouns in logic, 0 foreign tool tokens anywhere.
  ⚠️ The probes caught a **gawk-only `IGNORECASE`** in the box extractor which BSD awk silently
  ignores — every leaf read as having no checklist. Rewritten with POSIX `tolower()`.
- `2026-07-30` — `.2.3`: `make gate` → **7/7 green** (docs/process only; no check changed).
  Q1 re-applied to every shipped port — **4/4 pass**, each presuming only files this template
  ships (`README.md`, `MEMORY.md`, `docs/decisions/INDEX.md`, `docs/tasks/`, the driver), all
  verified present. Q1 re-applied to the backlog **changed one verdict**:
  `DESTRUCTIVE-TARGET-GUARD` measured hardcoding `rust/Makefile` + a `clean:` recipe ⇒ rejected
  as-is; `ROUTING-EVIDENCE` measured **0** build-system references ⇒ promoted to top.
- `2026-07-30` — `.2.2`: `make gate` → **7/7 green** (adds `WAIVER-ROUTING`).
  Neutrality: `sed 's/#.*//' scripts/check_waiver_routing.sh | grep -ciE '<domain nouns>'` → **0**;
  project references anywhere in the file (including comments) → **0**.
  Probes **5 pass / 0 fail** (`docs/tasks/artifacts/waiver_routing/run_waiver_routing_probes.sh`):
  GREEN-1 ordinary leaf passes · RED-1 unrouted waiver blocked · **GREEN-2 the same waiver WITH an
  owner passes** (the doctrine must not punish honesty) · **CTRL-1 the shipped form CATCHES a
  343,376-byte staged addition while the unfixed pipe form MISSES it (exit 0)** ⇒ the port closed a
  real fail-open · CTRL-2 an honest SCOPE statement is not bound.
  `bash -n` clean. ⚠️ CTRL-1's first fixture (113,776 B) did **not** reproduce the fail-open; the
  threshold was measured (65,606 B → no SIGPIPE, 131,139 B → SIGPIPE) and the fixture rebuilt past
  the band. The probe caught the over-claim.

- `2026-07-30` — `.2.1`: `make gate` → **6/6 green** (`MEMORY-ARCH`, `DOCPATH`,
  `TASK-TREE-OWNERSHIP`, **`README-STABILITY`** (new), `KNOWLEDGE-MAP`, `PROJECT-SPECIFIC`).
  Live readings: `README-STABILITY: OK — README.md is 73/300 lines, 3571/16384 bytes`;
  layer A 30 lines / 1,654 bytes against caps 50 / 7168.
  **RED arms:** a **13-line / 19,304-byte** fixture — comfortably under the 300-line cap — is
  REJECTED by the byte cap (exit 1), for both the README guard and the layer-A guard.
  ⭐ **CONTROL (the arm that matters):** the **retired** layer-A guard, extracted with
  `git show HEAD:scripts/check_memory_architecture.sh`, returns **exit 0** over that same
  19,304-byte file ⇒ the previous check was genuinely blind, so the change is proven necessary
  by execution rather than argued. `bash -n` clean on all 4 edited/added scripts.
  Neutrality: `grep -ciE 'pgen|grammar|parser|ebnf|systemverilog|regex' README_POLICY.md` → **0**.
  ⚠️ Not verified: no book chapter covers the memory layers or doctrines (`docs/book/` is still
  an introduction-only skeleton), so there was no book surface to sync — stated rather than
  implied.

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-07-24` | `.1` | enforcer (5 checks) · cargo metadata · commit-msg hook · KM gen | all green |

## Commit Log

- `2026-07-30` — `.2.2` — `BEDROCK-MAINTENANCE-0004`: port `WAIVER-ROUTING` (fixing its
  fail-open), write down the neutrality bar and the both-ways transfer rule;
  `DOCTRINE_VERSION` `0.2.0` → `0.3.0`.
- `2026-07-30` — `.2.3` — `BEDROCK-MAINTENANCE-0005`: put applicability ahead of
  neutralizability in the admission test; re-rank the backlog by benefit; reject
  `DESTRUCTIVE-TARGET-GUARD` as-is.
- `2026-07-30` — `.2.4` — `BEDROCK-MAINTENANCE-0006`: port the `TASK-ACCEPTANCE` universal core
  behind `.doctrine/` seams; `DOCTRINE_VERSION` `0.3.0` → `0.4.0`.
- `2026-09-04` — `.2.5` — `BEDROCK-MAINTENANCE-0007`: the day-one batch of the 2026-09 transfer —
  NO AGENT TRAILERS (rule + `commit-msg` hook), the handoff background-job census, the
  `LIVE-DOC-CURRENCY` principle (fields deleted, check registered); `DOCTRINE_VERSION` `0.4.0` → `0.5.0`.
  Paused by the maintainer before part 2 (`.2.6`).
- `2026-09-04` — `.2.6` — `BEDROCK-MAINTENANCE-0008`: part 2 — `LESSON-PROMOTION`, `ROUTING-EVIDENCE`,
  `GAP-CLAIM-CENSUS` ported (neutralized, fixtures re-worded), `TABLE-ARITY-RATCHET` rewritten minimal
  with an 8-arm self-test; backlog notes for the input-bound principles; `DOCTRINE_VERSION` `0.5.0` → `0.6.0`.
- `2026-09-04` — `.2.6` — `BEDROCK-MAINTENANCE-0009`: CORRECTION — the neutrality census the `.2.6` leaf
  published (0 / 0 / 0 / 0) was 0 / 0 / 1 / 0 at `-0008`; the word is re-worded and the claim corrected in place.
- `2026-09-04` — `.2.7` — `BEDROCK-MAINTENANCE-0011`: `bootstrap.sh` acts on the repository it lives in, not on
  the caller's directory; proven from the parent directory.
- `2026-09-04` — `.2.7` — `BEDROCK-MAINTENANCE-0010`: `bootstrap.sh` seeds `docs/tasks/BOOTSTRAP.md` (a done
  leaf owning the bootstrap, evidence from the run itself) and prints the first-commit command; proven on a
  fresh clone through the first commit. `DOCTRINE_VERSION` `0.6.0` → `0.6.1`.

- `2026-07-30` — `.2.1` — `BEDROCK-MAINTENANCE-0002`: adopt the README Stability Policy and
  close the layer-A size-cap bypass; `DOCTRINE_VERSION` `0.1.0` → `0.2.0`.

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `.1` | genesis commit — bedrock spine extraction | full spine, verified green |

## Changelog

- `2026-07-24`: Created the maintenance tree; `.1` spine-extraction done, `.2` transfer loop active.
