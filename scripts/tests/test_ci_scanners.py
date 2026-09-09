"""Scanner integrity/lifecycle controls; isolated child copies use a synthetic release."""

from dataclasses import replace
import gzip
import hashlib
import io
import json
import os
from pathlib import Path
import platform
import shutil
import signal
import subprocess
import sys
import tarfile
import tempfile
import time
import unittest
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import ci_scanners as scanners
import project_env
import run_pg_tests as process_owner


class ScannerTests(unittest.TestCase):
    def setUp(self):
        parent = project_env.local_directory(project_env.ROOT, "target/ci-workflow-controls/scanners")
        self.base = Path(tempfile.mkdtemp(prefix="control-", dir=parent))
        self.root = self.base / "checkout with 'quotes'"
        self.root.mkdir()
        self.release = scanners.release_for("gitleaks", platform.system(), platform.machine())
        self.env = dict(os.environ)
        self.success = False

    def tearDown(self):
        if self.success:
            shutil.rmtree(self.base)
        else:
            print(f"scanner control retained: {self.base.relative_to(project_env.ROOT)}", file=sys.stderr)

    def archive(self, entries):
        output = io.BytesIO()
        with tarfile.open(fileobj=output, mode="w:") as archive:
            for name, payload, kind in entries:
                member = tarfile.TarInfo(name)
                member.type = kind
                if kind == tarfile.REGTYPE:
                    member.size = len(payload)
                    archive.addfile(member, io.BytesIO(payload))
                else:
                    member.linkname = "unrelated"
                    archive.addfile(member)
        data = gzip.compress(output.getvalue(), mtime=0)
        path = self.base / "fixture.tar.gz"
        path.write_bytes(data)
        self.release = replace(self.release, size=len(data), sha256=hashlib.sha256(data).hexdigest())
        return path

    def worker(self, version="gitleaks version 8.30.1"):
        return (
            "#!" + sys.executable + "\n"
            "import json,os,sys\nfrom pathlib import Path\n"
            "if sys.argv[1:]==['--version']:\n"
            f"    print({version!r})\n    raise SystemExit(0)\n"
            "root=Path.cwd()\n"
            "event={'argv':sys.argv[1:],'cwd':str(root),'cargo_home':os.environ['CARGO_HOME'],"
            "'tmpdir':os.environ['TMPDIR'],'ambient_config':os.environ.get('GITLEAKS_CONFIG'),"
            "'tls_log':os.environ.get('SSLKEYLOGFILE'),'qlog':os.environ.get('QLOGDIR')}\n"
            "Path(os.environ['SCAN_EVENT']).write_text(json.dumps(event))\n"
            "report=Path(sys.argv[sys.argv.index('--report-path')+1])\n"
            "report.write_text('[{\"Secret\":\"REDACTED\"}]')\n"
            "print('instrumented scanner result')\n"
            "raise SystemExit(int(os.environ.get('SCAN_EXIT','0')))\n"
        ).encode()

    def child_fixture(self, *, version="gitleaks version 8.30.1", corrupt=False, curl_exit=0):
        archive = self.archive([("gitleaks", self.worker(version), tarfile.REGTYPE)])
        if corrupt:
            data = bytearray(archive.read_bytes())
            data[-1] ^= 1
            archive.write_bytes(data)
        scripts = self.root / "scripts"
        scripts.mkdir()
        for name in ("ci_scanners.py", "ci_env.py", "project_env.py", "run_pg_tests.py"):
            shutil.copyfile(project_env.ROOT / "scripts" / name, scripts / name)
        # Only the private child-copy trust record changes. Production pins and
        # verification code remain untouched; tests never accept a runtime pin override.
        path = scripts / "ci_scanners.py"
        source = path.read_text()
        override = f"PINNED_ARCHIVES[{self.release.archive!r}]=({self.release.size},{self.release.sha256!r})\n\n"
        source = source.replace('if __name__ == "__main__":', override + 'if __name__ == "__main__":')
        path.write_text(source)
        tools = self.base / "installed-tools"
        tools.mkdir()
        curl = tools / "curl"
        curl.write_text(
            "#!" + sys.executable + "\nimport json,os,shutil,sys\nfrom pathlib import Path\n"
            "Path(os.environ['CURL_EVENT']).write_text(json.dumps({'pid':os.getpid(),'argv':sys.argv[1:]}))\n"
            "shutil.copyfile(os.environ['SOURCE_ARCHIVE'],sys.argv[sys.argv.index('--output')+1])\n"
            f"raise SystemExit({curl_exit})\n"
        )
        curl.chmod(0o700)
        self.env.update({"PATH": str(tools) + os.pathsep + self.env["PATH"],
                         "SOURCE_ARCHIVE": str(archive), "CURL_EVENT": str(self.base / "curl.json"),
                         "SCAN_EVENT": str(self.base / "scan.json"),
                         "GITLEAKS_CONFIG": "ambient-config-must-not-apply",
                         "SSLKEYLOGFILE": str(self.base / "unowned-tls.log"),
                         "QLOGDIR": str(self.base / "unowned-qlog")})
        return path

    def run_child(self, script, *args):
        result = subprocess.run([sys.executable, "-B", str(script), "gitleaks", *args],
                                cwd=self.base, env=self.env, capture_output=True, text=True, timeout=15)
        directories = list((self.root / "target/ci-scanners").iterdir())
        self.assertEqual(len(directories), 1)
        workspace = directories[0]
        receipt = json.loads((workspace / "scanner.json").read_text())
        for event in [self.base / "curl.json"]:
            if event.exists():
                self.assertFalse(process_owner.group_exists(json.loads(event.read_text())["pid"]))
        if "last_child" in receipt:
            self.assertFalse(process_owner.group_exists(receipt["last_child"]["pid"]))
        return result, workspace, receipt

    def test_verified_installation_only_skips_gate_and_retires_executables(self):
        result, workspace, receipt = self.run_child(self.child_fixture(), "--verify-only")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(receipt["scope"], "version-only")
        self.assertTrue(receipt["version_verified"])
        self.assertEqual(receipt["last_child"]["state"], "reaped")
        self.assertFalse((self.base / "scan.json").exists())
        self.assertFalse((workspace / "download.tar.gz").exists())
        self.assertFalse((workspace / "gitleaks").exists())
        self.assertEqual((workspace / "version.log").read_text().strip(), "gitleaks version 8.30.1")
        args = json.loads((self.base / "curl.json").read_text())["argv"]
        self.assertEqual(args[0], "--disable")
        self.assertEqual(args[args.index("--proto") + 1], "=https")
        self.assertEqual(args[args.index("--proto-redir") + 1], "=https")
        self.assertEqual(args[args.index("--max-filesize") + 1], str(self.release.size))
        self.success = True

    def test_gate_refusal_is_nonzero_redacted_local_and_retains_evidence(self):
        script = self.child_fixture()
        self.env["SCAN_EXIT"] = "1"
        result, workspace, receipt = self.run_child(script)
        self.assertEqual(result.returncode, 1, result.stderr)
        self.assertEqual(receipt["state"], "completed")
        self.assertEqual(receipt["exit_code"], 1)
        event = json.loads((self.base / "scan.json").read_text())
        self.assertEqual(event["cwd"], str(self.root))
        self.assertIn("--redact=100", event["argv"])
        self.assertNotIn("--no-git", event["argv"])
        self.assertIsNone(event["ambient_config"])
        self.assertIsNone(event["tls_log"])
        self.assertIsNone(event["qlog"])
        for key in ("cargo_home", "tmpdir"):
            self.assertTrue(Path(event[key]).is_relative_to(self.root))
        report = event["argv"][event["argv"].index("--report-path") + 1]
        self.assertFalse(Path(report).is_absolute())
        self.assertEqual(json.loads((self.root / report).read_text())[0]["Secret"], "REDACTED")
        self.assertIn("instrumented scanner result", (workspace / "check.log").read_text())
        self.assertFalse((workspace / "gitleaks").exists())
        self.success = True

    def test_checksum_change_refuses_before_executable_or_gate(self):
        result, workspace, receipt = self.run_child(self.child_fixture(corrupt=True))
        self.assertEqual(result.returncode, 2, result.stderr)
        self.assertEqual(receipt["state"], "failed")
        self.assertIn("checksum", result.stderr)
        self.assertFalse((workspace / "gitleaks").exists())
        self.assertFalse((workspace / "version.log").exists())
        self.assertFalse((self.base / "scan.json").exists())
        self.success = True

    def test_archive_length_mismatch_refuses_before_execution(self):
        script = self.child_fixture()
        archive = Path(self.env["SOURCE_ARCHIVE"])
        archive.write_bytes(archive.read_bytes() + b"extra")
        result, workspace, receipt = self.run_child(script)
        self.assertEqual(result.returncode, 2, result.stderr)
        self.assertEqual(receipt["state"], "failed")
        self.assertFalse((workspace / "gitleaks").exists())
        self.assertFalse((workspace / "version.log").exists())
        self.success = True

    def test_terminal_cancellation_retains_receipt_and_reaps_version_child(self):
        slow = ("#!" + sys.executable + "\nimport time\ntime.sleep(60)\n").encode()
        with mock.patch.object(self, "worker", return_value=slow):
            script = self.child_fixture()
        driver = subprocess.Popen(
            [sys.executable, "-B", str(script), "gitleaks", "--verify-only"],
            cwd=self.base, env=self.env, stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT, text=True, start_new_session=True,
        )
        observed = None
        try:
            deadline = time.monotonic() + 10
            while driver.poll() is None and time.monotonic() < deadline:
                for receipt_path in self.root.glob("target/ci-scanners/*/scanner.json"):
                    receipt = json.loads(receipt_path.read_text())
                    if receipt.get("last_child", {}).get("phase") == "version":
                        observed = (receipt_path, receipt["last_child"]["pid"])
                        break
                if observed:
                    break
                time.sleep(0.02)
            self.assertIsNotNone(observed, "version child did not start")
            driver.send_signal(signal.SIGTERM)
            output, _ = driver.communicate(timeout=15)
            self.assertEqual(driver.returncode, 130, output)
            receipt = json.loads(observed[0].read_text())
            self.assertEqual(receipt["state"], "interrupted")
            self.assertFalse(process_owner.group_exists(observed[1]))
            self.assertFalse((self.base / "scan.json").exists())
            self.assertTrue((observed[0].parent / "version.log").exists())
        finally:
            process_owner.stop_group(driver)
            driver.stdout.close()
        self.success = True

    def test_download_nonzero_prevents_execution_even_with_correct_bytes(self):
        result, workspace, receipt = self.run_child(self.child_fixture(curl_exit=7))
        self.assertEqual(result.returncode, 2, result.stderr)
        self.assertEqual(receipt["state"], "failed")
        self.assertFalse((workspace / "version.log").exists())
        self.assertFalse((workspace / "gitleaks").exists())
        self.success = True

    def test_wrong_version_prevents_gate_and_preserves_diagnostics(self):
        result, workspace, receipt = self.run_child(self.child_fixture(version="old-version"))
        self.assertEqual(result.returncode, 2, result.stderr)
        self.assertEqual(receipt["state"], "failed")
        self.assertIn("unexpected version", result.stderr)
        self.assertFalse((self.base / "scan.json").exists())
        self.assertTrue((workspace / "download.tar.gz").exists())
        self.assertEqual((workspace / "version.log").read_text().strip(), "old-version")
        self.success = True

    def test_archive_extracts_only_the_expected_regular_bytes(self):
        path = self.archive([("LICENSE", b"retained inside archive", tarfile.REGTYPE),
                             ("gitleaks", b"exact executable bytes", tarfile.REGTYPE)])
        self.assertEqual(scanners.executable_bytes(self.release, path), b"exact executable bytes")
        self.assertFalse((self.base / "LICENSE").exists())
        self.success = True

    def test_unsafe_missing_and_duplicate_archive_members_refuse(self):
        cases = [
            [("gitleaks", b"", tarfile.SYMTYPE)],
            [("gitleaks", b"", tarfile.LNKTYPE)],
            [("../escape", b"bad", tarfile.REGTYPE), ("gitleaks", b"ok", tarfile.REGTYPE)],
            [("/absolute", b"bad", tarfile.REGTYPE), ("gitleaks", b"ok", tarfile.REGTYPE)],
            [("gitleaks", b"one", tarfile.REGTYPE), ("gitleaks", b"two", tarfile.REGTYPE)],
            [("missing", b"wrong name", tarfile.REGTYPE)],
        ]
        for entries in cases:
            with self.subTest(entries=entries):
                path = self.archive(entries)
                with self.assertRaises(ValueError):
                    scanners.executable_bytes(self.release, path)
        self.assertFalse((self.base / "escape").exists())
        self.success = True

    def test_archive_expansion_binary_and_member_count_bounds_refuse(self):
        path = self.archive([("gitleaks", b"ninebytes", tarfile.REGTYPE)])
        for setting, limit in [("MAX_TAR_BYTES", 100), ("MAX_BINARY_BYTES", 8), ("MAX_MEMBERS", 0)]:
            with self.subTest(setting=setting), mock.patch.object(scanners, setting, limit):
                with self.assertRaises(ValueError):
                    scanners.executable_bytes(self.release, path)
        self.success = True

    def test_preexisting_download_link_is_preserved_before_external_work(self):
        outside = self.base / "sentinel"
        outside.write_bytes(b"unchanged")
        destination = self.base / "download.tar.gz"
        destination.symlink_to(outside)
        with mock.patch.object(scanners, "run_command") as run:
            with self.assertRaisesRegex(ValueError, "already exists"):
                scanners.download(self.release, destination, self.env)
            run.assert_not_called()
        self.assertEqual(outside.read_bytes(), b"unchanged")
        self.success = True

    def test_version_timeout_reaps_its_exact_child(self):
        workspace = self.root / "version-timeout"
        workspace.mkdir()
        binary = workspace / "gitleaks"
        binary.write_text("#!" + sys.executable + "\nimport time\ntime.sleep(60)\n")
        binary.chmod(0o700)
        pids = []
        with mock.patch.object(scanners, "VERSION_SECONDS", 0.5):
            with self.assertRaises(subprocess.TimeoutExpired):
                scanners.version_check(self.release, binary, workspace, self.env, started=pids.append)
        self.assertEqual(len(pids), 1)
        self.assertFalse(process_owner.group_exists(pids[0]))
        self.success = True

    def test_release_selection_is_explicit_and_unknown_platform_refuses(self):
        for system, machine in scanners.PLATFORMS:
            for tool in ("cargo-deny", "gitleaks"):
                release = scanners.release_for(tool, system, machine)
                self.assertTrue(release.url.startswith("https://github.com/"))
                self.assertEqual(len(release.sha256), 64)
                self.assertLess(release.size, scanners.MAX_ARCHIVE_BYTES)
        with self.assertRaisesRegex(ValueError, "platform unsupported"):
            scanners.release_for("gitleaks", "unreviewed", "unknown")
        self.assertEqual(scanners.release_for("gitleaks", "Darwin", "arm64"),
                         scanners.release_for("gitleaks", "Darwin", "aarch64"))
        command = scanners.scanner_command("cargo-deny", Path("tool"), self.root, self.root)
        self.assertEqual(command, ["tool", "--locked", "check"])
        self.success = True


if __name__ == "__main__":
    os.umask(0o077)
    unittest.main()
