# External security review: the threat-model review and the penetration test

The two Internet-gate lines this project cannot produce for itself (`ROADMAP.md`
§16.12, lines 1 and 9; register rows B1 and B2, `SIGNOFF-REPAIR.13.1`). §2.6 requires
an **independent** security reviewer for Internet qualification and says AI review
does not satisfy it, so neither the maintainers nor an AI agent can clear these.
This runbook says, step by step, what the **director** does, what the **project**
does, and what evidence closes each line. It follows the route
`docs/decisions/2026-09-16_internet-qualification-route.md` decided: build a frozen
candidate, apply to an open-source audit programme, and commission commercially if
none has engaged twelve weeks after the freeze.

Written 2026-09-25, after the director greenlit Internet exposure over HTTPS with
OAuth sign-in (`docs/decisions/2026-09-25_the-director-greenlights-internet-exposure-security-first.md`).

⚖️ **When this runbook applies:** before ReasonBraid is opened to ANY person or
organisation other than its owner. The director chose, the same day, to go
owner-only first (`docs/decisions/2026-09-25_the-internet-gate-opens-owner-only-first.md`):
that profile is gated by the eight §16.12 lines this project produces, an internal
test programme and a tested owner allowlist, and does not wait for this review.

## What is already done

- ✅ **The vulnerability channel is live.** GitHub private vulnerability reporting is
  enabled on `github.com/rdje/reasonbraid` (checked 2026-09-25 through the GitHub API:
  `enabled: true`), and `SECURITY.md` names it. §16.12's last line needs it, and an
  auditor will ask for it.

## The order, and why it is this order

A penetration test examines a **running system**, and a review of a threat model
examines a **design**. Paying for either before the thing it examines is stable
wastes the engagement: findings land on code that is about to change, and the retest
is against a different system. So:

1. **Project:** close the corrective blocking items (cross-organisation, integrity,
   false claims, lying gates). The director agreed this comes first.
2. **Project:** build the Internet exposure candidate and FREEZE it
   (`SIGNOFF-REPAIR.14`, `PARTICIPATION.4` and `.5`): the HTTPS listener, OAuth
   sign-in, the remote MCP connector, the web client, and nothing turned on for the
   public.
3. **Project:** make the threat model and the test scope fit for review (step P1,
   P2 below). This can start before the freeze and should be done by it.
4. **Director:** engage the reviewer (steps D1–D5). Start the programme applications
   as soon as the scope document exists; programmes take time to answer.
5. **Reviewer:** reviews the threat model, then tests the frozen candidate.
6. **Project:** fixes every critical and high finding; **reviewer** retests.
7. **Director:** as release authority, signs the gate record, or cancels.

## Director's steps (only a named human can do these)

### D1 — Decide the route and the date it falls back

- **Route A, an open-source audit programme.** The repository is public, the software
  is unreleased and there is no revenue, which is the profile these programmes exist
  for. Candidates to check (eligibility changes; the project checks it before you
  apply, and none is asserted to accept this project): OSTIF (the Open Source
  Technology Improvement Fund), the OpenSSF's Alpha-Omega, Germany's Sovereign Tech
  Agency, and NLnet's NGI funds (their audits are carried out by partner firms).
- **Route B, a commercial engagement.** Pick firms with published reports on Rust
  services, OAuth 2.1 and multi-tenant authorization. Ask two or three for a
  fixed-price quote for the scope in P2.
- **The fallback date is already decided:** if no programme has engaged twelve weeks
  after the candidate freeze, go commercial. Nothing to decide here but the budget.

### D2 — Choose the reviewer by these criteria

Ask each candidate, in writing:

1. **Independence:** no one on the team contributed code, design or advice to
   ReasonBraid. (That is what §2.6 means; a friend who helped design it does not
   qualify.)
2. **Relevant work:** a sample or public report covering web APIs with OAuth 2.1,
   multi-tenant authorization (one customer reaching another's data) and Rust.
3. **Method:** they test against a named standard. Ask for OWASP ASVS level 2 at
   least, the OWASP Web Security Testing Guide, and the MCP specification's security
   guidance for the connector.
4. **White-box:** they read the source as well as attack the running system. The
   repository is public; this is cheaper and finds more than a black-box test.
5. **Severity scale:** every finding rated with CVSS (version 3.1 or 4.0), so
   "critical" and "high" mean the same thing to everyone.
6. **Retest included:** the price includes one retest of the fixed findings and a
   **retest letter** stating which are resolved.
7. **Threat-model review included:** the same firm reviews the threat model before
   testing. One engagement, one report, one retest is the simplest path.

### D3 — Sign the engagement

You sign as the release authority §2.6 names. The documents:

- **Statement of work:** the scope from P2, dates, deliverables (threat-model review
  memo, test report, retest letter), price, and the retest.
- **Rules of engagement:** exactly which hosts and accounts they may attack, the test
  window, who to call if something breaks, and that no real user data exists on the
  test system. The project drafts this (P3); you sign it.
- **Confidentiality:** the report stays private until findings are fixed; the
  summary and the retest letter may be published.

### D4 — Be the named contact during the test

A tester who finds something severe (for example, one organisation reading
another's data) calls the contact immediately rather than waiting for the report.
That contact is you, or someone you name. The project triages within the day.

### D5 — Sign the gate record, or cancel

When the retest letter says every critical and high finding is resolved, you sign
the G6 gate record (the project prepares it, step P6). If a critical or high finding
cannot be fixed, §16.12 says the release is **cancelled**, not waived, and you record
that instead. Medium and low findings are either fixed or recorded as known limits
with an owner.

## Project's steps (the maintainers and the agents working in this repository)

### P1 — Make the threat model fit for review (`SIGNOFF-REPAIR.13.1`)

`spec/threat-model.md` is an 85-line skeleton from 2026-09-06, written before the
HTTPS, OAuth and connector work existed. A reviewer needs, for the frozen candidate:

- a data-flow diagram per surface: the HTTPS API, OAuth sign-in, the remote MCP
  connector, the web client, the node channel, acquisition of web resources;
- the trust boundaries on that diagram and what crosses each;
- the assets (§16.1) and who must never reach them;
- threats per boundary, by STRIDE, and the abuse cases §16.6 and §16.11 name,
  including prompt injection through a connector;
- for each threat, the control that answers it **and the test that proves the
  control**, by file and test name;
- the residual risks, stated plainly.

### P2 — Write the scope and environment document

- The frozen candidate's commit, and how to run it.
- The URLs in scope; everything else out of scope.
- **Test accounts:** at least two separate organisations (tenants), each with an
  administrator, an ordinary member and an agent, so the tester can try to cross
  from one to the other. Site-operator credentials in a separate, stated account.
- The connector: how to add the test server to ChatGPT, Claude and Gemini as a
  custom connector.
- What must hold (the properties to attack): no cross-organisation reads or writes,
  no privilege escalation, budgets cannot be overspent, evidence cannot be forged,
  publications cannot be forged, sign-in cannot be bypassed, no server-side request
  forgery through resource acquisition.

### P3 — Stand up the test environment and draft the rules of engagement

A dedicated instance on its own host and database, seeded only with test data,
reachable by the testers, and destroyed after the retest.

### P4 — The other eight §16.12 lines are ours, and they must pass before the test

The reviewer should find what we could not, not what our own gate lists. Before the
engagement starts, the prompt-injection action-boundary suite (B3) and the other
local lines pass and are recorded.

### P5 — Triage, fix, and prepare the retest

Every finding becomes a task-tree leaf the day it arrives, classified by the bug bar
(`docs/decisions/2026-09-25_the-corrective-tree-ends-at-a-bug-bar.md`): critical and
high block the release. Each fix is observed failing first and gets a control, as
every repair here does.

### P6 — Prepare the gate record

For each of lines 1 and 9: the reviewer's name, the dates, the report's SHA-256
digest (the report itself may stay private), the findings and their dispositions,
and the retest letter. The book's blockers page and `LIVE_STATUS.md` change B1 and
B2 to met only when that record exists and the director has signed it.

## What closes each line

| Line | Closed by | Not closed by |
| --- | --- | --- |
| 1 — externally reviewed threat model and abuse cases | a written review memo from an independent reviewer, its findings answered, and the director's signature on the gate record | a threat model we wrote and reviewed ourselves, or an AI review |
| 9 — penetration test, critical and high findings resolved or release cancelled | a report from an independent tester against the frozen candidate, a retest letter saying every critical and high finding is resolved, and the director's signature | a test by the maintainers, a scan by an automated tool alone, or findings "accepted" instead of fixed |
