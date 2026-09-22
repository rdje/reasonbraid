#!/usr/bin/env bash
# backup.sh — the dev-profile PostgreSQL backup (PHASE-2.4.1; ROADMAP §17.5).
#
# A custom-format pg_dump into a dated file under BACKUP_DIR (default
# target/backups/, on the repository's own volume). The dev profile's dump is
# PLAINTEXT (no object store, no key story — the encryption deferral is named
# in the .4.3 record). A backup that has never been RESTORED is not a recovery
# control: scripts/restore.sh is the restore test, and the guard's
# backup_restore suite runs both.
#
# SIGNOFF-REPAIR.4.6.1.5.2: only after pg_dump succeeds, a RECEIPT is written
# beside the dump (<dump>.backup.json: bytes, sha256, taken_at, database name),
# which is what `rb-server --backup-dir` reports. The URL is echoed REDACTED —
# this script used to print $DATABASE_URL verbatim, password included.
set -euo pipefail

: "${DATABASE_URL:?set DATABASE_URL (the control-plane database)}"

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
RECEIPT="$ROOT/scripts/backup_receipt.py"
DEST="${BACKUP_DIR:-$ROOT/target/backups}"
mkdir -p "$DEST"
STAMP="$(date -u +%Y%m%d-%H%M%S)"
FILE="$DEST/reasonbraid-$STAMP.dump"
# Second-precision names collide when two backups start in the same second, and
# pg_dump --file overwrites: refuse rather than replace a dump that has a receipt.
if [ -e "$FILE" ]; then
    echo "backup: $FILE already exists — not overwriting it; retry in a second" >&2
    exit 1
fi

echo "backup: dumping $(python3 -B "$RECEIPT" redact "$DATABASE_URL") -> $FILE"
pg_dump --format=custom --no-owner --file "$FILE" "$DATABASE_URL"
echo "backup: wrote $FILE ($(wc -c < "$FILE" | tr -d ' ') bytes)"
python3 -B "$RECEIPT" write-backup "$FILE" "$DATABASE_URL"
