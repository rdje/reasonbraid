# Blockers

This page lists **everything that is currently blocking progress**, who can clear
each one, and what it does *not* block. It is maintained as a register rather
than as a narrative: a blocker leaves this page when it is cleared, never because
it has gone quiet.

Each row is owned by a task-tree leaf under `docs/tasks/SIGNOFF-REPAIR.md`
(`SIGNOFF-REPAIR.13`, the blocker register). Ownership and visibility are treated
as two separate obligations here — a blocker recorded only in the task tree is
owned but invisible from outside, which is how a held decision sits untouched for
days with every individual rule followed.

## Blocked on a decision only the director can make

**Nothing, as of 2026-09-15.** Both rows that stood here are cleared, and *how*
they cleared is worth more than the fact that they did — see
[Cleared](#cleared) at the foot of this page.

## The route out, decided 2026-09-16

⭐ **Three of the four outside blockers share one prerequisite, and it is work
inside this repository.** The Phase-7 subtraction record has the exposure profile
waiting on the qualification gaps while two of those gaps wait on the exposure
profile. That circle is an artefact of the record's stated reason, not of the
gate: ROADMAP §16.12 blocks Internet-capable **deployment** and §25.1 blocks
**exposing** remote enrollment. Neither forbids *building* a candidate and
leaving it off.

`SIGNOFF-REPAIR.14` now owns an exposure-profile candidate — built, frozen, never
deployed — under a standing prohibition that no leaf in it may turn the profile
on or claim any part of G6/G7. Once it is frozen, the external review and the
penetration test have something to run against, which is exactly what their own
revisit triggers ask for.

The other decisions, with their rejected alternatives, are in
`docs/decisions/2026-09-16_internet-qualification-route.md`: apply to an
open-source audit programme first with a commercial fallback twelve weeks after
the candidate freeze; report vulnerabilities through GitHub private reporting
rather than a published address; and clear the name in the United States, the
European Union and the holder's own jurisdiction.

⛔ **None of this advances the gate by itself.** G6/G7 remains **NOT MET** and
B1–B4 remain open. What changed is that three of them now have a named next
action here instead of a wait.

## Who clears each one, and what is owed HERE

⚠️ **This table changed shape on 2026-09-17, and the reason is worth a sentence.**
It used to carry an `Ack?` column meaning *"has the director engaged with this
row"* — a field whose value is a fact about the **reader**, which the register
cannot observe and the maintainer cannot set. Measured: it was `no` on every open
row for its whole life and never once anything else. The maintainer had been
given the disposition to decide; the column made the director's attention the
precondition for a row to stop being repeated at him.

⭐ The column now answers the question a blocker register must answer: **is there
anything left that this repository can do?** That is the only thing separating a
row the maintainer should still be working from one genuinely parked on an
outsider — and replacing it changed three answers immediately.

| # | Blocker | Who clears it | Owed here? | What is owed, or the trigger |
| --- | --- | --- | --- | --- |
| **B1** | Externally reviewed **threat model** and abuse cases | An independent reviewer | **yes (deferred)** | `spec/threat-model.md` exists; its **fitness for review** is owed, and `SIGNOFF-REPAIR.14`'s frozen exposure candidate is what a reviewer would review. §2.6: AI review does not satisfy the requirement |
| **B2** | **Penetration test**, critical and high findings resolved | An outside engagement | **yes (deferred)** | The **scope and environment** document, and `.14`'s candidate freeze — which is this row's own stated trigger. §19.6: the release is *cancelled*, not waived, if findings stand |
| **B3** | **Prompt-injection action-boundary suite** | — | **yes (deferred)** | It lands with `.14`'s first action-bearing surface. The `ACTION-BOUNDARY` gate already fails the commit that ships one and says B3 is due |
| **B4** | **Public-name clearance** (ADR-001) | Professional trademark, company, package and domain clearance | **no** | ⭐ The only row with nothing owed here. Jurisdictions decided; the rename cost is measured at 67 files / 191 lines. §2.7: the exact-name screen is explicitly *not* legal clearance |

## Deferred, by director instruction of 2026-09-18

> *"The internet exposure is not high priority right now. It needs to fully work
> on the local network first."*

B1, B2 and B3 are the Internet-exposure blockers, and they are now
**`yes (deferred)`**: the in-repo work is still owed and still has a leaf, but it
no longer rides the frontier and no commit is spent on it until the local-network
path is complete.

⛔ **`yes (deferred)` is not `no`.** A row parked on an outsider has nothing left
for this repository to do. These three have work left; it is simply not current.
The register keeps them visible — surfaced once per session with the ruling and
the trigger — because the whole point of the register is that an owned but
unsurfaced blocker is harder to notice than an unowned one.

⛔ **And nothing about what may be CLAIMED changes.** Phase 7's G6/G7 remains
**NOT MET**, and `SIGNOFF-REPAIR.14`'s standing prohibition against turning the
exposure profile on is untouched. Deferring work is not permission to describe
the system as Internet-ready; if anything it makes the claim further off.

### The trigger, defined

"The LAN fully works" means **ReasonBraid's behaviour matches the roadmap's
objective on a private network**. The roadmap already partitions on this line:
its header separates *Initial deployment: trusted hosts on a private LAN* from
*Target deployment: arbitrary Internet-connected hosts*, and its gate table
carries an *unlocks* column in which **G6 Internet security is the only gate
whose unlock is a transport**.

So the bar is **G0–G5 genuinely met, plus G7 earned on the LAN**, with G6 out of
scope until the director says otherwise.

G7 is inside the bar because its evidence is mostly transport-independent —
whether a restore restores, whether the system survives losing a node, whether
it is instrumented — so earning it on the LAN is the order that does not waste
the work, and chaos exercises are cheap on a private network and expensive after
exposure.

⛔ **G7 splits, and the split matters.** Its *structural* legs are earned once.
Its *numeric* legs — load thresholds and SLO targets — are anchored to a LAN and
say nothing about Internet latency, loss or adversarial traffic; they are
re-derived under the Internet posture rather than carried across.

⚠️ **G7 is the least-built of the LAN gates**, and the bar is entered with that
known: backup/restore is real (`scripts/backup.sh`, `scripts/restore.sh` and a
suite), the metrics surface exists without objectives, load exists only as a
dev-scale storm control, and chaos/game-day work does not exist at all.

⛔ **Nothing about the blockers themselves has moved.** B1, B2 and B4 still need
outside parties, `.14` is still under its standing prohibition against turning
the exposure profile on, and G6/G7 remains **NOT MET**. What changed is which of
these rows the maintainer treats as work — and it turns out three of the four
were being reported as *external with nothing owed* while this very page already
said they share one in-repo prerequisite.

`ReasonBraid` remains a **working name**. Do not assume the crate, domain or
handle names are obtainable. This does not affect repository visibility, which is
public deliberately.

## Known and not blocked, but you should not read past them

| # | Item | Status |
| --- | --- | --- |
| **C1** | **The remote gate is green, and 167 commits sit beyond it.** All three workflows last ran **successfully** at `c17841c`, which is exactly `origin/main`. 🔴 This row previously read *"remote CI has never run"* and that was **false** — see below | The push cadence is roughly 300 commits and the branch is inside it, so this is not a schedule failure. What may not be claimed is that the *current* head has been remotely gated |
| **C2** | ✅ **CLOSED.** Gate records counted their shipped lines before the full source review; **all five have now been re-derived**, line by line, each verdict carrying the command that produces it — G1–G2, G3, G4, G5 and G6–G7 | The records were never dishonest — each claimed the evidence its suites then carried. What was never measured is those suites' **coverage**, and now it has been. ⛔ No record's conclusion changed: G6–G7 remains NOT MET for Internet exposure. ⚠️ One strand of G4 is un-discharged and tracked separately |

## C1 was wrong, and how it was wrong is the useful part

Until 2026-09-17 this page stated that **remote CI had never run**. Measured
against the remote rather than against this project's own documents:

| Measurement | Value |
| --- | --- |
| Workflow runs, total | **41** |
| Conclusions | **29 success, 12 failure** |
| Latest `doctrines` / `rust` / `supply-chain` | **success**, all at `c17841c` |
| `origin/main` | **`c17841c`** — the last commit CI observed *is* the remote head |

So the remote gate is **green**, and the honest limit is narrower: 167 local
commits sit beyond the last remotely-gated commit, which the ~300-commit cadence
explicitly permits.

⭐ **The row was written from `COMMIT.md`'s cadence note rather than from the
remote — and that same page contradicted it four lines further down**, where its
provenance reads *"director instruction 2026-09-11, during the first remote-CI
repair sequence."* The twelve measured failures are largely that sequence: the
`rust` workflow failed across six consecutive pushes and then went green, which is
exactly the one-repair-per-push pattern `COMMIT.md` describes as its own standing
exception.

⛔ **Pushing early is still not the fix.** That exception applies *while a remote
gate is red*. It is green, so the cadence governs — the same conclusion the
register reached before, now resting on a true premise instead of a false one.

Full measurement and the corrected downstream reasoning:
`docs/decisions/2026-09-17_remote-ci-has-run-and-is-green.md`.

## What C2 means for the Internet gate

The G6–G7 gate record concluded **NOT MET for Internet exposure** and said so
plainly, which is the honest outcome. It also mapped §16.12's ten lines as seven
shipped and three missing.

Line (2) of the ten is *authenticated enrollment, rotation, revocation and
tenant-isolation tests*. Since that record was written, the corrective review has
reproduced — live, against supported surfaces — a caller in one tenant receiving
another tenant's entire thread projection, an administrator reading another
tenant's node inbox in full, a prune destroying another tenant's rows, a
participant answering another tenant's recruitment call, and both publish verbs
admitting any enrolled principal. All are repaired.

⛔ So Internet exposure has **three external blockers and one internal one**.
The internal one now has a measured answer, recorded in
`docs/decisions/2026-09-15_g6g7-shipped-lines-re-derived.md`:

| §16.12 line | Verdict |
| --- | --- |
| (2) enrollment, rotation, revocation, tenant-isolation tests | **must be re-earned** — all four subjects had live defects, all since repaired |
| (3) non-escalation and confused-deputy tests | **must be re-earned** — including the confused-deputy shape itself |
| (4) SSRF, DNS-rebinding, redirect, archive-bomb suite | **narrowed** — and its open defect is now **repaired**: the Git pack classified only its first destination and auto-followed up to five hops behind a name-resolution hook that never runs for a bare IP address. Every hop is now classified by the same policy, a refused hop is named rather than failing as a transport error, and it was reproduced before it was repaired |
| (6) dependency, SBOM, provenance, release-signing | **stands**, with the dependency ledger's empty tested-version rows noted |
| (7) backup restore and compromised-key recovery | **narrowed** — the exercise runs; its fixture's own defects are open |
| (8) rate-limit, cost-circuit-breaker, notification-storm | **must be re-earned** |
| (10) incident runbooks, contacts, disclosure | **stands** |

⚠️ **Re-earning the three is not repair work — the defects are fixed.** What is
missing is the coverage measurement that would let each line be counted again,
which is a different activity. The gate's conclusion is unchanged: **NOT MET for
Internet exposure**, and the original record is added to rather than edited.

## Cleared

A row leaves the register above when it is cleared, and lands here with the date
and the reason. This section exists so that a reader who saw a blocker can find
out what happened to it, rather than watching it disappear.

| # | Was blocking | Cleared | How |
| --- | --- | --- | --- |
| **A1** | An administrative read of `/v1/admin/metrics` left no record of who took it | 2026-09-15 | ⭐ **The question dissolved under measurement.** It was held two days for a choice between three route shapes, and all three shared a false premise — that the route must *name* a tenant. It does not: a principal belongs to exactly one tenant structurally, so the tenant is derivable from the authenticated caller. No wire change, no broken caller, no new authority-selection path. The read now commits a record bound to the caller's own tenant |
| **B5** | The licence was declared in all 13 manifests and **granted nowhere** | 2026-09-15 | The copyright holder supplied the one fact only he held. `LICENSE-MIT` and `LICENSE-APACHE` now ship at the repository root, carrying the texts the declarations already named. Both were **copied from licence files shipped by crates in this workspace's own dependency graph and checked byte-for-byte**, not reproduced from memory — a paraphrased licence is not a licence |

⚠️ **A1 is the one to learn from, and the lesson is not that held decisions
dissolve.** It is that A1 sat for two days with nobody testing its premise,
because *held for a decision* reads like a settled state rather than like work
still owed. The instruction that cleared it — unblock the blockers, with a
rationale — is what prompted anyone to check whether the question was real.

⛔ Clearing B5 does **not** touch B4. A licence grants copyright permissions in
the work; it says nothing about the name on it, and the public-name clearance
above is a separate external gate.
