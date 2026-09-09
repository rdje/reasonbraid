# Compiler artifact disposition at the checkpoint

Owner: `SIGNOFF-REPAIR.11.4.3.1.6`; REPAIR-0041. Source baseline `0a16063`.
Raw evidence and task-local diagnostic scripts: `target/compiler-artifact-controls`.
This operation changes generated incremental data and documentation. Production,
compiler configuration, dependencies and historical diagnostic files are preserved.

## Inventory and selection

The fresh census records device 16777244, UID 501, 6,160 directories and 293,029
files totaling 85,345,202,740 logical bytes beneath `target/debug/incremental`.
There are 11,335 `.bin` files / 53,487,776,750 bytes; 9,684 / 42,987,212,776 bytes
are older than 24 hours. No symlink, foreign-device or nonregular entry is found.
Available space exceeds three TB; this is periodic hygiene, not a space emergency.

Select whole older finalized sessions, never isolated dependency-graph/cache
components. Retain every session tied at each crate variant's greatest encoded
microsecond timestamp, all working/partial/unknown layouts, and anything younger
than 24 hours by encoded timestamp or directory/file modification time. This
operation additionally retains every session containing a hard-linked file; it
makes no assumption about other links' ownership or physical allocation.

A census hashes 3,545 tracked source and retained textual diagnostic files and finds
no concrete incremental session reference. It includes diagnostic trees outside
compiler build output; it does not claim a binary-debug-information search or
read every database page. The exact source/control executables remain untouched.
The final selection has 645 sessions / 1,984 singly-linked files / 5,116,558,334
logical bytes. Finalized-session exclusions are 2,340 newest, 626 with hard links
and 149 young; working/partial data is outside that finalized selection.

The frozen manifest SHA-256 is
`541ffa9f961b3632e93608fa32dc1dda04d88051d461f2bf94f3d614651c653d`.
It records each root-relative path, device, inode, owner, mode, size, modification
and change time, link count, session/lock correspondence and retained newest
timestamp. Its census/reference hashes bind the selection to the inspected inputs.
Age alone does not authorize deletion; this is not an unattended cleanup command.

## Pinned compiler contract and native falsification

The installed compiler is rustc 1.98.0, commit
`88d9e12ae178fab0fb5cc050a94da85685d449ea`, aarch64-apple-darwin.
The exact official [incremental persistence source](https://raw.githubusercontent.com/rust-lang/rust/88d9e12ae178fab0fb5cc050a94da85685d449ea/compiler/rustc_incremental/src/persist/fs.rs)
defines immutable finalized sessions, shared copying locks and exclusive collection
locks. A lock is the sibling `s-<timestamp>-<random>.lock`. Its macOS
[lock implementation](https://raw.githubusercontent.com/rust-lang/rust/88d9e12ae178fab0fb5cc050a94da85685d449ea/compiler/rustc_data_structures/src/flock/unix.rs)
uses whole-file POSIX `fcntl(F_SETLK)`. Python `fcntl.lockf` exercises that protocol;
a generic API called flock is insufficient evidence of interoperability.

The first link-stage probe contains a corrected pre-execution quoting error, then
compiles successfully but fails its assertion that the compiler still holds the
lock. Preserve `native-control.json` and `native-96k76a5v`. Pinned fs.rs explicitly
releases the lock when publishing a finalized session. The corrected observation
in `link-stage-control.json` records an already finalized session while the linker
is held, both shared/exclusive locks available, successful output and consumed
compiler/program groups. This explains the probe's faulty phase assumption;
it is not a compiler locking defect.

The stronger `gc_control.py` creates an isolated real incremental cache. Holding
an exclusive lock across two later native compilations preserves the old session's
file hashes; releasing it lets the compiler retire that session and its lock.
The same shared-lock control passes. Seven compilations and seven executable
invocations all return zero and print the expected program output; every group is
consumed. `gc-control.json` records both protected identities and terminal results.
These native controls qualify this pinned macOS compiler, not other platform locks.

| Pinned source snapshot | SHA-256 |
| --- | --- |
| pinned-fs.rs | `6d7bf397f1c9186cb55b983d06568569f94aa4f20e48324dda6e1188c8f54136` |
| pinned-flock.rs | `b4e3a9b23a4b81aa827c060beb382a08fd302568082e7a21c01bb224633b458c` |
| pinned-flock-unix.rs | `a3dfcc8c2e5837070e0df739c93493bd0e338c625286d8bc641bdb6cf869f6de` |

## Guarded retirement and residue

`retire.py` first requires a consumed native project-process census with no active
project jobs. It verifies the frozen manifest hash, repository volume and UID,
opens every ancestor without following symlinks, checks its original identity and
refuses writable foreign/group/other paths. Each session's existing, unique lock
is opened once and held exclusively through deletion. A busy lock is retained.
Closing another descriptor to the same lock could release POSIX process locks,
so the critical section never reopens that lock file.

Before removing any session contents, verify its exact layout and every file's
original metadata. Use descriptor-relative no-follow checks and unlink operations,
then remove only the verified empty original directory and sync its parent. Retain
the 645 zero-byte lock files; the compiler may collect them later. All 645 selected
sessions retire; zero locks are busy. `retirement.json` is flushed/synced after each
unit so interrupted progress is recoverable. A changed identity refuses; this is
not adversarial containment against another process with the same account's full
filesystem authority.

Independent `verify_retirement.py` returns zero: exactly 645 directories / 1,984
files are absent, with 5,515 directories / 291,045 files / 80,228,644,406 logical
bytes remaining. Every retained file's original metadata is equal; all retained
directory identities match, allowing only expected parent modification times.
All 3,545 inspected source/diagnostic contents still match their original hashes.
No failed browser, publisher, scanner, PostgreSQL fixture, shared global store,
release/dependency binary or diagnostic log is deleted. Logical removed bytes
are not claimed as physical disk-space reclamation on a cloning filesystem.

## Affected workflow and closure

`cargo check --offline --locked -p reasonbraid-server --lib` passes, rc=0,
375.636 seconds, with its process group consumed. Its read-only one-second native sample
completed capture but exceeded the thirty-second supervisor deadline during symbol
processing; no stack report was produced. The sampler group cleanup was consumed
and the exact log/timeout retained. This adds no wait-stack diagnosis to `.11.2`.
A bounded retry finds no still-running compiler and starts no sampler. The
original timeout remains preserved; the successful check supplies no wait-stack
diagnosis. `make book`, eight rendered markers, unchanged production/README
inspection and `git diff --check` pass, rc=0. All results are consumed.
Full checkpoint execution
remains `.11.4.3.1.2`; no full local/remote CI or push is claimed by this cleanup.
The unresolved startup investigation `.11.2` retains all its evidence.
