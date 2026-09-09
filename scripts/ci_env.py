#!/usr/bin/env python3
"""Execute CI commands with checkout-local stores; optionally install pinned Rust.

Installed Python/rustup/OS executables are read-only inputs. Compiler installation
and the configured writable stores belong to this checkout. This is an environment
launcher, not a filesystem sandbox for arbitrary commands or explicit paths.
"""

from __future__ import annotations

import argparse
import os
from pathlib import Path
import re
import signal
import subprocess
import sys
import tomllib

from project_env import (
    ROOT, STORES, local_directory, project_environment, toolchain_directory,
)
from run_pg_tests import RunnerError, TERMINAL_SIGNALS, interrupted, run_command


INSTALL_TIMEOUT_SECONDS = 20 * 60
CI_OVERRIDES = (
    "DATABASE_URL", "RB_DEMO", "RB_LIVE_CODEX", "RB_LIVE_CLAUDE",
    "GITLEAKS_CONFIG", "GITLEAKS_CONFIG_TOML", "RB_READONLY_TOOLCHAIN",
    "RUSTUP_TOOLCHAIN", "RUSTUP_DIST_SERVER", "RUSTUP_UPDATE_ROOT",
    "RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS",
)
WRAPPERS = (
    "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER", "CARGO_BUILD_RUSTC_WRAPPER",
    "CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER",
)


def local_compiler(root: Path, environment: dict[str, str]) -> Path:
    """Require an executable compiler wholly inside the owned installation store."""
    compiler = toolchain_directory(root, environment)
    relative = compiler.relative_to(root)
    local_directory(root, str(relative / "bin"))
    if not compiler.is_relative_to(Path(environment["RUSTUP_HOME"])):
        raise ValueError("CI compiler leaves its installation store")
    for name in ("cargo", "rustc", "rustdoc", "rustfmt", "cargo-clippy"):
        path = compiler / "bin" / name
        if (
            path.is_symlink() or path.stat().st_dev != root.stat().st_dev
            or not os.access(path, os.X_OK)
        ):
            raise ValueError(f"CI compiler component is not a local executable: {name}")
    return compiler


def ci_environment(
    root: Path, ambient: dict[str, str], *, rust: bool = False,
    install_timeout: float = INSTALL_TIMEOUT_SECONDS,
) -> dict[str, str]:
    """Establish owned stores before running an installer or the requested command."""
    environment = {key: value for key, value in ambient.items() if key not in CI_OVERRIDES}
    for name, relative in STORES.items():
        environment[name] = str(local_directory(root, relative))
    environment["PYTHONDONTWRITEBYTECODE"] = "1"
    for name in WRAPPERS:
        environment[name] = ""
    if not rust:
        return environment

    installation = local_directory(root, ".project-data/installed-toolchains")
    environment["RUSTUP_HOME"] = str(installation)
    with (root / "rust-toolchain.toml").open("rb") as source:
        pin = tomllib.load(source)["toolchain"]["channel"]
    if not isinstance(pin, str) or not re.fullmatch(r"[0-9]+\.[0-9]+\.[0-9]+", pin):
        raise ValueError("CI requires an exact numeric Rust toolchain pin")
    toolchains = local_directory(root, ".project-data/installed-toolchains/toolchains")
    if any(candidate.is_symlink() for candidate in toolchains.glob(f"{pin}-*")):
        raise ValueError("CI compiler directory is a symlink")
    # Reuse only a complete installed compiler. If discovery fails, rustup may
    # complete a missing installation; never delete or overwrite its files here.
    try:
        toolchain_directory(root, environment)
    except ValueError:
        result = run_command([
            "rustup", "toolchain", "install", pin, "--profile", "minimal",
            "--component", "rustfmt", "--component", "clippy", "--no-self-update",
        ], environment, timeout=install_timeout)
        result.check_returncode()
    compiler = local_compiler(root, environment)
    environment = project_environment(root, environment, compiler)
    environment["RUSTUP_HOME"] = str(installation)
    return environment


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--rust", action="store_true",
        help="provision the repository-pinned Rust toolchain locally",
    )
    parser.add_argument("command", nargs=argparse.REMAINDER)
    args = parser.parse_args()
    command = args.command[1:] if args.command[:1] == ["--"] else args.command
    if not command:
        parser.error("provide a command")
    os.umask(0o077)
    for sig in TERMINAL_SIGNALS:
        signal.signal(sig, interrupted)
    try:
        environment = ci_environment(ROOT, dict(os.environ), rust=args.rust)
        os.chdir(ROOT)
        # The command replaces the launcher: there is no unconsumed parent job.
        os.execvpe(command[0], command, environment)
    except KeyboardInterrupt:
        print("ci-env: interrupted; installer cleanup consumed", file=sys.stderr)
        return 130
    except (OSError, ValueError, KeyError, RunnerError, subprocess.SubprocessError) as error:
        print(f"ci-env: refused: {error}", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
