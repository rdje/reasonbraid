# ADR-031 — The routing policy: deterministic rules with the human authority outranking them — the learned routing is a shadow recommendation, never a raise

- **Status:** `accepted` (evidence-gated — the §13.8 contract:
  the case-class vocabulary, the rule-based policy, and the
  shadow recommendation are the shapes the `.5.2`/`.5.3` leaves
  implement)
- **Date:** `2026-09-07`
- **Leaf:** `PHASE-5.5.1`
- **Requirements:** `ROADMAP.md` §13.8 (routing policy)

## Context

The `.5` census mapped §13.8 against the shipped surface. The
routing decision today is the CLIENT's choice: the thread create
carries the explicit `workflow_profile` (the `.1` lane's
validated reference), and the bare thread defaults to the
hardcoded `quick_advice` — no rule, no case-class vocabulary, no
policy exists (`git grep -n "routing" HEAD --
crates/reasonbraid-server/src/` finds only the HTTP router and
the node channel's route comments; the `Classification` enum is
the SECURITY classification, not a routing class). The reusable
pieces: the §13.1 built-in profiles map §13.8's left side
(`quick_advice` = low-risk/simple, `evidence_review` =
factual/current, `independent_panel` = uncertain/high-value,
`critique`/`architecture_decision` = design/policy,
`policy_proposal` = binding/high-impact), the `.4` evaluation
service's trials and gates (the evidence a learned policy would
consume), and the create boundary's validated-profile
resolution.

## Decision

- **The case class is a submitted input, never a derived
  judgment.** The routing case classes are the §13.8 rows:
  `simple`, `factual`, `uncertain`, `design_policy`,
  `governed`, `correlated`, `diminishing`. The client (or the
  calling agent) submits the class with the create; the service
  never classifies on its own (a derived class would be an
  untested judgment — the class is honest data, the routing is
  the policy's arithmetic).
- **The rule-based policy is a deterministic table.** The
  policy maps each class to ONE arm (a registered workflow
  profile): the §13.8 rows as the built-in rules (`simple` →
  `quick_advice`, `factual` → `evidence_review`, `uncertain` →
  `independent_panel`, `design_policy` → `critique`, `governed`
  → `policy_proposal`, `correlated` → `independent_panel`,
  `diminishing` → `quick_advice`). The resolution is a lookup —
  the same class always resolves to the same arm; the result
  names the rule id (the audit surface).
- **The human authority outranks the rule.** The policy applies
  at the create boundary ONLY when the client names no explicit
  profile (the class is submitted); the explicit profile always
  wins — a rule can never override a named choice (the policy
  recommends, the caller decides).
- **The learned routing is a SHADOW RECOMMENDATION, never a
  raise.** A learned-routing recommendation maps a class to an
  arm drawn from the EXISTING registered profiles — it cannot
  raise authority, spend, data access, or side-effect scope
  (§13.8's constraint): the recommendation is recorded with its
  evidence reference (the `.4` trial/gate results it rests on),
  it is NEVER applied by the service, and an arm outside the
  registered set is the typed refusal. The `.5` lane ships the
  recommendation surface; flipping it to a policy is a FUTURE
  lane's decision with its own gate.
- **Every selection is recorded.** The policy's resolution and
  the recommendation ride their own records (the rule id + the
  class + the arm + the evidence reference) — the routing
  decision is auditable, never a silent default.

## Consequences

- `.5.2` implements the rule-based policy (the built-in rules,
  the deterministic resolution, the create-boundary
  application) and `.5.3` the shadow recommendation surface —
  each against this contract verbatim; a deviation is a
  contract change.
- The create's explicit-profile path is untouched: the policy
  adds the default for the un-named case, so every existing
  thread's routing stays what its caller chose.
- The `.6` G5 exit consumes the routing records: the "the
  deliberation improves answers" claim rests on the policy's
  per-class evidence, not on the recommendation's.

answers:

- **A routing rule is arithmetic over honest inputs.** The
  class is submitted data and the table is deterministic — the
  policy adds no intelligence and therefore no untested
  judgment; the learned half (where the judgment lives) is
  confined to the shadow surface.
- **The explicit profile is the caller's authority.** Making
  the rule subordinate to the named choice keeps the policy
  from ever silently overriding a human decision — the same
  argument as ADR-016's "the unknown profile is a refusal, not
  a default", inverted: here the KNOWN choice is the
  authority.
- **Shadow first is the constraint's mechanism.** A learned
  recommendation that cannot raise scope can only re-order
  what already exists — recorded, referenced, never applied,
  it is evidence the future policy gate can weigh.
