---
answers:
  - What must pass before ReasonBraid is reachable on the Internet by its owner alone?
  - What must pass before it is opened to other people or organisations?
  - Is the independent security review still required, and when?
---
# The Internet gate opens owner-only first

- **Type:** decision
- **Status:** active — amends `ROADMAP.md` §16.12 for one profile; to be folded into v0.5.0, since v0.4.1 is frozen
- **Owner:** `SIGNOFF-REPAIR.14` (the exposure candidate), `PARTICIPATION.4` and `.5` (sign-in, the web client, the connector), `SIGNOFF-REPAIR.13.1` (the external lines)
- **Date:** 2026-09-25
- **Work unit:** `REASONBRAID-DOC-0175`
- **Source:** the director chose option (B) of the three offered the same day (keep the gate as written; split it; drop the independent review).

## The fact / decision

The director, verbatim: *"The initial bring will consist of only me and the agent I have access to using my own account to their platforms so (1) should be the first route to take. Once I and you are confident REASONBRAID is really good then we can think of opening it to other people or organizations, but we need to do it with extrem care."*

§16.12's Internet gate becomes two gates, one per profile.

**1. The owner-only Internet profile comes first.** ReasonBraid is reachable over HTTPS with OAuth sign-in, and only the director's own identities can sign in: their human account, and their own agents on the platforms they use through their own accounts. No other person's or organisation's data is on the server. It is gated by our own evidence, all of it required:

- the eight §16.12 lines this project can produce itself: authenticated enrollment, rotation, revocation and tenant-isolation tests; authorization non-escalation and confused-deputy tests; the SSRF, DNS-rebinding, redirect and archive-bomb suite; the prompt-injection action-boundary suite; the dependency, SBOM, provenance and release-signing pipeline; the backup-restore and compromised-key recovery exercise; the rate-limit, cost-circuit-breaker and notification-storm tests; and the incident runbooks, contacts, evidence preservation and disclosure process;
- the owner-only property itself, as a control: an identity not on the owner's list is refused at sign-in and at every tool call, observed failing first;
- the internal test programme: correctness (each behaviour observed failing first; OAuth 2.1 conformance with PKCE, resource indicators and token-audience validation; MCP conformance; web-client end-to-end tests), robustness (fuzzing every input parser, property tests, malformed input, kill-and-restart), stability (load and soak tests, connection and rate limits, memory and time bounds) and security (cross-organisation and escalation tests, an automated web-security scan of the running server, a TLS configuration check, mutation testing of security-critical code, and an adversarial review by a separate agent);
- the corrective exit bar met first (`REASONBRAID-DOC-0162`), which the director agreed.

**2. Opening to anyone else keeps the whole of §16.12,** including the two lines that need an independent party: the externally reviewed threat model and the penetration test with critical and high findings resolved or the release cancelled. It happens only when the director and the maintainers are both confident, and with the care the director asked for. `docs/runbooks/external-security-review.md` is its checklist.

## Why

- **The risk the independent review guards against is other people's data.** The code here was written and tested largely by AI agents, and shared blind spots are real: the corrective programme found dozens of cross-organisation defects in code that had passed its own tests. While only the owner's own data is on the server, the owner carries that risk knowingly; once anyone else's data is, an independent reviewer is the control this project cannot give itself.
- **The owner-only profile still clears every line the project can produce,** so the split moves only the two external lines, and only to the point where they protect someone.
- **Owner-only is enforced, not assumed.** A profile described as owner-only but reachable by any OAuth identity would be the open profile under another name, so the allowlist is a tested control of the gate.

## How to apply

- Build the candidate for the owner-only profile first (`SIGNOFF-REPAIR.14`, `PARTICIPATION.4` and `.5`), after the corrective exit bar.
- Serve the owner-only profile only when its gate record, listing the evidence above, is signed by the director.
- Nothing admits a second person or organisation until the full §16.12 record, with the independent review and the penetration test, is signed.
