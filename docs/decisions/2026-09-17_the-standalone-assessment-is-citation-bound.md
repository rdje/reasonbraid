# The standalone assessment route is bound to the citing tenant

- Date: 2026-09-17
- Status: accepted
- Owner: `SIGNOFF-REPAIR.11.14.3.8`
- Supersedes: nothing. It CLOSES the limit `.11.14.3.3` published in
  `docs/book/src/deployment.md` under "The two assessment namespaces".
- Related: `2026-09-16_evidence-is-shared-the-read-is-tenant-bound.md` (the read
  side), `2026-09-16_the-deliberation-flow-owns-the-evidence-chain.md` (the
  `assess` step), `2026-09-17_the-assessment-namespace-is-part-of-the-row.md`
  (the replay key).

## The finding, in the product's own words

`crates/reasonbraid-server/src/threads.rs`'s `assess` step gates on
`snapshots::is_cited_by` with the comment

> Without this, an assessment would be a way to learn that a snapshot exists —
> the enumeration `.11.14.1` closed on the read side.

`POST /v1/assessments` is the other writer. Both call the SAME
`claims::submit`, which selected `snapshot_objects.bytes` joined to
`evidence_snapshots` on `snapshot_id` alone — **no tenant predicate** — and then
reported the outcome. Nothing on the standalone path applied that gate.

## The census, by command, in both directions

Every surface that names a `snapshot_id`, with the gate it reaches:

```bash
grep -n "cited_snapshot\|is_cited_by" crates/reasonbraid-server/src/*.rs
grep -n '"/v1/snapshots\|"/v1/assessments' crates/reasonbraid-server/src/api.rs
```

| Surface | Citation-bound before this decision |
| --- | --- |
| `GET /v1/snapshots/{id}` | yes — `snapshots::get_for_tenant` |
| `DELETE /v1/snapshots/{id}` | yes — `cited_snapshot` |
| `GET /v1/snapshots/{id}/derivations` | yes — `cited_snapshot` |
| `GET /v1/snapshots/{id}/assessments` | yes — `cited_snapshot` |
| `GET /v1/snapshots/stale` | yes — filtered per tenant |
| `thread.contribute` kind `assessment` (the `assess` step) | yes — `is_cited_by` |
| **`POST /v1/assessments`** | **no** |

Seven surfaces, six bound, exactly one not. That is the enumeration the finding
is made of, not an impression.

## What it answered, measured RED first

`the_standalone_assessment_route_is_bound_to_the_citing_tenant`, run against the
unrepaired code, with a second tenant holding an `snp_` id it never cited:

| The probe | The answer |
| --- | --- |
| an identifier that names nothing | `400` `the cited snapshot does not exist` |
| the owner's real snapshot, excerpt NOT in its bytes | `400` `the excerpt does not appear in the snapshot's bytes — the citation is refused` |
| the owner's real snapshot, excerpt IS in its bytes | **`200`**, with a stored `assessment_id` |

Three distinguishable answers. The first two separate *exists* from *does not
exist* — the existence oracle. ⭐ The third is stronger and is what makes this
worth repairing rather than publishing: a `200` says a **chosen substring
appears in bytes the caller was never allowed to read**, which is a content
probe, and it also wrote a row.

⚠️ Stated at its real width: `snapshot_id` is `snp_`-prefixed and unguessable
and no list verb returns another tenant's ids, so this is an ORACLE requiring
the caller to already hold the identifier — materially weaker than
`.11.14.1`'s `GET /v1/snapshots/stale`, which returned every row.

## The decision

**The route takes the citation gate**, and the gate lives in `claims::submit` —
the function both writers reach — before anything about the snapshot is read.
A caller whose tenant holds no `evidence_citations` row for the snapshot
receives one refusal, `AssessmentError::SnapshotNotCited`, whose message
carries **no identifier and no fact about the snapshot**, so an identifier that
names nothing and one that names a snapshot the caller never cited are the same
bytes.

The `assess` step keeps its own `is_cited_by` check. That is deliberate
redundancy of the kind this project already uses for the profile writer's
`UNIQUE` backstop: the step's check is the **named refusal** — it names the
snapshot and the step, which a deliberation needs — and the store's is the
**invariant**, which no future writer can be built around. For the deliberation
path the store's gate should never fire.

## The alternatives, and why each was rejected

1. **A uniform refusal without a gate** — make `SnapshotMissing` and
   `ExcerptAbsent` indistinguishable. ❌ It closes the existence oracle and
   leaves the content probe, which is the stronger leg: a `200` still reports
   that a chosen substring appears in unread bytes. It also costs the
   *legitimate* caller the diagnosis while buying the least.
2. **Leave it and publish the oracle** — the disposition `.11.14.3.4` may yet
   take for `resource_references`. ❌ The difference is nameable: a reference is
   a locator a contributor chose to cite, while a snapshot is the acquired
   bytes, and this project has already adjudicated the snapshot question six
   times in one direction. `docs/CLAIM_VERIFICATION.md` §3 leg 2 is explicit —
   when a case of the same shape has been ruled, a finding must name the
   difference or the earlier ruling wins.
3. **A tenant predicate inside the bytes select** — ❌ impossible by
   construction. `evidence_snapshots` carries no tenant column *by design*
   (`2026-09-16_evidence-is-shared-the-read-is-tenant-bound.md`): one row serves
   every tenant that acquired the same bytes. The binding has to be the separate
   citation question.
4. **The gate at the HTTP handler rather than in the store** — ❌ it leaves the
   defect exactly where the finding located it. The named defect is that the
   shared function has no tenant predicate; a third writer added later would
   inherit the hole.

## The compatibility break, and the measurement that bounded it

This changes a shipped route: a caller that assessed a snapshot its tenant never
cited received `200` and now receives `400`.

⭐ **The objection the leaf recorded — "a principal legitimately assessing
evidence another team acquired would start being refused" — was measured rather
than accepted, and it does not survive the census above.** Every read of that
snapshot already answers a non-citing tenant `404`: the snapshot row, its
derivations, its assessments, the staleness list, and the delete. So that
workflow cannot function today — the excerpt check was being run over bytes the
caller cannot see, cannot list and cannot delete. The gate removes no working
path.

The supported cross-team path is the one the book already documents:
re-acquiring the snapshot records the second citation on the replay, after which
the second tenant assesses the shared row normally. The control exercises
exactly that, which is the bound that keeps this from being a blackout.

## What it does NOT settle

- ⚠️ **Two principals inside ONE tenant can still alias each other** on the
  standalone route. The citation gate does not touch that — both are admitted to
  the row by the authoring gate — and making `author` itself trustworthy remains
  `SIGNOFF-REPAIR.7.4`.
- ⚠️ `AssessmentError::SnapshotMissing` and `ExcerptAbsent` are **kept and still
  distinguish**, for a caller that HAS cited the snapshot. That is deliberate:
  the diagnosis is owed to a caller entitled to the bytes, and no longer
  discloses anything to a caller that is not.
- ⚠️ A `claim_id` is still a caller label in the `external` namespace. The gate
  bounds which *snapshots* an assertion may name, not which *claims*.

## Verification

- RED: `(400 "does not exist", 400 "excerpt …", 200 asn_…)` against unrepaired
  code, reproduced by `the_standalone_assessment_route_is_bound_to_the_citing_tenant`.
- GREEN: all three probes `400` with one identical message and **0** rows
  written; the citing tenant still assesses (`200`) and still receives the
  excerpt diagnosis (`400`); a second tenant that acquires the shared bytes
  assesses them normally and holds a separate row.
- `RB_DEMO=0 bash scripts/run_pg_tests.sh profiles` — **45 passed, 0 failed**.
- `cargo clippy -p reasonbraid-server --all-targets --locked -- -D warnings`,
  `cargo fmt --all -- --check`.
