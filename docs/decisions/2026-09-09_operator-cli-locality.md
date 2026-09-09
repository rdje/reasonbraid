---
answers:
  - Why does rb-site accept only an explicit loopback database target?
  - How does operator tooling avoid shared PostgreSQL credential files?
  - What storage checks precede a site authority operation?
---
# Operator CLI connection and storage locality

- Owner: `SIGNOFF-REPAIR.3.2.2`.
- Authority: the director's project-data locality policy and delegated site-authority decision.
- Companion: `docs/decisions/2026-09-09_site-operator-authority.md`.

The operator tool requires an explicit database login and target through
RB_SITE_DATABASE_URL or --database-url. It has no inherited default target and
does not create a database, apply migrations, provision roles or grant permissions.
The current SQLx dependency has no TLS transport; permitting remote plaintext
operator credentials would extend the supported deployment boundary without a
qualified transport. The tool therefore accepts numeric loopback addresses only,
with explicit port/database/login and only the sslmode=disable URL option.

Before any authority or audited inspection write, it checks the selected current
database and the data directory's filesystem device against the current repository.
The database must be local and on that volume. The setting must be inspectable
by the selected login; superuser or pg_read_all_settings supplies that permission.
Run from within the repository on a Unix deployment. Failure to establish these
facts refuses the operation before an audit is attempted.

The CLI clears ambient PostgreSQL overrides before starting its runtime. It
decodes selected URL components explicitly and constructs options with SQLx's
new_without_pgpass, avoiding home/passfile and certificate-file lookup. Explicit
loopback constructor defaults avoid the driver's default socket-directory search.
The tool writes JSON to its caller and creates no state directory or credential
file. Help and error paths hide the selected URL and driver connection details.

Source inspection also found that SQLx's custom PGPASSFILE does not suppress a
home fallback when that file is missing or has no matching entry. The owned test
runner now supplies a matching synthetic fixture credential in a mode-0600 file
on the repository volume. This is test data for a trust-authenticated disposable
server, not a production credential. The driver control uses its public parsed
URL interface to verify that local selection without printing any credential.
The detailed mechanism and source are in
`docs/decisions/2026-09-09_disposable-postgresql-runner.md`; other direct entrypoints
remain concretely owned by `SIGNOFF-REPAIR.11.2`.

The real CLI controls exercise selected-identity issuance, non-operator refusals,
missing storage-inspection permission, hostile ambient PG overrides, redacted
target errors and bounded audited listings. The task leaf records exact results;
source implementation alone is not runtime qualification. These constraints do
not qualify development HTTP authentication or the pending registry route repair.
