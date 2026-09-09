# CLI state publication qualification

Owner: SIGNOFF-REPAIR.3.3.4.3.3.3.3.1.1. Product baseline: 8c08558.
Status: qualified StateFile storage API; all twelve selected controls, all-target CLI strict lint and book verification pass. Full CLI writer/request integration remains separate.

## Source census and matched controls

CLI lib.rs Config/StateFile load/save, both run_enroll/run_thread_create state
writers and main.rs eager state load were reviewed. Config defaults relative to
CWD and accepts arbitrary paths; save creates directories and writes state.json
in place, without sync or process exclusion. The two real CLI harness workspaces
are target/rb-cli-e2e-full-flow and target/rb-cli-e2e-denials, derived through
CARGO_MANIFEST_DIR plus lexical parent components. No shared Rust root/store
helper was found. rb-site requires repository discovery and explicitly refuses
unsupported non-Unix storage verification. Cached Cargo.lock Rustix 1.1.4 has
safe openat/mkdirat/renameat/unlinkat/flock/fsync and Apple fullfsync wrappers.

Command: `python3 -B scripts/project_env.py cargo test --locked -p reasonbraid-cli --test state_publication -- --nocapture --test-threads=1`.
Result rc=101, one pass/three failures; build 48.89s, execution 0.02s.

- An already-open reader reads replacement bytes after StateFile::save; its old
  snapshot is not retained. This deterministic handle observation needs no
  timing assumption about catching a partial write.
- Symlink and hardlink cases both return success and overwrite the separately
  owned original JSON target. Every target is inside the unique repo-volume
  fixture; no off-volume write or actual user-file mutation occurred.
- A separate Python process acknowledges an exclusive flock on state.lock.
  StateFile::save still succeeds promptly and changes state.json. The holder is
  killed and waited for before the outcome assertion; there is no detached job.
- Missing-state load returns empty version-zero state without creating storage.

The four unique fixtures were removed by their owning RAII fixtures, including
on expected assertion failures. Result consumed; target/cli-state-controls has
no state-* or sync-probe-* residue. The log/exit remain as owned evidence.

## Actual filesystem primitive probe

The fixed local probe creates one uniquely named directory and one file on
repository device 16777244, writes its owned bytes and invokes file fsync,
file F_FULLFSYNC, directory fsync and directory F_FULLFSYNC. All four return
success on this macOS host. Exact file/directory removal is verified; probe
result consumed. This demonstrates API availability on this volume, not physical
power-cut survival, disk-failure recovery or other-platform qualification.

Pinned Rustix source fs/fd.rs explicitly distinguishes Apple fullfsync from fsync;
fs/fcntl_apple.rs exposes the safe wrapper. Source fs/at.rs documents descriptor-
relative opens/renames; FlockOperation::NonBlockingLockExclusive is available.
The following primary references were also read:

- [Linux fsync manual](https://man7.org/linux/man-pages/man2/fsync.2.html): file synchronization requires a separate directory synchronization for its directory entry.
- [Linux rename manual](https://man7.org/linux/man-pages/man2/rename.2.html): atomic replacement preserves existing open descriptions; renameat supports directory descriptors. Network-filesystem failure semantics need their own qualification.
- [Linux flock manual](https://man7.org/linux/man-pages/man2/flock.2.html): nonblocking exclusive advisory locks use open-description lifetime, and do not prevent noncooperating writers.
- [Apple fcntl manual](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/fcntl.2.html): platform full-synchronization operation; corroborated by the actual four-call probe.

The docs.rs version URLs were unavailable through the browsing tool and the
Open Group rename endpoint returned 403. The exact pinned local source and the
successful primary manuals were used; no unseen page is claimed as evidence.

## Candidate and corrected qualification

The first candidate unit run returned rc=101 (7 pass / one failure, build 5m36s,
execution 1.07s). The unreserved-mode test created its alleged ambiguous file
through ordinary write under the project's umask 077; the actual file was 0600,
which is valid reserved work. Fix the fixture by explicitly setting 0644. The
first strict lint returned rc=101 for collapsible_if; collapse the conditional.
Both complete log/exit pairs were consumed after the original tool-session
handles were unavailable following context compaction. No background result is
inferred from silence. The escalated pre-commit process census returned handoff: OK; result consumed. No successful final claim relies on the earlier source revision.

Source review also improves replaced-lock diagnostics before validating the
unlinked original inode, rejects special bits on reserved work, scopes the new
runtime suite to the implemented Linux/macOS targets, and gives the excessive-
path case an exact fixture-local absence assertion. Normalize only the existing
CLI live harness's manifest-derived repository root; its two fixed fixture names
and cleanup remain unchanged. This unit does not claim a new live HTTP run.

Final command: `python3 -B scripts/project_env.py cargo test --locked -p reasonbraid-cli --lib --test state_publication -- --nocapture --test-threads=1`.
Result rc=0, 8 library controls plus 4 publication controls; build 13.90s,
execution 0.75s + 0.14s. The original three defects now pass unchanged desired
assertions, and the missing-state read still creates nothing. Final strict lint and book verification also pass, as recorded below.

Five new library controls exercise twelve malformed-state cases with exact
byte preservation, empty legacy version zero/canonical legacy UUID retention,
8 MiB read/output bounds, five injected publication checkpoints and later
reserved-work recovery, four ambiguous working-file types/modes, replaced lock
identity, path traversal/link/legacy ambiguity and depth refusal. The other
three library controls preserve principal/tenant lookup and JSON round trips.
Faults are injected at deterministic internal phase boundaries, not actual
power cuts, physical disk failures or arbitrary network-filesystem behavior.
The separate Python holder acknowledges its flock before the attempted save;
SIGKILL and wait establish release and successful subsequent publication.

## Following work

The StateFile primitive is implemented. Then integrate both actual HTTP state writers under
one held lock in .3.1.2; pending bootstrap requests and interruption/restart remain
the following recovery children. Do not weaken assertions or commit them as an
ignored desired feature. Baseline and mechanism evidence remain durable here.

Baseline log SHA-256: `3aad34692797238c5e6c87849d13d1359e27198420b93266d7ee9f06a68aa232`.
Filesystem probe JSON SHA-256: `3d1f8f056c01600cffdfb5ba5547bf25a9271ac109e7be440083155823768ed7`.

## Final verification

`python3 -B scripts/project_env.py cargo clippy --locked -p reasonbraid-cli --all-targets -- -D warnings`
returns rc=0 in 2m22s. Final test/lint orchestration result is consumed. No PostgreSQL
cluster was started for this storage-only unit. `cargo fmt --all --check`,
`git diff --check`, `make book`, twelve rendered storage markers, the CLI chapter
link and one parsed JSON example pass. Unique state-*, unit-* and sync-probe-*
fixtures are absent. README remains 52 lines/2054 bytes. LIVE_STATUS category
values are unchanged; its corrective evidence and the roadmap/book frontier
advance to whole-writer integration. No full CI or push occurs in this leaf.

Retained log SHA-256 values (all under target/cli-state-controls):

- candidate.log: `27de73757274df8ca587930561abb4e42c542f0b57587849ae2ef8a62d4fa146` (2833 bytes).
- candidate.exit: `39b8dc3fc8b44765c8e6f1adee04c5b465e555ab791cc42d0d9e810d5b64297c` (4 bytes).
- lint.log: `43900987a7f566742431fb0fa7db53306c8f601505ebb5f44515c8d5f18dfcbc` (1401 bytes).
- lint.exit: `39b8dc3fc8b44765c8e6f1adee04c5b465e555ab791cc42d0d9e810d5b64297c` (4 bytes).
- final.log: `345b0d2448122e23cca9dc7b58eeceaa8cc6fff807280f9fafb145e91011db02` (1806 bytes).
- final.exit: `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa` (2 bytes).
- final-lint.log: `800abc0b4310d9e219dfe9e79252b8b0c8ba2119cb30bc325a92e23d4cdfbf2d` (1192 bytes).
- final-lint.exit: `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa` (2 bytes).

Final escalated `python3 -B scripts/project_env.py bash scripts/check_no_background_jobs.sh` returns rc=0: no project-owned background job; three session processes have only inherited CWD and zero repository handles. Its result is consumed before the commit.
