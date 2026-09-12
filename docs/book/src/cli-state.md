# CLI local state and recovery

The CLI stores principal names and thread-to-tenant mappings in state.json.
This is local convenience state; the server remains authoritative for tenants,
principals, threads and request outcomes. A server response and successful local
publication are separate events.

## Selecting the store

Run rb from within this repository. The default directory is .reasonbraid-cli
below the current repository root, including when the command is launched from
a subdirectory. Override it with a repository-relative path:

```sh
REASONBRAID_CLI_STATE=target/rb-state rb enroll human alice --json
REASONBRAID_CLI_STATE=target/rb-state rb thread create --subject "Review" --objective "Decide with evidence" --as alice
```

Both commands select the same directory from any repository subdirectory, unless
an ambiguous legacy CWD-relative directory is present. Older releases interpreted
relative state paths from the working directory. If that old location exists
separately, the new store refuses rather than silently abandoning its contents.
Run from the repository root and explicitly select the intended relative path.
No automatic migration or deletion of the old directory occurs.

Runtime absolute paths below the current repository are accepted, but do not
persist a checkout-specific absolute path in configuration. Parent traversal,
symlinked directories, paths outside the repository, the repository root itself
and paths longer than 64 components refuse. The store checks the repository
volume and effective ownership for every opened directory/file. Group/other
writable objects, linked files and nonregular files refuse. New directories use
0700 and new files 0600; no home-cache or temporary-directory fallback exists.
Reading a missing state returns an empty mapping without creating storage.

The storage implementation currently supports Linux and macOS. This repair's
native runtime controls run on the macOS repository volume; other platforms
refuse explicitly. Arbitrary network filesystems and physical power-cut survival
have no qualification claim here.

## Valid snapshots

Version two adds the implemented [bootstrap recovery record schema](cli-bootstrap-state.md).
New-human enrollment emits those records before HTTP; matching pending work reuses
its key and other writers refuse. Version-zero/version-one wire shapes remain unchanged.

A version-one snapshot looks like this:

```json
{
  "version": 1,
  "principals": {
    "alice": {
      "kind": "human",
      "id": "hpr_00000000-0000-7000-8000-000000000001",
      "tenant": "ten_00000000-0000-7000-8000-000000000001"
    }
  },
  "threads": {
    "thr_00000000-0000-7000-8000-000000000001": {
      "tenant_id": "ten_00000000-0000-7000-8000-000000000001",
      "subject": "Review"
    }
  }
}
```

Principal kinds are human or role, and identifiers must match their canonical
hpr_/rol_/ten_/thr_ families. Older canonical UUID versions remain valid identities.
Empty version-zero snapshots remain readable. The maximum encoded file is 8 MiB.
Unknown versions or fields, duplicate names/fields, malformed identities and
non-object records refuse. A valid save also refuses to erase an invalid current
file. Preserve such evidence and resolve the inconsistency before proceeding;
the CLI does not silently reset it to empty state.

## Publication and interruptions

A save acquires a nonblocking operating-system lock on the reserved state.lock
file. Another live writer causes an error. The guard explicitly unlocks when its
operation ends, then closes its File. The filename normally remains. After abrupt
process death, an inherited descriptor can retain exclusion until its last shared
reference closes; the normal release destructor cannot run then. Never delete state.lock to bypass contention.
The lock coordinates cooperating clients, and the directories must remain
stable during use.

The store writes a complete private state.json.next, synchronizes its contents,
atomically replaces state.json, and synchronizes the directory before reporting
success. macOS also uses F_FULLFSYNC after fsync. Readers that already opened the
old file retain that complete snapshot; later opens see the replacement.

Before replacement, an error leaves the old state authoritative and may leave
reserved work. Once replacement is attempted, an error explicitly reports
`local state replacement/durability is unconfirmed`: either old or new state may
need reconciliation. Never infer server rollback from a local storage error.

On a later write, the lock holder can remove an interrupted state.json.next only
when it is an owned, same-volume, single-link regular file, exactly mode 0600
without special permission bits, and at most 8 MiB. Linked, oversized, directory
or otherwise ambiguous objects remain untouched and cause refusal. These two
reserved names belong to the store; keep unrelated files elsewhere. Working
bytes are never recovered as authoritative state merely because they exist.

## CLI writer lifetime

Enrollment and thread creation acquire the same nonblocking state lock and
validate the current snapshot before dispatching HTTP. A busy or invalid store
therefore refuses before sending an effect request. The lock stays held across
the response, the fresh state update and synchronized publication. An overlapping
writer fails promptly; run it again only after the first command finishes and
you understand any reported server uncertainty.

Named thread creation resolves `--as` from the locked snapshot. An explicit
`--tenant` overrides the command's tenant without changing the principal's saved
tenant. Both writers preserve unrelated principals and threads when adding their
own result. For example, enrollment of bob followed by a thread creation still
retains alice and all previously stored thread mappings, whichever writer ran
first.

The library's run_thread_create entrypoint still accepts an explicitly supplied
PrincipalRef and uses it as supplied. The CLI uses run_thread_create_named for
fresh name resolution. Both entrypoints share the held-store implementation.
StateFile::save remains a full-snapshot replacement API: callers that independently
load and later save must not assume it merges a stale snapshot for them. A save
also refuses transitions that discard recovery metadata or replace an unresolved
bootstrap identity.

A failed request, local validation error or future cancellation drops the writer,
explicitly releasing exclusion without publishing further in-memory changes. An
inherited child descriptor no longer delays this normal release. New-human bootstrap
already published its pending identity before HTTP and retains it on failure. A local publication error
still follows the before/after-replacement rules above. Process death while
awaiting HTTP preserves the previously published snapshot; it does not prove
that the server did nothing. Never delete the lock filename to force progress.

## Bounded transport

The CLI's HTTP client has explicit bounds, and they exist because the lock
above is held across the response: a peer that accepts a connection and never
answers would otherwise hold it for as long as the process lives.

| Bound | Value | What it is for |
| --- | --- | --- |
| Connect | 10 s | a peer that neither accepts nor refuses a dial |
| Whole request | 60 s | a peer that accepts and never answers; covers the response body |
| Reply size | 8 MiB | matched to the local-state limit, so a reply the store could never hold is refused instead of buffered |

The whole-request ceiling is deliberately well above the server's own
15-second whole-operation budget. A legitimate request that waits behind a
tenant guard is never cut by it — the bound exists for a peer that never
answers, not to second-guess a server that is working.

A refusal preserves what recovery needs. The pending request keeps its
ORIGINAL key, the store's exclusion is released, and repeating the command
reuses that key rather than minting a new one: a new key would be a second
logical bootstrap against a server that may already hold the first. A timeout
says nothing about whether the server committed, so nothing here infers a
rollback from it.

An oversized reply is a named refusal rather than a truncation — a prefix of a
JSON outcome is not an outcome — and it too retains the pending key.

Redirects are **not followed**. The server base is configured by flag or
environment, and a redirect would carry a request bearing the development
principal header, and a bootstrap request key, to a host the operator never
named. A 3xx from the configured endpoint is reported as the server response
it is.

The CLI now persists/sends bootstrap_request_id before new-human HTTP enrollment,
validates a complete keyed reply, publishes its principal and receipt, then clears
pending under the same lock. After cleanup, use `--resume-bootstrap` to recover
the retained historical result if output was lost; a normal no-pending invocation
intentionally creates another tenant. See the
[request and recovery examples](cli-bootstrap-state.md).
Broader interruption/restart qualification remains open. Successful lock release
alone is not evidence that repeating an unkeyed request is safe.

Completion-capacity preflight is implemented under
SIGNOFF-REPAIR.3.3.4.3.3.3.3.2.3.1: thirty-one selected controls, the final boundary
matrix and strict CLI lint pass; all results consumed and fixtures absent. Before
publishing pending or sending HTTP, the CLI validates that the complete encoded
principal/receipt snapshot fits the 8 MiB limit. A near-limit pending snapshot
alone is insufficient. Refusal preserves the original snapshot and any pending
key without dispatch. The sizing sample stays private memory; only an actual
checked outcome or a saved historical receipt can be published or reported.
This checks the format limit, not physical disk reservation or later write success.

## Inherited-descriptor qualification

The original close-only guard retained exclusion when a child inherited its lock
descriptor. The guard now explicitly unlocks before File close, including errors
immediately after acquisition. For example, an embedding application can finish
or cancel a run_thread_create call and start another writer while an unrelated
forked child still holds the old description. Valid overlapping writers continue
to fail promptly; close-on-exec and complete snapshot synchronization stay intact.

All 32 distinct CLI tests pass under default concurrency. A permanent control
retains the real lock description in a child across success, encoding error,
publication error, discard and unwind; it fails all five paths on unchanged
production and passes after repair. It also verifies that a successor stays
exclusive when the old child exits. Six independent raw-fork public-API scenarios
confirm immediate release after success, HTTP error and future cancellation,
with unchanged failure/cancellation snapshots. Strict CLI lint and format pass.

The original concurrent checkpoint failure did not preserve its holder, so these
controls do not establish its exact historical spawn path. Abrupt process death
does not run a release destructor; surviving inherited references need separate
restart qualification, concretely owned by the broader interruption/restart leaf.
Never unlink state.lock or infer server rollback from local contention. Evidence:
`docs/tasks/artifacts/signoff_review/state-writer-lock-release.md`.
