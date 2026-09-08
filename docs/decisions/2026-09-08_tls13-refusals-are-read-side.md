# TLS 1.3 refusals are read-side, never connect-side (`PHASE-7.1.2`)

- Date: 2026-09-08 · Leaf: `PHASE-7.1.2` · Decision record (the mTLS transport proof's testing contract)

## Context

The `.1.2` offline test proves the mTLS refusal: a cert-less client must be
refused at the transport (the §16.2 mutual authentication). The first draft
asserted the refusal on the CLIENT's `TlsConnector::connect` result — and the
test failed: `connect` returned `Ok` while the SERVER's accept returned the
certificate-required error. The asymmetry is TLS 1.3's own: the client sends
its Finished and considers the handshake complete BEFORE the server's
`certificate_required` alert arrives, so the client-side connect legitimately
succeeds and the refusal only surfaces on the first read (the alert or the
EOF).

## Decision

- **The refusal proof is the SERVER's accept + the CLIENT's first read.**
  Every future TLS test asserts the transport refusal on the server side
  (`acceptor.accept(stream).await` returns the certificate-required error)
  and, on the client side, on the first read — never on the client's
  `connect` alone, which may return `Ok` for a handshake the server has
  already refused.
- **The client's first read must not deliver data.** The cert-less leg's
  client assertion accepts `Err(_) | Ok(0)` (the alert or the EOF) and
  rejects any data byte — a `Ok(1)` would mean the refusal was bypassed.
- **The two accepts are sequential in ONE listener.** The roundtrip test
  drives the issued client and the cert-less client against the same
  acceptor in one tokio task, so the refusal is measured against the exact
  config that passed the mutual handshake.

answers:

- **TLS 1.3's client-Finished-before-alert ordering makes connect-side
  refusal assertions structurally unsound** — the server's verdict exists
  independently of whether the client has observed it yet.
- **The transport proof stays honest by measuring the server's refusal** (the
  config under test) **and the client's observable failure** (the alert/EOF),
  not a race-prone midpoint.
- **The application-layer binding is unchanged by this**: the transport
  verifies the CA membership; the fingerprint → the principal binding stays
  the channel's `verify_cert_proof` (the layered defense — ADR-034).
