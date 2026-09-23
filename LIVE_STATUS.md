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

## 2026-09-23 — One machine can no longer silence another machine's answer by reusing its message id (`SIGNOFF-REPAIR.4.3.1`)

`REASONBRAID-REPAIR-0455`. The first of the four inbox-identity gaps found by `REASONBRAID-DOC-0152`.

- 🔴 **Before:** every machine chooses its own message ids, but the server treated them as if they were unique across all machines. If machine B had already used a message id, machine A's own message under that id was treated as a repeat: it was dropped, and A's finished answer was never counted. And when a machine reconnected and asked "do you already hold my result for this job?", the server answered from *any* machine's records — so B's receipt could close A's uncertain job as done, and B's message id was shown to A.
- ✅ **Now:** receipts are kept per machine. A repeat is only a repeat of that same machine's own message; another machine's use of the same id is that machine's own first message. The reconnect questions are answered only from the asking machine's own receipts.
- ✅ Existing receipts were not rewritten; the database key was widened (an additive change).
- ✅ Tested: the two new checks failed on the old code exactly where predicted and pass now (the machine-channel suite: 49 tests); a deliberately broken version that answered the reconnect questions from any machine's records was caught; six further suites that touch receipts pass unchanged (121 tests); strict lint clean.

## 2026-09-23 — The machine-inbox checklist checked against the code: four gaps found and scheduled (`SIGNOFF-REPAIR.4.3`)

`REASONBRAID-DOC-0152`. A review; no code changed.

- 🔴 One machine can, by reusing a message id, cause another machine's finished answer to be ignored, or be told about another machine's receipts.
- 🔴 If an operator clears out a machine's old delivered work entirely, the machine can be locked out, and later work can be silently skipped.
- 🔴 Two machines holding a job with the same name in one organisation would have only one answer counted.
- 🔴 Two jobs sent to one machine at the same instant can collide, and one is refused.
- ✅ All four are scheduled in order of risk, each to be proven with a failing test first.

## 2026-09-23 — Ending a partnership now also stops work that was already on its way (`SIGNOFF-REPAIR.5.3.6`)

`REASONBRAID-REPAIR-0454`. Completes the federation work (`SIGNOFF-REPAIR.5.3`) and with it the directory, recruitment and federation lane (`SIGNOFF-REPAIR.5`).

- 🔴 **Before:** after a partnership ended, new work for a partner's agent was refused, but work already queued could still be sent to the partner's machine (topic included) and run there.
- ✅ **Now:** the partner's machine is no longer offered that queued work, and on its next check-in it is told it no longer works for the importing organisation, so anything it already holds for them is refused instead of run.
- ✅ Tested: the new checks failed on the old code and pass now; two deliberately broken versions (the machine keeping the old organisation; the server listing organisations by inbox contents) were each caught; twelve suites pass, including the end-to-end CLI.

## 2026-09-23 — The partnership checklist re-checked: four items hold, one gap found and scheduled (`SIGNOFF-REPAIR.5.3`)

`REASONBRAID-DOC-0151`. A review; no code changed.

- ✅ **Holds:** recruiting a partner's agent under your own permissions; importing all-or-nothing; no duplicate imports; audit receipts for every cross-organisation act (the new delivery receipts point at records by id, which is sound inside one installation; tamper-proof fingerprints are scheduled for when partners run separate installations).
- 🔴 **Gap found:** if a partnership is ended while work is already queued for a partner's agent, the partner's machine can still be sent that work (including the conversation's topic) and can still run it. Nothing is recorded on the importing side — the answer is refused — but the partner still sees the topic after the partnership ended. Scheduled as the next fix.

## 2026-09-23 — Each delivery to a partner's machine now leaves an audit receipt on both sides (`SIGNOFF-REPAIR.5.3.5.3.3`)

`REASONBRAID-REPAIR-0453`. Completes cross-organisation recruitment (`SIGNOFF-REPAIR.5.3.5`).

- 🔴 **Before:** when a partner's machine picked up a job for an imported agent, neither organisation's audit trail recorded that work had crossed between them.
- ✅ **Now:** the moment the partner's machine confirms it has the job, both sides record a receipt in the same step. The importing side's receipt names the confirmation; the partner's side names the permission record the job runs under. Confirming twice never creates duplicates, and ordinary same-organisation work leaves no such receipt.
- ✅ With this, the whole "recruit a partner's agent" feature is complete: advertise the call, request to join, import, run on either machine, deliver, answer, audit.
- ✅ Tested: the new checks failed on the old code and pass now; two deliberately wrong versions (receipts for same-organisation work; receipts for plain traffic) were each caught; eight suites pass.

## 2026-09-23 — Agents that set no availability were getting no work; fixed (`SIGNOFF-REPAIR.11.4.7.2.1.5.3.2.2.3`)

`REASONBRAID-REPAIR-0452`.

- 🔴 **Before:** an agent whose profile didn't mention availability at all (the most common case) was sent **no work**, while the "who is available" listing showed it as available. Its profile stored "no availability" as an empty value, and the delivery code misread that as a broken setting and held everything back. The tests never noticed because they used a slightly different "empty" shape than real profiles do.
- ✅ **Now:** delivery and the availability listing read the setting through one shared function, so an agent with no availability set gets its work, and the two can't disagree again.
- ✅ Found while testing the next feature, which it was blocking; that work was set aside safely and resumes next.
- ✅ Tested: a new test using exactly what a real profile stores failed on the old code (no work delivered) and passes now; recreating the old misreading was caught; all delivery suites pass.

## 2026-09-23 — A partner agent's answer from its home machine is now credited to it (`SIGNOFF-REPAIR.5.3.5.3.1.4`)

`REASONBRAID-REPAIR-0451`. Completes "a partner's agent runs on its home machine" (`SIGNOFF-REPAIR.5.3.5.3.1`).

- 🔴 **Before:** when the partner's machine sent back the agent's answer, the server credited it to the partner machine's *own* agent instead of the imported one. That agent isn't part of the conversation, so the answer was rejected.
- ✅ **Now:** the answer is credited to the agent the job was for, which the server reads from its own record of the job, not from anything the machine says. So the contribution appears in the conversation under the imported agent, with the importing organisation's permissions. If the partnership ended after the job was sent, the late answer is refused.
- ✅ The whole path now works end to end: import → listed in the directory → seated on a panel → work delivered to the partner's machine → checked against the right organisation's revocations → answer credited correctly. What remains is an audit receipt on both sides for each delivery (next task).
- ✅ Tested: the new check failed on the old code (the answer was refused as the wrong agent) and passes now; switching off the "does this machine still run this agent" check was caught; four suites pass.

## 2026-09-23 — A partner's agent now receives its work on the partner's own machine (`SIGNOFF-REPAIR.5.3.5.3.1.3`)

`REASONBRAID-REPAIR-0450`.

- 🔴 **Before:** a partner agent bound to its home machine could join a conversation, but the work that followed was addressed to a machine that doesn't exist and sat there unread.
- ✅ **Now:** the work goes to the partner's machine, still as the importing organisation's job, so that machine checks it against the importing organisation's revocations (the previous step made that possible). The "too much unread work" safety limit is counted on the machine that will actually hold it. If the partnership has ended, accepting new work is refused with a clear reason instead of silently queueing it.
- ⚠️ **Not yet:** the partner machine's *answer* is still credited to the wrong identity and rejected. That's the next task, and the book says so.
- ✅ Tested: the new checks failed on the old code and pass now; two deliberately broken versions (wrong inbox; limit counted on the wrong machine) were each caught; the profile, work, channel and invitation suites pass.

## 2026-09-23 — A machine working for two organisations now checks each job against the right organisation's revocations (`SIGNOFF-REPAIR.5.3.5.3.2`)

`REASONBRAID-REPAIR-0449`.

- 🔴 **Before:** a machine only ever knew one organisation's revocation counter, its own, and checked every job against it before running it. That was harmless while a machine only ever worked for its own organisation. But a partner's agent running on its home machine (the feature being built) would put two organisations' jobs on one machine. Then a revocation in one organisation could wrongly block the other's valid work, or let stale work through.
- ✅ **Now:** the server tells the machine the current counter of every organisation whose jobs it holds, and the machine checks each job against **its own** organisation's counter. A job from an organisation the machine hasn't been told about yet is refused, never judged by someone else's counter.
- ✅ This is the safety step that had to come first; sending partner agents their actual work is next.
- ✅ Tested: new tests on both the machine side and the server side; recreating the old single-counter behaviour on each side made those tests fail; every machine- and channel-related suite passes.

## 2026-09-23 — The "who is available" listing now honours directory-sharing agreements, like the search does (`SIGNOFF-REPAIR.5.1.6`)

`REASONBRAID-REPAIR-0448`. Closes the directory-privacy work again (`SIGNOFF-REPAIR.5.1`).

- 🔴 **Before:** two organisations that agreed to share their directories saw each other's agents in more detail in the search than in the "who is available" listing, which ignored the agreement. Nothing was over-shared; the listing showed less than the book promised.
- ✅ **Now:** both use one shared rule, so they always agree: under an agreement, a partner's agents show the organisation-level detail; without one, only the public basics; never the private fields.
- ✅ Tested: the new test failed on the old code and passes now. A deliberately broken rule that over-shares was caught by three tests at once, covering both the search and the listing.

## 2026-09-23 — The directory now lists a partner's agent that runs on the partner's machine (`SIGNOFF-REPAIR.5.3.5.3.1.2`)

`REASONBRAID-REPAIR-0447`.

- 🔴 **Before:** an imported partner agent bound to the partner's own machine could join calls, but the directory search and the "who is available" listing never showed it, because both only listed machines, and it has no machine of its own here.
- ✅ **Now:** both list it, under the importing organisation, showing which machine it runs on. A third organisation looking at the listing sees the agent but is **not** told which partner's machine it runs on, since that would reveal who works with whom. When the partnership ends, the agent disappears from both lists.
- 🔎 **Found and scheduled:** the "who is available" listing never gives a partner organisation the wider view that a directory-sharing agreement promises (the search does). Nothing is over-shared; it shows less than documented. Owned as the next task.
- ✅ Tested: the new checks failed on the old code and pass now; two deliberately broken versions (revealing the machine to everyone; filing the agent under the wrong organisation) were each caught; the profile suite passes (88).

## 2026-09-23 — A partner's agent can be recruited to run on the partner's own machine (`SIGNOFF-REPAIR.5.3.5.3.1.1`)

`REASONBRAID-REPAIR-0446`.

- 🔴 **Before:** when an organisation imported a partner's agent, the imported agent could only act if the importing organisation set up a machine for it themselves. There was no way to say "the partner's agent keeps running on the partner's machine".
- ✅ **Now:** the import can choose `origin`: the agent keeps running on the partner's own machine, and it can join and be seated on calls without the importer enrolling anything. This works only while both organisations' recruitment agreement is in force. If either side withdraws, the agent is immediately treated as having no machine, and it never silently falls back to a local one. The import is refused if the partner's agent has no machine at all.
- ⚠️ **Not yet:** the directory search doesn't list such an agent yet (next step), and sending it actual work waits on a safety change to how a machine checks revocations for two organisations at once. Both are tracked, and the book says so.
- ✅ Tested: the new test failed on the old code and passes now; three deliberately broken versions (ignoring the agreement, looking up the wrong machine, reading the wrong machine's facts) were each caught; the profile, card, federation, audit and upgrade suites pass.

## 2026-09-23 — An agent's owner now sees its whole profile; the privacy defaults are confirmed and explained (`SIGNOFF-REPAIR.5.1.5`)

`REASONBRAID-REPAIR-0445`. Closes the directory-privacy work (`SIGNOFF-REPAIR.5.1`).

- 🔴 **Before:** an agent and its owner were promised "the full profile" but never saw two parts of it: which running instance the agent is, and its own privacy settings. Separately, a code comment claimed every profile field is private unless the agent says otherwise, which was never true, and another comment described the privacy rule backwards.
- ✅ **Now:** the agent and its owner see both parts; nobody else does. The two wrong comments are corrected.
- ⚖️ **A decision taken, and measured:** an agent that never sets privacy settings keeps the current defaults — its name and purpose visible to everyone, its skills visible to its own organisation, sensitive details hidden. Making everything private by default was tested: the agent then became invisible to its own organisation's searches, so nobody could recruit it. The book now explains the defaults and how to override them.
- ✅ Tested: the new test failed on the old code and passes now; the "everything private" version was run and the test caught it; the profile suite (87) and three neighbouring suites pass.

## 2026-09-23 — Unknown facts about an agent no longer count as variety on a panel (`SIGNOFF-REPAIR.5.1.3`)

`REASONBRAID-REPAIR-0444`.

- 🔴 **Before:** when a call closes, the panel is ranked partly on variety — agents on different providers or tools are less likely to fail the same way. But missing information was counted as variety. An agent with no facts at all got the best possible score, an agent could hide a shared provider by not declaring it, a fact nobody declared was reported as "varies across the panel", and a provider named "x" was counted as matching a *tool* named "x".
- ✅ **Now:** only facts that are actually known count, and each is compared only with the same kind of fact. Unknown scores zero, so declaring less can never help. The panel record now says how many agents did not declare each fact, and says "nobody declares this" instead of "varies".
- ✅ The book explains the variety score with worked examples, for the first time.
- ✅ Tested: four new tests (one per problem) failed on the old code and pass now; two deliberately broken versions of the fix were each caught; the full profile suite passes (86).

## 2026-09-23 — A search's ranking weights must be between 0 and 1 (`SIGNOFF-REPAIR.5.1.4`)

`REASONBRAID-REPAIR-0443`.

- 🔴 **Before:** when someone searched the directory for agents, they could say how much each of six factors should count — and any number was accepted. A negative number turned a factor upside down (asking for *diverse* agents returned the *least* diverse first), and two huge numbers made scores infinite, so the ranking fell back to alphabetical order. No error was shown either way.
- ✅ **Now:** each weight must be a number from 0 to 1. Anything else is refused with an error naming the weight. Nothing useful is lost: only the order matters, so any balance between factors still fits in that range (1 and 0.25 means "four times as much").
- ✅ The book now explains the search request, the six factors and their weights — it never had.
- ✅ Tested: the new end-to-end test failed on the old code and passes now; two deliberately broken versions of the check were both caught; the full profile suite passes (86).

## 2026-09-23 — An expired skill endorsement no longer qualifies an agent (`SIGNOFF-REPAIR.5.1.2`)

`REASONBRAID-REPAIR-0442`.

- 🔴 **Before:** a skill in an agent's profile can carry an expiry date, and nothing ever read it. An endorsement that lapsed last year still got the agent found by searches, admitted to calls and seated on panels.
- ✅ **Now:** a skill counts only until its expiry. A search leaves the agent out, a request to join is refused with the date it expired, and the panel is checked again when the call closes, so a skill that lapses between joining and closing does not win a seat. Expiry works exactly like a permission's expiry, so the two never disagree about the boundary moment.
- ✅ Two details handled deliberately: someone who cannot see an agent's skills is never told when one expired, and an agent listing the same skill twice is judged on its best current one.
- ✅ Tested at both levels. Run against a copy that ignores expiry, the new end-to-end test failed; restored, the full profile suite passes.

## 2026-09-23 — The directory-privacy checklist checked against the code: three items already hold, three are real gaps now scheduled (`SIGNOFF-REPAIR.5.1`)

`REASONBRAID-DOC-0149`. A review; no code changed.

- ✅ **Already holds, with the code that does it named:** tests compare organisations whose agents are equally qualified, so a hidden agent is hidden for privacy and not for skill; two simultaneous profile edits or endorsements cannot overwrite each other; an agent at its declared workload limit is shown as busy and is not recruited.
- 🔴 **Three real gaps, each now a task:** a skill whose endorsement has **expired** still counts in a search; an agent that **declares nothing** about which AI provider it runs on is ranked as the most independent choice, the opposite of cautious; and the **ranking weights** a searcher sends are unchecked, so a negative weight can deliberately pick the most look-alike panel.
- ⚠️ Three wording and default problems in the profile's privacy settings, found earlier, are grouped into a fourth task. Whether a new profile should start out hidden needs a decision, because a hidden profile cannot be found by a search. That decision will be made in its own task, with the effect on search measured first.
- Order: expiry first, then the weights, then the missing facts, then the privacy defaults.

## 2026-09-23 — The operator views are now proven served by the real server, not only present in the code (`SIGNOFF-REPAIR.4.6.1.7`)

`REASONBRAID-REPAIR-0416`.

- 🔴 The automatic check I added yesterday only proved each operator view was *written in the code*. Two views live in separate parts that the server must plug in at start-up, and forgetting to plug one in would have gone unnoticed.
- ✅ The server's parts are now assembled by one shared function. A new test assembles the server the same way and calls every operator view. Unplugging the backup view makes that test fail at once, while the old check stayed green, which was exactly the gap.

## 2026-09-23 — This session's findings re-checked against the real running server (DOC-0136)

`REASONBRAID-DOC-0136`. Verification; no code changed.

- At your request, every finding was re-checked in a way different from how it was first produced, mostly by starting the real server and calling it.
- ✅ **Held.** All 8 operator views answer on the running server. An agent really can start only one automatic thread (the second attempt returns the same thread). The backup script prints and stores no password even when the address contains one. All four audit corrections hold. The stored-but-never-checked agent settings really are ignored. The disk-space figure matches to the byte.
- 🔴 **Two things were wrong.**
  - An old review row still said "open" although it had been re-checked the day before. The correction had been written elsewhere but never on the row itself. Now fixed, and one count moves accordingly.
  - I told you 17 commits; it was 16.
- ⚠️ **Three new gaps found, each now tracked.**
  - My new operator-view check proves a route is *written in the code*, not that the running server *serves* it.
  - The working-hours setting accepts any text (for example "never").
  - Two thread views disagree about whether a missing thread exists.

## 2026-09-23 — The pre-wake checks for agents were mostly never built (`SIGNOFF-REPAIR.11.4.7.2.1.5.3.2`)

`REASONBRAID-DOC-0135`. A decision; no code changed.

- The roadmap lists checks a machine should make before waking an agent (is it within working hours? is the tool allowed? is the adapter healthy?), plus limits on an agent's permission to start threads by itself (which topics, who, how often, how deep, how much spend, what side effects).
- 🔴 Most are missing. The machine checks only budget and duplicates. Two settings an agent can declare, **working hours and wake policy, are stored but never checked**. Of the six limits on self-started threads, only spend is part of the permission.
- 🔴 The working-hours check was deliberately passed from one earlier task to the next, and then dropped without comment.
- Split into four tracked tasks. First is the "how often" limit, which will ship together with the fix for the one-thread-per-tenant bug.

## 2026-09-23 — Decided: an imported partner agent can run on a machine you enrol for it today, and on the partner's own machine once three pieces are built (`SIGNOFF-REPAIR.5.3.5.3`)

`REASONBRAID-REPAIR-0441`, with the decision `REASONBRAID-DOC-0148`, taken under your delegation of today. With this, nothing in the plan waits on you.

- ⚖️ **Both, as you suggested — because the roadmap asks for two different things.** *Portable agent cards* means a card can be run elsewhere: you enrol a machine for the imported identity, vouch for its skills yourself, and it works under the permission you gave it. *Remote recruitment* means the partner's own agent is invoked where it lives, executing what your permission allows — the way agent-to-agent federation is done today. Each keeps the rule that a partner never authorises anything here; only your permission does.
- ✅ **The first works now, with no new code.** Tested end to end: an imported agent asked to join a call and was refused for having no machine; after enrolling a machine for it and attesting its skill locally, the same request succeeded and it was seated on the panel, its origin recorded throughout.
- 🔴 **The second needs three things first**, now recorded as tasks in order: recording which machine an imported identity runs on; teaching the machine software to judge each job against the right organisation's revocation counter (today it knows only its own, so it cannot safely work for two); and a receipt on both sides for every job that crosses.
- ⚠️ Left open: whether one imported identity may be bound to two machines at once.

## 2026-09-23 — A permission can now carry conditions, and the first one is a time window (`SIGNOFF-REPAIR.11.4.7.2.1.5.4.3`)

`REASONBRAID-REPAIR-0440`, with the decision `REASONBRAID-DOC-0147`, taken under your delegation of today.

- 🔴 **Before:** the roadmap listed "conditions" among a permission's dimensions and nothing said what a condition was, so the field was deliberately never built.
- ⚖️ **Decided:** conditions are a closed, typed list the server understands completely — the shape modern authorization systems settled on once they stopped writing rules in prose. A kind of condition exists only when the server can refuse a malformed one when the permission is issued, evaluate it at every use from facts it already has, name it in a refusal, and it is tested and documented. Anything less is not a condition.
- ✅ **The first kind:** a daily time window, in the same format and with the same parser as an agent's working hours. A permission with a window only works inside it; outside, the refusal says so and names the time. Rejected for now, each with a measured reason: a "purpose" (nothing declares one), a requesting network (the server records no address), a "human present" check (the next kind, once the check has the facts it needs), and separation of duties (the policy chain's).
- ✅ Tested: unknown kinds, empty lists, malformed windows and a person carrying a condition are all refused; the operator's list shows the condition; a permission whose window is shut is refused with the reason and one whose window is open works. Deliberately making the server ignore a failed condition was caught.

## 2026-09-23 — An agent that is deliberately not being woken now says so: presence gains a seventh state, `held` (`SIGNOFF-REPAIR.11.4.7.2.1.5.3.2.2.1`)

`REASONBRAID-REPAIR-0439`, with the decision `REASONBRAID-DOC-0146`, taken under your delegation of today.

- 🔴 **Before:** an agent whose own settings keep it from being woken — "manual only", or outside its working hours — was handed no work, and still reported itself as *available*. The roadmap's six presence words had no word for it.
- ⚖️ **Decided:** a seventh word, `held`, with the reason beside it (`manual_only`, `off_hours`, or an unreadable settings block). Not `draining`, which means winding down, and not "available with a footnote", because the state is the one field everyone reads and it must not lie. Every presence model that has faced this question answers it with a distinct do-not-disturb state. The roadmap was amended in the same commit so it, the code and the handbook stay aligned.
- ✅ `held` sits below `draining` and above `busy`: a policy is a declaration about the agent, like zero capacity, and unlike the count of what it holds right now. A search does not recruit a held agent unless it asks for held ones.
- ✅ Tested in the unit derivation and end to end through the directory: manual-only → held; off-hours → held; inside hours → not held; zero capacity → draining. Run against the previous code first, the held agent read *offline* — the test's agent had no live lease, which the test now provides.

## 2026-09-23 — A directory search no longer shows another organisation's agents as if the searcher were one of them (`SIGNOFF-REPAIR.5.1.1`)

`REASONBRAID-REPAIR-0438`.

- 🔴 **Before:** when a member of one organisation searched the directory, the server decided once how much that member may see — "a member sees the organisation view" — and applied that to every agent found, including agents of *other* organisations. So a member read a partner's or a stranger's organisation-only fields, and an agent's organisation-only skill could satisfy a search it should have been invisible to. The presence listing had always done this right; the handbook described the search's behaviour as the design.
- ✅ **Now** the search decides per agent, by the searcher's relation to *that agent's* organisation: full or organisation view for its own, organisation view for a partner with a visibility agreement, outsider view for everyone else — and each agent is judged, ranked and shown at that level. An outsider-only skill neither qualifies an agent nor appears.
- ✅ Tested: a member's search does not find a partner agent whose skill is organisation-only; once the agent publishes the skill to the network it is found, shown at the outsider view; with a visibility partnership it is shown at the organisation view. Run against the previous code first, the partner agent was found on a skill the member could not see.
- 🔎 Two of the test failures in that first run were my own mistakes in the test files, both reverted before the green run and recorded as such.

## 2026-09-23 — An agent's machine is now told, on connecting, how many calls are waiting for it (`SIGNOFF-REPAIR.5.3.5.1.1`)

`REASONBRAID-REPAIR-0437`. With this, the whole advertisement of calls — durable, federated, and prompt — is in place; recruiting a partner agent now waits only on your decision about whose machine runs it.

- 🔴 **Before:** an agent could look up the calls offered to it, but nothing told its machine that one was waiting; a machine that did not ask found out late.
- ✅ **Now** every time a machine connects (its regular check-in), the server tells it how many open calls are waiting for its agent — offers still inside their join window that the agent has not yet answered. The reference machine writes that to its log and points at where to read them. An offer never enters the machine's work queue: it is a notice, not a job, because nobody has authorised any work yet.
- ✅ Tested through the real machine client: one open offer → the check-in says one; after the agent answers, and with a closed call and a lapsed one also offered, the check-in says zero. Deliberately breaking the count so answered offers still counted was caught.
- ⚠️ The machine does nothing with the notice beyond saying so; what an agent's adapter should do with an offer is a later, separate design.

## 2026-09-23 — A partner organisation's agent can now ask to join a call, and the organiser gets its card to import (`SIGNOFF-REPAIR.5.3.5.2`)

`REASONBRAID-REPAIR-0436`, with the design record `REASONBRAID-DOC-0145`.

- 🔴 **Before:** an agent from a partner organisation could see a network-wide call it was offered, but its answer was simply refused and nothing recorded that it wanted in. The refusal was right — it has no permission here — but the wish, and the card the organiser would need to act on it, had nowhere to go.
- ✅ **Now** such an agent's "join" is recorded as a **join request** on the call, carrying the agent's own exported card and its fingerprint — the same card it would export by hand — provided the two organisations hold a two-way recruitment partnership (a card crosses only with both operators' consent). The organiser sees the request in the call's inspection and resolves it with the ordinary card import; the imported agent's origin is recorded. The request is never counted as a joiner when the call closes.
- ✅ A foreign agent that was never offered the call, or one that answers anything but "join", hears exactly the old refusal, so a call's existence cannot be discovered by guessing ids. An offered agent without the recruitment partnership is told which partnership is missing.
- ✅ Tested end to end: the refusal naming the missing partnership, the recorded request with its card, the old words for a decline and for an un-offered agent, the close seating only the local joiner, and the import from the request's card. Run against the previous code first, the request was refused with the old words.
- 🔎 Writing the test exposed a rule worth knowing: for a network-wide call, only capabilities an agent publishes to the network count — an agent that keeps them visible to its own organisation only is offered (by interest) and then found ineligible (by capability). That is by design and now written down.
- ⚠️ Still open, waiting on you: an imported agent has no machine here, so it cannot yet take part in the call it asked to join.

## 2026-09-23 — A network-wide call now reaches matching agents in partner organisations (`SIGNOFF-REPAIR.5.3.5.1.2`)

`REASONBRAID-REPAIR-0435`.

- 🔴 **Before:** a call could be opened "for the network", but its offers only ever went to agents in the organiser's own organisation. The partnership that lets a partner see this organisation's directory changed what partners could read, never what they were invited to.
- ✅ **Now** a network-wide call is also offered to matching agents in every organisation that holds a two-way, unexpired visibility partnership with the organiser's — in the same step as the local offers. A call scoped to the organisation, a one-sided or revoked partnership, or a partnership without visibility offers nothing outside, which is the roadmap's rule: cross-organisation recruitment is opt-in, never the default.
- ✅ What a partner agent sees of a foreign offer is the call, its requirements and its window — not the conversation thread, which lives in the other organisation and is named only once a join lands. A partner agent still cannot *answer* a foreign call; recording that wish as a request for the organiser to resolve is the next task.
- ✅ Tested: no partnership → nobody offered; partnership → the partner agent is offered and sees no thread, and its answer is refused; an organisation-scoped call → nobody outside; a revoked partnership → nobody outside. Run against the previous code first, the partner agent was never offered.

## 2026-09-23 — An agent can now see the calls it was offered (`SIGNOFF-REPAIR.5.3.5.1`)

`REASONBRAID-REPAIR-0434`.

- 🔴 **Before:** when a call for participants was opened, the server recorded which agents it was offered to, and that record went nowhere. No agent could ask "what have I been offered?"; the only readers of a call were its organiser and the organisation's administrator. Agents learned of calls out of band.
- ✅ **Now** an agent lists the open calls offered to it, newest last, each with the call's requirements and window and with its own answer if it has given one. Closed calls and calls past their join deadline drop off. A person gets a refusal (calls are offered to agents, not people), and so does an unknown caller, rather than an empty list that could be mistaken for an answer.
- ⚖️ The first idea — pushing the offer into the agent's machine's work queue — was set aside with a reason: that queue holds authorised work the machine executes, and an offer is neither authorised work nor something to execute. The durable record the roadmap asks for is the offer itself, which already survives the machine being offline; what was missing was a way to read it. Telling an online machine promptly that an offer is waiting is the next, separate task.
- ✅ Tested: two matching agents are offered, a third with other interests is not; the offer shows before and after joining; the person and the stranger are refused; the closed call disappears. Run without the new route first, the request was swallowed by the "inspect one call" route and answered "no call named offered".

## 2026-09-23 — Recruiting an agent from a partner organisation was measured before building, and the first thing missing is not about partners at all (`SIGNOFF-REPAIR.5.3.5`)

`REASONBRAID-DOC-0144`. A design census, no code changed.

- 🔴 **A call for participants reaches nobody — in the organisation that opened it or any other.** Opening a call records who was "offered" it, and nothing delivers that offer: no message reaches the agent's machine, and no agent can list the calls it was offered. Agents learn of calls by being told out of band. The roadmap's advertisement of calls to eligible online agents, with durable entries for offline ones, was never built.
- 🔴 The earlier phase that shipped partnerships deferred cross-organisation recruitment "until the call machinery's remote surface exists" — which is the feature itself, so nothing could ever trigger it.
- 🔴 Even after a partner agent's card is imported, the imported agent has no machine here, so it cannot take part in anything. **Whose machine should run an imported agent's work — the partner's, executing what this organisation authorised, or a machine this organisation enrols for it — is a decision for you.** It is recorded as waiting on you.
- 🔴 The directory search shows a partner agent's fields as if the searcher were a member of the partner's organisation; the presence listing gets this right. Owned by the existing visibility task.
- Split into three: deliver offers to agents' machines (in one organisation first, then to partner organisations under the visibility partnership), record a partner agent's wish to join as a request the organiser resolves by importing its card, and the execution question above.

## 2026-09-23 — A partnership can now be given an end date, and a hidden crash on re-accepting a partnership was found and fixed (`SIGNOFF-REPAIR.5.3.4`)

`REASONBRAID-REPAIR-0433`.

- 🔴 **Before:** a partnership between two organisations lasted until someone remembered to revoke it. There was no way to say "for this quarter".
- ✅ **Now** the proposing side may set an end date. A partnership past its end date counts as absent everywhere: it no longer widens what the partner can see, an agent card cannot be imported under it, and the partner cannot accept against it. A date in the past is refused outright. Changing the date resets the direction so the partner accepts again, but it does not change the fingerprint of the terms — an end date is not a term.
- 🔎 **Found by the new test, and fixed:** accepting a partnership a second time against the partner's unchanged terms — after adjusting one's own side and re-accepting — crashed with a database error instead of an answer, because the audit-receipt table refused a second receipt naming the same partner record. That could have happened since these verbs were made transactional; the earlier tests never took that path. A receipt now records each acceptance, so two acceptances are two receipts.
- ✅ Tested: the end date lapsing (the partner falls back to the outsider's view), the refusal to accept against a lapsed partnership, the past-date refusal, and the two receipts. Run against the previous code first, the lapsed partnership still widened the partner's view.
- ⚠️ The upgrade test was hardened as well: it now says out loud when a migration it was written to measure has moved out of its reach, instead of silently testing less.

The entries before those above were rotated into reachable Git history at the
**ninth rotation** (`SIGNOFF-REPAIR.11.4.1.6`, which owns this ledger’s rotation). The exact predecessor — this file as it
stood at the commit named below, which is the object every retired record was
checked against before this notice was written — is:

```bash
git show f87d458dfe574ea75bba6452c043639a35642a31:LIVE_STATUS.md
```

That snapshot is 53139 bytes and 319 lines, and contains 29 dated
entries; its Git blob is `3782a1a11ff245c39a17b55da6aeb67ffcd3c0eb` and its SHA-256 is
`ae9a90abdc062fc4377747b2671da6296eface614d05a4b109f5340d5d5ec6eb`. It carries the eighth rotation's
notice in turn, and each earlier notice names the one before it, so the chain
walks all the way back. `docs/decisions/2026-09-09_changelog-rotation.md` holds
the first transition's evidence.

⛔ **11 record(s) rotated out, 19 kept, lossless** — every retired heading was retrieved from the
predecessor named above before this notice was written, and every figure in it was re-derived from that object with
`git rev-parse`, `git cat-file` and SHA-256 rather than typed. ⭐ The cut is DERIVED, not chosen: it retires whole
records until the ledger has at least 10 commits of runway at the p90 entry size measured over the last
60 non-rotation commits — because two rotations that stopped at the threshold instead left 344 and 296 bytes and
the first forced another rotation on the very next commit (`SIGNOFF-REPAIR.11.4.1.6`).
