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
set -euo pipefail

: "${BACKUP_FILE:?set BACKUP_FILE (a target/backups/*.dump)}"
: "${RESTORE_DATABASE_URL:?set RESTORE_DATABASE_URL (the TARGET database — create it first: createdb <name>)}"

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
RECEIPT="$ROOT/scripts/backup_receipt.py"

python3 -B "$RECEIPT" check "$BACKUP_FILE"
echo "restore: $BACKUP_FILE -> $(python3 -B "$RECEIPT" redact "$RESTORE_DATABASE_URL")"
pg_restore --clean --if-exists --no-owner --exit-on-error \
    --dbname "$RESTORE_DATABASE_URL" "$BACKUP_FILE"
MIGRATIONS="$(psql -X -q -t -A -v ON_ERROR_STOP=1 -d "$RESTORE_DATABASE_URL" \
    -c "SELECT count(*) FROM _sqlx_migrations WHERE success")"
python3 -B "$RECEIPT" write-restore "$BACKUP_FILE" "$RESTORE_DATABASE_URL" "$MIGRATIONS"
echo "restore: done"
