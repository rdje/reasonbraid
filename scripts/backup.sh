#!/usr/bin/env bash
# backup.sh — the dev-profile PostgreSQL backup (PHASE-2.4.1; ROADMAP §17.5).
#
# A custom-format pg_dump into a dated file under target/backups/. The dev
# profile's dump is PLAINTEXT (no object store, no key story — the encryption
# deferral is named in the .4.3 record). A backup that has never been
# RESTORED is not a recovery control: the restore exercise is the guard's
# backup_restore suite, and scripts/restore.sh is the manual path.
set -euo pipefail

: "${DATABASE_URL:?set DATABASE_URL (the control-plane database)}"

STAMP="$(date +%Y%m%d-%H%M%S)"
DEST="$(dirname "$0")/../target/backups"
mkdir -p "$DEST"
FILE="$DEST/reasonbraid-$STAMP.dump"

echo "backup: dumping $DATABASE_URL -> $FILE"
pg_dump --format=custom --no-owner --file "$FILE" "$DATABASE_URL"
echo "backup: wrote $FILE ($(wc -c < "$FILE" | tr -d ' ') bytes)"
