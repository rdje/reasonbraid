# Small-swarm orchestration is the existing objective, and is deliberately unscheduled

- Date: 2026-09-18
- Status: accepted as DIRECTION; **unscheduled** — it does not displace the LAN bar
- Owner: director instruction, 2026-09-18; no task-tree leaf is opened (see *Why no leaf*)
- Related: `README.md` (the objective), `crates/reasonbraid-server/src/workflows.rs`
  (the step vocabulary), `docs/decisions/2026-09-18_lan-completeness-precedes-internet-exposure.md`
  (the bar this must not displace), `SIGNOFF-REPAIR.11.7.1` (the roadmap freeze),
  ROADMAP §13.1 (the built-in profiles), ADR-016.

## The ask

> *"I want ReasonBraid to [have] those capabilit[ies], to be able to orchestrate
> and coordinate the work of 2-5 agents for now, and maybe we can scale it
> afterwards. And this small or big swarm of agents should be able to work
> collectively on any given problem. Right now, it is only about programming
> issues, project governance and administration."*

Prompted by two external developments, recorded here with their dates because
both postdate this assistant's training data and neither is independently
established.

## The two reference points, as REPORTED rather than as established

⚠️ Both are press-reported, and the first is explicitly unverified. They are
recorded as context for a direction, never as evidence for a claim.

- **OpenAI / Navier–Stokes, reported 2026-09-08/09.** Roughly **10,000 agents**
  of an unreleased model were deployed on **2026-08-28** and produced a result in
  about **88 hours** — a blow-up result, that the equations can break down under
  extreme conditions. ⛔ Human researchers were in the loop throughout; the result
  is **not independently verified**; OpenAI states it will not claim the prize;
  and mathematician Tristan Buckmaster has publicly disputed attribution, saying
  the work muscled in on sub-questions he and others had solved.
- **xAI Grok 4.20, launched 2026-02-17.** **Four** specialised agents on ONE
  shared backbone — shared weights, prefix/KV cache and input context — with short
  RL-trained debate rounds, at a marginal cost near **1.5–2.5×** a single pass
  rather than 4×.

⭐ Those are OPPOSITE bets: Grok buys cheap tight coupling on one model; OpenAI
bought scale. ReasonBraid is neither, and that is the point below.

## The finding: this is the objective, not a pivot

`README.md` already states it — *authorized humans and agents can ask a durable
network a question without knowing who is online*, with Rust, not an LLM,
enforcing identity, authorization, ordering, budgets and publication.

And the machinery is less programming-specific than the current usage suggests.
The shipped workflow step vocabulary in `crates/reasonbraid-server/src/workflows.rs`
is `solicit`, `blind_solicit`, `critique`, `revise`, `synthesize`, `assess`,
`vote`, `adjudicate`, `decide`, `approve`, `moderate`, `present`, `retrospect`,
`evidence_request`, `quick_advice`. **Not one of those verbs is
programming-specific.** A profile is versioned CONFIGURATION over existing verbs —
that file is explicit that a step "names an existing contribution/terminal/budget
surface; nothing else is expressible, by construction" — so changing domain is
writing a profile, not changing code. Real `claude.rs` and `codex.rs` adapters
already dispatch to actual agent harnesses.

⭐ `blind_solicit` matters more than its size: contributions that cannot see each
other. That is genuine independence, and the roadmap explicitly REJECTED
"response similarity as independence" as an unsafe idea inherited from a
predecessor. For swarm work that rejection is an asset, not a limitation.

## The two honest constraints

**1. The binding constraint is the ACCEPTOR, not orchestration.** The premise is
that model output stays untrusted until DETERMINISTIC rules accept it. Programming
has real acceptors — compilers, test suites, this repository's own doctrine gates.
"Any given problem" frequently has none. The Navier–Stokes episode is the
illustration: a swarm produced something, and the unresolved difficulties are that
humans cannot verify it quickly and cannot agree who contributed what. ⭐ Those two
difficulties are precisely what this platform is for, which is the differentiator
worth leaning into rather than competing on agent count.

**2. Two to five agents is in range; thousands is a different system.** The
current design serialises administrative acts on one site guard row and runs
per-tenant transactions under budget admission — correct for small-N governed
deliberation, wrong for a ten-thousand-agent scheduler. ⛔ "Scale afterwards" is an
explicit re-architecture decision when it comes, not an extrapolation from this
one.

## Why no leaf is opened

Deliberately, and this is the ownership rather than an omission. The LAN bar was
defined hours earlier (G0–G5 plus G7 on the LAN); opening a leaf here would put
work on the frontier that the director has just ranked below it. The direction is
recorded so it survives the session, and it is scheduled when the director says so.

⚠️ **And part of it is already inside the LAN bar.** `blind_solicit`, rounds,
quorum and adjudication are G3 governance and G5 quality surfaces, and G5 is
currently *withdrawn*. Re-earning G5 therefore exercises much of this capability
anyway — the two are not in competition for most of the distance.

## The named first workload: `rdje/bedrock`

Named by the director on 2026-09-18: **help build `github.com/rdje/bedrock`**, the
template every project of theirs should adopt, distilled from PGEN's doctrines,
policies and rules. Cloned locally at `/Volumes/SSD/Documents/github/bedrock`
(same volume, §13) and actively maintained under its own `BEDROCK-MAINTENANCE`
tree at `DOCTRINE_VERSION 0.6.1`.

⭐ **It is an unusually good first workload, for the precise reason this record
worries about elsewhere: it SOLVES the acceptor problem.** "Is this template
good?" has a deterministic answer — clone it fresh, bootstrap, and see whether a
first commit passes the template's own gates. bedrock's own `BEDROCK-MAINTENANCE-0010`
already runs exactly that trial (clone → bootstrap 13/13 → commit green → `make
gate` green → `make check` green, idempotent re-run). A domain with a mechanical
acceptor is rare, and it is what makes this a fair first test rather than a demo.

⭐ **And reasonbraid is the best available evidence source for it, with no new
capability required.** This repository has run the spine hard for 258 commits and
knows which doctrines FIRE. Measured 2026-09-18, comparing registered enforcer
arrays: **bedrock registers 11; reasonbraid registers 18 plus 2 conditional; the
difference is 7 doctrines reasonbraid has and bedrock does not, and 0 the other
way** — `FRONTIER-STATUS`, `HEADING-DEPTH`, `INDEX-FRONTIER`, `LOCKSTEP-CLAIM`,
`RUST-FORMATTING`, `SELF-TEST`, `TASK-STATUS`. Every one is spine-shaped rather
than domain-shaped, so every one is a candidate. ⚠️ `RUST-FORMATTING` is
language-specific and would need to be conditional in a general template.

⛔ **What makes that list worth more than the scripts is the EVIDENCE attached to
each**, which a template cannot generate for itself: a founding defect, a cost
against the enforcer, and whether it has ever caught anything. `SELF-TEST` catches
nothing today and prevents a class; `FRONTIER-STATUS` (0.06 s) caught its own
author within the hour; `RUST-FORMATTING` (0.66 s) caught a defect two commits
old; `DOCTRINE-REGISTRY` refused a backticked description that the driver would
have executed. A template shipping doctrines is a claim; a template shipping
doctrines *with their firing records* is a measurement.

⛔ **bedrock is a GITHUB TEMPLATE REPOSITORY, not a syncable remote** (director, 2026-09-18). A project is instantiated through GitHub's *Use this template* — the whole repository is copied — then cloned and bootstrapped locally, which is exactly this repository's own first two commits (`b932c05 "Initial commit"`, `823c2bc "bootstrapped from bedrock"`), and there is **no bedrock remote**. ⚠️ So there is no pull, no fork relationship and no sync path in either direction: `update_scaffold.sh` here names *reasonbraid* as the template source for reasonbraid's OWN descendants. Anything that reaches bedrock is carried by hand.
⛔ **bedrock is a SEPARATE repository with its own task tree.** Nothing here may
edit it; work there is owned by `BEDROCK-MAINTENANCE` leaves, and the scaffold
direction already runs the other way (`scripts/update_scaffold.sh`).

## What to do first when it IS scheduled

Not "support any problem", which is unfalsifiable. **Add ONE non-programming
domain as a profile plus a STATED ACCEPTOR, and see what breaks.** It is cheap —
configuration, not code — and it tests the actual claim: can the existing verbs
carry a domain whose acceptor is not a test suite? Name the acceptor before the
profile; a domain with no acceptor is a domain this platform cannot honestly serve
yet, and saying so is the finding.

⛔ **If this changes the OBJECTIVE rather than scheduling work under it, it is a
roadmap-version decision, not a task-tree one.** `ROADMAP.md` is frozen and
`SIGNOFF-REPAIR.11.7.1` is blocked on exactly that question.
