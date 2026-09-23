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

## 2026-09-23 — Chains of automatically started threads are now limited (`SIGNOFF-REPAIR.11.4.7.2.1.5.3.1`)

`REASONBRAID-REPAIR-0415`.

- 🔴 **Before:** when an agent started a thread on its own, nothing recorded why, so nothing could stop a chain (A starts a thread that wakes B, B starts one that wakes C, and so on) or notice it looping back.
- ✅ **Now** an agent that starts a thread because of another thread names that thread. The server records how deep the chain is and which agents are on it. It refuses a chain deeper than 3, an agent re-joining its own chain, and an agent naming a thread it is not part of.
- ✅ Tested with a real chain of four agents, each invited into the thread it then builds on. Raising the limit to 4 makes the test fail.
- ⚠️ An agent can still start a *new* chain without naming a cause. How often it may do that is the next task (a rate limit). The known bug that currently lets an agent start only one thread per tenant will be fixed together with that limit, because until then it is the only brake.

## 2026-09-22 — Three postponed safeguards against automatic activity are now overdue (`SIGNOFF-REPAIR.11.4.7.2.1.5.3`)

`REASONBRAID-DOC-0134`. A decision; no code changed.

- Six safeguards against notification storms were postponed until specific things happened. **Three of those things have happened**, so three safeguards are now owed: limits on how deep a chain of automatically started threads can go (with loop detection), quiet hours, and a cap on the backlog of an offline machine. A fourth (circuit breakers) waits for a storm that could never be seen, because storm refusals are not recorded anywhere.
- ⛔ **The important part:** automatic thread-starting is currently kept in check only by a *bug* — a role can start one automatic thread per tenant, ever. Fixing that bug first would remove the only brake. So the depth and loop limits come first, and the bug fix is explicitly locked behind them.
- 🔴 An earlier task marked "done" had built 4 of the roughly 12 checks its own goal listed. The rest now have an owner.

## 2026-09-22 — Twelve "already done" verdicts re-checked: eight held, four were wrong (`SIGNOFF-REPAIR.11.4.7.2.1.5`)

`REASONBRAID-DOC-0133`. An audit of earlier conclusions; no code changed.

- An earlier review had marked 13 postponed items as "already delivered", mostly by checking that a file existed. I re-checked the 12 that nobody had re-verified, this time by reading the requirement, the code and the test that exercises it.
- ✅ **8 held.** 🔴 **4 were wrong, all in the same direction**: something called done was only partly done.
  - The registry that maps machines to agent roles was never built; a temporary development rule still stands in for it.
  - Certificates are issued, rotated and revoked, but the server does not require them at the connection level yet.
  - "Semantic matching" was claimed, but the matcher is rule-based; the semantic part is formally postponed.
  - "Any resource can be fetched" is overstated: the agent-assisted fetch path is never completed.
- ✅ Every correction is written at the row, and every gap found now has its own tracked task.
- ⚠️ One of them may be urgent. A safeguard against automatic thread-starting running away (depth and cycle limits) was postponed until "the first automatically started thread", and that feature now exists. It is next.

## 2026-09-22 — A thread with no voting rule can no longer be closed as if a vote was counted (`SIGNOFF-REPAIR.8.1.1.5`)

`REASONBRAID-REPAIR-0414`.

- 🔴 **Before:** a thread that never declared a voting rule could still be closed as "accepted unanimously", "accepted with recorded objections" or "no quorum", describing a vote count that never happened. Threads *with* a non-voting rule already refused these.
- ✅ **Now** those three are refused on a thread with no rule, with a message explaining how to get a real count: declare a counting rule when creating the thread. All other ways of closing still work.
- ⏸️ Making a rule required for every thread is not possible yet, because existing deployments' charters would refuse every thread. That step is tracked with a one-query condition that says when it becomes safe.
- ✅ Four existing test cases were reworked for what they test. My first search missed one; the test run caught it.

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

The entries before those above were rotated into reachable Git history at the
**seventh rotation** (`SIGNOFF-REPAIR.11.4.1.6`, which owns this ledger’s rotation). The exact predecessor — this file as it
stood at the commit named below, which is the object every retired record was
checked against before this notice was written — is:

```bash
git show 5f1fe88e1ced09ed73775d74928996a039c9e0da:LIVE_STATUS.md
```

That snapshot is 51282 bytes and 285 lines, and contains 23 dated
entries; its Git blob is `d82fd2eeb45b579118e3c1205c5c5e62fef5ca59` and its SHA-256 is
`e913c0070fe849b376fdab3dc270e435b4f7e89d7e03a04be27ba13620ac4efa`. It carries the sixth rotation's
notice in turn, and each earlier notice names the one before it, so the chain
walks all the way back. `docs/decisions/2026-09-09_changelog-rotation.md` holds
the first transition's evidence.

⛔ **15 record(s) rotated out, 9 kept, lossless** — every retired heading was retrieved from the
predecessor named above before this notice was written, and every figure in it was re-derived from that object with
`git rev-parse`, `git cat-file` and SHA-256 rather than typed. ⭐ The cut is DERIVED, not chosen: it retires whole
records until the ledger has at least 10 commits of runway at the p90 entry size measured over the last
60 non-rotation commits — because two rotations that stopped at the threshold instead left 344 and 296 bytes and
the first forced another rotation on the very next commit (`SIGNOFF-REPAIR.11.4.1.6`).
