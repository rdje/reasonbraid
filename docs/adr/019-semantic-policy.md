# ADR-019 — The semantic policy: a versioned digest-pinned document with the authority in the grants, never the label — the resolution is seven steps and fails closed

- **Status:** `accepted` (evidence-gated — the §15.1–15.3 contract:
  the `PolicyVersion`/`PolicySetVersion` shapes, the ownership
  binding, and the seven-step resolution are the shapes the
  `.1.2`/`.1.3` leaves implement)
- **Date:** `2026-09-07`
- **Leaf:** `PHASE-6.1.1`
- **Requirements:** `ROADMAP.md` §23 queue item 019 (the canonical
  policy schema and the merge/precedence rules); §15.1 (the
  canonical semantic model), §15.2 (the repository structure),
  §15.3 (the layering and the precedence)

## Context

The `.1` census mapped §15.1–15.3 against the shipped surface.
The policy schema is a GREENFIELD: no `PolicyVersion` exists
(`git grep -c "PolicyVersion" HEAD -- crates/ migrations/` →
rc=1), no layer/precedence rules, no impact maps, no
policy-ownership binding. The block is LIFTED: the dependencies
the frontier row named all ship — the authority (Phase 2's
boundaries/grants/audit), the deliberation (Phase 5, closed),
the Git/object consistency (Phase 4's git pack + the snapshots),
the correction model (the revision lineage + the
supersession/retraction vocabulary). The reusable pieces: the
versioned-config precedent (the Phase-5.1 profile registry —
the versioned entry + the digest pinning), the content-addressing
(ADR-011), the authority model (the grant checks — the §15.3
step-1 substrate), the evidence pipeline (Phase 4), the
deliberation records (Phase 5).

## Decision

- **The policy is a versioned, digest-pinned document, never
  prose.** The `PolicyVersion` is a typed record: the stable
  policy id, the semantic version, the ADR-011 digest over the
  canonical document bytes, the lifecycle status, the
  title/intent/rationale/domain/risk class, the owning
  authority (a GRANT reference — see below), the normative
  statements with STABLE clause ids, the applicability
  selectors + the explicit non-applicability, the
  dependencies/conflicts/precedence hints, the exception
  schema, the evidence/decision provenance (the Phase-4/5
  references), the verification requirements, the
  review-by/outcome triggers, the projection requirements +
  the minimum compiler version, the migration/suspension/
  rollback/deprecation guidance. The same registry pattern as
  ADR-016: the unknown version is the typed refusal, the
  digest pins the measurement.
- **The ownership is the authority binding — the label grants
  nothing.** The policy's owning authority is a reference to
  the Phase-2 authority model (the boundary + the grant +
  the charter digest); §15.3's rule ("security/administrative
  policy is not automatically superior by label") is
  STRUCTURAL: the resolution's authority step consults the
  grants, never the layer's name. A policy whose owning
  authority does not resolve is invalid at registration time.
- **The `PolicySetVersion` is a lock manifest.** The set
  resolves a compatible, authorized collection for a target;
  the manifest records every version, digest, dependency,
  charter/grant basis, compiler, and projection — the
  publication truth (§15.2: the signed manifests/content
  digests identify immutable publication truth).
- **The resolution is seven deterministic steps, fail-closed.**
  Per §15.3: (1) verify the issuer authority for the
  target/domain/action; (2) filter by the applicability +
  the effective interval; (3) resolve the dependencies and
  the explicit conflicts; (4) apply the charter-defined
  precedence/specificity; (5) evaluate the valid exceptions
  and waivers; (6) FAIL CLOSED on an unresolved binding
  conflict; (7) produce the explanation tree. The explanation
  tree is part of the RESULT (the resolved clause names its
  path — the audit surface), and step 6 is never a silent
  pick.
- **The impact maps are derivable coverage, never claims.**
  The clause → target/domain/action coverage is computed from
  the applicability selectors + the resolution records; the
  map states what the clauses cover, never what they
  achieve.

## Consequences

- `.1.2` implements the typed `PolicyVersion` + the validation
  + the registry + the ownership metadata and `.1.3` the
  seven-step resolution + the exceptions + the explanation
  tree + the impact maps — each against this contract
  verbatim; a deviation is a contract change.
- The `.2` lifecycle leaf (the proposal/review/approval
  records) consumes the ownership binding: the approval's
  authority proof is the grant the policy already references.
- The `.4` publication leaf's signed manifests are the
  `PolicySetVersion` lock manifests the `.1.2` registry
  stores.

answers:

- **The digest makes the policy a document, not a mood.** The
  policy's normative force is its versioned content-addressed
  record; the prose fields (the rationale, the intent) are
  metadata the clauses reference — the same separation the
  thread's event log makes between the content and the
  projection.
- **Authority is a grant fact, not a layer name.** The
  §15.3 label trap (the "security policy" that overrides
  everything) is defused structurally: the resolution asks
  the grants, and a label-only policy fails the ownership
  check at registration.
- **Fail-closed is the resolution's honesty.** A conflict
  the rules cannot settle must surface as a refusal with the
  explanation tree — a silent precedence pick would be the
  same dishonesty as a decided close carrying unresolved
  items (`.2.4.1`'s family rule).
