# Remote CI has run, is green, and C1 was never true

- Date: 2026-09-17
- Status: accepted
- Owner: `SIGNOFF-REPAIR.13.3` (register row **C1**)
- Corrects: `SIGNOFF-REPAIR.13`'s register row C1, `docs/book/src/blockers.md`,
  `SIGNOFF-REPAIR.13.3`'s own stated finding, and one downstream conclusion in
  `SIGNOFF-REPAIR.11.4.7.2.3` that reasoned from the false premise.

## The claim that was published

> **C1 — Remote CI has never run.** Every gate result recorded anywhere in this
> project — including the one full checkpoint pass — is a **local** qualification.

It sat in the blocker register from 2026-09-15, in the book, and in `.13.3`'s
finding. It was never true.

## The measurement

```bash
gh api "repos/rdje/reasonbraid/actions/runs?per_page=1" --jq '.total_count'   # -> 41
gh api "repos/rdje/reasonbraid/actions/runs?per_page=100" \
   --jq '[.workflow_runs[].conclusion] | group_by(.) | map({c:.[0], n:length})'
   # -> [{"c":"failure","n":12},{"c":"success","n":29}]
for wf in doctrines rust supply-chain; do
  gh run list --workflow="$wf.yml" --limit 1 \
     --json conclusion,headSha,createdAt --jq '.[0]'
done
```

**41 workflow runs — 29 success, 12 failure.** The latest run of each of the
three workflows is **success**, all at `c17841c`, 2026-09-11T22:01:27Z. And
`git rev-parse origin/main` is `c17841c`: the last commit CI observed *is* the
remote head.

What that green covers, read from the workflow files rather than assumed:
`doctrines` runs the doctrine enforcer; `supply-chain` runs pinned `cargo-deny`
and pinned Gitleaks over history; `rust` runs the workspace check, the Python
controls with the full owned PostgreSQL collection, and the book build.

## The correct statement

> The remote gate **is green at `origin/main`**. The limit is that **167 local
> commits sit beyond the last remotely-gated commit** — which the project's own
> ~300-commit cadence explicitly permits.

That is a materially different claim from the one published, and it changes the
remedy. "CI has never run" implies an instrument to be stood up; the truth is an
instrument that works, is green, and is simply 167 commits behind.

## Root cause

⭐ **The row was written from `COMMIT.md`'s cadence note rather than from the
remote — and `COMMIT.md` itself contradicted it.** That file's push-cadence
section ends:

> *Provenance: director instruction 2026-09-11, during the first remote-CI repair
> sequence.*

A "remote-CI repair sequence" is a sequence of remote CI runs. The 12 failures
measured above are largely it: `rust` failed at REPAIR-0076, 0077, 0078, 0079,
0080 and 0083, then went green at 0086 — exactly the one-repair-per-push pattern
`COMMIT.md` describes as its standing exception. The register's author read the
cadence rule on that page and did not read the provenance line four lines below
it.

⛔ This is `docs/CLAIM_VERIFICATION.md` leg 1 in its plainest form: a number
quoted from the project's own documentation instead of re-derived from the
source. The source here was one `gh` call away.

## What follows from the correction

- **C1 stops being a caveat about what may be claimed.** 41 remote results
  exist; the checkpoint pass is corroborated by a green remote run at the same
  head, not orphaned.
- ⛔ **`.13.3`'s instruction "do not fix this by pushing early" STANDS, and now
  rests on a true premise.** `COMMIT.md`'s standing exception applies *while a
  remote gate is red*. The remote is **green**, so the exception does not apply
  and the ~300 cadence governs. The earlier reasoning reached the right answer
  from the wrong fact.
- ⚠️ **One downstream conclusion was built on the false premise and is corrected
  rather than deleted.** `.11.4.7.2.3` wrote that `make deny` and `make
  secret-scan` "live only in `.github/workflows/supply-chain.yml`, which runs in
  REMOTE CI — and remote CI has never run … it is why two supply-chain gates have
  been red with nobody able to see it." The supply-chain workflow **ran and was
  green** at `c17841c`. The true reason those gates are currently unseen is the
  167-commit unpushed gap plus the fact that both findings post-date the last
  push — which that leaf had already established in its next sentence. The
  conclusion survives; its stated cause does not.

## Durability (leg 3), and the gate deliberately NOT added

The claim is stated **against a named commit** (`c17841c`) with the command that
re-derives it, rather than as a standing fact — because "CI is green" goes stale
on the next push, and a number nothing re-derives goes stale silently.

⛔ **No doctrine check is added, and the reason is stated rather than left as an
omission.** Every gate in `scripts/check_doctrines.sh` runs offline in a
pre-commit hook; a check that queries GitHub would fail without network or
without `gh` auth, turning an unrelated outage into a blocked commit. A gate that
cannot run is worse than a documented command, so the command is documented and
the register row names the commit it was measured at.
