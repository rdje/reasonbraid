---
answers:
  - How do B1, B2, B3 and B4 get unblocked?
  - Is building the Internet exposure profile blocked by the qualification gate?
  - Should the external security review be commissioned commercially or through an OSS audit programme?
  - Which jurisdictions does the ReasonBraid name clearance cover?
  - Where do people report a vulnerability?
---
# The Internet-qualification route: build the candidate, then commission the review against it

- **Type:** decision
- **Status:** accepted
- **Owner:** `SIGNOFF-REPAIR.14` (the candidate lane); the blocker register is `SIGNOFF-REPAIR.13`
- **Date:** 2026-09-16

## The structural finding that reorders everything

`docs/decisions/2026-09-08_phase7-subtraction-record.md` records four items and
their revisit triggers:

| | Not built | Revisit trigger |
| --- | --- | --- |
| S-1 | the Internet exposure (public enrollment endpoint, mTLS serve wiring, public listener) | "the three gaps close" |
| S-2 (**B1**) | the externally reviewed threat model | "the external reviewer engages" |
| S-3 (**B3**) | the prompt-injection action-boundary suite | "the exposure profile's first action-bearing surface" |
| S-4 (**B2**) | the penetration test | "the exposure profile's **candidate freeze**" |

That is a circular wait: S-1 waits on the gaps, and two of the three gaps wait on
S-1. **It is an artefact of S-1's stated reason, not of the gate.**

- §16.12: *"Internet-capable **deployment** is blocked until all of these pass."*
- §25.1: *"Before Phase 7, do not **expose** remote enrollment to the Internet if
  the qualification gate is incomplete."*

Both gate *deployment* and *exposure*. Neither forbids **building** the
candidate. Building a frozen, un-deployed exposure profile is permitted, and it
is the single prerequisite B1, B2 and B3 share.

## Decision 1 — build the exposure-profile candidate, un-deployed

**Accepted.** `SIGNOFF-REPAIR.14` owns it. It is a candidate: the code and the
declared profile exist, the gate still forbids turning it on, and no commit in
this lane may deploy it or claim qualification.

⭐ **It starts closer than the subtraction record suggests.**
`crates/reasonbraid-server/src/mtls.rs` already builds a complete
`rustls::ServerConfig` with a `WebPkiClientVerifier` — and
`grep -rn "mtls::" -- crates` finds its only callers in `tests/mtls.rs:42` and
`:72`. `grep -c mtls crates/reasonbraid-server/src/bin/rb-server.rs` returns
**0**: the server never serves TLS. This is a finished module with no production
caller, the same shape `.13.1.1` records for `verify_ladder`.

## Decision 2 — apply to an OSS audit programme first, with a dated fallback

**Accepted, with a trigger rather than a hope.** The routes are not equivalent in
cost and this project's situation picks one:

- The repository is **public**, the software is **unreleased**, and there is no
  revenue. That is precisely the profile OSS audit programmes exist for.
- §2.6 requires **independence**, not procurement: *"Independent reviewer
  required for Internet qualification"*. A programme-arranged auditor satisfies
  it exactly as a purchased one does.
- B2's own trigger is the candidate freeze, so the application window overlaps
  the build at no cost to the schedule.

⛔ **The fallback is dated, because "we applied" is not a plan.** If no programme
has engaged **twelve weeks after the candidate freeze**, commission commercially.
Recorded so the wait has an end.

⚠️ Programme eligibility criteria change and none is asserted here as accepting
this project. Checking eligibility is the lane's first task, not this record's
claim.

## Decision 3 — the vulnerability channel is GitHub private reporting

**Accepted, and it publishes no personal address.**

`SECURITY.md` currently says, in its own words, *"this policy does not assert a
dedicated reporting endpoint exists."* §16.12's last line requires *"incident
runbooks, contacts, evidence preservation, and disclosure process"*, and no
channel also closes off every independent-findings route that could feed B1/B2.

The channel is **GitHub private vulnerability reporting** on
`github.com/rdje/reasonbraid`:

- authenticated and confidential, with no address to publish or rotate;
- it creates a draft security advisory, which is the evidence-preservation and
  coordinated-disclosure machinery §16.12 asks for, already built;
- it needs no new infrastructure and no personal data in a tracked file.

⛔ **Deliberately NOT an email address.** A personal address in a public file is
permanent, unrotatable and belongs to a person rather than to the project.

⚠️ **One action is the repository owner's and cannot be done from the tree:**
enable *Settings → Code security → Private vulnerability reporting*. `SECURITY.md`
states the channel and marks it pending until that switch is on, rather than
promising a route that does not yet accept reports.

## Decision 4 — the name clearance covers three jurisdictions, and the protocol is written down

**Accepted.** ADR-001's rollback trigger asks for *"professional trademark,
company/product, package, executable, domain, app-store, and repository clearance
in intended jurisdictions"* and never said which. They are:

1. **United States** — crates.io, GitHub and the app stores are US entities, so a
   US conflict is the one that can actually remove a published artifact.
2. **European Union** — one EUIPO filing covers 27 member states, and the OSS
   funding bodies most likely to engage under Decision 2 are EU-based.
3. **The holder's own jurisdiction** — where a company or trading name would be
   registered.

⭐ **The rename cost is measured, not feared**, which is what makes deferring the
professional search defensible: the name appears in **67 source files, 191
lines** — 144 as crate-path identifiers (`reasonbraid_*`), 18 as prose
`ReasonBraid`, 64 hyphenated — and every shipped binary is already neutral
(`rb`, `rb-server`, `rb-site`, `rb-node`, `rb-journal`, `rb-release-manifest`).
A collision is a bounded mechanical rename of crate names and import paths.

⛔ **B4 blocks package, domain and marketing release only.** It does not block the
repository being public (settled 2026-09-09), any repair, or Decisions 1–3.

## What this record does NOT claim

It does not claim any programme will engage, that the searches have been run, or
that building the candidate advances the gate by itself. The gate is **NOT MET**
and B1–B4 remain open. What changes is that three of the four now have a
named next action inside this repository instead of a wait.
