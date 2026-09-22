---
answers:
  - Is a publication's immutable content written outside PostgreSQL, or not?
  - Which of the roadmap's two named stores — the Git repository and the content-addressed object store — is in scope before G9?
  - Why was the audit that opened this leaf wrong, and how wrong?
  - If both stores exist, what is actually missing?
  - Why is an external object-store backend deferred, and on what trigger a later pass can evaluate?
  - What does this cost, per option, rather than by implication?
---
# The publication store is built; the gap is that nothing reads it and nothing operates it

- **Type:** decision
- **Status:** accepted
- **Owner:** `SIGNOFF-REPAIR.9.3.5`
- **Date:** 2026-09-22
- **Cites:** ROADMAP §6.3 (authoritative state), §15.7 (canonical publication protocol),
  §15.8 (reconciliation matrix), §7.2 (crate responsibilities);
  `docs/CLAIM_VERIFICATION.md` (re-derive · falsify · durability);
  `docs/decisions/2026-09-22_the-remaining-roadmap-gaps-are-sequenced-by-exposure-then-deletion.md`
  (this is that record's item 2)

## Context

`SIGNOFF-REPAIR.9.3.5` was opened on 2026-09-22 by `.11.4.7.2.1.4.1`, whose tranche-1
adjudication returned **`fired and open`** for the Phase-0 deferral *Git publication
reconciliation + object store experiments*, with the words *Phase 4 is `done` and **both
halves are absent***. The leaf inherited that as its title: *a publication's immutable
content is never written, so the manifest digests nothing outside PostgreSQL*.

🔴 **That verdict is FALSE, and this record retracts it.** Both halves exist. The audit
measured a module name instead of a behaviour, and the leaf it opened carried the error
forward as its premise.

## The retraction, re-derived three independent ways

`docs/CLAIM_VERIFICATION.md` asks three different questions, so three different routes were
used rather than one route three times.

| leg | route | result |
| --- | --- | --- |
| **re-derive** | read the call graph rather than grep one token: `publications.rs:595` → `crate::publisher::missing_objects`; `api.rs:4061` → `crate::publisher::publish` | `crates/reasonbraid-server/src/publisher.rs` implements §15.7 steps **5–8** — the staging branch, the fetch-back digest re-derivation, the write-once immutable ref, and the effective channel's compare-and-swap, on `gix` plumbing with no git CLI |
| **falsify** | could it be dead or later code? `git log --diff-filter=A -- …/publisher.rs` | born **`f53d73d`, 2026-09-08**, leaf `PHASE-6.4.3.2` — **fourteen days BEFORE the 2026-09-22 audit (`1a87d31`) that called it absent.** The audit was wrong when it was made, not overtaken |
| **durability** | is it live, or a compiled ornament? `cargo test -p reasonbraid-server --test publisher` | **7 passed, 0 failed**, including `the_publisher_writes_the_refs_and_verifies_the_fetch_back` and `the_stale_cas_expectation_refuses_and_the_immutable_never_moves` |

⛔ **Why the original measurement failed, stated so it is not filed as bad luck.** It ran
`git grep -n "git::" -- crates/reasonbraid-server/src/publications.rs` → 0, and concluded *a
publication is never reconciled against a ref*. The publication module is named
`publisher::`, and `git.rs` really is evidence acquisition — so the auditor's *second*
observation was correct and its inference from the first was not. **A grep keyed on one
module path is not an absence proof.** This is the **third** recorded instance of that class
in this repository, after `.11.8.2` (the route census demanding one space between a method
and its path) and `census_book_coverage.py`'s coarse mention test.

## What is actually shipped, measured in both directions

| roadmap commitment | status | evidence |
| --- | --- | --- |
| §15.7 steps 5–8 — the Git publication write | ✅ **shipped and wired** | `publisher::publish`, reached from `POST /v1/policy-publications/{id}/publish`; 7/7 green |
| §15.7 step 4 — the manifest digest is the server's product | ✅ shipped | `publications::stage` composes and hashes; a caller-asserted digest that disagrees is refused (`.9.2.1.3.1`) |
| the digest binds what is written | ✅ shipped | `api.rs:4053` refuses when the re-composed manifest ≠ the staged digest, **before** the publisher is called; then the publisher re-derives the digest from the written blob |
| §15.8 — the reconciliation matrix, as a DECISION | ✅ shipped | `reconciler.rs`, all six rows exhaustive, `PHASE-6.4.3.3` |
| §15.8 — the reconciliation, **OPERATED** | 🔴 **ABSENT** | `git grep "reconciler::"` outside its own file returns **one** hit: `tests/reconciler.rs:6`. Nothing in production observes Git state and applies an action |
| §6.3 — the content-addressed object store, as a CONTRACT | ✅ shipped | `migrations/0028`: `snapshot_objects (digest TEXT PRIMARY KEY, bytes BYTEA NOT NULL)`; the digest is verified against the bytes on write (`DigestMismatch`), identical bytes are one row across tenants, and `migrations/0080` adds an `external-reference` class for bytes held elsewhere |
| §6.3 — the object store's BACKING, outside PostgreSQL | ⚠️ **PostgreSQL `BYTEA`** | no blob-service dependency in any manifest; §6.3's letter is *object store; PostgreSQL stores digest and metadata* |
| publication content **retrievable** by its manifest digest | 🔴 **ABSENT** | the four routes are `POST`/`GET /v1/policy-publications` and `POST …/{id}/{effective,failed,publish}`. There is no read of the bundle at all |

## Decision

**The publication content store is in scope before G9, it is two-thirds built, and the
remaining work is not "choose a store" — it is to READ and to OPERATE what is already
written.** Concretely:

1. ✅ **The Git repository is KEPT and is already the canonical content store.** No choice is
   made here by preference: §6.3 names *signed publication manifest plus immutable Git/object
   content*, §15.7 spells out the ref protocol step by step, and the shipped code is that
   protocol. Cost of keeping it: **zero** — it exists and is green.
2. ✅ **The content-addressed object store is KEPT as a contract and is already honoured.**
   Content-addressing is the property §6.3 actually requires of it — digest-keyed identity,
   dedup, write-time verification — and all three hold today.
3. ⏸️ **An object-store BACKING outside PostgreSQL is DEFERRED**, with the trigger below.
4. 🔴 **Three real gaps are opened as children**, because the roadmap commitments they serve
   are unmet: `.9.3.5.1` (operate the reconciler), `.9.3.5.2` (retrieval by manifest digest),
   `.9.3.5.3` (the book documents none of this).

### The deferral, with a trigger a later pass can EVALUATE

⛔ `.11.4.7.2.1` measured 27 of 27 deferrals naming a phase that had already closed, so this
one names a **measurable condition instead of a milestone**:

> Move `snapshot_objects.bytes` to an external content-addressed backend when **either**
> `fetcher::FetchLimits::max_bytes` is raised above **16 MiB**, **or** `SELECT
> sum(length(bytes)) FROM snapshot_objects` exceeds **10 GiB** in any deployment.

Both are readable on any day, by anyone, without a judgement call — one is a constant in
`fetcher.rs`, the other is one query.

**Why those numbers, rather than a preference.** Every object that can reach the store passes
the fetcher, whose ceiling is **4 MiB** (`fetcher.rs:59`), and the extraction pack's input
ceiling is 16 MiB with a 4 MiB output ceiling. PostgreSQL stores a 4 MiB value out-of-line in
TOAST as a matter of course; the size class that makes an external blob service *necessary*
rather than *tidier* is one the product cannot currently produce. The 10 GiB arm exists
because volume, not object size, is the other way this becomes real.

**The cost, stated rather than implied.**

| option | cost | what it buys today |
| --- | --- | --- |
| keep `BYTEA` | 0 | backup, PITR, transactions and tenancy already cover the bytes — one store to operate, one to restore |
| add an external backend now | a new crate (`reasonbraid-object-store` in §7.2 is still unbuilt), a vendor dependency, a second failure domain, a second thing to back up, and §20's *object-store outage* kill-point becomes reachable | nothing measurable at a 4 MiB ceiling |

⛔ The deferral is about the **backend**, never about content-addressing. If the addressing
contract were ever weakened, this record does not cover it.

## Consequences

- 🔴 **Three published verdicts are corrected in `docs/tasks/SIGNOFF-REPAIR.md`** — tranche 1's
  row 4, tranche 2's row 13 (its object-store clause) and the tranche-1 summary. The row-4
  disposition changes from `fired and open` to **`discharged`**: both spikes the Phase-0
  deferral named were run and shipped, in `PHASE-6.4.3.2` and `PHASE-6.4.3.3`.
- ⭐ **The leaf gets SMALLER and sharper.** It opened as *choose and build a publication
  store*; it closes as *three bounded gaps on a store that already exists*. That is the wave-B
  ordering rule working exactly as `.15` argued it would.
- ⚠️ **`.9.3.5.1` is the one that matters.** A reconciliation matrix nothing runs is a decision
  document with a type-checker: §15.8 says *reconciliation is idempotent and regularly
  exercised under kill points*, and it is exercised by three unit tests and no deployment.
- ⚠️ **Not claimed:** that the Git publication half is complete against §15.7 in every detail.
  Signing (step 3's *for required profiles, sign the manifest*) was not audited by this record
  and is not asserted either way.

## What would make this wrong

- ⛔ If §6.3's *object store* is read as naming a **deployment topology** rather than a storage
  contract, then item 2 is not honoured and the deferral in item 3 is the whole commitment. The
  reading taken here is that §6.3's column is *Authority* — which store is authoritative for the
  bytes — and a digest-keyed table is authoritative in exactly that sense.
- ⛔ If a publication bundle ever needs to be served to a party that cannot be given the Git
  repository, `.9.3.5.2` stops being a gap-closing read surface and becomes a distribution
  mechanism with its own authority rules. Nothing asks for that today.
- ⛔ If the fetcher's ceiling rises for an unrelated reason, the deferral trigger fires by
  design and must be honoured rather than re-argued.

## Alternatives considered

1. **Accept the audit and build a store.** Rejected on measurement: it would have built a
   second Git publisher beside a green one. This is the rework the wave rule exists to prevent,
   and it was one `git log` away from being started.
2. **Adopt an S3-compatible backend now, since §7.2 names `reasonbraid-object-store`.** Rejected:
   §7.2 is a responsibility map, not a delivery schedule, and the responsibility it names —
   *content-addressed blob/artifact abstraction* — is discharged. Adding a vendor dependency to
   satisfy a crate name is preference dressed as compliance.
3. **Defer the whole leaf to G9.** Rejected: two of the three gaps are unmet roadmap
   commitments with shipped machinery already behind them, and a deferral naming a closed-phase
   milestone is the defect `.11.4.7.2.1` measured at 27 of 27.
4. **Close the leaf as fully discharged.** Rejected: it is the mirror of the error being
   retracted. Finding the write path does not license a verdict about the read path — the same
   inference-from-one-observation that produced the false `fired and open`.
