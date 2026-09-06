# The adapter boundary

The node supervises harnesses through a **narrow, vendor-neutral contract**
(`ROADMAP.md` §11.2). This chapter describes the Phase 0 boundary: what the
contract promises, how the deterministic fake adapter behaves, and how an
indeterminate attempt stays honest.

## The contract

Every adapter implements one trait (`crates/reasonbraid-adapter/src/contract.rs`):

```text
capabilities()   -> AdapterCapabilities   // declared, never inferred
invoke(request, operation_id)
                 -> FailedBeforeDispatch  // refused BEFORE any provider contact
                  | Accepted(ack, handle) // dispatched; the ack is NOT a result
cancel(operation_id)    -> Confirmed | BestEffort | Ignored
query_status(operation_id) -> Unsupported | Supported(result)
normalize_usage(raw_receipt) -> NormalizedUsage
```

Four properties make the boundary safe:

1. **Dispatch acknowledgement is distinct from completion.** The ack carries the
   provider's request handle; the result arrives later — streamed chunks, then
   `completed` or `failed_known` — or never.
2. **An unsupported status lookup is a fact, not advice.** When a lost response
   cannot be proven, the attempt is journaled `outcome_unknown` and the caller
   receives an error that carries **no retry recommendation**. Retrying an
   indeterminate attempt needs duplicate-risk authorization (§14.6) — a policy
   decision, never made inside the boundary.
3. **No credentials cross the contract.** There is no credential field; an
   adapter resolves its own credentials out of band (§16.5). The fixture corpus
   is mechanically scanned for credential-shaped content.
4. **Capabilities are declared.** Streaming, cancellation strength, provider
   idempotency, status lookup, tool support, and the policy-injection mode —
   callers branch on these, never on a provider name.

## The fake adapter

`FakeAdapter` (`§11.6`'s conformance oracle) plays a per-operation script:

| Script step | Behavior |
| --- | --- |
| `emit_chunk` / `malformed_output` | stream one opaque chunk (never parsed at this boundary) |
| `complete` | terminal result, optionally with a usage receipt |
| `fail_known` | terminal, definitive failure |
| `fail_before_dispatch` | `invoke` refuses before any provider contact |
| `hang_forever` | no events until cancelled; the cancel is `Confirmed` and ends the stream |
| `ignore_cancellation` | the cancel is reported `Ignored`; the run continues |
| `lose_response` | dispatch acked, then nothing — only a status lookup can prove the result |

There are no sleeps: the hang resolves exactly on cancellation, so every test is
deterministic and the same script always yields the same event sequence.

## The supervisor

`execute_attempt` (`crates/reasonbraid-node/src/supervisor.rs`) is the boundary
discipline from the journal chapter made executable: the attempt is journaled
`prepared`, the dispatch boundary is recorded **before** `invoke` runs, the
stream's chunks pass through verbatim, and the attempt lands on

```text
failed_before_dispatch · completed · failed_known · outcome_unknown
```

A stream that ends without a result is a lost response: `outcome_unknown` in
the journal, moved only by a proven lookup (`completed`/`failed_known`) or an
authorized adjudication (`reconciled`).

## Honest limits (Phase 0)

- The fake is a dev-only oracle; the first REAL harness qualifies against the
  same corpus in `.4.2` (vendor types stay inside its own module).
- A confirmed cancellation still leaves the result unknowable, so it lands on
  `outcome_unknown` — `cancelled_known` remains out of Phase 0.
- Deadline and budget enforcement are the caller's (WP5 types the reservations).
