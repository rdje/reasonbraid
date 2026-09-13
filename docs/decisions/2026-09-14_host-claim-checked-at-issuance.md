---
answers:
  - Where is a node's host claim validated, and against what rule?
  - Why is the certificate library's own verdict used instead of a DNS grammar?
  - Should the host claim accept set be narrowed to real DNS names?
---
# The host claim is checked where a human typed it, against the certificate library's own verdict

- **Type:** decision
- **Status:** accepted
- **Owner:** `SIGNOFF-REPAIR.4.1.6`
- **Date:** 2026-09-14

## Context

`ca::issue_node_leaf` builds a node's workload certificate with the enrollment
token's host claim as its subject alternative name. That claim is a caller
string: `POST /v1/nodes/enroll-tokens` bound `req.host_claim` straight into
`node_enrollment_tokens`, redemption required only that the two spellings match,
and the claim then reached `CertificateParams::new(vec![host_claim]).expect(…)`.

Measured through the supported routes before anything changed: issuance returned
`200` for a non-ASCII claim, and redemption **panicked** at `ca.rs:142`. The
node received a transport error rather than an answer; the server kept serving;
`nodes` held zero rows; and the token was left `used_at = NULL`. Because an
outstanding unused token refuses a second issuance for the same node id, that
node id could not be enrolled again until the token lapsed — bounded to the
token's lifetime (default one hour, maximum one day) only because
`SIGNOFF-REPAIR.4.1.1` already makes a lapsed token superseded.

A probe then asked rcgen 0.14.10 what it actually refuses. Of eight host claims
— `host-a`, `not a dns name!!`, the empty string, 300 characters, `héllo`,
`*.example.com`, `1.2.3.4` and `..` — **exactly one** returned an error: the
non-ASCII one. Eight inputs are a sample, not the boundary; what they establish
is that the panic is reachable and that the accepted set is very wide.

## Options

1. **Check at issuance, using the library's own verdict.** The issuance route
   asks `ca::check_host_claim`, which is `CertificateParams::new` and nothing
   else, and refuses with the existing typed `invalid_command`.
2. **Check at issuance against a DNS-name grammar written here.** Narrower, and
   arguably what a subject alternative name should be.
3. **Only make the issuer fallible**, and let the refusal surface at redemption.
4. Leave it, and treat the panic as unreachable in practice.

## Decision

**Option 1, plus the fallible issuer from option 3.** Two changes, and the split
matters:

- `issue_node_leaf` returns `Result<IssuedLeaf, HostClaimRefused>`. A function
  whose only way to report a bad caller string is to unwind leaves its caller's
  typed error path unreachable, which is the rule `SIGNOFF-REPAIR.4.2.7`
  promoted. Both call sites map it to a typed `400 invalid_command`.
- The issuance route calls `ca::check_host_claim` beside the existing node-id
  shape check and lifetime range check, because that is where a human typed the
  value. This is what makes the unenrollable-node state unreachable rather than
  merely survivable.

**The accept set is NOT narrowed.** `check_host_claim` asks the library and
restates no grammar, so every claim a deployment can use today it can still use
tomorrow, and the check cannot drift from what issuance will actually do.

Option 2 is **deferred, not rejected**, and the reason is a compatibility
question this repair is not the place to answer: narrowing would refuse claims
that existing deployments may already have enrolled, and `1.2.3.4` and
`*.example.com` are plausible operator inputs whose treatment deserves its own
evidence. Option 3 alone was rejected because it leaves the administrator's typo
costing a node id its enrollment window. Option 4 was rejected on the
reproduction.

## Consequences

- `POST /v1/nodes/enroll-tokens` gains a refusal it did not have. It is a
  narrowing of accepted input on a wire route, and it refuses only claims that
  could never have produced a certificate anyway.
- A stored host name that predates this check can still reach redemption or
  rotation; both now answer with a typed refusal instead of dropping the
  connection, and a control drives that path with a directly written token row.
- A stricter grammar remains available later. Nothing here assumes the current
  accept set is correct — only that it is unchanged.

## Revisit trigger

An operator needs a host claim the library accepts and a verifier will not
honour, or an upgrade changes what the certificate library accepts. Either makes
the deferred option 2 a live question again.
