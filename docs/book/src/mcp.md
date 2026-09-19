# The MCP surface

ReasonBraid exposes its verbs as MCP tools. The rule the whole surface is built
on (ADR-024) is that **a tool is the HTTP handler's re-expression, never a new
authority path** — so a tool that no handler backs is not exposed, and a tool
that is exposed runs the handler's own authorization by *calling* it.

⛔ **Read the reachability section at the end before planning against this
chapter.** The tools are implemented and tested; nothing serves them to an MCP
client yet.

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

## Reachability — no MCP client can reach this today

⛔ **Nothing serves these tools.** `reasonbraid-mcp` builds as a library, no
other crate depends on it, it declares no binary, and the tool router is
constructed only inside the crate's own `#[cfg(test)]` module. There is no
stdio transport, no HTTP transport, and no `rb-server` wiring.

So the six tools above run under `cargo test` and nowhere else. They are
correct, they are exercised against a live database, and they are **not a
feature an operator can switch on**. Treat this chapter as the contract the
transport will be built against.
