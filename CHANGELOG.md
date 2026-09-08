# CHANGELOG.md

> Entries older than `2026-09-06` are rotated into the git history (the
> README-STABILITY rotation threshold) — `git log --follow CHANGELOG.md`
> carries the full record.

## 2026-09-08 — The compatibility matrix: the evidence-bound fill (`PHASE-8.4.2`)

- `docs/compatibility-matrix.md` — the 12 measured rows over the `.4.1` schema (the three dev adapters × the conformance suite, the real-provider live runs NAMED `untested`, the corpus row, the R0–R2 live-roundtrip rows, the gated R3/R5/RX named `untested`) — a cell comes from a run, never from a sibling.
- `scripts/check_compatibility_matrix.sh` — the mechanical re-derivation (the matrix's sdk_version column carries the contract's CURRENT token; the cited evidence artifacts exist) — wired into the doctrine gate's project slot (`check_doctrines.project.sh`): the matrix is evidence-bound, never prose-bound. Frontier → `.4.3`.

## 2026-09-08 — The SDK contract: the version token + the resolver surface (`PHASE-8.4.1`)

- `SDK_VERSION = "1"` + the `Adapter::sdk_version()` provided method in `crates/reasonbraid-adapter/src/contract.rs` (the default keeps the three adapters source-compatible; the harness refuses a mismatch instead of guessing the semantics) — the token is the compatibility matrix's first axis; a contract bump invalidates the qualifications.
- The resolver surface moved to the SDK home: `reasonbraid-adapter::resolver` (the `ResolverAdvertise` type + the ADR-018 vocabulary consts + the `isolation_error` validation) — the server's capability registry re-imports the single shape (the dependency promoted from dev-dependencies). The acquisition-execution trait is the named `.4.4` follow-on.
- The matrix schema accepted (`docs/decisions/2026-09-08_sdk-compatibility-matrix-schema.md`, top-level `answers:`): the six columns, the measured-only fill rules (the explicit `untested`, never blank), the fixture corpus as the replay oracle. Frontier → `.4.2`.

## 2026-09-08 — The adapter/resolver SDK census at the seams: the contract ships, the versioned surface is the greenfield (`PHASE-8.4`)

- The census (no code): SHIPPED — the adapter CONTRACT (`crates/reasonbraid-adapter/src/contract.rs`: the `Adapter` trait + the contract types), the three dev adapters, the §19.4 conformance harness + the failure-fixture corpus, the six-box qualification checklist, the server-internal resolver registry (`resolvers.rs`), the ADR-027 signing machinery (`rb-release-manifest` + the five-rung ladder vocabulary). GREENFIELD — the VERSIONED SDK surface (the adapter contract is crate-internal; the resolver side has no public trait), the compatibility MATRIX, the third-party CERTIFICATION gate, and the ADR-027 ladder's LOAD-side verification (the rungs exist as the vocabulary; the downloaded-adapter check machinery does not). Decomposed: `.4.1` the SDK contract → `.4.2` the matrix → `.4.3` the certification → `.4.4` the allowlist. Frontier → `.4.1`.

## 2026-09-08 — The MCP write tools demonstrated live: the tool-path roundtrip (`PHASE-8.3.5.3`)

- The live tool-path roundtrip (`the_write_tools_roundtrip_the_qualified_gate_live` in the crate's tests, DATABASE_URL-gated): the granted `respond` through the TOOL HANDLER (the `McpTools` method over the rmcp tool shape) lands the effect + the quota use; the ungranted role's refusal surfaces as the typed tool error (`handler:unauthorized`); the unconfigured quota surfaces the fail-closed (`quota_unconfigured`); the `join_call` decline + the `propose_policy_change` ride the same handlers through the tools — the compatibility demonstrated at the tool path, never inferred. The guard now runs `cargo test -p reasonbraid-mcp` (the live leg rides the guard's DATABASE_URL; the offline sweep skips it). **The `.3` lane (the MCP surface) is COMPLETE.** Frontier → `.4`.

## 2026-09-08 — The three MCP write tools: the qualified profile's tool layer (`PHASE-8.3.5.2`)

- The `McpTools` handle (the `ReadTools` rename — the handle now carries the write half) in `crates/reasonbraid-mcp`: the three write tools with the typed schemas — `respond` (the principal + the tenant + the thread + the minimal `ContributePayload` WITHOUT the tenant — the seam injects it), `join_call` (the kind + the optional decline reason), `propose_policy_change` (the flat ProposalInput fields). NO schema names a token field — the tokens never enter the thread content; the remote MCP metadata never grants authority. The handlers call the `.3.5.1` seam (`mcp_write_internal`) and surface the gate's typed refusals as the tool errors (the family + the message); the per-verb LOCAL grants + the audit ride the handlers.
- The conformance fixtures: the six-tool router (exactly the three reads + the three writes — the `.3.3` write-names-refuse pin is superseded), the write-schema goldens, the token-exclusion check. The advanced contribution fields + the extended response vocabulary are the named follow-on (the minimal demonstration profile). Frontier → `.3.5.3`.

## 2026-09-08 — The MCP write-half's qualified gate + the quota binding: the same handlers, the call-volume bound (`PHASE-8.3.5.1`)

- The `mcp_write` seam (`crates/reasonbraid-server/src/mcp_write.rs`): the qualified gate adds ONLY the controls ADR-024 names — the ENROLLMENT binding (the tool's principal must be the tenant's recorded principal, via the SAME `reader_tenant` the HTTP handlers run) and the PER-PRINCIPAL QUOTA (the `.1.3.2` `principal` scope, re-opened: the SAME fail-closed `check_in_tx`; the quota counts the ADMITTED CALLS; an exhaustion denial COMMITS its row — a refusal is never silent). The three delegates ride the SAME handlers: `respond` over the thread-command pipeline (the idempotency → the `thread_contribute` grant → the domain → the audit, with the deterministic replay key + the seam-injected tenant), `join_call` over the extracted `respond_to_call_core`, `propose_policy_change` over `register_proposal` — the per-verb LOCAL grants + the audit ride the handlers, never a new authority path.
- The quota binding: migration `0051_mcp_principal_quota.sql` (the 0047 backfill pattern — every existing principal gets the dev default 1000 write calls/hour) + the row creator at the enroll + the card-import paths (the identity row implies its quota row — the fail-closed check never meets a new principal unbound).
- The seam exposures in `api.rs`: `run_thread_command` returns `(StatusCode, Value)` (the three HTTP call sites wrap); `respond_to_call_core` extracted; `request_hash`/`CommandTarget`/`reader_tenant` pub(crate). The lib seam `mcp_write_internal` carries the gate + the delegates for the `.3.5.2` tools. The live suite `tests/mcp_write.rs` (the guard's 29th): the four measured legs (the enrollment refusals, the use/denial counts + the fail-closed, the granted respond with the audited allowance + the handler-refused ungranted role, the call decline + the proposal). Frontier → `.3.5.2`.

## 2026-09-08 — The MCP write-half census at the seams: the handlers ship, the qualified gate is the greenfield (`PHASE-8.3.5`)

- The census (no code): the three write handlers SHIP — `respond` rides `thread.contribute` over the thread-command pipeline (the idempotency → the `thread_contribute` authz → the domain → the audit), `join_call` rides the call-respond verb (the enrolled-ROLE-only gate + the eligibility re-resolution), `propose_policy_change` rides the policy-proposal verb (the enrolled-principal gate + `register_proposal`); the per-verb LOCAL grants + the audit ride the same pipelines. The gaps: no MCP write seam (the qualified profile's re-expression), the per-principal quota UNBOUND (the machinery ships scope-generic — `SCOPE_PRINCIPAL` + the fail-closed `check_in_tx` — but no row creator + no caller: the `.1.3.2` named deferral's trigger FIRES), and the write tools OFF (the router refuses the three write names). Decomposed: `.3.5.1` the qualified gate + the quota binding → `.3.5.2` the three write tools → `.3.5.3` the fixtures + the live demonstration. Frontier → `.3.5.1`.

## 2026-09-08 — The MCP listen-stream durability: the durable state stays in ReasonBraid (`PHASE-8.3.4`)

- The ADR-024 listen-stream contract's machinery (migration `0050_mcp_listen_state.sql` + `crates/reasonbraid-server/src/mcp_listen.rs`): the listen stream is the EPHEMERAL transport state; the durable state — the subscription, the last accepted ReasonBraid cursor, the delivery ids, the 64-id dedup window — stays in REASONBRAID. `record_delivery_in_tx` does the FOR UPDATE dedup check (the replay SKIP leaves the cursor unchanged; the first delivery registers; the accepted delivery advances) in the caller's transaction, so the state commits WITH the delivery's effects. The reconnect ritual: reauthorize → recreate → reconcile → resume from the OWN cursor via the pure `resume_plan` (the possible-gap flag names the no-replay condition — never stronger than the upstream proves). The lib seam `mcp_listen_internal` carries the four names for the `.3.5` transport (the re-export also resolved the guard's build warnings — the pub-in-private-mod dead-code lint). The live suite `tests/mcp_listen.rs` (the guard's 28th): the register → the duplicate-skip → the cursor-advance → the state-read legs; the FK-purge ripple handled. Frontier → `.3.5`.

## 2026-09-08 — The MCP read-half: the rmcp-pinned read tools + the conformance fixtures (`PHASE-8.3.3`)

- The new `crates/reasonbraid-mcp` (the §9.6 name) over the **rmcp 3.2.0** exact pin (`default-features = false` + the explicit `macros`/`server`/`transport-async-rw` — the MCP 2026-07-28 baseline is the SDK's native target). The three READ tools (`get_thread`, `list_inbox`, `get_policy_bundle`) ride the SAME queries + the SAME authorization as the HTTP handlers (the principal rides the tool's argument — the dev-profile trust shape; the thread read runs the per-reader classification); the write tools stay OFF until the qualified profile. The conformance fixtures: the offline tests pin the tool ROUTER (exactly the three read tools — the write names refuse), the input SCHEMAS (the principal + the targets), and the principal wire-space bound. The supply-chain gate caught the darling split (the rmcp-macros' 0.24 vs the derive_builder family's 0.20 — the reviewed skip rows). The Streamable-HTTP transport + the live roundtrip ride `.3.4`. Frontier → `.3.4`.

## 2026-09-08 — The MCP SDK census + the pin decision (`PHASE-8.3.2`)

- The census (the crates.io checks): the OFFICIAL `mcp-server`/`mcp-client` crates are the STALE 0.1.0 (2025-02-27 — the abandoned early SDK; the "official" name no longer means the active line); the official SDK's ACTIVE home is **`rmcp` 3.2.0** (2026-08-31, Apache-2.0, Rust 1.88, the modelcontextprotocol/rust-sdk repository — the full feature map including the `reqwest-tls-no-provider` option for the workspace's provider rule). The pin: `rmcp = "=3.2.0"`, `default-features = false`, the explicit feature list lands with the `.3.3` read-half code; the protocol profile: the MCP 2026-07-28 baseline + the Streamable HTTP transport. No code. Frontier → `.3.3`.

## 2026-09-08 — ADR-024: the MCP surface — the same handlers, the read-first split, the listen stream is transport state (`PHASE-8.3.1`)

- ADR-024 accepted (`docs/adr/024-mcp-interoperability-surface.md`, top-level `answers:`): the tools ARE the same command handlers (the MCP surface is the HTTP verbs' re-expression — never a new authority path; a tool no handler backs is NOT exposed); the READ/WRITE split (the read tools first; the write tools as the QUALIFIED profile — the enrolled principal, the per-verb local grants, the per-principal quota re-opens the `.1.3.2` `principal` scope, the audit; the remote metadata never grants authority, the tokens never enter the thread content); the listen stream is the EPHEMERAL transport state (the durable subscription/cursor/delivery-ids/dedup stay in REASONBRAID; the reconnect reauthorizes → recreates → reconciles → resumes from the OWN cursor → surfaces the possible-gap — never stronger than the upstream proves); the version profile (the official SDK, the tested release pin, the independent conformance fixtures). No code. Frontier → `.3.2`.

## 2026-09-08 — The MCP census: the parking-lot trigger fires (`PHASE-8.3`)

- The census at the seams: nothing ships for MCP (the §9.6 surface is the greenfield); the contract is precise — the tool vocabulary (`ask_network`/`get_thread`/`respond`/`join_call`/`list_inbox`/`propose_policy_change`/`get_policy_bundle`) maps to the SAME command handlers as HTTP, the resources (the timelines/evidence/policy sets), the client role, the listen-stream contract (the ephemeral transport state + the ReasonBraid-owned durable subscription/cursor/dedup — the reconnect reauthorizes + recreates + reconciles + resumes from the own cursor + surfaces the possible-gap); ADR-024 is reserved. The parking-lot entry (the director's 2026-09-08 brainstorm) is this lane's trigger: the READ half first, the WRITE half as the qualified capability profile (the grant-scoped principal + the per-principal quota — the `.1.3.2` named deferral re-opens). Decomposed: `.3.1` the census + ADR-024 → `.3.2` the SDK pin → `.3.3` the read-half → `.3.4` the listen-stream durability → `.3.5` the write-half profile.

## 2026-09-08 — The A2A wire-level demonstration + the qualification (`PHASE-8.2.4`)

- The facade's suite gains the JSON-RPC 2.0 roundtrip over the REAL a2a-lf 0.3.0 wire shapes (the `JsonRpcRequest` (SendMessage) → the serialize/deserialize → the facade maps → the `JsonRpcResponse::success` → the roundtrip; the unknown-method typed `JsonRpcError` refusal — 5 tests). The tested revision: the a2a-lf 0.3.0 types + the JSON-RPC 2.0 envelope. The QUALIFICATION RECORD: the WIRE-level compatibility is DEMONSTRATED; the transport profile (the a2a-server-lf/a2a-client-lf harness) stays the named follow-on — the §9.7 "test only the profiles actually needed" rule (no external A2A peer exists in the dev profile); the broad Internet agent interoperability claim stays GATED; the gateway deployable-off. **The `.2` lane is COMPLETE** — frontier → `.3` (the MCP lane).

## 2026-09-08 — The A2A facade core: the pinned types + the recorded semantic losses (`PHASE-8.2.3`)

- The new `crates/reasonbraid-a2a`: the `a2a-lf` **0.3.0** exact pin (the core types — no features, no transport, no provider vote; the server/client crates ride the `.2.4` demonstration with the no-provider pins). The `SemanticLosses` record (the five §9.7 dimensions — the authority/budget/evidence/decision-rule/lifecycle, each recorded; a bare A2A message carries none of the local machinery, so all five record lost), `map_message` (the text survives, the external role preserves as the wire value), `map_task_request` (the external task id preserves verbatim), `response_message` (the round-trip). The 3-test offline suite runs over the REAL a2a-lf 0.3.0 types — the first demonstrated compatibility. The local-command wiring + the transport ride `.2.4`. Frontier → `.2.4`.

## 2026-09-08 — The A2A dependency census + the pin decision (`PHASE-8.2.2`)

- The census (the crates.io checks): `a2a-lf` 0.3.0, `a2a-server-lf` 0.4.3, `a2a-client-lf` 0.2.3 — all Apache-2.0, Rust 1.85, the a2aproject/a2a-rs family. The profile: the JSON-RPC/REST (the protocol binding factory's default — the only profile the slice needs). THE SUPPLY-CHAIN FINDING: the crates' DEFAULT features enable `rustls-tls` (= `reqwest/rustls` → the aws-lc-rs provider) — a provider vote, the workspace single-provider rule's third occurrence; the `.2.3` facade must pin `default-features = false` + `rustls-no-provider` + the ring-pinned rustls explicitly. The Cargo.toml/lock additions ride the `.2.3` facade (the pin lands WITH the use); the conformance revision records at the `.2.4` roundtrip. No code. Frontier → `.2.3`.

## 2026-09-08 — ADR-025: the A2A facade with recorded semantic losses (`PHASE-8.2.1`)

- ADR-025 accepted (`docs/adr/025-a2a-interoperability-surface.md`, top-level `answers:`): the facade boundary (A2A maps the compatible semantics; the message is an INPUT the local machinery evaluates — the local grants authorize, the local budgets bound, the local policy digests decide; the Agent Card confers nothing); the semantic losses are RECORDED per exchange (the authority/budget/evidence/decision-rule/lifecycle — never a silent merge); the version profile (the JSON-RPC/REST first, the exact crate versions pin, the tested conformance revision records, the pre-1.0 compatibility DEMONSTRATED by the roundtrip — never inferred); the qualification precedes the broad claims, the gateway deployable-off. No code. Frontier → `.2.2`.

## 2026-09-08 — The A2A census: the facade is the greenfield, the baseline re-checked (`PHASE-8.2`)

- The census at the seams: nothing ships for A2A (the §9.7 facade is the greenfield); the contract is precise — the facade boundary (A2A is the interoperability facade, never the governance protocol), the semantic-losses map (the authority/budget/evidence/decision-rule/lifecycle each record their loss), the Agent Card confers nothing; the official `a2a-lf`/`a2a-client-lf`/`a2a-server-lf` crates + the `a2a-cli` exist on crates.io (the 2026-09-04 baseline re-checked — the pre-1.0 compatibility must be DEMONSTRATED, not inferred); ADR-025 is reserved. Decomposed: `.2.1` the census + ADR-025 → `.2.2` the dependency pin → `.2.3` the facade → `.2.4` the compatibility demonstration + the qualification.

## 2026-09-08 — The cross-domain audit receipts: the cross-reference, never the merge (`PHASE-8.1.4`)

- Migration 0049 (`cross_domain_receipts` — the two tenants, the kind, the `remote_ref` (the remote domain's digest-pinned reference), the `local_ref` (the local record)) + `receipts.rs` (the in-transaction record — the receipt commits WITH the action) + the read surface (`GET /v1/audit/receipts?tenant_id=…`). The card import records the receipt (the remote_ref = the card's digest, the local_ref = the fresh role); the measured legs ride the cards suite (the guard's 27th). The receipts CROSS-REFERENCE: the remote reference verifies against the REMOTE domain's records, the local chain stays the local truth. The guard's first runs caught the receipts-FK purge ripple (the 18 lists). **The `.1` lane is COMPLETE** — frontier → `.2` (the A2A interoperability).

## 2026-09-08 — The portable agent cards: the digest-pinned export + the four-rung import (`PHASE-8.1.3`)

- `cards.rs` (the `AgentCard` — the origin identity + the §10.1 profile + the canonical field order + the `sha256:<hex>` digest) + the two routes: the EXPORT (`GET /v1/profiles/{role_id}/card` — the role or its tenant admin mints the portable form) and the IMPORT (`POST /v1/profiles/cards/import` — the ADR-027 ladder: the digest rung (the re-derivation), the compatibility rung (`agent-card/1`), the allowlist rung (the EFFECTIVE recruitment agreement — the `.1.2` machinery), the capability rung (the fresh local role + the boundary-checked DEFAULT grant — the card's self-asserted capabilities never confer authority — + the imported profile via the content-addressed path). The measured suite (the guard's 27th) proves the export, the refusals, the clean import, and the default-grant-only invariant. The guard's first runs caught the profile-FK purge gaps. Frontier → `.1.4`.

## 2026-09-08 — The federation machinery: the named agreement widens exactly what it names (`PHASE-8.1.2`)

- Migration 0048 (`federation_agreements` — the NAMED pairing with the `directory_visibility` + the `recruitment` scopes and the proposed/accepted/revoked state) + `federation.rs` (the propose/accept/revoke verbs + the EFFECTIVE check: BOTH directions accepted AND both carry the scope) + the three tenant_admin-gated routes. The widening binds at the reader-classification seam: a network reader under the effective directory-visibility agreement reads the TENANT view; no agreement (or a one-sided/revoked one) stays the pseudonym; a third tenant never inherits. The measured suite (the guard's 26th) proves the five legs; the recruitment scope ships as the vocabulary (the call-panel widening is the named deferral). The guard's first runs caught the FK-purge ripple (the table joined all 15 purge lists) + the migration-boundary move. Frontier → `.1.3`.

## 2026-09-08 — ADR-026: the federation trust agreement (`PHASE-8.1.1`)

- ADR-026 accepted (`docs/adr/026-federation-trust-agreement.md`, top-level `answers:`): the federation is EXPLICIT — the agreement record is the single capability source (no record, no cross-domain effect, never a transitive default); the remote domain NEVER authorizes local effects (the remote agreement vouches for the remote half, the local grant acts locally — the §25 kill line as the invariant); the visibility rides the shipped scopes + the opt-in; the remote recruitment is the agreement-scoped opt-in; the cross-domain receipts CROSS-REFERENCE, never merge; the portable cards verify through the ADR-027 five-rung ladder. No code. Frontier → `.1.2`.

## 2026-09-08 — The federation lane opens: the `.1` block lifts (`PHASE-8.1`)

- The census at the seams: the block ("stable trust and compatibility contracts") LIFTS — the trust contracts (the workload identity, the `.2.4` enrollment policy, the revocation drill, the audit groundwork) and the compatibility contracts (the wire envelopes, the ADR-027 adapter ladder) ship + are measured by the Phase-7 guard of record. The lane's five pieces split: the visibility scopes ship in their intra-tenant form (the network-pseudonym class + the ADR-034 explicit opt-in), the remote recruitment, the portable cards, and the cross-domain receipts are the greenfield; ADR-026 is reserved. Decomposed: `.1.1` the census + ADR-026 → `.1.2` the visibility + the recruitment → `.1.3` the portable cards → `.1.4` the receipts. The PHASE-8 tree is `active` — frontier → `.1.1`.

## 2026-09-08 — The G6–G7 gate package — PHASE 7 IS CLOSED (`PHASE-7.5.2`)

- The gate record (`docs/decisions/2026-09-08_phase7-gate-record.md`, top-level `answers:`): **G6–G7 NOT MET for the Internet exposure — Met as the hardening-machinery exit for the LAN profile.** The exposure stays UNCLAIMED (the §25.1 kill/pivot holds by design — the phase never exposed anything); the seven shipped §16.12 lines cite the guard; the three external gaps (the reviewed threat model, the prompt-injection suite, the pen-test) are the named preconditions. The subtraction record (S-1…S-12: the exposure, the review, the prompt-injection suite, the pen-test, the app role, the external stores, the confidential evaluator, the scaling, the dependency SBOM, the export disposition, the regions, the scale/human game days — each with its trigger), the explicit unsupported matrix, and the evidence manifest (the guard of record: rc=0 25 live suites + the demo, rc=0 70 offline suites, the load-harness run, the signed release). **The PHASE-7 tree is COMPLETE** — the next executable work is `PHASE-8.1`.

## 2026-09-08 — The G6–G7 evidence census: the ten rows with their citations (`PHASE-7.5.1`)

- The per-row map (the guard of record: rc=0, 25 live suites + the demo; rc=0, 70 offline suites): (2) the enrollment/rotation/revocation/isolation — the channel/enrollment/replacement/mTLS/RLS suites; (3) the non-escalation + the confused-deputy — the four adversarial legs; (4) the SSRF/rebinding/redirect/archive-bomb — the ssrf + the fetcher + the extraction suites; (6) the dependency/SBOM/signing pipeline — the release-tool suite + `make release` rc=0 + deny/secret-scan green; (7) the restore + the compromised-key recovery — the backup/restore suite + the replacement drill + the signing-key runbook; (8) the rate-limit/breaker/storm — the quota + the budget + the storm controls; (10) the runbooks/disclosure — the thirteen-family catalogue + SECURITY.md. The three GAPS (the reviewed threat model, the prompt-injection suite, the pen-test) are the exposure's preconditions, each with its trigger. The §25.1 reading: the exit claims the HARDENING MACHINERY + the LAN profile. No code. Frontier → `.5.2`.

## 2026-09-08 — The G6–G7 census: seven of the ten lines ship, the three external gaps hold the kill/pivot (`PHASE-7.5`)

- The census maps the §16.12 ten-line gate against the shipped lanes: SHIPPED — the authenticated enrollment/rotation/revocation/tenant-isolation tests, the non-escalation + the confused-deputy suite, the SSRF/rebinding/redirect/archive-bomb suite, the dependency/SBOM/provenance/signing pipeline, the restore + the compromised-key exercise, the rate-limit/breaker/storm tests, and the runbooks/contacts/evidence/disclosure. THE THREE EXTERNAL GAPS: the externally reviewed threat model, the prompt-injection action-boundary suite, and the penetration test. The §25.1 kill/pivot therefore HOLDS — the Internet exposure stays UNCLAIMED; the exit names the hardening machinery + the LAN profile with the three gaps as the exposure's preconditions. Decomposed: `.5.1` the evidence census → `.5.2` the gate package.

## 2026-09-08 — The game-day catalogue + the pen-test stance: the guard is the exercise (`PHASE-7.4.3`)

- `docs/decisions/2026-09-08_game-days-pentest.md` (top-level `answers:`): the shipped game days ARE the guard — the eight mapped exercises (the replacement drill, the restore, the demo's kill points, the migration upgrade, the hostile + the non-escalation suites, the load harness, the adapter conformance), each cross-referenced to a runbook's closure tests; the named gaps are scale/human-shaped (the multi-node churn, the tabletop, the load-driven game — each with its trigger); the pen-test stance: every finding becomes a task-tree leaf with an owner + a regression test, the remediation lands before the gate closes, and the record stays empty until the test runs (no invented findings). **The `.4` lane is COMPLETE** — frontier → `.5` (the G6–G7 exit).

## 2026-09-08 — The §18.6 runbook set: the thirteen-family catalogue completes (`PHASE-7.4.2`)

- The twelve remaining runbooks land in `docs/runbooks/` — the provider outage/ambiguous charge, the credential compromise, the notification storm, the runaway budget, the poisoned resource, the database failover/loss, the object loss, the Git/DB publication mismatch, the signing-key incident, the cross-tenant exposure suspicion, the audit-chain break, and the rollback/suspension/DR — each in the §18.6 shape over the SHIPPED controls (the real verbs, surfaces, and guard suites as the closure tests) with the honest limits stated: the failover record names the missing failover machinery (the restore path IS the control), the signing-key record rides the `.2.3` manifest tool, the cross-tenant detection is the re-derivation probe, the audit-chain record names the ADR-022 deferral. No code. Frontier → `.4.3`.

## 2026-09-08 — The load harness: the capacity feeder with the first measured run (`PHASE-7.4.1`)

- `scripts/load_harness.sh`: the scripted concurrent driver — the server boots against the caller's database, then N `thread.contribute` commands at C workers (each a fresh request id + idempotency key — the full claim → authorize → validate → apply path). The per-request `status seconds` lines land in `target/load/latencies.txt`; the summary prints the p50/p95 + the throughput and the exit gates on every command committing. The MEASURED run: 200 commands at 8 workers → 1.057s wall, ingress→commit p50 0.0033s / p95 0.0079s, 189.2 commands/s, 0 failures (`target/load_harness_run.log`) — the `.3` criteria's first feed (the aggregate-write seam's trigger measurement). The worker-throughput + the channel-latency legs are the named follow-ons. The first run caught the bare-`wait` trap (it also joins the backgrounded server — the fix waits the worker PIDs only). Frontier → `.4.2`.

## 2026-09-08 — The capacity-and-incident census: the `.4` lane opens at the seams (`PHASE-7.4`)

- The census, measured: the runbook set is ONE record of the §18.6 thirteen-family catalogue (`node-lost-replaced.md`); the load harness does not exist (the bench harness is the deliberation benchmark — the `.3` criteria's trigger measurements have no feeder); the game-days are the three shipped exercises (the replacement drill, the restore exercise, the demo's kill points) with the remaining families as the named gaps; the pen-test is EXTERNAL (the remediation rides its findings). Decomposed: `.4.1` the load harness → `.4.2` the runbook set → `.4.3` the game-day catalogue + the pen-test record.

## 2026-09-08 — The coordinator extraction criteria: nothing scales until a measurement names the bottleneck (`PHASE-7.3`)

- The census: the coordinator is the deliberate SINGLE-WRITER design (ADR-002's named property); the measurements are catalogue-named-not-instantiated (the only empirical number is SLO-5, the issuance baseline). The record (`docs/decisions/2026-09-08_coordinator-extraction-criteria.md`, top-level `answers:`) makes the mandate mechanical: the extraction trigger is a MEASUREMENT, never a hunch — five seams (the aggregate writes, the outbox worker, the node channel, the CA issuance, the evaluation) each name their `.4` trigger measurement + their horizontal form over the shipped machinery (the claim keys, the lease/fencing, the node keying — a re-arrangement, never a rebuild); zero extractions today. No code. Frontier → `.4`.

## 2026-09-08 — The public-enrollment contract: the vetting is re-derivation, the exposure stays a qualified profile (`PHASE-7.2.4`)

- `docs/decisions/2026-09-08_public-enrollment-contract.md` (top-level `answers:`): the enrollment is a STAGED vetting ladder (the identity claim → the capability declaration → the operator's acceptance → the cert issuance; the public form adds ONE stage — the claim VETTING, re-derived never trusted); the suspicion → the quarantine is the OPERATOR's typed action over the shipped machinery (the §16.11 counters + the quarantine verb + the evidence rule — the policy connects them, nothing new builds); the revocation propagation inherits the shipped ladder verbatim; the exposure is a QUALIFIED profile under the `.5` gate (ADR-034's qualified-surface rule — never an experimental default). **The `.2` lane is COMPLETE** — frontier → `.3`.

## 2026-09-08 — The signed release manifests: the digest-pinned manifest + the Ed25519 signature (`PHASE-7.2.3`)

- The new `crates/reasonbraid-release-tool` (`rb-release-manifest`): `keygen` (the release identity key — raw PKCS8 DER, 0600, never overwrites; gitignored), `generate` (the per-binary `sha256:<hex>` digests + the canonical manifest — the fixed field order + the sorted binaries map — + the Ed25519 signature over the exact bytes at `<out>.sig`), `verify` (the signature over the manifest's exact bytes + the RE-DERIVED digests — a changed binary or a tampered manifest refuses). `make release` gains the step end-to-end (the keygen on first use → the generate → the verify; the measured run: 4 binaries signed + verified). The offline roundtrip suite proves the refusals. The dependency-level SBOM (the SPDX/CycloneDX graph) is the named deferral — the manifest is the artifact-level SBOM. Frontier → `.2.4`.

## 2026-09-08 — The disclosure + the supported-version policy (`PHASE-7.2.2`)

- `SECURITY.md` lands: the reporting path names the ACCOUNTABLE OWNER (no invented public channel — the repo is private, the ADR-001 gate); the vetting rides the claim-verification discipline (a report is re-derived, never trusted); the embargo is the honest private-repo shape (the fix ships with its leaf + its regression test + its disclosure note); the supported-version window (the latest + the previous minor) begins at the first public release together with the disclosure channel + the CVE pipeline. The README's stale Status block (Phase 6.1) is fixed to the Phase-7 state + the security row added. No code. Frontier → `.2.3`.

## 2026-09-08 — ADR-027: the signing-and-distribution contract (`PHASE-7.2.1`)

- ADR-027 accepted (`docs/adr/027-signing-and-distribution.md`, top-level `answers:`): ONE Ed25519 release identity per channel (the dev placement — the releaser's key; the protected identities + the reproducible builders are the named deferrals); the release MANIFEST is the single verification unit (the per-binary `sha256:<hex>` digests + the manifest digest + the Ed25519 signature over the canonical JSON — the ring provider); the downloaded-adapter verification is the ORDERED, FAIL-CLOSED ladder (allowlist → digest → signature → API compatibility → capability manifest — a failure refuses at its rung); the shipped dev adapters satisfy the ladder by construction. No code. Frontier → `.2.2`.

## 2026-09-08 — The software-supply-chain census: the `.2` lane opens at the seams (`PHASE-7.2`)

- The census, measured: the quarantine + the revocation propagation ship in their LAN forms (the `.2.4` two-way quarantine, the `.1.3.3` evidence rule, the cert revocation + the epoch fence); ADR-027 ("plugin/adapter signing and distribution") is RESERVED with no record; the SBOM/provenance, the signed updates, and the disclosure policy are the greenfield (no SECURITY.md); the public-enrollment surface is the greenfield BY DESIGN (the `.5` kill/pivot forbids exposing remote enrollment with an incomplete gate). Decomposed: `.2.1` the census + ADR-027 → `.2.2` the disclosure + the supported-version policy → `.2.3` the SBOM + the signed release artifacts → `.2.4` the public-enrollment contract.

## 2026-09-08 — The classification-driven controls: the confidential dispatch refuses (`PHASE-7.1.4.3`)

- The evaluator-access control binds at the dispatch (the single choke point — the accept/challenge/auto-initiation paths): `Classification::has_qualified_evaluator` (the dev registry qualifies `general` only), the in-transaction gate, and the typed `classification_unqualified` (409, in the status map so the stored rejection replays the original status). A confidential thread stays creatable + inspectable — the control refuses the PROVIDER use, never the thread. The measured suite (the guard's 25th): the confidential accept refuses with the typed code + no work item lands (the invitation-iff-work invariant holds); the general dispatch stays intact. The retention/export/region controls: the census found no decision point for the retention (no thread-classified data enters the snapshot store) — the `.1.4.1` "binds at the sweep" is revised to the trigger-named deferral. **The `.1` lane is COMPLETE** — frontier → `.2` (the public-node enrollment + the quarantine).

## 2026-09-08 — The secret-store declared profiles: the registry is the seam (`PHASE-7.1.4.2`)

- `secret_store.rs`: the declared registry — `DECLARED_PROFILES` (the shipped `dev_database`), `SecretStore::resolve` (the boot-time seam; the undeclared name is the typed refusal naming itself + the declared set), `load_ca_material` (the CA row read THROUGH the store). `ca::ensure_server_ca_with_store` routes the CA material through the resolved profile (the inline SELECT is gone — one read path); the `ensure_server_ca` convenience keeps the tests on the SAME implementation. `rb-server` gains `--secret-store-profile` (default `dev_database`), resolved once before the CA load — the undeclared profile refuses the boot, never a silent fallback. The external stores join as new registry entries (the named extension point). Measured: the offline unit suite (the resolution + the refusal, 3/3); the read-through routing rides every live suite + the demo. Frontier → `.1.4.3`.

## 2026-09-08 — The declared-profile contract: the registry is the only seam (`PHASE-7.1.4.1`)

- `docs/decisions/2026-09-08_declared-profiles-secrets-classification.md` (top-level `answers:`): the secret store is a DECLARED PROFILE — the registry is the only seam, the shipped `dev_database` (the plaintext rows + the hashed node secret) is the honest dev stance named, and the undeclared store is the typed `secret_store_unconfigured` refusal. Each classification control refuses at ITS decision point: the evaluator control at the dispatch (no confidential-qualified evaluator in the dev profile → the typed refusal), the retention at the sweep; the region/export controls have no decision point yet — the named deferrals. The confidential classification stays creatable (the controls refuse the provider use, never the thread). No code. Frontier → `.1.4.2`.

## 2026-09-08 — The secrets-and-classification census: the declared profiles are the greenfield (`PHASE-7.1.4`)

- The census at the seams, measured: the dev profile's secrets are the plaintext rows (`server_ca`, the enrollment tokens) + the HASHED node secret (`node_keys` — the handshake compares the digest); no store interface. The `Classification` (general/confidential) is RECORDED-ONLY — the Phase-1 deferral stands, and the ADR-034 "silent general" is the shipped state. No region/export machinery; the snapshot `retention_class` is a free string, never classification-driven. Decomposed: `.1.4.1` the declared-profile contract → `.1.4.2` the secret-store profiles → `.1.4.3` the classification-driven controls (the dev subset).

## 2026-09-08 — The quarantine preserves the evidence: the retention never deletes a quarantined row (`PHASE-7.1.3.3`)

- The census found the retention gap: a dead-lettered row is acknowledged BY DEFINITION, so the prune's age-based delete swept the quarantined rows — the disposition destroyed the evidence the §16.11 rule exists to protect. Fixed: the prune gains `AND quarantined_at IS NULL`. The contract (`docs/decisions/2026-09-08_quarantine-preserves-evidence.md`, top-level `answers:`): the quarantine is a ROW FACT; the retention never deletes it; the replay re-arm clears the mark (the delivery state), never the evidence; the prune stays the only age-based removal with the measured receipt. The measured suite (`tests/quarantine.rs`, the guard's 24th) proves the quarantined row + its reason survive while the sweep still removes its target. **The `.1.3` lane is COMPLETE** — frontier → `.1.4`.

## 2026-09-08 — The quotas: windowed per-key ceilings with recorded denials (`PHASE-7.1.3.2`)

- Migration 0047: `usage_quotas` (the per-key WINDOWED ceilings — the tenant/principal/resolver/destination scopes, the ceiling + the window) + `quota_events` (the recorded uses AND denials — a refusal is never silent). The check rides the budget-machinery pattern: the in-tx `quota::check_in_tx` records a `use` under the ceiling and a `denial` + the typed refusal at it; the events commit with the guarded action. Fail-closed: a scope with no quota is the typed `quota_unconfigured` (503); the migration backfills the dev default (1000 invites/hour) and the enroll path creates it in the tenant's own transaction.
- The shipped binding: the per-tenant INVITE bound (the invitation-storm surface — `thread.invite` checks before dispatching; the refusal rides the commit-on-refusal pattern so the denial row survives). The wire gains `quota_exceeded` (429) + `quota_unconfigured` (503). The principal/resolver/destination bindings are the named deferrals. The migration_upgrade seed now writes the pre-upgrade schema's own shape (the new enroll depends on the new schema). Frontier → `.1.3.3`.

## 2026-09-08 — The RLS defense-in-depth: the fail-closed tenant claim on the command core (`PHASE-7.1.3.1`)

- Migration 0046: `ENABLE` + `FORCE ROW LEVEL SECURITY` on `aggregate_state`/`event_log`/`idempotency` with `USING/WITH CHECK (tenant_id = current_setting('app.tenant_id', true))` — fail-closed (the unset claim matches no row). The claim is the transaction-local `app.tenant_id` GUC, set as the first statement of the command transaction (`agg::claim_in_tx` + `agg::apply_fresh_in_tx`) and via the `rls::with_tenant_claim` wrapper for the inspection reads (api.rs) + the policy-lane EXISTS checks (lifecycle.rs, now tenant-threaded).
- The measured proof (`tests/rls.rs`, live — the guard's 22nd suite): a non-superuser probe role reads ZERO rows unset, sees only its tenant's rows under the claim, and a foreign-tenant INSERT is refused at the DATABASE. The honest limits + deferrals ride `docs/decisions/2026-09-08_rls-tenant-claim.md`: the dev profile's superuser bypasses RLS (the binding role change rides the deployment profile), the outbox stays exempt (the worker's cross-tenant queue), the remaining 19 tenant-keyed tables stay on the application layer. Frontier → `.1.3.2`.

## 2026-09-08 — The tenant-isolation census: the RLS + the quotas are the greenfield (`PHASE-7.1.3`)

- The census at the seams, measured: `tenant_id` is the first-layer key on the identity/authority/budget/inbox/enrollment surface; `grep -rn "ROW LEVEL SECURITY" migrations/` → nothing (the named defense-in-depth is unshipped); `grep -rln "quota" crates/ migrations/` → no machinery; the quarantine rows (`quarantined_at` + `quarantine_reason`, migration 0010) already preserve the evidence in-place but the §16.11 rule is unarticulated. Decomposed: `.1.3.1` the RLS layer → `.1.3.2` the quotas → `.1.3.3` the quarantine-preserving-evidence rule.

## 2026-09-08 — The mTLS workload identity: the TLS 1.3-only mutual-auth transport (`PHASE-7.1.2`)

- `ca::issue_serving_cert` (the ServerAuth-EKU serving leaf) + the `mtls` module: `build_server_config` (TLS 1.3-only, ring-pinned, the `WebPkiClientVerifier` over the deployment CA — the client MUST chain to it) and `build_client_config` (the CA root + the node leaf). The split holds: the transport verifies the CA membership; the fingerprint → the principal binding stays the application proof (`node_channel::verify_cert_proof` — the layered defense). The offline roundtrip proves both legs (the issued client connects + a byte; the cert-less client is refused at the transport). The production serve wiring stays the deployment-profile concern (named).
- The first test draft hit a real TLS 1.3 asymmetry — the client's `connect()` completes before the server's `certificate_required` alert arrives, so the refusal is read-side, never connect-side — promoted to `docs/decisions/2026-09-08_tls13-refusals-are-read-side.md` (the future TLS tests' contract). Frontier → `.1.3`.

## 2026-09-07 — ADR-034: the hardening contract (`PHASE-7.1.1`)

- ADR-034 accepted (`docs/adr/034-internet-hardening.md`): the Internet capability is claimed per the QUALIFIED SURFACE (never in general); the transport is context, never identity (the certificate fingerprint is the principal); the isolation is defense in depth (the RLS is the second layer); the quotas bound the abuse over the budget machinery; the secrets/regions are declared profiles. No code.

## 2026-09-07 — The Internet-hardening lane is decomposed at the census seams (`PHASE-7.1`)

- The `.1` block is LIFTED: the Phase-1 channel + the CA/certificate infra and the Phase-2 authority/budget ship; the per-principal quotas, the secret-manager integration, and the regional controls are the greenfield. Children: `.1.1` ADR-034 → `.1.2` the mTLS identity → `.1.3` the isolation + the quotas → `.1.4` the secrets + the regions.

## 2026-09-07 — The G3 gate package — PHASE 6 IS CLOSED (`PHASE-6.7.2`)

- **G3 Met as machinery, blocked as binding use**: the authority/consent/quorum/publication/correction tests are green (the clause map); the BLOCKER — the binding policy use — is discharged by subtraction per §25.1 (the exit claims the machinery; the owners' acceptance stays the open condition). The §19.8 subtraction record, the Demonstration B nine-step walk, and the evidence manifest land. The frontier moves to `PHASE-7.1` (the Internet-hardening lane).

## 2026-09-07 — The G3 evidence census (`PHASE-6.7.1`)

- The nine Demonstration-B steps each map to the shipped machinery with their records + their tests (the proposal → the `.2` records; the recruitment → Phase 3; the evidence → Phase 4; the deliberation → Phase 5; the decision + the separate approval → the `.2` chain; the projections → the `.3` compiler; the crash-surviving publication → the `.4` machine; the canary → the `.5` waves; the correction → the `.5.3` operations); the G3 clause map covers the lanes; the §25.1 reading: the exit claims the MACHINERY, never the real owners' acceptance (the kill/pivot stays open). No code.

## 2026-09-07 — The G3-exit lane is decomposed at the census seams (`PHASE-6.7`)

- The G3 gate maps over the SHIPPED lanes (the authority/consent/quorum — the `.2` decisions/approvals; the publication — the `.4` machine + the reconciliation; the correction — the `.5.3` §4.7 operations); Demonstration B's nine steps walk the machinery. Children: `.7.1` the evidence census → `.7.2` the gate package.

## 2026-09-07 — The scheduled reviews land (`PHASE-6.6`)

- Migration 0045: the seven §15.11 review triggers, the DUE evaluation over the `.5.3` records (one due review per (publication, trigger) — the dedupe; the schedule is idempotent), the due → done transition, and the outcome-trigger vocabulary back-fill (an unknown trigger is now the typed refusal at the outcome registration). The verbs: `POST /v1/policy-reviews/schedule`, `GET /v1/policy-reviews`, `POST /v1/policy-reviews/{id}/done`. Measured: policy 11.

## 2026-09-07 — The drift and the corrections land — the `.5` lane is COMPLETE (`PHASE-6.5.3`)

- Migration 0044: the drift records (the six §15.10 categories over the desired/observed pair), the §4.7 corrections (the authority-grant re-check; the suspension/waiver REQUIRE the expiry; the supersession links the old; the retraction NEVER deletes — the correction is a new row), and the §15.11 outcome records. The verbs: `POST`/`GET /v1/policy-drift`, `/v1/policy-corrections`, `/v1/policy-outcomes`. Measured: policy 10. **The `.5` lane (the target deployment) is COMPLETE.**

## 2026-09-07 — The deployment records land (`PHASE-6.5.2`)

- Migration 0043: the authority-checked targets (the closed type vocabulary), the per-target assignments over the EFFECTIVE publication (the chain gate) with the DESIRED pair (the ref + the digest), and the receipt attesting the OBSERVED digest + the state — the drift's comparison input. The verbs: `POST`/`GET /v1/deployment-targets`, `POST`/`GET /v1/deployments`, `POST /v1/deployments/{target}/{publication}/receipt`. Measured: policy 9.

## 2026-09-07 — ADR-021: the target-deployment contract (`PHASE-6.5.1`)

- ADR-021 accepted (`docs/adr/021-target-deployment.md`): the deployment is per-target waves, never globally atomic; the receipt attests the digest (never the hope); the drift is the six-way §15.10 vocabulary; the §4.7 correction authorities stay distinct (the reversal is fast, the authority is not universally lower). No code.

## 2026-09-07 — The target-deployment lane is decomposed at the census seams (`PHASE-6.5`)

- The deployment lane is the greenfield (no target/wave/receipt/drift/correction record exists); the `.4` effective publications + the digests are the inputs. Children: `.5.1` ADR-021 → `.5.2` the deployment records + the waves → `.5.3` the drift + the corrections.

## 2026-09-07 — The reconciliation matrix lands — the `.4` lane is COMPLETE (`PHASE-6.4.3.3`)

- The `reconciler` module: the pure `reconcile` function over (the DB state, the observed Git state, the expected id) → the six §15.8 actions (the idempotent retry, the verify-and-advance, the stop-and-alert, the freeze-and-repair, the **quarantine-and-adjudicate — never a silent promote**, the out-of-band alert); the kill-point tests map every matrix row + prove the idempotency (the same pair yields the same action). Measured: reconciler 3. **The `.4` lane (the signed canonical publication) is COMPLETE.**

## 2026-09-07 — The Git publication half lands (`PHASE-6.4.3.2`)

- The `publisher` module (the gix plumbing — no CLI): the blobs + the filename-sorted tree + the root commit, the staging branch, the fetch-back verification (the re-derived digest), the IMMUTABLE publication ref (the written-once — the re-publish is the typed refusal), and the EFFECTIVE channel via the compare-and-swap (the stale expectation is the typed `CasMismatch`). The `POST /v1/policy-publications/{id}/publish` verb drives the half and marks the record effective with the ref ids. Measured: publisher 2 + policy 8.

## 2026-09-07 — The publication-store contract (`PHASE-6.4.3.1`)

- The decision record (`docs/decisions/2026-09-07_publication-store-contract.md`): the LOCAL bare repository (the remote-publication profile is a named deferral), the three-ref scheme (the staging branch, the immutable publication ref — written once, the effective channel via the compare-and-swap with the expected old id), and the gix write path (no git CLI). No code.

## 2026-09-07 — The Git-publication half is decomposed at the census seams (`PHASE-6.4.3`)

- The write half is the greenfield (the Phase-4 R1 pack only ACQUIRES — no commit/ref-write path exists; the reconciler exists nowhere); the `.4.2` publication records are the matrix's DB half. Children: `.4.3.1` the publication-store contract → `.4.3.2` the Git publication half → `.4.3.3` the reconciliation matrix + the kill-point tests.

## 2026-09-07 — The publication records land (`PHASE-6.4.2`)

- Migration 0042: the publication is its own aggregate row (never folded into the approval) with the chain-verified references (the proposal must be APPROVED, the decision + the approval must belong to it, the projection must exist), the manifest digest, and the typed state machine (staged → effective with the Git object ids | failed with the reason — never a skip). The verbs: `POST`/`GET /v1/policy-publications` + `/effective` + `/failed`. Measured: policy 7.

## 2026-09-07 — ADR-020: the canonical-publication contract (`PHASE-6.4.1`)

- ADR-020 accepted (`docs/adr/020-canonical-publication.md`): the publication is the nine-step §15.7 state machine over the staged record; the Git refs are the publication truth with the compare-and-swap idempotency; the reconciliation is the six §15.8 rules (the never-silent-promote); the signatures ride the manifest digest. No code.

## 2026-09-07 — The publication lane is decomposed at the census seams (`PHASE-6.4`)

- The inputs + the substrate ship (the decisions/approvals, the byte-identical projections, the Git machinery, the CA keys, the transactional outbox); the publication records + the reconciliation matrix are the greenfield. Children: `.4.1` ADR-020 → `.4.2` the records + the staging → `.4.3` the Git publication + the reconciliation.

## 2026-09-07 — The Codex and the Claude projections land — the `.3` lane is COMPLETE (`PHASE-6.3.3`)

- The compiler crate gains the `codex` (the AGENTS.md fragment — the backticked clause ids) and the `claude` (the CLAUDE.md fragment — the plain ids) renderers over the same stable-sorted core; the §15.5 coverage ships: the backtick escape (the harness-parse safety), the 8192-character statement ceiling (the oversized statement DECLARES itself, never truncates), the per-target byte-identical repeats. Measured: compiler 8 + policy 6. **The `.3` lane (the deterministic compiler) is COMPLETE.**

## 2026-09-07 — The compiler core lands (`PHASE-6.3.2`)

- The hermetic `reasonbraid-policy-compiler` crate (no database/network/clock): the pure `compile` function with the stable sort, the generic bundle + the `policy.lock` renderers, the escaping, the declared unrepresentable, and the ADR-011 digest over the rendered bytes. The server gains migration 0041 + the `POST`/`GET /v1/policy-projections` verbs (the resolve → the compile → the record). Measured: compiler 5 + policy 5.

## 2026-09-07 — ADR-033: the projection-compiler contract (`PHASE-6.3.1`)

- ADR-033 accepted (`docs/adr/033-projection-compiler.md`): the compiler renders the resolved set, never re-resolves; the rendering is byte-identical by construction; the unrepresentable clause is the DECLARED refusal (never a silent omission); the compiler is a separate hermetic crate; the projection digest is the publication's verification primitive. No code.

## 2026-09-07 — The compiler lane is decomposed at the census seams (`PHASE-6.3`)

- The projection compiler is the greenfield (no projection, no target vocabulary, no unrepresentable declaration exists); the `.1` resolution's clause set is its input. Children: `.3.1` ADR-033 → `.3.2` the compiler core → `.3.3` the Codex + the Claude projections.

## 2026-09-07 — The approval records land — the `.2` lane is COMPLETE (`PHASE-6.2.3`)

- Migration 0040: the approval is its OWN row with the AUTHORITY PROOF — the grant re-checked at the approval boundary (active, unexpired, held by the approver — the §4.5 identity/authority at the action time); the quorum snapshot rides the row; the approval advances the proposal (`decided` → `approved`). The verbs: `POST`/`GET /v1/policy-approvals`. Measured: policy 4. **The `.2` lane (the policy lifecycle) is COMPLETE.**

## 2026-09-07 — The proposal and the decision records land (`PHASE-6.2.2`)

- Migration 0039: the proposal is a REFERENCE (the policy version + the deliberation thread — never a copy) with the typed stage machine; the decision freezes the electorate snapshot at the action time and references a verdict of the PROPOSAL's thread (a foreign verdict refuses); one proposal, one decision (the stage gate). The verbs: `POST`/`GET /v1/policy-proposals` + `POST`/`GET /v1/policy-decisions`. Measured: policy 3.

## 2026-09-07 — ADR-032: the policy-lifecycle contract (`PHASE-6.2.1`)

- ADR-032 accepted (`docs/adr/032-policy-lifecycle.md`): the five records never fold (the discussion rides the thread, the decision/approval/publication/deployment are their own rows); the proposal is a reference, not a copy; the decision freezes the electorate snapshot at the action time; the approval re-checks the grant at the approval boundary. No code.

## 2026-09-07 — The lifecycle lane is decomposed at the census seams (`PHASE-6.2`)

- The §15.6 lifecycle records are the greenfield (no proposal/approval/decision row exists); the substrate ships: the `.1` policy registry, the Phase-5 deliberation threads (one thread, multiple decisions — the verdict kind), the Phase-2 authority model (the proofs' substrate). Children: `.2.1` ADR-032 → `.2.2` the proposal + the decision records → `.2.3` the approvals + the proofs.

## 2026-09-07 — The seven-step resolution lands — the `.1` lane is COMPLETE (`PHASE-6.1.3`)

- The `POST /v1/policies/resolve` pipeline runs the seven §15.3 steps (the authority check, the applicability filter, the dependencies/conflicts, the DAG precedence, the exception schemas, the FAIL-CLOSED binding conflict, the explanation tree); the impact map (`GET /v1/policies/{id}/{version}/impact`) is the derivable coverage. Measured: policy 2. **The `.1` lane (the semantic policy schema) is COMPLETE.**

## 2026-09-07 — The typed policy schema lands (`PHASE-6.1.2`)

- Migration 0038: the `PolicyVersion` document (the §15.1 fields — the stable clause ids, the applicability, the exception schema, the provenance) with the OWNERSHIP metadata validated against the authority model (the owning authority must be an ACTIVE grant — the label grants nothing); the ADR-011 digest shape, the semantic version, the closed lifecycle vocabulary, the unique clause ids. The verbs: `POST`/`GET /v1/policies`. Measured: policy 1.

## 2026-09-07 — ADR-019: the semantic-policy contract (`PHASE-6.1.1`)

- ADR-019 accepted (`docs/adr/019-semantic-policy.md`): the policy is a versioned digest-pinned document (never prose); the ownership is the authority binding (the label grants nothing); the `PolicySetVersion` is a lock manifest; the resolution is the seven §15.3 steps, fail-closed, with the explanation tree riding the result. No code.

## 2026-09-07 — The semantic-policy lane is decomposed at the census seams (`PHASE-6.1`)

- The `.1` block is LIFTED: the authority (Phase 2), the deliberation (Phase 5 — now closed), the Git/object consistency (Phase 4), and the correction model all ship; the policy schema itself is the greenfield. Children: `.1.1` ADR-019 (the queue's canonical-policy-schema item) → `.1.2` the typed `PolicyVersion` + the registry → `.1.3` the seven-step layering/precedence + the impact maps.

## 2026-09-07 — The G5 gate package — PHASE 5 IS CLOSED (`PHASE-5.6.2`)

- **G5 Met as a subtraction gate**: the "deliberation improves answers" claim is withdrawn per §25.1 (the first controlled evaluation reads H1 null on the 4-case sample); the honest-inconclusive machinery ships and tests; the benchmark thresholds ship as the instrument; the §19.8 subtraction record + the evidence manifest + the gate record land; the README's stale status line is fixed. The frontier moves to `PHASE-6.1` (the semantic policy lane).

## 2026-09-07 — The G5 evidence census (`PHASE-5.6.1`)

- The claim census: the README carries NO quality claim to retract (its claim is the mechanism one — shipped and tested); the bench's first live run reads H1 null on the 4-case differential sample (no structured workflow beat `single`; the burden 2–4×); five subtraction candidates named (S-1 the narrowed structure claim, S-2 the deferred calibration, S-3 the already-shadow learned routing, S-4 the supported cost claim, S-5 the README's stale status line); the declared domains are the seven §13.8 case classes. No code.

## 2026-09-07 — The G5-exit lane is decomposed at the census seams (`PHASE-5.6`)

- The G5 gate ships as the `.4.4` SERVICE (the calibration + the baseline/threshold gates); the exit needs the PACKAGE: the claim census (the docs' quality claims vs the evidence), the gate record, the §19.8 subtraction record, and the narrowed product claims. Children: `.6.1` the evidence census → `.6.2` the gate package.

## 2026-09-07 — The shadow recommendation lands — the `.5` lane is COMPLETE (`PHASE-5.5.3`)

- Migration 0037: the recommendation maps a class to an arm drawn from the EXISTING registered profiles (never a raise), names its `.4` evidence reference, and is recorded with `applied: false` on its face — the create boundary keeps resolving the RULE table (the shadow proof). Measured: routing 2. **The `.5` lane (the routing policy) is COMPLETE.**

## 2026-09-07 — The rule-based routing policy lands (`PHASE-5.5.2`)

- Migration 0036: the seven §13.8 rows as the built-in rules (one deterministic arm per case class); the resolution is a lookup with an append-only audit row; the create boundary applies the policy ONLY when no explicit profile is named (the explicit profile always wins; the bare thread keeps the `quick_advice` default); the arm must be a registered profile (a phantom arm fails closed). The verbs: `GET /v1/routing/rules`, `POST /v1/routing/resolve`, `GET /v1/routing/resolutions`. Measured: routing 1.

## 2026-09-07 — ADR-031: the routing-policy contract (`PHASE-5.5.1`)

- ADR-031 accepted (`docs/adr/031-routing-policy.md`): the case class is a submitted input (never a derived judgment); the rule-based policy is a deterministic table; the human authority outranks the rule (the explicit profile always wins); the learned routing is the shadow recommendation — an existing arm, recorded with its evidence, never applied, never a raise. No code.

## 2026-09-07 — The routing-policy lane is decomposed at the census seams (`PHASE-5.5`)

- The §13.8 census: the routing decision is the CLIENT's choice today (the create carries the explicit profile; the bare thread defaults to the hardcoded `quick_advice`) — no rule, no case-class vocabulary, no policy; the §13.1 built-ins map §13.8's rows. Children: `.5.1` ADR-031 → `.5.2` the rule-based policy → `.5.3` the shadow recommendation.

## 2026-09-07 — The calibration and the regression gates land — the `.4` lane is COMPLETE (`PHASE-5.4.4`)

- Migration 0035: the calibration accumulates the Brier + the confidence over the NAMED runs (each must be registered — never a fabrication); the gate records the baseline + the threshold; the evaluation compares each measured case against the baseline minus the threshold and APPENDS its result (a drop below is the typed failure with the delta — the gate never rewrites a result, it only blocks). Measured: evaluation 3. **The `.4` lane (the versioned evaluation service) is COMPLETE.**

## 2026-09-07 — The shadow routing trials land (`PHASE-5.4.3`)

- Migration 0034: the trial records the declared seed + the arms + the cohorts + the case ids, and the SERVER computes the seeded assignment (a dependency-free splitmix64 — the `std` hasher is not stable across releases, the draw must be): the same seed + cases re-draw the same assignment. The per-arm results append (never overwrite); the trial never changes production routing (the `.5` lane's decision consumes the records). Measured: evaluation 2.

## 2026-09-07 — The evaluation-service core lands (`PHASE-5.4.2`)

- Migration 0033: the versioned corpus registry (the declared 64-hex digests) + the experiment run records (the workflow arm, the corpus reference, the DECLARED seed — a non-deterministic run without one is the typed refusal — the trial count, the harness's results). The four verbs (`POST`/`GET /v1/evaluations/corpora`, `POST`/`GET /v1/evaluations/runs`) record, never re-grade: the service RECORDS, the WP7 harness MEASURES. The pg script gained the `evaluation` suite.

## 2026-09-07 — ADR-017: the evaluation-service contract (`PHASE-5.4.1`)

- ADR-017 accepted (`docs/adr/017-evaluation-service.md`): the service RECORDS, the harness MEASURES (one grading implementation); the registry is versioned + digest-pinned; the experiment records declare their seeds; the randomized routing trials are shadow-only; the cohorts are recorded labels; the calibration accumulates; the regression gate is the blocking-only G5 threshold. No code.

## 2026-09-07 — The evaluation lane is decomposed at the census seams (`PHASE-5.4`)

- The §13.7/§19.5 census: the WP7 bench harness (Phase 0) is a substantial substrate — the digest-carrying versioned corpus, the four deliberation workflows over the real Adapter contract, the deterministic grading (Brier + rubrics), the spread-bearing reports, the ScriptedAgent self-test; the GREENFIELD is the service itself (the case registry, the run records, the randomized trials, the cohorts, the calibration record, the regression gates). Children: `.4.1` ADR-017 → `.4.2` the service core → `.4.3` the routing experiments + the cohorts → `.4.4` the calibration + the gates.

## 2026-09-07 — The synthesis record lands — the `.3` lane is COMPLETE (`PHASE-5.3.3`)

- The `synthesis` record rides a `summary`-kind contribution on the `synthesize` step: the synthesizer identity, the event-log input range (validated — `1 <= from <= to <=` the thread's max version, so the transformation is re-derivable), the source links, the coverage report. The live pass caught the `.1.3`-lane gap: the create handlers resolved the profile only when named, so a bare thread's steps were EMPTY (the step gates read `none`) — both handlers now resolve always (`None` → `quick_advice`). Measured: profiles 31. **The `.3` lane (moderator/synthesizer constraints) is COMPLETE.**

## 2026-09-07 — The moderation kinds land (`PHASE-5.3.2`)

- The closed moderation vocabulary (`classify`, `request_clarification`, `propose_close`, `draft_summary`, `identify_unanswered`) rides the contribute verb: the capability-shaped fields refuse on it (the §13.5 prohibitions by construction), the action references its target via `ref_event_id` (must exist in the thread), the `moderate` step joins the step vocabulary and gates the kinds, and the action is challengeable — the appeal IS the challenge. Measured: profiles 30.

## 2026-09-07 — ADR-030: the moderation/synthesis contract (`PHASE-5.3.1`)

- ADR-030 accepted (`docs/adr/030-moderation-and-synthesis.md`): the moderation action is a contribution (never a new authority); the closed kind set refuses the capability-shaped fields — the §13.5 prohibitions hold by construction; the appealable action IS the existing challenge; the `moderate` step joins the vocabulary; the synthesis record is re-derivable derived content. No code.

## 2026-09-07 — The moderator/synthesizer lane is decomposed at the census seams (`PHASE-5.3`)

- The §13.5 census: the moderator is ZERO machinery (no role, no kind, no step — the step vocabulary has no `moderate`); the synthesizer is half-shipped (the `synthesize` step name + the `.2.4.1` minority-report shapes). Children: `.3.1` ADR-030 → `.3.2` the moderation kinds → `.3.3` the synthesis record.

## 2026-09-07 — The contribution-side execution lands — the `.2` lane is COMPLETE (`PHASE-5.2.4.2`)

- The kind vocabulary gains `evidence_request` (targets ONE claim digest of THIS thread — the JSONB scan over the server-computed records; the request is not an acquisition) and `verdict` (the judged digest + the rule + the §13.4 outcome, canonicalized); the step gates execute the ADR-016 composition (request → `evidence_request`, verdict → `adjudicate`); the round advance generalizes to the step advance (one step per round, clamped at the terminal — the blind commitment flag is its special case); the `evidence_reference` kind refuses empty refs. Measured: profiles 29. **The `.2` lane (blind-first contributions, structured claims, evidence requests, adjudication, minority reports, the twelve terminals) is COMPLETE.**

## 2026-09-07 — The close vocabulary lands (`PHASE-5.2.4.1`)

- The close outcome speaks §13.4's twelve terminals: `decided`/`inconclusive` stay accepted aliases but never persist — the event and the projection's `close_outcome` carry the canonical name; the family rule replaces the `.1.5.3` check (a decision terminal with an unresolved register is the typed refusal); the minority report rides the close event (synthesizer, input range, sources, coverage). Measured: profiles 28.

## 2026-09-07 — The close/contribute seam splits the `.2.4` leaf (`PHASE-5.2.4`)

- The `.2.4` census: the CLOSE side (a two-valued outcome vs §13.4's twelve; no minority report) and the CONTRIBUTION side (no `evidence_request`/`verdict` kinds; the `evidence_reference` kind accepts empty refs). Children: `.2.4.1` the close vocabulary → `.2.4.2` the contribution-side execution.

## 2026-09-07 — The blind-first lane lands (`PHASE-5.2.3`)

- A contribution posted during the `blind_solicit` step carries `blind: true`; the round advance is the commitment point (the step moves past `blind_solicit`, the event records `blind_committed`); the read surface serves a still-blind contribution to non-authors as `blind_until: round_advance` + the content digest — the ledger keeps the full body (a read rule, never a store rewrite); a non-author challenge of a blind contribution is the typed refusal. Measured: profiles 27.

## 2026-09-07 — The structured records land (`PHASE-5.2.2`)

- The contribute body carries `claims` (content-only input; the server computes the ADR-011 digest — the wire never supplies one); claims ride a `claim`-kind contribution only; the challenge body gains `claim_digest` (the structured objection names ONE server-derived claim of the target — a foreign digest is the typed refusal); the projection carries the `structured_claims` counter; the free-text wire stays valid. Measured: profiles 26.

## 2026-09-07 — ADR-029: the structured-deliberation contract (`PHASE-5.2.1`)

- The `.2.1` leaf accepted ADR-029 (`docs/adr/029-structured-deliberation.md`): the typed claim/objection/revision records are SHAPES over the existing verbs (never a new capability); the blind commitment point is a READ-SURFACE rule (the ledger holds the blind content from post time; the round advance commits); the evidence request is a contribution, not an acquisition; the adjudication is an attributable verdict; the minority report carries the coverage report; the close speaks §13.4's twelve terminals. No code.

## 2026-09-07 — The deliberation lane is decomposed at the census seams (`PHASE-5.2`)

- The §13.4/§13.6 census: FOUR greenfields (the blind-first visibility, the structured claim/objection/revision records, the evidence requests, the adjudication + the minority reports + the terminals) against the reusable pieces (the contribution kind vocabulary, the open_challenges + unresolved registers, the Phase-4 evidence pipeline). Children: `.2.1` ADR-029 → `.2.2` the structured records → `.2.3` the blind-first lane → `.2.4` the requests + the adjudication + the reports.

## 2026-09-07 — The workflow-profile lane is complete: the steps ride the projection (`PHASE-5.1.3`)

- The projection carries the resolved step sequence + the current index: the create seats step 0 with the registry's steps, the close advances to the terminal step — the lifecycle's own transitions are the only step transitions, so the authorization/budget/lifecycle invariants ride every step by construction.
- Measured: profiles 25 (the suite grew 24→25). **`.1` COMPLETE** — frontier → `.2` (the blind-first contributions lane).

## 2026-09-07 — The profile registry ships: the validated reference replaces the stored string (`PHASE-5.1.2`)

- `migrations/0032`: the eight §13.1 built-ins as versioned entries.
- `src/workflows.rs`: the twelve-kind step vocabulary, the three composition rules (the known kinds, the terminal last, the adjudicate-after-blind), the resolve/register/list surfaces.
- The thread's `workflow_profile` is now a VALIDATED reference: the unknown id is the typed refusal at the create boundary; the bare thread defaults to `quick_advice`; the Phase-1 enum is gone.
- Measured: profiles 25. Frontier → `.1.3` (the profile-driven execution).

## 2026-09-07 — ADR-016 is accepted: the profile composes verbs, never capabilities (`PHASE-5.1.1`)

- The workflow profile is VERSIONED CONFIGURATION over the thread aggregates; the composition invariants (no profile bypasses authorization/budget/lifecycle — an invalid profile is invalid at validation time); the eight §13.1 built-ins are the initial registry; the unknown profile is a typed refusal, never a stored string.
- Durable in `docs/adr/016-workflow-profiles.md` (top-level `answers:`). No code. Frontier → `.1.2` (the registry + the validation).

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
