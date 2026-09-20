# The MCP surface

ReasonBraid exposes its verbs as MCP tools. The rule the whole surface is built
on (ADR-024) is that **a tool is the HTTP handler's re-expression, never a new
authority path** — so a tool that no handler backs is not exposed, and a tool
that is exposed runs the handler's own authorization by *calling* it.

The tools are served by **`rb-mcp`**, a binary that speaks MCP over stdio — see
[Connecting a client](#connecting-a-client-rb-mcp-over-stdio) at the end for what
to point a client at, and for what is deliberately not served.

## The tool vocabulary

Six tools ship. Each takes a `principal` argument carrying the caller's id —
the development profile's trust shape, the same value the HTTP surface takes in
its principal header.

### The read half

| Tool | Returns | Admission, as the code performs it | HTTP twin |
| --- | --- | --- | --- |
| `get_thread` | one thread's current projection and its events | `authorize_inspection` on `Thread { tenant, thread }`, then the tenant-bound `thread_inspection` select | `GET /v1/threads/{id}` |
| `list_inbox` | one node's inbox rows with their delivery states | `authorize_tenant_admin` for the named tenant, then `inbox_inspection` | `GET /v1/nodes/inbox` |
| `get_policy_bundle` | the site's registered policy documents, digest-pinned | the caller must be enrolled; then `policy::list` | `GET /v1/policies` |

Each read names a **tenant argument**, and that named tenant is both what
authority is checked against and what the select is bound to — the two cannot
disagree without the read returning nothing. Deriving the tenant from the target
instead would answer a question the caller did not ask, and refusing on a
mismatch *before* authorizing would tell an unauthorized caller which tenant owns
the target.

⚠️ **`get_policy_bundle` takes no tenant, and the omission is the point.** The
policy registry is site-global: `policy_versions` has no tenant column and no
site filters on one. A tenant argument there could only be ignored or used to
label the response with a scope that did not produce it. The HTTP surface
returns the same set to any enrolled caller.

### The write half — the qualified capability profile

| Tool | Does | What the handler itself brings |
| --- | --- | --- |
| `respond` | contribute to a thread | the full pipeline: idempotency, the `thread_contribute` grant, the audit record |
| `join_call` | answer a recruitment call (`join`, `decline`, `observe`, …) | the enrolled role, the call's state, and — for the participation kinds only — eligibility. **No per-verb grant and no audit row** |
| `propose_policy_change` | register a policy-change proposal | an enrolment check and the insert. **Neither a per-verb grant nor an audit row** |

⚠️ **The three are not uniform, and an earlier version of this documentation
said they were.** `join_call` has no `GrantAction` for answering a call, and
`propose_policy_change` writes no audit record — but neither tool is weaker than
its HTTP verb, which does exactly the same. The gap is in the verb, not in the
MCP expression of it.

Every write tool passes the **qualified gate** first, and the gate is the only
thing the MCP seam adds:

1. **The enrolment binding** — the principal's recorded tenant must equal the
   tenant the call names. The handler therefore never sees a tenant the caller
   merely claimed.
2. **The per-principal quota** — a fail-closed bound on call volume. It counts
   **admitted calls, not effects**: one accepted call is one unit whether it
   contributes a sentence or a chapter. A refusal at the ceiling **commits its
   denial row**, because a refusal is a recorded fact; an unconfigured scope
   records nothing, because nothing was written.

## What a client sees when it is refused

A refusal comes back as a tool error whose text is `family: message`.

| Family | Means |
| --- | --- |
| `invalid_command` | an id the tool could not parse, or a payload it could not index |
| `unauthorized` | the caller may not read this — including an unenrolled principal asking for the policy bundle |
| `scope_hidden` | the target's existence is not disclosed to this caller |
| `enrollment` | the write gate: unenrolled, or the call's tenant is not the principal's |
| `quota_unconfigured` | the write gate has no quota for this principal, and refuses fail-closed |
| `quota_exceeded` | the principal's write quota is exhausted; the denial is recorded |
| `handler:<code>` | the domain handler refused, carrying its own reason code (see [Errors and reason codes](errors.md)) |
| `internal` | a storage failure |

A malformed `principal` is refused earlier still, as an MCP invalid-parameters
error rather than a tool result.

## What ADR-024 names and this surface does not expose

- **`ask_network`** — named in the ADR's vocabulary and **not implemented**. It
  appears in no crate.
- **MCP resources** — the ADR names thread timelines, evidence and authorized
  policy sets as read *resources*. None is exposed; the equivalent reads are
  available as tools only.
- **The listen stream** has its own chapter: [The MCP listen gateway](mcp-listen.md).

## Connecting a client: `rb-mcp` over stdio

The tools are served by **`rb-mcp`**, a binary that speaks MCP over its own
standard input and output. A client spawns it, and the process lives for that
one session.

```bash
DATABASE_URL=postgres://localhost/reasonbraid ./target/release/rb-mcp
```

Most MCP clients take that as a command plus an environment, for example:

```json
{
  "command": "/path/to/reasonbraid/target/release/rb-mcp",
  "env": { "DATABASE_URL": "postgres://localhost/reasonbraid" }
}
```

On connect the server answers `initialize` as `reasonbraid`, advertises the
`tools` capability, and negotiates protocol version **`2025-11-25`** — the
latest `rmcp 3.2.0` offers. The six tools above are what `tools/list` returns.

⛔ **It opens no socket, and that is the point.** A stdio server accepts no
inbound connection: the client owns the process and its lifetime, so serving the
tools does not touch the Internet-exposure question that
[Blockers](blockers.md) tracks. Closing the client's end of the pipe ends the
session.

⚠️ **The identity rides each tool call, not the transport.** stdio carries no
header, so every tool takes the caller's `principal` as an argument — the same
value the HTTP surface takes in its header — and the tool's own authorization
decides the answer. Spawning `rb-mcp` grants nothing by itself.

⚠️ **The database connection is lazy.** The process starts, handshakes and lists
its tools without touching PostgreSQL; a tool that needs the database reports a
connection failure as that tool's own error. A client can therefore discover the
surface even while the deployment is down.

### What is not served

⛔ **There is no HTTP transport.** The Streamable-HTTP server profile was priced
and deferred: it adds three crates, one of which is a second major of `base64`,
and this workspace's supply-chain policy denies duplicate versions outright.
Taking it means resolving that split deliberately rather than silently —
`docs/decisions/2026-09-20_the-mcp-server-transport-is-stdio-first.md` records
the measurement and what a later leaf owes.

⛔ **ReasonBraid is an MCP server here, not an MCP client.** Consuming an
upstream MCP server's listen stream through the SDK is a separate dependency and
a separate decision; see [The MCP listen gateway](mcp-listen.md) for what that
surface does and does not claim.
