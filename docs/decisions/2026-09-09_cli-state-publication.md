---
answers:
  - How does CLI state remain on the current repository volume?
  - What does a successful StateFile save establish about publication?
  - How are interrupted local state writes and ambiguous working files handled?
  - Why does atomic file replacement still require a lock across a CLI update?
  - When does the CLI resolve a named actor and release writer exclusion?
---
# Publish bounded CLI snapshots under an operating-system lock

- Owner: `SIGNOFF-REPAIR.3.3.4.3.3.3.3.1.1` (storage API); `.1.2` implements complete CLI writer integration.
- Status: StateFile storage API qualified by twelve selected controls, all-target CLI strict lint and book verification. Whole writer integration passes nineteen selected controls (seventeen library/storage/writer controls and two live CLI compatibility controls), final all-target strict lint and book checks; results/shutdown consumed and fixtures/cluster absent. Pending request/recovery remains the following children.
- Evidence: docs/tasks/artifacts/signoff_review/cli-state-publication.md.

## Scope and repository binding

StateFile::load/save use a private store module. Discover the current repository
from the working directory and its ancestors using the workspace, CLI manifest
and migrations markers. Resolve relative paths from that root. Runtime absolute
input is accepted only below that same root; parent traversal, the root itself
and more than 64 path components refuse. No path is persisted as a checkout-
specific absolute value. Config preserves non-Unicode environment paths instead
of silently substituting a default.

Open directories descriptor-relatively with no symlink following. Each directory
and state/lock/working-file descriptor must be on the repository device, owned
by the effective user and not writable by group or others. Files must be regular
and single-link. New directories use mode 0700, new files 0600, subject to the
caller making permissions still more restrictive. No home or temporary fallback
exists. A relative path whose former CWD-relative directory exists separately
refuses as ambiguous; select the intended repository-relative directory from
the root. Missing-state reads create nothing.

The current implementation supports Linux/macOS APIs and explicitly refuses
other platforms. Native runtime evidence is macOS on the actual repository
volume. Read-only installed toolchain/OS dependencies are necessary exceptions
to project-data locality. This is not qualification of arbitrary network storage,
physical power interruption or a hostile process running as the same user.
Directories must remain stable while the store is in use.

## State format and bounds

The encoded/read file limit is 8 MiB; read length is checked before and during
reading, and serialization refuses before extending beyond the bound. Version
zero is valid only with empty maps; version one preserves principals and threads.
Version two adds the separately qualified recovery metadata in
docs/decisions/2026-09-09_cli-bootstrap-state.md; legacy zero/one wire shapes
remain unchanged. Unknown versions/fields, duplicate fields/map keys, sequence-shaped records,
wrong principal kinds and malformed/noncanonical typed identities refuse. Older
canonical UUID identity versions remain supported; UUIDv7 is a separate new
bootstrap request-key requirement. Invalid existing bytes are preserved even
when the caller tries to save an otherwise valid replacement.

## Lock, publication and error phase

state.lock is a stable reserved file. An exclusive nonblocking advisory flock
lives with a close-on-exec descriptor; contention refuses. Process exit releases
the lock, so the existence of the filename is never evidence of a live writer.
Never remove a lock file as a stale-lock recovery mechanism. Check the held and
current lock identities before publication; an unlinked/replaced inode refuses.

Under the lock, validate the existing state before replacing it. state.json.next
is the reserved working name, never a source of truth. An existing working file
is removable only when it is an owned, same-volume, regular single-link file of
mode exactly 0600 (including no special mode bits) within the 8 MiB bound. Other
objects are preserved and refused. This reservation belongs exclusively to the
store; do not put unrelated data at either reserved filename.

Create the working file exclusively, write the complete validated snapshot,
synchronize the file, rename it over state.json in the same directory, then
synchronize the directory before acknowledging success. Directory creation and
reserved-work cleanup also synchronize their parent entries. Synchronization
uses fsync, followed on macOS by the safe Rustix F_FULLFSYNC wrapper. The already
pinned Rustix 1.1.4 supplies safe filesystem/process APIs; no raw unsafe application
code or new downloaded package is introduced.

Failures before the replacement call preserve the original published snapshot
and may leave the reserved working file. From the replacement call onward, an
error says replacement/durability is unconfirmed: the caller must not infer that
the old snapshot remains authoritative. Failure unwinding does not unlink a
possibly swapped pathname or delete the published state. A later lock holder
validates and cleans only the reserved work before a fresh publication. Existing
read descriptors retain the complete snapshot they originally opened.

## Whole writer transaction boundary

The private Writer owns a fresh StateFile and the Publication's directory/lock
handles. Writer::open acquires exclusion and validates state before HTTP client
construction/dispatch. Writer::publish consumes the same owner after its complete
in-memory update; the lock remains held through encoding, replacement and sync.
Drop releases local exclusion on request/validation error or future cancellation,
without publishing an incomplete update. Post-replacement errors still carry
uncertain durability.

run_enroll and both thread-creation entrypoints use that owner. The existing
run_thread_create preserves explicitly supplied PrincipalRef semantics. The new
run_thread_create_named resolves a name and optional tenant override from the
locked snapshot and then uses the same private operation. main.rs omits its
former eager read for these two writers; other verbs still take a read snapshot.
This avoids a separate pre-lock actor selection in the actual CLI.

StateFile::save remains an atomic full-snapshot convenience API and validates
encoded bounds before creating storage. It does not merge arbitrary stale caller
snapshots. The Publication helper shares descriptor/lock/publication mechanics
with the held writer, preserving all previously qualified storage boundaries.

The two real-server CLI fixtures now use unique repository-volume directories,
bounded output and explicit child kill/wait on failure/deadline, plus joined
server shutdown and pool closure on success. No previous fixed-name residue is
deleted. Normal real-process controls consume both HTTP/server and OS lock-holder
lifetimes before assertions; abort-on-drop fallbacks bound failed fixtures.

## Pending protocol ownership

Pinned Reqwest defaults to no connect/read/whole-request timeout, so a stalled
peer can keep a live writer's lock until cancellation. The pending-request child
owns a matched stalled-response control, explicit HTTP limits and key retention
on timeout. Cancellation/local lock release cannot prove remote rollback.
Durable bootstrap identity, strict complete reply recovery and restart
qualification remain the following children. No unkeyed replay-safety claim
follows from this writer integration.
