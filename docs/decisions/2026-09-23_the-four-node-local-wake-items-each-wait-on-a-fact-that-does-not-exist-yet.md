---
answers:
  - Which of §11.5's node-local wake items can be built against the adapters that ship today?
  - What does each one wait on, and how is that condition read?
  - Where do notification controls stand after the storm-control repairs of 2026-09-22/23?
---
# The four node-local wake items each wait on a fact that does not exist yet

- **Type:** decision
- **Status:** accepted
- **Owner:** `SIGNOFF-REPAIR.11.4.7.2.1.5.3.2.4`
- **Date:** 2026-09-23
- **Cites:** ROADMAP §11.5 (*before wake, the node evaluates* … *recursion, duplicate, and
  notification controls pass; required tools/resources are allowed; the adapter is healthy and
  billing route valid*), §10.7, §11.2 (the adapter contract), §14.5 (billing routes);
  `docs/decisions/2026-09-23_the-wake-checklist-is-a-node-gate-and-the-auto-grant-carries-six-bounds.md`
  (DOC-0135, item 4)

## Context

DOC-0135 left four wake-checklist items to this leaf: notification controls, required
tools/resources, adapter health and billing route, each to be *adjudicated against the
adapters that actually ship, then built or given an evaluable trigger*. Every claim below
cites the command that decides it, run at `afa5706`.

## What ships

- `rb-node` constructs exactly one adapter: `grep -n "Adapter" crates/reasonbraid-node/src/bin/rb-node.rs`
  → `FakeAdapter::new(script, lookup, capabilities)` with `tool_support: false`.
- Two CLI adapters exist as library code (`crates/reasonbraid-adapter/src/codex.rs`,
  `claude.rs`), both declaring `tool_support: false`; the node binary constructs neither.
- The adapter contract (`crates/reasonbraid-adapter/src/contract.rs`): `capabilities()`,
  `invoke`, `cancel`, `query_status`, `normalize_usage`. **No health method.** A
  `RunRequest` carries `payload`, `deadline`, `budget_hint` — **no tool requirement.**
  `AdapterCapabilities` carries `streaming`, `cancellation`, `provider_idempotency`,
  `status_lookup`, `tool_support`, `policy_injection` — **no billing route.** The
  contract is pinned by `SDK_VERSION = "1"` and a third-party certification matrix
  (`tests/third_party_certification.rs`), so a field added to it is a contract version.
- `grep -rn -i billing crates/ --include=*.rs` → **0**. `BudgetDimensions` carries calls,
  tokens and wall-clock; §14.1's *money by currency/billing route* dimension has no field.
- `grep -rn -i "urgency\|priority" crates/reasonbraid-server/src/api.rs
  crates/reasonbraid-server/src/recruitment.rs` → **0**: a call has no urgency class.

## The four, adjudicated

| item | today | verdict | the trigger, and how to read it |
| --- | --- | --- | --- |
| notification controls (§10.7) | fan-out caps per tenant and initiator (`PHASE-3.4`); the invite quota (`0047`); the `initiator` quota (REPAIR-0417); quiet hours (REPAIR-0419); node-level quarantine of a command (`POST /v1/nodes/quarantine`) | ⚠️ **partly built** — three of §10.7's controls remain: the offline backlog cap is `SIGNOFF-REPAIR.11.4.7.2.1.5.3.3` (next); digest/coalescing and principal quarantine wait | *digest/coalescing for low-urgency calls*: a call carries an urgency class — `grep -rn -i urgency crates/reasonbraid-server/src` > 0. *Quarantine for noisy principals*: `SIGNOFF-REPAIR.11.4.7.2.1.5.3.4` records storm-control refusals, which is what identifies a noisy principal |
| required tools/resources allowed | nothing: no run carries a tool requirement, no grant or profile lists allowed tools, every shipped adapter declares `tool_support: false` (the Claude CLI adapter passes a fixed `--tools` argument to its process, `claude.rs:84`, which is the adapter's own configuration, not a per-run requirement) | ⏸️ **not buildable**: a bound needs a run that declares what it requires, and none does | `RunRequest` gains a tool requirement, or an adapter the node constructs declares `tool_support: true` — `grep -n "tool" crates/reasonbraid-adapter/src/contract.rs` shows more than the capability flag |
| adapter healthy | no health method on the contract; the node's `journal.health()` is the SQLite journal's, not the adapter's; the fake adapter has nothing to be unhealthy about | ⏸️ **not buildable for the node that ships**: a probe over the fake proves nothing, and the CLI adapters are not constructed by `rb-node` | `rb-node` constructs a CLI adapter — `grep -n "CodexCliAdapter\|ClaudeCliAdapter" crates/reasonbraid-node/src/bin/rb-node.rs` > 0. Then a health probe is *the binary is present and its version is one the contract accepts* (§11.2's *version and compatibility evidence*) |
| billing route valid | no field, no vocabulary, no policy | ⏸️ **two halves**: RECORDING (§14.5: *the adapter records the configured route*) is a contract change — a `billing_route` on `AdapterCapabilities` bumps `SDK_VERSION` and the certification matrix; VALIDITY needs a tenant policy naming accepted routes, which nothing declares | recording: the next `SDK_VERSION` bump (`grep -n 'SDK_VERSION: &str' crates/reasonbraid-adapter/src/contract.rs`), owned by `SIGNOFF-REPAIR.11.4.7.2.1.5.3.2.4.1`; validity: a tenant policy names accepted routes |

## Decision

- ✅ `.5.3.2.4` closes as adjudicated: none of the four is buildable against the node that
  ships without inventing the fact it would evaluate, and inventing it would be the
  declared-and-unread defect this whole family exists to remove.
- 🔨 `.5.3.2.4.1` — the billing-route RECORDING half, opened on the SDK version trigger so
  the contract changes once rather than per field.
- The three remaining §10.7 controls stay owned where they were: the backlog cap by
  `.5.3.3`, refusal recording (the breaker's and quarantine's precondition) by `.5.3.4`,
  digest/coalescing on the urgency trigger.
- ⛔ `.5.3.2` — the leaf that owned *`PHASE-3.5.3` named the checklist and built four of its
  gates* — closes with this record: its four children are decided, five of the six grant
  bounds are carried and read, the wake gate evaluates three declarations, and what
  remains is named with the condition that reopens it.
