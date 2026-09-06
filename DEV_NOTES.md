# DEV_NOTES.md

## _(2026-09-06)_ — ReasonBraid-only naming: 90 scaffold-name tokens swept from 28 files

- **Census before reword, always.** (case-insensitive `git grep` census over the scaffold-name token) → 90 occurrences in 28 tracked files: the template's own name had survived the bootstrap in provenance comments (the scaffold tracker ids), the version file, the scaffold-pull tooling, and the landing page. An ordered token map (compounds first, bare tokens last) plus prose polish removed every one; the facts survived (`REASONBRAID-MAINTENANCE.N` ids, `reasonbraid-scaffold 0.6.1` version string). A naked sed for the bare token first would have mangled the compounds and the crate names.
- **A guard's fixture must mirror the docs it guards.** The README-STABILITY self-test exercised the scaffold-URL span; when the README moved to `<reasonbraid-url>`, the fixture moved with it — a self-test asserting a placeholder the landing page no longer uses teaches the wrong lesson.
- `promotion: declined (the directive and its census are recorded in the MAINT-2 leaf; no durable cross-cutting fact beyond the rebrand)`. **The PHASE-0 tree is complete — next executable work: `PHASE-1.1`.**

## _(2026-09-06)_ — README_POLICY re-adoption: the closure leg caught real destinations on its first run

- **The routing-pressure-closure leg reproduced the upstream cautionary tale in miniature.** The moment the guard actually censused the tree it flagged three genuinely unrouted destinations — `COMMIT.md`, `docs/adr/001-uncleared-working-name.md`, and the scaffold-URL placeholder inside the scaffold span — plus a real measured legacy ceiling (`CHANGELOG.md` at 48,495 bytes against a provisional 10,240). A guard that had never been asked the question could never have caught them; the upstream policy's 1,547,057-byte neighboring sink starts exactly this way.
- **Derived caps beat template defaults.** 300 lines / 16,384 bytes was meaningless for a 47-line landing page; 60 / 2,400 is the reviewed survivor plus explicit headroom, and raising it now requires a task-tree decision — the cap became a contract instead of folklore.
- **BSD `cut` on a no-delimiter line prints the WHOLE line (GNU prints empty).** The control-field census initially swallowed the registry's comment lines and flagged `README.md` / `scripts/check_readme_stability.sh` as unrouted destinations. Fix: `grep -v '^#'` before the field cut. Portability lesson for every future bash guard.
- Promoted to `docs/decisions/2026-09-06_readme-policy-readoption.md` (`answers:` present). **Frontier `PHASE-0-MAINT-2` (scaffold-reference cleanup, director directive).**

## _(2026-09-06)_ — Phase 0 exit gate closed: the owner signs the go record, the agent records it

- **The signature closes a gate that only the owner can close.** ADR-002 moved `proposed` → `accepted` on the accountable owner's explicit session decision ("Sign ADR-002 (GO) now"), and the ADR's signature line records WHO signed and WHEN — the agent drafts and records; the signature itself is the owner's act. KICKOFF §7's last item ("a named owner signs a go, rework, pivot, or stop record") is now satisfied, so Phase 0 formally exits and the PHASE-1 tree opens at `.1`.
- **Tree states carry the handoff, not chat.** PHASE-0's frontier became `MAINT-1` (executing next), PHASE-1 flipped `proposed` → `active` with `.1` unblocked, and LIVE_STATUS gained a Phase 1 row — a fresh session reads the same next-action from the durable layers with zero conversation context.
- **A dating anomaly surfaced and was recorded, not rewritten.** The host clock and git commit timestamps say 2026-09-06; the previous session's records carry 2026-09-07 dates (filenames and changelog entries inside 2026-09-06 commits). Today's records use the machine-consistent 2026-09-06; the anomaly is flagged to the director rather than renamed (history is immutable; re-dating committed records is churn with no corrective value).
- `promotion: declined (the acceptance is recorded IN the ADR itself — docs/adr/002-phase1-scope.md status + signature; no separate cross-cutting fact beyond it)`. **Frontier `PHASE-0-MAINT-1` (executing).**

## _(2026-09-07)_ — WP8 gate package: the subtraction record is the architecture ratchet's counterweight

- **The SubtractionRecord forced honest accounting of what Phase 0 did NOT do.** §19.8's shape turns "we didn't get to X" into a decision with a revisit trigger. The deferrals that matter most: the authenticated streaming channel + workload identity (revisit: any non-loopback exposure), the second real adapter (revisit: Phase 1), and the shared wire crate (revisit: a second consumer — the per-side `deny_unknown_fields` duplication is deliberate until then).
- **The 2×-estimate gate is arithmetic, not vibes.** Phase 0 measured ≈ 9.5 engineer-weeks against the roadmap's 8–14 range — no review triggered — but the number is now written down where a future phase can compare against it (§20.1.2: "re-estimate from measured throughput").
- **Two new risk rows came straight out of the `.7` real run** — provider run-to-run variance on identical prompts (code-002 single: 1.000 → 0.667 across runs) and ~16k ambient input tokens per real call. Both were observable only because the harness recorded ACTUAL usage and kept per-case scores; an average-only report would have hidden both.
- **The go decision is drafted, not self-signed.** ADR-002 is `proposed` with the GO recommendation and a pending signature line — the accountable owner (the director) signs; an agent drafting the package must not close its own gate.
- No new code in this leaf (documents + registers only) — the TASK-ACCEPTANCE boxes record that the gate's evidence comes from the WP1–WP7 suites, not from new tooling. **Frontier: exhausted; awaiting the signature.**

## _(2026-09-07)_ — WP7 benchmark: the first real run falsified the harness before any claim could ride on it

- **The scripted oracle CANNOT see prompt-wiring bugs — the real run can.** The critique/revise workflow rendered the SAME template for the critique and the revision leg, so every revision call was instructed to critique: all four real `critique_revise` rows came back with NO confidence line (`structure_valid: false`) and fact-001's "revision" broke a correct answer (1.0 → 0.0). The scripted agent answers by ROLE and never reads the prompt, so the corpus self-test stayed green through the whole defect. Lesson: prompt wiring needs a prompt-level check — the corpus now carries `the_critique_and_revision_prompts_are_distinct` (distinct templates, role-naming instructions), and the workflow renders `critique`/`revision` separately.
- **A trap that flags the question's own echo is a false positive machine.** The honesty trap (any digit in the answer) flagged a refusal that merely quoted "2026" back from the statement. Now it flags only numbers NOT present in the statement — still deterministic, no longer self-defeating.
- **Real cost accounting surprised us in a good way to have measured**: each `codex exec` call carried ~16k input tokens of ambient overhead (the user's Codex config), independent of the benchmark's ~100-character prompts — the H6 accounting would have been fantasy without recording ACTUAL usage. The harness records provider-reported tokens, so the overhead is visible instead of assumed away.
- **The benchmark's own verdict on itself was negative-or-null on this sample** (single agent matched or beat the structured workflows on the four differential cases at 1× the calls) — that is the WP7 acceptance's point: it narrows the routing claim for the WP8 memo rather than decorating it. See `docs/evidence/2026-09-07_benchmark-codex-run.md`.
- Promoted to `docs/decisions/2026-09-07_deliberation-benchmark.md` (`answers:` present). **Frontier `.8`.**

## _(2026-09-07)_ — WP6 node wiring: three bugs the demo and the suite caught before they shipped

- **The live suite caught a domain-semantics inversion in the first dispatch draft.** The revise work item originally carried the CHALLENGED CONTRIBUTION's event id as its target; the domain's `thread.revise` targets a CHALLENGE. The test failed with the server's own `invalid_command: revision target … is a contribution, not a challenge` — the contribution id is only the author-lookup key, the challenge's own event id is the revise target. Fixed; the assertion now checks the revise work item targets the challenge event.
- **`$$` inside a `( … )` subshell is the SCRIPT's pid, not the subshell's.** The demo's first pidfile scheme recorded the script's own pid — `node_kill` SIGKILLed the demo itself, the cleanup trap died before killing the server, and a later run hit `AddrInUse` with five leaked processes. Now: local nodes capture `$!` of the directly backgrounded binary; remote nodes capture the remote `$!` via `nohup … & echo \$!`. Every kill is followed by a `wait` reap (also silences bash's `Killed: 9` job banners).
- **`wait_for` under `set -e` is a footgun.** A probe timeout returning 1 aborted the script before the FAIL summary could print. Timeouts now record the FAIL and return 0 — the summary exit status decides.
- **A test-harness purge race, same class as `command_api`'s correct pattern.** The first `node_work` run failed `active == 1` because `pool()` purged the shared tables BEFORE the suite mutex was acquired, so a parallel test in the same binary purged rows mid-test. Guard first, purge second — matching `command_api`, which already had it right.
- **JSON shape assumptions bite in demo scripts.** `rb inspect thread --json` nests the events list one level deep (`.events.events[]`, the wrapper of three API views), and `--json` is PRETTY-printed — raw `"state":"closed"` greps fail; the checks now allow optional whitespace (`grep -Eq`). `jq` became the extraction tool of record (documented dependency of the demo).
- **Two independent dedupe layers, both exercised.** The duplicate-transport leg proves the `node_events` receipt dedupe (`accepted:false`) AND the idempotency-claim replay (same work result under a NEW event id still yields exactly one contribution) — the demo re-POSTs the node's ORIGINAL submission reconstructed from `rb-journal events`.
- Promoted to `docs/decisions/2026-09-07_node-channel-wiring.md` (`answers:` present). **Frontier `.7`.**

## _(2026-09-06)_ — WP6 control API: the subset checker caught the bootstrap bug, and rejections became idempotent results

- **The `.5.1` temporal subset rule caught THIS leaf before it shipped.** The first live run of the enroll bootstrap failed: a role grant created microseconds after its boundary "outlived" it (`grant.expires_at > boundary.expires_at`), the same wall-clock-skew class the `.5.1` fixtures exposed. Fix: dev grants are COEXTENSIVE with their boundary's validity window (`valid_from`/`expires_at` copied from the boundary) — and the failure itself is the evidence the checker binds.
- **Rejections are the command's semantic result, stored for replay.** A denied or domain-refused command stores `{"ok": false, "error": {code, message}}` in the idempotency row, and a replay reproduces the ORIGINAL status (stable code→status map) and body. This required the `tx` split — claim FIRST, then authorize/validate/apply — because a replay must return the original result WITHOUT re-validating against state the original command may have since changed (a replayed contribution after close must not fail).
- **The `FOR UPDATE` read is the consistency trick.** The domain validation reads the projection with `FOR UPDATE`; `apply_fresh_in_tx` re-reads the SAME row in the SAME transaction — so the version derived for the write can never diverge from the state validated. No check-then-write race, no second locking scheme.
- **The e2e run caught a classic URL bug the unit layer could not.** The CLI's inspect joined `/events` AFTER the query string (`?tenant_id=…/events`), corrupting the tenant param — the real-binary suite failed loudly with the server's own `invalid_command`. Lesson: the e2e suite earns its place by exercising the actual bytes the binary sends.
- **Clippy's `too_many_arguments` struck the verb runner (8/7)** — grouped into `ThreadVerbArgs`, the same class as `.5.1`'s `policy_digest` fix. And `clone_on_copy` hit `BudgetDimensions` (it derives Copy) — removed the clone, kept the one `String` clone the projection needs.
- **Axum's Json extractor answers forged fields with 422**, not the handler's 400 — the `.3.2` channel convention; the test asserts the 422 + the serde rejection naming the field.
- Promoted to `docs/decisions/2026-09-06_control-api-cli.md` (`answers:` present). **Frontier `.6.2`.**

## _(2026-09-06)_ — WP5 budget: one invariant, two ledgers, and a mandatory parameter that audits its own refusals

- **The acceptance is one sentence enforced twice:** "no provider dispatch without an applicable reservation" — the SERVER refuses to issue what the ceiling cannot cover (with a denial ROW), and the NODE refuses to dispatch what it has not been issued (journaled `failed_before_dispatch`, adapter never invoked — proven with a counting adapter). Two ledgers, one invariant (§14.3 step 4 is a LOCAL check by design).
- **Fail-closed coverage caught its own doc lie.** The first `covers` shipped with a doc comment claiming untracked dimensions "impose no constraint" while the code denied them; the tests exposed the contradiction and fail-closed was pinned. A ceiling that does not meter a dimension cannot vouch for it — period.
- **Refusals are results, not errors.** A refused dispatch returns a `FailedBeforeDispatch` report with the reason journaled as evidence — the attempt trail is complete for what did NOT happen. This matches `.5.1`'s denial-row philosophy (the audit covers refusals).
- **Indeterminate attempts keep their hold** (§14.6: release only amounts not potentially consumed). This cost the supervisor a deliberate asymmetry: pre-dispatch refusals release, completions settle actual usage, ambiguity holds — and the hold is the signal that adjudication is still owed.
- **Patch surgery on tests is a smell.** Mass-editing call sites with regex + helper insertion produced THREE distinct mangling rounds (nested helpers, dropped parens, misattached `#[tokio::test]`). The lesson: when a signature change touches many call sites, edit the files directly and compile after each file — not regex-batch then fix-forward.
- Promoted to `docs/decisions/2026-09-06_budget-reservation.md` (`answers:` present). **WP5 complete; frontier `.6.1`.**

## _(2026-09-06)_ — WP5 authority: the subset checker was more precise than the fixtures, and that is the point

- **The temporal subset rule caught the fixtures before they caught it.** The first live run failed 6/9: every grant "outlived its boundary" because each fixture helper read its own `Utc::now()` — a grant built microseconds after its boundary exceeded the window by those microseconds. A wall-clock-skew bug class that a weaker checker would have shipped silently; the fixtures now use wide boundary windows, and the failure itself is the evidence the rule binds.
- **Serde's tagged enums cannot wrap a sequence in a newtype variant** — `TargetSelector::Threads(Vec<ThreadId>)` cannot serialize (`cannot serialize tagged newtype variant containing a sequence`). Struct-like variants (`Threads { threads }`) fix it. A rule to internalize: any tagged enum variant holding a Vec must be struct-like.
- **`should_implement_trait` earned its keep again** — four authority `from_str` helpers became real `FromStr` impls with a shared `UnknownAuthorityName` error (the same lint that shaped `ProviderAttemptState` in `.3.1`); and `policy_digest` went from 8 params to 6 by passing the boundary struct (clippy's `too_many_arguments`).
- **The sqlx executor-shape split is real:** `&PgPool` and `&mut Transaction` satisfy `Executor` differently, so a shared loader abstraction fights the type system. The pragmatic shape: pool-based loaders for the public paths, INLINED lookups in the transactional path, and `apply_command_in_tx` as a generic `E: DerefMut + for<'c> &'c mut E::Target: Executor<'c>` (the `.2.1` body extracted with its public signature untouched — its 5 tests stayed green through the refactor).
- **Denials are audited events.** The acceptance reads "every command records … decision" — a refused command commits its denial record (reason + digest) and applies NOTHING; the audit trail is complete for what did NOT happen, not just what did.
- Promoted to `docs/decisions/2026-09-06_authority-boundary.md` (`answers:` present). **Frontier `.5.2`.**

## _(2026-09-06)_ — WP4 first real harness: the boundary that REVEALS its handle in the stream, and the lookup that honestly does not exist

- **The acceptance's honest leg was designed to be exercised by a REAL adapter — and Codex exercised it.** `codex exec` has no first-class status query for a past attempt (`exec resume` CONTINUES a thread and bills again; it is not a lookup), so `query_status` is `Unsupported`, a lost response lands `outcome_unknown` with no retry language, and the streamed thread id stays attached as the proof handle an operator would adjudicate with. No capability was fabricated to make the demo prettier.
- **Providers reveal request handles at different times.** The contract's `DispatchAck` carried the handle "when known"; Codex reveals its thread id in the stream's FIRST event, after dispatch. The contract gained `AttemptEvent::ProviderRequestId`, and the supervisor attaches streamed handles exactly like ack-carried ones. The ack ≠ completion acceptance now has its sharpest proof: the ack carries NOTHING, the handle arrives later, and the result later still.
- **The stub boundary caught a mis-wiring exactly as it should.** The first stub branched on `$1` — which is `exec`, not the prompt — so every scenario misbehaved and the suite failed loudly. The lesson is the same as the PG-queue lesson: test doubles must re-derive their inputs the way the REAL boundary receives them (here: the prompt is the LAST argv of `codex exec …`).
- **Live dispatch is one gated command away, never accidental:** `RB_LIVE_CODEX=1 cargo test … -- --ignored`. Default CI never spends a token; the ledger's revalidation trigger (CLI release) and the release gate both re-run it deliberately.
- **The vendor boundary stayed vendor-free:** no Codex DTO entered core; the only core-touching change across `.4.1`+`.4.2` is the one proof-gated machine edge from `.4.1`. Credentials: none — the adapter has no credential field; Codex uses its ambient login.
- Promoted to `docs/decisions/2026-09-06_real-adapter-codex.md` (`answers:` present) + `docs/evidence/2026-09-06_codex-adapter-qualification.md`. **WP4 complete; frontier `.5.1`.**

## _(2026-09-06)_ — WP4 adapter boundary: the conformance corpus caught the boundary-vs-refusal conflict, and two probes caught the rest

- **The corpus earned its keep on the FIRST replay.** `fail_before_dispatch` failed the moment it met the supervisor: the `.3.1` rule journals `dispatched` BEFORE `invoke` (conservative, crash-safe), but the machine had no edge to record the adapter's certified "no dispatch ever began". The fix is a proof-gated correction edge — `(dispatched, fail_before_dispatch) → failed_before_dispatch` — the exact inverse of the §11.3 lookup-proof edges, and the `.3.1` record's philosophy holds: only PROOFS move the machine, never guesses.
- **Two more real bugs, probed not guessed** (the ack test hung twice, with different causes): (1) `Notify::notify_waiters` loses a wake if the waiter has not registered yet — a scheduling race invisible without stress; `notify_one` stores a permit and is the correct primitive for one-shot signals (used in the fake's hang-cancel AND the test double). (2) The supervisor looped past terminal events — a stream yielding `Completed` repeatedly spun forever; the loop now breaks on the FIRST terminal event. A stream is not a source of multiple results.
- **The indeterminate outcome is an error, not a success.** `execute_attempt` returns `Err(OutcomeUnknown)` with the attempt journaled `outcome_unknown` — and its Display deliberately contains no "retry" (a test asserts the absence). Retrying ambiguity needs duplicate-risk authorization (§14.6); the boundary never volunteers advice.
- **The ack ≠ completion acceptance is proven AT the journal boundary**, not by assertion: a signaling test adapter pauses between the dispatch ack and the result, and the test observes the attempt durably `dispatched` in the journal in that window.
- **Credentials are enforced mechanically, not by convention:** the corpus's credential-shape scan (api_key/secret/password/credential/bearer) is a red test — a fixture with a credential fails CI. The contract has no credential field at all.
- Rejected and recorded: `async-trait` (native `async fn` in traits + documented `#[allow(async_fn_in_trait)]`), sleeps for the hang (Notify instead), parsing provider output in the adapter (chunks are opaque), a `cancelled_known` state (a confirmed cancel still leaves the result unknowable — honest `outcome_unknown`), and any blanket retry helper.
- Promoted to `docs/decisions/2026-09-06_fake-adapter.md` (`answers:` present). **Frontier `.4.2`.**

## _(2026-09-06)_ — WP3 node channel: the node reports what it durably holds; the server replays the tail; reconciliation gates schedulability

- §17.4 steps 1–7 became a protocol: the node reports its resume facts (last acked cursor, pending operation ids, ambiguous attempts), the server replays `cursor > reported` plus two directive kinds (`adjudicated` when it holds a receipt for the operation's event, `needs_adjudication` otherwise), and the node applies everything before becoming schedulable. The load-bearing rules that make it sound:
- **The node is authoritative for what it durably holds.** Replay is computed from the node's report, never from the server's ack bookkeeping (which exists for operations, not replay). A node reporting a cursor AHEAD of the server's ledger is refused with `version_conflict` — a journal-lost-class anomaly must stop the world, not re-base it.
- **The pending-operation exchange must DO something or it is ceremony.** First cut: `known_events` was redundant — the node re-emitted every pending event anyway, so the server's receipt report changed nothing. Fixed: a known event is marked acknowledged locally and NOT re-sent; only events the server never got travel the wire. The exchange became load-bearing, and the test proves it (1 receipt for the known event, 1 for the unknown one).
- **Schedulability is a state machine, not a flag someone remembers to set.** `Offline → Reconciling → Schedulable`, the terminal step written ONLY after the handshake is fully applied; `emit_event` refuses before it; ANY reconcile failure returns to `Offline`. Ambiguity does NOT block schedulability — a `needs_adjudication` attempt stays bounded and visible while the node works (the exit gate wants "ambiguous outcomes visible and bounded", not "everything terminal").
- **The first live run failed exactly one test — a fixture bug, not a protocol bug:** the test emitted for an operation id the journal had never created, and the FK correctly refused it. The protocol passed 11/11 on its first live run; the fix was in the test (use the replay-created operation id), recorded here for the pattern: seed through the real path, not parallel ids.
- Transport is HTTP/1 JSON (axum + reqwest) over loopback, unauthenticated: real sockets for the experiment, the planned §9.3 stack for later phases, and a loud "dev-only until WP5" boundary. A bespoke TCP protocol was rejected as throwaway code; SSE push, node-command leases, and the shared wire crate are deferred with owners (ADR-006/WP8, WP5, `reasonbraid-protocol`).
- Promoted to `docs/decisions/2026-09-06_node-channel.md` (`answers:` present). **WP3 complete; frontier `.4.1`.**

## _(2026-09-06)_ — WP3 node journal: the boundary record precedes the dispatch; ambiguity is recovered, never guessed

- KICKOFF WP3 / §11.4 / §17.4 require the node to persist a fact before advancing the corresponding boundary — and the dispatch is THE boundary that makes or breaks honest recovery. The fix: `record_dispatch` commits `prepared → dispatched` (with the provider request id when known) BEFORE the adapter is invoked, and a test proves a SECOND connection already sees the boundary record before the adapter would run.
- Recovery then has exactly two honest answers: `prepared` → `safe_to_redeliver` (the boundary was never crossed — claiming ambiguity would forbid a safe redelivery AND poison the operator's ambiguity signal), `dispatched` → `outcome_unknown` (the node may have dispatched; it cannot know). `prove_result` is the only exit carrying a result (adapter status lookup), `reconcile` the authorized adjudication — the §11.3 provider-lookup edges now exist in the core machine.
- **The core machine gained `failed_known` + `(dispatched, fail_known)` + `(outcome_unknown, complete|fail_known)`.** This supersedes the `.1.3` note that `failed_known` is out of Phase 0: the exclusion was about GUESSING, and a proven failure is not a guess. `cancelled_known` stays out. (Kill-risk Q4's honest minimal answer is preserved — indeterminate stays `outcome_unknown` until proof or adjudication.)
- **Durability is recorded, not just set.** `synchronous=FULL` is a per-connection pragma: any OTHER connection reading the file sees its own default, so a health view that reads pragmas would report a lie. The journal verifies the profile on its live connection at open AND writes it into `journal_meta`, which is what the read-only CLI reports. WAL alone is not a power-loss guarantee (§11.4); FULL is the conservative dev default.
- **Kill-point coverage as a table.** KP-1…KP-9 walk every `.3.1` seam (before/after command record, operation, prepare, dispatch, result, ambiguity, event emission, ack) by dropping the journal handle mid-flight with no checkpoint — each transition is its own transaction, so what survives IS what a killed process leaves. No sleeps, no mocks: the caller-supplied clock is stored as RFC 3339 TEXT.
- **The CLI is read-only by construction** (`SQLITE_OPEN_READONLY`) and proven non-mutating by byte-comparing the journal file after every inspection; WAL mode lets it run beside a live node (tested with the writer handle held open).
- Driver choice: sqlx SQLite over rusqlite — one driver stack with the server's Postgres side, `sqlx::migrate!` already proven in this repo, async-native for the future Tokio node. The first path dependency in the repo (`reasonbraid-core`) had to be version-pinned (`version = "0.1.0"`) or cargo-deny's wildcard ban rejects it.
- Promoted to `docs/decisions/2026-09-06_node-journal.md` (`answers:` present). **Frontier `.3.2`.**

## _(2026-09-06)_ — WP2 leased outbox worker: three commit points, per-claim fencing tokens, and a queue that owns its tests

- `ROADMAP.md` §17.3 / KICKOFF WP2 require leased claims plus "fencing tokens prevent a stale worker from committing after a newer lease," but `.2.1` left the outbox write-only. The fix is a three-phase loop where **each phase is its own commit point** — `claim_ready` (atomic `UPDATE … FOR UPDATE SKIP LOCKED` issuing a fresh `gen_random_uuid()` token + expiry + attempt++), `deliver` (dedupe sink keyed on `event_id`), `complete` (`WHERE lease_token = current AND lease_until > now`; otherwise `LeaseLost`). KICKOFF's kill points 3–5 are precisely the seams between those commits.
- **Fencing has two independent legs, both load-bearing:** a superseded claim fails the token check; an expired lease fails the liveness check *even with a matching token* (it must re-claim first). An attempt-CAS alone (the cheaper rejected design) cannot refuse the second case.
- **The clock is caller-supplied** (`chrono` → `TIMESTAMPTZ` via sqlx's `chrono` feature): tests advance past a lease expiry by passing `now + 61s`, never by sleeping or faking the DB clock.
- **The first live run failed 7/7 and the failure taught the real lesson** (TOOLBOX: probed, not guessed): the outbox is ONE shared queue, and my tests ran in parallel — each test's global oldest-first claim stole rows other tests (and the `atomic_transaction` binary) had seeded, and even single-threaded runs leaked leased-but-incomplete rows into later tests once their leases lapsed. The suite now serializes under a module-level async mutex, purges the queue under the guard, and cleans its own item at the end. A shared queue demands exclusive ownership from its tests; the worker API itself was correct throughout.
- Rejected designs recorded in the decision record: dispatching inside the claim (collapses kill points), a worker-global epoch table (per-claim token suffices; epoch earns its keep only for Phase 2 all-items quarantine), DB-clock expiry, and a `next_eligible_at` backoff column (nothing fails delivery yet — Phase 2 retry policy will add it).
- Promoted to `docs/decisions/2026-09-06_outbox-worker-fencing.md` (`answers:` present). **WP2 complete; frontier `.3.1`.**

## _(2026-09-06)_ — WP2 atomic transaction: claim-first idempotency, four tables in one commit

- `ROADMAP.md` §8.6 / KICKOFF WP2 require "one transaction writes current state, ordered event, idempotency result, and outbox item," but nothing enforced it — four autocommit `INSERT`s could tear, and a "check-then-insert" idempotency check races under redelivery. The fix is structural: `apply_command` claims the `(tenant_id, idempotency_key)` primary key **first** (`INSERT … ON CONFLICT DO NOTHING`), so the unique index is the concurrency control — a redelivered message either replays (same hash → original stored result) or conflicts (different hash).
- The four writes (idempotency claim, `event_log`, `aggregate_state`, `outbox`) run on one transaction; a failure at any step drops it and rolls back the claim too. The outbox FK → `event_log` makes "outbox row implies durable event" a schema fact, not an assertion.
- **The proof needs a live Postgres** — the tests skip when `DATABASE_URL` is unset (so `make check` stays green offline) and run for real only in `scripts/run_pg_tests.sh` (ephemeral `initdb`/`pg_ctl`, no background service) and the `pg-tests` CI job. "successful response ⇔ committed durable state" is asserted by reading all four tables back from a *separate* connection after commit.
- **Honest limit:** `next_version = MAX+1` under `FOR UPDATE` serializes writers to an *existing* aggregate, but a fresh aggregate's first insert isn't gap-locked — two concurrent first-writes to the same new aggregate aren't fully serialized. Phase-1 concern, out of WP2's single-writer scope.
- **`deny.toml` was wrong for the tool it names.** It was authored against an old cargo-deny schema (when deps were zero) and only TOML-parsed, never run through cargo-deny — so the first real `make deny` failed. Corrected for cargo-deny 0.20: `[advisories].unmaintained` is a *scope* (`all`/`workspace`/`transitive`/`none`), not a lint level (`deny`); added `BSD-3-Clause` for `subtle` (constant-time crypto, via sqlx SCRAM); `skip` for the reviewed `getrandom`/`hashbrown`/`syn` sqlx-tree duplicates. Lesson: a config for a tool that isn't installed is a *draft*, not a gate.
- Promoted to `docs/decisions/2026-09-06_atomic-transaction.md` (`answers:` present).

## _(2026-09-06)_ — WP1 typed errors + reason-code registry: complete §9.8, unknown codes preserved

- `ROADMAP.md` §9.8 lists reason codes but not how to treat an unknown one. The two naive shapes both fail: a closed enum *rejects* the future (deserialization error), a bare `String` *loses* the typing of the known set. The fix is a two-layer `ReasonCode` — `Known(KnownReasonCode)` + `Unknown(String)` with `#[serde(untagged)]` — which gets both properties at once.
- `KnownReasonCode` is the *complete* 20-code §9.8 registry (not a demo subset): a "stable registry" re-carved every leaf isn't stable, and client/server must be able to name any §9.8 code consistently. Forward-compat is `Unknown`'s job, not a reason to trim the list.
- `DomainError` carries code + tri-state `Retryability` (`no`/`yes`/`requires_authorization`) + safe `message` + optional `correlation_id` + filtered `details`; secrets/policy internals/cross-tenant existence stay off the type.
- `From<TransitionError> for DomainError` maps `.1.3`'s deterministic rejection to `invalid_transition`, proving the registry classifies real errors rather than sitting unused.
- Acceptance tests: every known code round-trips to its snake_case name; `future_semantic_reason` deserializes to `Unknown` and re-serializes verbatim; a near-miss (`invalid_transition_typo`) is preserved, not misclassified.
- Promoted to `docs/decisions/2026-09-06_reason-codes.md` (`answers:` present). **WP1 complete.**

## _(2026-09-06)_ — WP1 state machines: minimal lifecycles, deterministic fallible `apply`

- `ROADMAP.md` §8.4 lists lifecycle *states* but not *edges*; §8.6 requires "a deterministic aggregate may accept and translate to an event." The gap is closed with three minimal state enums whose only operation is `apply(transition) -> Result<state, TransitionError>` — total, deterministic, fallible, no panics, no history rewinds.
- Chosen edges: thread `open → closing → closed` (two-step close, not a direct `open → closed`) plus `open/closing → cancelled`; participation `invited → {accepted, declined, expired}` and `accepted → left`; provider-attempt `prepared → {dispatched, failed_before_dispatch}`, `dispatched → {completed, outcome_unknown}`, `outcome_unknown → reconciled`.
- A *proven* post-dispatch failure (`failed_known`/`cancelled_known`) is deliberately out of Phase 0 scope — the honest minimal answer to an indeterminate attempt is `outcome_unknown → reconciled` (kill-risk Q4), not a guessed failure.
- Exhaustive tests assert BOTH that every listed edge resolves to its target AND that every unlisted (state, transition) pair is rejected — rejection is a property of the table, not a side effect. State enums serialize `snake_case`; transition enums are transient (the wire catalogue is backlog 6).
- Added the deferred `ProviderAttemptId` (`patt`) to complete the WP1 distinct-types acceptance.
- Promoted to `docs/decisions/2026-09-06_state-transitions.md` (`answers:` present).

## _(2026-09-06)_ — WP1 envelopes: intent in, authority out, forgery rejected

- `ROADMAP.md` §9.1 sketches the command/event split but nothing enforced it — serde ignores unknown fields by default, so a struct that merely *omits* authoritative fields would still accept them from a client. The fix is mechanical: `#[serde(deny_unknown_fields)]` on `CommandEnvelope`, `ClientContext`, and `CommittedEvent` makes the same deserialization that accepts a valid command reject a forged one.
- `CommandEnvelope` = intent only (operation, `request_id`, idempotency key, optional expected aggregate version, opaque `body`, correlation/causation context); `CommittedEvent` = server-assigned authority (event id, tenant, aggregate, sequence, actor principal, timestamps, authorization record, schema version). Optional fields are nullable (`null` on the wire, `#[serde(default)]` on read) to match §9.1's explicit nulls.
- Golden fixtures (`fixtures/`) cover every wire payload the demo uses: `command-thread-create.json`, `event-thread-created.json`, and a `command-with-authoritative-fields.json` that must fail. `schemars` (derive + a manual `Id<K>` impl) generates JSON Schema goldens (`schema/`) guarded by a drift test; regenerate with `cargo test -p reasonbraid-core -- --ignored write_schema_goldens`.
- Timestamps stay `String` (RFC 3339) until ADR-010 pins the time type; `ActorPrincipalId` (`agt`) is opaque — which principal kind it names is a WP5 concern.
- Promoted to `docs/decisions/2026-09-06_envelope-representation.md` (`answers:` present).

## _(2026-09-06)_ — WP1 strong IDs: branded newtypes over UUIDv7, prefix-checked on the wire

- Landed `crates/reasonbraid-core` (first real crate; the scaffold's placeholder `crates/app` binary is removed). `KICKOFF.md` §3 names this crate "IDs, envelopes, minimal thread and attempt states".
- ID representation: a generic `Id<K>` newtype over `uuid::Uuid` (v7) branded by a zero-sized marker `K`; eight families (tenant, human principal, host, node, agent role, agent incarnation, run, thread), each a distinct three-letter wire prefix validated on deserialization.
- The non-interchangeability guarantee is tested two ways: `TypeId::of::<X>()` pairwise-distinct (compile-time newtypes, not aliases) and serde round-trip with wrong-prefix rejection (wire-level non-confusability).
- Dependencies `serde` + `uuid` (dev `serde_json`) are all permissive-licensed; licenses checked against `deny.toml`'s allow list by hand (cargo-deny itself runs in CI, not installed locally).
- Promoted to `docs/decisions/2026-09-06_id-representation.md` (`answers:` present).

## _(2026-09-06)_ — G0 contract drafts: ID-scheme gap closed, drafts live in `spec/`

- `ROADMAP.md` §19.1 names six illustrative requirement-ID prefixes (ID / AUTH / DELIV / RES / POL / SEC) but §20.2 gates G0 on five boundaries — identity, authority, **thread**, delivery, **budget**. Thread and budget had no prefix, so the "stable IDs" acceptance could not be met without a choice.
- Decided: add `THREAD-*` and `BUDGET-*` (first-class now); reserve `RES-*` (Phase 4), `POL-*` (Phase 6), `SEC-*` (Phase 7). Gap-fill for backlog 4, not a feature — the frozen roadmap is untouched.
- Contract drafts live under `spec/` (beside the code, §7.1/§19.1), not `docs/`; each is headed "draft — not normative".
- Promoted to `docs/decisions/2026-09-06_g0-contract-id-scheme.md` (`answers:` present).

## _(2026-09-06)_ — supply-chain skeleton

- Added `deny.toml` (cargo-deny: advisories/bans/licenses/sources), `.github/workflows/supply-chain.yml` (cargo-deny + gitleaks secret scan), and `docs/ci.md`; the Makefile gained `make deny` / `make secret-scan`.
- The `deny.toml` schema was copied from the authoritative cargo-deny `main` template (EmbarkStudios repo), not reconstructed from memory: current shape is `[graph]`/`[advisories]`/`[bans]`/`[bans.std-replacements]`/`[sources]`/`[licenses]`, with no `version` key.
- `cargo-deny` and `gitleaks` are NOT installed locally; the Makefile targets forward to them and CI installs them. Local runs need `cargo install cargo-deny` / `brew install gitleaks`.
- Validated: `deny.toml` parses (python3 `tomllib`), `make -n deny` → `cargo deny check`, `make -n secret-scan` → `gitleaks detect --source . --redact`, `make gate` 13/13, `make check` 1 test ok.

## _(2026-09-06)_ — external dependency ledger skeleton

- Created `docs/dependencies/external-ledger.yaml` from `ROADMAP.md` §7.4: one entry per protocol/SDK/CLI/provider/harness, `checked_at` dated, a `revalidation_trigger` per row.
- Stubbed MCP, A2A, Codex, and Claude rows from the 2026-09-04 corrected baseline (§28.1). `license` is `"unverified"` until a spike records it from package metadata — never asserted from memory.
- Validated with `ruby -ryaml` (4 entries, required fields present) so the file parses clean before it is committed.

## _(2026-09-05)_ — KICKOFF.md is a companion, not a second roadmap

- Director dropped both `ROADMAP.md` (v0.4.1 master) and `KICKOFF.md` (Phase 0 execution).
- They are one pair: the master is frozen scope/gates; the kickoff is the Phase 0 task board.
- Promoted to `docs/decisions/2026-09-05_kickoff-companion-to-roadmap.md` (`answers:` present).

## _(2026-09-04)_ — a template's trial must include the first commit

- Every gate was green on the generated project and the first commit still failed: the doctrines judge STAGED
  code, and nothing had been staged until the user tried. Trial the path a user walks, to its end.
- `grep -c` prints `0` and exits 1. `$(grep -c … || echo 0)` therefore yields `0⏎0` — a second line — which
  here started a flush-left line inside a checklist bullet and hid its evidence from the box-scoped extractor.
  Capture the count, then default the empty case; never append a fallback to grep's own output.

## _(2026-09-04)_ — a green gate that judges nothing is the class a template must not ship

- Two of the four doctrine ports in `.2.6` were wrong on first run and their own RED self-test arms said so:
  a `python3 - <<'PY'` detector whose stdin was the heredoc (every arm read 0 rows), and a `grep -c … | grep -qx 0`
  control under `pipefail` (`grep -c` prints 0 and exits 1). A self-test with only GREEN arms would have passed both.
- The neutrality bar is measured, not felt: `grep -ciE 'grammar|parser|…'` over each ported script → 0, after the
  generic uses of "corpus" and "grammar" were re-worded ("tree", "syntax") so the count means what it says.

Detailed technical notes — root cause, implementation, validation — per slice. The
engineering-continuity surface (not the public docs; that's `docs/book/`). Newest first.

## _(YYYY-MM-DD)_ — bootstrap

Repo created from the ReasonBraid spine template: durable 4-layer memory, task-tree tracking, the
strict commit workflow, and the mechanical doctrine enforcer are in place and enforced by
git hooks + CI. No project code yet.
