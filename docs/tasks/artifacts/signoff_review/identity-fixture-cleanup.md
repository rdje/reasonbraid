# Certified-node residue in identity fixtures

Owner: `SIGNOFF-REPAIR.11.4.3.1.2.6`; REPAIR-0047. Predecessor:
`b0cddfe878520b0368402c44c16a5ba63ba26b05`. Raw controls:
`target/identity-fixture-controls`; failed checkpoint:
`target/checkpoint-ci/full-b0cddfe`.

## What the checkpoint actually established

All 337 tracked non-Markdown files matched the source manifest before execution
and again during independent failure disposition, before this repair. Nine gates
passed: format, strict all-target/all-feature lint, workspace binary builds,
workspace tests, Python controls, book, doctrines, cargo-deny and Gitleaks.
The workspace gate returned zero in 2392.371 seconds, including sixteen real
browser integration tests with the pinned testing runtime. Its 95 successful
result blocks contain 609 nominal passes, zero failures and three explicit ignores
(the schema writer and two live-provider tests). The 271 database-unset skip
messages mean those nominal passes cannot establish live database coverage.

The separate owned PostgreSQL gate returned **101** in 1572.928 seconds. Its first
thirteen registered suites passed; all three `identity_store` tests failed during
fixture setup. The following twenty-six commands and demonstration were unstarted.
The database stopped and remains in `target/pg-tests/run-hbkw0tgu`; it was not
deleted or restarted. Its `command-14.log` retains the three failures. This is a
partial checkpoint, with no public push or remote-CI pass claimed.

`failed-checkpoint-verification.json` independently verifies source identities,
gate statuses, command counts, retained stopped database and absence of all 51
recorded process groups. The native project census also passed before the new
reproduction cases started. These checks establish cleanup of the recorded work;
they do not erase the failed gate or qualify unstarted suites.

## Root cause and falsification

The identity fixture deletes `nodes` but omitted `node_certificates`, whose
`node_certificates_node_id_fkey` uses the existing restrictive foreign key.
The full run's failing node ID is the node produced by the preceding `node_work`
suite. Migration `0011_workload_certificates.sql` and the live catalog agree.

Two independently owned database cases reproduce the distinction:

| Case | Observed result | Disposition |
| --- | --- | --- |
| Original identity suite, fresh database | 3 pass, exit 0; command 14.548717s | Successful cluster removed |
| Original node_work → identity_store | 8 node tests pass, then 3 identity tests fail with SQLSTATE 23503 | Stopped cluster `target/pg-tests/run-fs3xmlih` retained |
| New regression, original cleanup | Named regression fails at the same certificate FK; exit 101 | Stopped cluster `target/pg-tests/run-fvgomcyo` retained |

After node_work and after the failed identity command, read-only snapshots both
show one node, certificate, host, tenant and deployment CA. A clean identity run
passes, so passing the suite alone would have missed this ordering defect.
The diagnostic runner uses the existing owned-cluster and consumed-process APIs;
expected failure exits are recorded explicitly, not presented as test passes.

The live catalog exposes 41 public foreign keys. An explicit-array census finds
eighteen node-purge lists and exactly one omitted certificate dependency. It also
finds fifteen other dependency candidates: fourteen lists omit MCP listener state
before tenants, and the CLI list omits spend breakers. Their concrete runtime and
repair owner is `.11.4.3.1.2.7`. This census covers the identified array-shaped
fixtures, not arbitrary Rust or SQL expressions.

## Repair and verification

Only `crates/reasonbraid-server/tests/identity_store.rs` changes executable code.
The cleanup list now removes certificates before nodes. The new regression seeds
a tenant/host/node/certificate hierarchy, verifies that direct parent deletion
still returns **23503** with the exact certificate constraint, invokes the real
fixture setup again, and asserts all four hierarchy counts are zero. The opaque
certificate bytes exercise storage dependencies; actual certificate issuance is
also exercised by the ordered node_work case. No production migration, foreign
key, application path or unrelated deployment-CA cleanup changes.

| Repaired case | Result | Command elapsed time |
| --- | --- | --- |
| Exact new regression | 1 pass, exit 0 | 34.111584s, including compilation/startup |
| Complete identity suite, fresh database | 4 pass, exit 0 | 16.075238s |
| node_work → complete identity suite | 8 pass, then 4 pass; both exit 0 | 22.750950s + 24.901937s |

All case results and shutdowns are consumed. Successful case clusters are removed;
the two new failing baselines and original checkpoint database remain. The final
ordered snapshot has no nodes, certificates or hosts, retains the deployment CA,
and contains the tenant created by the last successful identity test.
Formatting, strict all-target/all-feature server lint (230.519544 seconds), book
and final independent checks pass, exit zero. The final verifier proves all
seventeen recorded case/check groups absent, all other 336 non-Markdown source
files unchanged, original failure-evidence hashes preserved, exactly fifteen
remaining `.2.7` dependency candidates and nine rendered book markers correct.
README remains 52 lines/2,017 bytes; LIVE_STATUS categories are unchanged.
Exact result: `target/identity-fixture-controls/final-verification.json`.

Re-derivation uses actual test logs and live FK/table snapshots. Falsification uses
fresh-versus-ordered original execution and the regression before the one-entry
cleanup change. Durability is this record, the indexed decision, task acceptance
and immutable commit; raw command/source hashes remain in `baseline-identities.json`
and the per-case receipts. Full checkpoint resumption follows `.2.7`.

## Native startup observations remain separate

During the b0cddfe run, bounded samples captured journal, authority and budget
executables at `_dyld_start` before application entry, with respectively 92, 85
and 84 main-thread samples and 96 KiB footprints. Each later completed without
intervention. The budget test body reported 0.07 seconds after its prolonged
startup wait. These observations do not identify the underlying host cause.

For the distinct atomic_transaction executable, macOS log events with the same
PST path identity show scan-work start and completion/wakeup separated by
21.868428 seconds. This identifies participation in host security evaluation,
not total process-start latency, CPU time, or the cause of the other waits.
Exact-name authority and budget queries yielded only their own query events;
that is not proof that evaluation was absent. The proposed budget `--list`
comparison did not execute because its original process had already completed.

Preserve `journal-startup-*`, `authority-startup-*`, `budget-startup-*`,
`budget-list-comparison.json`, `native-startup-os.*`, `named-startup-os.*`,
`atomic-evaluation-os.*` and `startup-evaluation-interval.json` beneath the
checkpoint directory. Existing `.11.2` owns further diagnosis. No host security
policy, signature, quarantine attribute or OS setting was changed.
