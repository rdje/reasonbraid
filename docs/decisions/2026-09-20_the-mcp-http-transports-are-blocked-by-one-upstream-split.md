---
answers:
  - Why does ReasonBraid not use the MCP SDK's Streamable-HTTP transport?
  - Is the MCP listen gateway SDK-backed?
  - What would it take to enable the rmcp HTTP client or server transport?
  - Who requires base64 0.22 and who requires 0.23?
---
# The MCP HTTP transports are blocked by one upstream split, and it is not ours to resolve

- **Type:** decision
- **Status:** accepted; both halves declined with a named trigger
- **Owner:** `SIGNOFF-REPAIR.6.6`
- **Date:** 2026-09-20
- **Related:**
  `docs/decisions/2026-09-20_the-mcp-server-transport-is-stdio-first.md` (which
  found the same blocker from the server side), ADR-024, ROADMAP §9.6

## The question two leaves were asking separately

`SIGNOFF-REPAIR.6.6` asks whether the rmcp Streamable-HTTP **client** transport
should be taken, so the listen gateway's conformance is SDK-backed rather than
hand-written. `SIGNOFF-REPAIR.6.8` asked the same of the **server** transport.
They were opened as different leaves with different dependency sets.

🔴 **They have one blocker, and it is the same crate.** Measured with
`cargo metadata` over the resolved graph, each feature added in turn and the
manifest restored afterwards:

| rmcp feature | New packages |
| --- | --- |
| `transport-io` (the stdio server, taken) | **0** |
| `client` (the SDK client role alone) | **0** |
| `transport-streamable-http-client` | **2** — `base64 0.23.1`, `sse-stream 0.2.6` |
| `transport-streamable-http-client-reqwest` | **3** — the above + `wasm-streams 0.5.0` |
| `transport-streamable-http-server` | **3** — the above two + `async-trait 0.1.92` |

Every HTTP profile, client or server, brings `base64 0.23.1`. `deny.toml` sets
`multiple-versions = "deny"`, and `base64` is not in the reviewed `skip` list.

## Why this is not a matter of effort

The split is **rmcp against the rest of the tree**, and the counts are one-sided:

| Requirement | Packages |
| --- | --- |
| `base64 ^0.22` | **12** — `axum`, `reqwest 0.12`, `reqwest 0.13`, `sqlx-core`, `sqlx-mysql`, `sqlx-postgres`, `hyper-util`, `tower-http`, `gix-transport`, `chromiumoxide`, `a2a-lf`, and `reasonbraid-server` itself |
| `base64 ^0.23` | **1** — `rmcp 3.2.0` |

⛔ **So there is no move this project can make.** It cannot raise the twelve —
they are third-party pins across the HTTP, database, git and browser stacks — and
it cannot lower rmcp's. The only resolutions are upstream: the ecosystem
converging on `base64 0.23`, or `rmcp` widening its requirement to accept
`0.22`.

⭐ **That re-grades both deferrals.** They were recorded as *not yet*; they are
**blocked on a third party**, which is a different status and one a reader should
not have to infer from a dependency table.

⚠️ **The requirement is feature-gated, which is why the current build is clean.**
`base64` is an optional dependency of `rmcp`, reached through
`client-side-sse` / `server-side-http`. With the taken feature set the crate is
not in the graph at all — measured, not assumed: `transport-io` adds zero
packages.

## The decisions

**1. The Streamable-HTTP client transport is DECLINED, and the listen gateway
stays hand-written.** `SIGNOFF-REPAIR.6.2.4` proved the five-step ritual over a
real socket with dependencies already in the tree, and that evidence is
unaffected by this. What remains unavailable is the *claim* — ⛔ no document in
this repository may describe the MCP listen transport as SDK-backed or
conformance-tested, which is the limit `.6.2.4` already wrote into the book and
which this record leaves standing.

**2. The Streamable-HTTP server transport stays deferred** for the same reason,
and its record now points here rather than restating the measurement.

**3. Adding `base64` to `deny.toml`'s skip list is REFUSED.** That list is
reviewed, each entry carries the reason two majors coexist without crossing a
boundary, and its own header says to re-review when a third version appears.
`base64` is an *encoding* crate reached by the HTTP stack, the database driver
and the git transport alike; waving a duplicate of it through to enable a
transport nobody is yet allowed to expose is the exact trade the policy exists
to force a decision about — and the decision is no.

## The trigger

⭐ **This is a wait, not a task, and it has a mechanical check.** A future leaf
re-opens it when either resolution lands:

```bash
# Does the HTTP client transport still split base64?
python3 -B scripts/project_env.py cargo metadata --format-version 1 | \
  python3 -c 'import json,sys; print(sorted({(p["name"],p["version"]) for p in json.load(sys.stdin)["packages"] if p["name"]=="base64"}))'
```

One version means the split has closed and both transports become an ordinary
feature decision again. ⚠️ Until then, neither is a slice anybody can work, and
opening a leaf that waits on a third party would put an unworkable item on a
frontier — the shape `SIGNOFF-REPAIR.11.26`'s residue already occupies once.
