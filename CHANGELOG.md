# CHANGELOG.md

## 2026-09-23 — A machine that is not properly connected no longer starts paid work (`SIGNOFF-REPAIR.4.4.4.2.2`)

`REASONBRAID-REPAIR-0464`. Closes the fourth recovery gap found by `REASONBRAID-DOC-0153`.

- 🔴 **Before:** a machine whose reconnect had failed still sent work to the provider and paid for it, although it could not yet deliver the answer. If it ever noticed, it shut itself down.
- ✅ **Now:** such a machine refuses to start paid work. The work waits untouched, and the machine reconnects and carries on instead of shutting down. Safety checks that need no connection still run first.
- ✅ Tested: a new check failed on the old code (the machine did the work) and passes now; eleven existing checks now run on properly connected test machines; 89 machine tests, six live suites (91 tests) and the full two-machine demonstration pass; three deliberately broken versions were all caught; strict lint clean.

## 2026-09-23 — The machine's own tests can now see an answer being sent (`SIGNOFF-REPAIR.4.4.4.2.1`)

`REASONBRAID-REPAIR-0463`. Test equipment for the fourth recovery gap; no product behaviour changed.

- 🔴 **Before:** the machine's own tests could never get a machine properly connected, so nothing below the slow full-system tests could see an answer actually being sent. A deliberately broken machine that claimed "sent" while sending nothing passed all 83 of them.
- ✅ **Now:** a small stand-in server lets a test machine connect for real. New checks show an answer sent at once when connected, and an answer kept while disconnected then sent exactly once on reconnect under its original id.
- ✅ Tested: the broken "claims sent, sends nothing" machine is now caught by the machine's own tests; 87 machine tests pass; strict lint clean.

## 2026-09-23 — A finished, paid-for answer can no longer be lost between "done" and "sent" (`SIGNOFF-REPAIR.4.4.4.1`)

`REASONBRAID-REPAIR-0462`. The first half of the fourth recovery gap found by `REASONBRAID-DOC-0153`.

- 🔴 **Before:** a machine recorded "the provider finished" and the answer to send as two separate saves. If it died between them, or was merely not yet reconnected, the answer was thrown away. The work was paid for, marked finished, and never sent; nothing could bring it back. Seven existing tests treated that loss as normal.
- ✅ **Now:** the "finished" record and the answer are saved together in one step. If the machine cannot send the answer right away, the answer waits and goes out on the next reconnect, under its original id so it is never counted twice.
- ⚠️ Still to do (tracked, next): a machine that is not yet reconnected should not start paid work at all.
- ✅ Tested: the seven tests, rewritten to demand the waiting answer, failed on the old code (7 of 7) and pass now; two new crash tests; 83 machine tests and six live suites (91 tests) pass; nine deliberately broken versions were tried: four caught, three not buildable, one caught only by the live suites, one hand-made "two separate saves" version caught; strict lint clean.

## 2026-09-23 — The tests guarding the hand-off to the provider can now see whether the provider was reached (`SIGNOFF-REPAIR.4.4.3`)

`REASONBRAID-REPAIR-0461`. The third of the seven recovery gaps found by `REASONBRAID-DOC-0153`.

- 🔴 **Before:** one test was meant to prove that a replacement machine refuses outdated work instead of sending it to the provider. Its stand-in provider was set up to refuse by itself, though. The test also passed with the safety check switched off, because the stand-in's own refusals produced the same outcome. Nothing in the tests could count provider calls.
- ✅ **Now:** the stand-in provider counts every call. The tests use one that *would* succeed, then check it was called zero times and that the recorded refusal is the safety check's own. Every step's result is checked, not thrown away, and a second test now looks for the right kind of event (a revision, not a contribution).
- ⭐ **Shown, not assumed:** with the safety check deliberately broken, the OLD test still passed and the new one failed. A second test the review had flagged turned out to catch the break already through an earlier check; only its last line was blind, and that line is fixed too.
- ✅ Tested: 13 + 2 live tests pass; 134 tests in the provider and machine packages pass; strict lint clean. No product behaviour changed.

## 2026-09-23 — A refused answer's cost is now counted, and the machine is told it was refused (`SIGNOFF-REPAIR.4.4.2`)

`REASONBRAID-REPAIR-0460`. The second of the seven recovery gaps found by `REASONBRAID-DOC-0153`.

- 🔴 **Before:** when the server refused a machine's finished answer (say, the agent's permission was withdrawn, or the conversation had closed), the provider's cost was never counted — the budget hold simply lapsed — and the machine was told only "received", so it believed the work had landed.
- ✅ **Now:** a refused answer's cost is counted against the budget exactly like an accepted one, because the provider did the work either way. The machine is told the answer was refused and why, records it, and says so in its log. The budget charged is always the one the server attached to the job, never one the machine names.
- ⚠️ Still to do (tracked): the operator's inbox view does not yet show that an answer was refused.
- ✅ Tested: a new check failed on the old code (the budget hold stayed open) and passes now; a full run with a real machine closing the conversation mid-job shows the cost counted and the refusal recorded; two deliberately broken versions (no counting on refusal; the machine ignoring the refusal) were each caught; the machine's 81 tests and five further suites (160 tests) pass; strict lint clean.

## 2026-09-23 — An unresolved provider call is no longer "settled" by the machine's own give-up note, and an operator's ruling now lands as ruled (`SIGNOFF-REPAIR.4.4.1`)

`REASONBRAID-REPAIR-0459`. The first, and worst, of the seven recovery gaps found by `REASONBRAID-DOC-0153`.

- 🔴 **Before:** when a machine could not tell whether a paid provider call had happened, the server declared the case settled as soon as it held *any* message from the machine about that job — including the machine's own "I refused to retry this" report, or the answer from a later retry. The doubt vanished with no evidence. And when an operator ruled "it did not happen" or "it did happen", the machine recorded neither: every ruling became a bare "settled", with the operator's reasoning lost.
- ✅ **Now:** only the machine's own answer *for that specific attempt* settles the doubt; a give-up note or another attempt's answer leaves it open and visible. An operator's ruling is recorded as ruled — "did not happen" or "did happen" — with the ruling itself kept as the evidence. A ruling the machine does not understand leaves the case open rather than closing it.
- ⚖️ **Corrected a claim from an earlier fix:** "did not happen" was described as making the machine re-run the work by itself. It never did and still does not; re-asking is the thread owner's decision, and the fix's record now says so.
- ✅ Tested: two new checks failed on the old code exactly where predicted (a give-up note settled the doubt; a "did not happen" ruling was recorded as a bare "settled") and pass now; two deliberately broken versions (any message settles the doubt; the ruling flattened again) were each caught; the machine's own 81 tests and six further suites (127 tests) pass; strict lint clean.

## 2026-09-23 — The machine-recovery checklist checked against the code: five gaps found, one piece missing, all scheduled (`SIGNOFF-REPAIR.4.4`)

`REASONBRAID-DOC-0153`. A review; no code changed.

- 🔴 **Worst:** when a machine cannot tell whether a paid provider call happened, the server may declare it "settled" on the strength of the machine's own "I gave up on this" report — with no evidence at all. And when an operator rules on such a case, the machine ignores which way the ruling went.
- 🔴 A finished answer the server refuses (say, after the agent's permission was withdrawn) is silently dropped: its spend is never counted and the machine is never told.
- 🔴 Three tests meant to guard the hand-off to the provider cannot tell the safety check from a provider outage; they are fixed before the hand-off is touched.
- 🔴 If the machine dies in the instant between recording "done" and recording the answer, the paid-for answer is lost for good.
- 🔴 A network blip while sending an answer, or an unresolvable provider outcome, stops the whole machine process; and the machine waits for ever on a server that never replies.
- ❌ A time limit is computed for every provider call and nothing enforces it; nor is the reply's size bounded.
- ⚖️ Retrying an unresolvable call is correctly refused by the machine, but nobody can yet authorize one.
- ✅ All seven are scheduled in order of risk, each to be proven with a failing test first.

## 2026-09-23 — Two jobs sent to one machine at the same instant no longer collide: proven, and the inbox checklist is complete (`SIGNOFF-REPAIR.4.3.4`, closing `SIGNOFF-REPAIR.4.3`)

`REASONBRAID-REPAIR-0458`. The last of the four inbox-identity gaps found by `REASONBRAID-DOC-0152`; with it the whole checklist (`SIGNOFF-REPAIR.4.3`) is met.

- 🔴 **Before:** two jobs handed to one machine at the same moment could be given the same number, and the second was refused.
- ✅ **Now:** the per-machine counter introduced two fixes ago (`REASONBRAID-REPAIR-0456`) already makes the second job wait its turn and take the next number. This change proves it rather than re-fixing it: a test holds the counter the way a job in flight does, watches the database report the second job waiting, releases it, and sees it land with the next number; then eight jobs at once all land with eight consecutive numbers.
- ✅ The test was then run against the old numbering to show it refuses: nothing waits, and the test fails.
- ✅ The inbox checklist is now met in full: receipts and reconnect answers are per machine, the counter survives clearing out old work, answers and the "answered" state are per machine, and simultaneous jobs are serialized.
- ✅ Tested: the new test passes on the current code (51 tests in the machine-channel suite) and was then run against the old numbering, where it failed as it should because nothing waited; three further suites that hand out job numbers pass unchanged (113 tests); strict lint clean.

## 2026-09-23 — Two machines holding a job with the same name now each get their answer counted (`SIGNOFF-REPAIR.4.3.3`)

`REASONBRAID-REPAIR-0457`. The third of the four inbox-identity gaps found by `REASONBRAID-DOC-0152`.

- 🔴 **Before:** job names are unique per machine, not per organisation, so two machines in one organisation could hold jobs with the same name. The server's "have I already counted this answer?" check looked only at the job name, so the second machine's answer was treated as a clash with the first's and thrown away. And the inbox view marked a job "answered" on one machine when the answer had come from the other.
- ✅ **Now:** the answer check and the "answered" state both ask *which machine* as well as *which job*. Each machine's answer is counted; a machine re-sending its own answer is still recognised as a repeat.
- ✅ Checked before choosing the fix: today the server never gives two machines the same job name (each job is named after the one event that created it, and one event goes to one machine), so nothing was lost in practice; the check simply permitted it. Old records were re-labelled where it was unambiguous which machine they belonged to.
- ✅ Tested: two new checks failed on the old code exactly where predicted (one answer counted where two were owed; a job wrongly marked answered) and pass now; two deliberately broken versions (the answer check back on the job name alone; the answered state ignoring the machine) were each caught; thirteen suites pass (264 tests) after one test that read the stored answer by the old label was updated; strict lint clean.

## 2026-09-23 — Clearing out a machine's old delivered work no longer locks it out or hides later work (`SIGNOFF-REPAIR.4.3.2`)

`REASONBRAID-REPAIR-0456`. The second of the four inbox-identity gaps found by `REASONBRAID-DOC-0152`.

- 🔴 **Before:** the server worked out "how far has this machine got" by looking at the highest-numbered item still in its inbox. When an operator cleared out old delivered items, that number could drop — so a machine that had confirmed up to item 30 was refused on its next check-in as "ahead of the server", and new items handed out afterwards could be numbered 1, 2, 3 again, which the machine had already seen and would skip. Work went silently undelivered and counted against the machine's backlog for ever.
- ✅ **Now:** the server keeps a durable per-machine counter that only ever goes up. Every new item is numbered above it, so clearing out old items removes items, never numbers. A machine that confirmed up to 30 reconnects fine, and the next item is number 31.
- ✅ Existing machines' counters were seeded from what they already held, so nothing moved. A machine whose entire inbox had already been cleared before this change cannot have its lost number recovered; the change says so.
- ✅ Tested: the new prune-everything-then-reconnect check failed on the old code exactly where predicted and passes now (the inbox suite: 12 tests); two deliberately broken versions (the counter overwritten by the inbox's highest number; the reconnect check ignoring the counter) were each caught; ten further suites that hand out or read cursors pass unchanged (223 tests); strict lint clean.

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

## 2026-09-23 — A partnership's terms now have a fingerprint, and accepting one records exactly which terms the partner had offered (`SIGNOFF-REPAIR.5.3.1`)

`REASONBRAID-REPAIR-0432`.

- 🔴 **Before:** when an organisation accepted a partnership, the audit receipt — which is supposed to name the partner's record by its fingerprint — named the partner's *id* instead, because a partnership record had nothing to fingerprint. And an organisation could "accept" a partnership the partner had never proposed, even though the refusal message claimed the partner had to propose first.
- ✅ **Now** every partnership direction carries a fingerprint of its terms, computed by the server, and existing rows were given theirs by the same recipe during the upgrade — checked on a real pre-upgrade row. Accepting reads the partner's current offer and records its fingerprint twice: in the audit receipt, and on the accepting row. So the trail says which terms each side saw when it agreed. If one side later changes its terms, its own acceptance is cleared, while the partner's row still names the terms it agreed to.
- ✅ One change on the wire: accepting when the partner has made no offer is refused, naming the partner. That is what the message always said.
- ✅ Tested end to end, including the refusal, the fingerprints on both sides, and a change of terms. Run against the previous code and schema first, the acceptance went through with nothing on the other side.
- 🔴 The handbook's example of an audit receipt showed a kind and an id shape the code never wrote; corrected to the real shape.

## 2026-09-23 — Importing the same partner agent twice now returns the original, and every imported agent records where it came from (`SIGNOFF-REPAIR.5.3.2`)

`REASONBRAID-REPAIR-0431`.

- 🔴 **Before:** an imported agent was identified only by its display name. Importing the same agent again was refused with a message about the name being taken; importing it again under a new name created a second, unrelated local agent; and an unrelated agent that happened to share a name was refused as if it were a repeat. Nothing recorded which partner agent a local one came from — only whoever still held the card knew.
- ✅ **Now** every import records its origin: the partner organisation, the partner's agent id, the fingerprint of the card that landed, who authorised it and when. Importing an agent that is already here — under any name, from any later card — returns the original local agent, flagged as a repeat, together with the fingerprint of the card on file. Nothing is written twice. The name refusal remains, but only for a genuinely different agent that shares the name.
- ✅ The agent's owner or administrator sees the origin on the agent's profile; partners and outsiders do not.
- ✅ Tested end to end: the repeat, the renamed repeat, the origin on the profile, and the name refusal for a different agent. Run against the previous code first, the repeat was refused with the name message.
- ⚠️ Left open, recorded: whether a *newer* card for an already-imported agent should update the local profile, and under whose authority. Today it does not, and the answer says so.

## 2026-09-23 — An imported agent's permission is now issued by the administrator who authorised the import (`SIGNOFF-REPAIR.5.3.3`)

`REASONBRAID-REPAIR-0430`.

- 🔴 **Before:** when an administrator imported an agent's card from a partner organisation, the local permission the agent received was recorded as issued by a made-up identity — a fresh id that belonged to nobody — even though the administrator who authorised the import was known to the code and simply never used.
- ✅ **Now** the permission names that administrator as its issuer. If an organisation is administered by an agent rather than a person, the old limitation stays, with the reason written beside it.
- ✅ Tested by importing a card and reading the permission back: its issuer is the importing organisation's administrator and is enrolled there. On the previous code the test failed with a random id.
- ⚠️ Still true: the database does not force an issuer to be a real principal, and the ordinary enrolment of an agent still records a made-up issuer until the development bootstrap is replaced.

## 2026-09-23 — The federation work was measured before building: two of five items are already done, two are real defects, and one is a feature nobody has built (`SIGNOFF-REPAIR.5.3`)

`REASONBRAID-DOC-0143`. A census, no code changed.

- ✅ **Already done by earlier repairs:** importing an agent card from a partner organisation is one all-or-nothing transaction, and a partnership being revoked at the same moment as an import or a partnership change is handled in the right order.
- 🔴 **Defect 1:** when an organisation accepts a partnership, the audit receipt is supposed to name the partner's record by its fingerprint. It names the partner's *id* instead — no fingerprint, no record — because a partnership record has no version to fingerprint. Nothing tests it.
- 🔴 **Defect 2:** importing the same agent twice is refused only because its display name is already taken. The same agent under a different name imports again as a second identity, and an unrelated agent with the same name is refused as if it were a repeat. Nothing records which remote agent a local one came from; only whoever still holds the card knows.
- 🔴 **Defect 3:** an imported agent's local permission is issued by a made-up identity, even though the administrator who authorised the import is right there and unused.
- ❌ **Not built:** recruiting an agent from a partner organisation *through a call*. Today the only cross-organisation path is importing its card; calls never leave their own organisation.
- ⚖️ **A reviewer's worry was set aside with a reason:** changing partnership terms without the partner re-agreeing cannot widen anything, because each side's own declaration bounds the effect. What is missing is the record of which terms each side saw — which defect 1 supplies.
- Split into five tasks, smallest first: the issuer, then the origin record and repeats, then the fingerprinted receipt, then an expiry date, then the cross-organisation call.

## 2026-09-23 — Each step of a recruitment call now happens all at once, so two people acting at the same moment cannot corrupt it (`SIGNOFF-REPAIR.5.2.4`)

`REASONBRAID-REPAIR-0429`. With this, the recruitment-and-initiation repair area (`SIGNOFF-REPAIR.5.2`) is closed.

- 🔴 **Before:** opening, answering and closing a call were each several separate database writes, and each had a hole, measured: two opens at the same moment both slipped under the "at most four open calls per initiator" limit; an agent that joined while the organiser was closing was recorded as joined but left off the panel; two closes at the same moment ended in a database error instead of an answer; and if writing the offers failed halfway, the call stayed behind, advertised to nobody.
- ✅ **Now** each step is one all-or-nothing transaction. Opens within one organisation take turns, so the limit is decided after the previous open has finished. A close holds the call while it works: a second close is told "the call is closed", and an answer that arrives during a close waits and is then told the same, with nothing recorded — an answer to a closed call belongs to no panel, so it is refused rather than kept as a "late" entry nobody reads. If any part of an open fails, no call is left behind.
- ✅ Each of the four situations is a test that stages the collision on purpose and checks the outcome. All four failed on the previous code exactly as described. Weakening the close's hold to a shared one was tried as well: the two closes then deadlock and the late answer slips in, so the strict hold is doing real work.
- ✅ The decision record explains why an open uses a per-organisation lock rather than locking the organisation's own row (every other reader would queue behind it) or reusing the permission-issuing lock (opening a call is not issuing a permission).

## 2026-09-23 — A recruitment call can no longer close with fewer panelists than it asked for (`SIGNOFF-REPAIR.5.2.5`)

`REASONBRAID-REPAIR-0428`.

- 🔴 **Before:** when a call was closed, the "at least N panelists" rule was checked against everyone who had *said* they would join. Only afterwards did the server re-check whether each of them still qualified, and drop those who no longer did. So an agent that joined while qualified and then lost its qualification (for example, its owner's endorsement of a skill lapsed) still counted toward the minimum, and the call could close with fewer panelists than required — even none.
- ✅ **Now** the minimum is checked on the panel that is actually selected, after the re-check. If too few still qualify, the close is refused with a message stating how many are required, how many still qualify, and how many had joined — and the call stays open, so the organiser can wait for more joiners or re-qualify one.
- ✅ Tested by making exactly that happen: an endorsed agent joins, its endorsement lapses, the close is refused and the call stays open; the owner endorses it again and the same close succeeds with the agent on the panel. Run against the previous code first, the test showed the old behaviour precisely: a closed call with an empty panel.
- ⚠️ The remaining recruitment-call problem — each of the three transitions is several separate database writes rather than one, so two simultaneous closes can collide — is the next task.

## 2026-09-23 — A recruitment call's three transitions are each several separate writes, and its minimum is checked before the filter that can empty the panel (`SIGNOFF-REPAIR.5.2`)

`REASONBRAID-DOC-0142`. A decision; no code changed.

- I checked what the recruitment task still owes after this week's repairs. The identity bindings are done. Two things are not.
- 🔴 **None of open, answer or close is a single transaction.** Opening a call counts, then inserts the call, then writes each advertisement as a separate step, so two simultaneous opens can both pass the cap and a crash can leave a call half-advertised. Closing reads, then writes the panel and the status as two steps, so two simultaneous closes collide on the database's own key with a raw "500", and an answer can slip in between the close's read and its write.
- 🔴 **The minimum is checked against who said "join", not against who is still eligible.** The close then drops anyone whose eligibility lapsed, so a call can close with a panel smaller than its minimum, down to empty.
- ✅ Two follow-ups, tracked: the minimum on the selected panel first (small, reproducible), then each transition as one transaction.

## 2026-09-23 — An administrator can now settle a job whose outcome the machine could not prove, and the machine applies the decision (`SIGNOFF-REPAIR.11.4.7.2.1.5.5`)

`REASONBRAID-REPAIR-0427`.

- 🔴 **Before:** when a machine crashed after possibly calling a provider and could not tell whether the call happened, the roadmap's fourth way out — a human decides — did not exist. The machine could only wait for a server receipt or prove it itself, and the administrator's view listed three actions, none of them a verb.
- ✅ **Now** the organisation's administrator records a verdict — "it completed" or "it did not happen" — with a reason. The verb is authorised, written to the audit trail like every other administrative action (the fifteenth kind), and bound to the organisation's own machines. The machine picks the verdict up at its next check-in and closes the question; until then the listing shows the verdict as awaiting the machine. A second verdict, an unknown verdict, or another organisation's administrator are refused.
- ✅ Tested end to end with a real machine journal: the ambiguity, the refusals, the recorded verdict and its audit entry, the listing, and the machine applying it. Making the check-in ignore the verdict leaves the job ambiguous, which the test catches.
- ✅ With this, every buildable item in the re-derivation family opened on 22 September is closed; what remains waits on your two decisions or on the first deployment beyond localhost.

## 2026-09-23 — Correction: the issuance record I called "owed now" cannot be written without first replacing the development bootstrap (`SIGNOFF-REPAIR.11.4.7.2.1.5.4.4`)

`REASONBRAID-DOC-0141`. A correction of this morning's DOC-0140; no code changed.

- 🔴 DOC-0140 said an audit record for each permission's issuance could be added now. One read later that is wrong: every audit record of that kind must point at an *admitted* request, and the enrolment step — where the default permission is issued — is deliberately not admitted: it is the development bootstrap that trusts whoever calls it.
- ✅ Giving it an admitted issuer is the same change that would let a permission be signed: replacing the development bootstrap at the first deployment beyond localhost. So the two halves wait on one trigger, and the record says so. The census figure I cited (zero records at enrolment) was true and beside the point: the absence is by design, not by omission.
- ✅ With this, the four missing permission fields are all settled: one built, one waiting on a missing concept, one on your decision, one on the bootstrap's replacement.

## 2026-09-23 — A permission can now hold one agent to fewer decision rules than the organisation's charter allows (`SIGNOFF-REPAIR.11.4.7.2.1.5.4.1`)

`REASONBRAID-REPAIR-0426`.

- 🔴 **Before:** the only limit on which decision rule a new thread could use was the organisation's charter. An issuer could not say "this agent may open threads, but only under owner-decides".
- ✅ **Now** the enrolment step can declare that list on the agent's permission; malformed lists are refused by name (a permission that creates no threads, an empty list, an unknown rule); the administrator's permission list shows it; and when the agent creates a thread, the server reads the list from the exact permission that admitted the request and refuses a rule outside it, before the charter check. A permission with no list leaves the charter to decide, as before.
- ✅ Tested: a charter allowing two rules, a narrowed agent refused one and allowed the other, an unconstrained agent allowed both. Making the check permissive, or the validation permissive, makes the tests fail.

## 2026-09-23 — Of the four permission fields the roadmap lists and the code lacks, one is owed now, one waits on a missing concept, one needs your decision, and one is two halves (`SIGNOFF-REPAIR.11.4.7.2.1.5.4`)

`REASONBRAID-DOC-0140`. A decision; no code changed.

- The roadmap's permission record lists four fields that exist nowhere in the code: policy domains, decision-rule constraints, conditions, and a signature-or-record. I checked what each would attach to.
- 🔨 **Decision-rule constraints are owed now**: each organisation's charter already limits which decision rules a thread may use, and a permission can narrow that for one agent; the reader exists. Built next.
- ⏸️ **Policy domains wait**: nothing in the system evaluates a domain at all — even the enrolment boundary's own domain list is compared to nothing — so a permission-level list would bind nothing. It reopens the moment any domain is evaluated.
- 💡 **Conditions need your decision**: the roadmap names the field and nothing says what a condition is (a purpose? a time window? a network?). Not built until that is decided, so it is never a field that is stored and ignored.
- ✂️ **The signature-or-record field is two halves**: the record half (an audit entry for each permission's issuance — the enrolment's default permission has none today) is owed now; the signature half waits for the first deployment beyond localhost, with mutual TLS.

## 2026-09-23 — Which machine ran each agent is now recorded, and the "one machine, one agent" rule is a declared limit, not a hidden one (`SIGNOFF-REPAIR.11.4.7.2.1.5.1`)

`REASONBRAID-REPAIR-0425` with decision DOC-0139.

- The question was whether the project owes a registry mapping machines to agents before the stable release. Today a development rule makes a machine's id the agent's id, and seven database joins and two dispatch paths rely on it.
- ⚖️ **Decided:** the roadmap binds an agent to a machine only through its "incarnation" (the record of which provider, model and harness the agent ran as), so a declaring registry is not owed before the stable release; the release gate asks for declared limits, and this one is declared in the book in two places.
- ✅ **Built:** each incarnation now records which machine declared it, at the one place it is written, and the administrator's incarnation view shows it. Today that always equals the agent id, by the rule; the day a directory replaces the rule, the history is already there, and the exact condition that reopens the registry question is a one-line query.
- ✅ Tested: an enrolment records its machine on the incarnation and the view shows it; removing the write makes the test fail.

## 2026-09-23 — An offline agent's backlog of undelivered work is now capped, and the cap is visible (`SIGNOFF-REPAIR.11.4.7.2.1.5.3.3`)

`REASONBRAID-REPAIR-0424`.

- 🔴 **Before:** if an agent's machine never reconnected, work kept piling up for it without limit — the roadmap's "maximum offline backlog" existed only on paper.
- ✅ **Now** a machine already holding 64 undelivered jobs (a development-scale figure, not a measured one) is handed nothing more. The action that would have handed it the job — an agent accepting an invitation, or a challenge that would send a revision back to the author — is refused and undone, so an invitation never exists without its work. The refusal is recorded like every storm control, and the administrator's node view shows each machine's backlog against the cap before the refusal ever happens.
- ✅ Tested: a machine seeded at the cap shows "64 of 64", the accept is refused and rolled back with the record naming the machine, the cap and the thread, and after one job is taken the same accept lands. Removing the cap makes the test fail.
- ✅ With this, the whole storm-control family opened on 22 September is closed: every control whose trigger had fired is built, recorded and readable.

## 2026-09-23 — Every storm-control refusal is now recorded, so a storm can actually be seen (`SIGNOFF-REPAIR.11.4.7.2.1.5.3.4`)

`REASONBRAID-REPAIR-0423`.

- 🔴 **Before:** when the server refused a request as a storm control — too many open calls, an agent re-joining its own chain, a chain too deep — the caller got a "429" and nothing was written down. The roadmap's circuit breakers were postponed until "the first multi-tenant storm is observed", and nothing could observe one.
- ✅ **Now** every such refusal is written down before it is answered, by the one piece of code that is allowed to produce that answer, so no refusal can be given without a record. Each row says which control refused, its limit, who was refused, which thread they named, and the exact words they were given. An administrator lists them with one call.
- ✅ The breakers' trigger is now a plain question with an answer: have two or more organisations been refused within an hour?
- ✅ Tested on the existing storm tests: the fifth call in a row and the two chain refusals each appear on the record with the right control, limit and words. Removing the write makes the tests fail. The first run caught that the fan-out record named an internal handle instead of the person; fixed before commit.

## 2026-09-23 — The last four pre-wake checks each wait on something that does not exist yet (`SIGNOFF-REPAIR.11.4.7.2.1.5.3.2.4`)

`REASONBRAID-DOC-0138`. A decision; no code changed.

- The roadmap's pre-wake checklist has four items left: notification controls, required tools, adapter health and billing route. I checked each against what the machine actually runs.
- ⏸️ None can be built honestly today: the machine runs only the test adapter; the adapter contract has no health check, a job carries no list of tools it needs, nothing anywhere names a billing route, and a call has no urgency class to coalesce by. Building a check over a fact that does not exist would be the "declared but never read" mistake this whole series removed.
- ✅ Each item now carries the exact condition that reopens it, in a form a command can read. The billing-route half that *records* the route is queued for the next adapter-contract version so the contract changes once.
- ✅ With this, the parent task — the pre-wake checklist and the six permission limits — is closed: nine controls were built across this series, and what remains is named with its trigger.

## 2026-09-23 — A thread an agent started on its own now limits who its calls may reach, and a call must name a real thread (`SIGNOFF-REPAIR.11.4.7.2.1.5.3.2.3.3`)

`REASONBRAID-REPAIR-0422`.

- 🔴 **Before:** "audience" existed nowhere in the code, so a permission could not say how widely a thread started by an agent may recruit. And opening a call checked nothing about the thread it named — three tests had been passing for months by naming thread ids that never existed.
- ✅ **Now** a permission can say `audience: tenant` or `network`; a thread an agent starts remembers which permission admitted it; and a call on such a thread that would reach the whole network is refused when the permission says "tenant only". A call must also name a thread that really exists in that organisation, or it is refused the same way every thread view refuses.
- ✅ Tested: the thread shows its permission, a network-wide call is refused and a tenant-wide one lands, a person's own thread is unaffected, and a made-up thread id is refused. The three old tests now create real threads and keep every check they had. Making the audience check permissive, or dropping the remembered permission, makes the test fail.

## 2026-09-23 — An automatic-thread permission can now carry its limits, and they are read from the permission that actually admitted the request (`SIGNOFF-REPAIR.11.4.7.2.1.5.3.2.3.1` and `.2`)

`REASONBRAID-REPAIR-0421`.

- 🔴 **Before:** nothing could set the limits an automatic-thread permission is supposed to carry, and the one limit that was read (spend) was taken as the largest across all of an agent's permissions — including expired ones — rather than from the permission that admitted the request.
- ✅ **Now** the enrolment step, which already declares an agent's actions, also declares its permission's spend limit, allowed topics and maximum chain depth. Bad declarations are refused with the reason (limits on a permission without the action, an empty topic list, a depth of zero or above 3), and the administrator's permission list shows them.
- ✅ When an agent starts a thread, the server reads those limits from the exact permission that admitted the request. An expired permission with a bigger spend limit no longer raises the ceiling. The topic limit applies alongside the agent's own declared interests, and the depth limit can be tighter than the site's 3.
- ✅ Tested end to end, including the expired-permission case that used to slip through. Removing the reader or the check makes the tests fail.

## 2026-09-23 — The limits an automatic-thread permission is supposed to carry have no way to be set (`SIGNOFF-REPAIR.11.4.7.2.1.5.3.2.3`)

`REASONBRAID-DOC-0137`. A decision; no code changed.

- 🔴 The roadmap says the permission to start threads automatically carries six limits: topic, audience, rate, depth, spend and side effects. I checked where each could come from. **There is no way to issue a permission with any of them**: no request issues a permission at all (only "list" and "revoke" exist), the enrolment step issues one with no limits, and every limited permission the tests use was written straight into the database by the test.
- 🔴 The one limit that is read today (spend) is read from the wrong place: the server takes the largest spend limit across all of an agent's permissions, including expired ones, instead of the permission that actually admitted the request — which the audit record already names.
- 🔴 "Audience" exists nowhere in the code, and no action that causes a side effect is tied to a thread, so two of the six limits have nothing to attach to yet.
- ✅ Decided, in order: first a way to declare the limits (at enrolment, the same trusted step that already declares an agent's actions), then reading them from the admitting permission, then the audience limit on the calls an automatic thread opens, and side effects when there is something to bound. A general permission-issuing service is a separate piece of the roadmap and is not started here.

## 2026-09-23 — An agent's declared capacity now limits how much work it is handed (`SIGNOFF-REPAIR.11.4.7.2.1.5.3.2.2.2`)

`REASONBRAID-REPAIR-0420`.

- 🔴 **Before:** an agent could declare "I take at most 2 jobs at a time", and the server would show it as "busy" when it held two — and still hand it a third. The number changed what was displayed, not what was delivered; the book said so twice.
- ✅ **Now** the server hands an agent at most its declared capacity minus what it already holds, using the same count that decides "busy". At capacity it is handed nothing until it finishes something; an agent that declares no capacity is handed everything, as before.
- ✅ Tested: three jobs queued for an agent declaring 2 — it is handed two, then nothing while it holds both, then the third when one finishes. Removing the limit makes the test fail.

## 2026-09-23 — Two agent settings that were stored and ignored now mean something, and bad values are refused (`SIGNOFF-REPAIR.11.4.7.2.1.5.3.2.2`)

`REASONBRAID-REPAIR-0419`.

- 🔴 **Before:** an agent's profile could declare "working hours" and a "wake policy", and the server stored any text for either — "never", "manual_only", anything — and then ignored both. Work was delivered and the agent could start threads regardless of what it had declared.
- ✅ **Now** each setting has an exact form the server refuses to violate (working hours as a UTC window such as `22:00-06:00`; wake policy `auto` or `manual_only`), and one rule that both surfaces obey: outside its hours, or under `manual_only`, an agent is handed no work and may not start a thread on its own. A value that somehow reached storage without passing that check holds the agent rather than being ignored, and says which field.
- ✅ Tested at both surfaces and at the write: bad values are refused by name, delivery is held and released by the clock and by the policy, and self-started threads are refused in the same words. Disabling the rule, or the check, makes the tests fail.
- ⚠️ Two follow-ups are open and owned: a positive concurrency number still does not cap how much work a node is handed, and an agent held by its policy still shows as "available" in the presence view, which touches the roadmap's fixed vocabulary of six states — that one is yours to decide.

## 2026-09-23 — Every thread view now gives the same answer for a thread you cannot see (`SIGNOFF-REPAIR.17`)

`REASONBRAID-REPAIR-0418`.

- 🔴 **Before:** asking about a thread that does not exist, or that belongs to another organisation, got two different answers depending on which view you asked. The thread and budget views said "not visible". The timeline said "no events" as if the thread existed, and the audit view listed the record of your own asking.
- ✅ **Now** all four views answer "not visible" in exactly the same words. Nothing had leaked, but a reader could tell "no such thread" from "not yours" by which view they asked.
- ✅ Tested: your own thread still answers on all four views; a foreign thread and a made-up id get the identical refusal on all four. The test failed on the old code at the timeline view, exactly as measured yesterday.

## 2026-09-23 — An agent can now start more than one automatic thread, and how often is limited (`SIGNOFF-REPAIR.11.4.7.2.1.5.3.2.1` with `SIGNOFF-REPAIR.5.2`)

`REASONBRAID-REPAIR-0417`.

- 🔴 **Before:** an agent could start exactly one automatic thread per tenant, ever — every later attempt silently returned the first thread. That bug was also the only thing stopping an agent from starting threads without limit, so it could not be fixed on its own.
- ✅ **Now** each automatic start carries its own key, so a retried delivery still returns the same thread but a new start creates a new one. And every agent has a limit on automatic starts per hour (1000 in the development setting), recorded like every other usage limit: a refused start is written down, an agent with no limit row is refused rather than let through, and a person's ordinary thread creation is not counted. Both changes are in one commit, so at no point was automatic starting unlimited.
- ✅ Tested end to end: two starts make two threads, a retry replays, the limit refuses at the ceiling, the refusal is recorded once, and an agent with no limit row is refused. Reverting either half makes the test fail.
- ⚠️ The hourly number is a development default, not a measured one. The limit is per agent; making it part of the grant itself is a later task.

## 2026-09-23 — The operator views are now proven served by the real server, not only present in the code (`SIGNOFF-REPAIR.4.6.1.7`)

`REASONBRAID-REPAIR-0416`.

- 🔴 The automatic check I added yesterday only proved each operator view was *written in the code*. Two views live in separate parts that the server must plug in at start-up, and forgetting to plug one in would have gone unnoticed.
- ✅ The server's parts are now assembled by one shared function. A new test assembles the server the same way and calls every operator view. Unplugging the backup view makes that test fail at once, while the old check stayed green, which was exactly the gap.

The entries before those above were rotated into reachable Git history at the
**forty-eighth rotation** (`SIGNOFF-REPAIR.11.4.1.6`, which owns this ledger’s rotation). The exact predecessor — this file as it
stood at the commit named below, which is the object every retired record was
checked against before this notice was written — is:

```bash
git show cc02a39c1b8667e26318afebfebe05c7978e445d:CHANGELOG.md
```

That snapshot is 93066 bytes and 681 lines, and contains 72 dated
entries; its Git blob is `a6a55b8b09dd5c24b9ed5a57a9a0d274624d7465` and its SHA-256 is
`a40ec0cbfb298b5460104f1d339a2fef2e08596884f6d32426ae4e82fdc81f33`. It carries the forty-seventh rotation's
notice in turn, and each earlier notice names the one before it, so the chain
walks all the way back. `docs/decisions/2026-09-09_changelog-rotation.md` holds
the first transition's evidence.

⛔ **16 record(s) rotated out, 57 kept, lossless** — every retired heading was retrieved from the
predecessor named above before this notice was written, and every figure in it was re-derived from that object with
`git rev-parse`, `git cat-file` and SHA-256 rather than typed. ⭐ The cut is DERIVED, not chosen: it retires whole
records until the ledger has at least 10 commits of runway at the p90 entry size measured over the last
60 non-rotation commits — because two rotations that stopped at the threshold instead left 344 and 296 bytes and
the first forced another rotation on the very next commit (`SIGNOFF-REPAIR.11.4.1.6`).
