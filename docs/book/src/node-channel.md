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

   **A rotated identity is written to disk before it is used** (`.4.2.9`): the
   node persists the fresh certificate and key beside its journal, in the same
   two files it loads at start, so a rotation survives a restart. Until it did,
   a node that rotated and restarted came back holding the superseded
   certificate — usually harmless, because it simply rotated again on the next
   handshake, but a node that stayed down until that certificate expired could
   not rotate at all (rotation requires a usable certificate) and had to be
   re-enrolled by an operator.
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

   **Every one of those conditions is part of the WRITE, not a check before
   it** (`.4.2.3`). The renewal is a single `UPDATE` whose `WHERE` names the
   node, the epoch it verified, the lease's own expiry and the certificate's
   liveness, so nothing can change between the decision and the effect. A
   heartbeat whose admission check passed and whose write was then delayed —
   behind a lock, a saturated pool, or simply the scheduler — cannot revive a
   lease that reached its expiry in the meantime: the node is told the lease
   expired, and the answer is the same one it gets when the expiry is already
   past on arrival. The lease clock the renewal reads is the **database's**, the
   same clock `online` is derived from below, so a renewal can never succeed for
   a node the presence surface simultaneously reports `offline`.
6. **A renewal also requires a usable certificate.** Extending a lease is the
   one channel operation that asks whether the node is still trusted: if the
   node holds no workload certificate that is both unrevoked and unexpired, the
   heartbeat is refused with a message that says so, distinct from a fenced
   token — because re-handshaking fixes a fenced token and cannot fix a
   withdrawn credential. Poll, ack and events deliberately do **not** ask for
   ADMISSION — they still answer on the fencing token alone — but `poll`'s
   delivery asks the same question before it hands over any row, so a node that
   may no longer extend its lease is handed no further work either; see
   [Revocation and a running session](#revocation-and-a-running-session).

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

**Every channel WRITE re-checks the lease inside its own transaction** with the
lease row locked, so a session fenced by a newer handshake cannot ride an
admission check into a write: `events` since `.2.2`, the lease renewal since
`.4.2.3`, and `ack` since `.4.2.4`. This matters most for `ack`, because
acknowledgement is what makes an inbox row terminal — and a terminal row is
what the retention prune below is allowed to delete. A stale acknowledgement
would therefore make work the *new* session is still holding eligible for
deletion, leaving the node's ledger and the server's permanently disagreed.

`poll` deliberately keeps the plain check and is **not** wrapped, because it
writes nothing: it reads the cursor, the replay tail and the revocation epoch,
and a fenced session that receives a stale tail leaves no trace — the rows stay
in the inbox and replay to whoever holds the lease. An in-transaction
re-verification there would buy a lock on every delivery poll and no invariant.

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
  gates re-entry, it does not rewrite the running session. What that costs, and
  what bounds it, is below.

### Revocation and a running session

Revoking a node does not cut the session it is running. That is a deliberate
choice — a node mid-command is not made to have never existed — and it means a
revoked node keeps polling, acking and emitting results until its lease runs
out. The bound on that tail is the lease clock: 60 s.

The bound is what had to be repaired. A heartbeat used to verify only the
fencing token, which reads no ledger fact, so a revoked node renewed its own
lease every minute, never reached the handshake that would have refused it, and
stayed fully operational indefinitely. The handshake and the certificate
rotation are the only two channel operations that re-present a certificate, and
a node that never stops never performs either. Revocation of a running node did
nothing, and kept doing nothing.

So the renewal now asks the question the handshake asks: does this node still
hold a certificate that is neither revoked nor expired? If not, the lease stops
being extended, and the session ends on the clock it already had. Nothing cuts
it short — the refused renewal moves no expiry — and the check sits on the
renewal rather than on the fencing verification precisely so that poll, ack and
events keep the tail the design intends.

### What the tail still receives — nothing new

That left one question open, and it is now decided: during the tail, the server
hands the node **no work at all**. Revoking a node is the operator withdrawing
trust in an identity; handing that identity newly enqueued work for a further
minute is not a courtesy to a running session, it is fresh authority granted
after the withdrawal.

So the delivery read asks the same question the renewal asks — does this node
hold a certificate that is neither revoked nor expired? — and while the answer
is no, the tail is empty. Concretely:

| Surface | A revoked node, inside its remaining lease |
| --- | --- |
| `poll` | **admitted**, `current_cursor` answers truthfully, **zero commands** |
| `ack` | admitted — the node still settles what it holds |
| `events` | admitted — results for work already in hand still land |
| `heartbeat` | refused `401 credential_refused`; the lease is not extended |
| `rotate` / `handshake` | refused `401` (the certificate proof) |

Two properties make that safe, and both are pinned by controls:

- **The work is withheld, not dropped.** The rows stay in the inbox with their
  cursors. This is why the filter sits on the delivery read rather than on the
  dispatch: the replacement ritual re-enrols the *same* node id with a fresh
  certificate, and the withheld tail then replays to it in cursor order. A
  refusal at dispatch time would have destroyed that work instead.
- **It is a filter, not a refusal.** `poll` still admits on its fencing check
  and `ack`/`events` are untouched, so the session in flight is still not cut —
  it finishes and reports the work it already has, and receives nothing more.
  This is the same shape the zero-concurrency wake gate uses.

The predicate is *a usable certificate today*, not *ever revoked*. The presence
view's `suspended` flag is close but is not the same question: it reads
"a revoked certificate exists and no unrevoked one does", so it correctly leaves
a replaced node unsuspended — but it never asks whether the surviving
certificate is still in date. A node whose only certificate has simply lapsed
reads `suspended: false` and is still handed nothing, because its renewal has
already stopped too.

### A captured proof cannot be replayed

Both authenticated operations — the handshake and the rotation — prove
possession of the workload key by signing a canonical set of fields. Neither set
used to contain anything that changed between requests, and the rotation's was
entirely static for a given node and certificate. So a captured request stayed
valid for as long as the certificate did, and could simply be sent again.

Driven against the unrepaired product, that meant:

| Replayed | Answer |
| --- | --- |
| A captured handshake | `200` — the replay takes a **new lease**, and the legitimate node's next heartbeat is refused `401`. It is fenced out of its own session. |
| A captured rotation | `200` — and a **second private key** for the same identity. Repeat for a third. |

Both requests now carry a `nonce`, chosen fresh by the node for each request and
covered by the signature. The server consumes it once: the first presentation
inserts it, and any later presentation of the same bytes is refused `401`.

**A nonce rather than a server challenge**, and the reasons are practical: a
challenge costs an extra round trip on every handshake and every rotation, a
timestamp-and-skew-window design would put a clock dependency on the
authentication path, and this is already the shape enrollment tokens use.

⚠️ **This refuses replays, not reconnects.** A node whose response was lost
retries with a *fresh* nonce and is served normally — including a second
rotation, which rotation's additive contract already allows. Consuming the
*certificate* instead would have been simpler and would have broken exactly that
recovery, because a retry and a replay present the same certificate.

The nonce is consumed only *after* the signature verifies, so an unauthenticated
caller cannot burn a value a legitimate node was about to use. Consumed nonces
are pruned per node on the next proof; a replay is useless once the certificate
it presents has expired, so retention outlives that by a wide margin.

⚠️ `nonce` is **required** on both requests — a request that omits it is `422`,
the same treatment `cert_der` and `proof_signature` already get. Server and node
ship together.

### A rotation in flight cannot outlive the revocation

Certificate rotation is additive — the old fingerprint stays valid until it
expires or is revoked, so a running session is never cut — and that raised a
question the design had not answered: what happens to a rotation that is already
under way when the operator revokes the node?

It used to finish. The rotation verified the proof, looked up the host and
inserted the new certificate as three separate statements, so its *decision*
("this certificate is live") and its *effect* ("here is a new live certificate")
were not atomic. A revocation that committed between them did not see the new
row, and the withdrawn node was left holding a usable certificate — which, since
both the lease renewal and the work delivery ask whether such a certificate
exists, handed it back its lease and its work.

Rotation now runs as one transaction that takes the node's row **first**, and
only then re-reads the certificate it was shown. That is the same row, in the
same mode, the revocation already takes before it changes anything, so the two
simply serialize:

| Order | Result |
| --- | --- |
| The rotation reaches the node row first | It commits a new certificate; the revocation then runs and revokes **both** identities. |
| The revocation reaches it first | The rotation waits, re-reads the certificate on the far side of the revocation, finds it revoked, and is refused `401`. |

Either way a revoked node holds no usable certificate, which is the property the
two checks above depend on.

⚠️ **The remaining honest limit.** Nothing at the transport re-checks a
certificate: the dev profile's wire is plain HTTP (see the deployment chapter's
Honest Boundaries), so a certificate's own expiry does not bound any surface
that never presents one. The database-side checks on renewal and delivery are
what carry it instead.

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
- `lease_expires_at` is **written** from the server process clock (`now + TTL`)
  and **read** from the database clock — by `online` above, and since `.4.2.3`
  by the renewal itself. On one host those agree; nothing requires them to, so
  the lease's real duration carries whatever process↔database skew exists. The
  60 s is therefore a nominal TTL, not a guaranteed one. This predates `.4.2.3`
  and is unchanged by it — that repair made the renewal agree with the presence
  surface rather than introducing a new comparison — and it is tracked
  separately.
- Two live processes for one node id fence each other by design (each
  handshake rotates the token) — that is the fencing contract making staleness
  visible, not a bug.
- The node **trusts the control plane's response body** to be well-formed
  enough to parse. It no longer aborts on a malformed one — a rotate response
  whose hex fields are not hex is refused as `Malformed` and the node keeps
  running (`.4.2.7`) — but nothing authenticates the response itself, because
  the development transport is plain HTTP. A node talking to a hostile server
  is not a threat this profile addresses; the mTLS streaming profile is where
  that changes.
- Live delivery is a poll of the tail; the streaming profile is the formal
  ADR-006 decision.
- A quarantined row leaves a permanent hole in the node's cursor ledger (the
  node never holds it) — acknowledgements still converge because the ack path
  covers it. Quarantined rows are pruned like any other delivered row once the
  retention window passes.
