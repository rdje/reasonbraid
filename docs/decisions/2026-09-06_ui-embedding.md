# 2026-09-06_ui-embedding.md

## Context

`.1.6.2` (the static shell, per the `ui-direction` decision): the page
(`web/index.html`, `web/app.js`, `web/style.css`) must reach the browser
without a build pipeline and without adding a deployment artifact to the
single-binary `rb-server`.

## Decision

- **Embed at compile time.** The assets live in `crates/reasonbraid-server/web/`
  and are compiled into the binary via `include_str!` — served by a state-free
  `ui_router` at `/`, `/app.js`, `/style.css`. One binary, no runtime paths
  (the repo can move anywhere — §12), no filesystem reads, no new artifact
  class, no frontend toolchain. The page sits in the server crate because it
  is a server asset, not a separate application.
- **The page is a client like any other.** It performs same-origin GETs with
  the dev-profile `x-reasonbraid-principal` header and the `tenant_id` query —
  the server's `thread_inspect`/`tenant_admin` gates and audit rows apply
  unchanged. The `ui_router` adds NO API route, NO grant, NO write path.
- **Read-only and text-safe by construction, enforced mechanically.** The
  offline tests assert: the page references ONLY the documented GET surfaces;
  it names no write verb; it never assembles HTML from data (the app does not
  even name the API that would allow it); and the three routes serve with
  typed content types.
- Identity inputs (principal + tenant) persist in localStorage only; nothing
  is sent anywhere but the same-origin server.

## Consequences

- The `.1.6.3` demo beat can assert the shell without a browser: curl the
  page, grep `app.js` for the exact endpoint paths, and fetch one live view
  with the dev header — the same data the page renders.
- A future richer UI (if ever) replaces `ui.rs`/`web/`, not the API; the
  read surfaces stay the contract.

answers:

- **"No build pipeline" means "no runtime file layout".** Compile-time
  embedding is the only form that keeps a single deployable binary AND
  root-relative portability; a runtime `--web-dir` would reintroduce paths
  and drift.
- **The shell must not become a second client contract.** It speaks the
  existing wire, presents the existing header, and inherits every denial —
  so nothing about the page can weaken authorization.
