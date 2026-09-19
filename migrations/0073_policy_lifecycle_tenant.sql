-- 0073_policy_lifecycle_tenant.sql — SIGNOFF-REPAIR.6.1.5.2: the policy
-- lifecycle STORES the tenant its write already derives.
--
-- `docs/decisions/2026-09-19_the-policy-library-is-shared-the-lifecycle-is-its-tenants.md`
-- decided the question `.6.1.5` was opened over, and it had two subjects.
-- `policy_versions` — the LIBRARY — is site-wide BY DESIGN and gets NO column
-- here: its ownership model is a grant, and a policy only one tenant can read is
-- not governance. The nine LIFECYCLE tables below are tenant-owned BY OMISSION:
-- `register_proposal` takes a `tenant_id`, refuses a thread outside it, and then
-- stores nothing at all.
--
-- ⭐ WHICH IS WHY THIS IS NOT THE TRAP `.6.1.5` FORBIDS. That leaf forbids a
-- tenant-scoped READ over an unscoped write, because it hides rows from their
-- own author. These writes are not unscoped — storing the tenant RECORDS a
-- binding the write already performs and throws away. ⛔ The reads are NOT bound
-- here; `.6.1.5.3` owns all 32 of them, and the split exists so that no row can
-- be hidden before every write records an owner.
--
-- ⛔ THE TENANT OF A NEW ROW IS DERIVED, NEVER ACCEPTED ON THE WIRE
-- (`docs/decisions/2026-09-18_node-presence-is-read-by-its-own-tenant.md`), and
-- it is derived from TWO different places because the rows are two kinds of
-- thing:
--
--   * `policy_proposals`, `policy_decisions`, `policy_approvals` take the
--     CALLER's tenant — the write has already proved the caller owns the thread,
--     the verdict event, or the proposal's thread (`.6.1.5.1`, `.6.1.5.1.1`).
--   * `policy_publications`, `policy_drift`, `policy_corrections`,
--     `policy_outcomes`, `policy_reviews` take their PARENT's tenant. A drift
--     observation about Alice's publication carrying Mallory's tenant would be
--     invisible to Alice while she is the only party it concerns — `.6.1.5`'s
--     trap, re-entered from the write side.
--   * `policy_projections` takes the caller's by AUTHORSHIP. It has no ancestor
--     at all (see below).
--
-- THE ROWS ALREADY STORED, decided per table as `.7.1.2.2`'s were, because the
-- tables record different things:
--
--   * EIGHT of the nine are derivable by lineage, transitively, to
--     `aggregate_state` — the proposal names a thread, and everything else names
--     a proposal or a publication. The backfill below is a derivation, never a
--     guess.
--   * 🔴 `policy_projections` is derivable by NOTHING. `ProjectionRequest` is
--     `{projection_id, target, resolution, lock}` and `ResolutionRequest` is
--     `{policies, target, exception_grants}`: no thread, no proposal, no tenant
--     anywhere on the path. A projection is a compiled artifact of the shared
--     library, and its tenant is its AUTHOR — a fact the old schema never
--     recorded. Its historical rows keep `tenant_id IS NULL` and are read by
--     NOBODY, exactly as `routing_recommendations` does in `0072`.
--
-- ⚠️ A proposal whose `thread_id` resolves to no thread, or to a thread id
-- present under two tenants, also stays NULL, and so does everything descended
-- from it. Left NULL deliberately: the read predicate is `tenant_id = $1`, which
-- never matches NULL, so an unattributable row is invisible rather than
-- misattributed. Inventing an owner for a governance record is worse than losing
-- its visibility.

ALTER TABLE policy_proposals   ADD COLUMN tenant_id TEXT REFERENCES tenants (tenant_id);
ALTER TABLE policy_decisions   ADD COLUMN tenant_id TEXT REFERENCES tenants (tenant_id);
ALTER TABLE policy_approvals   ADD COLUMN tenant_id TEXT REFERENCES tenants (tenant_id);
ALTER TABLE policy_projections ADD COLUMN tenant_id TEXT REFERENCES tenants (tenant_id);
ALTER TABLE policy_publications ADD COLUMN tenant_id TEXT REFERENCES tenants (tenant_id);
ALTER TABLE policy_drift       ADD COLUMN tenant_id TEXT REFERENCES tenants (tenant_id);
ALTER TABLE policy_corrections ADD COLUMN tenant_id TEXT REFERENCES tenants (tenant_id);
ALTER TABLE policy_outcomes    ADD COLUMN tenant_id TEXT REFERENCES tenants (tenant_id);
ALTER TABLE policy_reviews     ADD COLUMN tenant_id TEXT REFERENCES tenants (tenant_id);

-- Step 1 — the root of every lineage: the proposal's own thread.
--
-- ⛔ `aggregate_state` is keyed `(tenant_id, aggregate_id)`, so an aggregate id
-- is unique per tenant rather than globally. A thread id present under two
-- tenants therefore derives NOTHING: the `HAVING count(*) = 1` is what makes
-- this a derivation instead of a coin toss.
UPDATE policy_proposals p
   SET tenant_id = t.tenant_id
  FROM (
        SELECT aggregate_id, min(tenant_id) AS tenant_id
          FROM aggregate_state
         WHERE aggregate_type = 'thread'
         GROUP BY aggregate_id
        HAVING count(*) = 1
       ) t
 WHERE p.tenant_id IS NULL
   AND p.thread_id = t.aggregate_id;

-- Step 2 — the records of a proposal: the decision, the approval, the
-- publication. Each names its proposal, and an orphan of an unattributed
-- proposal stays NULL because the join requires a tenant to copy.
UPDATE policy_decisions d
   SET tenant_id = p.tenant_id
  FROM policy_proposals p
 WHERE d.tenant_id IS NULL
   AND d.proposal_id = p.proposal_id
   AND p.tenant_id IS NOT NULL;

UPDATE policy_approvals a
   SET tenant_id = p.tenant_id
  FROM policy_proposals p
 WHERE a.tenant_id IS NULL
   AND a.proposal_id = p.proposal_id
   AND p.tenant_id IS NOT NULL;

UPDATE policy_publications u
   SET tenant_id = p.tenant_id
  FROM policy_proposals p
 WHERE u.tenant_id IS NULL
   AND u.proposal_id = p.proposal_id
   AND p.tenant_id IS NOT NULL;

-- Step 3 — the records of a publication: the drift, the correction, the
-- outcome, the scheduled review. These run AFTER step 2, because the
-- publication they read is itself only attributed there.
UPDATE policy_drift d
   SET tenant_id = u.tenant_id
  FROM policy_publications u
 WHERE d.tenant_id IS NULL
   AND d.publication_id = u.publication_id
   AND u.tenant_id IS NOT NULL;

UPDATE policy_corrections c
   SET tenant_id = u.tenant_id
  FROM policy_publications u
 WHERE c.tenant_id IS NULL
   AND c.publication_id = u.publication_id
   AND u.tenant_id IS NOT NULL;

UPDATE policy_outcomes o
   SET tenant_id = u.tenant_id
  FROM policy_publications u
 WHERE o.tenant_id IS NULL
   AND o.publication_id = u.publication_id
   AND u.tenant_id IS NOT NULL;

UPDATE policy_reviews r
   SET tenant_id = u.tenant_id
  FROM policy_publications u
 WHERE r.tenant_id IS NULL
   AND r.publication_id = u.publication_id
   AND u.tenant_id IS NOT NULL;

-- ⛔ `policy_projections` gets NO backfill statement, and its absence is the
-- decision rather than an omission: nothing in the schema can attribute those
-- rows to an author.

-- The indexes match the read shape `.6.1.5.3` will bind: every list verb over
-- these tables orders `created_at DESC`.
CREATE INDEX policy_proposals_tenant_idx    ON policy_proposals    (tenant_id, created_at DESC);
CREATE INDEX policy_decisions_tenant_idx    ON policy_decisions    (tenant_id, created_at DESC);
CREATE INDEX policy_approvals_tenant_idx    ON policy_approvals    (tenant_id, created_at DESC);
CREATE INDEX policy_projections_tenant_idx  ON policy_projections  (tenant_id, created_at DESC);
CREATE INDEX policy_publications_tenant_idx ON policy_publications (tenant_id, created_at DESC);
CREATE INDEX policy_drift_tenant_idx        ON policy_drift        (tenant_id, created_at DESC);
CREATE INDEX policy_corrections_tenant_idx  ON policy_corrections  (tenant_id, created_at DESC);
CREATE INDEX policy_outcomes_tenant_idx     ON policy_outcomes     (tenant_id, created_at DESC);
CREATE INDEX policy_reviews_tenant_idx      ON policy_reviews      (tenant_id, created_at DESC);
