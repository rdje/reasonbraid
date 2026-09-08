---
answers:
  - How do project commands keep caches and temporary files on the repository volume?
  - Which installed tools remain read-only external inputs?
  - How can the locked Cargo cache be copied without deleting shared data?
---
# Repository-local command environment

- Decision: 2026-09-09, `SIGNOFF-REPAIR.2.1`.
- Authority: the director's storage/locality directive.

## Measured cause

At review baseline `b58c646`, CARGO_HOME and RUSTUP_HOME were unset. The default
Cargo store and TMPDIR were on device `16777232`; the repository and target were
on `16777244`. Merely locating PostgreSQL under target did not cover compiler,
package, Python, XDG or temporary stores.

## Decision and scope

`scripts/project_env.py` derives paths from its repository root, validates every
store component against symlinks and volume changes, and executes a command with
local Cargo/install/build, rustup, temporary, XDG, compiler-cache and CLI stores.
Data lives in `.project-data/` and build output in `target/`; both are ignored.
Temporary files use restrictive permissions. No saved configuration pins a checkout
location. The Makefile's working commands use the launcher.

The launcher directly selects the installed pinned Rust toolchain's executables;
it does not invoke rustup to install or mutate a shared toolchain. Installed Rust
1.98.0 and the required Python, PostgreSQL, mdBook and OS libraries are read-only
toolchain inputs. On this host the compiler is found under the runtime-derived
user rustup toolchains directory; Python/PostgreSQL/mdBook are Homebrew tools.
`RB_READONLY_TOOLCHAIN` may explicitly select an installed pinned directory.
`.project-data/rustup` remains the writable rustup store for child processes.

Cache seeding reads only an explicitly selected Cargo home. It copies crates.io
archives selected by Cargo.lock, verifying each archive against the lock checksum,
and copies the associated sparse-index records with source/destination hash checks.
It does not copy Cargo credentials, global configuration, installed tools, or the
shared cache database. Every file is atomically published only after verification.
The shared source is not provably unique to ReasonBraid and is therefore retained.
There is no global-cache deletion or claim of a completed off-volume residue purge.

The launcher controls standard environment defaults; commands must still obey the
storage policy for explicit output paths and intrinsic worker behavior. Remaining
entrypoint/CI and worker fixes are owned by `SIGNOFF-REPAIR.11.2`, `.11.3`, `.7.3`
and `.4.2`. This environment is not a filesystem sandbox.

## Evidence and usage

Five focused tests cover default override, relocation, symlink escape, checksum
refusal and preserved-source seeding. The local seed contains 932 files / 98,176,507
bytes, including 484 locked archives. Its generated receipt is
`.project-data/cargo-seed-receipt.json`, SHA-256
`5a66226dd75c8ea82f3ec327ca6d26e29b0987799fbf8c6d0992ac59266d57c1`.
An offline locked Cargo metadata run resolved 496 packages with zero manifest paths
outside the repository. Cargo/rustc report 1.98.0; rustfmt/clippy execute directly
from that installed toolchain. A real child process received repository-local cwd,
temporary and Cargo paths, with shell-shaped argument text preserved literally.
The owning leaf records the focused compile/test and doctrine checks.

```bash
python3 -B scripts/project_env.py --print
python3 -B scripts/project_env.py --seed-cargo-cache "$HOME/.cargo"
python3 -B scripts/project_env.py cargo test --locked --offline -p reasonbraid-core
make book
```

A machine without a complete shared cache can populate the local one using
`python3 -B scripts/project_env.py cargo fetch --locked` when network access is
available. The cache is outside target so an ordinary Cargo clean does not remove it.

The environment choices follow the primary [Cargo environment reference](https://doc.rust-lang.org/cargo/reference/environment-variables.html)
and [Cargo home guide](https://doc.rust-lang.org/cargo/guide/cargo-home.html), consulted
2026-09-09. The archives/index are reusable cache inputs; extracted sources are
regenerated locally by Cargo.
