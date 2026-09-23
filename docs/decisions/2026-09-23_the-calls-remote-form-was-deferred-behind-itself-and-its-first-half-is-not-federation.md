---
answers:
  - What did PHASE-8.1.2 defer when it shipped the recruitment scope column, and behind what trigger?
  - How does an open recruitment call reach a role today, in its own tenant or another?
  - Why can an imported role not join a call, even after the import?
  - What is the smallest honest slice of the call's remote form, and what waits on the director?
  - Does the directory match surface classify a foreign candidate by its own tenant?
---
# The call's remote form was deferred behind itself, and its first half is not federation

- **Type:** decision
- **Status:** accepted
- **Owner:** `SIGNOFF-REPAIR.5.3.5`
- **Date:** 2026-09-23
- **Cites:** `docs/adr/026-federation-trust-agreement.md` (*the remote recruitment is the
  agreement-scoped opt-in*); ROADMAP §10.5 (*a call specifies eligibility, audience, …
  advertisement window*), ROADMAP:296 (*advertise calls to eligible online agents and retain
  durable inbox entries for eligible offline agents*), ROADMAP:743 (`CallForParticipation`:
  *audience, eligibility query, advertisement window*), ROADMAP:549 (*federation gateways* as a
  later boundary), ROADMAP:596 (federation between deployments post-v1); `docs/tasks/PHASE-8.md`
  (`PHASE-8.1.2`'s Done note); `docs/decisions/2026-09-23_the-federation-goal-line-two-items-met-two-live-defects-and-the-calls-remote-form-unbuilt.md`
  (DOC-0143); `docs/decisions/2026-09-23_the-node-to-role-binding-is-a-ledger-fact-not-a-registry-before-g9.md`
  (DOC-0139)

## Context

DOC-0143 opened `SIGNOFF-REPAIR.5.3.5` as a feature lane with a design census first: the
recruitment agreement's only consumer is the card import, and the call machinery has no
cross-tenant path. This record is that census. It was to name who imports, what an offer
carries across a boundary, what the origin's operator sees, and how a foreign role's join
becomes an import request. Measuring the pieces changed the question.

## The census

**1. The deferral's trigger was the feature itself.** `PHASE-8.1.2`'s Done note: *the
recruitment scope column ships as the vocabulary — the cross-tenant CALL-panel widening is the
named deferral (the trigger: the call machinery's remote-panel surface)*. The remote-panel
surface IS the cross-tenant widening; a deferral behind its own feature is a deferral nothing
can fire — the shape `SIGNOFF-REPAIR.11.4.7.2.1` names, one lane over.

**2. An open call reaches no role, in any tenant.** `open_recruitment_call` writes one
`recruitment_offers` row per subscriber whose interests match (`recruitment.rs::offer_to_subscribers`,
tenant-bound) and nothing carries it further: `grep -rn "recruitment_offers" crates/reasonbraid-server/src/node_channel.rs crates/reasonbraid-node/src --include=*.rs`
→ 0, `grep -rn "Advertisement\|\"advertisement\"" crates/reasonbraid-server/src crates/reasonbraid-core/src crates/reasonbraid-node/src --include=*.rs`
→ 0. The only readers of a call are its initiator and the tenant's administrator
(`GET /v1/calls/{call_id}`); a role learns of a call out of band and responds by id. §10.5's
*advertisement window* and ROADMAP:296's *advertise calls to eligible online agents and retain
durable inbox entries for eligible offline agents* are the offer row's comment, not a delivery.
⛔ So the remote form's first half — an offer reaching a federated subscriber — has no
intra-tenant half to extend. It is not federation work; it is the advertisement §10.5 named.

**3. A foreign response is refused, and rightly.** `respond_to_call_core` derives the
respondent's tenant from the call and refuses another tenant's principal
(`REASONBRAID-REPAIR-0185`). Under ADR-026 a foreign role cannot act locally without a local
grant, and the only path to one is the card import — which the CALL'S tenant administrator
performs under the recruitment agreement.

**4. An imported role cannot join a call either.** The import lands a directory identity with a
grant, a quota and a profile — and no node: `respondent_candidate` reads `node_presence` for the
respondent, and an imported role has no presence row, no incarnation and no machine in the
importing tenant. `incarnations.node_id` (DOC-0139) binds a role to the node that ran it, and
the dev rule collapses node and role into one id. So the remote form's LAST half — the foreign
role's work delivered to a node — is a question the kill line does not answer by itself: the
remote domain vouches for its records and the local grant acts locally, but WHOSE machine runs
the imported identity's turns? Either the origin tenant's node runs a local identity's work
(the foreign operator executes what the importing tenant authorized), or a federated
participant runs on a node the importing tenant enrols (the identity crosses, the machine does
not). ROADMAP:549 puts *federation gateways* among later boundaries and ROADMAP:596 puts
federation between deployments post-v1; neither says which of the two shapes holds inside one
deployment. 💡 **That is the director's, and it is surfaced as such.**

**5. The directory match surface classifies every candidate by the READER, not by the
candidate's tenant.** `directory_match` (`crates/reasonbraid-server/src/api.rs`) walks
`node_presence` across all tenants, computes one `reader_class` for the caller — a same-tenant
member gets `Tenant` — and filters every returned profile with it
(`filter_profile(p, reader_class)`), so a foreign candidate's tenant-view fields reach a member
of another tenant. `directory_presence` classifies per tenant (`if tenant == reader_tenant`),
which is the right shape. This is `SIGNOFF-REPAIR.5.1`'s goal line verbatim — *apply
visibility per candidate tenant* — recorded there with this evidence; the offer's federated
half must classify the way `directory_presence` does, not the way the match does.

## Decision — three children, in this order

1. **`SIGNOFF-REPAIR.5.3.5.1` — an open call reaches its eligible subscribers.** The
   intra-tenant half first: an offer is delivered to the subscriber's node as durable inbox
   work (the §10.5 advertisement; online nodes promptly, offline nodes on reconnect after their
   cursor, expiring with the call's window), the subscriber can read the calls offered to it,
   and the storm caps already bound the fan-out. Then the federated half: a `network`-scope
   call's offers reach subscribers in tenants holding the EFFECTIVE directory agreement, carrying
   no more than the network view already discloses — the call id, the expression, the window —
   classified per candidate tenant.
2. **`SIGNOFF-REPAIR.5.3.5.2` — a foreign response is a recorded join request.** A federated
   subscriber's `join` is not a join: it is recorded on the call as a request naming the origin
   role, carrying the card it exported, visible to the call's initiator and administrator, and
   resolved by the existing import (`REASONBRAID-REPAIR-0431`'s provenance names the call it
   was requested for). No local effect until the import; the receipt trail on both sides.
3. **`SIGNOFF-REPAIR.5.3.5.3` — the imported identity's turns run somewhere.** Waits on the
   director's answer to item 4, and on the node-binding lane DOC-0139 opened. Recorded, not
   built.

⛔ Not decided here: whether an offer to a federated subscriber discloses the thread's subject.
The network view of a call is its expression and its window; the thread stays hidden until a
join lands. The leaf that builds the federated half decides with the RED.
