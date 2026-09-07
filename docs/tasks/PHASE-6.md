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
    Status: `done`
    Goal: the proposal + the decision records — the typed
      proposal (the target policy version + the thread
      reference + the status), the decision record (the
      rule + the electorate snapshot + the verdict
      reference — one thread, multiple decisions), the
      lifecycle status machine (the draft → decided →
      approved → published → deployed stages as the typed
      statuses).
    Roadmap: §15.6
    Done (`2026-09-07`): the proposal + the decision records
      landed per ADR-032 — migration 0039
      (`policy_proposals`: the proposal = a REFERENCE (the
      policy version + the thread) with the typed status
      vocabulary (draft/decided/approved/published/deployed/
      withdrawn); `policy_decisions`: the rule + the frozen
      ELECTorate snapshot + the verdict reference);
      `crates/reasonbraid-server/src/lifecycle.rs` (NEW):
      the `register_proposal` (the policy version + the
      thread must exist — the references resolve, never
      dangle), the `record_decision` (the proposal must be
      at `draft` — one proposal, one decision; the
      electorate names at least one participant; the verdict
      must be a `verdict`-kind contribution of the
      PROPOSAL's thread — the foreign verdict refuses; the
      decision advances the proposal to `decided`), the list
      verbs; the api: `POST`/`GET /v1/policy-proposals` +
      `POST`/`GET /v1/policy-decisions`. Measured (policy
      3): the proposal registers as a reference; the ghost
      policy + the ghost thread refusals; the decision with
      the frozen snapshot + the verdict; the stage advance;
      the second-decision refusal (the stage gate); the
      foreign-verdict + the empty-electorate refusals; the
      lists. Frontier → `.2.3`.

  - ID: `PHASE-6.2.3`
    Status: `done`
    Goal: the approval records + the authority proofs — the
      approval row (the proposal + the approver + the
      AUTHORITY PROOF: the grant check at the approval
      boundary — the §4.5 identity/authority at the action
      time), the quorum snapshot (the electorate + the
      denominator + the abstentions), the separate-record
      enforcement (the discussion/decision/approval are
      distinct rows, never folded).
    Roadmap: §4.5, §15.6
    Done (`2026-09-07`): the approval records landed per
      ADR-032 — migration 0040 (`policy_approvals`: the
      approval is its OWN row — never folded into the
      decision — with the approver, the grant id, and the
      quorum snapshot); `lifecycle.rs` gains the
      `record_approval` (the proposal must be at `decided` —
      the stage gate; the decision must BELONG to the
      proposal — a foreign decision refuses; the AUTHORITY
      PROOF: the grant must be ACTIVE, unexpired, and HELD
      BY the approver (the §4.5 identity/authority at the
      action time) — a mismatched or unknown grant refuses;
      the non-empty quorum; the approval advances the
      proposal to `approved`) + the list; the api: `POST`/
      `GET /v1/policy-approvals`. Measured (policy 4): the
      full chain (the proposal → the decision → the
      approval with the matching grant), the stage advance,
      the second-approval refusal, the mismatched-proof
      refusal, the foreign-decision refusal, the draft-stage
      refusal, the empty-quorum refusal, the list. The first
      live pass caught the suite's purge gap (the lifecycle
      tables leaked between the tests) + the shared
      WrongStage message shape — fixed. **`.2` COMPLETE** —
      frontier → `.3`.

- ID: `PHASE-6.3`
  Status: `done`
  Goal: deterministic compiler plus initial Codex and Claude-family projections; unrepresentable clauses fail closed
  Roadmap: §15.5
  Children: `.3.1`–`.3.3` (decomposed `2026-09-07` at the census
    seams): `.3.1` ADR-033 + the census (the compiler contract:
    the deterministic rendering — the same inputs + compiler +
    profile + target parameters produce BYTE-IDENTICAL output;
    the target vocabulary; the UNREPRESENTABLE clause is the
    DECLARED refusal, never a silent omission) → `.3.2` the
    compiler core (the resolved set → the generic bundle +
    the policy.lock; the deterministic serialization) → `.3.3`
    the Codex + the Claude projections (the AGENTS.md +
    CLAUDE.md renderers, the unrepresentable declarations,
    the projection tests — the loss/ordering/escaping/size/
    harness-conflict coverage).
  Done (`2026-09-07`): the census at the seams. The compiler
    is a GREENFIELD: no projection exists, no target
    vocabulary, no unrepresentable declaration (`git grep -c
    "projection" HEAD -- crates/` → the hits are the thread/
    profile projections, not the policy compiler). The
    REUSABLE pieces: the `.1` resolution (the resolved clause
    set is the compiler's input), the `.1` registry's digest
    shapes (the byte-identical proof's substrate), the
    policy.lock's inputs (the set + the authority facts).
    The §23 queue has no compiler entry — the lane opens
    ADR-033. Frontier → `.3.1`.

  - ID: `PHASE-6.3.1`
    Status: `done`
    Goal: ADR-033 + the census — the compiler contract: the
      deterministic rendering (the byte-identical guarantee),
      the target vocabulary (the generic bundle, the
      AGENTS.md, the CLAUDE.md, the policy.lock — the
      §15.5 list's initial set), the unrepresentable clause
      as the DECLARED refusal (never a silent omission), the
      hermetic build rule (the compiler is a separate crate —
      the clean-worker doctrine). No code.
    ADR: 033
    Roadmap: §15.5
    Done (`2026-09-07`): ADR-033 accepted (evidence-gated) —
      `docs/adr/033-projection-compiler.md` (top-level
      `answers:`): the compiler renders the RESOLVED set,
      never re-resolves (a re-resolving compiler would be a
      second judge — the ADR-017 trap); the rendering is
      byte-identical by construction (the pure function of
      the ordered clause list — no clock, no ambient state;
      the projection's ADR-011 digest is the §15.7
      publication's verification primitive); the target
      vocabulary is the initial §15.5 set (the generic
      bundle, the AGENTS.md, the CLAUDE.md, the policy.lock —
      the MCP/host/checklist targets are named deferrals: a
      target without an adapter is the typed refusal); the
      unrepresentable clause is a DECLARED refusal (the
      `unrepresentable` list with the reason — never a
      silent omission); the compiler is a separate HERMETIC
      crate (no database/network/ambient state — the
      clean-worker doctrine, the extraction precedent). No
      code changed. Frontier → `.3.2`.

  - ID: `PHASE-6.3.2`
    Status: `done`
    Goal: the compiler core — the resolved clause set → the
      GENERIC instruction bundle + the `policy.lock` (the
      versions, the digests, the dependencies, the authority
      basis), the deterministic serialization (the stable
      renderer — the byte-identical proof), the projection
      record (the target + the profile + the digest).
    Roadmap: §15.5
    Done (`2026-09-07`): the compiler core landed per ADR-033 —
      `crates/reasonbraid-policy-compiler` (NEW, hermetic:
      no database/network/clock — serde + sha2 only): the
      `CompileRequest`/`InputClause`/`LockedPolicy`/
      `CompiledArtifact`/`Unrepresentable` shapes, the
      `compile` (the pure function: the STABLE sort — the
      clause id then the policy id — the generic bundle
      renderer with the escaping, the `policy.lock` renderer
      with the stable rows, the unrepresentable declaration
      for the control-char statements, the unknown-target
      refusal), the ADR-011 digest over the rendered bytes;
      the server: migration 0041 (`policy_projections`: the
      artifact record), `crates/reasonbraid-server/src/
      projections.rs` (NEW: the `project` — the `.1.3`
      resolve → the compile → the record), the api: `POST`/
      `GET /v1/policy-projections`. Measured (compiler 5 +
      policy 5): the byte-identical repeat, the shuffled
      stable order, the newline escaping + the control-char
      declaration, the lock rows, the unknown-target
      refusal; the live suite: the generic projection with
      the declared unrepresentable, the repeat's identical
      digest, the lock projection, the duplicate refusal.
      Frontier → `.3.3`.

  - ID: `PHASE-6.3.3`
    Status: `done`
    Goal: the Codex + the Claude projections — the AGENTS.md
      + the CLAUDE.md renderers over the same core, the
      UNREPRESENTABLE declarations (a clause that cannot ride
      a target names itself — never a silent drop), the
      projection tests (the loss/ordering/escaping/size
      limits/harness conflicts — §15.5's list).
    Roadmap: §15.5
    Done (`2026-09-07`): the Codex + the Claude projections
      landed per ADR-033 — the compiler crate gains the
      `codex` (the AGENTS.md fragment: the backticked clause
      ids — the codex convention) + the `claude` (the
      CLAUDE.md fragment: the plain ids) renderers over the
      same stable-sorted core; the §15.5 coverage: the
      escaping (the backtick escape — the statement backticks
      must not break the harness's parse), the size limit
      (the 8192-char statement ceiling — the oversized
      statement DECLARES itself, never truncates), the
      ordering + the loss (the stable sort + the shared
      unrepresentable path), the harness shapes (the two
      bundles' distinct conventions); the byte-identical
      guarantee rides every target. Measured (compiler 8 +
      policy 6): the codex backticked ids + the claude plain
      ids, the per-target repeat, the oversized-statement
      declaration, the backtick escape; the live suite: the
      two targets through the projection verb with their
      harness shapes + the digests. **`.3` COMPLETE** —
      frontier → `.4`.

- ID: `PHASE-6.4`
  Status: `done`
  Goal: signed canonical publication protocol; kill-point-tested Git/PostgreSQL reconciliation
  Backlog: 39
  ADR: 020
  Roadmap: §15.7–15.8
  Children: `.4.1`–`.4.3` (decomposed `2026-09-07` at the census
    seams): `.4.1` ADR-020 + the census (the publication
    contract: the nine-step §15.7 state machine, the
    publication records, the immutable-ref rules, the §15.8
    reconciliation matrix) → `.4.2` the publication records +
    the staging (the publication aggregate over the decision +
    the approval + the projection, the staged state, the
    manifest digests) → `.4.3` the Git publication + the
    reconciliation (the ref writes + the compare-and-swap,
    the reconciliation matrix rows, the kill-point tests).
  Done (`2026-09-07`): the census at the seams. The
    publication INPUTS ship: the decisions + the approvals
    (`.2` — the authority proofs), the byte-identical
    projections + the digests (`.3` — the verification
    primitives). The SUBSTRATE ships: the Git machinery
    (Phase 4's gix pack + the snapshots), the CA/signature
    keys (the Phase-2 certificate infra), the transactional
    outbox (the §15.7 step-4's "one transaction stores
    staged + outbox"). The GREENFIELD: no publication
    record, no staged state, no ref protocol, no
    reconciliation (the §15.8 matrix's rows exist nowhere).
    The §23 queue's 020 (the Git publication refs +
    signatures + reconciliation) is THIS leaf's ADR.
    Frontier → `.4.1`.

  - ID: `PHASE-6.4.1`
    Status: `done`
    Goal: ADR-020 + the census — the publication contract:
      the nine-step §15.7 state machine (the lock, the
      clean-worker compile, the sign, the one-transaction
      staged + outbox, the staging write, the fetch-back
      verify, the immutable ref + the effective channel via
      the compare-and-swap, the effective record, the
      deployment offers), the publication records (the
      staged/effective/failed states), the §15.8
      reconciliation matrix (the PostgreSQL/Git state pairs →
      the reconciler action — the idempotent, the
      kill-point-exercised, the never-silent-promote). No
      code.
    ADR: 020
    Roadmap: §15.7–15.8
    Done (`2026-09-07`): ADR-020 accepted (evidence-gated) —
      `docs/adr/020-canonical-publication.md` (top-level
      `answers:`): the publication is the NINE-STEP §15.7
      state machine over the staged record (each step typed —
      a step that cannot complete is the typed failure,
      never a skip); the publication record is the machine's
      aggregate (the decision + the approval + the projection
      references + the manifest digest + the
      staged/effective/failed state + the Git object ids —
      never folded into the approval); the Git refs are the
      publication truth, PostgreSQL coordinates (the
      compare-and-swap is the idempotency primitive — a ref
      that moved is the typed failure, never a force-push);
      the reconciliation is the SIX §15.8 rules (the
      idempotent retry, the verify-and-advance, the
      stop-and-alert, the freeze-and-repair, the
      quarantine-and-adjudicate — never a silent promote, the
      out-of-band alert); the signatures ride the MANIFEST
      digest, never the prose. No code changed. Frontier →
      `.4.2`.

  - ID: `PHASE-6.4.2`
    Status: `done`
    Goal: the publication records + the staging — the
      publication aggregate (the decision + the approval +
      the projection references), the staged state, the
      manifest (the digests + the authority basis), the
      stage machine (the staged → effective → failed
      transitions).
    Roadmap: §15.7
    Done (`2026-09-07`): the publication records landed per
      ADR-020 — migration 0042 (`policy_publications`: the
      aggregate row — the decision/approval/projection
      references + the manifest digest + the typed state +
      the Git object ids + the failure reason);
      `crates/reasonbraid-server/src/publications.rs`
      (NEW): the `stage` (the §15.7 steps 1–4's record
      half: the proposal must be APPROVED — the chain gate;
      the decision + the approval must BELONG to the
      proposal; the projection must exist; the manifest
      digest rides the ADR-011 shape), the `mark_effective`
      (the staged → effective with the NON-EMPTY Git object
      ids — the §15.7 step 8), the `mark_failed` (the typed
      failure with the reason — never a skip), the list; the
      api: `POST`/`GET /v1/policy-publications` + `POST
      /v1/policy-publications/{id}/effective` + `POST
      /v1/policy-publications/{id}/failed`. Measured (policy
      7): the full chain (the policy → the thread + the
      verdict → the proposal → the decision → the approval →
      the projection → the staged publication), the
      effective transition with the object ids, the bad-
      digest + the ghost-projection + the ghost-decision
      refusals, the terminal re-transition refusal, the
      second chain's typed FAILURE with the reason, the
      newest-first list. Frontier → `.4.3`.

  - ID: `PHASE-6.4.3`
    Status: `done`
    Goal: the Git publication + the reconciliation — the
      staging-branch write + the fetch-back verification,
      the immutable ref + the effective channel via the
      compare-and-swap, the §15.8 reconciliation matrix
      (the six state pairs → the actions), the kill-point
      tests (the idempotent reconciler, the never-silent
      promote).
    Roadmap: §15.7–15.8
    Children: `.4.3.1`–`.4.3.3` (decomposed `2026-09-07` at
      the census seams): `.4.3.1` the publication-store
      contract + the census (the local bare repository, the
      ref scheme, the write path's gix surface — the
      decision record) → `.4.3.2` the Git publication half
      (the staging write + the fetch-back verification + the
      ref compare-and-swap) → `.4.3.3` the reconciliation
      matrix + the kill-point tests (the six §15.8 rules,
      the idempotent reconciler, the never-silent promote).
    Done (`2026-09-07`): the census at the seams. The WRITE
      half is a greenfield: the Phase-4 R1 pack ACQUIRES
      (the gix clone/fetch — no commit/ref-write path
      exists: `git grep -c "write.*ref\|commit_tree" HEAD --
      crates/reasonbraid-server/src/git.rs` → rc=1); the
      §15.8 reconciler exists nowhere. The reusable pieces:
      the `.4.2` publication records (the staged/effective/
      failed states — the matrix's DB half), the gix library
      (the object + the ref plumbing the write path needs),
      the digest machinery (the fetch-back verification's
      primitive). The store: a LOCAL bare repository (the
      §15.2 "Git is a reviewable representation" — the
      LAN-slice shape; the remote publication rides a later
      profile). Frontier → `.4.3.1`.

  - ID: `PHASE-6.4.3.1`
    Status: `done`
    Goal: the publication-store contract + the census — the
      decision record: the local bare repository, the ref
      scheme (the immutable `refs/rb/publications/<id>` +
      the effective `refs/rb/effective` channel), the
      staging-branch shape (the publication id + the digest),
      the write path's gix surface (the commit-tree + the
      reference update with the EXPECTED old id — the CAS).
      No code.
    Roadmap: §15.2, §15.7
    Done (`2026-09-07`): the store contract landed —
      `docs/decisions/2026-09-07_publication-store-
      contract.md`: the LOCAL bare repository (the §15.2
      LAN-slice shape; the remote-publication profile is a
      named deferral), the ref scheme (the staging
      `refs/rb/staging/<id>`, the IMMUTABLE
      `refs/rb/publications/<id>` — written once, never
      moved, the EFFECTIVE `refs/rb/effective` — the
      compare-and-swap ref with the EXPECTED old id), the
      write path's gix surface (the commit-tree + the
      `PreviousValue` reference update + the fetch-back
      digest re-derivation — no git CLI, the pure-Rust
      doctrine). No code changed. Frontier → `.4.3.2`.

  - ID: `PHASE-6.4.3.2`
    Status: `done`
    Goal: the Git publication half — the publisher module:
      the staging write (the bundle + the manifest into the
      staging branch), the fetch-back verification (the
      content digest re-derived), the immutable ref + the
      effective channel via the compare-and-swap (the
      expected-old-id reference updates), the §15.7 steps
      5–8's Git half.
    Roadmap: §15.7
    Done (`2026-09-07`): the Git publication half landed per
      ADR-020 + the store contract —
      `crates/reasonbraid-server/src/publisher.rs` (NEW):
      the `publish` (the gix plumbing — no CLI: the blobs +
      the FILENAME-SORTED tree + the raw-signature root
      commit, the staging branch via the `Any` previous, the
      FETCH-BACK verification (the re-derived digest), the
      IMMUTABLE ref via the `MustNotExist` (the
      written-once — the re-publish is the typed refusal),
      the EFFECTIVE channel via the
      `MustExistAndMatch`/`MustNotExist` compare-and-swap
      (the stale expectation is the typed CasMismatch));
      the api: `POST /v1/policy-publications/{id}/publish`
      (the declared repo path — the dev-trusted operator
      surface; the manifest composed from the records; the
      record marks effective with the ref ids); the
      `load` exposures on the publications + the
      projections. Measured (publisher 2 offline + policy
      8 live): the ref writes + the fetch-back equality, the
      second publication's CAS onto the first's effective,
      the stale-expectation refusal, the immutable
      written-once refusal; the live verb drives the Git
      half to the effective record with the two object ids,
      the terminal re-publish + the non-repository
      refusals. Frontier → `.4.3.3`.

  - ID: `PHASE-6.4.3.3`
    Status: `proposed`
    Goal: the reconciliation matrix + the kill-point tests —
      the six §15.8 rules over (the DB state, the Git
      state) → the action, the idempotent reconciler (the
      repeated runs converge), the kill-point tests (the
      staged/absent retry, the staged/conflicting stop, the
      failed/later-appearing quarantine — never a silent
      promote).
    Roadmap: §15.8

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
| 1 | `PHASE-6.4.3.3` | `proposed` | `.4.3.2` done — the Git publication half (the gix publisher: the sorted tree, the fetch-back, the written-once immutable, the CAS effective; publisher 2 + policy 8); the reconciliation matrix + the kill-point tests execute next |

## Changelog

- `2026-09-07`: `.4.3.2` done — the Git publication half
  (the gix publisher: the ref writes + the fetch-back + the
  written-once immutable + the CAS effective channel; the
  publish verb); publisher 2 + policy 8; frontier → `.4.3.3`.
- `2026-09-07`: `.4.3.1` done — the publication-store
  contract (the local bare repository, the staging/
  immutable/effective ref scheme, the CAS write path); no
  code; frontier → `.4.3.2`.
- `2026-09-07`: `.4.3` decomposed at the census seams — the
  write half is the greenfield (the R1 pack only acquires;
  the reconciler exists nowhere); children `.4.3.1` (the
  publication-store contract) → `.4.3.2` (the Git
  publication half) → `.4.3.3` (the reconciliation matrix +
  the kill-point tests); frontier → `.4.3.1`.
- `2026-09-07`: `.4.2` done — the publication records
  (migration 0042: the aggregate row, the chain-verified
  references, the staged/effective/failed machine); policy 7;
  frontier → `.4.3`.
- `2026-09-07`: `.4.1` done — ADR-020 accepted (the
  publication contract: the nine-step machine, the
  compare-and-swap idempotency, the six §15.8 reconciliation
  rules, the manifest-digest signatures); no code; frontier →
  `.4.2`.
- `2026-09-07`: `.4` decomposed at the census seams — the
  publication inputs + the substrate ship (the decisions/
  approvals, the byte-identical projections, the Git + the
  CA + the outbox); the publication records + the
  reconciliation are the greenfield; children `.4.1`
  (ADR-020 + the census) → `.4.2` (the records + the
  staging) → `.4.3` (the Git publication + the
  reconciliation); frontier → `.4.1`.
- `2026-09-07`: `.3.3` done — the Codex + the Claude
  projections (the AGENTS.md + the CLAUDE.md renderers, the
  size + escape declarations, the harness shapes); compiler
  8 + policy 6; **`.3` COMPLETE** — frontier → `.4`.
- `2026-09-07`: `.3.2` done — the compiler core (the
  hermetic `reasonbraid-policy-compiler` crate + the
  generic/lock renderers + migration 0041 + the projection
  verbs); compiler 5 + policy 5; frontier → `.3.3`.
- `2026-09-07`: `.3.1` done — ADR-033 accepted (the
  compiler contract: the byte-identical renderer, the
  declared unrepresentable, the hermetic crate, the
  projection digest); no code; frontier → `.3.2`.
- `2026-09-07`: `.3` decomposed at the census seams — the
  compiler is the greenfield (no projection exists; the `.1`
  resolution is its input); children `.3.1` (ADR-033 + the
  census) → `.3.2` (the compiler core) → `.3.3` (the Codex +
  Claude projections); frontier → `.3.1`.
- `2026-09-07`: `.2.3` done — the approval records (migration
  0040: the authority proof — the grant re-check at the
  approval boundary, the quorum snapshot, the separate row);
  policy 4; **`.2` COMPLETE** — frontier → `.3`.
- `2026-09-07`: `.2.2` done — the proposal + the decision
  records (migration 0039: the reference-shaped proposal,
  the frozen electorate snapshot, the verdict reference, the
  typed stage machine); policy 3; frontier → `.2.3`.
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


## Acceptance Checklist (PHASE-6.2.2)

The CODE change owned by this leaf:
`migrations/0039_policy_lifecycle.sql` (NEW — the proposal +
the decision tables), `crates/reasonbraid-server/src/
lifecycle.rs` (NEW — the shapes + the register_proposal +
the record_decision + the lists),
`crates/reasonbraid-server/src/lib.rs` (the module),
`crates/reasonbraid-server/src/api.rs` (the four verbs),
`crates/reasonbraid-server/tests/policy.rs` (the new test) —
`\.rs$` + `(^|/)migrations/`.

- [x] **REPRODUCE / ISSUE** — the pre-leaf surface: no
  proposal/decision row existed (the `.2` census — the
  lifecycle records were the greenfield).
- [x] **ROOT CAUSE (WHY + WHERE)** — `git grep -c
  "policy_proposals\|policy_decisions\|ProposalInput" 2f4d802
  -- crates/ migrations/` → rc=1 (nothing before this leaf).
  The fix point is the ADR-032 contract: the reference-shaped
  proposal + the frozen-snapshot decision + the stage
  machine.
- [x] **ADDRESSED (verified)** — measured before→after. Before:
  the grep above. After: `DATABASE_URL=… cargo test -p
  reasonbraid-server --test policy
  the_proposal_and_the_decision_stay_separate_records` →
  `test result: ok. 1 passed` (also inside the full live
  suite: `running 3 tests … ok`) — the proposal as a
  reference, the ghost policy + the ghost thread refusals,
  the decision with the frozen snapshot + the verdict, the
  stage advance, the second-decision refusal, the foreign-
  verdict + the empty-electorate refusals, the lists.
- [x] **NO REGRESSION** — `cargo test --all` → rc=0, 58 suites;
  `bash scripts/run_pg_tests.sh` → rc=0, 21 live suites + the
  demo `ALL acceptance checks passed`
  (`target/pg527_guard.log`);
  `cargo clippy --all --all-targets -- -D warnings` → rc=0;
  `cargo fmt --all -- --check` → rc=0; `make gate` → 13/13 at
  commit.
- [x] **FIX** — `0039_policy_lifecycle.sql`, `src/lifecycle.rs`,
  `src/lib.rs`, `src/api.rs`, `tests/policy.rs`.
- [x] **LOCKSTEP** — CHANGELOG, MEMORY, LIVE_STATUS, this tree's
  logs above, `docs/TASK_TREE.md` frontier — same commit (the
  KNOWLEDGE_MAP regen produced no diff; no new DEV_NOTES
  heading).


## Acceptance Checklist (PHASE-6.2.3)

The CODE change owned by this leaf:
`migrations/0040_policy_approvals.sql` (NEW — the approval
table), `crates/reasonbraid-server/src/lifecycle.rs` (the
`ApprovalInput`/`StoredApproval` shapes, the
`record_approval`, the list), `crates/reasonbraid-server/src/
api.rs` (the two verbs), `crates/reasonbraid-server/tests/
policy.rs` (the new test + the purge-gap fix) — `\.rs$` +
`(^|/)migrations/`.

- [x] **REPRODUCE / ISSUE** — the pre-leaf surface: no
  approval row existed (the `.2` census — the lifecycle
  stopped at the decision).
- [x] **ROOT CAUSE (WHY + WHERE)** — `git grep -c
  "policy_approvals\|ApprovalInput\|record_approval" 232047f
  -- crates/ migrations/` → rc=1 (nothing before this leaf).
  The fix point is the ADR-032 approval contract: the
  authority proof at the action time + the separate row.
- [x] **ADDRESSED (verified)** — measured before→after. Before:
  the grep above. After: `DATABASE_URL=… cargo test -p
  reasonbraid-server --test policy
  the_approval_carries_its_authority_proof` → `test result:
  ok. 1 passed` (also inside the full live suite: `running 4
  tests … ok`) — the full chain with the matching grant, the
  stage advance, the second-approval refusal, the
  mismatched-proof refusal, the foreign-decision refusal,
  the draft-stage refusal, the empty-quorum refusal, the
  list. The first live pass caught the suite's purge gap
  (the lifecycle tables leaked across the tests) + the
  shared WrongStage message shape — fixed.
- [x] **NO REGRESSION** — `cargo test --all` → rc=0, 58 suites;
  `bash scripts/run_pg_tests.sh` → rc=0, 21 live suites + the
  demo `ALL acceptance checks passed`
  (`target/pg528_guard.log`);
  `cargo clippy --all --all-targets -- -D warnings` → rc=0;
  `cargo fmt --all -- --check` → rc=0; `make gate` → 13/13 at
  commit.
- [x] **FIX** — `0040_policy_approvals.sql`, `src/lifecycle.rs`,
  `src/api.rs`, `tests/policy.rs`.
- [x] **LOCKSTEP** — CHANGELOG, MEMORY, LIVE_STATUS, this tree's
  logs above, `docs/TASK_TREE.md` frontier — same commit (the
  KNOWLEDGE_MAP regen produced no diff; no new DEV_NOTES
  heading).


## Acceptance Checklist (PHASE-6.3.2)

The CODE change owned by this leaf:
`crates/reasonbraid-policy-compiler/` (NEW — the hermetic
crate: the Cargo.toml, the lib.rs shapes + the `compile`, the
tests/compiler.rs suite), `migrations/0041_policy_projections.sql`
(NEW), `crates/reasonbraid-server/src/projections.rs` (NEW —
the `project` + the list), `crates/reasonbraid-server/src/
lib.rs` + `Cargo.toml` (the module + the dependency),
`crates/reasonbraid-server/src/api.rs` (the two verbs),
`crates/reasonbraid-server/tests/policy.rs` (the new test) —
`\.rs$` + `\.toml$` + `(^|/)migrations/`.

- [x] **REPRODUCE / ISSUE** — the pre-leaf surface: no
  projection, no compiler, no target vocabulary, no
  unrepresentable declaration (the `.3` census).
- [x] **ROOT CAUSE (WHY + WHERE)** — `git grep -c
  "policy_projections\|reasonbraid-policy-compiler\|ProjectionRequest"
  cbde3b2 -- crates/ migrations/` → rc=1 (nothing before
  this leaf). The fix point is the ADR-033 core: the pure
  renderer + the projection record.
- [x] **ADDRESSED (verified)** — measured before→after. Before:
  the grep above. After: `cargo test -p
  reasonbraid-policy-compiler` → `test result: ok. 5
  passed` — the byte-identical repeat, the shuffled stable
  order, the newline escaping + the control-char
  declaration, the lock rows, the unknown-target refusal;
  `DATABASE_URL=… cargo test -p reasonbraid-server --test
  policy the_projection_compiles_the_resolved_set_byte_identical`
  → `test result: ok. 1 passed` (also inside the full live
  suite: `running 5 tests … ok`) — the generic projection
  with the declared unrepresentable, the repeat's identical
  digest, the lock projection, the unknown-target + the
  duplicate refusals, the list.
- [x] **NO REGRESSION** — `cargo test --all` → rc=0, 61 suites
  (the compiler crate added its two);
  `bash scripts/run_pg_tests.sh` → rc=0, 21 live suites + the
  demo `ALL acceptance checks passed`
  (`target/pg529_guard.log`);
  `cargo clippy --all --all-targets -- -D warnings` → rc=0;
  `cargo fmt --all -- --check` → rc=0; `make gate` → 13/13 at
  commit.
- [x] **FIX** — `crates/reasonbraid-policy-compiler/`,
  `0041_policy_projections.sql`, `src/projections.rs`,
  `src/lib.rs`, `src/api.rs`, `tests/policy.rs`,
  `crates/reasonbraid-server/Cargo.toml`.
- [x] **LOCKSTEP** — CHANGELOG, MEMORY, LIVE_STATUS, this tree's
  logs above, `docs/TASK_TREE.md` frontier — same commit (the
  KNOWLEDGE_MAP regen produced no diff; no new DEV_NOTES
  heading).


## Acceptance Checklist (PHASE-6.3.3)

The CODE change owned by this leaf:
`crates/reasonbraid-policy-compiler/src/lib.rs` (the `codex` +
`claude` targets + the `render_codex`/`render_claude`
renderers + the backtick escape + the `STATEMENT_LIMIT`),
`crates/reasonbraid-policy-compiler/tests/compiler.rs` (the
three new tests), `crates/reasonbraid-server/tests/policy.rs`
(the new live test) — `\.rs$`.

- [x] **REPRODUCE / ISSUE** — the pre-leaf surface: the
  compiler rendered only the generic + the lock targets (the
  `.3.2` core); the Codex/Claude harness shapes + the size
  limit + the backtick escape were absent.
- [x] **ROOT CAUSE (WHY + WHERE)** — `git grep -c
  "render_codex\|render_claude\|STATEMENT_LIMIT" 4ccb1c8 --
  crates/reasonbraid-policy-compiler/` → rc=1 (nothing
  before this leaf). The fix point is the ADR-033 target
  vocabulary: the two harness renderers over the same core.
- [x] **ADDRESSED (verified)** — measured before→after. Before:
  the grep above. After: `cargo test -p
  reasonbraid-policy-compiler` → `test result: ok. 8
  passed` — the codex backticked ids + the claude plain ids,
  the per-target byte-identical repeat, the oversized-
  statement declaration (never a truncation), the backtick
  escape; `DATABASE_URL=… cargo test -p reasonbraid-server
  --test policy the_codex_and_claude_projections_ride_the_verb`
  → `test result: ok. 1 passed` (also inside the full live
  suite: `running 6 tests … ok`) — the two targets through
  the projection verb with their harness shapes + the
  digests. The first passes caught the escape-assertion
  layering (the backtick literal) + the filter-remnant
  syntax — fixed.
- [x] **NO REGRESSION** — `cargo test --all` → rc=0, 61 suites;
  `bash scripts/run_pg_tests.sh` → rc=0, 21 live suites + the
  demo `ALL acceptance checks passed`
  (`target/pg530_guard.log`);
  `cargo clippy --all --all-targets -- -D warnings` → rc=0;
  `cargo fmt --all -- --check` → rc=0; `make gate` → 13/13 at
  commit.
- [x] **FIX** — `crates/reasonbraid-policy-compiler/src/lib.rs`,
  `crates/reasonbraid-policy-compiler/tests/compiler.rs`,
  `crates/reasonbraid-server/tests/policy.rs`.
- [x] **LOCKSTEP** — CHANGELOG, MEMORY, LIVE_STATUS, this tree's
  logs above, `docs/TASK_TREE.md` frontier — same commit (the
  KNOWLEDGE_MAP regen produced no diff; no new DEV_NOTES
  heading).


## Acceptance Checklist (PHASE-6.4.2)

The CODE change owned by this leaf:
`migrations/0042_policy_publications.sql` (NEW — the
publication table), `crates/reasonbraid-server/src/
publications.rs` (NEW — the shapes + the stage +
mark_effective + mark_failed + the list),
`crates/reasonbraid-server/src/lib.rs` (the module),
`crates/reasonbraid-server/src/api.rs` (the four verbs),
`crates/reasonbraid-server/tests/policy.rs` (the new test) —
`\.rs$` + `(^|/)migrations/`.

- [x] **REPRODUCE / ISSUE** — the pre-leaf surface: no
  publication row existed (the `.4` census — the chain
  stopped at the approval; the Git truth had no record).
- [x] **ROOT CAUSE (WHY + WHERE)** — `git grep -c
  "policy_publications\|PublicationInput\|mark_effective"
  177bb19 -- crates/ migrations/` → rc=1 (nothing before
  this leaf). The fix point is the ADR-020 record half: the
  aggregate row + the typed state machine.
- [x] **ADDRESSED (verified)** — measured before→after. Before:
  the grep above. After: `DATABASE_URL=… cargo test -p
  reasonbraid-server --test policy
  the_publication_stages_and_marks_its_typed_state` →
  `test result: ok. 1 passed` (also inside the full live
  suite: `running 7 tests … ok`) — the full chain to the
  staged publication, the effective transition with the
  object ids, the bad-digest + the ghost-projection + the
  ghost-decision refusals, the terminal re-transition
  refusal, the second chain's typed FAILURE with the reason,
  the newest-first list.
- [x] **NO REGRESSION** — `cargo test --all` → rc=0, 61 suites;
  `bash scripts/run_pg_tests.sh` → rc=0, 21 live suites + the
  demo `ALL acceptance checks passed`
  (`target/pg531_guard.log`);
  `cargo clippy --all --all-targets -- -D warnings` → rc=0;
  `cargo fmt --all -- --check` → rc=0; `make gate` → 13/13 at
  commit.
- [x] **FIX** — `0042_policy_publications.sql`,
  `src/publications.rs`, `src/lib.rs`, `src/api.rs`,
  `tests/policy.rs`.
- [x] **LOCKSTEP** — CHANGELOG, MEMORY, LIVE_STATUS, this tree's
  logs above, `docs/TASK_TREE.md` frontier — same commit (the
  KNOWLEDGE_MAP regen produced no diff; no new DEV_NOTES
  heading).


## Acceptance Checklist (PHASE-6.4.3.2)

The CODE change owned by this leaf:
`crates/reasonbraid-server/src/publisher.rs` (NEW — the
`publish` over the gix plumbing), `crates/reasonbraid-server/
src/publications.rs` + `src/projections.rs` (the `load`
exposures), `crates/reasonbraid-server/src/lib.rs` (the
module), `crates/reasonbraid-server/src/api.rs` (the publish
verb), `crates/reasonbraid-server/tests/publisher.rs` (NEW —
the offline suite), `crates/reasonbraid-server/tests/
policy.rs` (the live test) — `\.rs$`.

- [x] **REPRODUCE / ISSUE** — the pre-leaf surface: the
  write half was the greenfield (the R1 pack only acquires;
  no commit/ref-write path — `git grep -c
  "pub fn publish\|PublishedRefs\|refs/rb/effective" 7f572cd
  -- crates/reasonbraid-server/src/publisher.rs
  crates/reasonbraid-server/tests/publisher.rs` → rc=1).
- [x] **ROOT CAUSE (WHY + WHERE)** — the fix point is the
  ADR-020 + the store contract's Git half: the staging
  branch, the fetch-back, the immutable ref, the CAS
  effective channel — over the gix plumbing.
- [x] **ADDRESSED (verified)** — measured before→after. Before:
  the grep above. After: `cargo test -p reasonbraid-server
  --test publisher` → `test result: ok. 2 passed` — the ref
  writes + the fetch-back equality, the CAS onto the first
  effective, the stale-expectation refusal, the immutable
  written-once refusal; `DATABASE_URL=… cargo test -p
  reasonbraid-server --test policy the_publish_verb_drives_the_git_half`
  → `test result: ok. 1 passed` (also inside the full live
  suite: `running 8 tests … ok`) — the publish verb → the
  effective record with the two object ids, the terminal
  re-publish + the non-repository refusals. The first
  passes caught the gix API shapes (the `RefEdit`/`Target`
  form, the filename-sorted tree requirement, the error
  message detection) — fixed.
- [x] **NO REGRESSION** — `cargo test --all` → rc=0, 62 suites
  (the publisher suite added one);
  `bash scripts/run_pg_tests.sh` → rc=0, 21 live suites + the
  demo `ALL acceptance checks passed`
  (`target/pg532_guard.log`);
  `cargo clippy --all --all-targets -- -D warnings` → rc=0;
  `cargo fmt --all -- --check` → rc=0; `make gate` → 13/13 at
  commit.
- [x] **FIX** — `src/publisher.rs`, `src/publications.rs`,
  `src/projections.rs`, `src/lib.rs`, `src/api.rs`,
  `tests/publisher.rs`, `tests/policy.rs`.
- [x] **LOCKSTEP** — CHANGELOG, MEMORY, LIVE_STATUS, this tree's
  logs above, `docs/TASK_TREE.md` frontier — same commit (the
  KNOWLEDGE_MAP regen produced no diff; no new DEV_NOTES
  heading).
