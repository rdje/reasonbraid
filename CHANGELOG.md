# CHANGELOG.md

## 2026-09-22 — An adjudicator's verdict can no longer claim a vote count or pick its own rule (`SIGNOFF-REPAIR.8.1.1.3`)

`REASONBRAID-REPAIR-0413`.

- 🔴 **Before:** one person's verdict could say "accepted unanimously" and name any decision rule it liked, even on a thread governed by a different rule. A test did exactly that.
- ✅ **Now** a verdict states only what it judged and its conclusion. The rule it applies is taken from the thread, or recorded as none. The three outcomes that describe a vote count (unanimous, accepted with objections, no quorum) are refused on a verdict, because one person cannot count votes. A verdict must also say what it judged.
- ✅ The 16 test cases that sent verdicts were each reworked for what they were testing, not simply patched; one became the refusal test. Switching the new check off makes that test fail.

## 2026-09-22 — A thread can say what it is supposed to produce (`SIGNOFF-REPAIR.11.4.7.2.1.2.2`)

`REASONBRAID-REPAIR-0412`.

- 🔴 The roadmap's first demonstration has a person create a thread with an objective **and an expected artifact**, but the system had no field for the artifact. Sending one was rejected.
- ✅ `rb thread create --expected-artifact "…"` (optional) records it. `rb inspect thread` shows it directly under the thread's state and stop reason, and the web console shows it too. An empty or unreasonable value is refused, and nothing is created.
- ✅ Tested through the full create → discuss → close → inspect walk. The same test fails against the old code.

## 2026-09-22 — The operator-view checklist is now checked automatically (`SIGNOFF-REPAIR.4.6.1.6`)

`REASONBRAID-REPAIR-0411`.

- ✅ A new check maps each of the roadmap's nine operator views (§18.5) to the server routes that provide it, and verifies on every commit that those routes still exist. Renaming or removing one now blocks the commit instead of silently leaving the view missing. This was proven by renaming a real route.
- 📊 Result: **8 of 9 shown**. The ninth (how old the last audit checkpoint is) waits for the audit hash chain, which the roadmap defers until the system is deployed beyond a single machine. The check names that reason every time it runs.

## 2026-09-22 — The server reports whether backups exist and have been test-restored (`SIGNOFF-REPAIR.4.6.1.5.2`)

`REASONBRAID-REPAIR-0410`.

- 🔴 **Before:** the backup and restore scripts worked but left no record, so the server could not say whether a usable backup existed. The backup script also **printed the database address including its password**, and two backups in the same second could overwrite each other.
- ✅ **Now** each script leaves a small receipt next to the backup, and only when its step succeeded. The restore script first checks the backup file is intact, and afterwards checks that the restored copy really contains the database structure.
- ✅ `GET /v1/admin/backups` lists every backup, whether its file is still intact, and whether it has passed a restore test. Following the roadmap, it says a backup is an acceptable recovery control **only once it has been test-restored**.
- ✅ The password is no longer printed or recorded, and an existing backup is never overwritten.
- ✅ The test really runs both scripts: a fresh backup is listed but "not accepted", becomes "accepted" after a restore test, and becomes "not accepted" again when the backup file is damaged (the restore test also refuses it). Removing the "must have been restored" rule makes the test fail.
- 📊 The roadmap's operator-view list (§18.5) is now **8 of 9 shown**. The last item waits on the audit hash chain.

## 2026-09-22 — An operator can list the incidents still in progress (`SIGNOFF-REPAIR.4.6.1.5.1`)

`REASONBRAID-REPAIR-0409`.

- ✅ `GET /v1/admin/incidents` (and `rb inspect incidents`) lists the tenant's incidents that are not yet resolved, oldest first. An incident is an "incident review" discussion thread, opened and closed with the normal thread commands, so nothing new is stored.
- ✅ The test opens an incident, an ordinary thread, and another tenant's incident, then resolves the first: only the right one is listed, and it disappears once resolved. Two deliberate sabotages of the query (listing resolved incidents; listing non-incident threads) were each caught.

## 2026-09-22 — Decided how backups and incidents will be shown (`SIGNOFF-REPAIR.4.6.1.5`)

`REASONBRAID-DOC-0132`. A decision; no code changed.

- ✅ **Incidents:** the system already has an "incident review" type of discussion thread for operational and security events. An active incident is simply one of those threads that is still open, so nothing new has to be invented, only a list.
- ✅ **Backups:** the backup and restore scripts will write a small receipt file next to each backup, and only after their step succeeds. The restore script will first check that the backup file is intact, then check the restored copy. The server reads these receipts and re-checks the files. Per the roadmap, a backup that has never been test-restored does not count as a recovery control.
- Split into two steps: the incident list first, then backups.

## 2026-09-22 — The server reports the health of what it depends on (`SIGNOFF-REPAIR.4.6.1.4`)

`REASONBRAID-REPAIR-0408`.

- 🔴 **Before:** nothing checked the server's dependencies after start-up. If the database went away, the only sign was the next request failing.
- ✅ **Now** the server checks its database, secret store, certificate authority and (if configured) publication folder every 10 seconds. `GET /v1/health` reports each one as up, down, stale (not checked recently) or not yet checked, with when it was last seen working. It answers 200 when everything is fine and 503 otherwise.
- ✅ It needs no login, on purpose: a health check that needed the database could never report that the database is down. It therefore reveals only names, states and times. Error details go to the server log.
- ⭐ Running the real server with its database stopped showed that one slow check held up the others. All checks now run at the same time, and a test proves it (4.0 s before, 2.0 s after).
- ✅ Every "down" in the tests comes from really stopping something: a database dropped, a folder removed, an expired certificate. The book and the database-failure runbook are updated.

## 2026-09-22 — A test suite broken by an earlier change today is fixed (`SIGNOFF-REPAIR.8.2.5.4`)

`REPAIR-0407`. Test-only; no product behaviour changed.

- 🔴 Earlier today, REPAIR-0392 made recording evaluation data an operator-level action that requires a stated reason. One test suite (`routing`) still recorded its setup data the old way, so 2 of its 4 tests had failed ever since. It was found while checking the previous change against neighbouring suites.
- ✅ The suite now sets up its data the new way. None of its checks were changed, and all 4 pass. A search found no product code (CLI, benchmark, demo) still using the old way.
- ⚠️ Lesson: a change to how a route works must be tested against every suite that calls that route, not only the one being edited.

## 2026-09-22 — An operator can now see every refused evidence lookup (`SIGNOFF-REPAIR.4.6.1.2`)

`REASONBRAID-REPAIR-0406`.

- 🔴 **Before:** when the system refused to fetch evidence for a reference (the destination was blocked, no fetcher could meet the safety requirements, or a usage limit was hit), it told the caller why, then forgot. Only the usage-limit refusal was counted, and that count said neither which reference nor who asked.
- ✅ **Now every refusal is recorded** with the exact words the caller was given. A tenant administrator can list them, newest first, at `GET /v1/admin/resolution-refusals` or with `rb inspect refusals`.
- ✅ Two answers are deliberately left out. "Not found" is excluded because it must not reveal that another tenant's reference exists. "Invalid request" is excluded because it is the caller's own input error, not a refusal.
- ✅ The test causes all three kinds of refusal for real, without touching the network. With recording switched off it fails (0 rows where 3 are expected); with it restored everything passes (profiles 67/67, plus core, CLI and three neighbouring suites).

## 2026-09-22 — An operator can now see uncertain provider attempts without reaching the node (`SIGNOFF-REPAIR.4.6.1.1`)

`REASONBRAID-REPAIR-0405`.

- 🔴 **Before:** when a node reconnected after a crash, it reported attempts whose outcome it could not know ("the call may have happened"). The server told the node what to do and then kept nothing, so an operator had no way to list them.
- ✅ **Now the server records each report.** A new admin read, `GET /v1/admin/nodes/ambiguous-attempts`, and the CLI command `rb inspect ambiguous` list the ones still open. Each item shows when it was first and last reported, and the list comes with the three safe ways to resolve an attempt, who takes each one, and the one thing never to do (re-send the call "to check", which can charge twice).
- ✅ An entry closes on its own when the server receives the result, or when the node settles it locally. Closed entries are kept, together with how they ended.
- ✅ Only the tenant's own administrator can read the list. Another tenant's administrator is refused, and the refusal is recorded.
- ✅ Tested on a real node and connection: with the recording switched off the new test fails, and with it restored everything passes (42/42 plus core, CLI and a second suite). The book's node-channel, authority, CLI and deployment chapters and the runbook are updated.

## 2026-09-22 — Five missing operator views each need something to record their facts first (`SIGNOFF-REPAIR.4.6.1`)

`REASONBRAID-DOC-0131`. A planning correction; no code changed.

- 🔴 **The earlier plan said three of the five missing admin views only needed a new read route. That was wrong.** A route can only show what has been saved, and none of the three facts is saved. The server decides what a reconnecting node should do about an uncertain attempt and then forgets it. It refuses an acquisition and forgets the refusal. The audit checkpoint does not exist yet.
- ✅ The work is split into six owned steps, one per view plus a checker: uncertain attempts first, then resolver refusals, health, backup/incidents, and finally a script that re-checks all nine §18.5 items so this cannot drift again.
- ⏸️ The audit-checkpoint age waits for the audit hash chain (ADR-022). The chain is built at the first non-loopback deployment or the G7 gate, whichever comes first.
- ⚠️ A design problem to solve in the health step: the admin routes check permissions in the database, so a health route gated that way could never report that the database is down.
- 📖 The deployment chapter now lists exactly what an operator cannot see yet, and which step owns each item.

## 2026-09-22 — The book has a chapter on where published policy lives (`SIGNOFF-REPAIR.9.3.5.3`)

`REASONBRAID-DOC-0130`. Documentation only.

- ✅ **New chapter: *The publication store*.** It explains what a publish writes into Git, the three pointers it moves and when each is refused, the order of the steps, every refusal message, and how an interrupted publish is recovered. The book previously never mentioned these Git pointers at all.
- ⭐ The examples are copied from a real run, not typed by hand. Doing that exposed the two defects fixed in the previous commit.
- ✅ The two sections on this topic that I added to the CLI chapter earlier today moved into the new chapter, so the topic has one home.

## 2026-09-22 — A refused re-publish no longer leaves a stray pointer behind (`SIGNOFF-REPAIR.9.3.5.3.1`)

`REASONBRAID-REPAIR-0404`. Two small defects, found while generating the book's examples from a real run instead of writing them by hand.

- 🔴 **Publishing different content under an existing publication id was correctly refused — but it still moved the "staging" pointer** to the content that was refused. The check now runs before anything moves.
- 🔴 **The message I wrote last commit for a failed version check read badly** ("found `expected …, found …`"). It now states what was expected and what was found once each.
- ✅ Publisher 12/12, policy 28/28; both fixes shown to be caught when undone.

## 2026-09-22 — A published policy can be read back, and is checked before it is sent (`SIGNOFF-REPAIR.9.3.5.2`)

`REASONBRAID-REPAIR-0403`.

- 🔴 **Published policy content could be written but never read back** through the product. Its fingerprint (digest) was checked only at the moment of writing.
- ✅ **New read: `GET /v1/policy-bundles/{manifest_digest}`.** It returns exactly what was published, and first checks that the stored content still matches its recorded fingerprints — both the manifest and the policy text. If anything was altered, the answer is a refusal naming the altered file, never the altered content.
- ✅ Only the publication's own tenant can read it; to anyone else it looks like it does not exist.
- ✅ The test really alters the stored Git data to prove the refusal. When I deliberately disabled the checks, the altered content was served and another tenant could read it — both caught.
- ✅ Policy 28/28; lint, format, book and the surface ledger clean. Documented in the policy chapter.

## 2026-09-22 — Interrupted publications are recovered by `rb-reconciler` (`SIGNOFF-REPAIR.9.3.5.1.2`)

`REASONBRAID-REPAIR-0402`. The recovery logic the roadmap describes (§15.8) finally runs.

- ✅ **New tool: `rb-reconciler`.** It checks every publication against its Git repository. If a publish was interrupted — before writing, or after writing but before the database heard — it finishes the job. Anything that needs judgment (a conflicting version, a failed publication whose content later appeared, a published version that went missing) is reported and left alone, never decided by the machine.
- ✅ Running it twice changes nothing the second time. That was measured, not assumed: re-publishing the same content produces the identical Git commit and is a no-op.
- 🔴 **It found an older bug on its first run**: when publishing expected a "current version" that did not exist, the error said "the write failed" instead of "the version you expected is not there", because the code recognised that failure by the wording of one specific error message. It now looks at the actual state instead.
- ⭐ When I deliberately broke the tool's safety rules, the layers underneath still refused — the database will not promote a failed publication and Git will not move a published version — so the protection is layered.
- ✅ Publisher 10/10, policy 28/28; four broken versions each caught; lint, format, book and both binary ledgers clean. Documented in the CLI chapter.

## 2026-09-22 — A publish records where it is writing before it writes (`SIGNOFF-REPAIR.9.3.5.1.1`)

`REASONBRAID-REPAIR-0401`. The first half of making interrupted publications recoverable.

- ✅ **Before touching Git, a publish now writes down which repository it is using** (relative to the publication folder, so the record survives the folder moving) and what it expects the current published version to be. If the server dies halfway, the record says where to look.
- ✅ **Publishing the same thing twice now produces the identical Git commit**, because the commit is stamped with the publication's own staging time instead of the current clock. Recovery can therefore recompute what it should find.
- 🔴 **Caught before it shipped**: recording the plan before checking the repository even exists would have locked a publication onto a wrong location forever. It now checks first.
- ⭐ One of the deliberately broken versions hinted that a retried publish now completes cleanly on its own — promising for recovery, but noted as something to measure, not assume.
- ✅ Publisher 8/8, policy 27/27, migration upgrade 8/8; four broken versions each caught; lint, format and book clean.

## 2026-09-22 — The publication reconciler cannot be switched on yet, and why (`SIGNOFF-REPAIR.9.3.5.1`)

`REASONBRAID-DOC-0129`. A re-scope before building; no code changed.

- 🔴 **The recovery logic for interrupted publications exists, but nothing could use it even if it were connected.** A publication does not record which Git repository it was written to, so after a crash there is nowhere to look.
- 🔴 **Publishing the same content twice produces two different Git commits**, because each is stamped with the current time. The recovery plan assumes a retry produces the same commit, and compares against an expected one — neither is possible today.
- ✅ **The roadmap already says the fix** (§15.7 step 4): record the intended Git operation in the database *before* writing to Git. Split into two steps: record the operation and make commits reproducible first, then connect the recovery logic, with crash tests.

## 2026-09-22 — A moderator is a seat that may only moderate (`SIGNOFF-REPAIR.11.4.7.2.1.3.1`)

`REASONBRAID-REPAIR-0400`. The moderator role the roadmap (§13.5) describes, now that the vote and the quorum it must stay away from exist.

- ✅ **A thread can seat an agent as moderator** when inviting it. A moderator may classify messages, ask for clarification, propose closing a round, point out unanswered claims and draft summaries — and nothing else. Voting, adding evidence, inviting people, advancing the round, and closing are all refused, and each refusal names the rule it enforces. This holds even if the agent has permission for those actions elsewhere.
- ✅ **A moderator is never counted as a voter.** If it were, its forbidden ballot would stay missing forever and every unanimous vote would fail.
- 🔴 **Two conclusions from an earlier review of mine were wrong, and building on them showed it.** It missed that the five moderation actions already existed, and its proof that nobody can delete dissent searched a database table that does not exist — so it would have "passed" no matter what. The real check (nothing ever edits or deletes the event log) does hold.
- 🔴 **Testing the tests found a leak**: when one test failed halfway, it left a custom workflow behind and made an unrelated test fail. Each test in that suite now starts from a clean workflow list.
- ✅ New book chapter: *Moderating a thread* — the first documentation of the moderation actions at all. Profiles 66/66; lint, format and book clean.

## 2026-09-22 — An approval's quorum is copied from its decision, and a control that could not see storage was fixed (`SIGNOFF-REPAIR.11.4.7.2.1.2.3.2`)

`REASONBRAID-REPAIR-0399`. The second half of the policy-approval repair; `.11.4.7.2.1.2.3` closes.

- 🔴 **Before this commit, an approver typed the quorum their own approval rested on**, and it was stored as given.
- ✅ **Now the quorum is copied from the decision being approved** — which, since the last commit, is itself read from the thread's count. An approver who names a different or larger quorum is refused. A decision recorded before decisions were derived cannot be approved at all, because there is nothing honest to copy.
- ✅ The whole policy chain now runs charter → thread → decision → approval, and no step takes the caller's word for the result.
- 🔴 **One of my own tests could not see what it claimed to check.** It compared the approval's *response*, which the server builds from the correct value regardless of what it stored — so a deliberately broken version that stored the wrong quorum still passed. The test now reads the stored record back, and the broken version fails.
- ⚠️ Recusal does not exist yet, so "a recused member does not count" is true only because nobody can be recused. Stated, not claimed.
- ✅ Policy 27/27; lint, format and the book clean. New book section: *Approvals*.

## 2026-09-22 — A policy decision is the record of its thread's counted close (`SIGNOFF-REPAIR.11.4.7.2.1.2.3.1`)

`REASONBRAID-REPAIR-0398`. The first half of the policy-approval repair, re-scoped by measurement.

- 🔴 **The leaf would have compared two claims.** It asked to check an approval's quorum against *the recorded electorate* — but the decision stored that electorate, and its rule, exactly as the caller typed them. Nothing read the thread's result.
- 🔴 **The book documented a request that could never work**: its example electorate used `members`, and the code only counts `participants`.
- ✅ **A policy decision now reads the proposal's thread**: the thread must have declared a rule, be closed, and have an outcome the server worked out that accepts the proposal. The decision stores that thread's rule, voters, count and charter — not what the request says. If the request states a rule or voters that differ, it is refused.
- ✅ Twelve policy tests had been deciding on threads that were still open and had no rule; each now goes through a thread that its owner closes under `owner_decides`. Nothing else in them changed.
- ⭐ One of the three mutation tests reproduced the original defect exactly: with the rule check removed, a thread whose closer simply *typed* "accepted unanimously" became a binding policy decision. With the check in place it is refused.
- ✅ Policy 27/27, profiles 65/65, charters 8/8, migration upgrade 8/8; lint, format and book clean. Next: the approval copies this derived record.

## 2026-09-22 — A thread declares its rule, the vote is counted, and a close that says otherwise is refused (`SIGNOFF-REPAIR.8.1.1.2`)

`REASONBRAID-REPAIR-0397`. The second half of item 6, and `.8.1.1` closes with it.

- 🔴 **Before this commit, the person closing a thread simply stated the result** — `accepted_unanimously`, `no_quorum` and the rest — and the server stored it. Nothing counted anything.
- ✅ **Now a thread can declare its `decision_rule` when it is created.** The rule is checked against the tenant's charter, and the thread records which charter allowed it. A rule nothing can evaluate yet (`role_weighted`, `human_committee`), a counted rule on a workflow with no `vote` step, a rule the charter does not allow, and a charter nobody registered are all refused.
- ✅ **Members vote with a `ballot` contribution on the `vote` step** — approve, reject or abstain, once each. Who may vote is fixed the moment voting opens.
- ✅ **The close works out the outcome.** Leave `outcome` out and the server fills in the counted result. State one that disagrees and the close is refused, naming both words. The close record now says where its outcome came from (`derived` or `caller_asserted`) and carries the count.
- ⭐ **The charter is read only after the caller is authorized.** A test proves why that matters: moving the read earlier makes an outsider receive an answer about another tenant's charter, and the test goes RED.
- ⚠️ **A thread with no rule still closes on the closer's word** — deliberately, because every existing deployment's charter is a placeholder and a required rule would block all thread creation. The record now says `caller_asserted`, and `.8.1.1.5` owns ending it.
- ✅ Unit 172/172; live `profiles` 65/65 (the twelve-outcome test untouched) and `charters` 8/8; three mutations each RED on their target; strict lint, format and the book clean. New book chapter: *Deciding a thread: rules, ballots and the counted close*.

## 2026-09-22 — The count exists before anything calls it, and a mutation that survived caught a false number of mine (`SIGNOFF-REPAIR.8.1.1.1`)

`REASONBRAID-REPAIR-0396`. The first half of item 6: the ballot vocabulary, the count and the close classification, as pure functions in `crates/reasonbraid-server/src/decisions.rs`. Nothing calls them yet, so nothing can break.

- 🔴 **The leaf under-quoted its own contract.** ROADMAP §13.3 says *every rule defines* thirteen things — electorate, quorum, denominator, abstentions, recusals, timeouts, unreachable members, role replacement, changed incarnations, veto scope, amendments, ties and terminal outcomes. The new record `docs/decisions/2026-09-22_a-counted-rule-derives-its-outcome-and-a-rule-it-cannot-count-is-refused.md` answers each one or names the child that owns it.
- ✅ **What the count decides.** `majority_of_electorate`, `unanimity` and `consensus` are counted from ballots. `owner_decides` is decided by who closes, and `advisory_synthesis` can only end `advisory_answer_only`. `role_weighted` and `human_committee` have no bar anything can evaluate, so they will be refused at creation (`.8.1.1.4`).
- ⭐ **Abstention is decided by the roadmap's own sentence**: under unanimity it withholds, so the result is `deadlocked` (§13.4, *failure/deadlock if any applicable member withholds it*).
- 🔴 **A falsification mutation survived, and it exposed a false witness I had written.** I justified a rounding guard with `0.51 × 100 = 51.00000000000001`; the product is exactly `51.0`, so removing the guard changed nothing. Measured instead: 13 of 10,000 threshold/electorate pairs really do land one step above a whole number (`0.56 × 25 = 14.000000000000002`). The guard was right; the reason was not. With the real witnesses, all three mutations go RED by name and the source is restored byte-identical.
- ✅ 20/20 unit tests; strict lint and format clean.

## 2026-09-22 — Sixteen claims from this session re-derived: fourteen hold, and the two that moved are both about this project's own audit (`SIGNOFF-REPAIR.13.4.7`)

`REASONBRAID-DOC-0128`. The director's *ensure your findings still hold*, over `REPAIR-0394` … `DOC-0127`. Every claim re-derived by a route structurally different from the one that produced it.

- ✅ **FOURTEEN HELD, two of them STRONGER than published.** The publication half was re-derived not by its add-date but by EXISTENCE AT THE AUDIT'S OWN COMMIT — `git cat-file -e 1a87d31:…/publisher.rs` succeeds, and `git show 1a87d31:…/publications.rs | grep -c "publisher::"` returns **3**: the module the audit concluded was absent appears three times in the very file it grepped. The roll-up disagreement was re-derived by literal verdict-cell counts rather than a line-window regex: **12 · 2 · 3 · 2 · 2 = 21** against **3** sentences asserting *13 discharged*.
- ✅ Also held by fresh routes: the reconciler has no production caller (`GitState` lives in exactly two files, its module and its test — the `reconcile(` route is useless, 24 hits mostly from the node's unrelated journal verbs); no handler returns a publication bundle (the only `projection.bytes` is inside the write path); §18.5's two absences re-derived from the router's own 55-path list; 16 quorum sites with exactly 1 comparison; §13.5's six acts and six prohibitions; §26's two items; 13 runbooks; 13 + 8 charter controls.
- 🔎 **A TOOLING TRAP WORTH RECORDING: `git grep -E` does NOT honour `\s`, while the system `grep -E` does.** The quorum re-derivation first returned **0 predicates** and looked like a refutation; `[[:space:]]` returns 1. A control proved the shell's own `grep -E` accepts `\s`, which localized it to git's regex engine rather than to the claim.
- ⚠️ **One absence survived a challenge rather than being re-asserted.** The new route for *enforce format/length* found 3 hits — all `fetcher.rs`'s URL-length bound, a transport limit on evidence acquisition, not a moderator act. `.11.4.7.2.1.3`'s record had already named that ambiguity as *what would make this wrong*; it now has a measured instance behind it.
- 🔴 **MOVED 1, and it is the worse of the two, because it is a claim about an audit's reliability made by miscounting which audit.** `.11.4.7.2.1.5` opened saying FOUR of `.11.4.7.2.1.4`'s twenty-one verdicts had been re-derived with two wrong. ⛔ **Only TWO belong to that pass** — `.9.3.5` and `.4.6` came from its tranches; `.11.4.7.2.1.2` and `.11.4.7.2.1.3` were opened by **`.11.4.7.2.1`**, a separate audit over SIX deferrals. Each leaf's own `- Opened:` line says so and I did not read them. ⭐ **The correction makes the finding STRONGER: this pass's re-derived record is 2 of 2 WRONG, not 2 of 4**, and the softer *twice it shrank, once it grew, once it held* reading was an artefact of averaging two audits together.
- 🔴 **MOVED 2 — an off-by-one in a quoted requirement, published five times.** I wrote that the SLO record carries *§18.4's exact **nine** fields*; §18.4 names **EIGHT**, and the 2026-09-07 table has nine content columns because it adds an objective column of its own. ⛔ The verdict is unaffected — the record carries every field §18.4 names — but a number attributed to the roadmap must be the roadmap's.
- ⭐ **BOTH MISSES ARE THE SAME SHAPE, AND IT IS THE SHAPE THIS SESSION SPENT SIX COMMITS ON**: a figure read off a document at a glance instead of counted from it. The audit inferred an absence from one grep, `.4.6`'s row graded four nouns at once, and I attributed nine fields to a sentence naming eight and four re-derivations to a pass that owns two. ⛔ Fourteen of sixteen held, so the work is sound; **both that moved were COUNTS, and neither was re-derived before publication.**
- ✅ Corrections applied at every site: the `.11.4.7.2.1.5` leaf, the `.4.6` leaf and its frontier row, the observability decision record and its index row, `MEMORY.md`, `docs/TASK_TREE.md` and the three ledgers. No code changed. Doctrine gate **26/26 green**.

## 2026-09-22 — Two of the adjudication pass's verdicts have been re-derived, both were wrong, and twelve `discharged` ones never have (`SIGNOFF-REPAIR.11.4.7.2.1.5`)

`REASONBRAID-DOC-0127`. A task-tree ownership slice: the finding this batch produced about its OWN audit, given an executable owner rather than a note.

- 🔴 **TWO OF THE TWENTY-ONE VERDICTS HAVE NOW BEEN INDEPENDENTLY RE-DERIVED AND BOTH WERE WRONG.** ⚠️ *(This entry first said FOUR with two wrong; `.13.4.7` corrected it one commit later — the other two re-derivations belong to `.11.4.7.2.1`'s separate six-deferral audit, and both of those held.)* Tranche 1's **row 4** was graded `fired and open` and both its halves had shipped **fourteen days before the audit** (`.9.3.5`); tranche 2's **row 14** was graded as one open thing and is **1 open · 1 partial · 2 discharged** (`.4.6`); two held (`.11.4.7.2.1.2`, `.11.4.7.2.1.3`).
- ⭐ **Across BOTH audits the record is twice-shrank, once-grew, once-held — so the lesson is *re-derive in EITHER direction*.** ⛔ But that sample spans two passes: **this one's own re-derived record is 2 of 2 wrong.**
- ⛔ **BUT BOTH WERE ROWS SOMETHING LATER OPENED A LEAF AGAINST.** A row graded `discharged` opens no leaf, is read by nobody again, and its failure mode is the one that matters: **a gap called closed stays closed.** ✅ Of the twenty-one rows, **thirteen** carry `discharged` and **twelve have never been re-derived by anyone** — only row 4 has, and it moved.
- ⚠️ **Not a claim that any `discharged` verdict IS wrong** — a claim about EXPOSURE. Both known errors came from the same method (a verdict taken from one grep, or over several nouns at once), and that method was applied to all twenty-one rows equally.
- ⭐ **The pass has now been wrong about itself four times and right about itself twice** — it caught its own population double-counting, and its roll-up asserted 13 discharged / 1 open where its own tables counted 12 / 2. That is the argument for finishing the job, not for distrusting it.
- ⛔ **Sequenced AFTER `.8.1.1`**, on `.15`'s own first ordering rule: this is an audit of records, while `.8.1.1` is a live governance surface publishing a claim it never derived, and a live exposure outranks everything.
- ✅ Task-tree only; no code, no book, no behaviour. Doctrine gate **26/26 green**.

## 2026-09-22 — The charter has a home for the decision rules §4.1 says it defines, and it is content-addressed (`SIGNOFF-REPAIR.11.4.7.2.1.2.1`)

`REASONBRAID-REPAIR-0395`. The first BUILD after `.15`'s five wave-B decisions, and the prerequisite `.11.4.7.2.1.2` found.

- 🔴 **BEFORE THIS COMMIT THE CHARTER HAD A NAME AND NO CONTENT.** ROADMAP §4.1 says each tenant has a *versioned `GovernanceCharter`* defining *allowed decision rules and approval thresholds*; `enrollment_boundaries.charter_digest` was a `TEXT` label nothing resolved — the shipped values are `dev-charter-digest`, `dev-charter-000`, `fixture-charter` — and §13.3's seven rule families existed only as prose.
- ✅ **CONTENT-ADDRESSED, AND THAT IS WHAT MAKES IT VERSIONED.** `migrations/0084_governance_charters.sql` keys a charter by the digest of its own canonical content, so changing the allowed set writes a DIFFERENT row and the old one stays readable forever. ⭐ **That discharges the acceptance's hardest clause with no bookkeeping to get wrong**: `reasonbraid_core::authority` ALREADY folds `charter_digest` into every authorization decision record, so a decision taken on Monday names Monday's charter by digest and that row is immutable by construction. A `version` column with `valid_from`/`superseded_at` would have made the same guarantee depend on range queries nobody re-derives.
- ⛔ **`tenant_id` is INSIDE the digested content** — two tenants allowing the same rules hold two rows, because a charter is a tenant's governance document and not a shared template, so the `snapshot_objects` precedent (identical bytes = one object) deliberately does not apply. ⚠️ And re-registering identical content is IDEMPOTENT rather than a duplicate refusal, unlike `evaluation_corpora` whose coordinate is a caller-chosen first-come namespace: one key is chosen, the other is derived from the thing itself.
- ✅ **REGISTRATION IS A SITE ACT, BY DERIVATION RATHER THAN CAUTION.** §4.1 makes the charter the document that CONSTRAINS a tenant, and §4.4 makes the enrollment boundary naming its digest a root/parent-granted ceiling ⇒ a tenant that could rewrite its own allowed decision rules would hold the ceiling it is bound by. `Action::CharterRegister` takes the shape `2026-09-09_site-operator-authority.md` defines — authorization, effect and audit as one ordered transaction.
- ✅ **SEVEN WIRE NAMES, NOT NINE.** §13.3's second family is *simple **or** supermajority* and its fifth is *role-weighted **or** chambered*; splitting either would be this project choosing a vocabulary the roadmap did not state. ⭐ The threshold is what §4.1 already supplies for the first — which is why it names rules **and** thresholds in one breath. `majority_of_electorate` is the ONLY family that takes one and MUST have one, in `(0.5, 1.0]`; a threshold on `unanimity` is a contradiction because unanimity IS 1.0, and one on `advisory_synthesis` would gate a family §13.3 defines as having *no binding decision*.
- ⛔ **THE READ PATH RESOLVES THROUGH THE ACTIVE BOUNDARY AND FAILS CLOSED.** `GET /v1/decision-rules/{rule}` goes tenant → active boundary → the digest that boundary was ISSUED under, never by `tenant_id` directly, because §4.4 makes the boundary the ceiling. A boundary whose digest resolves to nothing answers *the question cannot be answered* — the LIVE case, since every boundary issued before `0084` carries a label. ⛔ And a refusal names the rule and NEVER enumerates the charter's set.
- ✅ **FALSIFIED THREE WAYS, EACH RED BY NAME, source restored BYTE-IDENTICAL (SHA-256 verified before and after).** (A) the fail-closed arm made fail-OPEN → `a_boundary_whose_charter_resolves_to_nothing_fails_closed` RED. (B) `NotAllowed` made to enumerate → RED in both the unit and the live suite. (C) `for_tenant` made to resolve by tenant id newest-first → `a_tenant_with_no_active_boundary_has_no_charter` RED.
- ⭐ **AND MUTATION C TOLD ME SOMETHING I HAD NOT ASSERTED**: `changing_the_set_leaves_the_earlier_charter_exactly_as_it_was` PASSED under it, so that control rests on content-addressing ALONE and not on the boundary path. Recorded because a control surviving a mutation aimed elsewhere is evidence about WHAT it tests.
- ✅ **THE SURFACES ARE JUDGED, NOT JUST ADDED.** `SURFACE-JUDGEMENT` refused the commit until both route families carried verdicts with their live witnesses; `docs/book/src/governance-charter.md` is the chapter and the book's `C3` row now states the remaining gap for the director.
- ⚠️ **§4.1's WAIVER provision is decided OUT of scope here and says so**, because §4.1 names it as a SEPARATE provision from *allowed decision rules* and this leaf implements one of nine families. ⛔ `.8.1.1` must not read the silence as *waivers are impossible*. ⚠️ `role_weighted`'s weighting has no schema; the family can be allowed and its weights have no home.
- ⚠️ **A behaviour, not an absence**: an existing deployment reads *not answerable* until a charter is registered and a boundary reissued naming it. Nothing breaks, because nothing consumed decision rules before this commit.
- ✅ `charters` unit **13/13**, live **8/8**; clippy `-D warnings` clean; `make book` renders; the doctrine gate **26/26 green**.

## 2026-09-22 — The moderator is a role boundary, not a feature, and five of its six acts already ship (`SIGNOFF-REPAIR.11.4.7.2.1.3`)

`REASONBRAID-DOC-0126`. Item 5 of the `.15` sequence — **WAVE B IS COMPLETE: all five decisions are taken.**

- 🔴 **A CORRECTION TO THE LEAF'S OWN ACCEPTANCE, made before using it.** It asks for *the three prohibitions*; §13.5 lists **SIX** — add a vote or approval; suppress a visible dissent except through an appealable moderation action; fabricate evidence or silently change citations; change electorate, quorum, or proposal digest; authorize more spend or tools; publish or deploy policy. All six are adjudicated.
- ⭐ **FIVE OF THE SIX PERMITTED ACTS ALREADY EXIST AS VERBS — AND NONE IS BOUNDED TO A ROLE.** *Classify messages* → `ContributionKind`, declared by the AUTHOR on their own contribution and never by anyone over another's. *Request clarification* → `Question` and `EvidenceRequest`, which targets one claim digest. *Propose round closure* → `thread.advance_round`, gated only by `ensure_participant`, so ANY participant may advance and nothing merely *proposes*. *Identify unanswered claims* → `CoverageItem { item, included, reason }`. *Draft summaries* → `Summary` + `SynthesisInput`. 🔴 Only *enforce format/length* has no verb at all.
- ⭐ **SO §13.5 IS A ROLE BOUNDARY, NOT A FEATURE LIST.** What is missing is not the ability to classify, ask, propose, summarize or report coverage — it is a principal RESTRICTED to those acts. Reading the deferral as *build a moderator feature* is what made it look large.
- ⭐ **FOUR OF THE SIX PROHIBITIONS ARE ALREADY CARRIED by mechanisms nobody wrote for §13.5.** *Suppress a visible dissent* is **structurally impossible** — `git grep -niE "DELETE FROM thread_contributions|UPDATE thread_contributions"` returns no match, so contributions are append-only and there is no suppression verb to restrict. *Fabricate evidence* — `snapshots::submit` refuses a `raw_digest` that is not the digest of its bytes. *Authorize more spend* — the grant's spend bound refuses an excess (`api.rs:4965`, `:4978`). *Publish or deploy policy* — grant-gated via `owning_authority`.
- 🔴 **THE TWO UNENFORCED PROHIBITIONS ARE THE SAME TWO DEFECTS THE SIBLING LEAF FOUND, REACHED FROM THE OPPOSITE DIRECTION.** `.11.4.7.2.1.2` found the declared-not-computed quorum by reading the approval path; this leaf found it by asking what would stop a moderator changing one. ⭐ **A defect reached twice by independent routes is the strongest evidence this pass produces** — the same signal that made `.9.3.5` a lane.
- ✅ **DECIDED: the moderator is an AUTHORITY SCOPE, not a deliberation feature** — the work is in the role and grant vocabulary, not the thread engine. ⛔ **BLOCKED on `.8.1.1` and `.11.4.7.2.1.2.1`/`.2.3`**: a moderator built today would be forbidden from touching a ballot there is none of and from changing a quorum any approver can already declare freely. **Enforcing a prohibition against nothing is not enforcement**, and the tests would pass vacuously.
- ✅ ***Enforce format/length* is DECLINED on its own merits** — the only CONTENT control among six procedural ones. §13.6 constrains form as a WORKFLOW-PROFILE property (*blind initial positions*, *randomized order*, *no display of vote totals before a configured commitment point*) rather than as a principal's discretion; a per-message ruling by a principal is the shape §13.6 is written against. If it returns, it returns as a profile property.
- ⛔ **THE BENCHMARK ARM IS NOT THE PRODUCT ROLE**, stated explicitly because conflating them is what made this half invisible for the life of the deferral. §19.5 measures a *moderator/synthesis flow* as one ROUTING SHAPE among four; §13.5's moderator is a GOVERNED PARTICIPANT in a real thread. This record does not supersede the benchmark and no later leaf may read its existence as coverage.
- ⭐ **THE LEAF DID DELETE PART OF ITSELF, as `.15` predicted the cheapest item might** — one act declined outright, five re-described as already shipped. Owner of what survives: `.11.4.7.2.1.3.1`, sequenced after its blockers.
- ✅ Docs-only; no Rust changed. All six acts and all six prohibitions measured at this commit. Doctrine gate **26/26 green**.

## 2026-09-22 — A decision rule is a charter-scoped vocabulary, and it never ships without the tally that reads it (`SIGNOFF-REPAIR.11.4.7.2.1.2`)

`REASONBRAID-DOC-0125`. Item 4 of the `.15` sequence, and it gates item 6. The first premise in three that GREW under measurement.

- ⭐ **THIS PREMISE HOLDS, AND MEASUREMENT MADE IT LARGER** — the opposite of the two leaves before it. `.9.3.5` and `.4.6` were both graded open over work already shipped; this row was graded open and is open. ⛔ The lesson is not *the adjudication pass is unreliable* but *a verdict must be re-derived, in either direction* — twice it shrank, once it grew.
- 🔴 **THE CONTRACT, QUOTED FROM FIVE PLACES with their sections, so no identifier is guessed.** **§3.2** names *decision rule, electorate policy, quorum, veto scope, and human role when applicable* as one of eight creation bullets. **§13.3** lists **seven** rule families, including *unanimity … with explicit abstention semantics*. **§13.2** steps 10–11: *snapshot electorate and collect vote/approval actions under the declared rule*, then ***compute the deterministic decision result***. **§4.1** makes the charter define *allowed decision rules and approval thresholds*. **§26** step 2: *objective, **expected artifact**, participant constraints, budget, and manual decision rule*.
- ⭐ **§26's SENTENCE SETTLES WHAT THE LEAF CARRIED OPEN.** The deferral recorded that `objective` stands in for the expected artifact; §26 lists **both in one enumeration, as two items**. The stand-in reading is refuted by the roadmap's own sentence rather than by preference.
- 🔴 **THE MEASUREMENT.** `threads::CreateBody` carries neither field. `git grep -n "expected_artifact\|decision_rule" -- crates migrations` returns **one** hit — `reasonbraid-a2a`'s `SemanticLosses.decision_rule`, a **bool** recording that a remote decision rule is ALWAYS LOST. `enrollment_boundaries.charter_digest` is a `TEXT` digest, not a structure, and no table holds an allowed-rule set.
- 🔴 **AND §13.2 STEP 11 IS UNIMPLEMENTED AT EVERY DECISION SURFACE THE PRODUCT HAS — wider than the deferral row said.** Thread close: `VerdictInput` is `{ target_digest, rule: String, outcome }`, the rule a free-form string beside an outcome the same caller declares, with `AcceptedByRule` assertable although no rule was ever declared. Policy approval: `ApprovalInput.quorum` is a `serde_json::Value` whose ONLY check is a non-empty `participants` array (`crates/reasonbraid-server/src/lifecycle.rs:503`–`510`) — the approver supplies the quorum that justifies their own approval, and nothing verifies the participants exist, are eligible, are non-recused, or that a threshold was met.
- ⭐ **That is `.8.2.4`'s and `.16`'s shape at the governance layer — a stored record asserting a provenance nothing derived** — and here the asserted word is the product's entire output.
- ✅ **DECIDED: a TYPED, CHARTER-SCOPED vocabulary, never a string.** §13.3's seven families, with §4.1's charter naming the allowed subset per tenant, refused at the create boundary on the rule `workflow_profile` already follows. ⛔ **It does NOT ship as a create field on its own** — a declared rule nothing evaluates is this same defect one step earlier, and the interval where the field is declared-but-unread is exactly when a caller learns to rely on it. It lands in the SAME commit as the tally (`.8.1.1`).
- ✅ **`expected_artifact` IS separable and is the one thing here that can ship alone**, because no section asks anything to EVALUATE it — it is read by the human who closes the thread.
- 🔴 **A PREREQUISITE NOBODY OWNED, invisible before this measurement**: the charter cannot hold the allowed rules §4.1 says it defines. Without it the create field validates against nothing and degrades into a global enum, which §4.1 does not say. Owner `.11.4.7.2.1.2.1`.
- ✅ **OWED BEFORE G9, not on a preference**: §19.6's release gate matrix gives G3 the line *authority/consent/**quorum**/publication/correction tests | binding policy use*, and G3 was exited as MACHINERY with binding use still gated. A quorum the caller declares is not a quorum test. ⛔ Unlike `.4.6`'s sink this is NOT deferrable behind a trigger — that one had no reader today, this one publishes a governance claim it never derived to whoever asks.
- ✅ Three children opened: `.2.1` the charter prerequisite, `.2.2` the expected artifact, `.2.3` the policy-approval quorum. Docs-only; no Rust changed. Every quoted section number re-derived from `ROADMAP.md` at this commit. Doctrine gate **26/26 green**.

## 2026-09-22 — The observability row is four commitments, two were already discharged, and the sink is deferred on a readable fact (`SIGNOFF-REPAIR.4.6`)

`REASONBRAID-DOC-0124`. Item 3 of the `.15` sequence. Second leaf running that shrank under measurement.

- ⛔ **ONE DEFERRAL ROW CARRIED FOUR NOUNS** — *OpenTelemetry, operator dashboards, SLO baselines, game days* — carried as a unit since the Phase-1 subtraction record. The leaf's own acceptance named why that is the defect: **a single verdict over four nouns is what hid this for a phase.** Each is adjudicated separately, with the command that produces it.
- ✅ **SLO BASELINES — DISCHARGED.** `docs/decisions/2026-09-07_phase2-slo-hypotheses.md` carries §18.4's **exact nine fields** (population, exclusions, window, statistic, target, error budget, owner, consequence) over SLO-1…SLO-5, with a zero error budget whose consequence is that a red pass halts the frontier. ⭐ Every unmeasured latency family is named with its trigger rather than given an invented number — §18.4's *initial targets are hypotheses* honoured rather than quoted.
- ✅ **GAME DAYS — DISCHARGED.** `docs/decisions/2026-09-08_game-days-pentest.md` maps **eight shipped exercises** to runbook closure tests, over the **13** runbooks in `docs/runbooks/` — one per §18.6 family. Its three named gaps already carry evaluable triggers.
- ⭐ **BOTH DISCHARGES PREDATE THE AUDIT THAT GRADED THE ROW OPEN** (2026-09-07 and 2026-09-08 against a 2026-09-22 tranche). ⛔ **Second consecutive leaf where a deferral row was graded open over work already shipped**, after `.9.3.5`: there the cause was a grep keyed on the wrong module, here a verdict taken over four nouns at once.
- ⏸️ **THE OPENTELEMETRY SINK — DEFERRED ON A READABLE FACT**, because `.11.4.7.2.1` measured 27 of 27 deferrals naming a closed phase, and this leaf exists because `telemetry.rs:3` restated its own trigger where nothing evaluates it. The condition: `rb-server` started with a `--host` that is not a loopback address — the deployment leaving §6.6's **Developer** profile (*Loopback/single host*) — or a second control-plane process. ⛔ Both readable on any day.
- ⚠️ **WHY, AND THE COST STATED RATHER THAN IMPLIED.** §18.1 class 3 is *distributed traces … across API, coordinator, node, adapter, resolver, and publisher*; at one process bound to `127.0.0.1` (the shipped default, `rb-server.rs:24`) there is no second process for a trace to cross, and the operator is the developer reading the JSON lines `telemetry.rs` already emits beside `GET /v1/admin/metrics`. **The redaction cost is not a makeweight**: §18.2 forbids prompt text, credentials, secret-bearing URLs, private evidence and model output in span attributes, so an egress path added before there is anything to correlate ships that risk early for nothing measurable.
- ⚠️ **OPERATOR DASHBOARDS — PARTIAL, KEPT AND SCOPED: 4 covered · 3 partial · 2 absent** over §18.5's nine bullets. 🔴 Absent: **service and dependency health with freshness** and **backup/restore status and active incidents** — `git grep -cniE "health|readyz|livez"` and `"backup|restore_status"` over `api.rs` both return **no match**, and the router has no health route at all. ⚠️ Partial: **ambiguous attempts** (the resolution ships as a `Directive` per attempt at the node handshake, `crates/reasonbraid-server/src/node_channel.rs:284`; nothing LISTS them), **resolver denials** (acquisitions covered, denials listed nowhere) and **checkpoint age** (verification ships as `/v1/audit/receipts`; `checkpoint` has no match in `api.rs`).
- ⭐ **THE TWO ABSENCES SHARE ONE SHAPE**: health and backup status are the only two bullets describing the SYSTEM rather than a domain aggregate. Every covered bullet had a domain surface to hang off; the two with no aggregate behind them are the two nobody built. ⚠️ The three partials share another — in each the MECHANISM ships and only the operator's view of it is missing. Owner `.4.6.1`, a wave-C build.
- ⛔ `telemetry.rs:3`'s prose trigger is SUPERSEDED by the decision record; the comment stays accurate but is no longer where the condition lives.
- ✅ Docs-only slice; no Rust changed. All four verdicts re-run at this commit; the doctrine gate **26/26 green**.

## 2026-09-22 — The publication store is built; the audit that called it absent grepped the wrong module name (`SIGNOFF-REPAIR.9.3.5`)

`REASONBRAID-REPAIR-0394`. Item 2 of the `.15` sequence. The leaf retracts its own premise and gets smaller.

- 🔴 **THE LEAF'S PREMISE WAS FALSE AND IS RETRACTED.** `.9.3.5` opened on 2026-09-22 as *a publication's immutable content is never written, so the manifest digests nothing outside PostgreSQL*, inheriting tranche 1's `fired and open` for the Phase-0 deferral *Git publication reconciliation + object store experiments*. Both halves were run and shipped.
- ⛔ **THE AUDIT MEASURED A MODULE NAME, NOT A BEHAVIOUR.** It ran `git grep -n "git::" -- crates/reasonbraid-server/src/publications.rs` → 0 and inferred *a publication is never reconciled against a ref*. The publication module is **`publisher::`**: `publications.rs:595` calls `publisher::missing_objects` and `api.rs:4061` calls `publisher::publish`. Its SECOND observation — that `git.rs` serves evidence acquisition — was correct; the inference from the first was not. **A grep keyed on one module path is not an absence proof**, the THIRD instance of that class after `.11.8.2`'s route census and `census_book_coverage.py`'s mention test.
- ✅ **RE-DERIVED THREE INDEPENDENT WAYS, one per `CLAIM_VERIFICATION.md` leg rather than one pass repeated.** Re-derive: `publisher.rs` implements §15.7 steps **5–8** — staging branch, fetch-back digest re-derivation, write-once immutable ref, compare-and-swap effective channel, on `gix` plumbing. Falsify: `git log --diff-filter=A` dates it `f53d73d`, **2026-09-08**, *fourteen days BEFORE* the audit (`1a87d31`) — wrong when made, not overtaken. Durability: `cargo test --test publisher` **7 passed, 0 failed**.
- ✅ **AND THE OBJECT-STORE HALF IS NOT ABSENT EITHER.** `migrations/0028` ships `snapshot_objects (digest TEXT PRIMARY KEY, bytes BYTEA NOT NULL)` — digest-keyed identity, cross-tenant dedup, the digest verified against the bytes on write; `migrations/0080` adds an `external-reference` class. The vendor-name grep (`aws-sdk|s3|minio`) was the wrong instrument for a storage CONTRACT.
- ✅ **BOTH NAMED STORES ARE KEPT, quoted from the roadmap rather than chosen by preference** — §6.3 authoritative state, §15.7 the publication protocol, §15.8 the matrix, §7.2 the crate map.
- ⏸️ **ONLY THE EXTERNAL BACKING IS DEFERRED, on two readable facts rather than a milestone**: `fetcher::FetchLimits::max_bytes` raised above **16 MiB** (today 4 MiB), or `SELECT sum(length(bytes)) FROM snapshot_objects` above **10 GiB**. Every object passes a 4 MiB fetch ceiling and TOAST holds that unaided; the alternative buys a vendor dependency, a second failure domain and a reachable *object-store outage* kill-point for nothing measurable. ⛔ The deferral is about the BACKEND, never about content-addressing.
- 🔴 **THE THREE REAL GAPS, each now an executable leaf.** `.9.3.5.1` — the §15.8 matrix is decided and **never operated**: `git grep "reconciler::"` outside its own file returns **one** hit, `tests/reconciler.rs:6`, so all six rows and 3 green units are reachable only from a test binary while §15.8 requires *regularly exercised under kill points*. `.9.3.5.2` — a published bundle is **readable by nothing**: four routes, no read of a bundle at all, so the digest binds at write (`api.rs:4053`, then the publisher's fetch-back) and at no other time. `.9.3.5.3` — the book documents **none** of it: `refs/rb/` → 0 hits in `docs/book/src`.
- 🔴 **THE CORRECTION EXPOSED A SECOND DEFECT NOBODY HAD RE-DERIVED.** The 21-row roll-up asserted **13 discharged / 1 fired-and-open** while its own tables counted **12 / 2** at `4b4d9e3`. Row 4's correction makes the published figures exact — **for a different reason than they gave**, which is why it is recorded rather than quietly restated. The one remaining `fired and open` is tranche 2's **row 14** (the OpenTelemetry sink, `.4.6`), never row 4.
- ⭐ **THE LEAF SHRANK UNDER MEASUREMENT** — *choose and build a publication store* → *three bounded gaps on a store that already exists*. That is `.15`'s wave-B ordering rule working: a duplicate publisher was one `git log` away from being built.
- ⚠️ **Not claimed:** that §15.7 is complete in every detail. Manifest SIGNING (step 3's *for required profiles, sign the manifest*) was not audited and is asserted neither way.
- ✅ Docs-only slice; no Rust changed. publisher **7/0**; the doctrine gate **26/26 green**.

## 2026-09-22 — A domain refusal is not an authority denial, and the 403 shipped one commit ago told authorized callers a falsehood (`SIGNOFF-REPAIR.16`)

`REASONBRAID-REPAIR-0393`. The director left the call to me; testing my own argument is what changed it.

- 🔴 **THE FLAW WAS A CONFLATION, NOT A MISSING FACT.** I defended the 403 with the existence-oracle argument: answering before the gate would let an unauthorized caller probe a registry. That argument is about **ORDER**, and says nothing about **RENDERING**. A caller who has passed the gate holds the grant and is entitled to the answer, so a 400 discloses nothing.
- 🔴 **AND WHAT SHIPPED CONTRADICTED ITSELF ON THE WIRE**: `403 {"code": "that corpus version is already registered", "message": "a current site grant for this action and its actual boundary are required"}`. The message is false of the caller who received it. An operator debugging CI reads 403 and checks their grants; the defect was a typo. It also counted input errors in `authorization_denials`, a security metric.
- ⭐ **FOURTH INSTANCE OF ONE CLASS IN A SINGLE SESSION, AND THIS ONE WAS MINE.** `Error::Refused` carried both meanings in one variant — as `EvaluationError::Duplicate` meant *taken* and *does not exist*, and as an `Err(_)` from an INSERT meant *duplicate* and *store fault*. One name, two meanings, and the boundary renders whichever it was handed.
- ⭐ **THE CORRECT PRINCIPLE WAS ALREADY WRITTEN DOWN — AT EXACTLY ONE CALL SITE.** `site_registry_response` hand-matched one reason string to render it as a bad request, commenting *a domain refusal, not an authority one: the caller held the grant*. That one-off is why the principle reached **one of sixteen** refusals. A rule implemented as a remembered special case holds exactly where it was remembered.
- ✅ **THE FIX IS AT THE TYPE.** `Error::Denied` (403, and the only case counted as an authorization denial) beside `Error::Refused` (400, with its reason as `code` and the `audit_id` beside it). Splitting the variant made the compiler find every arm, including one in the operator CLI a grep would have missed. ⭐ The line is checkable: exactly **two** reasons in the whole site layer mean the caller lacks authority.
- ⛔ **NOT A LOOSENING, and the structure guarantees it**: `authorized()` returns `Denied` before the effect closure runs, so an unauthorized caller still gets 403 having learned nothing. Only the ordering ever closed the oracle, and the ordering is unchanged. The audit is unchanged too — both are recorded `denied`, because the act did not take effect either way.
- ⭐ **EVERY TEST CALL SITE NOW STATES WHICH CLASS IT EXPECTS.** One helper matching both is how a suite of eleven tests could not tell them apart; there are now two, each panicking if handed the other.
- ✅ Falsified by restoring the rendering shipped one commit ago — red by name, source restored byte-identical. `evaluation` 3/3, `policy` 26/26, `site_authority` 11/11, `site_operator_cli` 3/3, `site_registry_http` 8/8, `profiles` 63/63, `mcp_write` 7/7, `allowlist` 2/2, `regions` 3/3; clippy 0 warnings; fmt, book and the doctrine gate rc=0.

## 2026-09-22 — The evaluation harness is site-operator gated, and gating it exposed three defects nothing else could reach (`SIGNOFF-REPAIR.8.2.5.3`, closing `.8.2.5`)

`REASONBRAID-REPAIR-0392`. Item 1 of the sequence is complete: the only live exposure of the five gaps is closed.

- ✅ **ALL SEVEN WRITES ARE SITE ACTS.** Authorization, the effect and the audit are one ordered transaction, in the shape DOC-0029 ruled and four existing site surfaces already use. Until now each admitted on bare enrolment across seven tenant-less tables, so any enrolled principal in the deployment set the standard the whole site measured against.
- ⭐ **TWO CAPABILITIES, DERIVED RATHER THAN PREFERRED**: `evaluation_record` sets the standard and `gate_evaluate` measures against it, because §4.1's charter names separation-of-duties and a party that measures must not be able to move the standard.
- ✅ **FALSIFIED IN BOTH DIRECTIONS.** Removing the gate from one route returns the pre-repair `400 invalid_command` — the exposure itself. Collapsing the two actions into one refuses a `gate_evaluate` holder who should be able to evaluate. Both sources restored byte-identical.
- 🔴 **THREE DEFECTS THE GATE MADE VISIBLE, none reachable before, which is the argument for gating rather than merely auditing.** A failed INSERT **aborted the audit transaction**, so a duplicate returned 500 rather than an audited refusal — `policy::register`'s own comment records the identical trap and its remedy. `ghost_run`, `ghost_gate` and the absent-trial refusal were all modelled as `Duplicate`, so one enum arm meant both *taken* and *does not exist*, and a caller was told *that calibration id already exists* when the defect was a run that does not. And the first refusal strings were one coarse class for the whole module, now a per-act pair.
- ⚠️ **TWO REFUSALS MOVE 400 → 403, deliberately.** Whether a coordinate is taken, or a named run or gate exists, is a question about the database, answered inside the gate as an audited refusal. Answering it earlier would hand a caller with no site authority an existence oracle over a registry it may not write. ✅ Input validation is untouched and still returns its typed 400 — exactly what `.8.2.5.2` was built to preserve, and the suite now asserts the two side by side.
- ⚠️ **A documented wire change on two routes**, whose bare bodies move under `results` / `scores` beside `reason`; the other five gain a `reason` field.
- ✅ **READS STAY ON ENROLMENT, STATED RATHER THAN CHANGED BY OMISSION**: the tables are site-wide by design and §19.7's gate manifest is meant to be auditable. Gating them is a separate confidentiality decision and is explicitly not taken here.
- ✅ **AN INDEPENDENT INSTRUMENT MEASURED THE REPAIR WITHOUT BEING ASKED.** `SHARED-REGISTRY-WRITES` refused the commit because its baseline pins the SET, not the count: all seven evaluation routes moved from `identity only` to `site authority`, printed line by line. Site-global writers stay **25**; those on **identity alone fall 16 → 9**, and the tables reached by an enrolment-only write **17 → 10**. ⭐ The first number not moving is the informative half — the tables are still site-global and are meant to be; what left is the routes' unguarded admission.
- ⛔ **The dated records that restate the old figures are NOT rewritten**, and `.7.1.2.1` set that precedent for this exact situation when it gated `workflow_profiles`. Only the live baseline and the census's own pinned arms are updated, each with the one change that caused the movement — that instrument's established practice, and its `--self-test` returns 45/45.

- ✅ `evaluation` **3/3** with six new controls, `policy` **26/26**, `site_authority` **11/11**, `migration_upgrade` **8/8**; clippy 0 warnings; fmt, book and the doctrine gate rc=0.

## 2026-09-22 — Validation moves ahead of the gate, and no test had to change (`SIGNOFF-REPAIR.8.2.5.2`)

`REASONBRAID-REPAIR-0391`. A seam the gate turned out to need, found by implementing the one before it.

- 🔴 **THE GATE CANNOT WRAP THESE WRITES AS THEY STAND.** `authorized()` renders a refusal as a `&'static str` authorization reason, and `site_receipt_response` sends that to the caller — so an input error inside a site act becomes a **403 about authority for a 400 about input**, silently undoing the message quality `.8.2.1` through `.8.2.4` had just built.
- ⭐ **THE CONTRACT IS ALREADY WRITTEN DOWN, in `site_authority::workflows`'s own doc comment**: validate in the HTTP layer BEFORE the site call and keep it a typed 400, because *a caller that fails validation learns nothing about authority, and one that passes it still meets the gate* — and the write re-validates regardless, so the registry never stores an invalid row whatever called it. `crate::policy::validate` is called ahead of `site::register_policy` for the same reason.
- ✅ **SIX `validate_*` FUNCTIONS EXTRACTED**, each the pure head of its write and each still called first by it: the two digest shapes, the trial count and declared seed, the non-empty arms and cases with their cohort kinds, the run ids and Brier range, the threshold with the baseline's shape and every baseline score, and every measured score.
- ⚠️ **TWO HONESTY NOTES A *NO BEHAVIOUR CHANGE* CLAIM OWES.** `record_trial_results` gains no validator, because its only check is that the trial exists and that needs the database — so it will reach the gate with nothing to run ahead of it, which is correct rather than missing. And `evaluate_gate` now checks the submitted scores before it looks the gate up, so a malformed score against a nonexistent gate reports the score rather than the ghost gate; both were 400s and no control covers that pair.
- ⭐ **THE EMPTY-SCORES REFUSAL DELIBERATELY DID NOT MOVE.** *Compared no case* is a fact about the baseline's intersection with the submitted scores, so it is not decidable without the stored row. It stays inside the write, and the validator's doc comment says so rather than leaving a reader to wonder why one check moved and its neighbour did not.
- ✅ **THE EVIDENCE A SEAM CAN OFFER**: `evaluation` 3/3 live and 3/3 unit, with `git diff --stat` over the test directory **empty**. No test was edited. Clippy 0 warnings, fmt and the doctrine gate rc=0.

## 2026-09-22 — The two evaluation site actions land before anything constructs them (`SIGNOFF-REPAIR.8.2.5.1`)

`REASONBRAID-REPAIR-0390`. Item 1 of the sequence begins, in `.9.3.4`'s measured decomposition.

- ⭐ **THE DIFF REACHES NOTHING, WHICH IS THE POINT.** `Action::EvaluationRecord` and `Action::GateEvaluate` with their wire names, plus one additive migration widening both `CHECK` constraints. No handler, no test, no other module — a variant nothing constructs changes no behaviour, and a widened constraint admits names no row holds.
- ⭐ **TWO ACTIONS, NOT ONE, AND THE SPLIT IS DERIVED RATHER THAN PREFERRED.** ROADMAP §4.1's charter names *separation-of-duties and conflict-of-interest constraints*, and §19's release flow has two parties: one maintains the corpus, the runs and the gate baselines, while CI evaluates gates continuously. ⛔ **A party that measures against a standard must not be able to move the standard**, so `evaluation_record` covers the six standard-setting writes and `gate_evaluate` covers running a gate.
- ⛔ **THE MIGRATION CANNOT FAIL AGAINST EXISTING DATA AND SAYS SO.** The action set lives in two `CHECK` constraints rather than a lookup table, so widening it is a migration by construction, and no row can already hold either name.
- 🔎 **THE SCOPE WAS NARROWED MID-IMPLEMENTATION, BY THE LEAF'S OWN ACCEPTANCE.** It first also owned converting the seven evaluation writes from a pool to a connection. The compiler refused that seven times, because a handler holding a pool cannot pass it where a connection is wanted — and the clause *no handler changes, because a seam that changes a caller is not a seam* made that a scope error rather than a compile error to route around. ⭐ The churn would also have been scaffolding: `.2` puts the handlers back on a pool, since `authorized()` supplies the connection from inside.
- ✅ **VERIFIED BY THE SUITES THAT WOULD NOTICE**, not the cheap ones: `migration_upgrade` **8/8** (the constraint replacement applies over a populated schema), `site_authority` **11/11**, `evaluation` **3/3**, no test edited. Clippy 0 warnings, fmt and the doctrine gate rc=0.
- ⛔ **The book is deliberately unchanged**: no route behaves differently yet, and documenting a gate that does not gate is the drift the mdBook rule exists to prevent.

## 2026-09-22 — Every remaining roadmap gap is owned by an executable leaf, and the order is a rule rather than a preference (`SIGNOFF-REPAIR.15`)

`REASONBRAID-DOC-0123`. The director's instruction: own, track and work all five, and sequence the whole of it here.

- 🔴 **TWO OF THE FIVE HAD NO OWNER ANYONE COULD FINISH.** A finding routed to a container reads as owned and is not — `TOOLBOX.md`'s test is *can someone open that leaf and finish it*, and `.8.2` carries a goal line naming five mechanisms with no acceptance of its own. `.8.2.5` (the evaluation gate) and `.8.1.1` (the ballot) were created, each with its own acceptance.
- 🔴 **AND THE AUDIT'S REAL FIND: THE ONE FINDING EVERYONE CALLED BLOCKED IS THE MOST READY.** `.8.2` clause 1 and the shipped book chapter both publish *no `GrantAction` and no `TargetSelector` can name a corpus or a gate, so there is nothing for an authority check to bind to yet*. Wrong twice, and both halves were a reading where a measurement was available:
  - The gate does not bind through `GrantAction`. `docs/decisions/` already records DOC-0029's verdict for this exact family — **site-wide by design, gated by SITE-OPERATOR grants** — and four site-wide surfaces already use that mechanism.
  - `.9.3.4` did not close the vocabulary. It ruled `GrantAction` **EXTENDS**, and extended it five times.
- ⛔ **Both places that published it are corrected in this commit**, the task tree and the book, rather than the correction living only here.
- ⭐ **THE SEQUENCE COMES FROM A STATED RULE, so a later reader can check it rather than trust it**: a live exposure on a shipped surface first; then every outstanding scope decision, largest possible deletion first, because each is hours and a build against an undecided contract is rework by construction; then the builds in dependency order. Recorded in `docs/decisions/2026-09-22_the-remaining-roadmap-gaps-are-sequenced-by-exposure-then-deletion.md` with its alternatives and what would make it wrong.
- **The order**: the evaluation family's site-operator gate; the publication store decision; the observability scope; the decision-rule contract; the moderator's scope; then the counted outcome, then whatever the decisions keep.
- ⛔ **The dependency is enforced by the leaf, not by the record being remembered.** `.8.1.1`'s acceptance requires `.11.4.7.2.1.2`'s contract to be quoted and built against, so starting the ballot early fails its own gate.
- ⭐ **After the fifth item the remaining build is fully scoped**, which is a deliberate checkpoint: the true size of what is left is visible before any large build starts.

## 2026-09-22 — The gate binds its corpus and the calibration binds its runs, and the split's own table is what found them (`SIGNOFF-REPAIR.8.2.4`)

`REASONBRAID-REPAIR-0389`. `TOOLBOX.md` says to enumerate a goal line's mechanisms against the children a split produces. It paid on first use.

- ⭐ **THE TABLE SURFACED TWO DEFECTS NOBODY HAD RECORDED.** Mapping `.8.2`'s five goal-line mechanisms and four attached clauses against its three closed children left two rows with no owner and no verdict — and both turned out to be live.
- 🔴 **`record_gate` named a corpus and never asked whether it existed.** `record_run` and `create_trial`, in the same file, both run the existence check and refuse with `UnknownCorpus`; the gate was the one surface of the three that named a corpus version and validated only its threshold and baseline. With the check mutated out the server returns 200 and creates `{"gate_id":"g5-unbound","corpus_id":"cal-corpus-absent",…}`.
- 🔴 **`record_calibration` checked each run's EXISTENCE and never its eligibility.** With the check disabled a `blind` calibration accumulates a `sighted` run and the stored row asserts a provenance its runs do not share — which is exactly what *derive calibration from eligible runs* forbids.
- ⭐ **THE CALIBRATION'S REPAIR CHANGES THE QUESTION, NOT JUST THE ANSWER.** The loop asked for a boolean; it now selects the run's own `(corpus, version, workflow)` so the refusal NAMES what disagreed. A boolean could only have said no, and a caller holding two dozen run ids needs to know which one. `ineligible_run` is named separately from `ghost_run` because one says a run id is wrong and the other says the run is real and in the wrong calibration.
- ✅ **TWO NEGATIVE CONTROLS**, because a check that refuses everything is this defect mirrored: a gate against the registered corpus and a calibration over eligible runs both still return 200 in the same run.
- 🔎 **AND PLACING THEM COST A LESSON.** Written where the other refusals sit, the negatives REGISTER a gate and a calibration — and the walk's later list assertions pin exact counts, so the first green run failed at `gates.len() == 1`. Rewriting that expectation would have been this leaf quietly loosening an assertion it does not own. The block runs last instead, and the controls were re-falsified AFTER the move, because a relocated control is one whose red nobody has seen.
- ⛔ **`.8.2` stays open and the table says why**: two mechanisms have no owner, and clause 1 is blocked on vocabulary `.9.3.4` measured as absent. Naming them unowned is the point of writing the table.
- ✅ `evaluation` **3/3**, `policy` **26/26**, clippy **0 warnings**, fmt, `make book` and the doctrine gate rc=0; source restored byte-identical by SHA-256.

## 2026-09-22 — 179 runs, no occurrence, and the hunt is tracked so the next sample keeps its denominator (`SIGNOFF-REPAIR.11.26.3`)

`REASONBRAID-REPAIR-0388`. The frontier leaf's declared next action is *wait*, and waiting had no instrument.

- ⛔ **THE SAMPLE, WITH ITS DENOMINATOR**, because `.11.26`'s own note says an unstated one is not a measurement: **0 failures in 179 runs** of the pre-push script suite with the timing log armed and the stall watchdog in place — 40 on a quiet machine (17.8 / 19.1 / 24.8 s) and **139 while cargo builds, strict clippy, four live PostgreSQL suites and a full-history git replay ran alongside** (22.5 / 25.7 / 30.5 s).
- ⭐ **ROUND 2 IS THE STRONGER NEGATIVE.** The suite slowed by about **35%** under that load and never failed — a third measurement against *the machine was busy*, an explanation this lane has already withdrawn twice. ⛔ No lever was tried, deliberately: `.11.26` forbids a fourth.
- ⚠️ **AND THE PER-PHASE RECORD SURVIVES FOR ONLY 3 OF THE 179 — this leaf's own defect repeating.** `.11.26.2` lost a 240-invocation distribution to an untracked file that had been replaced; the first hunt written here bounded disk by deleting all but the last two runs, so 179 runs produced 3 runs of evidence.
- ⭐ **THE RULE IS NOW KEYED ON THE VERDICT, NOT ON AGE**: a passing run's artefacts are disposable, a failing run's are the entire point. `scripts/hunt_gate_stall.sh` is tracked, stops on the first non-zero rc, names the log and timing file it retained, and prints `runs= failures= wall_s min/mean/max` so a sample cannot be published without its denominator. ⛔ Not a gate, and it says so.
- ⭐ **ITS `--self-test` CAUGHT IT ON THE FIRST RUN**: arm 4 reported `prune_passing: command not found`, because the retention helper was defined BELOW the arm that calls it — so the arm asserting a passing run is pruned was testing nothing. `SELF-TEST`'s founding incident, in a new script, on day one. The retention arm is two-sided: the run the prune was not asked about must survive.
- ⛔ **`.11.26` stays open.** Its acceptance asks for ONE occurrence caught with the instrument, and 179 runs did not produce one.

## 2026-09-22 — The modulo moves into u64, and the control is built so the defect and the repair can disagree (`SIGNOFF-REPAIR.8.2.3`)

`REASONBRAID-REPAIR-0387`. The evaluation chapter's third and last published limit.

- 🔴 **ONE CAST AGAINST ONE SENTENCE.** The shadow-trial arm was chosen with `(draw as usize) % arms.len()`, narrowing a `u64` draw BEFORE the modulo — which truncates to 32 bits on a 32-bit host — while the comment three lines above promised *stable across runs and platforms* as the function's whole purpose. The fix reorders it: `draw % arms.len()` in `u64`, narrowed after, where the result is already smaller than a `usize`.
- ⭐ **THE HARD PART IS THAT THE CONTROL CANNOT GO RED ON THIS HOST BY ITSELF.** On 64 bits `draw as usize` is the identity, so old and new agree for every input and an ordinary control passes before AND after — the anti-pattern `docs/CLAIM_VERIFICATION.md` opens with. The constant is chosen to break the tie: `0x1_0000_0003 % 5` is **4**, while its low 32 bits give **3**.
- ✅ **THE ARM IS TWO-SIDED**, asserting the right answer and that the 32-bit narrowing gives a different one — so a future edit that made the constant narrow harmlessly fails the second assertion instead of silently satisfying the first.
- ✅ **FALSIFIED BY SIMULATING THE TARGET THIS PROJECT CANNOT RUN.** A mutant writing `((draw as u32) as usize) % arms` makes exactly one control fail, by name, with the other two green.
- ✅ **TWO MORE PURE CONTROLS**, because one arm over one constant is a claim about that constant: the index stays inside the arm list across `0`, `1`, `u32::MAX`, the wide draw and `u64::MAX` for every list length 1..=8; and the draw is stable for one `(seed, case_id)` and moves when either does. ⚠️ This module had **no** unit tests at all before.
- ⛔ **Not claimed: that anything here has run on a 32-bit target.** It has not; the defect was latent. It is repaired because a trial row's contract is that `(seed, case_id, arms)` reproduces it, and a guarantee true only of the hosts tried so far is what this lane keeps finding.
- ⭐ **With `.8.2.1` and `.8.2.2`, every published limit the evaluation chapter carried is now a repair record.**

## 2026-09-22 — A store fault is the server's, and the caller's duplicate is the one the database says it is (`SIGNOFF-REPAIR.8.2.2`)

`REASONBRAID-REPAIR-0386`. The evaluation chapter's second published limit, repaired.

- 🔴 **THE DEFECT, REPRODUCED LIVE.** A PL/pgSQL trigger raising on every `evaluation_corpora` INSERT made the running server answer `{"code":"invalid_command","message":"corpus 'cal-corpus-faulted' version 1 already exists — register a new version or run id instead of overwriting"}`. A database refusing every write told the caller its input was a duplicate, with HTTP **400**.
- ⭐ **THE CENSUS IS 12 SITES, NOT THE 9 THE CLAUSE RECORDED**, and the two it missed are a spelling: one write uses `map_err` rather than `Err(_) =>`, so a grep keyed on the code SHAPE could not see it, and its neighbouring existence check maps a store fault to `Duplicate` too. ⛔ A population keyed on a shape rather than on the call is a failure mode this project has measured before.
- ⭐ **THE FIX IS A CLASSIFIER AND A MAPPING, AND NEITHER IS A JUDGEMENT.** `write_failure` asks the database — `is_unique_violation()` — so the caller's `Duplicate` survives for exactly the one error shape that means it, and every other becomes `Storage(sqlx::Error)` with the cause reachable through `source`. The five existence-SELECT arms convert **outright**, with no classifier, because the boolean beside each one already carries the genuine refusal.
- ⭐ **AND THE HTTP MAPPING MOVED INTO ONE PLACE.** All seven evaluation handlers wrote `invalid_command(error.to_string())` by hand; they now write `error.into()` against a single `From` impl. Seven copies of one decision is how one ends up disagreeing with the other six.
- ✅ **FALSIFIED TWICE, EACH MUTANT ISOLATING ONE HALF**: removing the `From` impl's `Storage` arm returns 400 with *the evaluation store is unavailable*, proving the mapping; forcing every write failure back to `Duplicate` returns the *already exists* message, proving the classifier.
- ✅ **TWO NEGATIVE CONTROLS IN THE SAME RUN**, because a repair that turns every refusal into a 500 is this defect mirrored: a genuine duplicate still returns **400** — with the very message the store fault used to borrow, so the two are now distinguishable on the wire — and so does a genuinely unregistered corpus, which reaches the caller through the existence boolean rather than the query's error arm.
- ⚠️ **DECLARED LIMIT.** The injected fault is on a WRITE, because a `BEFORE INSERT` trigger is the clean reversible injection this shared cluster allows. The five read arms are converted by inspection and covered only by the negative that their genuine refusal still returns 400. The write half is qualified by observation, the read half by argument, and they are not the same strength.
- ✅ `evaluation` **3/3**, `policy` **26/26**, clippy **0 warnings**, fmt, `make book` and the doctrine gate rc=0; both sources restored byte-identical by SHA-256.

## 2026-09-22 — A gate that compares no case is refused, and the read side now obeys the write side's rules (`SIGNOFF-REPAIR.8.2.1`)

`REASONBRAID-REPAIR-0385`. A product defect the book had been publishing as a limit.

- 🔴 **THE DEFECT, CAPTURED AS THE SERVER'S OWN RESPONSE.** `evaluate_gate` with an empty score object returned `{"failures":[],"gate_id":"g5-blind","passed":true}` — and the `INSERT` runs before the response is built, so that pass over nothing was **appended to `evaluation_gate_results`** as the gate's durable record. A non-numeric measurement dropped its case silently, and one outside `[0, 1]` was compared as written, so `5.0` cleared every threshold.
- ⭐ **THE EVIDENCE IS A CONTRAST INSIDE ONE FILE.** `record_gate`, twenty lines up, refuses an empty baseline, a non-numeric score and one outside `[0, 1]`, each by name. The write side's rules are now the read side's rules, using the same `out_of_range` constructor so the message shape matches.
- ⛔ **WHAT WAS NOT CHANGED, AND IT WAS THE EASY THING TO CHANGE.** The comparison still skips an unmeasured baseline case: *the caller owns the coverage* is a declared contract, and turning it into *every case must be measured* would be a different product decision arriving inside a defect repair. Only the empty intersection refuses; the new `compared` and `unmeasured` fields make the surviving contract legible rather than merely legal.
- ✅ **FALSIFIED THREE TIMES, ONE MUTANT PER ARM**, each red naming its own arm — and the second and third print the mechanism rather than just a failure: `compared: 1, unmeasured: 1` is the string dropping its case while the gate passes on the other, and `compared: 2` with `5.0` submitted is the out-of-range value clearing a `0.8` baseline. ⛔ Without the new counts in the response neither would have been distinguishable from the first.
- ✅ **TWO NEGATIVE CONTROLS**, because three refusals with no accepting case is a gate that cannot pass: a partial evaluation still returns 200 with its counts, and `{1.0, 0.0}` is accepted, so the bound is the closed range.
- ✅ `evaluation` **3/3**, `policy` **26/26**, strict clippy, `cargo fmt --check`, `make book` and the doctrine gate all rc=0; `evaluation.rs` restored byte-identical by SHA-256 after every mutant.
- ⭐ **THE BOOK CHANGES THE MOST.** It carried this as a published limit telling readers not to treat a green gate as evidence. That paragraph is now the repair's record, the chapter's opening warning loses the clause that named it, and the gate section documents the three refusals and the two counts.

## 2026-09-22 — All 21 rows carry a verdict, and 38 rows are 36 distinct deferrals (`SIGNOFF-REPAIR.11.4.7.2.1.4.3`, closing `.4`)

`REASONBRAID-DOC-0122`. Tranche 3 of 3, and the pass corrects a population this lane published two commits ago.

- ✅ **FOUR MORE `discharged`**: the adapter conformance kit (`tests/adapter_conformance.rs`, `tests/conformance/`, an 11-entry fixture corpus with its permanent-failure cases, and a third-party certification suite); the Phase-3 directory row, a superset of the gate record's #1 whose every added noun lands too; resource acquisition, including `migrations/0029_derivations.sql` — the derivation graph, the one noun a reader would expect to be missing; and the Phase-6 policy stack. ⚠️ That last one is discharged as MACHINERY: what a publication actually writes is `.9.3.5`'s open question, and the two are not the same claim.
- ⚠️ **TWO MORE SPLITS.** Row 18 is **3 of 6**: the workflow engine, minority reports and the synthesizer half landed; decision rules and expected-artifact semantics are `.11.4.7.2.1.2`, the moderator half is `.11.4.7.2.1.3`. Row 20 is **2 of 3**: federation and the A2A/MCP gateways shipped, and Internet hardening is not met and says so — `.14` owns the exposure profile under a standing prohibition on deploying it.
- 🔴 **ONE FRESH ABSENCE, AND THE SUPERSET IS WHY IT SURFACED: votes/abstentions.** `vote` exists as a workflow STEP in `TERMINAL_KINDS` and in the `policy_proposal` profile, so any census keyed on the word returns hits — but there is no ballot table and `abstain` appears **nowhere** in `crates` or `migrations`. The step exists; the ballot does not. ⛔ The gate record's narrower *decision-rule create fields* would never have led a reader here; the subtraction record's wider row did.
- ✅ **THE POPULATION, CORRECTED BY THE PASS ITSELF.** **38 deferral rows** is exact and was measured by command. There are **36 distinct deferral statements**: rows 9 and 21 are verbatim restatements of the gate record's #4 and #6. ⛔ Two further rows CONTAIN an already-graded gate row and are deliberately NOT collapsed — a containment is not a duplicate, and folding them in would erase the extra nouns, one of which is the absence above.
- ⭐ **THE PASS IS COMPLETE.** 13 discharged, 3 splits, 2 fired and superseded, 2 not yet triggered, 1 fired and open — and every open half names an executable owner with its own acceptance. None is parked.

## 2026-09-22 — Tranche 2's seven verdicts, and the first proof that the deferral population double-counts (`SIGNOFF-REPAIR.11.4.7.2.1.4.2`)

`REASONBRAID-DOC-0121`. The Phase-1 subtraction record's Phase-2 rows, graded one command each.

- ✅ **FIVE `discharged`**: the workload certificate lifecycle (issuance in `ca.rs`, `node_certificates` in the schema, **45** revocation sites in `node_admin.rs`, and presence derived from whether every certificate is revoked); scoped grants and cached-decision rules (`authority/` is a module, `0013_cached_decisions.sql` is the store); the provider-attempt and spend-breaker family; production leases and fencing, where `0015_lease_epoch.sql`'s epoch IS the fencing token and `dead_letter` appears at 11 sites; and the incarnation/run writers.
- 🔴 **AND THAT LAST ONE IS A VERBATIM DUPLICATE OF THE GATE RECORD'S #4.** The two records describe one deferral twice. ⛔ *38 deferral ROWS* is exact; *38 distinct deferrals* is not, and neither is *21 unfinished jobs*. Tranche 3 owes the corrected distinct count, because three of the four overlaps sit in it.
- ⚠️ **ROW 13 SPLITS, AND IT IS THE TRANCHE'S MOST USEFUL RESULT.** `backup_restore.rs` and `migration_upgrade.rs` are live suites, so two thirds of *backup/PITR, object/Git inventory, upgrade/rollback testing* is discharged — and a row-level verdict would have called the whole thing done and buried the object-store absence for the **second time in two tranches**. ⭐ One gap reaching two independent deferral rows is what makes `.9.3.5` a lane rather than a tidy-up.
- 🔴 **ONE `fired and open`: the observability row.** `telemetry.rs` ships a genuine dev-profile slice — JSON-line operational logs and an in-process registry under ADR-023's four-record doctrine — and its own header carries the deferral: *the OpenTelemetry sink stays the ADR-023 trigger*. No sink dependency exists, and every SLO, dashboard and game-day hit in the tree is documentation. ⛔ A revisit condition in prose with nothing evaluating it, this time inside a source comment. Owned by a new `.4.6`, whose acceptance splits the row's four nouns because carrying them as one unit is what hid them.
- ⭐ **A SHAPE WORTH NAMING: work done AHEAD of its trigger is `discharged`.** The certificate lifecycle's condition — non-loopback exposure — has never fired, and Phase 2 identity shipped the work anyway. The verdict is about the work, so it closes; saying otherwise leaves a finished item on a backlog for ever.

The entries before those above were rotated into reachable Git history at the
**forty-fourth rotation** (`SIGNOFF-REPAIR.11.4.1.6`, which owns this ledger’s rotation). The exact predecessor — this file as it
stood at the commit named below, which is the object every retired record was
checked against before this notice was written — is:

```bash
git show d36b5e178164639a106cb22180ca455ef83b2b92:CHANGELOG.md
```

That snapshot is 91577 bytes and 391 lines, and contains 29 dated
entries; its Git blob is `c0fbcdeb59db65975946659004ecae837e8c5855` and its SHA-256 is
`222ca45348f0757ed74275a707f138647a30afd8744f53e9f315cc32a54583ae`. It carries the forty-third rotation's
notice in turn, and each earlier notice names the one before it, so the chain
walks all the way back. `docs/decisions/2026-09-09_changelog-rotation.md` holds
the first transition's evidence.

⛔ **14 record(s) rotated out, 16 kept, lossless** — every retired heading was retrieved from the
predecessor named above before this notice was written, and every figure in it was re-derived from that object with
`git rev-parse`, `git cat-file` and SHA-256 rather than typed. ⭐ The cut is DERIVED, not chosen: it retires whole
records until the ledger has at least 10 commits of runway at the p90 entry size measured over the last
60 non-rotation commits — because two rotations that stopped at the threshold instead left 344 and 296 bytes and
the first forced another rotation on the very next commit (`SIGNOFF-REPAIR.11.4.1.6`).
