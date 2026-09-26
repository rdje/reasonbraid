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

Each writer checks the server's reply against its own request before it touches
the store. An enrollment reply must name the kind, name and tenant that were
asked for, and a canonical principal ID of that kind. A thread-creation reply
must carry a canonical thread ID. A reply that does not answer the request is
the server's error and is refused as one:

```text
$ rb enroll role reviewer --tenant ten_… --json
error: malformed server response: the enrollment reply names another tenant than the one asked for
```

Nothing is recorded, and the store keeps its earlier bytes. The server may still
have acted on the request. An enrollment repeated with the same tenant, kind and
name returns the original principal, but a repeated thread creation creates
another thread, so inspect the tenant's threads before you retry one.

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

## The configured endpoint is checked

Every verb canonicalises `--server` / `REASONBRAID_SERVER` before opening a
socket, using the same check the bootstrap path has always applied: an absolute
HTTP(S) URL, at most 4096 bytes, with **no URL credentials, query or fragment**.

This is not a cosmetic tidy-up. A base carrying userinfo is not ignored by the
transport — it is sent as Basic credentials, and a control measured exactly
that on the wire before the check was added. A base with credentials, a query
or a fragment is refused, and the refusal does not echo the credential it
refused.

A live configured base is NORMALISED (an uppercase scheme is accepted), while a
bootstrap record's stored server identity must already be canonical. Those are
deliberately different: one is configuration input, the other is a durable
binding that a later recovery compares against.

## New-human bootstrap

A human enrollment without `--tenant` saves its request key before sending it,
validates the complete keyed reply, publishes the principal and receipt, then
clears the pending request under the same lock. After that cleanup,
`--resume-bootstrap` recovers the saved result if the output was lost, and a
normal invocation with nothing pending creates another tenant. Before any of
this, the CLI checks that the completed receipt will fit the store's 8 MiB limit
and refuses before sending if it would not. That checks the format limit, not
disk space. [Bootstrap recovery records](cli-bootstrap-state.md) has the
examples. A released lock after a failure is not evidence that repeating an
unkeyed request is safe.

## A child process does not keep the store locked

The writer's guard unlocks explicitly before it closes the lock file, on
success, on an error right after acquisition, on discard and on unwind. A child
process that inherited the lock's descriptor therefore does not keep the store
locked once the writer is done. For example, an embedding application can
finish or cancel a run_thread_create call and start another writer while an
unrelated forked child is still running. Overlapping writers still fail
promptly, and close-on-exec and complete snapshot synchronization are unchanged.
A permanent control keeps the real lock description in a child across all five
paths, and checks that a successor stays exclusive when the old child exits.

Abrupt process death runs no release code, so a child that outlives a killed
writer can still hold the lock; that case is not yet qualified. Never delete
state.lock to force progress, and never infer from local contention that the
server rolled back.
