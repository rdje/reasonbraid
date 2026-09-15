---
answers:
  - Which of G6-G7's seven shipped lines still re-derive against the repaired code?
  - Does the Phase-7 gate record's conclusion change?
  - What blocks Internet exposure now, and which part of it is internal?
  - Why does the R1 Git pack follow redirects when the R0 fetcher refuses to?
---
# G6–G7's seven shipped lines, re-derived: three must be re-earned, two are narrowed, two stand

- **Type:** decision
- **Status:** accepted
- **Owner:** `SIGNOFF-REPAIR.11.4.7.1`
- **Date:** 2026-09-15
- **Supersedes:** nothing. It **adds** to `2026-09-08_phase7-gate-record.md`, which is
  left byte-unchanged (`docs/decisions/` supersedes rather than mutates).

## Context

The Phase-7 gate record concluded **G6–G7 NOT MET for Internet exposure**, Met only as
the hardening-machinery exit for the Trusted-LAN profile, and mapped §16.12's ten lines as
**seven shipped with measured evidence** and **three external preconditions** that do not
exist. That conclusion is honest and is **not disturbed here**.

What the record could not know is whether the suites behind the seven *covered* the paths a
later full source read would find. It was written on 2026-09-08; the corrective review began
on 2026-09-09. Measured rather than assumed: of 198 repair commits, **110 touched product
source or a migration, and every one of them postdates the gate record** — so the whole
repair corpus is "since".

## Decision

Each of the seven carries a verdict from a closed set — `stands` / `narrowed` /
`must be re-earned`.

| §16.12 | Line | Verdict | Why |
| --- | --- | --- | --- |
| (2) | authenticated enrollment, rotation, revocation, **tenant-isolation** tests | **must be re-earned** | All FOUR subjects have live defects reproduced and repaired since. Enrollment: a token expiring unused locked its node out permanently (`.4.1.1`); a node id owned by another tenant answered `500` and nearly answered with a certificate (`.3.5.1`). Rotation: a rotation in flight could outlive a revocation (`.4.2.1`); a rotated identity was never persisted (`.4.2.9`). Revocation: a revoked node renewed its own lease indefinitely (`.4.1.3`); revocation was not bound to the tenant (`.3.1`). Tenant isolation: five separate cross-tenant reads and writes (`.6.1.1`, `.3.5.3`, `.3.3.4.10.3`, `.6.1.2`, `.9.2.1.2`) |
| (3) | authorization non-escalation, confused-deputy tests | **must be re-earned** | `.9.3.1` found five sites asking whether an authority EXISTS where the question is whether the caller HOLDS it, in four spellings, and grant ids are derivable from principal ids — the confused-deputy shape itself. `.3.3.2` found a thread-limited grant reaching a tenant target. `.3.4.2` found a revoked delegation replaying as a success with zero audit rows |
| (4) | SSRF / DNS-rebinding / redirect / archive-bomb suite | **narrowed** | R0 and R1 have OPPOSITE redirect designs. `fetcher.rs` sets `Policy::none()` and follows redirects manually, re-classifying every hop. `git.rs` sets `Policy::limited(5)` — auto-follow — with a DNS-resolver hook as its only destination control, while its own doc claims "every dial (redirect hops included) passes the destination policy". hyper-util skips the resolver for an IP-literal host, in its own words: *"If the host is already an IP addr (v4 or v6), skip resolving the dns and start connecting right away."* So the line stands for R0 and must be re-earned for R1. Source-measured; runtime reproduction is `SIGNOFF-REPAIR.7.2`'s, record `R-44-45-1` |
| (6) | dependency / SBOM / provenance / release-signing pipeline | **stands**, narrowed | No repair touched it; `make deny`, the secret scan and the signed release manifest are unchanged. One narrowing: the external dependency ledger still carries empty `tested_versions` for MCP and A2A, which §7.4 requires fresh at release gates (`.11.4`) |
| (7) | backup restore and compromised-key recovery exercise | **narrowed** | The restore exercise still runs in the owned runner. The fixture's own defects that `R-59-2` names are unrepaired: a hand-rolled URL parser (`url_parts`) that mishandles percent-encoded credentials and IPv6, and raw driver errors reaching the log. Owner `.11.3` |
| (8) | rate-limit, cost-circuit-breaker, notification-storm tests | **must be re-earned** | `.3.3.4.9` measured the spend breaker's two administrative verbs as "the weakest administrative path in the server": each admitted under a *shared* guard and then mutated **on the connection pool** — outside any transaction, under no guard, recording nothing about what it did. That was true when this line was counted as shipped |
| (10) | incident runbooks, contacts, evidence preservation, disclosure | **stands** | No repair touched it. It is documentation, and `SECURITY.md` plus the thirteen-family runbook catalogue ship unchanged |

- **The gate's CONCLUSION is unchanged: G6–G7 remains NOT MET for Internet exposure.** The
  three external preconditions are restated unchanged, with their triggers: the externally
  reviewed threat model (§2.6 — AI review does not satisfy it), the prompt-injection
  action-boundary suite (arrives with the first action-bearing exposure surface), and the
  penetration test (§19.6 — the release is cancelled, not waived, if critical or high
  findings stand). `SIGNOFF-REPAIR.13.1` carries them as blocker rows.
- **What changes is the internal half.** Internet exposure now has **three external blockers
  and one internal one**: three of the seven counted lines must be re-earned and two are
  narrowed, so five of the seven cannot currently be cited as-is.
- ⛔ **The gate record is not dishonest and this record must not be read as saying so.** Its
  claim is about the evidence its suites carried on its date, and that claim is true. The
  defect is a gate line counted against a suite whose *coverage* was never measured — the
  same shape as a test whose name promises more than it exercises, one level up.

## Consequences

- Any statement that §16.12's seven shipped lines are satisfied must now cite this record
  alongside the gate record. The book's blockers and qualification chapters do.
- Re-earning lines (2), (3) and (8) is not new work: the defects are repaired. What is
  missing is the *coverage measurement* that would let the line be counted again — which is
  a different activity from repairing, and is owned per line by the leaves named above.
- The R1 redirect finding is the only one of the seven that is still an **open defect**
  rather than a repaired one. It is `SIGNOFF-REPAIR.7.2`'s.

## answers:

- **Every repair postdates the gate record**, so "since" is exact rather than rhetorical:
  the earliest corrective commit is 2026-09-09 and the gate closed 2026-09-08.
- **Two of the seven stand.** Saying which is the point of a closed verdict set: a review
  that returned "all suspect" would be as useless as one that returned "all fine".
- **The R1/R0 contrast is the sharpest evidence in this record**, because it is two
  implementations of one property inside one product, with the weaker one documenting the
  stronger one's behaviour.
