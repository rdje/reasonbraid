# Evidence — Codex-family adapter qualification (first real harness)

- **Date:** `2026-09-06`
- **Leaf:** `PHASE-0.4.2`
- **Status:** `reported`
- **Decision owner:** engineering decision during leaf `PHASE-0.4.2` (recorded in
  `docs/decisions/2026-09-06_real-adapter-codex.md`)
- **Related ADR:** none (ADR-012, provider-attempt ambiguity, is deferred to WP8's ADR set)

## Question

Can the first REAL harness be supervised through the `.4.1` adapter contract on the
narrowest supported machine boundary — with dispatch acknowledgement distinct from
completion, an honest unsupported status lookup, and no vendor type or credential in
the core or the fixtures?

## Competing options

1. **Codex-family CLI** (`codex exec --json`, installed 0.153.4 at
   `/opt/homebrew/bin/codex`): a documented JSONL machine interface with a
   `thread.started` thread id, streamed `item.completed` events, and a `turn.completed`
   usage receipt. Apache-2.0 (verified from `openai/codex` LICENSE, 2026-09-06).
2. **Claude-family CLI** (`claude -p`, installed 2.1.263): non-interactive print mode
   with `--resume`/`--continue`; roadmap §11.6 additionally requires evaluating a
   supervised sidecar vs the CLI path and subscription-credit vendor-terms compliance.
3. **MCP active-client path** (§11.6): an already-running harness calling ReasonBraid
   tools — a different integration (inbound), not the outbound supervised-attempt
   adapter this leaf qualifies.

## Fixture

- Reproduce the live qualification:
  `RB_LIVE_CODEX=1 cargo test -p reasonbraid-node --test codex_live -- --ignored --nocapture`
  (one bounded dispatch, prompt `"Reply with exactly: ok"`; `--skip-git-repo-check
  --ephemeral --sandbox read-only` — no files touched, no session persisted).
- Offline supervision mechanics: `cargo test -p reasonbraid-adapter --test codex_adapter`
  and `cargo test -p reasonbraid-node --test supervisor_codex_stub` (stub binary under
  the repo's build dir; same subprocess boundary, fake provider).
- Host: macOS dev host, `codex-cli 0.153.4`, ambient user login (no credentials in the
  repo; the adapter has no credential field at all).

## Observable result

```text
command: RB_LIVE_CODEX=1 cargo test -p reasonbraid-node --test codex_live -- --ignored --nocapture
output:  LIVE CODEX OK: attempt patt_… completed via thread 01a0760b-…  ·  test result: ok. 1 passed
```

Measured on the real harness:

- the JSONL stream is `thread.started` (thread id — a provider handle) →
  `item.completed` (the reply) → `turn.completed` with an exact `usage` receipt;
- dispatch ack ≠ completion holds: the ack carries NO handle (Codex reveals it in the
  stream), the result arrives later, and the supervisor attaches the streamed thread id
  to the attempt as the proof handle;
- `query_status` is `Unsupported` (no first-class status query for a past attempt) —
  the acceptance's honest leg is therefore exercised by the REAL adapter;
- no cost field in receipts → normalized `cost` stays `None` (unknown, never zero);
- cancellation is `BestEffort` (kill the child; no provider proof of stopping);
- the receipt shape is re-derived (not asserted from memory): the probe event stream is
  recorded above the adapter implementation, and the offline tests pin the parsing.

Legs: **re-derive** — the live test + stub tests are tracked and re-runnable. **falsify**
— the offline stub suite fails on the arg-position bug found while building (stub
branched on `$1` instead of the last arg), and the fake corpus still drives every
outcome through the same supervisor. **durable** — tests, ledger row, and this report
are tracked; the live run is re-runnable on any host with an authenticated Codex.

## Decision

The first real adapter is the **Codex-family CLI** behind `codex exec --json`
(`docs/decisions/2026-09-06_real-adapter-codex.md`). **Recommendation for the second
adapter (Claude-family): late Phase 0 only if the WP6 two-host demo wants two real
harnesses — otherwise Phase 1.** Rationale: the WP4/WP6 slice works with one real +
one fake (`KICKOFF.md` §4 WP6 allows this, recorded as a limitation); Claude adds the
sidecar-vs-CLI evaluation and subscription-credit vendor-terms compliance, which is
scope better spent on WP5–WP6 in Phase 0. This is a recommendation, not a decision — the
director owns it (recorded in the decision record's open question).

## Deletion plan

This report and the stub tests stay as long as the Codex adapter exists (they pin the
invocation and receipt shape). The live test remains `#[ignore]`-gated forever — it
consumes real credits on every run. If the `codex exec --json` interface changes, the
ledger row's `revalidation_trigger` fires and this report becomes `obsolete` (superseded
by a new qualification report); git history preserves the old measurements.
