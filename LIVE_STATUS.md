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

## 2026-09-24 — A budget hold for a lost answer now stays in place (`SIGNOFF-REPAIR.4.5.1`)

`REASONBRAID-REPAIR-0483`. The first fix from the budget review (`REASONBRAID-DOC-0156`).

- 🔴 **Before:** when a provider's answer was lost, we could not know whether the call was charged, yet its budget hold expired after ten minutes and the money could be lent to other work.
- ✅ **Now:** when a machine reports that it cannot retry such a job without permission, the server marks the job's hold "outcome unknown", and the hold keeps counting after its window. The limit check, the spending cut-off and the usage report share one rule, and the thread's budget page shows the mark.
- 🔴 **Also found:** a spending cut-off set on only some measures (say, calls) trips on the very first job, because every job also asks for tokens and time (`.4.5.6`). Nothing yet releases a marked hold after an operator's ruling (`.4.5.1.1`).
- ✅ Tested: the new test failed on the old code and passes now; five deliberately broken versions were all caught; six live suites pass; strict lint clean. The stall hunt (`.11.26`) ran 120 times with no stall.
- Technical: migration `0107` `budget_reservations.outcome_unknown_at`; `budget::holding!` read by admission, `check_spend_breaker_in_tx` and `admin_usage`; `hold_for_unknown_outcome_in_tx` from the `work_dead_lettered` fold on `RETRY_REQUIRES_AUTHORIZATION`; `GET /v1/threads/{id}/budget` row field `outcome_unknown_at`.

## 2026-09-24 — Budget review: a held allowance for a lost answer quietly expires after ten minutes (`SIGNOFF-REPAIR.4.5`)

`REASONBRAID-DOC-0156`. A review of the budget and quota rules against the code; no code changed.

- 🔴 **Found:** when a provider call's answer is lost, its budget hold is meant to stay in place in case the call did charge. In fact every hold stops counting ten minutes after it was placed, so the allowance can be lent out again. Yesterday's notes and the operator's advice said "the original stays held"; that is only true for ten minutes. The book, the command-line page and the runbook now say so; `.4.5.1` fixes the rule itself.
- 🔴 **Found:** a machine's reported usage is added up without an overflow check, so an absurd report could crash the server or make the budget look nearly empty (`.4.5.2`). Two admissions at the same moment can both pass a limit only one should (`.4.5.3`).
- ✅ **Holds:** a settled hold cannot be settled twice; a machine can only settle the hold the server recorded (a proof test is owed, `.4.5.4`). The server's outbound queue has no worker yet (`.4.5.5`).
- Technical: held query `r.status = 'active' AND r.expires_at > $2`; dispatch hold `Duration::minutes(10)`; `BudgetDimensions::add` plain `u64 +`, no `[profile.release]`; no `FOR UPDATE`/advisory lock in `budget.rs`/`quota.rs`; `claim_ready` has no production caller.

## 2026-09-24 — Operators can re-run a job whose outcome is unknown, from the screen's advice and the command line (`SIGNOFF-REPAIR.4.4.7.2.3`)

`REASONBRAID-REPAIR-0482`. Completes the machine-recovery work found by `REASONBRAID-DOC-0153`.

- ✅ **Now:** the operator's list of recovery options shows "ask again, accepting a possible duplicate charge" as available. It names who takes it (the tenant's administrator) and how. The command-line tool can do it: `rb node replay … --allow-possible-duplicate --reason "…"`. It insists on a reason, and refuses a reason given without the approval.
- ⭐ **The whole machine-recovery review is done:** every gap found on 23 September is closed. That covers lost answers, answers refused or wedged, stuck machines, timeouts, retries, budgets and the operator's views.
- ✅ Tested: the availability check failed on the old code and passes now; the command-line rules are checked, and all seven deliberately broken versions were caught; live suites pass; strict lint clean.

## 2026-09-24 — An administrator can now authorize re-running a job whose outcome is unknown (`SIGNOFF-REPAIR.4.4.7.2.2`)

`REASONBRAID-REPAIR-0481`.

- 🔴 **Before:** when a provider call's outcome was unknown, the machine rightly refused to run the job again on its own. But nothing let anyone approve a re-run while accepting the risk of paying twice, so the job stayed stuck.
- ✅ **Now:** the tenant's administrator can approve the re-run with a stated reason. The job gets its own new budget hold (the original stays reserved in case the first call did charge), the machine runs it again, and the answer lands. The approval is recorded in the audit trail under its own name, applies only to jobs stuck for exactly this reason, and is refused if the conversation's budget has no room.
- ⚠️ Next: the command-line action, and marking the option available on the operator's screen.
- ✅ Tested: a full live run (a lost answer, the approval, the re-run, the answer landing) that could not happen before; the audit vocabulary checks; five further live suites; both deliberately broken versions were caught; strict lint clean.

## 2026-09-24 — A machine can now receive permission to retry a job whose outcome is unknown (`SIGNOFF-REPAIR.4.4.7.2.1`)

`REASONBRAID-REPAIR-0480`. The first part of the "ask again, accepting a possible duplicate" action.

- 🔴 **Before:** once a machine had a job, nothing sent later could change it. That included permission to retry a job whose outcome is unknown, so even an authorized retry could never reach the machine.
- ✅ **Now:** when a job is sent again with a fresh approval, the machine takes two things from it: whether a possible duplicate is allowed, and which budget hold to run under. It keeps everything that says what the job is. A resend can extend what a machine may risk; it can never change what it was asked to do.
- ⚠️ Next: the server-side action that sends such a resend, then the command-line action and a full end-to-end check.
- ✅ Tested: two new checks failed on the old code and pass now; 112 machine tests and two live suites pass; every deliberately broken version was caught; strict lint clean.

## 2026-09-24 — The recovery options now say which ones actually exist (`SIGNOFF-REPAIR.4.4.7.1`)

`REASONBRAID-REPAIR-0479`.

- 🔴 **Before:** the list of recovery options for a job with an unknown outcome included "ask again, accepting a possible duplicate charge", as if it could be done. It could not, and the runbook and the book said the same.
- ✅ **Now:** every option says whether it is available today. That one says it is not, why, and what will change it. The runbook and the book describe what an operator can really do: a revision can be requested again with a new challenge; a first answer cannot be requested again yet.
- ⚠️ Next: building the "ask again, accepting a possible duplicate" action itself.
- ✅ Tested: a new check failed on the old code and passes now; the live suite passes; a deliberately broken version was caught.

## 2026-09-24 — Checked: operators were told to use a recovery action that does not exist (`SIGNOFF-REPAIR.4.4.7`)

`REASONBRAID-DOC-0155`. A review; no code changed.

- 🔴 **Found:** when a job's outcome is unknown, the operator's screen, the runbook and the book all suggest "ask again, accepting the risk of a duplicate charge". Nothing lets anyone do that. For a first answer (as opposed to a revision) there is no way at all to ask again today.
- ⚖️ **Decided:** first make the screen, runbook and book tell the truth about which actions are available now; then build the "ask again, accepting a possible duplicate" action properly, with its own separate budget, because the roadmap names it as one of the four ways to handle an unknown outcome.

## 2026-09-24 — The charter lookup no longer blames the caller for server faults (`SIGNOFF-REPAIR.4.4.10.1.2`)

`REASONBRAID-REPAIR-0478`.

- 🔴 **Before:** looking up a governance charter answered any database problem as "your request is invalid", and quoted the database's internal error message back to the caller. A real server fault was blamed on the user, and internal details leaked.
- ✅ **Now:** a server fault is reported as a server fault, with the details kept in the server's log. A request containing the null character gets the clear "cannot be stored" reply. Publications were checked too: that character cannot reach their storage, because the input is validated first.
- ✅ Tested: a new check (the null character, plus a simulated database fault) failed on the old code and passes now; three live suites pass; both deliberately broken versions were caught; strict lint clean.

## 2026-09-24 — More places now say clearly when input can never be stored (`SIGNOFF-REPAIR.4.4.10.1.1`)

`REASONBRAID-REPAIR-0477`.

- 🔴 **Before:** in seven places (agent profiles, snapshots, derivations, assessments, references, quotas and evaluations), input holding the null character was still answered "internal server error" rather than the clear "cannot be stored" reply.
- ✅ **Now:** all seven give the clear, permanent reply through one shared rule; genuine server faults still report as server faults.
- ⚠️ Tracked next: two remaining places (publications and charters) that lose the information needed to tell the two apart.
- ✅ Tested: a new check failed on the old code (a profile with that character) and passes now; the shared rule is checked against every kind of database error; six live suites pass; strict lint clean.

## 2026-09-24 — Time spent on jobs now counts against a conversation's time budget (`SIGNOFF-REPAIR.4.4.6.1`)

`REASONBRAID-REPAIR-0476`.

- 🔴 **Before:** each conversation has a time budget (10 minutes by default) alongside its call and token budgets. Time was never charged once a job finished, so the budget only limited how many jobs could run at the same moment, never the total. However long the jobs took altogether, the time budget never ran out.
- ✅ **Now:** each machine times every job, rounding up to whole seconds, and reports the time alongside the tokens. The server charges it, so a conversation's time budget runs out like its token budget does.
- ✅ Tested: two checks (one on the machine, one against the real server) failed on the old code and pass now; 199 core and machine tests and three live suites pass. The deliberately broken versions exposed a gap ("always charge one second" went unnoticed because every test job was quick); a longer job was added, and all are now caught. Strict lint clean.

## 2026-09-24 — The web console's inbox panel works (`SIGNOFF-REPAIR.4.4.2.2`)

`REASONBRAID-REPAIR-0475`. A defect found by the previous fix.

- 🔴 **Before:** the console's "Inspect inbox" panel had never worked. It asked the server with the wrong parameter name, so every request was refused, and it tried to show a field that does not exist. The check meant to keep the console honest had itself been written with the wrong name, so it agreed with the mistake.
- ✅ **Now:** the panel works and shows each job's delivery state, whether its answer was refused (and why), and any quarantine. The check now tests the panel against the server's own definitions, so the two cannot drift apart unnoticed again.
- ✅ Tested: the new check failed on the old console, once for each of the two mistakes, and passes now; strict lint clean.

## 2026-09-24 — Operators can now see when a finished job's answer was refused (`SIGNOFF-REPAIR.4.4.2.1`)

`REASONBRAID-REPAIR-0474`.

- 🔴 **Before:** in the operator's inbox view, a job whose answer the server refused (for example because the conversation had closed) looked exactly like one whose answer was accepted. Both showed as "done".
- ✅ **Now:** each job in the view shows whether its answer was refused, and why. Accepted answers show nothing extra. The same information reaches the assistant-facing (MCP) view.
- 🔴 **Found along the way (tracked, fixed next):** the web console's inbox panel has never worked. It asks the server in the wrong way, and it reads a field that does not exist.
- ✅ Tested: two checks failed on the old code and pass now; six live suites pass; a deliberately broken version was caught; strict lint clean.

## 2026-09-24 — An answer containing the null character now gets through, marked (`SIGNOFF-REPAIR.4.4.10.3`)

`REASONBRAID-REPAIR-0473`. The last of the three fixes for the lock-out measured by `REASONBRAID-DOC-0154`.

- 🔴 **Before:** an answer containing the invisible "null" character could never be stored, so the whole paid answer was lost over one character.
- ✅ **Now:** the machine swaps each null character for the standard "unreadable character" symbol (�), which shows exactly where it was, and the answer records how many were swapped. Nothing else in the answer changes, and answers without the character are untouched.
- ⚖️ **A decision taken for you to review:** the alternative was to reject such an answer outright, keeping the principle that answers pass through unchanged but losing the work. The reasoning is recorded, and switching is a one-line change.
- ✅ Tested: a new check failed on the old code (the answer was lost) and passes now; the real server stores such an answer as a contribution; 108 machine tests pass; all eight deliberately broken versions were caught; strict lint clean.

## 2026-09-24 — A machine no longer locks itself out on an answer the server can never accept (`SIGNOFF-REPAIR.4.4.10.2`)

`REASONBRAID-REPAIR-0472`. The second of the three fixes for the lock-out measured by `REASONBRAID-DOC-0154`.

- 🔴 **Before:** when the server refused an answer permanently (because of what the answer contained), the machine treated it as a broken connection. It reconnected, resent the same answer, was refused again, and never got back to work.
- ✅ **Now:** the machine records the refusal (the server's reason included), stops offering that answer, and carries on with its other work. Only refusals about the answer's own contents count as permanent. Login problems, outages and version mismatches are still handled by reconnecting.
- ⚠️ Tracked next: the answer itself is still lost when it contains the null character. The last fix decides what the machine does with that character.
- ✅ Tested: two new checks (one on first sending, one on resending after a reconnect) failed on the old code and pass now; 105 machine tests, five live suites and the two-machine demonstration pass; all ten deliberately broken versions were caught; strict lint clean.

## 2026-09-24 — The server now says clearly when an answer can never be stored (`SIGNOFF-REPAIR.4.4.10.1`)

`REASONBRAID-REPAIR-0471`. The first of the three fixes for the lock-out measured by `REASONBRAID-DOC-0154`.

- 🔴 **Before:** input holding the invisible "null" character, which the database cannot store, was answered "internal server error", a reply that looks like a temporary outage. A machine given that reply for its answer kept retrying for ever.
- ✅ **Now:** such input is refused with a clear, permanent "cannot be stored" reply, and nothing of it is kept. This covers both the machines' channel and the command interface people use. A genuine server fault on normal input still reports as a server fault, so the two are never confused.
- ⚠️ Tracked next: the machine side (stop retrying anything refused permanently); and seven less-used command paths that still give the old reply.
- ✅ Tested: a new check failed on the old code and passes now; eight live suites pass; all five deliberately broken versions of the new rule were caught, after the first round showed a missing check (a real server fault was being confused with bad input), which was added; strict lint clean.

## 2026-09-24 — Measured: a single invisible character in an answer can lock a machine out for good (`SIGNOFF-REPAIR.4.4.10`)

`REASONBRAID-DOC-0154`. A check of a suspected risk; it turned out to be real.

- ✅ **Confirmed safe:** the largest answer a machine can now produce (256 KB, `.4.4.6`) is accepted by the server, even in its most expensive encoding. Double that is refused, so the server's limit really exists.
- 🔴 **Found:** an answer containing one particular invisible character (the "null" character, which the database cannot store) is refused by the server with a message that looks like a temporary outage. The machine keeps retrying the same answer and can never get back to work. A provider can produce that character; the test provider's own sample of garbled output contains it.
- ⚖️ Now three tracked fixes, in order: the server rejects such an answer clearly and permanently; a machine stops retrying anything the server rejects permanently; and a machine decides up front what to do with that character.

## 2026-09-24 — A provider that never answers no longer freezes the machine, and answers have a size limit (`SIGNOFF-REPAIR.4.4.6`)

`REASONBRAID-REPAIR-0470`. The sixth recovery gap found by `REASONBRAID-DOC-0153`.

- 🔴 **Before:** every job came with a time limit, but nothing enforced it. A provider that took a job and never answered froze the machine for good. Answers could also be any size, including sizes the server would refuse to accept.
- ✅ **Now:** when the time limit passes, the machine tells the provider to stop and records the job as "outcome unknown", with the reason. It never guesses the job failed or succeeded, because the provider may have done the work. An answer larger than 256 KB is stopped and recorded as failed, naming the limit; a partial answer is never passed off as a whole one.
- ✅ Tested: two checks failed on the old code (a frozen job, and an oversized answer accepted) and pass now, plus four more; 102 machine tests, four live suites and the two-machine demonstration pass. The deliberately broken versions first exposed a gap: nothing checked that the provider was actually told to stop. The checks were strengthened until all were caught. Strict lint clean.
- ⚠️ Tracked next: whether a machine can get stuck on an answer the server refuses outright; and charging a job's elapsed time to its budget.

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

The entries before those above were rotated into reachable Git history at the
**twelfth rotation** (`SIGNOFF-REPAIR.11.4.1.6`, which owns this ledger’s rotation). The exact predecessor — this file as it
stood at the commit named below, which is the object every retired record was
checked against before this notice was written — is:

```bash
git show fcb2a7c9c532fb7a014d59e633d6e098dfa51085:LIVE_STATUS.md
```

That snapshot is 52353 bytes and 362 lines, and contains 36 dated
entries; its Git blob is `7d5b77fb5218cefe08c86ab8bac5f1e627698754` and its SHA-256 is
`da4cdaf62caac258e0bea9c6a70337386e6b61ef0ad1a0c9b97e203ace771356`. It carries the eleventh rotation's
notice in turn, and each earlier notice names the one before it, so the chain
walks all the way back. `docs/decisions/2026-09-09_changelog-rotation.md` holds
the first transition's evidence.

⛔ **12 record(s) rotated out, 25 kept, lossless** — every retired heading was retrieved from the
predecessor named above before this notice was written, and every figure in it was re-derived from that object with
`git rev-parse`, `git cat-file` and SHA-256 rather than typed. ⭐ The cut is DERIVED, not chosen: it retires whole
records until the ledger has at least 10 commits of runway at the p90 entry size measured over the last
60 non-rotation commits — because two rotations that stopped at the threshold instead left 344 and 296 bytes and
the first forced another rotation on the very next commit (`SIGNOFF-REPAIR.11.4.1.6`).
