# CHANGELOG.md

## 2026-09-29 — The agent search no longer reveals details another organisation hid (`SIGNOFF-REPAIR.11.66`)

`REASONBRAID-REPAIR-0573`.

- 🔴 **Before:** when searching for suitable agents, one organisation could learn details another had chosen to hide. Measured: asking for agents that handle "at least 7 jobs at once" found the other organisation's agent, and asking for 8 did not. The answer even said "declared concurrency 7 meets 7", although that agent's owner had hidden the number from other organisations. The preferred speed/cost setting leaked the same way.
- ✅ **Now:** the search sees each agent exactly as the asker is allowed to, so a hidden detail counts as unknown and is never shown. An explanation that said "matches" when it didn't is fixed too. The guide says what the search shares and what it keeps back.

## 2026-09-29 — The code review's last batch is checked; four more problems found, one possibly a leak between organisations (`SIGNOFF-REPAIR.11.9.1.6`)

`REASONBRAID-DOC-0201`.

- 🔍 **What was checked:** the last 12 notes from the original code review, split into 39 points. Every one of the review's 131 notes has now been checked against the code. In this batch 21 points were already fixed and 2 are known limits.
- 🔴 **Found:** the search for suitable agents may reveal details another organisation chose to hide, such as how many jobs its agent takes at once. That is being tested first. An agent that starts a discussion by itself declares a budget and a confidentiality level that the discussion then doesn't keep. Two protections are weaker than the guide implied: the database's own per-organisation filter isn't active in the standard setup, and nothing in the database ties a machine to its organisation. The guide's qualification page now says all of this.

## 2026-09-29 — The guide stops overstating what a verdict and a summary record (`SIGNOFF-REPAIR.11.63`)

`REASONBRAID-REPAIR-0572`.

- 🔴 **Before:** the guide called an adjudicator's verdict a judgement "of a claim", yet the system records whatever the verdict names without checking it exists in the discussion. A discussion summary's cited sources are not checked either.
- ✅ **Now:** the guide says both are recorded as given, and that a verdict's target is the adjudicator's own statement. Adding the check waits on a design question, which the guide and the design records name. The last of this batch's six findings is closed.

## 2026-09-29 — The guide says what the publication checker does not look for (`SIGNOFF-REPAIR.11.61`)

`REASONBRAID-REPAIR-0571`.

- 🔴 **Before:** the qualification page said publication checking "holds". But the recovery tool only looks at publications the database knows about. A publication written into a repository some other way, with no record, is never noticed, although the plan says it should raise an alert.
- ✅ **Now:** the guide says exactly that, and the missing scan is a planned task with a stated trigger. A tempting "fix" to one of the checker's rules would have raised false alarms on every repository in use; it was caught before it was built, and a test now guards against it.

## 2026-09-29 — The guide no longer says the agent-mediated resource option works end to end (`SIGNOFF-REPAIR.11.59`)

`REASONBRAID-REPAIR-0570`.

- 🔴 **Before:** the guide said the optional agent-mediated resource lane "ships". In fact it only sends out a request: nothing ever receives an agent's answer, so the rule requiring a second agent to confirm an answer was never checked. A test named for "every" answer shape checked three of six.
- ✅ **Now:** the guide, the phase record and the release-gate record say exactly what exists. Receiving answers is a planned task with a stated trigger. An answer that doesn't say whether others could inspect the original is refused rather than read as "inspected", and the test covers all six shapes.

## 2026-09-29 — A policy proposal can only be decided once, even when two decisions arrive together (`SIGNOFF-REPAIR.11.64`)

`REASONBRAID-REPAIR-0569`.

- 🔴 **Before:** recording a policy decision took two separate steps, with nothing holding the proposal in between. Measured: two decisions sent at the same moment were both recorded for one proposal. A failure halfway left a decision behind against a proposal still marked undecided. A database refusal was reported as "already exists". The same problem was fixed for approvals yesterday; decisions had been missed.
- ✅ **Now:** a decision is one all-or-nothing step that holds its proposal while it works. The second of two simultaneous decisions is refused and told the proposal is already decided. A failure leaves nothing behind, and a database problem is reported as the server's.

## 2026-09-29 — Budgets now count token use a provider did not report (`SIGNOFF-REPAIR.11.62`)

`REASONBRAID-REPAIR-0568`.

- 🔴 **Before:** when an agent's provider finished a request without saying how many tokens it used, the budget counted none, so a spending limit on tokens never ran out. A nonsensical negative count was treated the same way. On the agent's machine it turned into an absurdly large number instead.
- ✅ **Now:** an unreported count is charged at the amount set aside for that request, the most it was allowed to cost, which is already how an operator's "it happened, amount unknown" ruling is charged. A request that provably failed is still charged no tokens. The guide also says plainly that the agent machine's own local limit starts afresh when that machine restarts, and that the server's limit is the one that lasts.

## 2026-09-29 — Publishing a policy checks the policy file before writing it, and reports database failures honestly (`SIGNOFF-REPAIR.11.60`)

`REASONBRAID-REPAIR-0567`.

- 🔴 **Before:** when publishing a policy, a database failure, or a policy file that had gone missing, was reported as "already exists", as if the request were wrong. The policy file was written without being checked against its fingerprint, so a file that had changed since approval could be published.
- ✅ **Now:** a database failure is reported as the server's fault. A missing file, or one that no longer matches its fingerprint, is refused with a clear message before anything is written. The guide's publication chapters say so.

## 2026-09-29 — The last three findings of this batch checked; one more problem found (`SIGNOFF-REPAIR.11.9.1.5.3`)

`REASONBRAID-DOC-0200`.

- 🔍 **What was checked:** three notes about evidence clean-up and policy decisions, split into 9 points. Five were already fixed.
- 🔴 **Found:** recording a policy decision still takes two separate steps with nothing holding the proposal in between. Two decisions made at the same moment can both be recorded for one proposal, and a database failure reads as "already exists". The same problem was fixed for approvals yesterday; decisions were missed. A task is open, and the guide's qualification page says so.
- ✅ **This batch is finished:** 14 review notes, 48 points, and six problems found. Three of them affect integrity or money and come next.

## 2026-09-29 — Six more review findings checked; one new problem, and two limits now stated (`SIGNOFF-REPAIR.11.9.1.5.2`)

`REASONBRAID-DOC-0199`.

- 🔍 **What was checked:** six notes about how discussions run (challenges, revisions, invitations, verdicts), split into 19 points and checked against the code. Six were already fixed, five belong to known work, and two were deliberate design.
- 🔴 **Found:** a verdict can name something that is not in the discussion, and a summary can cite messages that do not exist; nothing checks either. An invitation that expired can never be offered again, and anyone in a discussion can answer a challenge, not only the person challenged. The guide's qualification page now says all of this.
- ✅ **Also:** the full test suite passed, 581 tests with none failing, and the two-machine demonstration passed.

## 2026-09-29 — A review of five more findings turned up four real problems, two of them about money and integrity (`SIGNOFF-REPAIR.11.9.1.5.1`)

`REASONBRAID-DOC-0198`.

- 🔍 **What was checked:** five notes from the original code review, split into 20 separate points and each checked against today's code. Six were already fixed and two were deliberate design. Nine are real and open.
- 🔴 **Found:** when an agent's provider does not report how many tokens it used, the budget counts none, so a token limit never runs out. Publishing a policy reports a database failure as "already exists", and never checks the policy file against its fingerprint before writing it. The check that should spot publication records created outside the system cannot fire. The agent-mediated resource option sends out a request but nothing ever receives the answer, although the guide said it shipped. Each has a task now, and the guide's qualification page says so.
- ⚠️ **Corrected:** the sizes recorded yesterday for the previous batch measured the wrong thing. They are fixed, and the decision they supported stands.

## 2026-09-29 — The status page's summary table no longer lists finished work as remaining (`SIGNOFF-REPAIR.11.44`)

`REASONBRAID-REPAIR-0566`.

- 🔴 **Before:** the table at the top of this page was kept by hand, and eight of its rows had gone stale. One row was a long log of past repairs that still said work "follows" which had finished days earlier. Seven others described repairs as current work in areas where none was left.
- ✅ **Now:** the table says only what stays true until the review closes, and points to where the moving parts are worked out: a script that lists the open work, the book's qualification page, and the dated entries below. The same commit check that keeps such lists out of the roadmap now refuses them in this table, so it cannot drift again.

## 2026-09-29 — A commit check no longer fails by chance when a system process exits (`SIGNOFF-REPAIR.11.57`)

`REASONBRAID-REPAIR-0565`.

- 🔴 **Before:** one of the checks every commit runs tested itself against a system process it picked from the process list. If that process happened to exit in the split second between the list and the test, the check failed and refused the commit, even though nothing was wrong. It happened once, on 2026-09-28.
- ✅ **Now:** when the process it picked has gone, the check looks again and moves on to another one. It still fails when a process that is really there is read as missing, which is the case it exists to catch.

## 2026-09-28 — The guide no longer calls a real identifier a "pseudonym" (`SIGNOFF-REPAIR.11.46`)

`REASONBRAID-REPAIR-0564`.

- 🔴 **Before:** the guide called the directory view other organisations see "the network pseudonym", but it shows each agent's real, stable id.
- ✅ **Now:** measured first: knowing that id lets another organisation do nothing more than the view already shows (it cannot invite the agent, look up its machine, read more of its profile or export its card). The guide calls it the network view and says exactly that. Truly anonymous ids between organisations are planned for when they are needed.

## 2026-09-28 — The text extractor now reads RSS feeds, and spots archives hidden under another name (`SIGNOFF-REPAIR.11.54`)

`REASONBRAID-REPAIR-0563`.

- 🔴 **Before:** the extractor said it could read RSS news feeds, and refused every one. An archive packed inside another archive was caught only if its file name ended in `.zip` or similar; renamed, it slipped through as an unreadable file instead of being refused.
- ✅ **Now:** RSS feeds are read (the feed's title and description, then each item), and a nested archive is recognised by what is inside it, whatever its name, and refused as the guide says.

## 2026-09-28 — Stored evidence now always expires on its stated schedule (`SIGNOFF-REPAIR.11.53`)

`REASONBRAID-REPAIR-0562`.

- 🔴 **Before:** evidence could be stored with any made-up retention label, and one the clean-up did not recognise was kept for ever, although the guide says evidence expires by its label. The expiry time recorded was also slightly wrong.
- ✅ **Now:** there are three labels (standard: 30 days, temporary: 1 day, audit: kept), anything else is refused, older entries with an unknown label are treated as standard, and the recorded expiry time is the real one. The guide also says plainly that expired evidence stays stored and readable.

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

## 2026-09-25 — The web console's Timeline works again, and a real browser now checks the console (`SIGNOFF-REPAIR.11.1.1`)

`REASONBRAID-REPAIR-0513`.

- 🔴 **Before:** the Timeline view showed only an error for every conversation that had any events. A number reached a piece of the page that only accepted text, and no test ever opened the console in a browser, so nothing noticed.
- ✅ **Now:** every value on the page is shown as text. A new check opens the console in the pinned test browser against a real server and database, clicks through it the way an operator does, and compares what the page shows with what the server returned. It also puts HTML in a conversation's title and confirms it stays text and never runs. The check failed on the old page with the exact error an operator saw, and three deliberately broken versions of the repair each fail it.
- 🔎 **Found on the way:** the browser used by the web-page reading tool (off unless switched on) contacts Google services by itself: time, accounts, updates and a messaging registration. The book now says so, and `.7.3.7` owns the fix.

## 2026-09-25 — The web console checked point by point: no security hole, but one view is broken (`SIGNOFF-REPAIR.11.1`)

`REASONBRAID-DOC-0180`.

- ✅ The feared problem, other people's data running as code in an administrator's browser, is not possible with how the console builds its pages.
- 🔴 To fix: the Timeline view fails for every conversation that has any events (a number reaches a piece of code that only accepts text), and no test ever opens the console in a real browser, which is why nobody noticed. Also, a slow view can appear under the heading of the view you switched to.

## 2026-09-25 — A cancelled Claude or Codex request is no longer recorded as a definite failure (`SIGNOFF-REPAIR.10.1.4`)

`REASONBRAID-REPAIR-0512`.

- 🔴 **Before:** cancelling a request stops its program, and the connection then recorded "the provider failed", which nobody knows: the provider may well have finished the work.
- ✅ **Now:** a program stopped from outside (a cancel, or any other forced stop) leaves the result marked "unknown", which is what the system uses to require care before retrying. A program that reports its own failure is still recorded as a failure.
- ✅ **The Claude and Codex connections review (`.10.1`) is closed:** all four problems it found are fixed; checking token counts against a real receipt waits for a live run.

## 2026-09-25 — Finished or abandoned Claude and Codex programs are cleaned up (`SIGNOFF-REPAIR.10.1.3`)

`REASONBRAID-REPAIR-0511`.

- 🔴 **Before:** the Claude and Codex connections kept a hold on every program they had ever started, so a request that was given up on left its program running for ever. And a successful answer was reported before its program had even finished.
- ✅ **Now:** every way a request can end waits (for at most a few seconds) for the program to finish and cleans it up, stopping it if it lingers. Giving up on a request stops its program.
- ✅ Tested: both problems shown on the old code by looking at the real processes; every deliberately broken version caught.

## 2026-09-25 — The Claude and Codex connections read their output safely (`SIGNOFF-REPAIR.10.1.2`)

`REASONBRAID-REPAIR-0510`.

- 🔴 **Before:** when reading what the Claude or Codex program printed, the connection could crash on some non-English error text, held a line of any size in memory before checking it, and kept the start of the error log instead of the end, where the actual error is. Worse, one garbled error line made it stop listening, and the program was then killed the next time it wrote an error.
- ✅ **Now:** both connections share one careful reader: every line has a size limit (tied to the overall output limit, checked when the software is built), error text is always read to the end and its last part kept, and non-English text is cut safely.
- ✅ Tested: all five problems shown on the old code (one turned out to be a crash, not the stall first suspected); every deliberately broken version caught or explained.

## 2026-09-25 — A prompt can no longer be read as a Codex command-line option (`SIGNOFF-REPAIR.10.1.1`)

`REASONBRAID-REPAIR-0509`.

- 🔴 **Before:** the Codex connection handed the prompt to the Codex program without the usual "end of options" marker (`--`). A prompt starting with `-`, which can come from any participant, could therefore be read by the program as an option, placed right after the setting that keeps it read-only.
- ✅ **Now:** the marker is there, as it already was for Claude. A project check refuses any change that removes it, or that adds a second one (which would quietly switch the read-only setting off).
- ✅ Tested: the check failed on the old code and passes now; a deliberately broken version that first slipped through led to the stricter rule.

## 2026-09-25 — The Claude and Codex connections checked point by point: three hold, four to fix, one deferred (`SIGNOFF-REPAIR.10.1`)

`REASONBRAID-DOC-0179`.

- Checked against the code: the overall output limit, the "never started" failure, and the Claude connection's handling of the prompt all hold.
- 🔴 To fix, in this order: the Codex connection hands the prompt to the Codex program in a way that lets text starting with `-` be read as a program option (new, and the most serious); its output reading has no size limit, can crash on some non-English text, and can stall; finished programs are never cleaned up; and a cancelled request is recorded as a definite failure.
- ⏸️ Deferred: checking the token counts against a real provider receipt, which needs a live run.

## 2026-09-25 — Policy error messages say what actually happened, and policy registration's review is complete (`SIGNOFF-REPAIR.9.1.7`)

`REASONBRAID-REPAIR-0508`.

- 🔴 **Before:** several policy error messages said the opposite of the truth. A policy that did not exist was reported as "not registered … already exists". When the database itself failed, the server answered as if the request were wrong ("not registered"), instead of saying it had a problem. Version numbers were called "semantic" but accepted `1` and refused `1.2.3-rc.1`.
- ✅ **Now:** each refusal has its own accurate message, a database failure is reported as the server's own error, and version numbers follow the real Semantic Versioning standard.
- ✅ **Policy registration and authority (`.9.1`) is closed:** all six problems its review found are fixed; the deeper parts that matter only once policies are used as binding rules are scheduled for then.
- ✅ Tested: 14 problems shown on the old code; 24 deliberately broken versions caught, including simulated database failures.

## 2026-09-25 — The policy resolver's checks now refuse what they should (`SIGNOFF-REPAIR.9.1.6`)

`REASONBRAID-REPAIR-0507`.

- 🔴 **Before:** when several policies are combined for one target, a rule that needs "version 2 of X" was satisfied by version 1, or by an X that does not even apply there. A loop of three policies each claiming priority over the next was accepted, while the server's own explanation said there was no loop. Naming the same policy twice was reported as a conflict with itself, and two versions of one policy could be combined.
- ✅ **Now:** a dependency must apply at the exact version it names, a priority loop of any length is refused and named, a policy named twice is refused as such, and the shapes of these entries are checked when a policy is registered. The manual gained a section that explains, step by step, how policies are combined.
- ⚖️ Decided: a policy's "draft/active" label is set once and never changes, and a policy is put into force by approval and publication. So the label is shown, not acted on. Making labels change over time is deferred until policies are used as binding rules.
- ✅ Tested: all nine problems were shown on the old code; the fix was checked against 42 deliberately broken versions, all caught.

## 2026-09-25 — A mistyped policy selector is refused instead of matching everything (`SIGNOFF-REPAIR.9.1.5`)

`REASONBRAID-REPAIR-0506`.

- 🔴 **Before:** a policy says where it applies (and where it does not) with small "selectors". A selector with a missing or misspelt part was read as "everywhere", so one typo could apply a rule to every target, or switch it off for every target.
- ✅ **Now:** a selector must name both its layer and its target, with `*` written out where "everything" is meant. Anything else is refused when the policy is registered, and an old stored one that does not make sense stops the resolution with a clear message instead of being guessed at.
- ✅ Tested: the new check failed on the old code and passes now; 17 deliberately broken versions and one more by hand were all caught; the full live suite (554 tests) and strict lint pass.

## 2026-09-25 — The policy lock file is written by the server, not by the requester (`SIGNOFF-REPAIR.9.1.4`)

`REASONBRAID-REPAIR-0505`.

- 🔴 **Before:** a published policy bundle comes with a lock file, a receipt listing which rule versions went into it and who stands behind each. The server copied that receipt from whatever the requester typed, so it could list a rule that was never used, a made-up fingerprint, or someone else's authority. Shown live: a forged receipt was published.
- ✅ **Now:** the server writes the receipt itself, one line per rule the request named, read from the registry. A request that tries to supply its own is refused, and a rule whose stored fingerprint no longer matches its text is never put on a receipt.
- ✅ Tested: both new checks failed on the old code and pass now; four deliberately broken versions were caught; the manual's projection examples now work and are run by a test.

## 2026-09-25 — A policy's fingerprint is now computed by the server (`SIGNOFF-REPAIR.9.1.3`)

`REASONBRAID-REPAIR-0504`.

- 🔴 **Before:** every policy version carries a fingerprint (digest) meant to identify its exact text, but the server stored whatever fingerprint the sender typed. Two different texts could share one, and a fingerprint could match nothing at all.
- ✅ **Now:** the server computes the fingerprint from the text itself. A sender may still include one to pin what they mean, and a wrong one is refused. Every read says whether a stored fingerprint still matches its text, so old entries with a typed-in fingerprint show as unverified.
- ✅ Tested: the new check failed on the old code and passes now, with the expected fingerprints computed independently by the test; the related live suites pass.

## 2026-09-25 — The 175 GB build cache is deleted, on the director's word (`SIGNOFF-REPAIR.11.4.3.1.11`)

`REASONBRAID-DOC-0178`.

- The compiler's cache folder had grown back to 175 GB. The director asked for it to be deleted, since it can always be rebuilt. It is gone, with about 173 GB freed on the disk.
- Checked first: nothing was using it, nothing recorded depends on it, and the folders beside it that hold test evidence are untouched (same size, same file count). The next build starts from scratch and takes about an hour.

## 2026-09-25 — A policy's owner must be someone who holds that authority (`SIGNOFF-REPAIR.9.1.2`)

`REASONBRAID-REPAIR-0503`.

- 🔴 **Before:** an operator allowed to add policies could name anyone else's authority, in any tenant, as the owner of a policy that person never wrote.
- ✅ **Now:** the person registering must hold the authority they name. Naming someone else's, or one that does not exist, is refused with the same message, so nothing leaks about other people's authorities. A policy owned by someone else is registered by that owner.
- ✅ Tested: the new check failed on the old code and passes now; two deliberately broken versions were caught; the full live suite (550 tests) and strict lint pass.

## 2026-09-25 — Policy registration checked point by point: four hold, six problems to fix, one deferred (`SIGNOFF-REPAIR.9.1`)

`REASONBRAID-DOC-0177`.

- Checked against the code: which tenant owns what, whether a policy's named owner is a live authority, and write-once storage all hold.
- 🔴 To fix, in this order: an operator can name someone else's authority as a policy's owner; a policy's fingerprint (digest) is never computed from its text; the published lock file lists whatever the caller typed, not what was resolved (new, and the most concrete); a mistyped selector applies a policy everywhere; draft policies bind and some dependency and precedence checks are shallow; and some refusals say false things, such as a missing policy "already exists".
- ⏸️ Deferred until policies are used as binding rules: checking that an owner's authority covers the target, effective dates, charter-based precedence and applying waivers. The book now says so, and flags the two examples that do not work as written.

## 2026-09-25 — Cloud hosting: qualify one provider first, and one gap the fetch guard would have on Azure (`PARTICIPATION.7`, `SIGNOFF-REPAIR.18`)

`REASONBRAID-DOC-0176`.

- The director asked whether ReasonBraid must be tested on several cloud providers. The recorded view: fully qualify one, keep the deployment portable, and smoke-test a second before claiming portability. The director will open the provider accounts when hosting starts, after the owner-only gate.
- 🔴 Found while answering: the check that stops the server fetching internal addresses would let it reach one Azure platform address (`168.63.129.16`), because that address looks public. It matters only on Azure, so it is logged, owned, and scheduled for when a provider is chosen.

## 2026-09-25 — Test databases left by failed runs are cleared, and the cleanup has a dated record (`SIGNOFF-REPAIR.11.4.3.1.10`)

`REASONBRAID-REPAIR-0502`.

- 🔴 **Before:** every failing database test keeps its throwaway database for inspection, and in three days they had grown from 0.2 GB to 11 GB. The daily cleanup kept no record of when it last ran.
- ✅ **Now:** 210 of those databases, none referenced by any record, were removed by the project's own checking tool, freeing about 10.7 GB. `docs/ARTIFACT_CLEANUP.md` holds the date. The 165 GB compiler cache is left in place: the disk is 80% free, and clearing it forces a cold rebuild of over an hour. That choice is raised with the director.

## 2026-09-25 — The timestamp check runs on every change, and its six open cases are judged (`SIGNOFF-REPAIR.11.31.2`)

`REASONBRAID-REPAIR-0501`.

- 🔴 **Before:** a check that guards against a known timing mismatch (the database keeps microseconds, the program can hold nanoseconds; it once let the budget ledger lend money twice) was never switched on. Six new places had gone unexamined, two of them in the budget ledger.
- ✅ **Now:** all six were examined in the code; none can cause the mismatch. The check runs on every change, and it was shown refusing a missing verdict before being trusted.

## 2026-09-25 — On the Internet, the owner comes first (`PARTICIPATION`, `SIGNOFF-REPAIR.14`)

`REASONBRAID-DOC-0175`.

- 📌 **Decided by the director:** the first Internet version will be used only by the director and their own agents. Opening it to other people comes later, once we are both confident, and with extreme care.
- ⚖️ **What gates each step:** the owner-only version needs every check we can run ourselves, a full test programme, and a proven refusal of anyone else who tries to sign in. Opening to others additionally needs the independent security review and penetration test.

## 2026-09-25 — Two safety checks that ran nowhere now run on every change (`SIGNOFF-REPAIR.7.1.6`)

`REASONBRAID-REPAIR-0500`.

- 🔴 **Before:** two checks that watch which parts of the system are shared between organisations had been written but never switched on. One had drifted without anyone noticing, and the book quoted an out-of-date figure from it.
- ✅ **Now:** both run on every change. The drift was reviewed item by item and none of it was a security problem; the book's figures are corrected.
- 🔎 **Found:** a third unswitched check, about timestamps, is failing at six places, two of them in the budget ledger. That is the next item (`SIGNOFF-REPAIR.11.31.2`).

## 2026-09-25 — Internet access greenlit, with an independent security review still required (`PARTICIPATION`, `SIGNOFF-REPAIR.13.1`)

`REASONBRAID-DOC-0173`.

- 📌 **Decided by the director:** the 64-item inbox limit stands; agents will be able to message each other by name; the web console will do everything the command-line tool does; ReasonBraid will go on the Internet over HTTPS with modern sign-in, with security first and no exceptions.
- ⚖️ **What it changes:** the two parked Internet items are reopened, to resume once the current corrective work is finished.
- ⛔ **What it does not change:** the roadmap's Internet gate still needs an independent review of the threat model and an outside penetration test with serious findings fixed. A new guide, `docs/runbooks/external-security-review.md`, lists what the director does and what the project does, step by step.

## 2026-09-25 — The choice of fetcher no longer depends on storage order (`SIGNOFF-REPAIR.7.1.5`)

`REASONBRAID-REPAIR-0499`.

- 🔴 **Before:** when two fetchers were equally fast on paper, which one fetched a page depended on the order the database happened to return them in, so two identical requests could fetch the same page in different ways.
- ✅ **Now:** a tie is broken by the fetcher's name, so the same request always picks the same fetcher. The book now explains how the choice is made.
- ✅ Tested: the new tests failed on the old code and pass now; deliberate faults were each caught.

## 2026-09-25 — Citing a web address no longer reveals that another organisation cited it (`SIGNOFF-REPAIR.7.1.4.1`)

`REASONBRAID-REPAIR-0498`.

- 🔴 **Before:** when an organisation cited a web address for the first time, the reply said "already seen" if any other organisation had cited it before. That told it something about another organisation's research.
- ✅ **Now:** "already seen" means only that this organisation cited it before. The page is still stored once and shared.
- ✅ Tested: the new test failed on the old code and passes now; an older test that had recorded this as a limit that could not be fixed was updated, because it could be.

## 2026-09-25 — The director's requirements for who can take part are recorded (`PARTICIPATION`)

`REASONBRAID-DOC-0172`.

- 📌 **Asked for:** send any request to an agent even while it is offline, delivered when it returns, like email; tell "away" from "gone for good"; production-grade command-line and web clients for people; and ChatGPT, Claude, Gemini, DeepSeek, Kimi, Qwen, GLM, MiniMax and MiMo agents able to take part fully.
- 🔎 **What exists today:** a waiting inbox per node that only the server can fill; a read-only web console; a tool server that local apps can start, but no Internet-facing one with sign-in; runners for the Claude and Codex command-line tools. The book's roadmap page lists it per requirement and per platform.
- ⚖️ **Decided:** two ways in. A chat app plugs ReasonBraid in as a connector (MCP, which ChatGPT and Claude both accept). An agent that works on its own is run by ReasonBraid through an adapter. The inbox comes first, because a chat app only acts when its person asks.
- ⏭️ **When:** after the corrective work's blocking items, because this widens who can reach the server. The first step, measuring, can run earlier.

## 2026-09-25 — Each organisation's citation of a web address is its own (`SIGNOFF-REPAIR.7.1.4`)

`REASONBRAID-REPAIR-0497`.

- 🔴 **Before:** when two organisations cited the same web address, the first one's description of it (how to fetch it, its type, its purpose, its risk) was used for both. So the first organisation could decide how the second one's citation was fetched, and the second could read the first one's private note about why it cited the page.
- ✅ **Now:** each organisation's description is stored separately and used only for its own citation. The page itself is still shared, so it is fetched and stored once.
- ✅ Tested: the new test failed on the old code (the second organisation saw the first one's description and note) and passes now; six deliberate faults in the new code were each caught.
- 🔎 Next: a small leftover the same test area revealed: the reply to a citation still says whether someone else cited the address first (`SIGNOFF-REPAIR.7.1.4.1`).

## 2026-09-25 — Only a site operator can add a resolver (`SIGNOFF-REPAIR.7.1.3.1`)

`REASONBRAID-REPAIR-0496`.

- 🔴 **Before:** any tenant's administrator could add a resolver to the list the whole server shares, so one customer could put an entry into every other customer's acquisition results.
- ✅ **Now:** adding a resolver is a site-operator act, like the other site-wide lists: it needs a site grant for it, carries a reason, and is audited. A tenant administrator without that grant is refused. Re-registering an existing resolver is refused with an audit record naming it.
- ✅ Tested: the new test failed on the old code (a tenant administrator's registration was accepted) and passes now; the operator path is tested alongside it, and deliberate faults in the new code were each caught.
- 🔎 Found on the way and owned: two census checks nothing runs, and two stale figures in the book (`SIGNOFF-REPAIR.7.1.6`).

## 2026-09-25 — Governance charters can be registered again (`SIGNOFF-REPAIR.9.1.1`)

`REASONBRAID-REPAIR-0495`.

- 🔴 **Before:** registering a governance charter needs a site-operator permission, but that permission had been added to the code without the database change that lets it be stored. No one could hold it, so every charter registration was refused. The book listed it as a normal permission.
- ✅ **Now:** the permission can be granted, and an operator holding it can register a charter while anyone else is refused. A new test lists every site permission from the code and checks the database accepts each one, so this cannot slip again.
- ✅ Tested: the new test failed on the old database, naming exactly this permission, and passes now.

## 2026-09-25 — Closing checks finished: nine items reviewed, seven real problems found (`SIGNOFF-REPAIR.12`)

`REASONBRAID-DOC-0163`, `0165`–`0171`. No code changed.

- ✅ **Closed** after checking the evidence: machine-acquisition safety (`.7.2`), bootstrap waits, and two tooling items.
- ⏸️ **Deferred:** the optional browser pack's storage and process limits, which only matter when that pack is switched on.
- 🔴 **Found, and now queued to fix:** four evidence problems, two evaluation problems and one publication problem (listed in the entries below). These are exactly what "every sub-item done" had hidden.
- ⏭️ **Next:** the must-fix items, starting with the ones that cross between organisations.

## 2026-09-25 — Three finished items formally closed (`SIGNOFF-REPAIR.11.4.5`, `.3.3.4.3.3.3.3.2.3`, `.11.4.3.1.7`)

`REASONBRAID-DOC-0167`–`0169`. Closing checks; no code changed.

- ✅ Each had all its work done and verified, and had simply never been marked closed. Each closure names the commits and the tests that prove it.

## 2026-09-25 — Policy publication checked: one problem found (`SIGNOFF-REPAIR.9.2`)

`REASONBRAID-DOC-0166`. A closing check; no code changed.

- 🔴 **Must be fixed:** the step that marks a policy publication as effective or failed checks the current state and then writes without locking, so two at the same moment can overwrite each other, and a live publication can be turned into a failed one (`.9.2.2`).
- ✅ **Holds:** where a publication is written, which approval and policy it is tied to, the Git checks, the safe update of the live pointer, and the reconciliation between the database and Git.

## 2026-09-25 — Evaluation checked: two problems found (`SIGNOFF-REPAIR.8.2`)

`REASONBRAID-DOC-0165`. A closing check; no code changed.

- 🔴 **Must be fixed:** an evaluation trial accepts the same option listed twice, which silently gives it twice the share of cases (`.8.2.6`). And when a thread is created, its routing decision is logged before the request is authorised, so a refused request still leaves a log entry for a thread that never existed (`.8.2.7`).
- ✅ **Holds:** quality gates refuse missing or non-numeric results and are tied to the right test set, calibration ignores runs that do not belong to it, and only site operators can write evaluation records.

## 2026-09-25 — Checking each fix is now about five times faster, with the same rigour (`SIGNOFF-REPAIR.12.1`)

`REASONBRAID-DOC-0164`. A measured decision; no product code changed.

- ✅ **Measured:** the slowest check deliberately breaks the new code in several ways and confirms a test catches each one. The same ten breaks took **77 minutes** before, and now take **14 minutes**, with all ten still caught. The saving comes from rebuilding only the tests that cover the code, instead of about 50 unrelated test programs each time.
- ⚠️ **A wrong turn, kept on record:** the obvious setting made it slower, 2 hours, because the tool ignored it at the build step. The logs showed why.
- ✅ **The rule:** the full live test run happens on any change to shared foundations, otherwise at least every three fixes, and always before publishing.

## 2026-09-25 — Evidence checked: four real problems found (`SIGNOFF-REPAIR.7.4`)

`REASONBRAID-DOC-0163`. A closing check; no code changed.

- 🔴 **Must be fixed:** evidence that was deleted can still be built on, still supports assessments, and is quietly cited again if the same content is fetched again (`.7.4.6`). Anyone can sign an assessment with someone else's name (`.7.4.7`). Resubmitting an assessment with a different quote silently keeps the first one (`.7.4.8`). Fetching evidence again does not renew its "fresh until" date, although the book says it does (`.7.4.9`).
- ⏸️ **Deferred with a trigger:** evidence quarantine (the book already says it does not exist), unchecked derivation labels, and a write order that can leave an unused stored file behind.
- ✅ **Holds:** a quoted excerpt must really appear in the evidence; deletion uses the server's clock; one organisation cannot delete evidence another relies on.

## 2026-09-25 — The repair work now has a finish line (`SIGNOFF-REPAIR.12`)

`REASONBRAID-DOC-0162`. Decisions taken at the director's request; no product code changed.

- 🔴 **Why:** the repair work was meant to end when every finding was closed, but every review finds new ones: yesterday 17 items were closed and 17 opened, so it never got closer. A first count also said 50 open items when the true number is 65, because it missed an older way items record their state.
- ✅ **The finish line:** the repair work ends at a "bug bar", the rule security and reliability teams use to decide what must be fixed before a release. What must be fixed: anything that lets one organisation touch another's data, money or service; anything that silently loses or corrupts data, budgets, evidence or publications; anything the book claims that the code does not do; and any check that gives a false answer. Everything else is deferred with a written trigger that reopens it, and the book states the limit meanwhile.
- ✅ **The triage of all 65:** 8 close after a quick check, 1 duplicate, 25 must be fixed (plus a new one: counting open items reliably), 12 deferred with triggers, 19 group headings that close with their parts. Every open item now says which it is.
- ⏳ **Faster checking, same rigour, being measured:** the slow step, mutation testing, rebuilds about 50 unrelated test programs for every change it tries. My first attempt to narrow it made it slower (2 hours instead of 77 minutes), because the tool ignored the setting at the build step. A second setting is being measured; the result and the rule go in the next entry.
- 📖 The book's qualification chapter gains "When the corrective work ends", and its out-of-date row on machine recovery and budgets now says that work is complete.

## 2026-09-25 — A fake fetcher can no longer silently stop everyone's web fetches (`SIGNOFF-REPAIR.7.1.3`)

`REASONBRAID-REPAIR-0494`.

- 🔴 **Before:** any organisation's administrator could register a "fetcher" advertised as fast. It would be ranked first for every organisation, the server had nothing to run it with, and every fetch then returned an empty answer with no error. Reproduced live: the test's leftover entry even silenced four other tests.
- ✅ **Now:** the server uses the first ranked fetcher it can actually run, and names any it skipped. If none can run, the answer says so by name.
- ⏭️ **Next:** decide who should be allowed to add fetchers to the shared list at all (`.7.1.3.1`).
- ✅ Tested: the new test failed on the old code and passes now; the deliberately broken versions were caught; broad live suite and strict lint pass.

## 2026-09-24 — Resource and resolver ownership reviewed: three real gaps found (`SIGNOFF-REPAIR.7.1`)

`REASONBRAID-DOC-0161`. A review against the code; no code changed.

- 🔴 **Most serious:** any organisation's administrator can register a new "resolver" (the component that fetches a web resource), and the list is shared by everyone. A fake one advertised as fast would outrank the real fetcher. Every organisation's fetches would then silently do nothing, with no error shown. Owned by `.7.1.3`, next.
- 🔴 **Also real:** the first organisation to cite a web address decides how it is fetched for everyone else (`.7.1.4`); and when two fetchers tie, which one is used is left to chance (`.7.1.5`).
- ✅ **Holds:** fetched content is checked against its expected fingerprint, a corrected fetcher description replaces the old one completely, the isolation rules compare in the right direction, and who may write to the shared lists was settled earlier.

## 2026-09-24 — Tenant-owned administration checked and closed (`SIGNOFF-REPAIR.3.5`)

`REASONBRAID-DOC-0160`. A closing check; no code changed.

- ✅ **Closed:** every piece of this review was already done, but it had never been formally closed. Each of its goals was checked against the code and the test that proves it. One organisation's administrator cannot change or read another organisation's machines, queues, enrollment tokens or spending cut-off. A frozen organisation's administrator can still look at its own records, as approved.
- ⏭️ **Next:** ownership of resources and resolvers (`.7.1`), which has not been reviewed yet.

## 2026-09-24 — The server's certificate authority now renews itself, and the node-machine review is complete (`SIGNOFF-REPAIR.4.1.8.2`)

`REASONBRAID-REPAIR-0493`. The second of the two steps decided in `REASONBRAID-DOC-0159`.

- 🔴 **Before:** nothing ever renewed the certificate authority. After a year every machine would have been locked out at once.
- ✅ **Now:** when a third of the authority's life is left (about four months for a one-year authority), the server creates its successor by itself, without a restart, and switches new certificates to it. Machines certified by the old one keep working. With several servers, exactly one creates the successor and the others adopt it. The health check has a new line, `server_ca_renewal`, which only turns red if a due renewal fails.
- ⭐ **The node-machine review is complete:** enrollment, certificates, leases, delivery, recovery and budgets.
- ✅ Tested: the live test shows one successor created, reused on every later check, adopted by a second server, and the old authority's machines still trusted. Every deliberately broken version was caught, some only after I added tests. This one could not be shown failing on the old code, because the feature did not exist there at all; that is recorded. Broad live suite and strict lint pass.

The entries before those above were rotated into reachable Git history at the
**fifty-eighth rotation** (`SIGNOFF-REPAIR.11.4.1.6`, which owns this ledger’s rotation). The exact predecessor — this file as it
stood at the commit named below, which is the object every retired record was
checked against before this notice was written — is:

```bash
git show cb893ac478656c7288322ba4bfb7710a90c57d92:CHANGELOG.md
```

That snapshot is 88460 bytes and 887 lines, and contains 116 dated
entries; its Git blob is `0ab9916191399ac8eaa921bdbf94c7a8e7f9fab6` and its SHA-256 is
`930d81eacbc3716834cdc8908de14cf57a12d147b15aa4a048ae1938af2c6601`. It carries the fifty-seventh rotation's
notice in turn, and each earlier notice names the one before it, so the chain
walks all the way back. `docs/decisions/2026-09-09_changelog-rotation.md` holds
the first transition's evidence.

⛔ **3 record(s) rotated out, 114 kept, lossless** — every retired heading was retrieved from the
predecessor named above before this notice was written, and every figure in it was re-derived from that object with
`git rev-parse`, `git cat-file` and SHA-256 rather than typed. ⭐ The cut is DERIVED, not chosen: it retires whole
records until the ledger has at least 10 commits of runway at the p90 entry size measured over the last
60 non-rotation commits — because two rotations that stopped at the threshold instead left 344 and 296 bytes and
the first forced another rotation on the very next commit (`SIGNOFF-REPAIR.11.4.1.6`).
