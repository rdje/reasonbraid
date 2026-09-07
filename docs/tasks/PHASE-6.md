# PHASE-6: policy and doctrine governance

## Metadata

- Tree ID: `PHASE-6`
- Status: `proposed`
- Roadmap lane: Phase 6 (`ROADMAP.md` §20.8); Governance track
- Created: `2026-09-05`
- Estimate: 18–30 engineer-weeks plus governance review
- Depends on: authority model, rigorous deliberation, Git/object consistency, correction model
- Exit: G3 and publication portions of G7. A full policy lifecycle — including failed publication recovery and later correction — can be reconstructed without relying on chat prose.

## Goal

Canonical semantic policy, deterministic projections, signed publication,
per-target deployment (not globally atomic), drift, and designed correction.
Consensus does not grant authority. Demonstration B (`ROADMAP.md` §26.2).

## Non-Goals

- Silent overwrite or history deletion.
- Globally atomic multi-repository rollout.
- Universally weaker retraction authority than adoption.

## Task Tree

- ID: `PHASE-6.1`
  Status: `done`
  Goal: semantic policy schema, layer/precedence/exception rules, impact maps, ownership metadata
  Backlog: 38
  ADR: 019
  Roadmap: §15.1–15.3
  Children: `.1.1`–`.1.3` (decomposed `2026-09-07` at the census
    seams): `.1.1` ADR-019 + the census (the semantic-policy
    contract: the `PolicyVersion`/`PolicySetVersion` shapes,
    the seven-step layered resolution with the fail-closed
    rule, the ownership = the authority binding) → `.1.2` the
    policy schema (the typed `PolicyVersion` + the validation
    + the versioned registry + the ownership metadata) →
    `.1.3` the layering + the precedence (the seven-step
    resolution, the exceptions/waivers, the explanation tree,
    the impact maps).
  Done (`2026-09-07`): the census at the seams — the block is
    LIFTED (the frontier row's "blocked on authority,
    deliberation, Git/object, correction model" is resolved:
    the authority ships (Phase 2's boundaries/grants/audit),
    the deliberation ships (Phase 5 closed), the Git/object
    consistency ships (Phase 4's git pack + the snapshots),
    the correction model ships (the revision lineage + the
    supersession/retraction vocabulary rides `.5`). The
    GREENFIELD: no policy schema exists (no `PolicyVersion`,
    no layer/precedence rules, no impact maps, no
    policy-ownership binding — `git grep -c "PolicyVersion"
    HEAD -- crates/ migrations/` → rc=1); the REUSABLE
    pieces: the versioned-config precedent (the Phase-5.1
    profile registry — the versioned entry + the digest
    pinning), the content-addressing (ADR-011), the authority
    model (the grant checks — the §15.3 step-1 substrate),
    the evidence pipeline. The §23 queue's 019 (the canonical
    policy schema + the merge/precedence rules) is THIS
    leaf's ADR. Frontier → `.1.1`.

  - ID: `PHASE-6.1.1`
    Status: `done`
    Goal: ADR-019 + the census — the semantic-policy
      contract: the `PolicyVersion` shape (the §15.1 fields),
      the `PolicySetVersion` (the compatible authorized
      collection + the lock manifest), the seven-step layered
      resolution (the issuer authority, the applicability, the
      dependencies/conflicts, the charter precedence, the
      exceptions, the FAIL-CLOSED binding conflict, the
      explanation tree), the ownership = the authority binding
      (the policy's owning authority rides the grants — the
      label grants nothing, §15.3). No code.
    ADR: 019
    Roadmap: §15.1–15.3
    Done (`2026-09-07`): ADR-019 accepted (evidence-gated) —
      `docs/adr/019-semantic-policy.md` (top-level `answers:`):
      the policy is a versioned digest-pinned DOCUMENT (the
      ADR-011 digest over the canonical bytes — never prose);
      the `PolicyVersion` carries the §15.1 fields (the stable
      clause ids, the applicability + the explicit
      non-applicability, the exception schema, the
      provenance, the triggers); the ownership is the
      AUTHORITY BINDING (the owning authority is a grant
      reference — the label grants nothing, §15.3's rule made
      structural; an unresolvable owning authority is invalid
      at registration); the `PolicySetVersion` is a LOCK
      MANIFEST (every version/digest/dependency/grant basis —
      the §15.2 publication truth); the resolution is the
      seven deterministic steps with the FAIL-CLOSED binding
      conflict (never a silent pick) and the explanation tree
      riding the result; the impact maps are derivable
      coverage, never achievement claims. No code changed.
      Frontier → `.1.2`.

  - ID: `PHASE-6.1.2`
    Status: `done`
    Goal: the policy schema — the typed `PolicyVersion` (the
      §15.1 fields: the stable id + the semantic version +
      the digest + the lifecycle status, the normative
      statements with the stable clause ids, the applicability
      selectors, the dependencies/conflicts/precedence hints,
      the exception schema, the ownership metadata) + the
      validation (the clause ids, the digest, the ownership
      reference) + the versioned registry.
    Roadmap: §15.1, §15.2
    Done (`2026-09-07`): the typed policy schema landed per
      ADR-019 — migration 0038 (`policy_versions`: the §15.1
      fields — the stable clause ids ride the JSONB clauses,
      the applicability + the non-applicability + the
      exception schema + the provenance + the ownership);
      `crates/reasonbraid-server/src/policy.rs` (NEW): the
      `PolicyVersionInput`/`ClauseStatement` shapes, the
      `register` (the ADR-011 digest shape, the semantic
      version, the closed lifecycle vocabulary, the non-empty
      clauses, the UNIQUE stable clause ids, the OWNING
      AUTHORITY must be an ACTIVE grant — the label grants
      nothing, an unresolvable owner is invalid at
      registration; the duplicate version refuses) + the
      `list` (the FromRow struct — the 18 columns exceed the
      tuple impl's ceiling); the api: `POST`/`GET
      /v1/policies` (the enrolled gate); the pg script gained
      the `policy` suite. Measured (policy 1): the document
      registers with the fields echoing, the NEW version of
      the same policy registers, the seven refusals (the
      duplicate, the bad digest, the bad semver, the unknown
      lifecycle, the duplicate clause ids, the empty clauses,
      the ghost authority), the newest-first list, the
      unenrolled refusal. Frontier → `.1.3`.

  - ID: `PHASE-6.1.3`
    Status: `done`
    Goal: the layering + the precedence — the seven-step
      resolution (the issuer authority check, the
      applicability filter, the dependency/conflict
      resolution, the charter precedence/specificity, the
      exceptions/waivers, the fail-closed on the unresolved
      binding conflict, the explanation tree), the impact
      maps (the clause → the target/domain/action coverage).
    Roadmap: §15.3
    Done (`2026-09-07`): the layering + the precedence landed
      per ADR-019 — `policy.rs` gains the `ResolutionRequest`
      (the named set + the target + the requested waivers)
      and the `resolve` (the SEVEN §15.3 steps, each
      recorded: (1) the issuer authority — each owning grant
      must be ACTIVE + unexpired; (2) the applicability
      filter — the layer/target selector match, the empty
      applicability matches everything, the suspended/
      retracted policies excluded; (3) the dependencies must
      be IN the set + the explicit conflicts OUT; (4) the
      precedence hints form a DAG — a cycle is the refusal,
      the `over` edges settle the collisions transitively;
      (5) the requested waivers must ride a policy's
      exception schema; (6) the clause-id collision across
      the applicable policies settles by the precedence or
      FAILS CLOSED — the unresolved binding conflict is the
      typed refusal, never a silent pick; (7) the explanation
      tree rides the result); the `impact` (the derivable
      coverage — the clauses × the declared applicability,
      never an achievement claim); the api: `POST
      /v1/policies/resolve` + `GET
      /v1/policies/{id}/{version}/impact`. Measured (policy
      2): the happy resolution (the precedence wins the c1
      collision, the seven explanation steps, the empty
      conflicts), the fail-closed refusal, the missing
      dependency, the unknown + the allowed waiver, the ghost
      reference, the suspended exclusion, the precedence
      cycle, the impact map + the ghost impact refusal.
      **`.1` COMPLETE** — frontier → `.2`.

- ID: `PHASE-6.2`
  Status: `done`
  Goal: proposal/review/approval records with authority proofs and quorum snapshots
  Roadmap: §4.5, §15.6
  Acceptance: discussion, decision, approval, publication, and deployment remain separate records
  Children: `.2.1`–`.2.3` (decomposed `2026-09-07` at the census
    seams): `.2.1` ADR-032 + the census (the lifecycle contract:
    the five records stay separate — discussion/decision/
    approval/publication/deployment; the proposal references a
    policy version + a deliberation thread; the decision
    references the verdict + the electorate snapshot; the
    approval references the authority proof) → `.2.2` the
    proposal + the decision records → `.2.3` the approval
    records + the authority proofs + the quorum snapshots.
  Done (`2026-09-07`): the census at the seams. The §15.6
    lifecycle records are a GREENFIELD: no proposal record, no
    approval record, no decision record exists (`git grep -c
    "proposal" HEAD -- crates/reasonbraid-server/src/` → the
    hits are the profile/step names, not lifecycle records).
    The REUSABLE pieces: the `.1` policy registry (the
    proposal targets a policy version), the Phase-5
    deliberation machinery (the proposal's discussion rides a
    THREAD — the contribute/challenge/revise/verdict verbs,
    the twelve terminals, the minority report; "one thread
    may yield multiple decisions" maps to the verdict kind),
    the Phase-2 authority model (the grants/audit — the
    authority proof's substrate), the Phase-5 electorate
    facts (the thread's participants — the quorum snapshot's
    source). The §23 queue has no lifecycle entry (020/021
    are the `.4`/`.5` leaves') — the lane opens ADR-032.
    Frontier → `.2.1`.

  - ID: `PHASE-6.2.1`
    Status: `done`
    Goal: ADR-032 + the census — the policy-lifecycle
      contract: the five records stay SEPARATE (the
      discussion, the decision, the approval, the
      publication, the deployment — §15.6's closure list);
      the proposal references a policy version + a
      deliberation thread; the decision references the
      verdict + the ELECTorate snapshot (the authority at
      the action time — §4.5); the approval references the
      authority proof (the grant check at the approval
      boundary). No code.
    ADR: 032
    Roadmap: §4.5, §15.6
    Done (`2026-09-07`): ADR-032 accepted (evidence-gated) —
      `docs/adr/032-policy-lifecycle.md` (top-level
      `answers:`): the five records NEVER fold (the
      discussion rides the existing THREAD machinery — the
      verdict kind is its decision-shaped contribution, one
      thread → multiple decisions; the decision/approval/
      publication/deployment are their own typed rows
      referencing the previous stage's id; a stage skipping
      its predecessor is the typed refusal); the proposal is
      a REFERENCE (the policy version + the digest + the
      thread — never a content copy); the decision freezes
      the ELECTorate snapshot at the action time (the
      participants + the denominator + the abstentions —
      §4.5's "at action time"); the approval carries its
      AUTHORITY PROOF (the grant re-checked at the approval
      boundary — a lapsed grant refuses); the publication/
      deployment record ids are reserved for the `.4`/`.5`
      leaves. No code changed. Frontier → `.2.2`.

  - ID: `PHASE-6.2.2`
    Status: `proposed`
    Goal: the proposal + the decision records — the typed
      proposal (the target policy version + the thread
      reference + the status), the decision record (the
      rule + the electorate snapshot + the verdict
      reference — one thread, multiple decisions), the
      lifecycle status machine (the draft → decided →
      approved → published → deployed stages as the typed
      statuses).
    Roadmap: §15.6

  - ID: `PHASE-6.2.3`
    Status: `proposed`
    Goal: the approval records + the authority proofs — the
      approval row (the proposal + the approver + the
      AUTHORITY PROOF: the grant check at the approval
      boundary — the §4.5 identity/authority at the action
      time), the quorum snapshot (the electorate + the
      denominator + the abstentions), the separate-record
      enforcement (the discussion/decision/approval are
      distinct rows, never folded).
    Roadmap: §4.5, §15.6

- ID: `PHASE-6.3`
  Status: `proposed`
  Goal: deterministic compiler plus initial Codex and Claude-family projections; unrepresentable clauses fail closed
  Roadmap: §15.5

- ID: `PHASE-6.4`
  Status: `proposed`
  Goal: signed canonical publication protocol; kill-point-tested Git/PostgreSQL reconciliation
  Backlog: 39
  ADR: 020
  Roadmap: §15.7–15.8

- ID: `PHASE-6.5`
  Status: `proposed`
  Goal: canary target deployment, PR/apply adapters, receipts, drift, waivers, suspension, supersession, retraction
  ADR: 021
  Roadmap: §15.9–15.11, §4.7

- ID: `PHASE-6.6`
  Status: `proposed`
  Goal: outcome monitoring and scheduled review triggers
  Roadmap: §15.11

- ID: `PHASE-6.7`
  Status: `proposed`
  Goal: G3 exit + Demonstration B; publication portion of G7; subtraction record
  Gate: G3; G7 publication portion
  Kill/pivot: do not ship binding policy governance unless real owners accept the authority/correction model (`ROADMAP.md` §25.1)

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `PHASE-6.2.2` | `proposed` | `.2.1` done — ADR-032 accepted (the five records never fold, the proposal is a reference, the decision freezes the snapshot, the approval re-checks the grant); the proposal + the decision records execute next |

## Changelog

- `2026-09-07`: `.2.1` done — ADR-032 accepted (the
  policy-lifecycle contract: the five records never fold, the
  proposal is a reference, the decision freezes the
  electorate snapshot, the approval re-checks the grant); no
  code; frontier → `.2.2`.
- `2026-09-07`: `.2` decomposed at the census seams — the
  lifecycle records are the greenfield (no proposal/approval/
  decision row exists; the threads + the authority model +
  the `.1` registry are the substrate); children `.2.1`
  (ADR-032 + the census) → `.2.2` (the proposal + the
  decision records) → `.2.3` (the approvals + the proofs);
  frontier → `.2.1`.
- `2026-09-07`: `.1.3` done — the seven-step layering +
  precedence (the fail-closed resolution with the
  explanation tree + the impact maps); policy 2; **`.1`
  COMPLETE** — frontier → `.2`.
- `2026-09-07`: `.1.2` done — the typed policy schema
  (migration 0038: the §15.1 document with the stable clause
  ids, the authority-checked ownership, the versioned
  registry); policy 1; frontier → `.1.3`.
- `2026-09-07`: `.1.1` done — ADR-019 accepted (the
  semantic-policy contract: the digest-pinned document, the
  ownership = the authority binding, the seven-step
  fail-closed resolution, the lock-manifest set); no code;
  frontier → `.1.2`.
- `2026-09-07`: `.1` decomposed at the census seams — the block
  is LIFTED (the authority/deliberation/Git-object/correction
  dependencies all ship; the policy schema is the greenfield);
  children `.1.1` (ADR-019 + the census) → `.1.2` (the policy
  schema) → `.1.3` (the layering + the precedence); frontier
  → `.1.1`.
- `2026-09-05`: Created from `ROADMAP.md` §20.8, §15, §26.2, backlog 38–39.


## Acceptance Checklist (PHASE-6.1.2)

The CODE change owned by this leaf:
`migrations/0038_policy_registry.sql` (NEW — the policy table),
`crates/reasonbraid-server/src/policy.rs` (NEW — the shapes +
the register + the list), `crates/reasonbraid-server/src/
lib.rs` (the module), `crates/reasonbraid-server/src/api.rs`
(the two verbs), `crates/reasonbraid-server/tests/policy.rs`
(NEW — the suite), `scripts/run_pg_tests.sh` (the policy
suite joins the guard) — `\.rs$` + `(^|/)migrations/` +
`scripts/`.

- [x] **REPRODUCE / ISSUE** — the pre-leaf surface: no policy
  schema existed (the `.1` census — `git grep -c
  "PolicyVersion" HEAD -- crates/ migrations/` → rc=1).
- [x] **ROOT CAUSE (WHY + WHERE)** — `git grep -c
  "policy_versions\|PolicyVersionInput\|ClauseStatement"
  c6ed5e7 -- crates/ migrations/` → rc=1 (nothing before this
  leaf). The fix point is the ADR-019 schema: the typed
  document + the authority-checked ownership + the versioned
  registry.
- [x] **ADDRESSED (verified)** — measured before→after. Before:
  the grep above. After: `DATABASE_URL=… cargo test -p
  reasonbraid-server --test policy
  the_policy_registry_validates_the_digest_pinned_document` →
  `test result: ok. 1 passed` — the document registers (the
  fields echo), the NEW version registers, the seven refusals
  (the duplicate / the bad digest / the bad semver / the
  unknown lifecycle / the duplicate clause ids / the empty
  clauses / the GHOST owning authority), the newest-first
  list, the unenrolled refusal. The first live pass caught
  the list-count miscount (two rows, not three) — fixed.
- [x] **NO REGRESSION** — `cargo test --all` → rc=0, 58 suites;
  `bash scripts/run_pg_tests.sh` → rc=0, 21 live suites + the
  demo `ALL acceptance checks passed`
  (`target/pg525_guard.log`);
  `cargo clippy --all --all-targets -- -D warnings` → rc=0;
  `cargo fmt --all -- --check` → rc=0; `make gate` → 13/13 at
  commit.
- [x] **FIX** — `0038_policy_registry.sql`, `src/policy.rs`,
  `src/lib.rs`, `src/api.rs`, `tests/policy.rs`,
  `scripts/run_pg_tests.sh`.
- [x] **LOCKSTEP** — CHANGELOG, MEMORY, LIVE_STATUS, this tree's
  logs above, `docs/TASK_TREE.md` frontier — same commit (the
  KNOWLEDGE_MAP regen produced no diff; no new DEV_NOTES
  heading).


## Acceptance Checklist (PHASE-6.1.3)

The CODE change owned by this leaf:
`crates/reasonbraid-server/src/policy.rs` (the
`ResolutionRequest`/`ResolutionTarget`/`PolicyRef`/
`ResolvedClause`/`Resolution` shapes, the seven-step
`resolve`, the `impact` map), `crates/reasonbraid-server/src/
api.rs` (the two verbs), `crates/reasonbraid-server/tests/
policy.rs` (the new test) — `\.rs$`.

- [x] **REPRODUCE / ISSUE** — the pre-leaf surface: the
  registry stored the policies but nothing resolved them —
  no layering, no precedence, no conflict rule, no impact
  map (the `.1` census's greenfield).
- [x] **ROOT CAUSE (WHY + WHERE)** — `git grep -c
  "ResolutionRequest\|binding_conflict\|fn resolve" d60c459
  -- crates/reasonbraid-server/src/policy.rs` → rc=1
  (nothing before this leaf). The fix point is the ADR-019
  resolution contract: the seven deterministic steps with
  the fail-closed binding conflict.
- [x] **ADDRESSED (verified)** — measured before→after. Before:
  the grep above. After: `DATABASE_URL=… cargo test -p
  reasonbraid-server --test policy
  the_seven_step_resolution_fails_closed` → `test result:
  ok. 1 passed` (also inside the full live suite: `running 2
  tests … ok`) — the happy resolution (the precedence wins
  the c1 collision, the seven explanation steps, the empty
  conflicts), the fail-closed refusal, the missing
  dependency, the unknown + the allowed waiver, the ghost
  reference, the suspended exclusion, the precedence cycle,
  the impact map + the ghost impact refusal.
- [x] **NO REGRESSION** — `cargo test --all` → rc=0, 58 suites;
  `bash scripts/run_pg_tests.sh` → rc=0, 21 live suites + the
  demo `ALL acceptance checks passed`
  (`target/pg526_guard.log`);
  `cargo clippy --all --all-targets -- -D warnings` → rc=0;
  `cargo fmt --all -- --check` → rc=0; `make gate` → 13/13 at
  commit.
- [x] **FIX** — `src/policy.rs`, `src/api.rs`,
  `tests/policy.rs`.
- [x] **LOCKSTEP** — CHANGELOG, MEMORY, LIVE_STATUS, this tree's
  logs above, `docs/TASK_TREE.md` frontier — same commit (the
  KNOWLEDGE_MAP regen produced no diff; no new DEV_NOTES
  heading).
