# A fixture that recreated `public`, and the refusal it hid

Owner: `SIGNOFF-REPAIR.11.4.3.1.2.14`; REPAIR-0065. Source: `165cb3a`.
Raw evidence: `target/checkpoint-ci/full-165cb3a` (receipts, logs and a forensic
copy of the failing cluster); the original failed cluster is retained untouched
at `target/pg-tests/run-d4kvj1l5`.

## The checkpoint stop

The full pre-push checkpoint ran the first three commands green and stopped at
the fourth:

| Command | Result |
| --- | --- |
| `01-build` workspace binaries | rc=0, 364s |
| `02-check` format, strict lint, workspace tests with the pinned browser | rc=0, 3,922s |
| `03-python` control modules | rc=0, 24s |
| `04-pg-demo` full owned PostgreSQL collection with `--demo` | **rc=101, 3,137s** |

Thirty-six of forty suites started; thirty-five passed with 259 tests. The
thirty-sixth, `site_authority`, returned `9 passed; 1 failed`:
`issuance_uses_database_identity_and_rechecks_membership_after_waiting` failed
where an unprivileged database login must be refused `Error::OperatorRequired`.
Four later suites, the demonstration and checkpoint gates five to eight never
ran. This is a failed checkpoint, not a partial pass.

**A note on the exit code.** The background invocation ended
`run.sh …; echo "DRIVER EXIT rc=$?"`, so the shell's status was the echo's and
the harness reported "completed (exit code 0)" for a checkpoint that had
stopped. The driver's own receipt said `CHECKPOINT STOPPED at 04-pg-demo`.
Reading the receipt rather than the exit code is what caught it; a command whose
last statement is an `echo` has thrown its status away.

## Narrowing

| Step | Observation |
| --- | --- |
| `site_authority` alone, fresh cluster | 10 passed — the failure is ordering-dependent, not inherent |
| Forensics on a COPY of the retained cluster | the outsider role is a member of `pg_read_all_settings` only, the member role additionally of `reasonbraid_site_operator` — the fixture seeded its roles correctly, so this is not a mis-seeded control |
| `allowlist regions site_authority` | 10 passed — the immediate predecessors are not the trigger |
| the failing cluster's own `postgres.log` | one `ERROR: permission denied for schema public`, and its `STATEMENT` is verbatim the `operator_identity` privilege probe |
| `migration_upgrade site_authority` | **reproduced deterministically** — two suites instead of thirty-six |

The assertion had been `assert!(matches!(…, Err(Error::OperatorRequired)))`,
which discards the value it received. That opacity is the first defect: it is
why a one-line answer needed a cluster autopsy. The three such assertions in the
file now name what they observed, and the reproduction immediately reported:

```text
an unprivileged database login must be refused OperatorRequired; it received
Err(Sql(Database(PgDatabaseError { code: "42501",
  message: "permission denied for schema public", … })))
```

## Root cause, re-derived from the catalogue

`crates/reasonbraid-server/tests/migration_upgrade.rs` recreates the database
from scratch four times with `DROP SCHEMA public CASCADE; CREATE SCHEMA public`.
A fresh *database* ships `public` with a default ACL; a manually created
*schema* does not inherit it. The two clusters compared directly:

| Database | `public` owner | `nspacl` |
| --- | --- | --- |
| the failing one, after `migration_upgrade` | `postgres` | `{postgres=UC/postgres}` |
| a pristine one on the same server | `pg_database_owner` | `{pg_database_owner=UC/pg_database_owner,=U/pg_database_owner}` |

The pristine ACL's `=U/` entry is PUBLIC's `USAGE`. The recreated schema has no
PUBLIC entry at all, so from that suite onward **every non-owner role in the
shared database lost the ability to resolve a qualified name**. Twenty-odd
suites later, `site_authority`'s outsider hit it.

The probe that hit it was:

```sql
pg_catalog.has_table_privilege(CURRENT_USER, 'public.site_audit', 'INSERT')
```

Resolving the qualified *name* requires schema USAGE, so the probe raised 42501
instead of returning `false`. `operator_identity` propagated `Error::Sql`, and
`issue_boundary` returned that instead of `OperatorRequired`.

## Two defects, both repaired

**1. The fixture mutates database-wide privilege state and never restores it.**
This is the shared-state class the `.2.7` family repaired for rows, in a place
no cleanup plan can reach: an ACL is not a row any fixture deletes. All four
recreate sites now go through one helper that restores both lost properties:

```sql
ALTER SCHEMA public OWNER TO pg_database_owner;
GRANT USAGE ON SCHEMA public TO PUBLIC;
```

A single helper rather than four copies, so the fifth site cannot forget.

**2. The service reported a dependency failure where the truth was a refusal.**
Independently of the fixture, a caller without schema USAGE received
`Error::Sql` — HTTP 500 `dependency_unavailable` — when the honest answer is
`OperatorRequired`, HTTP 403 `site_authority_required`. The probe now identifies
the audit table by catalogue OID, which needs no schema privilege, and a missing
table yields `false` rather than NULL. The membership recheck, the freshly
parsed simple-protocol statement and the absence of caller interpolation are all
preserved.

This second defect is NOT a privilege escalation. The outsider was refused
throughout; only the *classification* was wrong. The forensic query confirms the
outsider never held operator membership.

## Verified

| Check | Result |
| --- | --- |
| `migration_upgrade site_authority` (the reproduced sequence) | 4 + 11 passed |
| new control `a_caller_without_schema_usage_is_still_refused_as_a_non_operator` | passes on the repair |
| the same control against the UNCHANGED production query | fails with SQLSTATE 42501 — the control is falsified, not merely green |
| `cargo clippy -p reasonbraid-server --all-targets --locked --offline -- -D warnings` | rc=0, 2m11s |
| `cargo fmt --all -- --check` | rc=0 |

The new control revokes USAGE, captures the outcome, **restores the grant, and
only then asserts** — so a failing expectation can never leave the shared
database without USAGE on `public`. It also probes that the outsider genuinely
lacked USAGE while the call was made, so a future change that silently restores
the grant cannot turn it into a control that proves nothing.

## Honest limits

- The full checkpoint has NOT passed. This repairs the suite that stopped it;
  the four unrun suites, the demonstration and gates five to eight remain
  unqualified until a complete run.
- `02-check` took 3,922 seconds against roughly 670 in earlier records. That
  difference is unexplained and is not claimed to be benign.
- The retained failed cluster and its logs are preserved; the forensic work ran
  on a copy so the original bytes are unchanged.
