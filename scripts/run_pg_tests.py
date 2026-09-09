#!/usr/bin/env python3
"""Run selected integration suites in a supervised, disposable PostgreSQL cluster."""

from __future__ import annotations

import argparse
from contextlib import contextmanager
import json
import os
from pathlib import Path
import secrets
import shutil
import signal
import socket
import subprocess
import sys
import tempfile
import time
from typing import Callable

from project_env import ROOT, local_directory, project_environment, toolchain_directory

SERVER_SUITES = (
    "pg_guard atomic_transaction outbox_worker node_channel authority budget command_api node_work "
    "aggregate_library identity_store node_enrollment node_inbox invitations backup_restore "
    "migration_upgrade escalation node_replacement profiles evaluation routing policy rls "
    "quota quarantine classification federation cards mcp_listen mcp_write allowlist regions"
).split()
SUITES = {name: ("reasonbraid-server", name) for name in SERVER_SUITES}
SUITES.update({"mcp": ("reasonbraid-mcp", None), "cli_end_to_end": ("reasonbraid-cli", "cli_end_to_end")})
TERMINAL_SIGNALS = (signal.SIGINT, signal.SIGTERM, signal.SIGHUP)
CHILD_EXEC = """
import os, signal, sys
signal.pthread_sigmask(signal.SIG_UNBLOCK, (signal.SIGINT, signal.SIGTERM, signal.SIGHUP))
os.execvpe(sys.argv[1], sys.argv[1:], os.environ)
"""


class RunnerError(RuntimeError):
    pass


def group_exists(pid: int) -> bool:
    try:
        os.killpg(pid, 0)
        return True
    except ProcessLookupError:
        return False


def stop_group(process: subprocess.Popen, *, postgres: bool = False) -> None:
    """Reap the direct child and prove its process group is gone before cleanup."""
    if postgres:
        stages = ((signal.SIGINT, 20), (signal.SIGQUIT, 5))
    else:
        stages = ((signal.SIGTERM, 5), (signal.SIGKILL, 5))
    for sig, seconds in stages:
        process.poll()
        if not group_exists(process.pid):
            process.wait()
            return
        try:
            os.killpg(process.pid, sig)
        except ProcessLookupError:
            process.wait()
            return
        deadline = time.monotonic() + seconds
        while time.monotonic() < deadline:
            process.poll()
            if not group_exists(process.pid):
                process.wait()
                return
            time.sleep(0.05)
    raise RunnerError(f"process group {process.pid} has not stopped; retain its workspace")


@contextmanager
def uninterrupted_cleanup():
    handlers = {
        sig: signal.signal(sig, signal.SIG_IGN)
        for sig in (signal.SIGINT, signal.SIGTERM, signal.SIGHUP)
    }
    try:
        yield
    finally:
        for sig, handler in handlers.items():
            signal.signal(sig, handler)


@contextmanager
def deferred_interrupts():
    """Publish a new child handle before delivering a pending terminal signal."""
    previous = signal.pthread_sigmask(signal.SIG_BLOCK, TERMINAL_SIGNALS)
    try:
        yield
    finally:
        signal.pthread_sigmask(signal.SIG_SETMASK, previous)


def child_argv(argv: list[str]) -> list[str]:
    # The child inherits the temporary mask. A tiny explicit-interpreter
    # trampoline restores it before exec, without a preexec_fn in the parent.
    return [sys.executable, "-B", "-c", CHILD_EXEC, *argv]


def run_command(
    argv: list[str], env: dict[str, str], *, output=None, timeout=None,
    started: Callable[[int], None] | None = None, capture: bool = False,
    input_text: str | None = None,
) -> subprocess.CompletedProcess:
    process = None
    try:
        with deferred_interrupts():
            process = subprocess.Popen(
                child_argv(argv), cwd=ROOT, env=env,
                stdout=subprocess.PIPE if capture else output,
                stderr=subprocess.STDOUT if output or capture else None,
                stdin=subprocess.PIPE if input_text is not None else None,
                start_new_session=True, text=capture or input_text is not None,
            )
        if started:
            started(process.pid)
        if capture or input_text is not None:
            stdout, _ = process.communicate(input=input_text, timeout=timeout)
        else:
            process.wait(timeout=timeout)
            stdout = None
        return subprocess.CompletedProcess(argv, process.returncode, stdout)
    finally:
        if process:
            with uninterrupted_cleanup():
                try:
                    stop_group(process)
                finally:
                    if capture and process.stdout:
                        process.stdout.close()
                    if process.stdin:
                        process.stdin.close()


def pg_environment(ambient: dict[str, str], pg_bin: Path) -> dict[str, str]:
    # libpq services/options could redirect a client or alter readiness SQL.
    env = {
        key: value for key, value in ambient.items()
        if not key.startswith("PG") and key != "DATABASE_URL"
    }
    env.update({
        "PGCONNECT_TIMEOUT": "2",
        "PATH": str(pg_bin) + os.pathsep + env.get("PATH", os.defpath),
    })
    return env


def available_port(requested: int) -> int:
    with socket.socket() as reservation:
        reservation.bind(("127.0.0.1", requested))
        return reservation.getsockname()[1]


def pg_tools(ambient: dict[str, str]) -> Path:
    selected = ambient.get("PG_BIN")
    if not selected and ambient.get("PG_PREFIX"):
        selected = str(Path(ambient["PG_PREFIX"]) / "bin")
    if not selected:
        postgres = shutil.which("postgres")
        if postgres:
            selected = str(Path(postgres).parent)
        elif shutil.which("brew"):
            result = run_command(["brew", "--prefix", "postgresql@16"], ambient, capture=True, timeout=10)
            if result.returncode == 0:
                selected = str(Path(result.stdout.strip()) / "bin")
    if not selected:
        raise RunnerError("set PG_BIN to installed PostgreSQL 16 binaries")
    directory = Path(selected).resolve(strict=True)
    for tool in ("postgres", "initdb", "psql", "createdb", "dropdb", "pg_dump", "pg_restore"):
        if not os.access(directory / tool, os.X_OK):
            raise RunnerError(f"missing PostgreSQL tool: {tool}")
    version = run_command([str(directory / "postgres"), "--version"], ambient, capture=True, timeout=10)
    if version.returncode or not version.stdout.startswith("postgres (PostgreSQL) 16."):
        raise RunnerError("PG_BIN must select PostgreSQL 16")
    return directory


class Cluster:
    def __init__(self, root: Path, pg_bin: Path, env: dict[str, str], port: int = 0):
        self.root, self.pg_bin, self.env = root, pg_bin, pg_environment(env, pg_bin)
        self.port = available_port(port)  # Occupied explicit ports fail before creating data.
        parent = local_directory(root, "target/pg-tests")
        self.path = Path(tempfile.mkdtemp(prefix="run-", dir=parent))
        self.relative = str(self.path.relative_to(root))
        self.env.update({"PGPASSFILE": str(self.path / "unused.pgpass"), "PGSYSCONFDIR": str(self.path)})
        self.token = secrets.token_hex(24)
        self.database = "rb_test_" + secrets.token_hex(12)
        self.process: subprocess.Popen | None = None
        self.log = None
        self.unsafe_cleanup = False
        self.command_count = 0
        self.receipt = {
            "workspace": self.relative, "database": self.database,
            "port": self.port, "state": "created",
        }
        self.record()

    def record(self):
        pending = self.path / "runner.next.json"
        with pending.open("w") as stream:
            stream.write(json.dumps(self.receipt, indent=2) + "\n")
            stream.flush()
            os.fsync(stream.fileno())
        os.replace(pending, self.path / "runner.json")

    def command_started(self, pid: int):
        self.receipt.pop("command_exit", None)
        self.receipt.update({"command_pid": pid, "state": "running"})
        self.record()

    def run_test_command(self, command: list[str]) -> int:
        self.command_count += 1
        log_path = self.path / f"command-{self.command_count}.log"
        self.receipt["command_log"] = str(log_path.relative_to(self.root))
        try:
            with log_path.open("w") as output:
                return run_command(
                    command, self.env, output=output, started=self.command_started,
                ).returncode
        finally:
            # Preserve diagnostics during execution and replay them without
            # loading an unbounded test log into memory.
            if log_path.exists():
                with log_path.open(errors="replace") as output:
                    shutil.copyfileobj(output, sys.stdout)
                sys.stdout.flush()

    def sql_args(self, database: str) -> list[str]:
        return [
            str(self.pg_bin / "psql"), "-X", "-w", "-h", "127.0.0.1",
            "-p", str(self.port), "-U", "postgres", "-d", database,
            "-v", "ON_ERROR_STOP=1", "-At",
        ]

    def create_database(self):
        # Recheck identity on the SAME connection that creates the database.
        # A port reused after the readiness probe must not receive a mutation.
        query = r"""
SELECT CASE WHEN current_setting('data_directory') = :'expected_directory'
  AND current_setting('reasonbraid.test_owner', true) = :'expected_owner'
THEN format('CREATE DATABASE %I', :'new_database')
ELSE 'SELECT 1/0 /* runner identity mismatch: refuse creation */'
END
\gexec
"""
        result = run_command(self.sql_args("postgres") + [
            "-v", f"expected_directory={self.path / 'data'}",
            "-v", f"expected_owner={self.token}",
            "-v", f"new_database={self.database}",
        ], self.env, output=self.log, input_text=query, timeout=10)
        if result.returncode:
            raise RunnerError("database creation failed or server identity changed")

    def start(self):
        self.log = (self.path / "postgres.log").open("w")
        initialized = run_command([
            str(self.pg_bin / "initdb"), "-D", str(self.path / "data"),
            "-A", "trust", "-U", "postgres", "--no-instructions",
        ], self.env, output=self.log, timeout=60)
        if initialized.returncode:
            raise RunnerError("initdb failed; inspect retained postgres.log")
        # Disable Unix sockets entirely: their default directory is often off-volume.
        with deferred_interrupts():
            self.process = subprocess.Popen(child_argv([
                str(self.pg_bin / "postgres"), "-D", str(self.path / "data"),
                "-h", "127.0.0.1", "-p", str(self.port), "-k", "",
                "-c", "max_connections=100", "-c", f"reasonbraid.test_owner={self.token}",
            ]), cwd=self.root, env=self.env, stdout=self.log, stderr=subprocess.STDOUT, start_new_session=True)
        self.receipt.update({"postgres_pid": self.process.pid, "state": "starting"})
        self.record()
        deadline = time.monotonic() + 30
        while time.monotonic() < deadline:
            if self.process.poll() is not None:
                raise RunnerError("owned PostgreSQL exited during startup")
            identity_query = (
                "SELECT current_setting('data_directory'), "
                "current_setting('reasonbraid.test_owner', true)"
            )
            result = run_command(
                self.sql_args("postgres") + ["-c", identity_query],
                self.env, capture=True, timeout=4,
            )
            if result.returncode == 0:
                expected = f"{self.path / 'data'}|{self.token}"
                if result.stdout.strip() != expected or self.process.poll() is not None:
                    raise RunnerError("port reached a server without this runner's identity; refusing mutation")
                break
            time.sleep(0.1)
        else:
            raise RunnerError("owned PostgreSQL did not become ready")
        self.create_database()
        self.env.update({
            "DATABASE_URL": f"postgres://postgres@127.0.0.1:{self.port}/{self.database}?sslmode=disable",
            "RB_TEST_CLUSTER": self.relative, "RB_TEST_OWNER": self.token,
            "RB_TEST_DATABASE": self.database, "RUST_TEST_THREADS": "1",
        })
        self.receipt["state"] = "ready"
        self.record()
        print(f"pg-tests: owned cluster {self.relative}; database {self.database}; port {self.port}", flush=True)

    def finish(self, success: bool):
        try:
            if self.process:
                stop_group(self.process, postgres=True)
            if self.unsafe_cleanup:
                raise RunnerError("a test command could not be reaped; retain its workspace")
        except (OSError, RunnerError):
            self.receipt["state"] = "shutdown-unverified"
            self.record()
            raise
        finally:
            if self.log:
                self.log.close()
        self.receipt["state"] = "stopped"
        self.record()
        if success:
            shutil.rmtree(self.path)
            if self.path.exists():
                raise RunnerError("cluster cleanup left residue")
            print(f"pg-tests: stopped and removed {self.relative}", flush=True)
        else:
            print(f"pg-tests: stopped; failure evidence retained in {self.relative}", file=sys.stderr, flush=True)


def cargo_command(suite: str) -> list[str]:
    package, test = SUITES[suite]
    command = ["cargo", "test", "--locked", "-p", package]
    if test:
        command += ["--test", test]
    return command + ["--", "--nocapture", "--test-threads=1"]


def interrupted(signum, _frame):
    raise KeyboardInterrupt(f"signal {signum}")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("suites", nargs="*", help="named integration suites; no names runs the broad suite")
    parser.add_argument("--list", action="store_true", help="list supported suite names without starting PostgreSQL")
    parser.add_argument("--demo", action="store_true", help="also run the two-host demonstration")
    parser.add_argument("--port", type=int, default=0, help="explicit loopback port; default chooses an available port")
    args = parser.parse_args()
    if args.list:
        print("\n".join(SUITES))
        return 0
    if not 0 <= args.port <= 65535:
        parser.error("port must be between 0 and 65535")
    unknown = sorted(set(args.suites) - SUITES.keys())
    if unknown:
        parser.error("unknown suite(s): " + ", ".join(unknown))
    os.umask(0o077)
    cluster = None
    succeeded = False
    code = 1
    handlers = {sig: signal.getsignal(sig) for sig in (signal.SIGINT, signal.SIGTERM, signal.SIGHUP)}
    try:
        ambient = dict(os.environ)
        env = project_environment(ROOT, ambient, toolchain_directory(ROOT, ambient))
        cluster = Cluster(ROOT, pg_tools(ambient), env, args.port)
        for sig in (signal.SIGINT, signal.SIGTERM, signal.SIGHUP):
            signal.signal(sig, interrupted)
        cluster.start()
        suites = list(dict.fromkeys(args.suites)) if args.suites else list(SUITES)
        commands = [cargo_command(suite) for suite in suites]
        if args.demo or (not args.suites and ambient.get("RB_DEMO", "1") != "0"):
            commands.append(["bash", "scripts/demo_two_host.sh", "--database-url", cluster.env["DATABASE_URL"]])
        for command in commands:
            label = " ".join(command[:command.index("--")]) if "--" in command else "demonstration"
            print("pg-tests: running " + label, flush=True)
            cluster.receipt["command"] = label
            try:
                code = cluster.run_test_command(command)
            except (OSError, RunnerError):
                cluster.unsafe_cleanup = True
                raise
            cluster.receipt.update({"command_exit": code, "state": "ready" if code == 0 else "command-failed"})
            cluster.record()
            if code:
                code = code if code > 0 else 128 - code
                break
        else:
            succeeded, code = True, 0
    except KeyboardInterrupt:
        code = 130
        print("pg-tests: interrupted; stopping owned processes", file=sys.stderr)
    except (OSError, ValueError, RunnerError, subprocess.TimeoutExpired) as error:
        code = 1
        print(f"pg-tests: refused: {error}", file=sys.stderr)
    finally:
        # Repeated terminal signals must not interrupt proof of shutdown.
        if cluster:
            for sig in (signal.SIGINT, signal.SIGTERM, signal.SIGHUP):
                signal.signal(sig, signal.SIG_IGN)
            try:
                cluster.finish(succeeded)
            except (OSError, RunnerError) as error:
                print(f"pg-tests: cleanup unverified: {error}; retain {cluster.relative}", file=sys.stderr)
                code = 1
        for sig, handler in handlers.items():
            signal.signal(sig, handler)
    return code


if __name__ == "__main__":
    raise SystemExit(main())
