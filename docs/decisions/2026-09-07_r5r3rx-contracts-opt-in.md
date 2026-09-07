# 2026-09-07_r5r3rx-contracts-opt-in.md

## Context

`PHASE-4.5` (the R5 credential broker, the R3 browser, the RX agent-mediated
acquisition — the HIGHEST-RISK lane, "do not enable by default") decomposed
at the census seams: NOTHING exists — no browser/MCP crate in the lock, only
the `.1.2` opaque `credential_binding_ref` and the fetcher's
no-ambient-credentials baseline. The §12.3 R3/R5/RX rows + §12.8 need the
typed contracts + the enablement gate before any machinery (`.5.2`) exists.

## Decision

### R5 — the credential broker (local, opaque, disclosed)

- The broker is LOCAL: credentials never enter a reference (the
  `credential_binding_ref` stays the opaque handle), never log, never
  persist outside the broker's own store (the OS keychain, at rest).
- The delegated session: the broker resolves the binding ref AT THE REQUEST
  BOUNDARY into the transport `Authorization` (or cookie) for THAT
  acquisition only — the fetcher's no-ambient-credentials baseline is
  preserved (the credential is per-request and explicit, never ambient).
- The explicit-disclosure rule: every authenticated acquisition records
  WHAT was disclosed (the credential class — e.g. `github-token-read` —
  the target host, the time) in the receipt. Never silent.

### R3 — the browser (bounded, logged, contained)

- The bounded interaction: the step budget (navigate/click/type/scroll),
  the network log (every request the page makes is recorded — the
  disclosure), the rendering policy (downloads and file dialogs denied),
  page JavaScript allowed but the budget KILLS the worker on the trip (the
  R2 killing-budget pattern).
- The isolation claim is the ladder's TOP: R3 enablement REQUIRES a
  `vm_container` runtime (the gate refuses to open otherwise) — the honest
  claim, deployment-checked, never a downgrade. The browser runs as a
  subprocess inside that boundary.
- The browser-runtime census (measured `2026-09-07`): `cargo add --dry-run`
  → chromiumoxide 0.9.1 (the pure-Rust ASYNC CDP client — chosen) vs
  headless_chrome 1.0.22 (the older sync client). The ENGINE binary is a
  pinned chromium — NOT vendored (the pure-Rust doctrine has no browser
  engine); its provenance is named (the OS package or a pinned release
  URL) and the worker verifies the version at startup. The binary's
  provenance is the residual the threat model carries.

### RX — the §12.8 vocabulary (agent-mediated acquisition)

- The acquisition-call response shapes are a TYPED vocabulary:
  `immutable_snapshot`, `minimal_excerpt`, `structured_fact` (with
  provenance), `redacted_derivative`, `test_receipt`, `refusal` (with the
  limitation). The enrolled agent answers with one of these — never an
  unstructured blob.
- The not-inspected-original record: the network records that other
  participants may NOT have inspected the original (the disclosure flag on
  the receipt).
- The second-verifier rule: a local claim can require a second authorized
  verifier without the raw private source leaving its host.

### The OPT-IN gate (the lane's own doctrine)

- The packs ship COMPILED but DISABLED. The enablement is a named
  configuration change (the server config + the environment), recorded in
  the server's startup facts; the registry entries exist ONLY when the
  gate is open; the resolve never returns a disabled pack; the default is
  OFF everywhere — dev, CI, the demo.

## Consequences

- The `.5.2` machinery and the `.5.3` receipts implement these contracts
  verbatim; a deviation is a contract change.
- R3's `vm_container` requirement means the browser pack stays DISABLED in
  every current deployment profile — the gate makes the honest claim
  mechanical, not a promise in a doc.

answers:

- **A credential is a disclosure, not a permission.** The broker's job is
  to make the delegated session EXPLICIT: the binding ref stays opaque, the
  credential attaches per-request, and the receipt records the disclosure
  class — the silent ambient credential is the attack the contract forbids.
- **The browser's isolation claim is deployment-checked, not declared.**
  The pack refuses to open without a real container boundary — a claimed
  `vm_container` that the deployment cannot verify is the overclaim the
  ADR-018 ladder exists to prevent.
- **The opt-in gate is the highest-risk lane's only safe default.** The
  machinery ships compiled; the gate is a configuration CHANGE, recorded —
  "off by default" is a state the resolve path checks, not a note.
- **The pure-Rust doctrine ends at the browser engine.** None exists; the
  honest answer is a pinned chromium with named provenance + a startup
  version check — the binary is the residual the threat model carries.
