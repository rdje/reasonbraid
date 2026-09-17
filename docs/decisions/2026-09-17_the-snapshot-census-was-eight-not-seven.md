# The snapshot census was eight, not seven — and the eighth surface was unbound

- Date: 2026-09-17
- Status: accepted
- Owner: `SIGNOFF-REPAIR.11.14.3.15`
- **Corrects** `2026-09-17_the-standalone-assessment-is-citation-bound.md`
  (`.11.14.3.8`, REPAIR-0222), whose published census is false. That record is
  left byte-unchanged; this one supersedes its number and its table.

## What was published, and what is true

`.11.14.3.8` published, in its leaf, its decision record, `deployment.md`,
`CHANGELOG.md` and `LIVE_STATUS.md`:

> *"Seven surfaces, six bound, exactly one not. That is the enumeration the
> finding is made of, not an impression."*

Re-derived on the **identifier** rather than on the route table and the
`cited_snapshot` call sites:

| Surface | Citation-bound |
| --- | --- |
| `GET /v1/snapshots/{id}` | yes — `snapshots::get_for_tenant` |
| `DELETE /v1/snapshots/{id}` | yes — `cited_snapshot` |
| `GET /v1/snapshots/{id}/derivations` | yes — `cited_snapshot` |
| `GET /v1/snapshots/{id}/assessments` | yes — `cited_snapshot` |
| `GET /v1/snapshots/stale` | yes — filtered per tenant |
| `thread.contribute` kind `assessment` | yes — `is_cited_by` |
| `POST /v1/assessments` | yes, **since REPAIR-0222** |
| **`POST /v1/derivations`** | **no** |

**Eight, six bound, two not.** The number was wrong and one of the two defects
went unrepaired for seven commits.

## How it happened, stated as the mechanism rather than as regret

`POST /v1/derivations` names its parent as `parent_snapshot_id` in the request
**BODY**. A census built from `Path(snapshot_id)` extractors and `cited_snapshot`
call sites cannot see it.

⭐ **This is the FIRST instance of the blind spot
`docs/knowledge/a-census-is-as-wide-as-its-key.md` describes**, and that note was
written from the *second* (`.11.14.3.11`, where `POST /v1/snapshots` names a
`reference_id` in its body). The rule was correct, published, and **never applied
backwards to the census that had already made the same mistake**. A rule learned
and not swept is a rule applied once.

⛔ **Graded on all three axes** (`docs/CLAIM_VERIFICATION.md` §4.1), and none
survives: the **PROSE** — *every surface naming a snapshot id is bound but one* —
is false rather than imprecise; the **NUMBER** moved 7→8 and 1→2; the **NAMED
INSTANCE**, an enumerated table, omitted a row. That record's own standard says a
named instance is exact with no tolerance band.

⚠️ And the acceptance test it sets — *"when asked whether you stand by it, the
answer is yes, immediately, with no keyboard"* — was failed. This census was
re-derived only because the director asked whether the findings hold.

## The defect the false number hid

`submit_derivation` admitted on enrolment alone and passed the caller's
`parent_snapshot_id` to `derivations::submit`, whose parent check was:

```sql
SELECT EXISTS (SELECT 1 FROM evidence_snapshots WHERE snapshot_id = $1)
```

No tenant predicate, and the function took **no tenant at all**. Reproduced: a
second tenant's derivation against a snapshot it had never cited **succeeded** —
`200 {"derivation_id":"drv_01a0b0c6c1367e22abcd461ab1bf3001"}` — while an absent
id was refused.

⚠️ **Width, stated before it is inflated.** `snapshot_id` is unguessable and no
list verb returns another tenant's ids, so it is an ORACLE requiring the caller to
hold the id — the same class as the two already repaired. The written half was
bounded by the read: `GET /v1/snapshots/{id}/derivations` is citation-bound, so
the foreign tenant could not read its own derivation back. ⛔ That bounds the harm
and does not excuse it — an invisible write into another tenant's evidence graph
is worse evidence than a visible one, which is the disposition `.11.14.3.11`
already took for the identical shape.

## The decision

**The derivation write takes the citation binding, in the store**, as the two
before it do. `derivations::submit` gains the citing tenant and its parent check
becomes one statement joining `evidence_citations`. ⛔ **No new error variant**: a
parent the caller did not cite answers the `ParentMissing` an absent one gets.

⚠️ **The derivation GRAPH stays shared**, and the control asserts it: once the
second tenant cites the parent, both tenants read both children. That is
`.11.14.2`'s disposition — a derivation is content-addressed the way a snapshot
is — and this repair binds the WRITE without narrowing the read.

## What this changes about the other censuses in the family

Every census this session published was re-derived under the director's audit.
The rest hold:

| Claim | Re-derived |
| --- | --- |
| `.11.14.3.4`: three routes under `/v1/resources`, two unbound store readers | holds — and its reach limit was already recorded by `.11.14.3.11` |
| `.11.14.3.11`: one `WHERE reference_id` site, so no route lists a reference's snapshots | holds — **1** |
| `.11.14.3.13`: nothing reads `evidence_snapshots.original_locator` for a decision | holds |
| `.11.14.3.7`: `quota::check_in_tx` has 2 callers, 4 scope kinds, 12 `OP_` constants | holds — ⚠️ a raw `grep -c 'pub const SCOPE_'` returns **5** because it matches the `SCOPE_KINDS` array itself; the array's own type is `[&str; 4]` |
| `.11.14.3.7`: `axum-core-0.5.6` `DEFAULT_LIMIT = 2_097_152`, no `DefaultBodyLimit` in our source | holds |
| `.11.14.3.10`: no production path registers a broker binding | holds — 3 sites, all under `#[cfg(test)]`; 0 in `src/bin` |
| `.13.4`: 13 hits over the live documents, none a bare assertion | holds |
| 21 fixture plans gained `reference_registrations` | holds — 21 of 21 |

## Verification

- RED, against the exact unrepaired store: the foreign derivation succeeded while
  the absent parent was refused; `52 passed; 1 failed`.
- GREEN: foreign and absent are refused in the **same words** with **0** rows
  attached; the citing tenant derives normally; the second tenant acquires the
  same bytes, records its citation and then derives; and **both tenants read both
  children**, so the shared graph is unchanged.
