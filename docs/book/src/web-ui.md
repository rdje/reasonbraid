# The web console

`rb-server` serves a **read-only inspection console** at `/` (`.1.6.2`): a
vanilla static page — plain HTML + JS, no framework, no frontend build
pipeline — embedded into the single binary at compile time. The CLI remains
the primary, fully-covering surface; the page is a convenience window over
the same endpoints.

## What it shows

Every view is one of the existing inspection GETs, rendered as-is:

| View | Endpoint |
| --- | --- |
| Threads (list) | `GET /v1/threads?tenant_id=…` |
| Thread (state, participants, invitations, rounds, budget dimensions) | `GET /v1/threads/{thread_id}?tenant_id=…` |
| Timeline (the ordered event log) | `GET /v1/threads/{thread_id}/events?tenant_id=…` |
| Audit (authorization records) | `GET /v1/threads/{thread_id}/audit?tenant_id=…` |
| Budget (ceiling + reservation rows, incl. denials — `.1.6.1`) | `GET /v1/threads/{thread_id}/budget?tenant_id=…` |
| Node presence (derived from the lease clock; the caller's own tenant) | `GET /v1/nodes/presence?node_id=…` |
| Node inbox (tenant_admin) | `GET /v1/nodes/inbox?node=…&tenant_id=…` |

## Identity

The page asks for the same two facts the CLI presents:

- the principal id (`hpr_…` or `rol_…`) — sent as the dev-profile
  `x-reasonbraid-principal` header;
- the tenant id (`ten_…`) — sent in the query string.

The values stay in the browser's localStorage; every request is same-origin,
so the server's authorization, audit, and visibility gates apply **exactly as
they do to the CLI**. A role without `thread_inspect` sees the same typed 403
the CLI prints; a role without `tenant_admin` cannot open the inbox view.

## Honest limits

- **Read-only by construction.** The page performs no write: no POST, no
  command envelope. Anything that mutates state happens through the CLI (or
  the API directly).
- **Text-safe rendering.** Every datum renders through `textContent`; HTML is
  never assembled from thread content or evidence text, which stays inert
  untrusted data. The offline test suite enforces this mechanically (the page
  references only the documented GET surfaces, names no write verb, and never
  assembles HTML from data).
- **Dev-profile trust.** The header is trusted (the Phase 0/1 dev stance) —
  the page adds nothing on top of it; workload identity is Phase 2 (ADR-006/
  ADR-007).

Try it: start the server (`make demo` does, or
`rb-server --database-url …`), open `http://127.0.0.1:4310/`, enter the
principal and tenant ids you enrolled, and click **Load**.
