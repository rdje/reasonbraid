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
| Node inbox (tenant_admin): each row's command, delivery state, the result's refusal if the control plane refused it, and any quarantine | `GET /v1/nodes/inbox?node_id=…&tenant_id=…` |

⚠️ **The inbox panel did not work until `SIGNOFF-REPAIR.4.4.2.2`.** It sent
`?node=`, but the route requires `node_id`, so every request was refused. It
also displayed `state`, a field no inbox row has ever carried (the field is
`delivery_state`). This page documented the broken parameter as well. The
console's check now parses the panel's query with the route's own extractor,
and compares every field the panel reads with the fields the server writes, so
the panel and the server cannot drift apart unnoticed again.

⚠️ **The Timeline did not render until `SIGNOFF-REPAIR.11.1.1`.** For every
thread with at least one event it showed only this:

```text
HTTP 0: client error: TypeError: Failed to execute 'appendChild' on 'Node':
parameter 1 is not of type 'Node'.
```

The event's version number went into a table cell as a number, and the page's
rendering helper passed anything that was not a string to the browser as if it
were a page element. Every check the console had read `app.js` as text, and
none of them ran it, so nothing noticed. The helper now turns any value that is
not already a page element into text: a number or `true`/`false` as written, a
structured value as its JSON. A browser control now runs the page (see
[How the console is checked](#how-the-console-is-checked)).

The four thread reads give **one answer** for a thread the caller cannot see:
`404 scope_hidden`, whether the id belongs to another tenant or to no thread at
all (`SIGNOFF-REPAIR.17`). Until that repair the timeline answered an absent
thread with `200 {"events": []}` and the audit view with the caller's own
inspection records, while the thread and budget views answered `404` — one
question, two answers. Nothing leaked, since no other tenant's data appeared,
but a reader could tell "no such thread" from "not yours" by which view they
asked. See [`scope_hidden`](errors.md).

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
- **Text-safe rendering.** Every datum renders as text; HTML is never
  assembled from thread content or evidence text, which stays inert untrusted
  data. The offline test suite enforces this mechanically (the page references
  only the documented GET surfaces, names no write verb, and never assembles
  HTML from data), and the browser control below checks it in a real browser.
- **What you see is what you last asked for.** Each view draws into a space
  of its own that replaces the previous view the moment you click, so its
  heading shows while it loads. An answer that arrives after you moved on (to
  another view, another thread, or another identity) is dropped instead of
  drawn. The presence and inbox panels keep only the answer to your latest
  click. Until `SIGNOFF-REPAIR.11.1.2` a slow answer was drawn wherever you
  were by then: clicking Timeline and then Audit on a slow server could put
  the timeline's rows under the Audit heading, and saving a new identity could
  show the previous tenant's threads under it.
- **Dev-profile trust.** The header is trusted (the Phase 0/1 dev stance) —
  the page adds nothing on top of it; workload identity is Phase 2 (ADR-006/
  ADR-007).

Try it: start the server (`make demo` does, or
`rb-server --database-url …`), open `http://127.0.0.1:4310/`, enter the
principal and tenant ids you enrolled, and click **Load**.

## How the console is checked

Two kinds of check, because they catch different things:

- **Reading the page's code** (`crates/reasonbraid-server/src/ui.rs`, offline):
  the page names only the documented read routes, never writes `innerHTML`,
  never sends a write, and the inbox panel's query and fields match the
  server's own types.
- **Running the page in a real browser**
  (`crates/reasonbraid-server/tests/console_browser.rs`): the real API and
  console run over a throwaway PostgreSQL, and the pinned Chrome for Testing
  build walks through the page the way an operator does: type the principal
  and tenant, click **Load**, open the thread, click **Timeline**, then
  **Audit**. Each view is compared cell by cell with what the server returned
  for the same read. A second run puts markup in a thread's subject and
  objective, for example `<img src=x onerror=…>`, and checks that the Threads,
  Thread and Timeline views show it as text, build no element from it, and run
  none of it. A third hands the page's rendering helper every kind of value
  (numbers, `true`/`false`, objects, lists, text that looks like HTML) and
  checks that each becomes plain text.
- **Late answers** (same file). The test server can hold back one chosen
  request until the test releases it, so a slow answer is made on demand
  rather than hoped for. Four runs use it: the Timeline held while the
  operator opens the Audit; tenant A's thread list held while the operator
  saves tenant B's identity; and, in the presence and inbox panels, a first
  check held while a second one answers. Each releases the held answer, waits
  until the page has received it, and checks that nothing on screen changed.
  The inbox run also checks that a queued command shows with the delivery
  state the server reports.

Run the browser check locally with:

```bash
python3 -B scripts/ci_browser.py -- bash scripts/run_pg_tests.sh console_browser
```

`ci_browser.py` downloads and verifies the pinned browser; `run_pg_tests.sh`
starts a throwaway database. Without the browser the check skips and prints a
`SKIP (R3_BROWSER_BIN unset)` line even when the run passes; without a database
it skips as every database suite does. In CI it runs in the `pg-tests` job, the one with a database, which
runs its suites through `ci_browser.py` for this reason. The browser is kept
off the network: no hostname resolves, and the console is reached by address
(`127.0.0.1`).
