#!/usr/bin/env bash
# restore.sh — the restore TEST: a backup.sh dump into an ISOLATED database
# (PHASE-2.4.1; ROADMAP §17.5: restore automation into an isolated
# environment, never into the live database).
#
# SIGNOFF-REPAIR.4.6.1.5.2: the dump is checked against its backup receipt
# BEFORE anything is restored (a truncated or altered dump is refused), the
# restored database must carry the applied migrations, and only then is a
# restore receipt written (<dump>.restore.json). §17.5: a backup that has never
# been restored is not a recovery control — this receipt is what makes one.
#
# SIGNOFF-REPAIR.11.3.2: `pg_restore --clean` drops what it restores over, and
# nothing stopped it being pointed at the live database or any other populated
# one. The target must now be EMPTY — a database made for the test with
# `createdb` — and must not name DATABASE_URL's database; both refusals come
# before anything is dropped, and neither has an override, because a restore
# TEST into a database that already holds data is not the procedure §17.5
# describes. The connection rides libpq's environment, never a command line,
# where any local user can read it (`SIGNOFF-REPAIR.11.3`).
set -euo pipefail

: "${BACKUP_FILE:?set BACKUP_FILE (a target/backups/*.dump)}"
: "${RESTORE_DATABASE_URL:?set RESTORE_DATABASE_URL (the TARGET database — create it first: createdb <name>)}"

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
RECEIPT="$ROOT/scripts/backup_receipt.py"

python3 -B "$RECEIPT" check "$BACKUP_FILE"
python3 -B "$RECEIPT" guard-restore-target
echo "restore: $BACKUP_FILE -> $(python3 -B "$RECEIPT" redact-env RESTORE_DATABASE_URL)"

# Only the target's own variables reach the clients: every libpq variable is
# cleared first, so an ambient PGSSLMODE or PGPASSWORD meant for another server
# cannot ride along.
PG_NAMES="$(python3 -B "$RECEIPT" pg-names)"
while IFS= read -r name; do
    [ -n "$name" ] && unset "$name"
done <<< "$PG_NAMES"
PG_ENV="$(python3 -B "$RECEIPT" pg-env RESTORE_DATABASE_URL)"
while IFS= read -r assignment; do
    [ -n "$assignment" ] && export "$assignment"
done <<< "$PG_ENV"

OCCUPIED="$(psql -X -q -t -A -v ON_ERROR_STOP=1 -c "
    SELECT count(*) FROM pg_catalog.pg_class c
    JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace
    WHERE n.nspname NOT IN ('pg_catalog', 'information_schema')
      AND n.nspname NOT LIKE 'pg\_%'
      AND c.relkind IN ('r', 'p', 'v', 'm', 'S', 'f')")"
if [ "$OCCUPIED" != "0" ]; then
    echo "restore: the target database is not empty ($OCCUPIED relations) — the restore test" \
        "drops what it restores over, so it restores only into a database made for it" \
        "(createdb <name>); nothing was restored" >&2
    exit 1
fi

pg_restore --clean --if-exists --no-owner --exit-on-error \
    --dbname "$PGDATABASE" "$BACKUP_FILE"
MIGRATIONS="$(psql -X -q -t -A -v ON_ERROR_STOP=1 \
    -c "SELECT count(*) FROM _sqlx_migrations WHERE success")"
python3 -B "$RECEIPT" write-restore "$BACKUP_FILE" RESTORE_DATABASE_URL "$MIGRATIONS"
echo "restore: done"
