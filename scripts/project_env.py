#!/usr/bin/env python3
"""Run project commands with repository-local writable stores.

Installed compiler binaries are read-only toolchain inputs. Cache seeding reads
only the caller-selected crates.io cache; it never copies credentials or deletes
shared source data. Requires Python 3.11+ for Cargo.lock parsing.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import shutil
import sys
import tempfile
import tomllib


ROOT = Path(__file__).resolve().parents[1]
STORES = {
    "CARGO_HOME": ".project-data/cargo",
    "CARGO_INSTALL_ROOT": ".project-data/cargo",
    "CARGO_TARGET_DIR": "target",
    "CARGO_BUILD_TARGET_DIR": "target",
    "CARGO_BUILD_BUILD_DIR": "target",
    "RUSTUP_HOME": ".project-data/rustup",
    "TMPDIR": ".project-data/tmp",
    "TMP": ".project-data/tmp",
    "TEMP": ".project-data/tmp",
    "XDG_CACHE_HOME": ".project-data/cache",
    "XDG_DATA_HOME": ".project-data/data",
    "XDG_CONFIG_HOME": ".project-data/config",
    "XDG_STATE_HOME": ".project-data/state",
    "SCCACHE_DIR": ".project-data/cache/sccache",
    "CCACHE_DIR": ".project-data/cache/ccache",
    "CCACHE_TEMPDIR": ".project-data/tmp",
    "REASONBRAID_CLI_STATE": ".project-data/cli",
}
CRATES_IO = "registry+https://github.com/rust-lang/crates.io-index"


def local_directory(root: Path, relative: str) -> Path:
    """Create a local directory, refusing symlinks and mount escapes first."""
    path = Path(relative)
    if path.is_absolute() or ".." in path.parts:
        raise ValueError(f"store must be repository-relative: {relative}")
    device = root.stat().st_dev
    current = root
    for part in path.parts:
        current = current / part
        if current.is_symlink():
            raise ValueError(f"store contains a symlink: {current.relative_to(root)}")
        if current.exists():
            if not current.is_dir() or current.stat().st_dev != device:
                raise ValueError(f"store leaves repository volume: {current.relative_to(root)}")
        else:
            current.mkdir(mode=0o700)
    return current


def toolchain_directory(root: Path, ambient: dict[str, str]) -> Path:
    """Find the pinned, already installed compiler without running rustup."""
    with (root / "rust-toolchain.toml").open("rb") as source:
        pin = tomllib.load(source)["toolchain"]["channel"]
    explicit = ambient.get("RB_READONLY_TOOLCHAIN")
    if explicit:
        candidates = [Path(explicit).expanduser().resolve()]
    else:
        rustup = Path(ambient.get("RUSTUP_HOME", str(Path.home() / ".rustup")))
        machine = {"arm64": "aarch64", "AMD64": "x86_64"}.get(
            platform.machine(), platform.machine()
        )
        candidates = sorted((rustup / "toolchains").glob(f"{pin}-{machine}-*"))
    required = ("cargo", "rustc", "rustdoc", "rustfmt", "cargo-clippy")
    candidates = [
        p for p in candidates
        if p.name.startswith(f"{pin}-")
        and all((p / "bin" / name).is_file() for name in required)
    ]
    if len(candidates) != 1:
        raise ValueError(
            f"expected one installed Rust {pin} toolchain; "
            "set RB_READONLY_TOOLCHAIN to its directory (read-only input)"
        )
    return candidates[0].resolve()


def project_environment(root: Path, ambient: dict[str, str], toolchain: Path) -> dict[str, str]:
    env = dict(ambient)
    for name, relative in STORES.items():
        env[name] = str(local_directory(root, relative))
    compiler_bin = toolchain / "bin"
    env.update({
        "RB_READONLY_TOOLCHAIN": str(toolchain),
        "RUSTC": str(compiler_bin / "rustc"),
        "RUSTDOC": str(compiler_bin / "rustdoc"),
        "RUSTFMT": str(compiler_bin / "rustfmt"),
        "RUSTC_WRAPPER": "",
        "RUSTC_WORKSPACE_WRAPPER": "",
        "CARGO_BUILD_RUSTC_WRAPPER": "",
        "CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER": "",
        "CARGO_BUILD_DEP_INFO_BASEDIR": str(root),
        "PYTHONDONTWRITEBYTECODE": "1",
        "PATH": str(compiler_bin) + os.pathsep + ambient.get("PATH", os.defpath),
    })
    return env


def digest(path: Path) -> str:
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def copy_verified(source: Path, destination: Path, expected: str) -> int:
    """Publish a checked file atomically; an interrupted copy is never usable."""
    if source.is_symlink() or not source.is_file():
        raise ValueError(f"cache input is not a regular file: {source.name}")
    if digest(source) != expected:
        raise ValueError(f"cache checksum mismatch: {source.name}")
    if destination.is_symlink():
        raise ValueError(f"cache destination is a symlink: {destination.name}")
    if destination.is_file() and digest(destination) == expected:
        return destination.stat().st_size
    temporary: Path | None = None
    try:
        with tempfile.NamedTemporaryFile(dir=destination.parent, delete=False) as output:
            temporary = Path(output.name)
            with source.open("rb") as input_file:
                shutil.copyfileobj(input_file, output)
            output.flush()
            os.fsync(output.fileno())
        if digest(temporary) != expected:
            raise ValueError(f"cache changed during copy: {source.name}")
        size = temporary.stat().st_size
        os.replace(temporary, destination)
        return size
    finally:
        if temporary is not None:
            temporary.unlink(missing_ok=True)


def index_path(name: str) -> str:
    name = name.lower()
    if len(name) < 3:
        return f"{len(name)}/{name}"
    if len(name) == 3:
        return f"3/{name[0]}/{name}"
    return f"{name[:2]}/{name[2:4]}/{name}"


def regular_cache_input(home: Path, source: Path) -> Path:
    current = home
    for part in source.relative_to(home).parts:
        current = current / part
        if current.is_symlink():
            raise ValueError(f"cache input contains a symlink: {source.name}")
    if not source.is_file():
        raise ValueError(f"missing cache input: {source.name}")
    return source


def seed_cargo_cache(root: Path, source_home: Path) -> dict[str, object]:
    """Copy locked archives and their sparse-index records; leave source intact."""
    source_home = source_home.resolve(strict=True)
    registry = source_home / "registry"
    indexes = [
        p for p in (registry / "index").glob("index.crates.io-*")
        if (p / "config.json").is_file() and (p / ".cache").is_dir()
    ]
    if len(indexes) != 1:
        raise ValueError("expected one crates.io sparse index in the selected Cargo home")
    index = indexes[0]
    with (root / "Cargo.lock").open("rb") as lock_file:
        packages = tomllib.load(lock_file)["package"]
    plan: dict[str, tuple[Path, str]] = {}
    config = regular_cache_input(source_home, index / "config.json")
    plan[f"registry/index/{index.name}/config.json"] = (config, digest(config))
    for package in packages:
        if "source" not in package:
            continue
        if package["source"] != CRATES_IO:
            raise ValueError("cache seed supports only the locked crates.io source")
        name, version, checksum = (package[k] for k in ("name", "version", "checksum"))
        if not re.fullmatch(r"[A-Za-z0-9_-]+", name) or not re.fullmatch(r"[A-Za-z0-9.+_-]+", version):
            raise ValueError("invalid package path in Cargo.lock")
        if not re.fullmatch(r"[a-f0-9]{64}", checksum):
            raise ValueError("invalid package checksum in Cargo.lock")
        archive = f"registry/cache/{index.name}/{name}-{version}.crate"
        plan[archive] = (regular_cache_input(source_home, source_home / archive), checksum)
        record = f"registry/index/{index.name}/.cache/{index_path(name)}"
        source = regular_cache_input(source_home, source_home / record)
        plan[record] = (source, digest(source))
    # Preflight every input before publishing any file.
    for source, expected in plan.values():
        if source.is_symlink() or not source.is_file() or digest(source) != expected:
            raise ValueError(f"missing or corrupt cache input: {source.name}")
    copied_bytes = 0
    records = []
    for relative, (source, expected) in sorted(plan.items()):
        destination = root / ".project-data/cargo" / relative
        local_directory(root, str(destination.parent.relative_to(root)))
        size = copy_verified(source, destination, expected)
        copied_bytes += size
        records.append({"path": str(destination.relative_to(root)), "bytes": size, "sha256": expected})
    receipt = {
        "source": "caller-selected shared Cargo cache; read-only, not deleted",
        "files": len(records), "bytes": copied_bytes, "records": records,
    }
    receipt_path = root / ".project-data/cargo-seed-receipt.json"
    temporary: Path | None = None
    try:
        with tempfile.NamedTemporaryFile(mode="w", dir=receipt_path.parent, delete=False) as output:
            temporary = Path(output.name)
            output.write(json.dumps(receipt, indent=2) + "\n")
            output.flush()
            os.fsync(output.fileno())
        os.replace(temporary, receipt_path)
    finally:
        if temporary is not None:
            temporary.unlink(missing_ok=True)
    return {"files": len(records), "bytes": copied_bytes, "receipt": str(receipt_path.relative_to(root))}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--print", action="store_true", dest="show")
    parser.add_argument("--seed-cargo-cache", type=Path, metavar="SOURCE_HOME")
    parser.add_argument("command", nargs=argparse.REMAINDER)
    args = parser.parse_args()
    if not args.command and not args.show and not args.seed_cargo_cache:
        parser.error("provide a command, --print, or --seed-cargo-cache")
    os.umask(0o077)
    try:
        if args.seed_cargo_cache:
            print(json.dumps(seed_cargo_cache(ROOT, args.seed_cargo_cache), sort_keys=True))
        if args.show or args.command:
            toolchain = toolchain_directory(ROOT, dict(os.environ))
            env = project_environment(ROOT, dict(os.environ), toolchain)
            if args.show:
                print(json.dumps({key: env[key] for key in (*STORES, "RB_READONLY_TOOLCHAIN")}, indent=2))
            if args.command:
                command = args.command[1:] if args.command[0] == "--" else args.command
                if not command:
                    parser.error("missing command after --")
                os.chdir(ROOT)
                os.execvpe(command[0], command, env)
    except (OSError, ValueError, KeyError) as error:
        print(f"project-env: refused: {error}", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
