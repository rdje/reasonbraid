#!/usr/bin/env bash
# restore.sh — restore a backup.sh dump into an ISOLATED database
# (PHASE-2.4.1; ROADMAP §17.5: restore automation into an isolated
# environment, never into the live database).
set -euo pipefail

: "${BACKUP_FILE:?set BACKUP_FILE (a target/backups/*.dump)}"
: "${RESTORE_DATABASE_URL:?set RESTORE_DATABASE_URL (the TARGET database — create it first: createdb <name>)}"

echo "restore: $BACKUP_FILE -> $RESTORE_DATABASE_URL"
pg_restore --clean --if-exists --no-owner --exit-on-error \
    --dbname "$RESTORE_DATABASE_URL" "$BACKUP_FILE"
echo "restore: done"
