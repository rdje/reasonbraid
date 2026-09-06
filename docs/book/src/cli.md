# The CLI

Phase 0 ships a command-line surface for the vertical slice: `rb` (the
`reasonbraid-cli` crate) drives the control plane over HTTP JSON. Every state
inspection happens through this surface — **no database surgery**.

## Starting the control plane

`rb-server` applies the migrations and serves the control API plus the node
channel (dev profile, loopback):

```text
$ rb-server --database-url postgres://postgres@127.0.0.1:55432/reasonbraid
rb-server listening on http://127.0.0.1:4310 (Phase 0 dev profile)
```

## The flow

```text
$ rb enroll human alice
enrolled human `alice` as hpr_… in tenant ten_…
boundary: bnd_ten_…

$ rb enroll role reviewer --tenant ten_…
enrolled role `reviewer` as rol_… in tenant ten_…

$ rb thread create --subject "should we ship?" --objective "decide with evidence" --as alice
created thread thr_… (state: open)

$ rb thread invite --thread thr_… --agent reviewer --as alice
$ rb thread contribute --thread thr_… --text "Ship it: the experiments are green." --as reviewer
$ rb thread challenge --thread thr_… --target evt_… --text "Which experiments?" --as alice
$ rb thread revise --thread thr_… --target evt_… --text "The SQLite kill-point sweep…" --as reviewer
$ rb thread close --thread thr_… --reason "decision reached" --as alice
$ rb thread cancel --thread thr_… --reason "no longer needed" --as alice
```

The create verb also takes the typed profile fields (`.1.1.3`):

```text
$ rb thread create --subject "…" --objective "…" --as alice \
    --classification confidential \
    --workflow-profile critique-revise \
    --allow-join-requests
```

- `--classification` — `general` (default) | `confidential`.
- `--workflow-profile` — `single-agent` (**the stated default**, ADR-002's
  routing decision) | `blind-independent` | `critique-revise` | `moderator`.
- `--allow-join-requests` — off by default: explicit participants first (§20.3).

`thread.cancel` is the **abandonment terminal** (`open|closing → cancelled`,
reason recorded) — distinct from a decided close; both are inspectable, and a
cancelled thread refuses further content verbs with `invalid_transition`.

The CLI keeps a local state dir (`./.reasonbraid-cli`, or
`REASONBRAID_CLI_STATE`): names → principal ids, and the thread → tenant mapping,
so `--as alice` and `--as reviewer` resolve without retyping ids (raw `hpr_…` /
`rol_…` ids are accepted directly). The server is `http://127.0.0.1:4310` or
`REASONBRAID_SERVER`.

## What happens per command

One transaction: the idempotency claim, the authorization decision (an audit row,
allowed or denied), the domain validation against the locked thread projection,
the event + state + outbox writes — and, on create, the thread's budget ceiling.
The event order is the audit timeline; every event names its actor, and every
decision records the 64-hex policy digest it was evaluated under.

Refusals are typed and stable: `unauthorized` (denied, with the audit record id),
`invalid_transition` (the core state machine refused the move), `invalid_command`,
`idempotency_mismatch`, `scope_hidden` (the thread is not visible in this scope —
existence is never confirmed), `unauthenticated` (the dev principal header is
missing or malformed). A retried command with the same idempotency key and body
returns the ORIGINAL result — including a rejection's original status.

## Inspecting

```text
$ rb inspect thread thr_…
thread: thr_… (tenant ten_…)
subject: should we ship?
state: closed — reason: decision reached
participants:
  hpr_…: accepted
  rol_…: accepted
counters: contributions=1 revisions=1 open_challenges=0
events:
  evt_… #1 thread.created
  evt_… #2 thread.participant_invited
  evt_… #3 thread.contribution_submitted
  evt_… #4 thread.challenge_posted
  evt_… #5 thread.revision_submitted
  evt_… #6 thread.closed
audit:
  authz_… allowed thread_invite …
```

`rb inspect threads` lists a tenant's threads; `--json` prints the raw API
responses for scripting.

## Node administration

The node-side of the vertical slice is administered with one verb (`.1.2.1`):

```text
$ rb node issue-token --node rol_… --host-claim dev-host --as alice --tenant ten_… --json
{ "token_id": "ntk_…", "nonce": "…", "expires_at": "…" }
```

The token is bound to the tenant, the expected node id, the host claim, and the
nonce; it is consumed ONCE by the node (`rb-node --enroll-token … --enroll-nonce
… --node-secret …`) before any channel traffic. The node id may be a `nod_…`
node id or the `rol_…` agent-role wire id the dev profile serves (one node, one
role). Issuance is a `tenant_admin`-authorized, audited decision; a re-issue
while an unused token is outstanding is a typed 409.

## Honest limits (Phase 1)

- **Development credentials**: the CLI presents a trusted principal header, and
  the node channel proves a dev shared secret (HMAC key-proof) — the server is
  the dev trust store. No certificate issuer yet (workload identity is
  ADR-006/ADR-007, Phase 2).
- Enroll is the dev bootstrap: a human creates the tenant boundary and its own
  admin grant (grant issuance is dev-trusted); a role's grant comes from
  `--actions` (default `thread_contribute`).
- Agent-side execution (a node receiving an invitation and contributing through
  the authenticated channel) is the `.6.2`/`.1.2.2` two-host demonstration;
  this chapter covers the control-plane command surface both halves share.
