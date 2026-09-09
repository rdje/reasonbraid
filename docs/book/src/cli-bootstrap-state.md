# Bootstrap recovery records

The version-two storage schema is implemented. Keyed CLI enrollment and the
explicit `--resume-bootstrap` operation remain planned in the next slice; the
current CLI does not yet emit these recovery records or send a bootstrap key.
Ordinary writers already refuse a stored pending request before HTTP.

## Versions and compatibility

Version zero remains the empty legacy snapshot. Version one retains only version,
principals and threads. Their serialized shapes remain unchanged: neither gains
a null bootstrap field. Version two requires a nonempty bootstrap recovery record.
Older qualified clients refuse this version instead of ignoring pending intent.

Rust StateFile callers gain the optional bootstrap field. Existing construction
with `..StateFile::default()` continues to work. The public BootstrapRecovery,
BootstrapRequest, CompletedBootstrap and BootstrapOutcome types describe the data;
StateFile load/save perform the complete storage validation. These records are
neither credentials nor authenticated proof of server authority.

## Pending request

A snapshot before dispatch retains the request identity while preserving any
existing principal and thread maps. This minimal example has no earlier maps:

```json
{
  "version": 2,
  "principals": {},
  "threads": {},
  "bootstrap": {
    "pending": {
      "request_id": "req_00000000-0000-7000-8000-000000000001",
      "server": "http://127.0.0.1:4310",
      "name": "alice",
      "actions": null
    },
    "completed": null
  }
}
```

The request describes human enrollment without an existing tenant. Its request_id
must be canonical lowercase, hyphenated, req_-prefixed RFC UUIDv7. The server base
is a canonical absolute HTTP(S) URL of at most 4096 bytes, without URL credentials,
query or fragment. Root/trailing slashes and default ports use the canonical base
representation. An endpoint string does not prove that a replacement database
at that address is the same authoritative store.

Names remain exact; no normalization or inference of identity from equal names
occurs. The nullable actions field retains original input for stable resends,
although the current server deliberately ignores human action input. Nullable
fields must be present: omitting actions, pending or completed is malformed
recovery data, rather than an invitation to guess what was intended.

The whole state file still has the 8 MiB encoded/read bound. Unknown versions,
unknown/duplicate fields, non-object records, malformed identities, inconsistent
source references and conflicting bindings refuse without erasing stored bytes.
Canonical older UUID versions remain valid human/tenant source identities; only
the client request key is specifically required to be UUIDv7.

## Completed receipt

After the principal/outcome is published and pending cleanup completes, the most
recent completed request remains available. A corresponding snapshot is:

```json
{
  "version": 2,
  "principals": {
    "alice": {
      "kind": "human",
      "id": "hpr_00000000-0000-7000-8000-000000000002",
      "tenant": "ten_00000000-0000-7000-8000-000000000003"
    }
  },
  "threads": {},
  "bootstrap": {
    "pending": null,
    "completed": {
      "request": {
        "request_id": "req_00000000-0000-7000-8000-000000000001",
        "server": "http://127.0.0.1:4310",
        "name": "alice",
        "actions": null
      },
      "outcome": {
        "bootstrap_request_id": "req_00000000-0000-7000-8000-000000000001",
        "kind": "human",
        "name": "alice",
        "principal_id": "hpr_00000000-0000-7000-8000-000000000002",
        "tenant_id": "ten_00000000-0000-7000-8000-000000000003",
        "boundary_id": "bnd_ten_00000000-0000-7000-8000-000000000003",
        "grant_id": "grt_hpr_00000000-0000-7000-8000-000000000002",
        "replayed": false
      }
    }
  }
}
```

Principal and tenant IDs come from the server independently of the client request
key. Every outcome field is required. Its key, human kind and name must match the
saved request; principal/tenant IDs and their boundary/grant references must
agree. The replayed value records the returned response and may be true. The
receipt is historical: later legitimate local mapping changes need not erase it.
Only one most recent completed receipt is retained, alongside at most one pending
request; this is not an unbounded client history.

## Publication rules

The same writer guard can publish multiple synchronized snapshots without
releasing exclusion. The planned HTTP flow uses these already qualified storage
transitions:

| Snapshot | Pending | Completed receipt | Required local principal |
| --- | --- | --- | --- |
| Before HTTP | New request | Previous receipt, if any | Existing maps preserved |
| Outcome publication | Same request retained | Matching checked outcome | Matching human/tenant mapping published |
| Pending cleanup | Cleared | Matching receipt retained | Matching mapping still present |

A later save cannot silently discard recovery metadata, replace an unresolved
request, or replace the prior completed receipt with unrelated data. Clearing
pending requires its matching completed outcome and principal mapping in the
published snapshot. A valid first snapshot may restore recorded metadata into an
empty/legacy store; that validates local data rather than authenticating a server.

A failure before replacement preserves the earlier snapshot. A failure after
replacement is attempted reports unconfirmed durability; the same guard remains
held until its owner finishes or is dropped. If pending became visible before
an interrupted acknowledgment, the key remains recoverable and ordinary writers
refuse it. No separate unsynchronized pending-file deletion is used.

## Planned recovery intent

After an interrupted request, matching pending work will reuse its saved key.
A different name/server will refuse while pending remains. A normally fresh
invocation after completion will still mean a distinct new tenant. Explicit
`--resume-bootstrap` will select recovery instead, including from the retained
completed receipt when CLI output was lost after local cleanup.

File publication cannot establish whether a person consumed stdout. The explicit
recovery intent handles that ambiguity without treating equal names as proof of
the same logical operation. The next CLI and deadline slices must qualify the
actual HTTP/reply/output behavior before this becomes a completed client-recovery
feature. Local process death or lock release never establishes server rollback.
