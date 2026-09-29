# LIVE_STATUS.md — authoritative live progress tracker

Rows use only **Done · Mostly Done · In Progress · Not Started**. This is a current
snapshot. Historical implementation and verification records live in the phase
task-trees and git; the pre-review snapshot is `9c2d2ba:LIVE_STATUS.md`.

## Current status

This table states only what holds until the corrective review closes, so it names no task-tree
leaf (`PLAN-STATES-TARGETS` refuses one): a restated leaf goes false as soon as the leaf moves.
The moving parts are derived. **Open work**, by bug-bar class: `python3 -B scripts/census_open_leaves.py`.
**Where the build differs from the frozen plan**, and the limit each deferral leaves: the book's
[qualification review](docs/book/src/qualification-review.md). **What each repair changed**: the dated
entries below. Phases 0–6 read *Mostly Done* because each tree is complete and the corrective review
re-examines its claims; they return to *Done* when the review requalifies them.

| Area | Status | Current evidence and remaining work |
| --- | --- | --- |
| Roadmap and task-tree conversion | Done | `PROGRAM` maps all phases, gates, backlog items, ADRs and demonstrations; `RB-SEED.2` holds the original census. |
| Claim-verification policy | Done | Local policy matches the director-authorized pgen donor at the startup comparison; subsequent claims still need all three verification legs. |
| Discipline and continuity | In Progress | The corrective review owns local storage, disposable verification, doctrine defects and document containment. `scripts/check_doctrines.sh` enforces every mechanizable doctrine at each commit and in CI, and the ledgers rotate losslessly through Git history. |
| Phase 0 — contracts and experiments | Mostly Done | Historical G0 package and owner signoff retained in `PHASE-0`; the corrective review re-examines its authority, budget and adapter claims. |
| Phase 1 — LAN vertical slice | Mostly Done | Historical demonstration retained in `PHASE-1`, and the web console carries a real-browser control; the corrective review re-examines its authority and demonstration claims. |
| Phase 2 — identity, delivery and recovery | Mostly Done | Historical machinery retained in `PHASE-2`; the corrective review re-examines revocation, fencing, budget and recovery. |
| Phase 3 — directory and recruitment | Mostly Done | Historical machinery retained in `PHASE-3`; the corrective review re-examines tenant visibility, recruitment and automatic initiation. |
| Phase 4 — resources and evidence | Mostly Done | Historical G4 record retained in `PHASE-4`; the corrective review re-examines acquisition isolation, evidence integrity and retention. |
| Phase 5 — deliberation and evaluation | Mostly Done | Historical G5 subtraction gate withdrew the quality-lift claim; the corrective review re-examines workflow and evaluation. |
| Phase 6 — governance | Mostly Done | Historical G3 machinery exit retained; binding use remains gated; the corrective review re-examines policy, publication and deployment. |
| Phase 7 — Internet qualification | Mostly Done | Hardening machinery exists; G6/G7 Internet exposure remains NOT MET. The director greenlit Internet exposure over HTTPS with OAuth on 2026-09-25, security first; owner-only first (the director and their own agents), gated by our own evidence; opening to anyone else still needs an independent threat-model review and penetration test (`docs/runbooks/external-security-review.md`). External threat-model, injection and penetration-test evidence remain required. |
| Phase 8 — federation and interoperability | In Progress | Through regional routing historically recorded in `PHASE-8`. Store-and-forward, exit export/import and the G8 gate remain there, and resume when the corrective review closes; shared authority and protocol gaps come first. |
| Phase 9 — stable release | Not Started | G9 requires sustained operational evidence and the outstanding release decisions. |
| Corrective review | In Progress | It ends at a bug bar, not at exhaustion (`REASONBRAID-DOC-0162`): every leaf in a blocking class (cross-tenant; integrity of state, money, evidence or publication; a false claim; a gate that lies) closed with reproducible evidence, every deferred leaf with a readable trigger and a limit the qualification review states, the qualification review reconciled with measured behaviour, and the full CI checkpoint green on a pushed commit. |

## 2026-09-29 — The online test run can find its compiler again (`SIGNOFF-REPAIR.11.4.3.1.2.29`)

`REASONBRAID-REPAIR-0577`.

- 🔴 **Before:** on GitHub's test machines, a helper added last week rebuilt its settings from scratch and lost track of the compiler the run had just installed, so the database tests could not start.
- ✅ **Now:** the helper keeps the compiler it was given. Nothing changes when the tests run on this machine.

## 2026-09-29 — A library the Git support relies on was updated for a published safety flaw (`SIGNOFF-REPAIR.11.69`)

`REASONBRAID-REPAIR-0576`.

- 🔴 **Before:** a small library used to read Git fingerprints had a published flaw. On Intel and AMD processors it could read slightly past the end of its data. The check that runs before every upload caught it and stopped the upload.
- ✅ **Now:** the library is on the fixed version, nothing else changed, and the Git features' tests pass.

## 2026-09-29 — The system now tests its handling of malformed and hostile files automatically (`SIGNOFF-REPAIR.11.4.7.2.1.1`)

`REASONBRAID-REPAIR-0575`.

- 🔴 **Before:** a plan made at the start said that once the system began reading untrusted files and pages from the internet, it would get "fuzz" testing: feeding it deliberately damaged inputs to find crashes. The system started doing that, and the testing never happened.
- ✅ **Now:** every code path that reads outside data (PDF, ZIP, TAR, news feeds, compressed pages, certificates and a few smaller ones) is fed thousands of deliberately damaged versions of valid inputs on every test run. A tool finds those code paths automatically, so new ones can't be forgotten. No crash was found in over 100,000 attempts, and the tests were shown to catch a crash deliberately planted to check them. The guide says what this testing doesn't cover.

## 2026-09-29 — A discussion an agent starts by itself keeps the confidentiality level it declared (`SIGNOFF-REPAIR.11.65`)

`REASONBRAID-REPAIR-0574`.

- 🔴 **Before:** an agent starting a discussion on its own could declare it confidential. The check passed, and the discussion was then created as ordinary, so its work could go to evaluators that confidential work is kept from. Measured: a discussion declared "internal" was stored as general.
- ✅ **Now:** the discussion keeps its level, and anything the agent declares other than "general" is treated as confidential, the safe choice. The guide also says plainly that the declared money budget is only checked against the agent's permission, because nothing counts money yet. The real limit is the discussion's allowance of calls, tokens and time.

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

The entries before those above were rotated into reachable Git history at the
**nineteenth rotation** (`SIGNOFF-REPAIR.11.4.1.6`, which owns this ledger’s rotation). The exact predecessor — this file as it
stood at the commit named below, which is the object every retired record was
checked against before this notice was written — is:

```bash
git show 21487bcf69e9efa4ceecfeaa92ccffa3f14a34bf:LIVE_STATUS.md
```

That snapshot is 53970 bytes and 488 lines, and contains 62 dated
entries; its Git blob is `1e138a5c04db68a56f991de03072f78de3b56b61` and its SHA-256 is
`22e869a0b4013a43d0b9b1532f0cab42f4a0a09f50197620e019f5871950a394`. It carries the eighteenth rotation's
notice in turn, and each earlier notice names the one before it, so the chain
walks all the way back. `docs/decisions/2026-09-09_changelog-rotation.md` holds
the first transition's evidence.

⛔ **11 record(s) rotated out, 52 kept, lossless** — every retired heading was retrieved from the
predecessor named above before this notice was written, and every figure in it was re-derived from that object with
`git rev-parse`, `git cat-file` and SHA-256 rather than typed. ⭐ The cut is DERIVED, not chosen: it retires whole
records until the ledger has at least 10 commits of runway at the p90 entry size measured over the last
60 non-rotation commits — because two rotations that stopped at the threshold instead left 344 and 296 bytes and
the first forced another rotation on the very next commit (`SIGNOFF-REPAIR.11.4.1.6`).
