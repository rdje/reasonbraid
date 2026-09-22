# LIVE_STATUS.md — authoritative live progress tracker

Rows use only **Done · Mostly Done · In Progress · Not Started**. This is a current
snapshot. Historical implementation and verification records live in the phase
task-trees and git; the pre-review snapshot is `9c2d2ba:LIVE_STATUS.md`.

## Current status

| Area | Status | Current evidence and remaining work |
| --- | --- | --- |
| Roadmap and task-tree conversion | Done | `PROGRAM` maps all phases, gates, backlog items, ADRs and demonstrations; `RB-SEED.2` holds the original census. |
| Claim-verification policy | Done | Local policy matches the director-authorized pgen donor at the startup comparison; subsequent claims still need all three verification legs. |
| Discipline and continuity | In Progress | Repair ownership is recorded; `.2` and `.11` own local storage, disposable verification, doctrine defects and document containment. `.11.4.1` rotates the recent changelog through verified Git history; broader containment remains open. |
| Phase 0 — contracts and experiments | Mostly Done | Historical G0 package and owner signoff retained in `PHASE-0`; affected authority/budget/adapter assertions require corrective evidence. |
| Phase 1 — LAN vertical slice | Mostly Done | Historical demonstration retained in `PHASE-1`; current authority, console and demo-script findings remain open. |
| Phase 2 — identity, delivery and recovery | Mostly Done | Historical machinery retained in `PHASE-2`; revocation, fencing, budget and recovery repairs are `.3`–`.4`. |
| Phase 3 — directory and recruitment | Mostly Done | Historical machinery retained in `PHASE-3`; tenant visibility, recruitment and automatic initiation repairs are `.5`. |
| Phase 4 — resources and evidence | Mostly Done | Historical G4 record retained in `PHASE-4`; acquisition isolation, evidence integrity and retention repairs are `.7`. |
| Phase 5 — deliberation and evaluation | Mostly Done | Historical G5 subtraction gate withdrew the quality-lift claim; workflow and evaluation repairs are `.8`. |
| Phase 6 — governance | Mostly Done | Historical G3 machinery exit retained; binding use remains gated; policy/publication/deployment repairs are `.9`. |
| Phase 7 — Internet qualification | Mostly Done | Hardening machinery exists; G6/G7 Internet exposure remains NOT MET. Local repairs and external threat-model, injection and penetration-test evidence remain required. |
| Phase 8 — federation and interoperability | In Progress | Through regional routing historically recorded; `.5.3` store-and-forward, `.5.4` exit export/import and `.6` G8 remain. Shared authority and protocol gaps are prerequisite repairs. |
| Phase 9 — stable release | Not Started | G9 requires sustained operational evidence and the outstanding release decisions. |
| Corrective review | In Progress | `.1`, `.2.1`, `.2.2`, `.3.1` complete; `.3.2.1` site service passes ten live controls and strict lint. `.3.2.2` operator CLI passes 3 live controls plus adjacent service/ownership checks and strict lint; `.3.2.3` HTTP enforcement passes 8 focused controls; the selected 58-test security run and final 12-test registry run pass. `.3.3.1` core subject serialization passes 49 unit + 3 subject controls and 40 live compatibility tests; `.3.3.2` bound evaluation passes 51 core unit + 3 subject tests, 6 evaluator controls, 32 live authority/command API tests and strict lint; all results consumed and the owned cluster removed. `.3.3.3.1` actual-parent command selection passes 37 live tests, six evaluator controls and strict lint. `.3.3.3.2.1` frozen-read eligibility passes 40 live tests, ten pure evaluator controls and strict lint; all results consumed and the owned cluster removed. `.3.3.3.2.2.1` provenance and strict record/selector decoding passes 51 core units, seven metadata/subject controls, 44 live authority/HTTP/upgrade tests and strict lint; all confirmation/shutdown results consumed, owned cluster removed. Seven HTTP inspection receipt producers pass 45 live authority/API tests, ten pure evaluator controls and strict lint; all results and shutdown consumed. Exact scoped receipt readback passes 18 live authority tests and 30 HTTP tests plus strict lint, including final denied-record readback; all results/shutdown consumed. Actual-parent selection and frozen-read audit/readback `.3.3.3` are complete; The `.3.3.4.1` source census/contract is complete (42 named-call locations plus transitive/mutation coverage). `.3.3.4.2` qualifies the guard/connection owner and migration delivery: 35 controls (34 live / one pure), 28 directory rebuild/cache checks and four-crate strict lint pass, including cancelled-BEGIN replacement and the formerly stale authority executable. Seven acceptance-checker controls also pass, correcting nested-evidence owner selection while preserving real-tree box refusals. All results/shutdown consumed; owned clusters/probes removed. Grant error classification `.3.3.4.3.1` passes 56 live authority/HTTP/card controls and focused strict lint: storage causes survive, malformed active data returns safe 500, genuine structural 400s and exact failure/recovery effects are verified. All results/shutdown consumed; four owned clusters absent. Standalone writer/status integration `.3.3.4.3.2` now passes 85 selected controls (84 live / one pure), focused strict lint and book checks: five guarded paths, live parent issuance, preserved malformed status, public commit uncertainty and exact race/failure/recovery controls. The old contention fixture now verifies the observed target→guard dependency. All results/shutdown consumed; three owned clusters absent. Typed rollback support `.3.3.4.3.3.1` passes 89 selected controls (88 live / one pure), focused strict lint and rendered book checks: domain errors roll back provisional rows/new anchors, existing anchors remain, deferred anchor faults cannot replace refusals, and SQL causes/deadlines/commit uncertainty survive. All results/shutdown consumed; both owned clusters absent. Complete enrollment `.3.3.4.3.3.2` passes 97 selected controls (96 live / one pure), final focused strict lint and rendered book checks: one exclusive transaction before replay through every write, database-time liveness, honest concurrent replay, complete rollback and preserved commit phase. All results/shutdown consumed; three owned clusters absent. Bootstrap uncertainty `.3.3.4.3.3.3.1` is now runtime-confirmed: a 500 commit-uncertain response can precede the original committed tenant, and a repeated no-key request creates a distinct tenant. All 25 selected controls (24 live / one pure), focused strict lint and book checks pass; all results/shutdown consumed, owned cluster absent. Server bootstrap recovery `.3.3.4.3.3.3.2` passes 73 selected controls (72 live / one pure), the final eleven-control fixture rerun, three-crate all-target strict lint and final fixture lint. Canonical RequestId, complete guarded outcomes, exact conflicts, concurrent rollback/replay, actual commit/response-loss recovery, malformed-storage refusal and one shared deadline are qualified. All results/shutdown are consumed and four owned clusters are absent. StateFile storage `.3.3.4.3.3.3.3.1.1` passes twelve selected controls, all-target CLI strict lint and rendered book checks. Linked-target overwrite, changed open-reader snapshots and ignored process exclusion are repaired; strict bounded preservation and synchronized replacement are qualified on the native macOS repository volume. All results are consumed and unique fixtures are absent. Whole CLI writer integration `.3.3.4.3.3.3.3.1.2` passes nineteen selected controls, final all-target CLI strict lint and book checks: pre-dispatch exclusion, fresh actor/merge, both overlap orders, process-loss release and preserved real-server flow/denials. All results/shutdown are consumed; unique fixtures and the owned cluster are absent. Recovery schema `.3.3.4.3.3.3.3.2.1` passes twenty-four selected controls, all-target CLI strict lint and book checks: strict pending/completed records, preserved legacy wire shape, guarded multi-publication continuity and zero-dispatch refusal while pending. All results are consumed and unique fixtures absent; the interrupted pre-main launch and unchanged-binary successful retry are preserved honestly. Keyed CLI/explicit recovery `.3.3.4.3.3.3.3.2.2` passes thirty-three selected controls, final output-window rerun and strict CLI lint: durable matching request before dispatch, original-byte complete reply checks, same-key failure recovery, historical local receipt recovery without HTTP and deliberate distinct fresh tenant creation. The actual unread-output interruption retains its original result; all results/shutdown consumed, unique fixtures and owned cluster absent. Completion-capacity preflight `.3.3.4.3.3.3.3.2.3.1` passes thirty-one selected controls, final fresh/pending/exact-fit matrix and strict CLI lint: an oversized completion refuses before HTTP with exact snapshot/pending preservation, while the exact-limit case succeeds. All results consumed and unique fixtures absent; sizing samples never become outcome evidence and physical disk reservation is not claimed. Bounded HTTP waits and restart qualification follow; ordinary no-key behavior remains intentionally distinct. The final administrative effect REPRESENTATION `.3.3.4.7.1` passes 10 representation controls within 68 core tests, strict core lint and the falsified object-only control; its census derives the closed fourteen-operation set from 27 guarded-admission call sites. Its STORAGE `.3.3.4.7.2` then passes 8 live controls (falsified 5/3 against three attributable injections): migration 0058 keyed on the admission with a composite tenant-binding foreign key, a writer on the caller's already-guarded transaction, and a tenant-filtered reader. `.3.3.4.8` is the first producer of an effect record; `.3.3.4.9` is the second, putting spend-breaker arm/reset onto one exclusive-guard transaction — 25 live controls, the affected set 5 suites / 94 tests, falsified 17/8 against the exact pre-`.9` handlers.. Fixture ownership `.11.2.1.1` censuses all 61 `create_dir_all` sites in tracked Rust with 0 unresolved and repairs the 3 that adopted: exclusive per-call creation on the repository volume, falsified against an occupied path, with 89 selected controls across five suites (63 of them `profiles` on a disposable cluster), strict lint for both crates and the 25-check gate green. |

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

The entries before those above were rotated into reachable Git history at the
**sixth rotation** (`SIGNOFF-REPAIR.11.4.1.6`, which owns this ledger’s rotation). The exact predecessor — this file as it
stood at the commit named below, which is the object every retired record was
checked against before this notice was written — is:

```bash
git show 38dedf9f51706715427decce5e72a31d2e587681:LIVE_STATUS.md
```

That snapshot is 48751 bytes and 206 lines, and contains 12 dated
entries; its Git blob is `5936b8e2b28bcbae320a2ae4adbe1340e2ce8fe0` and its SHA-256 is
`a37c15b3423f14e43c35e088922b0b393abddf9f418d2e3e50dc94f607352f1e`. It carries the fifth rotation's
notice in turn, and each earlier notice names the one before it, so the chain
walks all the way back. `docs/decisions/2026-09-09_changelog-rotation.md` holds
the first transition's evidence.

⛔ **11 record(s) rotated out, 2 kept, lossless** — every retired heading was retrieved from the
predecessor named above before this notice was written, and every figure in it was re-derived from that object with
`git rev-parse`, `git cat-file` and SHA-256 rather than typed. ⭐ The cut is DERIVED, not chosen: it retires whole
records until the ledger has at least 10 commits of runway at the p90 entry size measured over the last
60 non-rotation commits — because two rotations that stopped at the threshold instead left 344 and 296 bytes and
the first forced another rotation on the very next commit (`SIGNOFF-REPAIR.11.4.1.6`).
