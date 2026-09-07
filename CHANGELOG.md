# CHANGELOG.md

> Entries older than this session's `.4` lane (the R0–R2 pack history) are
> rotated into the git history (the README-STABILITY rotation threshold) —
> `git log --follow CHANGELOG.md` carries the full record.

# CHANGELOG.md

> Entries older than `2026-09-06` are rotated into the git history (the
> README-STABILITY rotation threshold) — `git log --follow CHANGELOG.md`
> carries the full record.

# CHANGELOG.md

## 2026-09-07 — Phase 5 opens: the census found the workflow profile is an unvalidated string (`PHASE-5.1`)

- The CLI passes `workflow_profile` through to the thread body; `threads.rs` stores it verbatim — no DSL, no validation, no step composition, and ADR-016 is unopened.
- Reusable: the Phase-1 state machines, the typed contributions, the Phase-2 budgets, the Phase-4 evidence pipeline (the `evidence_review` profile's substrate).
- Decomposed: `.1.1` ADR-016 + the census → `.1.2` the profile registry + the validation → `.1.3` the profile-driven execution. Frontier → `.1.1`.

## 2026-09-07 — Phase 4 is closed: the G4 gate is Met (`PHASE-4.7.2`)

- The gate package: the evidence manifest (every G4 clause → a re-runnable artifact), the gate record (**Met**, five named deferrals, top-level `answers:`), the subtraction record (the §20.6 rows 31–35 shipped + the deferrals — no empty lists).
- The supply-chain re-run: `make deny` rc=0 (the R2/R3 duplicate families reviewed + skipped with the rationale; the uluru MPL-2.0 exception narrowed) + `make secret-scan` rc=0 (163 commits, no leaks).
- The tree flips `done`; the frontier moves to `PHASE-5.1`; the book's roadmap chapter reflects the completion.

## 2026-09-07 — The G4 hostile suite ships: eight refusal scenarios, one gate-citable test (`PHASE-4.7.1`)

- `profiles 23` (`the_g4_hostile_suite_names_every_refusal`): the loopback/private/mapped-form refusals through the resolution path, the userinfo refusal, the unsupported scheme's unresolvable-now, the fake digest 400, the unknown assessment kind, the forged-field 422 — each names its reason.
- The worker-side hostile cases ride the extract crate's nine offline refusals (the bomb, the traversal, the encrypted/JS PDFs). Frontier → `.7.2` (the G4 gate record + the subtraction record).

## 2026-09-07 — The G4 exit opens: the refusals exist, the consolidated proof does not (`PHASE-4.7`)

- The explicit-failure machinery is measured per-lane (the refusal matrices, the budget trips, the fake digest/excerpt refusals, the unresolvable-now) — but the G4 gate has no consolidated suite and no subtraction record.
- Decomposed: `.7.1` the hostile-content suite (ONE gate-citable test result) → `.7.2` the G4 gate record + the subtraction record. Frontier → `.7.1`.

## 2026-09-07 — The evidence pipeline is complete: the retention enforces, the freshness surfaces (`PHASE-4.6.4`)

- `migrations/0031`: the `license`, `fresh_until`, `refreshed_at` columns.
- `src/snapshots.rs`: the retention TTLs (the audit class never expires — binding decisions stay addressable), the `expire_due` enforcement (the tombstone rides the class's TTL), the `stale` surface, and the re-fetch policy (the replay refreshes the freshness).
- The verbs (`POST /v1/snapshots/expire-due` with the `at` override, `GET /v1/snapshots/stale`). Measured: profiles 22. **`.6` COMPLETE** — frontier → `.7` (the G4 hostile-content suite).

## 2026-09-07 — The claim-evidence graph ships: the citation is validated, not asserted (`PHASE-4.6.3`)

- `migrations/0030`: the assessment edges — the five kinds (the CHECK constraint), the author/verifier, the excerpt + selector, the rationale, the authority/freshness/independence/uncertainty, the replay index.
- `src/claims.rs`: the typed submission + the CITATION VALIDATION — the excerpt MUST appear in the snapshot's raw bytes (the fake excerpt is refused; citation existence alone never satisfies an evidence gate) — plus the two read surfaces.
- Measured: profiles 21 — the true excerpt accepts + replays, the fake excerpt refuses, the unknown kind names itself. Frontier → `.6.4` (the license/retention + the freshness).

## 2026-09-07 — The derivation graph ships: every transformation is an edge (`PHASE-4.6.2`)

- `migrations/0029`: the `Derivation` edges — the parent link, the derived kind, the content's OWN verified ADR-011 digest, the replay index.
- `src/derivations.rs`: the typed submission (the content MUST hash to the declared digest; the parent must exist; the same parent + kind + digest replays), the `children_of` traversal.
- The verbs + the R2 chunk auto-derivations (the extract chunks land as the snapshot's edges). Measured: profiles 20. Frontier → `.6.3` (the claim-evidence graph + the citation validation).

## 2026-09-07 — The snapshot store ships: the content-addressing is verified, the deletion is a tombstone (`PHASE-4.6.1`)

- `migrations/0028`: `snapshot_objects` (the bytes under their ADR-011 digest — identical bytes, one row) + `evidence_snapshots` (the §12.6 shape; the tombstone state rides the row).
- `src/snapshots.rs`: the typed submission, the VERIFIED digest (the bytes must hash to the declared one — never trusted), the replay, the tombstone (the reason + the time, idempotent).
- The verbs (`POST`/`GET`/`DELETE /v1/snapshots`) + the resolve handler's R0/R2/R5 auto-submits (the acquired bytes land with the provider receipts + the disclosure policies).
- Measured: profiles 19 — the roundtrip, the replay, the mismatch 400, the tombstone. Frontier → `.6.2` (the derivation graph).

## 2026-09-07 — The snapshots lane opens: the receipts exist, nothing persists them (`PHASE-4.6`)

- The `.2`–`.5` packs produce the ADR-011 receipt shapes; NOTHING stores them — no `EvidenceSnapshot`, no `Derivation` edges, no claim-evidence assessments, no tombstone (the object store is the Phase-4 blocker's last leg, the `.1` census's named trigger).
- Decomposed at the census seams: `.6.1` the snapshot store + the tombstone → `.6.2` the derivation graph → `.6.3` the claim-evidence graph + the citation validation → `.6.4` the license/retention + the freshness. Frontier → `.6.1`.

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
