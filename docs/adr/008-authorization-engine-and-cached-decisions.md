# ADR-008 — Authorization engine and cached-decision semantics

- **Status:** `accepted` (evidence-gated — the `.1.5.1` spike prototyped the
  cache semantics as pure, tested functions)
- **Date:** `2026-09-07`
- **Leaf:** `PHASE-2.1.5.1`
- **Requirements:** `ROADMAP.md` §23 queue item 008; §16.4 (authorization
  architecture — the cached-decision + fail-open/fail-closed sentences);
  §11.1 (node responsibilities — "caches only the minimum authorized
  directory/policy state"); §17.1 (the store-authority table — the node
  journal is explicitly NOT authoritative for "global grants or final
  decisions")

## Context

Two open halves: **the engine** — §16.4 names OPA/Rego/Cedar or a small
purpose-built evaluator as candidates — and **the cache** — §16.4 fixes the
rule ("Nodes may cache only explicitly cacheable decisions and must honor
expiry and revocation freshness requirements. When the authority service is
unavailable, each action class has a declared fail-open or fail-closed rule;
publication, secret access, grant changes, and irreversible writes fail
closed") without naming the mechanism. The dev profile's reality: the server
makes every decision in-transaction at admission (the Phase-0 `.5.1`
purpose-built evaluator), and the node dispatches delivered commands at an
irreversible boundary (the provider contact) possibly seconds after admission
— a revocation between admission and dispatch must refuse at the boundary.

## Options

1. **Engine: adopt an external policy engine** (OPA/Cedar) — a new runtime
   dependency, a second decision language, and a migration of the shipped
   evaluator's grants/boundaries/risk-ceiling semantics.
2. **Engine: keep the shipped purpose-built in-tx evaluator** and record it
   accepted-with-evidence (the ADR-006 precedent) — the candidate comparison
   parks behind a measured trigger.
3. **Cache: none** — fresh evaluation everywhere (today's state; the boundary
   re-ask is a full server round trip per dispatch).
4. **Cache: the admission decision rides the delivery** — the server attaches
   the decision metadata (authz_ref + policy_digest + decided_at + the epoch
   at decision time) to each delivered command; the node caches ONLY those
   server-made decisions and honors expiry + epoch at the dispatch boundary.

## Evidence

- **There is no cache machinery today**: `grep -rn 'cache'
  crates/reasonbraid-node/src/ crates/reasonbraid-server/src/` → 0 matches —
  while the plumbing is pre-shaped: the node journal's `authz_ref` column
  exists with every writer binding `None` (journal.rs:144/437/444; writers
  at worker.rs:149 + node.rs:208), and the authorization record already
  holds `policy_digest`/`policy_version`/`decided_at` (migration 0004).
- **The store-authority table (§17.1) decides option 4's shape**: the node
  SQLite journal is explicitly NOT authoritative for "global grants or final
  decisions" — the node can never locally RE-EVALUATE a grant; it can only
  cache a decision the server already made. An engine swap (option 1) would
  not change that.
- **The spike's semantics are pure, tested functions**: `CachedDecision` +
  `CacheVerdict` + the `ActionClass` fail table in the core crate —
  `cargo test -p reasonbraid-core` → `test result: ok. 44 passed` (the five
  new tests: a fresh, epoch-current cached allow dispatches; an expired one
  is stale; an epoch bump invalidates a fresh entry; a cached deny is never
  widened by time; irreversible/admin writes fail closed, reads fail open).
- **The shipped evaluator already covers the dev profile**: typed grants
  inside typed boundaries, risk ceilings, the deterministic policy digest +
  version, and the audited decision record — the §16.4 comparison criteria
  (expressiveness, embeddability, explanation, versioning, latency, partial
  evaluation) have no dev-profile need the shipped engine fails. A
  re-platform is a rewrite with no measured trigger.

## Choice

Option 2 + option 4. The engine stays the shipped purpose-built in-tx
evaluator (accepted with evidence). The cache is the admission decision
riding the delivery: the node caches ONLY the server-made decisions for its
own delivered commands (the §11.1 minimum state), honors the `.1.5.1` rules —
a 60-second freshness TTL from `decided_at`; a per-tenant revocation epoch
bumped by every revocation write, with a cached entry recording the epoch it
was decided under (a bump invalidates it, however fresh); a cached deny is
never widened by time; and the §16.4 fail table (irreversible + administrative
writes fail closed, local-journal reads fail open) governs the unreachable-
store case.

## Consequences

- `.1.5.2` implements: the delivery-carried decision metadata, the tenant
  revocation epoch (migration 0013) bumped in the same transaction as every
  `.1.3` revocation write, the journal's cached-decision storage (the
  pre-shaped `authz_ref` finally gains a value), and the dispatch-boundary
  evaluation.
- Honest limits: the dev profile's invalidation is **poll-bounded** — a
  revocation refuses at the dispatch boundary once the node has SEEN the
  bumped epoch (worst case one poll interval), not certificate-grade push.
  The TTL bounds staleness to 60s + one poll; the epoch bounds correctness to
  one poll.

## Rollback / revisit trigger

- The OPA/Cedar comparison reopens when policy authoring needs a language the
  shipped engine cannot express (multi-operator authored policy, a measured
  expressiveness gap) — with the §16.4 comparison criteria as the scoring
  sheet.
- Push-based invalidation (the server drives the bump, not the poll) reopens
  at Internet qualification (G6), where poll-bounded freshness is no longer
  the honest profile.
