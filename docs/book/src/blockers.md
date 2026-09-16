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

## Blocked on the outside world

No amount of work inside this repository produces any of these. The first three
are what block **Internet exposure**; §16.12's remaining seven lines shipped with
measured evidence, subject to the re-derivation noted below.

| # | Blocker | Who clears it | Why it cannot be self-supplied |
| --- | --- | --- | --- |
| **B1** | Externally reviewed **threat model** and abuse cases | An independent reviewer | §2.6: AI review does not satisfy an independent-review requirement for Internet qualification |
| **B2** | **Penetration test**, with critical and high findings resolved | An outside engagement | §19.6: the release is *cancelled*, not waived, if findings stand |
| **B3** | **Prompt-injection action-boundary suite** | Arrives with the exposure profile's first action-bearing surface | There is nothing to attach it to yet — and as of 2026-09-15 that is enforced rather than asserted: the `ACTION-BOUNDARY` gate pins the four facts this deferral rests on, so the commit that lands an action-bearing surface fails and says B3 is due |
| **B4** | **Public-name clearance** (ADR-001) | Professional trademark, company, package and domain clearance | §2.7: the exact-name screen performed is explicitly *not* legal clearance |

`ReasonBraid` remains a **working name**. Do not assume the crate, domain or
handle names are obtainable. This does not affect repository visibility, which is
public deliberately.

## Known and not blocked, but you should not read past them

| # | Item | Status |
| --- | --- | --- |
| **C1** | **Remote CI has never run.** Every gate result recorded anywhere in this project — including the one full checkpoint pass — is a **local** qualification. The project's own doctrine names the *remote* run as the authoritative pre-push gate | The push cadence is roughly 300 commits and the branch is inside it, so this is not a schedule failure. It is a limit on what may be claimed |
| **C2** | **Gate records counted their shipped lines before the full source review.** Five records exist and four count lines as shipped. The first has now been re-derived: of G6–G7's seven, **three must be re-earned, two are narrowed, two stand** | The records are not dishonest — each claims the evidence its suites then carried. What was never measured is those suites' **coverage**. Four records remain to re-derive |

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
