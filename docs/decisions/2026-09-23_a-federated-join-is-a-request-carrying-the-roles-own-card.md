---
answers:
  - What happens when a role in another tenant answers a call it was offered?
  - Why is a foreign join not a join, and why does the wire vocabulary not grow?
  - How does the role's card reach the call's tenant, and under which agreement?
  - Why can a foreign role not learn a call's existence by probing its id?
  - What does the call's tenant do with a join request, and what still cannot happen after the import?
---
# A federated join is a request carrying the role's own card

- **Type:** decision
- **Status:** accepted
- **Owner:** `SIGNOFF-REPAIR.5.3.5.2`
- **Date:** 2026-09-23
- **Cites:** `docs/adr/026-federation-trust-agreement.md` (*the remote domain never authorizes
  local effects; the explicit acceptance is the participant's and the operators'*); ROADMAP
  §10.5 (the eight response kinds), §9.4 (*cross-tenant existence is not leaked*);
  `docs/decisions/2026-09-23_the-calls-remote-form-was-deferred-behind-itself-and-its-first-half-is-not-federation.md`
  (DOC-0144); `REASONBRAID-REPAIR-0431` (the import's provenance), `REASONBRAID-REPAIR-0435`
  (the federated offer)

## Context

REPAIR-0435 offers a network-scope call to subscribers in tenants holding the effective
directory agreement. Such a subscriber could list the offer and do nothing with it: its `join`
was `403 only a principal enrolled in the call's tenant responds to it`, correctly under
ADR-026 — it has no local grant — and nothing recorded that it wanted in. DOC-0144 asked four
questions of this half; the answers below are what shipped.

## Decision

1. **The shape: a stored kind the wire never carries.** The role sends `{"kind":"join"}` —
   the §10.5 vocabulary is unchanged on the wire — and the server records
   `response_kind = 'join_request'` in `recruitment_responses`, one per respondent, with the
   origin tenant in the payload. The stored vocabulary exceeds the wire's by one kind, and that
   is the point: the close counts `join` and so never seats a request, and the inspection lists
   every response and so shows it, with no new route and no new column. A ninth wire kind would
   have let a foreign role *claim* to join; the server decides what a foreign join is.
2. **The card: the role's own export, minted by the server at request time.** A role cannot hand
   its card to another tenant's administrator through any route (the export is full-class
   only), and an import by reference would move the digest rung onto the server's own word. So
   the request carries exactly what `GET /v1/profiles/{role_id}/card` would mint for the role —
   the same code path (`mint_card`) — with its digest, and the ordinary import re-derives it.
   The request IS the role's export, attached for the call's tenant.
3. **Three gates, and two of them speak the old words.** Only a `join` is a request (a foreign
   decline, observe or recommend has no local meaning and records nothing); the call must have
   been OFFERED to the role, which happens only for a network-scope call under the directory
   agreement; and the two tenants must hold the EFFECTIVE recruitment agreement, because a card
   crosses only under the operators' consent on both sides. The first two refuse with the
   sentence an un-offered foreign principal has always heard, so a call's existence cannot be
   learned by probing its id (§9.4); the third names the missing agreement, because an offered
   role under the directory agreement already knows the call exists.
4. **Eligibility is judged as for a local join.** The expression is re-resolved against the
   foreign role's own facts before the request is recorded — a role that hides its capability
   from the network is offered (by interests) and then found ineligible (by claims), exactly as
   a local role would be.
5. **Resolution is the ordinary import; what follows waits.** The initiator or administrator
   copies the request's card and digest into `POST /v1/profiles/cards/import`; the provenance
   names the origin role (REPAIR-0431). The imported role has no node in the importing tenant,
   so it cannot join the call — that is `SIGNOFF-REPAIR.5.3.5.3`, waiting on the director. The
   request therefore stays a request until then, and the book says so.

## Consequences

- No wire vocabulary change; no new route; one new stored kind, documented in the recruitment
  chapter beside the eight.
- ⛔ Not decided here: whether the import should name the call it was requested for
  (`card_imports.requested_for_call`), so an eventual join by the imported role is attributable
  to the request. It matters only once `.5.3.5.3` lets the imported role act; recorded there.
