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

## 2026-09-24 — A machine that cannot reconnect now waits longer between tries (`SIGNOFF-REPAIR.4.4.5.3`)

`REASONBRAID-REPAIR-0469`. Completes the fifth recovery gap found by `REASONBRAID-DOC-0153`.

- 🔴 **Before:** a machine that could not reconnect tried again every second, for ever. A server down for an hour met 3,600 reconnect attempts from each machine.
- ✅ **Now:** the wait doubles after each failure (1, 2, 4 … seconds) up to a minute, and goes back to one second once a reconnect succeeds.
- ✅ Tested: the waiting rule has its own check; 96 machine tests and the two-machine demonstration pass; three deliberately broken versions were all caught; strict lint clean.

## 2026-09-24 — A delivered job now shows as done, and stops counting against the machine's capacity (`SIGNOFF-REPAIR.4.4.9`)

`REASONBRAID-REPAIR-0468`. A defect found while testing the previous fix.

- 🔴 **Before:** a job a real machine had finished and delivered never showed as "done" in the operator's inbox view, only "received". Worse, the same check decides how busy a machine is, so every finished job kept counting as "in progress". A machine allowed N jobs at a time stopped getting new work after its first N, until old records were cleaned out. The tests had not noticed because they built their fake results in a shape no real machine produces.
- ✅ **Now:** the server recognises a finished job by the job the answer names, so it shows as done and frees its capacity slot at once. The tests now use results shaped the way a real machine sends them, plus one end-to-end check with a real machine.
- ✅ Tested: five checks failed on the old code (two of them the capacity checks) and pass now; six live suites and the two-machine demonstration pass; a deliberately broken version was caught; strict lint clean.

## 2026-09-24 — Finished jobs are no longer reported as undeliverable (`SIGNOFF-REPAIR.4.4.8`)

`REASONBRAID-REPAIR-0467`. A defect found while testing the previous fix.

- 🔴 **Before:** one round after a job finished successfully, the machine reported it as "undeliverable", and the server filed it in the dead-letter pile. An operator looking at the inbox saw every successful job marked as failed.
- ✅ **Now:** the machine treats a finished job as finished and says nothing more about it. The server also refuses on its own to file a job as undeliverable once its answer has arrived, whoever sends the report.
- 🔴 **Found along the way (now tracked, fixed next):** a successfully delivered job shows as "received" instead of "done" in the operator's inbox view.
- ✅ Tested: two new checks (one on the machine, one against the real server) failed on the old code and pass now; 185 core and machine tests, six live suites and the two-machine demonstration pass; three deliberately broken versions were all caught; strict lint clean.

## 2026-09-23 — A dropped connection or one unanswerable reply no longer shuts a machine down (`SIGNOFF-REPAIR.4.4.5.2`)

`REASONBRAID-REPAIR-0466`. The second part of the fifth recovery gap found by `REASONBRAID-DOC-0153`.

- 🔴 **Before:** three ordinary problems stopped a machine's program entirely: the connection dropping while it sent an answer, a provider reply going missing, and one job with unreadable instructions. The last also left every job after it in the same round undone.
- ✅ **Now:** a dropped connection makes the machine reconnect and resend. A missing reply is recorded as "unknown" and left for a person to decide, as designed. A job with unreadable instructions is reported once as undeliverable, and the other jobs carry on.
- 🔴 **Found along the way (now tracked, fixed next):** every job that finishes successfully is wrongly reported as "undeliverable" one round later.
- ✅ Tested: three new checks failed on the old code and pass now; 94 machine tests, six live suites and the two-machine demonstration pass; four deliberately broken versions were all caught; strict lint clean.

## 2026-09-23 — A machine no longer waits for ever on a server that stops answering (`SIGNOFF-REPAIR.4.4.5.1`)

`REASONBRAID-REPAIR-0465`. The first part of the fifth recovery gap found by `REASONBRAID-DOC-0153`.

- 🔴 **Before:** if the server accepted a machine's connection and then went silent, the machine waited for ever. It stalled completely, and because nothing reported an error, it never tried to reconnect. Measured: still waiting after 45 seconds, with no end in sight.
- ✅ **Now:** the machine gives up after 10 seconds trying to connect, or 30 seconds waiting for an answer. It then treats the silence like any other broken connection and reconnects. Measured: it gives up at 30 seconds exactly.
- ✅ Tested: the new checks use a stand-in server that accepts and never replies; 91 machine tests and three live suites pass; two deliberately broken versions were caught (one more could not be built); strict lint clean. One gap is stated in the record: only a 30-second check, run once and not kept, would notice the unbounded client being put back by hand.

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

The entries before those above were rotated into reachable Git history at the
**tenth rotation** (`SIGNOFF-REPAIR.11.4.1.6`, which owns this ledger’s rotation). The exact predecessor — this file as it
stood at the commit named below, which is the object every retired record was
checked against before this notice was written — is:

```bash
git show 298c90ad1f22e475822c853b40e502919398a64b:LIVE_STATUS.md
```

That snapshot is 52212 bytes and 330 lines, and contains 31 dated
entries; its Git blob is `7431cd41afed44e1fa575a1cb88ea888bd042842` and its SHA-256 is
`c593ea0be5417483d18e24d0d8e5f189b6394844b03399c8d010f1b5e11e1b0f`. It carries the ninth rotation's
notice in turn, and each earlier notice names the one before it, so the chain
walks all the way back. `docs/decisions/2026-09-09_changelog-rotation.md` holds
the first transition's evidence.

⛔ **10 record(s) rotated out, 22 kept, lossless** — every retired heading was retrieved from the
predecessor named above before this notice was written, and every figure in it was re-derived from that object with
`git rev-parse`, `git cat-file` and SHA-256 rather than typed. ⭐ The cut is DERIVED, not chosen: it retires whole
records until the ledger has at least 10 commits of runway at the p90 entry size measured over the last
60 non-rotation commits — because two rotations that stopped at the threshold instead left 344 and 296 bytes and
the first forced another rotation on the very next commit (`SIGNOFF-REPAIR.11.4.1.6`).
