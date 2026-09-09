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
file. Another live writer causes an error. Process exit releases the lock; the
filename normally remains. Never delete state.lock to bypass contention.
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

## Current integration limit

The StateFile library's save operation replaces the full supplied snapshot.
A separate load, modification and save can still lose another client's update.
Enrollment and thread creation currently call HTTP before their local load/save;
the next owned integration holds one guard across that whole workflow. Until it
lands, atomic publication alone does not make concurrent CLI updates safe.

The CLI also does not yet persist/send bootstrap_request_id before new-human
HTTP enrollment. A lost bootstrap response or local write failure can therefore
leave a created tenant without a recovered local identity. Repeating the human
name without --tenant can create a distinct tenant. The server's keyed recovery
API is implemented; durable CLI request handling and interruption/restart
qualification are the following repair children.
