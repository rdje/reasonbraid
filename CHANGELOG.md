# CHANGELOG.md

> Entries older than this session's `.4` lane (the R0–R2 pack history) are
> rotated into the git history (the README-STABILITY rotation threshold) —
> `git log --follow CHANGELOG.md` carries the full record.

# CHANGELOG.md

> Entries older than `2026-09-06` are rotated into the git history (the
> README-STABILITY rotation threshold) — `git log --follow CHANGELOG.md`
> carries the full record.

# CHANGELOG.md

## 2026-09-07 — The highest-risk lane is wired, and the gate is structural (`PHASE-4.5.3`)

- `resolvers.rs`: the startup sync (`sync_gated_entries`) — opening registers the R3/R5/RX rows, closing REMOVES them; the disabled pack has no row, so the resolve can never return it. The auth filter routes credential-carrying references to the `credential` class only.
- `fetcher.rs` + `browse.rs` + `broker.rs`: the per-request authenticated fetch (the credential attaches for THAT acquisition only), the render pre-flight + spawner, the disclosure-bearing `AuthenticatedReceipt` and the network-log `BrowserReceipt`.
- The handler's R5/R3/RX branches run behind the enabled belt; the binary syncs the gate at startup (`RB_ENABLE_R5R3RX`, OFF by default).
- Measured: profiles 18 — closed → the unresolvable-now; open → the authenticated loopback refusal names the class (the SSRF proof through the authenticated path), the render pre-flight refuses before any spawn, the §12.8 capability call publishes; closed again → the rows are gone. **`.5` COMPLETE (the gated lane)** — frontier → `.6` (the snapshots + derivation-graph lane).

## 2026-09-07 — The highest-risk lane's machinery ships, compiled but unwired (`PHASE-4.5.2`)

- `crates/reasonbraid-browse`: the R3 browser worker — the stdio protocol, the bounded interaction (navigate/click/scroll/type + the step budget + the wall-clock ceiling), the network-log disclosure, the provenance-named browser startup check; two tests against the REAL Chrome (the local render + the step-budget refusal before any navigation).
- `src/broker.rs`: the R5 credential broker — the opaque binding ref, the per-request attach, the REDACTED Debug (the value never logs), the `DisclosureRecord`.
- `src/mediated.rs`: the typed §12.8 vocabulary — the six response shapes, the not-inspected-original record, the second-verifier rule.
- Compiled but UNWIRED — the gate is the `.5.3` wiring's. Frontier → `.5.3`.

## 2026-09-07 — The highest-risk lane's contracts are decided: disclosed, contained, and off by default (`PHASE-4.5.1`)

- R5: the LOCAL credential broker — the opaque binding ref, the per-request delegated session, the explicit-disclosure receipt (a credential is a disclosure, not a permission).
- R3: the bounded browser — the step + network-log budgets, the killing worker, and the deployment-checked `vm_container` requirement (the gate refuses to open without it); the census measured chromiumoxide 0.9.1 over headless_chrome 1.0.22; the engine is a pinned, provenance-named chromium with a startup version check.
- RX: the typed §12.8 vocabulary (the six response shapes, the not-inspected-original record, the second-verifier rule).
- The OPT-IN gate: compiled but DISABLED; the enablement is a named recorded change; the resolve never returns a disabled pack.
- Durable in `docs/decisions/2026-09-07_r5r3rx-contracts-opt-in.md` (top-level `answers:`). No code. Frontier → `.5.2` (the gated machinery).

## 2026-09-07 — The R3/R5/RX lane opens: the census found NOTHING exists (`PHASE-4.5`)

- No browser/MCP crate in the lock; the credential surface is the `.1.2` opaque binding-ref plus the fetcher's no-ambient-credentials baseline; §12.8 (the agent-mediated vocabulary) has no machinery.
- Decomposed at the census seams: `.5.1` the three contracts + the OPT-IN gate (default OFF — the packs ship compiled but disabled) → `.5.2` the machinery (gated) → `.5.3` the receipt + the wiring. Frontier → `.5.1`.

## 2026-09-07 — Pack R2 is complete: the pipeline, the receipt, and the media-type routing (`PHASE-4.4.3`)

- `migrations/0027`: the R2 install record (`r2-extract-worker` — the extraction media types, egress `listed` + sandbox `process` — the first ladder-up, the kill-on-budget-trip evidence).
- `src/extraction.rs`: the `ExtractionReceipt` (the Derivation edge — the parent digest, the derived chunk digests, the extractor version, the excluded list) and the spawner (ONE request line, ONE response line, the time budget KILLS the worker).
- `resolvers.rs` + `api.rs`: the resolve's media-type filter (hinted references rank the extraction pack; hintless ones keep the acquisition-only path) and the handler's pipeline (the R0 acquisition under the `.2.1` policy, then the killing-budget worker).
- Measured: profiles 17 — the hinted reference pipelines, the loopback refusal names the class through the resolution path. **`.4` COMPLETE (pack R2)** — frontier → `.5` (the opt-in private/authenticated connectors — the highest-risk lane).

## 2026-09-07 — The extraction worker ships: the stdio quarantine parses the four formats (`PHASE-4.4.2`)

- `crates/reasonbraid-extract` (the new workspace crate): ONE JSON request in, ONE response out, exit — the fresh process IS the quarantine. The per-format parsers (the PDF text layer, the one-level zip/tar archives, the Atom/RSS feeds) derive the chunks (each with its own ADR-011 digest + the parent digest), and the refusal list is mechanical and named (encrypted/JS PDFs, nested archives, traversal, the ratio brake over the compressed envelope, the ceilings).
- Nine tests including two stdio roundtrips spawning the built binary. Frontier → `.4.3` (the receipt + the R2 pack wiring).

## 2026-09-07 — The R2 contract is decided: extraction is a Derivation, parsed in a worker quarantine (`PHASE-4.4.1`)

- The parser census, measured: lopdf 0.44.0 (chosen) vs pdf 0.10.0 (rejected as the lower-level API), zip 8.6.0, tar 0.4.46, atom_syndication 0.12.10 — all pure Rust.
- The contract (`docs/decisions/2026-09-07_r2-extraction-contract.md`, top-level `answers:`): the extraction always produces a Derivation (parent digest + extractor version + derived chunk digests); the parsers run in `process`-class worker processes — the first ladder-up, the stdio quarantine with killing budgets; the named refusals (encrypted/JS PDFs, nested archives, traversal, bombs); the media-type routing (hinted references pipeline acquire→extract).
- No code. Frontier → `.4.2` (the extraction workers).
