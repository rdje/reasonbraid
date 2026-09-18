# CHANGELOG.md

## 2026-09-18 — Correction: REPAIR-0259's severity claim, and a reproduction that modelled a path nobody uses

⚠️ **The previous entry said *"a project that pulled this spine could not make its first commit"*. That is wrong, and the reproduction behind it was a fixture rather than the product.**

- **How projects are actually created, from the director and confirmed in this repository's own history:** `rdje/bedrock` is a **GitHub template repository**, not a syncable remote. A new project is instantiated through GitHub's *Use this template*, which copies the WHOLE repository, and is then cloned and bootstrapped locally — `b932c05 "Initial commit"` followed by `823c2bc "bootstrapped from bedrock"`. The remotes confirm it: **reasonbraid's remote set contains no bedrock entry** — `origin` is `rdje/reasonbraid` and nothing else. ⚠️ bedrock of course has its OWN `origin` (`rdje/bedrock.git`), which is how the template itself is maintained; what is absent is a link FROM an instantiated project BACK to the template.
- ⛔ **So a fresh project receives every script and commits fine.** Copying only the `NEUTRAL` paths into a bare `git init` was a model of a creation path nobody uses, and its `10 doctrine breach(es)` measured my own fixture.
- ✅ **What remains true, and why the repair stands.** `update_scaffold.sh` is an UPDATE mechanism and only copies listed files — it deletes nothing. `NEUTRAL` carried the registry and not the 7 scripts it names, so syncing into a project instantiated from an OLDER snapshot installs a registry naming checks that never arrive and breaks that project's gate. A real hazard at update time for an older project; not at creation.
- ⚠️ **Corrected severity:** the defect is *the update mechanism cannot deliver the doctrines the registry names*, not *the spine ships broken*. `SCAFFOLD-COVERAGE` keeps its 0.02 s on that basis — registry-versus-list drift is silent and bites the project being updated — as a consistency guard rather than a shipped-broken guard. The root-cause finding is untouched: `NEUTRAL` is a second copy of the registry that nothing derives, and it had drifted by seven.
- 🔎 **The method failure, which is the useful part.** I built a fixture, reproduced a failure inside it, and reported the fixture's behaviour as the product's. **A reproduction is evidence only if the path it models is one that occurs.** The falsify leg asks what would have to be true for the claim to be false — here, *"projects are created some other way"* — and I never checked how a project is created before asserting creation was broken.

## 2026-09-18 — The scaffold shipped an enforcer registering checks it did not carry (`SIGNOFF-REPAIR.11.23`)

🔴 **A project that pulled this spine could not make its first commit.** `scripts/update_scaffold.sh` carried `check_doctrines.sh` — the registry of 18 doctrines — and **not the 7 check scripts that registry names**. Copying only the `NEUTRAL` paths into a bare `git init` repository printed seven `?? (missing/not executable: …)` rows and `10 doctrine breach(es) — commit blocked`.

- The seven: `HEADING-DEPTH`, `TASK-STATUS`, `LOCKSTEP-CLAIM`, `INDEX-FRONTIER`, `FRONTIER-STATUS`, `RUST-FORMATTING`, `SELF-TEST`. **Every one was earned here by a measured defect** — and the list they were missing from is the only mechanism that moves a doctrine anywhere, so the spine's improvements were accumulating in the instance and could not leave it.
- ⭐ **Fourth instance of one shape:** `NEUTRAL` is a second copy of a fact the registry owns and nothing derived it — as `INDEX-FRONTIER` is the index's copy of the frontier, and `FRONTIER-STATUS` a row's copy of a leaf's status. Each drifted the moment the first copy grew.
- ✅ **Fixed and re-reproduced:** the seven join `NEUTRAL`, `SCAFFOLD-COVERAGE` is registered at **0.02 s**, and the bare-repo run now prints **zero** `??` rows. Four breaches remain there, none a scaffold defect — three are empty-project content checks, and the fourth is below.
- 🔴 **The gate failed its own reproduction, which is the useful part.** `update_scaffold.sh` is deliberately absent from its own `NEUTRAL` (a sync script overwriting itself mid-run is its own hazard), so a project may legitimately have no scaffold at all. That is **NOT CHECKED** — never a pass, never a breach.
- 🔴 **And falsification found the gate's own `$EXEMPT` guard DEAD.** Mutating it away changed nothing, because the parser anchored on the array literal and never saw the conditional `DOCTRINES+=("…")` form that registers `KNOWLEDGE-MAP` and `PROJECT-SPECIFIC`. Both forms are parsed now. ⚠️ Widening the parser then over-matched the registry's own explanatory comment — the third *key too loose* in one session.
- ⛔ **Backflow is still manual.** `update_scaffold.sh` pulls downstream only and `rdje/bedrock` is the root template with nothing to pull from. What changed is that the seven are now **transportable** and the class cannot silently recur; carrying them up belongs to a `BEDROCK-MAINTENANCE` leaf there.

## 2026-09-18 — A gate blind to the abbreviation of the very noun it is built on (`SIGNOFF-REPAIR.11.2.4`)

🔴 **`check_visibility_policy.sh` exists because the superseded private-repository instruction "had already leaked past two" reviews. All four of its clauses were anchored on the full word, so two live instances written in the abbreviated copular form sat in the corpus invisibly and the gate returned rc=0 over them.**

- ✅ **The widening is DERIVED, not invented** (`.11.2.2`'s rule). A census of every tracked-Markdown line carrying a repo word beside `privat` — about 60 lines — shows the abbreviation is the ONLY uncovered spelling that *states* a visibility. The many `private-repository instruction` lines put `private` BEFORE the noun and are corrections of the superseded policy, so no clause matches them and none should. One substitution in three clauses; **delta 3 → 11 matches, `comm` confirms none lost**.
- 🔴 **Six of the eight new matches were the project documenting its own blind spot.** Writing down what the instances read puts the trigger into the corpus, so the widened gate flagged `MEMORY.md`, `docs/TASK_TREE.md` and three places in the tree. That is `check_self_tests.sh`'s founding incident — *a control searched for a string it contained* — reached from the documentation side.
- ✅ **Dispositioned by a rule rather than case by case: except what must stay VERBATIM, reword what is PROSE.** Five live sentences now NAME the abbreviated form instead of instantiating it, following `check_self_tests.sh`'s own precedent of rewording rather than path-excluding, so those surfaces stay policed. Three preserved-evidence lines are excepted with reasons: a dated decision record (decisions supersede rather than mutate), a dated `Done (2026-09-08)` entry whose correction is the line above it, and the leaf's own reproduction command, where the literal sentence *is* the evidence.
- ✅ **Falsified four ways**: the pattern reverted (three positives red), a bare `privat[a-z]*.repo` clause added (**all four** negatives red, including two real corrections), a genuinely new sentence injected into the book (refused by file and line — what the gate is *for*), and a bogus exception added (`stale exception` by name).
- ⭐ **And `FRONTIER-STATUS`, registered one commit earlier, caught this leaf leaving a stale frontier row behind** — the defect it was built for, firing on its author within the hour.
- ⛔ Not claimed: that the pattern is complete. It covers the spellings this corpus contains today; the census command is recorded so the next reader re-derives rather than trusts.

## 2026-09-18 — Small-swarm orchestration recorded as the existing objective, unscheduled (director direction)

The director asked for ReasonBraid to orchestrate 2–5 agents working collectively on any given problem, scaling later, prompted by the reported OpenAI Navier–Stokes swarm and xAI's Grok 4.20 multi-agent model.

- **It is not a pivot.** `README.md` already states the objective, and the shipped step vocabulary in `workflows.rs` — `solicit`, `blind_solicit`, `critique`, `revise`, `synthesize`, `assess`, `vote`, `adjudicate`, `decide`, `approve`, `moderate`, `present`, `retrospect`, `evidence_request`, `quick_advice` — contains **no programming-specific verb**. A profile is versioned configuration over existing verbs, so changing domain is writing a profile rather than changing code; real `claude.rs`/`codex.rs` adapters already dispatch to agent harnesses.
- ⛔ **The binding constraint is the ACCEPTOR, not orchestration.** Output stays untrusted until deterministic rules accept it. Programming has acceptors — compilers, suites, this repo's own gates. "Any problem" frequently has none, which is exactly what the Navier–Stokes episode illustrates: a swarm produced a result that remains unverified and whose attribution is disputed. Those two difficulties are what this platform is for.
- ⚠️ **2–5 agents is in range; thousands is a re-architecture**, not an extrapolation — the current design serialises administrative acts on one site guard row under per-tenant transactions.
- ⛔ **Unscheduled, and no leaf is opened** — deliberately, because the LAN bar was defined hours earlier and a leaf would put this above it. Much of the capability sits inside that bar regardless: `blind_solicit`, rounds, quorum and adjudication are G3/G5 surfaces and G5 is currently withdrawn.
- ⚠️ The two external results are recorded as **reported, not established** — the Navier–Stokes claim is explicitly unverified — because a direction may rest on them but no claim may.

## 2026-09-18 — Internet exposure deferred; the LAN path is the priority (director instruction)

> *"The internet exposure is not high priority right now. It needs to fully work on the local network first."*

- **B1, B2 and B3 become `yes (deferred)`** in the blocker register — a third state the register did not have. The in-repo work is still owed and `SIGNOFF-REPAIR.14` still owns it; what changes is that it leaves the frontier and no commit is spent on it until the local-network path is complete.
- ⛔ **`yes (deferred)` is deliberately not `no`.** `.13` was built around the observation that an owned-but-unsurfaced blocker is harder to notice than an unowned one, so these rows stay visible and are surfaced once per session with their ruling and trigger.
- ⛔ **Priority moved; claims did not.** Phase 7's G6/G7 stays **NOT MET** and `.14`'s standing prohibition against turning the exposure profile on is untouched. Deferring work is not permission to describe the system as Internet-ready.
- ✅ **The trigger is DEFINED, and the roadmap already encoded it.** The director clarified it in two steps: behaviour matches the roadmap's objective on the LAN, and G7 is earned on the LAN before the Internet. The roadmap's header separates *initial: private LAN* from *target: arbitrary Internet hosts*, and its gate table's *unlocks* column makes **G6 the only gate whose unlock is a transport**. So the bar is **G0–G5 genuinely met plus G7 on the LAN**, G6 out of scope.
- ⛔ **The first candidate is WITHDRAWN as wrong in KIND**, not detail: it was phase arithmetic, and repairs closing can diverge from behaviour matching in both directions.
- ⛔ **G7 splits, recorded now so it cannot become a false claim later.** Structural legs — restore, survival of induced failure, instrumentation — are earned once. Load thresholds and SLO targets are anchored to a LAN and get re-derived under the Internet posture.
- ⚠️ **Measured before accepting the bar:** G7 is the least-built LAN gate — backup/restore real, metrics without objectives, load dev-scale only, and chaos/game-day returning **0** hits repo-wide. G5 is *withdrawn* rather than unverified, so re-earning it is inside the bar too.
- Recorded in `docs/decisions/2026-09-18_lan-completeness-precedes-internet-exposure.md`; the register, the book's blockers chapter, the frontier row for `.14` and `MEMORY.md` all carry it.

## 2026-09-18 — The frontier table gains a row per state change and never retires the old one (`SIGNOFF-REPAIR.11.22`)

🔴 **DOC-0049 reported a stale frontier pointer. Censusing it found the cause is structural, and worse than the symptom.**

- **The census, run before any rule (`.11.6`): 66 rows naming 49 distinct leaves — 13 leaves carry 2 to 4 rows each, and 10 rows' status columns contradict their leaf.** A leaf gets a row when it opens (`pending`) and another when it closes (`done`), and the opening row is never removed. `.11.14.3.10` had four rows, three stale.
- ⭐ **It caught its author in the act.** `.11.21` had a `done` row at `1a18` and a `pending` row at `1a16` — both written by me two commits earlier, in this same session.
- 🔴 **A third historical firing, found by the census rather than remembered:** `.11.15` records `.7.4.1` sitting at row 2 for seven commits after closing. With REPAIR-0253's five rows and row 1's own pointer, the founding population is three separate firings.
- ⚠️ **Two false starts in the census itself, recorded because they are one lesson twice.** `s.index("## Current Frontier")` matched an earlier prose mention and censused a table 400 KB away, returning `0 rows`; then the slice ran to the next `## ` heading and swept in unrelated tables. Anchoring to `^## Current Frontier$` and bounding to the contiguous pipe lines fixed them. Both are `a-key-too-loose-returns-the-wrong-instance` — in the leaf immediately after the one that repaired a key defect of the same shape.
- ✅ **The rule, with the history case explicitly permitted:** every row's status column equals its leaf's own status, and row 1's leaf is not finished. ⛔ It is NOT "no finished leaves in the table" — most of the table is closed rows kept as recent history. A self-test arm asserts the legal `done` row so the rule cannot drift into the stricter one.
- ✅ **Both status forms read.** 350 leaves, 28 carry no `Status:` line and 27 of those state it in an `Opened:` bullet; reading one form would report 28 false disagreements. `active` is a real third value and is not finished.
- ✅ **Reconciled:** 9 stale rows removed, 1 corrected (`.9.3.4` `pending` → `active`). **`FRONTIER-STATUS` registered** as doctrine 16 of 20 at **0.06 s**, falsified five ways and against the real defect in both shapes.

## 2026-09-18 — A frontier row pointed at a leaf closed forty commits earlier (`SIGNOFF-REPAIR.11.22`)

🔴 **Row 1 of the frontier — the one row a fresh session resumes from — named `.7.4.5`, which has been `done` since REPAIR-0216.** It was found only by opening the leaf in order to work on it.

- **This is the second firing in two days.** REPAIR-0253 reconciled **five** rows whose status column said `pending` while their own leaves said `done` (`.11.18`, `.11.17`, `.11.14.3.7`, `.11.14.3.5`, `.11.4.7.2`). Two hand reconciliations is a population, not an anecdote.
- **Censused rather than asserted:** `git grep -lni "frontier" -- scripts .githooks knowledge-map` returns **10** scripts, and **0** relate a row's status column to its leaf's `Status:` line. ⭐ Two apparent hits were false positives of a loose key — `check_task_status.sh` matched its own self-test FIXTURES, `bootstrap.sh` matched a heredoc it WRITES — a key defect of exactly the shape repaired one commit earlier in `.11.8.1`.
- ⛔ **Three gates look adjacent and every one of them correctly misses it.** `INDEX-FRONTIER` compares `docs/TASK_TREE.md`'s Frontier column to the tree's own row 1 — both named `.7.4.5`, and two files agreeing is precisely what it asks for. `TASK-STATUS` requires one `Status:` line per leaf and never reads a table. `TABLE-ARITY-RATCHET` reads cell counts, never cell meaning. The defect lives in the seam between three checks each doing its job.
- ⚠️ **The rule is not "no closed leaves in the table".** Rows `1a11`–`1a18` deliberately carry `done` leaves as history and should. What is wanted is that a row's status column AGREES with its leaf's own `Status:`, and that row 1 names a `pending` leaf.
- ✅ Owned by `.11.22` with its acceptance: census the whole table before proposing a rule (`.11.6`), permit the `done`-as-history case explicitly, and if a gate is registered, see it RED against both a stale `pending` row and a row-1 pointer at a closed leaf. The pointer itself is corrected to `.11.2.4`, the tree's true next-ranked pending leaf.

## 2026-09-18 — Unformatted Rust reached `main`, because nothing checked per commit (`SIGNOFF-REPAIR.11.21`)

🔴 **`crates/reasonbraid-server/src/git.rs` sat on `main` failing `cargo fmt --all -- --check` at two sites, and was found only because a LATER leaf happened to run the check while verifying a file it had not touched.**

- ⭐ **A policy gap, not a lapse.** `CLAUDE.md` §16 routes the full CI to pre-push and the per-commit gate to the doctrine enforcer, whose 18 registered checks contained no formatting check; `cargo fmt --check` appeared only in `COMMIT.md`'s pre-push list. Pushes go out in batches of ~300 commits, so unformatted code could sit on `main` for as many commits as that — and did, from REPAIR-0251/0252.
- ✅ **Measured before the rule was proposed (`.11.6`).** Three runs each, warm: `cargo fmt --all -- --check` at **0.81 / 0.66 / 0.66 s**, the enforcer at **12.57 / 12.52 / 13.00 s**. About **5 %**, and `.11.5`'s constraint holds — nobody routes around two-thirds of a second.
- ⛔ **Leaving it in the pre-push list was rejected on evidence, not taste**: that is exactly where it already was, and it is the arrangement that let the defect ship. A check running once per ~300 commits cannot say WHICH commit broke formatting, so the cost of finding out is the thing being economised on.
- ✅ **`RUST-FORMATTING` is now doctrine 16 of 19**, whole-workspace rather than staged-scope — deliberately, because the defect was in a file no staged change touched. ⚠️ That is affordable only because the workspace is formatted as of this commit; a strict check over pre-existing debt would have needed a ratchet like `TABLE-ARITY`.
- ✅ **Seen RED, then green.** `let  _deliberately = 1 ;` injected into `git.rs::expired` produced `❌ RUST-FORMATTING … 1 doctrine breach(es) — commit blocked`, naming the file by repository-relative path; restored byte-identical, the gate returns to green over 19 checks. Its own self-test reaches the TOOL rather than just the verdict, and FAILS rather than skipping when `rustfmt` is absent.
- 🔴 **The registry's own guard caught me while registering it.** My first description spelled the command in backticks — and each registry entry is a bash double-quoted string, so that is command substitution the driver would EXECUTE on every commit and in CI. `DOCTRINE-REGISTRY` refused it by name. `DOCTRINE_ENFORCEMENT.md` documents this exact trap from a previous occurrence; the gate worked, and so would have reading the section first.
- ⚠️ A number corrected in passing: the enforcer costs ~12.6 s, not the 3.15 s of `.11.4.3.1.7.2`. That figure is anchored to its own leaf rather than wrong — the registry has grown — so the current one is named beside it rather than edited into history.

## 2026-09-18 — A route key that was too loose and too tight in one expression (`SIGNOFF-REPAIR.11.8.1`)

🔴 **The route-documentation census was wrong in BOTH directions, and its own self-test asserted the defect rather than catching it.**

- **The mechanism, one function.** `stem()` DELETED every `{param}`: `re.sub(r"\{[^}]*\}", "", route).replace("//", "/")`. That is too loose and too tight at once.
- ⛔ **Too loose — a false `described`.** `/v1/snapshots/{snapshot_id}` and `/v1/snapshots` collapse onto one key, so a contract line for one credits the other. Re-derived by hand: `/v1/calls` was counted documented because `GET /v1/calls/{call_id}` appears at `authority.md:833`, while a grep for a bare `… /v1/calls` contract line returns nothing. `/v1/policy-publications` was credited the same way by `POST /v1/policy-publications/{id}/publish`.
- ⛔ **Too tight — a false `absent`.** A parameter in the MIDDLE of a path leaves a stem no book writes: `/v1/snapshots/{id}/derivations` became `/v1/snapshots/derivations`, though the book names it at `deployment.md:584`.
- ✅ **21 of 104 routes misclassified** — 18 wrongly `absent`, 3 wrongly credited as documented. Corrected today: **104 routes — 49 described, 3 mentioned, 52 absent**. And apples-to-apples, at `.11.8`'s own commit `0fbb85f` in a throwaway on-volume worktree with only the key replaced: **103 routes — 30 described, 3 mentioned, 70 absent**, against its published **22 / 1 / 80**.
- ⚠️ **A correction to this file.** REPAIR-0254's entry attributed `35 described / 68 absent` to `.11.8`. Those are what the DEFECTIVE instrument reports over today's corpus; `.11.8` published `22 / 1 / 80` over 103 routes. A number needs its producer *and* its moment.
- ✅ **The repair is to normalise, not delete** — every `{whatever}` becomes `{}` on both sides, so a key matches the book's own spelling and two routes stay two keys — plus a right match edge so `/v1/snapshots` cannot match inside `GET /v1/snapshots/{}`.
- ⛔ **No LEFT edge, measured rather than assumed.** It guards a hazard this surface has **0** of, and it would cost a real answer: `curl -i "$RB_URL/v1/admin/grants"` would read `absent`. Instead `tail_collisions()` prints the count every run, so the day the hazard appears the reader is told — the assumption is checked, not commented.
- 🔴 **The old self-test's third arm asserted `stem("/v1/calls/{call_id}/respond") == "/v1/calls/respond"`** — codifying a key no book writes as the correct answer. Not a control that could not see the defect: one that demanded it. Falsified five ways, each red by name; a sixth attempt was a bad mutant of mine (`return [] or sorted(...)` evaluates to the sorted list) and is recorded rather than counted.
- ⛔ Unchanged: 52 is not 52 defects, `described` is still a proxy, and no gate is proposed — a better key does not touch `.11.6`'s reasoning.

## 2026-09-18 — A citation is withdrawn by its tenant; a shared row is tombstoned by the site (`SIGNOFF-REPAIR.7.4.4`)

🔴 **One verb carried two acts that do not share an authority. A snapshot two tenants cite is ONE row, so `DELETE /v1/snapshots/{id}` removed tenant B's evidence when tenant A asked — and stamped B's receipt with A's reason.**

- **Reproduced RED first, and the reproduction printed the defect.** Two tenants submit the same `(locator, digest)` pair; the second gets `replay: true` and the SAME `snapshot_id` — one row, two citers. A deletes, and B's read returns `"deleted_at":"2026-09-18T09:09:11Z","deletion_reason":"tenant A no longer relies on this"`. `.11.14.1` had bound the verb to a CITING tenant, which stops a stranger and does not separate two citers, because both are citers.
- ✅ **The split.** *Withdrawing a citation* is a statement about one tenant's own reliance and is the tenant's to make. *Tombstoning the row* is a statement about shared bytes, which needs the authority the retention sweep already needs — `retention_class` is a column on the shared row, `.7.4.3`'s own argument. So `DELETE` withdraws, and `POST /v1/snapshots/{id}/tombstone` is a site act under the existing `evidence_expire` grant.
- ⛔ **Two obvious alternatives rejected for the same reason.** *Refuse the delete when another tenant cites the row* and *tombstone only when the last citer leaves* both make one tenant's observable outcome depend on whether a STRANGER cites it — the cross-tenant existence §9.8 forbids. The second is worse: it hands any tenant the shared-row authority by the back door of being the only citer.
- ✅ **The withdrawal is RECORDED, not deleted** (§12.9, *never a silent disappearance*). `migrations/0070` adds `withdrawn_at`/`withdrawn_by`/`withdrawal_reason`; a plain `DELETE` would have left no answer to *who stopped relying on this, when and why*. Re-citing restores the row and keeps the original `cited_at`/`cited_by`.
- ✅ **The irreversibility is removed for the tenant and kept for the tombstone.** A withdrawal is undone by citing again; a tombstone stays permanent but is now reachable only through an authorized, audited site act. Undoing one would make the §12.9 record of it a lie.
- ✅ **Falsified four ways, each red BY NAME**: the withdrawal tombstoning again (B's sentence, plus the roundtrip control), the disclosure predicate dropping `withdrawn_at IS NULL` (A's 404), `record_citation` back to `DO NOTHING` (the restore arm), and the site route with no authority (the 403 arm). `profiles` 56/0, `evaluation` 3, `command_api` 39, `migration_upgrade` 4, clippy `-D warnings` rc=0.
- ✅ `the_snapshot_store_roundtrips_replays_and_tombstones` is renamed `…_and_withdraws` with its three moved assertions named in the body — nothing was deleted to make it green.
- 🔎 Two findings routed rather than reported: **`.11.8.1`** — the route census's `stem()` collapses `/v1/snapshots/{id}` onto `/v1/snapshots`, so an item route is counted *described* by the line documenting the submit route, while a mid-path parameter yields a stem no book writes; `.11.8`'s published `35 described / 68 absent` is wrong in both columns. **`.11.21`** — `git.rs` is committed unformatted, because the per-commit enforcer has no formatting check.

## 2026-09-18 — A retention rule with no retirement, on both populations at once (`SIGNOFF-REPAIR.7.3.2.1`)

🔴 **Two test suites retain a workspace on failure, deliberately and correctly — and nothing had ever retired one. Measured at closure: 1,896,248,619 bytes across 37 fixtures, of which 99.93 % is reproducible payload and 1.34 MB is the evidence they exist for.**

- **The premise had got worse while the leaf sat pending.** `target/pg-tests` went from the 12 clusters / 599 MiB this leaf recorded when it opened to **25 clusters / 1,372,355,705 bytes** in two days. `target/browser-lifetime-controls` held 12 fixtures / 523,892,914 bytes.
- ⭐ **What a retained fixture must keep was measured, not chosen.** Of one fixture's 56,546,450 bytes, `profile/` is 56,394,675 — 36.7 MiB of it a single `model.tflite` Chrome downloaded — and the evidence is 1,293 bytes in 7 files. ⛔ **Two of those seven sit INSIDE the payload directory**: `owner.json` and `completion.json` are the production worker's own receipts, carrying `browser_group` and `cleanup_confirmed`. Dropping `.project-data` wholesale — the obvious rule, and the first one written — would have destroyed exactly the record `.7.3.2` exists to preserve.
- ✅ **One rule, both populations: keep the receipt, drop the reproducible payload.** `scripts/census_retained_fixtures.py` censuses and reduces. It **never deletes a fixture and never signals a process** — every liveness probe is signal 0, honouring `.7.3.2`'s "never signal a historical numeric PID solely from a stale receipt". An id in use keeps a fixture, so a recycled id costs disk, never evidence.
- ✅ **Result: 34 of 37 fixtures reduced, 1,824,989,121 bytes dropped, 0 deleted**, residue census green and `du` agreeing independently (`target/browser-lifetime-controls` 511,614 KiB → **384 KiB**; `target/pg-tests` 1,340,190 KiB → **70,464 KiB**). ⭐ The citation guard fired in PRODUCTION rather than only in its self-test, keeping `run-9_ueev0t` whole because three tracked files name it.
- 🔴 **Mutation found three of this instrument's own controls vacuous.** Flipping `except PermissionError: return True` to `return False` — which would let it reduce a fixture whose browser is alive under another uid — passed every arm, because all nine liveness arms probed *our own* pid, where signal 0 succeeds and the EPERM branch never runs. Deleting the kept-tree equality check entirely passed too. So did removing the pre-removal re-check, which lived inline in `main()` where nothing drove it. All three are repaired with arms that fail for their own reason, and all six mutants are now red.
- ✅ `scripts/census_pg_test_clusters.py` removes a cluster outright, a different disposition owned by `.11.4.3.1.7` and left intact. It now refuses any cluster carrying `retired.json`, so it cannot undo this repair.
- 📄 Promoted: `docs/knowledge/a-guard-your-own-process-cannot-reach.md`.

## 2026-09-18 — Three ref clauses, and a fourth defect that made all three unreachable (`SIGNOFF-REPAIR.7.2.8`)

🔴 **`acquire_into` passed the URL's fragment to `remote_at`, so every selector-bearing acquisition failed at the remote. The three clauses this leaf was opened for all sit downstream of a path nothing could reach.**

- **Why it survived:** nothing ever drove the feature. **13** `acquire_local` call sites, none with a fragment; the only `#refs/tags/v1.0` in the suite is a `fetch_refspec` *unit* test that never reaches a transport. 🔎 A careful reading found three real defects in unreachable code, and the FIRST control found the reason it was unreachable — `TOOLBOX.md`'s tools-first argument arriving from the other side.
- ✅ **An annotated tag resolves, and that was decided rather than defaulted.** It advertises as `Ref::Peeled`; the match covered only `Symbolic`/`Direct`, so it reported `NoHeadRef` for a ref the remote had advertised. Refusing by name was weighed and loses on the contract: the acquisition exists to produce an immutable commit, a tag is the commonest way to name one, and `Peeled` hands over exactly that commit.
- ✅ **An ambiguous short name is refused BY NAME**, listing both full refs — and only when the two point at **different** commits, so a repository that merely tags its own branch tip still resolves. ⛔ Refusal rather than `gitrevisions`' precedence order, deliberately: this is not a CLI, and a silent pick between two commits is the shape this project refuses everywhere else.
- ✅ **`refs/heads/..` is refused.** The `refs/` arm returned early, so the `..` guard below it never saw a `refs/` selector while the charset admits `.`.
- ✅ **Two falsification passes, because one could not separate the defects.** Neutralising the three resolution repairs together reds three arms by name — the ambiguity one reporting the arbitrary winner as an observation. The fragment repair needed its own pass: with it neutralised, all three selector controls fail at the remote and would have masked whether the others fired for their own reasons.
- ⭐ **`ACQUISITION-KIND-DOC`, registered one leaf earlier, refused the change** — `python3 -B scripts/census_reason_codes.py --check` named the undocumented `ambiguous_ref_selector` the moment the wire mapping was added. ⚠️ Precisely: it refused the CHANGE, by command, before any commit was attempted; the pre-commit hook never had to fire, because the check was run by hand first. A gate catching its own author within the hour is better evidence than its calibration was.
- ⭐ `.7.2`'s mechanism table is now fully discharged: consumption bounds by `.7.2.7` and `.7.2.9`, supported refs here.

## 2026-09-18 — A safety default disabled by our own correctness (`SIGNOFF-REPAIR.7.2.9`)

🔴 **`gix` ships a 16 MiB per-object allocation brake. This project had it switched off — by the very property that makes its acquisition repositories safe in every other respect.**

- **The mechanism, read in the pinned dependency.** `gix-pack 0.74.2` refuses to allocate for an object larger than `alloc_limit_bytes` (`gix-pack-0.74.2/src/data/file/decode/entry.rs:475`), and gix supplies `ALLOC_LIMIT_IF_REDUCED_TRUST_DEFAULT` = 16 MiB. ⛔ It applies **only at `Trust::Reduced`**, and `init_opts` sets `git_dir_trust = Some(Trust::Full)` unconditionally (`gix-0.87.1/src/init.rs:76`). Every acquisition repository goes through `init_opts`, so the brake never engaged: a remote-supplied pack entry could declare any size and gix would ask the allocator for it.
- ⭐ **The defect lives in the seam between two correct decisions, in two codebases.** The repository is fully trusted *because the server created it* — the same fact that makes `open::Options::isolated()` the right config posture — and full trust is exactly what disables the limit. Nobody would find this by reading either codebase alone.
- ✅ **The repair refuses nothing that would have succeeded**, which is why this limit and not gix's. It is set to `max_bytes`, the ceiling the acquisition already declares for its whole object database: an object larger than that could never have passed anyway. gix's 16 MiB was rejected for the opposite reason — it sits *below* what this project's own ceilings permit.
- ⛔ **No crafted pack, and the leaf says so.** The acceptance offered *demonstrate it or withdraw the concern*; reading the dependency established both that the bound exists and the exact condition disabling it — stronger than a synthetic fixture, and it does not rest on one nobody will re-run.
- ✅ The control proves the setting **lands** (`an-injection-must-be-shown-to-land`); what gix does with the limit is gix's behaviour, tested by gix. One arm red by name, the negative arm labelled. 17 tests pass, clippy `-D warnings` rc=0, and the book's ceiling table gains the row.

## 2026-09-18 — A second refusal vocabulary, documented nowhere (`SIGNOFF-REPAIR.7.2.10`)

🔴 **An acquisition refusal travels in `acquisition_error.kind`, not in `code`. Forty-one strings a client branches on, and the book documented none of them.**

- ⭐ **It is NOT a `REASON-CODE-DOC` gap, and checking that first is why this is a leaf rather than a correction.** The struct carries `kind`; the §9.8 registry, `KnownReasonCode` and the book's error table are all about `code`. The two vocabularies share **0** strings — measured. So the census keying on `code:` was right, and the gap it left was real.
- ⭐ **Derived from the PRODUCERS, and three keys were wrong before the fourth was right** — `a-census-is-as-wide-as-its-key`, three times in one leaf. A `kind: match &error {` marker missed the `GitError` site, written without the `&`, hiding 16 kinds. A `kind:\s*"…"` regex also matched `derived_kind:` and `actor_kind:`, sweeping in `chunk`, `database`, `approval`, `decision`. A fixed 50-line window overran `FetchError::kind` and took the `snake_case` of a `#[serde(rename_all = …)]`. All three are pinned as arms, each with its own red.
- ✅ **The instrument's 41 agrees exactly with a hand derivation reached a different way**, and the book's table was ASSERTED equal to the producers before it was written — a transcription slip could not survive. Descriptions come from each variant's own `Display` message.
- ⚠️ **The set is OPEN at five sites** that forward a worker-chosen kind, and the page says so: a client must preserve an unrecognised `kind` exactly as it preserves an unrecognised `code`.
- ✅ **Gated as `ACQUISITION-KIND-DOC`, calibrated at 0 of 200 commits**, standing population discharged to 0 first, and fired RED end to end — removing one book row takes the enforcer from rc=0 to rc=1 naming the field.
- ⭐ **`.11.18.2`'s discriminator applies in the unusual direction:** the `code` half has long been gated, so a replay of it would be survivorship; the `kind` half was policed by nothing, which is what makes this zero real.
- ⛔ A third hardcoded self-test total is counted now. This one was **accurate** at 14 — which is the point: it was one added arm from being wrong, and this leaf was that arm.

## 2026-09-18 — A refusal wired end to end and constructed by nothing (`SIGNOFF-REPAIR.7.2.7`)

🔴 **`GitError::TimedOut` was declared, carried a message, and was mapped to a wire string — and nothing could produce it. `GitLimits::max_time` was declared, defaulted to 120 s, and read by no code path. `gix` was handed an interrupt flag nobody could raise.**

- **The defect, re-derived and larger than the leaf recorded.** Five of `GitLimits`' six fields were enforced; the sixth bounded nothing. And it was not merely an unread field: the refusal existed in three files — the variant (`crates/reasonbraid-server/src/git.rs:100`), its Display message (`:158`), the wire mapping (`crates/reasonbraid-server/src/api.rs:2646`) — with no construction site.
- ⭐ **The hook already existed.** `gix`'s `receive` polls a `&AtomicBool` throughout the transfer; this module passed `AtomicBool::default()`. Giving it a flag a watchdog raises bounds the **transfer**, not merely the caller's wait — the difference from a `tokio::time::timeout` around the `spawn_blocking` handle, which returns on time and leaves the worker transferring. The chain was read in the pinned dependency: `receive` → `gix_protocol::fetch` → `Bundle::write_to_directory` → `interrupt::Read`, which fails the next `read` once the flag is set.
- ✅ **The "before allocation" clause is NARROWED, with the measurement that narrows it.** All five other ceilings trip AFTER `receive` — objects and bytes from the written object database, files and depth during the tree walk. So consumption before the first allocation check is bounded **in seconds, not in bytes**, and an operator should budget disk for what the link delivers in `max_time`. Published in the book as a six-row ceiling table with a *when it trips* column, read back out of the rendered page rather than trusted from the source.
- ⛔ **Two declines, each on a measurement rather than a judgement.** The decode brake: `gix-pack 0.74.2` has no ratio guard anywhere, but allocates with `try_reserve`, so an absurd declared size errors instead of aborting — a brake of `check_ratio`'s shape would duplicate that, and the aggregate residual it would not cover is routed to `.7.2.9`. The `checked_add` overflow guard: `GitLimits` is only ever `Default::default()` in production and `max_time` reaches no config, environment or request path, so the panic is unreachable; the trigger for revisiting is stated.
- ✅ **One arm red by name** — restoring the pre-fix behaviour in situ makes the acquisition succeed under a zero ceiling, 14 passed / 1 failed — and the two arms that pass both ways are labelled with why.
- Routed: `.7.2.9` (a pack expanding beyond its wire size in aggregate) and `.7.2.10` (`AcquisitionError.kind` is a second, undocumented refusal vocabulary — checked against the struct before it was claimed, and it is **not** a `REASON-CODE-DOC` gap).

## 2026-09-18 — A right conclusion on a false premise, graded separately (`SIGNOFF-REPAIR.3.4.3.1.1.1`)

🔴 **A signed-off leaf closed on "nothing enforces the stored value". Three production sites enforce it. Its conclusion still stands — for reasons it did not give.**

- **The premise, refuted by command.** `git grep -n 'expires_at > now()' -- crates/reasonbraid-server/src` returns `crates/reasonbraid-server/src/node_channel.rs:673`, `:1121`, `:1158` — lease acquisition, lease renewal, the usability check.
- ⭐ **Read from the PRE-REPAIR source at `9446034`, not from today's**, because REPAIR-0136 rewrote both call sites. `now_offset()` truncates to whole seconds so `signed = floor(T_sign) + 600`, while the row stored `now + 600` at full precision: `stored − signed = T_now − floor(T_sign)`. `rotate` sampled `now` AFTER signing, so its difference is never negative; `enroll` sampled it **11 awaits and 16 database calls** before signing, so it is negative whenever a second boundary falls in between.
- ✅ **The refusal window is real at `enroll` and unreachable.** It is the final sub-second of a 600 s certificate, and the node rotates at `ROTATE_REMAINING_SECS = 300` — the halfway point. A node inside it got there by clock skew and is within a second of the handshake refusing it anyway.
- ⭐ **Graded on all three §4.1 axes separately, which is the point:** PROSE **holds** and is now re-derived; NUMBER, none carried; NAMED INSTANCE **false, and withdrawn** — exact, no tolerance band. A single verdict would have had to choose between "the record was wrong" and "the record was fine", and both are false.
- ✅ **The control is named by its ABSENCE:** `git grep -c ... 9446034 -- crates` → 0. Nothing pre-repair compared the stored instant to the signed one, and the control the repair added prevents the divergence rather than observing the window. Stated rather than built — a control reachable only by disabling the rotate trigger tests the harness.

## 2026-09-18 — A leaf discharged its own gap claims by having been verified (`SIGNOFF-REPAIR.11.2.5`)

🔴 **`GAP-CLAIM-CENSUS` asked "does this section contain a command?" — and `TASK-ACCEPTANCE` puts one in every closed leaf by construction. Measured: 103 claim lines, 0 undischarged. Inert everywhere.**

- **The defect, by controlled experiment.** Two sections, the identical claim line, one variable: whether a ticked `- [x] **ADDRESSED**` box sat beside it. The one with the box was not flagged. ⭐ A control calibrated against a PROXY for a census rather than against a census.
- ✅ **Six candidate discharge tests priced against the same 103 claims** — same-line 65.0%, same-bullet 59.2%, bullet-or-next 37.9%, within-3-lines 14.6%, boxes-excluded 9.7%, and the shipped rule **1.0%**. ⛔ Proximity was measured and rejected, not skipped: the 34-line incidental case it was aimed at is one it accepts.
- 🔴 **The 9.7% variant's extra nine were CLASSIFIED, not counted, and 6 were FALSE POSITIVES** — a claim inside a box discharged by a sibling box, which is the normal structure of an acceptance record. That classification is what bought the final clause and the difference between 10 standing instances and 1.
- ✅ **1 of 200 commits (0.5%), 1 true positive, 0 false positives** — and the replay is honest by `.11.18.2`'s discriminator: the looser rule was enforced throughout, but the tightening it adds was policed by nothing. Two independent implementations agree exactly at 103/1.
- ⭐ **The predicate shipped with a redundant clause and the falsification sweep deleted it.** `own[j]==own[i]` is subsumed — same bullet implies same section and equal box-ness — and dropping it changed no arm and no corpus number. It was found by a degenerate arm that should have failed and did not.
- 🔴 **The repaired gate's FIRST real catch was a false claim inside a signed-off leaf.** `.3.4.3.1.1` closed asserting *"nothing enforces the stored value"*; `git grep -n 'expires_at > now()' -- crates/reasonbraid-server/src` returns three production sites. ⭐ The record's conclusion survives for a different reason than the one it gave — the signed `not_after` is enforced first at the mTLS handshake — and the direction that argument does not cover is routed to `.3.4.3.1.1.1` rather than waved. Standing population **0 of 104**.

## 2026-09-18 — A replay over policed history measures deterrence, not cost (`SIGNOFF-REPAIR.11.18.2`)

🔴 **Two calibrations of the same candidate returned 0 of 559 commits and only one of them meant anything.**

- **The trap.** Once a rule is enforced, every commit that LANDED had already been edited until it passed, so a replay across that interval returns 0 **by construction** — and that 0 reads exactly like *"this rule would never have fired"*. It measures the gate's DETERRENCE, not the rule's COST, in the same units.
- ⚠️ **It is the neighbour of the window trap, not the same one.** `calibrate-over-the-history-that-contains-the-instance` is about a window too short to contain the defect, and widening fixes that. Here the whole interval is enforced and **widening makes it worse** — more policed history is more zeros.
- ⭐ **ABSORBED into that note rather than extracted, which needed a third clause beyond `.11.20.2`'s criterion.** This is the same activity with a different failure mode, neither thesis nor tool. What decides it: the host's `answers:` line already claims *"my proposed gate catches nothing in the calibration, is it unnecessary"* — so **a note that advertises a question owes every answer to it**, and extracting would have made the host a confidently wrong reply.
- ✅ **Mechanizability answered by census, not judgement.** Of this project's three calibration-bearing instruments, **2 of 3** replay over a corpus their own `--check` gates (`census_positional_refs.py` 12 of 200, `census_broken_tables.py` 10 of 200; `census_mirror_numbers.py` is a review instrument). Detection costs **17 ms** against a 10.8 s replay. ⭐ The case rests on growth, not on 5–6%: the overlap is the fraction of the window after registration, so it reaches 100% exactly when a mature gate is re-argued.
- ✅ **Shipped as a DISCLOSURE, not a refusal** — the post-registration number is the right one when the question is *does the gate still hold*, and only the caller knows which question they are asking.
- ⚠️ The helper is duplicated in two scripts, named rather than hidden: **26 tracked `scripts/*.py`, 0 sibling imports**, so the project's first shared module is a structural change this leaf does not own. Trigger for revisiting: a third instrument needing it.
- 🔴 **Adding the arms caught a live false total.** `census_broken_tables.py`'s self-test printed a hardcoded `23 arms` while running **21** — measured two independent ways. Counted now, the same repair `.11.20.1` made one leaf earlier: the second instance of one `TOOLBOX.md` hazard in two leaves.
- ⭐ Restated outside software: *"how many drivers would this speed limit catch?"*, measured on a road that already has a camera.

## 2026-09-18 — The advisory and the blocker never governed the same corpus (`SIGNOFF-REPAIR.11.18.1`)

🔴 **`GAP-CLAIM-CENSUS` advised on 69 files and enforced on 15, for the whole of its life, because the script spelled its population twice.**

- **The defect.** `--all` iterates `git ls-files 'docs/tasks/*.md'` — **69** files, because git's `*` crosses `/` — while the blocker filtered `^docs/tasks/[^/]*\.md$` and saw **15**. The backlog reported **102 claim lines across 6 files**; enforcement covered **97 across 5**. **54** nested artifacts were invisible to the blocking path, and nothing said so.
- ⛔ **The cause is the two spellings, not the exclusion.** Each path named the population itself, and the `TEMPLATE.md` exclusion lived *inside the staged loop* where `--all` could not reach it. One `governed_filter` now serves both, so the agreement is structural rather than remembered — and the self-test arms run that filter itself, covering both callers by construction.
- ✅ **The "a signoff artifact is a dated record, not a living leaf" defence was refuted by measurement**, not argued down: **66 of 559 commits** modify a nested artifact, and `RECONCILIATION.md` is the clause ledger three tranches are actively writing.
- ✅ **Calibrated and free:** replayed over **all 559 commits**, the extension would have blocked **0** — `REASON-CODE-DOC`'s shape.
- ⛔ **Full history rather than a window, and the control is what showed why.** The same harness over the TOP-LEVEL corpus also returns 0 — and that number is worthless, because the blocker has governed those files since the initial commit, so every commit that landed had already been made to pass. **A replay can only price a rule over history the rule did not police.** Routed to `.11.18.2`.
- ⭐ **The harness was proved before its zero was believed.** A synthetic unbacked claim through the same classifier returns `BLOCKED`; its censused twin returns nothing. Without that pair, *nothing to find* and *broken replay* are the same output.
- ✅ Five in-situ wrong rules, five arms RED by name, no arm needing a label. And the gate end-to-end on a real staged nested artifact: rc **1** after, rc **0** (`NOT EVALUATED`) before.

## 2026-09-18 — A control that passes for an unrelated reason, promoted (`SIGNOFF-REPAIR.11.20.2`)

🔴 **A lesson that had fired in three consecutive leaves was reachable only from the tree it was written in.**

- **The gap, by command.** `grep -rl 'passes for an UNRELATED reason' docs/knowledge/ TOOLBOX.md docs/decisions/` returns nothing; over `docs/tasks/` it returns `SIGNOFF-REPAIR.md`. It was written down inside `a-scoping-defect-errs-in-one-direction`'s method section — reachable by subject, not by question.
- ⭐ **EXTRACTED rather than absorbed, and the criterion is the deliverable.** `.11.17.2` absorbed its lesson one commit earlier, so the opposite ruling needs a named difference: **absorb when the host note's THESIS is your rule; extract when your rule is a TOOL the host merely uses.** ⭐ The host got SHORTER — its method section is now a pointer plus its own instance — which is the test for a good extraction.
- ⭐ **New beyond relocation.** A **NEGATIVE** arm cannot be falsified against the old code: the old code under-reached, so the arm passes against it for the same reason it passes against the fix. *A positive arm is falsified against the past; a negative arm against the future you rejected* — a DEGENERATE implementation of the new rule. And running a suite against broken code is the only time your controls are tested as **reporters**.
- ✅ **Checked outside software**, per `docs/CLAIM_VERIFICATION.md` §0: a fire alarm that has never sounded and one with a flat battery are the same observation, and pressing the test button is also how you learn whether it carries to the stairwell.
- 🔴 **The acceptance's own last clause was UNSATISFIABLE, and that is the second finding.** It required the census to confirm the pointer by reporting `anchored:method`. `classify` was a precedence chain, so naming the leaf SHADOWED the note: a correct pointer, a correct note, and the instrument would not say so. ⚠️ The tempting move — drop the citation until the instrument agrees — is fitting the corpus to the measurement. Both anchors are reported now; the counts no longer partition and the output says so.
- ✅ Falsified from both sides: the precedence version reds `both-anchors` by name, a degenerate *always both* reds `leaf-only-claims-no-method` by name. `MEMORY.md` 3,883 → 3,659 bytes, warnings 8 → 7.

## 2026-09-18 — The memory census saw two warnings where eight stood (`SIGNOFF-REPAIR.11.20.1`)

🔴 **Its model was one bullet; the template the file had been conformed to uses several. 47% of `MEMORY.md` by weight was invisible to the instrument that exists to guard its weight.**

- **The defect.** `run` called `segment(warning_text(memory))`, and `warning_text` returns ONE bullet. Measured at `69cf2dc` before the model was touched: **2** warnings visible, **8** present, **1,801 of 3,809 bytes (47%)** in four bullets never read. `.11.20` fixed WHERE it looks; nothing fixed WHAT it counts.
- ⭐ **The new model is parsed from `MEMORY_ARCHITECTURE.md` §6, not from `MEMORY.md`.** A model fitted to the file it measures agrees with that file by construction and says nothing — and re-deriving it from current contents is how this census went stale the first time. A bullet the template names contributes the text after its key; every other bullet is warning text in its own right.
- ✅ **After:** 8 warnings, **2,312 of 3,809 bytes (61%)**, reported per bullet and per warning against the 7,168-byte cap. Weight is what the cap is about, so a census answering only *how many* to a reader about to evict for BYTES answers the wrong question.
- ⛔ **Two falsifications were needed, and that is the methodological finding.** The pre-fix model reds three arms by name. The NEGATIVE arms have no red against it at all — so a DEGENERATE `segment` (every bullet whole) was run to fire `keyed-bullet-without-marker-is-silent` and `no-markers`. **A negative arm falsified only against the old code is a negative arm nothing has tested.** Arms passing both ways are LABELLED in the source.
- 🔴 **The falsification exposed a defect in the self-test itself.** One new arm raised `IndexError` against the pre-fix model and took the whole suite down — a RED that names nothing, when a naming RED is the entire point. A self-test only ever run against working code never meets this.
- ⛔ **The hardcoded `CONTROL_COUNT = 17` is counted now** — `TOOLBOX.md` had already named the hazard, and it was found while adding arms to the very instrument it warns about.
- ✅ **`UNCITED` was re-decided against the widened population, and the answer changed.** The footer asserted a 2026-09-13 classification: 13 of 13 recorded somewhere durable, so UNCITED only ever costs a pointer. Re-classified over the 4 now visible: **3 recorded, 1 not** — an environment fact living only in `MEMORY.md`, ⭐ written there this same session by this author, into the file whose own first bullet forbids it. Routed to `docs/decisions/2026-09-18_a-cargo-process-is-not-evidence-of-this-repo.md`. UNCITED **4 → 2**, anchored **4 → 6**; the footer now carries both runs and says they disagree.
- Routed: the standing lesson *a control that passes for an unrelated reason* is in a leaf and reachable from nowhere else. `.11.20.2`.

## 2026-09-18 — A slash is not a resolution (`SIGNOFF-REPAIR.11.17.2`)

🔴 **`POSITIONAL-REF` decided a reference was resolvable from the SHAPE of the string. Of 251 such occurrences, 39 named no tracked file and one was a suffix of THREE — the exact ambiguity the gate exists to refuse, passed by it.**

- **The defect, at the source.** `classify` in `scripts/census_positional_refs.py` read `kind = "pathed" if "/" in ref`. The branch for a reference WITHOUT a slash consulted the tracked-file index; the branch WITH one did not. ⭐ The instrument had the check in hand and applied it to one branch only — `docs/knowledge/trust-comes-from-the-check-not-the-shape.md`, which now carries this as a second instance.
- **The population, pinned so it stays re-derivable.** `python3 -B scripts/census_positional_refs.py --at 6f91897`: of **251** occurrences containing a slash, **212** name a tracked file, **27** are a path-suffix of exactly one, **1** of three, **5** are dependency citations and **6** name nothing. ⚠️ The leaf's own opening table said 28/5 where the instrument says 27/6 — the totals and the headline 251/212/39 reproduce exactly, but the split came from an unrecorded pipeline and is reproducible by neither obvious suffix rule (segment-boundary gives 27/6, plain-string gives 31/2).
- **The repair is the OLD question asked of both branches**, which produced the classes rather than arguing them: `pathed` (exact), `partial` (a segment-boundary path-suffix of exactly one — accepted, as `unique` already is), `ambiguous` (several), `unresolved` (none), and `dependency` for a `<crate>-<version>` segment `Cargo.lock` pins.
- ⛔ **A suffix must fall on a SEGMENT boundary.** `crates/reasonbraid-core/src/authority.rs` ends with the *string* `core/src/authority.rs` and does not contain it as a *path*; a plain `endswith` would have turned this instrument's founding 93-false-positive hyphen bug into a false NEGATIVE. Pinned as self-test arm 17.
- ⭐ **The dependency class earns a failure mode, not an exemption.** `Cargo.lock` is the oracle, so `dependency-stale` — a citation into a version the project no longer builds — is now refused. 0 standing instances; both directions in the self-test.
- ⛔ **Two declines, on different grounds.** The in-crate path is NOT verified (it needs the vendored registry a cold clone lacks — green here, red there). Refusing `partial` is DECLINED on SHAPE rather than cost: priced at **4.5%** of 200 commits and affordable, but a partial path naming one tracked file resolves, and refusing it would gate tidiness where the doctrine gates ambiguity.
- ✅ **Calibrated before proposing** (`.11.6`): the three refused classes would have blocked **21 of 200 commits (10.5%)**, against the **9.5%** that argued this gate in and the 87%/93% that got two candidates rejected. Standing population **8 → 0**.
- ⭐ **And `--calibrate` stopped approximating.** It re-scanned only when a source BASENAME entered or left the tree — sound for a basename gate, wrong for a path one. Caching the PARSE and redoing the RESOLVE re-classifies every commit exactly for **1,713 blob reads** instead of ~70,000: **10.8 s for 200 commits**, where the approximation existed because an exact pass was thought to cost nine minutes.
- **Falsified in situ:** restoring the pre-fix classifier makes **11 of the 12 new arms fail by name** (13/24), the state the repository was actually in. The two that pass both ways are LABELLED in the source. The gate itself was fired red three times in the real enforcer, rc 0 → 1 per class.
- Routed: none. Next is `.11.20.1`.

## 2026-09-18 — The instrument guarding `MEMORY.md` had been dead since the commit that reshaped `MEMORY.md` (`SIGNOFF-REPAIR.11.20`)

🔴 **A census refused on every run for dozens of commits while its own `--self-test` reported 16 controls passing — and the defect it exists to prevent recurred in the meantime.**

- **The defect.** `python3 -B scripts/census_memory_warnings.py` exits 1 with `census: MEMORY.md has no '- **Next action:**' bullet`. It keys on a literal bullet name; `6199f43` — `SIGNOFF-REPAIR.11.4.2.3`, *"conform the resume pointer to the template that governs it"* — renamed that bullet to `next_action:` and did not update the reader. ⛔ **The commit that reshaped the file is the commit that blinded its guard.**
- 🔴 **`SELF-TEST` was green on it throughout.** All 16 controls are built from fixtures carrying the OLD bullet name. `TOOLBOX.md` already says *"a self-test written alongside the code shares its blind spots"*; this is the sharper case, where the blind spot was introduced later, by a commit to a different file.
- 🔴 **And the defect recurred.** `SIGNOFF-REPAIR.11.4.2.2` cleared `MEMORY.md` from 7,168 of 7,168 bytes to 1,395. One session of closing leaves took it back to **6,123** — 85% of the cap.
- ⭐ **The recurrence is `.11.16`'s shape, which is why the repair is not eviction.** A per-line byte census: **six standing-lesson bullets hold 3,983 bytes, 65% of the file**, and five of the six were PROMOTED to `docs/knowledge/` in the same session and then restated here in full. A layer-A pointer restating what layer C holds is a mirror nothing derives. ✅ **6,123 → 3,825 bytes with nothing lost** — each lesson is now a named slug, and all nine targets were verified tracked BEFORE the eviction.
- ⭐ **The arm that would have caught it reads the REAL file**, and asserts almost nothing about its content — only that the census can still locate what it is about. An arm coupled to the live wording fails on every honest edit and gets waived. Falsified by name: with the pre-fix key restored, `SELF-TEST FAIL live-corpus: the real MEMORY.md is unreadable to this census`, while the other 16 pass.
- ⛔ **The general gate is DECLINED on COST, not on shape.** *Every census still runs against its real corpus* would have fired on **1 of 12** before this commit and **0 of 12** after — `REASON-CODE-DOC`'s shape, which normally ships. It costs **3.9 s** against an enforcer costing **11.6 s**, a 34% increase, and `SELF-TEST` already runs each instrument's self-test — so the same property is free as a one-line arm inside it. ⭐ Compare the measurement that argued `SELF-TEST` in: 1.01 s on a 3.15 s enforcer. Four seconds on eleven is a different argument.
- ⚠️ **Honest limit:** nothing mechanically requires a census to HAVE a live-corpus arm — the `GAP-CLAIM-CENSUS` archetype. The remedy is a `TOOLBOX.md` statement where the next census author is standing.
- Routed: the census's model of WHERE warnings live is stale too — it counts 3 in one bullet and cannot see the 3,983 bytes in six others. `.11.20.1`.

## 2026-09-18 — `init.rs` was never in this repository (`SIGNOFF-REPAIR.11.17.1`)

🔴 **Eight published citations named "a file that no longer exists". It was a dependency's source all along, and every cited line is exact at the pinned version.**

- **The leaf's own premise was wrong**, and history says so rather than inference: `git log --all --diff-filter=A --name-only -- '*init.rs'` returns NOTHING. No such file has ever been tracked here under any name. The citations name `gix-0.87.1/src/config/cache/init.rs`, cited to explain how `gix` loads git configuration.
- ⭐ **And every one of the seven cited lines resolves to the code its prose quotes** at the version `Cargo.lock` pins — `:229` is `system: use_system,`, `:243` is `gix_config::file::includes::Options::follow(`, and so on. That is what makes the repair a completion rather than a guess.
- ⛔ **17 citations qualified, not the 8 the leaf counted.** Its census keyed on the bare basename `init.rs` and missed the partially-pathed `src/init.rs`, `src/lib.rs` and `src/open/permissions.rs` in the same tables — `a-census-is-as-wide-as-its-key`, inside the leaf that cites it.
- ⭐ **DECIDED: a dependency citation is written crate-and-version qualified.** It resolves for a reader AND dates itself, which a bare basename never could: a line number into a dependency moves on every upgrade, and the version is the only thing that says which source it was exact against. Rejected: annotating with this repo's commit (wrong axis — the drift is the dependency's), accepting it as a property of dated records (the `.11.17` precedent refuses it), and exempting third-party references (an exemption list to maintain, and it would have left the one genuinely unresolvable instance uncaught).
- ✅ **THE EXAMPLE-VERSUS-CITATION OBSTACLE IS ANSWERED, AND NOT BY TELLING THEM APART.** The leaf named it as the real design problem: the placeholder in `POSITIONAL-REF`'s own registry row is indistinguishable from a broken reference by basename alone. No instrument can make that distinction, so the gate does not try — **an illustrative example may not be WRITTEN in the positional form**, and the row is reworded rather than excluded. The fourth time this doctrine has policed its own description.
- ✅ **`unresolved` 9 → 0, and the arm ships**, calibrated before it was proposed: **2 of 200 commits (1.0%)**, and both are the instances discharged here. Zero false positives.
- ⚠️ **The first calibration did not finish** — re-classifying all tracked Markdown at every commit is ~70,000 blob reads; abandoned after nine minutes. `--calibrate` is now incremental and lives on the tracked instrument, so the number is re-derivable rather than believed.
- 🔴 **The new arm flagged this leaf's own closing prose on its first run** — writing the decision required naming both offending shapes, and naming them reproduced them. Both reworded.
- 🔎 **Routed: measuring this class measured the one next to it, and it is larger.** `pathed` is assigned on the presence of a slash alone, with no check that the path exists: of **251** `pathed` occurrences **39 do not resolve**, and one is a suffix of **three** tracked files — an ambiguous reference waved through by the gate whose purpose is refusing one. `.11.17.2`.

## 2026-09-18 — A table SWALLOWS the block that abuts it (`SIGNOFF-REPAIR.11.19.2`)

🔴 **`PHASE-3`'s eleven-line closing statement was rendering as eleven table rows, under a table header that had no rows of its own — and the gate that now catches it would have blocked every one of the nineteen commits in this project's history that introduced a table defect.**

- **The defect, and it is the MIRROR of the previous entry.** That one was a blank line where none belonged, SPLITTING a table. This is no blank line where one belongs, so the following block is ABSORBED: a GFM table body continues across any non-blank line, and each line becomes a row padded to the header's width. The renderer put *"**Tree complete.** Phase 3 is CLOSED…"* inside `<td>`.
- ⭐ **The repair is the corpus's own convention, not a judgement.** The table was a header and a delimiter with **no data rows at all**; `docs/tasks/PHASE-1.md` and `docs/tasks/PHASE-2.md` already close the same way — `## Current Frontier` then prose, no table. So the rowless pair is removed rather than a blank line inserted, which would have left a stray empty table no other tree has.
- ✅ **Verified by the RENDERER:** data rows **59 → 48** (−11, exactly the absorbed lines), tables **3 → 2**, and the paragraph moves from `<td>` to `<p>`.
- ⭐ **CALIBRATED OVER THE FULL HISTORY, AND THE WINDOW IS WHY.** Over the last 200 commits the rule blocks **1 (0.5%)** — but the `PHASE-3` instance is older than that, so the usual window cannot see the arm being added at all. Over all **551** commits it would have blocked **19 (3.4%)** — and every one of the nineteen introduced a defect this work has since repaired. ⛔ Not "3.4% looks acceptable": **the blocked set and the defect set are the same set.**
- ⛔ **The population was over-counted by a third on the first pass.** Keyed on "any non-blank body line not starting with a pipe" it reports **15**; four of those are a bullet list, and a list item ENDS a table. The answer is **11**. ⭐ Which constructs terminate a table without a blank line is NOT uniform and cannot be read off a specification — a list item, heading, blockquote and HTML block do; plain prose and indented continuation do not. Each was rendered separately; each is an arm.
- ⭐ **`BROKEN-TABLE` gains it as a SECOND ARM rather than a second doctrine** — both halves are the same subject, where the author put the table's boundary, measured over the same corpus by the same instrument. **23 arms, ten of them negatives.** Falsified against the real historical defect rather than a fixture.
- ⚠️ **What this does not do:** `TABLE-ARITY-RATCHET` still cannot see an absorbed line — one cell in a four-column table, and GFM PADS a short row, the silent direction that gate already declares. Not repaired, and it need not be: the cause is now caught, so widening the arity gate would be a second instrument reporting the same defect.

## 2026-09-18 — Twelve verification-log rows had lost their first two cells (`SIGNOFF-REPAIR.11.19.1`)

🔴 **An insert-at-top edit re-emitted the row it displaced without its date and leaf id — twelve times, over twelve commits — and asking the renderer why found two defects in the gate shipped one commit earlier.**

- **The defect.** Twelve rows of `docs/tasks/PHASE-2.md`'s Verification Log render with their columns SHIFTED LEFT and padded with two empty cells: the date and the leaf id are simply gone from the page. `git show e7a829c` has the `PHASE-2.4.3` row added well-formed; `git show bb42f65` has the hunk that dropped its first two cells while prepending a new row above it.
- ✅ **All 12 recovered unambiguously**, by matching each pipe-less line as a SUFFIX of every well-formed `| \`2026-…\` |` row in the file's history. ⭐ Confirmed by a property the match never uses: the recovered leaf ids come out strictly descending and continue the two rows above them.
- ✅ **Verified by the RENDERER, on a property no other row shares:** rows whose first cell is a date go **21 → 33**, rows ending in two empty cells go **12 → 0**, and the total row count is **76 both ways**. ⛔ That invariance is the point — a row-count check would have reported success before the repair.
- 🔴 **AND THE SECOND HALF OF THE ACCEPTANCE FOUND TWO DEFECTS IN `BROKEN-TABLE`, ONE COMMIT OLD.** Asked rather than assumed, mdbook says a table body continues across ANY non-blank line — `3 | 4` renders as two cells, bare prose as one, padded. The gate's scanner ended a table at the first pipe-less line, which is wrong in both directions at once: a **FALSE POSITIVE** reporting two adjacent tables separated by a blank line (ordinary Markdown) as 3 orphaned rows, and a **FALSE NEGATIVE** missing a blank line later in a table containing a pipe-less row.
- ⭐ **The terminators are not uniform and no specification reading would have produced them.** A list item, an ATX heading, a blockquote and an HTML block END a table with no blank line; plain prose and indented continuation text do NOT. Established one construct at a time against the renderer; that distinction took the mirror defect's population from an over-counted 15 to the real 11. All four are self-test arms; the gate now carries **18**.
- ⭐ **Falsified in situ:** the pre-fix scanner run against the new arms fails 11, 12 and 13 by name (3 / 0 / 0 against 0 / 1 / 1); the two no-regression arms pass both ways and are labelled.
- 🔴 **CORRECTION to the previous entry.** REPAIR-0238's entry says one of the twelve rows "renders outside the table entirely". It does not — all twelve are rows with shifted columns. That reading came from a probe whose key, the bare phrase *"inventory-groundwork deferral record"*, occurs **7 times** in the file, and `str.find` returned the first, ordinary prose **1,889 lines** above the row it was meant to find. ⚠️ A key too LOOSE fails the opposite way to one too narrow: not an undercount, a wrong instance. The live documents that restated it are swept; this ledger entry carries the correction rather than the previous one being rewritten.
- Routed: **11 lines of `docs/tasks/PHASE-3.md` are a wrapped closing paragraph ABSORBED into its frontier table** — the mirror of `.11.19`, visible only once the table model was corrected. Neither table gate reaches it. `.11.19.2`.

## 2026-09-18 — A blank line ENDS a Markdown table, and one sat inside the tree's own frontier (`SIGNOFF-REPAIR.11.19`)

🔴 **52 table rows across 3 tracked files were rendering as paragraphs of literal pipe-text — including all 46 rows of the active tree's Current Frontier, row 1 among them.**

- **The defect.** A blank line TERMINATES a GFM table. Every row after it stops being a row and comes back as one paragraph. ⛔ The SOURCE looks perfectly fine, which is why this survived every review that read the file rather than the page.
- ⭐ **Asked of the RENDERER, not the specification** — `TABLE-ARITY-RATCHET`'s practice, and it corrected two rules the ad-hoc probe had wrong: a delimiter row whose cell count differs from its header is **not a table at all**, and four spaces of indent is a **code block** while three is still a table. Each rule is rendered before being asserted, and each is a self-test arm.
- ⚠️ **The narrow key found 1; the wide key found 51.** Keyed on *"a blank right after the delimiter row"* the census reports one instance. Keyed on *"a blank anywhere in the body"* — the shape that actually ends a table — it reports **5 blanks across 3 files**. `a-census-is-as-wide-as-its-key` for the third time in one session, and by far the widest miss.
- 🔎 **`TABLE-ARITY-RATCHET` governs all three files and cannot see it**: it compares a row's cell count against its header's, and an ORPHANED ROW HAS NO HEADER to disagree with. `BOOK-LINKS`' founding shape a third time — *the doctrine's intent was satisfied and the rendering broke*.
- **The five blanks were decided MECHANICALLY, not by eye.** A stray blank and a deliberate separator between two tables differ observably: cells on each side, and whether a second header + delimiter follows. All five measured **4 cells before, 4 after, no delimiter after the blank** — so none is a second table, and deletion is right for all five.
- ✅ **All 52 discharged, verified by the RENDERER**: `<tr><td>` counts go `157 → 203`, `53 → 56`, `29 → 32` — **+52 exactly**, matching the census by a different route.
- ⭐ **`BROKEN-TABLE` SHIPS, calibrated across 200 commits before it was proposed** (`.11.6`): it would have blocked **1 commit (0.5%)**, and that commit is the one that INTRODUCED the defect. Zero false positives — `REASON-CODE-DOC`'s shape, not the backlog shape rejected at 87% and 93%. ⚠️ Three of its twelve arms are NEGATIVES (a blank that legitimately ends a table, a table at EOF, a table in a fence); without them the rule degenerates into "no blank line near a table".
- 🔎 **`86dd272` is the same commit that introduced `.11.16`'s stale `SELF-TEST` numbers** — one commit, two defects of different families, found by two instruments two leaves apart, neither visible in its diff.
- 🔴 **The leaf published a `0` that measured `12`, and checking it is what found the next defect.** Its boundary bullet claimed no table row in the corpus lacks a leading pipe. Twelve do, all in `docs/tasks/PHASE-2.md`, and they are verification-log rows that LOST their first two cells — one renders outside the table entirely. Both table gates are blind to the shape. The superseded claim is kept rather than edited into agreement. `.11.19.1`.

## 2026-09-18 — A restated number needs a PRODUCER, not a rule (`SIGNOFF-REPAIR.11.16`)

🔴 **The file that documents this project's gates calls itself "the human-readable mirror of the registry", and four of the numbers it mirrored had gone stale with nothing deriving them.**

- **The defect.** `DOCTRINE_ENFORCEMENT.md`'s `SELF-TEST` row said *"all 17 pass"* and *"11 of the 28 check/census scripts"*; measured, **29** and **9 of 38**. Its `FILE-TERMINATION` row said *"642 files scan"*; measured, **773**. ⛔ Every one was ALSO stale in the instrument's own header comment, so the registry was a mirror of a mirror and neither copy had a producer. Third instance of the shape `BOOK-FRONTIER` and `INDEX-FRONTIER` already gate.
- ⚠️ **The leaf's own first pass was off by a factor of seventeen** — it reported *"6 bolded numerals"* because it keyed on **bold**, and the file's numbers are mostly unbolded. The real population is **103 numerals across 19 rows**, by `scripts/census_mirror_numbers.py` (tracked, `--json`, `--calibrate`, `--self-test` with 12 arms).
- ⛔ **ALL THREE CANDIDATE GATES ARE MEASURED UNSOUND AND DECLINED**, each priced before it was proposed (`.11.6`): a sentence-level citation requirement fires on **68 of 103 (66%)**; minus four mechanical structural exclusions, **51 of 103 (50%)**; scoped to the staged diff — the `GAP-CLAIM-CENSUS` precedent — it would have blocked **10 of the 14 commits in 200 that add a numeral (71%)**. `.11.9`'s gate was rejected at 87% and `.11.15`'s at 93%; `POSITIONAL-REF` shipped at 9.5%.
- ⛔ **And a per-numeral allowlist is refused by this project's own sentence**, in the registry row two lines from the defect: `VISIBILITY-POLICY`'s *"an allowlist thirty entries long teaches bypass"*, against a population of **103**.
- ⭐ **THE MEASUREMENT REDIRECTED THE WORK.** The four stale numerals share a property the other 99 do not: each is a POPULATION SIZE the named instrument enumerates on every run and simply never printed. So `scripts/check_self_tests.sh --census` and `scripts/check_file_termination.sh --census` ship, the rows cite the command, and **no gate is registered**. A number derived on every run cannot go stale.
- ✅ **`REASON-CODE-DOC` is discharged by DELETION, not correction.** Its numbers had drifted since `.9.2.1.1` and became true again BY ACCIDENT when `.11.14.3.2` retired a code. Correcting them to today's was the cheap fix and the wrong one — a mirror that happens to agree teaches a reader it never drifted.
- **Also discharged:** `HEADING-DEPTH`'s founding measurement, which read as a present-tense claim and is false today (0 violations), re-anchored to the commit that measured it; and `TABLE-ARITY-RATCHET`'s self-test banner, which printed a hardcoded `9/9 arms` beside nine arms and now COUNTS them — falsified by injecting a tenth arm (`10/10`), restoring byte-identical, and reading `9/9` again.
- **ADDRESSED:** `bare` numerals **51 → 37**, all five repaired rows clear. ⚠️ The remaining 37 are unreviewed by any instrument — read once by hand and judged frozen. That is the declined gate's cost, named rather than hidden.
- Promoted: `docs/knowledge/a-restated-number-needs-a-producer.md` + a `TOOLBOX.md` statement. 🔴 Routed: **a blank line ENDS a Markdown table, and one sits inside this tree's own Current Frontier** — 45 rows render as literal pipe-text, 51 across 3 files once the key is widened from "after the delimiter row" to "anywhere in the body". `.11.19`.

## 2026-09-18 — A positional reference is exact only if a reader can resolve it (`SIGNOFF-REPAIR.11.17`)

🔴 **29 published source citations named two files each, and `DOCPATH` could not see any of them.**

- **The defect.** `docs/CLAIM_VERIFICATION.md` §4.1 grades a NAMED INSTANCE as exact with no tolerance band. A bare `profiles.rs` with a five-digit line names a 606-line source AND an 11,154-line suite, and the prose does not say which. ⭐ `DOCPATH` already wants repo-root-relative references — a bare basename SATISFIES it while naming nothing, which is `BOOK-LINKS`' founding shape exactly.
- ✅ **Leg 3 closed first:** `scripts/census_positional_refs.py` is the tracked producer (`--check`, `--json`, `--self-test`, 9 controls) and the ad-hoc pipeline is retired. **494 → 499 occurrences, 321 distinct, pathed 250, unique 240, ambiguous 0, unresolved 9.**
- ⭐ **The leaf's 29 reconciled EXACTLY once the unit was named:** 29 DISTINCT references, 47 OCCURRENCES. Recorded rather than quietly reconciled — a number that moves between two honest measurements is what leg 1 exists to catch, and the answer was a definition, not a defect.
- ⭐ **DECIDED: a positional reference carries a repo-root-relative PATH, and the gate ships — calibrated across 200 commits BEFORE it was proposed** (`.11.6`, which this leaf owed): of **415** references added, **48 were ambiguous (11.6%)**, blocking **19 commits (9.5%)**. Against **87%** (`.11.9`) and **93%** (`.11.15`), both rejected for teaching bypass.
- ✅ **All 47 discharged, population 0** — 17 by line count alone, 30 by reading the prose and CONFIRMING against the file. Several confirmations were exact: `crates/reasonbraid-server/src/policy.rs:151` is literally `fn is_semver(...)`.
- ⛔ **It gates AMBIGUITY, not DRIFT, and the discharge proved the distinction:** five references had already drifted and were confirmed by grepping the SYMBOL the prose names. Which FILE is fixable and stays fixed; which LINE moves with every insertion above it. The line numbers were not silently rewritten — they were exact when written.
- 🔴 **The gate flagged its own registry row, then its own leaf prose.** Fixed by RE-WORDING both times rather than by an exclusion, so the doctrine polices its own description — the `STORAGE-LOCALITY` founding incident, twice.
- 🔴 **And the falsification destroyed three of my own discharges.** The injection went into `LIVE_STATUS.md`; `git checkout --` then restored it to HEAD, taking three unrelated uncommitted repairs with it. Nothing failed and nothing warned. **`docs/knowledge/an-injection-must-be-shown-to-land.md` recommended that restore and is CORRECTED by the failure it caused**: stash rather than checkout, and check the restore for what should STILL be there.
- Routed: **9 references name no tracked file at all** — 8 `init.rs`, plus the registry row's own illustrative placeholder. 🔎 That placeholder is the finding: a gate on this class would have to tell an EXAMPLE from a CITATION, which the ambiguity gate never has to do. `.11.17.1`.

## Historical entries and exact retrieval

This is a recent digest. Older chronology remains in reachable Git history under
the rotation contract in `README_POLICY.md`. This file has rotated twenty-three times;
each rotation names the commit holding the ledger immediately before it, so the
chain walks back without guessing.

Retrieve the ledger immediately before the TWENTY-THIRD rotation (2026-09-18)
from the repository root:

```bash
git show 8f67cbcd37fa8ea598600a0b45e4400d71e7afb7:CHANGELOG.md
```

That snapshot is 95,703 bytes and contains 29 dated entries; its Git blob is
`656f77ac7fc771d8210ae4ede3c54c9aec283027`, and its SHA-256 is
`61f068622b606d4a1eb26fb1c9c2787c72f1e8410eddb302da43589fc1405461`. The newest
entry it holds that this digest no longer carries is
`2026-09-17 — The section scanner did not know what Markdown is (`SIGNOFF-REPAIR.11.18`)`.
It carries the TWENTY-SECOND rotation's notice in turn, which names the ledger before it.

Retrieve the ledger immediately before the TWENTY-SECOND rotation (2026-09-18)
from the repository root:

```bash
git show 78b52e798cff585a6510922ae5cbecce82e0f3c8:CHANGELOG.md
```

That snapshot is 94,680 bytes and contains 25 dated entries; its Git blob is
`7196f600aafecd635b41a80989085eea84dd5eee`, and its SHA-256 is
`5f55f5b50efd364b76b2a8266b15eb7c27e1e47f7646257bff34dfb4b7d08527`. The newest
entry it holds that this digest no longer carries is
`2026-09-17 — The plan checker had no commit-time trigger (`SIGNOFF-REPAIR.11.14.1.2`)`.
It carries the TWENTY-FIRST rotation's notice in turn, which names the ledger before it.

Retrieve the ledger immediately before the TWENTY-FIRST rotation (2026-09-17)
from the repository root:

```bash
git show f3022429193e25c47fb256b33d650ecefc66dd1b:CHANGELOG.md
```

That snapshot is 93,514 bytes and contains 31 dated entries; its Git blob is
`b765b420c475f55267e697172c6047556a7cbb93`, and its SHA-256 is
`c71a603db91de4957ac57286676115de73f464293438db8166e58ed39b693e66`. The newest
entry it holds that this digest no longer carries is
`2026-09-17 — The standalone assessment route is bound to the citing tenant (`SIGNOFF-REPAIR.11.14.3.8`)`.
It carries the TWENTIETH rotation's notice in turn, which names the ledger before it.

Retrieve the ledger immediately before the TWENTIETH rotation (2026-09-17)
from the repository root:

```bash
git show 1102b602272e10ed92cc2fb3da9b8d2730143c20:CHANGELOG.md
```

That snapshot is 91,931 bytes and contains 37 dated entries; its Git blob is
`2e598750319b67e22da91be9172e37fbdd1d29e0`, and its SHA-256 is
`b28dc01fe5f4cbaf67bc766f7ff9696e630025eb6c75980b52d590b4c8eff7c8`. The newest
entry it holds that this digest no longer carries is
`2026-09-16 — A credential stops at the origin the caller named (`SIGNOFF-REPAIR.7.2.6`)`.
It carries the NINETEENTH rotation's notice in turn, which names the ledger before it.

Retrieve the ledger immediately before the NINETEENTH rotation (2026-09-16)
from the repository root:

```bash
git show a581ab5eac5f5f4ad691aee8d7c2ddc6563ce136:CHANGELOG.md
```

That snapshot is 95,031 bytes and contains 27 dated entries; its Git blob is
`ddf72154df4cd968ab799e48355f69fc43c331e4`, and its SHA-256 is
`5b9fe69ecaa8b29f8896f92725039d83f1c6f339f79d02dbaae12dca22fab128`. The newest
entry it holds that this digest no longer carries is
`2026-09-15 — Census the five gate records, and find a wrong number inside a release gate (`SIGNOFF-REPAIR.11.4.7`)`.
It carries the EIGHTEENTH rotation's notice in turn, which names the ledger before it.

Retrieve the ledger immediately before the EIGHTEENTH rotation (2026-09-15)
from the repository root:

```bash
git show b6445d1818e08f7fbcd7a4ca05525a3add1dad80:CHANGELOG.md
```

That snapshot is 94,399 bytes and contains 28 dated entries; its Git blob is
`a17817dcd2e6f79efe176b4593eae45195cda305`, and its SHA-256 is
`fc0818d18865d8e5fb7174b541d50578f3cb5e96594581a108d66433cf6a25ca`. The newest
entry it holds that this digest no longer carries is
`2026-09-14 — Correcting a number this activity published (`SIGNOFF-REPAIR.11.9.1.3.1`, tranche 4a)`.
It carries the SEVENTEENTH rotation's notice in turn, which names the ledger before it.

Retrieve the ledger immediately before the SEVENTEENTH rotation (2026-09-15)
from the repository root:

```bash
git show 143d7a3c9167b5df39faa0473d8e1ba082c6f8ee:CHANGELOG.md
```

That snapshot is 93,783 bytes and contains 29 dated entries; its Git blob is
`1f82dd9504e85ed96c6a027d3d21cb5c8bbad0f7`, and its SHA-256 is
`804dadbb42ec9d1031c946455d76a754f26b5ceecf58cb4c4ee20766ab88a4df`. The newest
entry it holds that this digest no longer carries is
`2026-09-14 — Ask the renderer, not the specification (`SIGNOFF-REPAIR.11.9.1.1.3`, tranche 2 complete)`.
It carries the SIXTEENTH rotation's notice in turn, which names the ledger before it.

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
