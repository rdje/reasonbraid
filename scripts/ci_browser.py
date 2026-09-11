#!/usr/bin/env python3
"""Run trusted browser tests with a pinned, private Chrome for Testing runtime.

This selects a test dependency, not a sandbox for untrusted content. Each call owns
its download and installation; failures retain both for inspection. No desktop
browser, home cache, runtime checksum override or OS policy change is used.
"""

from __future__ import annotations

import argparse
from dataclasses import dataclass
import hashlib
import io
import json
import os
from pathlib import Path, PurePosixPath
import platform
import posixpath
import shutil
import signal
import stat
import subprocess
import sys
import tempfile
import zipfile
import zlib

from ci_env import ci_environment
from project_env import ROOT, local_directory
from run_pg_tests import TERMINAL_SIGNALS, interrupted, run_command, uninterrupted_cleanup


VERSION = "153.0.8010.36"
DOWNLOAD_SECONDS = 300
VERSION_SECONDS = 30
COMMAND_SECONDS = 60 * 60
MAX_ARCHIVE_BYTES = 300 * 1024**2
MAX_EXPANDED_BYTES = 1024**3
MAX_FILE_BYTES = 512 * 1024**2
MAX_MEMBERS = 20_000
# Official HTTPS archives, published object MD5 and local SHA-256 reviewed under
# SIGNOFF-REPAIR.11.4.3.1.2.5. Changes require fresh acquisition/layout evidence.
PINNED_ARCHIVES = {
    "mac-arm64": (191016009, "1f701ef60757c63c6ccf98afaf28291dd0c8d1457d3d738e81fd62201c230ad0"),
    "mac-x64": (201449122, "cddfd83fadf88808fb036f44282488b1cf3f1b50efacce470225ae41f14b0302"),
    "linux64": (195711476, "167a098c4fdec156b58a9f678c90a84f9072d789f9c6e7b35496a6987b8b7ef8"),
    "linux-arm64": (195918275, "dfc4955719c5d494c8507990506d2d5bed174c31bf89266aa2dc5593c6607e8b"),
}
PLATFORMS = {
    ("Darwin", "aarch64"): "mac-arm64",
    ("Darwin", "x86_64"): "mac-x64",
    ("Linux", "aarch64"): "linux-arm64",
    ("Linux", "x86_64"): "linux64",
}


@dataclass(frozen=True)
class Release:
    platform: str
    size: int
    sha256: str

    @property
    def directory(self) -> str:
        return "chrome-" + self.platform

    @property
    def url(self) -> str:
        return (f"https://storage.googleapis.com/chrome-for-testing-public/{VERSION}/"
                f"{self.platform}/{self.directory}.zip")

    @property
    def member(self) -> str:
        executable = ("Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing"
                      if self.platform.startswith("mac-") else "chrome")
        return self.directory + "/" + executable


def release_for(system: str, machine: str) -> Release:
    machine = {"arm64": "aarch64", "AMD64": "x86_64"}.get(machine, machine)
    selected = PLATFORMS.get((system, machine))
    if selected not in PINNED_ARCHIVES:
        raise ValueError("browser platform unsupported; select an explicitly reviewed archive")
    return Release(selected, *PINNED_ARCHIVES[selected])


def archive_bytes(release: Release, archive: Path) -> bytes:
    metadata = archive.lstat()
    if (not 0 < release.size <= MAX_ARCHIVE_BYTES or not stat.S_ISREG(metadata.st_mode)
            or metadata.st_size != release.size):
        raise ValueError("browser archive is not the pinned regular file")
    with archive.open("rb") as source:
        data = source.read(MAX_ARCHIVE_BYTES + 1)
    if len(data) != release.size or hashlib.sha256(data).hexdigest() != release.sha256:
        raise ValueError("browser archive checksum mismatch")
    return data


def archive_layout(package: zipfile.ZipFile, release: Release) -> tuple[dict, dict]:
    """Validate the entire layout before writing; framework links stay internal."""
    entries = package.infolist()
    if len(entries) > MAX_MEMBERS or sum(e.file_size for e in entries) > MAX_EXPANDED_BYTES:
        raise ValueError("browser archive exceeds its entry or expansion limit")
    members, links = {}, {}
    for entry in entries:
        name = entry.filename.removesuffix("/")
        path = PurePosixPath(name)
        mode = entry.external_attr >> 16
        kind = stat.S_IFMT(mode)
        if (not path.parts or path.parts[0] != release.directory or path.is_absolute()
                or str(path) != name or ".." in path.parts or "\\" in name
                or "\0" in entry.orig_filename or name in members or entry.flag_bits & 1
                or kind not in (0, stat.S_IFREG, stat.S_IFDIR, stat.S_IFLNK)
                or not 0 <= entry.file_size <= MAX_FILE_BYTES
                or (entry.is_dir() and (kind not in (0, stat.S_IFDIR) or entry.file_size))
                or (not entry.is_dir() and kind == stat.S_IFDIR)):
            raise ValueError("browser archive has an unsafe or duplicate member")
        members[name] = entry
        if kind == stat.S_IFLNK:
            if not 0 < entry.file_size <= 4096:
                raise ValueError("browser archive link exceeds its limit")
            target = package.read(entry).decode("utf-8")
            if not target or target.startswith("/") or "\0" in target or "\\" in target:
                raise ValueError("browser archive has an unsafe link")
            resolved = posixpath.normpath(posixpath.join(posixpath.dirname(name), target))
            if not resolved.startswith(release.directory + "/"):
                raise ValueError("browser archive link leaves its root")
            links[name] = (target, resolved)

    directories = {str(parent) for name in members for parent in PurePosixPath(name).parents}
    for name in members:
        for parent in PurePosixPath(name).parents:
            entry = members.get(str(parent))
            if entry is not None and not entry.is_dir():
                raise ValueError("browser archive writes beneath a file or link")
    for _, resolved in links.values():
        visited = set()
        while True:
            if resolved in visited or len(visited) >= 128:
                raise ValueError("browser archive link cycle or excessive depth")
            visited.add(resolved)
            parts = PurePosixPath(resolved).parts
            for count in range(1, len(parts) + 1):
                prefix = str(PurePosixPath(*parts[:count]))
                if prefix in links:
                    resolved = posixpath.normpath(posixpath.join(links[prefix][1], *parts[count:]))
                    if not resolved.startswith(release.directory + "/"):
                        raise ValueError("browser archive link leaves its root")
                    break
            else:
                break
        if resolved not in members and resolved not in directories:
            raise ValueError("browser archive link target is missing")
    main = members.get(release.member)
    if (main is None or main.is_dir() or release.member in links
            or not (main.external_attr >> 16) & 0o111 or main.file_size == 0):
        raise ValueError("browser archive executable is missing or not executable")
    return members, links


def extract(release: Release, archive: Path, destination: Path) -> dict:
    # Use the exact bytes just verified, not a second open of an alterable archive.
    data = archive_bytes(release, archive)
    with zipfile.ZipFile(io.BytesIO(data)) as package:
        members, links = archive_layout(package, release)
        destination.mkdir(mode=0o700)
        for name, entry in members.items():
            if name in links:
                continue
            target = destination / name
            target.parent.mkdir(mode=0o700, parents=True, exist_ok=True)
            if entry.is_dir():
                target.mkdir(mode=0o700, exist_ok=True)
                continue
            size = 0
            with package.open(entry) as source, target.open("xb") as output:
                while chunk := source.read(1024 * 1024):
                    size += len(chunk)
                    if size > entry.file_size:
                        raise ValueError("browser archive member expands beyond its declared size")
                    output.write(chunk)
            if size != entry.file_size:
                raise ValueError("browser archive member is truncated")
            target.chmod(0o700 if (entry.external_attr >> 16) & 0o111 else 0o600)
        # Files/directories exist before links. No member may have a link ancestor.
        for name, (target, _) in links.items():
            path = destination / name
            path.parent.mkdir(mode=0o700, parents=True, exist_ok=True)
            path.symlink_to(target)
        return {"entries": len(members), "links": len(links),
                "expanded_bytes": sum(e.file_size for e in members.values())}


def browser_environment(root: Path, ambient: dict[str, str]) -> dict[str, str]:
    environment = ci_environment(root, ambient)
    for key in ("R3_BROWSER_BIN", "CHROME_BIN", "CHROME_LOG_FILE", "SSLKEYLOGFILE", "QLOGDIR"):
        environment.pop(key, None)
    return environment


def save_receipt(workspace: Path, receipt: dict) -> None:
    with uninterrupted_cleanup():
        pending = workspace / "browser.json.next"
        with pending.open("w") as output:
            output.write(json.dumps(receipt, indent=2) + "\n")
            output.flush()
            os.fsync(output.fileno())
        pending.replace(workspace / "browser.json")


def execute(release: Release, workspace: Path, environment: dict[str, str],
            command: list[str], timeout: int, receipt: dict) -> int:
    def phase(name, argv, limit, *, output=None):
        def started(pid):
            receipt["children"].append({"phase": name, "pid": pid, "state": "started"})
            save_receipt(workspace, receipt)
        result = run_command(argv, environment, timeout=limit, output=output, started=started)
        receipt["children"][-1].update(state="reaped", exit_code=result.returncode)
        save_receipt(workspace, receipt)
        return result

    archive = workspace / "archive.zip"
    if archive.exists() or archive.is_symlink():
        raise ValueError("browser archive destination already exists")
    phase("download", [
        "curl", "--disable", "--fail", "--silent", "--show-error", "--location",
        "--proto", "=https", "--proto-redir", "=https", "--connect-timeout", "10",
        "--max-time", str(DOWNLOAD_SECONDS), "--max-filesize", str(release.size),
        "--output", str(archive), release.url,
    ], DOWNLOAD_SECONDS + 5).check_returncode()
    runtime = workspace / "runtime"
    receipt["layout"] = extract(release, archive, runtime)
    binary = runtime / release.member
    receipt.update(binary=str(binary.relative_to(ROOT)),
                   binary_sha256=hashlib.sha256(binary.read_bytes()).hexdigest())
    environment["R3_BROWSER_BIN"] = str(binary)
    with (workspace / "version.log").open("x") as output:
        phase("version", [str(binary), "--version"], VERSION_SECONDS, output=output).check_returncode()
    log = workspace / "version.log"
    if log.stat().st_size > 4096 or log.read_text().strip() != "Google Chrome for Testing " + VERSION:
        raise ValueError("browser executable returned an unexpected version")
    receipt["version_verified"] = True
    # `--version` parses a flag and exits: it proves the binary loads, not that
    # the rendering stack can. Chrome resolves libnss, libgbm, libxkbcommon and
    # friends only when a browser actually starts, so a host can pass the version
    # check and still be unable to render. On Linux, enumerate every unresolved
    # shared object directly — a census of the population, not a sample of it.
    # The result is recorded either way: an empty list is evidence too.
    if platform.system() == "Linux":
        missing: list[str] = []
        probe = subprocess.run(["ldd", str(binary)], capture_output=True, text=True,
                               timeout=VERSION_SECONDS, check=False)
        for line in probe.stdout.splitlines():
            if "not found" in line:
                missing.append(line.strip())
        (workspace / "shared-objects.log").write_text(probe.stdout)
        receipt["unresolved_shared_objects"] = missing
        if missing:
            raise ValueError(
                "the browser executable has unresolved shared objects; install its "
                "runtime dependencies: " + "; ".join(missing)
            )
    code = 0
    if command:
        receipt["state"] = "running"
        code = phase("command", command, timeout).returncode
    receipt.update(state="completed", exit_code=code)
    if code == 0:
        # Only this invocation's fresh payload is disposable. Failed tests retain
        # the selected executable; receipts and version evidence always survive.
        shutil.rmtree(runtime)
        archive.unlink()
        receipt["payload_retired"] = True
    return code if code >= 0 else 128 - code


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--verify-only", action="store_true", help="verify setup/version without running tests")
    parser.add_argument("--timeout", type=int, default=COMMAND_SECONDS, help="test command deadline in seconds (1–7200)")
    parser.add_argument("command", nargs=argparse.REMAINDER)
    args = parser.parse_args()
    command = args.command[1:] if args.command[:1] == ["--"] else args.command
    if bool(command) == args.verify_only or not 1 <= args.timeout <= 7200:
        parser.error("provide either --verify-only or a command, and a timeout from 1 to 7200")
    os.umask(0o077)
    for sig in TERMINAL_SIGNALS:
        signal.signal(sig, interrupted)
    workspace = None
    receipt = {"state": "preflight", "version": VERSION, "children": [],
               "scope": "version-only" if args.verify_only else "trusted-test-command"}
    try:
        release = release_for(platform.system(), platform.machine())
        environment = browser_environment(ROOT, dict(os.environ))
        parent = local_directory(ROOT, "target/ci-browser")
        workspace = Path(tempfile.mkdtemp(prefix=release.platform + "-", dir=parent))
        receipt.update(workspace=str(workspace.relative_to(ROOT)), platform=release.platform,
                       archive_sha256=release.sha256, archive_bytes=release.size)
        save_receipt(workspace, receipt)
        return execute(release, workspace, environment, command, args.timeout, receipt)
    except KeyboardInterrupt:
        receipt["state"] = "interrupted"
        print("ci-browser: interrupted; inspect retained receipt", file=sys.stderr)
        return 130
    except (OSError, ValueError, EOFError, RuntimeError, subprocess.SubprocessError,
            zipfile.BadZipFile, zlib.error) as error:
        receipt.update(state="failed", error_type=type(error).__name__)
        print(f"ci-browser: refused: {error}", file=sys.stderr)
        return 2
    finally:
        if workspace is not None:
            save_receipt(workspace, receipt)
            print(f"ci-browser: {receipt['state']}; evidence {workspace.relative_to(ROOT)}", flush=True)


if __name__ == "__main__":
    raise SystemExit(main())
