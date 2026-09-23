---
answers:
  - Whose machine runs an imported partner agent's work — the partner's, or one the importing tenant enrols?
  - Why both, and how does each keep ADR-026's kill line?
  - What does the local binding need today, and what proves it?
  - What does the origin binding need before it can be built, and in what order?
---
# An imported identity runs on a bound machine: local now, origin designed

- **Type:** decision
- **Status:** accepted — decided under the director's delegation of 2026-09-23 (*why not both … your decision, state of the art and signoff-grade, in the project's best interest*)
- **Owner:** `SIGNOFF-REPAIR.5.3.5.3`
- **Date:** 2026-09-23
- **Cites:** `docs/adr/026-federation-trust-agreement.md` (*the remote domain vouches for its own records, the local domain acts on its own grants*; *the portable cards verify through the ladder before they confer anything*); ROADMAP §20.10 (*remote recruitment, portable agent cards/profiles*), ROADMAP:549 (*federation gateways* as a later boundary), ROADMAP:596 (federation between deployments post-v1);
  `docs/decisions/2026-09-23_the-calls-remote-form-was-deferred-behind-itself-and-its-first-half-is-not-federation.md` (DOC-0144, item 4);
  `docs/decisions/2026-09-23_the-node-to-role-binding-is-a-ledger-fact-not-a-registry-before-g9.md` (DOC-0139);
  `docs/decisions/2026-09-23_a-federated-join-is-a-request-carrying-the-roles-own-card.md` (DOC-0145)

## Context

DOC-0144 measured that an import lands an identity, a grant, a quota and a profile — and no node,
so an imported role could not take part in the call it asked to join. It framed two shapes:
the origin tenant's node executes what the importing tenant authorized, or the importing tenant
enrols a node for the identity. The director asked *why not both*, and delegated the decision.

## Decision

**Both, as two explicit execution bindings of an imported identity — and they answer two
different roadmap items, which is why neither alone is right.**

1. **`local` — the identity crosses, the machine does not.** The importing tenant enrols a node
   for the imported identity, exactly as it enrols any role's machine (`POST /v1/nodes/enroll-tokens`
   + `/v1/nodes/enroll` with the imported role's id, the dev rule's node == role), attests the
   claims it will rely on with its OWN administrator's word (the card's attestations were the
   origin owner's), and the identity acts under the local grant on the local machine. This is
   what §20.10's *portable agent cards/profiles* means — a card is portable because a compatible
   runtime elsewhere can run under it — and it is the whole of ADR-026's model with nothing
   crossing but a description. It needs no new code: the control
   `an_imported_identity_acts_once_the_importing_tenant_binds_a_node_to_it` proves it end to end,
   with the negative arm that gives the binding its meaning — before the node exists the same
   join is refused for want of one.
2. **`origin` — the machine stays, the work crosses.** The imported identity's turns are
   delivered to the origin role's node, which executes them under the origin operator's runtime
   and adapters, bounded by the importing tenant's local grant to the imported identity (the
   kill line, unchanged: the origin never authorizes a local effect, it only executes what the
   local grant admitted). This is what §20.10's *remote recruitment* means and what A2A-style
   agent federation converged on: the remote agent is invoked where it lives; the local system
   holds an identity and a mandate for it, never a copy of the agent. Consent is already on
   the ledger twice — the recruitment agreement (the operators, both sides) and the join request
   (the participant, DOC-0145) — and ROADMAP:596 keeps this inside one deployment for now:
   the origin node is enrolled in the same server, so delivery is a routing fact, not a gateway.

## What `origin` needs, in order

- **`SIGNOFF-REPAIR.5.3.5.3.1` — the binding is a ledger fact.** `card_imports.executes_on`
  (NULL = local, or the origin role's node id), settable only when the origin role's node is
  enrolled in this deployment and the recruitment agreement stands, recorded as an
  administrative effect; every reader that resolves a role to its node
  (`respondent_candidate`, the dispatch's target, the presence reads) resolves through it — the
  dev rule's `node == role` becomes `COALESCE(executes_on, role_id)`, which is also the first
  consumer DOC-0139's registry lane was waiting for (`node_id <> role_id`).
- **`SIGNOFF-REPAIR.5.3.5.3.2` — a node evaluates each command against ITS tenant's epoch.**
  The node journal stores one `revocation_epoch` per channel (`channel_state`), the handshake
  hands one, and the cached-decision gate compares every command against it — a node executing
  for two tenants would judge A's admission by B's epoch. The command already carries its own
  tenant and the epoch it was decided under; the handshake must hand the current epoch of every
  tenant whose work the node holds, and the gate must compare per tenant. This is the node
  crate's change and the reason `origin` is not built in this commit.
- **`SIGNOFF-REPAIR.5.3.5.3.3` — receipts on both sides.** Each delivery across the binding is a
  cross-domain act: the importing side's receipt names the origin node's acknowledgement, the
  origin side's names the admission it executed under. The receipt table is per-acceptance since
  REPAIR-0433, so nothing structural stands in the way.

## Consequences

- Nothing waits on the director any more in this lane: `local` is available and proven,
  `origin` is designed with its three children in order, and the first of them is buildable now.
- ⛔ Not decided here: whether an origin-bound identity may ALSO be bound locally at the same time
  (two machines, one identity). The binding column is single-valued until a reason for two exists.
