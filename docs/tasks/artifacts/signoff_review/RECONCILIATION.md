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

⚠️ `declined` has **zero** rows today. That is not evidence it never happens — it
is invisible to every search, which is why the state exists in the vocabulary
before an instance does.

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
| `R-53-2` | 2 | unowned | `SIGNOFF-REPAIR.11.10` | the `route` half, which the same repair never reached. Live in `crates/reasonbraid-server/src/regions.rs`: three `.map_err(|_| RegionRefusal::…)` turn any `sqlx::Error` into `UndeclaredRegion` or `CrossRegionRefused` |
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

## Coverage

Tranche 1 is 7 records and 26 clauses: `R-31-32-1`, `R-47-2`, `R-53-4`, `R-63-1`,
`R-65-2`, `R-83-1` and `R-84-1`.

Tranche 2a is 6 records and 27 clauses: `R-31-32-2`, `R-53-2`, `R-53-3`,
`R-58-2`, `R-78-2` and `R-80-82-1`.

⛔ Every row carries a state from the closed set above, so a reader may take the
absence of a record as "not yet classified" and nothing else. `R-55-2` sits at
`census-2.md:222` between two of them in record order and is deliberately NOT
here: its narrowest candidate is named by 22 records, which puts it in
`SIGNOFF-REPAIR.11.9.1.5`.

⚠️ Tranche 2's other eight records are NOT here either: they are
`SIGNOFF-REPAIR.11.9.1.1.2` and `.11.9.1.1.3`, split off on the sizing
measurement in `SIGNOFF-REPAIR.11.9.1.1`.
