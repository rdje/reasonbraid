#!/usr/bin/env python3
"""Focused lifecycle controls; fixtures and subprocess state remain under target/."""

import json
import os
from pathlib import Path
import signal
import socket
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import run_pg_tests as runner


class PgRunnerTests(unittest.TestCase):
    def setUp(self):
        parent = runner.local_directory(runner.ROOT, "target/pg-runner-controls")
        self.temporary = tempfile.TemporaryDirectory(dir=parent)
        self.root = Path(self.temporary.name)

    def tearDown(self):
        self.temporary.cleanup()

    def cluster(self):
        with patch.object(runner, "available_port", return_value=55555):
            return runner.Cluster(self.root, self.root / "installed/tools", dict(os.environ))

    def test_caller_database_and_libpq_overrides_are_removed(self):
        ambient = {"DATABASE_URL": "postgres://production/important", "PGHOST": "foreign", "PGOPTIONS": "-c search_path=foreign", "PGSERVICE": "production", "PATH": "bin", "TMPDIR": "target/scratch"}
        env = runner.pg_environment(ambient, Path("tools"))
        self.assertNotIn("DATABASE_URL", env)
        self.assertNotIn("PGHOST", env)
        self.assertNotIn("PGOPTIONS", env)
        self.assertNotIn("PGSERVICE", env)
        self.assertEqual(env["TMPDIR"], "target/scratch")
        self.assertEqual(ambient["PGHOST"], "foreign")

    def test_success_removes_only_the_owned_workspace(self):
        cluster = self.cluster()
        sibling = cluster.path.parent / "unrelated"
        sibling.mkdir()
        (sibling / "keep").write_text("control")
        cluster.finish(True)
        self.assertFalse(cluster.path.exists())
        self.assertEqual((sibling / "keep").read_text(), "control")

    def test_fixture_password_prevents_a_home_passfile_fallback(self):
        cluster = self.cluster()
        passfile = Path(cluster.env["PGPASSFILE"])
        self.assertEqual(passfile.parent, cluster.path)
        self.assertEqual(passfile.read_text(), "*:*:*:*:reasonbraid-owned-fixture\n")
        self.assertEqual(passfile.stat().st_mode & 0o777, 0o600)
        self.assertEqual(passfile.stat().st_dev, self.root.stat().st_dev)
        self.assertEqual(cluster.env["PGHOST"], "127.0.0.1")
        self.assertEqual(cluster.env["PGPORT"], str(cluster.port))
        self.assertEqual(cluster.env["PGUSER"], "postgres")
        cluster.finish(True)

    def test_failure_retains_stopped_evidence(self):
        cluster = self.cluster()
        cluster.finish(False)
        self.assertEqual(json.loads((cluster.path / "runner.json").read_text())["state"], "stopped")

    def test_unverified_shutdown_never_deletes_data(self):
        cluster = self.cluster()
        cluster.process = object()
        sentinel = cluster.path / "sentinel"
        sentinel.write_text("must survive")
        with patch.object(runner, "stop_group", side_effect=runner.RunnerError("still running")):
            with self.assertRaises(runner.RunnerError):
                cluster.finish(True)
        self.assertEqual(sentinel.read_text(), "must survive")
        self.assertEqual(json.loads((cluster.path / "runner.json").read_text())["state"], "shutdown-unverified")

    def test_unreaped_test_process_prevents_deletion(self):
        cluster = self.cluster()
        cluster.unsafe_cleanup = True
        with self.assertRaises(runner.RunnerError):
            cluster.finish(True)
        self.assertTrue(cluster.path.exists())

    def test_occupied_port_refuses_before_creating_cluster(self):
        with socket.socket() as listener:
            listener.bind(("127.0.0.1", 0))
            listener.listen()
            with self.assertRaises(OSError):
                runner.Cluster(self.root, Path("tools"), dict(os.environ), listener.getsockname()[1])
        self.assertFalse((self.root / "target").exists())

    def test_timed_out_child_is_reaped(self):
        pidfile = self.root / "child.pid"
        code = "import os,time; from pathlib import Path; Path(os.environ['PIDFILE']).write_text(str(os.getpid())); time.sleep(60)"
        with self.assertRaises(subprocess.TimeoutExpired):
            runner.run_command([sys.executable, "-B", "-c", code], {**os.environ, "PIDFILE": str(pidfile)}, timeout=1)
        pid = int(pidfile.read_text())
        self.assertFalse(runner.group_exists(pid))
        with self.assertRaises(ChildProcessError):
            os.waitpid(pid, os.WNOHANG)

    def test_signal_during_spawn_cannot_lose_the_child(self):
        spawned = []
        original_popen = subprocess.Popen
        original_handler = signal.signal(signal.SIGTERM, runner.interrupted)

        def interrupt_after_spawn(*args, **kwargs):
            child = original_popen(*args, **kwargs)
            spawned.append(child)
            os.kill(os.getpid(), signal.SIGTERM)
            return child

        try:
            with patch.object(runner.subprocess, "Popen", side_effect=interrupt_after_spawn):
                with self.assertRaises(KeyboardInterrupt):
                    runner.run_command(
                        [sys.executable, "-B", "-c", "import time; time.sleep(60)"],
                        dict(os.environ),
                    )
            self.assertEqual(len(spawned), 1)
            self.assertFalse(runner.group_exists(spawned[0].pid), "spawn interrupted before handle publication left a child alive")
        finally:
            signal.signal(signal.SIGTERM, original_handler)
            for child in spawned:
                runner.stop_group(child)

    def test_test_failure_and_cleanup_failure_are_nonzero(self):
        with (
            patch.object(sys, "argv", ["run_pg_tests.py", "authority"]),
            patch.object(runner, "project_environment", return_value={}),
            patch.object(runner, "toolchain_directory"),
            patch.object(runner, "pg_tools"),
            patch.object(runner, "Cluster") as cls,
        ):
            cls.return_value.run_test_command.return_value = 9
            cls.return_value.finish.side_effect = runner.RunnerError("shutdown pending")
            self.assertEqual(runner.main(), 1)

    def test_failed_second_command_is_not_success(self):
        with (
            patch.object(sys, "argv", ["run_pg_tests.py", "authority", "regions"]),
            patch.object(runner, "project_environment", return_value={}),
            patch.object(runner, "toolchain_directory"),
            patch.object(runner, "pg_tools"),
            patch.object(runner, "Cluster") as cls,
        ):
            cls.return_value.run_test_command.side_effect = [0, OSError("missing executable")]
            self.assertEqual(runner.main(), 1)
            cls.return_value.finish.assert_called_once_with(False)

    def test_suite_selection_serializes_fixtures(self):
        self.assertEqual(runner.cargo_command("authority"), ["cargo", "test", "--locked", "-p", "reasonbraid-server", "--test", "authority", "--", "--nocapture", "--test-threads=1"])
        self.assertNotIn("--test", runner.cargo_command("mcp"))

    def test_startup_rejects_foreign_server_before_createdb(self):
        cluster = self.cluster()
        process = unittest.mock.Mock()
        process.pid = 12345
        process.poll.return_value = None
        with (
            patch.object(runner, "run_command", side_effect=[
                subprocess.CompletedProcess([], 0),
                subprocess.CompletedProcess([], 0, "foreign|wrong-token\n"),
            ]) as command,
            patch.object(runner.subprocess, "Popen", return_value=process),
        ):
            with self.assertRaisesRegex(runner.RunnerError, "without this runner's identity"):
                cluster.start()
            self.assertEqual(command.call_count, 2)  # initdb + read-only probe; no createdb
        cluster.process = None
        cluster.finish(False)


if __name__ == "__main__":
    unittest.main()
