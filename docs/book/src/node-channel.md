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

Every message carries a `channel_version` (currently **2**) and rejects unknown
fields, so a forged authoritative field or a future version fails loudly, on
both sides.

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
     "channel_version": 3,
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
   and an expiry 60 s out. The token is the channel's credential from then on:
   `events`, `ack`, and `poll` all carry it, and only the latest handshake's
   token is accepted — a second handshake **fences** the old token, so a stale
   process (one that missed the rotation) can write nothing.
5. **Heartbeats renew a LIVE lease** (`POST /v1/nodes/heartbeat`, the node
   heartbeats every ~15 s). A fenced token or an expired lease is refused; only
   a fresh handshake — a new certificate proof — restores the channel.

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

## Honest limits (Phase 2, after `.1.2.2`)

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
