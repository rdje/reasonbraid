"""`scripts/backup.sh` against a stand-in `pg_dump` (SIGNOFF-REPAIR.11.3.1).

The stand-in is first on PATH, records the arguments and the libpq environment
it was started with, and writes a dump or fails half-way. So these controls
observe what the script actually hands its client and what it leaves on disk,
without a database.
"""

import json
import os
from pathlib import Path
import shutil
import stat
import subprocess
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import project_env

ROOT = Path(__file__).resolve().parents[2]
URL = "postgres://operator:s3cret-pw@db.internal:5432/reasonbraid?sslmode=disable"

STAND_IN = """#!/usr/bin/env bash
# a stand-in pg_dump: record, then write a dump or fail half-way
printf '%s\\n' "$@" > "$RB_STAND_IN_LOG.argv"
env | grep '^PG' | sort > "$RB_STAND_IN_LOG.env"
out=""
while [ $# -gt 0 ]; do
    if [ "$1" = "--file" ]; then out="$2"; shift 2; else shift; fi
done
printf 'PGDMP-partial' > "$out"
if [ "$RB_STAND_IN_RACE" = "1" ]; then
    # another backup lands on the final name while this one is dumping
    base="$(basename "$out")"; base="${base#.}"; base="${base%.partial.*}"
    printf 'ANOTHER-BACKUP' > "$(dirname "$out")/$base"
fi
if [ "$RB_STAND_IN_FAIL" = "1" ]; then exit 1; fi
printf -- '-complete' >> "$out"
"""


class BackupScriptTests(unittest.TestCase):
    def setUp(self):
        parent = project_env.local_directory(project_env.ROOT, "target/backup-script-tests")
        self.work = Path(tempfile.mkdtemp(dir=parent))
        self.bin = self.work / "bin"
        self.bin.mkdir()
        shim = self.bin / "pg_dump"
        shim.write_text(STAND_IN)
        shim.chmod(0o755)
        self.dest = self.work / "backups"
        self.log = self.work / "stand-in"

    def tearDown(self):
        shutil.rmtree(self.work)

    def run_backup(self, fail: bool, race: bool = False) -> subprocess.CompletedProcess:
        env = {
            key: value for key, value in os.environ.items() if not key.startswith("PG")
        }
        env.update({
            "PATH": f"{self.bin}:{os.environ['PATH']}",
            "DATABASE_URL": URL,
            "BACKUP_DIR": str(self.dest),
            "RB_STAND_IN_LOG": str(self.log),
            "RB_STAND_IN_FAIL": "1" if fail else "0",
            "RB_STAND_IN_RACE": "1" if race else "0",
            "PGPASSWORD": "ambient-and-wrong",
            "PGOPTIONS": "-c statement_timeout=1",
        })
        # A PERMISSIVE umask, as an operator's shell usually has: the script must
        # make its own files owner-only rather than inherit a careful caller's.
        return subprocess.run(
            ["bash", str(ROOT / "scripts" / "backup.sh")],
            env=env, capture_output=True, text=True, timeout=120,
            preexec_fn=lambda: os.umask(0o022),
        )

    def listing(self) -> list[str]:
        return sorted(p.name for p in self.dest.iterdir()) if self.dest.exists() else []

    def test_an_interrupted_dump_leaves_nothing_that_looks_like_a_backup(self):
        done = self.run_backup(fail=True)
        self.assertNotEqual(done.returncode, 0, done.stdout + done.stderr)
        self.assertEqual(self.listing(), [], "no dump, no partial and no receipt remain")

    def test_a_dump_and_its_receipt_are_owner_only_and_whole(self):
        done = self.run_backup(fail=False)
        self.assertEqual(done.returncode, 0, done.stdout + done.stderr)
        names = self.listing()
        dumps = [n for n in names if n.endswith(".dump")]
        self.assertEqual(len(dumps), 1, names)
        self.assertEqual(set(names), {dumps[0], dumps[0] + ".backup.json"}, "nothing else is left behind")
        dump = self.dest / dumps[0]
        self.assertEqual(dump.read_bytes(), b"PGDMP-partial-complete")
        for path in (dump, self.dest / (dumps[0] + ".backup.json")):
            self.assertEqual(stat.S_IMODE(path.stat().st_mode), 0o600, path.name)
        self.assertEqual(stat.S_IMODE(self.dest.stat().st_mode), 0o700, "a directory it creates is owner-only")
        receipt = json.loads((self.dest / (dumps[0] + ".backup.json")).read_text())
        self.assertEqual(receipt["database"], "reasonbraid")

    def test_the_password_rides_libpq_s_environment_never_a_command_line(self):
        done = self.run_backup(fail=False)
        self.assertEqual(done.returncode, 0, done.stdout + done.stderr)
        argv = Path(str(self.log) + ".argv").read_text()
        environment = Path(str(self.log) + ".env").read_text().splitlines()
        self.assertNotIn("s3cret-pw", argv)
        self.assertNotIn("operator", argv)
        self.assertIn("PGPASSWORD=s3cret-pw", environment, "the target's password, not the ambient one")
        self.assertIn("PGSSLMODE=disable", environment)
        self.assertFalse(any(line.startswith("PGOPTIONS=") for line in environment),
                         f"an ambient libpq variable the URL does not set must not ride along: {environment}")
        self.assertNotIn("s3cret-pw", done.stdout + done.stderr)

    def test_a_dump_never_replaces_one_that_landed_on_its_name(self):
        done = self.run_backup(fail=False, race=True)
        self.assertNotEqual(done.returncode, 0, done.stdout + done.stderr)
        names = self.listing()
        self.assertEqual(len(names), 1, f"only the other backup remains: {names}")
        self.assertEqual((self.dest / names[0]).read_bytes(), b"ANOTHER-BACKUP")


if __name__ == "__main__":
    unittest.main()
