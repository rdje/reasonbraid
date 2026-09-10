# Validate complete fixture cleanup plans before deletion

Owner: `SIGNOFF-REPAIR.11.4.3.1.2.7.1`; REPAIR-0048. Predecessor:
`5723c6887aad0d503f15a99a958ddf59688fea9d`. Raw evidence:
`target/fixture-dependency-controls`. Production code, migrations, manifests and
the suite registry are unchanged. Existing fixture callers adopt this support
in `.2.7.2`–`.2.7.4`; this child does not claim they are already repaired.

## Reproduced failures

All 337 tracked non-Markdown source files matched the committed manifest before
the baseline cases and again before implementation. Each case used its own owned
PostgreSQL cluster and the actual existing producer and consumer test executables.

| Sequence | Producer | Consumer | Observed constraint |
| --- | --- | --- | --- |
| mcp_listen → identity_store | 1 pass, exit 0; one subscription/tenant remains | 4 fail, exit 101 | mcp_listen_state_tenant_id_fkey |
| Selected spend-breaker test → cli_end_to_end | 1 pass, exit 0; one breaker/tenant remains | 3 fail, exit 101 | spend_breakers_tenant_id_fkey |
| node_work → mcp_listen | 8 pass, exit 0; certified node and incarnation remain | 1 fails, exit 101 | incarnations_role_id_fkey |

The selected budget producer is
`a_spend_breaker_trips_refuses_and_resets`; six other budget tests are explicitly
filtered. MCP listen uses the internal durable-state API on PostgreSQL. This is
real database-state evidence, not MCP transport or LLM-agent qualification.
Read-only snapshots preserve the relevant dependent-row counts after failure;
they do not claim every earlier cleanup statement left all other data untouched.

All expected results and shutdowns are consumed. Retain these stopped databases:
`target/pg-tests/run-6m2gqupl`, `target/pg-tests/run-i3yumm48`, and
`target/pg-tests/run-767h67a4`. Per-case JSON records exact source hashes, arguments,
elapsed times, process groups, FK catalog and row counts. Original logs are not
overwritten by corrected or later execution.

## Census and repair boundaries

The prior node-only census found eighteen plans and fifteen remaining dependency
candidates after the certificate fix. Expanding to twenty-five explicit DELETE
loops finds twenty affected plans and seventy-five restrictive-FK dependency
edges against the migrated catalog. This does **not** mean seventy-five proved
runtime defects. The six additional partial plans are cards, quota,
classification, mcp_listen, federation and quarantine. The checker and caller
migrations have separate committed children rather than one uncontrolled edit.

The live catalog has 41 public FKs: 38 default NO ACTION and three CASCADE edges
through resource references, snapshots, assessments and derivations. Existing
cascade behavior is not newly classified as a product defect. The new explicit
cleanup contract requires dependent tables to be named, so future adoption makes
those existing deletion effects visible in each plan as well.

This inventory covers the identified array-shaped DELETE loops. It is not a
complete Rust/SQL parser and does not certify other fixture shapes or application
relationships without database FKs. Final coverage reconciliation remains `.2.7.4`.

## Qualified support contract

`crates/reasonbraid-server/tests/support/cleanup.rs` is private test support,
initially included only by `pg_guard`. Its caller supplies a pool obtained through
the existing disposable-database ownership proof, after migrations, and an explicit
ordered list of canonical public-table names.

Before acquiring a connection it rejects malformed names and duplicates. On one
verified acquired connection it checks supported table identities and all incoming
FK dependencies before issuing any DELETE. Names use lowercase ASCII letters,
underscores and non-leading digits, with PostgreSQL's 63-byte limit. Ordinary
public tables without inheritance/partitioning are supported; views, absent tables
and unsupported relations refuse. Every distinct child must be declared earlier
than its parent, including CASCADE and SET NULL/DEFAULT actions. Cross-schema
dependencies cannot be silently confused with a same-named public table. Cycles
refuse. Self-references remain subject to PostgreSQL's single-statement checks.

The checker does not discover and delete additional tables, disable constraints,
issue broad CASCADE, export a production API or reset the schema. It preserves
typed validation details and the original database-error cause. An empty plan is
a no-op. Callers must provide exclusive fixture use without concurrent DDL.
Preflight refusal is mutation-free; a later database/trigger failure can leave
earlier DELETE statements committed. No atomic rollback claim is made.

## Runtime controls

The existing `pg_guard` target now runs eight tests: five new live plan controls,
one existing live ownership control and two existing metadata controls. All pass,
with no skips or ignores, exit zero. Test body: **15.19 seconds**; full command,
including compilation/startup: **47.313559 seconds**. Its successful cluster
`target/pg-tests/run-xmpn46ur` is removed after consumed shutdown.

| New live control | What it falsifies |
| --- | --- |
| invalid_or_incomplete_plans_refuse_before_any_delete | Invalid/unknown/duplicate names, an empty missing child and reversed order cannot delete even an earlier listed witness; valid order and an empty plan have their stated effects. |
| views_inheritance_cross_schema_and_cycles_refuse_without_deletion | Unsupported relation kinds, a same-named child in another schema and both orders of a cycle cannot change protected rows. |
| cascading_and_set_actions_require_explicit_children | CASCADE, SET NULL and SET DEFAULT cannot affect an omitted child; explicit complete plans clear only their declared related rows and preserve an unrelated witness. |
| database_errors_keep_their_cause_and_late_partial_effects_visible | A real FK refusal survives, an injected late SQL error retains SQLSTATE P0001 and its source, earlier committed deletes stay visible, and corrected execution leaves FK enforcement intact. |
| self_referencing_table_is_deleted_as_one_statement | PostgreSQL accepts the qualified whole-table self-reference case while unrelated rows remain. |

Existing forged-proof, replacement-connection, unsafe-target and symlink controls
also pass. Format (0.505024 seconds), strict all-target/all-feature server lint
(11.360956 seconds) and book (0.202230 seconds) return zero. Independent final
verification confirms fourteen recorded groups absent, all three stopped baseline
databases retained, 336 other existing non-Markdown source files and all twenty-five
caller plans unchanged, 41 public FKs reconciled and nine rendered markers correct.
README remains 52 lines/2,017 bytes, LIVE_STATUS categories are unchanged and the
registry still has twelve packages, eighty-six test-enabled targets and forty
PostgreSQL commands. Diff checks pass; all results are consumed. An initial
source-scope diagnostic expected the opposite module declaration order. Its
failure is retained in verifier-order-diagnostic.log; correcting only that exact
rustfmt-order expectation makes the final verifier pass without changing code. Caller integration remains pending; the original
three failing consumer fixtures have not been silently changed in this child.

Claim verification is split between live producer/consumer failures, controls
that independently observe protected rows and exact database errors, and durable
source/result records. Retain the earlier identity and full-checkpoint failure
evidence. Full local/remote CI and the authorized public push remain incomplete.
