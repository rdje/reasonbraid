# DEV_NOTES.md

## 2026-09-24 — Two requests at once can no longer both squeeze under a limit (`SIGNOFF-REPAIR.4.5.3`)

`REASONBRAID-REPAIR-0485`. The third fix from the budget review (`REASONBRAID-DOC-0156`).

- 🔴 **Before:** the budget limit, the spending cut-off and the invite quota each looked at what was already used and then recorded the new use, with nothing stopping a second request in between. Two requests arriving together could both see room for one and both get in.
- ✅ **Now:** each check locks the record it decides against. A second request waits for the first to finish, then counts it and is refused if there is no room left.
- ✅ Tested: all three new tests failed on the old code and pass now. Each proves its own lock (removing any single lock fails exactly its own test), and each checks that the waiting request then decides correctly. The broad live suite passes.
- 🔴 **Also fixed (`SIGNOFF-REPAIR.11.35`):** that broad run found a test suite (`bootstrap_recovery`) failing on `main` since 23 September. A table was added to the middle of its checklist, which was matched by position, so every later entry shifted. It now matches by table name.
- Technical: `FOR UPDATE` on `budget_ceilings` (`create_reservation_in_tx`), `spend_breakers` (`check_spend_breaker_in_tx`, after the ceiling), `usage_quotas` (`quota::check_in_tx`); controls observe the wait in `pg_stat_activity`.

## 2026-09-24 — Budget totals can no longer overflow (`SIGNOFF-REPAIR.4.5.2`)

`REASONBRAID-REPAIR-0484`. The second fix from the budget review (`REASONBRAID-DOC-0156`).

- 🔴 **Before:** the budget was added up with plain arithmetic, and usage figures come from the machines. Two absurd reports (2⁶³ tokens each) crashed the server's limit check. In a release build they would have wrapped round to a small number and let the limit lend again.
- ✅ **Now:** a total too large to count is a clear refusal that names what overflowed. New work is refused, the usage report answers `ledger_overflow`, and a machine's own budget counts itself spent. Large usage is still recorded in full, because the work did happen.
- ✅ Tested: all four new tests failed on the old code and pass now; nine deliberately broken versions were all caught, two of them only after I strengthened a test; live suites pass; strict lint clean.
- Technical: `BudgetDimensions::add -> Result`, `BudgetError::Overflow`; `budget.rs` held/spend/projection `map_err` to `Unavailable`; `api::ledger_overflow` (`500 ledger_overflow`); `LocalBudget` try_reserve refuses, settle saturates to `u64::MAX` on overflow.

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
- Technical: `AMBIGUOUS_ATTEMPT_ACTIONS` re-ask → available, `who` = the tenant's administrator, `effect` = the verb; CLI `rb node replay --allow-possible-duplicate --reason` via `possible_duplicate_reason` + `replay_request_body` (unit-tested; mutants 7/7). Closes `.4.4.7.2`, `.4.4.7`, `.4.4`.

## 2026-09-24 — An administrator can now authorize re-running a job whose outcome is unknown (`SIGNOFF-REPAIR.4.4.7.2.2`)

`REASONBRAID-REPAIR-0481`.

- 🔴 **Before:** when a provider call's outcome was unknown, the machine rightly refused to run the job again on its own. But nothing let anyone approve a re-run while accepting the risk of paying twice, so the job stayed stuck.
- ✅ **Now:** the tenant's administrator can approve the re-run with a stated reason. The job gets its own new budget hold (the original stays reserved in case the first call did charge), the machine runs it again, and the answer lands. The approval is recorded in the audit trail under its own name, applies only to jobs stuck for exactly this reason, and is refused if the conversation's budget has no room.
- ⚠️ Next: the command-line action, and marking the option available on the operator's screen.
- ✅ Tested: a full live run (a lost answer, the approval, the re-run, the answer landing) that could not happen before; the audit vocabulary checks; five further live suites; both deliberately broken versions were caught; strict lint clean.
- Technical: core `AdministrativeOperation::NodeCommandReplayPossibleDuplicate` + `RETRY_REQUIRES_AUTHORIZATION`; `migrations/0106` (sixteen kinds); `authority::replay_command_with_possible_duplicate_in_one_transaction` (precondition `dead-lettered: retry_requires_authorization`, fresh reservation via `create_reservation_in_tx`, `jsonb_set` flag + reservation, re-sequence); `POST /v1/nodes/replay` `allow_possible_duplicate` + `reason` → `reservation_id`. Control `node_work::a_possible_duplicate_replay_re_runs_an_unknown_outcome`.

## 2026-09-24 — A machine can now receive permission to retry a job whose outcome is unknown (`SIGNOFF-REPAIR.4.4.7.2.1`)

`REASONBRAID-REPAIR-0480`. The first part of the "ask again, accepting a possible duplicate" action.

- 🔴 **Before:** once a machine had a job, nothing sent later could change it. That included permission to retry a job whose outcome is unknown, so even an authorized retry could never reach the machine.
- ✅ **Now:** when a job is sent again with a fresh approval, the machine takes two things from it: whether a possible duplicate is allowed, and which budget hold to run under. It keeps everything that says what the job is. A resend can extend what a machine may risk; it can never change what it was asked to do.
- ⚠️ Next: the server-side action that sends such a resend, then the command-line action and a full end-to-end check.
- ✅ Tested: two new checks failed on the old code and pass now; 112 machine tests and two live suites pass; every deliberately broken version was caught; strict lint clean.
- Technical: `journal.rs` `with_refreshed_authorization(held, redelivered)` refreshes only `allow_possible_duplicate` and `reservation` in `record_command`'s fresh-decision branch. Controls in `worker_retry_policy`. Mutants 1/1 + hand 2/2.

## 2026-09-24 — The recovery options now say which ones actually exist (`SIGNOFF-REPAIR.4.4.7.1`)

`REASONBRAID-REPAIR-0479`.

- 🔴 **Before:** the list of recovery options for a job with an unknown outcome included "ask again, accepting a possible duplicate charge", as if it could be done. It could not, and the runbook and the book said the same.
- ✅ **Now:** every option says whether it is available today. That one says it is not, why, and what will change it. The runbook and the book describe what an operator can really do: a revision can be requested again with a new challenge; a first answer cannot be requested again yet.
- ⚠️ Next: building the "ask again, accepting a possible duplicate" action itself.
- ✅ Tested: a new check failed on the old code and passes now; the live suite passes; a deliberately broken version was caught.
- Technical: `AMBIGUOUS_ATTEMPT_ACTIONS: [AmbiguousAttemptAction; 4]` with `unavailable_because`; response fields `available` / `unavailable_because`; `node_channel` control extended; runbook and `node-channel.md` corrected.

## 2026-09-24 — Checked: operators were told to use a recovery action that does not exist (`SIGNOFF-REPAIR.4.4.7`)

`REASONBRAID-DOC-0155`. A review; no code changed.

- 🔴 **Found:** when a job's outcome is unknown, the operator's screen, the runbook and the book all suggest "ask again, accepting the risk of a duplicate charge". Nothing lets anyone do that. For a first answer (as opposed to a revision) there is no way at all to ask again today.
- ⚖️ **Decided:** first make the screen, runbook and book tell the truth about which actions are available now; then build the "ask again, accepting a possible duplicate" action properly, with its own separate budget, because the roadmap names it as one of the four ways to handle an unknown outcome.

## 2026-09-24 — The charter lookup no longer blames the caller for server faults (`SIGNOFF-REPAIR.4.4.10.1.2`)

`REASONBRAID-REPAIR-0478`.

- 🔴 **Before:** looking up a governance charter answered any database problem as "your request is invalid", and quoted the database's internal error message back to the caller. A real server fault was blamed on the user, and internal details leaked.
- ✅ **Now:** a server fault is reported as a server fault, with the details kept in the server's log. A request containing the null character gets the clear "cannot be stored" reply. Publications were checked too: that character cannot reach their storage, because the input is validated first.
- ✅ Tested: a new check (the null character, plus a simulated database fault) failed on the old code and passes now; three live suites pass; both deliberately broken versions were caught; strict lint clean.
- Technical: `CharterError::UnrepresentableInput` / `PublicationError::UnrepresentableInput` built by classifying `storage` constructors; the charter GET maps `Storage` → logged 500 and the variant → `400 unrepresentable_input` (`api::unrepresentable()`). Control `command_api::the_charter_read_tells_the_callers_input_from_the_stores_fault` (NUL digest; table renamed away for one request).

## 2026-09-24 — More places now say clearly when input can never be stored (`SIGNOFF-REPAIR.4.4.10.1.1`)

`REASONBRAID-REPAIR-0477`.

- 🔴 **Before:** in seven places (agent profiles, snapshots, derivations, assessments, references, quotas and evaluations), input holding the null character was still answered "internal server error" rather than the clear "cannot be stored" reply.
- ✅ **Now:** all seven give the clear, permanent reply through one shared rule; genuine server faults still report as server faults.
- ⚠️ Tracked next: two remaining places (publications and charters) that lose the information needed to tell the two apart.
- ✅ Tested: a new check failed on the old code (a profile with that character) and passes now; the shared rule is checked against every kind of database error; six live suites pass; strict lint clean.
- Technical: `api::storage_failure(cause, context)` routes `EvaluationError`, both `QuotaError`, `AssessmentError`, `DerivationError`, `SnapshotError`, `ReferenceError` storage arms and `profile_error` through `unrepresentable_input`. Unit `storage_failures::only_unrepresentable_input_is_the_callers` (stand-in `DatabaseError`); live `profiles::a_profile_holding_nul_is_refused_as_the_callers`.

## 2026-09-24 — Time spent on jobs now counts against a conversation's time budget (`SIGNOFF-REPAIR.4.4.6.1`)

`REASONBRAID-REPAIR-0476`.

- 🔴 **Before:** each conversation has a time budget (10 minutes by default) alongside its call and token budgets. Time was never charged once a job finished, so the budget only limited how many jobs could run at the same moment, never the total. However long the jobs took altogether, the time budget never ran out.
- ✅ **Now:** each machine times every job, rounding up to whole seconds, and reports the time alongside the tokens. The server charges it, so a conversation's time budget runs out like its token budget does.
- ✅ Tested: two checks (one on the machine, one against the real server) failed on the old code and pass now; 199 core and machine tests and three live suites pass. The deliberately broken versions exposed a gap ("always charge one second" went unnoticed because every test job was quick); a longer job was added, and all are now caught. Strict lint clean.
- Technical: `BudgetDimensions::attempt_usage(input, output, wall_clock_seconds)`; supervisor `seconds_since(dispatched_at)` (ceil, min 1) at every settlement; `ExecutionReport.wall_clock_seconds`; worker adds `usage.wall_clock_seconds`; `api::settle_work_item_reservation` reads it. Controls: two in `supervisor_bounds`, one live assertion in `node_work`.

## 2026-09-24 — The web console's inbox panel works (`SIGNOFF-REPAIR.4.4.2.2`)

`REASONBRAID-REPAIR-0475`. A defect found by the previous fix.

- 🔴 **Before:** the console's "Inspect inbox" panel had never worked. It asked the server with the wrong parameter name, so every request was refused, and it tried to show a field that does not exist. The check meant to keep the console honest had itself been written with the wrong name, so it agreed with the mistake.
- ✅ **Now:** the panel works and shows each job's delivery state, whether its answer was refused (and why), and any quarantine. The check now tests the panel against the server's own definitions, so the two cannot drift apart unnoticed again.
- ✅ Tested: the new check failed on the old console, once for each of the two mistakes, and passes now; strict lint clean.
- Technical: `web/app.js` inbox panel → `node_id`, `delivery_state`, `result_refusal`; `ui.rs` `the_inbox_panel_speaks_the_servers_contract` parses the panel's query with `axum::extract::Query::<InboxInspectionParams>::try_from_uri` and checks every `r.<field>` against a serialized `InboxRow`; `web-ui.md` corrected.

## 2026-09-24 — Operators can now see when a finished job's answer was refused (`SIGNOFF-REPAIR.4.4.2.1`)

`REASONBRAID-REPAIR-0474`.

- 🔴 **Before:** in the operator's inbox view, a job whose answer the server refused (for example because the conversation had closed) looked exactly like one whose answer was accepted. Both showed as "done".
- ✅ **Now:** each job in the view shows whether its answer was refused, and why. Accepted answers show nothing extra. The same information reaches the assistant-facing (MCP) view.
- 🔴 **Found along the way (tracked, fixed next):** the web console's inbox panel has never worked. It asks the server in the wrong way, and it reads a field that does not exist.
- ✅ Tested: two checks failed on the old code and pass now; six live suites pass; a deliberately broken version was caught; strict lint clean.
- Technical: `api::inbox_inspection` LEFT JOIN `idempotency` on `(tenant_id, node_id || ':' || command_id)` → `InboxRow.result_refusal` = `response_result->'error'` when `ok = 'false'`. Controls in `node_work` (refused → code; applied → null). Hand mutant (pre-0104 key) caught.

## 2026-09-24 — An answer containing the null character now gets through, marked (`SIGNOFF-REPAIR.4.4.10.3`)

`REASONBRAID-REPAIR-0473`. The last of the three fixes for the lock-out measured by `REASONBRAID-DOC-0154`.

- 🔴 **Before:** an answer containing the invisible "null" character could never be stored, so the whole paid answer was lost over one character.
- ✅ **Now:** the machine swaps each null character for the standard "unreadable character" symbol (�), which shows exactly where it was, and the answer records how many were swapped. Nothing else in the answer changes, and answers without the character are untouched.
- ⚖️ **A decision taken for you to review:** the alternative was to reject such an answer outright, keeping the principle that answers pass through unchanged but losing the work. The reasoning is recorded, and switching is a one-line change.
- ✅ Tested: a new check failed on the old code (the answer was lost) and passes now; the real server stores such an answer as a contribution; 108 machine tests pass; all eight deliberately broken versions were caught; strict lint clean.
- Technical: `worker.rs` `storable_content(&chunks) -> (String, usize)`: U+0000 → U+FFFD, count in the result's `nul_replaced` (absent when 0). Decision `docs/decisions/2026-09-24_nul-in-provider-output-is-replaced-visibly.md`. Controls: unit, two `worker_refusals`, live `provider_output_holding_nul_lands_as_a_contribution`. Mutants 8/8.

## 2026-09-24 — A machine no longer locks itself out on an answer the server can never accept (`SIGNOFF-REPAIR.4.4.10.2`)

`REASONBRAID-REPAIR-0472`. The second of the three fixes for the lock-out measured by `REASONBRAID-DOC-0154`.

- 🔴 **Before:** when the server refused an answer permanently (because of what the answer contained), the machine treated it as a broken connection. It reconnected, resent the same answer, was refused again, and never got back to work.
- ✅ **Now:** the machine records the refusal (the server's reason included), stops offering that answer, and carries on with its other work. Only refusals about the answer's own contents count as permanent. Login problems, outages and version mismatches are still handled by reconnecting.
- ⚠️ Tracked next: the answer itself is still lost when it contains the null character. The last fix decides what the machine does with that character.
- ✅ Tested: two new checks (one on first sending, one on resending after a reconnect) failed on the old code and pass now; 105 machine tests, five live suites and the two-machine demonstration pass; all ten deliberately broken versions were caught; strict lint clean.
- Technical: `ChannelError::permanent_refusal()` (400/413/422, not `protocol_incompatible`; unit-pinned); `Node::deliver_at` is the one send path for `emit_event`, `deliver_journaled_event` and the reconcile's re-emission; a permanent refusal → `record_event_refusal` + `acknowledge_event` → `EventDelivery::Refused`. Stub `start_refusing(epochs, marker)`; `tests/worker_refusals.rs` 2 controls. Mutants 8/8 + hand 2/2.

## 2026-09-24 — The server now says clearly when an answer can never be stored (`SIGNOFF-REPAIR.4.4.10.1`)

`REASONBRAID-REPAIR-0471`. The first of the three fixes for the lock-out measured by `REASONBRAID-DOC-0154`.

- 🔴 **Before:** input holding the invisible "null" character, which the database cannot store, was answered "internal server error", a reply that looks like a temporary outage. A machine given that reply for its answer kept retrying for ever.
- ✅ **Now:** such input is refused with a clear, permanent "cannot be stored" reply, and nothing of it is kept. This covers both the machines' channel and the command interface people use. A genuine server fault on normal input still reports as a server fault, so the two are never confused.
- ⚠️ Tracked next: the machine side (stop retrying anything refused permanently); and seven less-used command paths that still give the old reply.
- ✅ Tested: a new check failed on the old code and passes now; eight live suites pass; all five deliberately broken versions of the new rule were caught, after the first round showed a missing check (a real server fault was being confused with bad input), which was added; strict lint clean.
- Technical: `api::unrepresentable_input` (SQLSTATE `22P05`/`22021`) → `400 unrepresentable_input` in `From<sqlx::Error>` for the node channel's `ApiError` and `ControlApiError`; `ApplyError::Sql` and unrepresentable `EvaluationError::Storage` routed through it; `status_for_code` maps it. Control `node_work::input_the_store_cannot_hold_is_refused_as_the_callers` (jsonb, text, command API, and an injected store fault staying 500).

## 2026-09-24 — Measured: a single invisible character in an answer can lock a machine out for good (`SIGNOFF-REPAIR.4.4.10`)

`REASONBRAID-DOC-0154`. A check of a suspected risk; it turned out to be real.

- ✅ **Confirmed safe:** the largest answer a machine can now produce (256 KB, `.4.4.6`) is accepted by the server, even in its most expensive encoding. Double that is refused, so the server's limit really exists.
- 🔴 **Found:** an answer containing one particular invisible character (the "null" character, which the database cannot store) is refused by the server with a message that looks like a temporary outage. The machine keeps retrying the same answer and can never get back to work. A provider can produce that character; the test provider's own sample of garbled output contains it.
- ⚖️ Now three tracked fixes, in order: the server rejects such an answer clearly and permanently; a machine stops retrying anything the server rejects permanently; and a machine decides up front what to do with that character.
- Technical: census of `node_channel::events`' error statuses (domain refusals are 200 + `refused`); `node_work::the_largest_result_a_node_can_produce_is_accepted` (256 KiB of U+0001 accepted; 512 KiB → 413). U+0000 → `jsonb` "unsupported Unicode escape sequence" → `500 dependency_unavailable` → `calls_for_reconcile` → re-emission → wedge. Split `.4.4.10.1`–`.3`.

## 2026-09-24 — A provider that never answers no longer freezes the machine, and answers have a size limit (`SIGNOFF-REPAIR.4.4.6`)

`REASONBRAID-REPAIR-0470`. The sixth recovery gap found by `REASONBRAID-DOC-0153`.

- 🔴 **Before:** every job came with a time limit, but nothing enforced it. A provider that took a job and never answered froze the machine for good. Answers could also be any size, including sizes the server would refuse to accept.
- ✅ **Now:** when the time limit passes, the machine tells the provider to stop and records the job as "outcome unknown", with the reason. It never guesses the job failed or succeeded, because the provider may have done the work. An answer larger than 256 KB is stopped and recorded as failed, naming the limit; a partial answer is never passed off as a whole one.
- ✅ Tested: two checks failed on the old code (a frozen job, and an oversized answer accepted) and pass now, plus four more; 102 machine tests, four live suites and the two-machine demonstration pass. The deliberately broken versions first exposed a gap: nothing checked that the provider was actually told to stop. The checks were strengthened until all were caught. Strict lint clean.
- ⚠️ Tracked next: whether a machine can get stuck on an answer the server refuses outright; and charging a job's elapsed time to its budget.
- Technical: supervisor `within` (`tokio::time::timeout_at`) over `invoke` and every `next()`; `cancel_within_grace` (5 s); `Terminal::Lost(reason)` → extracted `settle_unknown(UnknownOutcome{…}, …)`; `MAX_OUTPUT_BYTES = 256 KiB` → `Terminal::FailedKnown`; `tokio` `time` feature. `tests/supervisor_bounds.rs` 6 controls (`SlowCancel`, `SilentInvoke`). Mutants: first pass 3 missed in `cancel_within_grace`, strengthened to 0 missed; hand mutant on the `invoke` wait caught.

## 2026-09-24 — A machine that cannot reconnect now waits longer between tries (`SIGNOFF-REPAIR.4.4.5.3`)

`REASONBRAID-REPAIR-0469`. Completes the fifth recovery gap found by `REASONBRAID-DOC-0153`.

- 🔴 **Before:** a machine that could not reconnect tried again every second, for ever. A server down for an hour met 3,600 reconnect attempts from each machine.
- ✅ **Now:** the wait doubles after each failure (1, 2, 4 … seconds) up to a minute, and goes back to one second once a reconnect succeeds.
- ✅ Tested: the waiting rule has its own check; 96 machine tests and the two-machine demonstration pass; three deliberately broken versions were all caught; strict lint clean.
- Technical: `reasonbraid_node::reconcile_backoff(n)` = `1s.saturating_mul(2.saturating_pow(n)).min(60s)`; `rb-node` loops `while let Err(e) = node.reconcile()` with a per-recovery failure count. No jitter (decided; trigger: a fleet profile).

## 2026-09-24 — A delivered job now shows as done, and stops counting against the machine's capacity (`SIGNOFF-REPAIR.4.4.9`)

`REASONBRAID-REPAIR-0468`. A defect found while testing the previous fix.

- 🔴 **Before:** a job a real machine had finished and delivered never showed as "done" in the operator's inbox view, only "received". Worse, the same check decides how busy a machine is, so every finished job kept counting as "in progress". A machine allowed N jobs at a time stopped getting new work after its first N, until old records were cleaned out. The tests had not noticed because they built their fake results in a shape no real machine produces.
- ✅ **Now:** the server recognises a finished job by the job the answer names, so it shows as done and frees its capacity slot at once. The tests now use results shaped the way a real machine sends them, plus one end-to-end check with a real machine.
- ✅ Tested: five checks failed on the old code (two of them the capacity checks) and pass now; six live suites and the two-machine demonstration pass; a deliberately broken version was caught; strict lint clean.
- Technical: `migrations/0105_node_inbox_consumed_by_command.sql` redefines `node_inbox_state`'s `consumed` rung as `e.payload->>'kind' = 'work_result' AND e.payload->>'command_id' = i.command_id` (was `e.operation_id = i.command_id` since `0021`); four `node_channel.rs` fixtures re-shaped to `op_…` ids; `node_work::a_completed_item_is_never_dead_lettered` asserts `consumed`.

## 2026-09-24 — Finished jobs are no longer reported as undeliverable (`SIGNOFF-REPAIR.4.4.8`)

`REASONBRAID-REPAIR-0467`. A defect found while testing the previous fix.

- 🔴 **Before:** one round after a job finished successfully, the machine reported it as "undeliverable", and the server filed it in the dead-letter pile. An operator looking at the inbox saw every successful job marked as failed.
- ✅ **Now:** the machine treats a finished job as finished and says nothing more about it. The server also refuses on its own to file a job as undeliverable once its answer has arrived, whoever sends the report.
- 🔴 **Found along the way (now tracked, fixed next):** a successfully delivered job shows as "received" instead of "done" in the operator's inbox view.
- ✅ Tested: two new checks (one on the machine, one against the real server) failed on the old code and pass now; 185 core and machine tests, six live suites and the two-machine demonstration pass; three deliberately broken versions were all caught; strict lint clean.
- Technical: `RetryVerdict::Settled` for `completed` (`reconciled` stays `Refuse`, deliberately); `Worker::process` returns on `Settled`; `api.rs` `work_dead_lettered` fold adds `AND NOT EXISTS (… node_events e WHERE e.node_id = node_inbox.node_id AND e.payload->>'kind' = 'work_result' AND e.payload->>'command_id' = node_inbox.command_id)`. Controls: core `a_completed_attempt_is_settled_not_refused`, node `a_delivered_result_is_never_followed_by_a_dead_letter`, live `a_completed_item_is_never_dead_lettered`. Hand mutants 3/3.

## 2026-09-23 — A dropped connection or one unanswerable reply no longer shuts a machine down (`SIGNOFF-REPAIR.4.4.5.2`)

`REASONBRAID-REPAIR-0466`. The second part of the fifth recovery gap found by `REASONBRAID-DOC-0153`.

- 🔴 **Before:** three ordinary problems stopped a machine's program entirely: the connection dropping while it sent an answer, a provider reply going missing, and one job with unreadable instructions. The last also left every job after it in the same round undone.
- ✅ **Now:** a dropped connection makes the machine reconnect and resend. A missing reply is recorded as "unknown" and left for a person to decide, as designed. A job with unreadable instructions is reported once as undeliverable, and the other jobs carry on.
- 🔴 **Found along the way (now tracked, fixed next):** every job that finishes successfully is wrongly reported as "undeliverable" one round later.
- ✅ Tested: three new checks failed on the old code and pass now; 94 machine tests, six live suites and the two-machine demonstration pass; four deliberately broken versions were all caught; strict lint clean.
- Technical: `WorkerError::calls_for_reconcile()` += `Node(NodeError::Channel(_))`; `Worker::process` maps `SupervisorError::OutcomeUnknown` to a log + `Ok`; `Worker::tick` dead-letters a `MalformedPayload` item via `report_dead_letter` and continues. Stub: `poll`, `start_with_epochs`. `tests/worker_failures.rs` 3 controls; `node_replacement` updated to the new contract. Mutants: tool 2/2 on `calls_for_reconcile`, hand 2/2 on the two arms.

## 2026-09-23 — A machine no longer waits for ever on a server that stops answering (`SIGNOFF-REPAIR.4.4.5.1`)

`REASONBRAID-REPAIR-0465`. The first part of the fifth recovery gap found by `REASONBRAID-DOC-0153`.

- 🔴 **Before:** if the server accepted a machine's connection and then went silent, the machine waited for ever. It stalled completely, and because nothing reported an error, it never tried to reconnect. Measured: still waiting after 45 seconds, with no end in sight.
- ✅ **Now:** the machine gives up after 10 seconds trying to connect, or 30 seconds waiting for an answer. It then treats the silence like any other broken connection and reconnects. Measured: it gives up at 30 seconds exactly.
- ✅ Tested: the new checks use a stand-in server that accepts and never replies; 91 machine tests and three live suites pass; two deliberately broken versions were caught (one more could not be built); strict lint clean. One gap is stated in the record: only a 30-second check, run once and not kept, would notice the unbounded client being put back by hand.
- Technical: `ChannelTimeouts { connect: 10 s, request: 30 s }`, `bounded_client` (`connect_timeout` + `timeout`) in both `NodeChannel` constructors, `with_timeouts`/`timeouts()`; `tests/support/control_plane.rs` `StalledServer`; `tests/channel_timeouts.rs` 2 controls. RED probe 45 s timeout → GREEN 30.00 s. Mutants: `bounded_client → Default` and `timeouts → Default` caught, `with_timeouts → Default` unviable.

## 2026-09-23 — A machine that is not properly connected no longer starts paid work (`SIGNOFF-REPAIR.4.4.4.2.2`)

`REASONBRAID-REPAIR-0464`. Closes the fourth recovery gap found by `REASONBRAID-DOC-0153`.

- 🔴 **Before:** a machine whose reconnect had failed still sent work to the provider and paid for it, although it could not yet deliver the answer. If it ever noticed, it shut itself down.
- ✅ **Now:** such a machine refuses to start paid work. The work waits untouched, and the machine reconnects and carries on instead of shutting down. Safety checks that need no connection still run first.
- ✅ Tested: a new check failed on the old code (the machine did the work) and passes now; eleven existing checks now run on properly connected test machines; 89 machine tests, six live suites (91 tests) and the full two-machine demonstration pass; three deliberately broken versions were all caught; strict lint clean.
- Technical: gate in `Worker::process` after the retry and cached-decision gates, before `execute_attempt_emitting`, returning `WorkerError::Node(NodeError::NotSchedulable)` with nothing journaled; `WorkerError::calls_for_reconcile()` (`Channel` | `Node(NotSchedulable)`) drives `rb-node`'s loop; `support::control_plane::reconciled_node` shared by `worker_cached_decision` (6), `worker_retry_policy` (2) and `worker_dead_letter` (3). Mutants 3/3 caught (`worker.rs:376:12 delete !`, `calls_for_reconcile → true/false`).

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

The entries before those above were rotated into reachable Git history at the
**twelfth rotation** (`SIGNOFF-REPAIR.11.4.1.6`, which owns this ledger’s rotation). The exact predecessor — this file as it
stood at the commit named below, which is the object every retired record was
checked against before this notice was written — is:

```bash
git show 75fb20140e3b78a9a821cc36557bba0dcf45023d:DEV_NOTES.md
```

That snapshot is 73568 bytes and 430 lines, and contains 41 dated
entries; its Git blob is `42060e3a6f542727d42da75f4e1e1d4060d65756` and its SHA-256 is
`d2bc84f3f3df8d7b4fa031b9556fd5d2128a9502544b236dceab71f606816439`. It carries the eleventh rotation's
notice in turn, and each earlier notice names the one before it, so the chain
walks all the way back. `docs/decisions/2026-09-09_changelog-rotation.md` holds
the first transition's evidence.

⛔ **13 record(s) rotated out, 29 kept, lossless** — every retired heading was retrieved from the
predecessor named above before this notice was written, and every figure in it was re-derived from that object with
`git rev-parse`, `git cat-file` and SHA-256 rather than typed. ⭐ The cut is DERIVED, not chosen: it retires whole
records until the ledger has at least 10 commits of runway at the p90 entry size measured over the last
60 non-rotation commits — because two rotations that stopped at the threshold instead left 344 and 296 bytes and
the first forced another rotation on the very next commit (`SIGNOFF-REPAIR.11.4.1.6`).
