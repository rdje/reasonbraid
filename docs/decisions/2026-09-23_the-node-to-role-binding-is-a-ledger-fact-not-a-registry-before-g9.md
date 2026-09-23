---
answers:
  - Does ROADMAP §8 or §10 owe a node→role binding registry before the stable release?
  - Where does the roadmap actually bind a role to a machine, and does the code record it?
  - Which sites depend on the dev rule that a node id is the role id, and what replaces it?
---
# The node-to-role binding is a ledger fact, not a registry, before G9

- **Type:** decision
- **Status:** accepted
- **Owner:** `SIGNOFF-REPAIR.11.4.7.2.1.5.1`
- **Date:** 2026-09-23
- **Cites:** ROADMAP §8.1 (the identity hierarchy: `Host → NodeInstance → HarnessInstallation`
  and `AgentRole → AgentIncarnation → Session/Run`, with *`AgentIncarnation` states the actual
  provider/model/harness/configuration*), §10.6 (*leases prevent two node processes from waking
  the same role concurrently*), §10.7 (fan-out limits per node AND per role), the G9 row of §19
  (*signed artifacts/SBOM/provenance, runbooks, known limits, release authority*);
  `docs/decisions/2026-09-07_node-channel-wiring.md` (`NODE-BINDING-001`)

## Context

`.11.4.7.2.1.5`'s row 7 re-derivation found that no table binds a node to a role: the dev rule
`NODE-BINDING-001` makes a node id the role id it serves, and the code joins on it. The leaf
asked whether §8/§10 owe a binding registry before G9. Every claim below cites the command
that decides it, run at `9c0c256`.

## The census

- **Where the model binds them.** §8.1 places `NodeInstance` under `Host` and
  `AgentIncarnation` under `AgentRole`; the only link between the branches is that an
  incarnation *states the harness*, and a harness is an installation on a node. There is
  no direct node→role registry in the model. §10.6 and §10.7 name nodes and roles as
  distinct dimensions, so the model does not assume the collapse.
- **What the code does.** `grep -rn "role_id = np.node_id" crates/reasonbraid-server/src/api.rs`
  → **7** join sites (the presence and directory reads); the dispatch keys the inbox by
  `spec.agent_role` (**2** sites in `dispatch_work_in_tx`); node enrolment writes
  `incarnations.role_id = req.node_id` (`node_channel.rs`); the channel accepts a node id
  that parses as a role id as that role's principal. `grep -rn -i "dev rule\|node==role\|node id IS the" crates/reasonbraid-server/src`
  → **8** comment sites naming the rule.
- **What the incarnation ledger recorded.** `incarnations (incarnation_id, role_id,
  tenant_id, provider, model, harness, config, valid_from, valid_to)` — the harness as free
  text, and **no node**. So the one place §8.1 puts the binding did not record it.
- **What the book declares.** `deployment.md` (*One node, one role*) and
  `two-host-demo.md` (*a node id is the agent role wire id it serves*) both state the limit.
- **What G9 asks.** Artifacts, provenance, runbooks, **known limits**, release authority. A
  declared, documented limit satisfies the row; an undeclared collapse would not.

## Decision

1. **Not owed before G9 as a registry.** A registry that declares bindings — a role moving
   between machines without re-enrolment, one machine serving two roles — is a directory
   feature the roadmap places behind the dev rule's replacement, and G9's criterion for it is
   the declared limit, which the book carries.
2. **Owed now, and built with this record (`REASONBRAID-REPAIR-0425`): the fact.**
   `migrations/0092` adds `incarnations.node_id`; the one writer (node enrolment) records the
   node in hand; the backfill sets `node_id = role_id`, which is the rule's own truth for
   every existing row; `GET /v1/admin/incarnations` shows it. §8.1's binding now exists in the
   ledger rather than only in a rule, and a later directory derives from these rows.
3. **The trigger, readable:** `SELECT count(*) FROM incarnations WHERE node_id <> role_id;`
   is 0 under the rule and can only become non-zero once a directory replaces it. When it
   does, the seven joins and two dispatch keys move onto the fact — that lane opens then, not
   before.
4. ⛔ Not decided here: whether the dev rule is replaced before or after G9. That is the
   directory lane's, and this record gives it a column to land on.
