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

| # | Blocker | What it is waiting for | What it does **not** block |
| --- | --- | --- | --- |
| **A1** | An administrative read of `/v1/admin/metrics` leaves **no record of who took it** | A choice between three route shapes, each with a real cost: requiring `?tenant_id=` **breaks every existing caller**; making it optional-unless-you-hold-two-admin-grants turns a working request into a failing one later; deriving the tenant from the admitting grant is non-breaking but needs a new authority-selection path | Access. The hole that let a **revoked** boundary keep this surface open is closed and controlled. What remains is the audit record and the route's width |
| **B5** | The **licence** is undecided | The director | Nothing today. The workspace carries a licence field and the G1 gate names a licence policy, so the decision has consumers, but no current work fails without it |

## Blocked on the outside world

No amount of work inside this repository produces any of these. The first three
are what block **Internet exposure**; §16.12's remaining seven lines shipped with
measured evidence, subject to the re-derivation noted below.

| # | Blocker | Who clears it | Why it cannot be self-supplied |
| --- | --- | --- | --- |
| **B1** | Externally reviewed **threat model** and abuse cases | An independent reviewer | §2.6: AI review does not satisfy an independent-review requirement for Internet qualification |
| **B2** | **Penetration test**, with critical and high findings resolved | An outside engagement | §19.6: the release is *cancelled*, not waived, if findings stand |
| **B3** | **Prompt-injection action-boundary suite** | Arrives with the exposure profile's first action-bearing surface | There is nothing to attach it to yet |
| **B4** | **Public-name clearance** (ADR-001) | Professional trademark, company, package and domain clearance | §2.7: the exact-name screen performed is explicitly *not* legal clearance |

`ReasonBraid` remains a **working name**. Do not assume the crate, domain or
handle names are obtainable. This does not affect repository visibility, which is
public deliberately.

## Known and not blocked, but you should not read past them

| # | Item | Status |
| --- | --- | --- |
| **C1** | **Remote CI has never run.** Every gate result recorded anywhere in this project — including the one full checkpoint pass — is a **local** qualification. The project's own doctrine names the *remote* run as the authoritative pre-push gate | The push cadence is roughly 300 commits and the branch is inside it, so this is not a schedule failure. It is a limit on what may be claimed |
| **C2** | **No gate record's "shipped" count currently re-derives.** Five gate records exist and four count lines as shipped; all were counted *before* the full source review | The records are not dishonest — each claims the evidence its suites then carried. What was never measured is those suites' **coverage**, and the review has since reproduced cross-tenant defects inside the very subject one of those lines names |

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

⛔ So the honest position is that Internet exposure has **three external
blockers and one internal one**: the seven lines counted as shipped have to be
re-derived against the repaired code before they can be counted again. That
re-derivation is tracked, and it covers all five gate records rather than only
this one.
