# The WP4 adapter boundary: a narrow contract, a deterministic fake, and ambiguity that stays honest

- **Type:** `decision`
- **Date:** `2026-09-06`
- **Status:** `active`
- **Owner / source:** engineering decision during leaf `PHASE-0.4.1` (WP4 deterministic fake harness adapter + ambiguity fixtures)
answers: how does the adapter boundary let the node supervise any harness — stream, fail, hang, ignore cancellation, lose a response after dispatch — while keeping dispatch acknowledgement distinct from completion, making an unsupported status lookup an honest `outcome_unknown` (never a retry recommendation), and keeping vendor DTOs and credentials out of the core and the fixtures?

## The fact / decision

`crates/reasonbraid-adapter` lands the WP4 boundary (`KICKOFF.md` §3):

- **The contract** (`src/contract.rs`, `ROADMAP.md` §11.2): `Adapter` exposes
  `capabilities()` (streaming, cancellation strength, provider idempotency, status
  lookup, tool support, policy-injection mode — callers branch on capabilities, never
  on provider names), `invoke(request, operation_id)` →
  `FailedBeforeDispatch | Accepted(DispatchAck, AttemptHandle)`, `cancel`,
  `query_status` → `Unsupported | Supported(result)`, and `normalize_usage` (unknown
  dimensions stay `None`, never zero). **Dispatch acknowledgement is distinct from
  completion** by construction: the ack carries the provider request handle; the
  result arrives later as stream events (`output chunks → completed | failed_known`)
  or never. **No credential field exists in the contract** — adapters resolve their
  own credentials out of band (`§16.5`).
- **The fake** (`src/fake.rs`, `§11.6` conformance oracle): a per-operation
  `ScriptStep` script (`emit_chunk`, `malformed_output`, `complete`, `fail_known`,
  `fail_before_dispatch`, `hang_forever`, `ignore_cancellation`, `lose_response`) with
  NO sleeps — the hang is a cancellation `Notify`. The same script yields the same
  event sequence every time.
- **The corpus** (`fixtures/*.json`): ten sanitized outcome fixtures, mechanically
  credential-scanned and coverage-checked (every step and outcome class must appear).
- **The supervisor** (`crates/reasonbraid-node/src/supervisor.rs`):
  `execute_attempt` journals `prepared` → records the dispatch boundary → invokes →
  lands the attempt on `failed_before_dispatch | completed | failed_known |
  outcome_unknown`. A stream that ends without a terminal event is a lost response:
  the attempt is journaled `outcome_unknown` and ONLY a proven status lookup moves it;
  `Unsupported` surfaces `SupervisorError::OutcomeUnknown` — a fact, not a retry
  recommendation (retrying `outcome_unknown` requires duplicate-risk authorization,
  `§14.6` — a policy decision, never made here).
- **One machine extension** (`reasonbraid-core`): the edge
  `(dispatched, fail_before_dispatch) → failed_before_dispatch`. The `.3.1` rule
  records the dispatch boundary BEFORE invoking (the conservative, crash-safe order),
  so `dispatched` means "the provider MAY have been contacted". When the adapter then
  CERTIFIES that no dispatch ever began (its deterministic pre-dispatch refusal), the
  correction is a proven fact — the inverse of the §11.3 proof edges. This extends,
  not contradicts, the `.3.1` machine record.

## Why

KICKOFF WP4 acceptance: the fake must "stream, fail, hang, report usage, ignore
cancellation, and simulate a lost response after dispatch"; "dispatch acknowledgement
is distinct from completion"; "unsupported status lookup produces `outcome_unknown`,
not a retry recommendation"; "sanitized deterministic fixtures cover every adapter
outcome"; "vendor-specific DTOs do not enter `reasonbraid-core`". Kill-risk question 2
("can a real coding-agent harness be supervised through a narrow adapter without
contaminating the core domain model?") is the experiment this boundary runs.

## Measured behavior (legs: re-derive · falsify · durable)

- Re-derived: `cargo test -p reasonbraid-adapter` → `test result: ok. 12 passed`
  (fake behaviors) + `test result: ok. 3 passed` (corpus integrity: every step/outcome
  covered, credential scan, well-formedness); `cargo test -p reasonbraid-node` →
  `test result: ok. 17 passed` (journal) + `test result: ok. 8 passed`
  (`supervisor_fake`): the corpus drives every outcome to its expected journal
  terminal; a lost response without lookup is `outcome_unknown` with an error text
  containing no "retry"; a proven lookup lands `completed` with the provider handle
  attached; the hang resolves exactly on a confirmed cancel; the ignored cancel still
  completes; malformed chunks pass through verbatim; usage normalizes with honest
  confidence; **dispatch ack ≠ completion proven at the journal boundary** (a
  signaling adapter shows the attempt durably `dispatched` between ack and result).
- Falsified (the conformance corpus did its job): the FIRST corpus replay found a REAL
  design conflict — the supervisor journaled `dispatched` before `invoke` (the `.3.1`
  boundary rule) but the machine had no way to record the adapter's certified
  no-dispatch refusal, so `fail_before_dispatch` was rejected. Fixed with the
  proof-gated correction edge above. Two further real bugs surfaced and were probed,
  not guessed: (1) `Notify::notify_waiters` loses a wake when the waiter has not
  registered yet — the hang/cancel paths now use `notify_one`, which stores a permit
  (a scheduling race, invisible without a stress); (2) the supervisor kept pulling the
  stream after a terminal event — a handle yielding `Completed` repeatedly looped
  forever; the loop now breaks on any terminal event.
- Durable: the producer (both crates, the corpus, the tests) is tracked; the proof
  commands run in plain CI (no service — the fake and the journal are in-process).

## Rejected designs

- **`async-trait` crate** — the toolchain supports native `async fn` in traits; the
  auto-trait-bound limitation is accepted with a documented `#[allow(async_fn_in_trait)]`
  and a revisit trigger (a non-`Send` adapter).
- **Sleeps for the hang** — a `Notify` makes the hang resolve exactly on cancellation:
  fully deterministic, no timing.
- **Parsing provider output in the adapter** — chunks are opaque untrusted content at
  this boundary; a consumer's schema validator rejects malformed output, the adapter
  never interprets domain meaning.
- **A `cancelled_known` state** — a CONFIRMED cancel still leaves the result
  unknowable; Phase 0 keeps the honest `outcome_unknown` until proof/adjudication.
  `cancelled_known` stays deferred (see [[2026-09-06_state-transitions]]).
- **A shared wire crate or vendor DTOs anywhere** — the contract types are
  vendor-neutral; `reasonbraid-core` is untouched by this leaf except the one machine
  edge; the real adapter's vendor types will live inside its own module (`.4.2`).
- **A blanket "retry on failure" helper** — the supervisor never retries; retry
  decisions belong to the caller/policy, with explicit duplicate-risk authorization.

## How to apply

- Keep `invoke`'s refusal a CERTIFICATE: `FailedBeforeDispatch` promises no provider
  contact happened, and the journal records it from `dispatched` via the proof edge.
- Never add a credential field to the contract; keep the fixture credential scan red.
- New adapter outcomes = a new corpus fixture + coverage update, or the class is
  unsupported.
- The supervisor must stop at the FIRST terminal event; a stream is not a source of
  multiple results.
- The first real adapter (`.4.2`) qualifies against this corpus, semantically.
