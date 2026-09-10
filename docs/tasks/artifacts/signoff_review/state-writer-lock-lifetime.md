# State-writer lock lifetime diagnosis

Owner: `SIGNOFF-REPAIR.11.4.3.1.2.11.1`; REPAIR-0055. Source:
`8d1504dc38a5ce3a43888a6716f9d338d7dac131`. Raw checkpoint:
`target/checkpoint-ci/full-8d1504d`; controlled evidence:
`target/state-writer-lock-controls`. Production repair is .2.11.2.

## Original checkpoint disposition

Eight completed gates pass: format, strict full-workspace Clippy, binary build,
67 Python controls, book, thirteen doctrines, pinned cargo-deny and pinned Gitleaks.
The workspace gate returns 101 after 1695.811018 seconds. Its nineteen result
blocks contain 112 nominal passes and one failure; five CLI database tests print
the unset-DATABASE_URL skip notice and are not live database passes. All sixteen
browser controls pass. The wrapper retains its failed-run payload. PostgreSQL
and the local demonstration never start; no full-pass or push claim follows.

The twelve-test state_writers suite passes eleven and fails
explicit_principal_entrypoint_obeys_exclusion_and_releases_usage_failures at line
685. Its initial independent flock returns WouldBlock/OS35 before it calls the
entrypoint under test. Fixture::new has already saved the unique directory's
state. Existing Fixture::drop deletes that directory on panic, so the original
lock holder, descriptor and spawn interval cannot be recovered. No later probe
can retrospectively supply that missing observation.

Both state_writers.rs and state_store.rs match the preceding b0cddfe checkpoint.
All 341 non-Markdown hashes match the selected source. Exact original test/rb
binaries and both sources are copied and hash-verified before building a probe:

| Preserved input | SHA-256 |
| --- | --- |
| state_writers.rs | f1cc9b5a94252278dc399b9edbbdeaf846ff3acb83539be958f85e60ca4ee97b |
| state_store.rs | 303534a448ffb5923594994c7aa7ca968ecb55b69e9635142694a5008b7026d8 |
| state_writers executable | f98dfbd23af9ee7eabf7ac8b164a859fe45aebd89a9fae96ca88081b924a5824 |
| rb executable | 42370bd30eebf20dcb56880d87ef08d65e53a491286d873fd552b6baff9524b2 |

An unchanged default-concurrent rerun passes all twelve tests (4.496454 seconds),
the isolated failed test passes (0.258266), and a serial rerun passes twelve
(7.372430). These are 25 executions of twelve distinct controls. They neither
reproduce nor invalidate the intermittent original failure. Serial execution is
a diagnostic comparison, not a proposed workaround.

## Controlled actual-library reproduction

The separate on-volume probe links the unchanged reasonbraid-cli public API.
All 139 dependency package identities/checksums match the workspace; feature
selection differs. A gated owned loopback HTTP fixture lets run_thread_create
acquire its real Publication and reach dispatch. Locate that exact descriptor
with fstat, comparing its device/inode to state.lock, and require exactly one
match. A forked child witnesses FD_CLOEXEC on that inherited descriptor and waits
on a pipe; its branch uses only async-signal-safe libc operations and _exit.
The parent completes the request, receives an HTTP error, or drops the future.
An independently opened descriptor then probes exclusion. Release/reap the child
and repeat the independent probe. The supervisor bounds and consumes the process
lifetime. All six owned state snapshots remain as diagnosis evidence.

| Operation outcome | No child: lock after operation | Inherited child alive | After child exit | State |
| --- | --- | --- | --- | --- |
| Success | available | WouldBlock/35 | available | new thread published |
| HTTP error | available | WouldBlock/35 | available | original bytes unchanged |
| Future cancellation | available | WouldBlock/35 | available | original bytes unchanged |

All six cases observe valid exclusion while the actual writer is active. All
three inherited cases verify the parent's descriptor has closed while exclusion
persists; all three children exit zero and are reaped. The corrected probe
returns zero in 0.868513 seconds because these expected diagnostic observations
match. That is a reproduced product defect, not a green product acceptance test.
Its first build takes 400.223941 seconds; the observer-only rebuild takes 6.843695.

Preserve the preliminary observer failure: /dev/fd pathname metadata finds no
backing-file match on this host. That run returns 101 before fork. The corrected
observer uses fstat on open descriptor numbers; v1 source/log/fixture are retained.
The first checkpoint verifier also mistook thirteen historical evidence files
for fixture directories. Restricting its assertion to writer-* directories fixes
that diagnostic error without deleting the historical records.

## Cause, falsification and remaining boundary

Directory::lock acquires flock on a File; Publication owns it without an explicit
unlock. Successful completion, error and cancellation close the parent's File,
but close alone cannot remove the shared lock while another reference survives.
The controlled child supplies that reference. Removing the child from the same
experiment, and later exiting the child, independently falsify an ordinary
active-writer or pathname-collision explanation for the controlled retention.

The installed Apple SDK flock/fork manuals agree with
[Apple's flock reference](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/flock.2.html):
fork/dup share a lock reference. FD_CLOEXEC controls exec, not the earlier fork
interval. This mechanism is proved against the actual unchanged API on native
macOS. It is consistent with the original concurrent failure, whose precise
holder and spawn implementation remain unobserved. Do not claim a captured
original causal trace, a Linux runtime test or a process-crash reproduction.

The repair must own an explicit-release guard immediately after successful
acquisition, including subsequent validation failures; retaining a guard solely
in Publication would miss those earlier error paths. Preserve nonblocking
exclusion, close-on-exec, snapshot semantics and existing valid-contention tests.
Do not hide retention with sleeps, retries or blanket test serialization.
A destructor cannot run after SIGKILL or a crash: surviving inherited references
remain a separate process-loss boundary, now explicitly owned by the existing
broader CLI interruption/restart leaf .3.3.4.3.3.3.3.3. Permanent release controls
and the corrected implementation belong to the immediate .2.11.2 leaf.

## Verification and durability

Independent verification returns zero: all 341 source hashes and original binary
copies agree, all six outcomes/snapshots agree, and all fifty recorded checkpoint
and diagnostic groups are absent. No writer-* transient fixture remains. Retain
the six completed probe fixtures, preliminary failed fixture and original failed
checkpoint evidence. The historical proof is these recorded identities, matched
observations and procedure; future durability requires the permanent adversarial
regressions in .2.11.2, not dependence on ignored raw probe files. Book/rendered
checks and commit doctrine results are recorded in the owning leaf. The full
checkpoint, production repair, broader restart qualification and remote CI remain
incomplete at this diagnostic commit.

Subsequent correction: REPAIR-0056/.2.11.2 qualifies explicit release and permanent
controls in docs/tasks/artifacts/signoff_review/state-writer-lock-release.md.
The historical observations and attribution limits above remain unchanged.
