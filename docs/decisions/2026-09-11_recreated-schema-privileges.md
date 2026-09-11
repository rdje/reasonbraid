---
answers:
  - What must a fixture restore after recreating the public schema?
  - How should the site service answer a caller that cannot resolve a qualified name?
  - Why is a checkpoint that stopped never a partial pass?
---
# A recreated schema is not the schema a database shipped with

- Owner: `SIGNOFF-REPAIR.11.4.3.1.2.14`; REPAIR-0065.
- Evidence: docs/tasks/artifacts/signoff_review/site-operator-schema-usage.md.

A fresh database ships `public` owned by `pg_database_owner` with `USAGE`
granted to `PUBLIC`. `CREATE SCHEMA public` produces a schema owned by the
creating role with no PUBLIC grant at all. The two look identical in every
query that names a table and differ for every role that is not the owner, which
is why the difference can sit undetected in a shared test database for twenty
suites before surfacing somewhere unrelated.

A fixture that mutates database-wide privilege state owns restoring it. This is
the same rule the cleanup plans already carry for rows, applied where no cleanup
plan can reach: an access-control entry is not a row any fixture deletes. Where a
fixture recreates a schema it restores both properties it silently dropped — the
owner and the PUBLIC grant — through one shared helper, so the next site to be
added cannot forget one of them.

A service must answer with the truth it actually has. A caller lacking `USAGE`
on a schema cannot resolve a qualified name, so a privilege probe written as
`has_table_privilege(CURRENT_USER, 'public.site_audit', 'INSERT')` raises
SQLSTATE 42501 instead of returning false — and the service then reports a
dependency failure to a caller whose real problem is that they are not an
operator. Identify the object by catalogue OID, which every role may read, and
the refusal stays honest. An unprivileged caller is refused either way; the
defect is that the wrong reason was published.

A control that has never failed on the defect it was written for is not yet
evidence. This one was run against the unchanged production query and reproduced
the exact SQLSTATE before being accepted. Where a control must disturb shared
state to create its condition, it restores that state BEFORE it asserts, so a
failing expectation cannot leave the disturbance behind — and it separately
proves the condition held while the call was made, so it cannot quietly become a
control that tests nothing.

An assertion that discards the value it received costs more than it saves.
`assert!(matches!(x, Err(Expected)))` turns a one-line answer into a cluster
autopsy. Refusal assertions name what they observed.

A checkpoint that stopped is a failed checkpoint, never a partial pass, however
many earlier gates were green — and its result is read from its own receipts,
not from a caller's exit status, which a trailing `echo` silently replaces.
