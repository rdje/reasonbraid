#!/usr/bin/env python3
"""Run pinned, integrity-checked CI scanners with per-invocation owned files."""

from __future__ import annotations

import argparse
from dataclasses import dataclass
import gzip
import hashlib
import io
import json
import os
from pathlib import Path, PurePosixPath
import platform
import signal
import stat
import subprocess
import sys
import tarfile
import tempfile
import zlib

from ci_env import ci_environment
from project_env import ROOT, local_directory, project_environment, toolchain_directory
from run_pg_tests import (
    RunnerError, TERMINAL_SIGNALS, interrupted, run_command, uninterrupted_cleanup,
)


MAX_ARCHIVE_BYTES = 16 * 1024 * 1024
MAX_TAR_BYTES = 96 * 1024 * 1024
MAX_BINARY_BYTES = 64 * 1024 * 1024
MAX_MEMBERS = 64
DOWNLOAD_SECONDS = 120
VERSION_SECONDS = 30
CHECK_SECONDS = 20 * 60
# Official release metadata/checksum manifests reviewed under REPAIR-0035.
# Changes require new source and integrity evidence; never fetch a checksum at runtime.
PINNED_ARCHIVES = {
    "cargo-deny-0.20.2-aarch64-apple-darwin.tar.gz": (
        4517865, "fe67d82a10d8597a3549364cb733a3f9cc1bfff9031b7ae46384a9f2a72090c3",
    ),
    "cargo-deny-0.20.2-aarch64-unknown-linux-musl.tar.gz": (
        4631618, "995c82be0defc7a025cae49a2aa2644ce8245c9a3318fc4103907c6a285e8c7d",
    ),
    "cargo-deny-0.20.2-x86_64-apple-darwin.tar.gz": (
        4731454, "248da7f581724e470071990c088ffc55c811981715f4cbdb258621fb79f8b7a6",
    ),
    "cargo-deny-0.20.2-x86_64-unknown-linux-musl.tar.gz": (
        4936832, "9f12ed4c49936e09b48bf862b595cde2fe64fcbd9d74dfacac6131ca824c8d5f",
    ),
    "gitleaks_8.30.1_darwin_arm64.tar.gz": (
        7897593, "b40ab0ae55c505963e365f271a8d3846efbc170aa17f2607f13df610a9aeb6a5",
    ),
    "gitleaks_8.30.1_darwin_x64.tar.gz": (
        8359235, "dfe101a4db2255fc85120ac7f3d25e4342c3c20cf749f2c20a18081af1952709",
    ),
    "gitleaks_8.30.1_linux_arm64.tar.gz": (
        7601421, "e4a487ee7ccd7d3a7f7ec08657610aa3606637dab924210b3aee62570fb4b080",
    ),
    "gitleaks_8.30.1_linux_x64.tar.gz": (
        8230402, "551f6fc83ea457d62a0d98237cbad105af8d557003051f41f3e7ca7b3f2470eb",
    ),
}
PLATFORMS = {
    ("Linux", "x86_64"): ("x86_64-unknown-linux-musl", "linux_x64"),
    ("Linux", "aarch64"): ("aarch64-unknown-linux-musl", "linux_arm64"),
    ("Darwin", "x86_64"): ("x86_64-apple-darwin", "darwin_x64"),
    ("Darwin", "aarch64"): ("aarch64-apple-darwin", "darwin_arm64"),
}


@dataclass(frozen=True)
class Release:
    tool: str
    version: str
    url: str
    archive: str
    size: int
    sha256: str
    member: str
    version_output: str


def release_for(tool: str, system: str, machine: str) -> Release:
    machine = {"arm64": "aarch64", "AMD64": "x86_64"}.get(machine, machine)
    if (system, machine) not in PLATFORMS:
        raise ValueError("scanner platform unsupported; use an explicitly reviewed release")
    cargo_target, leaks_target = PLATFORMS[system, machine]
    if tool == "cargo-deny":
        version = "0.20.2"
        archive = f"cargo-deny-{version}-{cargo_target}.tar.gz"
        url = f"https://github.com/EmbarkStudios/cargo-deny/releases/download/{version}/{archive}"
        member = archive.removesuffix(".tar.gz") + "/cargo-deny"
        output = "cargo-deny " + version
    elif tool == "gitleaks":
        version = "8.30.1"
        archive = f"gitleaks_{version}_{leaks_target}.tar.gz"
        url = f"https://github.com/gitleaks/gitleaks/releases/download/v{version}/{archive}"
        member, output = "gitleaks", "gitleaks version " + version
    else:
        raise ValueError("unknown scanner")
    size, sha256 = PINNED_ARCHIVES[archive]
    return Release(tool, version, url, archive, size, sha256, member, output)


def download(
    release: Release, destination: Path, env: dict[str, str], *, started=None,
) -> None:
    """Use supervised curl so DNS/TLS/native waits share a bounded lifetime."""
    if not 0 < release.size <= MAX_ARCHIVE_BYTES:
        raise ValueError("archive size outside the reviewed limit")
    # The directory is exclusively created for this invocation. Refuse any
    # preexisting destination, including a dangling link, before external work.
    if destination.exists() or destination.is_symlink():
        raise ValueError("scanner download destination already exists")
    result = run_command([
        "curl", "--disable", "--fail", "--silent", "--show-error", "--location",
        "--proto", "=https", "--proto-redir", "=https",
        "--connect-timeout", "10", "--max-time", str(DOWNLOAD_SECONDS),
        "--max-filesize", str(release.size), "--output", str(destination), release.url,
    ], env, timeout=DOWNLOAD_SECONDS + 5, started=started)
    result.check_returncode()
    metadata = destination.lstat()
    if not stat.S_ISREG(metadata.st_mode) or metadata.st_size != release.size:
        raise ValueError("scanner archive is not the pinned regular file")
    if hashlib.sha256(destination.read_bytes()).hexdigest() != release.sha256:
        raise ValueError("scanner archive checksum mismatch")


def executable_bytes(release: Release, archive: Path) -> bytes:
    """Check original bytes before decompressing; never extract archive paths."""
    metadata = archive.lstat()
    if (
        not 0 < release.size <= MAX_ARCHIVE_BYTES
        or not stat.S_ISREG(metadata.st_mode) or metadata.st_size != release.size
    ):
        raise ValueError("scanner archive is not the pinned regular file")
    with archive.open("rb") as source:
        data = source.read(MAX_ARCHIVE_BYTES + 1)
    if len(data) > MAX_ARCHIVE_BYTES or hashlib.sha256(data).hexdigest() != release.sha256:
        raise ValueError("scanner archive checksum mismatch")
    with gzip.GzipFile(fileobj=io.BytesIO(data)) as compressed:
        expanded = compressed.read(MAX_TAR_BYTES + 1)
    if len(expanded) > MAX_TAR_BYTES:
        raise ValueError("scanner archive expands beyond its limit")
    binary = None
    with tarfile.open(fileobj=io.BytesIO(expanded), mode="r:") as package:
        for count, member in enumerate(package, start=1):
            path = PurePosixPath(member.name)
            if count > MAX_MEMBERS or path.is_absolute() or ".." in path.parts:
                raise ValueError("scanner archive has excessive or unsafe entries")
            if not (member.isdir() or member.isfile()):
                raise ValueError("scanner archive contains a non-regular entry")
            if member.name != release.member:
                continue
            if binary is not None or not member.isfile() or not 0 < member.size <= MAX_BINARY_BYTES:
                raise ValueError("scanner executable is duplicated or outside its size limit")
            stream = package.extractfile(member)
            if stream is None:
                raise ValueError("scanner executable is missing")
            with stream:
                binary = stream.read(MAX_BINARY_BYTES + 1)
            if len(binary) != member.size:
                raise ValueError("scanner executable is truncated")
    if binary is None:
        raise ValueError("scanner executable is missing")
    return binary


def install(
    release: Release, workspace: Path, env: dict[str, str], *, started=None,
) -> Path:
    archive = workspace / "download.tar.gz"
    download(release, archive, env, started=started)
    data = executable_bytes(release, archive)
    binary = workspace / release.tool
    with binary.open("xb") as output:
        output.write(data)
    binary.chmod(0o700)
    if hashlib.sha256(binary.read_bytes()).digest() != hashlib.sha256(data).digest():
        raise ValueError("scanner executable changed during publication")
    return binary


def version_check(
    release: Release, binary: Path, workspace: Path, env: dict[str, str], *, started=None,
) -> None:
    log = workspace / "version.log"
    with log.open("x") as output:
        result = run_command(
            [str(binary), "--version"], env, output=output,
            timeout=VERSION_SECONDS, started=started,
        )
    result.check_returncode()
    if log.stat().st_size > 4096 or log.read_text().strip() != release.version_output:
        raise ValueError("scanner executable returned an unexpected version")


def scanner_environment(root: Path, ambient: dict[str, str], tool: str) -> dict[str, str]:
    environment = ci_environment(root, ambient)
    # curl --disable is first to bypass default curlrc files. Diagnostic TLS/QUIC
    # output must not inherit an arbitrary destination from the calling shell.
    for key in ("SSLKEYLOGFILE", "QLOGDIR"):
        environment.pop(key, None)
    if tool == "cargo-deny":
        # ci_env --rust supplies a local compiler; the developer launcher may
        # instead supply its explicitly selected installed read-only compiler.
        compiler = toolchain_directory(root, ambient)
        environment = project_environment(root, environment, compiler)
    return environment


def scanner_command(tool: str, binary: Path, workspace: Path, root: Path) -> list[str]:
    if tool == "cargo-deny":
        return [str(binary), "--locked", "check"]
    if tool == "gitleaks":
        report = str((workspace / "gitleaks.json").relative_to(root))
        return [str(binary), "detect", "--source", ".", "--redact=100", "--no-color",
                "--report-format", "json", "--report-path", report, "--timeout", str(CHECK_SECONDS)]
    raise ValueError("unknown scanner")


def save_receipt(workspace: Path, receipt: dict) -> None:
    # Terminal cancellation cannot interrupt a small receipt replacement. A
    # failed child phase retains its last PID; inspect it before removing data.
    with uninterrupted_cleanup():
        pending = workspace / "scanner.json.next"
        with pending.open("w") as output:
            output.write(json.dumps(receipt, indent=2) + "\n")
            output.flush()
            os.fsync(output.fileno())
        pending.replace(workspace / "scanner.json")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("tool", choices=("cargo-deny", "gitleaks"))
    parser.add_argument("--verify-only", action="store_true", help="verify installation/version without running a gate")
    args = parser.parse_args()
    os.umask(0o077)
    for sig in TERMINAL_SIGNALS:
        signal.signal(sig, interrupted)
    workspace = None
    receipt = {"tool": args.tool, "scope": "version-only" if args.verify_only else "gate", "state": "preflight"}
    try:
        release = release_for(args.tool, platform.system(), platform.machine())
        environment = scanner_environment(ROOT, dict(os.environ), args.tool)
        parent = local_directory(ROOT, "target/ci-scanners")
        workspace = Path(tempfile.mkdtemp(prefix=args.tool + "-", dir=parent))
        receipt.update({"workspace": str(workspace.relative_to(ROOT)), "version": release.version,
                        "archive": release.archive, "archive_sha256": release.sha256})
        def started(phase):
            def record(pid):
                receipt["last_child"] = {"phase": phase, "pid": pid, "state": "started"}
                save_receipt(workspace, receipt)
            return record

        save_receipt(workspace, receipt)
        binary = install(release, workspace, environment, started=started("download"))
        receipt["last_child"]["state"] = "reaped"
        version_check(release, binary, workspace, environment, started=started("version"))
        receipt["last_child"]["state"] = "reaped"
        receipt["version_verified"] = True
        code = 0
        if not args.verify_only:
            command = scanner_command(args.tool, binary, workspace, ROOT)
            with (workspace / "check.log").open("x") as output:
                result = run_command(
                    command, environment, output=output, timeout=CHECK_SECONDS,
                    started=started("gate"),
                )
            receipt["last_child"]["state"] = "reaped"
            code = result.returncode
        receipt.update({"state": "completed", "exit_code": code})
        # Only these two exclusively created files are disposable. Keep logs,
        # redacted reports and the receipt, including on a scanner refusal.
        binary.unlink()
        (workspace / "download.tar.gz").unlink()
        return code
    except KeyboardInterrupt:
        receipt["state"] = "interrupted"
        print("ci-scanners: interrupted; inspect retained receipt", file=sys.stderr)
        return 130
    except (
        OSError, ValueError, KeyError, EOFError, zlib.error, RunnerError,
        subprocess.SubprocessError, tarfile.TarError,
    ) as error:
        receipt.update({"state": "failed", "error_type": type(error).__name__})
        print(f"ci-scanners: refused: {error}", file=sys.stderr)
        return 2
    finally:
        if workspace is not None:
            save_receipt(workspace, receipt)
            print(f"ci-scanners: {receipt['state']}; evidence {workspace.relative_to(ROOT)}", flush=True)


if __name__ == "__main__":
    raise SystemExit(main())
