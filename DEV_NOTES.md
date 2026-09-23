# DEV_NOTES.md

## 2026-09-23 — The machine's own tests can now see an answer being sent (`SIGNOFF-REPAIR.4.4.4.2.1`)

`REASONBRAID-REPAIR-0463`. Test equipment for the fourth recovery gap; no product behaviour changed.

- 🔴 **Before:** the machine's own tests could never get a machine properly connected, so nothing below the slow full-system tests could see an answer actually being sent. A deliberately broken machine that claimed "sent" while sending nothing passed all 83 of them.
- ✅ **Now:** a small stand-in server lets a test machine connect for real. New checks show an answer sent at once when connected, and an answer kept while disconnected then sent exactly once on reconnect under its original id.
- ✅ Tested: the broken "claims sent, sends nothing" machine is now caught by the machine's own tests; 87 machine tests pass; strict lint clean.
- Technical: `crates/reasonbraid-node/tests/support/control_plane.rs` `StubControlPlane` (`axum` 0.8 dev-dep; one `Cargo.lock` edge) serving `handshake`/`events`/`ack` with `HandshakeResponse`/`EventReceipt`/`AckResponse`; `tests/worker_delivery.rs` 4 controls. `cargo mutants --in-place` on `send_journaled|deliver_journaled_event`: 2 caught, 1 unviable, 0 missed; stub token mutant RED.

## 2026-09-23 — A finished, paid-for answer can no longer be lost between "done" and "sent" (`SIGNOFF-REPAIR.4.4.4.1`)

`REASONBRAID-REPAIR-0462`. The first half of the fourth recovery gap found by `REASONBRAID-DOC-0153`.

- 🔴 **Before:** a machine recorded "the provider finished" and the answer to send as two separate saves. If it died between them, or was merely not yet reconnected, the answer was thrown away. The work was paid for, marked finished, and never sent; nothing could bring it back. Seven existing tests treated that loss as normal.
- ✅ **Now:** the "finished" record and the answer are saved together in one step. If the machine cannot send the answer right away, the answer waits and goes out on the next reconnect, under its original id so it is never counted twice.
- ⚠️ Still to do (tracked, next): a machine that is not yet reconnected should not start paid work at all.
- ✅ Tested: the seven tests, rewritten to demand the waiting answer, failed on the old code (7 of 7) and pass now; two new crash tests; 83 machine tests and six live suites (91 tests) pass; nine deliberately broken versions were tried: four caught, three not buildable, one caught only by the live suites, one hand-made "two separate saves" version caught; strict lint clean.
- Technical: `Journal::record_completed_with_event` over private `apply_transition_emitting` (transition + ledger row + `INSERT … SELECT operation_id FROM attempts`, one tx); `execute_attempt_emitting(…, &ResultEventBuilder)` + `land_completed` on the runtime and status-lookup paths; `ExecutionReport.result_event`; `Node::deliver_journaled_event` → `EventDelivery::{Delivered, Deferred}` sharing `send_journaled` with `emit_event`. Mutants: `cargo mutants --in-place -o target/r4_4_3` 8 → 4 caught / 3 unviable / 1 missed (`send_journaled → Ok(())`, caught live by `node_work` 10/13); hand two-transaction cut RED at `a_completion_whose_result_cannot_be_written_does_not_happen`. A `proved` flag selecting an identical `Complete` transition was removed when the tool showed it had no possible observer.

## 2026-09-23 — The tests guarding the hand-off to the provider can now see whether the provider was reached (`SIGNOFF-REPAIR.4.4.3`)

`REASONBRAID-REPAIR-0461`. The third of the seven recovery gaps found by `REASONBRAID-DOC-0153`.

- 🔴 **Before:** one test was meant to prove that a replacement machine refuses outdated work instead of sending it to the provider. Its stand-in provider was set up to refuse by itself, though. The test also passed with the safety check switched off, because the stand-in's own refusals produced the same outcome. Nothing in the tests could count provider calls.
- ✅ **Now:** the stand-in provider counts every call. The tests use one that *would* succeed, then check it was called zero times and that the recorded refusal is the safety check's own. Every step's result is checked, not thrown away, and a second test now looks for the right kind of event (a revision, not a contribution).
- ⭐ **Shown, not assumed:** with the safety check deliberately broken, the OLD test still passed and the new one failed. A second test the review had flagged turned out to catch the break already through an earlier check; only its last line was blind, and that line is fixed too.
- ✅ Tested: 13 + 2 live tests pass; 134 tests in the provider and machine packages pass; strict lint clean. No product behaviour changed.
- Technical: `FakeAdapter::invocation_counter() -> Arc<AtomicU32>` (bumped on every `invoke`); `node_replacement::the_replacement_ritual_recovers_a_lost_node` scripts `Complete`, asserts 0 invocations, `MAX_DISPATCH_ATTEMPTS` `failed_before_dispatch` attempts whose evidence names *cached admission decision is stale* / *recorded epoch 0*, every tick `Ok`, and after the replay exactly 1 invocation and 1 `completed` attempt; `node_work::a_revocation_invalidates_the_cached_decision_at_the_next_dispatch` asserts invocations 1 → 1 across the refusal, the epoch pair in the evidence, and 0 `thread.revision_submitted`. Falsified with the `cargo mutants` diff `&&`→`||` in `CachedDecision::evaluate`, applied by `patch` and run through `run_pg_tests.sh`: new `node_replacement` RED (`left: 1`), HEAD `node_replacement` GREEN, `node_work` RED at its pre-existing lookup; restored, `shasum -a 256 -c` OK.

## 2026-09-23 — A refused answer's cost is now counted, and the machine is told it was refused (`SIGNOFF-REPAIR.4.4.2`)

`REASONBRAID-REPAIR-0460`. The second of the seven recovery gaps found by `REASONBRAID-DOC-0153`.

- 🔴 **Before:** when the server refused a machine's finished answer (say, the agent's permission was withdrawn, or the conversation had closed), the provider's cost was never counted — the budget hold simply lapsed — and the machine was told only "received", so it believed the work had landed.
- ✅ **Now:** a refused answer's cost is counted against the budget exactly like an accepted one, because the provider did the work either way. The machine is told the answer was refused and why, records it, and says so in its log. The budget charged is always the one the server attached to the job, never one the machine names.
- ⚠️ Still to do (tracked): the operator's inbox view does not yet show that an answer was refused.
- ✅ Tested: a new check failed on the old code (the budget hold stayed open) and passes now; a full run with a real machine closing the conversation mid-job shows the cost counted and the refusal recorded; two deliberately broken versions (no counting on refusal; the machine ignoring the refusal) were each caught; the machine's 81 tests and five further suites (160 tests) pass; strict lint clean.
- Technical: `api::settle_work_item_reservation` (stored `work.reservation.reservation_id`, reported usage) after each of the three `store_rejection`s and on success; `EventReceipt.refused: Option<ResultRefusal{code,message}>` both sides (serde default/skip); node journal migration `0005_event_refusals.sql` (`outgoing_events.refusal`, user_version 5), `Journal::record_event_refusal` / `event_refusals`, `Node::record_refusal` on `emit_event` and the reconcile re-emission. Controls: `a_revoked_grant_refuses_a_later_node_result` grown (settled with 41/17; `refused.code = unauthorized`); new `a_refused_result_is_settled_and_the_node_journals_the_refusal` (node_work, real worker). Mutants M2 (no settlement on refusal) and M3 (node ignores `refused`) caught. Schema pins 4→5 (`journal.rs`, `journal_cli.rs`); `node-journal.md` example 2→5.

## 2026-09-23 — An unresolved provider call is no longer "settled" by the machine's own give-up note, and an operator's ruling now lands as ruled (`SIGNOFF-REPAIR.4.4.1`)

`REASONBRAID-REPAIR-0459`. The first, and worst, of the seven recovery gaps found by `REASONBRAID-DOC-0153`.

- 🔴 **Before:** when a machine could not tell whether a paid provider call had happened, the server declared the case settled as soon as it held *any* message from the machine about that job — including the machine's own "I refused to retry this" report, or the answer from a later retry. The doubt vanished with no evidence. And when an operator ruled "it did not happen" or "it did happen", the machine recorded neither: every ruling became a bare "settled", with the operator's reasoning lost.
- ✅ **Now:** only the machine's own answer *for that specific attempt* settles the doubt; a give-up note or another attempt's answer leaves it open and visible. An operator's ruling is recorded as ruled — "did not happen" or "did happen" — with the ruling itself kept as the evidence. A ruling the machine does not understand leaves the case open rather than closing it.
- ⚖️ **Corrected a claim from an earlier fix:** "did not happen" was described as making the machine re-run the work by itself. It never did and still does not; re-asking is the thread owner's decision, and the fix's record now says so.
- ✅ Tested: two new checks failed on the old code exactly where predicted (a give-up note settled the doubt; a "did not happen" ruling was recorded as a bare "settled") and pass now; two deliberately broken versions (any message settles the doubt; the ruling flattened again) were each caught; the machine's own 81 tests and six further suites (127 tests) pass; strict lint clean.
- Technical: server `NodeChannelState::result_receipt_for_attempt` (`payload->>'kind' = 'work_result' AND payload->>'attempt_id' = $3`) drives the directive loop; the evidence and the `needs_adjudication` reason name the attempt. Node `Node::reconcile` matches `Directive::Adjudicated { terminal, evidence }`: `reconciled` → `Journal::reconcile_with_evidence`, `completed`/`failed_known` → `prove_result` with `{"adjudication": evidence}`; an unknown terminal is logged and left open. Controls: `a_receipt_that_is_not_this_attempts_result_does_not_adjudicate_it` (dead letter, another attempt's result, own result) and the grown operator control (`attempt_history` last `failed_known`; evidence names the admission); three fixtures' receipts now `work_result` naming their attempt. Mutants M2 (lookup accepts any event) and M3 (node flattens `failed_known`) caught.

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
- Technical: no product change. Control `concurrent_enqueues_to_one_node_serialize_on_its_mark` (node_channel, multi-thread flavour): a holder transaction seeds the mark row idempotently and takes it `FOR UPDATE` (seeded so the control refuses the old allocation at the property — nothing blocks — rather than at its precondition — no row); a spawned `enqueue` is observed by `blocked_on(pool, "node_inbox_cursors")` (`pg_stat_activity`, `wait_event_type = 'Lock'`), has written no row, lands with cursor 2 on release; a `JoinSet` of eight lands `3..=10`. Mutant M1 (the pre-0456 `MAX + 1` enqueue on a pooled connection) → nothing blocks, the control refuses. `next_cursor_in_tx`'s docblock now cites the control. `.4.3` closed with every goal-line clause and the attached clause reconciled MET.

## 2026-09-23 — Two machines holding a job with the same name now each get their answer counted (`SIGNOFF-REPAIR.4.3.3`)

`REASONBRAID-REPAIR-0457`. The third of the four inbox-identity gaps found by `REASONBRAID-DOC-0152`.

- 🔴 **Before:** job names are unique per machine, not per organisation, so two machines in one organisation could hold jobs with the same name. The server's "have I already counted this answer?" check looked only at the job name, so the second machine's answer was treated as a clash with the first's and thrown away. And the inbox view marked a job "answered" on one machine when the answer had come from the other.
- ✅ **Now:** the answer check and the "answered" state both ask *which machine* as well as *which job*. Each machine's answer is counted; a machine re-sending its own answer is still recognised as a repeat.
- ✅ Checked before choosing the fix: today the server never gives two machines the same job name (each job is named after the one event that created it, and one event goes to one machine), so nothing was lost in practice; the check simply permitted it. Old records were re-labelled where it was unambiguous which machine they belonged to.
- ✅ Tested: two new checks failed on the old code exactly where predicted (one answer counted where two were owed; a job wrongly marked answered) and pass now; two deliberately broken versions (the answer check back on the job name alone; the answered state ignoring the machine) were each caught; thirteen suites pass (264 tests) after one test that read the stored answer by the old label was updated; strict lint clean.
- Technical: `api::node_result_fold_key(node_id, command_id)` = `{node}:{command}`; `apply_node_result_in_tx` claims, applies the `Command` and stores its three rejections under it. `migrations/0104`: `CREATE OR REPLACE VIEW node_inbox_state` with `e.node_id = i.node_id` on the `consumed` rung (same columns; `node_presence` untouched) and the stored-row re-key where exactly one node holds the thread-work command id in the tenant. Controls: `two_nodes_holding_one_command_id_each_fold_their_own_result` (node_result_ordering: two roles on one thread, B's row given A's id, both fold, a new-event-id re-emission replays) and `a_foreign_nodes_result_does_not_read_this_nodes_row_consumed` (node_channel); `node_work`'s stored-rejection reader re-keyed. Mutants M2 (claim on the bare id) and M3 (rung without the node) caught.

## 2026-09-23 — Clearing out a machine's old delivered work no longer locks it out or hides later work (`SIGNOFF-REPAIR.4.3.2`)

`REASONBRAID-REPAIR-0456`. The second of the four inbox-identity gaps found by `REASONBRAID-DOC-0152`.

- 🔴 **Before:** the server worked out "how far has this machine got" by looking at the highest-numbered item still in its inbox. When an operator cleared out old delivered items, that number could drop — so a machine that had confirmed up to item 30 was refused on its next check-in as "ahead of the server", and new items handed out afterwards could be numbered 1, 2, 3 again, which the machine had already seen and would skip. Work went silently undelivered and counted against the machine's backlog for ever.
- ✅ **Now:** the server keeps a durable per-machine counter that only ever goes up. Every new item is numbered above it, so clearing out old items removes items, never numbers. A machine that confirmed up to 30 reconnects fine, and the next item is number 31.
- ✅ Existing machines' counters were seeded from what they already held, so nothing moved. A machine whose entire inbox had already been cleared before this change cannot have its lost number recovered; the change says so.
- ✅ Tested: the new prune-everything-then-reconnect check failed on the old code exactly where predicted and passes now (the inbox suite: 12 tests); two deliberately broken versions (the counter overwritten by the inbox's highest number; the reconnect check ignoring the counter) were each caught; ten further suites that hand out or read cursors pass unchanged (223 tests); strict lint clean.
- Technical: `migrations/0103_node_inbox_cursors.sql` (`node_inbox_cursors(node_id PK, high_water)`, seeded `MAX(cursor)` per node); `node_channel::next_cursor_in_tx` — `INSERT … ON CONFLICT DO UPDATE SET high_water = GREATEST(mark, MAX(cursor)) + 1 RETURNING high_water` — used by `enqueue` (now `pool.begin()`), `enqueue_in_tx` and the quarantine replay in `node_admin.rs`; `CURRENT_CURSOR_SQL` = `GREATEST(mark, MAX(cursor))` for `current_cursor` and `current_cursor_in_tx`. Control `a_nodes_cursor_survives_its_inbox_being_pruned` (node_inbox; three arms: reconnect at N after a full prune, the replay numbered above N, a fresh enqueue above that) with the `handshake_at` helper; mutants M2 (mark overwritten by the maximum) and M3 (reader ignores the mark) caught. 30 cleanup plans gain `node_inbox_cursors`.

## 2026-09-23 — One machine can no longer silence another machine's answer by reusing its message id (`SIGNOFF-REPAIR.4.3.1`)

`REASONBRAID-REPAIR-0455`. The first of the four inbox-identity gaps found by `REASONBRAID-DOC-0152`.

- 🔴 **Before:** every machine chooses its own message ids, but the server treated them as if they were unique across all machines. If machine B had already used a message id, machine A's own message under that id was treated as a repeat: it was dropped, and A's finished answer was never counted. And when a machine reconnected and asked "do you already hold my result for this job?", the server answered from *any* machine's records — so B's receipt could close A's uncertain job as done, and B's message id was shown to A.
- ✅ **Now:** receipts are kept per machine. A repeat is only a repeat of that same machine's own message; another machine's use of the same id is that machine's own first message. The reconnect questions are answered only from the asking machine's own receipts.
- ✅ Existing receipts were not rewritten; the database key was widened (an additive change).
- ✅ Tested: the two new checks failed on the old code exactly where predicted and pass now (the machine-channel suite: 49 tests); a deliberately broken version that answered the reconnect questions from any machine's records was caught; six further suites that touch receipts pass unchanged (121 tests); strict lint clean.
- Technical: `migrations/0102_node_events_keyed_per_node.sql` re-keys `node_events` to `(node_id, event_id)` and adds `node_events_node_operation_idx (node_id, operation_id)` (the `0003` operation-only index stays for the `consumed` rung until `.4.3.3`); `record_event_in_tx` conflicts on `(node_id, event_id)`; `event_id_for_operation(node_id, operation_id)` binds the node and both handshake callers pass `req.node_id`. Controls: `a_colliding_event_id_from_another_node_does_not_suppress_this_nodes_receipt` (two nodes through the real `POST /v1/nodes/events`) and `a_foreign_receipt_neither_adjudicates_nor_is_disclosed` (handshake directives + `known_events`, then the node's own receipt does both); mutant M2 (node predicate dropped) caught.

## 2026-09-23 — Ending a partnership now also stops work that was already on its way (`SIGNOFF-REPAIR.5.3.6`)

`REASONBRAID-REPAIR-0454`. Completes the federation work (`SIGNOFF-REPAIR.5.3`) and with it the directory, recruitment and federation lane (`SIGNOFF-REPAIR.5`).

- 🔴 **Before:** after a partnership ended, new work for a partner's agent was refused, but work already queued could still be sent to the partner's machine (topic included) and run there.
- ✅ **Now:** the partner's machine is no longer offered that queued work, and on its next check-in it is told it no longer works for the importing organisation, so anything it already holds for them is refused instead of run.
- ✅ Tested: the new checks failed on the old code and pass now; two deliberately broken versions (the machine keeping the old organisation; the server listing organisations by inbox contents) were each caught; twelve suites pass, including the end-to-end CLI.
- Technical: `replay`'s tail gains `(row tenant = node tenant OR role_execution(payload->>'agent_role').node_id = node)`; `epochs_and_server_time` = own tenant ∪ `role_execution.tenant_id WHERE node_id = node`; node `set_revocation_epochs` deletes then inserts (wholesale). Controls: node `a_tenant_left_out_of_the_map_loses_its_reference`; server `the_handshake_and_poll_carry_the_epochs_of_the_tenants_the_node_serves` (re-expressed); the origin profiles control's post-revocation poll arm. Mutants M2 (merge) and M3 (inbox tenants) caught. Fixture fix: `a_revoked_nodes_withheld_work_is_delivered_to_its_replacement` enqueues under its node's tenant.

## 2026-09-23 — Each delivery to a partner's machine now leaves an audit receipt on both sides (`SIGNOFF-REPAIR.5.3.5.3.3`)

`REASONBRAID-REPAIR-0453`. Completes cross-organisation recruitment (`SIGNOFF-REPAIR.5.3.5`).

- 🔴 **Before:** when a partner's machine picked up a job for an imported agent, neither organisation's audit trail recorded that work had crossed between them.
- ✅ **Now:** the moment the partner's machine confirms it has the job, both sides record a receipt in the same step. The importing side's receipt names the confirmation; the partner's side names the permission record the job runs under. Confirming twice never creates duplicates, and ordinary same-organisation work leaves no such receipt.
- ✅ With this, the whole "recruit a partner's agent" feature is complete: advertise the call, request to join, import, run on either machine, deliver, answer, audit.
- ✅ Tested: the new checks failed on the old code and pass now; two deliberately wrong versions (receipts for same-organisation work; receipts for plain traffic) were each caught; eight suites pass.
- Technical: `migrations/0101` (kind check + `origin_delivery`, `origin_execution`); `ACKNOWLEDGE_SQL … RETURNING cursor, tenant_id, command_id, authz_ref`; `acknowledge_in_tx` writes both receipts per marked cross-tenant admitted row; `acknowledge` runs it on its own transaction. Control grown (ack twice; own-tenant and plain rows beside); mutants M1 (own tenant) and M2 (plain traffic) caught; the first RED was confounded by the wake-gate defect and a clean RED was re-run.

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

The entries before those above were rotated into reachable Git history at the
**eleventh rotation** (`SIGNOFF-REPAIR.11.4.1.6`, which owns this ledger’s rotation). The exact predecessor — this file as it
stood at the commit named below, which is the object every retired record was
checked against before this notice was written — is:

```bash
git show 9824ffcf1aa533cf78fd327098a10d4c8be9b42f:DEV_NOTES.md
```

That snapshot is 71576 bytes and 402 lines, and contains 37 dated
entries; its Git blob is `d987719b9bdab737106030c2618bb14328b5a100` and its SHA-256 is
`9fe7fa4a1d227678f087c4fcfc3b42bba4b55fe439aa683d335f5b69b122a38b`. It carries the tenth rotation's
notice in turn, and each earlier notice names the one before it, so the chain
walks all the way back. `docs/decisions/2026-09-09_changelog-rotation.md` holds
the first transition's evidence.

⛔ **12 record(s) rotated out, 26 kept, lossless** — every retired heading was retrieved from the
predecessor named above before this notice was written, and every figure in it was re-derived from that object with
`git rev-parse`, `git cat-file` and SHA-256 rather than typed. ⭐ The cut is DERIVED, not chosen: it retires whole
records until the ledger has at least 10 commits of runway at the p90 entry size measured over the last
60 non-rotation commits — because two rotations that stopped at the threshold instead left 344 and 296 bytes and
the first forced another rotation on the very next commit (`SIGNOFF-REPAIR.11.4.1.6`).
