-- 0092_incarnations_node_id.sql — SIGNOFF-REPAIR.11.4.7.2.1.5.1 (ROADMAP §8.1):
-- which NODE INSTANCE an incarnation ran on, recorded as a fact.
--
-- §8.1 binds an AgentRole to a NodeInstance only through its incarnation — the
-- incarnation *states the actual provider/model/harness/configuration*, and the
-- harness is an installation on a node. No direct node→role registry is in the
-- model. The dev profile collapses the two (`NODE-BINDING-001`: a node id IS the
-- agent role wire id it serves), and the book declares it as a known limit.
--
-- This column records the binding where §8.1 puts it, at the one writer — the
-- node enrolment, which has the node id in hand — so the fact exists in the
-- ledger rather than only in a rule. A registry that DECLARES bindings (a role
-- moving between nodes without re-enrolment, one node serving two roles) is
-- deferred on a readable trigger: an incarnation whose node is not its role's id,
--   SELECT count(*) FROM incarnations WHERE node_id <> role_id;
-- which is 0 under the dev rule and can only become non-zero once a directory
-- replaces it (`docs/decisions/2026-09-23_the-node-to-role-binding-is-a-ledger-fact-not-a-registry-before-g9.md`).
--
-- The backfill is the dev rule's own truth: every existing incarnation was
-- declared by the node whose id is its role's.

ALTER TABLE incarnations ADD COLUMN node_id TEXT;
UPDATE incarnations SET node_id = role_id WHERE node_id IS NULL;
