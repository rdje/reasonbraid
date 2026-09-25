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

## 2026-09-25 — Evidence checked: four real problems found (`SIGNOFF-REPAIR.7.4`)

`REASONBRAID-DOC-0163`. A closing check; no code changed.

- 🔴 **Must be fixed:** evidence that was deleted can still be built on, still supports assessments, and is quietly cited again if the same content is fetched again (`.7.4.6`). Anyone can sign an assessment with someone else's name (`.7.4.7`). Resubmitting an assessment with a different quote silently keeps the first one (`.7.4.8`). Fetching evidence again does not renew its "fresh until" date, although the book says it does (`.7.4.9`).
- ⏸️ **Deferred with a trigger:** evidence quarantine (the book already says it does not exist), unchecked derivation labels, and a write order that can leave an unused stored file behind.
- ✅ **Holds:** a quoted excerpt must really appear in the evidence; deletion uses the server's clock; one organisation cannot delete evidence another relies on.

## 2026-09-25 — The repair work now has a finish line (`SIGNOFF-REPAIR.12`)

`REASONBRAID-DOC-0162`. Decisions taken at the director's request; no product code changed.

- 🔴 **Why:** the repair work was meant to end when every finding was closed, but every review finds new ones: yesterday 17 items were closed and 17 opened, so it never got closer. A first count also said 50 open items when the true number is 65, because it missed an older way items record their state.
- ✅ **The finish line:** the repair work ends at a "bug bar", the rule security and reliability teams use to decide what must be fixed before a release. What must be fixed: anything that lets one organisation touch another's data, money or service; anything that silently loses or corrupts data, budgets, evidence or publications; anything the book claims that the code does not do; and any check that gives a false answer. Everything else is deferred with a written trigger that reopens it, and the book states the limit meanwhile.
- ✅ **The triage of all 65:** 8 close after a quick check, 1 duplicate, 25 must be fixed (plus a new one: counting open items reliably), 12 deferred with triggers, 19 group headings that close with their parts. Every open item now says which it is.
- ⏳ **Faster checking, same rigour, being measured:** the slow step, mutation testing, rebuilds about 50 unrelated test programs for every change it tries. My first attempt to narrow it made it slower (2 hours instead of 77 minutes), because the tool ignored the setting at the build step. A second setting is being measured; the result and the rule go in the next entry.
- 📖 The book's qualification chapter gains "When the corrective work ends", and its out-of-date row on machine recovery and budgets now says that work is complete.

## 2026-09-25 — A fake fetcher can no longer silently stop everyone's web fetches (`SIGNOFF-REPAIR.7.1.3`)

`REASONBRAID-REPAIR-0494`.

- 🔴 **Before:** any organisation's administrator could register a "fetcher" advertised as fast. It would be ranked first for every organisation, the server had nothing to run it with, and every fetch then returned an empty answer with no error. Reproduced live: the test's leftover entry even silenced four other tests.
- ✅ **Now:** the server uses the first ranked fetcher it can actually run, and names any it skipped. If none can run, the answer says so by name.
- ⏭️ **Next:** decide who should be allowed to add fetchers to the shared list at all (`.7.1.3.1`).
- ✅ Tested: the new test failed on the old code and passes now; the deliberately broken versions were caught; broad live suite and strict lint pass.

## 2026-09-24 — Resource and resolver ownership reviewed: three real gaps found (`SIGNOFF-REPAIR.7.1`)

`REASONBRAID-DOC-0161`. A review against the code; no code changed.

- 🔴 **Most serious:** any organisation's administrator can register a new "resolver" (the component that fetches a web resource), and the list is shared by everyone. A fake one advertised as fast would outrank the real fetcher. Every organisation's fetches would then silently do nothing, with no error shown. Owned by `.7.1.3`, next.
- 🔴 **Also real:** the first organisation to cite a web address decides how it is fetched for everyone else (`.7.1.4`); and when two fetchers tie, which one is used is left to chance (`.7.1.5`).
- ✅ **Holds:** fetched content is checked against its expected fingerprint, a corrected fetcher description replaces the old one completely, the isolation rules compare in the right direction, and who may write to the shared lists was settled earlier.

## 2026-09-24 — Tenant-owned administration checked and closed (`SIGNOFF-REPAIR.3.5`)

`REASONBRAID-DOC-0160`. A closing check; no code changed.

- ✅ **Closed:** every piece of this review was already done, but it had never been formally closed. Each of its goals was checked against the code and the test that proves it. One organisation's administrator cannot change or read another organisation's machines, queues, enrollment tokens or spending cut-off. A frozen organisation's administrator can still look at its own records, as approved.
- ⏭️ **Next:** ownership of resources and resolvers (`.7.1`), which has not been reviewed yet.

## 2026-09-24 — The server's certificate authority now renews itself, and the node-machine review is complete (`SIGNOFF-REPAIR.4.1.8.2`)

`REASONBRAID-REPAIR-0493`. The second of the two steps decided in `REASONBRAID-DOC-0159`.

- 🔴 **Before:** nothing ever renewed the certificate authority. After a year every machine would have been locked out at once.
- ✅ **Now:** when a third of the authority's life is left (about four months for a one-year authority), the server creates its successor by itself, without a restart, and switches new certificates to it. Machines certified by the old one keep working. With several servers, exactly one creates the successor and the others adopt it. The health check has a new line, `server_ca_renewal`, which only turns red if a due renewal fails.
- ⭐ **The node-machine review is complete:** enrollment, certificates, leases, delivery, recovery and budgets.
- ✅ Tested: the live test shows one successor created, reused on every later check, adopted by a second server, and the old authority's machines still trusted. Every deliberately broken version was caught, some only after I added tests. This one could not be shown failing on the old code, because the feature did not exist there at all; that is recorded. Broad live suite and strict lint pass.

## 2026-09-24 — The server can hold more than one certificate authority (`SIGNOFF-REPAIR.4.1.8.1`)

`REASONBRAID-REPAIR-0492`. The first of the two steps decided in `REASONBRAID-DOC-0159`.

- 🔴 **Before:** the server had exactly one certificate authority, at every level: stored as one row, loaded once, and the only thing it trusted. Even a stored successor would have been ignored.
- ✅ **Now:** the server keeps every authority it has. New machine certificates come from the newest, and a machine certified by an older one keeps working until that authority expires. Nothing changes while there is only one.
- ⏭️ **Next:** the server creates the successor by itself before the current authority runs out, and warns if it ever fails to (`.4.1.8.2`).
- ✅ Tested: the new test failed on the old code; a live test shows a machine from the first authority still connecting after a second one takes over, and a new machine getting a certificate from the second. Every deliberately broken version was caught, two only after I added tests, including one for the exact second an authority expires. Broad live suite and strict lint pass.

## 2026-09-24 — Decided how the server's certificate authority renews itself (`SIGNOFF-REPAIR.4.1.8`)

`REASONBRAID-DOC-0159`. A design decision; no code changed.

- ✅ **Decided:** the server will keep more than one certificate authority. It trusts any that hasn't expired, issues from the newest, and creates a successor by itself once a third of the current one's life remains, with no restart. The health check will warn if that renewal ever fails to happen. This is how widely used systems (SPIFFE/SPIRE, cert-manager, Vault) do it.
- 🔎 **Checked first:** no machine pins the authority; only the server's own checks trust it. So it can change without touching any machine.
- ⏭️ **Next:** build it in two steps: first let the server hold several authorities (`.4.1.8.1`), then the automatic renewal and the warning (`.4.1.8.2`).

## 2026-09-24 — A machine's certificate can no longer outlive the authority that signed it (`SIGNOFF-REPAIR.4.1.7`)

`REASONBRAID-REPAIR-0491`.

- 🔴 **Before:** the server gave each machine a 10-minute certificate without checking when its own certificate authority expires. In the authority's last minutes, it signed certificates that outlived it.
- ✅ **Now:** a machine's certificate ends no later than the authority does. If the authority has 5 minutes or less left (the point at which a machine asks for a new certificate anyway), the server refuses to issue and says clearly that the authority must be renewed. The server and the machines now read that 5-minute figure from one shared place.
- ✅ Tested: the new test failed on the old code and passes now; all the deliberately broken versions that compile were caught (one only after I added a check of the refusal message); broad live suite passes.

## 2026-09-24 — Machine certificates reviewed: one worry cleared, two real gaps owned (`SIGNOFF-REPAIR.4.1`)

`REASONBRAID-DOC-0158`. A check of three open findings against the code; no code changed.

- ✅ **Cleared:** "a machine with only expired certificates never reads as suspended". Revoking a machine revokes all its certificates, expired ones included, so a revoked machine always reads suspended. One whose certificates simply ran out reads offline, which is the truth.
- 🔴 **Real:** the server's certificate authority is made for one year and nothing renews it. The health check does report it once it has expired, but nothing warns beforehand or stops new certificates being issued from it, so after a year every machine would fail at once (`.4.1.8`).
- 🔴 **Real:** a machine's certificate can be issued to outlive the authority that signed it (`.4.1.7`, next).

## 2026-09-24 — An operator's ruling on a lost answer now settles its budget hold, and the budget review is complete (`SIGNOFF-REPAIR.4.5.1.1`)

`REASONBRAID-REPAIR-0490`. The last item of the budget review (`REASONBRAID-DOC-0156`).

- 🔴 **Before:** since this morning, a hold for a lost answer stayed counted, correctly. But nothing ever let it go: when an operator ruled that the call had not been charged, the money stayed held for good.
- ✅ **Now:** when the machine applies the operator's ruling, the hold for exactly that attempt is released ("it did not charge") or charged in full ("it did, amount unknown"). This is keyed by attempt, so a re-run's separate hold is never confused with the original's.
- ⭐ **The budget review is done:** uncertain holds are counted and then settled, totals cannot overflow, simultaneous requests cannot squeeze past a limit, a machine cannot charge another tenant, and the spending cut-off behaves as documented. The outgoing-event queue waits for its first reader.
- ✅ Tested: the new test failed on the old code and passes now; three deliberately broken versions were caught; broad live suite and strict lint pass.

## 2026-09-24 — A spending cut-off that watches nothing is refused (`SIGNOFF-REPAIR.4.5.6.1`)

`REASONBRAID-REPAIR-0489`.

- 🔴 **Before:** an administrator could arm a spending cut-off that named no measure at all. After this morning's fix it would never trip: a silent alarm.
- ✅ **Now:** that request is refused with a clear message listing the four measures, and the refusal is recorded like any other administrative action. A measure set to zero still counts as named.
- ✅ Tested: the new test failed on the old code and passes now; two deliberately broken versions of the rule were caught; broad live suite and strict lint pass.

## 2026-09-24 — A replaced NUL character can now be traced and undone, by anyone reading the thread (`SIGNOFF-REPAIR.4.4.10.3.1`)

`REASONBRAID-REPAIR-0488`. Revisits an earlier decision at the director's request.

- 🔴 **Before:** our database cannot store the NUL character, so a machine swaps it for "�" (U+FFFD) and noted only how many it swapped. The swap could not be undone: a "�" the AI wrote itself looked the same as a swapped one, and the note never reached the thread people read.
- ✅ **Now:** the machine records exactly where each swap happened. The server checks that record against the text and keeps it on the contribution itself, so any reader can see what changed and restore the original exactly. A "�" the AI wrote itself stays unmarked.
- ✅ Tested: all three new or changed tests failed on the old code and pass now. Ten deliberately broken versions were all caught: two only after I added a test for them, and one breaks the server's hand-off of the positions. The live test rebuilds the original text from the positions and checks it matches exactly. Broad live suite and strict lint pass.
- Technical: node `storable_content` → maximal scalar-index runs `nul_positions` (replacing `nul_replaced`); `threads::check_nul_positions` on contribute/revise; event bodies carry `nul_positions` when present; the fold forwards it. Decision: `docs/decisions/2026-09-24_nul-in-provider-output-is-replaced-losslessly.md` (supersedes the count-only record).

## 2026-09-24 — A spending cut-off set on calls no longer trips on the first job (`SIGNOFF-REPAIR.4.5.6`)

`REASONBRAID-REPAIR-0487`.

- 🔴 **Before:** a tenant's spending cut-off set on only some measures, say 100 calls, tripped on the very first job. Every job also asks for tokens and time, and the check treated "not set" as "no room".
- ✅ **Now:** the cut-off watches only the measures it names: set at 100 calls, it trips at the hundred-and-first. The budget limit itself still refuses anything it doesn't measure, which is right for a limit.
- ⚠️ **Next:** a cut-off that names nothing at all can now never trip, and the arm command still accepts one (`.4.5.6.1`).
- ✅ Tested: the new test failed on the old code and passes now; five deliberately broken versions were all caught.
- Technical: core `BudgetDimensions::restricted_to`; `check_spend_breaker_in_tx` compares `threshold.covers(&projected.restricted_to(&threshold))`.

## 2026-09-24 — The server's outgoing-event queue waits for its first reader (`SIGNOFF-REPAIR.4.5.5`)

`REASONBRAID-DOC-0157`. The fifth item from the budget review; no code changed.

- ✅ **Found:** every change the server records also queues an outgoing event, but nothing reads that queue yet, and the "deliver" step only writes to a table nobody reads. The queue grows at the same rate as the event history, which is kept anyway, and it blocks nothing today.
- ⏸️ **Decided:** the queue's worker, its retry limit and its failed-message shelf get built together with the first thing that actually consumes events (the publication worker, a federation export, a webhook), or as soon as anything deletes old events. Recorded in `docs/decisions/`.

## 2026-09-24 — Proven: a machine cannot charge its work to someone else's budget (`SIGNOFF-REPAIR.4.5.4`)

`REASONBRAID-REPAIR-0486`. The fourth item from the budget review (`REASONBRAID-DOC-0156`).

- ✅ **Held, now proven:** a machine's result names a budget hold, but the server ignores that name and settles the hold it recorded itself. Nothing tested this. A new test has one tenant's machine name another tenant's hold, and requires the other tenant's hold to stay untouched.
- ✅ Tested: the test passes on today's code. When I broke the server so that it trusted the machine's name, this test failed (the other tenant was charged a million tokens) and every other test still passed. So it closes a real gap in our checks.
- Technical: `node_work.rs` `a_node_cannot_settle_a_foreign_reservation_by_naming_it`; hand mutant on `api::settle_work_item_reservation`.

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

The entries before those above were rotated into reachable Git history at the
**thirteenth rotation** (`SIGNOFF-REPAIR.11.4.1.6`, which owns this ledger’s rotation). The exact predecessor — this file as it
stood at the commit named below, which is the object every retired record was
checked against before this notice was written — is:

```bash
git show 6893d285fd6c3dc69b21039d9bdb30c61cb8e31b:LIVE_STATUS.md
```

That snapshot is 53144 bytes and 382 lines, and contains 39 dated
entries; its Git blob is `5f95f10802f8eccb9da19a95d7919ea70781c225` and its SHA-256 is
`cc7887ef2f6d18257f0be90c05494cadffd63261244d145d1a689966a85ddaa0`. It carries the twelfth rotation's
notice in turn, and each earlier notice names the one before it, so the chain
walks all the way back. `docs/decisions/2026-09-09_changelog-rotation.md` holds
the first transition's evidence.

⛔ **13 record(s) rotated out, 27 kept, lossless** — every retired heading was retrieved from the
predecessor named above before this notice was written, and every figure in it was re-derived from that object with
`git rev-parse`, `git cat-file` and SHA-256 rather than typed. ⭐ The cut is DERIVED, not chosen: it retires whole
records until the ledger has at least 10 commits of runway at the p90 entry size measured over the last
60 non-rotation commits — because two rotations that stopped at the threshold instead left 344 and 296 bytes and
the first forced another rotation on the very next commit (`SIGNOFF-REPAIR.11.4.1.6`).
