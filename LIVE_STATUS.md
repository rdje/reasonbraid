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
| Phase 1 — LAN vertical slice | Mostly Done | Historical demonstration retained in `PHASE-1`; the console review (`.11.1`) is closed with a real-browser control; current authority and demo-script findings remain open. |
| Phase 2 — identity, delivery and recovery | Mostly Done | Historical machinery retained in `PHASE-2`; revocation, fencing, budget and recovery repairs are `.3`–`.4`. |
| Phase 3 — directory and recruitment | Mostly Done | Historical machinery retained in `PHASE-3`; tenant visibility, recruitment and automatic initiation repairs are `.5`. |
| Phase 4 — resources and evidence | Mostly Done | Historical G4 record retained in `PHASE-4`; acquisition isolation, evidence integrity and retention repairs are `.7`. |
| Phase 5 — deliberation and evaluation | Mostly Done | Historical G5 subtraction gate withdrew the quality-lift claim; workflow and evaluation repairs are `.8`. |
| Phase 6 — governance | Mostly Done | Historical G3 machinery exit retained; binding use remains gated; policy/publication/deployment repairs are `.9`. |
| Phase 7 — Internet qualification | Mostly Done | Hardening machinery exists; G6/G7 Internet exposure remains NOT MET. The director greenlit Internet exposure over HTTPS with OAuth on 2026-09-25, security first; owner-only first (the director and their own agents), gated by our own evidence; opening to anyone else still needs an independent threat-model review and penetration test (`docs/runbooks/external-security-review.md`). Local repairs and external threat-model, injection and penetration-test evidence remain required. |
| Phase 8 — federation and interoperability | In Progress | Through regional routing historically recorded; `.5.3` store-and-forward, `.5.4` exit export/import and `.6` G8 remain. Shared authority and protocol gaps are prerequisite repairs. |
| Phase 9 — stable release | Not Started | G9 requires sustained operational evidence and the outstanding release decisions. |
| Corrective review | In Progress | `.1`, `.2.1`, `.2.2`, `.3.1` complete; `.3.2.1` site service passes ten live controls and strict lint. `.3.2.2` operator CLI passes 3 live controls plus adjacent service/ownership checks and strict lint; `.3.2.3` HTTP enforcement passes 8 focused controls; the selected 58-test security run and final 12-test registry run pass. `.3.3.1` core subject serialization passes 49 unit + 3 subject controls and 40 live compatibility tests; `.3.3.2` bound evaluation passes 51 core unit + 3 subject tests, 6 evaluator controls, 32 live authority/command API tests and strict lint; all results consumed and the owned cluster removed. `.3.3.3.1` actual-parent command selection passes 37 live tests, six evaluator controls and strict lint. `.3.3.3.2.1` frozen-read eligibility passes 40 live tests, ten pure evaluator controls and strict lint; all results consumed and the owned cluster removed. `.3.3.3.2.2.1` provenance and strict record/selector decoding passes 51 core units, seven metadata/subject controls, 44 live authority/HTTP/upgrade tests and strict lint; all confirmation/shutdown results consumed, owned cluster removed. Seven HTTP inspection receipt producers pass 45 live authority/API tests, ten pure evaluator controls and strict lint; all results and shutdown consumed. Exact scoped receipt readback passes 18 live authority tests and 30 HTTP tests plus strict lint, including final denied-record readback; all results/shutdown consumed. Actual-parent selection and frozen-read audit/readback `.3.3.3` are complete; The `.3.3.4.1` source census/contract is complete (42 named-call locations plus transitive/mutation coverage). `.3.3.4.2` qualifies the guard/connection owner and migration delivery: 35 controls (34 live / one pure), 28 directory rebuild/cache checks and four-crate strict lint pass, including cancelled-BEGIN replacement and the formerly stale authority executable. Seven acceptance-checker controls also pass, correcting nested-evidence owner selection while preserving real-tree box refusals. All results/shutdown consumed; owned clusters/probes removed. Grant error classification `.3.3.4.3.1` passes 56 live authority/HTTP/card controls and focused strict lint: storage causes survive, malformed active data returns safe 500, genuine structural 400s and exact failure/recovery effects are verified. All results/shutdown consumed; four owned clusters absent. Standalone writer/status integration `.3.3.4.3.2` now passes 85 selected controls (84 live / one pure), focused strict lint and book checks: five guarded paths, live parent issuance, preserved malformed status, public commit uncertainty and exact race/failure/recovery controls. The old contention fixture now verifies the observed target→guard dependency. All results/shutdown consumed; three owned clusters absent. Typed rollback support `.3.3.4.3.3.1` passes 89 selected controls (88 live / one pure), focused strict lint and rendered book checks: domain errors roll back provisional rows/new anchors, existing anchors remain, deferred anchor faults cannot replace refusals, and SQL causes/deadlines/commit uncertainty survive. All results/shutdown consumed; both owned clusters absent. Complete enrollment `.3.3.4.3.3.2` passes 97 selected controls (96 live / one pure), final focused strict lint and rendered book checks: one exclusive transaction before replay through every write, database-time liveness, honest concurrent replay, complete rollback and preserved commit phase. All results/shutdown consumed; three owned clusters absent. Bootstrap uncertainty `.3.3.4.3.3.3.1` is now runtime-confirmed: a 500 commit-uncertain response can precede the original committed tenant, and a repeated no-key request creates a distinct tenant. All 25 selected controls (24 live / one pure), focused strict lint and book checks pass; all results/shutdown consumed, owned cluster absent. Server bootstrap recovery `.3.3.4.3.3.3.2` passes 73 selected controls (72 live / one pure), the final eleven-control fixture rerun, three-crate all-target strict lint and final fixture lint. Canonical RequestId, complete guarded outcomes, exact conflicts, concurrent rollback/replay, actual commit/response-loss recovery, malformed-storage refusal and one shared deadline are qualified. All results/shutdown are consumed and four owned clusters are absent. StateFile storage `.3.3.4.3.3.3.3.1.1` passes twelve selected controls, all-target CLI strict lint and rendered book checks. Linked-target overwrite, changed open-reader snapshots and ignored process exclusion are repaired; strict bounded preservation and synchronized replacement are qualified on the native macOS repository volume. All results are consumed and unique fixtures are absent. Whole CLI writer integration `.3.3.4.3.3.3.3.1.2` passes nineteen selected controls, final all-target CLI strict lint and book checks: pre-dispatch exclusion, fresh actor/merge, both overlap orders, process-loss release and preserved real-server flow/denials. All results/shutdown are consumed; unique fixtures and the owned cluster are absent. Recovery schema `.3.3.4.3.3.3.3.2.1` passes twenty-four selected controls, all-target CLI strict lint and book checks: strict pending/completed records, preserved legacy wire shape, guarded multi-publication continuity and zero-dispatch refusal while pending. All results are consumed and unique fixtures absent; the interrupted pre-main launch and unchanged-binary successful retry are preserved honestly. Keyed CLI/explicit recovery `.3.3.4.3.3.3.3.2.2` passes thirty-three selected controls, final output-window rerun and strict CLI lint: durable matching request before dispatch, original-byte complete reply checks, same-key failure recovery, historical local receipt recovery without HTTP and deliberate distinct fresh tenant creation. The actual unread-output interruption retains its original result; all results/shutdown consumed, unique fixtures and owned cluster absent. Completion-capacity preflight `.3.3.4.3.3.3.3.2.3.1` passes thirty-one selected controls, final fresh/pending/exact-fit matrix and strict CLI lint: an oversized completion refuses before HTTP with exact snapshot/pending preservation, while the exact-limit case succeeds. All results consumed and unique fixtures absent; sizing samples never become outcome evidence and physical disk reservation is not claimed. Bounded HTTP waits and restart qualification follow; ordinary no-key behavior remains intentionally distinct. The final administrative effect REPRESENTATION `.3.3.4.7.1` passes 10 representation controls within 68 core tests, strict core lint and the falsified object-only control; its census derives the closed fourteen-operation set from 27 guarded-admission call sites. Its STORAGE `.3.3.4.7.2` then passes 8 live controls (falsified 5/3 against three attributable injections): migration 0058 keyed on the admission with a composite tenant-binding foreign key, a writer on the caller's already-guarded transaction, and a tenant-filtered reader. `.3.3.4.8` is the first producer of an effect record; `.3.3.4.9` is the second, putting spend-breaker arm/reset onto one exclusive-guard transaction — 25 live controls, the affected set 5 suites / 94 tests, falsified 17/8 against the exact pre-`.9` handlers.. Fixture ownership `.11.2.1.1` censuses all 61 `create_dir_all` sites in tracked Rust with 0 unresolved and repairs the 3 that adopted: exclusive per-call creation on the repository volume, falsified against an occupied path, with 89 selected controls across five suites (63 of them `profiles` on a disposable cluster), strict lint for both crates and the 25-check gate green. |

## 2026-09-28 — Try it live: `make showcase` (`SHOWCASE.1`)

`REASONBRAID-SHOWCASE-0001`.

- ✨ **New:** one command starts a throwaway ReasonBraid on your machine with two agents running, and opens a page at http://127.0.0.1:4320/. Ask a question and watch both agents answer; see what is fixed and what is still open, generated from the project itself; copy the matching command-line steps; and leave feedback, which is saved for the next working session to act on.
- ⚠️ **Limit:** the agents give scripted answers; no AI model is connected to them yet. Everything around them is the real system. Ctrl-C stops it all and cleans up.

## 2026-09-28 — Recovering a lost machine's work now asks the operator to accept that it may run twice (`SIGNOFF-REPAIR.11.52`)

`REASONBRAID-REPAIR-0561`.

- 🔴 **Before:** when a machine was lost mid-task, the recovery steps re-ran its work as if it had never started. Measured: the AI provider was called on the lost machine and again on its replacement, with no one accepting that risk, and the first call's cost was never counted.
- ✅ **Now:** the server refuses the plain re-run for work a lost machine received, and the operator re-runs it with an explicit "this may run twice" authorization and a reason. The re-run is paid separately, and the first call's budget stays held. The recovery guide and the command reference say so.

## 2026-09-28 — Closing a recruitment call now checks the initiator's permission, not just its name (`SIGNOFF-REPAIR.11.47`)

`REASONBRAID-REPAIR-0560`.

- 🔴 **Before:** the person who opened a call could close it on their name alone. Measured: with their permission revoked, they still closed it and fixed its panel.
- ✅ **Now:** closing re-checks the same permission that opening needed, so a revoked initiator cannot close, and neither can someone who holds that permission but did not open the call. The guide says so.

## 2026-09-28 — An imported agent card can no longer claim skills nobody certified (`SIGNOFF-REPAIR.11.45`)

`REASONBRAID-REPAIR-0559`.

- 🔴 **Before:** a card brought in from a partner tenant kept whatever trust level its claims stated. Measured: a card edited to say `certified` was imported as certified, and the directory ranks by that level. The test meant to check card versions also passed with that check deleted.
- ✅ **Now:** imported claims start at the lowest level, as a role's own do, and the importing tenant's owner can vouch for them. The test now fails if the version check is removed. The guide corrects a sentence that said the origin keeps a record of its cards; it does not.

## 2026-09-28 — The guide now says exactly which machine a recorded run is credited to (`SIGNOFF-REPAIR.11.48`)

`REASONBRAID-REPAIR-0558`.

- 🔴 **Before:** the guide said each recorded run names the machine identity that ran it. Measured: it names the identity current when the result arrived, which differs if the machine re-enrolled in between.
- ✅ **Now:** the guide says what is recorded and why; naming the exact one needs the node to report it, which is planned for when something relies on it. The guide also now says that results the server cannot use are kept but change nothing, and that the server trusts a node's reported usage as its tenant's own.

## 2026-09-28 — Approving a policy change is now all-or-nothing, and two approvals cannot both win (`SIGNOFF-REPAIR.11.55.1`)

`REASONBRAID-REPAIR-0557`.

- 🔴 **Before:** an approval was saved in two separate steps without a lock. Measured: two approvals of the same proposal sent at once were both recorded; if the second step failed, the approval stayed recorded while the proposal did not move; and a database failure was reported as "this approval already exists".
- ✅ **Now:** an approval is one step that locks its proposal: a second one at the same time is refused, a failure records nothing, and a database failure is reported as the server's. The guide says so.

## 2026-09-28 — The publication check now notices a moved "effective" pointer (`SIGNOFF-REPAIR.11.56`)

`REASONBRAID-REPAIR-0556`.

- 🔴 **Before:** the tool that checks published policy against its repository read the pointer that says which publication is in effect, and never compared it with anything. Measured: with the pointer moved or deleted by hand, it reported everything consistent.
- ✅ **Now:** for the newest publication in a repository, a missing or moved pointer is reported for a person to repair. Older publications, which a newer one has replaced, are not flagged for the pointer having moved on. The guide explains both.

## 2026-09-28 — A revoked boundary now stops the policy work done under it (`SIGNOFF-REPAIR.11.55`)

`REASONBRAID-REPAIR-0555`.

- 🔴 **Before:** after a tenant's owner revoked an enrollment boundary, a grant issued under it could still approve, correct, deploy or publish policy, and keep its policies resolving. Measured: an approval made right after the revocation was accepted and recorded.
- ✅ **Now:** a grant counts only while its boundary is active and inside its validity period, as everywhere else. The guide says so, and a sentence that still called a settled question open is corrected.
- 🔍 **Found:** an approval is written in two separate steps with no lock, so two at once, or a failure in between, can leave the record inconsistent. That is next.

## 2026-09-28 — A redirect can no longer lead the server to a private address written in IPv6 (`SIGNOFF-REPAIR.11.50`)

`REASONBRAID-REPAIR-0554`.

- 🔴 **Before:** when a git source redirected to an address written in IPv6 form, such as `[::1]` or the IPv4-in-IPv6 spelling of a private address, the server followed it without checking where it led. Measured: the redirect reached a service on the server's own machine. A redirect could also move to another port, and some reserved IPv6 ranges counted as public.
- ✅ **Now:** every redirect is checked however its address is written, keeps the scheme and port it started on, and only ordinary public IPv6 addresses are reachable. The guide describes the rule.

## 2026-09-28 — The last five review notes of this batch checked: revoked authority still approves policy (`SIGNOFF-REPAIR.11.9.1.4.3`)

`REASONBRAID-DOC-0196`.

- 🔍 **Checked:** 20 points from five review notes; 10 were already fixed or deliberately decided, and 7 were added to open work.
- 🔴 **Found:** revoking an enrollment boundary does not stop the grants under it from approving, correcting, deploying or publishing policy, although the guide says a revoked boundary freezes the next administrative write. And the tool that checks published policy against the repository never compares the "effective" pointer, so a moved one goes unnoticed. The guide's qualification page lists both, and a limitation already repaired months ago is now marked repaired.

## 2026-09-28 — Seven review notes on fetching and extraction checked: two more problems (`SIGNOFF-REPAIR.11.9.1.4.2`)

`REASONBRAID-DOC-0195`.

- 🔍 **Checked:** 33 points from seven review notes about fetching sources and extracting their text; 14 were already fixed or owned by open work, and 13 were added to open work that already covers them.
- 🔴 **Found:** evidence can be stored under any retention label, and one the expiry sweep does not know is kept for ever, while the guide says evidence expires by its label. And the text-extraction pack says it reads RSS feeds but refuses every one. The guide's qualification page lists both.

## 2026-09-28 — Ten more old review notes checked: six problems to fix, two to watch (`SIGNOFF-REPAIR.11.9.1.4.1`)

`REASONBRAID-DOC-0194`.

- 🔍 **Checked:** 51 points from ten review notes about recruitment, agent cards, node results and quotas; 24 were already fixed.
- 🔴 **Found, to fix first:** a redirect to an IPv6 address gets past the check that keeps the server from reaching private networks. Also: an imported agent card keeps whatever trust level it claims; the view other tenants see, which the guide calls a pseudonym, shows the real role id; a call's initiator can close it without its permission being checked again; a node's result can be filed against the wrong machine record; and after a machine is lost its unfinished work is replayed without the "this may run twice" authorization. The guide's qualification page now lists each, and reopens the two areas it had marked complete.
- ⏸️ **Found, to watch:** refused quota requests are all kept, and the git pack's walk of its own files skips a file it cannot read. Neither matters until someone the owner does not control can use the system.

## 2026-09-28 — Every code change needs its task record, including deletions, migrations and hooks (`SIGNOFF-REPAIR.11.40`)

`REASONBRAID-REPAIR-0553`.

- 🔴 **Before:** the check that every code change is owned by a task in the project's task tree missed four kinds of change: deleting code, renaming it, changing a database migration, and changing a git hook. Any of them could land with no task record. No past commit did.
- ✅ **Now:** the check counts every kind of change, and it and the acceptance check share one definition of what code is. There is no longer a way to skip it. The book's qualification page says so.

## 2026-09-28 — A changed answer to a recruitment call shows when it was given (`SIGNOFF-REPAIR.11.42`)

`REASONBRAID-REPAIR-0552`.

- 🔴 **Before:** when a participant changed its answer to a recruitment call, the call's record showed the new answer with the time of the old one.
- ✅ **Now:** the record shows the current answer and when it was given. The guide says that a new answer replaces the old one, and that closing a call picks a panel but invites no one.

## 2026-09-28 — The roadmap no longer carries a progress report that goes stale (`SIGNOFF-REPAIR.11.43`)

`REASONBRAID-REPAIR-0551`.

- 🔴 **Before:** the roadmap opened with a hand-written progress report, weeks out of date; it said work was still to come that had finished days earlier.
- ✅ **Now:** the roadmap states the plan and points to where progress is kept current (the live status and the book's qualification page, which records where today's build differs from the plan). A check refuses any commit that puts a progress narrative back into it. The book says the same.

## 2026-09-28 — Submitted evidence must state its true size (`SIGNOFF-REPAIR.11.41`)

`REASONBRAID-REPAIR-0550`.

- 🔴 **Before:** evidence submitted to the server could claim any file size, although the guide says the recorded size is the size of the stored file; and re-submitting the same file with the right size could not correct it.
- ✅ **Now:** a submission whose stated size is not its real size is refused, naming both numbers.

## 2026-09-28 — Another batch of old review notes checked, and two small record errors found (`SIGNOFF-REPAIR.11.9.1.3.5`)

`REASONBRAID-DOC-0193`.

- 🔍 **Checked:** 17 points from four review notes; most were already fixed, several by the evidence repairs of the last two days. Two live problems: submitted evidence can claim any file size (the guide says it is the stored size), and when a participant changes its answer to a recruitment call, the record keeps the time of its first answer. Both block the release and have their own tasks.

## 2026-09-28 — Invitations check who is invited and for how long (`SIGNOFF-REPAIR.11.39`)

`REASONBRAID-REPAIR-0549`.

- 🔴 **Before:** an invitation to a discussion accepted any well-formed participant ID, including another organisation's or one that does not exist (who could never join), and any expiry time: a huge one crashed the request, and a negative one created an invitation that had already expired.
- ✅ **Now:** only a participant of the discussion's own organisation can be invited, and the expiry must be between one second and a year. Both refusals say what was wrong.

## 2026-09-26 — A batch of old review notes checked against today's code (`SIGNOFF-REPAIR.11.9.1.3.4`)

`REASONBRAID-DOC-0191`.

- 🔍 **Checked:** 23 points from four old review notes. Most were already fixed. Two real issues surfaced: the check that every code change has a task misses deleted files, database migrations, scripts and git hooks (a check that can wrongly pass, so it blocks the release), and inviting a participant accepts a nonsensical expiry time or a participant from another organisation (who still cannot join). Each has its own task now.

## 2026-09-26 — The voting guide's "not built yet" list is true to the code (`SIGNOFF-REPAIR.8.1.1.6`)

`DOC-0190`.

- ✅ **Checked:** four voting refinements (stepping aside, replacing a voter, tracking which model version voted, changing a vote) are not built, and the guide says so and describes what happens instead. Each of those descriptions was checked against the code and holds; nothing needed fixing. The four are recorded as future work with a named trigger.

## 2026-09-26 — Two voting rules stay switched off until they are designed (`SIGNOFF-REPAIR.8.1.1.4`)

`REASONBRAID-REPAIR-0548`.

- ✅ **Decided:** "weighted by role" and "committee" approval stay refused when a discussion is created, as the guide already says, because neither has a design yet (how weights are written, who the committee is). The refusal now points at the task that will design them, which starts when the roadmap schedules them or an organisation asks for one.

## 2026-09-26 — The list of what still blocks the release is now counted by a tool, and a claim of mine was wrong (`SIGNOFF-REPAIR.12.2`)

`REASONBRAID-REPAIR-0547`.

- 🔴 **Correction:** the entry below this one said no documentation-honesty problems remained open. That was wrong: seven do. I had counted by hand from one kind of status line, and those seven record their state another way.
- ✅ **Now:** a tool counts every open task the way the release rule needs (60 open, 13 of them blocking), and every commit is refused if an open task has no severity. Four unclassified tasks were classified on the triggers they already named.

## 2026-09-26 — The agent-to-agent module now records what it loses (`SIGNOFF-REPAIR.6.3.1`)

`REASONBRAID-REPAIR-0546`.

- 🔴 **Before:** the agent-to-agent (A2A) module recorded "nothing was lost in translation" while its description said everything is, two of its tests passed on that contradiction, and its replies dropped the other side's task number.
- ✅ **Now:** every translation records all five kinds of loss, the tests check each one, and replies carry the task number. No problems of the data-loss, cross-organisation or false-documentation kinds remain open; what is left is checks and CI work.

## 2026-09-26 — What is left of the agent-to-agent module, measured (`SIGNOFF-REPAIR.6.3`)

`REASONBRAID-DOC-0187`.

- 🔍 **Found:** nothing in the product uses the agent-to-agent (A2A) module yet, and the qualification page already says it only converts message formats. Inside the module, it records "nothing was lost in translation" while its own description says everything is, a test passes on that contradiction, and replies drop the external task number.
- 📋 **Next:** fix the record, the test and the replies. Connecting real outside agents waits until the product first accepts their messages.

## 2026-09-26 — Re-collecting evidence now counts as fresh again, per organisation (`SIGNOFF-REPAIR.7.4.9`)

`REASONBRAID-REPAIR-0545`.

- 🔴 **Before:** the guide said collecting the same evidence again renews its "fresh until" date, but nothing was renewed, so evidence stayed on the "needs re-collecting" list. And because two organisations citing the same document share one record, the first one to collect it decided the other's date.
- ✅ **Now:** each organisation has its own date for the evidence it cites, re-collecting renews it, and no organisation's choice changes another's. Existing records kept their dates when the database was upgraded.

## 2026-09-26 — The tool recovers from being killed at any moment of its own saves (`SIGNOFF-REPAIR.3.3.4.3.3.3.3.3.3`)

`REASONBRAID-REPAIR-0544`.

- ✅ **Checked:** the tool was killed outright at seven exact moments between its own saves, including half-way through writing its file. Every time, running the same command again finished the job correctly: the same enrollment and no duplicate. Nothing needed fixing.
- ✅ **Closed:** this finishes the enrollment-recovery work and five parent tasks with it. The only kind of blocking issue that could lose or corrupt data now has no open items left.

## 2026-09-26 — A killed tool's leftover helper no longer gets a misleading error (`SIGNOFF-REPAIR.3.3.4.3.3.3.3.3.2`)

`REASONBRAID-REPAIR-0543`.

- ✅ **Checked:** if a program using the tool is killed while a process it started still holds the tool's lock, other runs are refused at once, nothing is damaged, and everything works again once that process ends, including a pending enrollment.
- 🔴 **Fixed:** the refusal said to wait for "another writer" to finish, but in this case there is no writer left to finish. It now names the real holder and says what to do; the guide explains how to find and end it safely.

## 2026-09-26 — Enrollment recovery survives a server restart (`SIGNOFF-REPAIR.3.3.4.3.3.3.3.3.1`)

`REASONBRAID-REPAIR-0542`.

- ✅ **Checked:** if the server restarts after an enrollment whose answer was lost, retrying fails while the server is down (keeping its key), then returns the original person and organisation once it is back, with no duplicate. Nothing needed fixing.
- 📝 **Corrected:** the guide said write failures were untested; they are tested at every step of a save. It now lists exactly what is and is not tested.

## 2026-09-26 — What is left of testing the tool's recovery under interruption, measured (`SIGNOFF-REPAIR.3.3.4.3.3.3.3.3`)

`REASONBRAID-DOC-0186`.

- 🔍 **Found:** six of the nine situations this task names are already tested. Three are not: the server restarting between a lost answer and the retry, the tool being killed while a child process still holds its lock, and a kill at the moments between the tool's own saves.
- 📋 **Next:** one sub-task each, in that order.

## 2026-09-26 — The command-line guide describes the recovery the tool has (`SIGNOFF-REPAIR.3.3.4.3.3.3.3.2.4.2`)

`REASONBRAID-REPAIR-0541`.

- 🔴 **Before:** the guide said the tool could not recover a lost enrollment ("not implemented yet"), which it has done since an earlier repair; another chapter said the same, and said an enrolled person gets nine admin rights where the code grants fourteen.
- ✅ **Now:** the chapters describe what the tool does, with the output and refusals it actually prints, and a test checks every example against the code that reads the real files. This finishes the request-recovery task; its testing under interruption (a killed process, a restarted server) is next.

## 2026-09-26 — The command-line tool checks the server's reply before saving it (`SIGNOFF-REPAIR.3.3.4.3.3.3.3.2.4.1`)

`REASONBRAID-REPAIR-0540`.

- 🔴 **Before:** when enrolling someone into an existing organisation or creating a thread, the tool saved what the server answered without checking it. A reply naming a different organisation was saved as the person's organisation, and a reply missing an ID produced an error blaming the tool's own files.
- ✅ **Now:** a reply that does not answer the request is refused as the server's error and nothing is saved. Checked against a real server, and by deliberately breaking the check 14 ways (every way that compiles was caught).

## 2026-09-26 — What is left of the command-line tool's enrollment recovery, measured (`SIGNOFF-REPAIR.3.3.4.3.3.3.3.2.4`)

`REASONBRAID-DOC-0185`.

- 🔍 **Found:** the task list expected old recovery code to remove, and there is none. What is left: the command-line guide still says the tool cannot recover a lost enrollment, which it has done for weeks; ordinary enrollment and thread creation save the server's reply without checking it against the request.
- 📋 **Next:** the reply check first, then the guide. A smaller display issue (a missing list printed as "0") is deferred, with its trigger named in the qualification chapter.

## 2026-09-26 — Every malformed request now gets a reason code (`SIGNOFF-REPAIR.11.36`)

`REASONBRAID-REPAIR-0539`.

- 🔴 **Before:** on 52 of the server's routes, a request whose body was not valid for the route got back a plain sentence with no reason code, although the error reference promises every refusal a code a program can act on.
- ✅ **Now:** every such refusal carries the code `invalid_command` and a message naming the problem field, on both the main API and the node channel. The status numbers stay as they were: 422 means the body has the wrong shape, 400 that it is not JSON at all.

## 2026-09-26 — Tests no longer leave empty folders behind, and the scripts review is complete (`SIGNOFF-REPAIR.11.3.6`)

`REASONBRAID-REPAIR-0538`.

- 🔴 **Before:** every successful run of the backup test left an empty folder behind (46 had piled up).
- ✅ **Now:** a successful run cleans up after itself and the test checks that it did; a failed run keeps its folder for inspection. The 46 empty folders are gone. This completes the review of the operational scripts (`.11.3`): backup, restore, development environment, demonstration and load test.

## 2026-09-26 — The demonstration's "this did not happen" checks can no longer pass by accident (`SIGNOFF-REPAIR.11.3.7`)

`REASONBRAID-REPAIR-0537`.

- 🔴 **Before:** three of the demonstration's checks that something did NOT happen would also pass if the tool doing the checking failed, and the responses it keeps as evidence were kept without noticing when a request had failed.
- ✅ **Now:** each of those checks first requires the tool to have worked, a failed request in the evidence is reported as a failure, and the check that the restarted server accepts a node requires a genuine acceptance.

## 2026-09-26 — The load test measures exactly what it was asked to (`SIGNOFF-REPAIR.11.3.4`)

`REASONBRAID-REPAIR-0536`.

- 🔴 **Before:** the load test could run more requests than asked (10 became 16), reported success when asked to run none, could hang forever on one stuck request, and mixed failed requests into its speed figures, so fast failures made the system look faster.
- ✅ **Now:** it runs exactly the number asked (or refuses), gives up on a request after a set time, and reports speed only over requests that succeeded, with failures listed beside them. The speed figures published on 2026-09-08 were not affected.

## 2026-09-26 — The development environment checks its own server and cleans up honestly (`SIGNOFF-REPAIR.11.3.3`)

`REASONBRAID-REPAIR-0535`.

- 🔴 **Before:** if another copy of the development environment was already running, a self-check reported success by talking to the other copy; the cleanup deleted the database folder even when the database had not stopped, and said all was well; and it could not be started from outside the project folder.
- ✅ **Now:** it checks that it is talking to its own server, deletes the database folder only once the database has stopped (and says so plainly if it cannot), and works from any folder.
- 🔎 **Found and tracked:** some of the demonstration's "this did not happen" checks would pass even if the tool running them failed (`.11.3.7`).

## 2026-09-26 — The demonstration checks it is talking to its own server, and its two-host mode works (`SIGNOFF-REPAIR.11.3.5`)

`REASONBRAID-REPAIR-0534`.

- 🔴 **Before:** if another program already used the demonstration's port, the demonstration carried on talking to it without noticing; it copied the database address, user included, into the evidence it produces for sharing; and its documented two-host mode could never have worked (paths were quoted in a way that broke them, the node program was looked for in the wrong folder, and a check looked at the wrong machine).
- ✅ **Now:** the demonstration stops at once if its own server did not start, the evidence shows the address without the user, and the two-host mode passes every check in a same-machine rehearsal (not yet between two real machines).

## 2026-09-26 — A backup is never half-written and never readable by others (`SIGNOFF-REPAIR.11.3.1`)

`REASONBRAID-REPAIR-0533`.

- 🔴 **Before:** an interrupted backup left a partial file under the name of a real one; the backup, a plaintext copy of the whole database, was readable by other users; and the database password was visible on the backup tool's command line.
- ✅ **Now:** a backup appears under its name only once it is complete and never replaces another, only its owner can read it, and the password is passed privately.

## 2026-09-26 — The restore test can no longer overwrite the live database (`SIGNOFF-REPAIR.11.3.2`)

`REASONBRAID-REPAIR-0532`.

- 🔴 **Before:** the restore test wiped and replaced whatever database it was pointed at, including the live one, and passed the database password on its command line, where other users of the machine could read it.
- ✅ **Now:** it refuses the live database by name and any database that already holds data, before touching anything, and it hands the connection details to the database tools privately rather than on the command line.

## 2026-09-26 — The operational scripts reviewed: six problems to fix (`SIGNOFF-REPAIR.11.3`)

`REASONBRAID-DOC-0183`.

- 🔎 **Found:** the restore script will wipe whatever database it is pointed at, the live one included; the database password is visible to other users on the machine while the backup, restore, load and demo scripts run, and the demo copies it into its shareable evidence; a backup that is interrupted leaves a partial file that looks complete; the development and load scripts can report success while talking to a different program on the same port; the load test runs more requests than asked and mixes failures into its speed figures; and one test leaves an empty folder behind on every run (34 so far).
- ✅ **Next:** each script gets its own fix, the restore script first.

## 2026-09-26 — The review page no longer lists problems that were already fixed (`SIGNOFF-REPAIR.11.37`)

`REASONBRAID-REPAIR-0531`.

- 🔴 **Before:** the book's review page, where a reader looks for what is still wrong, listed 24 problems whose fixes had already landed, some of them weeks earlier, and in two places contradicted its own summary table.
- ✅ **Now:** every one of the 24 was checked against the code. 21 now say what was fixed and how things work today, one says it was never a problem, one explains which half was fixed and which was a deliberate decision, and one small remaining gap got its own tracked item. An automatic check now refuses to let the page keep an open problem once all its fixes are done.

## 2026-09-26 — Deployment records no longer blame database failures on the caller (`SIGNOFF-REPAIR.9.3.3.6`)

`REASONBRAID-REPAIR-0530`.

- 🔴 **Before:** when the database failed while registering a target, assigning a policy, or recording drift, a correction or an outcome, the answer said the record "does not exist", the authority "is not active", or the record "already exists". Someone could re-create or abandon a request that was fine. A badly written expiry date was reported as missing.
- ✅ **Now:** a database failure is reported as the server's problem; "already exists" means exactly that; a badly written date is named as such. The deployment review (`.9.3.3`) is closed.

## 2026-09-26 — A drift record compares against what was actually assigned (`SIGNOFF-REPAIR.9.3.3.5`)

`REASONBRAID-REPAIR-0529`.

- 🔴 **Before:** a drift record (the note that a target is not running what was published) stored whatever "should be running" value the person recording it typed, so it could disagree with the assignment it was about.
- ✅ **Now:** that value must be the assignment's own, or the record is refused with a message naming it. What the target was seen running is still the observer's report.

## 2026-09-26 — Only a target's own authority can decide what it runs (`SIGNOFF-REPAIR.9.3.3.7`)

`REASONBRAID-REPAIR-0528`.

- 🔴 **Before:** assigning a published policy to a target only checked that the policy belonged to the person assigning it. Targets are shared across the whole site, so one organization could set what another organization's target should run.
- ✅ **Now:** only whoever holds the target's own authority can assign to it; anyone else is refused as unauthorized.
- 🔎 **Found and fixed:** the review page still listed a problem with policy corrections that was fixed weeks ago. It is corrected, and 10 more entries on that page that may be similarly out of date are tracked (`.11.37`).

## 2026-09-26 — The book no longer implies deployments roll out in stages (`SIGNOFF-REPAIR.9.3.3.4`)

`REASONBRAID-DOC-0182`.

- 🔴 **Before:** the book said a publication is assigned to a target in a "canary wave", which reads as a staged rollout where a failing first group stops the rest. Nothing does that: the wave number is only stored, and the system does not carry out deployments at all yet.
- ✅ **Now:** the book says the wave is a label you choose to group targets, and that nothing holds a later wave back. Rollout ordering is tracked for when the system first carries out a deployment itself.
- 🔎 **Found and tracked:** assigning a publication to a target checks no authority over that target (`.9.3.3.7`, next).

## 2026-09-26 — A target's reports are kept, not overwritten (`SIGNOFF-REPAIR.9.3.3.3`)

`REASONBRAID-REPAIR-0527`.

- 🔴 **Before:** each time a target reported which policy content it was running, the new report replaced the old one, so what it said before, and who said it, was lost.
- ✅ **Now:** every report is kept as its own record that cannot be edited afterwards, the target's current state is always its latest report, and the full history can be read in order. Reports that arrived before this change had already been lost.

## 2026-09-25 — Only the principal a target names can report what it runs (`SIGNOFF-REPAIR.9.3.3.2`)

`REASONBRAID-REPAIR-0526`.

- 🔴 **Before:** a receipt, the report of which policy content a target is actually running, could be filed by anyone in the owning organization, including the operator who had just said what it *should* run. The comparison that detects a target running the wrong thing was only as honest as whoever posted last.
- ✅ **Now:** a target names one reporter when it is registered (a person or an agent role), and only that principal can file its receipts; anyone else is refused with a message saying so. A target registered before this change takes no receipts until it is given a reporter, which is not possible yet and is tracked for when it is needed.

## 2026-09-25 — A deployment must deploy what its publication published (`SIGNOFF-REPAIR.9.3.3.1`)

`REASONBRAID-REPAIR-0525`.

- 🔴 **Before:** when someone assigned a published policy to a target, they also typed in which version and which content fingerprint the target should run, and the system only checked that the fingerprint looked like one. A target could be told to run content the publication never had, and later comparisons of "what should be running" against "what is running" used that made-up value.
- ✅ **Now:** the fingerprint must be the publication's own and the version one the publication recorded; anything else is refused with a message naming which one is wrong. The book shows where to read both.
- 🔎 **Found and tracked:** a drift record (the note that a target is not running what was published) still accepts a typed-in fingerprint (`.9.3.3.5`), and the deployment and drift records report a database failure as "does not exist" or "already exists" in 14 places (`.9.3.3.6`).

## 2026-09-25 — A commit check no longer mistakes a setting's name for a waiver (`SIGNOFF-REPAIR.11.2.10`)

`REASONBRAID-REPAIR-0524`.

- 🔴 **Before:** one of the automatic checks that runs on every commit treated the name of a setting, `REPEATED_WAIVER_THRESHOLD`, as if someone had written "this check doesn't apply to me", and refused a correct commit.
- ✅ **Now:** the check only reacts to the words it was built for, and it tests itself on every commit against thirteen known examples, so the same kind of mistake cannot come back unnoticed.

## 2026-09-25 — A policy can be reviewed more than once (`SIGNOFF-REPAIR.9.3.2`)

`REASONBRAID-REPAIR-0523`.

- 🔴 **Before:** once a policy had been reviewed for a given reason, it could never be reviewed for that reason again: the system silently failed to create the next review. The same silent failure made a database problem look like "nothing needs review", and a single exception granted to a policy (even an expired one) counted as a "repeated" exception.
- ✅ **Now:** each new problem after a completed review schedules a new review; only one review per policy and reason is open at a time; "repeated exceptions" means at least two still in force within 90 days; and a failure to create a review is reported as an error.

## 2026-09-25 — A database failure is no longer reported as "that record does not exist" (`SIGNOFF-REPAIR.9.2.3`)

`REASONBRAID-REPAIR-0522`.

- 🔴 **Before:** when the database failed while the system was recording a policy proposal, decision, approval or publication, the answer said the record the request named "does not exist", or that the approval's proof was invalid. Someone reading that could re-create the record or give up on it, when nothing was wrong with their request.
- ✅ **Now:** a database failure is reported as the server's problem on all of these, and a missing decision is called a decision (it said "proposal"). The policy publication review (`.9.2`) is closed.

## 2026-09-25 — A policy publication can only be finished once (`SIGNOFF-REPAIR.9.2.2`)

`REASONBRAID-REPAIR-0521`.

- 🔴 **Before:** a policy publication ends either "in force" or "failed". If both answers arrived at the same moment, both were accepted, the second overwrote the first, and the record could end up "in force" while still carrying the failure's reason. Both senders were told they had succeeded.
- ✅ **Now:** the first to arrive wins and the other is refused with a message saying what state it found. A missing publication is now called a publication in the error (it said "proposal"), and a database failure is reported as the server's problem rather than as "does not exist".
- 🔎 **Found and tracked:** the same "database failure reported as does not exist" mistake in 14 places of the proposal, decision and approval records (`.9.2.3`, next).

## 2026-09-25 — A refused conversation leaves no trace in the routing log (`SIGNOFF-REPAIR.8.2.7`)

`REASONBRAID-REPAIR-0520`.

- 🔴 **Before:** when a new conversation was routed to a working style automatically, the routing log was written before the system checked whether the request was allowed. A refused request left a log entry for a conversation that never existed, and re-sending a successful request added a duplicate entry.
- ✅ **Now:** the entry is written as part of creating the conversation, after the permission check, so it exists exactly when the conversation does. The evaluation and routing review (`.8.2`) is closed.

## 2026-09-25 — An experiment can no longer list the same option twice, and its errors say what is wrong (`SIGNOFF-REPAIR.8.2.6`)

`REASONBRAID-REPAIR-0519`.

- 🔴 **Before:** a routing experiment that listed an option twice gave that option a double share of the test cases, silently; a test case listed twice was stored twice. Separately, most of the evaluation tool's error messages were garbled into a sentence about a malformed fingerprint ("digest `the arms are empty` is not a 64-hex string").
- ✅ **Now:** both repeats are refused with a message naming the repeated item, and every refusal of the evaluation tool says plainly what is wrong.
- 🔎 **Tracked, not urgent:** if a stored baseline were ever damaged, the error would be reported as the user's mistake instead of a server fault; this cannot happen today because baselines are checked when saved (`.8.2.8`).

## 2026-09-25 — Changing an assessment is refused instead of silently ignored (`SIGNOFF-REPAIR.7.4.8`)

`REASONBRAID-REPAIR-0518`.

- 🔴 **Before:** sending an assessment again with a different quote or explanation was answered "accepted", with the first one's id, and the change was thrown away. In a conversation, the record even showed the new quote while the stored assessment kept the old one.
- ✅ **Now:** sending the same assessment again unchanged still returns the same one; sending it with anything changed is refused, and the message lists exactly what differs. Nothing is half-applied.

## 2026-09-25 — An assessment's author is whoever submitted it (`SIGNOFF-REPAIR.7.4.7`)

`REASONBRAID-REPAIR-0517`.

- 🔴 **Before:** when someone recorded that a piece of evidence supports (or contradicts) a claim, the request itself said who the author was, and who had verified it. Anyone could put someone else's name on it.
- ✅ **Now:** the server records the person who actually sent it as the author, and a request that tries to name an author or a verifier is refused with a message saying which field is not allowed.
- 🔎 **Found on the way:** a second person verifying an assessment, which the roadmap describes, does not exist yet (tracked, `.7.4.13`); and most routes answer a badly formed request without the error code the book promises (tracked and stated in the book, `.11.36`).

## 2026-09-25 — The two-host demonstration passes again (`SIGNOFF-REPAIR.4.4.2.2.1`)

`REASONBRAID-REPAIR-0516`.

- 🔴 **Before:** since the web console's inbox panel was fixed on 2026-09-24, the demonstration's check of the console still looked for the panel's old, broken query and failed. Nothing noticed because everyday test runs skip the demonstration, but the next publication to GitHub would have failed its automated checks on it.
- ✅ **Now:** the check looks for the corrected query and says what it actually checks, and the whole demonstration passes. The book's description of that step was corrected the same way.

## 2026-09-25 — Deleted evidence can no longer be built on, and fetching it again makes fresh evidence (`SIGNOFF-REPAIR.7.4.6`)

`REASONBRAID-REPAIR-0515`.

- 🔴 **Before:** when an operator or the retention sweep deleted a piece of evidence (it is kept, marked deleted, with a reason), three things still used it: new summaries could be filed against it, assessments could be checked against its text (in a conversation too), and fetching the same content again quietly pointed back at the deleted record.
- ✅ **Now:** the first two are refused with a message naming the deletion and its reason, and fetching the same content again creates a fresh record with its own history and retention period. The deleted record stays readable, as before.
- ⚖️ **Decided and recorded:** a deletion retires one fetched copy, not the content itself. Otherwise a routine expiry would become a permanent ban. Blocking content wherever it appears is a separate feature that does not exist yet (`.7.4.10`), and the book says so.

## 2026-09-25 — The web console no longer shows a late answer under the wrong view (`SIGNOFF-REPAIR.11.1.2`)

`REASONBRAID-REPAIR-0514`.

- 🔴 **Before:** each view cleared the page, fetched, then drew wherever the page was by then. On a slow server, clicking Timeline and then Audit could put the timeline's rows under the Audit heading, and saving a new identity could show the previous tenant's conversations under it.
- ✅ **Now:** each view draws into its own space, which a later click replaces, so a late answer is dropped. The presence and inbox panels keep only the answer to the latest click. Four new browser checks hold one answer back on purpose and confirm it changes nothing when it arrives; each failed on the old page, and six deliberately broken versions of the repair are each caught.
- ✅ **The console review (`.11.1`) is closed.**

The entries before those above were rotated into reachable Git history at the
**eighteenth rotation** (`SIGNOFF-REPAIR.11.4.1.6`, which owns this ledger’s rotation). The exact predecessor — this file as it
stood at the commit named below, which is the object every retired record was
checked against before this notice was written — is:

```bash
git show 6f8f0d8bbb7368b405ce371d929b6724ea86ce48:LIVE_STATUS.md
```

That snapshot is 53865 bytes and 477 lines, and contains 59 dated
entries; its Git blob is `7f476a7ecaf23271231f5d34162f7eb9689d3c38` and its SHA-256 is
`a89f94a042abb97ad58baf94c9aee5401bb1ec1963552463c00d18f5ec8375e1`. It carries the seventeenth rotation's
notice in turn, and each earlier notice names the one before it, so the chain
walks all the way back. `docs/decisions/2026-09-09_changelog-rotation.md` holds
the first transition's evidence.

⛔ **11 record(s) rotated out, 49 kept, lossless** — every retired heading was retrieved from the
predecessor named above before this notice was written, and every figure in it was re-derived from that object with
`git rev-parse`, `git cat-file` and SHA-256 rather than typed. ⭐ The cut is DERIVED, not chosen: it retires whole
records until the ledger has at least 10 commits of runway at the p90 entry size measured over the last
60 non-rotation commits — because two rotations that stopped at the threshold instead left 344 and 296 bytes and
the first forced another rotation on the very next commit (`SIGNOFF-REPAIR.11.4.1.6`).
