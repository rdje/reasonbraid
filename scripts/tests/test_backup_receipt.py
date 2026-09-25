"""The backup/restore receipts (SIGNOFF-REPAIR.4.6.1.5.2): what each one records,
what a restore test refuses, and that no credential is ever written or printed."""

import json
from pathlib import Path
import shutil
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import backup_receipt as receipt
import project_env

URL = "postgres://operator:s3cret@db.internal:5432/reasonbraid?sslmode=disable"


class ReceiptTests(unittest.TestCase):
    def setUp(self):
        parent = project_env.local_directory(project_env.ROOT, "target/backup-receipt-tests")
        self.dir = Path(tempfile.mkdtemp(dir=parent))
        self.dump = self.dir / "reasonbraid-20260922-000000.dump"
        self.dump.write_bytes(b"PGDMP" + bytes(range(256)) * 4)

    def tearDown(self):
        shutil.rmtree(self.dir)

    def test_a_backup_receipt_records_the_dump_and_no_credential(self):
        body = receipt.write_backup(self.dump, URL)
        written = (self.dir / (self.dump.name + ".backup.json")).read_text()
        self.assertEqual(json.loads(written), body)
        self.assertEqual(body["bytes"], self.dump.stat().st_size)
        self.assertEqual(body["database"], "reasonbraid")
        self.assertEqual(len(body["sha256"]), 64)
        self.assertNotIn("s3cret", written)
        self.assertNotIn("operator", written)
        self.assertNotIn("db.internal", written)

    def test_redact_keeps_host_port_and_database_and_drops_the_rest(self):
        self.assertEqual(receipt.redact(URL), "postgres://db.internal:5432/reasonbraid")

    def test_check_refuses_a_truncated_dump_and_an_altered_one(self):
        receipt.write_backup(self.dump, URL)
        self.assertEqual(receipt.check(self.dump)["dump"], self.dump.name)

        data = self.dump.read_bytes()
        self.dump.write_bytes(data[:-1])
        with self.assertRaisesRegex(receipt.ReceiptError, "bytes; its receipt recorded"):
            receipt.check(self.dump)

        self.dump.write_bytes(data[:-1] + bytes([data[-1] ^ 1]))  # same size, one bit
        with self.assertRaisesRegex(receipt.ReceiptError, "does not match its receipt"):
            receipt.check(self.dump)

    def test_a_dump_without_a_receipt_cannot_be_restore_tested(self):
        with self.assertRaisesRegex(receipt.ReceiptError, "has no backup receipt"):
            receipt.check(self.dump)

    def test_a_restore_receipt_needs_a_matching_dump_and_a_restored_schema(self):
        backup = receipt.write_backup(self.dump, URL)
        with self.assertRaisesRegex(receipt.ReceiptError, "restored no schema"):
            receipt.write_restore(self.dump, URL, 0)
        self.assertFalse((self.dir / (self.dump.name + ".restore.json")).exists())

        body = receipt.write_restore(self.dump, "postgres://127.0.0.1/restore_target", 88)
        self.assertEqual(body["sha256"], backup["sha256"])
        self.assertEqual(body["target_database"], "restore_target")
        self.assertEqual(body["migrations"], 88)

    def test_main_reports_a_refusal_as_exit_one_and_bad_usage_as_two(self):
        self.assertEqual(receipt.main(["x", "check", str(self.dump)]), 1)
        self.assertEqual(receipt.main(["x", "nonsense"]), 2)


class ConnectionTests(unittest.TestCase):
    """SIGNOFF-REPAIR.11.3.2: a URL reaches the clients through libpq's environment,
    never a command line, and a restore target that names the live database is
    refused."""

    def test_a_url_becomes_libpq_variables_decoded(self):
        env = receipt.libpq_environment(
            "postgres://op%40x:s3c%2Fret@db.internal:5433/reason%20braid?sslmode=require"
        )
        self.assertEqual(env, {
            "PGHOST": "db.internal", "PGPORT": "5433", "PGUSER": "op@x",
            "PGPASSWORD": "s3c/ret", "PGDATABASE": "reason braid", "PGSSLMODE": "require",
        })

    def test_what_libpq_cannot_carry_is_refused_not_dropped(self):
        for url, why in [
            ("postgres://a@h1,h2/db", "several hosts"),
            ("postgres://a@h/db?sslmode=disable&bogus=1", "`bogus`"),
            ("mysql://a@h/db", "not a postgres"),
            ("postgres://a:pa%0Ass@h/db", "newline"),
            ("postgres://a@h:port/db", "not a number"),
        ]:
            with self.assertRaises(receipt.ReceiptError, msg=url) as caught:
                receipt.libpq_environment(url)
            self.assertIn(why, str(caught.exception))
            self.assertNotIn("pa\nss", str(caught.exception))

    def test_every_carried_parameter_has_a_variable_the_scripts_clear(self):
        names = set(receipt.LIBPQ_PARAMETERS.values())
        env = receipt.libpq_environment(
            "postgres://u:p@h:1/d?" + "&".join(f"{k}=v" for k in receipt.LIBPQ_PARAMETERS
                                                 if k not in ("host", "port", "user", "password", "dbname")))
        self.assertLessEqual(set(env), names)

    def test_the_live_database_is_refused_as_a_restore_target(self):
        live = "postgres://postgres@127.0.0.1:5432/reasonbraid"
        for target in [live, "postgres://other@127.0.0.1/reasonbraid", "postgresql://127.0.0.1:5432/reasonbraid?sslmode=disable"]:
            with self.assertRaises(receipt.ReceiptError, msg=target) as caught:
                receipt.guard_restore_target({"RESTORE_DATABASE_URL": target, "DATABASE_URL": live})
            self.assertIn("names the live database", str(caught.exception))
            self.assertNotIn("postgres@", str(caught.exception))
        for target in ["postgres://postgres@127.0.0.1:5432/restore_test",
                       "postgres://postgres@127.0.0.1:5433/reasonbraid"]:
            receipt.guard_restore_target({"RESTORE_DATABASE_URL": target, "DATABASE_URL": live})
        receipt.guard_restore_target({"RESTORE_DATABASE_URL": live})
        with self.assertRaises(receipt.ReceiptError):
            receipt.guard_restore_target({})

    def test_no_script_hands_a_url_to_a_command_line(self):
        """The census of `.11.3`: a URL on a command line is readable by any local
        user for as long as the process runs. Grows as each script is repaired."""
        import re
        root = Path(__file__).resolve().parents[2]
        url_variable = re.compile(r"\$\{?(RESTORE_DATABASE_URL|DATABASE_URL)\b")
        for script in ["scripts/restore.sh"]:
            for number, line in enumerate((root / script).read_text().splitlines(), 1):
                code = line.split("#", 1)[0] if not line.lstrip().startswith(":") else ""
                self.assertIsNone(url_variable.search(code), f"{script}:{number}: {line.strip()}")

    def test_the_url_subcommands_read_a_variable_and_print_no_credential(self):
        import contextlib
        import io
        import os
        from unittest import mock
        with mock.patch.dict(os.environ, {"RB_TEST_URL": URL}):
            out = io.StringIO()
            with contextlib.redirect_stdout(out):
                self.assertEqual(receipt.main(["x", "redact-env", "RB_TEST_URL"]), 0)
            self.assertEqual(out.getvalue().strip(), "postgres://db.internal:5432/reasonbraid")
            out = io.StringIO()
            with contextlib.redirect_stdout(out):
                self.assertEqual(receipt.main(["x", "pg-env", "RB_TEST_URL"]), 0)
            self.assertIn("PGPASSWORD=s3cret", out.getvalue().splitlines())
        with mock.patch.dict(os.environ, {}, clear=True):
            self.assertEqual(receipt.main(["x", "pg-env", "RB_TEST_URL"]), 1)


if __name__ == "__main__":
    unittest.main()
