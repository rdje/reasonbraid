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

## The conformance harness (`.6.1`)

§19.4: a "works once" demo does not qualify an adapter. One suite
(`crates/reasonbraid-adapter/tests/adapter_conformance.rs`) runs every adapter —
the fake and both real CLI adapters (behind stub binaries) — through the same six
invariants: the capability manifest agrees with the verified boundary; a
never-dispatched lookup is honestly `Unsupported`; a refusal happens BEFORE any
provider contact; a lost response never invents a terminal event; a cancel never
exceeds the declared strength; an empty usage receipt is `Unknown`, never zero.
Each adapter registers scenarios (name + trigger + declared capabilities + the
tripping request) — a new adapter conforms by registering, not by re-proving the
contract in its own file.

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

## The first real adapter: the Codex-family CLI

`CodexCliAdapter` (`crates/reasonbraid-adapter/src/codex.rs`) supervises
`codex exec --json --skip-git-repo-check --ephemeral --sandbox read-only <prompt>`
as a child process — the narrowest supported machine interface (qualified
against codex-cli 0.153.4). The JSONL stream maps onto the contract:

| Codex event | Contract event |
| --- | --- |
| `thread.started` | `ProviderRequestId` (the thread id — attached to the attempt as the proof handle) |
| `item.completed` | `OutputChunk` (the streamed reply, verbatim) |
| `turn.completed` | `Completed { usage }` (exact token receipt) |
| non-zero exit | `FailedKnown` (definitive, with the stderr tail) |

The real harness exercises the contract's honest legs: **status lookup is
genuinely unsupported** (no first-class query for a past attempt), so a lost
response lands `outcome_unknown` — with the thread id attached as the handle
an operator would adjudicate with. Cancellation is `BestEffort` (kill the
child). The run payload travels as the **user prompt only**, and the adapter
holds no credentials (ambient Codex login).

The live qualification test is deliberately not run by default — it dispatches
to the real harness and spends a few tokens:

```text
RB_LIVE_CODEX=1 cargo test -p reasonbraid-node --test codex_live -- --ignored
```

Offline, the same supervision mechanics run against a stub binary in plain
`cargo test` (no provider spend).

## The second real adapter: the Claude-family CLI

`ClaudeCliAdapter` (`crates/reasonbraid-adapter/src/claude.rs`) supervises

```text
claude -p --output-format stream-json --restricted --tools '' --verbose -- <prompt>
```

as a child process — the `.4.2` mirror, qualified against Claude Code 2.1.263
(probe evidence in `target/claude-probes/`). The stream maps onto the contract:

| Claude event | Contract event |
| --- | --- |
| `system/init` (`session_id`) | `ProviderRequestId` (the session id — the proof handle, arriving before the first chunk) |
| `assistant` message text blocks | `OutputChunk` per text block (thinking blocks are skipped — the reply is the text) |
| `result` (`is_error:false`; `usage`, `total_cost_usd`) | `Completed { usage }` — exact tokens AND money (Claude reports cost; Codex reports tokens only) |
| `result` (`is_error:true`) | `FailedKnown` (the provider's own message) |
| non-zero exit | `FailedKnown` (with the stderr tail) |

`--restricted` removes the code-running tools and WebFetch, and `--tools ''`
disables all tools — the boundary is content-only. `--verbose` is not optional:
the CLI refuses `stream-json` without it, before any dispatch. The prompt
travels after `--` (the `--tools` flag is variadic and would otherwise swallow
it), as USER content — never config; the adapter holds no credentials (ambient
Claude login). Status lookup is honestly unsupported (`--resume` continues a
session; it does not query a past attempt), so a lost response stays
`outcome_unknown`.

The live qualification test is deliberately not run by default — it dispatches
to the real harness and spends a few tokens:

```text
RB_LIVE_CLAUDE=1 cargo test -p reasonbraid-node --test claude_live -- --ignored
```

## Honest limits

- The fake is the deterministic oracle; the Codex and Claude adapters are the
  two real ones, each qualified on one host and one CLI version (the dependency
  ledger revalidation triggers cover releases).
- A confirmed cancellation still leaves the result unknowable, so it lands on
  `outcome_unknown` — `cancelled_known` remains out of Phase 1.
- Deadline and budget enforcement are the caller's (WP5 types the reservations).
