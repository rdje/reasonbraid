---
answers:
  - Is ROADMAP §13.5's moderator in scope for the shipped thread engine?
  - Which of its six permitted acts exist as verbs today, and which do not?
  - What enforces each of its six prohibitions, rather than stating them as prose?
  - Why is the benchmark's moderator not the product's moderator?
  - What must exist before the role can be built, and why is building it first wrong?
  - Why is this leaf's count of the prohibitions corrected here?
---
# The moderator is a role boundary, not a feature — five of its six acts already ship, and none of them is bounded

- **Type:** decision
- **Status:** accepted
- **Owner:** `SIGNOFF-REPAIR.11.4.7.2.1.3`
- **Date:** 2026-09-22
- **Cites:** ROADMAP §13.2 (the deliberation reference flow), §13.5 (moderator and
  synthesizer boundaries), §13.6 (anti-herding), §19.5 (deliberation evaluation);
  `docs/decisions/2026-09-22_a-decision-rule-is-a-charter-scoped-vocabulary-and-never-ships-without-its-tally.md`
  (the sibling contract this depends on);
  `docs/decisions/2026-09-22_the-remaining-roadmap-gaps-are-sequenced-by-exposure-then-deletion.md`
  (this is that record's item 5, and the last wave-B decision)

## Context

`.11.4.7.2.1` split deferral #3: the synthesizer landed, the moderator did not. This leaf
owns whether §13.5's moderator is in scope for the shipped engine at all.

🔴 **A correction first, because the leaf miscounted its own subject.** Its acceptance asks
for *the three prohibitions*. §13.5 lists **six**: add a vote or approval; suppress a visible
dissent except through an appealable moderation action; fabricate evidence or silently change
citations; change electorate, quorum, or proposal digest; authorize more spend or tools;
publish or deploy policy. All six are adjudicated below.

## The measurement

✅ **The leaf's own measurement re-derives exactly.** `git grep -c "moderator" -- crates/reasonbraid-server/src crates/reasonbraid-core/src` returns **zero**; every `moderator` in the workspace is `reasonbraid-adapter`'s benchmark (`bench/v1/*.json`, `src/bench/*`, `src/bin/rb-bench.rs`, `tests/bench_harness.rs`) or the CLI verb that drives it.

⛔ **And the benchmark arm is NOT the product role**, which is the distinction that made this
half invisible for the life of the deferral. §19.5's benchmark measures a *moderator/synthesis
flow* as one **routing shape** among four, to compare deliberation topologies. §13.5's
moderator is a **governed participant in a real thread**, with acts it may perform and six it
may not. One is a measurement arm; the other is an authority scope. They share a word.

### §13.5's six permitted acts, each mapped to a verb or named absent

| # | permitted act | verb today | verdict |
| --- | --- | --- | --- |
| 1 | classify messages | `ContributionKind` — `Position`, `Claim`, `Assumption`, `EvidenceReference`, `EvidenceRequest`, `Question`, `Summary`, `Verdict`, `Assessment` | ⚠️ **the act exists, the moderation does not** — the kind is declared by the AUTHOR on their own contribution, never by anyone over another's |
| 2 | request clarification | `ContributionKind::Question`, and `EvidenceRequest` which targets ONE claim digest of this thread | ⚠️ exists as an ordinary participant act |
| 3 | propose round closure | `thread.advance_round` | ⚠️ exists, and is gated only by `ensure_participant` — **any** participant may advance, and nothing merely *proposes* |
| 4 | enforce format/length | — | 🔴 **absent: no verb at all** |
| 5 | identify unanswered claims | `CoverageItem { item, included, reason }` — the coverage report showing which objections and uncertainty were included, and the reason when excluded | ⚠️ exists as the SYNTHESIZER's act, riding the synthesis |
| 6 | draft summaries | `ContributionKind::Summary` carrying `SynthesisInput` (synthesizer identity/configuration, input event range, source links, coverage report) | ⚠️ exists as the synthesizer's act |

⭐ **Five of six exist as verbs, and NONE of them is bounded to a role.** That is the finding:
**§13.5 is a role boundary, not a feature list.** What is missing is not the ability to
classify, ask, propose, summarize or report coverage — it is a principal *restricted to* those
acts. Reading the deferral as *build a moderator feature* is what made it look large.

### §13.5's six prohibitions, stated as what would ENFORCE them

A prohibition over a role that does not exist cannot be tested, so each is given the mechanism
that would carry it if a moderator existed tomorrow.

| # | prohibition | what enforces it today |
| --- | --- | --- |
| 1 | add a vote or approval | ⚠️ **split.** The APPROVAL half is enforced — `lifecycle::approve` re-checks the grant (live, held by the approver, and the approver IS the authenticated caller; `.9.3.1`). 🔴 The VOTE half has nothing to enforce, because the ballot does not exist (`.8.1.1`) |
| 2 | suppress a visible dissent | ✅ **structurally impossible.** `git grep -niE "DELETE FROM thread_contributions\|UPDATE thread_contributions" -- crates/reasonbraid-server/src` → **no match**: contributions are append-only, so there is no suppression verb to restrict |
| 3 | fabricate evidence or silently change citations | ✅ `snapshots::submit` refuses a `raw_digest` that is not the digest of the bytes handed to it, and citations are rows in `evidence_citations` |
| 4 | change electorate, quorum, or proposal digest | ⚠️ **split three ways.** The proposal digest is SERVER-COMPUTED, so a forged one fails the membership check. The electorate does not exist. 🔴 The quorum is whatever the approver declares — `.11.4.7.2.1.2.3`'s open defect |
| 5 | authorize more spend or tools | ✅ the grant's spend bound covers the declared budget and a request exceeding it is refused (`api.rs:4965`, `:4978`) |
| 6 | publish or deploy policy | ✅ grant-gated — staging and publishing both carry `owning_authority` (`.9.2.1.2`) |

## Decision

**1. §13.5's moderator is IN SCOPE, and it is an AUTHORITY SCOPE rather than a deliberation
feature.** Five of its six acts are shipped verbs; what does not exist is a participant role
constrained to them. So the work is in the role and grant vocabulary, not in the thread engine.

**2. ⛔ It is BLOCKED, and on a named thing rather than on judgement.** Two of its six
prohibitions — *add a vote* and *change … quorum* — name mechanisms that do not exist or are
not yet derived. A moderator built today would be forbidden from touching a ballot there is
none of, and forbidden from changing a quorum that any approver can already declare freely.
**Enforcing a prohibition against nothing is not enforcement**, and the resulting tests would
pass vacuously. Blocked on `.8.1.1` (the ballot) and `.11.4.7.2.1.2.1`/`.2.3` (the charter's
thresholds and the derived quorum).

**3. ✅ The `enforce format/length` act is DECLINED, separately and on its own merits.** It is
the one act with no verb, and it is the only one of the six that is a *content* control rather
than a *procedural* one. §13.6's anti-herding measures deliberately constrain form — *blind
initial positions*, *randomized order*, *no display of vote totals before a configured
commitment point* — as properties of the workflow profile, not as a moderator's discretion. A
per-message length or format ruling exercised by a principal is the shape §13.6 is written
against. If it returns, it returns as a profile property.

**4. ⛔ The benchmark arm is NOT the product role and is not superseded by this record.**
`reasonbraid-adapter`'s moderator stays exactly what it is — §19.5's routing-shape measurement.
This record does not touch it, and no future leaf should read its existence as coverage.

**5. Owned by `.11.4.7.2.1.3.1`**, sequenced after the two blockers.

## Consequences

- ⭐ **The leaf did delete part of itself, as `.15` predicted the cheapest item might** — one
  of the six acts is declined outright, and the other five are re-described as already shipped.
  What survives is smaller and differently shaped than *build a moderator*.
- ⚠️ **Four of the six prohibitions are enforced TODAY by mechanisms nobody wrote for §13.5** —
  append-only contributions, digest-verified snapshots, grant-bounded spend, grant-gated
  publication. That is worth recording because it means the role is cheap when its blockers
  clear, not because it retires the requirement.
- 🔴 **The two unenforced prohibitions are the SAME two open defects the sibling leaf found**,
  reached from a completely different direction. `.11.4.7.2.1.2` found the declared-not-computed
  quorum by reading the approval path; this leaf found it by asking what would stop a moderator
  changing one. ⭐ **A defect reached twice by independent routes is the strongest evidence this
  pass produces**, and it is the same signal that made `.9.3.5` a lane.
- ⚠️ **`.8.1.1`'s scope is unchanged by this record.** The moderator does not add to the ballot;
  it only cannot be bounded until the ballot exists.

## What would make this wrong

- ⛔ If *enforce format/length* is read as a transport-level bound (a maximum contribution size)
  rather than a moderator's editorial ruling, decision 3 is answering a different question — and
  that bound may already exist as a request limit. The reading taken is §13.5's frame, which
  lists it beside *classify* and *request clarification* as discretionary acts of a person.
- ⛔ If a deployment needs a moderator before the ballot exists — a human chair of an advisory
  thread that never votes — decision 2's blocker is too strong for that case, and the role could
  ship with prohibition 1 recorded as not-yet-applicable. Nothing asks for that today.
- ⛔ If §19.5's benchmark is ever made to drive real threads, decision 4 inverts and the two
  moderators must be reconciled rather than kept apart.

## Alternatives considered

1. **Supersede this half of the deferral: the benchmark arm is the only intended home.**
   Rejected, and it is the reading this leaf existed to test. §13.2's reference flow says *the
   moderator proposes scheduling and summaries; it cannot alter votes, authority, or evidence
   records* — that is a product sentence about a governed participant, in the deliberation
   chapter, not a benchmark sentence.
2. **Build the role now and record prohibitions 1 and 4 as future work.** Rejected: it ships a
   role whose test suite passes vacuously on a third of its contract, which is the
   `AcceptedByRule` defect in a new place.
3. **Build only the five shipped acts under a `moderator` label.** Rejected: they are already
   reachable by every participant, so the label would add a name and no boundary — and a role
   that restricts nothing is worse than no role, because it reads as a control.
