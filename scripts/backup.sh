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
#
# SIGNOFF-REPAIR.11.3.1: the dump is written under a temporary name beside its
# destination and moved into place only when pg_dump has succeeded, by a hard
# link that refuses to replace an existing file, so an interrupted dump never
# sits at a name that looks like a backup. Everything is owner-only (umask 077:
# the dump is the whole control-plane database, in plaintext). And the
# connection reaches pg_dump through libpq's own environment variables, never
# its command line, where any local user could read the password.
set -euo pipefail
umask 077

: "${DATABASE_URL:?set DATABASE_URL (the control-plane database)}"

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
RECEIPT="$ROOT/scripts/backup_receipt.py"
DEST="${BACKUP_DIR:-$ROOT/target/backups}"
mkdir -p "$DEST"
STAMP="$(date -u +%Y%m%d-%H%M%S)"
FILE="$DEST/reasonbraid-$STAMP.dump"
# Second-precision names collide when two backups start in the same second:
# refuse rather than replace a dump that has a receipt. The move below refuses
# too, so a race between this check and it cannot replace one either.
if [ -e "$FILE" ]; then
    echo "backup: $FILE already exists — not overwriting it; retry in a second" >&2
    exit 1
fi
PARTIAL="$(mktemp "$DEST/.reasonbraid-$STAMP.dump.partial.XXXXXX")"
trap 'rm -f "$PARTIAL"' EXIT

# Only the database's own variables reach pg_dump: every libpq variable is
# cleared first, so an ambient one meant for another server cannot ride along.
PG_NAMES="$(python3 -B "$RECEIPT" pg-names)"
while IFS= read -r name; do
    [ -n "$name" ] && unset "$name"
done <<< "$PG_NAMES"
PG_ENV="$(python3 -B "$RECEIPT" pg-env DATABASE_URL)"
while IFS= read -r assignment; do
    [ -n "$assignment" ] && export "$assignment"
done <<< "$PG_ENV"

echo "backup: dumping $(python3 -B "$RECEIPT" redact-env DATABASE_URL) -> $FILE"
pg_dump --format=custom --no-owner --file "$PARTIAL" "$PGDATABASE"
if ! ln "$PARTIAL" "$FILE" 2>/dev/null; then
    echo "backup: $FILE appeared while dumping — not overwriting it; the dump is discarded" >&2
    exit 1
fi
rm -f "$PARTIAL"
echo "backup: wrote $FILE ($(wc -c < "$FILE" | tr -d ' ') bytes)"
python3 -B "$RECEIPT" write-backup "$FILE" DATABASE_URL
