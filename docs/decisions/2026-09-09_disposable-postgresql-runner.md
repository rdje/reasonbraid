---
answers:
  - How can a focused database test run without using an existing database?
  - What proves PostgreSQL has stopped before its files are removed?
  - How does a failed integration run preserve diagnostic evidence?
  - Why is a missing custom passfile insufficient to prevent SQLx home lookup?
---
# Disposable PostgreSQL runner

- Decision: 2026-09-09, `SIGNOFF-REPAIR.2.2.1`.
- Authority: the director's locality, focused-check and continuity requirements.

The previous runner used a fixed loopback port, ambient Cargo/cwd, and removed
its data directory after ignoring a failed stop command. Its single broad test
invocation also prevented bounded authority reproductions.

`scripts/run_pg_tests.sh` now enters the repository-local command environment
and runs `scripts/run_pg_tests.py`. Each invocation creates its own restricted
`target/pg-tests/run-*` directory, random database and available loopback port.
The runner selects installed PostgreSQL 16 tools read-only. PostgreSQL runs as a
foreground child in its own process group. Unix sockets are disabled; no default
socket is created outside the repository.

The runner also sets explicit numeric loopback constructor defaults and supplies
a private matching synthetic fixture password in `fixture.pgpass` under that
owned directory. The password is test data, not an authentication boundary; the
isolated server uses loopback trust authentication. Mode 0600 permits both SQLx
and libpq to consume it without consulting a shared credential store.

This corrects a source-level gap discovered during `SIGNOFF-REPAIR.3.2.2`:
SQLx 0.8.6's [passfile loader](https://github.com/launchbadge/sqlx/blob/v0.8.6/sqlx-postgres/src/options/pgpass.rs)
tries the home passfile if a custom PGPASSFILE is missing or has no matching
record. The old nonexistent `unused.pgpass` placeholder therefore did not stop
that fallback. The CLI integration control checks the driver's parsed options
contain the known owned fixture password, without printing credential material.
No real home passfile is used as a negative probe. Other direct entrypoints keep
their explicit `.11.2` repair ownership.

The runner discards caller DATABASE_URL and libpq connection/service/option
variables. Before creating the database, it reads the connected server's actual
data directory and a random setting supplied to its own PostgreSQL child. Both
must match, and the child must still be alive. An occupied requested port fails
before cluster creation; a port acquisition race cannot authorize a foreign
server. Creation rechecks both identity fields on the same database connection
that executes CREATE DATABASE: `psql` uses a conditional generated statement
with [\gexec](https://www.postgresql.org/docs/16/app-psql.html), quoted input
variables and ON_ERROR_STOP. A mismatched identity yields a read-only SQL error.
A live control redirects this connection to a second owned cluster and proves
that no database is created there. Runtime connection paths are derived from the current root. PostgreSQL's
internal data-directory metadata may contain absolute runtime paths; these are
confined to disposable generated files and are never portable project settings.

Named suites run sequentially with one Rust test thread because current fixtures
purge shared tables. No names runs the existing broad collection, followed by
the demonstration unless RB_DEMO=0. A focused run adds the demonstration only
with `--demo`. `--port` replaces the former PG_PORT override.

Every command runs in a separately supervised process group. Terminal signals
are deferred during child creation until its handle has been recorded; a small
Python exec trampoline restores the inherited signal mask in the child. This
closes a deterministically reproduced signal-during-spawn orphan race. The runner reaps it
and checks that the group has gone. Terminal signals unwind through cleanup.
PostgreSQL receives fast shutdown, then immediate shutdown if needed; the runner
waits for the server and its process group to disappear before removing data.
These signal meanings follow the [PostgreSQL shutdown documentation](https://www.postgresql.org/docs/16/server-shutdown.html);
foreground startup and socket options follow the [postgres reference](https://www.postgresql.org/docs/16/app-postgres.html).

Successful runs remove their exact workspace and check for residue. Failed runs
retain `postgres.log`, numbered command logs, database data and atomically replaced `runner.json` (server/command PIDs, command outcome and state) with state `stopped` after
verified shutdown. Unverified shutdown or an unreaped test command retains the
workspace, records `shutdown-unverified` and returns failure. Never remove a
retained cluster until its processes have been independently confirmed stopped.
A force-killed runner or host crash cannot execute cleanup; its receipt and
PostgreSQL metadata provide the recovery pointers.

## Scope and limits

This runner creates disposable test infrastructure. It uses loopback trust
authentication with synthetic fixtures, matching the existing RLS exercise; it
is intended for a trusted development host, never a production database service.
The random setting identifies the runner's own server, rather than establishing
a security boundary against another process with the same operating-system user.

Direct Rust tests still accept DATABASE_URL independently at this checkpoint.
`SIGNOFF-REPAIR.2.2.2` owns refusal before destructive SQL and migrating CI from
its service container to this runner. The local broad collection includes suites
that current CI does not invoke. Existing demo internals remain owned by `.11.3`.

## Verification

A controlled execution of the baseline script with a stop stub returning 9
returned success and removed the data; no real server was used for that negative
control. Twelve lifecycle controls cover ignored caller targeting, occupied ports,
foreign-server refusal before createdb, command timeout/reaping, stopped failure
evidence, unverified shutdown retention and correct nonzero results. Four real
PostgreSQL controls verify concurrent cluster isolation (including relocated
roots with spaces and apostrophes), creation refusal after a server change,
stopped evidence after a failed command, and SIGTERM cleanup
of both server and command. All fixtures remain under target and are removed
only after verified shutdown. The owning task leaf records the product-suite
result separately.
