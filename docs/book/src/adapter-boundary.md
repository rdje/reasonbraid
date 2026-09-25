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

The fixture corpus behind the fake is a PERMANENT replay oracle: a versioned
manifest (`fixtures/MANIFEST.json`) records one entry per fixture — the §19.4
items it proves, the adding leaf, the reason — and the corpus drift-checks
against it, so the oracle can only change additively with a recorded reason.

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

The fake also counts its own `invoke` calls — every call, whether the script
then accepts or refuses. `invocation_counter()` hands out a shared handle, taken
before the fake is moved into the worker that drives it:

```rust
let adapter = FakeAdapter::new(vec![ScriptStep::Complete { usage: None }], lookup, caps);
let invocations = adapter.invocation_counter();
let worker = Worker::new(node, adapter, budget, poll_interval);
worker.tick().await?; // a stale cached decision: refused at the dispatch boundary
assert_eq!(invocations.load(Ordering::SeqCst), 0); // the provider was never reached
```

This is how a test proves that a gate **ahead of** the adapter refused. That
proof holds only if the fake is scripted to *succeed*. A fake scripted with
`fail_before_dispatch` ends up at the same journaled `failed_before_dispatch`
whether or not the gate exists, so a control built on one passes even with the
gate removed (`SIGNOFF-REPAIR.4.4.3`). The node-side controls that guard the
cached-decision gate (`node_work`, `node_replacement`) script a completing fake,
assert that it was invoked zero times, and check the gate's own reason in the
journaled attempt.

## The supervisor

`execute_attempt` (`crates/reasonbraid-node/src/supervisor.rs`) is the boundary
discipline from the journal chapter made executable: the attempt is journaled
`prepared`, the dispatch boundary is recorded **before** `invoke` runs, the
stream's chunks pass through verbatim (with one exception, below), and the
attempt lands on

```text
failed_before_dispatch · completed · failed_known · outcome_unknown
```

A stream that ends without a result is a lost response: `outcome_unknown` in
the journal, moved only by a proven lookup (`completed`/`failed_known`) or an
authorized adjudication (`reconciled`).

### The one exception to *verbatim*: U+0000

No store in the platform can hold the NUL character: PostgreSQL refuses it in
`jsonb` and in `text`. A result carrying one could never be accepted, and the
whole paid answer would be lost over a single character. So when the node
builds a result, each U+0000 becomes U+FFFD, the Unicode replacement character,
visible where the NUL stood. The result also says exactly **where**, as
`nul_positions`: canonical runs `[start, len]` over Unicode scalar indices into
the delivered content (`SIGNOFF-REPAIR.4.4.10.3.1`):

```text
provider output   "before\0after\u{FFFD}"
delivered content "before\u{FFFD}after\u{FFFD}"   and   "nul_positions": [[6, 1]]
```

The replacement is **lossless**. A U+FFFD the provider emitted itself (the last
one above) is not in a run, so the original is exactly the content with the
run scalars set back to U+0000. Runs are maximal, so an all-NUL result is one
run, however long.

The positions do not stop at the node. The control plane validates them for
any author (runs in order, non-empty, never touching, in bounds, over U+FFFD
only; anything else is `400 invalid_command`). It then stores them on the
`thread.contribution_submitted` or `thread.revised` event, which is what every
reader and agent sees. A person contributing through the API may send the same
field under the same rule. A blind contribution's positions are withheld with
its content until the commitment point.

Nothing else changes, control characters included. Output without NUL is
delivered exactly as it came, with no `nul_positions` field. Until
`SIGNOFF-REPAIR.4.4.10.3.1` the result carried only a count, `nul_replaced`.
That lost the positions, confused a provider's own U+FFFD with a replaced NUL,
and never reached the contribution. The decision is recorded in
`docs/decisions/2026-09-24_nul-in-provider-output-is-replaced-losslessly.md`.

### Two bounds on every attempt

The supervisor ends an attempt itself in two cases (`SIGNOFF-REPAIR.4.4.6`):

| Bound | What happens when it is crossed | Where the attempt lands |
| --- | --- | --- |
| **The request's deadline.** The worker sets it from the reservation's wall-clock allowance (60 s when none is given, never more than an hour). It bounds the wait for the provider's acknowledgement *and* for every event after it | the adapter is asked to cancel, and its answer is waited for up to 5 s | `outcome_unknown`, because the provider may have run. The evidence names the deadline, where it passed, and the cancellation's answer. A status lookup, if the adapter has one, may still prove the result |
| **`MAX_OUTPUT_BYTES` (256 KiB)** of collected output | the adapter is asked to cancel | `failed_known`, naming the bound. The excess is never kept, and a shortened result is never delivered as if it were whole |

For example, a provider that accepts the request and then never answers leaves
this in the journal once its deadline passes:

```text
outcome_unknown  {"reason":"the deadline 2026-09-24T… passed with no result; cancellation: Confirmed"}
```

Why 256 KiB: a finished result travels to the control plane as one JSON body,
and the control plane accepts at most 2 MiB. JSON can spend up to six bytes on
one byte of content (a `\u0000` escape), so 256 KiB fits even at the worst.
A larger result could never be delivered, and the node would keep offering
it.

Before this repair the deadline was computed, documented as the caller's to
enforce, and enforced by no one. A provider that never answered held the
node's worker for ever, and the collected output had no limit.

## The first real adapter: the Codex-family CLI

`CodexCliAdapter` (`crates/reasonbraid-adapter/src/codex.rs`) supervises
`codex exec --json --skip-git-repo-check --ephemeral --sandbox read-only -- <prompt>`
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

**The prompt follows `--`, and that separator is a security boundary**
(`SIGNOFF-REPAIR.10.1.1`). The prompt is untrusted participant content, and it
reaches the CLI as an argument. Without `--`, a prompt beginning with `-` would be
read by the CLI's option parser, after `--sandbox read-only`, so it could
countermand the sandbox. The Codex adapter lacked the separator until that leaf;
the Claude adapter always had it, because `--tools` takes a variable number of
values. The `ACTION-BOUNDARY` doctrine now refuses a commit in which either
adapter's flags stop ending with `--`, or carry it more than once (a second `--`
earlier would turn the flags after it, the sandbox included, into plain text), and each adapter's tests send an
option-shaped prompt to a stub that reports the arguments it received. ⚠️ The stub
proves what the adapter passes. How the real Codex CLI treats a prompt after `--`
is the standard command-line convention, and only an `RB_LIVE_CODEX` run observes
it directly.

**What the adapters read, and how much** (`SIGNOFF-REPAIR.10.1.2`). Both CLI
adapters read the child's two streams as bytes, through one shared module:

- **An event line holds at most 2 MiB.** The supervisor accepts at most 256 KiB of
  output per attempt, and the worst JSON escape is six bytes per byte, so 2 MiB
  never cuts a line the supervisor could take. A longer line is not kept while it is
  read: the child is stopped, and the attempt ends as a known failure naming the
  bound, the way the supervisor treats output over its own. A build in which the two
  bounds fall out of step fails to compile.
- **A line that is not UTF-8 is skipped.** It cannot be an event, just as a human
  status line cannot. It used to end the stream, losing the answer after it.
- **stderr is drained to its end, whatever it contains, and its last 8 KiB are kept.**
  A failure reason carries the last 1 KiB of that, cut on a character boundary. The
  old drain read text lines and STOPPED at the first one that was not UTF-8. That
  closed the pipe, and the provider was then killed by its next write to stderr
  (`SIGPIPE`). It also kept the first 8 KiB rather than the last, so after one long
  line the line that said what went wrong was never stored, and its 1 KiB tail was a
  byte slice that crashed the adapter when it fell inside a multi-byte character.

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

## Qualifying a new adapter (the §19.4 manual checklist)

A new adapter qualifies when every box below is ticked with evidence — the
conformance half is mechanical, this half is the human gate over the live runs:

- [ ] The adapter passes the conformance harness (register scenarios — the
  capability manifest, the unsupported-operation honesty, the dispatch
  boundary, the lost-response honesty, the cancellation ceiling, the
  usage-accounting floor).
- [ ] The adapter-specific mechanics are tested offline against a stub binary
  (prompt travel, stderr tails, the child kill, the provider receipt shapes).
- [ ] One bounded REAL dispatch passes env-gated:
  `RB_LIVE_<ADAPTER>=1 cargo test -p reasonbraid-node --test <adapter>_live
  -- --ignored --nocapture` — a tiny prompt, sandbox read-only, no files
  touched, a generous reservation + local budget gating the dispatch, and the
  exact usage + money cost recorded with confidence.
- [ ] Credentials never cross the contract: the adapter resolves its own
  credentials out of band (env/process boundary), the struct carries no
  credential field, and the new fixtures (if any) pass the mechanical
  credential scan.
- [ ] The corpus manifest gains an entry for any new fixture (additive, with
  the recorded reason) — the exact-match guarantee fails otherwise.
- [ ] The dependency ledger records the CLI version qualified (its
  revalidation trigger rides provider releases).

## The other boundary in the same SDK: resolvers

`reasonbraid-adapter` publishes two third-party surfaces, and they are **not the
same shape**. A HARNESS is implemented in process, by writing a Rust type that
implements the `Adapter` trait above. A RESOLVER is not.

| | Harness | Resolver |
| --- | --- | --- |
| What a third party writes | a Rust type implementing `Adapter` | a `ResolverAdvertise` row **and a worker binary** |
| Where the code runs | inside the node process | in its own process |
| In-process trait | `Adapter` | **none, by decision** |
| How it is verified | the conformance harness + qualification evidence | ADR-027's five-rung load ladder over the signed binary |

### Why there is no acquisition trait

A resolver advertises an ADR-018 `sandbox_level`, and that field states **what
the resolver's code provides** — not what the operator is expected to arrange
around it. The server's resolution filters on it: a reference may require a
floor, and a pack offering less isolation is ineligible for it.

Code that runs inside the server process provides `none`. There is no
arrangement under which it provides more. So an in-process acquisition trait
would let a third-party pack advertise `constrained_process` or `vm_container`
while structurally being `none`, and the filter would admit it on the strength of
the claim. That is worse than an absent surface: it turns the one advertised
field the registry actually consults into a field it cannot back.

The isolation that the ladder describes is a **process** boundary, so the
execution surface is a process. Two of the six built-in packs already work this
way, and their advertised levels track it exactly:

| Pack | Runs as | `sandbox_level` |
| --- | --- | --- |
| `r0-https-fetcher` | the server process | `none` |
| `r1-git-fetcher` | the server process | `none` |
| `r2-extract-worker` | a child process per extraction | `process` |
| `r3-browser-worker` | a child process per render | `process` |
| `r5-credential-broker` | the server process | `none` |
| `rx-agent-mediated` | the server process (the acquisition is the node's) | `none` |

ADR-027's load ladder — allowlist, digest, signature, API compatibility,
capability manifest — verifies exactly that artefact: a signed binary. None of
its five rungs has a meaning for a trait implementation compiled into the server.

### What a third party can build today, and what is still missing

**Today:** the `ResolverAdvertise` row, which the capability registry consumes,
filters and ranks. That surface is stable and lives in the SDK crate.

⚠️ **Not yet:** the worker wire protocol is **not** published as a stable SDK
surface. The `extraction` and `browse` modules inside `reasonbraid-server` own
those request/response shapes, and promoting one of them into the SDK is a
separate decision with its own compatibility obligations. Said plainly so the
gap is a stated limit rather than something inferred from an absence.

## Honest limits

- The fake is the deterministic oracle; the Codex and Claude adapters are the
  two real ones, each qualified on one host and one CLI version (the dependency
  ledger revalidation triggers cover releases).
- A confirmed cancellation still leaves the result unknowable, so it lands on
  `outcome_unknown` — `cancelled_known` remains out of Phase 1.
- Deadline and budget enforcement are the caller's (WP5 types the reservations).
