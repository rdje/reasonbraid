# SIGNOFF-REPAIR: restore the roadmap's enforced guarantees

## Metadata

- Tree ID: `SIGNOFF-REPAIR`
- Status: `active`
- Created: `2026-09-09` (startup read began 2026-09-08).
- Owner: repo-local engineering; architecture decision delegated by the director.
- Parent: `PROGRAM`; prerequisite repairs for `PHASE-8.5.3`.
- Roadmap: `ROADMAP.md` §§4, 10–20, 25; existing guarantees, defect correction and qualification.
- Baseline: `9c2d2ba`; clean `main`, 269 commits ahead of the locally recorded origin/main.

## Goal

Repair the authority defect in shared registries and the related invariant failures identified by the full startup read. Every finding must be reproduced and fixed, or explicitly refuted with evidence. A historical passing suite is evidence for its exercised assertions, not a blanket production qualification.

## Read prerequisite and ownership

Roadmap: Yes. Tracked codebase: Yes. mdBook: Yes. All were read before this file, the first repository change of this session. The non-Markdown corpus was read in full: 3,276,496 Python characters including path separators, in 92 consecutive 36,000-character pages over sorted `git ls-files`. Markdown roadmap and book sources were read separately. No runtime verification was run during that read. The baseline corpus digest and file count are preserved in `docs/tasks/artifacts/signoff_review/INDEX.md`.

This tree is created before the census artifact, decision record, or live documentation changes it owns. Leaf `.1` owns those documentation changes and their verification. All later changes require their specific leaf to be active before editing. The source-review notes are candidates and source evidence; none is marked runtime-confirmed by this census.

## Execution contract

Run one bounded leaf and commit it through `COMMIT.md` before the next. Expand a leaf into smaller children before implementation if its safe scope requires it. Each code leaf must carry REPRODUCE, ROOT CAUSE, FIX, ADDRESSED, NO REGRESSION and LOCKSTEP evidence. Denial tests assert unchanged protected state as well as status; concurrency tests prove the relevant serialization. Focused checks accompany ordinary commits; full CI runs before push and selected major steps. Preserve the director's PNT instruction and approximately 300-commit push cadence.

The registry decision is explicit site-operator authority, issued only through operator-controlled tooling. Tenant administrator grants remain tenant-scoped. A site grant is bound to an actual enrollment boundary; revoking that boundary freezes its writes. Authorization and mutation serialize with revocation and produce durable audit evidence. This is the selected target contract, not a claim that it already ships.

## Task Tree

### SIGNOFF-REPAIR.1 — Review ownership and authoritative status

- Status: `done`.
- Sources / owned surfaces: `docs/tasks, MEMORY.md, LIVE_STATUS.md, README.md, docs/book`.
- Goal and acceptance: Preserve the full-read census, record the site-operator decision, correct current progress pointers, and map every finding to repair work. This leaf changes documentation only.
- Verification: passed documentation checks and controls; see Leaf .1 closure evidence. Runtime product reproduction remains owned by the implementation leaves.
- Commit: `REASONBRAID-REPAIR-0001` (resolve with `git log --grep`).

### SIGNOFF-REPAIR.2.1 — Repository-local execution environment

- Status: `pending`.
- Sources / owned surfaces: `Makefile, scripts, Cargo configuration`.
- Goal and acceptance: Derive caches, scratch, build and tool stores from the current repository; identify required read-only system/toolchain dependencies; inventory off-volume project data and use copy/verify/use/delete only for proven ownership.
- Verification: pending; capture the failing case, corrected case, and independent control in this leaf or its children before closure.
- Commit: pending.

### SIGNOFF-REPAIR.2.2 — Disposable PostgreSQL verification

- Status: `pending`.
- Sources / owned surfaces: `scripts/run_pg_tests.sh, tests pool helpers, migration_upgrade, backup_restore, rls`.
- Goal and acceptance: Refuse destructive tests against an unowned database; create isolated local cluster/database/roles, support focused suites, serialize shared fixtures, verify shutdown before cleanup, and prove failure-path residue handling.
- Verification: pending; capture the failing case, corrected case, and independent control in this leaf or its children before closure.
- Commit: pending.

### SIGNOFF-REPAIR.3.1 — Tenant-bound revocation

- Status: `pending`.
- Sources / owned surfaces: `api.rs, authority.rs revoke_grant/revoke_boundary`.
- Goal and acceptance: Reproduce a foreign grant/boundary revocation using an own-tenant admin; bind target tenant before mutation and audit; prove victim status and epoch unchanged on refusal, with legitimate revoke preserved.
- Verification: pending; capture the failing case, corrected case, and independent control in this leaf or its children before closure.
- Commit: pending.

### SIGNOFF-REPAIR.3.2 — Site-operator registry authority

- Status: `pending`.
- Sources / owned surfaces: `api.rs require_admin_any_tenant, allowlist.rs, regions.rs, operator tooling, new migration`.
- Goal and acceptance: Implement explicitly issued site authority for adapter and region mutations, deny tenant-admin escalation, check actual bound boundary and grant liveness, serialize revocation with effects, and record durable actor/action/target/reason/decision audit. Operator-controlled issuance and revocation must be tested; no HTTP enrollment minting.
- Verification: pending; capture the failing case, corrected case, and independent control in this leaf or its children before closure.
- Commit: pending.

### SIGNOFF-REPAIR.3.3 — Bound-boundary authorization and grant selection

- Status: `pending`.
- Sources / owned surfaces: `core authority, server authority.rs`.
- Goal and acceptance: Resolve a grant's actual boundary, enforce identity/tenant/subset/window correspondence, reject thread-only selectors for tenant actions, avoid latest-grant shadowing, and serialize all relevant authorization/mutation paths against revocation.
- Verification: pending; capture the failing case, corrected case, and independent control in this leaf or its children before closure.
- Commit: pending.

### SIGNOFF-REPAIR.3.4 — Delegation and cache freshness

- Status: `pending`.
- Sources / owned surfaces: `core delegation/cache, authority.rs, command envelopes`.
- Goal and acceptance: Enforce delegability, bounded depth, actor/subject participation and consent; bind replay hashes to target and authority context while preserving approved committed-replay semantics; make cached decision expiry and future-clock behavior explicit.
- Verification: pending; capture the failing case, corrected case, and independent control in this leaf or its children before closure.
- Commit: pending.

### SIGNOFF-REPAIR.3.5 — Tenant-owned administration

- Status: `pending`.
- Sources / owned surfaces: `api.rs node inbox/prune/quarantine/replay, breaker, enrollment, admin services`.
- Goal and acceptance: Use real target ownership inside the mutation transaction; replace foreign/nonexistent-node success fixtures and prove foreign reads/writes leave all affected rows unchanged; retain the approved own-tenant frozen-admin read carve-out.
- Verification: pending; capture the failing case, corrected case, and independent control in this leaf or its children before closure.
- Commit: pending.

### SIGNOFF-REPAIR.4.1 — Node enrollment and certificate lifecycle

- Status: `pending`.
- Sources / owned surfaces: `node_enrollment, node_channel, ca, rb-node, migrations`.
- Goal and acceptance: Bind token use to current issuing authority; recover expired unused token issuance; enforce host/node/incarnation tenant lineage; make replacement lineage and lease effects consistent; serialize rotation/revocation and bound renewal after revocation.
- Verification: pending; capture the failing case, corrected case, and independent control in this leaf or its children before closure.
- Commit: pending.

### SIGNOFF-REPAIR.4.2 — Handshake and lease fencing

- Status: `pending`.
- Sources / owned surfaces: `node_channel, channel client, certificate/key storage`.
- Goal and acceptance: Reproduce proof replay and fence races; prevent replay-based private-key recovery; validate current lease atomically for every fenced write; secure and atomically persist keys and rotations; distinguish shipped HTTP proof from TLS capability.
- Verification: pending; capture the failing case, corrected case, and independent control in this leaf or its children before closure.
- Commit: pending.

### SIGNOFF-REPAIR.4.3 — Durable inbox identity and cursors

- Status: `pending`.
- Sources / owned surfaces: `node_inbox, node_events, node_inbox_state, channel polling`.
- Goal and acceptance: Bind receipts/reconciliation/dedup to tenant and node; preserve monotonic cursors through pruning; serialize enqueue; prove command-id collisions across nodes cannot consume or hide foreign work.
- Verification: pending; capture the failing case, corrected case, and independent control in this leaf or its children before closure.
- Commit: pending.

### SIGNOFF-REPAIR.4.4 — Node recovery and unknown outcomes

- Status: `pending`.
- Sources / owned surfaces: `node journal/supervisor/worker, node_work/replacement tests`.
- Goal and acceptance: Close terminal-result/outgoing-event crash gaps, reconnect transport failures, bound streams/deadlines, reconcile actual evidence, settle refused results, and require explicit duplicate-risk authorization before replaying unknown provider outcomes.
- Verification: pending; capture the failing case, corrected case, and independent control in this leaf or its children before closure.
- Commit: pending.

### SIGNOFF-REPAIR.4.5 — Budget and quota serialization

- Status: `pending`.
- Sources / owned surfaces: `core budget, budget.rs, quota.rs, outbox worker`.
- Goal and acceptance: Reject arithmetic overflow/negative usage; bind reservations to tenant/thread/attempt; lock ceiling/breaker/quota admission, preserve uncertain holds, validate settlement transitions, fence delivery effects and completion, and prove concurrent ceilings.
- Verification: pending; capture the failing case, corrected case, and independent control in this leaf or its children before closure.
- Commit: pending.

### SIGNOFF-REPAIR.5.1 — Directory and profile isolation

- Status: `pending`.
- Sources / owned surfaces: `profiles, matching, presence, directory endpoints`.
- Goal and acceptance: Apply visibility per candidate tenant, use equally qualified foreign fixtures, prevent private-feature leaks, serialize updates/attestations, validate ranking bounds and missing dependence facts, and enforce capability expiry/concurrency.
- Verification: pending; capture the failing case, corrected case, and independent control in this leaf or its children before closure.
- Commit: pending.

### SIGNOFF-REPAIR.5.2 — Recruitment and autonomous initiation

- Status: `pending`.
- Sources / owned surfaces: `calls, offers, panels, auto-thread endpoints`.
- Goal and acceptance: Bind call/thread/tenant/actor and grants, make panel/close/offer transitions atomic, enforce post-filter minimums and concurrent caps, and make repeated legitimate auto initiation possible with full budget/topics/classification semantics.
- Verification: pending; capture the failing case, corrected case, and independent control in this leaf or its children before closure.
- Commit: pending.

### SIGNOFF-REPAIR.5.3 — Federation and portable cards

- Status: `pending`.
- Sources / owned surfaces: `agreements, card import/export, receipts`.
- Goal and acceptance: Complete remote recruitment under explicit local grants, bind receipts to actual digest references, transact identity/profile/quota/receipt import together, isolate replay and provenance, and prove revocation races cannot widen visibility or effects.
- Verification: pending; capture the failing case, corrected case, and independent control in this leaf or its children before closure.
- Commit: pending.

### SIGNOFF-REPAIR.6.1 — MCP read and write authorization

- Status: `pending`.
- Sources / owned surfaces: `mcp_read, mcp_write, reasonbraid-mcp`.
- Goal and acceptance: Require per-target tenant and grant checks on inbox/thread/policy reads and all writes; reject scalar payloads without panic; prove policy/recruitment handlers cannot use enrollment as authority; keep quota admission semantics accurately documented.
- Verification: pending; capture the failing case, corrected case, and independent control in this leaf or its children before closure.
- Commit: pending.

### SIGNOFF-REPAIR.6.2 — MCP durable delivery and transport

- Status: `pending`.
- Sources / owned surfaces: `mcp_listen, reasonbraid-mcp`.
- Goal and acceptance: Retain the latest bounded dedup window, serialize first delivery and monotonic cursor updates, validate state instead of dropping malformed data, and implement the documented reauthorization/reconnect/transport profile with real wire evidence.
- Verification: pending; capture the failing case, corrected case, and independent control in this leaf or its children before closure.
- Commit: pending.

### SIGNOFF-REPAIR.6.3 — A2A executable interoperability

- Status: `pending`.
- Sources / owned surfaces: `reasonbraid-a2a, compatibility records`.
- Goal and acceptance: Make semantic-loss defaults honest, wire mapped messages through local grants, and demonstrate a real independently connected peer; serialization-only tests must remain labeled as such.
- Verification: pending; capture the failing case, corrected case, and independent control in this leaf or its children before closure.
- Commit: pending.

### SIGNOFF-REPAIR.7.1 — Authenticated resource and resolver ownership

- Status: `pending`.
- Sources / owned surfaces: `resource_references, resolvers, API registration`.
- Goal and acceptance: Bind writes/reads to explicit tenant or site authority; prevent global URL first-writer poisoning and partial-upsert stale claims; enforce expected content digests, declared capabilities, deterministic ranking and supported execution.
- Verification: pending; capture the failing case, corrected case, and independent control in this leaf or its children before closure.
- Commit: pending.

### SIGNOFF-REPAIR.7.2 — HTTPS and Git acquisition safety

- Status: `pending`.
- Sources / owned surfaces: `fetcher, ssrf, git_acquire`.
- Goal and acceptance: Validate destinations and credential forwarding at every redirect and dial, including numeric IPv4/IPv6; bound download/decode/object/time consumption before allocation; prevent pipe deadlocks and remove owned scratch on every path; verify supported refs and canonical digest framing.
- Verification: pending; capture the failing case, corrected case, and independent control in this leaf or its children before closure.
- Commit: pending.

### SIGNOFF-REPAIR.7.3 — Extraction, browser and credential workers

- Status: `pending`.
- Sources / owned surfaces: `extract worker, browser worker, acquisition pipelines`.
- Goal and acceptance: Wire PDF/archive/feed inputs to supported acquisition; bound parse/decode/output, drain pipes concurrently, reap descendants, enforce browser redirect/subresource isolation and credential origin binding, and prove malicious local fixtures cannot escape.
- Verification: pending; capture the failing case, corrected case, and independent control in this leaf or its children before closure.
- Commit: pending.

### SIGNOFF-REPAIR.7.4 — Evidence integrity and retention

- Status: `pending`.
- Sources / owned surfaces: `snapshots, derivations, claim_assessments`.
- Goal and acceptance: Bind metadata and authors to authenticated actions, make object+snapshot writes atomic, refresh actual freshness horizons, retain tombstone honesty, restrict expiry clocks/scope, and distinguish excerpt existence from claim entailment with explicit evidence gates.
- Verification: pending; capture the failing case, corrected case, and independent control in this leaf or its children before closure.
- Commit: pending.

### SIGNOFF-REPAIR.8.1 — Workflow and thread state invariants

- Status: `pending`.
- Sources / owned surfaces: `thread engine, workflow registry, profiles tests`.
- Goal and acceptance: Version and authorize shared workflow registration, prevent built-in override and MAX+1 races, track each challenge's resolution once, recover expired invitations, validate duration bounds and record attribution, and reconcile durable unresolved challenges with close contracts.
- Verification: pending; capture the failing case, corrected case, and independent control in this leaf or its children before closure.
- Commit: pending.

### SIGNOFF-REPAIR.8.2 — Evaluation and routing evidence

- Status: `pending`.
- Sources / owned surfaces: `evaluation service, benchmark, routing`.
- Goal and acceptance: Bind corpora/digests/cases/arms/run references, validate seeds and duplicate assignments, reject missing/non-numeric gate measurements, derive calibration from eligible runs, and audit routing only in the intended authorized transaction.
- Verification: pending; capture the failing case, corrected case, and independent control in this leaf or its children before closure.
- Commit: pending.

### SIGNOFF-REPAIR.9.1 — Policy registration and authority

- Status: `pending`.
- Sources / owned surfaces: `policy registry, lifecycle, approvals, corrections`.
- Goal and acceptance: Tenant-scope all material records, bind claimed authorities to the authenticated caller and live action/scope/boundary, enforce immutable content digests and fail-closed selectors, and validate lifecycle/dependency/precedence/waiver semantics.
- Verification: pending; capture the failing case, corrected case, and independent control in this leaf or its children before closure.
- Commit: pending.

### SIGNOFF-REPAIR.9.2 — Atomic policy lifecycle and publication

- Status: `pending`.
- Sources / owned surfaces: `proposals/decisions/approvals, projections, publisher, reconciler`.
- Goal and acceptance: Serialize stage transitions, bind projection and manifest to approved policy, constrain filesystem targets, reject fabricated effective Git IDs, make CAS retries recoverable, verify both immutable and effective refs, and reconcile DB/Git failure points.
- Verification: pending; capture the failing case, corrected case, and independent control in this leaf or its children before closure.
- Commit: pending.

### SIGNOFF-REPAIR.9.3 — Deployment, correction and review lifecycle

- Status: `pending`.
- Sources / owned surfaces: `deployment assignments, drift, corrections, reviews`.
- Goal and acceptance: Bind desired digests/refs to publication, validate receipts and corrective authority, permit subsequent reviews after completed occurrences, enforce waiver constraints, and use relative test clocks with failure visibility.
- Verification: pending; capture the failing case, corrected case, and independent control in this leaf or its children before closure.
- Commit: pending.

### SIGNOFF-REPAIR.10.1 — Adapter subprocess supervision

- Status: `pending`.
- Sources / owned surfaces: `codex/claude adapters, process maps, fixtures`.
- Goal and acceptance: Bound UTF-8-safe stderr/stdout/chunk storage, drain concurrently, reap terminal children, prevent prompt-option injection, and preserve honest pre-dispatch versus unknown-outcome semantics.
- Verification: pending; capture the failing case, corrected case, and independent control in this leaf or its children before closure.
- Commit: pending.

### SIGNOFF-REPAIR.10.2 — Certification and release verification

- Status: `pending`.
- Sources / owned surfaces: `adapter certification, verification ladder, release-manifest tool`.
- Goal and acceptance: Bind adapter identity/capabilities/artifact to signed complete scenario evidence; require actual coverage of every invariant; secure key creation and manifest paths/hex parsing; implement load-side checks and document unsupported SDK execution.
- Verification: pending; capture the failing case, corrected case, and independent control in this leaf or its children before closure.
- Commit: pending.

### SIGNOFF-REPAIR.11.1 — Console behavior

- Status: `pending`.
- Sources / owned surfaces: `web/app.js, web-ui book chapter`.
- Goal and acceptance: Render numeric and structured values as inert text, prevent stale asynchronous views after navigation/identity change, and prove real browser timeline/audit behavior.
- Verification: pending; capture the failing case, corrected case, and independent control in this leaf or its children before closure.
- Commit: pending.

### SIGNOFF-REPAIR.11.2 — Repository safety and doctrine accuracy

- Status: `pending`.
- Sources / owned surfaces: `bootstrap/update_scaffold, check scripts, task acceptance probes`.
- Goal and acceptance: Protect populated repositories, preserve project indexes, enforce ownership before all changes, validate staged evidence for the actual leaf, reject failed process censuses, and correct table/path checks against primary specifications.
- Verification: pending; capture the failing case, corrected case, and independent control in this leaf or its children before closure.
- Commit: pending.

### SIGNOFF-REPAIR.11.3 — Operational scripts and evidence

- Status: `pending`.
- Sources / owned surfaces: `backup/restore/dev/demo/load scripts`.
- Goal and acceptance: Protect restore targets and secrets, use atomic restrictive backups, validate identifiers and quoting, verify HTTP status and negative controls, avoid fixed-port/output collisions, reap jobs, and ensure requested load counts and honest demo evidence.
- Verification: pending; capture the failing case, corrected case, and independent control in this leaf or its children before closure.
- Commit: pending.

### SIGNOFF-REPAIR.11.4 — Documentation containment and historical claims

- Status: `pending`.
- Sources / owned surfaces: `live docs, book, task records, external ledger, CI`.
- Goal and acceptance: Partition oversized live status/history, review adopted containment requirements, reconcile all phase/gate claims with measured behavior, refresh dependency evidence, and make pre-push CI discover every required live suite without counting skips as passes.
- Verification: pending; capture the failing case, corrected case, and independent control in this leaf or its children before closure.
- Commit: pending.

### SIGNOFF-REPAIR.12 — Requalification and return to roadmap

- Status: `pending`.
- Sources / owned surfaces: `all corrective leaves, PHASE-8.5.3/.5.4/.6, PHASE-9`.
- Goal and acceptance: Close or refute every census finding with reproducible evidence and owning leaf, run broad required gates before push, update qualification limits, then resume store-and-forward, export/import, G8 and later executable roadmap work. External judgment gates remain explicit, never self-certified.
- Verification: pending; capture the failing case, corrected case, and independent control in this leaf or its children before closure.
- Commit: pending.

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `SIGNOFF-REPAIR.2.1` | `pending` | local execution prerequisites |
| 2 | `SIGNOFF-REPAIR.2.2` | `pending` | safe focused runtime reproductions |
| 3 | `SIGNOFF-REPAIR.3.1` | `pending` | target validation currently follows committed revocation |
| 4 | `SIGNOFF-REPAIR.3.2` | `pending` | shared registry authority selected by delegated engineering judgment |

## Evidence routing

The census is partitioned under `docs/tasks/artifacts/signoff_review/`. Each record links to this tree's concrete repair leaves. Findings crossing several subsystems remain here until their shared mechanism is proved; no runtime reproduction is inferred from routing. Historical phase closures retain their original provenance and gain correction pointers.

## Blockers

None for the current documentation and repair work. G6/G7 external review, public-name clearance and license decisions remain their existing director/external-owned gates; they do not prevent local repairs.

## Verification Log

- `SIGNOFF-REPAIR.1`: startup full read completed; `git status --short --branch` returned only `## main...origin/main [ahead 269]` before creation. Documentation validation completed below.

## Leaf .1 closure evidence

- **REPRODUCE / ROOT CAUSE:** full source read at `9c2d2ba`; the registry helper selects an any-tenant admin grant without a boundary check. Runtime confirmation is explicitly pending `.3.2`. The live-document census measured 42,374 bytes / 18 lines and zero distinct Phase 5/6/7/9 rows (`python3 -B` over `Path.read_text().splitlines()` and row searches, rc=0).
- **FIX / ADDRESSED:** source notes preserved as 131 review records (including general caveats, not 131 confirmed defects), with references resolving to 35 repair leaves; no missing owner IDs. The new live snapshot has all ten phase rows, 33 lines / 3,414 bytes. In-memory negative controls removing a referenced owner and a phase row were detected (`doc controls: owner omission detected; phase omission detected; generated authority and qualification pages present; rc=0`).
- **NO REGRESSION:** `git diff --check` rc=0; `env TMPDIR="$PWD/target/doctrine_scratch/commit" mdbook build docs/book` rc=0, HTML generated; the generated authority and qualification pages were inspected for their expected content. `env TMPDIR="$PWD/target/doctrine_scratch/commit" bash scripts/check_doctrines.sh` printed `=== all doctrines green ===` (13 checks, rc=0). Commit hook rechecks the staged scope. Rust/runtime tests were not run: this leaf changes docs and navigation data, not product implementation.
- **LOCKSTEP:** roadmap security-correction pointer, programme and task indexes, Phase-8 correction, MEMORY, LIVE_STATUS, README navigation, book progress/authority/qualification, CHANGELOG, DEV_NOTES and decision index updated; Knowledge Map regenerated. README 49 lines / 1,914 bytes; MEMORY 22 lines / 2,230 bytes at validation. Source index holds the baseline digest and historical status reference.
- **Locality:** book output and doctrine scratch are under the repository, whose device ID and target device ID both measured `16777244`. Required installed mdBook executable `/opt/homebrew/bin/mdbook` was accessed read-only as a toolchain dependency; its output was repository-local. No Cargo/home cache access, provider calls or database runs occurred.
- **Policy review:** CLAIM_VERIFICATION matched the director-authorized donor at startup; README policy was already locally adopted and reviewed against its donor. Remaining containment/enforcement gaps are owned by `.11.4`; no automatic donor synchronization or cap increase occurred.

## Commit Log

- `SIGNOFF-REPAIR.1`: `REASONBRAID-REPAIR-0001 (leaf SIGNOFF-REPAIR.1): record corrective census and site authority decision`.
