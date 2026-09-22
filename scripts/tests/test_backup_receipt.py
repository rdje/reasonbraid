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


if __name__ == "__main__":
    unittest.main()
