# 2026-09-06_ui-direction.md

## Context

`.1.6` (backlog 18: basic Web UI/CLI for threads, nodes, inbox, budgets, audit
timeline). The director adopted the recommendation (2026-09-06): the UI is a
vanilla static page served by `rb-server`.

## Decision

- **The `.1.6` Web UI is a vanilla static page served by `rb-server`** — plain
  HTML + JS, no frontend build pipeline, no framework, no new artifact class.
  It mirrors the existing read surfaces (`GET /v1/threads`, `/v1/threads/{id}`,
  `/v1/nodes/inbox`, `/v1/nodes/presence`, budgets, the audit timeline) and
  keeps the CLI the primary, fully-covering surface.
- **No build pipeline** — the §2.2 "operational simplicity first" principle:
  the page ships as static files the server serves; a TypeScript toolchain is
  explicitly rejected for this slice (roadmap §7.1 allows TS, but nothing in
  backlog 18 needs it; revisit only with a measured need).

## Consequences

- The decomposition of `.1.6` splits at the static-shell seam (server-served
  page + the read endpoints it consumes) versus any new operator surface the
  gap census finds; the acceptance is "every inspection happens through the
  supported UI/CLI, not database surgery" (`ROADMAP.md` §26.1).

answers:

- **A read-only dashboard needs no build step.** The UI's only job is to render
  the inspection surfaces the API already serves; a static page keeps the
  deployment a single binary and the maintenance footprint zero.
- **The CLI stays the primary surface.** The UI is a convenience window over
  the same endpoints — it adds no new authority or write path, so the
  authorization/audit semantics are inherited, not re-implemented.
