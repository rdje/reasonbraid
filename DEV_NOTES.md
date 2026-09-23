# DEV_NOTES.md

## 2026-09-23 — Agents that set no availability were getting no work; fixed (`SIGNOFF-REPAIR.11.4.7.2.1.5.3.2.2.3`)

`REASONBRAID-REPAIR-0452`.

- 🔴 **Before:** an agent whose profile didn't mention availability at all (the most common case) was sent **no work**, while the "who is available" listing showed it as available. Its profile stored "no availability" as an empty value, and the delivery code misread that as a broken setting and held everything back. The tests never noticed because they used a slightly different "empty" shape than real profiles do.
- ✅ **Now:** delivery and the availability listing read the setting through one shared function, so an agent with no availability set gets its work, and the two can't disagree again.
- ✅ Found while testing the next feature, which it was blocking; that work was set aside safely and resumes next.
- ✅ Tested: a new test using exactly what a real profile stores failed on the old code (no work delivered) and passes now; recreating the old misreading was caught; all delivery suites pass.
- Technical: `wake::stored(block)` (None | JSON null → `Ok(None)`, typed → `Ok(Some)`, else `Err(FormatError::Block)`) replaces the replay's inline parse and `presence::hold_from_stored`'s own null test. Control `a_profile_without_availability_holds_nothing_back` (node_channel) + unit `a_stored_null_block_declares_nothing`; mutant (null read as a block) caught at both. Introduced by REPAIR-0419; REPAIR-0439's null-safe reader never reached the replay.

## 2026-09-23 — A partner agent's answer from its home machine is now credited to it (`SIGNOFF-REPAIR.5.3.5.3.1.4`)

`REASONBRAID-REPAIR-0451`. Completes "a partner's agent runs on its home machine" (`SIGNOFF-REPAIR.5.3.5.3.1`).

- 🔴 **Before:** when the partner's machine sent back the agent's answer, the server credited it to the partner machine's *own* agent instead of the imported one. That agent isn't part of the conversation, so the answer was rejected.
- ✅ **Now:** the answer is credited to the agent the job was for, which the server reads from its own record of the job, not from anything the machine says. So the contribution appears in the conversation under the imported agent, with the importing organisation's permissions. If the partnership ended after the job was sent, the late answer is refused.
- ✅ The whole path now works end to end: import → listed in the directory → seated on a panel → work delivered to the partner's machine → checked against the right organisation's revocations → answer credited correctly. What remains is an audit receipt on both sides for each delivery (next task).
- ✅ Tested: the new check failed on the old code (the answer was refused as the wrong agent) and passes now; switching off the "does this machine still run this agent" check was caught; four suites pass.
- Technical: `apply_node_result_in_tx` reads the acting role from the stored inbox row's `agent_role` (fallback: the node id, the dev rule) and, after the idempotency claim, requires `role_execution(role).node_id = node_id` (else `unauthorized`, stored as the rejection). The `.5.3.5.3.1` census missed this site (it parses the node id rather than joining on it). Control grown with a real node-crate channel for the origin node; mutant M1 (runs-here check off) caught.

## 2026-09-23 — A partner's agent now receives its work on the partner's own machine (`SIGNOFF-REPAIR.5.3.5.3.1.3`)

`REASONBRAID-REPAIR-0450`.

- 🔴 **Before:** a partner agent bound to its home machine could join a conversation, but the work that followed was addressed to a machine that doesn't exist and sat there unread.
- ✅ **Now:** the work goes to the partner's machine, still as the importing organisation's job, so that machine checks it against the importing organisation's revocations (the previous step made that possible). The "too much unread work" safety limit is counted on the machine that will actually hold it. If the partnership has ended, accepting new work is refused with a clear reason instead of silently queueing it.
- ⚠️ **Not yet:** the partner machine's *answer* is still credited to the wrong identity and rejected. That's the next task, and the book says so.
- ✅ Tested: the new checks failed on the old code and pass now; two deliberately broken versions (wrong inbox; limit counted on the wrong machine) were each caught; the profile, work, channel and invitation suites pass.
- Technical: `dispatch_work_in_tx` resolves `role_execution.node_id` first (none → `409 invalid_transition` *runs on no node*), counts `undelivered_in_tx` and enqueues on the resolved node; the item keeps the dispatching tenant, admission and decision epoch. Control grown: the origin test's work, backlog and lapsed-binding arms; mutants M1 (enqueue on role) and M2 (backlog on role) caught. Opened `.5.3.5.3.1.4` (the result path).

## 2026-09-23 — A machine working for two organisations now checks each job against the right organisation's revocations (`SIGNOFF-REPAIR.5.3.5.3.2`)

`REASONBRAID-REPAIR-0449`.

- 🔴 **Before:** a machine only ever knew one organisation's revocation counter, its own, and checked every job against it before running it. That was harmless while a machine only ever worked for its own organisation. But a partner's agent running on its home machine (the feature being built) would put two organisations' jobs on one machine. Then a revocation in one organisation could wrongly block the other's valid work, or let stale work through.
- ✅ **Now:** the server tells the machine the current counter of every organisation whose jobs it holds, and the machine checks each job against **its own** organisation's counter. A job from an organisation the machine hasn't been told about yet is refused, never judged by someone else's counter.
- ✅ This is the safety step that had to come first; sending partner agents their actual work is next.
- ✅ Tested: new tests on both the machine side and the server side; recreating the old single-counter behaviour on each side made those tests fail; every machine- and channel-related suite passes.
- Technical: server `node_channel.rs` `epochs_and_server_time` (own tenant ∪ inbox tenants, one statement with `clock_timestamp()`), `revocation_epochs: BTreeMap<String, i64>` replaces `revocation_epoch` in `HandshakeResponse`/`PollResponse` on both sides; node `migrations/0004_tenant_epochs.sql`, `Journal::{revocation_epoch_for, set_revocation_epochs, set_revocation_epoch_for}`, the gate reads `item.tenant_id`'s epoch. Controls `each_command_is_judged_by_its_own_tenants_epoch` (node) and `the_handshake_and_poll_carry_every_held_tenants_epoch` (server); pre-repair mutants on both sides caught.

## 2026-09-23 — The "who is available" listing now honours directory-sharing agreements, like the search does (`SIGNOFF-REPAIR.5.1.6`)

`REASONBRAID-REPAIR-0448`. Closes the directory-privacy work again (`SIGNOFF-REPAIR.5.1`).

- 🔴 **Before:** two organisations that agreed to share their directories saw each other's agents in more detail in the search than in the "who is available" listing, which ignored the agreement. Nothing was over-shared; the listing showed less than the book promised.
- ✅ **Now:** both use one shared rule, so they always agree: under an agreement, a partner's agents show the organisation-level detail; without one, only the public basics; never the private fields.
- ✅ Tested: the new test failed on the old code and passes now. A deliberately broken rule that over-shares was caught by three tests at once, covering both the search and the listing.
- Technical: `api.rs` `class_toward_foreign_tenant(pool, reader_tenant, tenant, memo)` replaces the match's inline agreement lookup and classifies the presence's foreign entries (the pool taken before the loop because the loop shadows `state`). Control `the_presence_listing_widens_to_the_tenant_view_under_a_directory_agreement`; mutant M1 (always `Tenant`) caught by 3 controls.

## 2026-09-23 — The directory now lists a partner's agent that runs on the partner's machine (`SIGNOFF-REPAIR.5.3.5.3.1.2`)

`REASONBRAID-REPAIR-0447`.

- 🔴 **Before:** an imported partner agent bound to the partner's own machine could join calls, but the directory search and the "who is available" listing never showed it, because both only listed machines, and it has no machine of its own here.
- ✅ **Now:** both list it, under the importing organisation, showing which machine it runs on. A third organisation looking at the listing sees the agent but is **not** told which partner's machine it runs on, since that would reveal who works with whom. When the partnership ends, the agent disappears from both lists.
- 🔎 **Found and scheduled:** the "who is available" listing never gives a partner organisation the wider view that a directory-sharing agreement promises (the search does). Nothing is over-shared; it shows less than documented. Owned as the next task.
- ✅ Tested: the new checks failed on the old code and pass now; two deliberately broken versions (revealing the machine to everyone; filing the agent under the wrong organisation) were each caught; the profile suite passes (88).
- Technical: `api.rs` `DIRECTORY_ROWS` (node rows ∪ origin-bound rows via `role_execution` where `node_id <> role_id`), `DIRECTORY_COLUMNS`, `DirectoryRow`, shared by `directory_match` (candidates keyed by `role_id`) and `directory_presence` (entries gain `role_id`; a foreign reader's entry omits `node_id` when it differs from `role_id`; the duplicated `"hold"` key removed). Control grown: `an_origin_bound_identity_runs_on_the_origin_node_while_the_agreement_stands`; mutants M2 (node disclosed) and M3 (origin tenant) caught. Opened `.5.1.6`.

## 2026-09-23 — A partner's agent can be recruited to run on the partner's own machine (`SIGNOFF-REPAIR.5.3.5.3.1.1`)

`REASONBRAID-REPAIR-0446`.

- 🔴 **Before:** when an organisation imported a partner's agent, the imported agent could only act if the importing organisation set up a machine for it themselves. There was no way to say "the partner's agent keeps running on the partner's machine".
- ✅ **Now:** the import can choose `origin`: the agent keeps running on the partner's own machine, and it can join and be seated on calls without the importer enrolling anything. This works only while both organisations' recruitment agreement is in force. If either side withdraws, the agent is immediately treated as having no machine, and it never silently falls back to a local one. The import is refused if the partner's agent has no machine at all.
- ⚠️ **Not yet:** the directory search doesn't list such an agent yet (next step), and sending it actual work waits on a safety change to how a machine checks revocations for two organisations at once. Both are tracked, and the book says so.
- ✅ Tested: the new test failed on the old code and passes now; three deliberately broken versions (ignoring the agreement, looking up the wrong machine, reading the wrong machine's facts) were each caught; the profile, card, federation, audit and upgrade suites pass.
- Technical: `migrations/0100_card_imports_executes_on.sql` (`card_imports.executes_on`, no FK to `nodes` by design; view `role_execution` — own id when unbound, the bound node while both recruitment directions are accepted and unexpired, else NULL); `cards::CardExecution`; `ImportCardRequest.execution`; `CardImportResult::NoOriginNode`; `profile_admin::Submitted`; `respondent_candidate` joins `role_execution` → `node_presence`; the close reads incarnations by the resolved node; `imported_from` gains `execution`/`executes_on`/`runs_on`. Control `an_origin_bound_identity_runs_on_the_origin_node_while_the_agreement_stands`; mutants M1 (agreement ignored), M2 (dev-rule respond), M3 (close reads local incarnation) caught. `.5.3.5.3.1` split: `.1.2` match/presence, `.1.3` dispatch after `.5.3.5.3.2`.

## 2026-09-23 — An agent's owner now sees its whole profile; the privacy defaults are confirmed and explained (`SIGNOFF-REPAIR.5.1.5`)

`REASONBRAID-REPAIR-0445`. Closes the directory-privacy work (`SIGNOFF-REPAIR.5.1`).

- 🔴 **Before:** an agent and its owner were promised "the full profile" but never saw two parts of it: which running instance the agent is, and its own privacy settings. Separately, a code comment claimed every profile field is private unless the agent says otherwise, which was never true, and another comment described the privacy rule backwards.
- ✅ **Now:** the agent and its owner see both parts; nobody else does. The two wrong comments are corrected.
- ⚖️ **A decision taken, and measured:** an agent that never sets privacy settings keeps the current defaults — its name and purpose visible to everyone, its skills visible to its own organisation, sensitive details hidden. Making everything private by default was tested: the agent then became invisible to its own organisation's searches, so nobody could recruit it. The book now explains the defaults and how to override them.
- ✅ Tested: the new test failed on the old code and passes now; the "everything private" version was run and the test caught it; the profile suite (87) and three neighbouring suites pass.
- Technical: `filter_profile` emits `incarnation_id` and `visibility` for `ReaderClass::Full` only; `VisibilityPolicy` and `field_visible` doc comments corrected. Clause-1 measurement: all-`self_only` default → suite 86/86 before the new control (no test reads a policy-less profile as a non-owner), and with it the tenant-mate's match returns no candidates. Control `a_profile_without_a_policy_takes_the_default_and_the_full_reader_sees_all_of_it` (discovery arm first). Record: `docs/decisions/2026-09-23_a-profile-without-a-policy-is-discoverable-by-its-tenant.md`.

## 2026-09-23 — Unknown facts about an agent no longer count as variety on a panel (`SIGNOFF-REPAIR.5.1.3`)

`REASONBRAID-REPAIR-0444`.

- 🔴 **Before:** when a call closes, the panel is ranked partly on variety — agents on different providers or tools are less likely to fail the same way. But missing information was counted as variety. An agent with no facts at all got the best possible score, an agent could hide a shared provider by not declaring it, a fact nobody declared was reported as "varies across the panel", and a provider named "x" was counted as matching a *tool* named "x".
- ✅ **Now:** only facts that are actually known count, and each is compared only with the same kind of fact. Unknown scores zero, so declaring less can never help. The panel record now says how many agents did not declare each fact, and says "nobody declares this" instead of "varies".
- ✅ The book explains the variety score with worked examples, for the first time.
- ✅ Tested: four new tests (one per problem) failed on the old code and pass now; two deliberately broken versions of the fix were each caught; the full profile suite passes (86).
- Technical: `matching::diversity(mine, others)` — mean over `dependence::ATTRIBUTES` (now `pub`, the one list) of `1 − sharers/declarers` per attribute known on both sides, else 0, denominator 5; `DependenceIndicator.undeclared` + the explanation cases (none / one / varies across K / groups; `; M of N do not declare one`). Controls: units (a1) `a_candidate_with_no_known_fact_scores_zero_diversity`, (a2) `declaring_an_unshared_fact_ranks_above_leaving_it_undeclared`, (c) `the_sharer_test_compares_the_same_attribute_only`, (b) `an_undeclared_attribute_is_unknown_not_variation`; the snapshot control's `lineage` arm. Mutants M1 (mean over declared) and M2 (any-attribute sharer) caught. Record: `docs/decisions/2026-09-23_diversity-is-a-mean-over-five-attributes-and-unknown-scores-zero.md`.

## 2026-09-23 — A search's ranking weights must be between 0 and 1 (`SIGNOFF-REPAIR.5.1.4`)

`REASONBRAID-REPAIR-0443`.

- 🔴 **Before:** when someone searched the directory for agents, they could say how much each of six factors should count — and any number was accepted. A negative number turned a factor upside down (asking for *diverse* agents returned the *least* diverse first), and two huge numbers made scores infinite, so the ranking fell back to alphabetical order. No error was shown either way.
- ✅ **Now:** each weight must be a number from 0 to 1. Anything else is refused with an error naming the weight. Nothing useful is lost: only the order matters, so any balance between factors still fits in that range (1 and 0.25 means "four times as much").
- ✅ The book now explains the search request, the six factors and their weights — it never had.
- ✅ Tested: the new end-to-end test failed on the old code and passes now; two deliberately broken versions of the check were both caught; the full profile suite passes (86).
- Technical: `RankingPreferences::validate` (first stray in declaration order, `!(0.0..=1.0).contains`, so NaN/∞ refused) called in `directory_match` before the directory read → `400 invalid_command`; `matching::by_rank` (`total_cmp`, then role id) replaces both `partial_cmp(..).unwrap_or(Equal)` sorts. Controls: unit `a_weight_outside_the_unit_interval_is_refused_by_name`, server `the_ranking_weights_are_bounded_and_a_stray_one_is_named`; mutants M1 (no lower bound) and M2 (no upper bound) caught. Book: `profiles.md` *Matching the directory*.

## 2026-09-23 — An expired skill endorsement no longer qualifies an agent (`SIGNOFF-REPAIR.5.1.2`)

`REASONBRAID-REPAIR-0442`.

- 🔴 **Before:** a skill in an agent's profile can carry an expiry date, and nothing ever read it. An endorsement that lapsed last year still got the agent found by searches, admitted to calls and seated on panels.
- ✅ **Now:** a skill counts only until its expiry. A search leaves the agent out, a request to join is refused with the date it expired, and the panel is checked again when the call closes, so a skill that lapses between joining and closing does not win a seat. Expiry works exactly like a permission's expiry, so the two never disagree about the boundary moment.
- ✅ Two details handled deliberately: someone who cannot see an agent's skills is never told when one expired, and an agent listing the same skill twice is judged on its best current one.
- ✅ Tested at both levels. Run against a copy that ignores expiry, the new end-to-end test failed; restored, the full profile suite passes.
- Technical: `matching::eligible(expression, candidate, at)`, `claim_live` (half-open), declared → visible → live → provenance, strongest live claim decides; `respondent_candidate` takes the instant; one instant per respond/close/match. Controls: 3 unit + `an_expired_claim_satisfies_no_eligibility_surface`; mutants M0 (expiry unread) and M1 (close judged a day early) both caught.

## 2026-09-23 — The directory-privacy checklist checked against the code: three items already hold, three are real gaps now scheduled (`SIGNOFF-REPAIR.5.1`)

`REASONBRAID-DOC-0149`. A review; no code changed.

- ✅ **Already holds, with the code that does it named:** tests compare organisations whose agents are equally qualified, so a hidden agent is hidden for privacy and not for skill; two simultaneous profile edits or endorsements cannot overwrite each other; an agent at its declared workload limit is shown as busy and is not recruited.
- 🔴 **Three real gaps, each now a task:** a skill whose endorsement has **expired** still counts in a search; an agent that **declares nothing** about which AI provider it runs on is ranked as the most independent choice, the opposite of cautious; and the **ranking weights** a searcher sends are unchecked, so a negative weight can deliberately pick the most look-alike panel.
- ⚠️ Three wording and default problems in the profile's privacy settings, found earlier, are grouped into a fourth task. Whether a new profile should start out hidden needs a decision, because a hidden profile cannot be found by a search. That decision will be made in its own task, with the effect on search measured first.
- Order: expiry first, then the weights, then the missing facts, then the privacy defaults.
- Technical: census over `matching.rs` / `dependence.rs` / `profiles.rs` at `f324f15`; children `.5.1.2` (expiry), `.5.1.3` (dependence facts: unknown scored 1.0, *varies* for undeclared, cross-attribute sharer test), `.5.1.4` (weights in [0,1]), `.5.1.5` (the three attached clauses).

## 2026-09-23 — Decided: an imported partner agent can run on a machine you enrol for it today, and on the partner's own machine once three pieces are built (`SIGNOFF-REPAIR.5.3.5.3`)

`REASONBRAID-REPAIR-0441`, with the decision `REASONBRAID-DOC-0148`, taken under your delegation of today. With this, nothing in the plan waits on you.

- ⚖️ **Both, as you suggested — because the roadmap asks for two different things.** *Portable agent cards* means a card can be run elsewhere: you enrol a machine for the imported identity, vouch for its skills yourself, and it works under the permission you gave it. *Remote recruitment* means the partner's own agent is invoked where it lives, executing what your permission allows — the way agent-to-agent federation is done today. Each keeps the rule that a partner never authorises anything here; only your permission does.
- ✅ **The first works now, with no new code.** Tested end to end: an imported agent asked to join a call and was refused for having no machine; after enrolling a machine for it and attesting its skill locally, the same request succeeded and it was seated on the panel, its origin recorded throughout.
- 🔴 **The second needs three things first**, now recorded as tasks in order: recording which machine an imported identity runs on; teaching the machine software to judge each job against the right organisation's revocation counter (today it knows only its own, so it cannot safely work for two); and a receipt on both sides for every job that crosses.
- ⚠️ Left open: whether one imported identity may be bound to two machines at once.
- Technical: control `an_imported_identity_acts_once_the_importing_tenant_binds_a_node_to_it` (no source change); children `.5.3.5.3.1` (`card_imports.executes_on`, `COALESCE(executes_on, role_id)` at every resolution), `.5.3.5.3.2` (per-tenant `revocation_epoch` in the node's journal + handshake), `.5.3.5.3.3` (receipts per delivery).

## 2026-09-23 — A permission can now carry conditions, and the first one is a time window (`SIGNOFF-REPAIR.11.4.7.2.1.5.4.3`)

`REASONBRAID-REPAIR-0440`, with the decision `REASONBRAID-DOC-0147`, taken under your delegation of today.

- 🔴 **Before:** the roadmap listed "conditions" among a permission's dimensions and nothing said what a condition was, so the field was deliberately never built.
- ⚖️ **Decided:** conditions are a closed, typed list the server understands completely — the shape modern authorization systems settled on once they stopped writing rules in prose. A kind of condition exists only when the server can refuse a malformed one when the permission is issued, evaluate it at every use from facts it already has, name it in a refusal, and it is tested and documented. Anything less is not a condition.
- ✅ **The first kind:** a daily time window, in the same format and with the same parser as an agent's working hours. A permission with a window only works inside it; outside, the refusal says so and names the time. Rejected for now, each with a measured reason: a "purpose" (nothing declares one), a requesting network (the server records no address), a "human present" check (the next kind, once the check has the facts it needs), and separation of duties (the policy chain's).
- ✅ Tested: unknown kinds, empty lists, malformed windows and a person carrying a condition are all refused; the operator's list shows the condition; a permission whose window is shut is refused with the reason and one whose window is open works. Deliberately making the server ignore a failed condition was caught.
- Technical: `reasonbraid_core::GrantCondition::WithinHours`, `AuthorityGrant.conditions`; `authority/conditions.rs` (`issuance_violations`, `holds`) wired into `create_grant_in_guard` and `evaluate`; migration 0099; `GrantRow` as a named `FromRow` struct; `EnrollRequest.conditions`; the grant listing; control `a_grants_conditions_are_typed_at_issuance_and_evaluated_at_admission`.

## 2026-09-23 — An agent that is deliberately not being woken now says so: presence gains a seventh state, `held` (`SIGNOFF-REPAIR.11.4.7.2.1.5.3.2.2.1`)

`REASONBRAID-REPAIR-0439`, with the decision `REASONBRAID-DOC-0146`, taken under your delegation of today.

- 🔴 **Before:** an agent whose own settings keep it from being woken — "manual only", or outside its working hours — was handed no work, and still reported itself as *available*. The roadmap's six presence words had no word for it.
- ⚖️ **Decided:** a seventh word, `held`, with the reason beside it (`manual_only`, `off_hours`, or an unreadable settings block). Not `draining`, which means winding down, and not "available with a footnote", because the state is the one field everyone reads and it must not lie. Every presence model that has faced this question answers it with a distinct do-not-disturb state. The roadmap was amended in the same commit so it, the code and the handbook stay aligned.
- ✅ `held` sits below `draining` and above `busy`: a policy is a declaration about the agent, like zero capacity, and unlike the count of what it holds right now. A search does not recruit a held agent unless it asks for held ones.
- ✅ Tested in the unit derivation and end to end through the directory: manual-only → held; off-hours → held; inside hours → not held; zero capacity → draining. Run against the previous code first, the held agent read *offline* — the test's agent had no live lease, which the test now provides.
- Technical: `PresenceState::Held`, `presence_state(…, hold: Option<&Hold>)`, `presence::hold_from_stored`, `Hold::wire_name`; `hold` on `PresenceResponse`, the admin presence listing and the directory presence; ROADMAP §10.2; `node-channel.md`.

## 2026-09-23 — A directory search no longer shows another organisation's agents as if the searcher were one of them (`SIGNOFF-REPAIR.5.1.1`)

`REASONBRAID-REPAIR-0438`.

- 🔴 **Before:** when a member of one organisation searched the directory, the server decided once how much that member may see — "a member sees the organisation view" — and applied that to every agent found, including agents of *other* organisations. So a member read a partner's or a stranger's organisation-only fields, and an agent's organisation-only skill could satisfy a search it should have been invisible to. The presence listing had always done this right; the handbook described the search's behaviour as the design.
- ✅ **Now** the search decides per agent, by the searcher's relation to *that agent's* organisation: full or organisation view for its own, organisation view for a partner with a visibility agreement, outsider view for everyone else — and each agent is judged, ranked and shown at that level. An outsider-only skill neither qualifies an agent nor appears.
- ✅ Tested: a member's search does not find a partner agent whose skill is organisation-only; once the agent publishes the skill to the network it is found, shown at the outsider view; with a visibility partnership it is shown at the organisation view. Run against the previous code first, the partner agent was found on a skill the member could not see.
- 🔎 Two of the test failures in that first run were my own mistakes in the test files, both reverted before the green run and recorded as such.
- Technical: `directory_match` — `own_class` for the clamp, `class_by_tenant` memoised via `has_effective_directory_agreement`, `scope_by_role` = min(expression scope, class), per-scope `rank` groups merged by the ranker's sort key, `filter_profile` at the candidate's class; control `the_match_surface_classifies_each_candidate_by_its_own_tenant`; `site-authority.md`'s directory paragraph corrected.

## 2026-09-23 — An agent's machine is now told, on connecting, how many calls are waiting for it (`SIGNOFF-REPAIR.5.3.5.1.1`)

`REASONBRAID-REPAIR-0437`. With this, the whole advertisement of calls — durable, federated, and prompt — is in place; recruiting a partner agent now waits only on your decision about whose machine runs it.

- 🔴 **Before:** an agent could look up the calls offered to it, but nothing told its machine that one was waiting; a machine that did not ask found out late.
- ✅ **Now** every time a machine connects (its regular check-in), the server tells it how many open calls are waiting for its agent — offers still inside their join window that the agent has not yet answered. The reference machine writes that to its log and points at where to read them. An offer never enters the machine's work queue: it is a notice, not a job, because nobody has authorised any work yet.
- ✅ Tested through the real machine client: one open offer → the check-in says one; after the agent answers, and with a closed call and a lapsed one also offered, the check-in says zero. Deliberately breaking the count so answered offers still counted was caught.
- ⚠️ The machine does nothing with the notice beyond saying so; what an agent's adapter should do with an offer is a later, separate design.
- Technical: `HandshakeResponse.offers_pending` on both mirrored wires (the node's with `#[serde(default)]`); `NodeChannelState::offers_pending`; the node's reconcile step 3.7 prints the count; control `an_online_node_is_told_how_many_offers_await_its_role`; `.5.3.5.1` closes.

## 2026-09-23 — A partner organisation's agent can now ask to join a call, and the organiser gets its card to import (`SIGNOFF-REPAIR.5.3.5.2`)

`REASONBRAID-REPAIR-0436`, with the design record `REASONBRAID-DOC-0145`.

- 🔴 **Before:** an agent from a partner organisation could see a network-wide call it was offered, but its answer was simply refused and nothing recorded that it wanted in. The refusal was right — it has no permission here — but the wish, and the card the organiser would need to act on it, had nowhere to go.
- ✅ **Now** such an agent's "join" is recorded as a **join request** on the call, carrying the agent's own exported card and its fingerprint — the same card it would export by hand — provided the two organisations hold a two-way recruitment partnership (a card crosses only with both operators' consent). The organiser sees the request in the call's inspection and resolves it with the ordinary card import; the imported agent's origin is recorded. The request is never counted as a joiner when the call closes.
- ✅ A foreign agent that was never offered the call, or one that answers anything but "join", hears exactly the old refusal, so a call's existence cannot be discovered by guessing ids. An offered agent without the recruitment partnership is told which partnership is missing.
- ✅ Tested end to end: the refusal naming the missing partnership, the recorded request with its card, the old words for a decline and for an un-offered agent, the close seating only the local joiner, and the import from the request's card. Run against the previous code first, the request was refused with the old words.
- 🔎 Writing the test exposed a rule worth knowing: for a network-wide call, only capabilities an agent publishes to the network count — an agent that keeps them visible to its own organisation only is offered (by interest) and then found ineligible (by capability). That is by design and now written down.
- ⚠️ Still open, waiting on you: an imported agent has no machine here, so it cannot yet take part in the call it asked to join.
- Technical: `recruitment::record_join_request` (`response_kind = 'join_request'`); `api::mint_card` shared by `get_profile_card` and the request; the foreign branch of `respond_to_call_core` (three gates, then the common checks, then the record); control `a_federated_subscribers_join_is_a_recorded_request`; DOC-0145.

## 2026-09-23 — A network-wide call now reaches matching agents in partner organisations (`SIGNOFF-REPAIR.5.3.5.1.2`)

`REASONBRAID-REPAIR-0435`.

- 🔴 **Before:** a call could be opened "for the network", but its offers only ever went to agents in the organiser's own organisation. The partnership that lets a partner see this organisation's directory changed what partners could read, never what they were invited to.
- ✅ **Now** a network-wide call is also offered to matching agents in every organisation that holds a two-way, unexpired visibility partnership with the organiser's — in the same step as the local offers. A call scoped to the organisation, a one-sided or revoked partnership, or a partnership without visibility offers nothing outside, which is the roadmap's rule: cross-organisation recruitment is opt-in, never the default.
- ✅ What a partner agent sees of a foreign offer is the call, its requirements and its window — not the conversation thread, which lives in the other organisation and is named only once a join lands. A partner agent still cannot *answer* a foreign call; recording that wish as a request for the organiser to resolve is the next task.
- ✅ Tested: no partnership → nobody offered; partnership → the partner agent is offered and sees no thread, and its answer is refused; an organisation-scoped call → nobody outside; a revoked partnership → nobody outside. Run against the previous code first, the partner agent was never offered.
- Technical: `offer_to_subscribers(…, federated)` — one `INSERT … SELECT` with the tenant arm OR an `EXISTS` over both `federation_agreements` rows (`accepted`, `directory_visibility`, unexpired); `list_offered_calls` adds `call_tenant_id`, `foreign`, and nulls `thread_id` for a foreign offer; control `a_network_scope_call_is_offered_across_an_effective_directory_agreement`.

## 2026-09-23 — An agent can now see the calls it was offered (`SIGNOFF-REPAIR.5.3.5.1`)

`REASONBRAID-REPAIR-0434`.

- 🔴 **Before:** when a call for participants was opened, the server recorded which agents it was offered to, and that record went nowhere. No agent could ask "what have I been offered?"; the only readers of a call were its organiser and the organisation's administrator. Agents learned of calls out of band.
- ✅ **Now** an agent lists the open calls offered to it, newest last, each with the call's requirements and window and with its own answer if it has given one. Closed calls and calls past their join deadline drop off. A person gets a refusal (calls are offered to agents, not people), and so does an unknown caller, rather than an empty list that could be mistaken for an answer.
- ⚖️ The first idea — pushing the offer into the agent's machine's work queue — was set aside with a reason: that queue holds authorised work the machine executes, and an offer is neither authorised work nor something to execute. The durable record the roadmap asks for is the offer itself, which already survives the machine being offline; what was missing was a way to read it. Telling an online machine promptly that an offer is waiting is the next, separate task.
- ✅ Tested: two matching agents are offered, a third with other interests is not; the offer shows before and after joining; the person and the stranger are refused; the closed call disappears. Run without the new route first, the request was swallowed by the "inspect one call" route and answered "no call named offered".
- Technical: `GET /v1/calls/offered` (`list_offered_calls`, mounted before `/v1/calls/{call_id}`); `recruitment_offers ⋈ recruitment_calls` for the calling role, `status = 'open'`, inside the window, with a correlated `response_kind`; the calls family witness `5:-` in `.doctrine/book_surface_verdicts.tsv`; control `a_subscriber_lists_the_calls_offered_to_it`; children `.5.3.5.1.1` (the prompt half) and `.5.3.5.1.2` (the federated half).

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
- Technical: `migrations/0097_federation_expires_at.sql`, `0098_cross_domain_receipts_are_events.sql` (drops the 0049 unique key); `FederationAgreementRequest.expires_at`; the upsert's `WHERE` gains `expires_at IS DISTINCT FROM EXCLUDED.expires_at`; the three predicates and the counterparty read gain `AND (expires_at IS NULL OR expires_at > now())`; control `an_expired_direction_widens_nothing_and_cannot_be_accepted_against`; the upgrade seed branches on `information_schema.columns`.

## 2026-09-23 — A partnership's terms now have a fingerprint, and accepting one records exactly which terms the partner had offered (`SIGNOFF-REPAIR.5.3.1`)

`REASONBRAID-REPAIR-0432`.

- 🔴 **Before:** when an organisation accepted a partnership, the audit receipt — which is supposed to name the partner's record by its fingerprint — named the partner's *id* instead, because a partnership record had nothing to fingerprint. And an organisation could "accept" a partnership the partner had never proposed, even though the refusal message claimed the partner had to propose first.
- ✅ **Now** every partnership direction carries a fingerprint of its terms, computed by the server, and existing rows were given theirs by the same recipe during the upgrade — checked on a real pre-upgrade row. Accepting reads the partner's current offer and records its fingerprint twice: in the audit receipt, and on the accepting row. So the trail says which terms each side saw when it agreed. If one side later changes its terms, its own acceptance is cleared, while the partner's row still names the terms it agreed to.
- ✅ One change on the wire: accepting when the partner has made no offer is refused, naming the partner. That is what the message always said.
- ✅ Tested end to end, including the refusal, the fingerprints on both sides, and a change of terms. Run against the previous code and schema first, the acceptance went through with nothing on the other side.
- 🔴 The handbook's example of an audit receipt showed a kind and an id shape the code never wrote; corrected to the real shape.
- Technical: `migrations/0096_federation_terms_digest.sql` (`terms_digest` NOT NULL backfilled by SQL, `accepted_against`); `federation::terms_digest` with two `hashlib`-pinned vectors; the propose upsert carries the digest and clears `accepted_against`; the acceptance reads own status then the counterparty's live row, `AcceptResult::NoCounterparty` → `409`; the receipt's `remote_ref` = the counterparty's digest; the upgrade suite seeds a pre-upgrade direction and holds the backfill to the Rust value; four card fixtures reordered to propose-both-then-accept-both.

## 2026-09-23 — Importing the same partner agent twice now returns the original, and every imported agent records where it came from (`SIGNOFF-REPAIR.5.3.2`)

`REASONBRAID-REPAIR-0431`.

- 🔴 **Before:** an imported agent was identified only by its display name. Importing the same agent again was refused with a message about the name being taken; importing it again under a new name created a second, unrelated local agent; and an unrelated agent that happened to share a name was refused as if it were a repeat. Nothing recorded which partner agent a local one came from — only whoever still held the card knew.
- ✅ **Now** every import records its origin: the partner organisation, the partner's agent id, the fingerprint of the card that landed, who authorised it and when. Importing an agent that is already here — under any name, from any later card — returns the original local agent, flagged as a repeat, together with the fingerprint of the card on file. Nothing is written twice. The name refusal remains, but only for a genuinely different agent that shares the name.
- ✅ The agent's owner or administrator sees the origin on the agent's profile; partners and outsiders do not.
- ✅ Tested end to end: the repeat, the renamed repeat, the origin on the profile, and the name refusal for a different agent. Run against the previous code first, the repeat was refused with the name message.
- ⚠️ Left open, recorded: whether a *newer* card for an already-imported agent should update the local profile, and under whose authority. Today it does not, and the answer says so.
- Technical: `migrations/0095_card_imports.sql` (`UNIQUE (tenant_id, origin_tenant_id, origin_role_id)`); `CardImportResult::Replayed { role_id, digest_on_file }` read after the allowlist rung under the exclusive guard, the row written after the enrollment row with `ON CONFLICT DO NOTHING RETURNING` (a miss is a storage failure); `imported_from` on `GET /v1/profiles/{role_id}` full class; 30 purge plans swept; control `an_import_is_identified_by_its_origin_not_its_label`.

## 2026-09-23 — An imported agent's permission is now issued by the administrator who authorised the import (`SIGNOFF-REPAIR.5.3.3`)

`REASONBRAID-REPAIR-0430`.

- 🔴 **Before:** when an administrator imported an agent's card from a partner organisation, the local permission the agent received was recorded as issued by a made-up identity — a fresh id that belonged to nobody — even though the administrator who authorised the import was known to the code and simply never used.
- ✅ **Now** the permission names that administrator as its issuer. If an organisation is administered by an agent rather than a person, the old limitation stays, with the reason written beside it.
- ✅ Tested by importing a card and reading the permission back: its issuer is the importing organisation's administrator and is enrolled there. On the previous code the test failed with a random id.
- ⚠️ Still true: the database does not force an issuer to be a real principal, and the ordinary enrolment of an agent still records a made-up issuer until the development bootstrap is replaced.
- Technical: `import_after_admission`'s `_principal` became `principal`; `let issuer = match principal { Human(h) => *h, Role(_) => HumanPrincipalId::new() }` feeds `dev_grant`; control `an_imported_roles_grant_is_issued_by_the_admitting_administrator` in `crates/reasonbraid-server/tests/cards.rs`.

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
- Technical: `recruitment::serialize_opens` (`pg_advisory_xact_lock(class, hashtext(tenant_id))`), `offer_to_subscribers` (one `INSERT … SELECT`), `call_locked(CallLock::{Shared, Exclusive})`; `serialize_opens`, `open_calls_by`, `open_call`, `offer_to_subscribers`, `record_response`, `call_locked`, `snapshot_panel` take `&mut PgConnection`, `call` and `responses` stay executor-generic; the caps re-exported for the controls. Harness: `hold_inserts` / `wait_for_waiters` in `crates/reasonbraid-server/tests/profiles.rs`. Record: `docs/decisions/2026-09-23_each-call-transition-is-one-transaction-and-a-late-join-is-refused.md`.

## 2026-09-23 — A recruitment call can no longer close with fewer panelists than it asked for (`SIGNOFF-REPAIR.5.2.5`)

`REASONBRAID-REPAIR-0428`.

- 🔴 **Before:** when a call was closed, the "at least N panelists" rule was checked against everyone who had *said* they would join. Only afterwards did the server re-check whether each of them still qualified, and drop those who no longer did. So an agent that joined while qualified and then lost its qualification (for example, its owner's endorsement of a skill lapsed) still counted toward the minimum, and the call could close with fewer panelists than required — even none.
- ✅ **Now** the minimum is checked on the panel that is actually selected, after the re-check. If too few still qualify, the close is refused with a message stating how many are required, how many still qualify, and how many had joined — and the call stays open, so the organiser can wait for more joiners or re-qualify one.
- ✅ Tested by making exactly that happen: an endorsed agent joins, its endorsement lapses, the close is refused and the call stays open; the owner endorses it again and the same close succeeds with the agent on the panel. Run against the previous code first, the test showed the old behaviour precisely: a closed call with an empty panel.
- ⚠️ The remaining recruitment-call problem — each of the three transitions is several separate database writes rather than one, so two simultaneous closes can collide — is the next task.
- Technical: the check moved from the raw joiner count to the eligible, ranked set between `rank_with_dependence` and `truncate` in `close_call` (`crates/reasonbraid-server/src/api.rs`); the refusal names all three figures; control `a_calls_minimum_is_enforced_on_the_selected_panel` in `crates/reasonbraid-server/tests/profiles.rs`; book `docs/book/src/recruitment.md` (the close section).

## 2026-09-23 — A recruitment call's three transitions are each several separate writes, and its minimum is checked before the filter that can empty the panel (`SIGNOFF-REPAIR.5.2`)

`REASONBRAID-DOC-0142`. A decision; no code changed.

- I checked what the recruitment task still owes after this week's repairs. The identity bindings are done. Two things are not.
- 🔴 **None of open, answer or close is a single transaction.** Opening a call counts, then inserts the call, then writes each advertisement as a separate step, so two simultaneous opens can both pass the cap and a crash can leave a call half-advertised. Closing reads, then writes the panel and the status as two steps, so two simultaneous closes collide on the database's own key with a raw "500", and an answer can slip in between the close's read and its write.
- 🔴 **The minimum is checked against who said "join", not against who is still eligible.** The close then drops anyone whose eligibility lapsed, so a call can close with a panel smaller than its minimum, down to empty.
- ✅ Two follow-ups, tracked: the minimum on the selected panel first (small, reproducible), then each transition as one transaction.

The entries before those above were rotated into reachable Git history at the
**tenth rotation** (`SIGNOFF-REPAIR.11.4.1.6`, which owns this ledger’s rotation). The exact predecessor — this file as it
stood at the commit named below, which is the object every retired record was
checked against before this notice was written — is:

```bash
git show 1291e49c240aba4c626cc290310ddd64b8b472f5:DEV_NOTES.md
```

That snapshot is 72864 bytes and 431 lines, and contains 41 dated
entries; its Git blob is `7fba0ab71f538b212972751b2f622d97d54d731b` and its SHA-256 is
`37bf0a72bbf86f07ec5d2ac98359bd8b68e4d49b35bf2ea85329121caa2f4284`. It carries the ninth rotation's
notice in turn, and each earlier notice names the one before it, so the chain
walks all the way back. `docs/decisions/2026-09-09_changelog-rotation.md` holds
the first transition's evidence.

⛔ **14 record(s) rotated out, 28 kept, lossless** — every retired heading was retrieved from the
predecessor named above before this notice was written, and every figure in it was re-derived from that object with
`git rev-parse`, `git cat-file` and SHA-256 rather than typed. ⭐ The cut is DERIVED, not chosen: it retires whole
records until the ledger has at least 10 commits of runway at the p90 entry size measured over the last
60 non-rotation commits — because two rotations that stopped at the threshold instead left 344 and 296 bytes and
the first forced another rotation on the very next commit (`SIGNOFF-REPAIR.11.4.1.6`).
