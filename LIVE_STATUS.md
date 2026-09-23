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

The entries before those above were rotated into reachable Git history at the
**eighth rotation** (`SIGNOFF-REPAIR.11.4.1.6`, which owns this ledger’s rotation). The exact predecessor — this file as it
stood at the commit named below, which is the object every retired record was
checked against before this notice was written — is:

```bash
git show 4894da31f5983ded5b864a2e873f32b495639cdc:LIVE_STATUS.md
```

That snapshot is 51610 bytes and 361 lines, and contains 35 dated
entries; its Git blob is `88b333c6d8ba10f3feb24ba7ae411f2c7626ea36` and its SHA-256 is
`1d7fd915fb0fb6d3068ffc76c3b93004fdffd9f50f9c4f7d164d6eda5c4c1379`. It carries the seventh rotation's
notice in turn, and each earlier notice names the one before it, so the chain
walks all the way back. `docs/decisions/2026-09-09_changelog-rotation.md` holds
the first transition's evidence.

⛔ **26 record(s) rotated out, 10 kept, lossless** — every retired heading was retrieved from the
predecessor named above before this notice was written, and every figure in it was re-derived from that object with
`git rev-parse`, `git cat-file` and SHA-256 rather than typed. ⭐ The cut is DERIVED, not chosen: it retires whole
records until the ledger has at least 10 commits of runway at the p90 entry size measured over the last
60 non-rotation commits — because two rotations that stopped at the threshold instead left 344 and 296 bytes and
the first forced another rotation on the very next commit (`SIGNOFF-REPAIR.11.4.1.6`).
