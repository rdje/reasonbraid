# 2026-09-06_claude-cli-adapter.md

## Context

`PHASE-1.4` (backlog 21): the second genuinely distinct harness adapter. The
director adopted the recommendation — the Claude Code CLI, built as the `.4.2`
Codex subprocess mirror, fake-first and env-gated. The live `claude` CLI is
installed on the dev host (2.1.263), so the real harness leg runs for real.

## Decision

- **The Claude-family adapter is the Claude Code CLI behind
  `claude -p --output-format stream-json --restricted --tools '' --verbose -- <prompt>`** —
  the same supervised-subprocess boundary as Codex: no SDK, no sidecar.
- **The wire format is pinned by live probes, never by documentation prose.**
  Three bounded dispatches against the installed 2.1.263 established the facts
  the adapter codes against: `system/init` carries `session_id` (the provider
  handle, before the first chunk); `assistant` text blocks are the reply
  (thinking blocks skipped); `result` carries `usage` AND `total_cost_usd`;
  `--verbose` is REQUIRED (the CLI refuses stream-json without it, pre-dispatch);
  the prompt must follow `--` (variadic `--tools` otherwise swallows it).
- **Claude reports money, so the normalized cost is known.** The receipt is the
  FULL result event (usage under `usage`, dollars under `total_cost_usd`);
  `normalize_usage` maps `cost: Some(total_cost_usd)` — the one normalization
  leg the Codex receipt cannot prove. Anthropic's token counts already include
  cache reads and thinking: no folding (Codex's reasoning fold is
  receipt-specific).
- **The boundary is content-only.** `--restricted` removes the code-running
  tools and WebFetch; `--tools ''` disables all tools. The prompt travels as
  USER content after `--`, never as config; no credentials cross the contract
  (ambient Claude login).
- **Status lookup stays honestly unsupported.** `claude --resume <session-id>`
  continues a session; it is not a query of a past attempt. A lost response
  stays `outcome_unknown`, same as Codex.

## Consequences

- `.1.4.1` landed `claude.rs` + the 10-test offline stub suite; `.1.4.2` ran
  the live qualification ONCE against the real CLI (1 passed in ~2 s: completed,
  exact usage + money cost, session id attached as the provider handle,
  unsupported lookup) — evidence in the dependency-ledger row (checked_at
  2026-09-06, tested 2.1.263) and on-volume in `target/claude-probes/`.
- `.1.4` is complete: Codex + Claude + the deterministic fake — two genuinely
  distinct harness adapters (backlogs 19–21).

answers:

- **Probe the wire before coding the wire.** A harness adapter's whole risk is a
  guessed event shape; three bounded live dispatches turned "documented
  somewhere" into "observed, with the exact refusal the CLI itself emits when a
  required flag is missing". The probe evidence stays in the repo (on-volume)
  so the claim is re-derivable, and the live test passed on its first run.
- **Receipt shape is adapter-specific, the contract is not.** Codex folds
  reasoning into output and reports no money; Claude reports money and
  pre-folded counts. Both land on the same `NormalizedUsage` — the differences
  live in each adapter's `normalize_usage`, never in the contract.
- **"Resume" is not "query".** A CLI that can continue a session is easy to
  mistake for one that can look a past attempt up; the honest capability
  declaration keeps a lost response `outcome_unknown` instead of silently
  treating resume as proof.
