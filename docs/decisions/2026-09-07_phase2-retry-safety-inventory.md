# Phase 2.7's no-false-safe-retry evidence inventory: the §16.12 line's dev-profile proof (`PHASE-2.7.3`)

- Date: 2026-09-07 · Leaf: `PHASE-2.7.3` · Decision record

## Context

§16.12's Internet gate demands the authorization non-escalation AND the
recovery-honesty evidence. The `.7.1` adversarial suite measured the
escalation fence; this record inventories the RETRY-honesty half: the
no-false-safe-retry property ("no false safe-retry of unknown attempts" —
the `.7` lane's third property) across every boundary where an attempt
could be silently re-dispatched. Each leg below is MEASURED (a named test
or drill in the guard), not argued.

## The inventory (each leg = a measured proof)

| # | Boundary | The false-safe-retry it would be | The measured proof |
| --- | --- | --- | --- |
| R-1 | The retry decision (pure) | retrying a budget-denied attempt, or an ambiguous one without authorization | `crates/reasonbraid-node/tests/worker_retry_policy.rs` (4 legs): a reserved pre-dispatch refusal redispatches; a budget denial is NEVER redispatched; an ambiguous outcome refuses WITHOUT the `allow_possible_duplicate` authorization; an authorized ambiguous outcome redispatches |
| R-2 | The worker's dispatch gate | an epoch-stale or revoked cached decision dispatching | the `.1.5.2` live leg (`node_work`): the revocation bumps the epoch 0→1 and the next dispatch refuses WITHOUT a re-ask (the adapter never runs) |
| R-3 | The quarantine replay gate | a dead-lettered attempt re-dispatching without the operator | the `.2.4` live leg (`node_work`): the once-only dead-letter report, the server's auto-quarantine in the receipt transaction, and the decision-scoped re-arm only via `POST /v1/nodes/replay` |
| R-4 | The machine-loss boundary (the replacement) | the replacement incarnation silently re-dispatching the lost node's in-flight work | the `.7.2` drill (`node_replacement`): the re-delivered decision is epoch-stale, the dispatch refuses FAIL-CLOSED, the row dead-letters, the operator replays, exactly one contribution lands |
| R-5 | The kill/reboot boundary | a killed node's in-flight attempt retrying on restart | the demo's SIGKILL beat (step 7): the attempt recovers `outcome_unknown` — bounded and visible, NEVER silently retried; the challenge stays unresolved |
| R-6 | The revocation fence around history | a revocation rewriting or re-executing the past | the `.7.1` adversarial test: the original command's replay returns its stored result verbatim (the claim precedes authorization by design — exactly ONE contribution event), the next NEW command refuses and is audited |

## answers:

- **The property holds at every boundary that exists** — the retry
  decision, the dispatch gate, the quarantine gate, the replacement
  fence, the reboot fence, and the history fence are each measured by a
  named leg on every guard pass; there is no unmeasured "safe retry"
  path in the dev profile.
- **The one boundary without a machine is the operator** — the
  `allow_possible_duplicate` flag and the replay verb are HUMAN
  decisions by design (§14.6); the inventory proves the machine refuses
  by default everywhere else.
- **The replacement fence is the surprise the drill measured** — the
  revocation epoch (`.1.5.2`) turned out to fence the replacement's
  re-delivery for free: the lost incarnation's cached decision is
  epoch-stale against the post-revocation epoch, so the re-dispatch
  refuses fail-closed (R-4) — a property the runbook's pre-drill draft
  had assumed was a deferral.
