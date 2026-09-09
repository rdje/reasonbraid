"""Pinned browser admission, archive containment and command lifetime controls."""

from dataclasses import replace
import hashlib
import io
import json
import os
from pathlib import Path
import platform
import shutil
import stat
import subprocess
import sys
import tempfile
import unittest
import zipfile

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import ci_browser as browser
import project_env
import run_pg_tests as owner


class BrowserSetupTests(unittest.TestCase):
    def setUp(self):
        parent = project_env.local_directory(project_env.ROOT, "target/ci-browser-tests")
        self.base = Path(tempfile.mkdtemp(dir=parent))
        self.root = self.base / "relocatable repo"
        self.root.mkdir()
        self.release = browser.release_for(platform.system(), platform.machine())
        self.env = dict(os.environ)
        self.success = False

    def tearDown(self):
        if self.success:
            shutil.rmtree(self.base)
        else:
            print(f"browser control retained: {self.base.relative_to(project_env.ROOT)}", file=sys.stderr)

    def archive(self, entries):
        output = io.BytesIO()
        with zipfile.ZipFile(output, "w", compression=zipfile.ZIP_DEFLATED) as package:
            for name, payload, mode in entries:
                entry = zipfile.ZipInfo(name)
                entry.create_system = 3
                entry.external_attr = mode << 16
                package.writestr(entry, payload)
        data = output.getvalue()
        path = self.base / "fixture.zip"
        path.write_bytes(data)
        self.release = replace(self.release, size=len(data), sha256=hashlib.sha256(data).hexdigest())
        return path

    def worker(self, version=None, *, delay=False):
        if version is None:
            version = "Google Chrome for Testing " + browser.VERSION
        return ("#!" + sys.executable + "\nimport sys,time\n"
                "assert sys.argv[1:]==['--version']\n"
                + ("time.sleep(60)\n" if delay else "") +
                f"print({version!r})\n").encode()

    def child_fixture(self, *, version=None, corrupt=False, curl_exit=0, delay=False):
        source_archive = self.archive([(self.release.member, self.worker(version, delay=delay), stat.S_IFREG | 0o755)])
        if corrupt:
            raw = bytearray(source_archive.read_bytes())
            raw[0] ^= 1
            source_archive.write_bytes(raw)
        scripts = self.root / "scripts"
        scripts.mkdir()
        for name in ("ci_browser.py", "ci_env.py", "project_env.py", "run_pg_tests.py"):
            shutil.copyfile(project_env.ROOT / "scripts" / name, scripts / name)
        script = scripts / "ci_browser.py"
        # Only the private child-copy pin changes. Production has no runtime
        # checksum or URL override and uses the same verification/execution path.
        patch = f"PINNED_ARCHIVES[{self.release.platform!r}]=({self.release.size},{self.release.sha256!r})\n"
        source = script.read_text().replace('if __name__ == "__main__":', patch + '\nif __name__ == "__main__":')
        script.write_text(source)
        tools = self.base / "tools"
        tools.mkdir()
        curl = tools / "curl"
        curl.write_text(
            "#!" + sys.executable + "\nimport json,os,shutil,sys\nfrom pathlib import Path\n"
            "Path(os.environ['CURL_EVENT']).write_text(json.dumps({'pid':os.getpid(),'argv':sys.argv[1:]}))\n"
            "shutil.copyfile(os.environ['SOURCE_ARCHIVE'],sys.argv[sys.argv.index('--output')+1])\n"
            f"raise SystemExit({curl_exit})\n"
        )
        curl.chmod(0o700)
        self.env.update(PATH=str(tools) + os.pathsep + self.env["PATH"],
                        CURL_EVENT=str(self.base / "curl.json"), SOURCE_ARCHIVE=str(source_archive),
                        BROWSER_EVENT=str(self.base / "command.json"), R3_BROWSER_BIN="/unowned-browser",
                        SSLKEYLOGFILE=str(self.base / "unowned.log"), CHROME_LOG_FILE="/unowned-chrome-log")
        return script

    def command(self, *, exit_code=0, sleep=False):
        return [sys.executable, "-B", "-c",
                "import json,os,sys,time;from pathlib import Path;"
                "Path(os.environ['BROWSER_EVENT']).write_text(json.dumps({"
                "'pid':os.getpid(),'cwd':str(Path.cwd()),'browser':os.environ['R3_BROWSER_BIN'],"
                "'tmp':os.environ['TMPDIR'],'tls':os.environ.get('SSLKEYLOGFILE'),"
                "'chrome_log':os.environ.get('CHROME_LOG_FILE'),'args':sys.argv[1:]}));"
                + ("time.sleep(60);" if sleep else "") + f"sys.exit({exit_code})", "argument with spaces"]

    def run_child(self, script, *args):
        # A hard kill of the launcher strands its independently owned downloader.
        # Use its signal handler to consume child shutdown before the outer wait
        # finishes, including when a fixture never reaches its first instruction.
        completed = owner.run_command([sys.executable, "-B", str(script), *args],
                                      self.env, capture=True, timeout=15)
        result = subprocess.CompletedProcess(completed.args, completed.returncode,
                                             completed.stdout, completed.stdout)
        workspaces = list((self.root / "target/ci-browser").iterdir())
        self.assertEqual(len(workspaces), 1)
        workspace = workspaces[0]
        receipt = json.loads((workspace / "browser.json").read_text())
        for child in receipt["children"]:
            self.assertFalse(owner.group_exists(child["pid"]))
        return result, workspace, receipt

    def test_setup_version_only_consumes_tools_and_retires_payload(self):
        result, workspace, receipt = self.run_child(self.child_fixture(), "--verify-only")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(receipt["scope"], "version-only")
        self.assertTrue(receipt["version_verified"])
        self.assertTrue(receipt["payload_retired"])
        self.assertFalse((workspace / "runtime").exists())
        self.assertFalse((workspace / "archive.zip").exists())
        self.assertFalse((self.base / "command.json").exists())
        self.assertEqual([c["phase"] for c in receipt["children"]], ["download", "version"])
        args = json.loads((self.base / "curl.json").read_text())["argv"]
        self.assertEqual(args[0], "--disable")
        for flag in ("--proto", "--proto-redir"):
            self.assertEqual(args[args.index(flag) + 1], "=https")
        self.assertEqual(args[args.index("--max-filesize") + 1], str(self.release.size))
        self.success = True

    def test_stalled_version_is_bounded_and_prevents_command(self):
        script = self.child_fixture(delay=True)
        script.write_text(script.read_text().replace("VERSION_SECONDS = 30", "VERSION_SECONDS = 1"))
        result, workspace, receipt = self.run_child(script, "--", *self.command())
        self.assertEqual(result.returncode, 2, result.stderr)
        self.assertEqual(receipt["error_type"], "TimeoutExpired")
        self.assertEqual(receipt["children"][-1]["phase"], "version")
        self.assertFalse((self.base / "command.json").exists())
        self.assertTrue((workspace / "runtime").is_dir())
        self.success = True

    def test_outer_deadline_allows_launcher_to_consume_its_downloader(self):
        script = self.child_fixture()
        curl = self.base / "tools/curl"
        curl.write_text(curl.read_text().replace("shutil.copyfile(", "import time; time.sleep(60)\nshutil.copyfile("))
        with self.assertRaises(subprocess.TimeoutExpired):
            owner.run_command([sys.executable, "-B", str(script), "--verify-only"],
                              self.env, capture=True, timeout=1)
        receipts = list((self.root / "target/ci-browser").glob("*/browser.json"))
        self.assertEqual(len(receipts), 1)
        receipt = json.loads(receipts[0].read_text())
        self.assertEqual(receipt["state"], "interrupted")
        self.assertEqual(receipt["children"][-1]["phase"], "download")
        self.assertFalse(owner.group_exists(receipt["children"][-1]["pid"]))
        self.assertFalse((receipts[0].parent / "runtime").exists())
        self.success = True

    def test_command_uses_verified_runtime_local_stores_and_exact_arguments(self):
        result, workspace, receipt = self.run_child(self.child_fixture(), "--", *self.command())
        self.assertEqual(result.returncode, 0, result.stderr)
        event = json.loads((self.base / "command.json").read_text())
        self.assertEqual(event["cwd"], str(self.root))
        self.assertEqual(event["browser"], str(self.root / receipt["binary"]))
        self.assertTrue(Path(event["tmp"]).is_relative_to(self.root))
        self.assertEqual(event["args"], ["argument with spaces"])
        self.assertIsNone(event["tls"])
        self.assertIsNone(event["chrome_log"])
        self.assertFalse(Path(receipt["binary"]).is_absolute())
        self.assertEqual(receipt["binary_sha256"], hashlib.sha256(self.worker()).hexdigest())
        self.assertFalse((workspace / "runtime").exists())
        self.success = True

    def test_corrupt_archive_refuses_before_extraction_version_or_command(self):
        result, workspace, receipt = self.run_child(self.child_fixture(corrupt=True), "--", *self.command())
        self.assertEqual(result.returncode, 2, result.stderr)
        self.assertIn("checksum", result.stderr)
        self.assertEqual(receipt["state"], "failed")
        self.assertFalse((workspace / "runtime").exists())
        self.assertFalse((workspace / "version.log").exists())
        self.assertFalse((self.base / "command.json").exists())
        self.assertTrue((workspace / "archive.zip").exists())
        self.success = True

    def test_failed_download_never_executes_payload(self):
        result, workspace, receipt = self.run_child(self.child_fixture(curl_exit=22), "--", *self.command())
        self.assertEqual(result.returncode, 2, result.stderr)
        self.assertEqual(receipt["children"][0]["exit_code"], 22)
        self.assertFalse((workspace / "runtime").exists())
        self.assertFalse((self.base / "command.json").exists())
        self.success = True

    def test_wrong_version_prevents_command_and_retains_installation(self):
        result, workspace, receipt = self.run_child(self.child_fixture(version="wrong version"), "--", *self.command())
        self.assertEqual(result.returncode, 2, result.stderr)
        self.assertIn("unexpected version", result.stderr)
        self.assertFalse((self.base / "command.json").exists())
        self.assertTrue((workspace / "runtime").is_dir())
        self.assertNotIn("version_verified", receipt)
        self.success = True

    def test_failed_command_preserves_exact_failure_and_browser_evidence(self):
        result, workspace, receipt = self.run_child(self.child_fixture(), "--", *self.command(exit_code=7))
        self.assertEqual(result.returncode, 7, result.stderr)
        self.assertEqual(receipt["exit_code"], 7)
        self.assertTrue((workspace / "runtime").is_dir())
        self.assertTrue((workspace / "archive.zip").is_file())
        self.assertNotIn("payload_retired", receipt)
        self.success = True

    def test_command_deadline_consumes_group_and_retains_uncertain_phase(self):
        result, workspace, receipt = self.run_child(self.child_fixture(), "--timeout", "1", "--", *self.command(sleep=True))
        self.assertEqual(result.returncode, 2, result.stderr)
        self.assertEqual(receipt["error_type"], "TimeoutExpired")
        event = json.loads((self.base / "command.json").read_text())
        self.assertFalse(owner.group_exists(event["pid"]))
        self.assertTrue((workspace / "runtime").is_dir())
        self.success = True

    def test_linked_store_refuses_before_download(self):
        script = self.child_fixture()
        outside = self.base / "outside"
        outside.mkdir()
        (self.root / "target").symlink_to(outside, target_is_directory=True)
        result = owner.run_command([sys.executable, "-B", str(script), "--verify-only"],
                                   self.env, capture=True, timeout=10)
        self.assertEqual(result.returncode, 2, result.stdout)
        self.assertIn("symlink", result.stdout)
        self.assertEqual(list(outside.iterdir()), [])
        self.assertFalse((self.base / "curl.json").exists())
        self.success = True

    def test_archive_size_and_nonregular_inputs_refuse_before_extraction(self):
        archive = self.archive([(self.release.member, self.worker(), stat.S_IFREG | 0o755)])
        destination = self.base / "runtime"
        archive.write_bytes(archive.read_bytes() + b"extra")
        with self.assertRaisesRegex(ValueError, "pinned regular file"):
            browser.extract(self.release, archive, destination)
        linked = self.base / "linked.zip"
        linked.symlink_to(archive)
        with self.assertRaisesRegex(ValueError, "pinned regular file"):
            browser.extract(self.release, linked, destination)
        self.assertFalse(destination.exists())
        self.success = True

    def test_unsafe_paths_types_and_links_refuse_before_any_extraction(self):
        root = self.release.directory
        valid = (self.release.member, self.worker(), stat.S_IFREG | 0o755)
        cases = [
            [("../escape", b"bad", stat.S_IFREG | 0o600)],
            [(root + "/../escape", b"bad", stat.S_IFREG | 0o600)],
            [(root + "//bad", b"bad", stat.S_IFREG | 0o600)],
            [(root + "/pipe", b"", stat.S_IFIFO | 0o600)],
            [(root + "/link", b"../../escape", stat.S_IFLNK | 0o777)],
            [(root + "/link", b"missing", stat.S_IFLNK | 0o777)],
            [(root + "/a", b"b", stat.S_IFLNK | 0o777), (root + "/b", b"a", stat.S_IFLNK | 0o777)],
            [(root + "/file", b"data", stat.S_IFREG | 0o600), (root + "/file/child", b"bad", stat.S_IFREG | 0o600)],
            [(root + "/a", b"dir", stat.S_IFLNK | 0o777), (root + "/a/child", b"bad", stat.S_IFREG | 0o600)],
        ]
        for index, extra in enumerate(cases):
            with self.subTest(case=index):
                archive = self.archive([valid, *extra])
                destination = self.base / "runtime"
                with self.assertRaises(ValueError):
                    browser.extract(self.release, archive, destination)
                self.assertFalse(destination.exists())
        self.success = True

    def test_missing_executable_and_expansion_limit_refuse(self):
        archive = self.archive([(self.release.directory + "/data", b"data", stat.S_IFREG | 0o600)])
        with self.assertRaisesRegex(ValueError, "executable is missing"):
            browser.extract(self.release, archive, self.base / "runtime")
        archive = self.archive([(self.release.member, self.worker(), stat.S_IFREG | 0o755)])
        from unittest.mock import patch
        with patch.object(browser, "MAX_EXPANDED_BYTES", 1), self.assertRaisesRegex(ValueError, "expansion limit"):
            browser.extract(self.release, archive, self.base / "runtime")
        self.assertFalse((self.base / "runtime").exists())
        self.success = True

    def test_internal_framework_link_chain_is_preserved_with_private_payload(self):
        root = self.release.directory
        archive = self.archive([
            (self.release.member, self.worker(), stat.S_IFREG | 0o755),
            (root + "/Framework/Versions/A/Resources/data", b"original", stat.S_IFREG | 0o644),
            (root + "/Framework/Versions/Current", b"A", stat.S_IFLNK | 0o777),
            (root + "/Framework/Resources", b"Versions/Current/Resources", stat.S_IFLNK | 0o777),
        ])
        destination = self.base / "runtime"
        layout = browser.extract(self.release, archive, destination)
        self.assertEqual(layout["links"], 2)
        resolved = destination / root / "Framework/Resources/data"
        self.assertEqual(resolved.read_bytes(), b"original")
        self.assertTrue(resolved.resolve().is_relative_to(destination))
        self.assertEqual(stat.S_IMODE(destination.stat().st_mode), 0o700)
        self.assertEqual(stat.S_IMODE((destination / self.release.member).stat().st_mode), 0o700)
        self.assertEqual(stat.S_IMODE(resolved.stat().st_mode), 0o600)
        self.success = True

    def test_platforms_are_exact_pins_and_unsupported_hosts_refuse(self):
        self.assertEqual(len(browser.PINNED_ARCHIVES), 4)
        for (system, machine), expected in browser.PLATFORMS.items():
            selected = browser.release_for(system, machine)
            self.assertEqual(selected.platform, expected)
            self.assertEqual(len(selected.sha256), 64)
            self.assertIn("/" + browser.VERSION + "/", selected.url)
        for host in (("Windows", "AMD64"), ("Linux", "riscv64")):
            with self.assertRaisesRegex(ValueError, "unsupported"):
                browser.release_for(*host)
        self.success = True


if __name__ == "__main__":
    unittest.main()
