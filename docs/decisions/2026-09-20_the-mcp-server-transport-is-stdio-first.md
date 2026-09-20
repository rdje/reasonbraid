---
answers:
  - How does an MCP client reach ReasonBraid's tools?
  - Which MCP server transport did the project take, and why that one?
  - What does the Streamable-HTTP server transport cost?
  - Is the MCP surface served from rb-server or its own binary?
---
# The MCP server transport is stdio first, and the alternative would break a doctrine

- **Type:** decision
- **Status:** accepted; `rb-mcp` ships with a transport-level control
- **Owner:** `SIGNOFF-REPAIR.6.8`
- **Date:** 2026-09-20
- **Related:** ADR-024 (MCP interoperability surface), ROADMAP §9.6,
  `docs/decisions/2026-09-18_lan-completeness-precedes-internet-exposure.md`,
  `docs/knowledge/a-deferral-dies-with-the-leaf-it-names.md`

## The gap

Six MCP tools are implemented, authorized and live-tested, and **no MCP client
can reach any of them.** Re-derived with `cargo metadata` rather than with a
grep: **0 workspace packages depend on `reasonbraid-mcp`**, and its only targets
are a `lib` and a build script. The `rmcp` pin carries
`macros + server + transport-async-rw`; nothing enables a transport that a
client can connect to, and no binary links the crate.

🔎 **The cause is recorded, not guessed.** `PHASE-8.3.3`'s Done list ends *the
Streamable-HTTP transport + the live roundtrip ride `.3.4`*; `PHASE-8.3.4`
shipped the listen-stream durable state and closed `done` without one. A
deferral that names a leaf dies with that leaf.

## The two candidate profiles, priced

`cargo metadata` over the resolved graph, with each feature added in turn and the
manifest restored afterwards:

| Profile | rmcp feature | New packages | Consequence |
| --- | --- | --- | --- |
| **stdio** | `transport-io` | **0** | `transport-io` = `transport-async-rw` (already on) + `tokio/io-std` — a feature of a crate the workspace already builds |
| Streamable-HTTP server | `transport-streamable-http-server` | **3**: `async-trait 0.1.92`, `base64 0.23.1`, `sse-stream 0.2.6` | 🔴 **breaks `make deny`** |

🔴 **The third row of that table is the decision.** `deny.toml` sets
`multiple-versions = "deny"` — *two versions of one crate hide a split decision —
force a choice* — and the workspace already resolves `base64 0.22.1`. The HTTP
server transport brings `base64 0.23.1`, which is **not** in the reviewed `skip`
list and would have to be added to it: a deliberate weakening of a supply-chain
doctrine, for an encoding crate, to enable a listener nobody is yet allowed to
expose.

## The decision

**stdio, in a binary of its own: `rb-mcp`.**

- ⭐ **Zero new dependencies**, so there is no supply-chain review to pass and no
  doctrine to weaken. The dependency question the leaf owned has a measured
  answer rather than an argued one.
- ⭐ **It is inside the LAN bar by construction, so it does not touch
  `SIGNOFF-REPAIR.14`.** A stdio server opens no socket and accepts no inbound
  connection: the client spawns the process and speaks over its stdin/stdout.
  The director's 2026-09-18 instruction — the LAN must fully work before
  Internet exposure — is satisfied without an argument, because there is no
  exposure to argue about.
- ⛔ **A binary of its own is forced, not preferred.** stdio is one process per
  client, owned by the client and living for the length of one session;
  `rb-server` is a long-running HTTP service shared by every caller. The two
  lifetimes cannot be the same process.
- ⚠️ **This is a FIRST profile, not the only one.** Streamable-HTTP remains the
  right answer for a LAN client that cannot spawn a process, and this record is
  what a later leaf re-opens: the `base64` split is the thing it must resolve,
  and resolving it means either upstream converging or an argued `skip` entry —
  never a silent one.

## What the control proves, and what it does not

The acceptance asked for a control driving a tool **through the transport**
rather than through the tool router, because calling `McpTools::tool_router()`
directly is exactly the evidence that was already there while nothing was
reachable.

`crates/reasonbraid-mcp/tests/stdio_transport.rs` spawns the real `rb-mcp`
binary and speaks newline-delimited JSON-RPC to its stdin, reading its stdout:
`initialize`, the `notifications/initialized` notification, `tools/list`, and a
`tools/call`.

⚠️ **Stated precisely rather than generously:** the offline control drives a tool
call whose refusal is decided before the pool is touched, so it proves the
transport, the framing, the handshake, the tool listing and the dispatch into a
real handler — **not** that a tool can return a database-backed answer. The
pg-gated arm covers that by driving a call that reaches the enrolment lookup and
returns its decision. Both are named for what they are in the test's own header.
