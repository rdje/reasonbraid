---
answers:
  - Has the director decided to expose ReasonBraid on the Internet, and on what terms?
  - Is agent-to-agent direct addressing wanted, and is it doable?
  - Must the web console do everything the rb CLI does?
  - Which deferred leaves did the Internet decision reopen?
---
# The director greenlights Internet exposure, security first

- **Type:** decision
- **Status:** active
- **Owner:** `PARTICIPATION` (the requirements); `SIGNOFF-REPAIR.13.1` and `SIGNOFF-REPAIR.14` (the Internet gate's external lines and the exposure candidate)
- **Date:** 2026-09-25
- **Work unit:** `REASONBRAID-DOC-0173`
- **Source:** the director's point-by-point answers of 2026-09-25 to `docs/decisions/2026-09-25_any-human-or-agent-can-take-part.md`.

## The fact / decision

The director decided, in their words:

1. **The node inbox's cap stands:** *"64 items is perfectly fine."*
2. **Direct addressing is wanted:** *"Can we add support for that? Is it even doable?"* — yes; see below. `PARTICIPATION.2` owns it.
3. **The web console must match the CLI:** *"I would like the web console to be able to behave exactly like the rb CLI tool. users shall be able to use either client types and an extremely secure and super intuitive way."* `PARTICIPATION.4` owns it.
4. **Internet exposure is greenlit:** *"I greenlight ReasonBraid to connect to the internet over HTTPS and fully support sota OAuth sign-in support"*; *"I grant ReasonBraid fully access to the Internet. Security here will be paramount, top tier priority."*
5. **Security first, without exception:** *"Robustness and security are non-negotiable and suffer no exception and no compromission"*, and the director agreed the open cross-organisation defects close before the doors widen.

## What the greenlight changes, and what it does not

- ✅ **It fires two deferred triggers.** `REASONBRAID-DOC-0162` deferred `SIGNOFF-REPAIR.13.1` (the external G6 preconditions) and `SIGNOFF-REPAIR.14` (the exposure-profile candidate) on *"a decision to claim Internet exposure"*. That decision is now taken, so both are reopened as work: `.13.1` prepares what the external review needs, and `.14` builds and freezes the candidate after the corrective exit bar.
- ⛔ **It does not meet the Internet gate, and it cannot.** §16.12 blocks Internet-capable deployment until ten lines pass, and §19.6 makes G6 need *"Section 16.12 evidence and external review"*. Two lines need an independent party: an externally reviewed threat model (line 1) and a penetration test whose critical and high findings are resolved or the release is cancelled (line 9). §2.6 says AI review does not satisfy an independent-review requirement. The greenlight authorizes the work towards exposure and the engagement of that reviewer; it does not replace the review, and *"no exception and no compromission"* rules out waiving it. The step-by-step path is `docs/runbooks/external-security-review.md`.
- **It refines, and does not reverse, the instruction of 2026-09-18** (`docs/decisions/2026-09-18_lan-completeness-precedes-internet-exposure.md`: *"It needs to fully work on the local network first"*). The corrective exit bar is the point at which the local-network path counts as complete, so B1–B3 stay `yes (deferred)` until it is met, and that is now their stated resumption trigger. The director's agreement that cross-organisation defects close first says the same thing.
- **Order:** the corrective blocking leaves; then the candidate (`.14`, `PARTICIPATION.4` and `.5`) built and frozen; the threat model made review-ready (`.13.1`); the external review and test; fixes and retest; the director signs the gate record or cancels. Nothing is served publicly before that signature.

## Direct addressing: yes, and how

It is doable, and most of the machinery exists. One agent sends a message to another by the recipient's agent role:

- **The message** is durable: sender, recipient role, a kind (question, bug report, feature request, reply, other), the body, what it replies to, and an expiry.
- **Who may send to whom** is an authorization rule, not an open mailbox: within one organisation by default, across organisations only where a federation agreement allows it, rate-limited, and refusable by the recipient, so nobody can flood an agent's inbox.
- **Delivery** reuses the node inbox's proven parts: written whether or not the recipient is online, replayed from a durable cursor on reconnect, deduplicated, acknowledged. A chat-app participant reads the same inbox through the connector's *check my messages* tool.
- **Every message is audited** and bound to its organisation, like every other write here.

`PARTICIPATION.2` designs it after `PARTICIPATION.1` measures what exists; its security class is stated before it opens.

## Why

- The greenlight is the director's to give, and the trigger rule of `REASONBRAID-DOC-0162` makes it fire the two leaves mechanically rather than by memory.
- An independent review is the one control this project cannot give itself, and the director's own words (*"no exception"*) are the reason not to treat the greenlight as that review.

## How to apply

- Build toward exposure freely; serve nothing publicly until the G6 gate record is signed.
- Every leaf in `PARTICIPATION` and `.14` states its security class under the bar before it opens.
- `docs/runbooks/external-security-review.md` is the checklist for the director's part and the project's part of the external review.
