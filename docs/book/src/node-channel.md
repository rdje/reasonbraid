# The node channel

The node connects **outbound** to the control plane (`ROADMAP.md` §9.3: nodes
initiate connections; ordinary deployments expose no inbound ports). This
chapter describes the Phase 1 channel: the authenticated reconnect exchange,
how delivery stays duplicate-safe, how leases make presence observable, and
how reconciliation gates the node's schedulability.

## Transport (Phase 1)

HTTP/1 JSON over the loopback development profile, served by the control plane
(axum) and spoken by the node (reqwest):

```text
POST /v1/nodes/handshake   the authenticated reconnect exchange
POST /v1/nodes/events      node results, deduplicated by the node-assigned event id
POST /v1/nodes/ack         cursor acknowledgement (delivery state)
POST /v1/nodes/poll        the live delivery tail after a cursor
POST /v1/nodes/heartbeat   lease renewal
GET  /v1/nodes/presence    observable online/offline state
POST /v1/nodes/enroll      the one-time-token enrollment (`.1.2.1`)
```

Every message carries a `channel_version` (currently **5**) and rejects unknown
fields, so a forged authoritative field or a future version fails loudly, on
both sides. Version 5 added the lease epoch (below); version 4 added the
cached-decision fields; version 3 was the certificate-proofed handshake
(`.1.2.2`).

## Authentication (`.1.2.2`, certificate-proofed)

The channel is authenticated end to end with the workload certificate
(ADR-007, `PHASE-2.1.2`). The dev trust store still holds the enrollment
secret, but the CHANNEL identity is the certificate:

1. **Enrollment** registers the node with a one-time token
   (`rb node issue-token` + `rb-node --enroll-token … --node-secret …`). The
   enrollment transaction issues a short-lived (10-minute) X.509 **workload
   certificate** — CN = the node id, SAN = the token's host claim — and the
   node persists `cert.der`/`key.der` beside its journal (the server-generated
   key is dev-escrowed: the `.1.2.1` trust-store stance).
2. **The handshake carries a certificate proof** — an ECDSA signature over the
   canonical channel fields (version, node id, cursor, pending operations,
   ambiguous attempts), made with the certificate's private key, plus the
   certificate itself:

   ```json
   {
     "channel_version": 4,
     "node_id": "rol_…",
     "last_acked_cursor": 3,
     "pending_operations": ["op_…"],
     "ambiguous_attempts": [
       { "attempt_id": "patt_…", "operation_id": "op_…" }
     ],
     "cert_der": "3082…hex DER…",
     "proof_signature": "3045…hex signature…"
   }
   ```

   The server verifies the leaf chains to its CA within its validity window,
   that its fingerprint is registered for THIS node (and not revoked/expired),
   and that the signature verifies — all **before any ledger fact is read**. A
   foreign, expired, unregistered, or wrongly signed certificate is refused
   `401 unauthorized` identically (no existence leak).
3. **Rotation** (`POST /v1/nodes/rotate`): the node presents its CURRENT
   certificate and receives a fresh key + certificate for the same node id.
   Rotation is additive — the old leaf stays valid until expiry or revocation —
   and the node rotates automatically when less than half the leaf's lifetime
   remains, so a running session is never cut.
4. **A successful handshake issues a lease**: a fresh random **fencing token**
   and an expiry 60 s out, plus a bumped **lease epoch** (`.2.2`). The token is
   the channel's credential from then on: `events`, `ack`, `poll`, and
   `heartbeat` all carry it AND the epoch it was issued under, and only the
   latest handshake's pair is accepted — a second handshake **fences** the old
   token, and a write or renewal from the fenced epoch matches nothing (a stale
   process that missed the rotation can neither write nor extend the lease it
   lost).
5. **Heartbeats renew a LIVE lease** (`POST /v1/nodes/heartbeat`, the node
   heartbeats every ~15 s). A fenced token, a stale epoch, or an expired lease
   is refused; only a fresh handshake — a new certificate proof — restores the
   channel. A heartbeat racing a newer handshake loses: its renewal carries the
   epoch it verified, and the write matches no row once the rotation lands
   (`.2.2` — the last writer is never a stale one).

## Presence and leases

`GET /v1/nodes/presence?node_id=…` exposes one node's observable state:

```json
{ "node_id": "rol_…", "online": true, "last_seen_at": "…", "lease_expires_at": "…" }
```

`online` is **derived from the lease expiry clock** — never a stored flag — so a
crashed process cannot leave a stale `online` row behind: within one TTL of its
last heartbeat the node is observably `offline`. An unenrolled node is a typed
`404 unknown_node`, not a fabricated `offline`. The fencing token itself is
never exposed by the presence surface (presence is an observability fact; the
token is a credential).

## Reconnect and cursor resume

The node is authoritative for what it **durably holds**. On reconnect it proves
its key and reports its resume facts (above). The server replies with every
command after `last_acked_cursor` (the replay tail), plus reconciliation
guidance and the fresh lease:

- **directives** — one per ambiguous attempt: `adjudicated` when the server
  holds a receipt for the operation's event (the node marks the attempt
  `reconciled`), `needs_adjudication` otherwise (the attempt stays visibly
  `outcome_unknown` — bounded, never silently retried).
- **known_events** — server-held receipts for the node's pending operations,
  so a result whose acknowledgement was lost is not re-sent.
- **fencing_token + lease_expires_at** — the new lease (see above).

A node that reports a cursor ahead of the server's ledger is **refused** with a
typed error: its journal saw commands this server cannot reproduce.

## Duplicate safety

The server replays, the node's journal deduplicates:

- a duplicated command never creates a second local operation (`operations` is
  keyed 1:1 on the command id), and
- a re-emitted event carries its **original id**, so the server's
  event-receipt primary key turns redelivery into a duplicate, never a second
  event.

## Schedulability gate

A node is `Offline → Reconciling → Schedulable`, and it becomes `Schedulable`
**only** after the full handshake round-trip is applied: crashed attempts
classified, replay journaled, directives applied, pending results re-emitted
(or acknowledged as known), cursor acknowledged on both sides. New work
(`emit_event`) is refused until then, and any failure drops the node back to
`Offline` — retrying the whole protocol is always safe because every step is
idempotent.

## Inbox hardening (`.1.2.3`)

Two operator actions harden the per-node inbox (both on the control API,
`tenant_admin`-authorized and audited — `rb node …` verbs in the CLI chapter):

- **Quarantine** (`rb node quarantine --node … --command … --reason …`): the
  row is marked quarantined WITH its reason, and the replay/poll paths skip it
  from then on — a quarantined command is **never re-delivered**, whatever
  cursor the node reports. The reason is stored with the row, so the skip is
  explainable. A node that received the command before the quarantine may
  still return a result: quarantine controls *delivery*, not result
  application.
- **Retention** (`rb node prune --node … --min-age-seconds …`): deletes
  DELIVERED rows older than the window — an explicit, measured operator action
  (the response reports `before`/`deleted`/`after`), never a background sweep.
- **Inspection** (`rb node inbox --node …`): every row's delivery +
  quarantine facts, in cursor order.
- **Revocation** (`rb node revoke --node … --reason …`, `.1.3.1`): marks the
  node's ACTIVE workload certificates revoked — the certificate-proof
  handshake refuses them at the next crossing (401) and presence reads
  `suspended` whatever the lease says. The live lease is not cut: suspension
  gates re-entry, it does not rewrite the running session.

Filtered delivery by eligibility stays with Phase 3's directory.

## Ordering a node result against an authority change

A node-emitted `work_result` folds into its thread through the same
claim → authorize → validate → apply flow a CLI command rides, so it needs the
same ordering against a tenant authority change that
[a thread command has](authority.md#ordering-a-thread-command-against-an-authority-change).

`POST /v1/nodes/events` now takes the tenant's authority guard **first** — before
the node's lease row, before the receipt, before the idempotency claim and before
every domain lock. The tenant comes from the inbox row the result names, read by
a plain lookup that takes no lock, so the guard is genuinely the first lock the
transaction holds and there is no order to invert. The mode is *shared*: results
from different nodes in a tenant still run concurrently, while a revocation's
*exclusive* mode fences every result that has not already passed that point.

This is an ordering that previously did not exist at all, rather than a race
window that has been narrowed.

The decision time is **database** time sampled after the guard and the
idempotency claim have both waited, so authority that ended while a result queued
is evaluated as it stands after the wait. Every effect the fold performs — the
work command lookup and the dead-letter quarantine — names the guarded tenant in
its own SQL, so a row that changed between the pre-lock lookup and the guarded
read matches nothing instead of being applied under a guard that does not cover
it.

| Order | Result |
| --- | --- |
| A revocation holds the exclusive guard when a result arrives | The result waits, then evaluates against the revoked authority. |
| A result holds its shared guard when a revocation arrives | The result commits; the revocation applies to what follows. |
| Authority ends while a result waits | The post-wait database time sees it ended, and the fold is refused. |
| Two results in the same tenant | Both hold the shared guard; neither blocks the other. |
| A result in an unrelated tenant | Unaffected — the guard is per tenant. |
| An ordinary channel receipt with no work command | No tenant-bound effect, so no guard: the receipt records and nothing else changes. |

An event with **no tenant-bound effect** takes no guard at all. That is the
common case for plain channel traffic, and it keeps the ordering from becoming a
toll on every message the channel carries.

### A rejected result and a failed one are different answers

A **rejected** application is a committed fact: the node did emit the event, so
its receipt commits, and the refusal is stored as the work command's idempotent
result. `accepted: true` means *this event id was new*, not *the work applied*.

A **storage failure** is not. It aborts the transaction, so the handler's COMMIT
would be executed as a ROLLBACK — and the handler used to answer `accepted: true`
for a receipt that was never written, which the node would then never re-emit.
The transaction's health is now probed before the commit, exactly as the tenant
guard's own runner does, so a discarded error cannot be reported as success: the
request fails with `dependency_unavailable`, nothing is written, and the node's
redelivery is free to try again.

Node credential and lease proof, partial-result handling and the budget
settlement guarantees keep their own repair owners; an authority guard does not
fix those mechanisms.

## Cached decisions (`.1.5.2`, ADR-008)

The node caches exactly one class of decision: the server's **admission
decision** that rides each delivered work item (ROADMAP §16.4: nodes may cache
only explicitly cacheable decisions; §11.1: the minimum authorized state). A
delivered command carries `authz_ref` (the admitting authorization record),
`policy_digest`, `decided_at`, and `revocation_epoch` — the tenant's epoch **at
decision time**. The handshake and poll responses carry the tenant's **current**
`revocation_epoch` and the server's own clock as `server_time`.

At the **dispatch boundary** (before any provider contact) the node evaluates
the cached decision against the declared rules:

- **Freshness** — a cached allow stands for at most **60 s** from `decided_at`.

  ⚠️ **Two clocks meet here, and the node corrects for the difference rather
  than guessing.** `decided_at` is the server's database clock; the node's own
  clock is what it would otherwise compare against. Every handshake and poll
  response therefore carries `server_time`, the server's clock at the moment it
  answered. The node measures `offset = server_time − (the midpoint of its send
  and receive instants)`, stores it beside the revocation epoch, and evaluates
  `decided_at` through it. The freshness comparison is then same-clock, and a
  node whose clock is wrong keeps working correctly.

  Without that correction the difference between two clocks read as *age*, in
  both directions and both wrong:

  | This node's clock | Uncorrected | Now |
  | --- | --- | --- |
  | ahead of the server by > 60 s | **every** dispatch refused — the node did no work at all | dispatches normally |
  | behind the server | decisions looked newer than they were, so a stale one could still dispatch | refused once it is genuinely older than 60 s |

  A disagreement of 5 s or more is **reported** by the node, naming both clocks
  and the offset. Before this, nothing anywhere reported that a node's clock
  disagreed with the server's.

  ⚠️ The offset is taken from the server's own statement, so it is **not a
  secure time source**. It adds no new trust: the node already accepts
  `revocation_epoch` from the same response, which is a stronger claim than the
  time. Treat it as a correction, not as clock security.

  ⛔ An earlier repair bounded this hazard without a wire change, by running the
  window from whichever came first, `decided_at` or the node's own receipt. That
  is **superseded**. The receipt is a local instant, and mixing it into a value
  compared against server-clock time made a correctly-corrected node refuse
  everything: a node 600 s behind computed its expiry from its own receipt and
  found every decision stale. Bounding a cross-clock comparison and removing it
  do not compose.

- **Revocation epoch** — every revocation write (node, grant, or boundary)
  bumps the tenant's epoch in the same transaction as the status change; a
  cached decision whose recorded epoch no longer matches the current one is
  invalidated, however fresh it looks. The node learns the current epoch from
  the next handshake/poll — so a revocation refuses the next dispatch within
  one poll interval (the honest dev bound).
- **Fail closed** — an expired or epoch-stale cached allow, a cached deny, or
  a command with **no** cached decision (a pre-migration row or plain channel
  traffic) is refused at the boundary and journaled as
  `failed_before_dispatch` with the reason. The adapter is never invoked and
  the refusal is never silently retried.

The journal is explicitly **not authoritative** for "global grants or final
decisions" (§17.1) — the cache borrows a server fact, it never re-evaluates a
grant locally.

## Honest limits (Phase 2, after `.1.5.2`)

- The channel identity is the workload certificate with the certificate-proof
  handshake (chain-to-CA + node-id fingerprint + signature). The TRANSPORT is
  still plain HTTP/1 (the mTLS streaming profile is a later hardening — the
  proof rides the channel, per ADR-006); the CA key and the node's dev-
  escrowed leaf key live with the control plane (ADR-007's Trusted-LAN
  placement, re-evaluated before any non-LAN exposure).
- The lease TTL is the dev constant 60 s; heartbeats are process-local (no
  persisted heartbeat state on the node side).
- Two live processes for one node id fence each other by design (each
  handshake rotates the token) — that is the fencing contract making staleness
  visible, not a bug.
- Live delivery is a poll of the tail; the streaming profile is the formal
  ADR-006 decision.
- A quarantined row leaves a permanent hole in the node's cursor ledger (the
  node never holds it) — acknowledgements still converge because the ack path
  covers it. Quarantined rows are pruned like any other delivered row once the
  retention window passes.
