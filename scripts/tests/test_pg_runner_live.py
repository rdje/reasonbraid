#!/usr/bin/env python3
"""Real PostgreSQL controls. Requires installed PG tools and loopback sockets."""

import json
import os
from pathlib import Path
import select
import shutil
import signal
import subprocess
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import run_pg_tests as runner


class LivePgRunnerTests(unittest.TestCase):
    def setUp(self):
        parent = runner.local_directory(runner.ROOT, "target/pg-runner-live-controls")
        self.root = Path(tempfile.mkdtemp(prefix="root with 'quotes'-", dir=parent))
        self.clusters = []
        self.safe_to_delete = True
        self.pg_bin = runner.pg_tools(dict(os.environ))
        self.env = dict(os.environ)

    def tearDown(self):
        # A failing shutdown retains all evidence; never rely on a temp-directory
        # destructor, which would remove live PostgreSQL files during unwinding.
        for cluster in self.clusters:
            if cluster.path.exists():
                cluster.finish(False)
        if self.safe_to_delete:
            shutil.rmtree(self.root)
        else:
            print(f"live-control: shutdown not verified; retained {self.root.relative_to(runner.ROOT)}", file=sys.stderr)

    def cluster(self):
        instance = runner.Cluster(self.root, self.pg_bin, self.env)
        self.clusters.append(instance)
        return instance

    def test_unique_clusters_ignore_caller_url_and_remove_only_owned_data(self):
        self.env.update({"DATABASE_URL": "postgres://unowned.invalid/never_use", "PGHOST": "unowned.invalid", "PGOPTIONS": "-c search_path=wrong"})
        first = self.cluster()
        first.start()
        second = self.cluster()
        second.start()
        self.assertNotEqual(first.port, second.port)
        self.assertNotEqual(first.database, second.database)
        self.assertEqual(first.path.stat().st_dev, runner.ROOT.stat().st_dev)
        for cluster in (first, second):
            result = subprocess.run(cluster.sql_args(cluster.database) + ["-c", "CREATE TABLE witness (value text); INSERT INTO witness VALUES ('owned'); SELECT value FROM witness"], env=cluster.env, capture_output=True, text=True, check=True)
            self.assertIn("owned", result.stdout)
        first.finish(True)
        self.assertFalse(runner.group_exists(first.process.pid))
        self.assertTrue(second.path.exists())
        result = subprocess.run(second.sql_args(second.database) + ["-c", "SELECT value FROM witness"], env=second.env, capture_output=True, text=True, check=True)
        self.assertEqual(result.stdout.strip(), "owned")
        second.finish(True)

    def test_failed_command_preserves_stopped_log(self):
        cluster = self.cluster()
        cluster.start()
        code = cluster.run_test_command([
            sys.executable, "-B", "-c", "print('failure witness'); raise SystemExit(7)",
        ])
        self.assertEqual(code, 7)
        self.assertEqual((cluster.path / "command-1.log").read_text().strip(), "failure witness")
        command_receipt = json.loads((cluster.path / "runner.json").read_text())
        self.assertEqual(command_receipt["state"], "running")
        self.assertFalse(runner.group_exists(command_receipt["command_pid"]))
        cluster.finish(False)
        self.assertFalse(runner.group_exists(cluster.process.pid))
        self.assertEqual(json.loads((cluster.path / "runner.json").read_text())["state"], "stopped")
        self.assertIn("database system is shut down", (cluster.path / "postgres.log").read_text())

    def test_server_change_before_creation_does_not_mutate_the_other_cluster(self):
        first = self.cluster()
        first.start()
        second = self.cluster()
        second.start()
        original_port = first.port
        try:
            first.port = second.port
            with self.assertRaisesRegex(runner.RunnerError, "server identity changed"):
                first.create_database()
            result = subprocess.run(
                second.sql_args("postgres") + [
                    "-c", f"SELECT count(*) FROM pg_database WHERE datname = '{first.database}'",
                ], env=second.env, capture_output=True, text=True, check=True,
            )
            self.assertEqual(result.stdout.strip(), "0")
        finally:
            first.port = original_port
        first.finish(True)
        second.finish(True)

    def test_signal_reaps_server_and_command(self):
        script = """
import json, os, signal, sys
from pathlib import Path
sys.path.insert(0, 'scripts')
import run_pg_tests as r
c = r.Cluster(Path(os.environ['CONTROL_ROOT']), r.pg_tools(dict(os.environ)), dict(os.environ))
for sig in (signal.SIGINT, signal.SIGTERM, signal.SIGHUP):
    signal.signal(sig, r.interrupted)
try:
    c.start()
    print('READY ' + json.dumps({'path': str(c.path), 'pid': c.process.pid}), flush=True)
    r.run_command([sys.executable, '-B', '-c', 'import time; time.sleep(60)'], c.env)
except KeyboardInterrupt:
    pass
finally:
    with r.uninterrupted_cleanup():
        c.finish(False)
"""
        process = subprocess.Popen([sys.executable, "-B", "-c", script], cwd=runner.ROOT, env={**self.env, "CONTROL_ROOT": str(self.root)}, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, start_new_session=True)
        self.safe_to_delete = False
        ready = None
        try:
            # Read bytes directly so TextIO read-ahead cannot hide a READY line
            # from select when multiple lines arrive together.
            output = b""
            for _ in range(40):
                if not select.select([process.stdout], [], [], 1)[0]:
                    continue
                chunk = os.read(process.stdout.fileno(), 4096)
                if not chunk:
                    break
                output += chunk
                for line in output.decode().splitlines():
                    if line.startswith("READY "):
                        ready = json.loads(line[6:])
                if ready:
                    break
            self.assertIsNotNone(ready, output.decode())
            process.send_signal(signal.SIGTERM)
            remaining, _ = process.communicate(timeout=30)
            self.assertEqual(process.returncode, 0, remaining)
            self.assertFalse(runner.group_exists(ready["pid"]))
            receipt = json.loads((Path(ready["path"]) / "runner.json").read_text())
            self.assertEqual(receipt["state"], "stopped")
            self.safe_to_delete = True
        finally:
            runner.stop_group(process)
            # If the driver crashed without completing shutdown, retain its
            # root; only this exact live process handle may be signalled here.
            if ready and runner.group_exists(ready["pid"]):
                self.safe_to_delete = False
                self.fail("driver left PostgreSQL running; evidence retained")


if __name__ == "__main__":
    os.umask(0o077)
    unittest.main()
