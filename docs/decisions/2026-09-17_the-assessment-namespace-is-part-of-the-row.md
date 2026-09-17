# The assessment namespace is part of the row

- Date: 2026-09-17
- Status: accepted
- Owner: `SIGNOFF-REPAIR.11.14.3.3`
- Supersedes nothing; corrects no prior record. Extends
  `2026-09-16_the-deliberation-flow-owns-the-evidence-chain.md` (`.11.14.3`) at
  the point it explicitly left open.

## The question the leaf was opened on

`.11.14.3.1` made a deliberation's `claim_id` the **server-minted claim digest**,
membership-checked against the thread. It left `POST /v1/assessments` standing,
deliberately: removing a shipped route is a breaking change ROADMAP §13.2 does
not require, and an assessment asserted outside any deliberation may be
legitimate. That left two writers putting two kinds of identifier into one
column, and `.11.14.3.3` owns three things — the population already written, the
standalone writer's disposition, and what `claim_id` becomes.

## The population, measured rather than assumed

The leaf's first acceptance clause is that the population is **measured by
command**. It is, in two independent ways, because "there is no production data"
is an assumption and not a measurement.

**The write census** — every path that can create a row:

```bash
git grep -n "INSERT INTO claim_assessments" -- .     # -> 1 hit: crates/reasonbraid-server/src/claims.rs:152
git grep -n "claims::submit" -- crates                # -> 2 call sites + 1 doc reference
git grep -ln "claim_assessments" -- . ':!crates/**/tests/**' ':!docs/**'
```

There is exactly **one** INSERT statement in the tree, reached by exactly **two**
callers (`api.rs`'s standalone route and `threads.rs`'s `assess` step). No
migration, seed, fixture or deployment artifact inserts a row — the third command
returns only the two migrations that *define* the table, the two source files,
the MCP fixture-purge table list, and live docs. And `git grep -n "clm_"` finds
the invented identifiers (`clm_budget`, `clm_beta`, `clm_g4`) **only inside
`crates/reasonbraid-server/tests/profiles.rs`**.

**The durability census, and the correction that made it honest.** The first
version of this leg ran:

```bash
find . -name PG_VERSION -not -path "./target/*"      # -> nothing
```

and concluded that no PostgreSQL cluster exists anywhere in the repository. ⛔
**The exclusion is the reason that check could not fail.** `run_pg_tests.py`
destroys its cluster on success and **retains it under `target/` as failure
evidence** — which is exactly the path the command excluded. Run without it:

```bash
find . -name PG_VERSION | wc -l        # -> 32
ls -d target/pg-tests/*/               # -> 3 retained clusters
```

and one of them, `target/pg-tests/run-9_ueev0t`, is **this leaf's own RED run**.
Rows exist right now.

⭐ The defensible statement is narrower, and it is carried by the WRITE census
above rather than by `find`:

> No row is carried in **tracked** state. `target/` is gitignored
> (`.gitignore:2`), every cluster that has ever held a row is destroyed on
> success — the GREEN run's own line is
> `pg-tests: stopped and removed target/pg-tests/run-wowo355p` — and retained
> only as failure evidence a developer deletes.

That is what the disposition below rests on, and it would still hold if the table
were full.

## What the measurement found that the leaf did not anticipate

The leaf framed this as a namespace tidiness question. Running a control found a
live defect, and it is the reason this is a repair rather than a recorded
position.

`claim_assessments_replay_idx` was `(claim_id, snapshot_id, assessment, author)`,
and `claims::submit` pre-checks on exactly those four columns before inserting.
On the standalone route **`author` is a caller-supplied label** — `.11.14.2`
established that the authorization never reads it. So a caller who typed a real
thread's minted claim digest, that thread's snapshot, its assessment kind and its
author matched all four columns, and the pre-check returned the **deliberation's
own `assessment_id`**.

RED, before the repair, from `the_two_assessment_writers_are_two_namespaces`:

```
assertion `left != right` failed: the standalone route aliased the deliberation's
own row and returned its assessment_id — the replay key does not carry the namespace
  left: "asn_01a0ac9b3bbb7d3293dcd08c20f9fc07"
 right: "asn_01a0ac9b3bbb7d3293dcd08c20f9fc07"
43 passed; 1 failed
```

⭐ That is an **existence oracle over another principal's deliberation**: a
caller learns that a given claim digest, snapshot and kind were asserted inside a
thread it never contributed to, and is handed the row's identifier. It is not an
enumeration — the caller must already hold the claim digest and the snapshot id.

🔴 **The first version of this record said the oracle was "bounded to one tenant
by `.11.14.2`'s authoring gate". That was wrong, and the way it was wrong is the
more useful half of this leaf.** The authoring gate binds the two READS; the
replay pre-check carries **no tenant predicate at all**. The single-tenant case
had been reproduced and the cross-tenant width was published **by reading the
SQL** — which `docs/CLAIM_VERIFICATION.md` leg 2 forbids precisely because
reading errs in the flattering direction. Measured afterwards, with the namespace
column already in place, a second tenant presenting the first tenant's `author`
label was handed its row id:

```
assertion `left != right` failed: a SECOND TENANT was handed the first tenant's
assessment_id — the replay key carries a caller-supplied `author` and no server-set tenant
  left: "asn_01a0ace7727c7b31bea32938bf807721"
 right: "asn_01a0ace7727c7b31bea32938bf807721"        43 passed; 1 failed
```

### The prior ruling this contradicts

`migrations/0064` justified taking a column rather than a citation table by
arguing that `claim_assessments_replay_idx` "carries the AUTHOR, so two tenants
asserting the same thing about the same evidence already hold two separate rows"
— and stated, **eight lines further down the same file**, that "`author` and
`verifier` remain unauthenticated caller strings". Both cannot be true. Two
tenants hold two rows only while they happen to type different labels.

⚠️ **And the control that should have caught it could not.** In
`an_assessment_is_read_by_the_tenant_that_authored_it`, each tenant submits its
own principal as `author`, so the two separate naturally. It **illustrated** the
property rather than testing it — two hypotheses predicting the same observation,
which is leg 2's definition of untested.

⭐ So `authored_by_tenant` — the SERVER-set column — joins the replay key and the
pre-check alongside the namespace. That is not a refinement; it is what makes
`0064`'s own sentence true. `0064` is left byte-unchanged: a shipped migration's
checksum is load-bearing, and `docs/decisions/` supersedes rather than mutates.

⛔ Making `author` itself trustworthy is **not** done here — `0064` defers that to
`SIGNOFF-REPAIR.7.4` by name and it is a wire-contract change. This stops `author`
from being load-bearing for IDENTITY, which is a smaller and separate claim.

## The decision

**The namespace is part of the row's identity, recorded by the server — and so is
the authoring tenant.** `migrations/0066` adds `claim_namespace` (`'thread'` |
`'external'`, NULL for pre-existing rows) and rebuilds the replay index as
`(claim_id, snapshot_id, assessment, author, claim_namespace, authored_by_tenant)
NULLS NOT DISTINCT`. The general rule both columns are instances of: **every
column of a replay key must be a value the submitter is entitled to assert**, and
a caller-set column is safe only where the key space is already partitioned by
something server-set.
`claims::submit` takes the namespace from its **call site**, never from the
submission, and it joins the pre-check as well as the index — an index alone
would not close the aliasing, because the pre-check short-circuits before the
insert ever runs.

This is the same conclusion `.11.14.3.2` reached one table over: an identifier
two writers mint differently is not one identifier, exactly as a reference's
identity turned out to be the `(locator, digest)` pair rather than the locator.

### The rows: left as they are, and the reason is not "no data"

No backfill. Two reasons, and the first is the one that would still hold if the
table were full:

- **There is nothing to migrate a row TO.** Turning a free-text `claim_id` into a
  thread claim digest requires a thread the row never named. And a pre-existing
  row's namespace is not recoverable even in principle: the only evidence would
  be the *shape* of `claim_id`, and a caller could always type the minted shape —
  which is precisely the collision this column exists to record. NULL therefore
  means "unattributed namespace", the disposition `0064` already took on this
  same table for the same kind of unrecoverable fact.
- The measured population of persisted rows is zero (above), so the choice
  changes nothing today — which is why it is stated as a rule rather than as a
  convenience.

This is the third time this family has taken the fail-closed no-backfill route
(`.11.14.1`, `.11.14.2`), and the third time it is published rather than assumed.

### The standalone writer: kept, labelled, and not filtered

Three alternatives were rejected:

- **Remove the route.** A breaking change §13.2 does not require, for a path
  whose use case `.1` already accepted as legitimate. Nothing in the product
  depends on its removal.
- **Mint a digest on the standalone route too.** ⭐ This looked like the
  symmetric answer and is the wrong one. What makes the thread path's identifier
  trustworthy is `claim_exists_in_thread` — the *membership check* — not the
  hashing. A standalone route has no thread to be a member of, so a digest there
  would be a hash of a caller's own string: still an invented namespace, now
  wearing the minted shape, and therefore **indistinguishable from a thread's by
  construction**. It would deepen the collision it was meant to fix.
- **Filter the claim-keyed read to `thread` rows.** That would make the
  standalone route write-only — its rows unreadable through the only surface
  keyed on what it writes — which is a worse outcome than the removal already
  declined, and arrived at by accident rather than decided.

So both namespaces ride `GET /v1/claims/{claim_id}/assessments`, and **each row
says which it is**. Disclosure is the repair because disclosure is what the
measured harm needs: nothing in the product reads `claim_assessments` to make a
decision — `git grep` finds the two list routes and nothing else, and "evidence
gate" appears in this area only in doc comments — so an ungated row cannot change
an outcome. It can only mislead a reader, and a labelled row does not.

## What this does not fix

⚠️ **Two principals inside ONE tenant can still alias each other** on the
standalone route, because `author` remains the caller's own string. Stated rather
than left implicit, and deliberately not repaired: it is not a disclosure — the
authoring gate already admits both of them to that row — so it is a
deduplication question, not a security one. The security dimension is the tenant
boundary, and that is closed.

⚠️ **Leg named rather than hidden (§4).** *Re-derived and falsified for the
cross-tenant case; the INTRA-tenant "not a disclosure" half is **reasoned, not
separately measured**.* It rests on `assessments_of_claim` filtering on
`authored_by_tenant`, which `an_assessment_is_read_by_the_tenant_that_authored_it`
covers — but no control drives two principals of ONE tenant at one key. ⛔ That is
the same reading-not-measuring move this record spent its length correcting, and
it is published as a named gap rather than restated as a measurement.

⚠️ `POST /v1/assessments` still has **no citation gate**. `claims::submit` reads
`snapshot_objects.bytes` for any `snapshot_id` with no tenant predicate and
reports whether the excerpt appears in it — while the `assess` step refuses a
snapshot this tenant did not cite, with `.11.14.3.1`'s own stated reason that
"an assessment would be a way to learn that a snapshot exists". The same argument
reaches the route that was left standing. Owned as `SIGNOFF-REPAIR.11.14.3.8`,
not reported: it is a separate defect with its own RED control and its own
compatibility question, and folding it into this slice would have made neither
decision cleanly. The census behind it, since it is a set claim: four
citation-gate call sites exist — `api.rs:3794`, `:3869`, `:4045` (all READS) and
`threads.rs:1662` (the `assess` step). Exactly one write path holds the gate, and
`submit_assessment` carries only the enrolment check. ⭐ This leaf's own
cross-tenant control arm reaches the excerpt check **because** of that gap, so it
demonstrates `.11.14.3.8` concretely rather than asserting it.

## Verification

Two RED arms, each reproduced before its own repair, and the second exists only
because this record's first version was audited against
`docs/CLAIM_VERIFICATION.md` rather than restated:

- **RED 1 (two writers, one tenant)** — `43 passed; 1 failed`, the standalone
  route returning `asn_01a0ac9b3bbb7d3293dcd08c20f9fc07`, the deliberation's own
  id. Closed by `claim_namespace`.
- **RED 2 (two tenants, one namespace)** — `43 passed; 1 failed`, a second tenant
  returning `asn_01a0ace7727c7b31bea32938bf807721`, the first tenant's id, with
  the namespace column ALREADY in place. Closed by `authored_by_tenant`.
- GREEN: `RB_DEMO=0 bash scripts/run_pg_tests.sh profiles`.
- `the_claim_assessments_validate_the_citation` is updated deliberately: the
  route's contract moves (the read gains `claim_namespace`; the replay key gains
  a column), and it now asserts that the standalone route's own row is
  `external`.
