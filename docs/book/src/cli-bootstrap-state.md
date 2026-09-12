# Bootstrap recovery records

Human enrollment without `--tenant` now saves a version-two recovery record before
sending a keyed bootstrap request. Matching pending work reuses that request;
`--resume-bootstrap` also recovers the most recent completed receipt when output
was lost. Other state writers refuse unresolved pending work before HTTP.
Thirty-three selected controls, the final output-window rerun and strict CLI lint
pass. All results/shutdown are consumed and unique fixtures/owned cluster absent.
Completion-capacity preflight is also qualified below. HTTP deadlines and broader
interruption qualification remain separately owned.

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
releasing exclusion. The keyed HTTP flow uses these storage transitions:

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

## Choosing fresh enrollment or recovery

Use the same repository-relative state directory and configured server for every
step of one operation:

```sh
export REASONBRAID_CLI_STATE=target/rb-state
rb --server http://127.0.0.1:4310 enroll human alice --json
```

Before that HTTP request, the CLI synchronizes a canonical request key, endpoint,
exact name and original action input. If the request fails or the process exits,
repeating the command while a matching pending request exists reuses that exact
request. A changed name or endpoint refuses before HTTP. New ignored human action
arguments do not replace the original saved action input. No automatically
invented replacement key follows a server, transport or malformed-reply error.

After pending cleanup, a normal invocation is intentionally fresh and creates a
new tenant, even if the name matches the most recent completion. If the earlier
output was lost, select recovery explicitly:

```sh
rb --server http://127.0.0.1:4310 enroll human alice --resume-bootstrap --json
```

This operation requires human enrollment without `--tenant`. Missing or mismatched
recovery refuses; it never falls through to creation. Only one most recent
completed request is retained, so a later completed bootstrap replaces the older
receipt available for this command. Use the same state directory; another store
does not possess this operation's recovery record.

A pending request with no completed outcome sends the saved key to the configured
server. If its completed receipt is already present, recovery restores that
historical principal mapping and reports the saved outcome locally without HTTP.
This also finishes cleanup when completion was published before an interruption.
An explicit resume after completed cleanup likewise uses the local receipt.

JSON output includes `recovery_source: "server"` for a checked HTTP outcome or
`recovery_source: "local_receipt"` for a retained local outcome. The original
`replayed` value stays unchanged during local recovery. For example:

```json
{
  "bootstrap_request_id": "req_00000000-0000-7000-8000-000000000001",
  "kind": "human",
  "name": "alice",
  "principal_id": "hpr_00000000-0000-7000-8000-000000000002",
  "tenant_id": "ten_00000000-0000-7000-8000-000000000003",
  "boundary_id": "bnd_ten_00000000-0000-7000-8000-000000000003",
  "grant_id": "grt_hpr_00000000-0000-7000-8000-000000000002",
  "replayed": false,
  "recovery_source": "local_receipt"
}
```

Human output says "recovered historical enrollment of" for local recovery.
A saved outcome does not prove current authority, current remote existence or
continuity of the database at that URL. It is not an authenticated credential.
File publication cannot establish whether a person or consuming process received
stdout; the explicit operation handles that ambiguity without inferring intent
from equal names. Local process death or lock release never establishes server
rollback.

The CLI requires the keyed server protocol and its complete matching reply.
Missing, duplicate or unknown fields, non-object JSON, wrong request identity and
inconsistent source references refuse, preserving pending intent. There is no
silent fallback to an unkeyed request against an older server. The public
run_enroll convenience function follows normal invocation semantics;
run_enroll_with_recovery exposes the explicit resume choice to Rust callers.

HTTP connect, whole-request and reply-size bounds are implemented; see
[bounded transport](cli-state.md#bounded-transport) for their values and for
what a refusal preserves. A timed-out or oversized-reply attempt keeps its
original request key and releases the store, so repeating the command resumes
the same logical bootstrap. Broader process/filesystem/server restart
qualification remains open; this flow does not claim universal automatic retry
or physical power-loss survival.

Completion-capacity preflight is implemented under
SIGNOFF-REPAIR.3.3.4.3.3.3.3.2.3.1: thirty-one selected controls, the final boundary
matrix and strict CLI lint pass; all results consumed and fixtures absent. Before
publishing pending or sending HTTP, the CLI validates that the complete encoded
principal/receipt snapshot fits the 8 MiB limit. A near-limit pending snapshot
alone is insufficient. Refusal preserves the original snapshot and any pending
key without dispatch. The sizing sample stays private memory; only an actual
checked outcome or a saved historical receipt can be published or reported.
This checks the format limit, not physical disk reservation or later write success.

For example, a store whose pending request fits but whose completed receipt would
be one byte over the limit refuses before sending the creation request. A
completion exactly at the limit is admitted when every other storage check
passes. Preserve the refused state and resolve its capacity deliberately; do not
switch to an empty state directory or invent another request key to bypass an
unresolved operation. Transport and later publication failures still require
the existing matching recovery behavior.

A size refusal reports `bootstrap completion preflight refused before HTTP` for
this invocation. A previously saved pending request can still represent an earlier
uncertain server attempt; the error does not declare that earlier attempt rolled
back. Both fresh-state and existing-pending size refusals preserve exact bytes.
