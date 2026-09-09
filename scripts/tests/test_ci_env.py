"""Real child/environment controls; instrumented rustup never downloads a compiler."""

import json
import os
from pathlib import Path
import shutil
import signal
import subprocess
import sys
import tempfile
import time
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import ci_env
import project_env
import run_pg_tests as process_owner


class CiEnvironmentTests(unittest.TestCase):
    def setUp(self):
        parent = project_env.local_directory(project_env.ROOT, "target/ci-workflow-controls")
        self.base = Path(tempfile.mkdtemp(prefix="environment-", dir=parent))
        self.root = self.base / "checkout with 'quotes'"
        self.root.mkdir()
        (self.root / "rust-toolchain.toml").write_text('[toolchain]\nchannel = "1.98.0"\n')
        scripts = self.root / "scripts"
        scripts.mkdir()
        for name in ("ci_env.py", "project_env.py", "run_pg_tests.py"):
            shutil.copyfile(project_env.ROOT / "scripts" / name, scripts / name)
        self.tools = self.base / "installed-tools"
        self.tools.mkdir()
        self.event = self.root / "installer.json"
        self.marker = self.root / "dispatched"
        self.env = dict(os.environ)
        self.env["PATH"] = str(self.tools) + os.pathsep + os.environ["PATH"]
        self.env["INSTALL_EVENT"] = str(self.event)
        for key in project_env.STORES:
            self.env[key] = str(self.base / "ambient-store")
        for key in ci_env.CI_OVERRIDES:
            self.env[key] = "inherited-control-must-not-apply"
        self.success = False

    def tearDown(self):
        if self.success:
            shutil.rmtree(self.base)
        else:
            print(f"ci-env control retained: {self.base.relative_to(project_env.ROOT)}", file=sys.stderr)

    def installer(self, mode="success"):
        script = self.tools / "rustup"
        script.write_text(
            "#!" + sys.executable + "\n"
            "import json,os,platform,sys,time\nfrom pathlib import Path\n"
            "event=Path(os.environ['INSTALL_EVENT'])\n"
            "temporary=event.with_suffix('.pending')\n"
            "temporary.write_text(json.dumps({'pid':os.getpid(),'argv':sys.argv[1:],"
            "'cargo_home':os.environ['CARGO_HOME'],'rustup_home':os.environ['RUSTUP_HOME'],"
            "'tmpdir':os.environ['TMPDIR'],'wrapper':os.environ['RUSTC_WRAPPER']}))\n"
            "temporary.replace(event)\n"
            + ("raise SystemExit(7)\n" if mode == "fail" else "")
            + ("time.sleep(60)\n" if mode == "wait" else "")
            + "machine={'arm64':'aarch64','AMD64':'x86_64'}.get(platform.machine(),platform.machine())\n"
            "compiler=Path(os.environ['RUSTUP_HOME'])/'toolchains'/('1.98.0-'+machine+'-fixture')/'bin'\n"
            "compiler.mkdir(parents=True)\n"
            "for name in ('cargo','rustc','rustdoc','rustfmt','cargo-clippy'):\n"
            "    binary=compiler/name\n"
            "    binary.write_text('#!/bin/sh\\necho instrumented-compiler\\n')\n"
            "    binary.chmod(0o700)\n"
        )
        script.chmod(0o700)

    def command(self, *args):
        return [sys.executable, "-B", str(self.root / "scripts/ci_env.py"), *args]

    def marker_command(self):
        return [sys.executable, "-B", "-c", "from pathlib import Path; Path('dispatched').write_text('ran')"]

    def assert_local(self, env):
        for key in project_env.STORES:
            self.assertTrue(Path(env[key]).is_relative_to(self.root), key)
            self.assertEqual(Path(env[key]).stat().st_dev, self.root.stat().st_dev)
        for key in ci_env.CI_OVERRIDES:
            if key != "RB_READONLY_TOOLCHAIN":
                self.assertNotIn(key, env)
        for key in ci_env.WRAPPERS:
            self.assertEqual(env[key], "")

    def test_exec_uses_relocated_root_and_local_temporary_file(self):
        moved = self.base / "moved checkout"
        self.root.rename(moved)
        self.root = moved
        probe = (
            "import json,os,tempfile; from pathlib import Path; "
            "fd,name=tempfile.mkstemp(); os.close(fd); p=Path(name); "
            "print(json.dumps({'cwd':os.getcwd(),'temporary':str(p),'env':"
            "{k:v for k,v in os.environ.items() if k in "
            + repr(list(project_env.STORES) + list(ci_env.CI_OVERRIDES) + list(ci_env.WRAPPERS))
            + "}})); p.unlink()"
        )
        result = subprocess.run(self.command("--", sys.executable, "-B", "-c", probe),
                                cwd=self.base, env=self.env, capture_output=True, text=True, timeout=10)
        self.assertEqual(result.returncode, 0, result.stderr)
        observed = json.loads(result.stdout)
        self.assertEqual(observed["cwd"], str(moved))
        self.assertTrue(Path(observed["temporary"]).is_relative_to(moved / ".project-data/tmp"))
        self.assert_local(observed["env"])
        self.assertFalse((self.base / "ambient-store").exists())
        self.success = True

    def test_installer_receives_local_stores_and_complete_pin_then_reuses_it(self):
        self.installer()
        env = ci_env.ci_environment(self.root, self.env, rust=True)
        self.assert_local(env)
        event = json.loads(self.event.read_text())
        self.assertEqual(event["argv"], ["toolchain", "install", "1.98.0", "--profile", "minimal",
                                        "--component", "rustfmt", "--component", "clippy", "--no-self-update"])
        for key in ("cargo_home", "rustup_home", "tmpdir"):
            self.assertTrue(Path(event[key]).is_relative_to(self.root))
        self.assertEqual(event["wrapper"], "")
        self.assertFalse(process_owner.group_exists(event["pid"]))
        compiler = Path(env["RB_READONLY_TOOLCHAIN"])
        self.assertTrue(compiler.is_relative_to(self.root / ".project-data/installed-toolchains"))
        (self.tools / "rustup").unlink()
        self.event.unlink()
        reused = ci_env.ci_environment(self.root, self.env, rust=True)
        self.assertEqual(reused["RB_READONLY_TOOLCHAIN"], str(compiler))
        self.assertFalse(self.event.exists())
        result = subprocess.run(["cargo", "--version"], cwd=self.root, env=reused,
                                capture_output=True, text=True, check=True, timeout=5)
        self.assertEqual(result.stdout.strip(), "instrumented-compiler")
        self.success = True

    def test_failed_installer_prevents_command_dispatch(self):
        self.installer("fail")
        result = subprocess.run(self.command("--rust", "--", *self.marker_command()),
                                cwd=self.base, env=self.env, capture_output=True, text=True, timeout=10)
        self.assertEqual(result.returncode, 2, result.stderr)
        self.assertIn("exit status 7", result.stderr)
        self.assertFalse(self.marker.exists())
        self.assertFalse(process_owner.group_exists(json.loads(self.event.read_text())["pid"]))
        self.success = True

    def test_timed_out_installer_is_reaped(self):
        self.installer("wait")
        with self.assertRaises(subprocess.TimeoutExpired):
            ci_env.ci_environment(self.root, self.env, rust=True, install_timeout=2)
        self.assertFalse(process_owner.group_exists(json.loads(self.event.read_text())["pid"]))
        self.assertFalse(self.marker.exists())
        self.success = True

    def test_terminal_signal_reaps_installer_before_launcher_returns(self):
        self.installer("wait")
        driver = subprocess.Popen(self.command("--rust", "--", *self.marker_command()),
                                  cwd=self.base, env=self.env, stdout=subprocess.PIPE,
                                  stderr=subprocess.STDOUT, text=True, start_new_session=True)
        try:
            deadline = time.monotonic() + 10
            while not self.event.exists() and driver.poll() is None and time.monotonic() < deadline:
                time.sleep(0.02)
            self.assertTrue(self.event.exists(), "installer did not start")
            driver.send_signal(signal.SIGTERM)
            output, _ = driver.communicate(timeout=15)
            self.assertEqual(driver.returncode, 130, output)
            self.assertFalse(process_owner.group_exists(json.loads(self.event.read_text())["pid"]))
            self.assertFalse(self.marker.exists())
        finally:
            process_owner.stop_group(driver)
            driver.stdout.close()
        self.success = True

    def test_linked_installation_store_refuses_before_installer(self):
        self.installer()
        data = self.root / ".project-data"
        data.mkdir()
        outside = self.base / "other-owned-fixture"
        outside.mkdir()
        (data / "installed-toolchains").symlink_to(outside, target_is_directory=True)
        with self.assertRaisesRegex(ValueError, "symlink"):
            ci_env.ci_environment(self.root, self.env, rust=True)
        self.assertFalse(self.event.exists())
        self.assertEqual(list(outside.iterdir()), [])
        self.success = True

    def test_linked_compiler_component_refuses_without_reinstallation(self):
        self.installer()
        env = ci_env.ci_environment(self.root, self.env, rust=True)
        cargo = Path(env["RB_READONLY_TOOLCHAIN"]) / "bin/cargo"
        saved = cargo.read_bytes()
        outside = self.base / "other-tool"
        outside.write_bytes(saved)
        outside.chmod(0o700)
        cargo.unlink()
        cargo.symlink_to(outside)
        self.event.unlink()
        with self.assertRaisesRegex(ValueError, "local executable"):
            ci_env.ci_environment(self.root, self.env, rust=True)
        self.assertFalse(self.event.exists())
        self.assertEqual(outside.read_bytes(), saved)
        self.success = True

    def test_floating_toolchain_pin_refuses_before_installer(self):
        self.installer()
        (self.root / "rust-toolchain.toml").write_text('[toolchain]\nchannel = "stable"\n')
        with self.assertRaisesRegex(ValueError, "exact numeric"):
            ci_env.ci_environment(self.root, self.env, rust=True)
        self.assertFalse(self.event.exists())
        self.success = True


if __name__ == "__main__":
    os.umask(0o077)
    unittest.main()
