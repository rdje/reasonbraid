---
answers:
  - What prevents integration tests from modifying an existing database?
  - How are replacement PostgreSQL connections checked before fixture writes?
  - How does the PostgreSQL CI job use the same ownership proof as local tests?
---
# Disposable test-pool ownership

- Decision: 2026-09-09, `SIGNOFF-REPAIR.2.2.2`.
- Extends `docs/decisions/2026-09-09_disposable-postgresql-runner.md`.

The baseline authority binary was run against a fresh controlled cluster after
removing the runner's three ownership environment variables. Its selected test
still passed and wrote two authorization rows. Thus DATABASE_URL alone enabled
fixture writes. This reproduction used disposable data owned by the review;
no existing external database was contacted.

The server's 30 existing PostgreSQL suites, CLI integration suite and MCP live
unit tests now obtain their initial pool through the shared test-only module
`crates/reasonbraid-server/tests/support/mod.rs`. The module is included only by
test targets and the MCP test configuration. It is not a production API.

Before connecting, the helper requires the runner's relative cluster directory,
owner token and database name. It finds the current repository from the working
directory, rejects traversal, symlinked or off-volume directories/files, and
reads a bounded receipt. The receipt must describe a running command, the same
workspace/database and the live postmaster PID. DATABASE_URL must match the
receipt's exact loopback endpoint, generated database and fixed connection
options. An absent DATABASE_URL preserves the ordinary offline skip; a supplied
URL without valid ownership fails before migrations or fixture cleanup.

The server proof is checked on every newly opened pool connection: actual data
directory, database and the runner's startup token must all match. The SQLx
[after_connect contract](https://github.com/launchbadge/sqlx/blob/v0.8.6/sqlx-core/src/pool/options.rs)
closes a connection when its callback fails and retries within the configured
acquisition deadline. A runtime control holds an already verified connection,
changes the role default only for new sessions, and proves a new acquisition is
refused. The role default is restored before asserting the result.

Backup/restore database creation and removal also use the verified pool instead
of separate administrative utility connections. Its dump/restore tools operate
on generated databases in the owned cluster. The migration and RLS exercises
retain their intended destructive assertions after initial ownership validation.
Suites and test threads remain serialized by the runner. The new `pg_guard`
suite covers unsafe targets, stale or malformed metadata, symlink refusal,
forged live proof and replacement connections. The existing authority negative
control now fails with missing ownership metadata and leaves zero public tables.

## CI and locality

The PG job selects Ubuntu 24.04 and invokes the same supervised runner instead
of a separately targeted service container. It provisions the pinned compiler
under `.project-data/installed-toolchains`, with Cargo, scratch and other stores
set from the repository launcher. Rustup self-update is disabled. Installed
rustup and PostgreSQL tools are read-only inputs; pg_config supplies the binary
directory at runtime. The full local suite list is also the PG job's list,
including the guard, MCP, CLI and demonstration.

The checked [runner image inventory](https://github.com/actions/runner-images/blob/main/images/ubuntu/Ubuntu2404-Readme.md)
lists Python 3.12, Rustup and PostgreSQL 16.15. The workflow's inline Python was
syntax-checked (system Ruby/Psych for YAML and Python AST for the embedded script)
and its commands reviewed locally; execution on GitHub remains
part of the next push. Other CI/locality repairs retain their existing `.11`
ownership. No global cache or toolchain was deleted or changed by this slice.

## Limits

The runner is for a trusted POSIX development/CI host. These checks prevent
accidental destructive tests against an existing target and bind fixture pools
to the owned server. Another process controlling the same operating-system
account is outside this test-infrastructure trust boundary. This infrastructure
proof does not qualify the product's tenant or site authorization invariants;
those remain the subsequent `.3` repair leaves.
