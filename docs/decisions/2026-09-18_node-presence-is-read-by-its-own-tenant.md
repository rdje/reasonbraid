# Node presence is read by its own tenant, and a foreign node is not found

- Date: 2026-09-18
- Status: accepted
- Owner: `SIGNOFF-REPAIR.3.5.5` (REPAIR-0261), from the 58-route GET census `SIGNOFF-REPAIR.3.5.4`
- Related: `SIGNOFF-REPAIR.3.5.2.1` (the tenant is DERIVED from the caller, never
  named on the wire), `SIGNOFF-REPAIR.3.5.3` (the inbox read that crossed the
  boundary), `docs/book/src/node-channel.md`, `docs/book/src/web-ui.md`,
  ROADMAP §9.8 (`scope_hidden`), §16.8.

## The defect

`GET /v1/nodes/presence?node_id=…` took no `HeaderMap`. It never learned who was
calling, never asked whether they may, and selected from `node_presence` — a view
that carries `tenant_id` (migration 0017, from `nodes.tenant_id NOT NULL`) — by
`node_id` alone.

Meanwhile `GET /v1/admin/nodes/presence?tenant_id=…` gated the SAME view behind a
`tenant_admin` grant. `rb-server.rs` merges both routers into one `app` on one
plain-HTTP listener, so the tenant-wide enumeration was authorized and the
per-node lookup beside it was open.

⚠️ Measured, not inferred: against the unrepaired handler an anonymous request
returned **200**. Because that handler read no headers at all, every caller took
that same path — the anonymous measurement is the measurement for all of them.

## The decision

> **Authenticate the caller and DERIVE the tenant from it. A node in another
> tenant answers `unknown_node`, exactly as a node that does not exist does.**

Three consequences, each chosen rather than fallen into:

1. **No wire change.** A principal belongs to exactly one tenant structurally
   (`human_principals.principal_id` and `agent_roles.role_id` are primary keys
   with one `tenant_id` each), so there is no second identifier to name and none
   to bind. This is `.3.5.2.1`'s shape, reached by its argument.
2. **The refusals are typed and distinct.** No principal is `401 unauthenticated`;
   a principal enrolled in no tenant is `403 unauthorized`. The caller learns
   which of the two failed, because neither answer depends on a node existing.
3. **A foreign node and an absent node give the SAME answer.** Distinguishing
   them would keep an existence oracle — precisely what §9.8's `scope_hidden`
   exists to prevent — and this file already took that stance for the handshake
   ("or whose node has no key — the same refusal: no existence leak").

## What was rejected

- ⛔ **Leave it public and document it.** Defensible in the abstract — presence is
  an observability fact, not a credential — but it cannot be reconciled with the
  authorized sibling reading the same view. Two routes over one view must not
  disagree about who may read it, and the admin route is the one with a stated
  reason.
- ⛔ **Accept `?tenant_id=` from the caller and authorize it.** The
  `inspect_node_inbox` shape after `.3.5.3`'s repair. Rejected: it changes a
  public route's contract to obtain a tenant the server can already derive, and a
  second caller-supplied identifier is a thing that must then be BOUND — the
  family of defects this census was built to find.
- ⛔ **Refuse everyone / narrow the payload.** Rejected as a non-answer; the
  control's POSITIVE arm asserts the owning tenant still reads the node in full,
  so a repair of that shape fails.

## Cost, measured

The only caller in this repository is the web console, and `web/app.js` already
sends `x-reasonbraid-principal` on **every** GET through its single `api()`
helper — its own comment says "so the server's gates apply exactly as they do to
the CLI". They now do. No node client calls this route: `reasonbraid-node`
contains no reference to it.

Eight tests in `node_channel.rs` asserted presence anonymously and now name a
principal. ⭐ Three of them failed the first repaired run for the right reason —
their node lives in a bootstrapped admin's tenant, not the suite's seed tenant,
so the seed reader was correctly a foreign caller.
