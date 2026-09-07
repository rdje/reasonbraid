# CHANGELOG.md

## 2026-09-07 — The offline-known distinction, measured: known-and-quiet, never fabricated (`PHASE-3.2.2`)

- `GET /v1/admin/nodes/presence?tenant_id=…` (tenant_admin-gated): the operator's enumeration of every enrolled node with its derived presence state + the lease clock — the offline-KNOWN rows ("this node is known, just quiet").
- The distinction is now one measured surface (node_channel 23): the never-leased node reads `offline` with null clocks; the expired-lease node reads `offline` WITH its past expiry visible; an unknown id stays the typed `unknown_node` 404 — never a fabricated offline; the non-admin is refused 403.
- The stale handling is pinned as the distinction's third leg (the existing fencing test: an expired lease's heartbeat is refused, only a fresh handshake re-leases); the offline-delivery expiry + max age stay the `.5` lane's named deferral. Frontier → `.2.3` (the privacy-filtered directory views).

## 2026-09-07 — The presence state machine: six states, one honesty precedence (`PHASE-3.2.1`)

- `crates/reasonbraid-server/src/presence.rs`: the §10.2 six states as the deterministic `presence_state(enrolled, suspended, lease_live, concurrency)` — an unknown id is never fabricated into an offline node, suspension outranks the lease (the 0017 rule), an expired lease reads `offline` (the known-but-quiet node), a profile-declared zero concurrency reads `draining`, and `busy` is the named `.4` capacity-accounting trigger (no input feeds it yet).
- The channel's presence response gains the derived `state` (the current profile version's declared concurrency feeds the derivation); presence reads only — it never changes enrollment.
- Measured: 5 pure derivation tests + the live `offline`/`available` legs in the node_channel suite; the guard 18 live suites + demo 34/34. Frontier → `.2.2` (the offline-known distinction + the stale handling).

## 2026-09-07 — `.2` split at the census seams (`PHASE-3.2`)

- The presence-lane census mapped §10.2 (backlog 27) against the shipped surface: the lease store (0009), the presence view (0012/0017), and the channel's one-node presence endpoint exist — but the response carries only `online` + `suspended` + the clock fields (no six-state machine: `grep -rn "draining\|busy\|offline_known" crates/` → nothing), the offline-KNOWN row (an enrolled node whose lease expired) is indistinguishable from an unknown node at the directory level, and the privacy-filtered views (counts/pseudonyms/no roster per the initiator's scope) do not exist.
- Children: `.2.1` the presence state machine (the six states DERIVED) → `.2.2` the offline-known distinction + the stale handling → `.2.3` the privacy-filtered directory views. Tree-only commit; frontier → `.2.1`.

## 2026-09-07 — The per-reader visibility enforcement — a hidden field is absent, never nulled — `.1` COMPLETE (`PHASE-3.1.3`)

- `filter_profile` (in `profiles.rs`) evaluates the per-field visibility policy against the reader's class: the role itself → FULL, the tenant owner → FULL (via the audited tenant-admin check), a same-tenant principal → TENANT, any other enrolled principal → NETWORK, an unenrolled principal reads nothing. A hidden field is OMITTED from the response — absent, never nulled — and the response names the applied class (`visibility: full|tenant|network`).
- The version history stays FULL-only (past versions may carry fields later reclassified); the claim provenance rides every visible capability.
- Measured (`tests/profiles.rs`, now 5 tests): the SAME profile read by four readers yields exactly the allowed field sets each — the tenant view absents the self-only fields, the network view absents the tenant+self fields, the history refuses the stranger. **`.1` COMPLETE**; frontier → `.2` (the lease-based presence lane).

## 2026-09-07 — The directory profile lands: typed, versioned, content-addressed (`PHASE-3.1.2`)

- Migration 0019 (`agent_profiles` + `profile_versions`): one current pointer per role + the content-addressed history — every write is a new version, the SHA-256 hash is server-computed over the typed profile, the old versions stay readable, and each version records its writer (the actor handle).
- `crates/reasonbraid-server/src/profiles.rs`: the typed §10.1 `AgentProfile` (the capabilities with the four provenance classes — self-asserted/owner-attested/benchmarked/certified — the interests/scopes/availability/ceilings, the per-field visibility policy, the grants-by-reference, the incarnation lineage) with `deny_unknown_fields`.
- The verbs: `PUT /v1/profiles/{role_id}` (ONLY the role itself writes — a self-declaration), `POST /v1/profiles/{role_id}/attest` (tenant_admin-audited: the claim's provenance upgrades to `owner_attested` with the evidence), the self/owner reads + the version history.
- Measured (`tests/profiles.rs`, in the guard): identical content hashes identically, changed content versions anew, strangers (owner AND sibling) cannot write or read, a dangling lineage reference is 400, a forged field is a typed 422, the attestation is audited. The guard grew to 18 live suites; the 0019 FK ripple updated every tenant-purging purge list (the first guard run caught it). Frontier → `.1.3` (the visibility enforcement + the read surface).

## 2026-09-07 — ADR-014: deterministic eligibility first, embeddings behind their trigger (`PHASE-3.1.1`)

- ADR-014 accepted (evidence-gated): §10.3's stage-1 eligibility is STRUCTURAL by the roadmap's own table (scope, status, capability requirements, policy restrictions, ceilings, budget availability) and the shipped authority machinery evaluates it deterministically — "an ineligible role is never restored by a high semantic score" pins the ordering.
- The embedding engine arrives behind its trigger (the first recruitment run whose stage-2 ranking under-selects without the semantic feature, measured on a non-loopback corpus) and in shadow mode — never an authorization control. The `.1.2` profile schema therefore carries NO embedding columns: the versioned profiles stay interpretable typed facts.
- No code changed. Frontier → `.1.2` (the profile schema + the write surface).

## 2026-09-07 — Phase 3 opens: the directory profile lane decomposed at the census seams (`PHASE-3.1`)

- The Phase-2 close delivered the tree's blocker (stable identity, inbox, and grants), so `PHASE-3` goes `active` and its `.1` (capability/interest/visibility profiles + versioned embeddings — backlog 26) decomposes at the census seams.
- The census found NOTHING shipped for the §10.1 registration profile: no table, no write verbs, no read surface, no visibility policy — the incarnation writer (`.1.6.1`) carries only the lineage sliver (provider/model/harness/config); ADR-014 (the semantic-index engine + the embedding lifecycle) is unopened.
- Children: `.1.1` ADR-014 (the dev profile's answer + the embedding trigger) → `.1.2` the profile schema + the versioned write surface (migration 0019) → `.1.3` the per-reader visibility enforcement + the read surface. Tree-only commit; frontier → `.1.1`.

## 2026-09-07 — Phase 2 CLOSED: the subtraction record + the G6–G7 feed (`PHASE-2.7.4`)

- `docs/decisions/2026-09-07_phase2-subtraction-record.md` (`answers:`) ships the mandatory §19.8 subtraction: thirteen named not-built items (S-1…S-13 — no Internet exposure, no SSRF/prompt-injection suites, no signing pipeline, no rate-limit machinery, no hash chain, no HA, no load numbers, no production RPO/RTO…), each with the exact profile whose arrival re-opens it.
- `docs/decisions/2026-09-07_phase2-gate-feed.md` (`answers:`) turns the phase's end into a checklist: every §16.12/G7 line is mapped to its Phase-2 evidence (the adversarial suite, the restore + the replacement drill, the SLO instrumentation, the circuit breakers) or named OPEN with its re-opening profile — the hand-off the next deployment profile consumes.
- **Phase 2 is CLOSED** — the exit line's three properties are measured: authority non-escalation (the `.7.1` adversarial suite), restore + node replacement (the `.4.1` exercise + the `.7.2` drill), no false safe-retry of unknown attempts (the `.7.3` six-leg inventory). The next executable work is `PHASE-3.1`.

## 2026-09-07 — ADR-022: the audit linkage is the groundwork; the retry-safety inventory names six measured legs (`PHASE-2.7.3`)

- ADR-022 accepted (evidence-gated): the shipped audit linkage (the actor/subject/grant/digest bindings, the deterministic UUIDv5 handles, the per-thread monotonic sequence, the audit reconstruction) IS the hash chain's groundwork; the chain + checkpoint + verification policy are deferred WITH their trigger (the first non-loopback deployment / the G7 ops gate).
- `docs/decisions/2026-09-07_phase2-retry-safety-inventory.md` (`answers:`) records the §16.12 line's dev-profile proof — six measured legs (the retry decision, the dispatch gate, the quarantine gate, the replacement fence, the reboot fence, the history fence) show the machine refuses every silent re-dispatch by default; the operator's `allow_possible_duplicate` flag and the replay verb are the only human paths, by design.
- No code changed. Frontier → `.7.4` (the Phase-2 subtraction record + the G6–G7 feed) — the last leaf of Phase 2.

## 2026-09-07 — The node-replacement ritual, measured end to end — the fence held (`PHASE-2.7.2`)

- `crates/reasonbraid-server/tests/node_replacement.rs` (in the guard) runs the whole total-machine-loss ritual: the lost node's attempt lands `outcome_unknown`, the journal is destroyed, the operator revokes (the old cert is fenced, the epoch bumps), the REPLACEMENT enrolls (a new cert + a new incarnation, `replaced` in the audit), the inbox tail replays to the fresh journal — and the no-false-safe-retry fence HOLDS: the re-delivered decision is epoch-stale, the dispatch refuses fail-closed (never a silent re-dispatch of the lost node's in-flight work), the row dead-letters, the operator replays, and exactly ONE contribution lands.
- The drill exposed four machinery gaps and each fix landed: the replacement enroll path (a node whose certs are ALL revoked is a declared loss — a fresh token + secret enroll the new incarnation, with the state check BEFORE any insert), migration 0018 (0008's unconditional token UNIQUE made the first token permanent — the comment always said "one UNUSED token"; the partial index pins it), the dev-key swap (one secret per node), and migration 0017 (suspended = a revoked cert AND no active cert — the replacement reads not-suspended while the old certs stay fenced).
- The runbook's recovery section + closure tests carry the ritual (the Phase-7 game day now exercises it, no longer builds it). The guard grew to 17 live suites; 50 offline suites. Frontier → `.7.3` (ADR-022 + the no-false-safe-retry inventory).

## 2026-09-07 — The escalation surfaces are attacked, measured: the non-escalation property suite (`PHASE-2.7.1`)

- `crates/reasonbraid-server/tests/escalation.rs` stages each escalation attempt as a REAL envelope over live PostgreSQL and asserts the refusal AND its audit row: a cross-tenant attempt refuses at every boundary (create 403 + audited, thread read 404 with no existence leak, key-replay 409 with the typed idempotency conflict); a delegation scope is the ceiling even for a deputy whose OWN grant covers the target (403 + the widening invariant named); the nuclear option cannot be re-armed (identity minting under a revoked boundary is refused 400 at enrollment, no row left); a revocation fences future work but never rewrites history (the original replay returns its stored result with exactly ONE contribution event, the next command 403 + audited).
- The first live run caught four real behaviors the tests then pinned (the conflict typing, the enrollment-boundary fence, the real event-type name, the real thread-scope requirement) — the suite is measured, not argued.
- The guard's live list gained the suite — 16 live suites + the demo 34/34. Frontier → `.7.2` (the node-replacement drill).

## 2026-09-07 — `.7` split at the census seams (`PHASE-2.7`)

- The exit-lane census mapped the three properties + the gate items against the shipped surface. The non-escalation FOUNDATIONS exist (deny-by-default evaluation, the subset checker, the freeze carve-out, the four escalation-adjacent authority tests) but no adversarial property suite names the §16.12 line (`grep -rn "escalat|cross.tenant"` over the server tests → 0 matches). The restore exercise + the replay machinery exist, but the node-replacement drill is the runbook's own named gap. The no-false-safe-retry property has four measured legs (`worker_retry_policy`). ADR-022 (audit hash-chain/checkpoint and verification policy) is unopened.
- Children: `.7.1` the non-escalation property suite → `.7.2` the node-replacement drill → `.7.3` ADR-022 + the no-false-safe-retry inventory → `.7.4` the Phase-2 subtraction record + the G6–G7 feed. Tree-only commit; frontier → `.7.1`.

## 2026-09-07 — The qualification checklist is one artifact and the absent surfaces are named deferrals — `.6` COMPLETE (`PHASE-2.6.3`)

- The book's adapter-boundary chapter gains the six-box qualification checklist (the conformance-harness pass, the stub-mechanics pass, the env-gated bounded live dispatch, the credential containment, the corpus-manifest entry, the dependency-ledger row) — the §19.4 last item as ONE artifact over the live runs the qualification already follows (`RB_LIVE_CODEX=1` / `RB_LIVE_CLAUDE=1`).
- `docs/decisions/2026-09-07_phase2-adapter-conformance-deferrals.md` (`answers:`): the five §19.4 items with no machinery or no surface to bind in the dev profile — rate-limit/backoff normalization, output-size limits, tool-call validation, the provider error taxonomy, prompt/policy projection fidelity — each named with the exact trigger that re-opens it.
- **`.6` COMPLETE**: the checkable half of §19.4 is mechanical (the harness + the pinned corpus), the manual half is the book's checklist, the absent half is named. No code changed. Frontier → `.7` (the exit lane).

## 2026-09-07 — The fixture corpus is pinned: a versioned manifest with mechanical permanence (`PHASE-2.6.2`)

- `crates/reasonbraid-adapter/fixtures/MANIFEST.json` (version 1) records one entry per fixture — the file, the §19.4 conformance-item keys it proves, the adding leaf, and the reason — and the corpus drift-checks against it: `manifest_matches_the_corpus_exactly` fails on any silent add, drop, rename, or edit, so the replay oracle can only change additively with a recorded reason.
- `every_conformance_item_is_covered_by_the_manifest` holds the coverage map mechanically: every §19.4 item key is proven by at least one fixture, except the four named dev-profile deferrals (rate-limit/backoff, tool validation, projection fidelity, secret containment — the latter rides the credential-scan test). Unknown keys fail as typos.
- The real adapters replay the corpus semantics through the `.6.1` harness scenarios; the credential scan stays the mechanical gate. Measured: the adapter lib suite 9 (incl. the two new guarantees); 48 offline suites; the guard 15 live suites + demo 34/34. Frontier → `.6.3` (the qualification checklist + the named deferrals).

## 2026-09-07 — One suite, three adapters: the conformance harness lands (`PHASE-2.6.1`)

- `crates/reasonbraid-adapter/tests/conformance/` (the harness) + `tests/adapter_conformance.rs` (the registrations): every adapter — the deterministic fake and both real CLI adapters — passes the SAME six §19.4 invariants: the capability manifest agrees with the scenario's verified boundary, a never-dispatched lookup is honestly `Unsupported`, a refusal happens before any provider contact, a lost response never invents a terminal event, a cancel never exceeds the declared strength, and an empty usage receipt is `Unknown`, never zero.
- The scenario shape (name + trigger + declared capabilities + the tripping request) is the registration surface: the fake registers from its fixture corpus's OWN declarations; Codex and Claude register behind the shared provider stubs (now in `tests/conformance/stubs.rs`, one copy instead of two) plus the missing-binary refusal.
- The duplicated cross-adapter tests (the capability-boundary and unsupported-lookup copies) left the per-adapter files — those keep their provider-specific mechanics (prompt travel, stderr tails, the child kill, the receipt shapes).
- Measured: `test result: ok. 3 passed` for the harness; the adapter crate's 8 suites green; 48 offline suites; the guard 15 live suites + demo 34/34. Frontier → `.6.2` (the permanent failure-fixture corpus).

## 2026-09-07 — `.6` split at the contract seams (`PHASE-2.6`)

- The census mapped §19.4's ten conformance items against the shipped adapter surface. EXISTS: the capability declaration + unsupported-operation behavior (`AdapterCapabilities` + `StatusLookupOutcome::Unsupported`), the honest ambiguous-outcome reporting (the `outcome_unknown` contract + the lose-response fixtures), the usage accounting with confidence (`NormalizedUsage` + `UsageConfidence`), the sanitized 10-fixture corpus (credential-scan + coverage tests), the cancellation/streaming semantics, and four per-adapter test files.
- ABSENT/partial, named: the conformance suite is FOUR separate files (no single contract harness); the corpus is not pinned as a replay oracle with a §19.4 coverage map; rate-limit/backoff normalization, output-size limits, tool-call validation, the provider error taxonomy, and prompt/policy projection fidelity have no machinery (projection fidelity is Phase 6's surface); the manual qualification checklist is not one artifact.
- Children: `.6.1` the conformance harness → `.6.2` the permanent failure-fixture corpus → `.6.3` the qualification checklist + the named deferrals. Tree-only commit; frontier → `.6.1`.

## 2026-09-07 — The guard IS the population: the SLO hypotheses + the first runbook — `.5` COMPLETE (`PHASE-2.5.3`)

- `docs/decisions/2026-09-07_phase2-slo-hypotheses.md` (`answers:`) instantiates the §18.4 SLO shape from the measurements that EXIST: SLO-1…SLO-4 (the guard's live assertions, the demo's 34 checks, the restore exercise, the reconcile-after-kill beats) target 100 % with a ZERO error budget — a red pass halts the frontier (the CI policy's shape); SLO-5 records the one measured latency baseline (issuance p50 63 µs/p95 69 µs).
- Every unmeasured latency family (control-plane ingress→commit, notification promptness, model/provider, human-wait, publication, deployment convergence) is NAMED with its trigger — never given an invented number; the SLO catalogue stays separated (§5's last line).
- `docs/runbooks/node-lost-replaced.md` lands with the full §18.6 shape (detection, authority, safe first actions, diagnostic queries, containment, recovery, evidence preservation, communication, closure tests) — the closure tests name the EXISTING exercises: the demo's SIGKILL beat, the revoke beat, the replay suites, and the `.4.1` restore.
- **`.5` COMPLETE** (ADR-023 → structured logs + metrics → SLO record + runbook). No code changed. Frontier → `.6` (the adapter conformance kit).

## 2026-09-07 — The refusal paths now measure themselves: structured logs + the admin metrics surface (`PHASE-2.5.2`)

- The observability slice lands (the §18.3 minimums that apply to the dev profile): `crates/reasonbraid-server/src/telemetry.rs` (NEW) holds the process-wide in-memory registry — seven counters (`authorization_denials`, `idempotency_replays`, `handshake_refusals`, `lease_refusals`, `dead_letters`, `results_folded`, `results_rejected`) — and the `log_event!` macro (JSON lines on stderr: `level`, `event`, the correlation fields; ADR-023's redaction rules hold — no prompt text, no credentials, no secret URLs).
- The increment sites sit on the real refusal paths: the authorize deny arms + the read-gate refusals, the idempotency replay outcome, the result fold/reject tails, the dead-letter auto-quarantine, and the channel's lease/proof verifies. The rejected-result `eprintln!` becomes a structured `log_event!`.
- `GET /v1/admin/metrics` exposes the counters (JSON). The gate: the caller HOLDS `tenant_admin` in any active grant — a process-global surface has no single tenant, so the holding check replaces the per-tenant row.
- MEASURED: `bash scripts/run_pg_tests.sh` → command_api 18 (the new test denies an authorization through the REAL API, then asserts the counter DELTA matches the denied record for the actor handle AND the surface agrees); 15 live suites + the demo 34/34 (`target/pg252d_guard.log`); 47 offline suites; clippy/fmt clean. Frontier → `.5.3` (the SLO record + the runbook).

## 2026-09-07 — ADR-023: the four records are separate systems, the sink is a named trigger (`PHASE-2.5.1`)

- ADR-023 accepted (evidence-gated): the four-record separation is the SHIPPED design (the operational `eprintln!` stream and the durable audit/event tables are separate systems by construction — no log line can become an audit record, no audit row is a log). The §18.2 redaction rules pin the FUTURE sink (never prompt text, credentials, secret-bearing URLs, private evidence, or model output in span attributes; sensitive IDs tokenized at the sink boundary).
- The OpenTelemetry dependency waits for the trigger (a non-loopback deployment or the G7 ops gate) — no placeholder telemetry stack in the dev profile. No code changed. Frontier → `.5.2` (the structured-log + metrics slice).

## 2026-09-07 — `.5` split at the contract seams (`PHASE-2.5`)

- The census found UNSTRUCTURED observability: 16 `eprintln!` sites (api 8, node_channel 2, worker 6), no metrics, no traces, no SLO record, no runbook — while the four-record doctrine (§18.1) is structurally TRUE (the operational logs and the durable audit/event tables are separate systems by construction). ADR-023 is unopened.
- Children: `.5.1` ADR-023 (the four-record answer + the §18.2 redaction rules pinning the future sink) → `.5.2` the structured-log + metrics slice (JSON logs + the admin metrics surface over the §18.3 minimums that apply) → `.5.3` the SLO record + the runbook slice. Tree-only commit.

## 2026-09-07 — The absent controls are named deferrals, not placeholder infrastructure — `.4` is COMPLETE (`PHASE-2.4.3`)

- `docs/decisions/2026-09-07_phase2-inventory-deferrals.md` lands (`answers:` present): the object-store inventory (Phase 4), the canonical Git mirror (Phase 6), the signing-key recovery (the first signed release), the multi-store reconciliation (the first multi-store restore), and the backup encryption each name the exact product surface whose arrival re-opens them.
- The subtraction doctrine governs: the `.4.1` restore exercise is the reconciliation that EXISTS (it proves the database restores on every guard pass); building an inventory for an absent object store would be the lie the doctrine forbids. No code changed.
- **`.4` COMPLETE**: the restore exercise runs on every guard pass, the upgrade path is measured, the absent controls are named. Frontier → `.5` (observability + dashboards).

## 2026-09-07 — The upgrade path, measured: an existing database upgrades and its data survives (`PHASE-2.4.2`)

- The guard gains the `migration_upgrade` suite: a runtime `Migrator` applies all but the last migration to a clean schema, the REAL API seeds the tenant + boundary rows, the remaining migrations apply over the existing data, and the rows + the post-upgrade API behavior (the role enroll) survive — §17.6's upgrade-an-existing-database path, which every fresh-migration suite left unexercised, now runs on every guard pass.
- The first run caught a real sqlx fact (multi-statement prepared queries are refused — the schema reset splits). 15 live suites + the demo 34/34. Frontier → `.4.3` (the inventory-groundwork deferral record).

## 2026-09-07 — The restore is the recovery control: a backup restored on every guard run (`PHASE-2.4.1`)

- `scripts/backup.sh` (custom-format pg_dump, dated file under `target/backups/`) + `scripts/restore.sh` (pg_restore --clean --if-exists --exit-on-error into a caller-chosen isolated database) land.
- The guard gains the restore EXERCISE (§17.5's last line — a backup that has never been restored is not a recovery control): the `backup_restore` suite seeds rows, takes a real pg_dump, MUTATES the live database, restores into an isolated database (createdb → pg_restore → assert → dropdb), and asserts the pre-mutation state came back. It skips offline or without the pg tools.
- The dev-profile dump is plaintext (the encryption + key story is the `.4.3` deferral). The guard grew to 14 suites; demo 34/34. Frontier → `.4.2` (the migration upgrade test).

## 2026-09-07 — `.4` split at the contract seams (`PHASE-2.4`)

- The census found NOTHING: no backup/restore tooling (`grep -rn 'pg_dump|pg_basebackup|restore' scripts/ Makefile` → no matches), the migrations are applied to a fresh database by every suite but the upgrade-an-EXISTING-database path is never exercised, and the object/Git inventory + the key recovery + the reconciliation have nothing to bind in the dev profile (no object store, no canonical Git) — named deferrals, not build targets.
- Children: `.4.1` the backup + restore automation with the measured restore exercise → `.4.2` the migration upgrade test → `.4.3` the inventory-groundwork deferral record. Tree-only commit.

## 2026-09-07 — The usage reconciliation: the estimates-vs-receipts picture, summed over the ledger (`PHASE-2.3.3`)

- `GET /v1/admin/usage` + `rb inspect usage` land: the tenant's held (active unexpired reservations) vs settled (actual usage) vs overrun (used minus reserved per dimension, floored) vs denied (with the reasons) picture — SUMMED over the same ledger rows the budget engine enforces against, plus the per-thread breakdown. Expired holds count nowhere. tenant_admin-gated, read-only.
- The live test is MEASURED: the seeded rows' arithmetic is recomputed in the test and must match the wire (held 2 / settled 8 / overrun 3 calls / denied 1, the expired hold excluded). command_api grew to 17; demo 34/34. **`.3` COMPLETE** — frontier → `.4` (backup/PITR + migrations).

## 2026-09-07 — The spend circuit breaker: a tripped latch refuses everything new, in the denial's own transaction (`PHASE-2.3.2`)

- Migration 0016 lands the per-tenant `spend_breakers` latch (threshold + tripped state + reason). The reservation path checks it FIRST, before any ceiling math: a tripped breaker refuses every new reservation with the typed reason; an armed breaker trips the moment the tenant's recorded spend (settled usage + active holds across ALL its ceilings) plus the request crosses the declared threshold — the trip and the refusal commit with the denial's own transaction, so the latch never lags the ledger it guards.
- The admin verbs (`POST /v1/admin/breakers` arm — re-arming clears a trip — `POST /v1/admin/breakers/reset`, `GET /v1/admin/breakers`) + `rb breaker arm|reset` + `rb inspect breakers`.
- The live tests prove the loop (the crossing trips + refuses, the latch refuses while tripped, the reset re-opens, a reset breaker re-trips); budget grew to 9; demo 34/34. The first guard re-run caught the FK leak (the breaker references tenants — every tenant-purging suite's list gained the row). Frontier → `.3.3` (the usage-reconciliation surface).

## 2026-09-07 — ADR-012 + ADR-013: the ambiguity contract and the budget invariants, accepted by promotion (`PHASE-2.3.1`)

- ADR-012 accepted (evidence-gated): the provider-attempt ambiguity contract is the shipped machinery — the WP3 boundary-first journal, the WP4 prove/adjudicate exits, and the `.2.3` pure retry classes (a risky re-run requires the explicit `allow_possible_duplicate` authorization). No silent retry, structurally.
- ADR-013 accepted (evidence-gated): the budget contract is the shipped WP5 engine — reserve before dispatch at both boundaries, settle with actual usage, overruns reported never clamped, holds on indeterminate attempts. Pricing snapshots are the named Phase-4+ trigger (the invariants stay pinned when they land).
- Both promote the existing decision records; no code changed. Frontier → `.3.2` (the spend circuit breakers).

## 2026-09-07 — `.3` split at the contract seams (`PHASE-2.3`)

- The census found the machinery largely shipped: the provider-attempt state machine (core, deterministic `apply`), the budget settlement (actual usage recorded, overruns reported never clamped), and the ambiguity basics (`outcome_unknown` → proof/adjudication; the `.2.3` retry gate's `retry_requires_authorization`). What's open: spend CIRCUIT breakers (backlog 23 — nothing stops NEW dispatches once a tenant's spend crosses a declared threshold; the ceiling only refuses per-reservation), the usage RECONCILIATION surface (backlog 25 — the settlement records usage but nothing reconciles held vs settled vs overrun), and ADR-012/013 (unopened, though their machinery shipped — the promotion precedent).
- Children: `.3.1` ADR-012/013 accepted-with-evidence → `.3.2` the spend circuit breakers → `.3.3` the reconciliation surface. Tree-only commit.

## 2026-09-07 — The two-way quarantine: a dead letter auto-quarantines, the operator replays it (`PHASE-2.2.4`)

- The terminal refusal (the `.2.3` retry gate's `Refuse`) reports a `work_dead_lettered` event — ONCE per operation (the outgoing-events dedup), best-effort (journaled first; a failed send defers to the reconcile's re-emit). The server auto-quarantines the inbox row with the reason IN the same transaction as the receipt.
- `POST /v1/nodes/replay` + `rb node replay` reverse it: the quarantine clears, the admission decision REFRESHES (`decided_at` now + the CURRENT revocation epoch), the row re-sequences to the delivery tail, and the node's replayed redelivery refreshes the cached decision — the retry count is DECISION-scoped, so the old refusals stop counting and the replay re-arms the dispatch.
- The live end-to-end leg proves the whole loop: the always-refusing worker dead-letters after the bounded retries, the server quarantines with the terminal reason, the operator replays, the completing worker re-dispatches — exactly one contribution. node_work grew to 8; demo 34/34. **`.2` COMPLETE** — frontier → `.3` (provider-attempt state machine + usage reconciliation).

## 2026-09-07 — The retry policy: classes over facts the worker already holds (`PHASE-2.2.3`)

- The pure `retry_decision` lands in the core crate (§14.6): `None`/`prepared` re-dispatch unconditionally (the boundary was never crossed); a `failed_before_dispatch` WITHOUT a reservation is terminal (the SERVER denied the budget — a retry cannot change it); WITH a reservation it retries bounded (3 attempts); `outcome_unknown` retries only with the delivery's explicit `allow_possible_duplicate` flag — without it the refusal names §9.8's `retry_requires_authorization`; a dispatched attempt belongs to proof/adjudication; terminal states never. Five tests; the core suite is 49.
- The work payload gains the typed flag (false in the dev profile — nothing dispatches with duplicate risk); the worker's binary skip became the retry gate (payload facts parsed first, the attempt count rides the `.1.6.2` accessor, a refusal logs the reason and the journal status stays the visible fact — never silently retried).
- Four worker-level legs prove the behavior (re-dispatch reaches the adapter; the budget denial stays ONE attempt — the demo's `failed_before_dispatch=1` beat is the contract; the ambiguous refusal stays one; the authorized one re-dispatches). Demo 34/34. Frontier → `.2.4` (dead-letter/replay).

## 2026-09-07 — The lease epoch: a stale heartbeat loses the race it never knew it ran (`PHASE-2.2.2`)

- Migration 0015 adds `node_leases.lease_epoch`; every handshake bumps it with the token — fencing is now a PAIR (token + epoch), and all four fenced writes (events/ack/poll/heartbeat) carry the epoch they saw.
- The renewal race is closed structurally: `renew_lease` rides the epoch in its WHERE, so a heartbeat that verified before a concurrent handshake matches no row once the rotation lands — the stale session can neither write nor extend the lease it lost. The events transaction re-verifies the pair `FOR UPDATE` (the check-vs-commit window: a rotation between admission and apply is observed).
- CHANNEL_VERSION 5. The deterministic state-level test proves the race (rotation bumps the epoch, a renewal from the fenced epoch is refused, a stale epoch with the CURRENT token is refused); the wire test re-proves every fenced surface; node_channel grew to 22; demo 34/34. Frontier → `.2.3` (the retry policy).

## 2026-09-07 — ADR-005: the PostgreSQL queue is the event transport, accepted with evidence (`PHASE-2.2.1`)

- The queue item closes by promotion, not by experiment: the WP2 leased outbox worker IS the PostgreSQL queue (claim with a per-row fencing token, deduped delivery, acknowledge only with the current token AND a live lease — kill points 3–5 proven), ADR-004 formalized the outbox that is the queue, ADR-006 accepted the pull channel that consumes it, and two phases of delivery leaves shipped on it (dispatch, quarantine/retention, revocation, the `.1.5.2` decision metadata, the `.1.6` run linkage).
- Subtraction stands: no NATS/JetStream, no second durability store. The broker revisit trigger is named (measured fan-out/push/replication need, with numbers). No code changed. Frontier → `.2.2` (lease/fencing hardening).

## 2026-09-07 — `.2` split at the contract seams (`PHASE-2.2`)

- The census found the Phase-1 machinery in place (the `.1.2.2` lease/fencing: 60s TTL, per-handshake token rotation, refused stale tokens; the `.1.2.3` quarantine/prune: operator-driven, reason-stored) while three contracts are open: a capability-aware RETRY policy (§14.6's provider-accepted-but-unproven class — only the reason code exists), a dead-letter/replay surface (quarantine is one-way today: nothing auto-quarantines after N refusals, nothing re-delivers), and ADR-005 (unopened — the Phase-0 `.2.2` outbox worker + the channel decisions are its evidence).
- Children: `.2.1` ADR-005 accepted-with-evidence → `.2.2` lease/fencing hardening (the renewal race, the lease epoch, the check-vs-commit window) → `.2.3` the retry policy → `.2.4` dead-letter/replay. Tree-only commit.

## 2026-09-07 — The run writer: the result receipt closes deferral #4 — `.1` is COMPLETE (`PHASE-2.1.6.2`)

- Migration 0014 adds `runs.attempt_id`; the result fold writes the run row **after the idempotency claim** — one result = one run, ever: a redelivered result replays the original application and writes no second run (the live test's two duplicate transports prove the count stays 1).
- The run links the result payload's attempt id to the role's CURRENT incarnation (`valid_to IS NULL`, latest `valid_from`); a result without an attempt id or an incarnation still folds (the linkage is best-effort, not a gate). The tenant_admin inspection (`GET /v1/admin/runs` + `rb inspect runs`) shows the run → attempt → incarnation → role chain.
- **`.1.6` complete — the Phase-1 gate-record deferral #4 closes — `.1` is COMPLETE**: all six census gaps from the pickup note are closed (cert lifecycle, revocation write paths, delegation, cached decisions, incarnation/run writers, the ledger identity row). The demo is at 34 checks. Frontier → `.2` (production-grade leases/fencing, retry, dead-letter).

## 2026-09-07 — The incarnation writer: enrollment stops discarding the §8.1 facts (`PHASE-2.1.6.1`)

- Deferral #4's first half closes: the enroll request gains the §8.1 facts the node KNOWS at start (`provider`/`model`/`harness`/`config` — all optional, `deny_unknown_fields` keeps the wire strict), and the enroll transaction writes the `incarnations` row — but only when the node id is the agent ROLE wire id it serves (the dev wiring; a plain `nod_…` node serves no role and records no incarnation, per the hierarchy's `role_id IS NOT NULL`).
- The response returns the `incarnation_id`; `rb-node` gains `--provider`/`--model`/`--harness`/`--config`; the tenant_admin surface inspects (`GET /v1/admin/incarnations` + `rb inspect incarnations`).
- No duplication is structural, not guarded: the one-token-per-node index makes a second token unissuable, the consumed-token reuse refuses BEFORE the writer, and rotation has no incarnation writer. The live test proves the row + facts + inspection + the refusal count; the demo gains the beat (33 checks). Frontier → `.1.6.2` (the run writer).

## 2026-09-07 — `.1.6` split at the incarnation-vs-run seam (`PHASE-2.1.6`)

- The census found the 0007 hierarchy SCHEMA-ONLY: `grep -rn 'INSERT INTO incarnations\|INSERT INTO runs' crates/` → no matches — deferral #4 ("the incarnation/run row writers are deferred to Phase 2 identity") is still open. The enroll request carries none of the §8.1 facts the node knows at start (`provider`/`model`/`harness`/`config`), and the dispatch boundary is node-local (the run row needs a server-side write keyed on the result receipt).
- Children: `.1.6.1` the incarnation writer at enrollment (the request gains the §8.1 facts, the transaction writes the row, `rb-node` gains the flags, re-enroll does not duplicate) → `.1.6.2` the run writer (a result receipt records the run linked to the incarnation + attempt). Tree-only commit.

## 2026-09-07 — The cache refuses at the dispatch boundary: a revocation stops the next dispatch, measured (`PHASE-2.1.5.2`)

- The `.1.5.1` semantics got wired to the real surfaces: migration 0013 (the per-tenant `revocation_epoch` + the inbox's decision columns), the three `.1.3` revocation writes bump the epoch **in their own transaction** (the status change and the invalidation commit together — no window where one is durable without the other), and the delivered work item carries the admission decision (`authz_ref` + `policy_digest` + `decided_at` + the epoch at decision time). The handshake/poll responses carry the tenant's CURRENT epoch; the wire growth is CHANNEL_VERSION 4.
- The node journals the cached decision (the pre-shaped `authz_ref` finally gains a value) and the worker's dispatch boundary evaluates it before any provider contact: a fresh, epoch-current cached allow dispatches; an expired, epoch-stale, denied, or ABSENT decision refuses — journaled as `failed_before_dispatch` with the reason, the adapter never invoked, never silently retried.
- **The measured acceptance leg**: the live test drives the REAL node worker — the fresh allow completes and the contribution lands; a grant revocation bumps the epoch 0→1; the NEXT dispatch, of work decided under epoch 0, is refused without a re-ask (the staleness rides the journaled evidence, no second contribution). Five offline gate tests + the live leg; the full guard green (12 suites + e2e + demo 32/32). **`.1.5` complete** — frontier → `.1.6` (the incarnation/run writers).

## 2026-09-07 — ADR-008: the cache is the admission decision, borrowed — never re-evaluated (`PHASE-2.1.5.1`)

- ADR-008 accepted (evidence-gated): the shipped in-tx evaluator stays the engine (an OPA/Cedar re-platform has no measured trigger — the comparison parks behind one), and the node caches ONLY the server's admission decisions riding its delivery — §17.1 settles the shape (the journal is explicitly not authoritative for "global grants or final decisions", so the node can never locally re-evaluate a grant).
- The spike landed the pure semantics in the core crate: `CachedDecision` + `CacheVerdict` + the `ActionClass`/`FailMode` table — a 60-second freshness TTL from `decided_at`, a per-tenant revocation epoch whose bump invalidates a fresh-looking entry, a deny that is never widened by time, and the §16.4 fail rule (irreversible + admin writes fail closed, reads fail open). Five offline tests; the core suite grew to 44.
- The verification caught a REAL drift from one leaf ago: `.1.4.2`'s envelope change never regenerated `command-envelope.schema.json` and its NO REGRESSION set never re-ran the core crate's own offline suite (where the golden-drift test lives). Regenerated here; the lesson is recorded in `docs/decisions/2026-09-07_verification-set-coverage.md`. Frontier → `.1.5.2` (the implementation).

## 2026-09-07 — `.1.5` split at the ADR-vs-implementation seam (`PHASE-2.1.5`)

- The census found the ROADMAP rule (§16.4 — cache only explicitly cacheable decisions, honor expiry + revocation freshness, per-action-class fail-open/fail-closed) with NO machinery: no decision cache in the node or server, no revocation epoch (the `.1.3` write paths bump none), and the poll payload carries no decision metadata — while the plumbing is PRE-SHAPED: the journal's `authz_ref` column exists with every writer binding `None`, and the authorization record already holds the digest/version/decided_at.
- Children: `.1.5.1` ADR-008 + the pure semantics spike (cacheable classes, freshness/expiry, the epoch invalidation, the fail-closed classifier — core-crate types, the `.1.4.1` precedent) → `.1.5.2` the implementation (the decision rides the delivery, the tenant epoch bumps on revocation, the node-side cache honors the rules). Tree-only commit.

## 2026-09-07 — Delegation rides the envelope: the dual evaluation landed (`PHASE-2.1.4.2`)

- The command envelope gained the optional `authority_context` (the ADR-009 shape — `on_behalf_of` + `purpose` + `scope`; the subject rides a plain string, the tagged-newtype wire fact from the spike). The CLI's thread verbs gained `--on-behalf-of`/`--purpose` (the scope defaults to the command's own target — the honest minimal attenuation).
- The authorize path now runs the **dual evaluation**: the caller's OWN grant must hold the action (a non-holder cannot delegate), the SUBJECT's grant is the authority source (the record and the policy digest bind the subject), and the scope ladder refuses a widening request with a typed 403 naming the §16.3 invariant. The `.1.3` revocation filters apply unchanged: a revoked subject grant refuses the delegation at the next decision.
- The acceptance test proves all four legs (narrower-succeeds with the subject audited, widening-refused, the caller check, revocation-freshness); the audit test moved to the dual semantics. command_api grew to 16; the full guard green (12 suites + e2e + demo 32/32 rc=0); clippy/fmt clean; gate 13/13. **`.1.4` complete** — frontier → `.1.5` (the cached-decision semantics).

## 2026-09-07 — ADR-009: delegation rides the envelope (`PHASE-2.1.4.1`)

- The representation spike decided **chain-in-envelope** for the dev profile: the plumbing was pre-shaped (`CommandAuthz.delegate_subject` + the audit subject split), expiry/revocation ride the existing `.1.3` grant filters, and the wire form beats a capability-token blob (the same facts PLUS a 64-byte signature — asserted in the core suite's size leg). ADR-009 records the choice + the capability-token revisit trigger (multi-hop chains or a measured re-presentation cost).
- The widening invariant (§16.3.1) landed as a pure, tested function: `DelegationConstraints` + `delegation_scope_is_subset` in the core crate — narrower/equal/empty scopes pass, a foreign thread and a tenant-wide request over a thread-scoped grant are refused. The core suite grew to 39.
- A wire note for `.1.4.2`: `GrantSubject` is a serde *tagged newtype* (its wire form is a plain string) — the envelope's `authority_context` must carry the subject as a string field, not the enum (the size probe proved the serialization refusal). Frontier → `.1.4.2` (the implementation).

## 2026-09-07 — `.1.4` split at the ADR-vs-implementation seam (`PHASE-2.1.4`)

- The census found the delegation plumbing PRE-SHAPED: `CommandAuthz.delegate_subject` exists (always `None`) and the authorization-records INSERT already writes the subject split when delegation applies — what's missing is the envelope field, the dual (caller + subject) evaluation, and the widening check.
- Children: `.1.4.1` ADR-009 + the representation spike (chain-in-envelope vs capability tokens, with the pure subset-check prototype) → `.1.4.2` the implementation. Tree-only commit.

## 2026-09-07 — The `Revoked` statuses got their write paths (`PHASE-2.1.3.2`)

- `POST /v1/admin/grants/{id}/revoke` + `POST /v1/admin/boundaries/{id}/revoke` (tenant_admin-audited, typed 404/409 refusals) — the evaluation's existing `status = 'active'` filters refuse the subjects at the NEXT decision: the tests prove a revoked grant's next command is a 403 with the audited denial while the human's own grant keeps working.
- **A boundary revocation is the tenant freeze** — every grant under it (including the bootstrap human's) is refused at the next decision, and the tenant is read-only until a future superseding act. The tests caught the trap: the admin's own authorization rides the ceiling, so the inspection lists would have 403'd too — the carve-out (`authorize_tenant_admin_read`) authorizes admin READS grant-directly: the freeze stops writes, never the operator's eyes. Recorded in `docs/decisions/2026-09-07_boundary-revocation-freeze.md`.
- The admin inspection lists (`GET /v1/admin/grants|boundaries`) + `rb grant revoke` / `rb boundary revoke` / `rb inspect grants|boundaries` land. The command_api suite grew to 15; the full guard green (12 suites + e2e + demo 32/32 rc=0); clippy/fmt clean; gate 13/13. **`.1.3` complete** — frontier → `.1.4` (the delegated authority context).

## 2026-09-07 — The operator can now revoke a node (`PHASE-2.1.3.1`)

- `POST /v1/nodes/revoke` (tenant_admin-audited — the authorization record IS the audit) marks the node's active workload certificates revoked; the `.1.2.2` handshake ladder refuses them at the next crossing (typed 401) — the refusal path already existed, this leaf wired the operator verb.
- Migration 0012 appends `suspended` to the presence view: a revoked node reads `suspended` whatever its lease says — the live lease is NOT cut (suspension gates re-entry, it does not rewrite the running session). A real Postgres gotcha: `CREATE OR REPLACE VIEW` appends columns at the END only — the first attempt inserted mid-list and the migrate step refused it.
- The typed refusals: an unknown node is a 404, a non-admin caller is the 403 + audited denial, a second revocation (no active certificate left) is a 409. `rb node revoke --node … --reason …`; the demo gains the revoke beat (32 checks, node B after its thread closes).
- The channel suite grew to 21; the full guard green (12 suites + e2e + demo 32/32 rc=0); clippy/fmt clean; gate 13/13. Frontier → `.1.3.2` (the grant/boundary revoke verbs).

## 2026-09-07 — `.1.3` split at the cert-vs-grant seam (`PHASE-2.1.3`)

- The census found the REFUSAL paths already exist — the `.1.2.2` handshake ladder checks `revoked_at`, and the grant/boundary evaluation filters `status = 'active'` — while NO write path exists (`grep -n 'revoke'` over api.rs + the CLI → no verbs) and the presence view derives online/offline only (no suspended state).
- Children: `.1.3.1` node/cert revocation (`POST /v1/nodes/revoke` + the suspended presence + `rb node revoke` + the demo beat) → `.1.3.2` the `grant revoke`/`boundary revoke` verbs. Tree-only commit.

## 2026-09-07 — The channel proves itself with the workload certificate (`PHASE-2.1.2.2`)

- **CHANNEL_VERSION 3**: the handshake's HMAC secret is retired — the node signs the canonical coverage with its workload certificate's key, and the server verifies **chain-to-CA + the validity window + the node-id fingerprint + the signature before any ledger read**. A foreign, expired, unregistered, or wrongly-signed certificate is a typed 401, identically to an unenrolled node.
- **Rotation is additive and automatic**: `POST /v1/nodes/rotate` issues a fresh key + certificate (a new fingerprint; the old leaf stays valid until expiry/revocation), and the node rotates when less than half the leaf's lifetime remains — a running session is never cut.
- **A real interop discovery, measured not guessed**: ring's `UnparsedPublicKey` refuses rcgen's well-formed SPKI DER yet accepts the bare EC point (the path webpki uses internally — which is why the chain check always worked). The ladder probe isolated every leg before the fix; recorded in `docs/decisions/2026-09-07_cert-proof-verification.md`.
- The 19 channel tests (17 migrated + the rotation pair) + the demo pass on v3 (31/31, the new cert-file beat included); the full guard green (12 suites + e2e + demo rc=0), 42 offline suites, clippy/fmt clean, `make deny` rc=0. The book's node-channel + two-host-demo chapters carry the new contract. **`.1.2` complete** — frontier → `.1.3` (revocation surfaces).

## 2026-09-07 — Enroll now issues a workload certificate (`PHASE-2.1.2.1`)

- **The CA that survives the demo's kill point.** Migration 0011 adds `server_ca` (ONE row per deployment) + `node_certificates`; the server generates its CA on first boot and LOADS it thereafter — the enrollment suite's rebuild test asserts two `ensure_server_ca` passes return the same key + cert, so a server SIGKILL + restart never orphans an issued leaf.
- **Enrollment issues the leaf inside the exactly-once transaction.** The token row was already the serialization point; the leaf (CN = the durable node id, SAN = the token's host claim, 10-minute validity) rides the same transaction — the replay refusal issues no second certificate (asserted).
- **The node stores `cert.der`/`key.der` beside its journal** (dev-escrowed key — the `.1.2.1` trust-store stance, ADR-007's honest limit) and logs the fingerprint. The HMAC channel is untouched: the demo passes with the files stored, unused — the coherent interim before the v3 swap.
- Full guard green: 12 live suites (node_enrollment now 4) + e2e + demo 30/30 rc=0, 42 offline suites, clippy/fmt clean, `make deny` rc=0 (rcgen's `x509-parser` feature entered the server graph without a ban), gate 13/13. Frontier → `.1.2.2` (the channel v3 cert-proof handshake + rotation).

## 2026-09-07 — `.1.2` split at the issuance-vs-channel seam (`PHASE-2.1.2`)

- The Phase-1 `.1.2.1`-first precedent applies again: cert issuance at enrollment and the channel v3 proof swap separate cleanly because a coherent interim exists (the cert is issued and stored while the HMAC channel stays live). Children: `.1.2.1` (migration 0011 `server_ca` + `node_certificates`, the persisted CA, the enroll response gains cert + dev-escrowed key, the node stores `cert.der`/`key.der`) → `.1.2.2` (CHANNEL_VERSION 3: the cert signature replaces the HMAC proof; rotation at ≤50% lifetime; the 17 channel suites + the demo move). Tree-only commit.

## 2026-09-07 — The issuance model is measured, not guessed: ADR-006/007 land (`PHASE-2.1.1`)

- **The spike made the refusal cases real before any product code existed.** `crates/reasonbraid-cert-spike` (a test-only experiment — no bin, so `make release` stays four binaries) drives a REAL rustls TLS 1.3 client-cert handshake and passed 6/6 verdicts: the trusted allowlisted leaf completes; foreign-CA, expired, and unregistered-fingerprint certificates are refused on BOTH sides; rotation is additive. Issuance latency N=200: **p50 63 µs / p95 69 µs** — cert issuance is effectively free at LAN scale.
- **ADR-007 adopts the project-local CA**: 10-minute workload leaves from a control-plane CA, gated by chain-to-the-CA + the node id → fingerprint binding; revocation = server-side status + short expiry (no OCSP/CRL at the LAN profile). step-ca/SPIRE were evaluated on their published operational model, not installed (recorded asymmetry).
- **ADR-006 is accepted-with-evidence**: the shipped outbound channel (WP3 + `.1.2.2`) is the transport decision — promoted to the ADR before `.1.2` changes the wire again.
- **The supply-chain gate chose the dependency shape**: `make deny`'s first run failed the bans check on two base64 versions (rcgen's optional `pem` feature pulled 0.23 against hyper-util's 0.22) — fixed by dropping the unused feature (`default-features = false`, DER-only), not by adding a skip entry. The dependency ledger gains the identity-stack row (rcgen 0.14.10, rustls 0.23.43 pinned). Frontier → `.1.2` (the certificate lifecycle + channel v3).

## 2026-09-07 — `.1` decomposed at the census seams: six gaps own the identity lane (`PHASE-2.1`)

- The pickup census (tool-backed greps over the authority engine, api.rs, the node, migrations, Cargo.tomls, and the ADR index) found the scoped-grant core + one-time enrollment tokens EXIST (Phase 1 carry) while six gaps own the lane: **no workload certificate machinery at all** (ADR-006/007/008/009 unopened), **no revocation write path** (`Revoked` statuses are types + tests only), **no delegated authority context** (§16.3's chain is refused), **no cached-decision semantics**, **no incarnation/run writers** (the Phase-1 gate-record deferral #4), and **no dependency-ledger identity-issuer row**.
- Decomposition: `.1.1` the ADR-006/007 issuance spike → `.1.2` the certificate lifecycle + channel v3 → `.1.3` revocation surfaces → `.1.4` the delegation context → `.1.5` cached decisions → `.1.6` incarnation/run writers. Tree-only commit; the lockstep docs carry the frontier.

## 2026-09-07 — Phase 1 closes: G1–G2 Met, Demonstration A passed 30/30 (`PHASE-1.8.2`)

- **The exit gate is closed.** The gate record (`docs/decisions/2026-09-07_phase1-gate-record.md`) marks G1–G2 **Met**: G1 = 39 offline suites + 12 live-PG suites + CLI e2e, `make deny` rc=0 (advisories/bans/licenses/sources ok), `make secret-scan` rc=0 (no leaks); G2 = real durable stores (PostgreSQL + SQLite journals), the node journal, two live-qualified adapters + the deterministic fake, and the recovery demonstration. The evidence manifest (`docs/evidence/2026-09-07_phase1-evidence-manifest.md`) maps every clause to re-runnable commands.
- **Demonstration A passed 30/30** — debug AND release-built (`target/gate82_guard.log`, `target/gate82_release_demo.log`): no manual relay, restart loses no accepted command, kill-after-dispatch → exactly one visible `outcome_unknown` (no silent retry), duplicate → one domain effect, offline inbox + resume, spend/uncertainty visible, `inconclusive` + the register, everything inspectable through the CLI/UI — the two psql reads are the documented credential oracle.
- **Five named deferrals, each with a trigger**: capability advertisement → Phase 3; expected-artifact/decision-rule fields + LLM synthesis → Phase 5; incarnation/run writers → Phase 2; fuzz → Phase 4 parsers; ops hardening (TLS, supervision, containers, PG automation) → Phase 2. The §19.8 subtraction record (`docs/decisions/2026-09-07_phase1-subtraction-record.md`) has no empty list.
- **Phase 1 is CLOSED** — the `PHASE-1` tree is `done`, the `PHASE-2` tree is `active` (frontier `.1`, identity/recovery), the book's roadmap chapter carries the transition. Full guard green at the close: 39 offline suites, 12 live suites, e2e, the demo 30/30 (debug + release), `make deny`, `make secret-scan`, `make gate` 13/13, `make book`.

## 2026-09-07 — The audit view reconstructs the story: the §26.1 demo evidence is complete (`PHASE-1.8.1`)

- The demo's header claimed "the audit view reconstructs the whole story" with no beat proving it — now there is one. The human contributes a position carrying an **evidence reference** (`.1.5.1`'s surface exercised end-to-end) and **section 11** rebuilds the demonstration through the supported read surfaces only (the console's own curl + dev-header pattern — no psql, no database surgery): A's audit records (invite → accept → contribute → close, each with a 64-hex policy digest), the ordered event timeline (create → accept → contribute → close), the reference riding the human contribution's event, B's denied reservation row with the engine's reason, and B's close authority.
- The first run caught a beat mis-read, not a product defect: the thread-scoped audit view **starts at the invite** because `thread.create` authorizes against the tenant scope (the thread does not exist yet) — the exact shape the `command_api` suite's asserted list already adjudicates. The beat now asserts the honest contract.
- The demo's two psql reads (the fencing token to forge the duplicate transport) gained comments naming the **credential-oracle distinction**: a least-privilege API must not expose a live credential, so the read stays — and every *state* assertion runs through the CLI/API/`rb-journal`.
- The demo now passes **30/30** (`rc=0`); the full guard green (12 live suites + e2e + demo); `make gate` 13/13; the book's two-host-demo chapter gains steps 10–12. **The §26.1 demo evidence is complete end-to-end** — frontier → `.1.8.2` (the G1–G2 gate package + the Phase 1 close).

## 2026-09-07 — `.1.8` decomposed at the census seams: the demo already proves §26.1 (`PHASE-1.8`)

- The tool-backed census (`grep -n` over the demo + `crates/` + `.github/` + `migrations/`) maps every §26.1 acceptance point to EXISTING demo evidence with real SIGKILL kill points: enrollment, no human relay, durable invitations + blind contributions, duplicate transport → exactly one domain effect, server restart loses no accepted command, node SIGKILL after dispatch → exactly one `outcome_unknown` with boundary history (no silent retry, no effect), the budget denial journaled before provider contact, spend visible (`.1.6.1`), `inconclusive` + the unresolved register (`.1.5.3`), and every state assertion through CLI/API/`rb-journal`/console — never psql (the two psql reads obtain the fencing token to forge the duplicate: a credential oracle, which a least-privilege API must not expose).
- **The last demo gap:** the header claims "the audit view reconstructs the whole story" but no beat asserts it, and no demo contribution carries an evidence ref → `.1.8.1` closes it.
- Named deferrals with triggers (ride the gate record): capability advertisement → Phase 3 directory; expected-artifact/manual-decision-rule fields + LLM synthesis → Phase 5; incarnation/run row writers → Phase 2 identity; fuzz baseline → Phase 4 parsers.
- Decomposition: `.1.8.1` (the audit-reconstruction demo leg) → `.1.8.2` (the G1–G2 gate package: `make deny` + `make secret-scan` evidence, the Phase-1 evidence manifest, the gate record, the subtraction record, the Phase 1 close). Tree-only commit.

## 2026-09-07 — The packaged LAN story: four self-contained binaries + a practiced runbook (`PHASE-1.7.2`)

- `make release` builds the four release binaries (`rb`, `rb-server`, `rb-node`, `rb-journal`) — self-contained: the migrations and the console embed at compile time, so a deployed binary needs no runtime path back to the checkout (§12).
- `deploy/README.md` is the LAN runbook: the two Phase 1 profiles (Developer = `make dev`; Trusted LAN = `rb-server --host 0.0.0.0` + enrolled outbound nodes), per-host steps, the demo's ssh two-host mode as the reference exercise, the honest limits (dev trust store, plain HTTP — Phase 2 owns TLS/mTLS), and the subtraction record (no config files, supervision units, containers, or PG automation — deferred to Phase 2 ops with triggers).
- **The packaging claim is verified by running the package:** the demo gained a `--release` build-root switch and passed **24/24 on the release binaries** (rc=0; the bundle's `env.txt` records the release root). `make demo` stays debug. The book gains the `deployment` chapter. Decision recorded: `docs/decisions/2026-09-07_deployment-packaging.md` (`answers:`). Full guard green: 12 live suites + e2e + demo rc=0; `make gate` 13/13; `make book` builds. **`.1.7` complete** — frontier → `.1.8`.

## 2026-09-07 — `make dev`: the one-command development environment (`PHASE-1.7.1`)

- `scripts/dev.sh` + the `make dev` target: one command boots an ephemeral on-volume PostgreSQL (§13 shape — `target/dev-ephemeral.*`, gitignored, removed on exit), starts `rb-server` in the foreground (migrations run on startup), and prints the console URL + the CLI hint. Ctrl-C stops everything with a residue census. An interactive dev loop — not the test harness.
- `bash scripts/dev.sh --check` is the permanent self-verification beat: the console serves at `/`, a real `rb` enroll + `inspect threads --as devcheck` round-trips through the control API, and the residue census reports 0. Its first three runs caught three authoring slips (the inspect verb shape, the dev-profile `--as` requirement, census-before-teardown ordering) — all fixed, beat green (`dev-check: OK`, rc=0).
- The book's introduction gains the Run-it section; the README quick start gains the `make dev` line (48 lines / 1,851 bytes — within the README caps). Full live guard green: all twelve server suites + CLI e2e + the two-host demo rc=0; `make gate` 13/13; `make book` builds. Frontier → `.1.7.2`.

## 2026-09-06 — `.1.7` decomposed at the census seams: dev loop + packaged LAN (`PHASE-1.7`)

- Gap census first: **no one-command dev environment** (`make dev`/dev script — none; the ephemeral-PG machinery is test-only inside `run_pg_tests.sh`), **no packaging** (no `deploy/` dir, no release build target, no install path — the demo builds DEBUG only), and **LAN surfaces partial + unexercised** — `rb-server --host/--port` binds any address, the node's `--server` points cross-host, migrations + UI embed at compile time (one self-contained binary), and the demo already carries a real ssh two-host mode — but nothing runs release binaries and no server-side runbook exists.
- Decomposition: `.1.7.1` the one-command dev loop (`scripts/dev.sh` + `make dev`: ephemeral on-volume PG, foreground server, residue census) → `.1.7.2` the release packaging + LAN story (`make release`, `deploy/` runbook, the book's `deployment` chapter, the release-built demo proof). Tree-only commit; `make gate` 13/13.

## 2026-09-06 — The demo proves the console without a browser; `.1.6` is complete (`PHASE-1.6.3`)

- The two-host demo gains **section 10**: the console's evidence beat. The same binary that serves the API serves the embedded page at `/`; curl is the browser stand-in, so the beat asserts the shell marker, that `app.js` references ONLY the seven documented read surfaces, names no write verb, and that the page's live same-origin fetch (dev-profile header + tenant) returns the demo's thread and its budget ledger. 6 new checks, all PASS on the first run (the run also caught a cosmetic script slip — backticks in a check label execute as command substitution — fixed, re-verified).
- The evidence bundle now carries `console-index.html`, `console-app.js`, `console-thread-a.json`, `console-budget-a.json` + a summary row; the book's two-host-demo chapter gains scenario step 11. Demo 24 PASS, `rc=0`; the full guard set green; `make gate` 13/13.
- **`.1.6` is COMPLETE** — backlog 18 done (`.1.6.1` budget read surface, `.1.6.2` embedded static shell, `.1.6.3` demo/evidence leg). Frontier → `.1.7` (local/LAN deployment packaging).

## 2026-09-06 — The embedded static shell: a read-only console in one binary (`PHASE-1.6.2`)

- `rb-server` now serves the inspection console at `/` (`/app.js`, `/style.css`): a vanilla HTML/JS page — no framework, no frontend build pipeline — **embedded at compile time** (`include_str!`), so the deployment stays a single binary with no runtime paths (§12). A state-free `ui_router` merged into the existing listener adds zero API routes.
- The page is a **client, not a surface**: same-origin GETs with the dev-profile `x-reasonbraid-principal` header + `tenant_id` query, so every gate, denial, and audit row applies exactly as to the CLI. It renders the threads list/detail, the event timeline, the audit records, the `.1.6.1` budget view, node presence, and the admin inbox. READ-ONLY (no write verb) and text-safe (HTML never assembled from data).
- The offline tests enforce the page's honesty mechanically: it references ONLY the documented GET surfaces, names no write verb, and never assembles HTML from data — the contract test's first run caught the page's own comment naming the forbidden API (reworded); the serving test proves the typed content types over a real listener. Server unit suite 5→7; `make book` gains the `web-ui` chapter. Decision recorded: `docs/decisions/2026-09-06_ui-embedding.md` (`answers:`).

## 2026-09-06 — The budget read surface: spend is visible without database surgery (`PHASE-1.6.1`)

- The `.1.6` census finding became a surface: `GET /v1/threads/{thread_id}/budget` returns the ceiling (dimensions, policy version, created time) plus every reservation row — held vs settled usage, denials with the budget engine's reasons, expiry/settle times. A READ-ONLY pass-through of the ledger rows the engine enforces against: nothing computed, nothing invented, so the §26.1 "spend and uncertainty are visible" fact follows the same rows.
- Gated by the existing `thread_inspect` path (the `get_thread` gate): a role without the grant is a typed 403 naming the audit record — the upcoming UI inherits this unchanged; no new grant, no write path, no new table. A thread whose ceiling row is missing reads `scope_hidden` (BUDGET-003: corruption, not an empty budget). Absent optional facts are OMITTED on the wire.
- CLI: `rb inspect budget <thread>` (mirrors `inspect thread`). New `command_api` test (create → accept-hold → deny → GET → settle → GET → role-403); the e2e drives the real binary. The suite's first run proved the surface right and the TEST wrong — the row's `reason` is the engine's raw detail (`the ceiling does not cover …`), not the dispatch site's prefix; the e2e's first run caught a positional-vs-`--thread` slip. All twelve live suites (command_api 13) + e2e + demo `rc=0` under the pinned 1.98.0 toolchain; 39 offline suites green; clippy + fmt clean; `make gate` 13/13. Decision recorded: `docs/decisions/2026-09-06_budget-read-surface.md` (`answers:`).

## 2026-09-06 — Toolchain pinned: `stable` is a moving pointer (`PHASE-1-MAINT-3`)

- Discovered during the `.1.6.1` verification: `cargo fmt --all -- --check` flagged pre-existing hunks in files the leaf never touched (`claude.rs`, `claude_adapter.rs`, `state.rs`, `invitations.rs`) — under BOTH installed rustfmt builds (`1.9.0-stable` 2026-04-14 from rustc 1.95.0 and 2026-08-18 from 1.98.0), so HEAD was simply not fmt-clean under the current stable channel.
- Root cause: the channel was `stable` everywhere (`rust-toolchain.toml` + CI `dtolnay/rust-toolchain`), the stable channel moved since the tree's last full fmt run, and recent leaves verified clippy but not fmt — the drift sat undetected.
- Fix: `rust-toolchain.toml` pins `1.98.0` (the newest installed), the CI toolchain inputs name it explicitly, and the tree was normalized once with `cargo fmt --all` under the pin. Clippy 1.98 clean; fmt `rc=0`. Decision recorded: `docs/decisions/2026-09-06_pinned-toolchain.md` (`answers:`).
- Observed alongside (out of scope, reconciled at the first push per §16): rust.yml's PG job omits the post-`.2.1` suites (`aggregate_library`, `identity_store`, `node_enrollment`, `node_inbox`, `invitations`).

## 2026-09-06 — `.1.6` decomposed at the census seams: budget read → static shell → evidence leg

- Gap census first (the pickup action the direction record prescribed): the existing read surfaces — `GET /v1/threads` (list), `GET /v1/threads/{id}` (projection + derived view), `GET /v1/threads/{id}/events` (the audit timeline), `GET /v1/threads/{id}/audit`, `GET /v1/nodes/presence`, `GET /v1/nodes/inbox` (admin) — already exist and the page mirrors them with the dev-profile `x-reasonbraid-principal` header + `tenant_id` query (same-origin, so the page inherits the existing gates unchanged).
- **Census finding: budgets have no read surface anywhere** — the ledger rows exist (`migrations/0005_budget.sql`: ceilings + reservations with status/usage/reason) but no GET endpoint and no CLI verb. The finding became `.1.6.1`.
- Decomposition: `.1.6.1` the budget read surface (read-only query over the existing ledger, inspect-gated) → `.1.6.2` the static shell (`web/{index.html,app.js,style.css}` embedded at compile time, served at `/` by `rb-server`, read-only, XSS-safe) → `.1.6.3` the demo/evidence leg. Tree-only commit; `make gate` 13/13.

## 2026-09-06 — `.1.6` UI direction decided: a vanilla static page served by `rb-server` (`PHASE-1.6`)

- The director adopted the recommendation (2026-09-06): the `.1.6` Web UI is a **vanilla static page served by `rb-server`** — plain HTML + JS, no frontend build pipeline, no framework. It mirrors the existing read surfaces (threads, nodes, inbox, budgets, audit timeline) and the CLI stays the primary, fully-covering surface. Decision recorded: `docs/decisions/2026-09-06_ui-direction.md` (`answers:`); the leaf's Note carries the direction for the decomposition. Tree-only commit; `make gate` 13/13.

## 2026-09-06 — The stderr-drain race: reproduced and fixed in both adapters (`PHASE-1-MAINT-2`)

- The tracked one-off `codex_adapter` failure (from the `.1.3.1` verification, then nameless) **reproduced with the failing test captured** during the `.1.5.3` verification: `nonzero_exit_produces_failed_known_with_the_stderr_tail` failed with an **EMPTY stderr tail** — the spawned stderr-drain task had not consumed the pipe's tail when the EOF path snapshotted the buffer after `child.wait()`. A real race: load only widens the scheduling window.
- Fix: `drain_stderr` returns its JoinHandle and the EOF path **awaits it (bounded at 5 s)** before the snapshot — in `codex.rs` AND its `claude.rs` mirror (mirrors inherit defects). The `FailedKnown` reason now carries the provider's stderr tail deterministically.
- Verified: 10× loop over both adapter suites (10/10 green) + the full offline workspace (39 suites) + clippy clean + `make gate` 13/13. Decision recorded: `docs/decisions/2026-09-06_stderr-drain-race.md` (`answers:`). **Both Phase-1 defect leaves are now closed** (`MAINT-1` §13 locality, `MAINT-2` the drain race) — 0 tracked defects remaining.

## 2026-09-06 — The honest close: `Inconclusive` is a core terminal (`PHASE-1.5.3`; `.1.5` complete)

- The core thread machine gains the **`Inconclusive` terminal** (`Closing → FinalizeInconclusive`; the exhaustive state-table + terminal-rejection tests extended — the canary pattern): a thread whose deliberation did not converge now ends honestly, distinct from a decided `closed` and the `cancelled` abandonment.
- `thread.close` gains `outcome` (`decided` **stated default** | `inconclusive`) and the `unresolved` register (the items that prevented the decision — they ride the close EVENT; event-layer growth, no projection change). A `decided` close carrying unresolved items is a typed 400 refusal — naming what is still open while claiming a decision would be dishonest; the terminal refuses further content verbs.
- CLI: `rb thread close --outcome inconclusive --unresolved …` (kebab-normalized, repeatable). The two-host demo's budget-denied thread B now closes INCONCLUSIVELY with the register asserted (18 PASS checks). New `command_api` test + the e2e's honest leg (its first run caught a missing `--json`, fixed). All twelve live suites (command_api 12) + e2e + demo `rc=0`; offline suites green ×3; clippy clean; `make gate` 13/13. Decision recorded: `docs/decisions/2026-09-06_honest-inconclusive-close.md` (`answers:`). **`.1.5` complete — backlog 17 done.**
- **`PHASE-1-MAINT-2` reproduced** during this leaf's offline verification: `codex_adapter::nonzero_exit_produces_failed_known_with_the_stderr_tail` failed with an EMPTY stderr tail — the stderr-drain task raced the EOF-path snapshot (a real race in the `.4.2` code and its `claude.rs` mirror). Repro + root cause recorded in the defect leaf; the fix executes next.

## 2026-09-06 — Server-assigned rounds: the advance verb and its grant (`PHASE-1.5.2`)

- Rounds landed as **server-assigned facts**: a new thread is round 1 (the additive `current_round` projection field, `#[serde(default)]`), every contribution lands in the current round and its event carries the number, and `thread.advance_round` (event `thread.round_advanced`) is the only mover — the client never names a round, so round skew cannot be submitted by construction.
- Advancement is a new **`thread_advance_round` grant** (the wire-name canary extended first — it failed until the registry row landed): humans carry it via the 9-action dev admin set; roles are deny-by-default (typed 403 with the audit row) — they shape content, humans shape the process. A closed thread refuses advancement (`invalid_transition`).
- CLI: `rb thread advance-round`; the two-host demo's THREAD_A now advances after the agent contribution and asserts the projection round (2) and the contribution's round (1) — 16 PASS checks total. The demo's first run caught a positional-vs-`--thread` slip in the new beat, fixed. All twelve live suites (command_api 11) + e2e + demo `rc=0`; clippy clean; `make gate` 13/13. Decision recorded: `docs/decisions/2026-09-06_rounds.md` (`answers:`).

## 2026-09-06 — The structured contribution body: typed kinds + evidence references (`PHASE-1.5.1`)

- Backlog 17's first contract landed: `thread.contribute` gains `kind` — a typed deny-unknown `ContributionKind` enum over the §8.5 initial subset (`position` **stated default** | `claim` | `assumption` | `evidence_reference` | `question` | `summary`) — and `evidence_refs`, a list of `{uri, digest?, note?}` REFERENCES (deny-unknown at the ref itself; absent fields are omitted on the wire). Out-of-registry kinds and foreign ref fields are typed 400 `invalid_command` refusals.
- Content lives in the event log, so the change is event-layer growth: the projection is untouched and pre-`.1.5.1` stored projections parse by construction. The events/inspection view renders kind + refs — nothing silently dropped.
- CLI: `rb thread contribute --kind …` (kebab→snake normalized, the `.1.1.3` lesson pre-applied) + repeatable `--evidence-uri`. New 4-leg `command_api` test + the e2e drives the kebab spelling through the real binary. The suite's first run caught a real wire-shape defect (absent ref fields serialized as `null` instead of omitted) — fixed with `skip_serializing_if`, rerun green: all twelve live suites + e2e + demo `rc=0`; clippy clean; `make gate` 13/13. Decision recorded: `docs/decisions/2026-09-06_structured-contributions.md` (`answers:`).

## 2026-09-06 — PHASE-1.5 decomposed: typed contribution bodies, rounds, honest close (`PHASE-1.5`)

- Tree-first decomposition on a measured gap census: the contribution body's `kind` is a FREE STRING (no typed enum), there are no evidence references, no round fields anywhere, and the core thread machine has NO `Inconclusive` terminal (Open/Closing/Closed/Cancelled only) — so backlog 17 needs three independent contracts.
- `.1.5.1` the structured contribution body (`kind` → a typed deny-unknown §8.5 enum + `evidence_refs` — references only, acquisition stays Phase 4); `.1.5.2` rounds (a contribution names the round; enforced at the boundary, visible in inspection); `.1.5.3` the honest close (`outcome: decided|inconclusive` + the unresolved register + the core `Inconclusive` terminal). Votes/abstentions and the workflow-phase concept defer to Phase 5's workflow engine. `make gate` → 13/13 at commit.

## 2026-09-06 — `.1.4` complete: the Claude adapter is LIVE-qualified (`PHASE-1.4.2`)

- The env-gated live qualification test (`crates/reasonbraid-node/tests/claude_live.rs`, `RB_LIVE_CLAUDE=1`, ignored by default — the `.4.2` codex_live mirror) dispatched ONE bounded real run through the real supervisor + journal and **passed on its first run**: completed, the reply streamed, exact usage, **money cost** (`total_cost_usd` — the leg Codex cannot prove), the session id attached as the provider handle, and the honest unsupported status lookup.
- The dependency-ledger Claude row is rewritten with the verified facts (checked_at 2026-09-06, tested 2.1.263, the stream-json interface, the required `--verbose`, content-only `--restricted --tools ''`, the conformance results); the book's adapter chapter gains the live-test command and the honest-limits bullets now name two real adapters; decision record `docs/decisions/2026-09-06_claude-cli-adapter.md` (`answers:`) + DEV_NOTES promoted to it.
- **`.1.4` is complete — backlogs 19–21 done: the deterministic fake (Phase 0), the Codex adapter (Phase 0), and now the Claude adapter.** Two genuinely distinct harness adapters exist; each is qualified on one host + one CLI version with a revalidation trigger. `cargo test --all` green (the live test SKIPs without the env var), clippy clean, `make gate` 13/13, `make book` builds. Frontier → `.1.5` (structured contributions).

## 2026-09-06 — The Claude CLI adapter core: `claude.rs`, the `.4.2` mirror (`PHASE-1.4.1`)

- Backlog 21's first half landed: `ClaudeCliAdapter` (`crates/reasonbraid-adapter/src/claude.rs`) supervises `claude -p --output-format stream-json --restricted --tools '' --verbose -- <prompt>` — the narrowest supported machine interface, qualified against the INSTALLED Claude Code 2.1.263. The stream maps: `system/init` (`session_id`) → `ProviderRequestId`; `assistant` text blocks → one chunk each (thinking blocks skipped — the reply is the text); `result` `is_error:false` → `Completed` with the FULL result event (usage under `usage`, money under `total_cost_usd` — Claude reports COST, so `cost` is `Some`, unlike Codex's `None`); `result` `is_error:true` → `FailedKnown` with the provider's own message; non-zero exit → `FailedKnown` with the stderr tail; EOF without a result → lost response.
- The wire facts were pinned by three bounded live probes BEFORE code (no guessed formats): `--verbose` is REQUIRED by the CLI with `stream-json` (the no-verbose probe was refused pre-dispatch), the prompt must follow `--` (variadic `--tools` otherwise swallows it), and Anthropic's token counts already include caches/thinking (no folding — the one place the Codex normalizer differs).
- `--restricted` + `--tools ''` make the boundary content-only; the adapter holds no credentials (ambient Claude login); status lookup is honestly `Unsupported` (`--resume` continues, it does not query). New `tests/claude_adapter.rs` (10 offline tests over a stub binary — the REAL subprocess boundary; the suite's first run caught a test-authoring slip on the multi-chunk assertion, fixed). All offline suites green + all twelve live-PG suites + CLI e2e + two-host demo `rc=0`; clippy clean; `make gate` 13/13. The book's adapter chapter gains the Claude section (the live-test command arrives with `.1.4.2`).

## 2026-09-06 — PHASE-1.4 decomposed: the Claude CLI adapter (`PHASE-1.4`)

- Tree-first decomposition on a measured gap census: backlogs 19 (deterministic fake) and 20 (Codex adapter) are Phase-0-proven (`.4.1`/`.4.2`), so `.1.4`'s delta is backlog 21 — the Claude-family adapter as the `.4.2` mirror. The live `claude` CLI is **installed** (2.1.263), so the real harness leg runs for real, env-gated like `RB_LIVE_CODEX`.
- The machine interface is VERIFIED against the real binary before any code: three bounded probes pinned the 2.1.263 stream — `system/init` carries `session_id`; `assistant` text blocks are the reply; `result` carries `usage` AND `total_cost_usd` (Claude reports money — normalized cost, unlike Codex); the CLI REFUSES `stream-json` without `--verbose` (pre-dispatch error). Probe evidence kept on-volume in `target/claude-probes/` (the /tmp originals deleted, residue-census-verified — §13).
- Children: `.1.4.1` the adapter core (`claude.rs` + offline stub suite) → `.1.4.2` live qualification + dependency-ledger row + book chapter + decision record. `make gate` → 13/13 at commit.

## 2026-09-06 — §13 same-volume locality for the ephemeral PG test cluster (`PHASE-1-MAINT-1`)

- The defect leaf opened during `.1.1.1` is closed: `scripts/run_pg_tests.sh` no longer defaults its ephemeral PostgreSQL data dir to `${TMPDIR:-/tmp}/reasonbraid-pg.XXXXXX`. The cluster now lives at `$ROOT/target/pg-ephemeral.XXXXXX` — derived at runtime from the script's own location, same volume as the repo, gitignored via `/target`, still unique per run, still cleaned by the exit trap (only the parent directory moved; the `mktemp`/cleanup mechanics are untouched).
- Verification is the script's own full rerun from the new location, twice: all **twelve** live server suites green (`test result: ok.` 4 + 5 + 9 + 5 + 9 + 3 + 4 + 17 + 3 + 3 + 6 + 7 `passed`) + CLI e2e `2 passed` + the two-host demo `ALL acceptance checks passed` (both `rc=0`); a 2 s poll observed the cluster on the repo volume mid-run and the residue census left nothing under `/tmp` or `target/` after. `make gate` → 13/13 at commit. Decision recorded: `docs/decisions/2026-09-06_same-volume-pg-ephemeral.md` (`answers:`).

## 2026-09-06 — Simple subscriptions: `thread.join`, enforced participant doors (`PHASE-1.3.2`)

- `thread.join` lands (event `thread.participant_joined`, `via: join_request`): a thread whose `allow_join_requests` is on admits the role directly as `accepted` — the self-request path, no invitation, no reservation machinery (a subscription is an ordinary `thread_contribute` act). A closed door and a double join are typed refusals.
- The `.1.1.3` participant rules became load-bearing: `allow_explicit_invites=false` now REFUSES the invite verb at the boundary (the join door still works), and `allow_join_requests` gates the join verb. The listing surface is the existing inspection view (full participant states + invitation meta).
- The race test taught a domain fact first: concurrent accept+remove BOTH succeed — they are COMPATIBLE transitions (accept-then-revoke is a legitimate sequence; the snapshot is `revoked` either way). The conflict pair is accept vs decline; the suite now races that and asserts exactly one winner. New fourth `invitations` test; `rb thread join`; full live-PG regression (twelve suites) + CLI e2e + two-host demo green (`rc=0`); clippy clean; `make gate` 13/13. Decision recorded: `docs/decisions/2026-09-06_join-subscriptions.md` (`answers:`). **The `.1.3` coordinator leaf is complete** (backlogs 15/16).

## 2026-09-06 — The explicit-participants contract: pending invitations, accept/decline/remove, dispatch-on-accept (`PHASE-1.3.1`)

- Backlog 16 + 15's membership semantics landed: `thread.invite` records a **pending offer** (typed optional `expires_in_seconds`; `invited_at`/`expires_at` ride the event AND the new additive `invitations` projection map) and **enqueues nothing** — the `.6.2` invite-time dispatch is gone.
- The invited role **accepts** (`thread.accept_invitation`, event `thread.invitation_accepted`) — and the accept is the transaction that dispatches the contribute work item with its reservation (`work_{accept_event_id}`: an accepted invitation exists iff its work does). `thread.decline_invitation` (event `thread.invitation_declined`) refuses the offer; `thread.remove_participant` (tenant_admin; the core machine gains `revoked` + the `Revoke` transition; event `thread.participant_removed`). Re-invitation is allowed over any terminal state.
- **The invitation IS the acceptance capability** (offer/reserve): accept/decline authorize against the pending invitation naming the actor, under a NEW `thread_invitation_respond` grant (the grant registry + its wire-name canary extended first; the role default gains it). An invited role may NOT act until accepted — the auto-accept on first contribution is gone (typed `invalid_transition` naming the accept verb).
- **Expiry is derived, never swept** (the lease-presence pattern): the inspection view reads past-expiry offers as `expired`, and accept/decline refuse them 409 — no sweeper, no stored flag, no expiry event.
- The wiring suites, the CLI e2e, and the two-host demo moved to the explicit contract (`rb thread accept` as the role before the node starts; the invite step produces no work). New `tests/invitations.rs` (3 live-PG tests: the full lifecycle + typed refusals; decline/expiry/re-invitation; the concurrent accept/remove race — exactly one winner). Full live-PG regression — **twelve** server suites green + CLI e2e + two-host demo `ALL acceptance checks passed` (`rc=0`); offline suites green; clippy clean; `make gate` 13/13. Decision recorded: `docs/decisions/2026-09-06_explicit-participants.md` (`answers:`).

## 2026-09-06 — PHASE-1.3 decomposed: invitation lifecycle, dispatch-on-accept, subscriptions (`PHASE-1.3`)

- The invitation/subscription leaf is decomposed into three signoff-sized children (tree-first; no code change), on a measured gap census: the `.6.2` wiring dispatches work IN the invite transaction with no acceptance step (`ensure_participant` auto-accepts an invited role on its first contribution), there are no accept/decline/expire/remove verbs or invitation records, `allow_join_requests` is typed but inert, and `allow_explicit_invites=false` is recorded but not enforced.
- `.1.3.1` the explicit-participants contract (backlog 16 + 15: invite records a PENDING invitation with a typed optional expiry and enqueues NO work; accept/decline/remove verbs; the work item + reservation enqueue with the ACCEPT event; derived expiry; the invitation IS the acceptance capability), `.1.3.2` simple subscriptions (`thread.join` under `allow_join_requests`, `allow_explicit_invites=false` enforcement, the subscription listing + CLI verbs). `make gate` → 13/13 green at commit. Amended same-day: the lifecycle and the dispatch move are one contract — the original `.1.3.2` merged into `.1.3.1` (separating them leaves an incoherent interim).

## 2026-09-06 — Durable inbox hardening: quarantine, measured retention, inspection (`PHASE-1.2.3`)

- Backlog 14's remainder landed: **quarantine is a database fact on the row** — migration 0010 adds `quarantined_at` + `quarantine_reason` to `node_inbox`, and the replay/poll queries filter `quarantined_at IS NULL`, so a quarantined command is **never re-delivered**, whatever cursor the node reports. The reason rides the row: the skip is explainable, never silent. Quarantine controls *delivery*, not result application (a result from a command delivered before the quarantine still applies).
- **Retention is an explicit, measured operator action**: `POST /v1/nodes/inbox/prune` deletes only DELIVERED rows older than the `min_age_seconds` window, with before/delete/after computed in ONE transaction — the response is the operator's receipt. No background sweeper.
- The operator surface is the **`tenant_admin`-audited control API** (the same gate as token issuance — the authorization record is the audit, no new audit table): `POST /v1/nodes/quarantine` (typed refusals: unknown command 400, re-quarantine 409, empty reason 400, non-admin 403), `GET /v1/nodes/inbox` (delivery + quarantine facts per row, with the payload), `POST /v1/nodes/inbox/prune` (negative window 400, non-admin 403).
- CLI: `rb node quarantine|inbox|prune`. New `tests/node_inbox.rs` (3 live-PG tests: quarantine skipped by replay AND poll + reason rides the row + inspection; typed refusals; measured prune with only old delivered rows gone). Full live-PG regression — **eleven** server suites green + CLI e2e + two-host demo `ALL acceptance checks passed` (`rc=0`); offline suites green; clippy clean; `make gate` 13/13. Decision recorded: `docs/decisions/2026-09-06_node-inbox-retention.md` (`answers:`). **The `.1.2` coordinator leaf is complete** (backlogs 11–14).

## 2026-09-06 — The authenticated node channel: key-proof handshake, leases, observable presence (`PHASE-1.2.2`)

- Backlog 13's remainder landed: the channel is now authenticated end to end, `CHANNEL_VERSION` **2**. The handshake carries an **HMAC-SHA256 key-proof** over the canonical channel fields (the mirrored `ProofCoverage` shape IS the canonicalization), keyed with the `.1.2.1` dev secret — verified in constant time and refused `401 unauthorized` BEFORE any ledger fact is read; a missing field is a malformed request (422), a wrong proof and an unenrolled node fail identically (no existence leak).
- A successful handshake issues a **lease** with a fresh **fencing token** (`fnc_<uuid>`, generated in PostgreSQL): the only token that renews the lease (`POST /v1/nodes/heartbeat` — live leases only) or guards `events`/`ack`/`poll`. Every handshake rotates it, so a stale process is fenced the moment a newer handshake lands. `poll` became a POST — the token never rides a query string.
- **Presence is derived, never stored:** migration 0009's `node_leases` + `node_presence` view compute `online` from the 60 s expiry clock — expiry flips a node observably `offline` (`GET /v1/nodes/presence`; the token is never exposed), and only a fresh key-proof restores it.
- The channel identity space widened: issuance + enrollment now accept the `rol_…` role wire id alongside `nod_…` (the dev wiring collapses node == role; a latent `.1.2.1` strictness the authenticated handshake exposed — a superset, the `.1.2.1` suites never asserted `nod`-only).
- The node client keeps the fencing token in shared state (worker poll + heartbeat task + reconcile rotation observe one lease); `rb-node --node-secret` is required and runs a 15 s heartbeat loop. All 13 channel tests moved to the authenticated contract + 4 new (refusal classes, renewal + presence, fencing rotation, expiry → offline → re-handshake); the two-host demo now enrolls its nodes, re-POSTs the duplicate with the live fencing token, and asserts presence online before AND after the server restart (14 PASS checks, `rc=0`). Full live-PG regression green; offline suites green; clippy clean; `make gate` 13/13. Decision recorded: `docs/decisions/2026-09-06_node-channel-auth.md` (`answers:`).

## 2026-09-06 — Dev-profile node enrollment: one-time tokens, one auditable transaction (`PHASE-1.2.1`)

- Backlog 11 landed: `migrations/0008_node_enrollment.sql` (one-time tokens bound to tenant + node id + host claim + nonce + expiry; `node_keys` holding the dev signing secret + its SHA-256 fingerprint; the `node_enroll_audit` refusal log) plus the hosts get-or-create index on the 0007 table.
- The flow: an authorized human issues a token (`POST /v1/nodes/enroll-tokens`, `tenant_admin` authority — the authorization engine audits the issuance); the node consumes it (`POST /v1/nodes/enroll`, the token IS the credential) with its dev secret — host + node + key + token-consumption + audit land in ONE transaction. Certificate issuance stays deferred to Phase 2 (ADR-007); the server-as-trust-store dev stance is the documented `.6.1` pattern.
- **One-time is a database fact:** `FOR UPDATE` on the token row + `used_at` makes replay impossible; every refusal (unknown/used/expired/mismatched/nonce) is a committed audit row before the typed error returns — the budget engine's denial-row pattern.
- Operator surface: `rb node issue-token` + `rb-node --enroll-token … --enroll-nonce … --host-claim … --node-secret …` (the node enrolls before any channel traffic). New `tests/node_enrollment.rs` (3 live-PG tests: one-time + identity rows; four audited refusal classes; tenant-admin-only issuance). Full live-PG regression + two-host demo green; offline suites green; clippy clean; `make gate` 13/13. Decision recorded: `docs/decisions/2026-09-06_node-enrollment.md` (`answers:`).

> The Phase-0/early-Phase-1 histories rotated into git history on 2026-09-07
> at the README-STABILITY 96,000-byte threshold (`git log -- CHANGELOG.md` is
> the query path).

Changelog-style summary of completed work + its validation (internal continuity surface;
the immutable audit trail proper is `git log` — memory layer D). Newest first.

> The Phase-0 RB-SEED and WP/G0 histories rotated into git history on
> 2026-09-07 at the
> README-STABILITY 96,000-byte threshold (`git log -- CHANGELOG.md` is the
> query path).
