#!/usr/bin/env python3
"""Receipts for the dev-profile backup and restore scripts (SIGNOFF-REPAIR.4.6.1.5.2).

`scripts/backup.sh` and `scripts/restore.sh` are the documented backup procedure
(ROADMAP §17.5). This helper is what makes their work REPORTABLE: each script
calls it only after its own step has succeeded, and it writes a receipt beside
the dump. `rb-server --backup-dir` reads the receipts, re-checks every dump's
size, and reports §18.5's backup/restore status
(`docs/decisions/2026-09-22_an-incident-is-an-open-incident-review-thread-and-a-backup-is-reported-by-its-receipts.md`).

    backup_receipt.py write-backup  DUMP URL_VARIABLE
    backup_receipt.py check         DUMP
    backup_receipt.py write-restore DUMP URL_VARIABLE MIGRATIONS
    backup_receipt.py redact        URL
    backup_receipt.py redact-env    URL_VARIABLE
    backup_receipt.py pg-env        URL_VARIABLE
    backup_receipt.py pg-names
    backup_receipt.py guard-restore-target

⛔ A receipt never carries a credential. A database URL is reduced to its
database NAME, and `redact` prints one without its user information. That is
what the scripts echo, because `backup.sh` used to print `$DATABASE_URL`
verbatim, password and all.

⛔ A URL never rides a command line (`SIGNOFF-REPAIR.11.3`). Any local user
reads another process's arguments, so a URL passed to `pg_restore` or to this
helper disclosed its password for as long as the process ran. The subcommands
that take a URL_VARIABLE read it from the ENVIRONMENT by name, and `pg-env`
turns it into libpq's own variables (`PGHOST`, `PGPASSWORD`, …), one
`NAME=value` per line, for the script to export before calling a client with no
connection argument at all.
"""

from __future__ import annotations

import datetime
import hashlib
import json
import os
import sys
from pathlib import Path
from urllib.parse import parse_qsl, unquote, urlsplit, urlunsplit

BACKUP_FORMAT = "reasonbraid-backup/1"
RESTORE_FORMAT = "reasonbraid-restore/1"
BACKUP_SUFFIX = ".backup.json"
RESTORE_SUFFIX = ".restore.json"


class ReceiptError(Exception):
    """A dump that does not match its receipt, or a receipt that cannot be used."""


def now_utc() -> str:
    return datetime.datetime.now(datetime.timezone.utc).isoformat(timespec="microseconds")


def sha256_of(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1 << 20), b""):
            digest.update(block)
    return digest.hexdigest()


def database_name(url: str) -> str:
    """The database a URL names, and nothing else from it."""
    name = urlsplit(url).path.lstrip("/")
    if not name:
        raise ReceiptError(f"the URL names no database: {redact(url)}")
    return name


def redact(url: str) -> str:
    """The URL without user information or query, safe to print or record."""
    parts = urlsplit(url)
    host = parts.hostname or ""
    # A port that is not a number is left out rather than raised on: the
    # redactor is what prints the refusal of such a URL (`SIGNOFF-REPAIR.11.3.2`
    # found it raising instead).
    try:
        port = parts.port
    except ValueError:
        port = None
    if port is not None:
        host = f"{host}:{port}"
    return urlunsplit((parts.scheme, host, parts.path, "", ""))


# libpq's environment variable for each URL component and query parameter it
# honours (PostgreSQL 16, "Environment Variables"). A parameter outside this
# table is REFUSED, never dropped: a silently ignored `sslmode` downgrades the
# connection without a word.
LIBPQ_PARAMETERS = {
    "host": "PGHOST",
    "port": "PGPORT",
    "user": "PGUSER",
    "password": "PGPASSWORD",
    "dbname": "PGDATABASE",
    "service": "PGSERVICE",
    "options": "PGOPTIONS",
    "application_name": "PGAPPNAME",
    "connect_timeout": "PGCONNECT_TIMEOUT",
    "sslmode": "PGSSLMODE",
    "sslcert": "PGSSLCERT",
    "sslkey": "PGSSLKEY",
    "sslrootcert": "PGSSLROOTCERT",
    "sslcrl": "PGSSLCRL",
    "channel_binding": "PGCHANNELBINDING",
    "gssencmode": "PGGSSENCMODE",
    "target_session_attrs": "PGTARGETSESSIONATTRS",
}


def libpq_environment(url: str) -> dict[str, str]:
    """libpq's variables for one `postgres://` URL, or `ReceiptError` if any part
    of it cannot be carried faithfully."""
    parts = urlsplit(url)
    if parts.scheme not in ("postgres", "postgresql"):
        raise ReceiptError(f"not a postgres:// URL: {redact(url)}")
    location = parts.netloc.rpartition("@")[2]
    if "," in location:
        raise ReceiptError(f"a URL naming several hosts is not supported: {redact(url)}")
    try:
        port = parts.port
    except ValueError:
        raise ReceiptError(f"the URL's port is not a number: {redact(url)}") from None
    env: dict[str, str] = {}
    if parts.hostname:
        env["PGHOST"] = parts.hostname
    if port is not None:
        env["PGPORT"] = str(port)
    if parts.username:
        env["PGUSER"] = unquote(parts.username)
    if parts.password is not None:
        env["PGPASSWORD"] = unquote(parts.password)
    name = unquote(parts.path.lstrip("/"))
    if name:
        env["PGDATABASE"] = name
    for key, value in parse_qsl(parts.query, keep_blank_values=True, strict_parsing=bool(parts.query)):
        if key not in LIBPQ_PARAMETERS:
            raise ReceiptError(f"the URL carries `{key}`, which no libpq variable carries: {redact(url)}")
        env[LIBPQ_PARAMETERS[key]] = value
    for variable, value in env.items():
        if "\n" in value or "\0" in value:
            raise ReceiptError(f"{variable} would hold a newline or NUL: {redact(url)}")
    return env


def url_from(variable: str) -> str:
    """The URL held by an environment variable, named rather than passed."""
    url = os.environ.get(variable, "")
    if not url:
        raise ReceiptError(f"{variable} is not set")
    return url


def database_identity(url: str) -> tuple[str, int, str]:
    """(host, port, database) as written — the comparison `guard_restore_target`
    makes. Literal: `localhost` and `127.0.0.1` differ, which is why the target
    must also be EMPTY."""
    env = libpq_environment(url)
    return (env.get("PGHOST", "").lower(), int(env.get("PGPORT", "5432")), env.get("PGDATABASE", ""))


def guard_restore_target(environ: dict[str, str]) -> None:
    """Refuse a restore target that names the live database (`DATABASE_URL`)."""
    target = environ.get("RESTORE_DATABASE_URL", "")
    if not target:
        raise ReceiptError("RESTORE_DATABASE_URL is not set")
    live = environ.get("DATABASE_URL", "")
    if live and database_identity(target) == database_identity(live):
        raise ReceiptError(
            f"RESTORE_DATABASE_URL names the live database ({redact(target)}, the same as "
            "DATABASE_URL); the restore test restores into an ISOLATED database, never the live one"
        )


def write_json(path: Path, body: dict) -> None:
    # Written whole to a sibling and renamed, so a reader never sees half a receipt.
    partial = path.with_name(path.name + ".partial")
    partial.write_text(json.dumps(body, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    partial.replace(path)


def write_backup(dump: Path, database_url: str) -> dict:
    body = {
        "receipt": BACKUP_FORMAT,
        "dump": dump.name,
        "bytes": dump.stat().st_size,
        "sha256": sha256_of(dump),
        "taken_at": now_utc(),
        "database": database_name(database_url),
    }
    write_json(dump.with_name(dump.name + BACKUP_SUFFIX), body)
    return body


def read_backup(dump: Path) -> dict:
    path = dump.with_name(dump.name + BACKUP_SUFFIX)
    try:
        body = json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError:
        raise ReceiptError(
            f"{dump.name} has no backup receipt ({path.name}); only a dump taken by "
            "scripts/backup.sh can be restore-tested"
        ) from None
    except json.JSONDecodeError as error:
        raise ReceiptError(f"{path.name} is not a readable receipt: {error}") from None
    if body.get("receipt") != BACKUP_FORMAT or body.get("dump") != dump.name:
        raise ReceiptError(f"{path.name} is not a backup receipt for {dump.name}")
    return body


def check(dump: Path) -> dict:
    """The dump matches its receipt byte for byte, or `ReceiptError` says how not."""
    receipt = read_backup(dump)
    if not dump.is_file():
        raise ReceiptError(f"{dump.name} is missing")
    size = dump.stat().st_size
    if size != receipt["bytes"]:
        raise ReceiptError(
            f"{dump.name} is {size} bytes; its receipt recorded {receipt['bytes']}"
        )
    digest = sha256_of(dump)
    if digest != receipt["sha256"]:
        raise ReceiptError(
            f"{dump.name} does not match its receipt: sha256 {digest}, "
            f"recorded {receipt['sha256']}"
        )
    return receipt


def write_restore(dump: Path, target_url: str, migrations: int) -> dict:
    receipt = check(dump)
    if migrations < 1:
        raise ReceiptError(
            f"the restored database carries {migrations} applied migrations; "
            "a restore test that restored no schema proves nothing"
        )
    body = {
        "receipt": RESTORE_FORMAT,
        "dump": dump.name,
        "sha256": receipt["sha256"],
        "restored_at": now_utc(),
        "target_database": database_name(target_url),
        "migrations": migrations,
    }
    write_json(dump.with_name(dump.name + RESTORE_SUFFIX), body)
    return body


def main(argv: list[str]) -> int:
    command, args = (argv[1], argv[2:]) if len(argv) > 1 else ("", [])
    try:
        if command == "write-backup" and len(args) == 2:
            body = write_backup(Path(args[0]), url_from(args[1]))
            print(f"backup: receipt {body['dump']}{BACKUP_SUFFIX} "
                  f"({body['bytes']} bytes, sha256 {body['sha256'][:12]}…)")
        elif command == "check" and len(args) == 1:
            receipt = check(Path(args[0]))
            print(f"restore: {receipt['dump']} matches its receipt "
                  f"(sha256 {receipt['sha256'][:12]}…)")
        elif command == "write-restore" and len(args) == 3:
            body = write_restore(Path(args[0]), url_from(args[1]), int(args[2]))
            print(f"restore: receipt {body['dump']}{RESTORE_SUFFIX} "
                  f"({body['migrations']} migrations in {body['target_database']})")
        elif command == "redact" and len(args) == 1:
            print(redact(args[0]))
        elif command == "redact-env" and len(args) == 1:
            print(redact(url_from(args[0])))
        elif command == "pg-env" and len(args) == 1:
            for variable, value in libpq_environment(url_from(args[0])).items():
                print(f"{variable}={value}")
        elif command == "pg-names" and not args:
            print("\n".join(sorted(LIBPQ_PARAMETERS.values())))
        elif command == "guard-restore-target" and not args:
            guard_restore_target(dict(os.environ))
        else:
            print(__doc__, file=sys.stderr)
            return 2
    except ReceiptError as error:
        print(f"backup_receipt: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
