---
answers:
  - Which items of `SIGNOFF-REPAIR.4.3`'s goal line (durable inbox identity and cursors) hold today, and which are live defects?
  - Where exactly does a node's identity fail to bind its events, reconciliation and results?
  - What happens to a node's cursor when its inbox is pruned empty?
  - In what order are `.4.3`'s children built, and why?
---
# The inbox-identity goal line censused: four live defects

- **Type:** decision
- **Status:** accepted
- **Owner:** `SIGNOFF-REPAIR.4.3`
- **Date:** 2026-09-23
- **Work unit:** `REASONBRAID-DOC-0152`
- **Cites:** ROADMAP §10.6 (the delivery ladder), §17.4 (reconnect, replay, dedup); `migrations/0003_node_inbox.sql`, `migrations/0078_node_inbox_offered.sql`; `docs/decisions/2026-09-23_the-federation-goal-line-after-the-remote-form-four-met-one-live-race.md` (DOC-0151 — the origin binding made two tenants' work on one node real).

## The fact / decision

Read against `e6e19f3`. The census was gathered by a delegated read-only pass and every load-bearing claim below was re-derived by `grep`/`sed` over the named lines before being recorded.

| Goal item | Verdict | Where |
| --- | --- | --- |
| Bind receipts / reconciliation / dedup to tenant and node | 🔴 LIVE → `.4.3.1` | `node_events.event_id` is the table's whole primary key (`migrations/0003_node_inbox.sql`), and `record_event_in_tx` inserts `ON CONFLICT (event_id) DO NOTHING` (`crates/reasonbraid-server/src/node_channel.rs:1816`): an event id already recorded by node B makes node A's submission of it `accepted = false`, and the events handler then skips A's fold. Event ids are node-chosen strings the server never namespaces. `event_id_for_operation` (`crates/reasonbraid-server/src/node_channel.rs:1109`) looks up `WHERE operation_id = $1` with no node predicate, and it answers BOTH the handshake's directives (a foreign receipt adjudicates this node's ambiguous attempt as `reconciled`) and `known_events` (a foreign event id disclosed to any node naming the operation). |
| Preserve monotonic cursors through pruning | 🔴 LIVE → `.4.3.2` | The cursor is `COALESCE(MAX(cursor), 0) + 1` at every writer (`crates/reasonbraid-server/src/node_channel.rs:688`, `:1783`; the quarantine replay at `crates/reasonbraid-server/src/authority/node_admin.rs:922`) and there is no durable high-water mark (`grep -rn "high_water\|inbox_cursor_floor" migrations` → none). The prune's delivered-rows `DELETE` can remove the top cursor, so after a node acknowledged up to N and its inbox was pruned below N: every handshake, poll and ack is refused `cursor_ahead` (`crates/reasonbraid-server/src/node_channel.rs:1959`, `:2250`, `:2275`) — the node is locked out — and once new enqueues climb back past N, the rows given cursors ≤ N are never offered (`cursor > $2`, `crates/reasonbraid-server/src/node_channel.rs:940`), never acknowledged, and count toward the backlog cap for ever. The attached clause is exactly this and no control exists (`node_work.rs`'s expiry-prune control empties an inbox and stops). |
| Command-id collisions across nodes cannot consume or hide foreign work | 🔴 LIVE → `.4.3.3` | The row lookups ARE node-bound (`load_command_for_tenant_in_tx`, the dead-letter `UPDATE`: `node_id AND command_id AND tenant_id`). Two surfaces are not: the result fold's idempotency claim is `(tenant_id, command_id)` (`migrations/0001`'s key; `apply_node_result_in_tx`), so two nodes holding one command id in one tenant fold at most once between them; and `node_inbox_state`'s `consumed` rung joins `node_events` on `e.operation_id = i.command_id` with no `e.node_id = i.node_id` (`0078:74`). Command ids are unique only per node (`UNIQUE (node_id, command_id)`). Dispatch ids (`work_{event_id}`) do not collide in practice; the key does not forbid it. |
| Serialize enqueue | 🔴 LIVE → `.4.3.4` | No writer serializes: `MAX + 1` under concurrency computes one cursor twice and the loser's insert raises the `(node_id, cursor)` key — the dispatching command rolls back rather than retrying (the docblock at `crates/reasonbraid-server/src/node_channel.rs:668-671` calls the profile *single-writer*). The origin binding (REPAIR-0446…0454) made two TENANTS' dispatches to one node possible, and a tenant guard orders neither against the other. No control exercises concurrent enqueue. |

## Why this order

By exposure. `.4.3.1` is reachable by any enrolled node deliberately (a chosen event id suppresses another node's result; a chosen operation id reads another node's receipt). `.4.3.2` needs an operator prune but then silently hides work and locks a node out. `.4.3.3` needs a colliding command id, which dispatch does not produce. `.4.3.4` fails loudly (a refused command) rather than silently. Each is built RED first; `.4.3.2`'s control is the attached clause's prune-everything-then-reconnect, written before the repair as the clause demands.

## How to apply

- Build `.4.3.1` → `.4.3.2` → `.4.3.3` → `.4.3.4`; `.4.3` closes with the last.
- ⛔ Do not repair the event key by rewriting stored ids: `node_events` rows are receipts; the key change is additive (`(node_id, event_id)`), and the dedup it serves is per node by definition.
