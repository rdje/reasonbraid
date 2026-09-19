---
answers:
  - Does the Phase 6 G3 gate record still hold against the repaired code?
  - Which of G3's authority/consent/quorum/publication/correction claims had to be re-earned?
  - Why can Demonstration B's evidence column no longer be checked?
---
# G3's seven claims, re-derived: three stand, one narrows, three had to be re-earned — and the evidence pointers no longer resolve

- Date: 2026-09-19
- Status: accepted
- Owner: `SIGNOFF-REPAIR.11.4.7.3` (under `.11.4.7`, blocker **C2**)
- Supersedes nothing; **`docs/decisions/2026-09-07_phase6-gate-record.md` is byte-unchanged** —
  `docs/decisions/` supersedes rather than mutates, and `.11.4.7` forbids editing the record.
- Related: `docs/evidence/2026-09-07_demonstration-b.md` (the nine-step walk and the clause map),
  `docs/decisions/2026-09-15_g1g2-sixteen-claims-re-derived.md` (the sibling, `.11.4.7.2`).

## Context

`.11.4.7` opened on the director's question of 2026-09-15 — *what is blocking the internet
exposure?* — and found that the tree could answer for the three external gaps and could not
honestly answer for the lines it counts as shipped. Its rule: re-derive every countable claim
of all five gate records against the **repaired** code and the suites that exist **today**,
with a verdict from the closed set `stands` / `narrowed` / `must be re-earned`, and the command
that produces it.

This is G3 — `ROADMAP.md` §20.6, the authority/consent/quorum/publication/correction gate that
blocks binding policy use. **Seven claims**: the outcome, the five clauses, and Demonstration B.

## The finding that came before any verdict: the evidence pointers are positions

G3's evidence is cited as `policy 4`, `policy 10`, `publisher 2`, `compiler 8` — an index into
a test file's source order. Positions are not identities. Resolved against the record's own
commit (`53ecd72`) and against `HEAD`:

```bash
git show 53ecd72:crates/reasonbraid-server/tests/policy.rs | grep -c '^async fn the_'   # 11
grep -c '^async fn ' crates/reasonbraid-server/tests/policy.rs                          # 14
```

🔴 **Two of the ten cited `policy` pointers no longer name the test they named**, because three
tests were inserted at positions 9, 10 and 14:

| pointer | cited for | resolved at `53ecd72` | resolves today |
| --- | --- | --- | --- |
| `policy 9` | Demonstration B step 8 (the canary deployment) | `the_deployment_rides_the_effective_publication_per_target` | `the_publish_verb_stays_inside_the_configured_repository_root` |
| `policy 10` | **the correction clause**, and Demo B step 9 | `the_drift_corrections_and_outcomes_ride_the_records` | `the_publication_verbs_require_an_authority_the_caller_holds` |

The other eight survive **by accident** — every insert landed after the highest one they use.

⭐ **And three pointers cannot be resolved from the document at all.** When the record was
written, `publisher` held exactly **2** tests, `reconciler` **3** and `compiler` **8** — each
equal to the highest index cited against it. So `publisher 2` is ambiguous between *"the
suite's two tests"* and *"test number two"*, and the two readings have since diverged:
`publisher` now holds **7**. Under the first reading the count is false; under the second the
pointer names a different test. The notation can only be disambiguated by reconstructing the
file at the record's own commit, which is precisely what a published evidence pointer exists
to avoid.

⛔ **This is not repaired in place.** The dated record and the dated evidence file are history;
the corrected pointers live in the verdict table below, **by test name**.

## The verdicts

Each is from the closed set `stands` / `narrowed` / `must be re-earned`, and each names the
command that produces it. Live evidence, run 2026-09-19 over every suite the record cites:

```bash
bash scripts/run_pg_tests.sh policy invitations            # policy 14/14, invitations 6/6, rc=0
python3 -B scripts/project_env.py cargo test --locked \
  -p reasonbraid-server --test publisher --test reconciler \
  -p reasonbraid-policy-compiler --test compiler           # 8/8, 7/7, 3/3, rc=0
```

**38 tests, 0 failures**, across the two database-backed suites and the three offline ones.

### 1. The outcome — "Met as machinery, blocked as binding use"

**stands.** Both halves hold. The machinery: `policy` is **14 passed / 0 failed**. The block:
`git grep -niE "binding policy|binding governance" -- docs/book/src README.md LIVE_STATUS.md`
returns exactly **one** hit, `docs/book/src/qualification-review.md:12`, and it states that
binding governance claims *remain gated by G6/G7*. Nothing has quietly claimed the binding use
the gate withheld.

### 2. The authority clause — the grant re-checks at the decision/approval/correction boundaries, plus the `.1.2` ownership binding

🔴 **must be re-earned — and it has been, by work that post-dates the gate.** `.9.3.1`
(REPAIR-0184) measured `corrections.rs::authority_holds(pool, grant_id)` taking the grant id
**and nothing else**: `SELECT EXISTS (… WHERE grant_id = $1 AND status = 'active' …)`. It never
received the caller, the action, the selector, the boundary or `valid_from` — it asked whether
a grant *exists*, not whether the caller *holds* it. The census found **5 sites in 4 spellings**
across `corrections`, `deployments`, `lifecycle` and `policy`, and the **approval** surface was
among them. All three boundaries this clause names were asking the wrong question. Separately,
`.9.2.1.2` (REPAIR-0192) found both publish verbs authorized by enrolment alone.

The clause's own evidence could not have caught either: `policy 1` and `policy 4` are green
against both the defective and the repaired code. What covers it today did not exist at the
gate — `citing_an_authority_requires_holding_it` and
`the_publication_verbs_require_an_authority_the_caller_holds`, both green in the run above.

⚠️ **Residual, open and not closed by this leaf.** `GrantAction` has ten variants, all
thread-shaped plus `TenantAdmin`, and `TargetSelector` is `TenantWide | Threads{…}`;
`git grep -n "Publication\|DeploymentTarget\|Correction" -- crates/reasonbraid-core/src/authority.rs`
returns **rc=1**. A grant cannot *express* coverage of a publication or a correction target, so
"the grant covers this action" is not a checkable question on those two surfaces. `.9.3.1`
recorded this; it stands.

### 3. The consent clause — the Phase-1 invitation-response machinery

**stands.** `invitations` is **6 passed / 0 failed**, covering the accept/remove lifecycle, the
concurrent accept-versus-decline single-winner property, decline/expiry/reinvitation, join-request
enforcement, and participant removal under tenant binding and delegation attenuation.

⚠️ Stated rather than left implicit: this is the **only** clause the record cites by lane and
not by test. Nothing in the record says which control carries it, so the re-derivation had to
choose the suite. The claim is true; the record does not make it checkable.

### 4. The quorum clause — the electorate snapshots frozen at the action time

**stands.** `policy 3` still resolves to `the_proposal_and_the_decision_stay_separate_records`,
and it passes.

### 5. The publication clause — the nine-step machine, the compare-and-swap, the six-row reconciliation

🔴 **must be re-earned — three live defects were found inside this exact lane after the gate,
and one of them was inside the clause's own evidence.**

- `.9.2.1.1` (REPAIR-0190) — the publish verb took its repository location from the request body.
- `.9.2.1.2` (REPAIR-0192) — both publish verbs were authorized by enrolment alone.
- `.9.2.1.3` (REPAIR-0191) — `mark_effective` recorded Git object ids it never looked for.

⭐ **The sharpest point is not the count, it is where one of them was.** `.9.2.1.1` found a
pre-existing defect in the third leg of `the_publish_verb_drives_the_git_half` — `policy 8`,
one of the two policy tests this clause cites. Part of the clause's evidence was vacuous at the
moment the gate was taken.

Today the clause stands on `policy 7`, `policy 8`, the offline `publisher` and `reconciler`
suites, and two controls that did not exist at the gate
(`the_publish_verb_stays_inside_the_configured_repository_root`,
`the_publication_verbs_require_an_authority_the_caller_holds`).

### 6. The correction clause — the `.5.3` §4.7 operations with the authority proofs

🔴 **must be re-earned.** Its *only* pointer, `policy 10`, no longer names the corrections test
(table above). And the "authority proofs" half is exactly what `.9.3.1` measured broken:
`corrections.rs` is the site that leaf **opened** on. The corrections control,
`the_drift_corrections_and_outcomes_ride_the_records`, is green today, and the authority half
was repaired by REPAIR-0184 — neither fact was available to the gate.

### 7. Demonstration B — the nine-step walk

**narrowed.** The walk still maps each of §26.2's nine steps onto machinery that still exists,
and the suites behind those steps are green. What it can no longer do is be *checked*: its
evidence column is thirteen positional pointers, two of which now resolve to different tests
and three of which are ambiguous by construction. The walk stands as a mapping and no longer
stands as a citation.

## Tally

**3 stand · 1 narrows · 3 must be re-earned** — and all three of the latter have since been
discharged by named repairs (REPAIR-0184, 0190, 0191, 0192), with one residual named above.

⛔ **The G3 blocker is unchanged**: binding policy use is still not claimed, and the real
owners' acceptance remains the open §25.1 kill/pivot condition. Nothing in this re-derivation
narrows or widens that.

## The leaf's own sub-question: the record's private-visibility sentence

`.11.4.7.3` asked whether this record needs a superseding note for its abbreviated copular
sentence — *"the authority/correction model has no external owners yet (the repo is private)"*,
which the `VISIBILITY-POLICY` gate cannot see because the subject and the predicate are split.

**Answer: no, and adding one would be a second copy nothing derives.** The sentence is already
registered verbatim in `.doctrine/visibility_exceptions.txt`, with its reason — *preserved
history; `docs/decisions/` supersedes rather than mutates* — and the correction already exists
as its own dated record, `docs/decisions/2026-09-09_public-repository-policy.md`. A third
statement of the same fact is the drift shape this tree keeps repairing.

⚠️ One thing the exception does not say, and it belongs here: the sentence's **premise** is
stale but the **deferral** it supports is not. The repository is public; it still has no
external owners who have accepted the authority/correction model, so deferral (2) stands on
its own terms and this re-derivation does not disturb it.

## Consequences

- The gate record and Demonstration B stay byte-unchanged; this record is the current reading.
- The positional-pointer defect is **not** G3's alone. `.11.4.7.1` (G6–G7) and `.11.4.7.4`
  (G4–G5) must resolve their own pointers against their own commits rather than against
  today's files, and this record is the reason.
- ⛔ **No doctrine gate is proposed for positional pointers.** `.11.6` forbids a threshold
  before its population, and the population here is four hand-written records; a pattern loose
  enough to catch `publisher 2` would also catch every ordinary number in prose. The remedy is
  the convention this record follows — cite the test by name — not a check.
