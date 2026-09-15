# CHANGELOG.md

## 2026-09-15 — The server says where a publication may be written, not the caller (`SIGNOFF-REPAIR.9.2.1.1`)

`POST /v1/policy-publications/{id}/publish` read `repo_path` out of the request body and handed it to `gix::open`. Any enrolled principal named any path on the server's filesystem, and the module's own header said the publication went "into the LOCAL bare repository".

- **The deployment now declares one root** (`rb-server --publication-repo-root`), the request names a location inside it, and `publisher::resolve_repository` is the single predicate that decides. ⭐ **Both sides are canonicalized before they are compared**, which is what lets one question answer two escapes: `..` is a walk the caller spells, a symlink is one the filesystem spells on the caller's behalf — and in the symlink case every component the caller named *is* inside the root, so a string containment test refuses the first and admits the second.
- **Fail-closed, and that is the contract rather than a detail.** `api_router(pool)` keeps its signature and declares no root, so an unconfigured deployment answers the new `503 publication_repository_unconfigured` instead of opening whatever arrived in the body. A declared root that does not resolve to a directory refuses the **boot**, before the migrations run, so a typo cannot leave a changed database behind with no server on it. ⛔ `.11.12` is untouched: the secret-store ordering and the `--host` gate remain its to decide.
- 🔴 **The repair exposed a pre-existing fixture defect, in the very leg it was told to preserve.** `the_publish_verb_drives_the_git_half`'s "a publish into a NON-repository path refuses (the store contract's open failure)" ran **after** the publication had been driven to `effective`, so the STAGE check answered it and `gix::open` was never reached — green for the wrong reason for as long as it has existed. Found by pointing the leg inside the configured root and watching it fail with ``is at stage `effective` `` instead of "does not open". It now runs first, while the publication is staged, and asserts the state afterwards so a reordering fails loudly.
- **Verified:** `bash scripts/run_pg_tests.sh policy` → `13 passed; 0 failed`, rc=0; `cargo test -p reasonbraid-server --test publisher` → `6 passed; 0 failed`, rc=0. The new live control carries **6 legs**, the offline one **8 arms**.
- ⚠️ **Falsified by a matched pair rather than by neutralizing the shipped predicate, and the reason is recorded rather than hidden**: the harness refused the source weakening as a security change. The contrast is stronger anyway because it is **durable** — the same three locations, against a server whose declared root is the directory above, are now *accepted* and reach the record exactly as a legitimate location does. One knob moves; the legs flip. A neutralization would have proved the same thing once and left nothing behind.
- ⚠️ Two limits stated rather than implied, in the leaf and in the book: containment is decided once per request against the filesystem as it then stands, so a symlink swapped before `gix::open` would not be seen; and the verb is still authorized by **enrolment alone** — that is `.9.2.1.2`, unchanged here.
- ⛔ **A new wire code, with the population it moves pinned.** `publication_repository_unconfigured` is an `ext` code, the documented forward-compatible path, and touches no frozen document. The census now reads **19 emitted, 10 unregistered, 0 undocumented**; `.11.7.1` owed a verdict on nine and now owes one on ten, pinned at `2e72571` so the correction is visible rather than silent.
- ✅ **`.9.2.1`'s build-cost note was re-measured, as it asked.** `cargo check -p reasonbraid-server --tests` finished in **2 m 39 s**, rc=0 — not the ">10 minutes" the decomposition recorded. ⛔ The two numbers answer different questions rather than contradicting each other, exactly as that note predicted: it measured a contended machine with a cold cache. 🔴 **And the re-measurement found the real mechanism, which is not CPU contention at all**: the first attempt sat at **3.84 s of CPU across 6 minutes**, blocked with all four threads asleep and its dep-graph file byte-identical, and cleared only after 3.8 GB of `.rmeta` was pulled into the page cache. The completed run spent **2 m 39 s wall for 46 s user**; the lib-only check, **3 m 01 s wall for 5.8 s user** — about 9 % CPU. This build is I/O-bound against the external repository volume. Evidence for `docs/decisions/2026-09-12_checkpoint-cost-model.md`, not a change to it.

## 2026-09-14 — Three repairs were sharing one leaf (`SIGNOFF-REPAIR.9.2.1`)

`.9.2.1` carried three acceptance clauses over the publication verbs. They are three different repairs: one invents server configuration and a path-containment predicate, one changes the **wire contract** of two shipped verbs, and one adds a repository read to a database transition. They shared a leaf because one reviewer found them together, not because they land together.

- **Decomposed into `.1` (repository root), `.2` (authority binding) and `.3` (object existence)**, each with its own acceptance and its own control observed RED before its fix — `.11.13`'s precedent, which converted eight containers into leaves that can actually be finished.
- ⚠️ **Two facts settled it rather than taste.** The acceptance requires each control RED first, and `cargo check -p reasonbraid-server --tests` does not complete inside ten minutes on this machine — so one commit would mean three unrelated repairs sharing a single red-to-green cycle, and a bisect could not separate them.
- ⚠️ The children are **ordered**: `.1` invents the configured root and `.3` needs a repository to look objects up in. `.2` is independent.
- **Measured while writing the children, not asserted:** `git grep -n '"git_object_ids": \["abc123"' crates/reasonbraid-server/tests/policy.rs` → **3** sites, one of them the deployment fixture. The suite meant to qualify the transition exercises it with ids that do not exist, which is why `R-73-74-3`'s framing is preserved as the stronger claim.
- Pending rose 53 → **55**, which is the healthy direction: three things carried as one unfinishable leaf are now three with acceptance attached.

## 2026-09-14 — The derived map was generated from the working tree and committed against the index (`SIGNOFF-REPAIR.11.4.5.4`)

`.githooks/pre-commit` regenerates `KNOWLEDGE_MAP.md` and stages it, so *"map-drift is structurally impossible"*. But the generator enumerated its sources with `ls docs/{tasks,decisions,knowledge}/*.md` — the **working tree** — while the map it staged was committed against the **index**. An untracked file in any of those directories entered the committed map as a link the commit does not contain.

- ⛔ **Found in this session's own commit.** `git show 991bbf8:KNOWLEDGE_MAP.md | grep -c 'a-control-is-calibrated…'` → **1**; `git ls-tree --name-only 991bbf8 docs/knowledge/` → **0**. A checkout of that commit has a map whose link does not resolve.
- ⛔ **Widening the census changed the answer.** `docs/knowledge/` alone gave 2; all three families the generator actually lists gave **3 of the 139** commits touching the map — the third, `0558ed6`, is a `docs/decisions/` record the narrow scan could not see. `MEMORY.md`'s standing *"scope a census whole"* warning names this failure and it still caught me, one leaf after the last time.
- ⭐ **All three share one shape**: a commit sequenced BETWEEN the work that creates a source file and the commit that adds it. Two are changelog rotations — and `.11.4.1.2` *requires* a rotation to be its own commit, so the spine's correct sequencing rule is exactly what exposed the generator's wrong source.
- **Fix:** derive from the commit — `git ls-files -- ':(glob)docs/<dir>/*.md'` to enumerate, `git show ":$f"` to read content. Removing the failure mode rather than detecting it, per `.11.4.5.3`'s ruling. It also stops an **unstaged** `answers:` edit being published into the map, the same root cause reached by the same reasoning.
- 🔴 **The first cut of the fix would have published 54 broken links.** A git pathspec wildcard **matches `/`** by default, so `git ls-files -- 'docs/tasks/*.md'` returns **69** paths where `ls` returns **15**, reaching into `docs/tasks/artifacts/`; with the link built from `basename`, every extra would have rendered as a link to a file that is not there. Caught by diffing the fresh render against the committed map — the one comparison that could see it — and corrected with `:(glob)`, fnmatch with `FNM_PATHNAME`. ⚠️ `MEMORY.md` warns that `*.rs` does **not** recurse; this is the same confusion in the opposite direction, with the two tools disagreeing on one glob.
- **Verified:** the corrected generator reproduces the committed map **byte-for-byte**, so the repair changes the failure mode and not the output. New `--self-test`, 7/7 arms in a throwaway `git init` repo under the repository-derived scratch, **falsified three ways**, each variant reddening only its own arms. `make gate` 18 checks green; `SELF-TEST` now covers **24** instruments at 1.59 s.
- ⛔ The three historical commits are **not** rewritten; each self-corrected within one commit. The repair stops the next one.

## 2026-09-14 — The arity gate modelled the opposite of the renderer, and its self-test agreed (`SIGNOFF-REPAIR.11.2.3`)

`scripts/check_table_arity.sh` flags Markdown rows whose cell count disagrees with their header — the defect matters because GFM **silently** drops the extras and pads the missing, so the page looks fine and the reader loses a column. Its splitter treated a pipe inside an inline code span as part of the cell. GFM does the opposite: a row is split into cells **before** inline parsing, so only a backslash escape protects a pipe.

- ⛔ **A green whole-corpus scan was measuring the wrong predicate.** Two rules run side by side over the same 323 tracked files at `dd4151b`: the shipped rule finds **0** arity-defective rows, the renderer's rule finds **1**. The leaf recorded 2; re-measured rather than inherited, because `RECONCILIATION.md:111` was genuinely repaired by REPAIR-0175 — so the one that remains is exactly the retained falsification target.
- ⭐ **Asked the renderer nine times before asserting anything.** A throwaway mdbook 0.5.2 book under the repository-derived `TMPDIR` (§13), cell counts read out of the emitted `<tr>`/`<td>`: `` | `x | y` | 2 | `` → **2 cells**, backticks LITERAL, the `2` **discarded**; `` | x \| y | 2 | `` → 2 cells; `` | `x \| y` | 2 | `` → `<code>x | y</code>` and `2` — literal pipes *and* the code span, which is the repair form.
- 🔴 **The `--self-test` was why it survived, not what would have caught it.** Its arm *"a pipe inside a code span is not a separator"* asserted `want=0` for the exact shape the renderer scores as a defect. Written from the same reading as the code, it could not disagree with it — `TOOLBOX.md`'s measured blind-spot rule, reproducing exactly.
- ⛔ **A wrong model is wrong in both directions.** The old parser also scored an unpaired backtick run as a defect; the renderer emits 2 cells and no defect. That arm was asserting a **false positive**, and correcting the rule removed it rather than preserving it.
- ⚠️ **The damage was on this project's own doctrine page.** The `INDEX-FRONTIER` registry row wrapped `` | — | `` in a code span, so the renderer cut the cell at *"8 completed trees write `"* — **516 of 1,027** characters — and published a bare `—` where `scripts/check_tree_index_frontier.sh` belongs. ⛔ The leaf's own note said the cut was at *"fixed this exact"*; measured, it is later, and the record is corrected rather than repeated. Repaired to `` `\| — \|` `` and re-rendered: **3 cells**, enforcer named, raw HTML `<code>| — |</code>`.
- ⭐ **The leaf's warning that the ratchet would block its own fix did not materialise**, and the reason is recorded instead of the relief: the ratchet pipes both `git show ":$f"` and `git show "HEAD:$f"` through the *same* working-tree parser, so a rule correction moves `now` and `before` together — here 0 vs 1, a fall, promoted silently. Same-commit repair was still right, for the published page rather than for the gate.
- **After:** 9/9 self-test arms, every one the renderer's verdict named in the arm's own title; `--all` → **0**; `make gate` → 18 checks green.
- **Promoted:** `docs/knowledge/a-control-is-calibrated-against-the-renderer.md` — a control over a format you did not implement is calibrated against the consumer, never the spec.

## 2026-09-14 — Graded on challenge: the findings stand, three decorations did not (`SIGNOFF-REPAIR.11.9.1`)

The director asked *"do you stand by your findings?"* — `docs/CLAIM_VERIFICATION.md` §4.1's grading question. Graded on all three axes, separately.

- **PROSE — stands, all three repairs.** The MCP reads ran a copy that omitted the authorization; the policy registry is site-global; citing an authority was the same as holding one on three surfaces; the call-response defect is in the shared core. Each was reproduced live and separately falsified.
- **NUMBER — stands.** 8→0, 6→0 and 5→0 legs are test-binary output, deterministic and re-derivable.
- **NAMED INSTANCE — three failed, and a named instance has no tolerance band.**

1. ⛔ **A carried line count I staled myself.** "the 728-line `policy.rs`" was measured, then the file was edited by REPAIR-0184 in the same session; it is now **731**. Removed at all **6** sites rather than refreshed — §5B: derive or delete, never re-carry. The claim it decorated (`git grep -n "tenant"` → rc=1) was re-checked and still holds after the repair.
2. ⛔ **A classified count published against a command that does not yield it.** "five sites" was correct as a classification and was published beside a raw `git grep`, so a reader re-deriving gets a different number — and the population itself was wrong, because `src/*.rs` is a glob that does **not recurse** and missed `src/authority/selection.rs`. Pinned at all **4** sites to `aee579a`: **9 sites, 5 of which ask the grant-validity question in four spellings**. ⚠️ `MEMORY.md`'s standing "scope a census whole" warning names this exact failure, and it still caught me.
3. ⛔ **An over-count assembled while writing.** "the third header asserting a check nobody wrote" is **two**. The third, `lifecycle.rs`'s approval comment, is accurate about its SQL — the check existed and matched a subject; the defect is that the subject was an unverified claim. Folding it in made the generalization fit a population it does not describe.

⭐ **The lesson is the shape, not the three instances**: every one of them is a number attached to a finding as *decoration* rather than as the finding's evidence. The evidence legs were re-derived, falsified and durable; the ornaments were none of those, and nothing was watching them.

## 2026-09-14 — The seam was named; the core was the defect (`SIGNOFF-REPAIR.6.1.2`)

- **Measured live before the repair, on BOTH surfaces:** a role enrolled in one tenant recorded a `decline` and a `recuse` against another tenant's recruitment call — through the MCP seam **and over plain HTTP**. A new seven-leg control reported **5 breaching**.
- 🔴 **The leaf named the MCP seam; the HTTP verb over the same core takes no tenant at all** and was equally open. ⛔ A repair at the seam would have left that verb wide open **and the seam's own suite green** — which is the argument for where the binding went: into `respond_to_call_core`, derived from the call, so both callers get it at once.
- ⛔ **The fixture was wrong first and three legs were green for the wrong reason.** The control posted to `/v1/calls/{id}/responses`; the route is `/respond`, so every HTTP leg was reading a 404 as a refusal — the positive leg included. The corrected fixture moved two breaches onto the surface the leaf had not implicated.
- ⛔ **One leg was removed rather than kept green.** The acceptance said "every response kind"; `join` is refused earlier, by the absence of an enrolled node, so it would pass without the binding ever being reached. The two kinds that skip the eligibility gate carry the claim.
- ⭐ **The proposal half was settled by measurement, as the leaf directed**: `register_proposal` has no grant check and no audit row — and neither does the HTTP verb behind it. The tool is not weaker than the HTTP surface, so the seam's header was describing a control neither has. The header now states what each of the three verbs actually brings, separately, because they are not uniform.
- **Promoted:** when two surfaces share a core, the binding belongs in the core — and the giveaway is a seam-level repair that leaves the seam's own suite green (`TOOLBOX.md`, `docs/knowledge/`).
- After: **0 of 7 legs breach**, 5/5.

## 2026-09-14 — Citing an authority is not holding one (`SIGNOFF-REPAIR.9.3.1`)

- **Measured live before the repair:** a principal in one tenant recorded a policy **retraction** under another tenant's grant, registered a deployment target owned by it, recorded a correction under a grant whose `valid_from` was tomorrow, and filed a policy **approval as another principal**. A new nine-leg control reported **6 breaching**.
- ⛔ Reachable rather than theoretical: the dev enrolment mints `grt_<principal_id>`, so naming another principal's grant needs nothing but their id.
- **The census took the family whole** — `git grep -n "FROM authority_grants" aee579a -- crates/reasonbraid-server/src/` (pinned to the pre-repair commit, because this repair CHANGED the population) returns **9 sites**, of which **5 ask the grant-validity question** — `corrections`, `deployments`, `lifecycle` and `policy` twice — **in four different spellings**; the other 4 ask different questions. ⛔ The population number was first taken with `src/*.rs`, a glob that does NOT recurse, and it missed `src/authority/selection.rs` — and `git grep -n "valid_from"` over those modules returns **rc=1**: not one of them consulted the column, though it is `NOT NULL`. The `expires_at IS NULL` arm four of them carried was dead for the same reason.
- ⭐ **The one site that bound a subject was the one that had had the attention, and it was still half-bound** — it matched the grant to `input.approver`, a string off the wire that nothing tied to the caller. A third instance of the same mechanism, which no leaf had named; repairing the two the leaf named would have shipped half a repair.
- **The fix is one definition, two shapes**: `authority::grant_is_live` and `authority::grant_held_by`. Three surfaces take the held-by form and receive the principal their handlers had already resolved and were discarding.
- ⭐ `.9.1`'s measured asymmetry is settled — registration no longer accepts a grant the resolver in the same module would refuse — while `.9.1`'s semantic question, whether a policy's owner must be the registrar's own grant, is deliberately left to it.
- ⚠️ **One acceptance leg was not expressible and is corrected rather than quietly weakened**: `GrantAction` and `TargetSelector` cannot NAME a publication, a deployment target or a correction, so "the grant's action covers the target" has nothing to compare. Opened as `.9.3.4`. This is the second such leg this session.
- **Falsified one arm at a time**: subject binding → legs A and B; `valid_from` → leg C; approver-is-the-caller → leg G alone.
- ⚠️ The falsification found a defect in the control itself: two approval legs shared a proposal, and a successful approval advances it, so each reported the other's outcome. Each leg now has its own.
- After: **0 of 9 legs breach**, 12/12, with the eleven pre-existing policy tests unchanged under a stricter registration path.
- Pending leaves 55 → 55: one closed, one opened. ⭐ Written 56 from arithmetic first; the tree's own re-derivation said 55.

## 2026-09-14 — A claim of sameness is worth its call graph (`SIGNOFF-REPAIR.6.1.1`)

- **The severest defect this reconciliation produced, repaired.** `reasonbraid-mcp` carried a private re-implementation of the server's three inspection reads while its own header claimed *"the SAME queries + the SAME authorization as the HTTP handlers"*. The copy ran neither: `list_inbox` declared a `principal` it never read, `get_policy_bundle` never read one either, and `get_thread` computed the foreign-reader class and returned the full projection beside it.
- **Measured live before the repair, on a real cluster:** a principal in tenant A received tenant B's ENTIRE thread projection — subject, objective, participants, creator, budget, ceiling id — with `"visibility":"network"` printed next to it. A new fourteen-leg control reported **8 legs breaching**.
- **The repair removes the failure mode instead of patching it.** A new `mcp_read` seam in the server crate (`mcp_read_internal`, beside `mcp_write_internal`) calls three halves now shared with the HTTP handlers — `authorize_inspection`, `thread_inspection`, `inbox_inspection` — and the MCP crate's copy is DELETED. Sameness is a call, not a sentence.
- ⚠️ **One part of the original finding was corrected by measurement rather than confirmed.** It read the policy bundle as a cross-tenant leak. `git grep -n "tenant"` over the registry's schema and its whole module returns **rc=1**: there is no tenant column and no site filters by one. The registry is site-wide on BOTH surfaces. What stood: no `WHERE`, no principal check, and a bundle labelled with the caller's tenant. What fell: the framing. The tool now runs the HTTP surface's enrolment gate and returns no tenant label; the schema question is opened as `.6.1.5` rather than answered by a filter on one read.
- **Falsified one arm at a time**, each neutralization proved landed by a gate-call census and each rc read directly: removing the thread gate → 3 breaches, leg A only; the inbox gate → 2, leg B only; the enrolment check → 1, leg C only. No arm carries another's proof.
- ⛔ `git diff` was useless as the landing proof — `mcp_read.rs` is a NEW file, so an unstaged edit to it produces no diff at all, and the first falsification's proof printed nothing while succeeding.
- Side effects, each deliberate: the MCP thread read now applies the `.1.3.1` derived view and the inbox read the `node_inbox_state` view, matching HTTP; `get_policy_bundle` loses its tenant parameter and the conformance golden asserts that absence; `chrono` becomes a dev-dependency.
- Pending leaves 54 → 54: one closed, one opened.
- **Promoted:** a claim of sameness is worth exactly the call graph that enforces it (`TOOLBOX.md`).

## 2026-09-14 — Routing is not ownership (`SIGNOFF-REPAIR.11.13`)

- **The director's correction.** This session surfaced six findings and reported an owner for each. Audited mechanically, **8 of the 10 named leaves carried no `- Acceptance:` line of their own and 6 had no children** — they could be read, not finished.
- ⭐ **The one properly-owned finding got that way by accident of vocabulary**: `.11.12` exists because the clause ledger classified its clauses `unowned`, and that state forces a new leaf. `owned` and `attach` measure whether a clause is *visible* to a leaf; neither asks whether the leaf is *executable*.
- **Eight containers censused and split into eleven bounded leaves**, each with its own reproduce, owns and acceptance, and each acceptance naming a control to be observed red before its repair: `.6.1.1`–`.6.1.4` (the MCP reads, the write seam's enrolment-as-authority, the untyped body index, the quota semantics), `.9.2.1` (the publish verbs' unbound path and fabricated Git ids), `.9.3.1`–`.9.3.3` (citing an authority equals holding one, a publication reviewable once for ever, the declared-versus-deployed digest), `.11.2.3` (the table-arity parser and the row still publishing broken), `.11.4.2.2` (the `MEMORY.md` cap).
- **Frontier re-ranked by severity** — the MCP cross-tenant read leak leads.
- ⚠️ The re-run audit reported two leaves as lacking acceptance and the audit was wrong; both were written `- Acceptance, …`. Fixed in the leaves, not by loosening the matcher.
- Pending leaves 47 → 54: the count rises because routing became ownership.
- **Promoted:** a leaf that cannot be picked up and finished is not an owner (`TOOLBOX.md`).
- No product code, schema, test or script changed. 18 doctrines green, `mdbook build` rc=0, `git diff --check` rc=0.

## 2026-09-14 — Correcting a number this activity published (`SIGNOFF-REPAIR.11.9.1.3.1`, tranche 4a)

- **Tranche 4 sized at 10,519 characters — 4.12x the proved-executable size — and split five ways** on each record's own narrowest candidate, with one declared deviation: a 423-character singleton folded into its sibling in the same tree family.
- 🔴 **A number this reconciliation itself published was wrong when written.** The clause ledger said a fixture accepts an unbound verdict digest "in four places"; `grep -c` says seven, across seven distinct test functions, and `git show` at the original commit proves the corpus did not move. Both sites corrected, the superseded figure named. **Promoted:** a count written while reading is not a count (`TOOLBOX.md`).
- 🔴 **Any enrolled principal can act as any authority whose grant id they know.** The corrections module's authority check takes the grant id and nothing else — no caller, action, selector, boundary, publication or valid-from — behind an endpoint that admits on enrolment alone. The deployments module repeats the shape.
- 🔴 **No second review can ever be scheduled for a (publication, trigger) pair**: the review id is deterministic and is the primary key, and the colliding insert's error is discarded. The same discard makes a storage failure return a successful empty schedule.
- 🔴 **`rb-server` migrates the database on the line before it validates the profile it refuses to boot without**, and its bind address carries no predicate at all. New leaf `SIGNOFF-REPAIR.11.12`, with the two halves to be decided separately.
- ⚠️ Two fixtures declare digests bound to nothing, and a metrics test's comment claims a counter assertion the test does not make.
- ⭐ The fifth two-records-one-finding pair: one waiver satisfying a trigger named "repeated", reached independently by two records.
- Validation: `--classified` 204 rows clean with all 40 `attach` clauses named by their owners, `--self-test` 49 controls, 18 doctrines green, `mdbook build` rc=0, `git diff --check` rc=0. No product code, schema, test or script changed.

## 2026-09-14 — Measure a deadlock as a ratio (`SIGNOFF-REPAIR.11.9.1.2.3`, tranche 3 complete)

- **Tranche 3c reconciled five records into 22 clause rows**: 16 `owned`, 6 `attach`, 0 `unowned`. Tranche 3 is complete at 15 records and 69 clauses.
- 🔴 **The browse worker waits for its child to exit before reading the child's piped stdout**, and the request it writes allows 4 MiB of output against an OS pipe buffer of at most 64 KiB — so the deadlock covers most of the intended range, not an edge case, and surfaces as a timeout. Its error path drops a live `Child` without killing or waiting; its timeout path kills the worker but not the browser beneath it.
- 🔴 **Four adapter defects, each at two sites**, because `codex.rs` and `claude.rs` share a shape: a byte slice of a `String` that panics off a char boundary, an unbounded line read under a bound checked before the append, a child map that is never removed from, and a success path that returns without awaiting the drain or waiting the child while the failure path does both.
- 🔴 **A deadline is computed, shipped, and read by nobody** — one grep hit across every adapter source and the supervisor, and it is the struct field's own declaration.
- 🔴 **A completed item that reaches the retry gate is dead-lettered and auto-quarantined**, and that report never acknowledges its own journal row while its only sibling call site does.
- ⚠️ **One clause was narrowed rather than confirmed** — `OutcomeUnknown`'s propagation is real and documented as deliberate by the type itself.
- ⚠️ **The narrowest candidate was the owner of only one of the five records**, the clearest instance yet that a record's candidate list is a suggestion.
- **Promoted:** prefer a ratio, a single-hit grep or a two-site contrast to a reading (`TOOLBOX.md`).
- Validation: `--classified` 179 rows clean with all 38 `attach` clauses named by their owners, `--self-test` 49 controls, 18 doctrines green, `mdbook build` rc=0, `git diff --check` rc=0. No product code, schema, test or script changed.

## 2026-09-14 — The sentence at the top of the MCP module (`SIGNOFF-REPAIR.11.9.1.2.2`, tranche 3b)

- **Tranche 3b reconciled five records into 27 clause rows**: 22 `owned`, 3 `attach`, **1 `none`** — the ledger's first, which completes the closed set of six states — and 0 `unowned`.
- 🔴 **The MCP read surface leaks across tenants on all three read tools while the module's first paragraph claims the same authorization as the HTTP handlers.** `policy_bundle` takes its tenant as an unused `_tenant_id` over a `SELECT` with no `WHERE` clause and returns every tenant's policy clauses labelled with the caller's own tenant id; `list_inbox` declares a `principal` argument and never reads it; `get_thread` classifies, gets `Network` for a foreign tenant, and returns the full projection anyway. ⛔ **CORRECTED by `.6.1.1` (REPAIR-0182):** `policy_versions` has NO tenant column — `git grep -n "tenant" -- migrations/0038_policy_registry.sql crates/reasonbraid-server/src/policy.rs` returns rc=1 — so the registry is SITE-GLOBAL and HTTP `api.rs::list_policies` returns the same unfiltered set to any enrolled caller. The missing `WHERE`, the unchecked principal and the false tenant label all STAND; the cross-tenant FRAMING does not. The schema question is `.6.1.5`.
- 🔴 **`join_call` is the third instance of the two-caller-identifiers family**: gated on the caller's tenant, then the call is fetched by id alone and the two are never compared. A decline, recommendation or recusal skips the eligibility gate entirely.
- 🔴 **`policy.rs` queries the grants table two ways twenty-seven lines apart** — registration on status alone, resolution on status and expiry — so an expired grant registers a policy version the resolver in the same file would refuse. Its selector default is fail-open where the owning goal line says fail-closed.
- ⚠️ **Two measurements narrowed the record rather than confirming it**, and both are recorded as narrowings: the `body["tenant_id"]` panic is real but unreachable through the typed MCP tool, and the quota-per-retry behaviour is documented verbatim in the module header.
- ⭐ `ATTACH-LANDED` fired on this leaf's three unattached clauses before they were written — the second consecutive tranche it has caught.
- 🔎 **`MEMORY.md` stands at 7,161 bytes against a 7,168-byte cap**, with three consecutive leaves forced to evict a standing warning to land. Routed to `.11.4.2` with its measurement.
- Validation: `--classified` 157 rows clean with all 32 `attach` clauses named by their owners, `--self-test` 49 controls, 18 doctrines green, `mdbook build` rc=0, `git diff --check` rc=0. No product code, schema, test or script changed.

## 2026-09-14 — One missing predicate at three sites (`SIGNOFF-REPAIR.11.9.1.2.1`, tranche 3a)

- **Tranche 3 sized at 5,041 characters and split** on the same narrowest-candidate boundary that formed it — 1,624 / 1,655 / 1,762, all inside the 1,405–3,006 range this activity has proved executable in one commit.
- **Tranche 3a reconciled five records into 20 clause rows**: 15 `owned`, 3 `attach`, 1 `handled`, 1 `declined`, 0 `unowned`.
- 🔴 **A lookup keyed on `operation_id`/`command_id` with the owning node absent from the predicate lives at three sites**, reached by two different records and connected by no leaf: `node_channel.rs::event_id_for_operation` (the reconciliation lookup the handshake performs), the receipt insert's `ON CONFLICT (event_id) DO NOTHING` over a primary key that does not include `node_id`, and migration 0021's `delivery_state` view marking a row `consumed` from another node's result. `SIGNOFF-REPAIR.4.3`'s goal line already demands the proof that would have caught all three.
- 🔴 **The cursor high-water mark is derived from the table prune deletes from.** `enqueue` writes `MAX(cursor)+1` and `current_cursor` reads `COALESCE(MAX(cursor), 0)` over `node_inbox`; prune everything and it rewinds to 0, re-issuing cursor 1 for a node that already acknowledged it. The prune test leaves a partial prefix that keeps the highest cursor, so `MAX` never has to answer for an empty inbox.
- ⭐ **A dated answer to a question the record left open:** the hardcoded `expires_at` of `2026-09-15` does NOT break the two `policy.rs` tests tomorrow — neither the writer nor the reader compares a waiver's expiry to `now()`. The clause is declined as a present defect, and the same measurement found that `repeated_waiver` fires on long-lapsed waivers and on the first waiver, which is written into `.9.3`.
- ⭐ **`ATTACH-LANDED`, registered one commit earlier, fired on this leaf's own three unattached clauses before they were written**, naming each row and its owner, and went green once the sentences landed. Its first production use caught the exact failure it was built for.
- **A sharper shed rule at the `MEMORY.md` byte cap:** a warning a GATE now enforces is the cheapest of all to shed, because the machine states it at the moment it matters and names the offending row.
- Validation: `--classified` 130 rows clean with all 29 `attach` clauses named by their owners, `--self-test` 49 controls, 18 doctrines green, `mdbook build` rc=0, `git diff --check` rc=0. No product code, schema, test or script changed.

## 2026-09-14 — `ATTACH-LANDED`: gating the one ledger state that cannot check itself (`SIGNOFF-REPAIR.11.11`)

- **The reconciliation ledger's `attach` state is the only one of six whose next action is not "none"** — it requires a sentence written into a leaf the classifier does not own — and every other property of a row is visible in the row itself. `--classified` could therefore validate everything except the thing that mattered.
- **Registered as `ATTACH-LANDED`**, the registry's 24th named doctrine (the enforcer's top-level count stays 18; it runs inside `PROJECT-SPECIFIC`, 7 sub-checks -> 8): an `attach` clause whose owning leaf's own section does not NAME its record now blocks the commit.
- **The leaf had published a prediction before any instrument could test it, so the census was reproduced at four points by two independent routes** — the new refusal copied into a detached worktree at each tranche's closing commit, and a throwaway re-implementation over `git show`. Both report **3 breaching of 3** at tranche 1's close (`5862837`), then **0 of 7**, **0 of 15** and **0 of 26**. The three named are exactly the set found by hand: `R-53-4` clauses 2 and 3 on `.7.1`, `R-63-1` clause 4 on `.11.4`.
- **Zero today, every historical instance caught, one bounded file** — the `REASON-CODE-DOC` shape. `SIGNOFF-REPAIR.11.9`'s rejected gate would have fired on 114 of 131. The same rule family, measured twice, decided oppositely both times.
- The rule is the record id in the owner's own section, never a phrase, and it does not require the literal words `ATTACHED CLAUSE`. The id carries a right-hand boundary because `R-53-4` is a prefix of `R-53-41`.
- Falsified twice and observed red both times, through the instrument and through the registered enforcer; both restored to rc=0.
- Validation: `--self-test` 49 controls (42 before), `--classified` 110 rows clean, 18 doctrines green, `mdbook build` rc=0, `git diff --check` rc=0. No product code, schema or test changed.

## 2026-09-14 — Ask the renderer, not the specification (`SIGNOFF-REPAIR.11.9.1.1.3`, tranche 2 complete)

- **Tranche 2c reconciled the last four source-census records — `R-76-77-2`, `R-87-1`, `R-88-1`, `R-90-1` — into 38 clause rows**: 21 `owned`, 11 `attach`, 6 `handled`, **0 `unowned`**. Tranche 2 is complete at 14 records and 84 clauses.
- **No new repair leaf was opened, and that is the measurement.** Tranches 2a and 2b each found a clause no leaf covered and opened `.11.10` and `.4.1.6`. Every one of these 38 reached an existing owner, because `.11.2` and `.11.3` already own the bootstrap, the doctrine gates and the backup/restore/dev/demo/load scripts.
- 🔴 **`check_table_arity.sh` disagrees with the renderer that publishes this project's book, and its own self-test locks the error in.** The gate exists to catch GFM's silent cell-dropping; its splitter treats a pipe inside an inline code span as part of the cell. Asked of mdbook/pulldown-cmark directly, a two-column row reading `` \| `x \| y` \| 2 \| `` renders as **two** cells and the `2` is discarded. Census over 322 tracked markdown files: the gate finds **0** defective rows, the renderer's rule finds **2** — both confirmed by rendering them.
- 🔴 **One of the two is the doctrine registry's own row.** `DOCTRINE_ENFORCEMENT.md:39` renders its third cell as a bare `—`, so the published page never names `scripts/check_tree_index_frontier.sh` as `INDEX-FRONTIER`'s enforcer. It is deliberately left unrepaired as the last real-world specimen, so `.11.2` can falsify its parser repair against a row it did not write.
- 🔴 **`RECONCILIATION.md:111` was the other, in this leaf's own deliverable**, truncating the `R-53-2` Evidence cell at "three `.map_err(". Repaired here, with the superseded text named.
- 🔴 **`pipefail` does not fix the demonstration's negative controls**, which is the record's own proposed remedy refuted by measurement: the four-way truth table is rc=0 in every cell, because `!` negates a pipeline whose status is 1 whether the CLI failed or simply matched nothing.
- 🔴 **The load harness reports `PASS: every command committed` for a run that issued zero requests** (`--commands 0` and `--commands -5` both satisfy its exit gates), and issues 104 for a requested 100.
- 🔴 **7 of the 9 doctrine checks that consume the staged file list then read the worktree**, `check_task_tree_ownership` among them; **`check_docpaths.sh` cannot see `/Volumes`**, this checkout's own prefix; and **the handoff census prints `handoff: OK` at rc=0 when both its process censuses fail**, while its `ps -Ao` arm covers 620 rows across 40 uids where its documentation claims one.
- ⛔ **Refuted and recorded rather than dropped:** `R-87-1`'s premise that this project retains `MAINTAINING.md` is false, and was false at the census baseline. The sentinel finding stands; the framing that placed this repository in the blast radius does not.
- **11 `attach` clauses written into `.5.2`, `.11.2` and `.11.3` in this commit**, per the ledger's rule; 26 of 26 `attach` rows verified present in their owners' sections by hand. **Promoted:** "ask the renderer, not the specification" (`TOOLBOX.md`).
- Validation: `--classified` 110 rows clean, `--self-test` 42 controls, 18 doctrines green, `mdbook build` rc=0, `git diff --check` rc=0. No product code, schema, test or script changed.

## 2026-09-14 — Graded on the question, and one claim failed (`SIGNOFF-REPAIR.11.9.1.1`, `.11.11` opened)

- The director asked whether I trusted the session's findings. The claim standard treats that as the grading event and its pass condition is answering yes in one word, with no keyboard. I could not, so I re-derived twelve published claims by routes different from the ones that produced them.
- **Eleven held.** The certification count (3 of 6 invariants asserted per run) re-derives from the arm structure — three early-return pushes are mutually exclusive. The reconciler never reads `git.effective`: a grep for the field finds only comments and its declaration. `filter_profile` emits exactly fourteen fields, neither of the two the record named. `regions::route` still has no production caller. The sizing re-derives at 2,554 / 7,192 / 2.82×. All **15** `attach` rows are present in their owner leaves — I did not repeat tranche 1's mistake. The first `declined` row holds: the module carries exactly one wire-absence claim and it is about hidden fields.
- 🔴 **One failed, and it was mine.** The eight uncited-but-named records were published with LINE NUMBERS. Two of the eight were stale before the session ended, having moved as this tree grew above them. A line number into a growing file is the `LIVE-DOC-CURRENCY` failure in another costume: true when written, false the next commit, and nothing checks it. All eight are now located by the heading they sit under, and the superseded form is named rather than quietly replaced.
- ⚠️ A second, softer correction: the reproduction for that number reports **13** today rather than 8, because a `## Commit acceptance` heading is not a leaf heading either and five of this session's own blocks name records. The 8 is a true statement about the population at leaf-open, not a stable quantity — now said so.
- The grading also produced a tracked gate candidate rather than a note. `SIGNOFF-REPAIR.11.11`: `attach` is the only ledger state whose next action is a sentence in someone else's leaf, and `--classified` cannot see whether it was written. Censused before proposing — 15 rows, **0 breaching today, 3 of 3 at tranche 1's close** — which is the shape this project allows a gate to have.
- Validation: 18 doctrines green, `mdbook` rc=0. Documentation only.

## 2026-09-14 — A host claim is checked where a human typed it (`SIGNOFF-REPAIR.4.1.6`)

- Reproduced through the supported routes: a non-ASCII host claim was accepted at token issuance and panicked when the node redeemed it. The node received a dropped connection rather than an answer; the server kept serving; no node row was written; and the token was left unconsumed.
- That last fact is the one the opening inference had not reached. An outstanding unused token refuses a second issuance for the same node id, so the node id could not be enrolled until the token lapsed — bounded to the token's lifetime (an hour by default, a day at most), and bounded only because `SIGNOFF-REPAIR.4.1.1` already supersedes a lapsed token.
- Two changes. `issue_node_leaf` returns a result instead of unwinding — a function whose only way to report a bad caller string is to panic leaves its caller's typed error path unreachable — and the issuance route checks the claim beside the node-id shape and the lifetime range, because that is where an operator typed it. The first makes the failure answerable; the second makes it unreachable.
- The accepted set is deliberately unchanged: the check asks the certificate library rather than restating a grammar, so it narrows nothing and cannot drift from what issuance would have done. Whether a stricter host-name grammar should bind is deferred with its compatibility question named, in `docs/decisions/2026-09-14_host-claim-checked-at-issuance.md`.
- Validation: 82 live tests across five suites and 103 library tests, rc=0; strict all-target lint on the server rc=0. Falsified in two halves, each hitting a disjoint control: removing the issuance check gave 18 passed / 1 failed with `left: 200, right: 400`; restoring the panic under the unchanged signature gave 19 passed / 2 failed, reproducing the transport error a second time.
- One documented wire narrowing on `POST /v1/nodes/enroll-tokens`; no migration and no schema change.

## 2026-09-14 — The first declined clause, and asking the library instead of the call site (`SIGNOFF-REPAIR.11.9.1.1.2`)

- Tranche 2b classified 19 clauses over four records: 1 handled, 8 owned, 8 attach, 1 unowned, and the ledger's **first `declined`** row. That state was put in the vocabulary before any instance existed, because a deliberate rejection is invisible to every search; the first one is a clause whose mechanism is real and whose asserted contradiction is not.
- One clause could not be settled by reading — `expect panic if malformed host` — because what the certificate library calls malformed is not knowable from the call site. A probe asked it: of eight host claims, rcgen 0.14.10 refused exactly one, the non-ASCII one. An empty string, 300 characters, `not a dns name!!`, `*.example.com` and `..` were all accepted.
- That produced two findings pointing opposite ways. The panic is real and reachable — the host claim travels unvalidated from token issuance to enrolment and is handed to an `expect` — and it is much narrower than "malformed" suggests. And the accept set is far wider than a DNS name, so the certificate's host binding is effectively unvalidated, which is the larger finding and not the one the record named. New owner `.4.1.6`, opened with the sample explicitly labelled a sample and the blast radius explicitly not claimed.
- All eight `attach` clauses were written into the leaves that own them in this same commit, under the rule added one commit ago: a completion invariant that accepts a failure as completion and an unbounded drain (`.10.2`); a certificate authority signed for a year with no renewal and a leaf never compared against its issuer (`.4.1`); a visibility default that publishes eleven of fourteen fields while its documentation promises self-only, a reversed doc example beside correct code, and a "full profile" reader that never receives two fields (`.5.1`); and a publication commit whose object id changes every second while its own documentation calls it idempotent (`.9.2`).
- Validation: `--classified` 72 clause rows, ledger clean; `--self-test` 42 controls; uncited 106 → 102; 18 doctrines green; `mdbook build` rc=0. No product code, test or script changed — the probe was an untracked file, run once and removed.

## 2026-09-14 — A failed read is not a verdict about the site (`SIGNOFF-REPAIR.11.10`)

- `regions::route` converted any `sqlx::Error` into a routing refusal. Reproduced with the probe declaring its own premise first: `dev-local` is a seeded declaration, and with the pool closed the routing answered ``the region `dev-local` is undeclared — the routing refuses``. A statement about the site's configuration, produced by a failure that touched no configuration.
- The same repair already existed one module away, with its reason in its own source: `SIGNOFF-REPAIR.3.2.1` put the `pair` verb on the site-authority service because "an unavailable database must never masquerade as an undeclared region". The review record named `pair` **and** `route`; only `pair` was reached, and the clause reconciliation found the other half.
- A new `RouteError` keeps the two families apart — `Refused(RegionRefusal)` for a verdict reached by reading the configuration, `Storage(sqlx::Error)` for a decision that could not be made. `RegionRefusal` is byte-identical, variants and wording, because the phase acceptance and the operator contract rest on those names. The storage message names neither the region nor "undeclared".
- Reachability stated honestly: `route` has no production caller today. It was repaired now rather than left to the store-and-forward lane that will inherit it, because an inheriting leaf censuses its own goal line.
- Validation: `run_pg_tests.sh regions` 3 passed / 0 failed, rc=0; strict clippy on the crate's lib and this test rc=0 with no warnings; `cargo fmt --all -- --check` rc=0. Falsified against the exact superseded mechanism — the `map_err` restored under the new signature gives `2 passed; 1 failed`, the single failure this leaf's control naming `Refused(UndeclaredRegion { region: "dev-local" })`, with the other two tests still green.

## Historical entries and exact retrieval

This is a recent digest. Older chronology remains in reachable Git history under
the rotation contract in `README_POLICY.md`. This file has rotated sixteen times;
each rotation names the commit holding the ledger immediately before it, so the
chain walks back without guessing.

Retrieve the ledger immediately before the SIXTEENTH rotation (2026-09-14)
from the repository root:

```bash
git show dd4151b9d5652a5cb2095e333c34282b20d1d613:CHANGELOG.md
```

That snapshot is 95,076 bytes and contains 35 dated entries; its Git blob is
`5f4dfee6e8bd8efafb9f8fc91f50f11143db83e5`, and its SHA-256 is
`6dfcb84b4f7a5c5e462ddf288772083308a0fbde0715a43c02837ea7f803292d`. The newest
entry it holds that this digest no longer carries is
`2026-09-14 — The ledger's own next action was never taken (`SIGNOFF-REPAIR.11.9.1.1`, `.11.9.1.1.1`)`.
It carries the FIFTEENTH rotation's notice in turn, which names the ledger before it.

Retrieve the ledger immediately before the FIFTEENTH rotation (2026-09-13)
from the repository root:

```bash
git show 2c1bbe80ee904026c171b3f82ace3b6fe8edab54:CHANGELOG.md
```

That snapshot is 92,548 bytes and contains 30 dated entries; its Git blob is
`a515c482b0e6e750208d36e538d0ff8777e3cc4a`, and its SHA-256 is
`b2b9b74cfa7ee1c87d2585aeb30f81108964978d441a759c29a1e2fcdaf1e301`. The newest
entry it holds that this digest no longer carries is
`2026-09-13 — Every rung of the proof ladder gets its own negative (`SIGNOFF-REPAIR.4.2.6`)`.
It carries the FOURTEENTH rotation's notice in turn, which names the ledger before it.

Retrieve the ledger immediately before the FOURTEENTH rotation (2026-09-13)
from the repository root:

```bash
git show 2912136f49316ba310753cf9672188c1d6815901:CHANGELOG.md
```

That snapshot is 95,013 bytes and contains 31 dated entries; its Git blob is
`0d6b714ce9a211a48b98aaaef1bf03c441a2c59f`, and its SHA-256 is
`055464b527c1169cb47944106eec5b61d196fcbad8ddd3be5aa9c281e235d64c`. The newest
entry it holds that this digest no longer carries is
`2026-09-13 — The self-test searched for a string it contained (`SIGNOFF-REPAIR.11.4.3.1.7.1`)`.
It carries the THIRTEENTH rotation's notice in turn, which names the ledger before it.

Retrieve the ledger immediately before the THIRTEENTH rotation (2026-09-13)
from the repository root:

```bash
git show 87a0abf90d0fa24e49ca85b5e121722fbacb7344:CHANGELOG.md
```

That snapshot is 95,400 bytes and contains 31 dated entries; its Git blob is
`2b9be00dffbbe7f5837f20710769c81d19ef02ae`, and its SHA-256 is
`ffabf15976215219ee32b88979a88853c237e7e9fead69f525fe51c99271609b`. The newest
entry it holds that this digest no longer carries is
`2026-09-13 — There are three clocks, not two (`SIGNOFF-REPAIR.3.4.3.1`)`.
It carries the TWELFTH rotation's notice in turn, which names the ledger before it.

Retrieve the ledger immediately before the TWELFTH rotation (2026-09-13) from
the repository root:

```bash
git show a972d89550df2aab9c554c474a3301f76d04c201:CHANGELOG.md
```

That snapshot is 95,460 bytes and contains 31 dated entries; its Git blob is
`6cb62466f857386535e63605f3c268442c66bad1`, and its SHA-256 is
`7beecd2a5acd1ecaca3df1e4353511c8239b983b95f3610630458855eaff29a2`. The newest
entry it holds that this digest no longer carries is
`2026-09-13 — A `join` carrying a decline was accepted as a join (`SIGNOFF-REPAIR.3.4.4`)`.
It carries the ELEVENTH rotation's notice in turn, which names the ledger before it.

Retrieve the ledger immediately before the ELEVENTH rotation (2026-09-13) from
the repository root:

```bash
git show f425ae0a5bff201bedde5d228c9e4659c7408740:CHANGELOG.md
```

That snapshot is 95,641 bytes and contains 31 dated entries; its Git blob is
`18388aa961ae05ef8dd1c104f35f1268cd239b71`, and its SHA-256 is
`2a3b7fb9241e341193b514947b9a5f9356659eebb6c919a4854ae6ae4f50bf55`. The newest
entry it holds that this digest no longer carries is
`2026-09-13 — The cache's freshness window compared two different clocks (`SIGNOFF-REPAIR.3.4.3`)`.
It carries the TENTH rotation's notice in turn, which names the ledger before it.

Retrieve the ledger immediately before the TENTH rotation (2026-09-13) from the
repository root:

```bash
git show 09b4f39d2d489877ec0738b980459a60865c08f9:CHANGELOG.md
```

That snapshot is 95,623 bytes and contains 31 dated entries; its Git blob is
`3b7216e39d03f40944868b6862dcfeefded11831`, and its SHA-256 is
`fba84ccdec894c49a7193bf473af067297deb36ab115bb6a729d6b3b601b904f`. The newest
entry it holds that this digest no longer carries is
`2026-09-13 — ⛔ Correction: the `delegable` flag governs chains that do not exist (`SIGNOFF-REPAIR.3.4.1`)`.
It carries the NINTH rotation's notice in turn, which names the ledger before it.

Retrieve the ledger immediately before the NINTH rotation (2026-09-13) from the
repository root:

```bash
git show be8cdb707f09d5d1746d5e991c5ff969b9a096d9:CHANGELOG.md
```

That snapshot is 92,812 bytes and contains 29 dated entries; its Git blob is
`a217f1b62579987706bc1ec5e417f5386332c398`, and its SHA-256 is
`9fe01e8e07ecafe729e0728f2cd2d8411d193555f8e73ae678508e7e96b337fc`. The newest
entry it holds that this digest no longer carries is
`2026-09-13 — The guard census, reconciled with instruments (`SIGNOFF-REPAIR.3.3.4.13`)`.
It carries the EIGHTH rotation's notice in turn, which names the ledger before it.

Retrieve the ledger immediately before the EIGHTH rotation (2026-09-13) from the
repository root:

```bash
git show c8e591a2dbc1dc5c1f85d428453f59b747da3249:CHANGELOG.md
```

That snapshot is 93,824 bytes and contains 27 dated entries; its Git blob is
`e42def45b1ea97ee33f50ee81b220d2b51ee5a2f`, and its SHA-256 is
`6dc11f303f1b0e329a78e301869559d87d493a3e6921b392028be7bc534f939b`. The newest
entry it holds that this digest no longer carries is
`2026-09-12 — The profile/card surface, censused and split`.
It carries the SEVENTH rotation's notice in turn, which names the ledger before it.

Retrieve the ledger immediately before the SEVENTH rotation (2026-09-13) from the
repository root:

```bash
git show b3802b16601fb4a2da67a8721d7659ae1e36aed9:CHANGELOG.md
```

That snapshot is 94,920 bytes and contains 27 dated entries; its Git blob is
`9591c6945fc43581906629a07b83a99afcad3bf4`, and its SHA-256 is
`fb90861ac348208d4f01557b3dac668ca7a1f9b35ff340ea8eaf67eba9216f56`. The newest
entry it holds that this digest no longer carries is
`2026-09-12 — Revocation becomes one transaction, from the admission to the evidence`.
It carries the SIXTH rotation's notice in turn, which names the ledger before it.

Retrieve the ledger immediately before the SIXTH rotation (2026-09-13) from the
repository root:

```bash
git show 711885c43aad15bf423f55581dee8b2e8f85ab40:CHANGELOG.md
```

That snapshot is 95,390 bytes and contains 28 dated entries; its Git blob is
`80fbb202f0780555d646345863d978f459ffbe15`, and its SHA-256 is
`5eb37168688d4b9e04c57894f54cc2762a87992f50e2a3fc56178eb5eea02d48`. The newest
entry it holds that this digest no longer carries is
`2026-09-12 — Census and retire the runner's retained clusters, with a tracked instrument`. It carries the FIFTH
rotation's notice in turn, which names the ledger before it.

Retrieve the ledger immediately before the FIFTH rotation (2026-09-12) from the
repository root:

```bash
git show 55d5f9edf4cacc2e321126e9a248f2d74a248fd1:CHANGELOG.md
```

That snapshot is 93,820 bytes and contains 34 dated entries; its Git blob is
`5670f2ab78d8ec43a889a0851821cff9635a4f70`, and its SHA-256 is
`6c29b639895357fa90bec167405e93268a07f51ca7ae3a7738537f2edb18a6ff`. The newest
entry it holds that this digest no longer carries is `2026-09-12 — Drive an R2
acquisition to its persisted evidence`. It carries the FOURTH rotation's notice
in turn, which names the ledger before it.

Retrieve the ledger immediately before the FOURTH rotation (2026-09-12) from the
repository root:

```bash
git show 1faac4e126325c61842fd17f56feeb32b6bd6f5f:CHANGELOG.md
```

That snapshot is 93,689 bytes and contains 45 dated entries; its Git blob is
`c32524d1c1a7b042cc6a8647136b8729d66d4ac6`. The newest entry it holds that this digest no longer
carries is `2026-09-11 — Own the Git acquisition workspace`. It carries the THIRD
rotation's notice in turn, which names the ledger before it.

Retrieve the ledger immediately before the THIRD rotation (2026-09-12) from the
repository root:

```bash
git show 0cda20cfa12c62269d14e5ea78412d1548deab68:CHANGELOG.md
```

That snapshot is 94,066 bytes and contains 68 dated entries — the 38 retained
above plus the 30 rotated out of it, the newest of which is
`2026-09-11 — Bind R2 responses to owned input bytes`. Its Git blob is
`75a374a51aa21a1d7d226dc63f6349bd0b9b477f`.

That snapshot in turn carries the SECOND rotation's notice, which names the
ledger before it:

```bash
git show f75106915ff1f1171b332451257388905b05d815:CHANGELOG.md
```

That snapshot is 93,956 bytes and contains 91 dated entries; its Git blob is
`b9aacfc467e1729cae1a5e76fe1d0adee6398c1d`. It carries the FIRST rotation's
notice in turn:

```bash
git show 25ed7d184203e2d8701800558b785b30c75bb4d0:CHANGELOG.md
```

That earliest snapshot contains 130 dated entries. Its Git blob is
`0bc51d581f9158ebafcef94cfb6717464722c6cb`; exact byte/line counts,
SHA-256 identities and the first transition's evidence are in
`docs/decisions/2026-09-09_changelog-rotation.md`. Use
`git log --follow -- CHANGELOG.md` for earlier versions. Keep the reachable Git
history when cloning or handing off; a shallow checkout may need the named commit
before retrieval. A missing object is a retrieval failure, never evidence that
history was empty.

Historical success statements describe the recorded revisions and assertions.
Current qualification is in `LIVE_STATUS.md` and the mdBook's qualification review;
open repairs remain tracked in `docs/tasks/SIGNOFF-REPAIR.md`.
