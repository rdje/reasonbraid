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

## The closed set of states

| State | Meaning | Next action |
| --- | --- | --- |
| `handled` | a leaf did the work, whether or not it ever named the record | none; the Evidence column names the leaf and its commit |
| `owned` | a leaf owns the clause AND that leaf's own text makes it visible, so its future census will read it | none; execute the owning leaf |
| `attach` | a leaf owns the surface but its own text does NOT make the clause visible | attach the clause to the leaf, or its split drops it — this is `SIGNOFF-REPAIR.11.9`'s exact mechanism, caught before it fires |
| `unowned` | no leaf covers it | open a leaf; the Owner column names the one this ledger opened |
| `declined` | considered and deliberately rejected | none; the Evidence column carries the reason |
| `none` | the record carries no finding | none |

⛔ `attach` is the state the whole activity exists to find. `handled` and `owned`
are good outcomes; `unowned` is a defect the ledger discovered; `attach` is a
defect the ledger discovered **before** it cost anything, which is the cheapest
place to catch it.

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

## Coverage

Tranche 1 is 7 records and 26 clauses: `R-31-32-1`, `R-47-2`, `R-53-4`, `R-63-1`,
`R-65-2`, `R-83-1` and `R-84-1`.

⛔ Every row carries a state from the closed set above, so a reader may take the
absence of a record as "not yet classified" and nothing else. `R-55-2` sits at
`census-2.md:222` between two of them in record order and is deliberately NOT
here: its narrowest candidate is named by 22 records, which puts it in
`SIGNOFF-REPAIR.11.9.1.5`.
