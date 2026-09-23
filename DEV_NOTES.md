# DEV_NOTES.md

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

## 2026-09-23 — An administrator can now settle a job whose outcome the machine could not prove, and the machine applies the decision (`SIGNOFF-REPAIR.11.4.7.2.1.5.5`)

`REASONBRAID-REPAIR-0427`.

- 🔴 **Before:** when a machine crashed after possibly calling a provider and could not tell whether the call happened, the roadmap's fourth way out — a human decides — did not exist. The machine could only wait for a server receipt or prove it itself, and the administrator's view listed three actions, none of them a verb.
- ✅ **Now** the organisation's administrator records a verdict — "it completed" or "it did not happen" — with a reason. The verb is authorised, written to the audit trail like every other administrative action (the fifteenth kind), and bound to the organisation's own machines. The machine picks the verdict up at its next check-in and closes the question; until then the listing shows the verdict as awaiting the machine. A second verdict, an unknown verdict, or another organisation's administrator are refused.
- ✅ Tested end to end with a real machine journal: the ambiguity, the refusals, the recorded verdict and its audit entry, the listing, and the machine applying it. Making the check-in ignore the verdict leaves the job ambiguous, which the test catches.
- ✅ With this, every buildable item in the re-derivation family opened on 22 September is closed; what remains waits on your two decisions or on the first deployment beyond localhost.
- Technical: `node_admin::adjudicate_ambiguous_attempt_in_one_transaction` (shared guard, `admit_or_return!`, `FOR UPDATE OF a`, `record_inbox_effect` with `AdministrativeOperation::NodeAttemptAdjudicate`); migration 0094 (four columns + the widened `administrative_effects` CHECK); `node_channel::handshake` asks `operator_verdict` before `event_id_for_operation`; `.doctrine/operator_surfaces.tsv` bullet 4 and the admin family witness `23:-`.

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
- Technical: `AuthorityGrant.decision_rule_constraints: Option<Vec<String>>`; migration 0093; `decision_rule_violations` in `create_grant_in_guard` (names parsed by `charters::DecisionRule::parse`); `EnrollRequest.decision_rule_constraints` → `dev_grant`; `list_grants` emits it; the Create arm of `run_thread_command` reads the admitting grant via `authorization_records.grant_id` before `charters::allows_on`.

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
- Technical: migration 0092 `incarnations.node_id` (backfilled `= role_id`); the enrolment insert binds `req.node_id`; `list_incarnations` emits `node_id`; the registry trigger is `SELECT count(*) FROM incarnations WHERE node_id <> role_id`.

## 2026-09-23 — An offline agent's backlog of undelivered work is now capped, and the cap is visible (`SIGNOFF-REPAIR.11.4.7.2.1.5.3.3`)

`REASONBRAID-REPAIR-0424`.

- 🔴 **Before:** if an agent's machine never reconnected, work kept piling up for it without limit — the roadmap's "maximum offline backlog" existed only on paper.
- ✅ **Now** a machine already holding 64 undelivered jobs (a development-scale figure, not a measured one) is handed nothing more. The action that would have handed it the job — an agent accepting an invitation, or a challenge that would send a revision back to the author — is refused and undone, so an invitation never exists without its work. The refusal is recorded like every storm control, and the administrator's node view shows each machine's backlog against the cap before the refusal ever happens.
- ✅ Tested: a machine seeded at the cap shows "64 of 64", the accept is refused and rolled back with the record naming the machine, the cap and the thread, and after one job is taken the same accept lands. Removing the cap makes the test fail.
- ✅ With this, the whole storm-control family opened on 22 September is closed: every control whose trigger had fired is built, recorded and readable.
- Technical: `node_channel::MAX_OFFLINE_BACKLOG`, `undelivered_in_tx` (`queued`/`offered` via `node_inbox_state`); `dispatch_work_in_tx` returns `DispatchRefusal::OfflineBacklog` before reserving; `run_thread_command` rolls back via `tx.rollback()` and records through `refuse_storm` (`backlog_refused`); `list_node_presence` emits `backlog: {undelivered, cap}`.

## 2026-09-23 — Every storm-control refusal is now recorded, so a storm can actually be seen (`SIGNOFF-REPAIR.11.4.7.2.1.5.3.4`)

`REASONBRAID-REPAIR-0423`.

- 🔴 **Before:** when the server refused a request as a storm control — too many open calls, an agent re-joining its own chain, a chain too deep — the caller got a "429" and nothing was written down. The roadmap's circuit breakers were postponed until "the first multi-tenant storm is observed", and nothing could observe one.
- ✅ **Now** every such refusal is written down before it is answered, by the one piece of code that is allowed to produce that answer, so no refusal can be given without a record. Each row says which control refused, its limit, who was refused, which thread they named, and the exact words they were given. An administrator lists them with one call.
- ✅ The breakers' trigger is now a plain question with an answer: have two or more organisations been refused within an hour?
- ✅ Tested on the existing storm tests: the fifth call in a row and the two chain refusals each appear on the record with the right control, limit and words. Removing the write makes the tests fail. The first run caught that the fan-out record named an internal handle instead of the person; fixed before commit.
- Technical: migration 0091 `storm_refusals`; `api.rs::refuse_storm` (the only value-spelling of the wire code; storage failure → internal error); four producers routed through it; `GET /v1/admin/storm-refusals` (`TenantAdminInspection::StormRefusals`); `.doctrine/book_surface_verdicts.tsv` admin family `21:-` → `22:-`; 26 checked purge plans gained the table.

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
- Technical: `reasonbraid_core::Audience`, `AutoBounds.audience` (absent fields omitted on the wire); `AutoLineage.initiating_grant` → `ThreadProjection.initiating_grant`; `open_recruitment_call` loads the projection via `load_thread_projection` (`404 scope_hidden` when absent) and checks `audience_admits(audience, expression.scope)`; a `create_thread` helper seeds the three call fixtures.

## 2026-09-23 — An automatic-thread permission can now carry its limits, and they are read from the permission that actually admitted the request (`SIGNOFF-REPAIR.11.4.7.2.1.5.3.2.3.1` and `.2`)

`REASONBRAID-REPAIR-0421`.

- 🔴 **Before:** nothing could set the limits an automatic-thread permission is supposed to carry, and the one limit that was read (spend) was taken as the largest across all of an agent's permissions — including expired ones — rather than from the permission that admitted the request.
- ✅ **Now** the enrolment step, which already declares an agent's actions, also declares its permission's spend limit, allowed topics and maximum chain depth. Bad declarations are refused with the reason (limits on a permission without the action, an empty topic list, a depth of zero or above 3), and the administrator's permission list shows them.
- ✅ When an agent starts a thread, the server reads those limits from the exact permission that admitted the request. An expired permission with a bigger spend limit no longer raises the ceiling. The topic limit applies alongside the agent's own declared interests, and the depth limit can be tighter than the site's 3.
- ✅ Tested end to end, including the expired-permission case that used to slip through. Removing the reader or the check makes the tests fail.
- Technical: `reasonbraid_core::AutoBounds { topics, max_depth }`; `AuthorityGrant.auto_bounds` (`serde(default)`); migration 0090; `auto_bounds_violations` in `create_grant_in_guard`; `EnrollRequest.spend_limits`/`auto_bounds` → `dev_grant`; `list_grants` emits both when present; `create_thread_auto` keeps the `Allowed` record id and joins `authorization_records → authority_grants` for the admitting grant's bounds; `auto_lineage(role, parent, ceiling)`.

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
- Technical: `wake::delivery_budget(concurrency, in_flight)`; `node_channel::replay` reads `node_presence.in_flight` in its pre-check and binds the budget as `LIMIT $3` on the tail CTE (`LIMIT NULL` = unbounded; `Some(0)` returns early), so only the rows handed over are marked `offered`.

## 2026-09-23 — Two agent settings that were stored and ignored now mean something, and bad values are refused (`SIGNOFF-REPAIR.11.4.7.2.1.5.3.2.2`)

`REASONBRAID-REPAIR-0419`.

- 🔴 **Before:** an agent's profile could declare "working hours" and a "wake policy", and the server stored any text for either — "never", "manual_only", anything — and then ignored both. Work was delivered and the agent could start threads regardless of what it had declared.
- ✅ **Now** each setting has an exact form the server refuses to violate (working hours as a UTC window such as `22:00-06:00`; wake policy `auto` or `manual_only`), and one rule that both surfaces obey: outside its hours, or under `manual_only`, an agent is handed no work and may not start a thread on its own. A value that somehow reached storage without passing that check holds the agent rather than being ignored, and says which field.
- ✅ Tested at both surfaces and at the write: bad values are refused by name, delivery is held and released by the clock and by the policy, and self-started threads are refused in the same words. Disabling the rule, or the check, makes the tests fail.
- ⚠️ Two follow-ups are open and owned: a positive concurrency number still does not cap how much work a node is handed, and an agent held by its policy still shows as "available" in the presence view, which touches the roadmap's fixed vocabulary of six states — that one is yours to decide.
- Technical: `crates/reasonbraid-server/src/wake.rs` holds both halves — `validate` (formats) and `hold` (`Draining` / `ManualOnly` / `OffHours` / `Unreadable`, in durability order). `node_channel::replay` evaluates the role's current `availability` block at `now()` before reading the tail, replacing the `concurrency = 0` SQL clause; `create_thread_auto` refuses `403` naming the hold; `put_profile` answers `400` and the card import `CardRefused`. DOC-0135 said the gate belongs on the node; it is at the server's delivery boundary because the profile and the clock are the server's — the record carries the correction.

## 2026-09-23 — Every thread view now gives the same answer for a thread you cannot see (`SIGNOFF-REPAIR.17`)

`REASONBRAID-REPAIR-0418`.

- 🔴 **Before:** asking about a thread that does not exist, or that belongs to another organisation, got two different answers depending on which view you asked. The thread and budget views said "not visible". The timeline said "no events" as if the thread existed, and the audit view listed the record of your own asking.
- ✅ **Now** all four views answer "not visible" in exactly the same words. Nothing had leaked, but a reader could tell "no such thread" from "not yours" by which view they asked.
- ✅ Tested: your own thread still answers on all four views; a foreign thread and a made-up id get the identical refusal on all four. The test failed on the old code at the timeline view, exactly as measured yesterday.
- Technical: `api.rs::thread_exists` is the tenant-bound `aggregate_state` predicate under the RLS claim; `get_events` and `get_audit` return `ControlApiError::scope_hidden()` when it is false. The audit case mattered most: `inspect()` records the `thread_inspect` authorization before the read, so the audit of an absent thread returned that record. CLI and web UI both treat a non-200 as an error already.

## 2026-09-23 — An agent can now start more than one automatic thread, and how often is limited (`SIGNOFF-REPAIR.11.4.7.2.1.5.3.2.1` with `SIGNOFF-REPAIR.5.2`)

`REASONBRAID-REPAIR-0417`.

- 🔴 **Before:** an agent could start exactly one automatic thread per tenant, ever — every later attempt silently returned the first thread. That bug was also the only thing stopping an agent from starting threads without limit, so it could not be fixed on its own.
- ✅ **Now** each automatic start carries its own key, so a retried delivery still returns the same thread but a new start creates a new one. And every agent has a limit on automatic starts per hour (1000 in the development setting), recorded like every other usage limit: a refused start is written down, an agent with no limit row is refused rather than let through, and a person's ordinary thread creation is not counted. Both changes are in one commit, so at no point was automatic starting unlimited.
- ✅ Tested end to end: two starts make two threads, a retry replays, the limit refuses at the ceiling, the refusal is recorded once, and an agent with no limit row is refused. Reverting either half makes the test fail.
- ⚠️ The hourly number is a development default, not a measured one. The limit is per agent; making it part of the grant itself is a later task.
- Technical: migration `0089_initiator_quota.sql` widens `usage_quotas.scope_kind` to a fifth kind `initiator` (scope id = role id) and backfills `quo_{role}_initiations` for existing roles; `quota::insert_initiator_default_in_tx` runs at both role-creation sites (enrolment, card import); `run_thread_command` checks the scope inside the create transaction only when `body.lineage` is set (server-set by the auto route), after the idempotency claim and authorization, storing a refusal as the invite quota does; `AutoCreateRequest.idempotency_key` is required and non-empty, and the row key is `auto_{role}_{key}`. Falsified: the old key → the second initiation replays (68/1); the check skipped → `(use, denial) = (0, 0)` (68/1).

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

## 2026-09-22 — The book has a chapter on where published policy lives (`SIGNOFF-REPAIR.9.3.5.3`)

`REASONBRAID-DOC-0130`. Documentation only.

- ✅ **New chapter: *The publication store*.** It explains what a publish writes into Git, the three pointers it moves and when each is refused, the order of the steps, every refusal message, and how an interrupted publish is recovered. The book previously never mentioned these Git pointers at all.
- ⭐ The examples are copied from a real run, not typed by hand. Doing that exposed the two defects fixed in the previous commit.
- ✅ The two sections on this topic that I added to the CLI chapter earlier today moved into the new chapter, so the topic has one home.

The entries before those above were rotated into reachable Git history at the
**eighth rotation** (`SIGNOFF-REPAIR.11.4.1.6`, which owns this ledger’s rotation). The exact predecessor — this file as it
stood at the commit named below, which is the object every retired record was
checked against before this notice was written — is:

```bash
git show 1bb870efa63e4515025ee2e6456ae78734aa0497:DEV_NOTES.md
```

That snapshot is 71816 bytes and 486 lines, and contains 47 dated
entries; its Git blob is `b4a60a59cd9be6e3853854a78f4a9e878bbb74f8` and its SHA-256 is
`827f25e373863893bdddcab738186e36960d33190379032680dcaff0abb2b596`. It carries the seventh rotation's
notice in turn, and each earlier notice names the one before it, so the chain
walks all the way back. `docs/decisions/2026-09-09_changelog-rotation.md` holds
the first transition's evidence.

⛔ **14 record(s) rotated out, 34 kept, lossless** — every retired heading was retrieved from the
predecessor named above before this notice was written, and every figure in it was re-derived from that object with
`git rev-parse`, `git cat-file` and SHA-256 rather than typed. ⭐ The cut is DERIVED, not chosen: it retires whole
records until the ledger has at least 10 commits of runway at the p90 entry size measured over the last
60 non-rotation commits — because two rotations that stopped at the threshold instead left 344 and 296 bytes and
the first forced another rotation on the very next commit (`SIGNOFF-REPAIR.11.4.1.6`).
