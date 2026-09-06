# The first real adapter: the Codex-family CLI behind `codex exec --json`, supervised as a subprocess

- **Type:** `decision`
- **Date:** `2026-09-06`
- **Status:** `active`
- **Owner / source:** engineering decision during leaf `PHASE-0.4.2` (WP4 first real harness)
answers: which real harness is qualified first behind the .4.1 adapter contract, on which boundary, and how do the acceptance legs (ack ≠ completion, honest unsupported lookup, no vendor types/credentials) hold on the REAL harness?

## The fact / decision

`crates/reasonbraid-adapter/src/codex.rs` lands the first REAL adapter:
[`CodexCliAdapter`] supervises the **Codex-family CLI** (`codex exec --json
--skip-git-repo-check --ephemeral --sandbox read-only <prompt>`) as a child process —
the narrowest supported machine interface (§11.6: "prefer current supported machine
interfaces"). Verified against codex-cli **0.153.4** (installed 2026-09-06):

- the JSONL stream is `thread.started` (thread id — the provider handle) →
  `item.completed` (streamed reply) → `turn.completed` (exact `usage` receipt);
- the contract gained [`AttemptEvent::ProviderRequestId`] because Codex reveals its
  handle in the stream, AFTER dispatch — the supervisor attaches it to the attempt
  exactly like an ack-carried id;
- **`query_status` is honestly `Unsupported`** (no first-class status query for a past
  attempt) — so the WP4 acceptance's honest leg is exercised by a REAL adapter: a lost
  response lands `outcome_unknown`, with no retry recommendation, and the attached
  thread id is the proof handle an operator would adjudicate with;
- cancellation is `BestEffort` (kill the child; no provider proof of stopping);
  provider idempotency keys: not exposed → `false`; receipts carry tokens, never
  money → normalized `cost` stays `None` (unknown, never zero);
- the run payload's `prompt` travels as the USER prompt — never as system
  config/policy (§16.6), and the adapter holds no credentials (ambient user login,
  §16.5).

The choice of Codex over Claude: both CLIs are installed on this host, but Codex's
`exec --json` is a first-class machine interface (event-typed stream, thread ids,
usage receipts), while Claude's path additionally requires the sidecar-vs-CLI
evaluation and subscription-credit vendor-terms compliance (§11.6) — heavier for the
FIRST adapter. The order follows the roadmap's own Codex-then-Claude sequence.

## Why

KICKOFF WP4: "implement one real adapter—Codex-family or Claude-family—using the
narrowest supported CLI/SDK boundary available on the selected development host."
This is the experiment for kill-risk question 2 (supervise a REAL harness through the
narrow adapter without contaminating the core). The WP4 acceptance legs — dispatch ack
distinct from completion, unsupported lookup → honest `outcome_unknown` — must hold on
a REAL harness, not only on the fake.

## Measured behavior (legs: re-derive · falsify · durable)

- Re-derived, live: `RB_LIVE_CODEX=1 cargo test -p reasonbraid-node --test codex_live
  -- --ignored --nocapture` → `test result: ok. 1 passed` — one bounded real dispatch
  (`"Reply with exactly: ok"`, 4.5 s) completed through the REAL supervisor + journal,
  the streamed thread id attached as the provider handle, usage normalized with exact
  confidence, `query_status` → `Unsupported`.
- Offline (no spend, tracked): `cargo test -p reasonbraid-adapter --test codex_adapter`
  → `test result: ok. 9 passed` (stub binary: spawn refusal, JSONL parsing, chunk
  order, non-zero exit → `failed_known` with the stderr tail, lost response with no
  terminal event, kill → `BestEffort`, receipt shapes) and
  `cargo test -p reasonbraid-node --test supervisor_codex_stub` → `test result: ok.
  2 passed` (completion attaches the streamed handle; lost response → `outcome_unknown`
  whose error carries no "retry").
- Falsified: while building the stub suite, the stub branched on `$1` instead of the
  last argument (the prompt) — every scenario misbehaved exactly as a mis-wired
  boundary should, and the tests caught it. The interface facts above are re-derived
  from the live event stream, never asserted from memory.
- Durable: the evidence report (`docs/evidence/2026-09-06_codex-adapter-qualification.md`),
  the ledger row (updated per its own revalidation trigger: `checked_at 2026-09-06`,
  `tested_versions ["0.153.4"]`, `license Apache-2.0` verified from the primary source),
  and the live test's gating are all tracked.

## Rejected designs

- **Claude-first** — the sidecar-vs-CLI evaluation and subscription-credit vendor-terms
  compliance (§11.6) are real scope; Codex's documented machine interface is the leaner
  first qualification. (Open question below keeps the door open for the second adapter.)
- **MCP active-client path as the first adapter** — that integration is inbound (an
  already-running harness calling ReasonBraid), not the outbound supervised-attempt
  boundary this leaf qualifies; it remains a separate Phase 0 option.
- **Wrapping an SDK in a sidecar for Codex** — unnecessary: the CLI's JSONL interface
  is supported and versioned on this host; a sidecar would add a second language
  runtime for no gain at this phase.
- **Fabricating a status lookup** — `codex exec resume` CONTINUES a thread (and bills
  again); it is not a status query, so the adapter reports `Unsupported` rather than
  pretending a lookup exists.
- **Putting cost estimates into `NormalizedUsage`** — the stream reports tokens only;
  cost stays `None` (unknown, not zero), per §14.1.

## How to apply

- The Codex adapter speaks `codex exec --json`; revalidate on the ledger's trigger
  (CLI release or invocation-mode change) before any release gate claims it.
- The live test stays `#[ignore]`-gated (`RB_LIVE_CODEX=1`) — it spends real credits.
- Never pass the run payload anywhere but the user prompt; never add a credential
  field to this adapter.
- A future status-lookup capability would arrive as a NEW provider feature — pin it by
  version and add a conformance fixture, or keep reporting `Unsupported`.

## Open question (director-owned)

Whether the second real adapter (Claude-family) lands in late Phase 0 or Phase 1. The
evidence report recommends **Phase 1** (one real + one fake suffices for the WP6 demo,
per `KICKOFF.md` §4 WP6; Claude adds sidecar/CLI evaluation and vendor-terms
compliance), but this is a product decision, not an engineering one.
