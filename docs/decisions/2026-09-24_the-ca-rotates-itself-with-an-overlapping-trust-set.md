---
answers:
  - How is the workload-identity CA renewed?
  - When does the server mint a successor CA?
  - Which CAs does the server trust for node leaves?
---
# The CA rotates itself, with an overlapping trust set, a third of its life before it expires

- **Type:** decision
- **Status:** active
- **Owner:** `SIGNOFF-REPAIR.4.1.8`
- **Date:** 2026-09-24
- **Work unit:** `REASONBRAID-DOC-0159`

## The fact / decision

The workload-identity CA is signed for 365 days and, until this decision, there was exactly one (`server_ca` row `ca_id = 1`) for the life of the deployment. It is held as one immutable `Arc<ServerCa>` for the life of the server process. The server will keep a **set** of CAs:

- **Trust:** a node leaf is accepted if it chains to ANY stored CA that is inside its own validity window, not only to the newest.
- **Issue:** leaves are signed by the newest CA (the highest `ca_id`).
- **Renew:** when the issuing CA has a third or less of its lifetime left, the next issuance mints a successor (`ca_id + 1`, inserted race-safely so concurrent servers converge on one), swaps it in for issuing in-process without a restart, and keeps the old CA in the trust set until it expires.
- **Report:** the readiness probe reports the CA as degraded when the issuing CA is inside the renewal window with no successor, which means renewal failed, well before expiry; `.4.1.7` already refuses issuance in the last rotation margin.

## Why

- **This is how deployed PKI rotates.** SPIFFE/SPIRE publishes the next authority into the trust bundle before it signs and keeps the old one until its last SVID expires; cert-manager renews a certificate when a third of its duration remains (its default `renewBefore`); Vault PKI rotates issuers while old issuers stay valid for verification. A restart-only renewal would make a year-long server the one that fails.
- **The overlap is cheap here.** Leaves live 600 s (ADR-007), so the old CA is needed for verification for ten minutes after the switch in practice. It is kept until its own expiry because that costs nothing and removes any timing argument.
- **Nothing outside the server pins the CA.** Census (`DOC-0159`): the only production trust anchors are `ca::verify_leaf_chain` (the channel's proof check) and the readiness probe; `mtls.rs` has no production caller, and `rb-node` authenticates with a signed proof, not a pinned root. So the trust set can change without touching any node.
- **A third, not a picked number.** It is cert-manager's documented default, it gives the readiness warning about four months of lead on a one-year CA, and it scales if the CA lifetime changes.

## How to apply

- Built in two steps: `.4.1.8.1` the CA set (storage by generation, verify against the set, issue from the newest; no behaviour change with one CA), then `.4.1.8.2` the in-process successor at the renewal threshold and the readiness report.
- The secret-store seam (`SecretStore::load_ca_material`) loads the whole set; a future non-database profile must store generations too.
- A compromised CA is still a documented recovery (ADR-007): rotation is for expiry, not for revocation of an issuer.
