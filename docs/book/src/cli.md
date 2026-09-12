# The CLI

Phase 0 ships a command-line surface for the vertical slice: `rb` (the
`reasonbraid-cli` crate) drives the control plane over HTTP JSON. The CLI covers
the established enrollment and thread workflows; the additional authorization
receipt surface below is accessed directly over HTTP.

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
$ rb thread accept --thread thr_… --as reviewer
$ rb thread contribute --thread thr_… --text "Ship it: the experiments are green." \
    --kind claim --evidence-uri https://example.org/evidence --as reviewer
$ rb thread advance-round --thread thr_… --as alice
$ rb thread challenge --thread thr_… --target evt_… --text "Which experiments?" --as alice
$ rb thread revise --thread thr_… --target evt_… --text "The SQLite kill-point sweep…" --as reviewer
$ rb thread close --thread thr_… --reason "decision reached" --as alice
$ rb thread cancel --thread thr_… --reason "no longer needed" --as alice
```

Enrollment replay is scoped to the existing tenant, kind and name. Repeating
`rb enroll role reviewer --tenant ten_…` returns the original principal with
`replayed: true` in `--json` output, without a new grant or changed action set.
Concurrent requests for that name now serialize through the full server
transaction. Replay still works after a boundary freeze; new enrollment refuses.
A repeated human enrollment without `--tenant` creates a separate new tenant.
Use the returned tenant ID when intending an existing-tenant replay.

A parent expiring during a guard wait cannot authorize new enrollment. Storage
failure returns a safe internal error; an unconfirmed commit returns
`commit_outcome_unconfirmed`. Inspect the relevant tenant state before retrying
that outcome. For a new bootstrap, a lost response can leave the CLI without the
server-generated tenant ID; operator database reconciliation may be needed. The
server now accepts an explicit bootstrap_request_id and returns its committed
creation outcome on matching retries, but this CLI does not yet persist or send
that key. Live qualification has confirmed a commit after the unconfirmed response.
The selected next CLI design
will persist a request ID before sending and retain it through local state
publication; this recovery behavior is not implemented yet. Retrying the same human
name without --tenant can create another tenant. These are server transaction guarantees; the CLI's local state-file
write occurs after the server response and is a separate persistence step.

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

The contribute verb takes the structured body (`.1.5.1`):

- `--kind` — `position` (default) | `claim` | `assumption` | `evidence-reference`
  | `question` | `summary` (the kebab spelling normalizes to the wire's
  snake_case, e.g. `evidence-reference` → `evidence_reference`).
- `--evidence-uri` (repeatable) — a reference the contribution cites, rendered in
  the inspection view. References only: citing a URI is not fetching it —
  acquisition arrives with a later phase (§3.7).

`thread.advance-round` moves the thread to its next round (`.1.5.2`):

- Rounds are **server-assigned**: a new thread is round 1, every contribution
  lands in the current round and carries it in the events view, and advancement
  is this explicit, auditable verb (event `thread.round_advanced`).
- Humans carry the `thread_advance_round` grant via the dev admin set; roles are
  deny-by-default (they shape content, humans shape the process).

`thread.close` takes the honest outcome (`.1.5.3`):

- `--outcome` — `decided` (default) | `inconclusive`: the honest terminal for a
  thread that ends WITHOUT a decision — state `inconclusive`, distinct from
  `closed` and from the `cancelled` abandonment terminal.
- `--unresolved` (repeatable) — the items that prevented the decision; they ride
  the close event and render in the events view. A decided close carrying
  unresolved items is refused: naming what is still open while claiming a
  decision would be dishonest.

`thread.cancel` is the **abandonment terminal** (`open|closing → cancelled`,
reason recorded) — distinct from a decided close; both are inspectable, and a
cancelled thread refuses further content verbs with `invalid_transition`.

Explicit participants (`.1.3.1`): an invite is a **pending offer** — the
invited role **accepts** (`rb thread accept --as reviewer`) before it may act,
and the accept is the transaction that dispatches the role's work item.
`rb thread decline` refuses the offer (a declined role may be re-invited);
`rb thread remove-participant --participant rol_…` is the tenant-admin
revocation, requiring live tenant-wide administrative authority; `--expires-in-seconds` on the invite offers a typed expiry
(derived — an expired offer reads `expired` and refuses accept/decline).
The server base — `--server` or `REASONBRAID_SERVER` — is checked before any
request: an absolute HTTP(S) URL without URL credentials, query or fragment.
See [bounded transport](cli-state.md#bounded-transport) and
[the configured endpoint](cli-state.md#the-configured-endpoint-is-checked).

`rb thread join` is the self-request path (`.1.3.2`): a thread created with
`--allow-join-requests` admits the role directly — no invitation — while
`allow_explicit_invites=false` refuses the invite verb (recorded rules are
enforced at the command boundary).

Participant removal uses a tenant-administration target, while the thread lookup
remains bound to that tenant. Delegated removal with `--on-behalf-of` now requests
tenant-wide scope because its existing action requires that scope. Both the caller
and the named source must have live tenant-wide administrative authority. Ordinary
thread verbs keep their single-thread delegation. See the
[authority contract](authority.md#removing-a-participant).

The CLI keeps a repository-root-relative state directory (`.reasonbraid-cli`, or
`REASONBRAID_CLI_STATE`): names → principal ids, and the thread → tenant mapping,
so `--as alice` and `--as reviewer` resolve without retyping ids (raw `hpr_…` /
`rol_…` ids are accepted directly). The server is `http://127.0.0.1:4310` or
`REASONBRAID_SERVER`.
See [CLI local state and recovery](cli-state.md) for path validation,
bounded snapshots, synchronized replacement, complete writer locking and the
remaining pending-bootstrap recovery limits.

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

The budget ledger is readable through the same surface (`.1.6.1`):

```text
$ rb inspect budget --thread thr_… --as alice
budget for thread thr_… (tenant ten_…)
ceiling: bdg_… (policy dev-budget-1) — created 2026-09-06T…
dimensions: {"calls":3,…}
reservations (2):
  bdg_res_… active held={"calls":1,…} usage=—
  bdg_res_… denied held={"calls":1,…} usage=— — reason: budget denied the dispatch: …
```

The view is read-only and `thread_inspect`-gated: it shows the ceiling plus
every reservation row — held vs settled usage, denials with their reasons —
the same ledger rows the budget engine enforces against, so spend and
uncertainty are visible without database surgery.

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

The inbox hardening verbs (`.1.2.3`) complete the node-admin surface — all
`tenant_admin`-authorized and audited:

```text
$ rb node quarantine --node rol_… --command work_evt_… --reason "poison payload" --as alice
quarantined command work_evt_… in node rol_…'s inbox (…)

$ rb node inbox --node rol_… --as alice
node rol_…'s inbox (3 rows):
  #1 work_evt_… — delivered
  #2 work_evt_… — QUARANTINED (poison payload)
  #3 work_evt_… — undelivered

$ rb node prune --node rol_… --min-age-seconds 604800 --as alice
pruned 2 delivered row(s) from node rol_…'s inbox (before 5, after 3, cutoff …)
```

A quarantined command is never re-delivered (the reason rides the row);
pruning deletes only DELIVERED rows older than the window and reports a
measured before/after — an explicit operator action, never a background sweep.

The revocation verb (`.1.3.1`) completes the node-admin surface — also
`tenant_admin`-authorized and audited:

```text
$ rb node revoke --node rol_… --reason "compromised adapter output" --as alice
node rol_… revoked (1 certificate(s), at 2026-09-07T…Z)
```

Revocation marks the node.s ACTIVE workload certificates revoked: its next
handshake is refused (the certificate-proof ladder sees the revoked row) and
presence reads `suspended` — the live lease, if any, is not cut. An unknown
node is a typed 404; a second revocation (no active certificate left) is a
typed 409.

The authority revocation verbs (`.1.3.2`) require tenant-admin admission and record
that admission decision:

```text
 grant revoke --grant grt_rol_… --reason "role retired" --as alice
grant grt_rol_… revoked (at 2026-09-07T…Z)

 boundary revoke --boundary bnd_ten_… --reason "tenant frozen" --as alice
boundary bnd_ten_… revoked (at 2026-09-07T…Z)

 inspect grants --as alice
 inspect boundaries --as alice
```

A revoked grant loses its authority at the next decision. Normal command
authorization records its refusal. A revoked boundary freezes writes under it;
own-tenant inspection remains available only with a structurally valid, unexpired
tenant-wide administrator grant. The exception ignores boundary status alone,
retaining parent ceilings and both validity windows. These administrative reads
commit explicit inspection admissions before fetching their response. Their HTTP
responses carry `x-reasonbraid-authorization`; use `curl -i` to retain it. The
authority chapter documents the eight routes, failure cases and receipt limits.
Retrieve a known receipt through
`GET /v1/admin/authorization-records/{record_id}?tenant_id=ten_…` with the same
principal header. This HTTP endpoint returns the earlier record and a separate
admission receipt for the lookup; `rb` has no dedicated receipt command yet.

A grant or boundary outside the acting tenant returns 404 without changing the
foreign target or its tenant's revocation epoch. A repeated grant revocation returns
409 without another epoch increment. Concurrent admitted revocations of the same
grant serialize at its row and advance the epoch once. The revocation reason is
required input; persisting it with the final effect outcome remains corrective work
under `SIGNOFF-REPAIR.3.3`. See the authority chapter for the admission/effect audit
boundary and verification status.

The incarnation surface (`.1.6.1`; deferral #4's first half): a node that
enrolls AS a role (the dev wiring — its id IS the role wire id) records the
§8.1 facts it declared at start, and the tenant_admin inspection shows them:

```text
 inspect incarnations --as alice
```

A plain `nod_…` node serves no role and records no incarnation (the hierarchy
binds incarnations to roles).

The run surface (`.1.6.2`; deferral #4's second half): a node-emitted result
receipt records the run linking its attempt to the incarnation that ran it —
one result = one run, ever (the idempotency claim dedupes redelivery first):

```text
 inspect runs --as alice
```

The dead-letter surface (`.2.4`): a terminal refusal auto-quarantines the
inbox row with the reason, and the operator REPLAYS it — the quarantine
clears, the admission decision refreshes, and the command re-enters the
delivery tail:

```text
 node replay --node rol_… --command work_evt_… --as alice
```

The spend circuit breaker (`.3.2`): a per-tenant latch — once the tenant's
recorded spend crosses the declared threshold, new dispatch reservations are
refused with the typed reason until the operator resets:

```text
 breaker arm --threshold '{"calls": 1000}' --as alice
 breaker reset --as alice
 inspect breakers --as alice
```

Both mutating verbs are admitted, applied and recorded in one guarded
transaction, and their HTTP responses carry `x-reasonbraid-authorization` like
the revocation verbs do (`curl -i` retains it). A reset answers `409` both when
no breaker is armed and when one is armed but untripped; the two are
distinguished in the durable effect record rather than on the wire. See
[arming and resetting a spend breaker](authority.md#arming-and-resetting-a-spend-breaker).

The usage reconciliation (`.3.3`): the estimates-vs-receipts picture summed
over the ledger rows — held vs settled vs overrun vs denied, per tenant and
per thread:

```text
 inspect usage --as alice
```

The delegation flags (`.1.4.2`) let an actor act ON BEHALF OF another
principal whose grant is the authority source:

```text
 thread contribute --thread thr_… --text "delegated" \\
    --on-behalf-of rol_… --purpose "owner is offline" --as alice
```

The server evaluates both the caller's own grant and the source's grant, and
refuses insufficient authority or scope with 403. The audit row identifies the
source. The CLI chooses scope according to the operation:

| Operation | Requested delegation scope |
| --- | --- |
| Create a thread | Tenant-wide, matching the creation target. |
| Remove a participant | Tenant-wide, matching tenant administration. |
| Other existing-thread verbs, such as invite or contribute | Only the named thread. |

For removal, supply the raw principal ID of an eligible administrative source:

```bash
rb thread remove-participant --thread "$THREAD_ID" --participant "$ROLE_ID" \
  --as organizer --on-behalf-of "$ADMIN_SOURCE_ID" --purpose "review complete"
```

The flag creates no grant. A caller without administration cannot borrow it from
the source; a thread-only, revoked or foreign-tenant source is insufficient. The
thread remains bound to the selected tenant. Real CLI controls cover this mapping
and retain ordinary one-thread delegation; broader consent/depth enforcement and
concurrent revocation ordering remain open under `.3.4`/`.3.3.4.4`. Evidence:
`docs/tasks/artifacts/signoff_review/cli-removal-delegation.md`.

## Honest limits (Phase 1)

- **Development credentials**: the CLI presents a trusted principal header, and
  the node channel proves a dev shared secret (HMAC key-proof) — the server is
  the dev trust store. No certificate issuer yet (workload identity is
  ADR-006/ADR-007, Phase 2).
- Enroll is the dev bootstrap: a human creates the tenant boundary and its own
  admin grant (grant issuance is dev-trusted); a role's grant comes from
  `--actions` (default `thread_contribute` plus `thread_invitation_respond`).
  Human action input is ignored in favor of the explicit development admin set;
  a role grant records a fresh human issuer handle, without authenticated issuer
  provenance. The complete server transaction and its limits are described in
  [Authority](authority.md#development-enrollment-transactions).
- Agent-side execution (a node receiving an invitation and contributing through
  the authenticated channel) is the `.6.2`/`.1.2.2` two-host demonstration;
  this chapter covers the control-plane command surface both halves share.
