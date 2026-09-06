# 2026-09-07_deployment-packaging.md

## Context

`.1.7.2` (the ROADMAP §20.3 build bullet "local/LAN deployment packaging"): the
census found no `deploy/` directory, no release build target, and no runbook —
while the surfaces for a LAN deployment already existed (config-free bind flags,
compile-time-embedded migrations + console, the demo's ssh two-host mode) and
nothing exercised release binaries.

## Decision

- **The Phase 1 package is four self-contained binaries + a runbook.** `make
  release` builds `rb`, `rb-server`, `rb-node`, `rb-journal`; migrations and
  the console embed at compile time, so a deployed binary needs no runtime
  path back to the checkout (§12). `deploy/README.md` is the operator
  surface; the book's `deployment` chapter is the product view.
- **Phase 1 ships two of the §6.6 profiles.** Developer (loopback, `make
  dev` — the `.1.7.1` loop) and Trusted LAN (enrolled nodes, plain HTTP, dev
  trust store). Public exposure remains the G6 gate, not a flag.
- **The packaging claim is verified by running the package.** The two-host
  demo gained a `--release` switch (one flag selects the build root — the
  script already had a build-root seam); the acceptance evidence is the demo
  passing 24/24 on `target/release` binaries. `make demo` stays debug (the
  standard guard exercises the default path).
- **Flags, not config files.** `rb-server --host/--port/--database-url` and
  the node's `--server` cover the Phase 1 surface; process supervision,
  containers, TLS, and PG provisioning automation are Phase 2 ops.

## Consequences

- The runbook's subtraction record names the deferred items with their
  revisit triggers (config files, launchd/systemd units, TLS, containers, PG
  automation, Internet exposure).
- A future profile (overlay/Internet) extends `deploy/` + the book chapter;
  the four-binary package and the release-built demo proof remain the
  verification pattern.

answers:

- **Prove the artifact, not only the behavior.** Debug builds prove the code;
  the release-built demo proves the shipped package — a one-flag build-root
  switch is the cheapest honest packaging verification, and it keeps the
  default guard on the fast path.
- **A runbook without a practiced path is fiction.** The demo's ssh
  two-host mode existed before the runbook; the runbook's job is to route
  operators to what is already repeatable, and to state the honest limits
  (dev trust store, plain HTTP) at the point of use.
- **Self-containment is the packaging invariant.** Everything a deployed
  binary needs ships inside it (migrations, console); anything outside
  (PostgreSQL, the node's journal volume) is explicitly operator-owned.
