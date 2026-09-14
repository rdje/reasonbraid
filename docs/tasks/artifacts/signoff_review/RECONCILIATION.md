# Clause reconciliation ledger — the source-census records, clause by clause

Owner: `SIGNOFF-REPAIR.11.9.1` and its children. This file is the durable form of
the classification that leaf's acceptance requires, so the next session resumes
from it rather than re-deriving it.

Re-read it mechanically with:

```bash
python3 -B scripts/census_record_reconciliation.py --classified
```

## What a row means

One row is **one clause of one record** — not one record. `SIGNOFF-REPAIR.11.9`'s
finding was that the reconcilable unit is a clause with an owner, because
`R-31-32-5` named three findings and only one ever found one.

`Clause` is the clause's ordinal within its record's body, counted in reading
order. It is a stable handle for a sentence, not a claim that the record's prose
divides cleanly at every full stop.

⛔ **A row's `State` is what the CLASSIFICATION found, and it does not change when
the owning leaf later does the work.** `unowned` means no leaf covered the clause
at classification time and this ledger opened the one the Owner column names —
`R-31-32-1` clause 4 stays `unowned` now that `.3.5.3` has closed it. Restating a
leaf's status here would make this a second copy of a fact the tree already owns,
which is the drift `INDEX-FRONTIER` exists to stop. Follow the Owner column for
the current status.

## The closed set of states

| State | Meaning | Next action |
| --- | --- | --- |
| `handled` | a leaf did the work, whether or not it ever named the record | none; the Evidence column names the leaf and its commit |
| `owned` | a leaf owns the clause AND that leaf's own text makes it visible, so its future census will read it | none; execute the owning leaf |
| `attach` | a leaf owns the surface but its own text does NOT make the clause visible | ⛔ **attach the clause to the owning leaf IN THE COMMIT THAT CLASSIFIES IT**, or its split drops it — this is `SIGNOFF-REPAIR.11.9`'s exact mechanism, caught before it fires |
| `unowned` | no leaf covers it | open a leaf; the Owner column names the one this ledger opened |
| `declined` | considered and deliberately rejected | none; the Evidence column carries the reason |
| `none` | the record carries no finding | none |

⛔ `attach` is the state the whole activity exists to find. `handled` and `owned`
are good outcomes; `unowned` is a defect the ledger discovered; `attach` is a
defect the ledger discovered **before** it cost anything, which is the cheapest
place to catch it.

🔴 **And `attach` is the one state whose row is not self-sufficient, which this
ledger learned the hard way.** Tranche 1 classified three clauses `attach` and
performed **none** of the three attachments: `SIGNOFF-REPAIR.7.1` carried no word
about the caller-declared `scheme` column, and `SIGNOFF-REPAIR.11.4` none about
`escalation.rs`'s now-false opening claim. A row saying "this leaf will drop the
clause" does not stop the leaf dropping it — only the sentence in the leaf does.
`SIGNOFF-REPAIR.11.9.1.1.1` attached all three, plus its own four, and the
vocabulary above now names the commit boundary rather than the intention.
⚠️ Every other state's next action is "none"; that asymmetry is why this one was
easy to miss, and why a future tranche should check its predecessors' `attach`
rows landed before adding its own.

⚠️ `declined` had **zero** rows until tranche 2b produced the first, which is
why the state exists in the vocabulary before an instance does: a deliberate
rejection is invisible to every search. It remains the only state that removes a
clause from view, so its Evidence cell is the one an auditor reads first.

## Tranche 1 — the records whose narrowest candidate leaf is named by three or fewer records

Ranking rationale and its measurement: `SIGNOFF-REPAIR.11.9.1`.

| Record | Clause | State | Owner | Evidence |
| --- | --- | --- | --- | --- |
| `R-31-32-1` | 1 | handled | `SIGNOFF-REPAIR.3.3.4.10.3` | `replay_command` runs one guarded transaction with a tenant-bound `FOR UPDATE` row selection; REPAIR-0117, falsified `left: 200, right: 400` |
| `R-31-32-1` | 2 | handled | `SIGNOFF-REPAIR.3.3.4.10.3` | `quarantine_command`'s two-statement pool sequence became one locked tenant-bound read and write; same commit and falsification |
| `R-31-32-1` | 3 | handled | `SIGNOFF-REPAIR.3.3.4.10.3` | `prune_node_inbox` binds its delete AND its before/after counts to the admitted tenant; the pre-repair probe destroyed a foreign inbox at `{"deleted":2,"before":2,"after":0}` |
| `R-31-32-1` | 4 | unowned | `SIGNOFF-REPAIR.3.5.3` | `inspect_node_inbox` is the FOURTH verb the record named and the parent census scoped itself to mutations. Live at `crates/reasonbraid-server/src/api.rs:1467`: admission on `params.tenant_id`, then `SELECT … FROM node_inbox_state WHERE node_id = $1` with no tenant predicate |
| `R-31-32-1` | 5 | owned | `SIGNOFF-REPAIR.3.4` | replay's epoch rebinding; `.3.3.4.10.3` reserves it explicitly — "Replay's cached-decision rebinding stays `.3.4`" |
| `R-47-2` | 1 | owned | `SIGNOFF-REPAIR.6.2` | goal line "Retain the latest bounded dedup window". Live at `crates/reasonbraid-server/src/mcp_listen.rs:56`: `next.push(…)` then `next.truncate(64)` keeps the FIRST 64 and never records id 65 onward |
| `R-47-2` | 2 | owned | `SIGNOFF-REPAIR.6.2` | goal line "monotonic cursor updates". Live: `SET last_cursor = $1` unconditionally, and the read binds the stored cursor to `_last` and discards it |
| `R-47-2` | 3 | owned | `SIGNOFF-REPAIR.6.2` | goal line "serialize first delivery". Live: `FOR UPDATE` on a missing row locks nothing, so two concurrent first deliveries both INSERT |
| `R-47-2` | 4 | owned | `SIGNOFF-REPAIR.6.2` | goal line "validate state instead of dropping malformed data". Live: `serde_json::from_value(window).unwrap_or_default()` silently forgets every seen id |
| `R-47-2` | 5 | owned | `SIGNOFF-REPAIR.6.2` | the leaf's acceptance already demands real wire evidence for reconnect and insertion concurrency |
| `R-53-4` | 1 | owned | `SIGNOFF-REPAIR.7.1` | goal line "prevent global URL first-writer poisoning". Live: `crates/reasonbraid-server/src/resources.rs:94` dedupes on `original_locator` alone, and `migrations/0023_resource_references.sql` has NO tenant column and `UNIQUE (original_locator, expected_digest)` |
| `R-53-4` | 2 | attach | `SIGNOFF-REPAIR.7.1` | `scheme` is a caller-declared column bound as `$2`, never derived from the locator, so a declared scheme may disagree with the URL. NOT visible in the leaf's goal line |
| `R-53-4` | 3 | attach | `SIGNOFF-REPAIR.7.1` | the record's R3 remark rides on clause 2 and shares its fate; recorded separately so a future reader is not told the two were merged |
| `R-53-4` | 4 | owned | `SIGNOFF-REPAIR.7.1` | goal line "enforce expected content digests". Live: `digest_error` validates only the `sha256:<64 hex>` FORMAT and no acquisition path reads `reference.expected_digest` |
| `R-63-1` | 1 | handled | `SIGNOFF-REPAIR.3.1` | REPAIR-0005 added the foreign-grant and foreign-boundary revocation controls the record said were missing, asserting unchanged victim status, epoch and authorization count |
| `R-63-1` | 2 | handled | `SIGNOFF-REPAIR.3.2.3` | the global registries reached explicit site-operator enforcement with eight HTTP controls, including both revocation orders |
| `R-63-1` | 3 | handled | `SIGNOFF-REPAIR.3.1` | the record asked that exact committed replay survive revocation rather than being blanket-refused; `revocation_fences_future_work_but_never_rewrites_history` still asserts it and the book's authority chapter states the contract |
| `R-63-1` | 4 | attach | `SIGNOFF-REPAIR.11.4` | residue the repair left: `crates/reasonbraid-server/tests/escalation.rs` still opens "cross-tenant isolation (the census found NO test asserting it)", which `.3.1` made false, and the suite name still says "at every boundary" while the added controls live in other suites |
| `R-65-2` | 1 | owned | `SIGNOFF-REPAIR.6.2` | the same window and cursor concurrency `R-47-2` names; the integration suite's two delivery ids cannot reach either |
| `R-83-1` | 1 | owned | `SIGNOFF-REPAIR.11.1` | goal line "Render numeric and structured values as inert text". Live: `crates/reasonbraid-server/web/app.js:30` passes a non-string child straight to `appendChild`, and `viewEvents` passes the numeric `aggregate_version` |
| `R-83-1` | 2 | owned | `SIGNOFF-REPAIR.11.1` | same mechanism for object-valued audit fields; the leaf's goal line names structured values as well as numbers |
| `R-83-1` | 3 | owned | `SIGNOFF-REPAIR.11.1` | goal line "prevent stale asynchronous views after navigation/identity change" |
| `R-83-1` | 4 | handled | `SIGNOFF-REPAIR.11.4.3.1.1` | `238051b` replaced `deny.toml`'s "workspace currently has ZERO dependencies" preamble with the real-graph wording the record asked for |
| `R-83-1` | 5 | owned | `SIGNOFF-REPAIR.11.4` | goal line "refresh dependency evidence". Live: `docs/dependencies/external-ledger.yaml` lines 43 and 70 still carry `tested_versions: []` for MCP and A2A |
| `R-84-1` | 1 | unowned | `SIGNOFF-REPAIR.11.2.2` | the doctrine gates' own scratch is off-volume, and `STORAGE-LOCALITY` scans `*.rs` only so it cannot see them. Five call sites in four tracked shell files |
| `R-84-1` | 2 | owned | `SIGNOFF-REPAIR.11.4` | the same external-ledger clause as `R-83-1` clause 5; two records reached one finding, which is what a clause-level ledger is for |

## Tranche 2a — the six records whose narrowest candidate leaf is `SIGNOFF-REPAIR.3.2`

Owner: `SIGNOFF-REPAIR.11.9.1.1.1`. Ranking rationale and its measurement:
`SIGNOFF-REPAIR.11.9.1`. Split rationale and its sizing: `SIGNOFF-REPAIR.11.9.1.1`.

⚠️ `SIGNOFF-REPAIR.3.2` is `done`. Three of these six records' clauses are
`handled` BY it, and two more asked it for the one thing it did not do — census
the shared-registry write scope before designing a closed set of six site
actions. Both of those are `attach`, and they are the same finding.

| Record | Clause | State | Owner | Evidence |
| --- | --- | --- | --- | --- |
| `R-31-32-2` | 1 | owned | `SIGNOFF-REPAIR.7.1` | goal line "Bind writes/reads to explicit tenant or site authority", owned surfaces `resolvers, API registration`. Live: `register_resolver` resolves the caller's OWN tenant and admits on `authorize_tenant_admin`, then writes `resolver_capabilities` — migration 0024 gives it no tenant column and a single-column `resolver_id` primary key |
| `R-31-32-2` | 2 | attach | `SIGNOFF-REPAIR.7.1` | "Must include scope census before design". `.7.1` names the BIND, not the obligation to census the whole shared-registry write scope first — which is what let `.3.2` pin six site actions while `/v1/resolvers` and `/v1/workflow-profiles` stayed outside. Same finding as `R-78-2` clause 2 |
| `R-53-2` | 1 | handled | `SIGNOFF-REPAIR.3.2.1` | the `pair` half. The site service probes both declarations with `?` propagation and carries the rule in its own source: "an unavailable database must never masquerade as an undeclared region" |
| `R-53-2` | 2 | unowned | `SIGNOFF-REPAIR.11.10` | the `route` half, which the same repair never reached. Live in `crates/reasonbraid-server/src/regions.rs`: three `.map_err(\|_\| RegionRefusal::…)` turn any `sqlx::Error` into `UndeclaredRegion` or `CrossRegionRefused` |
| `R-53-2` | 3 | handled | `SIGNOFF-REPAIR.3.2.1` | "declare empty id unvalidated". `RegistryName::new` refuses a blank, over-256-byte or control-character name before the command is built |
| `R-53-2` | 4 | handled | `SIGNOFF-REPAIR.3.2.1` | "typed validate / audited no-op evidence / unpair reason". `RegistryName` and `Reason` are the typed inputs; `Effect::write` records `noop` when `rows_affected` is 0; `UnpairRegions` carries a `Reason` that reaches the audit's `requested_reason` |
| `R-53-2` | 5 | handled | `SIGNOFF-REPAIR.3.2.1` | "need in_tx variants". `registry::execute` opens one transaction, locks the guard row, and runs the authority read, the domain probe, the mutation and the audit on it before a single commit |
| `R-53-3` | 1 | owned | `SIGNOFF-REPAIR.7.1` | goal line "prevent … partial-upsert stale claims", verbatim. Live: `ON CONFLICT (resolver_id) DO UPDATE` sets 6 of the row's 18 declared columns, leaving `locator_patterns`, `media_types`, `abilities`, `authentication_classes`, all four policies, both format lists and `latency_range_ms` at their first-written values |
| `R-53-3` | 2 | owned | `SIGNOFF-REPAIR.7.1` | goal line "Bind writes/reads to explicit tenant or site authority" and "enforce … declared capabilities". Live: `security_evidence` is bound straight from the caller's JSON, and the UPSERT's single-column key means a tenant admin can REPLACE the built-in `r0-https-fetcher` row |
| `R-53-3` | 3 | attach | `SIGNOFF-REPAIR.7.1` | "need inspect EGRESS_CLASSES already earlier". `resolve` applies `declared >= required` to both ADR-018 ladders, but ADR-018 decides the egress claim is "the MAXIMUM, never the minimum" — so comparing ceilings with `>=` admits a BROADER reach than required, while the module's comment reads the field as reach. Two tracked documents, two readings, no control. NOT visible in the leaf's goal line |
| `R-53-3` | 4 | owned | `SIGNOFF-REPAIR.7.1` | goal line "enforce … declared capabilities". Live: the eligibility query filters on `schemes`, `media_types` and `authentication_classes` only — `locator_patterns`, `max_bytes`, risk class and credential audience are stored and never read |
| `R-53-3` | 5 | owned | `SIGNOFF-REPAIR.7.1` | goal line "deterministic ranking". Live: the rank sorts on the latency midpoint with a stable sort, so ties keep the fetch order of a query that carries no `ORDER BY` |
| `R-53-3` | 6 | owned | `SIGNOFF-REPAIR.7.1` | goal line "supported execution". Live: `resolve` never sets `acquisition`, and the handler's match executes only the five built-in ids, so a third-party resolver ranking first falls through `_ => {}` and returns no acquisition, no error and `unresolvable_now: false` — having displaced the built-in that would have acquired |
| `R-58-2` | 1 | handled | `SIGNOFF-REPAIR.2.2.2` | the shared test-pool helper verifies runner-issued ownership against the connected server before any destructive fixture; the matched control moved the missing-metadata case from 2 writes to zero public tables |
| `R-58-2` | 2 | handled | `SIGNOFF-REPAIR.3.2.3` | `tests/allowlist.rs` now provisions through `site_fixture::provision`, the deployment-controlled issuance path, and keeps the non-admin 403s on both the list and the allow |
| `R-58-2` | 3 | handled | `SIGNOFF-REPAIR.3.2.3` | `tests/site_registry_http.rs` carries both revocation orders under a forced race, grant and boundary revocation separately, and a sibling-tenant grant control |
| `R-78-2` | 1 | owned | `SIGNOFF-REPAIR.7.1` | Live: `profiles.rs::the_resolver_registry_resolves_and_fails_explicitly` enrols a plain human and registers two resolvers with self-declared `egress_class`/`sandbox_level` through `POST /v1/resolvers`, with no site fixture. The test changes when the bind does |
| `R-78-2` | 2 | attach | `SIGNOFF-REPAIR.7.1` | "Core decision shared registry write scope at least region/adapter/resolver". The same clause as `R-31-32-2` clause 2, reached by a second record — and a record-level ledger would have counted it twice and still let it fall |
| `R-78-2` | 3 | owned | `SIGNOFF-REPAIR.8.1` | goal line "Version and authorize shared workflow registration, prevent built-in override and MAX+1 races". Live: `register_workflow_profile` requires ENROLMENT only; `workflow_profiles` has no tenant column; `register` inserts `COALESCE(MAX(version),0)+1` with `built_in = false` for any id, and resolution takes `ORDER BY version DESC LIMIT 1` |
| `R-80-82-1` | 1 | owned | `SIGNOFF-REPAIR.8.1` | goal line "reconcile durable unresolved challenges with close contracts". Live: the close gate is `is_decision_family() && !body.unresolved.is_empty()` over the CALLER's list; `projection.open_challenges` is maintained on both edges and read nowhere at close |
| `R-80-82-1` | 2 | owned | `SIGNOFF-REPAIR.8.1` | goal line "record attribution". Live: `SynthesisInput.synthesizer` and `MinorityReportInput.synthesizer` are plain `String`s resolved against no principal, and the test asserts the literal `hpr-sy-human`, which is not even the `hpr_` shape a real principal id carries |
| `R-80-82-1` | 3 | attach | `SIGNOFF-REPAIR.8.1` | the verdict's `target_digest` is a plain `String` copied verbatim into the event — neither format-validated nor bound to any proposal revision, so `sha256:00` against no claimed target is accepted. The leaf owns the thread engine and its profiles tests; this clause is NOT visible in its goal line |
| `R-80-82-1` | 4 | owned | `SIGNOFF-REPAIR.3.5` | goal line "replace foreign/nonexistent-node success fixtures". Live: `tests/quarantine.rs` seeds `node_inbox` rows for the string `nod_seeded_evidence`, which enrols as no node |
| `R-80-82-1` | 5 | owned | `SIGNOFF-REPAIR.9.2` | goal line "make CAS retries recoverable". Live: `publisher::publish` writes the immutable ref (`MustNotExist`) BEFORE the effective compare-and-swap and unwinds nothing on `CasMismatch`, so retrying the SAME publication id returns `ImmutableExists`. The test proves the stale-CAS refusal and separately an `ImmutableExists` on another id; it never retries the id that failed |
| `R-80-82-1` | 6 | owned | `SIGNOFF-REPAIR.9.2` | goal line "verify both immutable and effective refs". Live: `reconciler::reconcile` reads `git.effective` in NO branch, so §15.8's "effective / ref missing or moved → freeze" cannot fire for a moved effective channel. `the_consistent_pairs_are_quiet` asserts immutable 7 with effective 8 and expected 7 is `Consistent` |
| `R-80-82-1` | 7 | handled | `SIGNOFF-REPAIR.2.2.2` | ⭐ handled by a DIFFERENT mechanism than the record proposed, which is why the row says so: the record asked for a disposable guard AND project-owned role isolation. The guard landed, and it is what supplies the isolation — `rls_probe` can now only exist inside a runner-owned disposable cluster, so the role name never needed to become unique |
| `R-80-82-1` | 8 | handled | `SIGNOFF-REPAIR.3.2.3` | `tests/allowlist.rs` and `tests/regions.rs` both provision explicit site authority and assert the refusal for a principal without it; the HTTP suite adds the revoked-grant and revoked-boundary legs |

## Tranche 3a — the five records whose narrowest candidate leaf is `SIGNOFF-REPAIR.4.3`

Owner: `SIGNOFF-REPAIR.11.9.1.2.1`. Ranking: `SIGNOFF-REPAIR.11.9.1`. Split and
its sizing: `SIGNOFF-REPAIR.11.9.1.2`.

⭐ **Three of these rows are one mechanism at three sites** — a lookup keyed on
`operation_id`/`command_id` with the owning node never in the predicate — reached
by two different records. `SIGNOFF-REPAIR.4.3`'s goal line already demands the
proof that would have caught all three.

| Record | Clause | State | Owner | Evidence |
| --- | --- | --- | --- | --- |
| `R-48-49-3` | 1 | owned | `SIGNOFF-REPAIR.4.3` | goal line "Bind receipts/reconciliation/dedup to tenant and node". Live: `node_channel.rs::event_id_for_operation` is `SELECT event_id FROM node_events WHERE operation_id = $1 ORDER BY received_at LIMIT 1` — no `node_id`, no tenant — and it is the reconciliation lookup the HANDSHAKE performs, so any enrolled node naming a victim's operation id is handed the victim's event id |
| `R-48-49-3` | 2 | owned | `SIGNOFF-REPAIR.4.3` | same goal line, the dedup half. Live: the receipt insert is `ON CONFLICT (event_id) DO NOTHING`, and `migrations/0003_node_inbox.sql:31` makes `event_id` the whole primary key with `node_id` a plain column. A node that writes first under a known event id turns the rightful node's receipt into a silent no-op returning `false` |
| `R-48-49-3` | 3 | owned | `SIGNOFF-REPAIR.4.3` | "Needs scope + payload digest conflict" is the record's remedy and rides on clauses 1–2; recorded separately so a future reader is not told the three were merged. The goal line's "bind … to tenant and node" IS the scope half of it |
| `R-48-49-4` | 1 | owned | `SIGNOFF-REPAIR.4.3` | goal line "preserve monotonic cursors through pruning". Live: `enqueue` writes `(SELECT COALESCE(MAX(cursor), 0) + 1 FROM node_inbox WHERE node_id = $1)` and `current_cursor` reads `COALESCE(MAX(cursor), 0)` over the same rows — the high-water mark is DERIVED from a table prune deletes from |
| `R-48-49-4` | 2 | owned | `SIGNOFF-REPAIR.4.3` | same goal line. Live: prune everything and `MAX` is NULL, so `current_cursor` answers 0 while the node still holds its journal's high cursor — the `journal_lost`-class anomaly this module's own header names at `node_channel.rs:43` |
| `R-48-49-4` | 3 | owned | `SIGNOFF-REPAIR.4.3` | same goal line, the reuse half. Live: after a full prune the next enqueue re-issues cursor 1, and a node that already acknowledged cursor 1 discards it as seen — the work is silently dropped rather than refused |
| `R-48-49-4` | 4 | owned | `SIGNOFF-REPAIR.4.3` | "a monotonic high-water must be a separate durable counter, not the deletable inbox" is the record's remedy for clauses 1–3 and the only shape that survives a prune. Recorded separately; it shares their fate |
| `R-70-3` | 1 | attach | `SIGNOFF-REPAIR.4.3` | the TEST stops one step short of the defect. `node_inbox.rs::prune_deletes_only_delivered_rows_older_than_the_window` leaves survivors `cmd_prune_3`, `4`, `5` — a partial prefix that KEEPS the highest cursor, so `MAX` never returns NULL — and no prune-everything-then-reconnect control exists in the suite. The goal line demands one proof (`command-id collisions`) and not this one |
| `R-70-3` | 2 | owned | `SIGNOFF-REPAIR.4.3` | "the core MAX cursor bug remains" is the SAME finding as `R-48-49-4` clauses 1–2, reached by a second record — the fourth such pair in this activity, and the reason a record-level ledger would have counted it twice |
| `R-75-1` | 1 | owned | `SIGNOFF-REPAIR.9.3` | goal line "enforce waiver constraints". Live: `reviews::schedule_reviews` pushes one `(publication, "repeated_waiver")` pair per waiver ROW, so a single waiver satisfies a trigger named "repeated"; `policy.rs:3161` records exactly one and asserts the trigger fires, codifying it |
| `R-75-1` | 2 | owned | `SIGNOFF-REPAIR.9.3` | goal line "permit subsequent reviews after completed occurrences", verbatim. Live: the dedupe is `EXISTS (… WHERE publication_id = $1 AND trigger = $2 AND status = 'due')`, so only a still-`due` review suppresses a reschedule — and the suite never exercises the after-`done` path the goal line names |
| `R-75-1` | 3 | attach | `SIGNOFF-REPAIR.9.3` | that an existing defect id dedupes PERMANENTLY is untested. It is a coverage claim about a different key than clause 2's `(publication, trigger)` pair, and no part of the goal line reaches it |
| `R-75-1` | 4 | owned | `SIGNOFF-REPAIR.9.3` | goal line "use relative test clocks with failure visibility", verbatim. Live: `policy.rs:2811` and `:3161` both hardcode `expires_at` of `2026-09-15T00:00:00Z` |
| `R-75-1` | 5 | declined | `SIGNOFF-REPAIR.9.3` | ⭐ the record asks, with its own question mark, whether the absent future-validation makes those tests time-brittle NOW. MEASURED on 2026-09-14, both sides: `corrections::record_correction` requires `expires_at` to be present and RFC3339-parseable for a suspension or waiver and never compares it to `now()`; `reviews::schedule_reviews` selects `WHERE operation = 'waiver'` with no expiry predicate. ⛔ So the tests do NOT break on 2026-09-15 — declined AS A PRESENT DEFECT, with the record's own conditional ("time brittles when proper checks land") standing and clause 4 carrying the repair |
| `R-85-1` | 1 | owned | `SIGNOFF-REPAIR.4.1` | goal line "enforce host/node/incarnation tenant lineage", verbatim. Live: `migrations/0007` gives `nodes` two INDEPENDENT foreign keys — `host_id -> hosts` and `tenant_id -> tenants` — with nothing requiring them to agree, and repeats the shape for `incarnations` (`role_id` + `tenant_id`) and `runs` (`incarnation_id` + `tenant_id`) |
| `R-85-1` | 2 | owned | `SIGNOFF-REPAIR.3.3` | goal line "enforce identity/tenant/subset/window correspondence". Live: `migrations/0004_authority.sql:35` is `boundary_id TEXT NOT NULL REFERENCES enrollment_boundaries (boundary_id)` — by boundary id alone, never composite with the tenant |
| `R-85-1` | 3 | handled | `SIGNOFF-REPAIR.4.1.1` | ⭐ answered by the SOURCE rather than by the tree. `migrations/0059_node_token_supersede.sql` states the record's mechanism verbatim — "an EXPIRED token still has `used_at IS NULL`, so it stayed in the index permanently" — and records why the predicate could not become `AND expires_at > now()`: a partial index predicate must be IMMUTABLE, so the liveness is written down as `superseded_at` and the index keys on the stamp |
| `R-85-1` | 4 | owned | `SIGNOFF-REPAIR.4.3` | goal line "prove command-id collisions across nodes cannot consume or hide foreign work", verbatim — "consume" is the word the view uses. Live: `migrations/0021_node_inbox_delivery_state.sql:19`–`:21` calls a row `consumed` on `EXISTS (… node_events e WHERE e.operation_id = i.command_id AND e.payload->>'kind' = 'work_result')` with no `e.node_id = i.node_id`. ⭐ The THIRD site of `R-48-49-3`'s mechanism, reached by a second record |
| `R-85-1` | 5 | attach | `SIGNOFF-REPAIR.4.1` | `node_presence.suspended` ignores certificate EXPIRY. `migrations/0017` — the latest definition — is `EXISTS (revoked) AND NOT EXISTS (revoked_at IS NULL)`, and `node_certificates.expires_at` is `NOT NULL` and never consulted, so one expired unrevoked leaf reads as "active" and blocks `suspended` when no usable certificate exists. The goal line's "make replacement lineage and lease effects consistent" is about REPLACEMENT; expiry is a different predicate and the view has none |
| `R-85-1` | 6 | owned | `SIGNOFF-REPAIR.4.1` | "determine the intended observability contract and fix the drift conservatively" is the record's remedy and rides on clauses 1, 2, 4 and 5, whose owners are three different leaves. Recorded separately, with the largest share's owner named, so a future reader is not told it was merged |

## Tranche 3b — the five records whose narrowest candidate leaf is `SIGNOFF-REPAIR.6.1`

Owner: `SIGNOFF-REPAIR.11.9.1.2.2`. Ranking: `SIGNOFF-REPAIR.11.9.1`. Split and
its sizing: `SIGNOFF-REPAIR.11.9.1.2`.

⭐ This tranche carries the ledger's **first `none` row** — the sixth state, in
the vocabulary since tranche 1 and without an instance until now.

| Record | Clause | State | Owner | Evidence |
| --- | --- | --- | --- | --- |
| `R-31-32-3` | 1 | owned | `SIGNOFF-REPAIR.7.1` | goal line "Bind writes/reads to explicit tenant or site authority" — the READS half. Live: `api.rs::resolve_resource` computes `let enrolled = reader_tenant(…).is_some()` and refuses only when that is false. Enrolment is the whole check |
| `R-31-32-3` | 2 | owned | `SIGNOFF-REPAIR.7.1` | same goal line. Live: `crate::resources::get(&state.pool, &resource_id)` selects by the GLOBAL reference id with no tenant predicate — and `migrations/0023_resource_references.sql` has no tenant column to predicate on (`R-53-4` clause 1, already owned here) |
| `R-31-32-3` | 3 | owned | `SIGNOFF-REPAIR.7.1` | same goal line, the credential half. Live: `reference.credential_binding_ref` is read off that globally-keyed row and handed to the acquisition, so the binding a foreign tenant declared is the one used |
| `R-31-32-3` | 4 | owned | `SIGNOFF-REPAIR.7.4` | goal line "make object+snapshot writes atomic". Live and MEASURED as a contrast rather than asserted: `api.rs` calls `crate::snapshots::submit` at FOUR sites; `:2029` (R0) and `:2104` (R5) write `let _ = …`, discarding the Result, while `:2260` and `:3695` consume it. The acquisition answers success whether or not the durable evidence landed |
| `R-31-32-3` | 5 | owned | `SIGNOFF-REPAIR.11.4` | goal line "reconcile all phase/gate claims with measured behavior". The record asks that the book's durable-evidence claim be examined against clause 4's behaviour, which is a published-claim reconciliation and not a code repair |
| `R-48-49-1` | 1 | owned | `SIGNOFF-REPAIR.6.1` | goal line "reject scalar payloads without panic", verbatim. Live: `mcp_write.rs:149` is `body["tenant_id"] = serde_json::json!(tenant_id)` on a `serde_json::Value` the function received untyped, and `IndexMut<&str>` for `Value` panics on a scalar or an array. ⚠️ The MCP tool itself passes a TYPED `ContributePayload`, so the panic is not reachable through that surface today; `mcp_write::respond` is `pub` and takes `Value`, so the guard belongs at the seam rather than at the caller |
| `R-48-49-1` | 2 | owned | `SIGNOFF-REPAIR.6.1` | goal line "keep quota admission semantics accurately documented". Live: `gate()` runs and COMMITS its quota use before `run_thread_command` applies idempotency, so a replayed identical call spends quota again. ⭐ The module header states exactly this — "the use/denial event commits in the gate's own transaction, so the quota counts the ADMITTED CALLS, never the domain effects" — so the documentation is accurate today and this leaf owns keeping it so |
| `R-48-49-1` | 3 | owned | `SIGNOFF-REPAIR.6.1` | goal line "prove policy/recruitment handlers cannot use enrollment as authority" — the RECRUITMENT half. Live: `join_call` gates the principal against the caller-supplied `tenant_id`, then `respond_to_call_core(pool, principal, call_id, …)` fetches the call by `call_id` ALONE and never compares its tenant to the gated one. ⛔ Two caller-supplied identifiers, one checked — and a `decline`, `recommend` or `recuse` skips the eligibility gate entirely, so nothing else binds them |
| `R-48-49-1` | 4 | owned | `SIGNOFF-REPAIR.6.1` | same goal line, the POLICY half. Live: `propose_policy_change` runs `gate()` — enrolment plus quota — then `lifecycle::register_proposal(pool, tenant_id, &input)` with no grant check, while the seam's own header promises "the per-verb LOCAL grants + the audit ride the handlers themselves" |
| `R-50-3` | 1 | owned | `SIGNOFF-REPAIR.9.1` | goal line "enforce immutable content digests". Live: `policy::register` validates the digest's SHAPE with `is_sha256_hex` and stores it; no canonical document is submitted, hashed or retained, so the digest is a declaration about bytes the registry never sees |
| `R-50-3` | 2 | owned | `SIGNOFF-REPAIR.9.1` | goal line "bind claimed authorities to the authenticated caller and live action/scope/boundary". Live, and the asymmetry is inside ONE file: `policy.rs:185` admits a declared grant on `WHERE grant_id = $1 AND status = 'active'` while `:458`–`:459` queries the same table as `status = 'active' AND (expires_at IS NULL OR expires_at > now())`. Registration accepts an expired grant the resolver would refuse |
| `R-50-3` | 3 | owned | `SIGNOFF-REPAIR.9.1` | same goal line. Live: neither site reads the grant's boundary, its valid-from, or its action set — only its row's status |
| `R-50-3` | 4 | owned | `SIGNOFF-REPAIR.9.1` | goal line "enforce immutable content digests". Live: the stored `digest` is returned in reads and compared against nothing at use; no resolution path re-derives or verifies it |
| `R-50-3` | 5 | attach | `SIGNOFF-REPAIR.9.1` | `is_semver` is not SemVer, in BOTH directions. `policy.rs:151`–`:157` accepts 1 to 3 dot-separated groups of ASCII digits, so `1` and `007.8` pass, while `1.2.3-rc.1` and `1.2.3+build` — valid SemVer — are refused. ⛔ Version-format validation is in none of the goal line's five mechanisms, and the misleading part is the NAME: a reader takes it for the standard |
| `R-50-3` | 6 | owned | `SIGNOFF-REPAIR.9.1` | goal line "fail-closed selectors", verbatim. Live: `policy.rs:482`–`:488` reads each selector field with `.unwrap_or("*")` and then admits on `layer == "*"`, so a missing or non-string field becomes the WILDCARD — fail-open, in the one place the goal line names fail-closed |
| `R-50-3` | 7 | owned | `SIGNOFF-REPAIR.9.1` | goal line "validate lifecycle … semantics". Live: `policy.rs:510` excludes exactly two of the six vocabulary values — `row.lifecycle != "suspended" && row.lifecycle != "retracted"` — so `draft`, `superseded` and `deprecated` are all APPLICABLE and bind |
| `R-50-3` | 8 | owned | `SIGNOFF-REPAIR.9.1` | goal line "validate … dependency … semantics". Live: step 3 requires `ids.contains(&policy)` — pure set membership on the policy ID. The dependency's declared version is never read, and the `applicable` vector computed one step earlier is never consulted, so a dependency satisfied by a non-applicable policy passes |
| `R-50-3` | 9 | owned | `SIGNOFF-REPAIR.9.1` | goal line "validate … precedence … semantics". Live: step 4's refusal is `precedence.get(over).contains(row.policy_id)` — a TWO-edge check — while its own comment says "a cycle is the refusal (the charter precedence must be a DAG)". A three-edge cycle is not refused; `wins_over` carries a `seen` set so it terminates, and the winner becomes order-dependent instead |
| `R-50-3` | 10 | none | — | ⭐ the ledger's FIRST `none` row. "finish resolution next page" is the reviewer's own continuation marker across a page boundary of the census, not a finding about the product. Recorded rather than omitted, because the absence of a row is how a reader tells classified from unclassified |
| `R-6-27-7` | 1 | owned | `SIGNOFF-REPAIR.6.1` | goal line "Require per-target tenant and grant checks on inbox/thread/policy reads", verbatim. Live: `list_inbox` declares `principal` in `ListInboxParams` and its body NEVER READS IT — `inbox_rows(&self.pool, &args.tenant_id, &args.node_id)` is the whole call |
| `R-6-27-7` | 2 | owned | `SIGNOFF-REPAIR.6.1` | same goal line, and WORSE than the record states. `get_policy_bundle` ignores the principal, and `server::policy_bundle(pool, _tenant_id)` takes its tenant as an UNDERSCORE-PREFIXED unused parameter over `SELECT policy_id, version, digest, clauses FROM policy_versions ORDER BY policy_id, version` — no `WHERE` at all. Every tenant's policy clauses are returned, labelled with the caller's own tenant id in the response envelope |
| `R-6-27-7` | 3 | owned | `SIGNOFF-REPAIR.6.1` | same goal line, the THREAD read. Live: `get_thread` calls `classify`, which returns `Network` for any principal whose tenant differs from the thread's, and then returns the FULL projection from `thread_state(pool, &args.tenant_id, &args.thread_id)` regardless of the class. No `ThreadInspect` grant, no boundary; the `visibility` field reports "network" beside the whole state |
| `R-6-27-7` | 4 | attach | `SIGNOFF-REPAIR.6.1` | the MODULE DOCUMENTS an authorization it does not perform, on 3 of 3 read tools: `lib.rs:1`–`:13` claims "the SAME queries + the SAME authorization as the HTTP handlers" and "every read runs the reader classification". Two of the three run no classification at all. ⛔ The goal line says what the reads must REQUIRE; it does not reach the false claim a reader of this module is given first |
| `R-6-27-7` | 5 | owned | `SIGNOFF-REPAIR.6.1` | "must reproduce and own fix" is the record's ask and rides on clauses 1–3; recorded separately so a future reader is not told the five were merged |
| `R-66-1` | 1 | owned | `SIGNOFF-REPAIR.6.1` | goal line "keep quota admission semantics accurately documented". Live: `tests/mcp_write.rs`'s third case is headed "rides the SAME thread-command pipeline (the effect + the audited allowance + the quota use)", naming three things as one pipeline, while `mcp_write.rs`'s own header says the quota "commits in the gate's own transaction" — a separate one |
| `R-66-1` | 2 | attach | `SIGNOFF-REPAIR.6.1` | the test COUNTS and never asserts atomicity: it checks a 200, reads the events, and counts quota rows. No control interrupts between the gate's commit and the handler's to observe the split. Coverage is not documentation, and the goal line names only the latter |
| `R-66-1` | 3 | owned | `SIGNOFF-REPAIR.6.1` | same goal line. The record's own ask — decide whether counting DENIED attempts is intended, and reconcile it with the success-atomic reading the test's prose invites — is the decision "keep … accurately documented" requires before the wording can be called right |

## Tranche 3c — the five records whose narrowest candidate leaf is `SIGNOFF-REPAIR.10.1`

Owner: `SIGNOFF-REPAIR.11.9.1.2.3`. Ranking: `SIGNOFF-REPAIR.11.9.1`. Split and
its sizing: `SIGNOFF-REPAIR.11.9.1.2`.

⚠️ The narrowest CANDIDATE is `.10.1` for all five, and it is the OWNER for only
one of them. `R-36-39-8` lands on `.7.3` and three records land on `.4.4`, whose
own Sources line names the two test files two of them are about. That is
`SIGNOFF-REPAIR.11.9`'s rule in action: the candidate list is a suggestion.

| Record | Clause | State | Owner | Evidence |
| --- | --- | --- | --- | --- |
| `R-36-39-8` | 1 | owned | `SIGNOFF-REPAIR.7.3` | goal line "drain pipes concurrently", verbatim. Live: `browse.rs::run_browse` polls `try_wait()` in a sleep loop until the child EXITS and only then calls `read_to_string` on the piped stdout. ⭐ MEASURED as a ratio rather than asserted: the request it writes sets `max_output_bytes` to **4 MiB** while an OS pipe buffer is at most 64 KiB, so the worker blocks on its own write for any response above roughly a sixty-fourth of the configured limit, never exits, and the loop runs to `deadline` and returns `TimedOut` |
| `R-36-39-8` | 2 | owned | `SIGNOFF-REPAIR.7.3` | goal line "reap descendants". Live: the stdin write is `writeln!(stdin, "{request}").map_err(…)?` — the `?` returns while `child` is still alive, and dropping a `std::process::Child` neither kills nor waits, so the worker and everything under it is orphaned on that path |
| `R-36-39-8` | 3 | owned | `SIGNOFF-REPAIR.7.3` | same goal line, the timeout path. Live: the deadline branch runs `child.kill()` then `child.wait()`, which reaches the WORKER process only — the browser it spawned is a descendant and a parent's death does not propagate. `stderr` is `Stdio::null()`, so nothing is left to diagnose it with either |
| `R-36-39-8` | 4 | owned | `SIGNOFF-REPAIR.7.3` | "the same extraction spawner likely — inspect next" is the record's follow-on ask and rides on clauses 1–3; the extraction worker is this leaf's owned surface, so the inspection belongs to the same census |
| `R-6-27-3` | 1 | owned | `SIGNOFF-REPAIR.10.1` | goal line "Bound UTF-8-safe stderr/stdout/chunk storage", verbatim. Live at TWO sites — `codex.rs:268` and `claude.rs:308` — `buf[buf.len() - 1024..].to_string()` is a BYTE slice of a `String`, and `String`'s range index panics when that offset is not a char boundary. Reachable whenever a child writes non-ASCII to stderr and the buffer passes 1024 bytes, on the FAILURE path |
| `R-6-27-3` | 2 | owned | `SIGNOFF-REPAIR.10.1` | same goal line, the bound. Live at `codex.rs:219`–`:226` and `claude.rs:254`: `lines.next_line()` reads to a newline with NO length limit, so a child emitting megabytes without one allocates all of it before any check; and the check is `if buf.len() < 8192` BEFORE the append, so the real bound is 8,191 bytes plus one unbounded line |
| `R-6-27-3` | 3 | owned | `SIGNOFF-REPAIR.10.1` | goal line "reap terminal children". Live: `children: Mutex<HashMap<String, Arc<Mutex<Child>>>>` is `insert`ed at `codex.rs:141` / `claude.rs:168` and `get`-ed at `:167` / `:194`, and **nothing ever removes**. The map grows by one `Child` per operation for the life of the process, and each retained handle is an unreaped process |
| `R-6-27-3` | 4 | owned | `SIGNOFF-REPAIR.10.1` | same goal line, and the contrast is inside one function. Live: the EOF arm awaits the stderr drain with a 5-second bound and then `child.wait()`; the `turn.completed` arm sets `self.finished = true` and returns `AttemptEvent::Completed` IMMEDIATELY, awaiting neither. The SUCCESS path leaves the child unreaped and the drain task running; the failure path does it correctly |
| `R-6-27-3` | 5 | owned | `SIGNOFF-REPAIR.10.1` | goal line "preserve honest pre-dispatch versus unknown-outcome semantics". Live: the EOF arm maps `Some(s)` — ANY non-success exit status — to `AttemptEvent::FailedKnown`, a definitive remote verdict. The cancel path at `:174` is `start_kill()`, so a CANCELLED attempt produces exactly such a status and is reported as a known provider failure |
| `R-6-27-3` | 6 | attach | `SIGNOFF-REPAIR.10.1` | "usage mapping against providers unverified". `normalize_usage` maps each provider's receipt into `NormalizedUsage`, and no control compares either mapping against a real provider's documented field names. ⛔ Not one of the goal line's five mechanisms, and it is the one whose failure is SILENT: a mis-mapped token count settles a budget wrongly without any error |
| `R-6-27-9` | 1 | owned | `SIGNOFF-REPAIR.4.4` | goal line "bound streams". Live: `supervisor.rs:229` builds `let mut chunks = Vec::new()` and pushes every `OutputChunk` with no ceiling on count, bytes or time, returning them in `ExecutionReport.chunks`; `record_completed(&attempt_id, usage.as_ref(), now)` journals the USAGE and not the output |
| `R-6-27-9` | 2 | owned | `SIGNOFF-REPAIR.4.4` | goal line "Close terminal-result/outgoing-event crash gaps", verbatim. Live: the supervisor journals the completion and RETURNS; the worker persists the outgoing event afterwards, in a separate transaction. A crash between them loses the output, and the retry gate then reads a `completed` status |
| `R-6-27-9` | 3 | owned | `SIGNOFF-REPAIR.4.4` | goal line "settle refused results". Live: `retry.rs:80` is a catch-all `Some(_) => RetryVerdict::Refuse`, so every status outside the four named arms — `completed` included — refuses; and `worker.rs:275` maps EVERY refusal to `report_dead_letter`, which the server auto-quarantines. A completed item reaching the gate is quarantined as a dead letter |
| `R-6-27-9` | 4 | owned | `SIGNOFF-REPAIR.4.4` | goal line "outgoing-event crash gaps", and MEASURED as a contrast: `Node::emit_event` runs `record_outgoing_event` -> `send_event` -> `acknowledge_event`, while `Worker::report_dead_letter` runs `record_outgoing_event` -> `send_event` -> `eprintln!`. The ack exists, the sibling calls it, and this one site omits it, so a delivered dead letter stays pending in the journal for ever |
| `R-6-27-9` | 5 | owned | `SIGNOFF-REPAIR.4.4` | goal line "require explicit duplicate-risk authorization before replaying unknown provider outcomes". Live: `SupervisorError::OutcomeUnknown` is returned as an `Err` from `supervisor.rs:358` and wrapped by `WorkerError::Supervision`, so it leaves `tick()` rather than reaching the retry gate in that tick. ⚠️ NARROWED: the type's own doc says this is deliberate — "a fact, not a retry recommendation: what to do with it is the caller's decision" — and whether a caller's loop re-dispatches in-process is NOT established here |
| `R-6-27-9` | 6 | owned | `SIGNOFF-REPAIR.4.4` | goal line "bound streams/deadlines", verbatim. Live and settled by a census rather than a reading: `worker.rs:382` computes a deadline and ships it as `deadline: Some(deadline)`, and grepping every adapter source and the supervisor for `deadline` returns exactly ONE hit — `contract.rs:58`, the struct field's own declaration. The value is written by one side and read by nobody |
| `R-6-27-9` | 7 | owned | `SIGNOFF-REPAIR.4.4` | "need tool-backed reproduction after full read" is the record's own acceptance for clauses 1–6 and rides on them; it is also `TOOLBOX.md`'s standing requirement, so the leaf inherits it either way |
| `R-70-2` | 1 | attach | `SIGNOFF-REPAIR.4.4` | the negative control cannot tell the gate from the adapter. Live: `node_replacement.rs:549` scripts `FakeAdapter` with `ScriptStep::FailBeforeDispatch`, so the attempt fails whether or not the stale-decision gate refuses, while the comment above it claims "the replacement's dispatch refuses fail-closed (a stale decision…)". ⛔ This leaf's Sources line names the test; its goal line names product mechanisms only |
| `R-70-2` | 2 | attach | `SIGNOFF-REPAIR.4.4` | the record's remedy — count the dispatch calls and use an adapter that WOULD succeed — rides on clause 1 and shares its fate. Recorded separately so a future reader is not told the two were merged |
| `R-70-2` | 3 | attach | `SIGNOFF-REPAIR.4.4` | the loops discard the worker's errors. Live: `for _ in 0..4 { let _ = worker.tick().await; }` — the exact refusal reason is never observed, so a bounded retry exhausting is all the test can see, and that is consistent with several causes |
| `R-71-72-2` | 1 | attach | `SIGNOFF-REPAIR.4.4` | the assertion counts the wrong event type for its own claim. Live: `node_work.rs` ends `a_revocation_invalidates_the_cached_decision_at_the_next_dispatch` by counting `thread.contribution_submitted` events == 1 under the message "the refused revise dispatched nothing" — but a revise is a DISTINCT event type, so that count is 1 whether or not the revise landed. ⭐ The record is fair to the test and so is this row: it also asserts `attempts[0].status == "failed_before_dispatch"`, which IS direct evidence of the refusal path and is what makes it stronger than `R-70-2`'s |
| `R-71-72-2` | 2 | attach | `SIGNOFF-REPAIR.4.4` | the record's remedy — assert the adapter's call count AND the absence of revision events — rides on clause 1 and shares its fate |

## Coverage

Tranche 1 is 7 records and 26 clauses: `R-31-32-1`, `R-47-2`, `R-53-4`, `R-63-1`,
`R-65-2`, `R-83-1` and `R-84-1`.

Tranche 2a is 6 records and 27 clauses: `R-31-32-2`, `R-53-2`, `R-53-3`,
`R-58-2`, `R-78-2` and `R-80-82-1`.

Tranche 2b is 4 records and 19 clauses: `R-6-27-2`, `R-40-42-2`, `R-51-2` and
`R-52-2`.

Tranche 2c is 4 records and 38 clauses: `R-76-77-2`, `R-87-1`, `R-88-1` and
`R-90-1` — the last of which alone carries 21, because its body is 1,383
characters over six scripts. ⭐ **Tranche 2 is therefore COMPLETE**: 14 records,
84 clauses, across `.11.9.1.1.1`–`.3`.

Tranche 3a is 5 records and 20 clauses: `R-48-49-3`, `R-48-49-4`, `R-70-3`,
`R-75-1` and `R-85-1`. ⭐ It carries the ledger's SECOND `declined` row and its
fourth two-records-one-finding pair — and that pair is the first whose shared
finding is a SCHEMA shape rather than a code path.

Tranche 3b is 5 records and 27 clauses: `R-31-32-3`, `R-48-49-1`, `R-50-3`,
`R-6-27-7` and `R-66-1`. ⭐ It carries the ledger's FIRST `none` row, which
completes the closed set: every one of the six states now has an instance.

Tranche 3c is 5 records and 22 clauses: `R-36-39-8`, `R-6-27-3`, `R-6-27-9`,
`R-70-2` and `R-71-72-2`. ⭐ **Tranche 3 is therefore COMPLETE**: 15 records,
69 clauses, across `.11.9.1.2.1`–`.3`. ⚠️ Its narrowest candidate `.10.1` is
the OWNER of only one of this child's five records — the clearest instance yet
that a candidate list is a suggestion.

⛔ Every row carries a state from the closed set above, so a reader may take the
absence of a record as "not yet classified" and nothing else. `R-55-2` sits at
`census-2.md:222` between two of them in record order and is deliberately NOT
here: its narrowest candidate is named by 22 records, which puts it in
`SIGNOFF-REPAIR.11.9.1.5`.

⚠️ Tranche 3 onward is NOT here yet: `SIGNOFF-REPAIR.11.9.1.2`–`.6` hold the
remaining records, split at the natural gaps in the adopted ranking.

⚠️ The `declined` row is no longer hypothetical, and the caption above it says
which one it is. A reader scanning for defects should skip it; a reader auditing
the classification should read its Evidence cell first, because a wrong
`declined` is the only state that removes a clause from view.

## Tranche 2c — the four records whose narrowest candidate leaf is `SIGNOFF-REPAIR.11.2`

Owner: `SIGNOFF-REPAIR.11.9.1.1.3`. Ranking: `SIGNOFF-REPAIR.11.9.1`. Split and
its sizing: `SIGNOFF-REPAIR.11.9.1.1`.

⭐ **This tranche is the ledger's `attach` outlier — 11 of its 38 rows — and the
reason is structural rather than incidental.** Its four records are about SCRIPTS
and GATES, and the leaves that own those (`.11.2`, `.11.3`) carry goal lines
written as a short list of NAMED mechanisms. Anything in the surface but outside
the list is invisible by construction, which is exactly the condition `attach`
was defined for. The product leaves in tranches 1, 2a and 2b carry goal lines
written as PROPERTIES, and they produced 7 `attach` rows in 72.

| Record | Clause | State | Owner | Evidence |
| --- | --- | --- | --- | --- |
| `R-76-77-2` | 1 | owned | `SIGNOFF-REPAIR.5.2` | goal line "Bind call/thread/tenant/actor and grants". Live: `profiles.rs::the_open_call_storm_controls_hold_at_the_dev_scale` posts `/v1/calls` with the literal `thr_00000000-0000-7000-8000-000000000001` and asserts 200 four times; `the_open_call_advertises_to_the_subscribers` does the same with `…0002` |
| `R-76-77-2` | 2 | owned | `SIGNOFF-REPAIR.5.2` | same goal line, the SERVICE half. `open_recruitment_call` parses `thread_id` for FORMAT only, authorizes `ThreadInvite` on a target built from the caller's own `tenant_id` and `thread_id`, and inserts. `migrations/0020_recruitment_calls.sql` declares `thread_id TEXT NOT NULL` with no FK and no composite `(tenant_id, thread_id)` reference, so neither existence nor tenancy is checked anywhere |
| `R-76-77-2` | 3 | attach | `SIGNOFF-REPAIR.5.2` | "confirmed codified fixtures": the two tests above ENCODE the missing bind, so adding it turns them red, and the cheap green is to relax the assertion rather than seed a real thread. The leaf's goal line reaches the handler; nothing in it says a fixture depends on the defect. `.5.1`'s goal line carries a fixture clause and `.5.2`'s does not |
| `R-87-1` | 1 | owned | `SIGNOFF-REPAIR.11.2` | goal line "Protect populated repositories". Live: `scripts/bootstrap.sh:23` gates the whole de-template on `[ -n "$name" ] && [ -f MAINTAINING.md ]` — presence of a file, not a clean/pristine state. ⛔ The record's own second half is REFUTED: this project does NOT retain `MAINTAINING.md`, and did not at the census baseline (`git ls-tree 9c2d2ba` names no such path; `823c2bc` removed it). The sentinel finding stands; the "this active project" framing does not |
| `R-87-1` | 2 | owned | `SIGNOFF-REPAIR.11.2` | goal line "preserve project indexes". Live: inside that branch `bootstrap.sh` overwrites `MEMORY.md` and `docs/decisions/INDEX.md` with seed text and truncates `docs/TASK_TREE.md` from `## Active Task Trees` to EOF. In a copy that still holds `MAINTAINING.md` — the spine repo, by the script's own header — a named invocation destroys all three |
| `R-87-1` | 3 | owned | `SIGNOFF-REPAIR.11.2` | goal line "enforce ownership before all changes". Live: step 3 renames the crate at `:88` (a code change) and step 3b seeds the owning `docs/tasks/BOOTSTRAP.md` at `:100`. The comment at `:93`–`:99` argues the FIRST COMMIT passes, which is a different claim from the doctrine's |
| `R-87-1` | 4 | attach | `SIGNOFF-REPAIR.11.2` | portability, which no part of the goal line reaches. 8 bare `sed -i` sites in `bootstrap.sh`, the first at `:61` — AFTER `rm -f MAINTAINING.md …` and after both seeds are written — under `set -euo pipefail`, so a failure aborts mid-destruction. MEASURED: `/usr/bin/sed -i '<script>' <file>` returns rc=1 here, consuming the script as the backup suffix. ⚠️ Masked on this machine: `sed` resolves to GNU sed 4.10 via Homebrew's `gnubin`, so the run SUCCEEDS here and fails on a stock macOS |
| `R-87-1` | 5 | owned | `SIGNOFF-REPAIR.11.2` | goal line "correct table/path checks". Live: `scripts/check_docpaths.sh:11` builds its pattern from the `/Users` and `/home` prefixes alone. This repository's own checkout sits under `/Volumes`, so DOCPATH cannot see the one absolute path this project would actually leak — measured on a line carrying one path of each kind: the home-directory half matches and the `/Volumes` half does not |
| `R-87-1` | 6 | owned | `SIGNOFF-REPAIR.11.2` | goal line "correct table/path checks". Live: `scripts/check_compatibility_matrix.sh:19` asserts the token with `grep -q "\| \`$TOKEN\` \|"` over the whole file. `SDK_VERSION` is `"1"`, so ANY cell reading `` `1` `` in ANY column of ANY table satisfies it; nothing binds the token to the `sdk_version` column |
| `R-87-1` | 7 | attach | `SIGNOFF-REPAIR.11.2` | report integrity, which the goal line does not reach. `:36`–`:47` grep three test NAMES in the suite file and in the matrix. A name present in a source file is not a test that ran, so the matrix's `qualification` column can read `conformance-suite` for a test that is `#[ignore]`d, renamed in place or failing |
| `R-87-1` | 8 | handled | `SIGNOFF-REPAIR.11.2` | "Track bounded doctrine fixes separate from authority repair" is the record's ROUTING instruction rather than a finding, and the tree already obeys it: the doctrine fixes sit in `.11.2` and the authority repairs in the `.3`/`.4` families. Recorded separately so a future reader is not told it was merged into clause 7 |
| `R-88-1` | 1 | owned | `SIGNOFF-REPAIR.11.2` | goal line "reject failed process censuses". Live: `scripts/check_no_background_jobs.sh:43` sets `-uo pipefail` and NOT `-e`; `:69` and `:80` both end `2>/dev/null`. MEASURED by reproducing the control flow with both censuses failing: it prints `handoff: OK — no project-owned background job is running` and returns rc=0 |
| `R-88-1` | 2 | attach | `SIGNOFF-REPAIR.11.2` | an over-broad exclusion is not a failed census, so the goal line does not reach it. `:91` skips any command line containing `*/.codex/*` BEFORE the handle test, and the `case` matches the whole line. MEASURED: a `python3 scripts/run_pg_tests.py` invocation whose `--config` argument sits under a home-directory `.codex` tree is excluded on that ARGUMENT while holding repository files |
| `R-88-1` | 3 | attach | `SIGNOFF-REPAIR.11.2` | a documented-vs-actual scope divergence, which the goal line does not reach. `:33` states "it censuses THIS uid's processes" and `:80` runs `ps -Ao`, all uids. MEASURED here: 620 rows across 40 uids, 390 of them this uid — 37% of the command-line arm's population is other users. The stated honest limit ("a project job running as another user is not seen") is false for that arm |
| `R-88-1` | 4 | owned | `SIGNOFF-REPAIR.11.2` | goal line "correct table/path checks against primary specifications", and the record's "must verify primary GFM spec" is CONFIRMED by the renderer this project publishes with. `check_table_arity.sh:39`–`:44` treats a pipe inside a code span as non-separating; mdbook/pulldown-cmark splits on it and DISCARDS the excess. Its self-test arm at `:73` asserts 0 defects for `` \| `x \| y` \| 2 \| `` — a false green. Live damage: 2 tracked rows lose cells today while the gate reports 0 |
| `R-88-1` | 5 | owned | `SIGNOFF-REPAIR.11.2` | goal line "validate staged evidence for the actual leaf". MEASURED over the registered checks: 9 consume the staged file list, and **7 then read the WORKTREE** — `check_docpaths`, `check_lesson_promotion`, `check_lockstep_claim`, `check_routing_evidence`, `check_task_acceptance`, `check_task_tree_ownership`, `check_waiver_routing`. Only `check_gap_claims` and `check_table_arity` read `git show :path` |
| `R-88-1` | 6 | owned | `SIGNOFF-REPAIR.11.2` | "Maintain explicit evidence-limit doctrine, strengthen material bypasses" is the record's remediation ask; it rides on clauses 1–5 and shares their fate. Recorded separately so a future reader is not told the six were merged |
| `R-90-1` | 1 | owned | `SIGNOFF-REPAIR.11.3` | goal line "Protect restore targets". Live: `scripts/restore.sh:11` runs `pg_restore --clean --if-exists` against `$RESTORE_DATABASE_URL`, and NOTHING compares that URL against `$DATABASE_URL` or refuses a live target. The header at `:2`–`:4` promises "an ISOLATED database … never into the live database"; the promise is prose |
| `R-90-1` | 2 | owned | `SIGNOFF-REPAIR.11.3` | goal line "and secrets". Live: `scripts/restore.sh:10` echoes `$RESTORE_DATABASE_URL` verbatim, password included, before doing anything |
| `R-90-1` | 3 | handled | `SIGNOFF-REPAIR.2.2` | the PG-RUNNER half, and `.2.2`'s own disposition table says so: "PG runner now verifies process shutdown before deletion, records failure evidence and supports explicit ordering". Live in `run_pg_tests.py`: `finish` calls `stop_group`, records `shutdown-unverified` and re-raises, and removes the cluster only on success |
| `R-90-1` | 4 | attach | `SIGNOFF-REPAIR.11.3` | the DEV half of the same sentence, which the same disposition explicitly leaves open ("the other dev/demo/load/restore/scaffold findings remain `.11.2`–`.11.4`"). Live: `scripts/dev.sh:53`–`:55` runs `pg_ctl … stop >/dev/null 2>&1 \|\| true` then `rm -rf "$TMP"` unconditionally and prints "torn down" either way; `:114` then counts zero residue directories and PASSES while a postmaster may still be alive with its data directory deleted. "Reap jobs" does not reach a cleanup that lies |
| `R-90-1` | 5 | attach | `SIGNOFF-REPAIR.11.3` | `dev` is in this leaf's owned surfaces and this clause is in none of its goal-line mechanisms. Live: `dev.sh` computes `ROOT` at `:19` and never `cd`s to it, so `cargo build --bins -q` at `:66` builds the CALLER's repository while `:68`–`:70` run `$ROOT/target/debug/rb-server`. Build one tree, run another |
| `R-90-1` | 6 | attach | `SIGNOFF-REPAIR.11.3` | same absence in the goal line. MEASURED: `project_env.py` sets `CARGO_HOME=$ROOT/.project-data/cargo` and `TMPDIR=$ROOT/.project-data/tmp` on filesystem `100001c0000001a` (the repository's); the ambient environment leaves `CARGO_HOME` unset — cargo falls back to `$HOME/.cargo` — and `TMPDIR=/var/folders/…`, both on `10000100000001a`. A DIFFERENT filesystem, which §13 forbids. ⚠️ `make dev` wraps the script in `$(PROJECT_RUN)` and is safe; the script's OWN documented invocation `bash scripts/dev.sh --check` is not |
| `R-90-1` | 7 | owned | `SIGNOFF-REPAIR.8.1` | goal line "reconcile durable unresolved challenges with close contracts". ⭐ The SAME finding as `R-80-82-1` clause 1, reached from the FIXTURE side — the demo closes thread A on a decision terminal with a durable open challenge. ⛔ Not re-owned and the record's qualifier is preserved verbatim: it may be an explicit historical acceptance, so the repair must reconcile the behaviour rather than silently change it |
| `R-90-1` | 8 | owned | `SIGNOFF-REPAIR.11.3` | goal line "verify HTTP status and negative controls". Live: `demo_two_host.sh:481` and `:520` run `bash -c "! cli inspect thread … \| grep -q revision_submitted"`. ⛔ MEASURED, and it INVERTS the record's own remedy: the four-way truth table is rc=0 in ALL cells — adding `pipefail` changes nothing, because `!` negates a pipeline whose status is 1 whether the CLI failed or merely printed no match. The fix is to check the producer's status separately, not to set an option |
| `R-90-1` | 9 | owned | `SIGNOFF-REPAIR.11.3` | goal line "verify HTTP status". Live: every evidence-capturing `curl` in the demo (`:243`, `:369`, `:393`, `:418`, `:550`, `:556`, `:564`, `:568`, `:582`–`:588`) uses `curl -s` with no `-f` and no status inspection, so a 401 or 500 body is written to the evidence file and the run continues |
| `R-90-1` | 10 | owned | `SIGNOFF-REPAIR.11.3` | goal line "validate identifiers". Live: `:249`–`:267`, `:303`–`:305` and `:489` assign `jq -r` output with no `null` guard, unlike `load_harness.sh:67`/`:75` which do check. ⚠️ NARROWER than the record implies and the difference is measured: `set -e` plus `pipefail` DOES abort when `cli` exits non-zero, so the reachable case is a SUCCESSFUL call whose JSON lacks the field — `jq -r` then prints `null`, exits 0, and the literal string `null` travels on as a principal id |
| `R-90-1` | 11 | owned | `SIGNOFF-REPAIR.11.3` | goal line "ensure requested load counts". Live: `load_harness.sh:80` is a ceiling division and each of `CONCURRENCY` workers runs `PER_WORKER` requests. MEASURED: `--commands 100 --concurrency 8` issues 104, `--commands 10 --concurrency 3` issues 12, `--commands 1 --concurrency 8` issues 8 — and the summary and the throughput are computed from the ISSUED total |
| `R-90-1` | 12 | owned | `SIGNOFF-REPAIR.11.3` | same goal-line clause, and the sharper half. MEASURED: `--commands 0` and `--commands -5` both give `PER_WORKER=0`, issue ZERO requests, and satisfy both exit gates — `[ 0 -ge 0 ]` and `[ 0 -ge -5 ]` are true — so the script prints `PASS: every command committed (200) and the summary is recorded`. A green capacity run that measured nothing, reachable by a plain typo |
| `R-90-1` | 13 | owned | `SIGNOFF-REPAIR.11.3` | goal line "avoid fixed-port/output collisions". Live: `PORT=4391` at `:19` with no flag to change it, and `LOG_DIR="target/load"` with `latencies.txt` and `server.log` at fixed names. Two concurrent runs clobber both files and fight for the port |
| `R-90-1` | 14 | owned | `SIGNOFF-REPAIR.11.3` | same goal-line clause, the readiness half. Live: `:48` probes with `curl -s -o /dev/null` and no `-f`, so an UNRELATED server already listening on 4391 satisfies the wait; the harness then drives that server while its own `rb-server` exited on a bind failure |
| `R-90-1` | 15 | attach | `SIGNOFF-REPAIR.11.3` | not reached by "reap jobs" or by any other named mechanism. Live: the worker `curl` at `:91` carries no `--max-time` and no `--connect-timeout`, so one hung request stalls the run indefinitely; and `wait $WORKER_PIDS` at `:101` has its status discarded, so a worker subshell that died is invisible |
| `R-90-1` | 16 | attach | `SIGNOFF-REPAIR.11.3` | "honest demo evidence" is about the DEMO; the load summary's honesty is named nowhere. Live: `:109`–`:110` compute p50/p95 over EVERY row of the latencies file, non-200s included, so a fast refusal pulls the published percentile down |
| `R-90-1` | 17 | owned | `SIGNOFF-REPAIR.11.3` | goal line "reap jobs". Live: the EXIT trap at `:44` kills `$SERVER_PID` only — it never waits for it, and it never touches the worker subshells or the `curl` processes beneath them. `MEMORY.md`'s standing warning is the same fact: a parent's death does not propagate |
| `R-90-1` | 18 | handled | `SIGNOFF-REPAIR.2.2` | the suite-count and ordering half, dispositioned in `.2.2`'s own table: "Baseline had 30 server suites; the new guard makes the current count 31" and "supports explicit ordering". ⚠️ The count has since moved again — `run_pg_tests.sh --list` prints 43 today — which is why an exact count in prose was the wrong instrument and the runner's own `--list` is the right one |
| `R-90-1` | 19 | owned | `SIGNOFF-REPAIR.11.2` | goal line "preserve project indexes". Live: `scripts/update_scaffold.sh:35` lists `docs/TASK_TREE.md` in `NEUTRAL`, and `:64` copies the donor's version over it — destroying this project's 15-row Active Task Trees index, the one `INDEX-FRONTIER` gates. ⛔ The script's own header at `:7`–`:9` claims "all of docs/tasks/ + docs/decisions/ records is deliberately left alone", so the file contradicts itself |
| `R-90-1` | 20 | handled | `SIGNOFF-REPAIR.11.2.2` | REPAIR-0170. `update_scaffold.sh:19`–`:20` now derives its scratch from `$ROOT/target/doctrine_scratch` with the rule in a comment naming the leaf; this was the largest of the seven sites by bytes written, because it clones a repository into it |
| `R-90-1` | 21 | attach | `SIGNOFF-REPAIR.11.2` | the donor-copy half, which no goal-line mechanism reaches. Live: `:21`–`:22` take the `cp -R "$URL" "$tmp/reasonbraid"` branch for a LOCAL donor path, copying the donor's whole working directory — its `target/`, its caches and any untracked secrets — into this repository's scratch. The `git clone --depth 1` branch beside it copies only tracked history |

## Tranche 2b — the four records whose narrowest candidate leaf is `SIGNOFF-REPAIR.10.2`

Owner: `SIGNOFF-REPAIR.11.9.1.1.2`. Ranking: `SIGNOFF-REPAIR.11.9.1`. Split and
its sizing: `SIGNOFF-REPAIR.11.9.1.1`.

⭐ This tranche carries the ledger's **first `declined` row**. The state was put
in the vocabulary before any instance existed, precisely because a deliberate
rejection is invisible to every search; `R-51-2` clause 4 is the first one.

| Record | Clause | State | Owner | Evidence |
| --- | --- | --- | --- | --- |
| `R-6-27-2` | 1 | owned | `SIGNOFF-REPAIR.10.2` | goal line "require actual coverage of every invariant". Live: `certify` takes ONE scenario with ONE `Trigger`; two checks always run and the `match trigger` adds exactly one more — **3 of the 6** — while the conformance box it then writes reads "the six §19.4 invariants passed" |
| `R-6-27-2` | 2 | attach | `SIGNOFF-REPAIR.10.2` | the `Trigger::Complete` arm's terminal search matches `Completed { .. } \| FailedKnown { .. }` under a comment reading "a terminal `Completed` event exists", so a run that FAILED satisfies the completion invariant. That is a WRONG invariant, not missing coverage, and the goal line reaches only the latter |
| `R-6-27-2` | 3 | attach | `SIGNOFF-REPAIR.10.2` | `drain` is `while let Some(event) = handle.next().await` with no ceiling on count, bytes or time, and every trigger arm but one calls it. Nothing in the goal line bounds the harness's own resource use |
| `R-6-27-2` | 4 | owned | `SIGNOFF-REPAIR.10.2` | goal line "Bind adapter identity/capabilities/artifact to signed complete scenario evidence", which is the audit the clause asks for |
| `R-40-42-2` | 1 | handled | `SIGNOFF-REPAIR.4.2.7` | ⭐ handled AND the source carries its own disposition: `ca.rs` now holds an encoder-only module whose doc records that `from_hex` indexed `&s[i..i + 2]`, had no caller, and was REMOVED rather than repaired because a dead well-named decoder beside a private correct one is what the next caller reaches for |
| `R-40-42-2` | 2 | unowned | `SIGNOFF-REPAIR.4.1.6` | `issue_node_leaf` calls `CertificateParams::new(vec![host_claim]).expect("leaf params")`, and `host_claim` is an unvalidated caller string carried from token issuance into enrolment. MEASURED against rcgen 0.14: of eight inputs only the non-ASCII one returns `Err` — so the panic is reachable and the accept set is far wider than "valid DNS name" |
| `R-40-42-2` | 3 | attach | `SIGNOFF-REPAIR.4.1` | the CA's `not_after` is `now + 365 days`, `ensure_server_ca` loads the stored row without checking that window, and no renewal path exists. The goal line's "bound renewal after revocation" is about the LEAF; nothing in it reaches the CA's own lifetime |
| `R-40-42-2` | 4 | attach | `SIGNOFF-REPAIR.4.1` | a leaf is `now + LEAF_TTL_SECS` (600 s) with no comparison against the issuer's `not_after`, so a leaf issued in the CA's last ten minutes is signed to outlive its issuer. Same absence in the goal line as clause 3, recorded separately because a CA-renewal repair does not by itself add the comparison |
| `R-51-2` | 1 | attach | `SIGNOFF-REPAIR.5.1` | `VisibilityPolicy`'s doc says "every named field defaults to `self_only`"; its `Default` sets five fields `Network`, six `Tenant` and three `SelfOnly`, and `#[serde(default)]` means an omitted policy takes that. A submission that names no policy is published far wider than the doc promises. "Prevent private-feature leaks" points a census at `field_visible`, not at `Default` |
| `R-51-2` | 2 | attach | `SIGNOFF-REPAIR.5.1` | `field_visible`'s doc example is reversed: it says "a `tenant` field is visible to the tenant, the network, and the public", while `visibility_rank(field) <= reader_rank` makes a `Tenant` field visible to a Tenant reader and NOT to Network. The implementation is correct and the sentence beside it is not, which is the case a code census passes over |
| `R-51-2` | 3 | attach | `SIGNOFF-REPAIR.5.1` | `filter_profile` emits exactly the fourteen fields the policy names and then returns, so `incarnation_id` and `visibility` are absent for every reader — including `ReaderClass::Full`, whose own doc says "the role itself (or its accountable owner): the full profile" |
| `R-51-2` | 4 | declined | `SIGNOFF-REPAIR.5.1` | ⭐ **the ledger's first `declined` row.** The MECHANISM is real — a visible `Option::None` serialises to `null` rather than being omitted, since `AgentProfile` carries no `skip_serializing_if`. The CONTRADICTION the clause asserts is not: the only wire-absence claim in this module is `filter_profile`'s "a hidden field is ABSENT, never nulled", which speaks of HIDDEN fields and stays true — absent means hidden, `null` means visible-and-unset, and the two remain distinguishable. Declined as stated; the owner is named so a reader can reopen it against a different claim |
| `R-52-2` | 1 | attach | `SIGNOFF-REPAIR.9.2` | the commit is built with `signature(gix::date::Time::now_local_or_utc())` as author AND committer, so its object id depends on the wall-clock second. The staging step's own doc calls it "the idempotent re-write: the same content commits identically", which is false one second later. No part of the goal line reaches commit determinism |
| `R-52-2` | 2 | owned | `SIGNOFF-REPAIR.9.2` | goal line "make CAS retries recoverable". ⭐ The SAME finding as `R-80-82-1` clause 5, reached by a second record — the third such pair in this activity |
| `R-52-2` | 3 | owned | `SIGNOFF-REPAIR.9.2` | goal line "reconcile DB/Git failure points". Live: the handler builds the manifest inline and nothing durable records the commit id before the Git write, so the reconciler's `expected_immutable` has no stored source for a STAGED publication — which is exactly the §15.8 row it needs it for |
| `R-52-2` | 4 | owned | `SIGNOFF-REPAIR.9.2` | goal line "reconcile DB/Git failure points". Live: `publish` writes all three refs and only then does `mark_effective` touch the database, so a crash between them leaves Git effective and the row staged |
| `R-52-2` | 5 | owned | `SIGNOFF-REPAIR.9.2` | goal line "bind projection and manifest to approved policy". Live: the fetch-back re-reads `manifest_blob` alone — the bundle blob, the tree and the commit are never read back |
| `R-52-2` | 6 | owned | `SIGNOFF-REPAIR.9.2` | goal line "bind projection and manifest to approved policy". Live: `expected` is `digest_sha256_hex(manifest.as_bytes())` — the same string the call was just handed — so the check proves the object store round-tripped and compares no DECLARED digest at all |
| `R-52-2` | 7 | owned | `SIGNOFF-REPAIR.9.2` | goal line "constrain filesystem targets". Live: the module doc says "into the LOCAL bare repository" and `gix::open(repo_path)` accepts a non-bare one |
